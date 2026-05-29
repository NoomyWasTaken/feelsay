use crate::{
    app_error::AppError,
    platform::{
        AudioCaptureProvider, PlatformCapabilityProvider, SourceEnumerator, SourcePreviewProvider,
    },
    source::{AudioSource, PlatformCapabilities, SourcePreview},
};

pub struct MacosPlatformProvider;

impl SourceEnumerator for MacosPlatformProvider {
    fn list_sources(&self) -> Result<Vec<AudioSource>, AppError> {
        Ok(Vec::new())
    }
}

impl SourcePreviewProvider for MacosPlatformProvider {
    fn source_previews(&self) -> Result<Vec<SourcePreview>, AppError> {
        Ok(Vec::new())
    }
}

impl AudioCaptureProvider for MacosPlatformProvider {
    fn capture_provider_name(&self) -> &'static str {
        "macos-placeholder"
    }
}

impl PlatformCapabilityProvider for MacosPlatformProvider {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            platform: "macos".to_string(),
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
