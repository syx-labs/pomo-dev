use std::sync::{Mutex, MutexGuard};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Lock poisoned: {context}")]
    LockPoisoned { context: &'static str },

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Date parse error: {0}")]
    DateParse(#[from] chrono::ParseError),
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub trait MutexExt<T> {
    fn lock_or_err(&self, context: &'static str) -> AppResult<MutexGuard<'_, T>>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_or_err(&self, context: &'static str) -> AppResult<MutexGuard<'_, T>> {
        self.lock().map_err(|_| AppError::LockPoisoned { context })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_display_database() {
        let err = AppError::Database(rusqlite::Error::QueryReturnedNoRows);
        let msg = err.to_string();
        assert!(msg.contains("Database error"));
    }

    #[test]
    fn app_error_display_lock_poisoned() {
        let err = AppError::LockPoisoned { context: "test_lock" };
        assert_eq!(err.to_string(), "Lock poisoned: test_lock");
    }

    #[test]
    fn app_error_display_invalid_state() {
        let err = AppError::InvalidState("bad state".to_string());
        assert_eq!(err.to_string(), "Invalid state: bad state");
    }

    #[test]
    fn app_error_display_invalid_input() {
        let err = AppError::InvalidInput("missing field".to_string());
        assert_eq!(err.to_string(), "Invalid input: missing field");
    }

    #[test]
    fn app_error_to_string_conversion() {
        let err = AppError::InvalidInput("test".to_string());
        let s: String = err.into();
        assert_eq!(s, "Invalid input: test");
    }

    #[test]
    fn mutex_ext_lock_succeeds() {
        let m = Mutex::new(42);
        let guard = m.lock_or_err("test_context");
        assert!(guard.is_ok());
        assert_eq!(*guard.unwrap(), 42);
    }
}
