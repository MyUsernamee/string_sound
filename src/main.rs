use macroquad::{color::BLACK, time::draw_fps, window::{clear_background, next_frame}, *};

#[macroquad::main("String Synth")]
async fn main() {
    loop {
        clear_background(BLACK);

        draw_fps();

        next_frame().await
    }
}
