use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use super::AppEvent;

#[derive(Deserialize)]
struct WebhookConfig {
    url: String,
    #[serde(default)]
    headers: HashMap<String, String>,
}

pub fn dispatch_webhook(config_json: &str, event: &AppEvent) -> Result<(), String> {
    let config: WebhookConfig =
        serde_json::from_str(config_json).map_err(|e| format!("Invalid webhook config: {e}"))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let mut request = client.post(&config.url).json(event);

    for (key, value) in &config.headers {
        request = request.header(key.as_str(), value.as_str());
    }

    let response = request.send().map_err(|e| format!("Webhook request failed: {e}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "Webhook returned status {}",
            response.status().as_u16()
        ))
    }
}

pub fn test_webhook(config_json: &str) -> Result<String, String> {
    let test_event = AppEvent {
        event_type: "test".to_string(),
        payload: serde_json::json!({"message": "Test event from Pomodoro"}),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    dispatch_webhook(config_json, &test_event)?;
    Ok("Webhook test successful".to_string())
}
