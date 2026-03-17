use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use chrono::{Datelike, Utc};
use reqwest::blocking::Client;
use rusqlite::{params, Connection};
use serde_json::Value;

use crate::db;

// ── Error Type ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AiError {
    ProviderDisabled,
    NetworkError(String),
    ParseError(String),
    ConfigError(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::ProviderDisabled => write!(f, "AI provider is disabled"),
            AiError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            AiError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            AiError::ConfigError(msg) => write!(f, "Config error: {msg}"),
        }
    }
}

// ── Provider Trait ──────────────────────────────────────────────────────────

#[allow(dead_code)]
pub trait AiProvider: Send + Sync {
    fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, AiError>;
    fn name(&self) -> &str;
}

// ── Disabled Provider ───────────────────────────────────────────────────────

pub struct DisabledProvider;

impl AiProvider for DisabledProvider {
    fn generate(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, AiError> {
        Err(AiError::ProviderDisabled)
    }

    fn name(&self) -> &str {
        "disabled"
    }
}

// ── Ollama Provider ─────────────────────────────────────────────────────────

pub struct OllamaProvider {
    pub base_url: String,
    pub model: String,
}

impl AiProvider for OllamaProvider {
    fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, AiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        let url = format!("{}/api/generate", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "prompt": user_prompt,
            "system": system_prompt,
            "stream": false
        });

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        let json: Value = resp
            .json()
            .map_err(|e| AiError::ParseError(e.to_string()))?;

        json["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AiError::ParseError("Missing 'response' field in Ollama response".into()))
    }

    fn name(&self) -> &str {
        "ollama"
    }
}

// ── OpenAI Provider ─────────────────────────────────────────────────────────

pub struct OpenAiProvider {
    pub api_key: String,
    pub model: String,
}

impl AiProvider for OpenAiProvider {
    fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, AiError> {
        if self.api_key.is_empty() {
            return Err(AiError::ConfigError("OpenAI API key is not set".into()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ]
        });

        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        let json: Value = resp
            .json()
            .map_err(|e| AiError::ParseError(e.to_string()))?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                AiError::ParseError("Missing 'choices[0].message.content' in OpenAI response".into())
            })
    }

    fn name(&self) -> &str {
        "openai"
    }
}

// ── Anthropic Provider ──────────────────────────────────────────────────────

pub struct AnthropicProvider {
    pub api_key: String,
    pub model: String,
}

impl AiProvider for AnthropicProvider {
    fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, AiError> {
        if self.api_key.is_empty() {
            return Err(AiError::ConfigError("Anthropic API key is not set".into()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 1024,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ]
        });

        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        let json: Value = resp
            .json()
            .map_err(|e| AiError::ParseError(e.to_string()))?;

        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                AiError::ParseError("Missing 'content[0].text' in Anthropic response".into())
            })
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

// ── Factory ─────────────────────────────────────────────────────────────────

pub fn create_provider(
    provider_type: &str,
    api_key: &str,
    model: &str,
    base_url: &str,
) -> Box<dyn AiProvider> {
    match provider_type {
        "ollama" => Box::new(OllamaProvider {
            base_url: base_url.to_string(),
            model: model.to_string(),
        }),
        "openai" => Box::new(OpenAiProvider {
            api_key: api_key.to_string(),
            model: model.to_string(),
        }),
        "anthropic" => Box::new(AnthropicProvider {
            api_key: api_key.to_string(),
            model: model.to_string(),
        }),
        _ => Box::new(DisabledProvider),
    }
}

// ── AI Command Result ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AiCommandResult {
    pub action: String,
    pub params: HashMap<String, String>,
    pub message: String,
    pub success: bool,
}

// ── Parse AI Command ────────────────────────────────────────────────────────

/// Strip markdown code block wrappers (```json ... ``` or ``` ... ```) from AI responses.
fn strip_code_blocks(input: &str) -> &str {
    let trimmed = input.trim();

    // Try stripping ```json ... ``` or ``` ... ```
    let inner = if trimmed.starts_with("```json") {
        trimmed.strip_prefix("```json").unwrap_or(trimmed)
    } else if trimmed.starts_with("```") {
        trimmed.strip_prefix("```").unwrap_or(trimmed)
    } else {
        return trimmed;
    };

    // Strip trailing ```
    let inner = inner.strip_suffix("```").unwrap_or(inner);
    inner.trim()
}

