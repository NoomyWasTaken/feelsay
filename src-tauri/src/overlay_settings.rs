use crate::app_error::AppError;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const FILE_NAME: &str = "overlay-settings.json";

#[derive(Debug, Clone)]
pub struct OverlaySettingsService {
    path: PathBuf,
}

impl OverlaySettingsService {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            path: data_dir.join(FILE_NAME),
        }
    }

    pub fn load(&self) -> Result<OverlaySettings, AppError> {
        if !self.path.exists() {
            return Ok(OverlaySettings::default());
        }

        let content = fs::read_to_string(&self.path)?;
        let settings = serde_json::from_str::<OverlaySettings>(&content)
            .map_err(|error| AppError::Io(error.to_string()))?;
        Ok(settings.normalized())
    }

    pub fn save(&self, settings: OverlaySettings) -> Result<OverlaySettings, AppError> {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlaySettings {
    pub font_family: String,
    pub font_size: u32,
    pub font_weight: FontWeight,
    pub text_color: String,
    pub background_color: String,
    pub background_opacity: f32,
    pub outline_color: String,
    pub outline_width: u32,
    pub start_width: u32,
    pub start_height: u32,
    pub start_x: Option<i32>,
    pub start_y: Option<i32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayPlacement {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_accepted: bool,
}

impl OverlayPlacement {
    pub fn from_settings(settings: &OverlaySettings) -> Self {
        Self {
            x: settings.start_x.unwrap_or(0),
            y: settings.start_y.unwrap_or(0),
            width: settings.start_width,
            height: settings.start_height,
            is_accepted: false,
        }
    }
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            font_family: "Segoe UI".to_string(),
            font_size: 42,
            font_weight: FontWeight::Bold,
            text_color: "#ffffff".to_string(),
            background_color: "#000000".to_string(),
            background_opacity: 0.47,
            outline_color: "#000000".to_string(),
            outline_width: 2,
            start_width: 900,
            start_height: 180,
            start_x: None,
            start_y: None,
        }
    }
}

impl OverlaySettings {
    pub fn normalized(mut self) -> Self {
        let defaults = Self::default();

        self.font_family = normalize_font_family(&self.font_family, &defaults.font_family);
        self.font_size = self.font_size.clamp(18, 96);
        self.text_color = normalize_color(&self.text_color).unwrap_or(defaults.text_color);
        self.background_color =
            normalize_color(&self.background_color).unwrap_or(defaults.background_color);
        self.background_opacity = self.background_opacity.clamp(0.0, 1.0);
        self.outline_color = normalize_color(&self.outline_color).unwrap_or(defaults.outline_color);
        self.outline_width = self.outline_width.clamp(0, 8);
        self.start_width = self.start_width.clamp(300, 1800);
        self.start_height = self.start_height.clamp(80, 600);

        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontWeight {
    Normal,
    Bold,
}

impl FontWeight {
    pub fn win32_weight(self) -> i32 {
        match self {
            Self::Normal => 400,
            Self::Bold => 700,
        }
    }
}

pub fn hex_to_rgb(color: &str) -> (u8, u8, u8) {
    let normalized = normalize_color(color).unwrap_or_else(|| "#ffffff".to_string());
    let value = normalized.trim_start_matches('#');
    let red = u8::from_str_radix(&value[0..2], 16).unwrap_or(255);
    let green = u8::from_str_radix(&value[2..4], 16).unwrap_or(255);
    let blue = u8::from_str_radix(&value[4..6], 16).unwrap_or(255);
    (red, green, blue)
}

fn normalize_font_family(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();

    if trimmed.is_empty() || trimmed.len() > 80 {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_color(value: &str) -> Option<String> {
    let trimmed = value.trim();

    if let Some(hex) = normalize_hex_color(trimmed) {
        return Some(hex);
    }

    normalize_rgb_color(trimmed)
}

fn normalize_hex_color(value: &str) -> Option<String> {
    let hex = value.strip_prefix('#')?;

    if hex.len() == 3 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        let mut expanded = String::from("#");
        for character in hex.chars() {
            expanded.push(character.to_ascii_lowercase());
            expanded.push(character.to_ascii_lowercase());
        }
        return Some(expanded);
    }

    if hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Some(format!("#{}", hex.to_ascii_lowercase()));
    }

    None
}

fn normalize_rgb_color(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let body = lower.strip_prefix("rgb(")?.strip_suffix(')')?;
    let mut channels = body.split(',').map(|part| part.trim().parse::<u8>());

    let red = channels.next()?.ok()?;
    let green = channels.next()?.ok()?;
    let blue = channels.next()?.ok()?;

    if channels.next().is_some() {
        return None;
    }

    Some(format!("#{red:02x}{green:02x}{blue:02x}"))
}
