use crate::textures::Texture;
use crate::color::Color;
use std::sync::Arc;

// Structure that contains textures
pub struct TextureManager {
    textures: Vec<Arc<Texture>>, // Container for all textures
}

impl TextureManager {
    pub fn new() -> Self {
        TextureManager {
            textures: Vec::new(),
        }
    }

    // Add a texture to the container and return the index
    pub fn load_texture(&mut self, path: &str) -> usize {
        let texture = Arc::new(Texture::new(path));
        self.textures.push(texture);
        self.textures.len() - 1 // Returns the texture index
    }

    // Get a reference to the texture by index
    pub fn get_texture(&self, index: usize) -> &Arc<Texture> {
        &self.textures[index]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 2],
    pub reflectivity: f32,
    pub transparency: f32,
    pub refraction_index: f32,
    pub texture_index: Option<usize>, // Index of texture in TextureManager
    pub emissive_color: Option<Color>,
    pub emissive_intensity: f32,
}

impl Material {
    // Constructor for materials without texture
    pub fn new(
        diffuse: Color, 
        specular: f32, 
        albedo: [f32; 2],
        reflectivity: f32,
        transparency: f32,
        refraction_index: f32,
    ) -> Self {
        Material {
            diffuse, 
            specular, 
            albedo, 
            reflectivity,
            transparency, 
            refraction_index,
            texture_index: None,
            emissive_color: None,
            emissive_intensity: 0.0,
        }
    }

    // Constructor for materials with texture
    pub fn new_with_texture(
        texture_index: usize,
        specular: f32,
        albedo: [f32; 2],
        refraction_index: f32,
    ) -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular,
            albedo,
            reflectivity: 0.0,
            transparency: 0.0,
            refraction_index,
            texture_index: Some(texture_index),
            emissive_color: None,
            emissive_intensity: 0.0,
        }
    }

    // Constructor with emissive color
    pub fn new_with_emission(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 2],
        reflectivity: f32,
        transparency: f32,
        refraction_index: f32,
        emissive_color: Option<Color>,
        emissive_intensity: f32,
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            reflectivity,
            transparency,
            refraction_index,
            texture_index: None,
            emissive_color,
            emissive_intensity,
        }
    }

    pub fn is_emissive(&self) -> bool {
        self.emissive_intensity > 0.0
    }

    pub fn get_emission(&self) -> Color {
        if let Some(color) = self.emissive_color {
            color * self.emissive_intensity
        } else {
            Color::black()
        }
    }

    pub fn get_diffuse_color(&self, u: f32, v: f32, texture_manager: &TextureManager) -> Color {
        if let Some(texture_index) = self.texture_index {
            let texture = texture_manager.get_texture(texture_index);
            let x = (u * (texture.width as f32 - 1.0)) as usize;
            let y = ((1.0 - v) * (texture.height as f32 - 1.0)) as usize;
            texture.get_color(x, y)
        } else {
            self.diffuse
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: [0.0, 0.0],
            reflectivity: 0.0,
            transparency: 0.0,
            refraction_index: 0.0,
            texture_index: None,
            emissive_color: None,
            emissive_intensity: 0.0,
        }
    }
}