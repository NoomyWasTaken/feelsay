pub mod app_error;
pub mod app_state;
pub mod asr;
pub mod audio_capture;
pub mod commands;
pub mod overlay_settings;
pub mod platform;
pub mod source;
pub mod transcript;
pub mod translation;

use app_state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::new(app.path().app_data_dir()?));
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && matches!(event, tauri::WindowEvent::CloseRequested { .. })
            {
                println!("app: main close requested; cleaning up audio meter");
                let app = window.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    let _ = state.audio_meter().stop();
                    let _ = state.close_caption_process();
                    let _ = state.stop_overlay_placement();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_status,
            commands::start_audio_meter,
            commands::stop_audio_meter,
            commands::open_caption_window,
            commands::close_caption_window,
            commands::is_caption_window_open,
            commands::get_overlay_settings,
            commands::save_overlay_settings,
            commands::get_overlay_settings_store,
            commands::save_overlay_settings_store,
            commands::select_overlay_settings_profile,
            commands::start_overlay_placement,
            commands::get_overlay_placement,
            commands::stop_overlay_placement,
            commands::get_platform_capabilities,
            commands::list_available_sources,
            commands::get_source_previews
        ])
        .run(tauri::generate_context!())
        .expect("error while running Feelsay");
}
