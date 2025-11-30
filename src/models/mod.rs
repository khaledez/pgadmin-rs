// Data models module
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub name: String,
    pub size: Option<i64>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub schema: String,
    pub name: String,
    pub table_type: String,
    pub row_count: Option<i64>,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_pk: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<Option<String>>>,
    pub total_rows: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub affected_rows: Option<u64>,
    pub execution_time_ms: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistory {
    pub query: String,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
    pub error: Option<String>,
    pub row_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct TableDataParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub sort_column: Option<String>,
    pub sort_direction: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryExecuteForm {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub total_rows: i64,
    pub total_pages: u32,
}
