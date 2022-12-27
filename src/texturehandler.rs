use image::{Rgb32FImage};
use std::{collections::{HashSet, HashMap}, path::Path};
use glam::IVec2;
use pixels::wgpu::Color;

use crate::Material;

const DEFAULT_MATERIAL_PATH: &'static str = ".\\..\\data\\textures\\materials";

#[derive(Debug, )]
pub struct TextureInfo {
    pub texture: Rgb32FImage,
}

pub struct TextureHandler {
    loaded_textures: HashMap<Material, TextureInfo>,
}
impl TextureHandler {
    pub fn new() -> Self {
        let loaded_textures = HashMap::new();
        TextureHandler {
            loaded_textures,
        }
    }

    pub fn get_color_for_material(&mut self, pos: IVec2, material: Material) -> Color {
        for m in self.loaded_textures.keys() {
            if m == &material {
                let tex = &self.loaded_textures.get(m).unwrap().texture;
                return self.get_color_from_tex(pos, tex);
            };
        };
        let tex_name = match material {
            Material::Dirt => "dirt.png",
            Material::Sand => "sand.png",
            Material::Water => "water.png",
            _ => "../debug_color_02.png",
        };
        //let p = Path::;
        let cur_working_dir = std::env::current_dir().unwrap();
        let path = cur_working_dir.join("data").join("textures").join("materials").join(tex_name);
        
        if path.exists() {
            let s = path.to_str();
            if let Some(filepath) = s {
                let vec = std::fs::read(filepath).unwrap();
                let diffuse_bytes = vec.as_slice();
                let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
                let img = diffuse_image.to_rgb32f();
                let col = self.get_color_from_tex(pos, &img);
                self.loaded_textures.insert(material, TextureInfo {
                    texture: img,
                });
                return col;
            };
        };

        Color { r: 1.0, g: 0.0, b: 0.8, a: 1.0 }
    }

    fn get_color_from_tex(&self, pos: IVec2, tex: &Rgb32FImage) -> Color {
        let dims = tex.dimensions();
        let c = tex.get_pixel((pos.x % dims.0 as i32) as u32, (pos.y % dims.1 as i32) as u32);
        Color {r: c.0[0] as f64, g: c.0[1] as f64, b: c.0[2] as f64, a: 1.0}
    }
}
