use sqlx::{postgres::PgConnectOptions, PgPool};
use std::str::FromStr;
use std::sync::Mutex;

// Global mutex to serialize integration tests
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Get the test mutex to serialize tests
pub fn get_test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_MUTEX.lock().unwrap()
}

/// Create a test database pool
pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:very_secr3t@localhost:5432/pgadmin_test".to_string()
    });

    let options = PgConnectOptions::from_str(&database_url).expect("Failed to parse database URL");

    PgPool::connect_with(options)
        .await
        .expect("Failed to create database pool")
}

/// Clean up test data after tests
pub async fn cleanup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Drop tables and their associated types to avoid conflicts
    sqlx::query("DROP TABLE IF EXISTS test_table CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TYPE IF EXISTS test_table CASCADE")
        .execute(pool)
        .await
        .ok(); // Ignore error if type doesn't exist

    sqlx::query("DROP TABLE IF EXISTS users CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TYPE IF EXISTS users CASCADE")
        .execute(pool)
        .await
        .ok(); // Ignore error if type doesn't exist

    sqlx::query("DROP TABLE IF EXISTS posts CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TYPE IF EXISTS posts CASCADE")
        .execute(pool)
        .await
        .ok(); // Ignore error if type doesn't exist

    Ok(())
}

/// Seed test data for integration tests
pub async fn seed_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(50) NOT NULL UNIQUE,
            email VARCHAR(100) NOT NULL UNIQUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await?;

    // Clear existing data to ensure clean state
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO users (username, email) VALUES
         ('alice', 'alice@example.com'),
         ('bob', 'bob@example.com'),
         ('charlie', 'charlie@example.com')",
    )
    .execute(pool)
    .await?;

    Ok(())
}
