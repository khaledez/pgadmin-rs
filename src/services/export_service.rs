/// Export Service
///
/// Handles exporting query results and table data in multiple formats:
/// - CSV (comma-separated values)
/// - JSON (JavaScript Object Notation)
/// - SQL (INSERT statements)
use crate::models::QueryResult;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    CSV,
    JSON,
    SQL,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(ExportFormat::CSV),
            "json" => Some(ExportFormat::JSON),
            "sql" => Some(ExportFormat::SQL),
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::CSV => "csv",
            ExportFormat::JSON => "json",
            ExportFormat::SQL => "sql",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            ExportFormat::CSV => "text/csv; charset=utf-8",
            ExportFormat::JSON => "application/json; charset=utf-8",
            ExportFormat::SQL => "text/plain; charset=utf-8",
        }
    }
}

pub struct ExportService;

impl ExportService {
    /// Export query results to the specified format
    pub fn export(result: &QueryResult, format: ExportFormat) -> Result<String, String> {
        match format {
            ExportFormat::CSV => Self::export_csv(result),
            ExportFormat::JSON => Self::export_json(result),
            ExportFormat::SQL => Self::export_sql(result),
        }
    }

    /// Export as CSV format
    fn export_csv(result: &QueryResult) -> Result<String, String> {
        let mut csv = String::new();

        // Header row
        csv.push_str(&result.columns.join(","));
        csv.push('\n');

        // Data rows
        for row in &result.rows {
            let values: Vec<String> = row.iter().map(|v| Self::csv_escape(v)).collect();
            csv.push_str(&values.join(","));
            csv.push('\n');
        }

        Ok(csv)
    }

    /// Export as JSON format
    fn export_json(result: &QueryResult) -> Result<String, String> {
        let mut data = Vec::new();

        for row in &result.rows {
            let mut obj = serde_json::Map::new();
            for (i, col) in result.columns.iter().enumerate() {
                if i < row.len() {
                    obj.insert(col.clone(), row[i].clone());
                }
            }
            data.push(Value::Object(obj));
        }

        serde_json::to_string_pretty(&serde_json::json!({
            "columns": result.columns,
            "row_count": result.row_count,
            "execution_time_ms": result.execution_time_ms,
            "data": data
        }))
        .map_err(|e| format!("JSON serialization failed: {}", e))
    }

    /// Export as SQL INSERT statements
    fn export_sql(result: &QueryResult) -> Result<String, String> {
        let mut sql = String::new();

        // Add comment with metadata
        sql.push_str(&format!(
            "-- Exported {} rows in {}ms\n",
            result.row_count,
            result.execution_time_ms.unwrap_or(0)
        ));
        sql.push_str(&format!("-- Columns: {}\n\n", result.columns.join(", ")));

        // Generate INSERT statements
        if result.rows.is_empty() {
            sql.push_str("-- No data to insert\n");
        } else {
            for row in &result.rows {
                sql.push_str("INSERT INTO table_name (");
                sql.push_str(&result.columns.join(", "));
                sql.push_str(") VALUES (");

                let values: Vec<String> = row.iter().map(|v| Self::sql_value(v)).collect();
                sql.push_str(&values.join(", "));
                sql.push_str(");\n");
            }
        }

        Ok(sql)
    }

    /// Escape a value for CSV format
    fn csv_escape(value: &Value) -> String {
        let s = match value {
            Value::Null => String::new(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(_) => value.to_string(),
            Value::Object(_) => value.to_string(),
        };

        // Escape quotes and wrap in quotes if contains comma, quote, or newline
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s
        }
    }

