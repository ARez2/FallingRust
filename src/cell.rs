use pixels::wgpu::Color;
use crate::matrix::Matrix;
use glam::IVec2;
use strum_macros::EnumIter;


#[derive(Clone, Copy)]
pub enum MaterialType {
    Empty,
    Solid,
    MovableSolid,
    Liquid,
    Gas,
}



#[derive(Clone, Copy, PartialEq, EnumIter)]
pub enum Material {
    Empty,
    Sand,
    Water,
}
impl Material {
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

    pub fn get_type(&self) -> MaterialType {
        match self {
            Material::Empty => MaterialType::Empty,
            Material::Sand => MaterialType::MovableSolid,
            Material::Water => MaterialType::Liquid,
            _ => MaterialType::Solid,
        }
    }
}



#[derive(Clone, Copy)]
pub struct Cell {
    pub pos: IVec2,
    pub hp: u64,
    pub color: Color,
    pub material: Material,
    pub processed_this_frame: bool,
}



impl Cell {
    pub fn new(pos: IVec2) -> Self {
        Self {
            pos,
            material: Material::Empty,
            color: Material::Empty.get_color(),
            hp: Material::Empty.get_hp(),
            processed_this_frame: false,
        }
    }

    pub fn new_material(pos: IVec2, material: Material) -> Self {
        Self {
            pos,
            material: material,
            hp: material.get_hp(),
            color: material.get_color(),
            processed_this_frame: false,
        }
    }

    pub fn update(&mut self, matrix: &mut Matrix) {
        match self.material.get_type() {
            MaterialType::MovableSolid => self.step_movable_solid(matrix),
            _ => (),
        };
        //matrix.set_chunk_cluster_active(self.pos);
    }

    pub fn move_to_if_material(&self, matrix: &mut Matrix, pos: IVec2, materials: Vec<Material>) -> bool {
        let mat_at_pos = matrix.get_cell(pos);
        if let Some(mat_at_pos) = mat_at_pos {
            if materials.contains(&mat_at_pos.material) {
                matrix.set_cell(pos, *self);
                return true;
            };
        };
        false
    }

    pub fn step_movable_solid(&mut self, matrix: &mut Matrix) {
        let bottom = self.pos + IVec2::new(0, 1);
        if self.move_to_if_material(matrix, bottom, vec![Material::Empty, Material::Water]) {return;};
        
        let bottom_left = self.pos + IVec2::new(-1, 1);
        let bottom_right = self.pos + IVec2::new(1, 1);
        let mut first = bottom_left;
        let mut second = bottom_right;
        if rand::random() {
            first = bottom_right;
            second = bottom_left
        };
        if self.move_to_if_material(matrix, first, vec![Material::Empty, Material::Water]) {return;};
        self.move_to_if_material(matrix, second, vec![Material::Empty, Material::Water]);
    }
}

