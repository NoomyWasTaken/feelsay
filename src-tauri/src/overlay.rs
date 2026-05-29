use crate::{app_error::AppError, app_state::AppState, settings::OverlaySettings};
use tauri::{
    webview::Color, AppHandle, Emitter, LogicalSize, Manager, Size, WebviewUrl,
    WebviewWindowBuilder,
};

pub const OVERLAY_LABEL: &str = "overlay";
pub const OVERLAY_OPENED_EVENT: &str = "overlay-opened";
pub const OVERLAY_CLOSED_EVENT: &str = "overlay-closed";

pub fn show_overlay_window(app: &AppHandle, settings: &OverlaySettings) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        println!("overlay: existing overlay found; applying settings and focusing");
        window.set_background_color(Some(Color(0, 0, 0, 0)))?;
        window.set_size(Size::Logical(LogicalSize::new(
            settings.width as f64,
            settings.height as f64,
        )))?;
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    println!("overlay: creating overlay window");
    WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App("/overlay".into()))
        .title("FeelSay Overlay")
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .decorations(false)
        .always_on_top(true)
        .resizable(true)
        .inner_size(settings.width as f64, settings.height as f64)
        .min_inner_size(320.0, 96.0)
        .skip_taskbar(false)
        .build()?;

    println!("overlay: created overlay window");
    Ok(())
}

pub fn destroy_overlay_window(app: &AppHandle, reason: &str) -> Result<bool, AppError> {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        println!("overlay: destroying overlay window ({reason})");
        window.destroy()?;
        return Ok(true);
    }

    println!("overlay: destroy requested but no overlay exists ({reason})");
    Ok(false)
}

pub fn close_overlay(app: &AppHandle, state: &AppState, reason: &str) -> Result<(), AppError> {
    println!("overlay: close requested ({reason})");
    state.audio_meter().stop()?;

    let was_open = state.mark_overlay_closed();
    let destroyed = match destroy_overlay_window(app, reason) {
        Ok(destroyed) => destroyed,
        Err(error) => {
            if was_open {
                state.mark_overlay_open();
            }
            return Err(error);
        }
    };

    if was_open || destroyed {
        app.emit(OVERLAY_CLOSED_EVENT, ())?;
    }

    Ok(())
}

pub fn apply_overlay_settings(app: &AppHandle, settings: &OverlaySettings) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        window.set_background_color(Some(Color(0, 0, 0, 0)))?;
        window.set_size(Size::Logical(LogicalSize::new(
            settings.width as f64,
            settings.height as f64,
        )))?;
    }

    Ok(())
}
