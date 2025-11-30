// Schema service module
// Handles database schema inspection and metadata retrieval

use sqlx::{Pool, Postgres, Row};
use crate::models::{Database, Schema, TableInfo, ColumnInfo};

/// Lists all databases
pub async fn list_databases(pool: &Pool<Postgres>) -> Result<Vec<Database>, sqlx::Error> {
    let query = r#"
        SELECT
            datname as name,
            pg_database_size(datname) as size,
            rolname as owner
        FROM pg_database
        JOIN pg_authid ON pg_database.datdba = pg_authid.oid
        WHERE datistemplate = false
        ORDER BY datname
    "#;

    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await?;

    let databases = rows.iter()
        .map(|row| Database {
            name: row.get("name"),
            size: row.get("size"),
            owner: row.get("owner"),
        })
        .collect();

    Ok(databases)
}

/// Lists all schemas in the current database
pub async fn list_schemas(pool: &Pool<Postgres>) -> Result<Vec<Schema>, sqlx::Error> {
    let query = r#"
        SELECT
            schema_name as name,
            schema_owner as owner
        FROM information_schema.schemata
        WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        ORDER BY schema_name
    "#;

    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await?;

    let schemas = rows.iter()
        .map(|row| Schema {
            name: row.get("name"),
            owner: row.get("owner"),
        })
        .collect();

    Ok(schemas)
}

/// Lists all tables in a specific schema
pub async fn list_tables(
    pool: &Pool<Postgres>,
    schema: &str,
) -> Result<Vec<TableInfo>, sqlx::Error> {
    let query = r#"
        SELECT
            t.table_schema as schema,
            t.table_name as name,
            t.table_type,
            (SELECT count(*) FROM information_schema.columns c 
             WHERE c.table_schema = t.table_schema AND c.table_name = t.table_name) as col_count
        FROM information_schema.tables t
        WHERE t.table_schema = $1 AND t.table_type IN ('BASE TABLE', 'VIEW')
        ORDER BY t.table_name
    "#;

    let rows = sqlx::query(query)
        .bind(schema)
        .fetch_all(pool)
        .await?;

    let tables = rows.iter()
        .map(|row| TableInfo {
            schema: row.get("schema"),
            name: row.get("name"),
            table_type: row.get("table_type"),
            row_count: None,
            size: None,
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
            c.column_name as name,
            c.data_type,
            c.is_nullable,
            c.column_default as "default",
            CASE WHEN tc.constraint_type = 'PRIMARY KEY' THEN true ELSE false END as is_pk
        FROM information_schema.columns c
        LEFT JOIN information_schema.key_column_usage kcu
            ON c.table_schema = kcu.table_schema
            AND c.table_name = kcu.table_name
            AND c.column_name = kcu.column_name
        LEFT JOIN information_schema.table_constraints tc
            ON kcu.table_schema = tc.table_schema
            AND kcu.table_name = tc.table_name
            AND kcu.constraint_name = tc.constraint_name
            AND tc.constraint_type = 'PRIMARY KEY'
        WHERE c.table_schema = $1 AND c.table_name = $2
        ORDER BY c.ordinal_position
    "#;

    let rows = sqlx::query(query)
        .bind(schema)
        .bind(table)
        .fetch_all(pool)
        .await?;

    let columns = rows.iter()
        .map(|row| ColumnInfo {
            name: row.get("name"),
            data_type: row.get("data_type"),
            is_nullable: row.get::<String, _>("is_nullable") == "YES",
            is_pk: row.get::<bool, _>("is_pk"),
            default: row.get("default"),
        })
        .collect();

    Ok(columns)
}

/// Gets row count for a table
pub async fn get_table_row_count(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
) -> Result<i64, sqlx::Error> {
    let query = format!("SELECT count(*) as count FROM \"{}\".\"{}\"", schema, table);
    
    let count: (i64,) = sqlx::query_as(&query)
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}

/// Gets table size
pub async fn get_table_size(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
) -> Result<i64, sqlx::Error> {
    let query = format!(
        "SELECT pg_total_relation_size('\"{}\".\"{}\"%')",
        schema, table
    );
    
    let size: (Option<i64>,) = sqlx::query_as(&query)
        .fetch_one(pool)
        .await?;

    Ok(size.0.unwrap_or(0))
}

/// Gets detailed information about a table
pub async fn get_table_info(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
) -> Result<TableInfo, sqlx::Error> {
    let query = r#"
        SELECT
            t.table_schema as schema,
            t.table_name as name,
            t.table_type,
            (SELECT count(*) FROM information_schema.columns c 
             WHERE c.table_schema = t.table_schema AND c.table_name = t.table_name) as col_count
        FROM information_schema.tables t
        WHERE t.table_schema = $1 AND t.table_name = $2
    "#;

    let row = sqlx::query(query)
        .bind(schema)
        .bind(table)
        .fetch_one(pool)
        .await?;

    let row_count = get_table_row_count(pool, schema, table).await.ok();
    let size = get_table_size(pool, schema, table).await.ok();

    Ok(TableInfo {
        schema: row.get("schema"),
        name: row.get("name"),
        table_type: row.get("table_type"),
        row_count,
        size,
    })
}

/// Gets data from a table with pagination
pub async fn get_table_data(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
    page: u32,
    page_size: u32,
) -> Result<(Vec<Vec<Option<String>>>, i64), sqlx::Error> {
    let offset = (page - 1) * page_size;
    
    // Get total row count
    let count_query = format!("SELECT count(*) FROM \"{}\".\"{}\"", schema, table);
    let total_rows: (i64,) = sqlx::query_as(&count_query)
        .fetch_one(pool)
        .await?;

    // Get paginated data
    let data_query = format!(
        "SELECT * FROM \"{}\".\"{}\" LIMIT {} OFFSET {}",
        schema, table, page_size, offset
    );
    
    let rows = sqlx::query(&data_query)
        .fetch_all(pool)
        .await?;

    let data = rows.iter()
        .map(|row| {
            (0..row.len())
                .map(|i| {
                    row.try_get::<String, _>(i)
                        .or_else(|_| row.try_get::<i32, _>(i).map(|v| v.to_string()))
                        .or_else(|_| row.try_get::<i64, _>(i).map(|v| v.to_string()))
                        .or_else(|_| row.try_get::<f64, _>(i).map(|v| v.to_string()))
                        .or_else(|_| row.try_get::<bool, _>(i).map(|v| v.to_string()))
                        .ok()
                })
                .collect()
        })
        .collect();

    Ok((data, total_rows.0))
}

/// Quotes a PostgreSQL identifier to make it safe for use in queries
pub fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace("\"", "\"\""))
}
