use strum::IntoEnumIterator;

use crate::Material;



pub struct Brush {
    pub size: u16,
    pub material_index: usize,
    pub place_fire: bool,
}

impl Brush {
    pub fn new() -> Self {
        Self {
            size: 35,
            material_index: 0,
            place_fire: false,
        }
    }

    /// Converts the brush_material_index to a Material
    pub fn get_material_from_index(&self) -> Material {
        Material::iter().nth(self.material_index).unwrap()
    }

    pub fn increase_material_index(&mut self) {
        self.material_index += 1;
        if self.material_index >= Material::iter().count() {
            self.material_index = 0;
        };
    }

    pub fn decrease_material_index(&mut self) {
        if self.material_index == 0 {
            self.material_index = Material::iter().count() - 1;
        } else {
            self.material_index -= 1;
        };
    }
}
impl Default for Brush {
    fn default() -> Self {
        Self::new()
    }
}