use std::{cell::RefCell, collections::VecDeque, f32, iter::zip, process::exit, str::FromStr, sync::{Arc, Mutex}, vec};

use cpal::traits::StreamTrait;
use macroquad::{color::WHITE, input::{is_key_pressed, is_mouse_button_down, mouse_position}, miniquad::window::quit, shapes::draw_line, time::draw_fps, window::{clear_background, next_frame, screen_height, screen_width}, *};
use macroquad::color::BLACK;

use crate::audio::*;
use crate::simulation::*;
use crate::consts::*;

pub mod audio;
pub mod simulation;
pub mod consts;

#[macroquad::main("String Synth")]
async fn main() {

    let audio_data = init_audio();

    let dt = 1.0 / audio_data.2.sample_rate as f64 / SUBSTEPS as f64;
    let simulation_state = Arc::new(Mutex::new(init_simulation(SEGMENT_COUNT, DRUM_STIFFNESS, DRUM_DAMPING, dt)));
    let output_simulation_state = Arc::clone(&simulation_state);

    let string_output = move || {
        let mut simulation_state_lock = output_simulation_state.lock().expect("Failed to lock simulation state.");
        for i in 0..SUBSTEPS {
            step_simulation(&mut simulation_state_lock);
        }
        return get_simulation_output(&mut simulation_state_lock);
    };

    let output_stream = create_output_stream(&audio_data, string_output);
    output_stream.play().unwrap();

    loop {
        clear_background(BLACK);
        draw_fps();

        if is_key_pressed(input::KeyCode::Escape) {
            quit();
        }

        interact_simulation(&mut simulation_state.lock().expect("Failed to lock simulation state."));
        draw_simulation(&simulation_state.lock().expect("Failed to lock simulation state."));

        next_frame().await
    }
}

