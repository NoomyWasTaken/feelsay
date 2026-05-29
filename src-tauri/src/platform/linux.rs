use crate::{
    app_error::AppError,
    platform::{
        AudioCaptureProvider, PlatformCapabilityProvider, SourceEnumerator, SourcePreviewProvider,
    },
    source::{AudioSource, PlatformCapabilities, SourcePreview},
};

pub struct LinuxPlatformProvider;

impl SourceEnumerator for LinuxPlatformProvider {
    fn list_sources(&self) -> Result<Vec<AudioSource>, AppError> {
        Ok(Vec::new())
    }
}

impl SourcePreviewProvider for LinuxPlatformProvider {
    fn source_previews(&self) -> Result<Vec<SourcePreview>, AppError> {
        Ok(Vec::new())
    }
}

impl AudioCaptureProvider for LinuxPlatformProvider {
    fn capture_provider_name(&self) -> &'static str {
        "linux-placeholder"
    }
}

impl PlatformCapabilityProvider for LinuxPlatformProvider {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            platform: "linux".to_string(),
            source_enumeration_available: false,
            window_capture_available: false,
            system_audio_capture_available: false,
            microphone_capture_available: false,
            live_preview_available: false,
            supports_system_audio: false,
            supports_application_audio: false,
            supports_microphone: false,
            supports_live_preview: false,
            supports_loopback_capture: false,
        }
    }
}
