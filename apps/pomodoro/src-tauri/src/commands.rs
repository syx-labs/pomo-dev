use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::ai;
use crate::audio::{AudioEngine, AudioState, PresetLayer, SoundPreset};
use crate::db;
use crate::error::MutexExt;
use crate::integrations::{AppEvent, EventDispatcher, EventLogEntry, IntegrationConfig};
use crate::models::{
    Goal, HeatmapEntry, HourlyStats, PomodoroSession, ProjectStats, SessionType, StreakData, Task,
    TaskStatus, TimerState,
};
use crate::ollama;
use crate::timer::PomodoroTimer;

pub struct AppState {
    pub timer: Mutex<PomodoroTimer>,
    pub db: Mutex<Connection>,
    pub audio: Mutex<AudioEngine>,
    pub shutdown: Arc<AtomicBool>,
    pub ollama_cancel: Arc<AtomicBool>,
    pub dispatcher: EventDispatcher,
}

// ── Timer Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn start_timer(state: State<'_, AppState>) -> Result<TimerState, String> {
    let (result, session_type, linked_task_id) = {
        let mut timer = state.timer.lock_or_err("start_timer")?;
        timer.start();
        let session_type = timer.session_type.as_str().to_string();
        let linked_task_id = timer.linked_task_id.clone();
        (timer.get_state(), session_type, linked_task_id)
    };

    state.dispatcher.dispatch(
        &state.db,
        AppEvent {
            event_type: "session.started".to_string(),
            payload: serde_json::json!({
                "session_type": session_type,
                "linked_task_id": linked_task_id,
            }),
            timestamp: Utc::now().to_rfc3339(),
        },
    );

    Ok(result)
}

#[tauri::command]
pub fn pause_timer(state: State<'_, AppState>) -> Result<TimerState, String> {
    let (result, remaining_secs) = {
        let mut timer = state.timer.lock_or_err("pause_timer")?;
        timer.pause();
        let remaining = timer.remaining_secs;
        (timer.get_state(), remaining)
    };

    state.dispatcher.dispatch(
        &state.db,
        AppEvent {
            event_type: "session.paused".to_string(),
            payload: serde_json::json!({
                "remaining_secs": remaining_secs,
            }),
            timestamp: Utc::now().to_rfc3339(),
        },
    );

    Ok(result)
}

#[tauri::command]
pub fn resume_timer(state: State<'_, AppState>) -> Result<TimerState, String> {
    let mut timer = state.timer.lock_or_err("resume_timer")?;
    timer.resume();
    Ok(timer.get_state())
}

#[tauri::command]
pub fn skip_timer(state: State<'_, AppState>) -> Result<TimerState, String> {
    let mut timer = state.timer.lock_or_err("skip_timer")?;
    timer.skip();
    Ok(timer.get_state())
}

#[tauri::command]
pub fn reset_timer(state: State<'_, AppState>) -> Result<TimerState, String> {
    let mut timer = state.timer.lock_or_err("reset_timer")?;
    timer.reset();
    Ok(timer.get_state())
}

#[tauri::command]
pub fn get_timer_state(state: State<'_, AppState>) -> Result<TimerState, String> {
    let timer = state.timer.lock_or_err("get_timer_state")?;
    Ok(timer.get_state())
}

// ── Task Commands ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub project: Option<String>,
    pub due_date: Option<String>,
    pub estimated_pomos: Option<i32>,
}

