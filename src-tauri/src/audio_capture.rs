use crate::app_error::AppError;
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::{AppHandle, Emitter};

const AUDIO_LEVEL_EVENT: &str = "audio-level";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioLevelEvent {
    pub source_ids: Vec<String>,
    pub level: f32,
    pub status: AudioLevelStatus,
    pub is_mock: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioLevelStatus {
    Idle,
    Starting,
    Active,
}

pub trait AudioCaptureSession {
    fn stop(&mut self);
}

pub struct AudioMeterService {
    session: Mutex<Option<MockAudioMeterSession>>,
}

impl AudioMeterService {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
        }
    }

    pub fn start_mock_meter(
        &self,
        app: AppHandle,
        source_ids: Vec<String>,
    ) -> Result<(), AppError> {
        self.stop()?;

        let mut session = self
            .session
            .lock()
            .map_err(|error| AppError::Audio(error.to_string()))?;
        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_stop_requested = Arc::clone(&stop_requested);
        let thread_source_ids = source_ids.clone();
        let source_offset = source_phase_offset(&source_ids);

        let handle = thread::Builder::new()
            .name("feelsay-audio-meter".to_string())
            .spawn(move || {
                let _ = emit_audio_level(&app, &thread_source_ids, 0.0, AudioLevelStatus::Starting);

                let mut tick = source_offset;
                while !thread_stop_requested.load(Ordering::Relaxed) {
                    tick = (tick + 0.115) % std::f32::consts::TAU;
                    let level = 0.18 + ((tick.sin() + 1.0) * 0.5 * 0.46);
                    let _ =
                        emit_audio_level(&app, &thread_source_ids, level, AudioLevelStatus::Active);
                    thread::sleep(Duration::from_millis(120));
                }

                let _ = emit_audio_level(&app, &thread_source_ids, 0.0, AudioLevelStatus::Idle);
            })
            .map_err(|error| AppError::Audio(error.to_string()))?;

        *session = Some(MockAudioMeterSession {
            stop_requested,
            handle: Some(handle),
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), AppError> {
        let session = self
            .session
            .lock()
            .map_err(|error| AppError::Audio(error.to_string()))?
            .take();

        if let Some(mut session) = session {
            session.stop();
        }

        Ok(())
    }
}

impl Default for AudioMeterService {
    fn default() -> Self {
        Self::new()
    }
}

struct MockAudioMeterSession {
    stop_requested: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl AudioCaptureSession for MockAudioMeterSession {
    fn stop(&mut self) {
        self.stop_requested.store(true, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for MockAudioMeterSession {
    fn drop(&mut self) {
        self.stop();
    }
}

fn emit_audio_level(
    app: &AppHandle,
    source_ids: &[String],
    level: f32,
    status: AudioLevelStatus,
) -> Result<(), tauri::Error> {
    app.emit(
        AUDIO_LEVEL_EVENT,
        AudioLevelEvent {
            source_ids: source_ids.to_vec(),
            level,
            status,
            is_mock: true,
        },
    )
}

fn source_phase_offset(source_ids: &[String]) -> f32 {
    let offset = source_ids
        .iter()
        .flat_map(|source_id| source_id.bytes())
        .fold(0u32, |sum, byte| sum.wrapping_add(byte as u32))
        % 53;

    (offset as f32 / 53.0) * std::f32::consts::TAU
}
