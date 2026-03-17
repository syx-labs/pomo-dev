use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

// ── Curated Model Registry ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedModel {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub size_mb: u64,
    pub category: String,
}

pub fn get_curated_models() -> Vec<CuratedModel> {
    vec![
        CuratedModel {
            name: "llama3.2:1b".into(),
            display_name: "Llama 3.2 1B".into(),
            description: "Ultra-fast responses, good for quick coaching tips".into(),
            size_mb: 1300,
            category: "fast".into(),
        },
        CuratedModel {
            name: "llama3.2:3b".into(),
            display_name: "Llama 3.2 3B".into(),
            description: "Balanced speed and quality for daily briefings".into(),
            size_mb: 2000,
            category: "balanced".into(),
        },
        CuratedModel {
            name: "gemma3:4b".into(),
            display_name: "Gemma 3 4B".into(),
            description: "Strong reasoning, great for weekly reports".into(),
            size_mb: 3400,
            category: "balanced".into(),
        },
        CuratedModel {
            name: "qwen2.5:3b".into(),
            display_name: "Qwen 2.5 3B".into(),
            description: "Excellent instruction following for structured output".into(),
            size_mb: 1900,
            category: "balanced".into(),
        },
        CuratedModel {
            name: "phi4-mini".into(),
            display_name: "Phi-4 Mini".into(),
            description: "Microsoft's compact model, strong at analysis".into(),
            size_mb: 2400,
            category: "capable".into(),
        },
    ]
}

// ── Ollama API Types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    pub model: String,
    pub status: String,
    pub total: u64,
    pub completed: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullComplete {
    pub model: String,
    pub success: bool,
    pub error: Option<String>,
}

// ── API Functions ───────────────────────────────────────────────────────────

const HEALTH_TIMEOUT: Duration = Duration::from_secs(5);

pub fn check_health(base_url: &str) -> bool {
    Client::builder()
        .timeout(HEALTH_TIMEOUT)
        .build()
        .ok()
        .and_then(|c| c.get(base_url).send().ok())
        .is_some_and(|r| r.status().is_success())
}

pub fn list_local_models(base_url: &str) -> Result<Vec<OllamaModel>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let url = format!("{base_url}/api/tags");
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to connect to Ollama: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama returned status {}", resp.status()));
    }

    let tags: TagsResponse = resp
        .json()
        .map_err(|e| format!("Failed to parse Ollama response: {e}"))?;

    Ok(tags.models)
}

pub fn delete_model(base_url: &str, name: &str) -> Result<(), String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let url = format!("{base_url}/api/delete");
    let resp = client
        .delete(&url)
        .json(&serde_json::json!({ "name": name }))
        .send()
        .map_err(|e| format!("Failed to delete model: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to delete model '{}': status {}",
            name,
            resp.status()
        ));
    }

    Ok(())
}

pub fn pull_model(
    app_handle: tauri::AppHandle,
    base_url: String,
    name: String,
    cancel: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let result = pull_model_inner(&app_handle, &base_url, &name, &cancel);

        let complete = PullComplete {
            model: name,
            success: result.is_ok(),
            error: result.err(),
        };
        let _ = app_handle.emit("ollama:pull-complete", &complete);
    });
}

const PROGRESS_THROTTLE: Duration = Duration::from_millis(200);

