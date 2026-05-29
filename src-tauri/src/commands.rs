use crate::{
    app_error::CommandResult,
    app_state::AppState,
    overlay, platform,
    platform::{PlatformCapabilityProvider, SourceEnumerator, SourcePreviewProvider},
    settings::AppSettings,
    source::{AudioSource, PlatformCapabilities, SourcePreview},
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub state: SessionState,
    pub message: String,
    pub selected_source: Option<String>,
    pub overlay_available: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Starting,
    Listening,
    Error,
}

#[tauri::command]
pub fn get_app_status() -> AppStatus {
    AppStatus {
        state: SessionState::Idle,
        message: "Ready to configure captions.".to_string(),
        selected_source: None,
        overlay_available: true,
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    Ok(state.settings().load()?)
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> CommandResult<AppSettings> {
    let settings = state.settings().save(settings)?;
    overlay::apply_overlay_settings(&app, &settings.overlay)?;
    app.emit("settings-updated", &settings)
        .map_err(crate::app_error::AppError::from)?;
    Ok(settings)
}

#[tauri::command]
pub fn show_overlay(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
    let settings = state.settings().load()?;
    overlay::show_overlay_window(&app, &settings.overlay)?;
    state.mark_overlay_open();
    app.emit(overlay::OVERLAY_OPENED_EVENT, ())
        .map_err(crate::app_error::AppError::from)?;
    Ok(())
}

#[tauri::command]
pub fn hide_overlay(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
    state.audio_meter().stop()?;

    let was_open = state.mark_overlay_closed();
    let destroyed = match overlay::destroy_overlay_window(&app, "command") {
        Ok(destroyed) => destroyed,
        Err(error) => {
            if was_open {
                state.mark_overlay_open();
            }
            return Err(error.into());
        }
    };

    if destroyed || was_open {
        app.emit(overlay::OVERLAY_CLOSED_EVENT, ())
            .map_err(crate::app_error::AppError::from)?;
    }
    Ok(())
}

#[tauri::command]
pub fn start_audio_meter(
    app: AppHandle,
    state: State<'_, AppState>,
    source_ids: Vec<String>,
) -> CommandResult<()> {
    state.audio_meter().start_mock_meter(app, source_ids)?;
    Ok(())
}

#[tauri::command]
pub fn stop_audio_meter(state: State<'_, AppState>) -> CommandResult<()> {
    state.audio_meter().stop()?;
    Ok(())
}

#[tauri::command]
pub fn get_platform_capabilities() -> PlatformCapabilities {
    platform::current_provider().capabilities()
}

#[tauri::command]
pub fn list_available_sources() -> CommandResult<Vec<AudioSource>> {
    Ok(platform::current_provider().list_sources()?)
}

#[tauri::command]
pub fn get_source_previews() -> CommandResult<Vec<SourcePreview>> {
    Ok(platform::current_provider().source_previews()?)
}
