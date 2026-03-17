use crate::models::{SessionType, TimerState, TimerStatus};

#[derive(Debug, Clone)]
pub struct TimerSettings {
    pub work_duration: i32,
    pub short_break: i32,
    pub long_break: i32,
    pub cycles_before_long_break: i32,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            work_duration: 25 * 60,
            short_break: 5 * 60,
            long_break: 15 * 60,
            cycles_before_long_break: 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PomodoroTimer {
    pub status: TimerStatus,
    pub session_type: SessionType,
    pub remaining_secs: i32,
    pub current_cycle: i32,
    pub settings: TimerSettings,
    pub linked_task_id: Option<String>,
    pub current_session_id: Option<String>,
}

impl PomodoroTimer {
    pub fn new() -> Self {
        let settings = TimerSettings::default();
        Self {
            status: TimerStatus::Idle,
            session_type: SessionType::Work,
            remaining_secs: settings.work_duration,
            current_cycle: 1,
            settings,
            linked_task_id: None,
            current_session_id: None,
        }
    }

    /// Apply settings loaded from DB. Only updates durations and cycles;
    /// also adjusts remaining_secs if the timer is idle (not mid-session).
    pub fn apply_settings(&mut self, settings: TimerSettings) {
        self.settings = settings;
        if self.status == TimerStatus::Idle {
            self.remaining_secs = self.duration_for_session_type();
        }
    }

    pub fn start(&mut self) {
        if self.status == TimerStatus::Idle {
            self.remaining_secs = self.duration_for_session_type();
        }
        self.status = TimerStatus::Running;
    }

