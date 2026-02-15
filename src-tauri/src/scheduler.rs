use crate::state::AppState;
use tauri::Manager;
use tokio::time::{self, Duration};
use chrono::{Timelike, Datelike, Local};

pub async fn start_scheduler_task(app_handle: tauri::AppHandle) {
    let mut interval = time::interval(Duration::from_secs(30)); // Check every 30 seconds

    loop {
        interval.tick().await;

        let state_guard = app_handle.state::<AppState>();
        
        // Load settings from database
        let settings = match state_guard.database.load_settings() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Scheduler task failed to load settings: {}", e);
                continue;
            }
        };

        if !settings.bandwidth_scheduler_enabled {
            // If scheduler is disabled, ensure we are using global limits from settings
            let mut app_settings = state_guard.settings.write().await;
            app_settings.download_limit = settings.max_download_speed;
            app_settings.upload_limit = settings.max_upload_speed;
            continue;
        }

        let now = Local::now();
        let current_time_str = format!("{:02}:{:02}", now.hour(), now.minute());
        let current_day = now.weekday().num_days_from_sunday() as u8; // 0=Sunday, 6=Saturday

        let mut active_rule = None;

        for rule in &settings.bandwidth_schedule {
            if !rule.enabled {
                continue;
            }

            // Check if today is in the rule's days
            if !rule.days.contains(&current_day) {
                continue;
            }

            // Check if current time is within [start_time, end_time]
            // Simple string comparison works for HH:MM format
            if current_time_str >= rule.start_time && current_time_str <= rule.end_time {
                active_rule = Some(rule);
                break; // Use the first matching rule
            }
        }

        let mut app_settings = state_guard.settings.write().await;
        if let Some(rule) = active_rule {
            if app_settings.download_limit != rule.download_limit || app_settings.upload_limit != rule.upload_limit {
                tracing::info!("Applying scheduled limits: DL={} UL={}", rule.download_limit, rule.upload_limit);
                app_settings.download_limit = rule.download_limit;
                app_settings.upload_limit = rule.upload_limit;
            }
        } else {
            // No active rule, fallback to default limits
            if app_settings.download_limit != settings.max_download_speed || app_settings.upload_limit != settings.max_upload_speed {
                tracing::info!("Resuming default limits: DL={} UL={}", settings.max_download_speed, settings.max_upload_speed);
                app_settings.download_limit = settings.max_download_speed;
                app_settings.upload_limit = settings.max_upload_speed;
            }
        }
    }
}
