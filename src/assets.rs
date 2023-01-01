use std::{collections::HashMap, path::PathBuf};
use crate::Color;
use glam::IVec2;
use image::{Rgb32FImage, ImageBuffer};
use strum::IntoEnumIterator;

use crate::{Material, COLOR_EMPTY};


#[derive(Debug)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    //pub image: Rgb,
    pub pixels: Vec<u8>,
    pub being_used_by: usize,
}


pub struct Assets {
    // All the textures loaded in the game
    loaded_textures: HashMap<String, TextureInfo>,
    pub loaded_material_textures: HashMap<Material, TextureInfo>,
}

impl Assets {
    pub fn new() -> Self {
        let mut loaded_material_textures = HashMap::new();
        for mat in Material::iter() {
            let mat_filepath_buf = Assets::get_texturepath_from_material(mat);
            let mat_filepath = mat_filepath_buf.to_str();
            if let Some(m_path) = mat_filepath {
                //println!("Mat: {:?}   Path: {}", mat, m_path);
                loaded_material_textures.insert(mat, Assets::load_texture(m_path));
            };
        }
        Assets {
            loaded_textures: HashMap::new(),
            loaded_material_textures,
        }
    }

    /// If the texture has already been loaded, return it, else load it
    // pub fn add_texture_instance(&mut self, filepath: &str) -> Option<&mut TextureInfo> {
    //     if self.loaded_textures.contains_key(filepath) {
    //         let info = self.loaded_textures.get_mut(filepath).unwrap();
    //         info.being_used_by += 1;
    //         return Some(info);
    //     };

    //     let mut texture = Assets::load_texture(filepath);
    //     Some(&mut texture)
    // }

    pub fn load_texture(filepath: &str) -> TextureInfo {
        let mut info = TextureInfo {
            width: 32,
            height: 32,
            pixels: vec![],
            being_used_by: 0,
        };
        let vec = std::fs::read(filepath);
        if let Ok(vec) = vec {
            let texture_data = image::load_from_memory(vec.as_slice());
            if let Ok(texture_data) = texture_data {
                info.width = texture_data.width();
                info.height = texture_data.height();
                let rgb8 = texture_data.to_rgba8();
                info.pixels = rgb8.as_raw().clone();
                info.being_used_by = 1;
            };
        };
        info
    }

    // pub fn add_material_texture_instance(&mut self, material: Material) -> Option<&TextureInfo> {
    //     let mat_filepath_buf = Assets::get_texturepath_from_material(material);
    //     let mat_filepath = mat_filepath_buf.to_str();
    //     if let Some(m_path) = mat_filepath {
    //         self.add_texture_instance(m_path);
    //         return self.loaded_textures.get(m_path);
    //     };
    //     return None;
    // }

    pub fn get_texture(&self, filepath: &str) -> Option<&TextureInfo> {
        return self.loaded_textures.get(filepath);
    }

    /// Remove a texture user, if no one uses the texture, unload it
    // pub fn remove_texture_instance(&mut self, filepath: &str) -> bool {
    //     let texture = self.loaded_textures.get_mut(filepath);
    //     if let Some(texture) = texture {
    //         texture.being_used_by = texture.being_used_by.saturating_sub(1);
    //         if texture.being_used_by == 0 {
    //             self.loaded_textures.remove(filepath);
    //             return true;
    //         };
    //     };
    //     return false;
    // }

    pub fn get_color_from_texture_wrapped(&self, pos: IVec2, info: &TextureInfo) -> Color {
        let x = pos.x % info.width as i32;
        let y = pos.y % info.height as i32;
        let idx = (x + y * info.width as i32) as usize * 4;
        let c = &info.pixels[idx..idx+4];
        Color {
            r: c[0] as f64 / 255.0,
            g: c[1] as f64 / 255.0,
            b: c[2] as f64 / 255.0,
            a: c[3] as f64 / 255.0,
        }
    }

    pub fn get_color_for_material(&mut self, pos: IVec2, material: Material) -> Color {
        let tex = self.loaded_material_textures.get(&material);
        if let Some(tex) = tex {
            return self.get_color_from_texture_wrapped(pos, tex);
        };
        COLOR_EMPTY
    }

    fn get_texturepath_from_material(material: Material) -> PathBuf {
        let tex_name = match material {
            Material::Dirt => "dirt.png",
            Material::Sand => "sand.png",
            Material::Water => "water.png",
            Material::Rock => "rock.png",
            Material::Smoke => "smoke.png",
            Material::Wood => "wood.png",
            _ => "",
        };
        let mut path = std::env::current_dir().unwrap();
        path.push("data");
        path.push("textures");
        if tex_name == "" {
            path = path.join("debug_color_02.png");
        } else {
            path.push("materials");
            //path.push(tex_name);
            path = path.join("materials").with_file_name(tex_name);
        };
        path
    }
}