use crate::{audio_capture::AudioMeterService, settings::SettingsService};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AppState {
    audio_meter_service: AudioMeterService,
    overlay_open: AtomicBool,
    settings_service: SettingsService,
}

impl AppState {
    pub fn new(settings_service: SettingsService) -> Self {
        Self {
            audio_meter_service: AudioMeterService::default(),
            overlay_open: AtomicBool::new(false),
            settings_service,
        }
    }

    pub fn audio_meter(&self) -> &AudioMeterService {
        &self.audio_meter_service
    }

    pub fn settings(&self) -> &SettingsService {
        &self.settings_service
    }

    pub fn mark_overlay_open(&self) -> bool {
        !self.overlay_open.swap(true, Ordering::SeqCst)
    }

    pub fn mark_overlay_closed(&self) -> bool {
        self.overlay_open.swap(false, Ordering::SeqCst)
    }
}
