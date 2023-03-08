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
pub use renderer::NoiseRenderer;

pub const CHUNK_SIZE: usize = 16;
const NUM_CHUNKS: u32 = 32;//8 - 160, 16 - 80
pub const WIDTH: u32 = CHUNK_SIZE as u32 * NUM_CHUNKS;//
pub const HEIGHT: u32 = CHUNK_SIZE as u32 * NUM_CHUNKS;
pub const SCALE: f64 = 2.0;

pub const COLOR_EMPTY: Color = Color { r: 1.0, g: 0.0, b: 0.8, a: 1.0 };

pub type Rng = fastrand::Rng;
const SEED: u64 = 1234;
pub static mut RNG: Lazy<Rng> = Lazy::new(|| fastrand::Rng::with_seed(fastrand::u64(0..100)));
pub fn gen_range(min: f32, max: f32) -> f32 {
    return min + unsafe{&mut RNG}.f32() * max;
}

const EMPTY: Cell = Cell::new(glam::IVec2::new(0, 0), Material::Empty);
const WALL: Cell = Cell::new(glam::IVec2::new(0, 0), Material::Empty);

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


const MULTIPLIER_OPTIONS: [i32; 2] = [-1, 1];
/// Returns 1 or -1 at random
pub fn rand_multiplier() -> i32 {
    MULTIPLIER_OPTIONS[unsafe{&mut *RNG}.usize(0..MULTIPLIER_OPTIONS.len())]
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