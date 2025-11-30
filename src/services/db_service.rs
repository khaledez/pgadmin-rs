// Database service module
// Handles database connection management and pooling

use sqlx::{Pool, Postgres};
use crate::config::Config;

/// Creates and returns a PostgreSQL connection pool
pub async fn create_pool(config: &Config) -> Result<Pool<Postgres>, sqlx::Error> {
    let database_url = config.database_url();

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

/// Tests the database connection
pub async fn test_connection(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;

    Ok(())
}
