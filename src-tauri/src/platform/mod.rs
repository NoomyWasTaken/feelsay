use crate::{
    app_error::AppError,
    source::{AudioSource, PlatformCapabilities, SourcePreview},
};

pub mod linux;
pub mod macos;
pub mod windows;

pub trait SourceEnumerator {
    fn list_sources(&self) -> Result<Vec<AudioSource>, AppError>;
}

pub trait SourcePreviewProvider {
    fn source_previews(&self) -> Result<Vec<SourcePreview>, AppError>;
}

pub trait AudioCaptureProvider {
    fn capture_provider_name(&self) -> &'static str;
}

pub trait PlatformCapabilityProvider {
    fn capabilities(&self) -> PlatformCapabilities;
}

pub trait PlatformProvider:
    SourceEnumerator + SourcePreviewProvider + AudioCaptureProvider + PlatformCapabilityProvider
{
}

impl<T> PlatformProvider for T where
    T: SourceEnumerator + SourcePreviewProvider + AudioCaptureProvider + PlatformCapabilityProvider
{
}

#[cfg(target_os = "windows")]
pub fn current_provider() -> windows::WindowsPlatformProvider {
    windows::WindowsPlatformProvider
}

#[cfg(target_os = "linux")]
pub fn current_provider() -> linux::LinuxPlatformProvider {
    linux::LinuxPlatformProvider
}

#[cfg(target_os = "macos")]
pub fn current_provider() -> macos::MacosPlatformProvider {
    macos::MacosPlatformProvider
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn current_provider() -> linux::LinuxPlatformProvider {
    linux::LinuxPlatformProvider
}
