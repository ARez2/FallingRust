use glam::IVec2;
use pixels::wgpu::Color;
use rand::Rng;

use crate::{Cell, cell::{Material}, Chunk, chunk};

const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;


/// Generate a pseudorandom seed for the game's PRNG.
fn generate_seed() -> (u64, u64) {
    use byteorder::{ByteOrder, NativeEndian};
    use getrandom::getrandom;

    let mut seed = [0_u8; 16];

    getrandom(&mut seed).expect("failed to getrandom");

    (
        NativeEndian::read_u64(&seed[0..8]),
        NativeEndian::read_u64(&seed[8..16]),
    )
}


#[derive(Clone)]
pub struct Matrix {
    pub width: usize,
    pub height: usize,
    
    pub cells: Vec<Cell>,
    pub chunks: Vec<Vec<Chunk>>,

    pub debug_draw: bool,
}

impl Matrix {
    pub fn new_empty(width: usize, height: usize) -> Self {
        assert!(width != 0 && height != 0);
        let size = width.checked_mul(height).expect("too big");

        let mut cells = Vec::<Cell>::new();
        for y1 in 0..height {
            for x1 in 0..width {
                cells.push(Cell::new(IVec2::new(x1 as i32, y1 as i32)));
            };
        };
        
        let mut chunks = vec![];
        for y in (0..height as i32).step_by(CHUNK_SIZE) {
            let mut row = Vec::<Chunk>::new();
            for x in (0..width as i32).step_by(CHUNK_SIZE) {
                let chunk = Chunk {
                    should_step: false,
                    should_step_next_frame: true,
                    topleft: IVec2::new(x, y),
                    size: CHUNK_SIZE,
                };
                row.push(chunk);
            };
            chunks.push(row);
        }


        Self {
            width,
            height,

            cells,
            chunks,
            debug_draw: true,
        }
    }


    fn chunk_in_bounds(&self, chunk_pos: IVec2) -> bool {
        (chunk_pos.x >= 0 && chunk_pos.x < self.chunks.len() as i32) && (chunk_pos.y >= 0 && chunk_pos.y < self.chunks[0].len() as i32)
    }

    pub fn get_chunk_for_pos_not_mut(&self, pos: IVec2) -> Option<&Chunk> {
        let chunk_pos = pos / IVec2::new(CHUNK_SIZE_I32, CHUNK_SIZE_I32);
        if self.chunk_in_bounds(chunk_pos) {
            Some(&self.chunks[chunk_pos.y as usize][chunk_pos.x as usize])
        } else {
            None
        }
    }
    pub fn get_chunk_for_pos(&mut self, pos: IVec2) -> Option<&mut Chunk> {
        let chunk_pos = pos / IVec2::new(CHUNK_SIZE_I32, CHUNK_SIZE_I32);
        if self.chunk_in_bounds(chunk_pos) {
            Some(&mut self.chunks[chunk_pos.y as usize][chunk_pos.x as usize])
        } else {
            None
        }
    }
    pub fn set_chunk_active(&mut self, pos: IVec2) {
        let chunk_res = self.get_chunk_for_pos(pos);
        if let Some(chunk) = chunk_res {
            chunk.should_step_next_frame = true;
        }
    }
    pub fn set_chunk_cluster_active(&mut self, pos: IVec2) {
        for y in (pos.y-CHUNK_SIZE_I32..=pos.y+CHUNK_SIZE_I32).step_by(CHUNK_SIZE) {
            for x in (pos.x-CHUNK_SIZE_I32..=pos.x+CHUNK_SIZE_I32).step_by(CHUNK_SIZE) {
                self.set_chunk_active(IVec2::new(x, y));
            }
        }
    }




    pub fn get_cell(&self, pos: IVec2) -> Option<Cell> {
        if (pos.x < 0 || pos.x >= self.width as i32) || (pos.y < 0 || pos.y >= self.height as i32) {
            None
        } else {
            Some(self.cells[self.grid_idx(pos.x, pos.y).unwrap()])
        }

    }

