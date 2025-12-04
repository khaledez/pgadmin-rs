use serde::{Deserialize, Serialize};
/// Schema Operations Service
///
/// Handles DDL operations for creating and dropping database objects:
/// - Tables
/// - Views
/// - Indexes
/// - Sequences
/// - Functions
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTableRequest {
    pub table_name: String,
    pub schema: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropObjectRequest {
    pub object_name: String,
    pub schema: String,
    pub object_type: String, // TABLE, VIEW, INDEX, FUNCTION, SEQUENCE
    pub cascade: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIndexRequest {
    pub index_name: String,
    pub schema: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

pub struct SchemaOpsService;

impl SchemaOpsService {
    /// Create a new table
    pub async fn create_table(pool: &PgPool, req: &CreateTableRequest) -> Result<String, String> {
        if req.columns.is_empty() {
            return Err("At least one column is required".to_string());
        }

        // Validate schema and table names
        Self::validate_identifier(&req.schema)?;
        Self::validate_identifier(&req.table_name)?;

        let mut sql = format!(
            "CREATE TABLE IF NOT EXISTS \"{}\".\"{}\" (",
            req.schema, req.table_name
        );

        let column_defs: Result<Vec<String>, String> = req
            .columns
            .iter()
            .map(|col| {
                Self::validate_identifier(&col.name)?;
                let mut def = format!("\n  \"{}\" {}", col.name, col.data_type);

                if !col.nullable {
                    def.push_str(" NOT NULL");
                }

                if let Some(default) = &col.default {
                    def.push_str(&format!(" DEFAULT {}", default));
                }

                Ok(def)
            })
            .collect();

        let column_defs = column_defs?;
        sql.push_str(&column_defs.join(","));
        sql.push_str("\n)");

        // Execute the CREATE TABLE statement
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to create table: {}", e))?;

        Ok(format!(
            "Table {}.{} created successfully",
            req.schema, req.table_name
        ))
    }

    /// Drop a table, view, or other object
    pub async fn drop_object(pool: &PgPool, req: &DropObjectRequest) -> Result<String, String> {
        Self::validate_identifier(&req.schema)?;
        Self::validate_identifier(&req.object_name)?;

        let object_type = match req.object_type.to_uppercase().as_str() {
            "TABLE" => "TABLE",
            "VIEW" => "VIEW",
            "INDEX" => "INDEX",
            "SEQUENCE" => "SEQUENCE",
            "FUNCTION" => "FUNCTION",
            _ => return Err(format!("Unsupported object type: {}", req.object_type)),
        };

        let cascade = if req.cascade { "CASCADE" } else { "RESTRICT" };

        let sql = format!(
            "DROP {} IF EXISTS \"{}\".\"{}\" {}",
            object_type, req.schema, req.object_name, cascade
        );

        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to drop {}: {}", object_type, e))?;

        Ok(format!(
            "{} {}.{} dropped successfully",
            object_type, req.schema, req.object_name
        ))
    }

    /// Create an index
    pub async fn create_index(pool: &PgPool, req: &CreateIndexRequest) -> Result<String, String> {
        Self::validate_identifier(&req.schema)?;
        Self::validate_identifier(&req.index_name)?;
        Self::validate_identifier(&req.table_name)?;

        if req.columns.is_empty() {
            return Err("At least one column is required for an index".to_string());
        }

        // Validate column names
        for col in &req.columns {
            Self::validate_identifier(col)?;
        }

        let unique = if req.unique { "UNIQUE " } else { "" };
        let columns = req
            .columns
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "CREATE {}INDEX IF NOT EXISTS \"{}\" ON \"{}\".\"{}\" ({})",
            unique, req.index_name, req.schema, req.table_name, columns
        );

        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to create index: {}", e))?;

        Ok(format!("Index {} created successfully", req.index_name))
    }

    /// Get list of tables in a schema
    pub async fn list_tables(pool: &PgPool, schema: &str) -> Result<Vec<TableInfo>, String> {
        Self::validate_identifier(schema)?;

        let query = r#"
            SELECT 
                table_name,
                table_type
            FROM information_schema.tables
            WHERE table_schema = $1
            ORDER BY table_name
        "#;

        let rows = sqlx::query_as::<_, TableInfo>(query)
            .bind(schema)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to list tables: {}", e))?;

        Ok(rows)
    }

    /// Get table column definitions
    pub async fn get_table_columns(
        pool: &PgPool,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnDef>, String> {
        Self::validate_identifier(schema)?;
        Self::validate_identifier(table)?;

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

        let rows = sqlx::query_as::<_, ColumnDef>(query)
            .bind(schema)
            .bind(table)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get table columns: {}", e))?;

        Ok(rows)
    }

    /// Validate identifier (table/schema/column names)
    /// Prevents SQL injection by checking for valid PostgreSQL identifiers
    fn validate_identifier(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Identifier cannot be empty".to_string());
        }

        if name.len() > 63 {
            return Err("Identifier cannot be longer than 63 characters".to_string());
        }

        // Allow alphanumeric, underscores, and some special chars
        // PostgreSQL allows: a-z, A-Z, 0-9, _ and non-ASCII
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!(
                "Invalid identifier '{}': only alphanumeric and underscore allowed",
                name
            ));
        }

        // Cannot start with a digit
        if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Err(format!(
                "Invalid identifier '{}': cannot start with a digit",
                name
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TableInfo {
    pub table_name: String,
    pub table_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ColumnDef {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: String,
    pub column_default: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_identifier_valid() {
        assert!(SchemaOpsService::validate_identifier("users").is_ok());
        assert!(SchemaOpsService::validate_identifier("user_id").is_ok());
        assert!(SchemaOpsService::validate_identifier("_private").is_ok());
        assert!(SchemaOpsService::validate_identifier("table123").is_ok());
    }

    #[test]
    fn test_validate_identifier_invalid() {
        assert!(SchemaOpsService::validate_identifier("").is_err());
        assert!(SchemaOpsService::validate_identifier("123abc").is_err());
        assert!(SchemaOpsService::validate_identifier("user-table").is_err());
        assert!(SchemaOpsService::validate_identifier("user.table").is_err());
        assert!(SchemaOpsService::validate_identifier("user table").is_err());
    }

    #[test]
    fn test_validate_identifier_length() {
        let long_name = "a".repeat(64);
        assert!(SchemaOpsService::validate_identifier(&long_name).is_err());

        let valid_name = "a".repeat(63);
        assert!(SchemaOpsService::validate_identifier(&valid_name).is_ok());
    }
}