pub fn parse_ai_command(
    provider: &dyn AiProvider,
    user_input: &str,
) -> Result<AiCommandResult, AiError> {
    let system_prompt = r#"You are a Pomodoro app assistant. Parse the user's natural language command into a structured action.
Available actions:
- create_task: params {title, project?, priority?} — Create a new task
- start_timer: params {} — Start the pomodoro timer
- pause_timer: params {} — Pause the timer
- skip_timer: params {} — Skip to next session
- reset_timer: params {} — Reset the timer
- navigate: params {route} — Navigate to a view (/, /tasks, /stats, /settings)

Respond with JSON only: {"action":"action_name","params":{...},"message":"Brief confirmation"}
If you can't parse the command, use action "unknown" with an explanatory message."#;

    let response = provider.generate(system_prompt, user_input)?;
    let cleaned = strip_code_blocks(&response);

    let json: Value = serde_json::from_str(cleaned)
        .map_err(|e| AiError::ParseError(format!("Failed to parse AI response as JSON: {e}")))?;

    let action = json["action"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    let params: HashMap<String, String> = match json.get("params") {
        Some(Value::Object(map)) => map
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect(),
        _ => HashMap::new(),
    };

    let message = json["message"]
        .as_str()
        .unwrap_or("Command processed")
        .to_string();

    Ok(AiCommandResult {
        action,
        params,
        message,
        success: true,
    })
}

// ── Productivity Coach ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProductivityContext {
    pub avg_daily_sessions: f32,
    pub avg_session_duration_secs: f32,
    pub current_streak: i32,
    pub best_streak: i32,
    pub top_hours: Vec<(i32, i32)>,
    pub top_projects: Vec<(String, i32)>,
    pub today_sessions: i32,
    pub today_work_secs: i32,
    pub week_sessions: i32,
    pub week_work_secs: i32,
    pub day_of_week: String,
    pub consecutive_sessions_no_long_break: i32,
}

