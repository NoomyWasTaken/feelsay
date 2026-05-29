use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub original_text: String,
    pub translated_text: Option<String>,
    pub source_label: Option<String>,
    pub speaker_label: Option<String>,
    pub is_final: bool,
}

pub trait TranscriptStore {
    fn commit_segment(&self, segment: TranscriptSegment);
}
