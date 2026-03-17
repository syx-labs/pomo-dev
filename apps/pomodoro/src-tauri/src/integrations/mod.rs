pub mod discord;
pub mod slack;
pub mod webhook;

use std::sync::Mutex;

use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::db;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppEvent {
    pub event_type: String,
    pub payload: Value,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntegrationConfig {
    pub id: String,
    pub integration_type: String,
    pub name: String,
    pub config: String,
    pub enabled: bool,
    pub events: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventLogEntry {
    pub id: String,
    pub integration_id: String,
    pub event_type: String,
    pub payload: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

pub struct EventDispatcher;

impl EventDispatcher {
    pub fn new() -> Self {
        Self
    }

    /// Dispatch an event to all enabled integrations that subscribe to it.
    ///
    /// IMPORTANT: This method must NOT hold the DB lock during HTTP calls.
    /// Pattern: lock DB -> read integrations -> DROP lock -> HTTP calls -> lock DB -> log results -> DROP lock
    /// Errors are logged to event_log, never propagated.
    pub fn dispatch(&self, conn_mutex: &Mutex<Connection>, event: AppEvent) {
        // Step 1: Lock DB, read enabled integrations for this event type, drop lock
        let integrations = {
            let conn = match conn_mutex.lock() {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("EventDispatcher: failed to lock DB to read integrations");
                    return;
                }
            };
            match db::get_enabled_integrations_for_event(&conn, &event.event_type) {
                Ok(list) => list,
                Err(e) => {
                    eprintln!("EventDispatcher: failed to query integrations: {e}");
                    return;
                }
            }
            // conn lock dropped here
        };

        if integrations.is_empty() {
            return;
        }

        let payload_str = serde_json::to_string(&event).unwrap_or_default();

        // Step 2: Dispatch to each integration (no DB lock held)
        let mut results: Vec<EventLogEntry> = Vec::new();
        let now = Utc::now().to_rfc3339();

        for integration in &integrations {
            let result = match integration.integration_type.as_str() {
                "webhook" => webhook::dispatch_webhook(&integration.config, &event),
                "slack" => slack::dispatch_slack(&integration.config, &event),
                "discord" => discord::dispatch_discord(&integration.config, &event),
                other => Err(format!("Unknown integration type: {other}")),
            };

            let (status, error_message) = match result {
                Ok(()) => ("success".to_string(), None),
                Err(e) => {
                    eprintln!(
                        "EventDispatcher: error dispatching to '{}': {e}",
                        integration.name
                    );
                    ("error".to_string(), Some(e))
                }
            };

            results.push(EventLogEntry {
                id: Uuid::new_v4().to_string(),
                integration_id: integration.id.clone(),
                event_type: event.event_type.clone(),
                payload: payload_str.clone(),
                status,
                error_message,
                created_at: now.clone(),
            });
        }

        // Step 3: Lock DB, log all results, drop lock
        if let Ok(conn) = conn_mutex.lock() {
            for entry in &results {
                if let Err(e) = db::log_event(&conn, entry) {
                    eprintln!("EventDispatcher: failed to log event: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_event_serialization() {
        let event = AppEvent {
            event_type: "session.completed".to_string(),
            payload: serde_json::json!({"session_type": "work"}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: AppEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, "session.completed");
    }

    #[test]
    fn integration_config_serialization() {
        let config = IntegrationConfig {
            id: "int-1".to_string(),
            integration_type: "webhook".to_string(),
            name: "Test".to_string(),
            config: "{}".to_string(),
            enabled: true,
            events: "[]".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: IntegrationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "int-1");
        assert!(parsed.enabled);
    }

    #[test]
    fn event_log_entry_serialization() {
        let entry = EventLogEntry {
            id: "ev-1".to_string(),
            integration_id: "int-1".to_string(),
            event_type: "test".to_string(),
            payload: "{}".to_string(),
            status: "success".to_string(),
            error_message: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: EventLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, "success");
        assert!(parsed.error_message.is_none());
    }

    #[test]
    fn event_dispatcher_dispatch_no_integrations() {
        let conn = crate::db::init_in_memory().unwrap();
        let conn_mutex = Mutex::new(conn);
        let dispatcher = EventDispatcher::new();

        // Should not panic even with no integrations
        dispatcher.dispatch(
            &conn_mutex,
            AppEvent {
                event_type: "test.event".to_string(),
                payload: serde_json::json!({}),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
            },
        );
    }
}
