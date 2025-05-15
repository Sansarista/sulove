use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, RwLock};

use log::{error, info};
use sqlx::MySqlPool;

#[derive(Debug)]
pub struct ConfigurationManager {
    config_path: String,
    values: RwLock<HashMap<String, String>>,
    pub loaded: bool,
}

impl ConfigurationManager {
    pub fn new(config_path: &str) -> Result<Self, io::Error> {
        let mut manager = ConfigurationManager {
            config_path: config_path.to_string(),
            values: RwLock::new(HashMap::new()),
            loaded: false,
        };
        
        manager.load_from_file()?;
        
        Ok(manager)
    }
    
    fn load_from_file(&self) -> Result<(), io::Error> {
        let file = File::open(&self.config_path)?;
        let reader = BufReader::new(file);
        
        let mut values = self.values.write().unwrap();
        
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }
            
            // Parse key=value pairs
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                values.insert(key, value);
            }
        }
        
        info!("Loaded {} settings from {}", values.len(), self.config_path);
        
        Ok(())
    }
    
    pub fn load_from_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.loaded {
            return Ok(());
        }
        
        // Get database connection from the main database pool
        let pool = crate::get_database().get_pool();
        
        // Load settings from database
        let rows = sqlx::query!("SELECT `key`, `value` FROM emulator_settings")
            .fetch_all(pool)
            .map_err(|e| {
                error!("Failed to load settings from database: {}", e);
                e
            })?;
        
        let mut values = self.values.write().unwrap();
        
        for row in rows {
            values.insert(row.key, row.value);
        }
        
        info!("Loaded {} settings from database", rows.len());
        
        Ok(())
    }
    
    pub fn get_string(&self, key: &str) -> Result<String, String> {
        let values = self.values.read().unwrap();
        
        values.get(key)
            .cloned()
            .ok_or_else(|| format!("Configuration key not found: {}", key))
    }
    
    pub fn get_int(&self, key: &str) -> Result<i32, String> {
        let value = self.get_string(key)?;
        
        value.parse::<i32>()
            .map_err(|e| format!("Failed to parse '{}' as integer: {}", value, e))
    }
    
    pub fn get_bool(&self, key: &str) -> Result<bool, String> {
        let value = self.get_string(key)?;
        
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(format!("Failed to parse '{}' as boolean", value)),
        }
    }
    
    pub fn set(&self, key: &str, value: &str) -> Result<(), String> {
        let mut values = self.values.write().unwrap();
        values.insert(key.to_string(), value.to_string());
        Ok(())
    }
}