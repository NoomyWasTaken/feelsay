use crate::{
    audio_capture::AudioMeterService,
    overlay_settings::{OverlayPlacement, OverlaySettings, OverlaySettingsService},
};
use std::{
    fs, io,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Mutex,
};

pub struct AppState {
    audio_meter_service: AudioMeterService,
    overlay_settings_service: OverlaySettingsService,
    caption_process: Mutex<Option<Child>>,
    placement_process: Mutex<Option<Child>>,
    placement_file: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            audio_meter_service: AudioMeterService::default(),
            overlay_settings_service: OverlaySettingsService::new(data_dir),
            caption_process: Mutex::new(None),
            placement_process: Mutex::new(None),
            placement_file: std::env::temp_dir().join(format!(
                "feelsay-overlay-placement-{}.json",
                std::process::id()
            )),
        }
    }

    pub fn audio_meter(&self) -> &AudioMeterService {
        &self.audio_meter_service
    }

    pub fn overlay_settings(&self) -> &OverlaySettingsService {
        &self.overlay_settings_service
    }

    pub fn open_caption_process(&self) -> Result<(), crate::app_error::AppError> {
        let mut caption_process = self.caption_process.lock().expect("caption process lock");

        if let Some(child) = caption_process.as_mut() {
            if child.try_wait()?.is_none() {
                return Ok(());
            }
        }

        let settings = self.overlay_settings_service.load()?;
        let settings = serde_json::to_string(&settings)
            .map_err(|error| crate::app_error::AppError::Window(error.to_string()))?;
        let child = Command::new(std::env::current_exe()?)
            .arg("--caption-window-child")
            .env("FEELSAY_OVERLAY_SETTINGS", settings)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        *caption_process = Some(child);
        Ok(())
    }

    pub fn close_caption_process(&self) -> std::io::Result<()> {
        let mut caption_process = self.caption_process.lock().expect("caption process lock");

        if let Some(child) = caption_process.as_mut() {
            if child.try_wait()?.is_none() {
                child.kill()?;
                let _ = child.wait();
            }
        }

        *caption_process = None;
        Ok(())
    }

    pub fn caption_process_is_running(&self) -> std::io::Result<bool> {
        let mut caption_process = self.caption_process.lock().expect("caption process lock");

        if let Some(child) = caption_process.as_mut() {
            if child.try_wait()?.is_none() {
                return Ok(true);
            }
        }

        *caption_process = None;
        Ok(false)
    }

    pub fn start_overlay_placement(
        &self,
        settings: OverlaySettings,
    ) -> Result<OverlayPlacement, crate::app_error::AppError> {
        self.stop_overlay_placement()?;

        let settings = settings.normalized();
        let placement = OverlayPlacement::from_settings(&settings);
        self.write_overlay_placement(placement)?;

        let settings = serde_json::to_string(&settings)
            .map_err(|error| crate::app_error::AppError::Window(error.to_string()))?;
        let child = Command::new(std::env::current_exe()?)
            .arg("--caption-placement-child")
            .env("FEELSAY_OVERLAY_SETTINGS", settings)
            .env("FEELSAY_OVERLAY_PLACEMENT_FILE", &self.placement_file)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let mut placement_process = self
            .placement_process
            .lock()
            .expect("placement process lock");
        *placement_process = Some(child);

        Ok(placement)
    }

    pub fn get_overlay_placement(
        &self,
    ) -> Result<Option<OverlayPlacement>, crate::app_error::AppError> {
        let is_running = self.placement_process_is_running()?;

        if !self.placement_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.placement_file)?;
        let placement = serde_json::from_str::<OverlayPlacement>(&content)
            .map_err(|error| crate::app_error::AppError::Io(error.to_string()))?;

        if !is_running && !placement.is_accepted {
            let _ = fs::remove_file(&self.placement_file);
            return Ok(None);
        }

        Ok(Some(placement))
    }

    pub fn stop_overlay_placement(&self) -> std::io::Result<()> {
        let mut placement_process = self
            .placement_process
            .lock()
            .expect("placement process lock");

        if let Some(child) = placement_process.as_mut() {
            if child.try_wait()?.is_none() {
                child.kill()?;
                let _ = child.wait();
            }
        }

        *placement_process = None;

        match fs::remove_file(&self.placement_file) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn placement_process_is_running(&self) -> std::io::Result<bool> {
        let mut placement_process = self
            .placement_process
            .lock()
            .expect("placement process lock");

        if let Some(child) = placement_process.as_mut() {
            if child.try_wait()?.is_none() {
                return Ok(true);
            }
        }

        *placement_process = None;
        Ok(false)
    }

    fn write_overlay_placement(&self, placement: OverlayPlacement) -> std::io::Result<()> {
        if let Some(parent) = self.placement_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string(&placement).map_err(io::Error::other)?;
        fs::write(&self.placement_file, content)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("feelsay"))
    }
}
