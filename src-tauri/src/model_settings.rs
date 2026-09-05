use crate::app_error::AppError;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const FILE_NAME: &str = "model-settings.json";
const DEFAULT_MODEL_FILE: &str = "ggml-base.bin";
const DEFAULT_MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin";
const WHISPER_CPP_VERSION: &str = "v1.8.6";
const WHISPER_CPP_ZIP_URL: &str =
    "https://github.com/ggml-org/whisper.cpp/releases/download/v1.8.6/whisper-bin-x64.zip";

#[derive(Debug, Clone)]
pub struct ModelSettingsService {
    path: PathBuf,
    models_dir: PathBuf,
}

impl ModelSettingsService {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            path: data_dir.join(FILE_NAME),
            models_dir: data_dir.join("models"),
        }
    }

    pub fn load(&self) -> Result<ModelSettingsStore, AppError> {
        if !self.path.exists() {
            return Ok(ModelSettingsStore::default_for(&self.models_dir));
        }

        let content = fs::read_to_string(&self.path)?;
        let content = content.trim_start_matches('\u{feff}');
        let store = serde_json::from_str::<ModelSettingsStore>(content).map_err(|error| {
            AppError::Io(format!(
                "could not read model settings at {}: {error}",
                self.path.display()
            ))
        })?;

        Ok(store.normalized(&self.models_dir))
    }

    pub fn save(&self, store: ModelSettingsStore) -> Result<ModelSettingsStore, AppError> {
        let store = store.normalized(&self.models_dir);

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&store)
            .map_err(|error| AppError::Io(error.to_string()))?;
        fs::write(&self.path, content)?;

        Ok(store)
    }

    pub fn status(&self) -> Result<ModelStatus, AppError> {
        let store = self.load()?;
        Ok(ModelStatus::from_store(store))
    }

    pub fn install_default_assets(&self) -> Result<ModelSettingsStore, AppError> {
        fs::create_dir_all(&self.models_dir)?;
        let model_path = self.models_dir.join(DEFAULT_MODEL_FILE);
        let executable_path = self
            .models_dir
            .parent()
            .unwrap_or(&self.models_dir)
            .join("tools")
            .join(format!("whisper.cpp-{WHISPER_CPP_VERSION}"))
            .join("Release")
            .join("whisper-cli.exe");

        if !model_path.exists() {
            download_file(DEFAULT_MODEL_URL, &model_path)?;
        }

        if !executable_path.exists() {
            install_whisper_cpp(&executable_path)?;
        }

        if !model_path.exists() || !executable_path.exists() {
            return Err(AppError::Io(
                "default ASR assets were not installed correctly".to_string(),
            ));
        }

        let mut store = self.load()?;
        let model_index = store
            .models
            .iter()
            .position(|model| model.id == "whisper-base")
            .or(if store.models.is_empty() {
                None
            } else {
                Some(0)
            })
            .ok_or_else(|| AppError::Io("no model slot is available".to_string()))?;
        let model = &mut store.models[model_index];

        model.id = "whisper-base".to_string();
        model.name = "Whisper Base Multilingual".to_string();
        model.language = Some("Multilingual".to_string());
        model.path = model_path.to_string_lossy().to_string();
        model.executable_path = Some(executable_path.to_string_lossy().to_string());
        model.is_installed = true;
        model.file_size_bytes = file_size(&model.path);
        store.active_model_id = model.id.clone();

        self.save(store)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSettingsStore {
    pub active_model_id: String,
    pub models_dir: String,
    pub models: Vec<ModelMetadata>,
}

impl ModelSettingsStore {
    fn default_for(models_dir: &Path) -> Self {
        let default_path = models_dir.join(DEFAULT_MODEL_FILE);

        Self {
            active_model_id: "whisper-base".to_string(),
            models_dir: models_dir.to_string_lossy().to_string(),
            models: vec![ModelMetadata {
                id: "whisper-base".to_string(),
                name: "Whisper Base Multilingual".to_string(),
                runtime: "whisper.cpp".to_string(),
                language: Some("Multilingual".to_string()),
                path: default_path.to_string_lossy().to_string(),
                checksum_sha256: None,
                executable_path: None,
                file_size_bytes: None,
                is_installed: default_path.exists(),
                is_default: true,
                supports_translation: true,
            }],
        }
    }

    fn normalized(mut self, models_dir: &Path) -> Self {
        let fallback = Self::default_for(models_dir);

        if self.models_dir.trim().is_empty() {
            self.models_dir = fallback.models_dir;
        }

        if self.models.is_empty() {
            self.models = fallback.models;
        }

        for model in &mut self.models {
            let default_path = models_dir.join(DEFAULT_MODEL_FILE);
            let default_path = default_path.to_string_lossy();
            model.id = normalize_text(&model.id, "whisper-base");
            model.name = normalize_text(&model.name, "Whisper Base Multilingual");
            model.runtime = normalize_text(&model.runtime, "whisper.cpp");
            model.path = normalize_text(&model.path, &default_path);
            model.executable_path = normalize_optional_path(model.executable_path.as_deref());
            model.is_installed = Path::new(&model.path).exists();
            model.file_size_bytes = file_size(&model.path);
            model.supports_translation = model_supports_translation(model);
        }

        if !self
            .models
            .iter()
            .any(|model| model.id == self.active_model_id)
        {
            self.active_model_id = self.models[0].id.clone();
        }

        self
    }

    pub fn active_model(&self) -> Option<&ModelMetadata> {
        self.models
            .iter()
            .find(|model| model.id == self.active_model_id)
            .or_else(|| self.models.first())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub runtime: String,
    pub language: Option<String>,
    pub path: String,
    pub checksum_sha256: Option<String>,
    #[serde(default)]
    pub executable_path: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub is_installed: bool,
    pub is_default: bool,
    #[serde(default)]
    pub supports_translation: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub active_model: Option<ModelMetadata>,
    pub models_dir: String,
    pub is_ready: bool,
    pub message: String,
}

impl ModelStatus {
    fn from_store(store: ModelSettingsStore) -> Self {
        let active_model = store.active_model().cloned();
        let is_ready = active_model
            .as_ref()
            .map(model_runtime_ready)
            .unwrap_or(false);
        let message = match (is_ready, active_model.as_ref()) {
            (true, Some(model)) => format!("{} is ready.", model.name),
            (false, Some(model)) if model.is_installed && missing_runtime(model) => {
                "Missing whisper.cpp executable.".to_string()
            }
            (false, Some(model)) => format!("Missing model: {}", model.name),
            _ => "No ASR model configured.".to_string(),
        };

        Self {
            active_model,
            models_dir: store.models_dir,
            is_ready,
            message,
        }
    }
}

fn normalize_text(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn normalize_optional_path(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn file_size(path: &str) -> Option<u64> {
    fs::metadata(path).ok().map(|metadata| metadata.len())
}

pub fn model_supports_translation(model: &ModelMetadata) -> bool {
    let joined = [
        model.id.as_str(),
        model.name.as_str(),
        model.language.as_deref().unwrap_or(""),
        Path::new(&model.path)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or(""),
    ]
    .join(" ")
    .to_ascii_lowercase();

    !joined.contains(".en.")
        && !joined.contains(" english")
        && !joined.contains("english ")
        && !joined.ends_with("english")
        && !joined.contains("-en")
}

fn download_file(url: &str, path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(
            "& { param($Url, $OutFile) Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing }",
        )
        .arg(url)
        .arg(path)
        .status()?;

    if !status.success() {
        return Err(AppError::Io(format!("failed to download {url}")));
    }

    Ok(())
}

fn install_whisper_cpp(executable_path: &Path) -> Result<(), AppError> {
    let install_dir = executable_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| AppError::Io("invalid whisper.cpp install path".to_string()))?;
    fs::create_dir_all(install_dir)?;

    let zip_path = std::env::temp_dir().join(format!("whisper-bin-x64-{WHISPER_CPP_VERSION}.zip"));
    download_file(WHISPER_CPP_ZIP_URL, &zip_path)?;

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(
            "& { param($ZipPath, $Destination) Expand-Archive -Path $ZipPath -DestinationPath $Destination -Force }",
        )
        .arg(&zip_path)
        .arg(install_dir)
        .status()?;

    let _ = fs::remove_file(zip_path);

    if !status.success() {
        return Err(AppError::Io(
            "failed to extract whisper.cpp executable".to_string(),
        ));
    }

    Ok(())
}

fn model_runtime_ready(model: &ModelMetadata) -> bool {
    if !model.is_installed {
        return false;
    }

    #[cfg(feature = "local-asr")]
    {
        true
    }

    #[cfg(not(feature = "local-asr"))]
    {
        !missing_runtime(model)
    }
}

fn missing_runtime(model: &ModelMetadata) -> bool {
    !model
        .executable_path
        .as_deref()
        .map(|path| Path::new(path).exists())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(path: &str, language: Option<&str>) -> ModelMetadata {
        ModelMetadata {
            id: "test".to_string(),
            name: "Test model".to_string(),
            runtime: "whisper.cpp".to_string(),
            language: language.map(str::to_string),
            path: path.to_string(),
            checksum_sha256: None,
            executable_path: None,
            file_size_bytes: None,
            is_installed: false,
            is_default: false,
            supports_translation: false,
        }
    }

    #[test]
    fn english_only_models_do_not_support_translation() {
        assert!(!model_supports_translation(&model(
            "C:/models/ggml-base.en.bin",
            Some("English"),
        )));
    }

    #[test]
    fn multilingual_models_support_translation() {
        assert!(model_supports_translation(&model(
            "C:/models/ggml-base.bin",
            Some("Multilingual"),
        )));
    }
}
