use std::{fmt::Display, sync::Arc, ops::{DerefMut, Deref}};

use pixels::wgpu::Color;
use crate::matrix::Matrix;
use glam::{IVec2, Vec2};
use strum_macros::EnumIter;


#[derive(Clone, Copy, PartialEq)]
pub enum MaterialType {
    Empty,
    Solid,
    MovableSolid,
    Liquid,
    Gas,
}



#[derive(Clone, Copy, PartialEq, EnumIter, Debug)]
pub enum Material {
    Empty,
    Sand,
    Water,
}

impl Material {
    pub fn get_type(&self) -> MaterialType {
        match self {
            Material::Empty => MaterialType::Empty,
            Material::Sand => MaterialType::MovableSolid,
            Material::Water => MaterialType::Liquid,
            _ => MaterialType::Solid,
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Material::Empty => Color::BLACK,
            Material::Sand => Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 },
            Material::Water => Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
        }
    }

    pub fn get_hp(&self) -> u64 {
        match self {
            Material::Empty => 0,
            Material::Sand => 10,
            Material::Water => 20,
        }
    }

    pub fn get_density(&self) -> u64 {
        match self {
            Material::Empty => 0,
            Material::Sand => 300,
            Material::Water => 1,
        }
    }

    pub fn get_dispersion(&self) -> u8 {
        match self {
            Material::Empty => 0,
            Material::Sand => 1,
            Material::Water => 5,
        }
    }

    pub fn get_intertial_resistance(&self) -> f32 {
        match self {
            Material::Sand => 1.1,
            _ => 0.0,
        }
    }
}



#[derive(Clone, Copy, PartialEq)]
pub struct Cell {
    pub pos: IVec2,
    prev_pos: IVec2,
    pub velocity: Vec2,
    pub hp: u64,
    pub color: Color,
    pub material: Material,
    pub processed_this_frame: bool,
    pub is_free_falling: bool,
}


impl Cell {
    pub fn new(pos: IVec2) -> Self {
        Self {
            pos,
            prev_pos: pos,
            velocity: Vec2::ZERO,
            material: Material::Empty,
            color: Material::Empty.get_color(),
            hp: Material::Empty.get_hp(),
            processed_this_frame: false,
            is_free_falling: true,
        }
    }

    pub fn new_material(pos: IVec2, material: Material) -> Self {
        Self {
            pos,
            prev_pos: pos,
            velocity: Vec2::ZERO,
            material,
            hp: material.get_hp(),
            color: material.get_color(),
            processed_this_frame: false,
            is_free_falling: true,
        }
    }

    pub fn update(&mut self, matrix: &mut Matrix) -> bool {
        self.is_free_falling = self.pos.y != self.prev_pos.y;
        if self.is_free_falling {
            self.color = Color::BLUE;
        } else {
            self.color = Color::GREEN;
        };
        
        self.velocity += Vec2::new(0.0, 0.5);
        let did_change = match self.material.get_type() {
            MaterialType::MovableSolid => self.step_movable_solid(matrix),
            MaterialType::Liquid => self.step_liquid(matrix),
            _ => false,
        };
        self.prev_pos = self.pos;

        did_change
        //matrix.set_chunk_cluster_active(self.pos);
    }


    /// Tries to set a neighbouring cells "is_free_falling" to true based on inertia and that cells intertial resistance
    pub fn attempt_free_fall(&mut self) {
        if self.material.get_type() == MaterialType::MovableSolid {
            let chance = self.material.get_intertial_resistance();
            if rand::random::<f32>() > chance {
                self.is_free_falling = true;
            };
        };
    }


    pub fn swap_density(&mut self, matrix: &mut Matrix, pos: IVec2) -> bool {
        let mat_at_pos = matrix.get_cell(pos);
        if let Some(mat_at_pos) = mat_at_pos {
            let mat_density = mat_at_pos.material.get_density();
            if self.material.get_density() > mat_density {
                matrix.set_cell(pos, self, true);
                return true;
            };
        };
        false
    }

    pub fn step_movable_solid(&mut self, matrix: &mut Matrix) -> bool {
        if self.is_free_falling {
            for y in -1..=1 {
                for x in -1..=1 {
                    let p = IVec2::new(x, y);
                    if p.abs() == IVec2::ONE && p == IVec2::ZERO {
                        continue;
                    };
                    let neighbour = matrix.get_cell_mut(self.pos + p);
                    if let Some(n_cell) = neighbour {
                        n_cell.attempt_free_fall();
                    }
                }
            }
        };



        let bottom = self.pos + IVec2::new(0, self.velocity.y.round() as i32);
        if self.try_move(matrix, bottom, false) {
            self.is_free_falling = true;
            return true;
        };
        if !self.is_free_falling {
            return false;
        };
        let mut fac = 1.0;
        if self.velocity.x > 0.0 {
            fac = -1.0;
        } else if self.velocity.x == 0.0 {
            if rand::random() {
                fac = -1.0;
            };
        };
        self.velocity.x = (self.velocity.y / 2.0) * fac;
        self.velocity.y *= -0.1;
        
        let x_vel_check = self.velocity.x.round().abs().max(1.0) as i32;
        let disp = self.material.get_dispersion() as i32;
        let bottom_left = self.pos + IVec2::new(-1 * disp * x_vel_check, 1);
        let bottom_right = self.pos + IVec2::new(1 * disp * x_vel_check, 1);
        let mut first = bottom_left;
        let mut second = bottom_right;
        if rand::random() {
            first = bottom_right;
            second = bottom_left
        };
        if self.try_move(matrix, first, true) {
            return true;
        };
        if self.try_move(matrix, second, true) {
            return true;
        };
        return false;
    }

    pub fn step_liquid(&mut self, matrix: &mut Matrix) -> bool {
        if self.step_movable_solid(matrix) {return true;};
        
        let dir_multi = match rand::random::<bool>() {
            true => 1,
            false => -1,
        };
        if self.try_move(matrix, IVec2::new(self.material.get_dispersion() as i32 * dir_multi, 0), false) {return true;};
        return false;
    }


    fn try_move(&mut self, matrix: &mut Matrix, to_pos: IVec2, diagonal: bool) -> bool {
        let mut swapped = false;
        let mut last_possible_cell: Option<_> = None;
        
        let x0 = self.pos.x.max(0).min(matrix.width as i32);
        let y0 = self.pos.y.max(0).min(matrix.height as i32);
        for (x, y) in line_drawing::WalkGrid::new((x0, y0), (to_pos.x, to_pos.y)) {
            let cur_pos = IVec2::new(x as i32, y as i32);
            if cur_pos == self.pos {
                continue;
            };
            let target_cell = matrix.get_cell(cur_pos);
            if let Some(c) = target_cell {
                if c.material.get_density() < self.material.get_density() {
                    last_possible_cell = Some(c.clone());
                };
                if last_possible_cell.is_none() && !diagonal {
                    break;
                };
            } else {
                break;
            };
        };

        match last_possible_cell {
            None => (),
            Some(cell_2) => {
                if cell_2 != *self {
                    let cell_2_pos = cell_2.pos;
                    matrix.set_cell(cell_2_pos, self, true);
                    swapped = true;
                }
            },
        }

        return swapped;
    }
}

