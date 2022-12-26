use std::fmt::Display;

use pixels::wgpu::Color;
use crate::matrix::Matrix;
use glam::{IVec2, Vec2};

use crate::{Material, MaterialType};



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

    pub fn update(&mut self) {
        self.is_free_falling = self.pos.y != self.prev_pos.y;
        self.velocity += Vec2::new(0.0, 0.5);
        // let vel_round = self.velocity.round();
        // let desired_pos = self.pos + IVec2::new(vel_round.x as i32, vel_round.y as i32);
        self.prev_pos = self.pos;
        // return desired_pos;
}


    /// Tries to set a neighbouring cells "is_free_falling" to true based on inertia and that cells intertial resistance
    pub fn attempt_free_fall(&mut self) {
        if self.material.get_type() == MaterialType::MovableSolid {
            let chance = self.material.get_intertial_resistance();
            let rng = rand::random::<f32>();
            if rng > chance {
                self.is_free_falling = true;
                //self.color = Color::GREEN;
            };
        };
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {}, {}", self.material, self.pos, self.velocity)
    }
}