    pub fn set_cell_material(&mut self, pos: IVec2, material: Material) {
        let cell = Cell::new_material(pos, material);
        self.set_cell(pos, cell);
    }

    pub fn set_cell(&mut self, pos: IVec2, mut cell: Cell) {
        if (pos.x < 0 || pos.x > self.width as i32) || (pos.y < 0 || pos.y > self.height as i32) {
            return;
        };
        let old_pos = cell.pos;
        cell.pos = pos;
        let idx = self.grid_idx(pos.x, pos.y).unwrap();
        let old_idx = self.grid_idx(old_pos.x, old_pos.y).unwrap();
        self.cells[idx] = cell;
        self.cells[old_idx] = Cell::new(old_pos);

        self.set_chunk_cluster_active(pos);
    }


    pub fn update(&mut self) {
        for chunkrow in self.chunks.iter_mut() {
            for chunk in chunkrow.iter_mut() {
                chunk.start_step();
            }
        };


        let w = self.width as i32;
        let h = self.height as i32;

        for y in (0..h).rev() {
            for x in 0..w {
                let idx = x + y * w;
                let mut cell = self.cells[idx as usize];
                if cell.material == Material::Empty {
                    continue;
                };
                cell.processed_this_frame = false;
            };
        };

        let mut current_chunk = *self.get_chunk_for_pos(IVec2::new(0, h - 1)).unwrap();
        for y in (0..h).rev() {
            for x in 0..w {
                let cur_pos = IVec2::new(x, y);
                //println!("{}", cur_pos);
                let cur_chunk = self.get_chunk_for_pos(cur_pos);
                if let Some(cur_chunk) = cur_chunk {
                    if *cur_chunk != current_chunk {
                        current_chunk = *cur_chunk;
                    }
                }

                if current_chunk.should_step {
                    let idx = x + y * w;
                    let mut cell = self.cells[idx as usize];
                    if cell.material == Material::Empty {
                        continue;
                    };
                    if !cell.processed_this_frame {
                        cell.update(self);
                        cell.processed_this_frame = true;
                    };
                };
            }
        };
        //self.cells = self.scratch_cells.clone();
        //std::mem::replace(&mut self.cells, self.scratch_cells);
    }

    pub fn draw(&self, screen: &mut [u8]) {
        debug_assert_eq!(screen.len(), 4 * self.cells.len());
        for (c, pix) in self.cells.iter().zip(screen.chunks_exact_mut(4)) {
            let mut draw_color = c.color;
            
            let chunk = self.get_chunk_for_pos_not_mut(c.pos);
            if let Some(chunk) = chunk {
                if self.debug_draw && chunk.should_step {
                    draw_color = Color::RED;
                }
            };
            //println!("{}", c.pos);
            let color = [(draw_color.r * 255.0) as u8, (draw_color.g * 255.0) as u8, (draw_color.b * 255.0) as u8, (draw_color.a * 255.0) as u8];
            pix.copy_from_slice(&color);
        }
    }


    pub fn set_line(&mut self, x0: isize, y0: isize, x1: isize, y1: isize, material: Material) {
        // probably should do sutherland-hodgeman if this were more serious.
        // instead just clamp the start pos, and draw until moving towards the
        // end pos takes us out of bounds.
        let x0 = x0.max(0).min(self.width as isize);
        let y0 = y0.max(0).min(self.height as isize);
        for (x, y) in line_drawing::Bresenham::new((x0, y0), (x1, y1)) {
            if let Some(i) = self.grid_idx(x, y) {
                let pos = IVec2::new(x as i32, y as i32);
                self.cells[i] = Cell::new_material(pos, material);
                self.set_chunk_active(pos);
            } else {
                break;
            }
        }
    }

    pub fn grid_idx<I: std::convert::TryInto<usize>>(&self, x: I, y: I) -> Option<usize> {
        if let (Ok(x), Ok(y)) = (x.try_into(), y.try_into()) {
            if x < self.width && y < self.height {
                Some(x + y * self.width)
            } else {
                None
            }
        } else {
            None
        }
    }

    
}
