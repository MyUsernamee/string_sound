use std::{cell::RefCell, collections::VecDeque, f32, iter::zip, process::exit, str::FromStr, sync::{Arc, Mutex}, vec};

use cpal::{Device, DeviceId, Host, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use macroquad::{color::WHITE, input::{is_key_pressed, is_mouse_button_down, mouse_position}, miniquad::window::quit, shapes::draw_line, time::draw_fps, window::{clear_background, next_frame, screen_height, screen_width}, *};
use macroquad::color::BLACK;

const VERTICAL_SCALE: f32 = 1.0;
const VISUAL_THICKNESS: f32 = 1.0;
const SEGMENT_COUNT: usize = 1024;
const STRING_STIFFNESS: f64 = 343.0 * 343.0 * 1.0;
const STRING_DAMPING: f64 = 1.0 - 1e-3;
const STRING_LENGTH: f64 = 10.0;
const SUBSTEPS: usize = 1;
const VOLUME: f64 = 4e-3;
const INPUT_VOLUME: f64 = 4e-3;
const LOWPASS_RATIO: f32 = 0.0;
const MICROPHONE_ID: &str = "pipewire:alsa_input.usb-Razer_Inc_Razer_Seiren_Mini_UC2130L03305085-00.mono-fallback";

struct AudioData(Host, Device, Device, StreamConfig, StreamConfig);

fn init_audio() -> AudioData {
    let host_id = cpal::available_hosts()[0];
    let host = cpal::host_from_id(host_id).unwrap();

    println!("{:?}", Vec::from_iter(host.input_devices().unwrap().map(|a| a.id().unwrap())));
    let output_device = host.default_output_device().expect("Error opening device.");
    let input_device = host.default_input_device().expect("NO MIC");

    let mut supported_configs_range = input_device.supported_input_configs().expect("Unable to get output configs.");
    let supported_config = supported_configs_range.next().expect("Unable to get audio config for input?").with_sample_rate(44100);
    let input_config: StreamConfig = supported_config.into();

    let mut supported_configs_range = output_device.supported_output_configs().expect("Unable to get output configs.");
    let supported_config = supported_configs_range.next().expect("Unable to get audio config?").with_sample_rate(44100);
    let output_config: StreamConfig = supported_config.into();

    return AudioData(host, output_device, input_device, output_config, input_config);
}

fn create_input_stream<F>(audio_data: &AudioData, set_sample: F) -> cpal::Stream where F: Fn(f32) + Send + 'static {
    let device = &audio_data.2;
    let config = audio_data.4;
    let stream = device.build_input_stream(
        config,
        move |data, _info| {
            for ( index, sample ) in data.iter().enumerate() {
                set_sample(*sample); 
            }
        },
        move |err| {
            eprintln!("Error processing sample: {}", err);
        },
        None // None=blocking, Some(Duration)=timeout
    ).unwrap();
    return stream;
}

fn create_output_stream<F>(audio_data: &AudioData, get_sample: F) -> cpal::Stream where F: Fn() -> f32 + Send + 'static {
    let device = &audio_data.1;
    let config = audio_data.3;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                *sample = get_sample();
            }
        },
        move |err| {
            eprintln!("Error processing sample: {}", err);
        },
        None // None=blocking, Some(Duration)=timeout
    ).unwrap();
    return stream;
}

#[macroquad::main("String Synth")]
async fn main() {

    let audio_data = init_audio();

    let buffer_size = 1024usize;

    println!("{:?}", audio_data.4.buffer_size);

    let dt = 1.0 / audio_data.3.sample_rate as f64 / SUBSTEPS as f64;
    let simulation_state = Arc::new(Mutex::new(init_simulation(SEGMENT_COUNT, STRING_LENGTH, buffer_size, STRING_STIFFNESS, STRING_DAMPING, dt)));
    let output_simulation_state = Arc::clone(&simulation_state);
    let input_simulation_state = Arc::clone(&simulation_state);

    let string_output = move || {
        let mut simulation_state_lock = output_simulation_state.lock().expect("Failed to lock simulation state.");
        set_simulation_input(&mut simulation_state_lock);
        for i in 0..SUBSTEPS {
            step_simulation(&mut simulation_state_lock);
        }
        return get_simulation_output(&mut simulation_state_lock);
    };

    let string_input = move |sample| {
        let mut simulation_state_lock = input_simulation_state.lock().expect("Failed to lock simulation state.");
        simulation_state_lock.input_buffer.push_back(sample);
    };

    let output_stream = create_output_stream(&audio_data, string_output);
    let input_stream = create_input_stream(&audio_data, string_input);
    output_stream.play().unwrap();
    input_stream.play().unwrap();

    loop {
        clear_background(BLACK);
        draw_fps();

        if is_key_pressed(input::KeyCode::Escape) {
            quit();
        }

        interact_simulation(&mut simulation_state.lock().expect("Failed to lock simulation state."), VERTICAL_SCALE);
        draw_simulation(&simulation_state.lock().expect("Failed to lock simulation state."), VERTICAL_SCALE, VISUAL_THICKNESS);

        next_frame().await
    }
}