    pub fn pause(&mut self) {
        if self.status == TimerStatus::Running {
            self.status = TimerStatus::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.status == TimerStatus::Paused {
            self.status = TimerStatus::Running;
        }
    }

    pub fn skip(&mut self) {
        self.advance_session();
    }

    pub fn reset(&mut self) {
        self.status = TimerStatus::Idle;
        self.session_type = SessionType::Work;
        self.remaining_secs = self.settings.work_duration;
        self.current_cycle = 1;
        self.linked_task_id = None;
        self.current_session_id = None;
    }

    /// Decrements remaining time by one second.
    /// Returns `true` if the session just completed (remaining hit 0).
    pub fn tick(&mut self) -> bool {
        if self.status != TimerStatus::Running {
            return false;
        }

        if self.remaining_secs <= 0 {
            self.remaining_secs = 0;
            self.advance_session();
            return true;
        }

        self.remaining_secs -= 1;

        if self.remaining_secs == 0 {
            self.advance_session();
            return true;
        }

        false
    }

    fn advance_session(&mut self) {
        match self.session_type {
            SessionType::Work => {
                if self.current_cycle >= self.settings.cycles_before_long_break {
                    self.session_type = SessionType::LongBreak;
                    self.current_cycle = 1;
                } else {
                    self.session_type = SessionType::ShortBreak;
                }
            }
            SessionType::ShortBreak => {
                self.current_cycle += 1;
                self.session_type = SessionType::Work;
            }
            SessionType::LongBreak => {
                self.session_type = SessionType::Work;
            }
        }

        self.remaining_secs = self.duration_for_session_type();
        self.status = TimerStatus::Idle;
        self.linked_task_id = None;
        self.current_session_id = None;
    }

    fn duration_for_session_type(&self) -> i32 {
        match self.session_type {
            SessionType::Work => self.settings.work_duration,
            SessionType::ShortBreak => self.settings.short_break,
            SessionType::LongBreak => self.settings.long_break,
        }
    }

    pub fn get_state(&self) -> TimerState {
        TimerState {
            status: self.status.clone(),
            session_type: self.session_type.as_str().to_string(),
            remaining_secs: self.remaining_secs,
            current_cycle: self.current_cycle,
            total_cycles: self.settings.cycles_before_long_break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_timer() -> PomodoroTimer {
        PomodoroTimer::new()
    }

    fn make_short_timer() -> PomodoroTimer {
        let mut t = PomodoroTimer::new();
        t.settings = TimerSettings {
            work_duration: 3,
            short_break: 2,
            long_break: 5,
            cycles_before_long_break: 2,
        };
        t.remaining_secs = t.settings.work_duration;
        t
    }

    // ── Construction ────────────────────────────────────────────────────

    #[test]
    fn new_timer_starts_idle_with_work_session() {
        let t = make_timer();
        assert_eq!(t.status, TimerStatus::Idle);
        assert_eq!(t.session_type, SessionType::Work);
        assert_eq!(t.remaining_secs, 25 * 60);
        assert_eq!(t.current_cycle, 1);
        assert!(t.linked_task_id.is_none());
        assert!(t.current_session_id.is_none());
    }

    #[test]
    fn default_settings() {
        let s = TimerSettings::default();
        assert_eq!(s.work_duration, 25 * 60);
        assert_eq!(s.short_break, 5 * 60);
        assert_eq!(s.long_break, 15 * 60);
        assert_eq!(s.cycles_before_long_break, 4);
    }

    // ── Start / Pause / Resume ──────────────────────────────────────────

    #[test]
    fn start_from_idle() {
        let mut t = make_timer();
        t.start();
        assert_eq!(t.status, TimerStatus::Running);
        assert_eq!(t.remaining_secs, 25 * 60);
    }

    #[test]
    fn pause_from_running() {
        let mut t = make_timer();
        t.start();
        t.pause();
        assert_eq!(t.status, TimerStatus::Paused);
    }

    #[test]
    fn pause_noop_when_idle() {
        let mut t = make_timer();
        t.pause();
        assert_eq!(t.status, TimerStatus::Idle);
    }

    #[test]
    fn resume_from_paused() {
        let mut t = make_timer();
        t.start();
        t.pause();
        t.resume();
        assert_eq!(t.status, TimerStatus::Running);
    }

    #[test]
    fn resume_noop_when_idle() {
        let mut t = make_timer();
        t.resume();
        assert_eq!(t.status, TimerStatus::Idle);
    }

    // ── Tick ────────────────────────────────────────────────────────────

    #[test]
    fn tick_decrements_remaining() {
        let mut t = make_short_timer();
        t.start();
        let completed = t.tick();
        assert!(!completed);
        assert_eq!(t.remaining_secs, 2);
    }

    #[test]
    fn tick_noop_when_idle() {
        let mut t = make_timer();
        assert!(!t.tick());
    }

    #[test]
    fn tick_completes_session_when_reaching_zero() {
        let mut t = make_short_timer();
        t.start();
        // remaining = 3, tick 3 times: 3->2 (false), 2->1 (false), 1->0 (true, advance)
        assert!(!t.tick()); // 2
        assert!(!t.tick()); // 1
        assert!(t.tick());  // 0 -> advance
        // After a work session with cycles_before_long_break=2, cycle 1 -> short break
        assert_eq!(t.session_type, SessionType::ShortBreak);
        assert_eq!(t.status, TimerStatus::Idle);
        assert_eq!(t.remaining_secs, 2);
    }

    // ── Skip ────────────────────────────────────────────────────────────

    #[test]
    fn skip_advances_session() {
        let mut t = make_short_timer();
        t.start();
        t.skip();
        assert_eq!(t.session_type, SessionType::ShortBreak);
        assert_eq!(t.status, TimerStatus::Idle);
    }

    // ── Reset ───────────────────────────────────────────────────────────

    #[test]
    fn reset_returns_to_initial_state() {
        let mut t = make_short_timer();
        t.start();
        t.tick();
        t.linked_task_id = Some("task-1".into());
        t.current_session_id = Some("session-1".into());
        t.reset();
        assert_eq!(t.status, TimerStatus::Idle);
        assert_eq!(t.session_type, SessionType::Work);
        assert_eq!(t.remaining_secs, 3); // work_duration
        assert_eq!(t.current_cycle, 1);
        assert!(t.linked_task_id.is_none());
        assert!(t.current_session_id.is_none());
    }

    // ── Full cycle transitions ──────────────────────────────────────────

    #[test]
    fn full_cycle_work_short_work_long() {
        let mut t = make_short_timer(); // cycles_before_long_break = 2

        // Cycle 1: Work -> Short Break
        t.skip();
        assert_eq!(t.session_type, SessionType::ShortBreak);
        assert_eq!(t.current_cycle, 1);

        // Short Break -> Work (cycle increments to 2)
        t.skip();
        assert_eq!(t.session_type, SessionType::Work);
        assert_eq!(t.current_cycle, 2);

        // Cycle 2 (== cycles_before_long_break): Work -> Long Break
        t.skip();
        assert_eq!(t.session_type, SessionType::LongBreak);
        assert_eq!(t.current_cycle, 1); // reset after long break trigger

        // Long Break -> Work
        t.skip();
        assert_eq!(t.session_type, SessionType::Work);
        assert_eq!(t.current_cycle, 1);
    }

    // ── Apply Settings ──────────────────────────────────────────────────

    #[test]
    fn apply_settings_while_idle_updates_remaining() {
        let mut t = make_timer();
        assert_eq!(t.status, TimerStatus::Idle);
        let settings = TimerSettings {
            work_duration: 10,
            short_break: 3,
            long_break: 7,
            cycles_before_long_break: 2,
        };
        t.apply_settings(settings);
        assert_eq!(t.remaining_secs, 10);
    }

    #[test]
    fn apply_settings_while_running_preserves_remaining() {
        let mut t = make_timer();
        t.start();
        t.tick(); // remaining = 25*60 - 1
        let remaining_before = t.remaining_secs;
        let settings = TimerSettings {
            work_duration: 10,
            short_break: 3,
            long_break: 7,
            cycles_before_long_break: 2,
        };
        t.apply_settings(settings);
        assert_eq!(t.remaining_secs, remaining_before); // not changed
    }

    // ── get_state ───────────────────────────────────────────────────────

    #[test]
    fn get_state_reflects_timer() {
        let mut t = make_short_timer();
        t.start();
        let s = t.get_state();
        assert_eq!(s.status, TimerStatus::Running);
        assert_eq!(s.session_type, "work");
        assert_eq!(s.remaining_secs, 3);
        assert_eq!(s.current_cycle, 1);
        assert_eq!(s.total_cycles, 2);
    }

    // ── Edge: tick when remaining is already 0 ──────────────────────────

    #[test]
    fn tick_when_remaining_already_zero() {
        let mut t = make_short_timer();
        t.start();
        t.remaining_secs = 0;
        // Should advance immediately
        let completed = t.tick();
        assert!(completed);
        assert_eq!(t.session_type, SessionType::ShortBreak);
    }
}
