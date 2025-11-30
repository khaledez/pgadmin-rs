// Schema service module
// Handles database schema inspection and metadata retrieval

use sqlx::{Pool, Postgres, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub schema: String,
    pub table_name: String,
    pub table_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
}

/// Lists all tables in the database
pub async fn list_tables(pool: &Pool<Postgres>) -> Result<Vec<TableInfo>, sqlx::Error> {
    let query = r#"
        SELECT
            table_schema as schema,
            table_name,
            table_type
        FROM information_schema.tables
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY table_schema, table_name
    "#;

    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await?;

    let tables = rows.iter()
        .map(|row| TableInfo {
            schema: row.get("schema"),
            table_name: row.get("table_name"),
            table_type: row.get("table_type"),
        })
        .collect();

    Ok(tables)
}

/// Gets column information for a specific table
pub async fn get_table_columns(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, sqlx::Error> {
    let query = r#"
        SELECT
            column_name,
            data_type,
            is_nullable,
            column_default
        FROM information_schema.columns
        WHERE table_schema = $1 AND table_name = $2
        ORDER BY ordinal_position
    "#;

    let rows = sqlx::query(query)
        .bind(schema)
        .bind(table)
        .fetch_all(pool)
        .await?;

    let columns = rows.iter()
        .map(|row| ColumnInfo {
            column_name: row.get("column_name"),
            data_type: row.get("data_type"),
            is_nullable: row.get::<String, _>("is_nullable") == "YES",
            column_default: row.get("column_default"),
        })
        .collect();

    Ok(columns)
}
