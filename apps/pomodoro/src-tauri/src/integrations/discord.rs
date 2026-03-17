use std::time::Duration;

use serde::Deserialize;

use super::AppEvent;

#[derive(Deserialize)]
struct DiscordConfig {
    webhook_url: String,
}

pub fn dispatch_discord(config_json: &str, event: &AppEvent) -> Result<(), String> {
    let config: DiscordConfig =
        serde_json::from_str(config_json).map_err(|e| format!("Invalid Discord config: {e}"))?;

    let message = format_discord_message(event);
    if message.is_empty() {
        return Ok(());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let body = serde_json::json!({
        "content": message,
        "username": "Pomodoro",
    });

    let response = client
        .post(&config.webhook_url)
        .json(&body)
        .send()
        .map_err(|e| format!("Discord request failed: {e}"))?;

    if response.status().is_success() || response.status().as_u16() == 204 {
        Ok(())
    } else {
        Err(format!(
            "Discord returned status {}",
            response.status().as_u16()
        ))
    }
}

fn format_discord_message(event: &AppEvent) -> String {
    let task_title = event
        .payload
        .get("title")
        .or_else(|| event.payload.get("task_title"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match event.event_type.as_str() {
        "session.started" => format!("\u{1f345} Session started: {task_title}"),
        "session.completed" => "\u{2705} Session completed!".to_string(),
        "session.paused" => "\u{23f8}\u{fe0f} Session paused".to_string(),
        "task.created" => format!("\u{1f4dd} New task: {task_title}"),
        "task.completed" => format!("\u{1f389} Task completed: {task_title}"),
        _ => String::new(),
    }
}

pub fn test_discord(config_json: &str) -> Result<String, String> {
    let config: DiscordConfig =
        serde_json::from_str(config_json).map_err(|e| format!("Invalid Discord config: {e}"))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let body = serde_json::json!({
        "content": "\u{1f345} Pomodoro integration test successful!",
        "username": "Pomodoro",
    });

    let response = client
        .post(&config.webhook_url)
        .json(&body)
        .send()
        .map_err(|e| format!("Discord test failed: {e}"))?;

    if response.status().is_success() || response.status().as_u16() == 204 {
        Ok("Discord webhook test successful".to_string())
    } else {
        Err(format!(
            "Discord webhook returned status {}",
            response.status().as_u16()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_session_started() {
        let event = AppEvent {
            event_type: "session.started".to_string(),
            payload: serde_json::json!({"title": "Write tests"}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let msg = format_discord_message(&event);
        assert!(msg.contains("Write tests"));
        assert!(msg.contains("Session started"));
    }

    #[test]
    fn format_session_completed() {
        let event = AppEvent {
            event_type: "session.completed".to_string(),
            payload: serde_json::json!({}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let msg = format_discord_message(&event);
        assert!(msg.contains("Session completed"));
    }

    #[test]
    fn format_session_paused() {
        let event = AppEvent {
            event_type: "session.paused".to_string(),
            payload: serde_json::json!({}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let msg = format_discord_message(&event);
        assert!(msg.contains("paused"));
    }

    #[test]
    fn format_task_created() {
        let event = AppEvent {
            event_type: "task.created".to_string(),
            payload: serde_json::json!({"title": "New task"}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let msg = format_discord_message(&event);
        assert!(msg.contains("New task"));
    }

    #[test]
    fn format_task_completed() {
        let event = AppEvent {
            event_type: "task.completed".to_string(),
            payload: serde_json::json!({"title": "Done task"}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let msg = format_discord_message(&event);
        assert!(msg.contains("Done task"));
    }

    #[test]
    fn format_unknown_event_returns_empty() {
        let event = AppEvent {
            event_type: "unknown.event".to_string(),
            payload: serde_json::json!({}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let msg = format_discord_message(&event);
        assert!(msg.is_empty());
    }

    #[test]
    fn dispatch_invalid_config() {
        let event = AppEvent {
            event_type: "test".to_string(),
            payload: serde_json::json!({}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let result = dispatch_discord("not json", &event);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid Discord config"));
    }
}
