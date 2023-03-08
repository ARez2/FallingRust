use std::{sync::{RwLock, Arc, Mutex}, ops::Deref};

use glam::{IVec2};
use crate::{Color, WIDTH, HEIGHT, Rng, ASSETS, EMPTY, WALL};
use rayon::prelude::*;

use crate::{Cell, Assets, Material, Chunk, cell_handler, CHUNK_SIZE, brush::Brush};
const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_VEC: IVec2 = IVec2::new(CHUNK_SIZE_I32, CHUNK_SIZE_I32);
const NUM_CHUNKS_X: usize = (WIDTH / CHUNK_SIZE as u32) as usize;
const NUM_CHUNKS_Y: usize = (HEIGHT / CHUNK_SIZE as u32) as usize;

pub struct Matrix {
    pub width: usize,
    clamp_width: i32,
    pub height: usize,
    clamp_height: i32,
    
    cells: Vec<Cell>,
    data: Vec<usize>,
    pub chunks: Vec<Chunk>,

    pub debug_draw: bool,
    pub update_left: bool,
    pub brush: Brush,
    pub wait_time_after_frame: f32,
}

unsafe impl Send for Matrix {}

impl Matrix {
    pub fn new_empty(width: usize, height: usize) -> Self {
        assert!(width != 0 && height != 0);
        //let size = width.checked_mul(height).expect("too big");

        let cells = Vec::<Cell>::new();
        let data = vec![0; width * height];
        
        let mut chunks = vec![];
        for y in (0..height as i32).step_by(CHUNK_SIZE) {
            for x in (0..width as i32).step_by(CHUNK_SIZE) {
                let chunk = Chunk::new(IVec2::new(x, y), CHUNK_SIZE);
                chunks.push(chunk);
            };
        };


        Self {
            width,
            clamp_width: width as i32 - 1,
            height,
            clamp_height: height as i32 - 1,

            cells,
            data,
            chunks,

            debug_draw: false,
            brush: Brush::new(),
            update_left: true,
            wait_time_after_frame: 0.0,
        }
    }

    /// Checks wether the chunk position is valid
    pub fn chunk_in_bounds(&self, chunk_pos: IVec2) -> bool {
        (chunk_pos.x >= 0 && chunk_pos.x < self.width as i32 / CHUNK_SIZE_I32) && (chunk_pos.y >= 0 && chunk_pos.y < self.height as i32 / CHUNK_SIZE_I32)
    }
    
    /// Tells the chunk to be updated the next frame
    pub fn set_chunk_active(&mut self, pos: IVec2) {
        let chunk_pos = pos / CHUNK_SIZE_VEC;
        if self.chunk_in_bounds(chunk_pos) {
            self.chunks[chunk_pos.x as usize + chunk_pos.y as usize * NUM_CHUNKS_X].should_step_next_frame = true;
        };
    }
    
    /// Tells the chunk and all chunks around it to be updated the next frame
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

    /// Clamps the position to be within the bounds of the pixel buffer
    pub fn clamp_pos(&self, pos: IVec2) -> IVec2 {
        IVec2::new(std::cmp::max(0, std::cmp::min(pos.x, self.clamp_width)), std::cmp::max(0, std::cmp::min(pos.y, self.clamp_height)))
    }

    /// Converts the position into an index to be used in self.data
    fn cell_idx(&self, mut pos: IVec2) -> usize {
        //pos = self.clamp_pos(pos);
        (pos.x + pos.y * self.width as i32) as usize
    }

    /// Gets the index of the cell in self.cells at that position
    pub fn get_data_at_pos(&self, pos: IVec2) -> usize {
        let idx = self.cell_idx(pos);
        if idx >= self.data.len() {
            return 0;
        };
        self.data[idx]
    }

    /// Returns a reference to the cell at cell_index (which is written in self.data)
    fn get_cell_by_index(&self, cell_index: usize) -> Cell {
        if cell_index < 1 || cell_index > self.cells.len() {
            return EMPTY;
        };
        self.cells[cell_index - 1]
    }

