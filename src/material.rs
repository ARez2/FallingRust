use pixels::wgpu::Color;
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
    Dirt,
    Water,
}

impl Material {
    pub fn get_type(&self) -> MaterialType {
        match self {
            Material::Empty => MaterialType::Empty,
            Material::Sand => MaterialType::MovableSolid,
            Material::Dirt => MaterialType::MovableSolid,
            Material::Water => MaterialType::Liquid,
            _ => MaterialType::Solid,
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Material::Empty => Color::BLACK,
            Material::Sand => Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 },
            Material::Dirt => Color { r: 0.41, g: 0.25, b: 0.2, a: 1.0 },
            Material::Water => Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
        }
    }

    pub fn get_hp(&self) -> u64 {
        match self {
            Material::Empty => 0,
            Material::Sand => 10,
            Material::Dirt => 20,
            Material::Water => 20,
        }
    }

    pub fn get_density(&self) -> u64 {
        match self {
            Material::Empty => 0,
            Material::Sand => 300,
            Material::Dirt => 500,
            Material::Water => 1,
        }
    }

    pub fn get_dispersion(&self) -> u8 {
        match self {
            Material::Empty => 0,
            Material::Sand => 1,
            Material::Dirt => 1,
            Material::Water => 5,
        }
    }

    pub fn get_intertial_resistance(&self) -> f32 {
        match self {
            Material::Sand => 0.1,
            Material::Dirt => 0.9,
            _ => 0.0,
        }
    }
}