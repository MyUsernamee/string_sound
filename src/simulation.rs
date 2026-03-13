
use std::iter::zip;

use macroquad::{color::WHITE, input::{is_key_pressed, is_mouse_button_down, mouse_position}, math::Vec3, miniquad::window::quit, shapes::draw_line, time::draw_fps, window::{clear_background, next_frame, screen_height, screen_width}, *};

use crate::consts::*;

pub struct SimulationState {
    node_positions: Vec<Vec3>,
    fixed: Vec<bool>,
    k: f64,
    damping: f64,
    dt: f64,
}

pub fn init_simulation(count: usize, k: f64, damping: f64, dt: f64) -> SimulationState {
    let node_positions = vec![Vec3::ZERO; count];
    let fixed = vec![false; count];
    return SimulationState { node_positions, fixed, k, damping, dt};
}

pub fn get_simulation_output(simulation_state: &mut SimulationState) -> f32 {
    return 0.0 // TODO:
}

pub fn step_simulation(simulation_state: &mut SimulationState) {
    todo!()
}

pub fn draw_simulation(simulation_state: &SimulationState) {
    todo!()
}

pub fn interact_simulation(simulation_state: &mut SimulationState) {
    todo!()
}

