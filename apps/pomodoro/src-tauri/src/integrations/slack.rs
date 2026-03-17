use std::time::Duration;

use serde::Deserialize;

use super::AppEvent;

#[derive(Deserialize)]
struct SlackConfig {
    bot_token: String,
    #[serde(default = "default_emoji")]
    status_emoji: String,
}

fn default_emoji() -> String {
    ":tomato:".to_string()
}

pub fn dispatch_slack(config_json: &str, event: &AppEvent) -> Result<(), String> {
    let config: SlackConfig =
        serde_json::from_str(config_json).map_err(|e| format!("Invalid Slack config: {e}"))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let (status_text, status_emoji, status_expiration) = match event.event_type.as_str() {
        "session.started" => {
            let task_title = event
                .payload
                .get("task_title")
                .and_then(|v| v.as_str())
                .unwrap_or("Focus session");
            let expiration = chrono::Utc::now().timestamp() + 30 * 60; // 30 min default
            (
                format!("Deep focus \u{2014} {task_title}"),
                config.status_emoji.clone(),
                expiration,
            )
        }
        "session.completed" | "break.started" => {
            // Clear status or set break
            (String::new(), String::new(), 0)
        }
        _ => return Ok(()), // Ignore other events
    };

    let body = serde_json::json!({
        "profile": {
            "status_text": status_text,
            "status_emoji": status_emoji,
            "status_expiration": status_expiration,
        }
    });

    let response = client
        .post("https://slack.com/api/users.profile.set")
        .bearer_auth(&config.bot_token)
        .json(&body)
        .send()
        .map_err(|e| format!("Slack request failed: {e}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "Slack returned status {}",
            response.status().as_u16()
        ))
    }
}

pub fn test_slack(config_json: &str) -> Result<String, String> {
    let config: SlackConfig =
        serde_json::from_str(config_json).map_err(|e| format!("Invalid Slack config: {e}"))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get("https://slack.com/api/auth.test")
        .bearer_auth(&config.bot_token)
        .send()
        .map_err(|e| format!("Slack test failed: {e}"))?;

    if response.status().is_success() {
        let body: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse Slack response: {e}"))?;
        let team = body
            .get("team")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(format!("Connected to Slack team: {team}"))
    } else {
        Err(format!(
            "Slack auth test returned status {}",
            response.status().as_u16()
        ))
    }
}
