use std::sync::Arc;
use log::{error, info};
use sqlx::{MySql, MySqlPool, Pool};
use sqlx::mysql::MySqlPoolOptions;

use crate::core::configuration_manager::ConfigurationManager;

pub struct Database {
    pool: Pool<MySql>,
}

impl Database {
    pub fn new(config: Arc<ConfigurationManager>) -> Result<Self, Box<dyn std::error::Error>> {
        // Get database configuration
        let db_host = config.get_string("db.host").unwrap_or_else(|_| "localhost".to_string());
        let db_port = config.get_int("db.port").unwrap_or_else(|_| 3306);
        let db_name = config.get_string("db.name").unwrap_or_else(|_| "sulove".to_string());
        let db_user = config.get_string("db.username").unwrap_or_else(|_| "root".to_string());
        let db_pass = config.get_string("db.password").unwrap_or_else(|_| "".to_string());
        
        // Build connection string
        let connection_string = format!(
            "mysql://{}:{}@{}:{}/{}",
            db_user, db_pass, db_host, db_port, db_name
        );
        
        // Create connection pool
        let pool = MySqlPoolOptions::new()
            .max_connections(config.get_int("runtime.threads").unwrap_or(10) as u32 * 2)
            .min_connections(10)
            .connect_lazy(&connection_string)?;
        
        info!("Connected to database: {}@{}:{}/{}", db_user, db_host, db_port, db_name);
        
        Ok(Database { pool })
    }
    
    pub fn get_pool(&self) -> &Pool<MySql> {
        &self.pool
    }
    
    pub async fn test_connection(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(self.get_pool())
            .await
            .map(|_| ())
    }
}