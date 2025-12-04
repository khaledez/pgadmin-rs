use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
/// Query History Service
///
/// Tracks executed queries for easy re-execution and history viewing.
/// Stores queries in memory with configurable capacity.
use std::sync::Arc;
use tokio::sync::RwLock;

/// A single query history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier for this query
    pub id: String,
    /// The SQL query text
    pub query: String,
    /// When the query was executed
    pub executed_at: DateTime<Utc>,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Number of rows returned/affected
    pub row_count: Option<i64>,
    /// Was the query successful
    pub success: bool,
    /// Error message if query failed
    pub error: Option<String>,
}

impl HistoryEntry {
    /// Create a new successful query history entry
    pub fn new(query: String, duration_ms: u64, row_count: Option<i64>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            query,
            executed_at: Utc::now(),
            duration_ms,
            row_count,
            success: true,
            error: None,
        }
    }

    /// Create a failed query history entry
    pub fn failed(query: String, duration_ms: u64, error: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            query,
            executed_at: Utc::now(),
            duration_ms,
            row_count: None,
            success: false,
            error: Some(error),
        }
    }
}

/// Query history manager
///
/// Maintains a circular buffer of recent queries.
/// Useful for tracking what queries were run and enabling quick re-execution.
pub struct QueryHistory {
    /// Circular buffer of history entries
    entries: Arc<RwLock<Vec<HistoryEntry>>>,
    /// Maximum number of entries to keep
    max_entries: usize,
}

impl QueryHistory {
    /// Create a new query history manager
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Add a query to the history
    pub async fn add(&self, entry: HistoryEntry) {
        let mut entries = self.entries.write().await;
        entries.push(entry);

        // Keep only the last max_entries
        if entries.len() > self.max_entries {
            let drain_count = entries.len() - self.max_entries;
            entries.drain(0..drain_count);
        }
    }

    /// Get all history entries
    pub async fn get_all(&self) -> Vec<HistoryEntry> {
        self.entries.read().await.clone()
    }

    /// Get recent history entries
    pub async fn get_recent(&self, count: usize) -> Vec<HistoryEntry> {
        let entries = self.entries.read().await;
        entries.iter().rev().take(count).cloned().collect()
    }

    /// Get a specific entry by ID
    pub async fn get_by_id(&self, id: &str) -> Option<HistoryEntry> {
        let entries = self.entries.read().await;
        entries.iter().find(|e| e.id == id).cloned()
    }

    /// Get history entries by query text
    pub async fn get_by_query(&self, query: &str) -> Vec<HistoryEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.query.contains(query))
            .cloned()
            .collect()
    }

    /// Get successful queries only
    pub async fn get_successful(&self) -> Vec<HistoryEntry> {
        let entries = self.entries.read().await;
        entries.iter().filter(|e| e.success).cloned().collect()
    }

    /// Get failed queries only
    pub async fn get_failed(&self) -> Vec<HistoryEntry> {
        let entries = self.entries.read().await;
        entries.iter().filter(|e| !e.success).cloned().collect()
    }

    /// Clear all history
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }

    /// Get the total number of entries
    pub async fn count(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Get statistics about the query history
    pub async fn stats(&self) -> HistoryStats {
        let entries = self.entries.read().await;

        let total = entries.len();
        let successful = entries.iter().filter(|e| e.success).count();
        let failed = entries.len() - successful;

        let avg_duration = if total > 0 {
            let sum: u64 = entries.iter().map(|e| e.duration_ms).sum();
            sum / total as u64
        } else {
            0
        };

        let most_common = entries
            .iter()
            .fold(std::collections::HashMap::new(), |mut map, entry| {
                *map.entry(entry.query.clone()).or_insert(0) += 1;
                map
            })
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(query, _)| query);

        HistoryStats {
            total_queries: total,
            successful_queries: successful,
            failed_queries: failed,
            average_duration_ms: avg_duration,
            most_common_query: most_common,
        }
    }
}

