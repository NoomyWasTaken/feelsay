use crate::audio_capture::AudioMeterService;
use std::{
    process::{Child, Command, Stdio},
    sync::Mutex,
};

pub struct AppState {
    audio_meter_service: AudioMeterService,
    caption_process: Mutex<Option<Child>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            audio_meter_service: AudioMeterService::default(),
            caption_process: Mutex::new(None),
        }
    }

    pub fn audio_meter(&self) -> &AudioMeterService {
        &self.audio_meter_service
    }

    pub fn open_caption_process(&self) -> std::io::Result<()> {
        let mut caption_process = self.caption_process.lock().expect("caption process lock");

        if let Some(child) = caption_process.as_mut() {
            if child.try_wait()?.is_none() {
                return Ok(());
            }
        }

        let child = Command::new(std::env::current_exe()?)
            .arg("--caption-window-child")
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
