use crate::{
    app_error::{AppError, CommandResult},
    app_state::AppState,
    platform,
    platform::{PlatformCapabilityProvider, SourceEnumerator, SourcePreviewProvider},
    source::{AudioSource, PlatformCapabilities, SourcePreview},
};
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub state: SessionState,
    pub message: String,
    pub selected_source: Option<String>,
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
    }
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
pub fn open_caption_window(state: State<'_, AppState>) -> CommandResult<()> {
    state
        .open_caption_process()
        .map_err(|error| AppError::Window(error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn close_caption_window(state: State<'_, AppState>) -> CommandResult<()> {
    state
        .close_caption_process()
        .map_err(|error| AppError::Window(error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn is_caption_window_open(state: State<'_, AppState>) -> CommandResult<bool> {
    Ok(state
        .caption_process_is_running()
        .map_err(|error| AppError::Window(error.to_string()))?)
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
