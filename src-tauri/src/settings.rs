use crate::app_error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const SETTINGS_KEY: &str = "app_settings";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub overlay: OverlaySettings,
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), AppError> {
        self.overlay.validate()
    }

    pub fn normalized(&self) -> Result<Self, AppError> {
        Ok(Self {
            overlay: self.overlay.normalized()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlaySettings {
    pub width: u32,
    pub height: u32,
    pub opacity: f64,
    pub font_family: String,
    pub font_size: u32,
    pub text_color: String,
    pub background_opacity: f64,
    #[serde(default)]
    pub show_original_and_translation: bool,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            width: 760,
            height: 180,
            opacity: 0.94,
            font_family: "Inter, Segoe UI, system-ui, sans-serif".to_string(),
            font_size: 28,
            text_color: "#f8fafc".to_string(),
            background_opacity: 0.72,
            show_original_and_translation: false,
        }
    }
}

impl OverlaySettings {
    fn validate(&self) -> Result<(), AppError> {
        validate_range(self.width, 320, 1800, "Overlay width")?;
        validate_range(self.height, 96, 900, "Overlay height")?;
        validate_unit(self.opacity, "Overlay opacity")?;
        validate_range(self.font_size, 16, 72, "Overlay font size")?;
        validate_unit(self.background_opacity, "Overlay background opacity")?;
        validate_font_family(&self.font_family)?;
        validate_text_color(&self.text_color)?;
        Ok(())
    }

    fn normalized(&self) -> Result<Self, AppError> {
        let defaults = Self::default();
        let normalized = Self {
            width: self.width.clamp(320, 1800),
            height: self.height.clamp(96, 900),
            opacity: normalize_unit(self.opacity, defaults.opacity),
            font_family: self.font_family.trim().to_string(),
            font_size: self.font_size.clamp(16, 72),
            text_color: self.text_color.trim().to_string(),
            background_opacity: normalize_unit(
                self.background_opacity,
                defaults.background_opacity,
            ),
            show_original_and_translation: self.show_original_and_translation,
        };

        normalized.validate()?;
        Ok(normalized)
    }
}

pub struct SettingsService {
    db_path: PathBuf,
}

impl SettingsService {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(&app_data_dir)?;

        let service = Self {
            db_path: app_data_dir.join("feelsay.sqlite3"),
        };
        service.ensure_schema()?;
        service.ensure_defaults()?;
        Ok(service)
    }

    pub fn load(&self) -> Result<AppSettings, AppError> {
        let connection = self.open_connection()?;
        let stored = connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![SETTINGS_KEY],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        let Some(value) = stored else {
            return Ok(AppSettings::default());
        };

        let settings = serde_json::from_str::<AppSettings>(&value)
            .map_err(|error| AppError::Settings(format!("Saved settings are invalid: {error}")))?;
        settings.normalized()
    }

    pub fn save(&self, settings: AppSettings) -> Result<AppSettings, AppError> {
        let settings = settings.normalized()?;

        let value = serde_json::to_string(&settings)
            .map_err(|error| AppError::Settings(format!("Settings could not be saved: {error}")))?;
        let connection = self.open_connection()?;
        connection.execute(
            "INSERT INTO settings (key, value, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![SETTINGS_KEY, value],
        )?;

        Ok(settings)
    }

    fn ensure_schema(&self) -> Result<(), AppError> {
        let connection = self.open_connection()?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    fn ensure_defaults(&self) -> Result<(), AppError> {
        let connection = self.open_connection()?;
        let count = connection.query_row(
            "SELECT COUNT(*) FROM settings WHERE key = ?1",
            params![SETTINGS_KEY],
            |row| row.get::<_, u32>(0),
        )?;

        if count == 0 {
            self.save(AppSettings::default())?;
        }

        Ok(())
    }

    fn open_connection(&self) -> Result<Connection, AppError> {
        Ok(Connection::open(&self.db_path)?)
    }
}

fn validate_range(value: u32, min: u32, max: u32, label: &str) -> Result<(), AppError> {
    if (min..=max).contains(&value) {
        return Ok(());
    }

    Err(AppError::Settings(format!(
        "{label} must be between {min} and {max}."
    )))
}

fn validate_unit(value: f64, label: &str) -> Result<(), AppError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        return Ok(());
    }

    Err(AppError::Settings(format!(
        "{label} must be between 0 and 1."
    )))
}

fn normalize_unit(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        return value.clamp(0.0, 1.0);
    }

    fallback
}

fn validate_font_family(value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::Settings(
            "Overlay font family cannot be empty.".to_string(),
        ));
    }

    Ok(())
}

fn validate_text_color(value: &str) -> Result<(), AppError> {
    let hex = value.strip_prefix('#').ok_or_else(|| {
        AppError::Settings("Overlay text color must be a hex color like #f8fafc.".to_string())
    })?;

    if hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Ok(());
    }

    Err(AppError::Settings(
        "Overlay text color must be a hex color like #f8fafc.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn defaults_are_valid() {
        assert!(AppSettings::default().validate().is_ok());
    }

    #[test]
    fn settings_round_trip_through_sqlite() {
        let directory = unique_test_directory("round_trip");
        let service = SettingsService::new(directory.clone()).expect("service should initialize");

        let settings = AppSettings {
            overlay: OverlaySettings {
                width: 900,
                height: 220,
                opacity: 0.88,
                font_family: "Segoe UI".to_string(),
                font_size: 32,
                text_color: "#ffffff".to_string(),
                background_opacity: 0.64,
                show_original_and_translation: false,
            },
        };

        service
            .save(settings.clone())
            .expect("settings should save");

        assert_eq!(service.load().expect("settings should load"), settings);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn invalid_overlay_settings_return_safe_error() {
        let settings = AppSettings {
            overlay: OverlaySettings {
                opacity: 1.4,
                ..OverlaySettings::default()
            },
        };

        let error = settings.validate().expect_err("opacity should fail");
        assert_eq!(
            error.user_message(),
            "Overlay opacity must be between 0 and 1."
        );
    }

    #[test]
    fn save_clamps_numeric_overlay_settings() {
        let directory = unique_test_directory("clamps");
        let service = SettingsService::new(directory.clone()).expect("service should initialize");

        let settings = AppSettings {
            overlay: OverlaySettings {
                width: 1,
                height: 10_000,
                opacity: 2.0,
                font_size: 120,
                background_opacity: -1.0,
                ..OverlaySettings::default()
            },
        };

        let saved = service.save(settings).expect("settings should normalize");

        assert_eq!(saved.overlay.width, 320);
        assert_eq!(saved.overlay.height, 900);
        assert_eq!(saved.overlay.opacity, 1.0);
        assert_eq!(saved.overlay.font_size, 72);
        assert_eq!(saved.overlay.background_opacity, 0.0);
        let _ = fs::remove_dir_all(directory);
    }

    fn unique_test_directory(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();

        env::temp_dir().join(format!("feelsay_{name}_{timestamp}"))
    }
}
