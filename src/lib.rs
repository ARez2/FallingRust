pub mod cell;
pub use cell::Cell;

pub mod cellhandler;
pub use cellhandler::cell_handler;

pub mod material;
pub use material::{Material, MaterialType};

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


/// Returns 1 or -1 at random
pub fn rand_multiplier() -> i32 {
    match rand::random::<bool>() {
        true => 1,
        false => -1,
    }
}