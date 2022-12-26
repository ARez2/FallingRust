use glam::{IVec2, Vec2};
use log::{warn, debug, info};
use pixels::wgpu::Color;
use strum::IntoEnumIterator;

use crate::{Cell, Material, Chunk, cell_handler};

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


pub struct Matrix {
    pub width: usize,
    pub height: usize,
    
    cells: Vec<Cell>,
    data: Vec<usize>,
    pub chunks: Vec<Vec<Chunk>>,

    pub debug_draw: bool,
    pub brush_size: u8,
    pub brush_material_index: usize,
    pub update_left: bool,
}

impl Matrix {
    pub fn new_empty(width: usize, height: usize) -> Self {
        assert!(width != 0 && height != 0);
        let size = width.checked_mul(height).expect("too big");

        let mut cells = Vec::<Cell>::new();
        //cells.push(Cell::new(IVec2::new(-1, -1)));
        let data = vec![0; width * height];
        
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
            data,
            chunks,
            debug_draw: false,
            brush_size: 1,
            brush_material_index: 1,
            update_left: true,
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
    pub fn get_chunk_for_pos(&self, pos: IVec2) -> Option<&Chunk> {
        let chunk_pos = pos / IVec2::new(CHUNK_SIZE_I32, CHUNK_SIZE_I32);
        if self.chunk_in_bounds(chunk_pos) {
            Some(&self.chunks[chunk_pos.y as usize][chunk_pos.x as usize])
        } else {
            None
        }
    }
    pub fn get_chunk_for_pos_mut(&mut self, pos: IVec2) -> Option<&mut Chunk> {
        let chunk_pos = pos / IVec2::new(CHUNK_SIZE_I32, CHUNK_SIZE_I32);
        if self.chunk_in_bounds(chunk_pos) {
            Some(&mut self.chunks[chunk_pos.y as usize][chunk_pos.x as usize])
        } else {
            None
        }
    }
    pub fn set_chunk_active(&mut self, pos: IVec2) {
        let chunk_res = self.get_chunk_for_pos_mut(pos);
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



    pub fn is_in_bounds(&self, pos: IVec2) -> bool {
        (pos.x >= 0 && pos.x < self.width as i32) && (pos.y >= 0 && pos.y < self.height as i32)
    }
    fn clamp_pos(&self, pos: IVec2) -> IVec2 {
        IVec2::new(pos.x.max(0).min(self.width as i32 - 1), pos.y.max(0).min(self.height as i32 - 1))
    }
    fn cell_idx(&self, mut pos: IVec2) -> usize {
        pos = self.clamp_pos(pos);
        (pos.x + pos.y * self.width as i32) as usize
    }
    /// Converts the position into an index in self.data
    pub fn get_data_at_pos(&self, pos: IVec2) -> usize {
        let idx = self.cell_idx(pos);
        self.data[idx]
    }
    fn get_cell_from_cells(&self, cell_index: usize) -> Option<&Cell> {
        if cell_index < 1 || cell_index > self.cells.len() {
            return None;
        };
        Some(&self.cells[cell_index - 1])
    }
    fn get_cell_from_cells_mut(&mut self, cell_index: usize) -> Option<&mut Cell> {
        if cell_index < 1 || cell_index > self.cells.len() {
            return None;
        };
        Some(&mut self.cells[cell_index - 1])
    }

    pub fn get_cell(&self, pos: IVec2) -> Option<&Cell> {
        if !self.is_in_bounds(pos) {
            None
        } else {
            let cell_idx = self.get_data_at_pos(pos);
            if cell_idx == 0 {
                return None;
            };
            self.get_cell_from_cells(cell_idx)
        }
    }
    pub fn get_cell_mut(&mut self, pos: IVec2) -> Option<&mut Cell> {
        if !self.is_in_bounds(pos) {
            None
        } else {
            let cell_idx = self.get_data_at_pos(pos);
            if cell_idx == 0 {
                return None;
            };
            self.get_cell_from_cells_mut(cell_idx)
        }
    }
    pub fn get_cell_by_cellindex_mut(&mut self, cell_index: usize) -> Option<&mut Cell> {
        if cell_index > self.cells.len() {
            return None;
        };
        self.get_cell_from_cells_mut(cell_index)
    }


    /// Appends the cell to self.cells and updates self.data with its index
    pub fn add_cell_to_cells(&mut self, cell: &mut Cell) {
        let cell_at_pos = self.get_data_at_pos(cell.pos);
        // If there is already a cell at that position, replace that cell in self.cells with the new cell
        if cell_at_pos != 0 {
            let old = std::mem::replace(self.get_cell_from_cells_mut(cell_at_pos).unwrap(), *cell);
        } else {
            self.cells.push(*cell);
            let c_idx = self.cell_idx(cell.pos);
            self.data[c_idx] = self.cells.len();
        };
        //println!("self.cells len: {}, idx in data: {}", self.cells.len(), idx_in_data);
    }
    pub fn set_cell_material(&mut self, pos: IVec2, material: Material, swap: bool) {
        let mut cell = Cell::new_material(pos, material);
        self.add_cell_to_cells(&mut cell);
        self.set_cell_by_pos(pos, cell.pos, swap);
    }


    pub fn set_cell_by_pos(&mut self, pos: IVec2, cellpos: IVec2, swap: bool) {
        // Index of the cell inside self.data
        let cell_pos_index = self.cell_idx(cellpos);
        // Index of position where the cell wants to go inside self.data
        let target_pos_index = self.cell_idx(pos);
        
        let mut data_at_cellpos = self.get_data_at_pos(cellpos);
        let mut data_at_targetpos = self.get_data_at_pos(pos);
        if data_at_cellpos == 0 {
            return;
        };
        
        // if pos.y > 237 {
        //     for cell in self.cells.iter() {
        //         println!("Set cell by pos: {}", cell);
        //     };
        // };

        // Target cell is empty
        if data_at_targetpos == 0 || !swap {
            // if pos.y == 239 {
            //     println!("Calling cell: {} (cellpos: {}) wants to move to {}. Cell at that pos: {}", self.get_cell_from_cells(data_at_cellpos).unwrap(), cellpos, pos, self.data[target_pos_index]);
            // };
            if pos != cellpos {
                self.data[cell_pos_index] = 0;
            };
            self.data[target_pos_index] = data_at_cellpos;

            if let Some(calling_cell) = self.get_cell_from_cells_mut(data_at_cellpos) {
                calling_cell.pos = pos;
            };
        } else {
            let cellmat = self.get_cell_from_cells_mut(data_at_cellpos).unwrap().material;
            let target_cell = self.get_cell_from_cells_mut(data_at_targetpos).unwrap();
            let target_cellmat = target_cell.material;
            
            if cellmat == target_cellmat {
                return;
            };
            //let cell_at_target_idx = cell_at_target;
            //println!();
            //println!();
            //println!("Swap {} with {}", self.get_cell_from_cells(data_at_cellpos).unwrap(), self.get_cell_from_cells(data_at_targetpos).unwrap());
            self.get_cell_from_cells_mut(data_at_targetpos).unwrap().pos = cellpos;
            self.get_cell_from_cells_mut(data_at_cellpos).unwrap().pos = pos;

            self.data[cell_pos_index] = data_at_targetpos;
            self.data[target_pos_index] = data_at_cellpos;
            //println!("After swap {} ---- {}", self.get_cell_from_cells(data_at_cellpos).unwrap(), self.get_cell_from_cells(data_at_targetpos).unwrap());
        }

        
        // Set both positions chunks active (new and previous cell position)
        self.set_chunk_cluster_active(cellpos);
        self.set_chunk_cluster_active(pos);
    }

    pub fn set_cell(&mut self, pos: IVec2, cell: &mut Cell, swap: bool) {
        if !self.is_in_bounds(pos) {
            warn!("Pos out of range");
            return;
        };
        if self.get_data_at_pos(cell.pos) == 0 {
            if !self.cells.contains(cell) {
                self.add_cell_to_cells(cell);
            };
        };
        if cell.pos == pos {
            return;
        };
        self.set_cell_by_pos(pos, cell.pos, swap);
    }

    pub fn draw_brush(&mut self, pos: IVec2, material: Material) {
        let bs = self.brush_size as i32;
        if bs == 1 {
            self.set_cell_material(pos, material, false);
            return;
        };
        let bs_2 = bs as f32 / 2.0;
        let lower = bs_2.floor() as i32;
        let upper = bs_2.ceil() as i32;
        for y in (pos.y-lower..pos.y+upper).rev() {
            for x in pos.x-lower..pos.x+upper {
                self.set_cell_material(IVec2::new(x, y), material, false);
            };
        };
    }

    pub fn get_material_from_brushindex(&self) -> Material {
        Material::iter().nth(self.brush_material_index).unwrap()
    }



    pub fn update(&mut self) {
        // Tells all chunks that a new frame has begun
        for chunkrow in self.chunks.iter_mut() {
            for chunk in chunkrow.iter_mut() {
                chunk.start_step();
            }
        };

        // Helpers to only convert to i32 once
        let w = self.width as i32;
        let h = self.height as i32;

        // Tell every cells that a new frame has begun
        for cell in self.cells.iter_mut() {
            cell.processed_this_frame = false;
        };

        // Iterate all cells from the bottom up and either from left to right or the other way around
        for y in (0..h).rev() {
            if self.update_left {
                for x in (0..w).rev() {
                    self.step_all(x, y, w);
                }
            } else {
                for x in 0..w {
                    self.step_all(x, y, w);
                }
            };
            self.update_left = !self.update_left;
        };
    }


    /// Helper function to always execute the same logic regardless of wether iterating from the left or right side of the window
    fn step_all(&mut self, x: i32, y: i32, w: i32) {
        let cur_pos = IVec2::new(x, y);
        
        let cur_chunk = self.get_chunk_for_pos(cur_pos);
        if cur_chunk.is_none() {
            return;
        };
        let current_chunk = cur_chunk.unwrap();
        
        // If the chunk should process, update the cell
        if current_chunk.should_step {
            let idx = (x + y * w) as usize;
            let cell_idx = self.data[idx];
            if cell_idx == 0 {
                return;
            };
            let cell = self.get_cell_by_cellindex_mut(cell_idx).unwrap();
            if !cell.processed_this_frame {
                let cell = self.get_cell_by_cellindex_mut(cell_idx).unwrap();
                cell.update();
                cell.processed_this_frame = true;
                cell_handler::handle_cell(self, cell_idx);
            };
        };
    }


    pub fn draw(&self, screen: &mut [u8]) {
        debug_assert_eq!(screen.len(), 4 * self.cells.len());
        screen.fill(0);
        //.skip(1)
        for c in self.cells.iter() {
            
            let mut draw_color = c.color;
            
            let chunk = self.get_chunk_for_pos_not_mut(c.pos);
            if let Some(chunk) = chunk {
                if self.debug_draw && chunk.should_step {
                    draw_color = Color::RED;
                }
            };
            let idx = self.cell_idx(c.pos) * 4;
            let color = [(draw_color.r * 255.0) as u8, (draw_color.g * 255.0) as u8, (draw_color.b * 255.0) as u8, (draw_color.a * 255.0) as u8];
            screen[idx + 0] = color[0];
            screen[idx + 1] = color[1];
            screen[idx + 2] = color[2];
            screen[idx + 3] = color[3];
        }
    }


    pub fn set_line(&mut self, x0: isize, y0: isize, x1: isize, y1: isize, material: Material) {
        let x0 = x0.max(0).min(self.width as isize);
        let y0 = y0.max(0).min(self.height as isize);
        for (x, y) in line_drawing::Bresenham::new((x0, y0), (x1, y1)) {
            let pos = IVec2::new(x as i32, y as i32);
            if self.is_in_bounds(pos) {
                self.draw_brush(pos, material);
            } else {
                break;
            }
        }
    }
}
