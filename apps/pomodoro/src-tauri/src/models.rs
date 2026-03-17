use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Archived,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Archived => "archived",
        }
    }
}

impl ToSql for TaskStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for TaskStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        match s {
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            "archived" => Ok(TaskStatus::Archived),
            _ => Err(FromSqlError::Other(
                format!("Invalid task status: {s}").into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

impl SessionType {
    pub fn as_str(&self) -> &str {
        match self {
            SessionType::Work => "work",
            SessionType::ShortBreak => "short_break",
            SessionType::LongBreak => "long_break",
        }
    }
}

impl ToSql for SessionType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for SessionType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        match s {
            "work" => Ok(SessionType::Work),
            "short_break" => Ok(SessionType::ShortBreak),
            "long_break" => Ok(SessionType::LongBreak),
            _ => Err(FromSqlError::Other(
                format!("Invalid session type: {s}").into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: i32,
    pub project: Option<String>,
    pub due_date: Option<String>,
    pub estimated_pomos: i32,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSession {
    pub id: String,
    pub task_id: Option<String>,
    pub session_type: SessionType,
    pub duration_secs: i32,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub completed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub status: TimerStatus,
    pub session_type: String,
    pub remaining_secs: i32,
    pub current_cycle: i32,
    pub total_cycles: i32,
}

// ── Analytics Models ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapEntry {
    pub date: String,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakData {
    pub current: i32,
    pub best: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: i32,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub project: String,
    pub sessions: i32,
    pub total_secs: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub goal_type: String,
    pub target: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::types::{ToSqlOutput, Value, ValueRef};

    // ── TaskStatus ──────────────────────────────────────────────────────

    #[test]
    fn task_status_as_str() {
        assert_eq!(TaskStatus::Todo.as_str(), "todo");
        assert_eq!(TaskStatus::InProgress.as_str(), "in_progress");
        assert_eq!(TaskStatus::Done.as_str(), "done");
        assert_eq!(TaskStatus::Archived.as_str(), "archived");
    }

    #[test]
    fn task_status_to_sql() {
        let output = TaskStatus::Todo.to_sql().unwrap();
        match output {
            ToSqlOutput::Borrowed(ValueRef::Text(s)) => {
                assert_eq!(std::str::from_utf8(s).unwrap(), "todo");
            }
            _ => panic!("Expected borrowed text"),
        }
    }

    #[test]
    fn task_status_from_sql_valid() {
        let val = Value::Text("in_progress".to_string());
        let result = TaskStatus::column_result(ValueRef::from(&val));
        assert_eq!(result.unwrap(), TaskStatus::InProgress);
    }

    #[test]
    fn task_status_from_sql_invalid() {
        let val = Value::Text("invalid".to_string());
        let result = TaskStatus::column_result(ValueRef::from(&val));
        assert!(result.is_err());
    }

    // ── SessionType ─────────────────────────────────────────────────────

    #[test]
    fn session_type_as_str() {
        assert_eq!(SessionType::Work.as_str(), "work");
        assert_eq!(SessionType::ShortBreak.as_str(), "short_break");
        assert_eq!(SessionType::LongBreak.as_str(), "long_break");
    }

    #[test]
    fn session_type_from_sql_valid() {
        let val = Value::Text("long_break".to_string());
        let result = SessionType::column_result(ValueRef::from(&val));
        assert_eq!(result.unwrap(), SessionType::LongBreak);
    }

    #[test]
    fn session_type_from_sql_invalid() {
        let val = Value::Text("invalid".to_string());
        let result = SessionType::column_result(ValueRef::from(&val));
        assert!(result.is_err());
    }

    // ── Serde round-trips ───────────────────────────────────────────────

    #[test]
    fn task_serde_round_trip() {
        let task = Task {
            id: "t1".to_string(),
            title: "Test".to_string(),
            description: None,
            status: TaskStatus::Todo,
            priority: 1,
            project: Some("proj".to_string()),
            due_date: None,
            estimated_pomos: 4,
            sort_order: 0,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&task).unwrap();
        let parsed: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "t1");
        assert_eq!(parsed.status, TaskStatus::Todo);
    }

    #[test]
    fn timer_state_serde() {
        let state = TimerState {
            status: TimerStatus::Running,
            session_type: "work".to_string(),
            remaining_secs: 1500,
            current_cycle: 1,
            total_cycles: 4,
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: TimerState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, TimerStatus::Running);
        assert_eq!(parsed.remaining_secs, 1500);
    }

    #[test]
    fn session_type_serde_snake_case() {
        let json = r#""short_break""#;
        let st: SessionType = serde_json::from_str(json).unwrap();
        assert_eq!(st, SessionType::ShortBreak);
        let back = serde_json::to_string(&st).unwrap();
        assert_eq!(back, r#""short_break""#);
    }

    #[test]
    fn timer_status_serde_lowercase() {
        let json = r#""idle""#;
        let ts: TimerStatus = serde_json::from_str(json).unwrap();
        assert_eq!(ts, TimerStatus::Idle);
    }
}