fn set_simulation_input(simulation_state_lock: &mut SimulationState) {
    simulation_state_lock.string_heights[8] += simulation_state_lock.input_buffer.pop_front().unwrap_or(0.0) as f64 * INPUT_VOLUME;
}

struct SimulationState {
    string_heights: Vec<f64>,
    previous_string_heights: Vec<f64>,
    fixed: Vec<bool>,
    input_buffer: VecDeque<f32>,
    string_length: f64,
    last_sample: f32,
    k: f64,
    damping: f64,
    dt: f64,
}

fn init_simulation(count: usize, string_length: f64, input_buffer_size: usize, k: f64, damping: f64, dt: f64) -> SimulationState {
    let string_heights = vec![0.0f64; count];
    let previous_string_heights = vec![0.0f64; count];
    let fixed = vec![false; count];
    return SimulationState { string_heights, previous_string_heights, fixed, input_buffer: VecDeque::new(), string_length, k, damping, dt, last_sample: 0.0f32}
}

/// Calculates the force needed to force the end point to follow the dirichlet boundary
/// conditions. As such, it is proportional to the sound / energy transmitted to the body
/// instrument, and is responsible for the sound.
fn calculate_endpoint_force(simulation_state: &SimulationState) -> f64 {
    let i = simulation_state.string_heights.len() / 2;
    (simulation_state.string_heights[i] - simulation_state.previous_string_heights[i]) / simulation_state.dt * VOLUME
}

fn get_simulation_output(simulation_state: &mut SimulationState) -> f32 {

    let sample = calculate_endpoint_force(simulation_state) as f32;
    let new_sample = simulation_state.last_sample * LOWPASS_RATIO + sample * (1.0 - LOWPASS_RATIO);
    simulation_state.last_sample = new_sample;

    return new_sample;
    
}

fn get_dx(simulation_state: &SimulationState) -> f64 {
    return simulation_state.string_length / simulation_state.string_heights.len() as f64
}

fn step_simulation(simulation_state: &mut SimulationState) {
    let velocities = Vec::from_iter(zip(simulation_state.string_heights.iter(), simulation_state.previous_string_heights.iter()).map(|(a, b)| a - b));
    let mut new_heights = simulation_state.string_heights.clone();
    let count = new_heights.len();
    let dx = get_dx(simulation_state);

    let divergence = Vec::from_iter(simulation_state.string_heights.windows(3).map(|a| {
        (a[2] - 2.0 * a[1] + a[0]) / (dx*dx)
    }));

    for ((height, fixed), (velocity, div)) in zip(zip(new_heights[1..count-1].iter_mut(), simulation_state.fixed[1..count-1].iter()), zip(velocities[1..count-1].iter(), divergence)) {
        let force = simulation_state.k * div;
        if *fixed { continue; }
        *height += velocity * simulation_state.damping + force * simulation_state.dt.powf(2.0) * 0.5;
    }

    simulation_state.previous_string_heights.clone_from(&simulation_state.string_heights);
    simulation_state.string_heights.clone_from(&new_heights);
}

fn draw_simulation(simulation_state: &SimulationState, vertical_scale: f32, thickness: f32) {

    let string_height_count = simulation_state.string_heights.len() as f32;
    let dx = (screen_width() * 0.75) / string_height_count;
    for (index, height) in simulation_state.string_heights.windows(2).enumerate() {
        let mut x1 = dx * (index as f32 - string_height_count / 2.0);
        let mut x2 = dx * ((index + 1) as f32 - string_height_count / 2.0);
        let mut y1 = height[0] as f32 * vertical_scale * screen_height();
        let mut y2 = height[1] as f32 * vertical_scale * screen_height();

        x1 += screen_width() / 2.0;
        x2 += screen_width() / 2.0;
        y1 += screen_height() / 2.0;
        y2 += screen_height() / 2.0;

        draw_line(x1, y1, x2, y2, thickness, WHITE);
    }

}

fn interact_simulation(simulation_state: &mut SimulationState, vertical_scale: f32) {
    let mut mouse_index = (((mouse_position().0 / screen_width() - 0.5) / 0.75 + 0.5) * simulation_state.string_heights.len() as f32) as usize;
    mouse_index = mouse_index.clamp(1, simulation_state.string_heights.len() - 2);
    let mouse_height = (mouse_position().1 / screen_height() - 0.5) / vertical_scale;
    for a in simulation_state.fixed.iter_mut() {
        *a = false;
    }
    if is_mouse_button_down(input::MouseButton::Left) { 
        // simulation_state.string_heights[mouse_index] = mouse_height as f64; 
        // simulation_state.previous_string_heights[mouse_index] = mouse_height as f64; 
        simulation_state.fixed[mouse_index] = true;
        // let mut new_heights = Vec::new();
        // new_heights.resize(simulation_state.string_heights.len(), 0.0);
        // new_heights[SEGMENT_COUNT / 2] = 1.0;
        // simulation_state.string_heights.clone_from(&new_heights);
        // simulation_state.previous_string_heights.clone_from(&new_heights);
    }
}

