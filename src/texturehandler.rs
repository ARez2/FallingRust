use image::{Rgb32FImage, RgbImage};
use std::{collections::HashMap, path::PathBuf};
use glam::IVec2;
use pixels::wgpu::Color;

use crate::{Material};



#[derive(Debug, )]
pub struct TextureInfo {
    pub texture: Rgb32FImage,
    pub being_used_by: usize,
    pub byte_texture: RgbImage,
}

pub struct TextureHandler {
    pub loaded_textures: HashMap<Material, TextureInfo>,
}
impl TextureHandler {
    pub fn new() -> Self {
        let loaded_textures = HashMap::new();
        TextureHandler {
            loaded_textures,
        }
    }

    fn get_color_from_tex(&self, pos: IVec2, tex: &Rgb32FImage) -> Color {
        let dims = tex.dimensions();
        let c = tex.get_pixel((pos.x % dims.0 as i32) as u32, (pos.y % dims.1 as i32) as u32);
        Color {r: c.0[0] as f64, g: c.0[1] as f64, b: c.0[2] as f64, a: 1.0}
    }

    pub fn get_color_for_material(&mut self, pos: IVec2, material: Material) -> Color {
        let mut output_color = Color { r: 1.0, g: 0.0, b: 0.8, a: 1.0 };
        let mut i = 0;
        let mut key_idx = 0;
        for m in self.loaded_textures.keys() {
            i += 1;
            if m == &material {
                let tex = &self.loaded_textures.get(m).unwrap().texture;
                output_color = self.get_color_from_tex(pos, tex);
                key_idx = i;
            };
        };
        if key_idx > 0 {
            let info = self.loaded_textures.values_mut().nth(key_idx - 1).unwrap();
            info.being_used_by += 1;
        } else {
            let tex = self.load_material_texture(material);
            if let Some(tex) = tex {
                output_color = self.get_color_from_tex(pos, &tex.0);
                self.loaded_textures.insert(material, TextureInfo {
                    texture: tex.0,
                    being_used_by: 1,
                    byte_texture: tex.1,
                });
                println!("Add texture for {:?}", material);
            };
        };
        
        return output_color;
    }


    pub fn load_material_texture(&mut self, material: Material) -> Option<(Rgb32FImage, RgbImage)> {
        let path = self.get_texturepath_from_material(material);
        //println!("{:?}", path);
        if path.exists() {
            let s = path.to_str();
            if let Some(filepath) = s {
                return self.get_material_texture(material, filepath);
            };
        };
        None
    }


    fn get_texturepath_from_material(&self, material: Material) -> PathBuf {
        let tex_name = match material {
            Material::Dirt => "dirt.png",
            Material::Sand => "sand.png",
            Material::Water => "water.png",
            Material::Rock => "rock.png",
            Material::Smoke => "smoke.png",
            _ => "../debug_color_02.png",
        };
        let cur_working_dir = std::env::current_dir().unwrap();
        let path = cur_working_dir.join("data").join("textures").join("materials").join(tex_name);
        path
    }

    fn get_material_texture(&mut self, material: Material, filepath: &str) -> Option<(Rgb32FImage, RgbImage)> {
        let vec = std::fs::read(filepath);
        if let Ok(vec) = vec {
            let diffuse_bytes = vec.as_slice();
            let diffuse_image = image::load_from_memory(diffuse_bytes);
            if let Ok(diffuse_image) = diffuse_image {
                let byte_img = diffuse_image.to_rgb8();
                let img = diffuse_image.to_rgb32f();
                return Some((img, byte_img));
            };
        };
        None
    }
    

    pub fn remove_material(&mut self, material: Material) {
        let mat = self.loaded_textures.get_mut(&material);
        if mat.is_some() {
            let mat = mat.unwrap();
            mat.being_used_by = mat.being_used_by.saturating_sub(1);
            if mat.being_used_by == 0 {
                self.loaded_textures.remove(&material);
                println!("Remove texture for {:?}", material);
            };
        }
    }
}
