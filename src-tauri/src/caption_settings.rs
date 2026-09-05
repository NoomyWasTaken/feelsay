use crate::{
    app_error::AppError,
    translation::{
        default_translation_source_language, default_translation_target_language,
        TranslationLanguage,
    },
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const FILE_NAME: &str = "caption-settings.json";

#[derive(Debug, Clone)]
pub struct CaptionSettingsService {
    path: PathBuf,
}

impl CaptionSettingsService {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            path: data_dir.join(FILE_NAME),
        }
    }

    pub fn load(&self) -> Result<CaptionSettings, AppError> {
        if !self.path.exists() {
            return Ok(CaptionSettings::default());
        }

        let content = fs::read_to_string(&self.path)?;
        let content = content.trim_start_matches('\u{feff}');
        if content.trim().is_empty() {
            return Ok(CaptionSettings::default());
        }

        let settings = serde_json::from_str::<CaptionSettings>(content).map_err(|error| {
            AppError::Io(format!(
                "could not read caption settings at {}: {error}",
                self.path.display()
            ))
        })?;

        Ok(settings.normalized())
    }

    pub fn save(&self, settings: CaptionSettings) -> Result<CaptionSettings, AppError> {
        let settings = settings.normalized();

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&settings)
            .map_err(|error| AppError::Io(error.to_string()))?;
        fs::write(&self.path, content)?;

        Ok(settings)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CaptionSettings {
    #[serde(default)]
    pub mode: CaptionMode,
    #[serde(default)]
    pub speaker_labels_enabled: bool,
    #[serde(default = "default_translation_source_language")]
    pub translation_source_language: TranslationLanguage,
    #[serde(default = "default_translation_target_language")]
    pub translation_target_language: TranslationLanguage,
}

impl CaptionSettings {
    fn normalized(mut self) -> Self {
        if self.translation_target_language == TranslationLanguage::Auto {
            self.translation_target_language = TranslationLanguage::English;
        }
        self
    }
}

impl Default for CaptionSettings {
    fn default() -> Self {
        Self {
            mode: CaptionMode::Captions,
            speaker_labels_enabled: false,
            translation_source_language: default_translation_source_language(),
            translation_target_language: default_translation_target_language(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaptionMode {
    #[default]
    Captions,
    Translate,
    OriginalAndTranslation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caption_settings_default_to_captions_only() {
        assert_eq!(CaptionSettings::default().mode, CaptionMode::Captions);
        assert!(!CaptionSettings::default().speaker_labels_enabled);
        assert_eq!(
            CaptionSettings::default().translation_target_language,
            TranslationLanguage::English
        );
    }
}
