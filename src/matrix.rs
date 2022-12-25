use glam::IVec2;
use log::{warn, debug, info};
use pixels::wgpu::Color;
use strum::IntoEnumIterator;

use crate::{Cell, Material, Chunk, MaterialType, rand_multiplier};

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


//#[derive(Clone)]
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
        cells.push(Cell::new(IVec2::new(-1, -1)));
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



    fn is_in_bounds(&self, pos: IVec2) -> bool {
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
    pub fn get_cell_idx_data(&self, pos: IVec2) -> usize {
        let idx = self.cell_idx(pos);
        self.data[idx]
    }

    pub fn get_cell(&self, pos: IVec2) -> Option<&Cell> {
        if !self.is_in_bounds(pos) {
            None
        } else {
            let cell_idx = self.get_cell_idx_data(pos);
            if cell_idx == 0 {
                return None;
            };
            Some(&self.cells[cell_idx])
        }
    }
    pub fn get_cell_mut(&mut self, pos: IVec2) -> Option<&mut Cell> {
        if !self.is_in_bounds(pos) {
            None
        } else {
            let cell_idx = self.get_cell_idx_data(pos);
            if cell_idx == 0 {
                return None;
            };
            Some(&mut self.cells[cell_idx])
        }
    }
    pub fn get_cell_by_cellindex_mut(&mut self, cell_index: usize) -> Option<&mut Cell> {
        if cell_index > self.cells.len() {
            return None;
        };
        Some(&mut self.cells[cell_index])
    }


    /// Appends the cell to self.cells and updates self.data with its index
    pub fn add_cell_to_cells(&mut self, cell: &mut Cell) {
        let idx_in_data = self.cell_idx(cell.pos);
        if self.data[idx_in_data] != 0 {
            let old = std::mem::replace(&mut self.cells[self.data[idx_in_data]], *cell);
        } else {
            self.cells.push(*cell);
            self.data[idx_in_data] = self.cells.len() - 1;
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
        
        // Index of the cell inside self.cells
        let cell_index_in_data = self.data[cell_pos_index];
        // Index of "cell which is currently at the target position" inside self.cells
        let cell_at_target = self.data[target_pos_index];
        if cell_index_in_data == 0 {
            return;
        };

        let cell_index_in_data_idx = cell_index_in_data;

        // Target cell is empty
        if cell_at_target == 0 || !swap {
            if pos != cellpos {
                self.data[cell_pos_index] = 0;
            };
            self.data[target_pos_index] = cell_index_in_data_idx;
        } else {
            let cell_at_target_idx = cell_at_target;

            if cell_index_in_data_idx == cell_at_target_idx {
                return;
            };

            self.data[target_pos_index] = cell_at_target_idx;
            // Set the value of self.data at the old cells position to the value of the cell that was swapped
            self.data[cell_pos_index] = cell_index_in_data_idx;
            // Set the swapped cells position to the cells old position
            self.cells[cell_at_target_idx].pos = cellpos;
        }
        
        // Set both positions chunks active (new and previous cell position)
        self.set_chunk_cluster_active(cellpos);
        self.set_chunk_cluster_active(pos);

        // Set the position of cell to the target position
        self.cells[cell_index_in_data_idx].pos = pos;
    }

    pub fn set_cell(&mut self, pos: IVec2, cell: &mut Cell, swap: bool) {
        if !self.is_in_bounds(pos) {
            warn!("Pos out of range");
            return;
        };
        if self.get_cell_idx_data(cell.pos) == 0 {
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
        let mut bs = self.brush_size as i32;
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
            let cell_idx = cell_idx;
            let cell = self.get_cell_by_cellindex_mut(cell_idx).unwrap();
            //let processed = cell.processed_this_frame;
            //std::mem::drop(cell);
            // if cell.material == Material::Empty {
            //     return;
            // };
            if !cell.processed_this_frame {
                let cell = self.get_cell_by_cellindex_mut(cell_idx).unwrap();
                //let cellvel = cell.velocity;
                cell.update();
                cell.processed_this_frame = true;
                // let cellpos = cell.pos;
                // if cell.velocity != cellvel {
                //     self.set_chunk_cluster_active(cellpos);
                // };
                
                self.handle_cell(cell_idx);
            };
        };
    }



    fn handle_cell(&mut self, cell_index: usize) {
        let cell = self.get_cell_by_cellindex_mut(cell_index);
        if cell.is_none() {
            return;
        };
        let cell = cell.unwrap();
        // let material = cell.material;
        // std::mem::drop(cell);

        let did_move = match cell.material.get_type() {
            MaterialType::MovableSolid => self.movable_solid_step(cell_index),
            //MaterialType::Liquid => self.liquid_step(cell),
            _ => false,
        };
    }


    fn movable_solid_step(&mut self, cell_index: usize) -> bool {
        let mut bottom = IVec2::new(0, 1);
        {
            let cell = self.get_cell_by_cellindex_mut(cell_index);
            if cell.is_none() {
                return false;
            };
            let (freefall, cellpos) = {
                let cell = cell.unwrap();
                bottom = cell.pos + IVec2::new(0, cell.velocity.y.round() as i32);
                (cell.is_free_falling, cell.pos)
            };
            
            if freefall {
                for y in -1..=1 {
                    for x in -1..=1 {
                        let p = IVec2::new(x, y);
                        if p.abs() == IVec2::ONE && p == IVec2::ZERO {
                            continue;
                        };
                        let neighbour = self.get_cell_mut(cellpos + p);
                        if let Some(n_cell) = neighbour {
                            n_cell.attempt_free_fall();
                        };
                    }
                }
            };
        }

        
        if self.try_move(cell_index, bottom, false) {
            let cell = self.get_cell_by_cellindex_mut(cell_index).unwrap();
            cell.is_free_falling = true;
            return true;
        };

        //return false;
        let cell = self.get_cell_by_cellindex_mut(cell_index).unwrap();
        if !cell.is_free_falling {
            return false;
        };
        let mut fac = 1.0;
        if cell.velocity.x > 0.0 {
            fac = -1.0;
        } else if cell.velocity.x == 0.0 {
            if rand::random() {
                fac = -1.0;
            };
        };
        cell.velocity.x = (cell.velocity.y / 2.0) * fac;
        cell.velocity.y *= -0.1;

        // {
        //     let cell = self.get_cell_by_cellindex_mut(cell_index);
        //     if cell.is_none() {
        //         return false;
        //     };
        //     let cell = cell.unwrap();
        //     cell.velocity = cellvel;
        //     cell.is_free_falling = freefall;
        // }
        
        let x_vel_check = cell.velocity.x.round().abs().max(1.0) as i32;
        let disp = cell.material.get_dispersion() as i32;
        let bottom_left = cell.pos + IVec2::new(-1 * disp * x_vel_check, 1);
        let bottom_right = cell.pos + IVec2::new(1 * disp * x_vel_check, 1);
        let mut first = bottom_left;
        let mut second = bottom_right;
        if rand::random() {
            first = bottom_right;
            second = bottom_left
        };
        if self.try_move(cell_index, first, true) {
            return true;
        };
        if self.try_move(cell_index, second, true) {
            return true;
        };
        return false;
    }
    fn liquid_step(&mut self, cell_index: usize) -> bool {
        if self.movable_solid_step(cell_index) {
            return true;
        };

        let cell = self.get_cell_by_cellindex_mut(cell_index);
        if cell.is_none() {
            return false;
        };
        let cell = cell.unwrap();
        
        
        let horizontal_movement = IVec2::new(cell.material.get_dispersion() as i32 * rand_multiplier(), 0);
        if self.try_move(cell_index, horizontal_movement, false) {return true;};
        return false;
    }
    fn try_move(&mut self, cell_index: usize, to_pos: IVec2, diagonal: bool) -> bool {
        let mut last_possible_cell: Option<_> = None;
        
        let width = self.width as i32;
        let height = self.height as i32;
        
        let (cellpos, cellmat) = {
            let cell = self.get_cell_by_cellindex_mut(cell_index);
            if cell.is_none() {
                return false;
            };
            let cell = cell.unwrap();
            (cell.pos, cell.material)
        };
        if cellpos == to_pos {
            //println!("try move: attempt at moving to same cell");
            return false;
        };
        
        let x0 = cellpos.x.max(0).min(width);
        let y0 = cellpos.y.max(0).min(height);
        for (x, y) in line_drawing::WalkGrid::new((x0, y0), (to_pos.x, to_pos.y)) {
            let cur_pos = IVec2::new(x as i32, y as i32);
            if cur_pos == cellpos {
                continue;
            };
            let target_cell = self.get_cell(cur_pos);
            if let Some(tcell) = target_cell {
                if tcell.material.get_density() < cellmat.get_density() {
                    last_possible_cell = Some(cur_pos);
                };
                if last_possible_cell.is_none() && !diagonal {
                    break;
                };
            } else {
                // Cell is empty
                if self.is_in_bounds(cur_pos) {
                    last_possible_cell = Some(cur_pos);
                };
            };
        };

        match last_possible_cell {
            None => (),
            Some(last_pos) => {
                if last_pos != IVec2::new(x0, y0) {
                    //println!("{}  ------>  {}", cellpos, last_pos);
                    self.set_cell_by_pos(last_pos, cellpos, true);
                    return true;
                }
            },
        }

        return false;
    }







    pub fn draw(&self, screen: &mut [u8]) {
        debug_assert_eq!(screen.len(), 4 * self.cells.len());
        screen.fill(0);
        for c in self.cells.iter().skip(1) {
            
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
        // probably should do sutherland-hodgeman if this were more serious.
        // instead just clamp the start pos, and draw until moving towards the
        // end pos takes us out of bounds.
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