    /// Convert a value to SQL format
    fn sql_value(value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("'{}'", s.replace('\'', "''")),
            Value::Array(arr) => {
                // Arrays become ARRAY[] syntax
                let values: Vec<String> = arr.iter().map(|v| Self::sql_value(v)).collect();
                format!("ARRAY[{}]", values.join(", "))
            }
            Value::Object(_) => format!("'{}'", value.to_string().replace('\'', "''")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_export_format_from_str() {
        assert!(matches!(
            ExportFormat::from_str("csv"),
            Some(ExportFormat::CSV)
        ));
        assert!(matches!(
            ExportFormat::from_str("JSON"),
            Some(ExportFormat::JSON)
        ));
        assert!(matches!(
            ExportFormat::from_str("sql"),
            Some(ExportFormat::SQL)
        ));
        assert!(ExportFormat::from_str("invalid").is_none());
    }

    #[test]
    fn test_export_format_properties() {
        assert_eq!(ExportFormat::CSV.extension(), "csv");
        assert_eq!(ExportFormat::JSON.extension(), "json");
        assert_eq!(ExportFormat::SQL.extension(), "sql");

        assert_eq!(ExportFormat::CSV.content_type(), "text/csv; charset=utf-8");
        assert_eq!(
            ExportFormat::JSON.content_type(),
            "application/json; charset=utf-8"
        );
        assert_eq!(
            ExportFormat::SQL.content_type(),
            "text/plain; charset=utf-8"
        );
    }

    #[test]
    fn test_csv_export() {
        let result = QueryResult {
            columns: vec!["name".to_string(), "age".to_string()],
            rows: vec![
                vec![json!("Alice"), json!(30)],
                vec![json!("Bob"), json!(25)],
            ],
            row_count: 2,
            affected_rows: None,
            execution_time_ms: Some(100),
        };

        let csv = ExportService::export(&result, ExportFormat::CSV).unwrap();
        assert!(csv.contains("name,age"));
        assert!(csv.contains("Alice,30"));
        assert!(csv.contains("Bob,25"));
    }

    #[test]
    fn test_csv_export_with_special_chars() {
        let result = QueryResult {
            columns: vec!["name".to_string()],
            rows: vec![vec![json!("John, Doe")], vec![json!("It\"s quoted")]],
            row_count: 2,
            affected_rows: None,
            execution_time_ms: Some(50),
        };

        let csv = ExportService::export(&result, ExportFormat::CSV).unwrap();
        assert!(csv.contains("\"John, Doe\""));
        assert!(csv.contains("\"It\"\"s quoted\""));
    }

    #[test]
    fn test_json_export() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![json!(1), json!("Alice")]],
            row_count: 1,
            affected_rows: None,
            execution_time_ms: Some(50),
        };

        let json_str = ExportService::export(&result, ExportFormat::JSON).unwrap();
        assert!(json_str.contains("\"id\""));
        assert!(json_str.contains("\"name\""));
        assert!(json_str.contains("\"row_count\"") && json_str.contains("1"));
        assert!(json_str.contains("\"execution_time_ms\""));
    }

    #[test]
    fn test_sql_export() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![json!(1), json!("Alice")]],
            row_count: 1,
            affected_rows: None,
            execution_time_ms: Some(50),
        };

        let sql = ExportService::export(&result, ExportFormat::SQL).unwrap();
        assert!(sql.contains("INSERT INTO table_name"));
        assert!(sql.contains("(id, name)"));
        assert!(sql.contains("VALUES (1, 'Alice')"));
    }

    #[test]
    fn test_sql_export_with_null_and_strings() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![json!(null), json!("O'Reilly")],
                vec![json!(1), json!("Test")],
            ],
            row_count: 2,
            affected_rows: None,
            execution_time_ms: Some(50),
        };

        let sql = ExportService::export(&result, ExportFormat::SQL).unwrap();
        assert!(sql.contains("NULL"));
        assert!(sql.contains("'O''Reilly'"));
    }

    #[test]
    fn test_sql_export_empty() {
        let result = QueryResult {
            columns: vec!["id".to_string()],
            rows: vec![],
            row_count: 0,
            affected_rows: None,
            execution_time_ms: Some(10),
        };

        let sql = ExportService::export(&result, ExportFormat::SQL).unwrap();
        assert!(sql.contains("No data to insert"));
    }
}
