pub mod cell;
pub use cell::Cell;

pub mod cellhandler;
pub use cellhandler::cell_handler;

pub mod material;
pub use material::{Material, MaterialType};

pub mod reaction;


pub mod assets;
pub use assets::Assets;

pub mod gui;
pub use gui::Framework;

pub mod brush;
pub mod matrix;
pub use matrix::Matrix;

pub mod chunk;
pub use chunk::Chunk;

pub use pixels::wgpu::Color;


pub const CHUNK_SIZE: usize = 32;
const NUM_CHUNKS: u32 = 16;//8 - 160, 16 - 80
pub const WIDTH: u32 = CHUNK_SIZE as u32 * NUM_CHUNKS;
pub const HEIGHT: u32 = CHUNK_SIZE as u32 * NUM_CHUNKS;
pub const SCALE: f64 = 2.0;

pub const COLOR_EMPTY: Color = Color { r: 1.0, g: 0.0, b: 0.8, a: 1.0 };


pub struct UIInfo {
    pub num_frames: f32,
}
impl UIInfo {
    pub fn new() -> Self {
        UIInfo {
            num_frames: 30.0,
        }
    }
}


/// Returns 1 or -1 at random
pub fn rand_multiplier() -> i32 {
    match rand::random::<bool>() {
        true => 1,
        false => -1,
    }
}

pub fn darken_color(mut color: Color, amount: f64) -> Color {
    color.r *= amount;
    color.g *= amount;
    color.b *= amount;
    color.r = color.r.max(0.0).min(1.0);
    color.g = color.g.max(0.0).min(1.0);
    color.b = color.b.max(0.0).min(1.0);
    color
}