    /// Returns a reference to the cell at this position
    pub fn get_cell_by_pos(&self, pos: IVec2) -> Cell {
        if !self.is_in_bounds(pos) {
            WALL
        } else {
            let cell_idx = self.get_data_at_pos(pos);
            if cell_idx == 0 {
                EMPTY
            } else {
                self.get_cell_by_index(cell_idx)
            }
        }
    }

    /// Returns a reference to all the neighbor cells around a position
    pub fn get_neighbor_cells(&self, pos: IVec2, radius: i32) -> Vec<Cell> {
        let mut neighbors = Vec::with_capacity(4);
        if radius == 1 {
            let left = IVec2::new(1, 0);
            let down = IVec2::new(0, 1);
            let mut n = self.get_cell_by_pos(pos - left);
            if n != EMPTY && n != WALL {
                neighbors.push(n);
            };
            n = self.get_cell_by_pos(pos + left);
            if n != EMPTY && n != WALL {
                neighbors.push(n);
            };
            n = self.get_cell_by_pos(pos - down);
            if n != EMPTY && n != WALL {
                neighbors.push(n);
            };
            n = self.get_cell_by_pos(pos + down);
            if n != EMPTY && n != WALL {
                neighbors.push(n);
            };
        } else {
            for y in pos.y-radius..=pos.y+radius {
                for x in pos.x-radius..=pos.x+radius {
                    let cur_pos = IVec2::new(x, y);
                    let neigh = self.get_cell_by_pos(cur_pos);
                    if neigh != EMPTY && neigh != WALL {
                        neighbors.push(neigh);
                    };
                };
            };
        };
        return neighbors;
    }

    /// Appends the cell to self.cells and updates self.data with its index
    pub fn add_cell_to_cells(&mut self, mut cell: Cell) {
        cell.set_color(unsafe {ASSETS.get_color_for_material(cell.pos, cell.material)});

        let cell_at_pos = self.get_data_at_pos(cell.pos);
        // If there is already a cell at that position, replace that cell in self.cells with the new cell
        if cell_at_pos != 0 {
            let _ = std::mem::replace(&mut self.get_cell_by_index(cell_at_pos), cell);
        } else {
            let c_idx = self.cell_idx(cell.pos);
            self.cells.push(cell);
            self.data[c_idx] = self.cells.len();
        };
    }

    /// Replaces the cell at cellpos with the last cell in self.cells (faster than shifting) and updates self.data
    pub fn remove_cell_from_cells(&mut self, cellpos: IVec2) {
        if self.cells.is_empty() {
            return;
        };
        let cell_index = self.get_data_at_pos(cellpos);
        let data_idx = self.cell_idx(cellpos);
        if data_idx >= self.data.len() {
            return;
        };
        self.data[data_idx] = 0;
        self.set_chunk_cluster_active(cellpos);
        let cell_to_remove_idx = cell_index - 1;
        if cell_index == self.cells.len() {
            self.cells.remove(cell_to_remove_idx);
            return;
        };
        let cell = self.get_cell_by_index(cell_index);
        if cell != EMPTY && cell != WALL {
            let last_cell_pos = self.cells.last().unwrap().pos;
            let idx_of_last_cell_in_data = self.cell_idx(last_cell_pos);
            self.data[idx_of_last_cell_in_data] = cell_index;
            self.cells.swap_remove(cell_to_remove_idx);
        };
    }

    pub fn set_cell(&mut self, cell_index: usize, cell: Cell) {
        self.cells[cell_index] = cell;
    }

    pub fn update_cell(&mut self, cell: Cell) {
        let i = self.cell_idx(cell.pos);
        let celldata = self.data[i];
        if celldata == 0 {
            return;
        };
        self.cells[celldata - 1] = cell;
    }

    /// Places a cell at specified pos with the material given
    pub fn set_cell_material(&mut self, mut pos: IVec2, material: Material, swap: bool) {
        if material == Material::Empty {
            self.remove_cell_from_cells(pos);
            return;
        };
        pos = self.clamp_pos(pos);
        let cell = Cell::new(pos, material);
        self.add_cell_to_cells(cell);
        self.set_cell_by_pos(pos, pos, swap);
    }

