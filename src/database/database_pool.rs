use std::sync::Arc;
use sqlx::{MySql, Pool};

pub struct DatabasePool {
    pool: Arc<Pool<MySql>>,
}

impl DatabasePool {
    pub fn new(pool: Arc<Pool<MySql>>) -> Self {
        DatabasePool { pool }
    }
    
    pub fn get_pool(&self) -> Arc<Pool<MySql>> {
        self.pool.clone()
    }
}