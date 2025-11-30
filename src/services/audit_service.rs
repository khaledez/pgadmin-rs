/// Audit Logging Service
///
/// Provides audit logging functionality to track security-relevant events
/// such as query executions, failed authentication attempts, and schema modifications.
///
/// Audit logs can be used for:
/// - Security monitoring and incident response
/// - Compliance auditing (GDPR, HIPAA, SOC 2, etc.)
/// - Forensic analysis
/// - Performance troubleshooting

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit event types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    /// Query execution attempt
    QueryExecution,
    /// Authentication attempt
    AuthenticationAttempt,
    /// Authentication success
    AuthenticationSuccess,
    /// Authentication failure
    AuthenticationFailure,
    /// Schema modification (CREATE/DROP/ALTER)
    SchemaModification,
    /// Table data modification (INSERT/UPDATE/DELETE)
    DataModification,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// SQL error occurred
    SqlError,
    /// Dangerous query attempted
    DangerousQueryDetected,
    /// Access denied
    AccessDenied,
    /// Configuration change
    ConfigurationChange,
}

/// Audit event that gets logged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Type of event
    pub event_type: AuditEventType,
    /// IP address of the client
    pub client_ip: String,
    /// User identifier (if applicable)
    pub user_id: Option<String>,
    /// Action that was attempted
    pub action: String,
    /// Resource affected (table, schema, query, etc.)
    pub resource: String,
    /// Whether the action succeeded
    pub success: bool,
    /// Detailed information about the event
    pub details: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: AuditEventType,
        client_ip: String,
        action: String,
        resource: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            client_ip,
            user_id: None,
            action,
            resource,
            success: true,
            details: None,
        }
    }

    /// Set whether the event succeeded
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    /// Set additional details about the event
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Set the user ID for this event
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// Audit logger that stores events in memory and can be extended to persist to database
///
/// This implementation stores logs in memory for development and testing.
/// Production deployments should extend this to write to a persistent store.
pub struct AuditLogger {
    /// In-memory event storage (limit to last N events)
    events: Arc<RwLock<Vec<AuditEvent>>>,
    /// Maximum number of events to keep in memory
    max_events: usize,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::with_capacity(max_events))),
            max_events,
        }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) {
        let mut events = self.events.write().await;

        // Log to standard error for immediate visibility (important for security)
        tracing::warn!(
            event_id = %event.id,
            timestamp = %event.timestamp,
            event_type = ?event.event_type,
            client_ip = %event.client_ip,
            user_id = ?event.user_id,
            action = %event.action,
            resource = %event.resource,
            success = event.success,
            details = ?event.details,
            "Audit event logged"
        );

        events.push(event);

        // Keep only the last max_events in memory
        if events.len() > self.max_events {
            let drain_count = events.len() - self.max_events;
            events.drain(0..drain_count);
        }
    }

    /// Get all audit events (for testing/debugging)
    pub async fn get_events(&self) -> Vec<AuditEvent> {
        self.events.read().await.clone()
    }

    /// Get recent audit events (last N)
    pub async fn get_recent_events(&self, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(count).cloned().collect()
    }

    /// Filter events by type
    pub async fn get_events_by_type(&self, event_type: AuditEventType) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Filter events by client IP
    pub async fn get_events_by_ip(&self, ip: &str) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.client_ip == ip)
            .cloned()
            .collect()
    }

    /// Clear all events (useful for testing)
    #[cfg(test)]
    pub async fn clear(&self) {
        self.events.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::QueryExecution,
            "127.0.0.1".to_string(),
            "SELECT * FROM users".to_string(),
            "public.users".to_string(),
        );

        assert_eq!(event.event_type, AuditEventType::QueryExecution);
        assert_eq!(event.client_ip, "127.0.0.1");
        assert_eq!(event.action, "SELECT * FROM users");
        assert!(event.success);
    }

    #[tokio::test]
    async fn test_audit_event_builders() {
        let event = AuditEvent::new(
            AuditEventType::AuthenticationFailure,
            "192.168.1.1".to_string(),
            "Failed login attempt".to_string(),
            "login".to_string(),
        )
        .with_success(false)
        .with_details("Invalid password".to_string())
        .with_user("admin".to_string());

        assert_eq!(event.event_type, AuditEventType::AuthenticationFailure);
        assert!(!event.success);
        assert_eq!(event.details, Some("Invalid password".to_string()));
        assert_eq!(event.user_id, Some("admin".to_string()));
    }

    #[tokio::test]
    async fn test_audit_logger_logging() {
        let logger = AuditLogger::new(10);

        let event = AuditEvent::new(
            AuditEventType::QueryExecution,
            "127.0.0.1".to_string(),
            "SELECT 1".to_string(),
            "test".to_string(),
        );

        logger.log(event.clone()).await;

        let events = logger.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }

    #[tokio::test]
    async fn test_audit_logger_max_events() {
        let logger = AuditLogger::new(3);

        for i in 0..5 {
            let event = AuditEvent::new(
                AuditEventType::QueryExecution,
                "127.0.0.1".to_string(),
                format!("Query {}", i),
                "test".to_string(),
            );
            logger.log(event).await;
        }

        let events = logger.get_events().await;
        assert_eq!(events.len(), 3); // Only last 3 kept
    }

    #[tokio::test]
    async fn test_audit_logger_filter_by_type() {
        let logger = AuditLogger::new(10);

        let event1 = AuditEvent::new(
            AuditEventType::QueryExecution,
            "127.0.0.1".to_string(),
            "Query 1".to_string(),
            "test".to_string(),
        );

        let event2 = AuditEvent::new(
            AuditEventType::AuthenticationAttempt,
            "127.0.0.1".to_string(),
            "Login attempt".to_string(),
            "auth".to_string(),
        );

        logger.log(event1).await;
        logger.log(event2).await;

        let query_events = logger.get_events_by_type(AuditEventType::QueryExecution).await;
        assert_eq!(query_events.len(), 1);

        let auth_events = logger.get_events_by_type(AuditEventType::AuthenticationAttempt).await;
        assert_eq!(auth_events.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_logger_filter_by_ip() {
        let logger = AuditLogger::new(10);

        let event1 = AuditEvent::new(
            AuditEventType::QueryExecution,
            "192.168.1.1".to_string(),
            "Query 1".to_string(),
            "test".to_string(),
        );

        let event2 = AuditEvent::new(
            AuditEventType::QueryExecution,
            "192.168.1.2".to_string(),
            "Query 2".to_string(),
            "test".to_string(),
        );

        logger.log(event1).await;
        logger.log(event2).await;

        let ip1_events = logger.get_events_by_ip("192.168.1.1").await;
        assert_eq!(ip1_events.len(), 1);

        let ip2_events = logger.get_events_by_ip("192.168.1.2").await;
        assert_eq!(ip2_events.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_logger_recent_events() {
        let logger = AuditLogger::new(10);

        for i in 0..5 {
            let event = AuditEvent::new(
                AuditEventType::QueryExecution,
                "127.0.0.1".to_string(),
                format!("Query {}", i),
                "test".to_string(),
            );
            logger.log(event).await;
        }

        let recent = logger.get_recent_events(2).await;
        assert_eq!(recent.len(), 2);
        // Most recent should be last in the list (reversed)
    }
}