#[tauri::command]
pub fn create_task(
    state: State<'_, AppState>,
    input: CreateTaskInput,
) -> Result<Task, String> {
    let now = Utc::now().to_rfc3339();
    let task = Task {
        id: Uuid::new_v4().to_string(),
        title: input.title,
        description: input.description,
        status: TaskStatus::Todo,
        priority: input.priority.unwrap_or(0),
        project: input.project,
        due_date: input.due_date,
        estimated_pomos: input.estimated_pomos.unwrap_or(0),
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
    };

    {
        let conn = state.db.lock_or_err("create_task")?;
        db::create_task(&conn, &task).map_err(|e| e.to_string())?;
    }

    state.dispatcher.dispatch(
        &state.db,
        AppEvent {
            event_type: "task.created".to_string(),
            payload: serde_json::json!({
                "title": task.title,
                "project": task.project,
                "priority": task.priority,
            }),
            timestamp: Utc::now().to_rfc3339(),
        },
    );

    Ok(task)
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskInput {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub project: Option<String>,
    pub due_date: Option<String>,
    pub estimated_pomos: i32,
    pub sort_order: i32,
}

#[tauri::command]
pub fn update_task(
    state: State<'_, AppState>,
    input: UpdateTaskInput,
) -> Result<Task, String> {
    // Parse the status string into the enum, keeping IPC contract as string
    let status = match input.status.as_str() {
        "todo" => TaskStatus::Todo,
        "in_progress" => TaskStatus::InProgress,
        "done" => TaskStatus::Done,
        "archived" => TaskStatus::Archived,
        other => return Err(format!("Invalid task status: {other}")),
    };

    let now = Utc::now().to_rfc3339();
    let task = Task {
        id: input.id,
        title: input.title,
        description: input.description,
        status,
        priority: input.priority,
        project: input.project,
        due_date: input.due_date,
        estimated_pomos: input.estimated_pomos,
        sort_order: input.sort_order,
        created_at: String::new(), // not updated
        updated_at: now,
    };

    let old_status = {
        let conn = state.db.lock_or_err("update_task_read")?;
        let old = db::get_task_status(&conn, &task.id).map_err(|e| e.to_string())?;
        db::update_task(&conn, &task).map_err(|e| e.to_string())?;
        old
    };

    // Emit task.completed only when transitioning TO "done"
    if task.status == TaskStatus::Done && old_status.as_deref() != Some("done") {
        state.dispatcher.dispatch(
            &state.db,
            AppEvent {
                event_type: "task.completed".to_string(),
                payload: serde_json::json!({
                    "title": task.title,
                    "project": task.project,
                }),
                timestamp: Utc::now().to_rfc3339(),
            },
        );
    }

    Ok(task)
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock_or_err("delete_task")?;
    db::delete_task(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tasks(
    state: State<'_, AppState>,
    status: Option<String>,
    project: Option<String>,
) -> Result<Vec<Task>, String> {
    let conn = state.db.lock_or_err("list_tasks")?;
    db::list_tasks(
        &conn,
        status.as_deref(),
        project.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_tasks(
    state: State<'_, AppState>,
    task_ids: Vec<String>,
) -> Result<(), String> {
    let conn = state.db.lock_or_err("reorder_tasks")?;
    db::reorder_tasks(&conn, &task_ids).map_err(|e| e.to_string())
}

// ── Session Commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn start_pomodoro_for_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<TimerState, String> {
    // Phase 2B: Lock timer first, modify in-memory state, get what we need, drop lock
    let (session_id, session_type_str, duration) = {
        let mut timer = state.timer.lock_or_err("start_pomodoro_for_task")?;
        let session_id = Uuid::new_v4().to_string();
        let session_type_str = timer.session_type.as_str().to_string();
        let duration = timer.settings.work_duration;

        timer.linked_task_id = Some(task_id.clone());
        timer.current_session_id = Some(session_id.clone());
        timer.start();

        (session_id, session_type_str, duration)
    };
    // Timer lock is dropped here

    // Parse session type for the DB record
    let session_type = match session_type_str.as_str() {
        "work" => SessionType::Work,
        "short_break" => SessionType::ShortBreak,
        "long_break" => SessionType::LongBreak,
        _ => SessionType::Work,
    };

    let now = Utc::now().to_rfc3339();
    let session = PomodoroSession {
        id: session_id,
        task_id: Some(task_id),
        session_type,
        duration_secs: duration,
        started_at: now,
        ended_at: None,
        completed: false,
        notes: None,
    };

    // Try to persist to DB; if it fails, rollback the timer
    let conn = state.db.lock_or_err("start_pomodoro_for_task_db")?;
    if let Err(e) = db::create_session(&conn, &session) {
        drop(conn);
        let mut timer = state.timer.lock_or_err("start_pomodoro_for_task_rollback")?;
        timer.reset();
        return Err(format!("Database error: {e}"));
    }
    drop(conn);

    let timer = state.timer.lock_or_err("start_pomodoro_for_task_state")?;
    Ok(timer.get_state())
}

#[tauri::command]
pub fn complete_pomodoro(
    state: State<'_, AppState>,
) -> Result<TimerState, String> {
    let (session_id, was_running, session_type) = {
        let mut timer = state.timer.lock_or_err("complete_pomodoro")?;
        let was_running = timer.status == crate::models::TimerStatus::Running
            || timer.status == crate::models::TimerStatus::Paused;
        let session_id = timer.current_session_id.clone();
        let session_type = timer.session_type.as_str().to_string();

        // Reset the timer (advances to idle, clears linked task)
        timer.reset();

        (session_id, was_running, session_type)
    };

    // Mark session as completed in DB if there was an active session
    if was_running {
        if let Some(sid) = session_id {
            let ended_at = Utc::now().to_rfc3339();
            let conn = state.db.lock_or_err("complete_pomodoro_db")?;
            let _ = db::complete_session(&conn, &sid, &ended_at);
        }
    }

    let result = {
        let timer = state.timer.lock_or_err("complete_pomodoro_state")?;
        timer.get_state()
    };

    if was_running {
        state.dispatcher.dispatch(
            &state.db,
            AppEvent {
                event_type: "session.completed".to_string(),
                payload: serde_json::json!({
                    "session_type": session_type,
                }),
                timestamp: Utc::now().to_rfc3339(),
            },
        );
    }

    Ok(result)
}

#[tauri::command]
pub fn get_sessions_for_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<PomodoroSession>, String> {
    let conn = state.db.lock_or_err("get_sessions_for_task")?;
    db::get_sessions_for_task(&conn, &task_id).map_err(|e| e.to_string())
}

// ── Settings Commands ───────────────────────────────────────────────────────

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let conn = state.db.lock_or_err("get_setting")?;
    db::get_setting(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.db.lock_or_err("set_setting")?;
    db::set_setting(&conn, &key, &value).map_err(|e| e.to_string())?;
    drop(conn);

    // Phase 2D: Sync timer-related settings live
    // Frontend stores durations in MINUTES; Rust timer uses SECONDS
    match key.as_str() {
        "work_duration" | "short_break_duration" | "long_break_duration"
        | "cycles_before_long_break" => {
            if let Ok(parsed) = value.parse::<i32>() {
                if let Ok(mut timer) = state.timer.lock_or_err("set_setting_timer") {
                    match key.as_str() {
                        "work_duration" => timer.settings.work_duration = parsed * 60,
                        "short_break_duration" => timer.settings.short_break = parsed * 60,
                        "long_break_duration" => timer.settings.long_break = parsed * 60,
                        "cycles_before_long_break" => {
                            timer.settings.cycles_before_long_break = parsed
                        }
                        _ => {}
                    }
                    // If idle, update remaining_secs to reflect new duration
                    if timer.status == crate::models::TimerStatus::Idle {
                        let new_remaining = match timer.session_type {
                            SessionType::Work => timer.settings.work_duration,
                            SessionType::ShortBreak => timer.settings.short_break,
                            SessionType::LongBreak => timer.settings.long_break,
                        };
                        timer.remaining_secs = new_remaining;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub fn get_all_settings(
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    let conn = state.db.lock_or_err("get_all_settings")?;
    let settings = db::get_all_settings(&conn).map_err(|e| e.to_string())?;
    Ok(settings.into_iter().collect())
}

// ── Stats Commands ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DailyStats {
    pub date: String,
    pub total_sessions: usize,
    pub total_work_secs: i32,
    pub total_break_secs: i32,
}

#[tauri::command]
pub fn get_daily_stats(
    state: State<'_, AppState>,
    date: String,
) -> Result<DailyStats, String> {
    let conn = state.db.lock_or_err("get_daily_stats")?;

    // date is expected as "YYYY-MM-DD"
    let start = format!("{date}T00:00:00+00:00");
    let end = format!("{date}T23:59:59+00:00");

    let sessions = db::get_sessions_in_range(&conn, &start, &end).map_err(|e| e.to_string())?;

    let total_sessions = sessions.len();
    let mut total_work_secs = 0i32;
    let mut total_break_secs = 0i32;

    for s in &sessions {
        if s.session_type == SessionType::Work {
            total_work_secs += s.duration_secs;
        } else {
            total_break_secs += s.duration_secs;
        }
    }

    Ok(DailyStats {
        date,
        total_sessions,
        total_work_secs,
        total_break_secs,
    })
}

#[derive(Debug, Serialize)]
pub struct WeeklyStats {
    pub days: Vec<DailyStats>,
    pub total_sessions: usize,
    pub total_work_secs: i32,
}

#[tauri::command]
pub fn get_weekly_stats(
    state: State<'_, AppState>,
    start_date: String,
) -> Result<WeeklyStats, String> {
    let conn = state.db.lock_or_err("get_weekly_stats")?;

    let start_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?;

    let end_naive = start_naive + chrono::Duration::days(7);
    let start_ts = format!("{}T00:00:00+00:00", start_naive.format("%Y-%m-%d"));
    let end_ts = format!("{}T00:00:00+00:00", end_naive.format("%Y-%m-%d"));

    // Phase 4: Single aggregated query instead of 7 separate queries
    let aggregated =
        db::get_weekly_stats_aggregated(&conn, &start_ts, &end_ts).map_err(|e| e.to_string())?;

    // Build a map of day -> (work_secs, break_secs, session_count)
    let mut day_map: HashMap<String, (i32, i32, usize)> = HashMap::new();
    for stat in &aggregated {
        let entry = day_map.entry(stat.day.clone()).or_insert((0, 0, 0));
        entry.2 += stat.count;
        if stat.session_type == SessionType::Work {
            entry.0 += stat.total_secs;
        } else {
            entry.1 += stat.total_secs;
        }
    }

    let mut days = Vec::new();
    let mut total_sessions = 0usize;
    let mut total_work_secs = 0i32;

    for i in 0..7 {
        let day = start_naive + chrono::Duration::days(i);
        let day_str = day.format("%Y-%m-%d").to_string();

        let (day_work, day_break, day_count) =
            day_map.get(&day_str).copied().unwrap_or((0, 0, 0));

        total_sessions += day_count;
        total_work_secs += day_work;

        days.push(DailyStats {
            date: day_str,
            total_sessions: day_count,
            total_work_secs: day_work,
            total_break_secs: day_break,
        });
    }

    Ok(WeeklyStats {
        days,
        total_sessions,
        total_work_secs,
    })
}

// ── Analytics Commands ──────────────────────────────────────────────────────

#[tauri::command]
pub fn get_heatmap_data(
    state: State<'_, AppState>,
    year: i32,
) -> Result<Vec<HeatmapEntry>, String> {
    let conn = state.db.lock_or_err("get_heatmap_data")?;
    db::get_heatmap_data(&conn, year).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_streak_data(state: State<'_, AppState>) -> Result<StreakData, String> {
    let conn = state.db.lock_or_err("get_streak_data")?;
    db::get_streak_data(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_time_of_day_stats(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<HourlyStats>, String> {
    let conn = state.db.lock_or_err("get_time_of_day_stats")?;
    db::get_time_of_day_stats(&conn, &start_date, &end_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_stats(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ProjectStats>, String> {
    let conn = state.db.lock_or_err("get_project_stats")?;
    db::get_project_stats(&conn, &start_date, &end_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_goal(
    state: State<'_, AppState>,
    goal_type: String,
    target: i32,
) -> Result<Goal, String> {
    let now = Utc::now().to_rfc3339();
    let goal = Goal {
        id: Uuid::new_v4().to_string(),
        goal_type,
        target,
        created_at: now.clone(),
        updated_at: now,
    };

    let conn = state.db.lock_or_err("set_goal")?;
    db::create_goal(&conn, &goal).map_err(|e| e.to_string())?;

    Ok(goal)
}

#[tauri::command]
pub fn get_goals(state: State<'_, AppState>) -> Result<Vec<Goal>, String> {
    let conn = state.db.lock_or_err("get_goals")?;
    db::get_goals(&conn).map_err(|e| e.to_string())
}

// ── AI Commands ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn invoke_ai_command(
    state: State<'_, AppState>,
    prompt: String,
) -> Result<ai::AiCommandResult, String> {
    let conn = state.db.lock_or_err("invoke_ai_command")?;
    let provider_type = db::get_setting(&conn, "ai_provider")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "disabled".into());
    let api_key = db::get_setting(&conn, "ai_api_key")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let model = db::get_setting(&conn, "ai_model")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let base_url = db::get_setting(&conn, "ai_base_url")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "http://localhost:11434".into());
    drop(conn); // Release lock before HTTP call

    let provider = ai::create_provider(&provider_type, &api_key, &model, &base_url);
    ai::parse_ai_command(provider.as_ref(), &prompt).map_err(|e| e.to_string())
}

// ── AI Coach Commands ────────────────────────────────────────────────────

#[tauri::command]
pub fn get_daily_briefing(state: State<'_, AppState>) -> Result<ai::AiBriefing, String> {
    let (ctx, provider_type, api_key, model, base_url) = {
        let conn = state.db.lock_or_err("get_daily_briefing")?;
        let ctx = ai::build_context(&conn).map_err(|e| e.to_string())?;
        let provider_type = db::get_setting(&conn, "ai_provider")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "disabled".into());
        let api_key = db::get_setting(&conn, "ai_api_key")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let model = db::get_setting(&conn, "ai_model")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let base_url = db::get_setting(&conn, "ai_base_url")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "http://localhost:11434".into());
        (ctx, provider_type, api_key, model, base_url)
    };

    let message = if provider_type == "disabled" {
        ai::get_rule_based_briefing(&ctx)
    } else {
        let provider = ai::create_provider(&provider_type, &api_key, &model, &base_url);
        let prompt = ai::DAILY_BRIEFING_PROMPT
            .replace("{context}", &serde_json::to_string(&ctx).unwrap_or_default());
        provider
            .generate("You are a productivity coach.", &prompt)
            .map_err(|e| format!("{e}"))?
    };

    let stats_summary = format!(
        "Today: {} sessions | Streak: {} days | This week: {} sessions",
        ctx.today_sessions, ctx.current_streak, ctx.week_sessions
    );

    Ok(ai::AiBriefing {
        message,
        stats_summary,
    })
}

#[tauri::command]
pub fn get_session_debrief(
    state: State<'_, AppState>,
    session_type: String,
) -> Result<ai::AiDebrief, String> {
    let (ctx, provider_type, api_key, model, base_url) = {
        let conn = state.db.lock_or_err("get_session_debrief")?;
        let ctx = ai::build_context(&conn).map_err(|e| e.to_string())?;
        let provider_type = db::get_setting(&conn, "ai_provider")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "disabled".into());
        let api_key = db::get_setting(&conn, "ai_api_key")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let model = db::get_setting(&conn, "ai_model")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let base_url = db::get_setting(&conn, "ai_base_url")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "http://localhost:11434".into());
        (ctx, provider_type, api_key, model, base_url)
    };

    let message = if provider_type == "disabled" {
        ai::get_rule_based_debrief(&ctx, &session_type)
    } else {
        let provider = ai::create_provider(&provider_type, &api_key, &model, &base_url);
        let prompt = ai::SESSION_DEBRIEF_PROMPT
            .replace("{session_type}", &session_type)
            .replace("{context}", &serde_json::to_string(&ctx).unwrap_or_default());
        provider
            .generate("You are a productivity coach.", &prompt)
            .map_err(|e| format!("{e}"))?
    };

    Ok(ai::AiDebrief { message })
}

#[tauri::command]
pub fn get_weekly_report(state: State<'_, AppState>) -> Result<ai::AiReport, String> {
    let (ctx, provider_type, api_key, model, base_url) = {
        let conn = state.db.lock_or_err("get_weekly_report")?;
        let ctx = ai::build_context(&conn).map_err(|e| e.to_string())?;
        let provider_type = db::get_setting(&conn, "ai_provider")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "disabled".into());
        let api_key = db::get_setting(&conn, "ai_api_key")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let model = db::get_setting(&conn, "ai_model")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let base_url = db::get_setting(&conn, "ai_base_url")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "http://localhost:11434".into());
        (ctx, provider_type, api_key, model, base_url)
    };

    if provider_type == "disabled" {
        return Ok(ai::get_rule_based_report(&ctx));
    }

    let provider = ai::create_provider(&provider_type, &api_key, &model, &base_url);
    let prompt = ai::WEEKLY_REPORT_PROMPT
        .replace("{context}", &serde_json::to_string(&ctx).unwrap_or_default());
    let response = provider
        .generate("You are a productivity coach.", &prompt)
        .map_err(|e| format!("{e}"))?;

    let cleaned = strip_ai_code_blocks(&response);
    Ok(serde_json::from_str::<ai::AiReport>(cleaned)
        .unwrap_or_else(|_| ai::get_rule_based_report(&ctx)))
}

#[tauri::command]
pub fn get_settings_advice(state: State<'_, AppState>) -> Result<Vec<ai::SettingAdvice>, String> {
    let (ctx, settings, provider_type, api_key, model, base_url) = {
        let conn = state.db.lock_or_err("get_settings_advice")?;
        let ctx = ai::build_context(&conn).map_err(|e| e.to_string())?;
        let settings_vec = db::get_all_settings(&conn).map_err(|e| e.to_string())?;
        let settings: HashMap<String, String> = settings_vec.into_iter().collect();
        let provider_type = settings
            .get("ai_provider")
            .cloned()
            .unwrap_or_else(|| "disabled".into());
        let api_key = settings.get("ai_api_key").cloned().unwrap_or_default();
        let model_val = settings.get("ai_model").cloned().unwrap_or_default();
        let base_url = settings
            .get("ai_base_url")
            .cloned()
            .unwrap_or_else(|| "http://localhost:11434".into());
        (ctx, settings, provider_type, api_key, model_val, base_url)
    };

    if provider_type == "disabled" {
        return Ok(ai::get_rule_based_advice(&ctx, &settings));
    }

    let provider = ai::create_provider(&provider_type, &api_key, &model, &base_url);

    let work_min = settings.get("work_duration").and_then(|v| v.parse::<i32>().ok()).unwrap_or(25);
    let short_min = settings.get("short_break").and_then(|v| v.parse::<i32>().ok()).unwrap_or(5);
    let long_min = settings.get("long_break").and_then(|v| v.parse::<i32>().ok()).unwrap_or(15);
    let cycles = settings.get("cycles_before_long_break").and_then(|v| v.parse::<i32>().ok()).unwrap_or(4);

    let prompt = ai::SETTINGS_ADVISOR_PROMPT
        .replace("{work_min}", &work_min.to_string())
        .replace("{short_min}", &short_min.to_string())
        .replace("{long_min}", &long_min.to_string())
        .replace("{cycles}", &cycles.to_string())
        .replace("{context}", &serde_json::to_string(&ctx).unwrap_or_default());

    let response = provider
        .generate("You are a productivity coach.", &prompt)
        .map_err(|e| format!("{e}"))?;

    let cleaned = strip_ai_code_blocks(&response);
    Ok(serde_json::from_str::<Vec<ai::SettingAdvice>>(cleaned)
        .unwrap_or_else(|_| ai::get_rule_based_advice(&ctx, &settings)))
}

fn strip_ai_code_blocks(input: &str) -> &str {
    let trimmed = input.trim();
    let inner = if trimmed.starts_with("```json") {
        trimmed.strip_prefix("```json").unwrap_or(trimmed)
    } else if trimmed.starts_with("```") {
        trimmed.strip_prefix("```").unwrap_or(trimmed)
    } else {
        return trimmed;
    };
    inner.strip_suffix("```").unwrap_or(inner).trim()
}

// ── Sound Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn play_sound(
    state: State<'_, AppState>,
    name: String,
    volume: f32,
) -> Result<AudioState, String> {
    let mut audio = state.audio.lock_or_err("play_sound")?;
    audio.play_layer(&name, volume)?;
    Ok(audio.get_state())
}

#[tauri::command]
pub fn stop_sound(state: State<'_, AppState>, name: String) -> Result<AudioState, String> {
    let mut audio = state.audio.lock_or_err("stop_sound")?;
    audio.stop_layer(&name);
    Ok(audio.get_state())
}

#[tauri::command]
pub fn set_sound_volume(
    state: State<'_, AppState>,
    name: String,
    volume: f32,
) -> Result<AudioState, String> {
    let mut audio = state.audio.lock_or_err("set_sound_volume")?;
    audio.set_layer_volume(&name, volume);
    Ok(audio.get_state())
}

#[tauri::command]
pub fn set_master_volume(
    state: State<'_, AppState>,
    volume: f32,
) -> Result<AudioState, String> {
    let mut audio = state.audio.lock_or_err("set_master_volume")?;
    audio.set_master_volume(volume);
    Ok(audio.get_state())
}

#[tauri::command]
pub fn fade_sounds(
    state: State<'_, AppState>,
    direction: String,
) -> Result<AudioState, String> {
    let mut audio = state.audio.lock_or_err("fade_sounds")?;
    match direction.as_str() {
        "out" => audio.fade_out(),
        "in" => audio.fade_in(),
        other => return Err(format!("Invalid fade direction: {other}")),
    }
    Ok(audio.get_state())
}

#[tauri::command]
pub fn stop_all_sounds(state: State<'_, AppState>) -> Result<AudioState, String> {
    let mut audio = state.audio.lock_or_err("stop_all_sounds")?;
    audio.stop_all();
    Ok(audio.get_state())
}

#[tauri::command]
pub fn get_audio_state(state: State<'_, AppState>) -> Result<AudioState, String> {
    let audio = state.audio.lock_or_err("get_audio_state")?;
    Ok(audio.get_state())
}

#[tauri::command]
pub fn get_available_sounds(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let audio = state.audio.lock_or_err("get_available_sounds")?;
    Ok(audio.available_sounds())
}

#[tauri::command]
pub fn save_sound_preset(
    state: State<'_, AppState>,
    name: String,
    layers: String,
) -> Result<SoundPreset, String> {
    let now = Utc::now().to_rfc3339();
    let preset = SoundPreset {
        id: Uuid::new_v4().to_string(),
        name,
        layers,
        is_default: false,
        created_at: now,
    };

    let conn = state.db.lock_or_err("save_sound_preset")?;
    db::save_preset(&conn, &preset).map_err(|e| e.to_string())?;

    Ok(preset)
}

#[tauri::command]
pub fn list_sound_presets(state: State<'_, AppState>) -> Result<Vec<SoundPreset>, String> {
    let conn = state.db.lock_or_err("list_sound_presets")?;
    db::get_presets(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_sound_preset(
    state: State<'_, AppState>,
    id: String,
) -> Result<AudioState, String> {
    let conn = state.db.lock_or_err("load_sound_preset_db")?;
    let presets = db::get_presets(&conn).map_err(|e| e.to_string())?;
    drop(conn);

    let preset = presets
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("Preset not found: {id}"))?;

    let preset_layers: Vec<PresetLayer> =
        serde_json::from_str(&preset.layers).map_err(|e| format!("Invalid preset layers: {e}"))?;

    let mut audio = state.audio.lock_or_err("load_sound_preset_audio")?;
    audio.stop_all();

    for pl in &preset_layers {
        if let Err(e) = audio.play_layer(&pl.sound, pl.volume) {
            eprintln!("Warning: failed to play layer '{}': {e}", pl.sound);
        }
    }

    Ok(audio.get_state())
}

#[tauri::command]
pub fn delete_sound_preset(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock_or_err("delete_sound_preset")?;
    let rows = db::delete_preset(&conn, &id).map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err("Cannot delete default presets".to_string());
    }
    Ok(())
}

// ── Integration Commands ────────────────────────────────────────────────────

#[tauri::command]
pub fn create_integration(
    state: State<'_, AppState>,
    integration_type: String,
    name: String,
    config: String,
    events: String,
) -> Result<IntegrationConfig, String> {
    let now = Utc::now().to_rfc3339();
    let integration = IntegrationConfig {
        id: Uuid::new_v4().to_string(),
        integration_type,
        name,
        config,
        enabled: true,
        events,
        created_at: now.clone(),
        updated_at: now,
    };

    let conn = state.db.lock_or_err("create_integration")?;
    db::create_integration(&conn, &integration).map_err(|e| e.to_string())?;

    Ok(integration)
}

#[tauri::command]
pub fn update_integration(
    state: State<'_, AppState>,
    id: String,
    name: String,
    config: String,
    events: String,
    enabled: bool,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    let conn = state.db.lock_or_err("update_integration")?;
    db::update_integration(&conn, &id, &name, &config, &events, enabled, &now)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_integration(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock_or_err("delete_integration")?;
    db::delete_integration(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_integrations(state: State<'_, AppState>) -> Result<Vec<IntegrationConfig>, String> {
    let conn = state.db.lock_or_err("list_integrations")?;
    db::get_integrations(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn test_integration(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let config_data = {
        let conn = state.db.lock_or_err("test_integration")?;
        let integrations = db::get_integrations(&conn).map_err(|e| e.to_string())?;
        integrations
            .into_iter()
            .find(|i| i.id == id)
            .ok_or_else(|| format!("Integration not found: {id}"))?
    };

    match config_data.integration_type.as_str() {
        "webhook" => crate::integrations::webhook::test_webhook(&config_data.config),
        "slack" => crate::integrations::slack::test_slack(&config_data.config),
        "discord" => crate::integrations::discord::test_discord(&config_data.config),
        other => Err(format!("Unknown integration type: {other}")),
    }
}

#[tauri::command]
pub fn get_event_log(
    state: State<'_, AppState>,
    integration_id: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<EventLogEntry>, String> {
    let conn = state.db.lock_or_err("get_event_log")?;
    db::get_event_log(&conn, integration_id.as_deref(), limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

// ── Ollama Model Commands ────────────────────────────────────────────────────

fn get_ollama_base_url(conn: &Connection) -> Result<String, String> {
    db::get_setting(conn, "ai_base_url")
        .map_err(|e| e.to_string())
        .map(|v| match v {
            Some(s) if !s.trim().is_empty() => s,
            _ => "http://localhost:11434".into(),
        })
}

#[tauri::command]
pub fn ollama_check_health(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.db.lock_or_err("ollama_check_health")?;
    let base_url = get_ollama_base_url(&conn)?;
    drop(conn);
    Ok(ollama::check_health(&base_url))
}

#[tauri::command]
pub fn ollama_list_local_models(
    state: State<'_, AppState>,
) -> Result<Vec<ollama::OllamaModel>, String> {
    let conn = state.db.lock_or_err("ollama_list_local_models")?;
    let base_url = get_ollama_base_url(&conn)?;
    drop(conn);
    ollama::list_local_models(&base_url)
}

#[tauri::command]
pub fn ollama_get_curated_models() -> Vec<ollama::CuratedModel> {
    ollama::get_curated_models()
}

#[tauri::command]
pub fn ollama_pull_model(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let conn = state.db.lock_or_err("ollama_pull_model")?;
    let base_url = get_ollama_base_url(&conn)?;
    drop(conn);

    state.ollama_cancel.store(false, Ordering::Relaxed);
    let cancel = state.ollama_cancel.clone();

    ollama::pull_model(app_handle, base_url, name, cancel);
    Ok(())
}

#[tauri::command]
pub fn ollama_cancel_pull(state: State<'_, AppState>) -> Result<(), String> {
    state.ollama_cancel.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn ollama_delete_model(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let conn = state.db.lock_or_err("ollama_delete_model")?;
    let base_url = get_ollama_base_url(&conn)?;
    drop(conn);
    ollama::delete_model(&base_url, &name)
}
