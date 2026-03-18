// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod audio;
mod commands;
mod db;
mod error;
mod integrations;
mod models;
mod ollama;
mod timer;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use commands::AppState;
use error::MutexExt;
use models::TimerStatus;
use tauri::{Emitter, Manager};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use timer::{PomodoroTimer, TimerSettings};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Initialize database
            let conn = db::init_db(app.handle()).expect("Failed to initialize database");

            // Phase 2C: Load settings from DB and apply to timer
            // Frontend stores durations in MINUTES; Rust timer uses SECONDS
            let mut timer = PomodoroTimer::new();
            if let Ok(all_settings) = db::get_all_settings(&conn) {
                let mut settings = TimerSettings::default();
                for (key, value) in &all_settings {
                    if let Ok(parsed) = value.parse::<i32>() {
                        match key.as_str() {
                            "work_duration" => settings.work_duration = parsed * 60,
                            "short_break_duration" => settings.short_break = parsed * 60,
                            "long_break_duration" => settings.long_break = parsed * 60,
                            "cycles_before_long_break" => {
                                settings.cycles_before_long_break = parsed
                            }
                            _ => {}
                        }
                    }
                }
                timer.apply_settings(settings);
            }

            // Initialize audio engine (non-fatal — app works without audio)
            let sounds_dir = app
                .path()
                .resource_dir()
                .expect("Failed to get resource dir")
                .join("sounds");
            let audio_engine = match audio::AudioEngine::new(sounds_dir) {
                Ok(engine) => engine,
                Err(e) => {
                    eprintln!("Warning: Audio engine failed to initialize: {e}");
                    eprintln!("The app will run without sound support.");
                    audio::AudioEngine::silent()
                }
            };

            // Phase 5: Shutdown flag
            let shutdown = Arc::new(AtomicBool::new(false));

            // Create shared state
            let state = AppState {
                timer: Mutex::new(timer),
                db: Mutex::new(conn),
                audio: Mutex::new(audio_engine),
                shutdown: shutdown.clone(),
                ollama_cancel: Mutex::new(Arc::new(AtomicBool::new(false))),
                dispatcher: integrations::EventDispatcher::new(),
            };
            app.manage(state);

            // Set up system tray
            let start_pause = MenuItemBuilder::with_id("start_pause", "Start/Pause")
                .build(app)?;
            let skip = MenuItemBuilder::with_id("skip", "Skip")
                .build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&start_pause)
                .item(&skip)
                .separator()
                .item(&quit)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .tooltip("Pomodoro")
                .menu(&menu)
                .on_menu_event(move |app_handle, event| {
                    match event.id().as_ref() {
                        "start_pause" => {
                            let state = app_handle.state::<AppState>();
                            let lock_result = state.timer.lock_or_err("tray_start_pause");
                            if let Ok(mut timer) = lock_result {
                                match timer.status {
                                    TimerStatus::Idle => timer.start(),
                                    TimerStatus::Running => timer.pause(),
                                    TimerStatus::Paused => timer.resume(),
                                }
                                let timer_state = timer.get_state();
                                drop(timer);
                                let _ = app_handle.emit("timer:tick", &timer_state);
                            }
                        }
                        "skip" => {
                            let state = app_handle.state::<AppState>();
                            let lock_result = state.timer.lock_or_err("tray_skip");
                            if let Ok(mut timer) = lock_result {
                                timer.skip();
                                let timer_state = timer.get_state();
                                drop(timer);
                                let _ = app_handle.emit("timer:tick", &timer_state);
                            }
                        }
                        "quit" => {
                            let state = app_handle.state::<AppState>();
                            state.shutdown.store(true, Ordering::Relaxed);
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Start timer tick loop in background thread
            let app_handle = app.handle().clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(1));

                    let state = app_handle.state::<AppState>();

                    // Phase 5: Check shutdown flag
                    if state.shutdown.load(Ordering::Relaxed) {
                        break;
                    }

                    let lock_result = state.timer.lock_or_err("tick");
                    if let Ok(mut timer) = lock_result {
                        if timer.status == TimerStatus::Running {
                            let session_completed = timer.tick();
                            let timer_state = timer.get_state();

                            // If session completed, mark it in DB
                            if session_completed {
                                if let Some(session_id) = timer.current_session_id.clone() {
                                    let ended_at = chrono::Utc::now().to_rfc3339();
                                    let db_lock = state.db.lock_or_err("tick_db");
                                    if let Ok(conn) = db_lock {
                                        let _ =
                                            db::complete_session(&conn, &session_id, &ended_at);
                                    }
                                }
                            }

                            drop(timer);
                            let _ = app_handle.emit("timer:tick", &timer_state);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_timer,
            commands::pause_timer,
            commands::resume_timer,
            commands::skip_timer,
            commands::reset_timer,
            commands::get_timer_state,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::list_tasks,
            commands::reorder_tasks,
            commands::start_pomodoro_for_task,
            commands::complete_pomodoro,
            commands::get_sessions_for_task,
            commands::get_setting,
            commands::set_setting,
            commands::get_all_settings,
            commands::get_daily_stats,
            commands::get_weekly_stats,
            commands::get_heatmap_data,
            commands::get_streak_data,
            commands::get_time_of_day_stats,
            commands::get_project_stats,
            commands::set_goal,
            commands::get_goals,
            commands::invoke_ai_command,
            commands::get_daily_briefing,
            commands::get_session_debrief,
            commands::get_weekly_report,
            commands::get_settings_advice,
            commands::play_sound,
            commands::stop_sound,
            commands::set_sound_volume,
            commands::set_master_volume,
            commands::fade_sounds,
            commands::stop_all_sounds,
            commands::get_audio_state,
            commands::get_available_sounds,
            commands::save_sound_preset,
            commands::list_sound_presets,
            commands::load_sound_preset,
            commands::delete_sound_preset,
            commands::create_integration,
            commands::update_integration,
            commands::delete_integration,
            commands::list_integrations,
            commands::test_integration,
            commands::get_event_log,
            commands::ollama_check_health,
            commands::ollama_list_local_models,
            commands::ollama_get_curated_models,
            commands::ollama_pull_model,
            commands::ollama_cancel_pull,
            commands::ollama_delete_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
