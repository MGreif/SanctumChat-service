use std::fmt::Debug;

use diesel::{
    r2d2::{self, Pool},
    PgConnection,
};

use crate::config::EnvConfig;

pub trait IConnectionManager: Debug + Send + 'static {
    fn get(&self) -> Result<r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>, String>;
    fn new(env: EnvConfig) -> Self;
}

fn get_connection_pool(env_config: EnvConfig) -> Pool<r2d2::ConnectionManager<PgConnection>> {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(env_config.DATABASE_URL);
    let pool = Pool::new(manager).expect("Failed to create connection pool");
    pool
}

#[derive(Debug, Clone)]
pub struct ConnectionManager {
    pool: Pool<r2d2::ConnectionManager<PgConnection>>,
}

impl IConnectionManager for ConnectionManager {
    fn get(&self) -> Result<r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>, String> {
        match self.pool.get() {
            Ok(pool) => Ok(pool),
            Err(err) => Err(err.to_string()),
        }
    }
    fn new(env: EnvConfig) -> Self {
        let pool = get_connection_pool(env);
        ConnectionManager { pool }
    }
}
