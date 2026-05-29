use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSource {
    pub id: String,
    pub display_name: String,
    pub kind: SourceKind,
    pub is_available: bool,
    pub platform: String,
    pub metadata: Option<SourceMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMetadata {
    pub app_name: Option<String>,
    pub process_name: Option<String>,
    pub process_id: Option<u32>,
    pub process_path: Option<String>,
    pub window_title: Option<String>,
    pub endpoint_id: Option<String>,
    pub endpoint_flow: Option<AudioEndpointFlow>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioEndpointFlow {
    Capture,
    Render,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Application,
    Window,
    SystemAudio,
    OutputDevice,
    Microphone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSelection {
    pub mode: SourceSelectionMode,
    pub selected_source_ids: Vec<String>,
    pub display_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSelectionMode {
    Applications,
    System,
    Microphone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePreview {
    pub source_id: String,
    pub title: String,
    pub kind: SourceKind,
    pub thumbnail_url: Option<String>,
    pub thumbnail_data: Option<String>,
    pub is_live_preview_available: bool,
    pub css_preview: Option<CssPreviewMetadata>,
    pub mock_template: Option<MockPreviewTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssPreviewMetadata {
    pub label: String,
    pub accent_color: String,
    pub background: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MockPreviewTemplate {
    Browser,
    Chat,
    Meeting,
    Video,
    Game,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCapabilities {
    pub platform: String,
    pub source_enumeration_available: bool,
    pub window_capture_available: bool,
    pub system_audio_capture_available: bool,
    pub microphone_capture_available: bool,
    pub live_preview_available: bool,
    pub supports_system_audio: bool,
    pub supports_application_audio: bool,
    pub supports_microphone: bool,
    pub supports_live_preview: bool,
    pub supports_loopback_capture: bool,
}
