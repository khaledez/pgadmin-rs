mod common;

use common::{cleanup_test_data, create_test_pool, seed_test_data};

#[tokio::test]
async fn test_database_connection() {
    let pool = create_test_pool().await;
    let result = sqlx::query("SELECT 1 as num").fetch_one(&pool).await;

    assert!(result.is_ok(), "Failed to connect to test database");
}

#[tokio::test]
async fn test_list_schemas() {
    let pool = create_test_pool().await;

    let schemas: Vec<(String,)> = sqlx::query_as(
        "SELECT schema_name FROM information_schema.schemata
         WHERE schema_name NOT IN ('pg_catalog', 'information_schema')
         ORDER BY schema_name",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to list schemas");

    assert!(!schemas.is_empty(), "Should have at least one schema");
    assert!(
        schemas.iter().any(|(name,)| name == "public"),
        "Should have public schema"
    );
}

#[tokio::test]
async fn test_create_and_list_table() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;

    // Create test table
    let result = sqlx::query(
        "CREATE TABLE test_table (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "Failed to create test table");

    // List tables
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT table_name FROM information_schema.tables
         WHERE table_schema = 'public' AND table_name = 'test_table'",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to list tables");

    assert_eq!(tables.len(), 1, "Should have exactly one test_table");
    assert_eq!(tables[0].0, "test_table", "Table name should be test_table");

    // Clean up
    let _ = cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_table_columns() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;

    let columns: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_schema = 'public' AND table_name = 'users'
         ORDER BY ordinal_position",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to get columns");

    assert!(!columns.is_empty(), "Should have columns");
    assert!(
        columns.iter().any(|(name, _, _)| name == "id"),
        "Should have id column"
    );
    assert!(
        columns.iter().any(|(name, _, _)| name == "username"),
        "Should have username column"
    );

    // Clean up
    let _ = cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_query_data() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;

    let rows: Vec<(i32, String)> = sqlx::query_as("SELECT id, username FROM users ORDER BY id")
        .fetch_all(&pool)
        .await
        .expect("Failed to query data");

    assert_eq!(rows.len(), 3, "Should have 3 users");
    assert_eq!(rows[0].1, "alice", "First user should be alice");

    // Clean up
    let _ = cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_row_count() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count rows");

    assert_eq!(count, 3, "Should have 3 users");

    // Clean up
    let _ = cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_table_size() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;

    let size_result: Option<(String,)> =
        sqlx::query_as("SELECT pg_size_pretty(pg_total_relation_size('users')) as size")
            .fetch_optional(&pool)
            .await
            .expect("Failed to fetch table size");

    if let Some((size,)) = size_result {
        assert!(!size.is_empty(), "Size should be non-empty");
    }

    // Clean up
    let _ = cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_insert_and_retrieve() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;

    // Insert a new user
    sqlx::query("INSERT INTO users (username, email) VALUES ('dave', 'dave@example.com')")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    // Retrieve the user
    let user: Option<(String, String)> =
        sqlx::query_as("SELECT username, email FROM users WHERE username = 'dave'")
            .fetch_optional(&pool)
            .await
            .expect("Failed to fetch user");

    assert!(user.is_some(), "User should exist");
    let (username, email) = user.unwrap();
    assert_eq!(username, "dave", "Username should be dave");
    assert_eq!(email, "dave@example.com", "Email should match");

    // Clean up
    let _ = cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_data() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;

    // Update user
    let result =
        sqlx::query("UPDATE users SET email = 'newemail@example.com' WHERE username = 'alice'")
            .execute(&pool)
            .await;

    assert!(result.is_ok(), "Update should succeed");

    // Verify update
    let email: Option<(String,)> =
        sqlx::query_as("SELECT email FROM users WHERE username = 'alice'")
            .fetch_optional(&pool)
            .await
            .expect("Failed to fetch email");

    assert_eq!(
        email.unwrap().0,
        "newemail@example.com",
        "Email should be updated"
    );

    // Clean up
    let _ = cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_data() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;

    // Delete user
    let result = sqlx::query("DELETE FROM users WHERE username = 'alice'")
        .execute(&pool)
        .await;

    assert!(result.is_ok(), "Delete should succeed");

    // Verify deletion
    let user: Option<(String,)> =
        sqlx::query_as("SELECT username FROM users WHERE username = 'alice'")
            .fetch_optional(&pool)
            .await
            .expect("Failed to check user");

    assert!(user.is_none(), "User should be deleted");

    // Clean up
    let _ = cleanup_test_data(&pool).await;
}
