use std::{fmt::Display, rc::Rc};

use crate::{Color, darken_color};
use glam::{IVec2, Vec2};

use crate::{Material, MaterialType, rand_multiplier};



#[derive(Clone, PartialEq)]
pub struct Cell {
    pub pos: IVec2,
    prev_pos: IVec2,
    pub velocity: Vec2,
    pub hp: u64,
    pub base_color: Color,
    pub color: Color,
    pub material: Material,
    pub processed_this_frame: bool,
    pub is_free_falling: bool,
    pub is_on_fire: bool,
    pub was_on_fire_last_frame: bool,
}

impl Cell {
    /// Creates a new cell with the specified material
    pub fn new(pos: IVec2, material: Material) -> Self {
        Self {
            pos,
            prev_pos: pos,
            velocity: Vec2::ZERO,
            material,
            hp: material.get_hp(),
            base_color: material.get_color(),
            color: material.get_color(),
            processed_this_frame: false,
            is_free_falling: true,
            is_on_fire: false,
            was_on_fire_last_frame: false,
        }
    }

    /// Updates the cells properties
    pub fn update(&mut self) {
        self.velocity += Vec2::new(0.0, 0.5);
        
        if self.is_on_fire {
            self.hp = self.hp.saturating_sub(1);
            self.color = Color {r: 1.0, g: 0.25 + 0.25 * rand_multiplier() as f64, b: 0.0, a: 1.0}
        } else {
            self.color = darken_color(self.base_color, self.hp as f64 / self.material.get_hp() as f64);
        };
    }
    
    /// Updates the cells properties after the cells has been handled by the cellhandler
    pub fn post_update(&mut self) {
        self.is_free_falling = self.pos.y != self.prev_pos.y;
        // if self.is_free_falling {
        //     self.color = Color::GREEN;
        // } else {
            //     self.color = self.material.get_color();
            // };
                
        self.was_on_fire_last_frame = self.is_on_fire;
        self.prev_pos = self.pos;
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

    pub fn set_color(&mut self, color: Color) {
        self.base_color = color;
        self.color = color;
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {}, {}", self.material, self.pos, self.velocity)
    }
}
