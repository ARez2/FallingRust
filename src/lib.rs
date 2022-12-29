
pub mod cell;
pub use cell::Cell;

pub mod cellhandler;
pub use cellhandler::cell_handler;

pub mod material;
pub use material::{Material, MaterialType};

pub mod gui;
pub use gui::Framework;

pub mod texturehandler;
pub use texturehandler::TextureHandler;
pub mod matrix;
pub use matrix::Matrix;

pub mod chunk;
pub use chunk::Chunk;



pub const CHUNK_SIZE: usize = 16;
const NUM_CHUNKS: u32 = 40;//8 - 160, 16 - 80
pub const WIDTH: u32 = CHUNK_SIZE as u32 * NUM_CHUNKS;
pub const HEIGHT: u32 = CHUNK_SIZE as u32 * NUM_CHUNKS;
pub const SCALE: f64 = 2.0;


/// Returns 1 or -1 at random
pub fn rand_multiplier() -> i32 {
    match rand::random::<bool>() {
        true => 1,
        false => -1,
    }
}