/// Statistics about query history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_queries: usize,
    pub successful_queries: usize,
    pub failed_queries: usize,
    pub average_duration_ms: u64,
    pub most_common_query: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_history_entry() {
        let entry = HistoryEntry::new("SELECT 1".to_string(), 10, Some(1));
        assert_eq!(entry.query, "SELECT 1");
        assert_eq!(entry.duration_ms, 10);
        assert!(entry.success);
        assert!(entry.error.is_none());
    }

    #[tokio::test]
    async fn test_create_failed_entry() {
        let entry = HistoryEntry::failed(
            "SELECT * FROM invalid".to_string(),
            5,
            "relation does not exist".to_string(),
        );
        assert!(!entry.success);
        assert_eq!(entry.error, Some("relation does not exist".to_string()));
    }

    #[tokio::test]
    async fn test_add_to_history() {
        let history = QueryHistory::new(10);
        let entry = HistoryEntry::new("SELECT 1".to_string(), 10, Some(1));

        history.add(entry.clone()).await;

        let entries = history.get_all().await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, entry.id);
    }

    #[tokio::test]
    async fn test_max_entries() {
        let history = QueryHistory::new(3);

        for i in 0..5 {
            let entry = HistoryEntry::new(format!("SELECT {}", i), 10, Some(1));
            history.add(entry).await;
        }

        let entries = history.get_all().await;
        assert_eq!(entries.len(), 3); // Only last 3 kept
    }

    #[tokio::test]
    async fn test_get_recent() {
        let history = QueryHistory::new(10);

        for i in 0..5 {
            let entry = HistoryEntry::new(format!("SELECT {}", i), 10, Some(1));
            history.add(entry).await;
        }

        let recent = history.get_recent(2).await;
        assert_eq!(recent.len(), 2);
    }

    #[tokio::test]
    async fn test_get_by_id() {
        let history = QueryHistory::new(10);
        let entry = HistoryEntry::new("SELECT 1".to_string(), 10, Some(1));
        let entry_id = entry.id.clone();

        history.add(entry.clone()).await;

        let found = history.get_by_id(&entry_id).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, entry_id);
    }

    #[tokio::test]
    async fn test_get_by_query() {
        let history = QueryHistory::new(10);
        let entry1 = HistoryEntry::new("SELECT * FROM users".to_string(), 10, Some(5));
        let entry2 = HistoryEntry::new("SELECT * FROM products".to_string(), 15, Some(10));

        history.add(entry1).await;
        history.add(entry2).await;

        let found = history.get_by_query("users").await;
        assert_eq!(found.len(), 1);
    }

    #[tokio::test]
    async fn test_get_successful_queries() {
        let history = QueryHistory::new(10);
        let success = HistoryEntry::new("SELECT 1".to_string(), 10, Some(1));
        let failed = HistoryEntry::failed("INVALID".to_string(), 5, "syntax error".to_string());

        history.add(success).await;
        history.add(failed).await;

        let successful = history.get_successful().await;
        assert_eq!(successful.len(), 1);
        assert!(successful[0].success);
    }

    #[tokio::test]
    async fn test_get_failed_queries() {
        let history = QueryHistory::new(10);
        let success = HistoryEntry::new("SELECT 1".to_string(), 10, Some(1));
        let failed = HistoryEntry::failed("INVALID".to_string(), 5, "syntax error".to_string());

        history.add(success).await;
        history.add(failed).await;

        let failed_queries = history.get_failed().await;
        assert_eq!(failed_queries.len(), 1);
        assert!(!failed_queries[0].success);
    }

    #[tokio::test]
    async fn test_clear_history() {
        let history = QueryHistory::new(10);
        let entry = HistoryEntry::new("SELECT 1".to_string(), 10, Some(1));

        history.add(entry).await;
        assert_eq!(history.count().await, 1);

        history.clear().await;
        assert_eq!(history.count().await, 0);
    }

    #[tokio::test]
    async fn test_stats() {
        let history = QueryHistory::new(10);
        let entry1 = HistoryEntry::new("SELECT 1".to_string(), 100, Some(1));
        let entry2 = HistoryEntry::new("SELECT 1".to_string(), 200, Some(1));
        let entry3 = HistoryEntry::failed("BAD QUERY".to_string(), 50, "error".to_string());

        history.add(entry1).await;
        history.add(entry2).await;
        history.add(entry3).await;

        let stats = history.stats().await;
        assert_eq!(stats.total_queries, 3);
        assert_eq!(stats.successful_queries, 2);
        assert_eq!(stats.failed_queries, 1);
        assert_eq!(stats.average_duration_ms, 116); // (100 + 200 + 50) / 3
    }
}
