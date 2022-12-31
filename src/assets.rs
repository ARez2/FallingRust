use std::{collections::HashMap, path::PathBuf};
use pixels::wgpu::Color;
use glam::IVec2;
use image::{Rgb32FImage};

use crate::{Material, COLOR_EMPTY};


#[derive(Debug)]
pub struct TextureInfo {
    pub texture: Rgb32FImage,
    pub pixels: Vec<u8>,
    pub being_used_by: usize,
}


pub struct Assets {
    // All the textures loaded in the game
    loaded_textures: HashMap<String, TextureInfo>,
}

impl Assets {
    pub fn new() -> Self {
        Assets {
            loaded_textures: HashMap::new(),
        }
    }

    /// If the texture has already been loaded, return it, else load it
    pub fn add_texture_instance(&mut self, filepath: &str) -> Option<&mut TextureInfo> {
        if self.loaded_textures.contains_key(filepath) {
            let info = self.loaded_textures.get_mut(filepath).unwrap();
            info.being_used_by += 1;
            return Some(info);
        };

        let mut texture = None;
        let vec = std::fs::read(filepath);
        if let Ok(vec) = vec {
            let texture_data = image::load_from_memory(vec.as_slice());
            if let Ok(texture_data) = texture_data {
                let mut info = TextureInfo {
                    texture: texture_data.to_rgb32f(),
                    pixels: texture_data.to_rgb8().into_raw(),
                    being_used_by: 1,
                };
                self.loaded_textures.insert(filepath.to_string(), info);
                texture = Some(self.loaded_textures.get_mut(filepath).unwrap());
            };
        };
        texture
    }

    pub fn add_material_texture_instance(&mut self, material: Material) {
        let mat_filepath_buf = self.get_texturepath_from_material(material);
        let mat_filepath = mat_filepath_buf.to_str();
        if let Some(m_path) = mat_filepath {
            self.add_texture_instance(m_path);
        };
    }

    pub fn get_texture(&self, filepath: &str) -> Option<&TextureInfo> {
        return self.loaded_textures.get(filepath);
    }

    /// Remove a texture user, if no one uses the texture, unload it
    pub fn remove_texture_instance(&mut self, filepath: &str) -> bool {
        let texture = self.loaded_textures.get_mut(filepath);
        if let Some(texture) = texture {
            texture.being_used_by = texture.being_used_by.saturating_sub(1);
            if texture.being_used_by == 0 {
                self.loaded_textures.remove(filepath);
                return true;
            };
        };
        return false;
    }

    pub fn remove_material_texture_instance(&mut self, material: Material) {
        let mat_filepath_buf = self.get_texturepath_from_material(material);
        let mat_filepath = mat_filepath_buf.to_str();
        if let Some(m_path) = mat_filepath {
            self.remove_texture_instance(m_path);
        };
    }

    pub fn get_color_from_texture_wrapped(&self, pos: IVec2, tex: &Rgb32FImage) -> Color {
        let dims = tex.dimensions();
        let c = tex.get_pixel((pos.x % dims.0 as i32) as u32, (pos.y % dims.1 as i32) as u32);
        Color {r: c.0[0] as f64, g: c.0[1] as f64, b: c.0[2] as f64, a: 1.0}
    }

    pub fn get_color_for_material(&mut self, pos: IVec2, material: Material) -> Color {
        let tex = self.get_texture_for_material(material);
        if let Some(tex) = tex {
            return self.get_color_from_texture_wrapped(pos, &tex.texture);
        };
        COLOR_EMPTY
    }

    pub fn get_texture_for_material(&self, material: Material) -> Option<&TextureInfo> {
        self.get_texture(self.get_texturepath_from_material(material).to_str().unwrap())
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
        let path = std::env::current_dir().unwrap().join("data").join("textures").join("materials").join(tex_name);
        path
    }
}