fn pull_model_inner(
    app_handle: &tauri::AppHandle,
    base_url: &str,
    name: &str,
    cancel: &AtomicBool,
) -> Result<(), String> {
    // No timeout — model downloads can take a long time
    let client = Client::builder()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let url = format!("{base_url}/api/pull");
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "name": name }))
        .send()
        .map_err(|e| format!("Failed to start pull: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama returned status {}", resp.status()));
    }

    let reader = std::io::BufReader::new(resp);
    let mut last_emit = std::time::Instant::now() - PROGRESS_THROTTLE;
    let mut received_success = false;

    for line in reader.lines() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Pull cancelled".into());
        }

        let line = line.map_err(|e| format!("Failed to read response: {e}"))?;
        if line.is_empty() {
            continue;
        }

        // Parse streaming JSON line
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            // Check for error in response
            if let Some(err) = json.get("error").and_then(|e| e.as_str()) {
                return Err(err.to_string());
            }

            let status = json
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("");

            if status == "success" {
                received_success = true;
            }

            // Throttle progress events to avoid IPC spam
            let now = std::time::Instant::now();
            if now.duration_since(last_emit) < PROGRESS_THROTTLE {
                continue;
            }
            last_emit = now;

            let total = json.get("total").and_then(|t| t.as_u64()).unwrap_or(0);
            let completed = json
                .get("completed")
                .and_then(|c| c.as_u64())
                .unwrap_or(0);
            let percent = if total > 0 {
                (completed as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            let progress = PullProgress {
                model: name.to_string(),
                status: status.to_string(),
                total,
                completed,
                percent,
            };
            let _ = app_handle.emit("ollama:pull-progress", &progress);
        }
    }

    if received_success {
        Ok(())
    } else {
        Err("Pull ended without success confirmation".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Curated Models ─────────────────────────────────────────────────

    #[test]
    fn curated_models_not_empty() {
        let models = get_curated_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.name == "llama3.2:1b"));
    }

    #[test]
    fn curated_models_have_valid_categories() {
        let valid = ["fast", "balanced", "capable"];
        for model in get_curated_models() {
            assert!(
                valid.contains(&model.category.as_str()),
                "Invalid category '{}' for model '{}'",
                model.category,
                model.name
            );
        }
    }

    #[test]
    fn curated_models_have_nonzero_sizes() {
        for model in get_curated_models() {
            assert!(
                model.size_mb > 0,
                "Model '{}' has zero size_mb",
                model.name
            );
        }
    }

    #[test]
    fn curated_models_have_unique_names() {
        let models = get_curated_models();
        let names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "Duplicate model names found");
    }

    #[test]
    fn curated_models_names_contain_tag() {
        for model in get_curated_models() {
            // Ollama model names should either have a : tag or be a bare name
            assert!(
                !model.name.is_empty(),
                "Model has empty name: '{}'",
                model.display_name
            );
        }
    }

    // ── Health Check ───────────────────────────────────────────────────

    #[test]
    fn health_check_returns_false_for_invalid_url() {
        assert!(!check_health("http://127.0.0.1:1"));
    }

    #[test]
    fn health_check_returns_false_for_nonsense_url() {
        assert!(!check_health("not-a-url"));
    }

    // ── API Error Handling ─────────────────────────────────────────────

    #[test]
    fn list_local_models_returns_error_for_unreachable_host() {
        let result = list_local_models("http://127.0.0.1:1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to connect"));
    }

    #[test]
    fn delete_model_returns_error_for_unreachable_host() {
        let result = delete_model("http://127.0.0.1:1", "test-model");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to delete"));
    }

    // ── Deserialization ────────────────────────────────────────────────

    #[test]
    fn tags_response_deserializes_correctly() {
        let json = r#"{"models":[{"name":"llama3.2:1b","size":1300000000,"modified_at":"2024-01-01T00:00:00Z"}]}"#;
        let tags: TagsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(tags.models.len(), 1);
        assert_eq!(tags.models[0].name, "llama3.2:1b");
        assert_eq!(tags.models[0].size, 1_300_000_000);
    }

    #[test]
    fn tags_response_deserializes_empty_list() {
        let json = r#"{"models":[]}"#;
        let tags: TagsResponse = serde_json::from_str(json).unwrap();
        assert!(tags.models.is_empty());
    }

    #[test]
    fn pull_progress_serializes_correctly() {
        let progress = PullProgress {
            model: "test".into(),
            status: "downloading".into(),
            total: 1000,
            completed: 500,
            percent: 50.0,
        };
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("\"percent\":50.0"));
        assert!(json.contains("\"model\":\"test\""));
    }

    #[test]
    fn pull_complete_serializes_success() {
        let complete = PullComplete {
            model: "test".into(),
            success: true,
            error: None,
        };
        let json = serde_json::to_string(&complete).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"error\":null"));
    }

    #[test]
    fn pull_complete_serializes_failure() {
        let complete = PullComplete {
            model: "test".into(),
            success: false,
            error: Some("network error".into()),
        };
        let json = serde_json::to_string(&complete).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"network error\""));
    }

    // ── OllamaModel Serde Round-Trip ───────────────────────────────────

    #[test]
    fn ollama_model_round_trip() {
        let model = OllamaModel {
            name: "phi4-mini".into(),
            size: 2_400_000_000,
            modified_at: "2024-06-01T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&model).unwrap();
        let deserialized: OllamaModel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, model.name);
        assert_eq!(deserialized.size, model.size);
        assert_eq!(deserialized.modified_at, model.modified_at);
    }
}
