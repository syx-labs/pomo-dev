use rusqlite::{params, Connection, Result};
use tauri::Manager;

use crate::audio::SoundPreset;
use crate::integrations::{EventLogEntry, IntegrationConfig};
use crate::models::{Goal, HeatmapEntry, HourlyStats, PomodoroSession, ProjectStats, SessionType, StreakData, Task};

pub fn init_db(app_handle: &tauri::AppHandle) -> Result<Connection> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    let db_path = app_data_dir.join("pomodoro.db");
    let conn = Connection::open(db_path)?;

    run_migrations(&conn)?;

    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'todo',
            priority INTEGER NOT NULL DEFAULT 0,
            project TEXT,
            due_date TEXT,
            estimated_pomos INTEGER DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS pomodoro_sessions (
            id TEXT PRIMARY KEY,
            task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
            session_type TEXT NOT NULL,
            duration_secs INTEGER NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            completed INTEGER NOT NULL DEFAULT 0,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_task ON pomodoro_sessions(task_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_started ON pomodoro_sessions(started_at);
        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

        CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY,
            goal_type TEXT NOT NULL,
            target INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sound_presets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            layers TEXT NOT NULL,
            is_default INTEGER DEFAULT 0,
            created_at TEXT NOT NULL
        );

        INSERT OR IGNORE INTO sound_presets (id, name, layers, is_default, created_at)
        VALUES ('default-deep-focus', 'Deep Focus', '[{\"sound\":\"rain\",\"volume\":0.5},{\"sound\":\"whitenoise\",\"volume\":0.3}]', 1, '2024-01-01T00:00:00Z');

        INSERT OR IGNORE INTO sound_presets (id, name, layers, is_default, created_at)
        VALUES ('default-coffee-shop', 'Coffee Shop', '[{\"sound\":\"cafe\",\"volume\":0.6},{\"sound\":\"lofi\",\"volume\":0.3}]', 1, '2024-01-01T00:00:00Z');

        INSERT OR IGNORE INTO sound_presets (id, name, layers, is_default, created_at)
        VALUES ('default-nature', 'Nature', '[{\"sound\":\"nature\",\"volume\":0.7}]', 1, '2024-01-01T00:00:00Z');

        CREATE TABLE IF NOT EXISTS integrations (
            id TEXT PRIMARY KEY,
            integration_type TEXT NOT NULL,
            name TEXT NOT NULL,
            config TEXT NOT NULL,
            enabled INTEGER DEFAULT 1,
            events TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS event_log (
            id TEXT PRIMARY KEY,
            integration_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            status TEXT NOT NULL,
            error_message TEXT,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_event_log_created ON event_log(created_at);
        CREATE INDEX IF NOT EXISTS idx_event_log_integration ON event_log(integration_id);
        ",
    )?;

    Ok(())
}

pub fn create_task(conn: &Connection, task: &Task) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks (id, title, description, status, priority, project, due_date, estimated_pomos, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            task.id,
            task.title,
            task.description,
            task.status,
            task.priority,
            task.project,
            task.due_date,
            task.estimated_pomos,
            task.sort_order,
            task.created_at,
            task.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update_task(conn: &Connection, task: &Task) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET title = ?1, description = ?2, status = ?3, priority = ?4, project = ?5,
         due_date = ?6, estimated_pomos = ?7, sort_order = ?8, updated_at = ?9
         WHERE id = ?10",
        params![
            task.title,
            task.description,
            task.status,
            task.priority,
            task.project,
            task.due_date,
            task.estimated_pomos,
            task.sort_order,
            task.updated_at,
            task.id,
        ],
    )?;
    Ok(())
}

