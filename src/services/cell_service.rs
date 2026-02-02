use sqlx::{Pool, Postgres, Row};

/// Update a single cell value in a table
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `schema` - Schema name
/// * `table` - Table name
/// * `pk_column` - Primary key column name
/// * `pk_value` - Primary key value (as string)
/// * `column` - Column to update
/// * `value` - New value (None for NULL)
pub async fn update_cell(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
    pk_column: &str,
    pk_value: &str,
    column: &str,
    value: Option<&str>,
) -> Result<(), sqlx::Error> {
    // Build the UPDATE query
    // Using quoted identifiers to handle special characters
    let query = match value {
        Some(_) => format!(
            r#"UPDATE "{}"."{}" SET "{}" = $1 WHERE "{}" = $2"#,
            schema, table, column, pk_column
        ),
        None => format!(
            r#"UPDATE "{}"."{}" SET "{}" = NULL WHERE "{}" = $1"#,
            schema, table, column, pk_column
        ),
    };

    match value {
        Some(v) => {
            sqlx::query(&query)
                .bind(v)
                .bind(pk_value)
                .execute(pool)
                .await?;
        }
        None => {
            sqlx::query(&query)
                .bind(pk_value)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}

/// Get the primary key column for a table
pub async fn get_primary_key_column(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
) -> Result<Option<String>, sqlx::Error> {
    // Use pg_catalog for more reliable PK detection
    let query = r#"
        SELECT a.attname as column_name
        FROM pg_index i
        JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
        JOIN pg_class c ON c.oid = i.indrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE i.indisprimary
            AND n.nspname = $1
            AND c.relname = $2
        ORDER BY array_position(i.indkey, a.attnum)
        LIMIT 1
    "#;

    let result = sqlx::query(query)
        .bind(schema)
        .bind(table)
        .fetch_optional(pool)
        .await?;

    Ok(result.map(|row| row.get("column_name")))
}

/// Get a single cell value
pub async fn get_cell_value(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
    pk_column: &str,
    pk_value: &str,
    column: &str,
) -> Result<Option<String>, sqlx::Error> {
    let query = format!(
        r#"SELECT "{}"::text as value FROM "{}"."{}" WHERE "{}" = $1"#,
        column, schema, table, pk_column
    );

    let result = sqlx::query(&query)
        .bind(pk_value)
        .fetch_optional(pool)
        .await?;

    Ok(result.and_then(|row| row.try_get::<Option<String>, _>("value").ok().flatten()))
}

/// Insert a new row with default values
pub async fn insert_row(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
) -> Result<String, sqlx::Error> {
    // Get the primary key column
    let pk_column = get_primary_key_column(pool, schema, table)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    // Insert a row with DEFAULT values and return the PK
    let query = format!(
        r#"INSERT INTO "{}"."{}" DEFAULT VALUES RETURNING "{}"::text"#,
        schema, table, pk_column
    );

    let result = sqlx::query(&query).fetch_one(pool).await?;
    let pk_value: String = result.try_get(0)?;

    Ok(pk_value)
}

/// Delete a row by primary key
pub async fn delete_row(
    pool: &Pool<Postgres>,
    schema: &str,
    table: &str,
    pk_column: &str,
    pk_value: &str,
) -> Result<u64, sqlx::Error> {
    let query = format!(
        r#"DELETE FROM "{}"."{}" WHERE "{}" = $1"#,
        schema, table, pk_column
    );

    let result = sqlx::query(&query)
        .bind(pk_value)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cell_service_compiles() {
        // Basic compile test
    }
}