    /// Places a cell which is located at cellpos at the specified target position (pos)
    pub fn set_cell_by_pos(&mut self, pos: IVec2, cellpos: IVec2, swap: bool) -> Cell {
        let data_at_cellpos = self.get_data_at_pos(cellpos);
        if data_at_cellpos == 0 {
            return EMPTY;
        };

        // Index of the cell inside self.data
        let cell_pos_index = self.cell_idx(cellpos);
        // Index of position where the cell wants to go inside self.data
        let target_pos_index = self.cell_idx(pos);
        let data_at_targetpos = self.get_data_at_pos(pos);
        
        let mut cell = self.get_cell_by_index(data_at_cellpos);
        cell.pos = pos;
        let cellmat = cell.material;
        let _cell_velocity = cell.velocity;
        self.data[target_pos_index] = data_at_cellpos;
        self.cells[data_at_cellpos - 1] = cell;

        // Target cell is empty
        if data_at_targetpos == 0 || !swap {
            if pos != cellpos {
                self.data[cell_pos_index] = 0;
            };
        } else {
            let mut target_cell = self.get_cell_by_index(data_at_targetpos);
            if cellmat == target_cell.material {
                return cell;
            };
            target_cell.pos = cellpos;
            self.cells[data_at_targetpos - 1] = target_cell;
            self.data[cell_pos_index] = data_at_targetpos;
        };
        
        // Set both positions chunks active (new and previous cell position)
        self.set_chunk_active(cellpos);
        self.set_chunk_active(pos);
        let x_chunked = pos.x % CHUNK_SIZE_I32;
        let x_chunked_upper = CHUNK_SIZE_I32 - 1 - x_chunked;
        if x_chunked <= 5 || x_chunked_upper <= 5 {
            if x_chunked < x_chunked_upper {
                self.set_chunk_active(pos - IVec2::new(CHUNK_SIZE_I32, 0));
            } else {
                self.set_chunk_active(pos + IVec2::new(CHUNK_SIZE_I32, 0));
            }
        };
        let y_chunked = pos.y % CHUNK_SIZE_I32;
        let y_chunked_upper = CHUNK_SIZE_I32 - 1 - y_chunked;
        if y_chunked <= 5 || y_chunked_upper <= 5 {
            if y_chunked < y_chunked_upper {
                self.set_chunk_active(pos - IVec2::new(0, CHUNK_SIZE_I32));
            } else {
                self.set_chunk_active(pos + IVec2::new(0, CHUNK_SIZE_I32));
            }
        };
        cell
        //self.set_chunk_active(pos + cell_velocity.round().as_ivec2())
    }

    /// Places cells in the specified brush size
    pub fn draw_brush(&mut self, pos: IVec2, material: Material) {
        let bs = self.brush.size as i32;
        if bs == 1 && !self.brush.place_fire {
            self.set_cell_material(pos, material, false);
            return;
        };
        let bs_2 = bs as f32 / 2.0;
        let lower = bs_2.floor() as i32;
        let upper = bs_2.ceil() as i32;
        for y in (pos.y-lower..pos.y+upper).rev() {
            for x in pos.x-lower..pos.x+upper {
                let cur_pos = IVec2::new(x, y);
                if self.brush.place_fire {
                    let mut c = self.get_cell_by_pos(cur_pos);
                    if c.material.get_flammability() > 0.0 {
                        c.is_on_fire = true;
                        self.update_cell(c);
                    };
                    self.set_chunk_active(cur_pos);
                } else {
                    self.set_cell_material(cur_pos, material, false);
                };
            };
        };
    }