pub fn delete_task(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn list_tasks(
    conn: &Connection,
    status: Option<&str>,
    project: Option<&str>,
) -> Result<Vec<Task>> {
    let mut sql = String::from(
        "SELECT id, title, description, status, priority, project, due_date, estimated_pomos, sort_order, created_at, updated_at
         FROM tasks WHERE 1=1",
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(s) = status {
        sql.push_str(" AND status = ?");
        param_values.push(Box::new(s.to_string()));
    }
    if let Some(p) = project {
        sql.push_str(" AND project = ?");
        param_values.push(Box::new(p.to_string()));
    }

    sql.push_str(" ORDER BY sort_order ASC, created_at DESC");

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let tasks = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                priority: row.get(4)?,
                project: row.get(5)?,
                due_date: row.get(6)?,
                estimated_pomos: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<Task>>>()?;

    Ok(tasks)
}

pub fn reorder_tasks(conn: &Connection, task_ids: &[String]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    for (i, id) in task_ids.iter().enumerate() {
        tx.execute(
            "UPDATE tasks SET sort_order = ?1 WHERE id = ?2",
            params![i as i32, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn create_session(conn: &Connection, session: &PomodoroSession) -> Result<()> {
    conn.execute(
        "INSERT INTO pomodoro_sessions (id, task_id, session_type, duration_secs, started_at, ended_at, completed, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            session.id,
            session.task_id,
            session.session_type,
            session.duration_secs,
            session.started_at,
            session.ended_at,
            session.completed as i32,
            session.notes,
        ],
    )?;
    Ok(())
}

pub fn complete_session(conn: &Connection, id: &str, ended_at: &str) -> Result<()> {
    conn.execute(
        "UPDATE pomodoro_sessions SET completed = 1, ended_at = ?1 WHERE id = ?2",
        params![ended_at, id],
    )?;
    Ok(())
}

pub fn get_sessions_for_task(conn: &Connection, task_id: &str) -> Result<Vec<PomodoroSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, session_type, duration_secs, started_at, ended_at, completed, notes
         FROM pomodoro_sessions WHERE task_id = ?1 ORDER BY started_at DESC",
    )?;

    let sessions = stmt
        .query_map(params![task_id], |row| {
            let completed_int: i32 = row.get(6)?;
            Ok(PomodoroSession {
                id: row.get(0)?,
                task_id: row.get(1)?,
                session_type: row.get(2)?,
                duration_secs: row.get(3)?,
                started_at: row.get(4)?,
                ended_at: row.get(5)?,
                completed: completed_int != 0,
                notes: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<PomodoroSession>>>()?;

    Ok(sessions)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let settings = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<(String, String)>>>()?;
    Ok(settings)
}

pub fn get_sessions_in_range(
    conn: &Connection,
    start: &str,
    end: &str,
) -> Result<Vec<PomodoroSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, session_type, duration_secs, started_at, ended_at, completed, notes
         FROM pomodoro_sessions
         WHERE started_at >= ?1 AND started_at < ?2 AND completed = 1
         ORDER BY started_at ASC",
    )?;

    let sessions = stmt
        .query_map(params![start, end], |row| {
            let completed_int: i32 = row.get(6)?;
            Ok(PomodoroSession {
                id: row.get(0)?,
                task_id: row.get(1)?,
                session_type: row.get(2)?,
                duration_secs: row.get(3)?,
                started_at: row.get(4)?,
                ended_at: row.get(5)?,
                completed: completed_int != 0,
                notes: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<PomodoroSession>>>()?;

    Ok(sessions)
}

/// Single-query weekly stats: groups by day and session_type in one SQL query
/// instead of issuing 7 separate queries.
pub struct DaySessionStats {
    pub day: String,
    pub session_type: SessionType,
    pub count: usize,
    pub total_secs: i32,
}

pub fn get_weekly_stats_aggregated(
    conn: &Connection,
    start: &str,
    end: &str,
) -> Result<Vec<DaySessionStats>> {
    let mut stmt = conn.prepare(
        "SELECT date(started_at) as day, session_type, COUNT(*) as count, SUM(duration_secs) as total_secs
         FROM pomodoro_sessions
         WHERE started_at >= ?1 AND started_at < ?2 AND completed = 1
         GROUP BY day, session_type
         ORDER BY day ASC",
    )?;

    let stats = stmt
        .query_map(params![start, end], |row| {
            let count: i64 = row.get(2)?;
            Ok(DaySessionStats {
                day: row.get(0)?,
                session_type: row.get(1)?,
                count: count as usize,
                total_secs: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<DaySessionStats>>>()?;

    Ok(stats)
}

// ── Analytics Queries ───────────────────────────────────────────────────────

pub fn get_heatmap_data(conn: &Connection, year: i32) -> Result<Vec<HeatmapEntry>> {
    let mut stmt = conn.prepare(
        "SELECT date(started_at) as day, COUNT(*) as count
         FROM pomodoro_sessions
         WHERE completed = 1 AND session_type = 'work'
           AND strftime('%Y', started_at) = ?1
         GROUP BY day
         ORDER BY day ASC",
    )?;

    let year_str = year.to_string();
    let entries = stmt
        .query_map(params![year_str], |row| {
            Ok(HeatmapEntry {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<HeatmapEntry>>>()?;

    Ok(entries)
}

pub fn get_streak_data(conn: &Connection) -> Result<StreakData> {
    // Get all distinct dates with at least one completed work session, ordered descending
    let mut stmt = conn.prepare(
        "SELECT DISTINCT date(started_at) as day
         FROM pomodoro_sessions
         WHERE completed = 1 AND session_type = 'work'
         ORDER BY day DESC",
    )?;

    let days: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>>>()?;

    if days.is_empty() {
        return Ok(StreakData { current: 0, best: 0 });
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut best = 0i32;
    let mut streak = 0i32;
    let mut prev_date: Option<chrono::NaiveDate> = None;

    for day_str in &days {
        let date = chrono::NaiveDate::parse_from_str(day_str, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

        if let Some(prev) = prev_date {
            let diff = (prev - date).num_days();
            if diff == 1 {
                streak += 1;
            } else {
                if streak > best {
                    best = streak;
                }
                streak = 1;
            }
        } else {
            streak = 1;
        }

        prev_date = Some(date);
    }

    if streak > best {
        best = streak;
    }

    // Current streak: starts from today (or yesterday) and counts consecutive days
    let today_date = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());

    let mut current = 0i32;
    for (i, day_str) in days.iter().enumerate() {
        let date = chrono::NaiveDate::parse_from_str(day_str, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

        if i == 0 {
            let diff = (today_date - date).num_days();
            if diff > 1 {
                break; // No session today or yesterday — current streak is 0
            }
            current = 1;
        } else {
            let prev = chrono::NaiveDate::parse_from_str(&days[i - 1], "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
            if (prev - date).num_days() == 1 {
                current += 1;
            } else {
                break;
            }
        }
    }

    Ok(StreakData { current, best })
}

pub fn get_time_of_day_stats(
    conn: &Connection,
    start: &str,
    end: &str,
) -> Result<Vec<HourlyStats>> {
    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%H', started_at) AS INTEGER) as hour, COUNT(*) as count
         FROM pomodoro_sessions
         WHERE completed = 1 AND session_type = 'work'
           AND started_at >= ?1 AND started_at < ?2
         GROUP BY hour
         ORDER BY hour ASC",
    )?;

    let start_ts = format!("{start}T00:00:00+00:00");
    let end_ts = format!("{end}T23:59:59+00:00");

    let stats = stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok(HourlyStats {
                hour: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<HourlyStats>>>()?;

    Ok(stats)
}

pub fn get_project_stats(
    conn: &Connection,
    start: &str,
    end: &str,
) -> Result<Vec<ProjectStats>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(t.project, 'No Project') as project,
                COUNT(*) as sessions,
                SUM(ps.duration_secs) as total_secs
         FROM pomodoro_sessions ps
         LEFT JOIN tasks t ON ps.task_id = t.id
         WHERE ps.completed = 1 AND ps.session_type = 'work'
           AND ps.started_at >= ?1 AND ps.started_at < ?2
         GROUP BY project
         ORDER BY total_secs DESC",
    )?;

    let start_ts = format!("{start}T00:00:00+00:00");
    let end_ts = format!("{end}T23:59:59+00:00");

    let stats = stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok(ProjectStats {
                project: row.get(0)?,
                sessions: row.get(1)?,
                total_secs: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<ProjectStats>>>()?;

    Ok(stats)
}

pub fn create_goal(conn: &Connection, goal: &Goal) -> Result<()> {
    conn.execute(
        "INSERT INTO goals (id, goal_type, target, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            goal.id,
            goal.goal_type,
            goal.target,
            goal.created_at,
            goal.updated_at,
        ],
    )?;
    Ok(())
}

pub fn get_goals(conn: &Connection) -> Result<Vec<Goal>> {
    let mut stmt = conn.prepare(
        "SELECT id, goal_type, target, created_at, updated_at
         FROM goals ORDER BY created_at DESC",
    )?;

    let goals = stmt
        .query_map([], |row| {
            Ok(Goal {
                id: row.get(0)?,
                goal_type: row.get(1)?,
                target: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<Goal>>>()?;

    Ok(goals)
}

#[allow(dead_code)]
pub fn update_goal(conn: &Connection, id: &str, target: i32, updated_at: &str) -> Result<()> {
    conn.execute(
        "UPDATE goals SET target = ?1, updated_at = ?2 WHERE id = ?3",
        params![target, updated_at, id],
    )?;
    Ok(())
}

// ── Sound Presets ────────────────────────────────────────────────────────────

pub fn get_presets(conn: &Connection) -> Result<Vec<SoundPreset>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, layers, is_default, created_at FROM sound_presets ORDER BY is_default DESC, name ASC",
    )?;

    let presets = stmt
        .query_map([], |row| {
            let is_default_int: i32 = row.get(3)?;
            Ok(SoundPreset {
                id: row.get(0)?,
                name: row.get(1)?,
                layers: row.get(2)?,
                is_default: is_default_int != 0,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<SoundPreset>>>()?;

    Ok(presets)
}

pub fn save_preset(conn: &Connection, preset: &SoundPreset) -> Result<()> {
    conn.execute(
        "INSERT INTO sound_presets (id, name, layers, is_default, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, layers = excluded.layers",
        params![
            preset.id,
            preset.name,
            preset.layers,
            preset.is_default as i32,
            preset.created_at,
        ],
    )?;
    Ok(())
}

pub fn delete_preset(conn: &Connection, id: &str) -> Result<usize> {
    let rows = conn.execute(
        "DELETE FROM sound_presets WHERE id = ?1 AND is_default = 0",
        params![id],
    )?;
    Ok(rows)
}

// ── Integrations ─────────────────────────────────────────────────────────────

pub fn get_integrations(conn: &Connection) -> Result<Vec<IntegrationConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, integration_type, name, config, enabled, events, created_at, updated_at
         FROM integrations ORDER BY created_at DESC",
    )?;

    let rows = stmt
        .query_map([], |row| {
            let enabled_int: i32 = row.get(4)?;
            Ok(IntegrationConfig {
                id: row.get(0)?,
                integration_type: row.get(1)?,
                name: row.get(2)?,
                config: row.get(3)?,
                enabled: enabled_int != 0,
                events: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<IntegrationConfig>>>()?;

    Ok(rows)
}

pub fn get_enabled_integrations_for_event(
    conn: &Connection,
    event_type: &str,
) -> Result<Vec<IntegrationConfig>> {
    let pattern = format!("%\"{event_type}\"%");
    let mut stmt = conn.prepare(
        "SELECT id, integration_type, name, config, enabled, events, created_at, updated_at
         FROM integrations WHERE enabled = 1 AND events LIKE ?1",
    )?;

    let rows = stmt
        .query_map(params![pattern], |row| {
            let enabled_int: i32 = row.get(4)?;
            Ok(IntegrationConfig {
                id: row.get(0)?,
                integration_type: row.get(1)?,
                name: row.get(2)?,
                config: row.get(3)?,
                enabled: enabled_int != 0,
                events: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<IntegrationConfig>>>()?;

    Ok(rows)
}

pub fn create_integration(conn: &Connection, config: &IntegrationConfig) -> Result<()> {
    conn.execute(
        "INSERT INTO integrations (id, integration_type, name, config, enabled, events, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            config.id,
            config.integration_type,
            config.name,
            config.config,
            config.enabled as i32,
            config.events,
            config.created_at,
            config.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update_integration(
    conn: &Connection,
    id: &str,
    name: &str,
    config: &str,
    events: &str,
    enabled: bool,
    updated_at: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE integrations SET name = ?1, config = ?2, events = ?3, enabled = ?4, updated_at = ?5
         WHERE id = ?6",
        params![name, config, events, enabled as i32, updated_at, id],
    )?;
    Ok(())
}

pub fn delete_integration(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM integrations WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn log_event(conn: &Connection, entry: &EventLogEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO event_log (id, integration_id, event_type, payload, status, error_message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.id,
            entry.integration_id,
            entry.event_type,
            entry.payload,
            entry.status,
            entry.error_message,
            entry.created_at,
        ],
    )?;
    Ok(())
}

pub fn get_event_log(
    conn: &Connection,
    integration_id: Option<&str>,
    limit: i32,
) -> Result<Vec<EventLogEntry>> {
    let (sql, param_values): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
        if let Some(iid) = integration_id {
            (
                "SELECT id, integration_id, event_type, payload, status, error_message, created_at
                 FROM event_log WHERE integration_id = ?1
                 ORDER BY created_at DESC LIMIT ?2"
                    .to_string(),
                vec![
                    Box::new(iid.to_string()) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(limit),
                ],
            )
        } else {
            (
                "SELECT id, integration_id, event_type, payload, status, error_message, created_at
                 FROM event_log ORDER BY created_at DESC LIMIT ?1"
                    .to_string(),
                vec![Box::new(limit) as Box<dyn rusqlite::types::ToSql>],
            )
        };

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(EventLogEntry {
                id: row.get(0)?,
                integration_id: row.get(1)?,
                event_type: row.get(2)?,
                payload: row.get(3)?,
                status: row.get(4)?,
                error_message: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<EventLogEntry>>>()?;

    Ok(rows)
}

#[allow(dead_code)]
pub fn cleanup_old_logs(conn: &Connection, days: i32) -> Result<usize> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff.to_rfc3339();
    let rows = conn.execute(
        "DELETE FROM event_log WHERE created_at < ?1",
        params![cutoff_str],
    )?;
    Ok(rows)
}

pub fn get_task_status(conn: &Connection, id: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT status FROM tasks WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

// ── In-memory DB for tests ──────────────────────────────────────────────────

/// Create an in-memory SQLite database with all tables for testing.
#[cfg(test)]
pub fn init_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_task(id: &str) -> Task {
        let now = chrono::Utc::now().to_rfc3339();
        Task {
            id: id.to_string(),
            title: format!("Task {id}"),
            description: Some("A test task".to_string()),
            status: crate::models::TaskStatus::Todo,
            priority: 1,
            project: Some("TestProject".to_string()),
            due_date: None,
            estimated_pomos: 4,
            sort_order: 0,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    fn sample_session(id: &str, task_id: Option<&str>) -> PomodoroSession {
        let now = chrono::Utc::now().to_rfc3339();
        PomodoroSession {
            id: id.to_string(),
            task_id: task_id.map(|s| s.to_string()),
            session_type: SessionType::Work,
            duration_secs: 1500,
            started_at: now.clone(),
            ended_at: None,
            completed: false,
            notes: None,
        }
    }

    fn sample_goal(id: &str) -> Goal {
        let now = chrono::Utc::now().to_rfc3339();
        Goal {
            id: id.to_string(),
            goal_type: "daily_sessions".to_string(),
            target: 8,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    // ── Migration / Init ────────────────────────────────────────────────

    #[test]
    fn in_memory_db_creates_tables() {
        let conn = init_in_memory().expect("Failed to create in-memory DB");
        // Verify tables exist by querying them
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM pomodoro_sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migrations_seed_default_presets() {
        let conn = init_in_memory().unwrap();
        let presets = get_presets(&conn).unwrap();
        assert!(presets.len() >= 3, "Should have at least 3 default presets");
        assert!(presets.iter().all(|p| p.is_default));
    }

    // ── Task CRUD ───────────────────────────────────────────────────────

    #[test]
    fn create_and_list_task() {
        let conn = init_in_memory().unwrap();
        let task = sample_task("t1");
        create_task(&conn, &task).unwrap();

        let tasks = list_tasks(&conn, None, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Task t1");
    }

    #[test]
    fn update_task_changes_fields() {
        let conn = init_in_memory().unwrap();
        let mut task = sample_task("t1");
        create_task(&conn, &task).unwrap();

        task.title = "Updated Title".to_string();
        task.priority = 5;
        update_task(&conn, &task).unwrap();

        let tasks = list_tasks(&conn, None, None).unwrap();
        assert_eq!(tasks[0].title, "Updated Title");
        assert_eq!(tasks[0].priority, 5);
    }

    #[test]
    fn delete_task_removes_it() {
        let conn = init_in_memory().unwrap();
        create_task(&conn, &sample_task("t1")).unwrap();
        create_task(&conn, &sample_task("t2")).unwrap();

        delete_task(&conn, "t1").unwrap();
        let tasks = list_tasks(&conn, None, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "t2");
    }

    #[test]
    fn list_tasks_filters_by_status() {
        let conn = init_in_memory().unwrap();
        let mut task1 = sample_task("t1");
        task1.status = crate::models::TaskStatus::Todo;
        let mut task2 = sample_task("t2");
        task2.status = crate::models::TaskStatus::Done;
        create_task(&conn, &task1).unwrap();
        create_task(&conn, &task2).unwrap();

        let todos = list_tasks(&conn, Some("todo"), None).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, "t1");
    }

    #[test]
    fn list_tasks_filters_by_project() {
        let conn = init_in_memory().unwrap();
        let mut task1 = sample_task("t1");
        task1.project = Some("Alpha".to_string());
        let mut task2 = sample_task("t2");
        task2.project = Some("Beta".to_string());
        create_task(&conn, &task1).unwrap();
        create_task(&conn, &task2).unwrap();

        let alpha = list_tasks(&conn, None, Some("Alpha")).unwrap();
        assert_eq!(alpha.len(), 1);
        assert_eq!(alpha[0].id, "t1");
    }

    #[test]
    fn reorder_tasks_updates_sort_order() {
        let conn = init_in_memory().unwrap();
        create_task(&conn, &sample_task("a")).unwrap();
        create_task(&conn, &sample_task("b")).unwrap();
        create_task(&conn, &sample_task("c")).unwrap();

        reorder_tasks(&conn, &["c".into(), "a".into(), "b".into()]).unwrap();
        let tasks = list_tasks(&conn, None, None).unwrap();
        // Ordered by sort_order ASC
        assert_eq!(tasks[0].id, "c");
        assert_eq!(tasks[1].id, "a");
        assert_eq!(tasks[2].id, "b");
    }

    #[test]
    fn get_task_status_returns_none_for_missing() {
        let conn = init_in_memory().unwrap();
        assert_eq!(get_task_status(&conn, "nonexistent").unwrap(), None);
    }

    #[test]
    fn get_task_status_returns_correct_value() {
        let conn = init_in_memory().unwrap();
        create_task(&conn, &sample_task("t1")).unwrap();
        let status = get_task_status(&conn, "t1").unwrap();
        assert_eq!(status, Some("todo".to_string()));
    }

    // ── Session CRUD ────────────────────────────────────────────────────

    #[test]
    fn create_and_get_sessions_for_task() {
        let conn = init_in_memory().unwrap();
        create_task(&conn, &sample_task("t1")).unwrap();
        create_session(&conn, &sample_session("s1", Some("t1"))).unwrap();
        create_session(&conn, &sample_session("s2", Some("t1"))).unwrap();

        let sessions = get_sessions_for_task(&conn, "t1").unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn complete_session_marks_completed() {
        let conn = init_in_memory().unwrap();
        create_session(&conn, &sample_session("s1", None)).unwrap();
        let ended = chrono::Utc::now().to_rfc3339();
        complete_session(&conn, "s1", &ended).unwrap();

        // Verify via raw query
        let completed: i32 = conn
            .query_row(
                "SELECT completed FROM pomodoro_sessions WHERE id = ?1",
                params!["s1"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(completed, 1);
    }

    #[test]
    fn get_sessions_in_range_filters_completed() {
        let conn = init_in_memory().unwrap();

        let mut s1 = sample_session("s1", None);
        s1.started_at = "2024-06-15T10:00:00+00:00".to_string();
        s1.completed = true;
        // Store completed as 1 directly
        conn.execute(
            "INSERT INTO pomodoro_sessions (id, task_id, session_type, duration_secs, started_at, ended_at, completed, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![s1.id, s1.task_id, s1.session_type, s1.duration_secs, s1.started_at, s1.ended_at, 1, s1.notes],
        ).unwrap();

        let mut s2 = sample_session("s2", None);
        s2.started_at = "2024-06-15T12:00:00+00:00".to_string();
        create_session(&conn, &s2).unwrap(); // completed = 0

        let range = get_sessions_in_range(
            &conn,
            "2024-06-15T00:00:00+00:00",
            "2024-06-16T00:00:00+00:00",
        )
        .unwrap();
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].id, "s1");
    }

    // ── Settings ────────────────────────────────────────────────────────

    #[test]
    fn set_and_get_setting() {
        let conn = init_in_memory().unwrap();
        set_setting(&conn, "work_duration", "1500").unwrap();
        let val = get_setting(&conn, "work_duration").unwrap();
        assert_eq!(val, Some("1500".to_string()));
    }

    #[test]
    fn get_setting_returns_none_for_missing() {
        let conn = init_in_memory().unwrap();
        assert_eq!(get_setting(&conn, "nonexistent").unwrap(), None);
    }

    #[test]
    fn set_setting_upserts() {
        let conn = init_in_memory().unwrap();
        set_setting(&conn, "key1", "value1").unwrap();
        set_setting(&conn, "key1", "value2").unwrap();
        let val = get_setting(&conn, "key1").unwrap();
        assert_eq!(val, Some("value2".to_string()));
    }

    #[test]
    fn get_all_settings_returns_all() {
        let conn = init_in_memory().unwrap();
        set_setting(&conn, "a", "1").unwrap();
        set_setting(&conn, "b", "2").unwrap();
        let all = get_all_settings(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    // ── Goals ───────────────────────────────────────────────────────────

    #[test]
    fn create_and_get_goals() {
        let conn = init_in_memory().unwrap();
        create_goal(&conn, &sample_goal("g1")).unwrap();
        create_goal(&conn, &sample_goal("g2")).unwrap();

        let goals = get_goals(&conn).unwrap();
        assert_eq!(goals.len(), 2);
    }

    #[test]
    fn update_goal_changes_target() {
        let conn = init_in_memory().unwrap();
        create_goal(&conn, &sample_goal("g1")).unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        update_goal(&conn, "g1", 12, &now).unwrap();

        let goals = get_goals(&conn).unwrap();
        assert_eq!(goals[0].target, 12);
    }

    // ── Sound Presets ───────────────────────────────────────────────────

    #[test]
    fn save_and_get_preset() {
        let conn = init_in_memory().unwrap();
        let preset = SoundPreset {
            id: "custom-1".to_string(),
            name: "My Preset".to_string(),
            layers: "[]".to_string(),
            is_default: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        save_preset(&conn, &preset).unwrap();

        let presets = get_presets(&conn).unwrap();
        assert!(presets.iter().any(|p| p.id == "custom-1"));
    }

    #[test]
    fn delete_preset_only_deletes_non_default() {
        let conn = init_in_memory().unwrap();
        // Trying to delete a default preset returns 0 rows
        let rows = delete_preset(&conn, "default-deep-focus").unwrap();
        assert_eq!(rows, 0);

        // Non-default preset can be deleted
        let preset = SoundPreset {
            id: "custom-1".to_string(),
            name: "Custom".to_string(),
            layers: "[]".to_string(),
            is_default: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        save_preset(&conn, &preset).unwrap();
        let rows = delete_preset(&conn, "custom-1").unwrap();
        assert_eq!(rows, 1);
    }

    // ── Integrations ────────────────────────────────────────────────────

    #[test]
    fn create_and_list_integrations() {
        let conn = init_in_memory().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let config = IntegrationConfig {
            id: "int-1".to_string(),
            integration_type: "webhook".to_string(),
            name: "My Webhook".to_string(),
            config: r#"{"url":"https://example.com"}"#.to_string(),
            enabled: true,
            events: r#"["session.completed"]"#.to_string(),
            created_at: now.clone(),
            updated_at: now,
        };
        create_integration(&conn, &config).unwrap();

        let integrations = get_integrations(&conn).unwrap();
        assert_eq!(integrations.len(), 1);
        assert_eq!(integrations[0].name, "My Webhook");
    }

    #[test]
    fn update_integration_changes_fields() {
        let conn = init_in_memory().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let config = IntegrationConfig {
            id: "int-1".to_string(),
            integration_type: "webhook".to_string(),
            name: "Old Name".to_string(),
            config: "{}".to_string(),
            enabled: true,
            events: "[]".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        create_integration(&conn, &config).unwrap();

        update_integration(&conn, "int-1", "New Name", "{}", "[]", false, &now).unwrap();
        let integrations = get_integrations(&conn).unwrap();
        assert_eq!(integrations[0].name, "New Name");
        assert!(!integrations[0].enabled);
    }

    #[test]
    fn delete_integration_removes_it() {
        let conn = init_in_memory().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let config = IntegrationConfig {
            id: "int-1".to_string(),
            integration_type: "webhook".to_string(),
            name: "Test".to_string(),
            config: "{}".to_string(),
            enabled: true,
            events: "[]".to_string(),
            created_at: now.clone(),
            updated_at: now,
        };
        create_integration(&conn, &config).unwrap();
        delete_integration(&conn, "int-1").unwrap();

        let integrations = get_integrations(&conn).unwrap();
        assert!(integrations.is_empty());
    }

    #[test]
    fn get_enabled_integrations_for_event_filters() {
        let conn = init_in_memory().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        // Enabled, subscribes to session.completed
        let c1 = IntegrationConfig {
            id: "int-1".to_string(),
            integration_type: "webhook".to_string(),
            name: "Webhook1".to_string(),
            config: "{}".to_string(),
            enabled: true,
            events: r#"["session.completed","task.created"]"#.to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        // Disabled
        let c2 = IntegrationConfig {
            id: "int-2".to_string(),
            integration_type: "webhook".to_string(),
            name: "Webhook2".to_string(),
            config: "{}".to_string(),
            enabled: false,
            events: r#"["session.completed"]"#.to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        // Enabled but different event
        let c3 = IntegrationConfig {
            id: "int-3".to_string(),
            integration_type: "webhook".to_string(),
            name: "Webhook3".to_string(),
            config: "{}".to_string(),
            enabled: true,
            events: r#"["task.created"]"#.to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        create_integration(&conn, &c1).unwrap();
        create_integration(&conn, &c2).unwrap();
        create_integration(&conn, &c3).unwrap();

        let result = get_enabled_integrations_for_event(&conn, "session.completed").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "int-1");
    }

    // ── Event Log ───────────────────────────────────────────────────────

    #[test]
    fn log_and_get_events() {
        let conn = init_in_memory().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let entry = EventLogEntry {
            id: "ev-1".to_string(),
            integration_id: "int-1".to_string(),
            event_type: "session.completed".to_string(),
            payload: "{}".to_string(),
            status: "success".to_string(),
            error_message: None,
            created_at: now,
        };
        log_event(&conn, &entry).unwrap();

        let log = get_event_log(&conn, None, 10).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].id, "ev-1");

        // Filter by integration_id
        let log = get_event_log(&conn, Some("int-1"), 10).unwrap();
        assert_eq!(log.len(), 1);

        let log = get_event_log(&conn, Some("other"), 10).unwrap();
        assert!(log.is_empty());
    }

    // ── Heatmap ─────────────────────────────────────────────────────────

    #[test]
    fn heatmap_data_counts_work_sessions() {
        let conn = init_in_memory().unwrap();
        // Insert a completed work session on 2024-03-15
        conn.execute(
            "INSERT INTO pomodoro_sessions (id, session_type, duration_secs, started_at, completed)
             VALUES ('s1', 'work', 1500, '2024-03-15T10:00:00+00:00', 1)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO pomodoro_sessions (id, session_type, duration_secs, started_at, completed)
             VALUES ('s2', 'short_break', 300, '2024-03-15T10:30:00+00:00', 1)",
            [],
        ).unwrap();

        let data = get_heatmap_data(&conn, 2024).unwrap();
        assert_eq!(data.len(), 1); // only work sessions
        assert_eq!(data[0].count, 1);
    }

    // ── Streak ──────────────────────────────────────────────────────────

    #[test]
    fn streak_data_empty_db() {
        let conn = init_in_memory().unwrap();
        let streak = get_streak_data(&conn).unwrap();
        assert_eq!(streak.current, 0);
        assert_eq!(streak.best, 0);
    }

    // ── Weekly Aggregated Stats ──────────────────────────────────────────

    #[test]
    fn weekly_stats_aggregated() {
        let conn = init_in_memory().unwrap();
        conn.execute(
            "INSERT INTO pomodoro_sessions (id, session_type, duration_secs, started_at, completed)
             VALUES ('s1', 'work', 1500, '2024-03-11T10:00:00+00:00', 1)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO pomodoro_sessions (id, session_type, duration_secs, started_at, completed)
             VALUES ('s2', 'work', 1500, '2024-03-11T14:00:00+00:00', 1)",
            [],
        ).unwrap();

        let stats = get_weekly_stats_aggregated(
            &conn,
            "2024-03-11T00:00:00+00:00",
            "2024-03-18T00:00:00+00:00",
        )
        .unwrap();
        assert_eq!(stats.len(), 1); // one day
        assert_eq!(stats[0].count, 2);
        assert_eq!(stats[0].total_secs, 3000);
    }

    // ── Cleanup Old Logs ────────────────────────────────────────────────

    #[test]
    fn cleanup_old_logs_removes_old_entries() {
        let conn = init_in_memory().unwrap();
        let old_date = "2020-01-01T00:00:00+00:00";
        let entry = EventLogEntry {
            id: "ev-old".to_string(),
            integration_id: "int-1".to_string(),
            event_type: "test".to_string(),
            payload: "{}".to_string(),
            status: "success".to_string(),
            error_message: None,
            created_at: old_date.to_string(),
        };
        log_event(&conn, &entry).unwrap();

        let removed = cleanup_old_logs(&conn, 30).unwrap();
        assert_eq!(removed, 1);
    }
}