pub fn build_context(conn: &Connection) -> Result<ProductivityContext, rusqlite::Error> {
    let now = Utc::now();
    let today = now.format("%Y-%m-%d").to_string();
    let thirty_days_ago = (now - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();

    // Avg daily sessions & avg duration (last 30 days of completed work sessions)
    let (total_sessions_30d, total_duration_30d): (i32, i32) = {
        let start = format!("{thirty_days_ago}T00:00:00+00:00");
        let end = format!("{today}T23:59:59+00:00");
        let mut stmt = conn.prepare(
            "SELECT COUNT(*), COALESCE(SUM(duration_secs), 0)
             FROM pomodoro_sessions
             WHERE completed = 1 AND session_type = 'work'
               AND started_at >= ?1 AND started_at <= ?2",
        )?;
        stmt.query_row(params![start, end], |row| Ok((row.get(0)?, row.get(1)?)))?
    };
    let avg_daily_sessions = total_sessions_30d as f32 / 30.0;
    let avg_session_duration_secs = if total_sessions_30d > 0 {
        total_duration_30d as f32 / total_sessions_30d as f32
    } else {
        0.0
    };

    // Streaks
    let streak_data = db::get_streak_data(conn)?;

    // Top hours (last 30 days, top 3)
    let top_hours = {
        let hourly = db::get_time_of_day_stats(conn, &thirty_days_ago, &today)?;
        let mut sorted = hourly;
        sorted.sort_by(|a, b| b.count.cmp(&a.count));
        sorted.into_iter().take(3).map(|h| (h.hour, h.count)).collect::<Vec<_>>()
    };

    // Top projects (last 30 days, top 3)
    let top_projects = {
        let projects = db::get_project_stats(conn, &thirty_days_ago, &today)?;
        projects.into_iter().take(3).map(|p| (p.project, p.sessions)).collect::<Vec<_>>()
    };

    // Today's sessions
    let (today_sessions, today_work_secs): (i32, i32) = {
        let start = format!("{today}T00:00:00+00:00");
        let end = format!("{today}T23:59:59+00:00");
        let mut stmt = conn.prepare(
            "SELECT COUNT(*), COALESCE(SUM(duration_secs), 0)
             FROM pomodoro_sessions
             WHERE completed = 1 AND session_type = 'work'
               AND started_at >= ?1 AND started_at <= ?2",
        )?;
        stmt.query_row(params![start, end], |row| Ok((row.get(0)?, row.get(1)?)))?
    };

    // This week's sessions
    let week_start = {
        let weekday = now.weekday().num_days_from_monday();
        (now - chrono::Duration::days(weekday as i64)).format("%Y-%m-%d").to_string()
    };
    let (week_sessions, week_work_secs): (i32, i32) = {
        let start = format!("{week_start}T00:00:00+00:00");
        let end = format!("{today}T23:59:59+00:00");
        let mut stmt = conn.prepare(
            "SELECT COUNT(*), COALESCE(SUM(duration_secs), 0)
             FROM pomodoro_sessions
             WHERE completed = 1 AND session_type = 'work'
               AND started_at >= ?1 AND started_at <= ?2",
        )?;
        stmt.query_row(params![start, end], |row| Ok((row.get(0)?, row.get(1)?)))?
    };

    // Day of week
    let day_of_week = now.format("%A").to_string();

    // Consecutive sessions without a long break
    let consecutive_sessions_no_long_break = {
        let mut stmt = conn.prepare(
            "SELECT session_type FROM pomodoro_sessions
             WHERE completed = 1
             ORDER BY started_at DESC
             LIMIT 50",
        )?;
        let types: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        let mut count = 0i32;
        for t in &types {
            if t == "long_break" {
                break;
            }
            if t == "work" {
                count += 1;
            }
        }
        count
    };

    Ok(ProductivityContext {
        avg_daily_sessions,
        avg_session_duration_secs,
        current_streak: streak_data.current,
        best_streak: streak_data.best,
        top_hours,
        top_projects,
        today_sessions,
        today_work_secs,
        week_sessions,
        week_work_secs,
        day_of_week,
        consecutive_sessions_no_long_break,
    })
}

// ── Prompt Templates ───────────────────────────────────────────────────────

pub const DAILY_BRIEFING_PROMPT: &str = r#"You are a productivity coach analyzing Pomodoro data.
Generate a brief daily briefing (2-3 sentences) based on these stats:
{context}
Be encouraging, specific with numbers, and suggest one actionable tip for today.
Respond in plain text, no markdown."#;

pub const SESSION_DEBRIEF_PROMPT: &str = r#"You are a productivity coach. The user just completed a {session_type} session.
Stats: {context}
Give ONE brief sentence of feedback (max 20 words). Be encouraging or gently suggest a break if needed."#;

pub const WEEKLY_REPORT_PROMPT: &str = r#"You are a productivity coach analyzing weekly Pomodoro data.
Stats: {context}
Generate a weekly report with:
1. "highlights": 3 bullet points of achievements
2. "suggestions": 2 actionable improvement suggestions
3. "summary": 1 sentence overall summary
Respond as JSON: {"highlights":["..."],"suggestions":["..."],"summary":"..."}"#;

pub const SETTINGS_ADVISOR_PROMPT: &str = r#"You are a productivity coach analyzing Pomodoro settings.
Current settings: work_duration={work_min}min, short_break={short_min}min, long_break={long_min}min, cycles={cycles}
User stats: {context}
Suggest setting adjustments if the data supports it. Respond as JSON array:
[{"setting":"work_duration","current":25,"suggested":22,"reason":"Your focus drops after 22min on average"}]
If no changes needed, return empty array []."#;

// ── Response Structs ───────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct AiBriefing {
    pub message: String,
    pub stats_summary: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AiDebrief {
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AiReport {
    pub highlights: Vec<String>,
    pub suggestions: Vec<String>,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingAdvice {
    pub setting: String,
    pub current: i32,
    pub suggested: i32,
    pub reason: String,
}

// ── Rule-Based Fallbacks ───────────────────────────────────────────────────

pub fn get_rule_based_briefing(ctx: &ProductivityContext) -> String {
    if ctx.current_streak >= 7 {
        format!(
            "Amazing {}-day streak! You're averaging {:.1} sessions/day. Keep the momentum going today!",
            ctx.current_streak, ctx.avg_daily_sessions
        )
    } else if ctx.today_sessions as f32 > ctx.avg_daily_sessions && ctx.today_sessions > 0 {
        let diff = ctx.today_sessions as f32 - ctx.avg_daily_sessions;
        format!(
            "Productive day! {} sessions so far, {:.0} above your average. Great work!",
            ctx.today_sessions, diff
        )
    } else if ctx.today_sessions == 0 {
        if let Some((hour, _)) = ctx.top_hours.first() {
            format!(
                "Time to start! Your most productive hours are usually around {}:00. Let's get going!",
                hour
            )
        } else {
            "Ready for a fresh start! Begin your first session to build momentum.".to_string()
        }
    } else {
        format!(
            "You've completed {} sessions today. Keep going — consistency is key!",
            ctx.today_sessions
        )
    }
}

pub fn get_rule_based_debrief(ctx: &ProductivityContext, session_type: &str) -> String {
    if ctx.consecutive_sessions_no_long_break >= 3 && session_type == "work" {
        "Consider a longer break to recharge — you've been on a roll!".to_string()
    } else if ctx.today_sessions == 5 || ctx.today_sessions == 10 || ctx.today_sessions == 15 {
        format!("Milestone! {} sessions completed today!", ctx.today_sessions)
    } else {
        format!("Great session! {} completed today.", ctx.today_sessions)
    }
}

pub fn get_rule_based_report(ctx: &ProductivityContext) -> AiReport {
    let mut highlights = Vec::new();

    highlights.push(format!(
        "Completed {} sessions this week ({} minutes of focused work)",
        ctx.week_sessions,
        ctx.week_work_secs / 60
    ));

    if ctx.current_streak > 0 {
        highlights.push(format!(
            "Maintained a {}-day streak (best ever: {} days)",
            ctx.current_streak, ctx.best_streak
        ));
    } else {
        highlights.push("Start a new streak today — every day counts!".to_string());
    }

    if let Some((project, sessions)) = ctx.top_projects.first() {
        highlights.push(format!(
            "Most focused on '{}' with {} sessions",
            project, sessions
        ));
    } else {
        highlights.push("Assign projects to tasks to track focus areas.".to_string());
    }

    let mut suggestions = Vec::new();
    if ctx.avg_daily_sessions < 4.0 {
        suggestions.push("Try to reach at least 4 sessions per day for meaningful progress.".to_string());
    } else {
        suggestions.push("Your session count is solid — focus on session quality and deep work.".to_string());
    }

    if !ctx.top_hours.is_empty() {
        let best_hour = ctx.top_hours[0].0;
        suggestions.push(format!(
            "Schedule your most important work around {}:00 — that's your peak hour.",
            best_hour
        ));
    } else {
        suggestions.push("Track more sessions to discover your most productive hours.".to_string());
    }

    let summary = format!(
        "This week: {} sessions, {:.1} daily average, {}-day streak.",
        ctx.week_sessions, ctx.avg_daily_sessions, ctx.current_streak
    );

    AiReport {
        highlights,
        suggestions,
        summary,
    }
}

pub fn get_rule_based_advice(
    ctx: &ProductivityContext,
    settings: &HashMap<String, String>,
) -> Vec<SettingAdvice> {
    let mut advice = Vec::new();

    let work_min = settings
        .get("work_duration")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(25);

    let avg_focus_min = ctx.avg_session_duration_secs / 60.0;

    // If avg focus is less than 80% of work duration, suggest shorter work sessions
    if avg_focus_min > 0.0 && avg_focus_min < work_min as f32 * 0.8 {
        let suggested = (avg_focus_min.round() as i32).max(15);
        if suggested != work_min {
            advice.push(SettingAdvice {
                setting: "work_duration".to_string(),
                current: work_min,
                suggested,
                reason: format!(
                    "Your average focus lasts {:.0} minutes — a shorter session may improve completion rate",
                    avg_focus_min
                ),
            });
        }
    }

    advice
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AiError Display ─────────────────────────────────────────────────

    #[test]
    fn ai_error_display() {
        assert_eq!(AiError::ProviderDisabled.to_string(), "AI provider is disabled");
        assert_eq!(
            AiError::NetworkError("timeout".into()).to_string(),
            "Network error: timeout"
        );
        assert_eq!(
            AiError::ParseError("bad json".into()).to_string(),
            "Parse error: bad json"
        );
        assert_eq!(
            AiError::ConfigError("missing key".into()).to_string(),
            "Config error: missing key"
        );
    }

    // ── DisabledProvider ────────────────────────────────────────────────

    #[test]
    fn disabled_provider_returns_error() {
        let p = DisabledProvider;
        let result = p.generate("system", "user");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AiError::ProviderDisabled));
    }

    #[test]
    fn disabled_provider_name() {
        assert_eq!(DisabledProvider.name(), "disabled");
    }

    // ── OpenAI provider rejects empty key ───────────────────────────────

    #[test]
    fn openai_provider_rejects_empty_key() {
        let p = OpenAiProvider {
            api_key: String::new(),
            model: "gpt-4".to_string(),
        };
        let result = p.generate("system", "user");
        assert!(matches!(result.unwrap_err(), AiError::ConfigError(_)));
    }

    #[test]
    fn openai_provider_name() {
        let p = OpenAiProvider {
            api_key: "key".into(),
            model: "gpt-4".into(),
        };
        assert_eq!(p.name(), "openai");
    }

    // ── Anthropic provider rejects empty key ────────────────────────────

    #[test]
    fn anthropic_provider_rejects_empty_key() {
        let p = AnthropicProvider {
            api_key: String::new(),
            model: "claude-3".to_string(),
        };
        let result = p.generate("system", "user");
        assert!(matches!(result.unwrap_err(), AiError::ConfigError(_)));
    }

    #[test]
    fn anthropic_provider_name() {
        let p = AnthropicProvider {
            api_key: "key".into(),
            model: "claude-3".into(),
        };
        assert_eq!(p.name(), "anthropic");
    }

    // ── Ollama provider name ────────────────────────────────────────────

    #[test]
    fn ollama_provider_name() {
        let p = OllamaProvider {
            base_url: "http://localhost:11434".into(),
            model: "llama3".into(),
        };
        assert_eq!(p.name(), "ollama");
    }

    // ── Factory ─────────────────────────────────────────────────────────

    #[test]
    fn create_provider_ollama() {
        let p = create_provider("ollama", "", "llama3", "http://localhost:11434");
        assert_eq!(p.name(), "ollama");
    }

    #[test]
    fn create_provider_openai() {
        let p = create_provider("openai", "key", "gpt-4", "");
        assert_eq!(p.name(), "openai");
    }

    #[test]
    fn create_provider_anthropic() {
        let p = create_provider("anthropic", "key", "claude-3", "");
        assert_eq!(p.name(), "anthropic");
    }

    #[test]
    fn create_provider_unknown_falls_back_to_disabled() {
        let p = create_provider("unknown_provider", "", "", "");
        assert_eq!(p.name(), "disabled");
    }

    // ── strip_code_blocks ───────────────────────────────────────────────

    #[test]
    fn strip_code_blocks_plain_text() {
        assert_eq!(strip_code_blocks(r#"{"action":"start"}"#), r#"{"action":"start"}"#);
    }

    #[test]
    fn strip_code_blocks_json_fence() {
        let input = "```json\n{\"action\":\"start\"}\n```";
        assert_eq!(strip_code_blocks(input), "{\"action\":\"start\"}");
    }

    #[test]
    fn strip_code_blocks_plain_fence() {
        let input = "```\n{\"action\":\"start\"}\n```";
        assert_eq!(strip_code_blocks(input), "{\"action\":\"start\"}");
    }

    #[test]
    fn strip_code_blocks_no_trailing() {
        let input = "```json\n{\"action\":\"start\"}";
        assert_eq!(strip_code_blocks(input), "{\"action\":\"start\"}");
    }

    // ── parse_ai_command with mock provider ─────────────────────────────

    struct MockProvider {
        response: String,
    }

    impl AiProvider for MockProvider {
        fn generate(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, AiError> {
            Ok(self.response.clone())
        }
        fn name(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn parse_ai_command_valid_json() {
        let provider = MockProvider {
            response: r#"{"action":"create_task","params":{"title":"Test"},"message":"Created"}"#.to_string(),
        };
        let result = parse_ai_command(&provider, "create a task called Test").unwrap();
        assert_eq!(result.action, "create_task");
        assert_eq!(result.params.get("title").unwrap(), "Test");
        assert_eq!(result.message, "Created");
        assert!(result.success);
    }

    #[test]
    fn parse_ai_command_with_code_blocks() {
        let provider = MockProvider {
            response: "```json\n{\"action\":\"start_timer\",\"params\":{},\"message\":\"Started\"}\n```".to_string(),
        };
        let result = parse_ai_command(&provider, "start the timer").unwrap();
        assert_eq!(result.action, "start_timer");
    }

    #[test]
    fn parse_ai_command_missing_fields_uses_defaults() {
        let provider = MockProvider {
            response: r#"{}"#.to_string(),
        };
        let result = parse_ai_command(&provider, "something").unwrap();
        assert_eq!(result.action, "unknown");
        assert!(result.params.is_empty());
        assert_eq!(result.message, "Command processed");
    }

    #[test]
    fn parse_ai_command_invalid_json() {
        let provider = MockProvider {
            response: "not json at all".to_string(),
        };
        let result = parse_ai_command(&provider, "something");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AiError::ParseError(_)));
    }

    #[test]
    fn parse_ai_command_provider_error() {
        struct FailProvider;
        impl AiProvider for FailProvider {
            fn generate(&self, _: &str, _: &str) -> Result<String, AiError> {
                Err(AiError::NetworkError("connection refused".into()))
            }
            fn name(&self) -> &str {
                "fail"
            }
        }
        let result = parse_ai_command(&FailProvider, "test");
        assert!(matches!(result.unwrap_err(), AiError::NetworkError(_)));
    }

    // ── Rule-Based Briefing ─────────────────────────────────────────────

    fn base_context() -> ProductivityContext {
        ProductivityContext {
            avg_daily_sessions: 4.0,
            avg_session_duration_secs: 1500.0,
            current_streak: 3,
            best_streak: 10,
            top_hours: vec![(9, 5), (14, 3)],
            top_projects: vec![("Work".to_string(), 10)],
            today_sessions: 2,
            today_work_secs: 3000,
            week_sessions: 15,
            week_work_secs: 22500,
            day_of_week: "Monday".to_string(),
            consecutive_sessions_no_long_break: 1,
        }
    }

    #[test]
    fn briefing_streak_path() {
        let mut ctx = base_context();
        ctx.current_streak = 7;
        let msg = get_rule_based_briefing(&ctx);
        assert!(msg.contains("7-day streak"));
    }

    #[test]
    fn briefing_above_average_path() {
        let mut ctx = base_context();
        ctx.current_streak = 1;
        ctx.today_sessions = 6;
        ctx.avg_daily_sessions = 4.0;
        let msg = get_rule_based_briefing(&ctx);
        assert!(msg.contains("6 sessions"));
    }

    #[test]
    fn briefing_zero_sessions_with_top_hours() {
        let mut ctx = base_context();
        ctx.current_streak = 0;
        ctx.today_sessions = 0;
        let msg = get_rule_based_briefing(&ctx);
        assert!(msg.contains("9:00"));
    }

    #[test]
    fn briefing_zero_sessions_no_top_hours() {
        let mut ctx = base_context();
        ctx.current_streak = 0;
        ctx.today_sessions = 0;
        ctx.top_hours = vec![];
        let msg = get_rule_based_briefing(&ctx);
        assert!(msg.contains("fresh start"));
    }

    #[test]
    fn briefing_default_path() {
        let mut ctx = base_context();
        ctx.current_streak = 2;
        ctx.today_sessions = 1;
        ctx.avg_daily_sessions = 4.0;
        let msg = get_rule_based_briefing(&ctx);
        assert!(msg.contains("1 sessions"));
    }

    // ── Rule-Based Debrief ──────────────────────────────────────────────

    #[test]
    fn debrief_suggests_long_break() {
        let mut ctx = base_context();
        ctx.consecutive_sessions_no_long_break = 3;
        let msg = get_rule_based_debrief(&ctx, "work");
        assert!(msg.contains("longer break"));
    }

    #[test]
    fn debrief_milestone_at_5() {
        let mut ctx = base_context();
        ctx.today_sessions = 5;
        ctx.consecutive_sessions_no_long_break = 0;
        let msg = get_rule_based_debrief(&ctx, "work");
        assert!(msg.contains("Milestone"));
    }

    #[test]
    fn debrief_generic() {
        let mut ctx = base_context();
        ctx.today_sessions = 3;
        ctx.consecutive_sessions_no_long_break = 0;
        let msg = get_rule_based_debrief(&ctx, "work");
        assert!(msg.contains("Great session"));
    }

    // ── Rule-Based Report ───────────────────────────────────────────────

    #[test]
    fn report_has_three_highlights() {
        let ctx = base_context();
        let report = get_rule_based_report(&ctx);
        assert_eq!(report.highlights.len(), 3);
        assert_eq!(report.suggestions.len(), 2);
        assert!(!report.summary.is_empty());
    }

    #[test]
    fn report_no_streak() {
        let mut ctx = base_context();
        ctx.current_streak = 0;
        let report = get_rule_based_report(&ctx);
        assert!(report.highlights[1].contains("Start a new streak"));
    }

    #[test]
    fn report_no_projects() {
        let mut ctx = base_context();
        ctx.top_projects = vec![];
        let report = get_rule_based_report(&ctx);
        assert!(report.highlights[2].contains("Assign projects"));
    }

    // ── Rule-Based Advice ───────────────────────────────────────────────

    #[test]
    fn advice_suggests_shorter_sessions() {
        let mut ctx = base_context();
        ctx.avg_session_duration_secs = 900.0; // 15 min avg, work_duration default 25
        let settings: HashMap<String, String> =
            [("work_duration".to_string(), "25".to_string())]
                .into_iter()
                .collect();
        let advice = get_rule_based_advice(&ctx, &settings);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].setting, "work_duration");
        assert_eq!(advice[0].suggested, 15);
    }

    #[test]
    fn advice_no_change_when_focus_is_good() {
        let ctx = base_context(); // 1500s = 25min, work_duration default 25
        let settings: HashMap<String, String> =
            [("work_duration".to_string(), "25".to_string())]
                .into_iter()
                .collect();
        let advice = get_rule_based_advice(&ctx, &settings);
        assert!(advice.is_empty());
    }

    #[test]
    fn advice_no_change_when_no_sessions() {
        let mut ctx = base_context();
        ctx.avg_session_duration_secs = 0.0;
        let settings: HashMap<String, String> = HashMap::new();
        let advice = get_rule_based_advice(&ctx, &settings);
        assert!(advice.is_empty());
    }

    // ── build_context with in-memory DB ─────────────────────────────────

    #[test]
    fn build_context_empty_db() {
        let conn = crate::db::init_in_memory().unwrap();
        let ctx = build_context(&conn).unwrap();
        assert_eq!(ctx.today_sessions, 0);
        assert_eq!(ctx.week_sessions, 0);
        assert_eq!(ctx.current_streak, 0);
        assert_eq!(ctx.best_streak, 0);
        assert!(ctx.top_hours.is_empty());
        assert!(ctx.top_projects.is_empty());
    }
}
