pub mod app_error;
pub mod app_state;
pub mod asr;
pub mod audio_capture;
pub mod commands;
pub mod overlay;
pub mod platform;
pub mod settings;
pub mod source;
pub mod transcript;
pub mod translation;

use app_state::AppState;
use settings::SettingsService;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let settings_service = SettingsService::new(data_dir)?;
            app.manage(AppState::new(settings_service));
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && matches!(event, tauri::WindowEvent::CloseRequested { .. })
            {
                println!("app: main close requested; cleaning up overlay and audio meter");
                let app = window.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    let _ = state.audio_meter().stop();
                    let _ = state.mark_overlay_closed();
                }
                if let Err(error) = overlay::destroy_overlay_window(app, "main window close") {
                    eprintln!("failed to destroy overlay during main close: {error}");
                }
            }

            if window.label() == overlay::OVERLAY_LABEL
                && matches!(event, tauri::WindowEvent::CloseRequested { .. })
            {
                println!("overlay: native close requested");
                let app = window.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    let _ = state.audio_meter().stop();
                    if state.mark_overlay_closed() {
                        if let Err(error) = app.emit(overlay::OVERLAY_CLOSED_EVENT, ()) {
                            eprintln!("failed to emit overlay closed event: {error}");
                        }
                    }
                }
            }

            if window.label() == overlay::OVERLAY_LABEL
                && matches!(event, tauri::WindowEvent::Destroyed)
            {
                println!("overlay: destroyed");
                let app = window.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    let _ = state.audio_meter().stop();
                    if state.mark_overlay_closed() {
                        if let Err(error) = app.emit(overlay::OVERLAY_CLOSED_EVENT, ()) {
                            eprintln!("failed to emit overlay closed event: {error}");
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_status,
            commands::get_settings,
            commands::save_settings,
            commands::show_overlay,
            commands::destroy_overlay,
            commands::hide_overlay,
            commands::start_audio_meter,
            commands::stop_audio_meter,
            commands::get_platform_capabilities,
            commands::list_available_sources,
            commands::get_source_previews
        ])
        .run(tauri::generate_context!())
        .expect("error while running Feelsay");
}
