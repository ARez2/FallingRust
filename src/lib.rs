
pub mod cell;
pub use cell::Cell;

pub mod cellhandler;
pub use cellhandler::cell_handler;

pub mod material;
pub use material::{Material, MaterialType};

pub mod matrix;
pub use matrix::Matrix;

pub mod chunk;
pub use chunk::Chunk;


/// Returns 1 or -1 at random
pub fn rand_multiplier() -> i32 {
    match rand::random::<bool>() {
        true => 1,
        false => -1,
    }
}