    /// New frame. Update the matrix (includes cells and chunks)
    pub fn update(&mut self) {
        // Tells all chunks that a new frame has begun
        self.chunks.par_iter_mut().for_each(|chunk| {
            chunk.start_step();
        });

        // Helpers to only convert to i32 once
        let w = self.width as i32;
        let h = self.height as i32;

        // Tell every cells that a new frame has begun
        self.cells.par_iter_mut().for_each(|cell| {
            cell.processed_this_frame = false;
            cell.post_update();
        });


        // Iterate all cells from the bottom up and either from left to right or the other way around
        // let cell_indices: Vec<(usize, usize)> = self.cells.par_iter()
        //     .enumerate()
        //     .map(|(i, c)| {
        //         let p = c.pos / CHUNK_SIZE_VEC;
        //         let ch_i = p.x as usize + p.y as usize * NUM_CHUNKS_X;
        //         (i, ch_i)
        //     })
        //     .collect();
        // cell_indices.iter().for_each(|i| {
        //     let i = *i;
        //     cell_handler::handle_cell(self, i.0, i.1);
        // });
        
        let len = self.cells.len();
        let range = match self.update_left {
            true => (0..len).collect::<Vec<usize>>(),
            false => (0..len).rev().collect::<Vec<usize>>(),
        };

        let cells_to_update: Vec<Cell> = range
            .into_par_iter().map(|i| {
                self.cells[i]
            }).filter(|cell| {
                let chunk_pos = cell.pos / CHUNK_SIZE_VEC;
                if !self.chunk_in_bounds(chunk_pos) {
                    return false;
                };
                let chunk_index = chunk_pos.x as usize + chunk_pos.y as usize * NUM_CHUNKS_X;
                let cur_chunk = &self.chunks[chunk_index];
                cur_chunk.should_step
            })
        .collect();
        
        for cell in cells_to_update {
            let mut cell = self.get_cell_by_pos(cell.pos);
            if cell == EMPTY || cell == WALL {
                continue;
            };
            let hp = cell.hp;
            cell.update();
            cell.processed_this_frame = true;
            if cell.hp != hp || cell.is_on_fire || cell.was_on_fire_last_frame {
                self.set_chunk_cluster_active(cell.pos);
            };
            cell_handler::handle_cell(self, cell);
        };


// TODO: Check adding/ removing cells results in panic, maybe because of update loop


        self.update_left = !self.update_left;
    }

    /// Helper function to always execute the same logic regardless of wether iterating from the left or right side of the window
    fn step_all(&mut self, x: i32, y: i32) {
        let cur_pos = IVec2::new(x, y);
        
        let chunk_pos = cur_pos / CHUNK_SIZE_VEC;
        if !self.chunk_in_bounds(chunk_pos) {
            return;
        };
        let chunk_index = chunk_pos.x as usize + chunk_pos.y as usize * NUM_CHUNKS_X;
        let cur_chunk = &self.chunks[chunk_index];
        
        // If the chunk should process, update the cell
        if cur_chunk.should_step {
            let cell_idx = self.get_data_at_pos(cur_pos);
            if cell_idx == 0 {
                return;
            };
            let mut cell = self.get_cell_by_index(cell_idx);
            if !cell.processed_this_frame {
                let hp = cell.hp;
                cell.update();
                cell.processed_this_frame = true;
                if cell.hp != hp || cell.is_on_fire || cell.was_on_fire_last_frame {
                    self.set_chunk_cluster_active(cur_pos);
                };
                cell_handler::handle_cell(self, cell);
            };
        };
    }


    /// Renders all the cells into the pixel buffer
    pub fn draw(&self, screen: &mut [u8]) {
        //debug_assert_eq!(screen.len(), 4 * self.cells.len());

        // Faster solution for filling the array
        unsafe {
            std::ptr::write_bytes(screen.as_mut_ptr(), 0, screen.len());
        };

        for cell in self.cells.iter() {
            let mut draw_color = cell.color;
            
            let chunk_pos = cell.pos / CHUNK_SIZE_VEC;
            if self.chunk_in_bounds(chunk_pos) {
                let chunk = &self.chunks[(chunk_pos.x as usize + chunk_pos.y as usize * NUM_CHUNKS_X)];
                if self.debug_draw && chunk.should_step {
                    draw_color = Color::RED;
                };
            };
    
            let idx = self.cell_idx(cell.pos) * 4;
            let pixel_color = &mut screen[idx..idx+4];
            let color = [(draw_color.r * 255.0) as u8, (draw_color.g * 255.0) as u8, (draw_color.b * 255.0) as u8, (draw_color.a * 255.0) as u8];
            if pixel_color != color {
                pixel_color.copy_from_slice(&color);
            };
        }
    }

    /// Draws a line with the specified material
    pub fn set_line(&mut self, x0: isize, y0: isize, x1: isize, y1: isize, material: Material) {
        let x0 = x0.clamp(0, self.width as isize);
        let y0 = y0.clamp(0, self.height as isize);
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
