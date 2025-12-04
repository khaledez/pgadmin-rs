// Database service module
// Handles database-level operations (list, create, drop databases)

use crate::models::Database;
use sqlx::{Pool, Postgres, Row};

/// Lists all databases on the PostgreSQL server
pub async fn list_databases(pool: &Pool<Postgres>) -> Result<Vec<Database>, sqlx::Error> {
    let query = r#"
        SELECT
            d.datname as name,
            pg_catalog.pg_get_userbyid(d.datdba) as owner,
            pg_catalog.pg_size_pretty(pg_catalog.pg_database_size(d.datname)) as size,
            pg_catalog.pg_encoding_to_char(d.encoding) as encoding
        FROM pg_catalog.pg_database d
        WHERE d.datistemplate = false
        ORDER BY d.datname
    "#;

    let rows = sqlx::query(query).fetch_all(pool).await?;

    let databases = rows
        .iter()
        .map(|row| Database {
            name: row.get("name"),
            owner: row.get("owner"),
            size: row.get("size"),
            encoding: row.get("encoding"),
        })
        .collect();

    Ok(databases)
}

/// Creates a new database
pub async fn create_database(
    pool: &Pool<Postgres>,
    db_name: &str,
    owner: Option<&str>,
) -> Result<(), sqlx::Error> {
    // Validate database name (basic protection)
    validate_db_name(db_name)?;

    let query = if let Some(owner_name) = owner {
        validate_db_name(owner_name)?;
        format!("CREATE DATABASE \"{}\" OWNER \"{}\"", db_name, owner_name)
    } else {
        format!("CREATE DATABASE \"{}\"", db_name)
    };

    sqlx::query(&query).execute(pool).await?;
    Ok(())
}

/// Drops a database
pub async fn drop_database(pool: &Pool<Postgres>, db_name: &str) -> Result<(), sqlx::Error> {
    validate_db_name(db_name)?;

    let query = format!("DROP DATABASE \"{}\"", db_name);
    sqlx::query(&query).execute(pool).await?;
    Ok(())
}

/// Gets information about a specific database
pub async fn get_database_info(
    pool: &Pool<Postgres>,
    db_name: &str,
) -> Result<Database, sqlx::Error> {
    validate_db_name(db_name)?;

    let query = r#"
        SELECT
            d.datname as name,
            pg_catalog.pg_get_userbyid(d.datdba) as owner,
            pg_catalog.pg_size_pretty(pg_catalog.pg_database_size(d.datname)) as size,
            pg_catalog.pg_encoding_to_char(d.encoding) as encoding
        FROM pg_catalog.pg_database d
        WHERE d.datname = $1
    "#;

    let row = sqlx::query(query).bind(db_name).fetch_one(pool).await?;

    Ok(Database {
        name: row.get("name"),
        owner: row.get("owner"),
        size: row.get("size"),
        encoding: row.get("encoding"),
    })
}

/// Validates database name to prevent SQL injection
fn validate_db_name(name: &str) -> Result<(), sqlx::Error> {
    if name.is_empty() {
        return Err(sqlx::Error::Protocol(
            "Database name cannot be empty".into(),
        ));
    }

    // Check for invalid characters
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(sqlx::Error::Protocol(
            "Database name contains invalid characters".into(),
        ));
    }

    if name.len() > 63 {
        return Err(sqlx::Error::Protocol(
            "Database name is too long (max 63 chars)".into(),
        ));
    }

    Ok(())
}
