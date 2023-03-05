pub mod cell;
pub use cell::Cell;

pub mod cellhandler;
pub use cellhandler::cell_handler;

pub mod material;
pub use material::{Material, MaterialType};

pub mod reaction;


mod assets;
use assets::Assets;
pub static mut ASSETS: Lazy<Assets> = Lazy::new(|| Assets::new());

pub mod gui;
pub use gui::Framework;

pub mod brush;
pub mod matrix;
pub use matrix::Matrix;

pub mod chunk;
pub use chunk::Chunk;

use once_cell::sync::Lazy;
pub use pixels::wgpu::Color;

pub mod renderer;
use rand::RngCore;
pub use renderer::NoiseRenderer;

pub const CHUNK_SIZE: usize = 32;
const NUM_CHUNKS: u32 = 16;//8 - 160, 16 - 80
pub const WIDTH: u32 = CHUNK_SIZE as u32 * NUM_CHUNKS;//
pub const HEIGHT: u32 = CHUNK_SIZE as u32 * NUM_CHUNKS;
pub const SCALE: f64 = 2.0;

pub const COLOR_EMPTY: Color = Color { r: 1.0, g: 0.0, b: 0.8, a: 1.0 };

pub type Rng = rand::rngs::StdRng;
const SEED: u64 = 1234;
pub static mut RNG: Lazy<Rng> = Lazy::new(|| rand::SeedableRng::seed_from_u64(rand::thread_rng().next_u64()));
pub fn gen_range(min: f32, max: f32) -> f32 {
    let random = unsafe {
        (*RNG).next_u32()
    } as f32;
    return min + (random / std::u32::MAX as f32) * (max - min);
}


//pub type RngThr<'a> = std::sync::Arc<std::sync::Mutex<&'a mut rand::rngs::ThreadRng>>;

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
impl Default for UIInfo {
    fn default() -> Self {
        Self::new()
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
    color.r = color.r.clamp(0.0, 1.0);
    color.g = color.g.clamp(0.0, 1.0);
    color.b = color.b.clamp(0.0, 1.0);
    color
}