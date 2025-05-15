use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use log::{error, info};

use crate::habbohotel::guilds::{Guild, GuildPart, GuildPartType};

pub struct BadgeImager {
    cached_images: Arc<Mutex<HashMap<String, DynamicImage>>>,
}

impl BadgeImager {
    pub fn new() -> Self {
        let imager = BadgeImager {
            cached_images: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Load badge parts if enabled in config
        if crate::get_config().get_bool("imager.internal.enabled").unwrap_or(false) {
            if imager.reload() {
                info!("Badge Imager -> Loaded!");
            } else {
                error!("Badge Imager -> Disabled! Please check your configuration!");
            }
        }
        
        imager
    }
    
    // Create a deep copy of an image
    pub fn deep_copy(image: &DynamicImage) -> DynamicImage {
        image.clone()
    }
    
    // Recolor an image with a mask color
    pub fn recolor(image: &mut DynamicImage, mask_color: Rgba<u8>) {
        let (width, height) = image.dimensions();
        
        for x in 0..width {
            for y in 0..height {
                let pixel = image.get_pixel(x, y);
                
                // Skip transparent pixels
                if pixel[3] == 0 {
                    continue;
                }
                
                // Apply color mask
                let alpha = (pixel[3] as f32 / 255.0) * (mask_color[3] as f32 / 255.0);
                let red = (pixel[0] as f32 / 255.0) * (mask_color[0] as f32 / 255.0);
                let green = (pixel[1] as f32 / 255.0) * (mask_color[1] as f32 / 255.0);
                let blue = (pixel[2] as f32 / 255.0) * (mask_color[2] as f32 / 255.0);
                
                let new_pixel = Rgba([
                    (red * 255.0) as u8,
                    (green * 255.0) as u8,
                    (blue * 255.0) as u8,
                    (alpha * 255.0) as u8,
                ]);
                
                image.put_pixel(x, y, new_pixel);
            }
        }
    }
    
    // Convert hex color string to RGBA
    pub fn color_from_hex_string(color_str: &str) -> Rgba<u8> {
        match u32::from_str_radix(color_str, 16) {
            Ok(color) => {
                Rgba([
                    ((color >> 16) & 0xFF) as u8,
                    ((color >> 8) & 0xFF) as u8,
                    (color & 0xFF) as u8,
                    255, // Alpha
                ])
            },
            Err(_) => Rgba([255, 255, 255, 255]), // Default to white
        }
    }
    
    // Get position point for badge part
    pub fn get_point(image: &DynamicImage, image_part: &DynamicImage, position: u32) -> (u32, u32) {
        let (width, height) = image.dimensions();
        let (part_width, part_height) = image_part.dimensions();
        
        match position {
            0 => (0, 0), // Top left
            1 => ((width - part_width) / 2, 0), // Top center
            2 => (width - part_width, 0), // Top right
            3 => (0, (height / 2) - (part_height / 2)), // Middle left
            4 => ((width / 2) - (part_width / 2), (height / 2) - (part_height / 2)), // Middle center
            5 => (width - part_width, (height / 2) - (part_height / 2)), // Middle right
            6 => (0, height - part_height), // Bottom left
            7 => ((width - part_width) / 2, height - part_height), // Bottom center
            8 => (width - part_width, height - part_height), // Bottom right
            _ => (0, 0), // Default to top left
        }
    }
    
    // Reload badge parts from files
    pub fn reload(&self) -> bool {
        let badge_parts_path = crate::get_config().get_string("imager.location.badgeparts").unwrap_or_else(|_| "./badgeparts".to_string());
        let path = Path::new(&badge_parts_path);
        
        if !path.exists() {
            error!("BadgeImager output folder: {} does not exist!", badge_parts_path);
            return false;
        }
        
        let mut cached_images = self.cached_images.lock().unwrap();
        cached_images.clear();
        
        // Load badge parts from the game environment
        // This would need to be adapted to your actual guild manager implementation
        if let Some(guild_manager) = crate::get_game_environment().get_guild_manager() {
            for (part_type, parts) in guild_manager.get_guild_parts() {
                if *part_type == GuildPartType::Symbol || *part_type == GuildPartType::Base {
                    for (_, part) in parts {
                        if !part.value_a.is_empty() {
                            let file_path = path.join(format!("badgepart_{}", part.value_a.replace(".gif", ".png")));
                            match image::open(&file_path) {
                                Ok(img) => {
                                    cached_images.insert(part.value_a.clone(), img);
                                },
                                Err(_) => {
                                    info!("[Badge Imager] Missing Badge Part: {}", file_path.display());
                                }
                            }
                        }
                        
                        if !part.value_b.is_empty() {
                            let file_path = path.join(format!("badgepart_{}", part.value_b.replace(".gif", ".png")));
                            match image::open(&file_path) {
                                Ok(img) => {
                                    cached_images.insert(part.value_b.clone(), img);
                                },
                                Err(_) => {
                                    info!("[Badge Imager] Missing Badge Part: {}", file_path.display());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        true
    }
    
    // Generate badge image for a guild
    pub fn generate(&self, guild: &Guild) -> Result<(), String> {
        let badge = guild.get_badge();
        let output_path = crate::get_config().get_string("imager.location.output.badges").unwrap_or_else(|_| "./badges".to_string());
        let output_file = Path::new(&output_path).join(format!("{}.png", badge));
        
        if output_file.exists() {
            return Ok(());
        }
        
        // Parse badge code into parts
        let mut parts = vec![String::new(); 5];
        let mut count = 0;
        let mut i = 0;
        
        while i < badge.len() {
            if i > 0 && i % 7 == 0 {
                count += 1;
            }
            
            if count < parts.len() {
                if let Some(c) = badge.chars().nth(i) {
                    parts[count].push(c);
                }
            }
            
            i += 1;
        }
        
        // Create base image
        let mut image = RgbaImage::new(39, 39);
        
        // Process each badge part
        for part_code in parts {
            if part_code.is_empty() {
                continue;
            }
            
            if part_code.len() < 7 {
                continue;
            }
            
            let type_char = part_code.chars().next().unwrap_or('b');
            let id = part_code[1..4].parse::<u32>().unwrap_or(0);
            let color_id = part_code[4..6].parse::<u32>().unwrap_or(0);
            let position = part_code[6..7].parse::<u32>().unwrap_or(0);
            
            // Get guild parts from the game environment
            // This would need to be adapted to your actual guild manager implementation
            if let Some(guild_manager) = crate::get_game_environment().get_guild_manager() {
                let part_type = if type_char == 'b' { GuildPartType::Base } else { GuildPartType::Symbol };
                let part = guild_manager.get_part(part_type, id);
                let color = guild_manager.get_part(GuildPartType::BaseColor, color_id);
                
                if let Some(part) = part {
                    let cached_images = self.cached_images.lock().unwrap();
                    
                    // Process value_a (main part)
                    if !part.value_a.is_empty() {
                        if let Some(image_part) = cached_images.get(&part.value_a) {
                            let mut image_part = Self::deep_copy(image_part);
                            
                            if let Some(color) = color {
                                let color_rgba = Self::color_from_hex_string(&color.value_a);
                                Self::recolor(&mut image_part, color_rgba);
                            }
                            
                            let (x, y) = Self::get_point(&DynamicImage::ImageRgba8(image.clone()), &image_part, position);
                            
                            // Overlay the part onto the main image
                            image::imageops::overlay(&mut image, &image_part.to_rgba8(), x as i64, y as i64);
                        }
                    }
                    
                    // Process value_b (secondary part)
                    if !part.value_b.is_empty() {
                        if let Some(image_part) = cached_images.get(&part.value_b) {
                            let image_part = Self::deep_copy(image_part);
                            
                            let (x, y) = Self::get_point(&DynamicImage::ImageRgba8(image.clone()), &image_part, position);
                            
                            // Overlay the part onto the main image
                            image::imageops::overlay(&mut image, &image_part.to_rgba8(), x as i64, y as i64);
                        }
                    }
                }
            }
        }
        
        // Save the image
        match image.save(output_file) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to generate guild badge: {}. Error: {}", badge, e)),
        }
    }
    
    // Generate badge from badge code
    pub fn generate_badge(&self, badge_code: &str) -> Result<Vec<u8>, String> {
        // Create a temporary guild with the badge code
        let guild = Guild {
            id: 0,
            name: String::from("Temporary"),
            description: String::new(),
            badge: badge_code.to_string(),
            // Add other required fields with default values
        };
        
        // Generate the badge
        self.generate(&guild)?;
        
        // Read the generated file
        let output_path = crate::get_config().get_string("imager.location.output.badges").unwrap_or_else(|_| "./badges".to_string());
        let output_file = Path::new(&output_path).join(format!("{}.png", badge_code));
        
        match std::fs::read(output_file) {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("Failed to read generated badge: {}. Error: {}", badge_code, e)),
        }
    }
}