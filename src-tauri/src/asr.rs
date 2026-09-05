use crate::{
    app_error::AppError,
    audio_capture::{
        capture_default_loopback_frame, DiagnosticVoiceActivityDetector, PcmAudioFrame,
    },
    caption_settings::CaptionMode,
    diarization::{ExperimentalSpeakerLabeler, SpeakerLabel, SpeakerLabelConfidence},
    performance_settings::PerformanceRuntimeConfig,
    transcript::{TranscriptRuntimeConfig, TranscriptSegment},
    translation::{TranslationLanguage, TranslationRuntimeConfig},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command},
    sync::mpsc::{sync_channel, SyncSender, TrySendError},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "local-asr")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const TARGET_SAMPLE_RATE: u32 = 16_000;
const CAPTION_MAX_LINES: usize = 2;
const CAPTION_MAX_CHARS_PER_LINE: usize = 48;

pub trait AsrEngine {
    fn engine_id(&self) -> &'static str;
    fn transcribe(
        &self,
        samples: &[f32],
        mode: CaptionMode,
        translation: Option<&TranslationRuntimeConfig>,
    ) -> Result<AsrOutput, AppError>;
}

#[derive(Debug, Clone)]
pub struct AsrRuntimeConfig {
    pub model_path: PathBuf,
    pub executable_path: Option<PathBuf>,
    pub caption_path: PathBuf,
    pub transcript: Option<TranscriptRuntimeConfig>,
    pub caption_mode: CaptionMode,
    pub translation: Option<TranslationRuntimeConfig>,
    pub performance: PerformanceRuntimeConfig,
    pub source_label: Option<String>,
    pub speaker_labels_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsrDiagnosticResult {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrOutput {
    pub original_text: String,
    pub translated_text: Option<String>,
    pub display_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptionRuntimeState {
    #[serde(default)]
    pub committed_text: String,
    #[serde(default)]
    pub provisional_text: String,
    pub text: String,
    pub is_provisional: bool,
    #[serde(default)]
    pub updated_at_ms: u64,
    #[serde(default = "default_caption_max_lines")]
    pub max_lines: usize,
    #[serde(default)]
    pub source_label: Option<String>,
    #[serde(default)]
    pub speaker_label: Option<String>,
    #[serde(default)]
    pub speaker_confidence: Option<SpeakerLabelConfidence>,
}

pub struct WhisperAsrEngine {
    external: Option<ExternalWhisperCliEngine>,
    #[cfg(feature = "local-asr")]
    context: Option<WhisperContext>,
}

impl WhisperAsrEngine {
    pub fn load(config: &AsrRuntimeConfig) -> Result<Self, AppError> {
        if let Some(executable_path) = &config.executable_path {
            return Ok(Self {
                external: Some(ExternalWhisperCliEngine::new(
                    executable_path.clone(),
                    config.model_path.clone(),
                )?),
                #[cfg(feature = "local-asr")]
                context: None,
            });
        }

        #[cfg(not(feature = "local-asr"))]
        {
            Err(AppError::Asr(
                "set a whisper.cpp executable path in Settings before starting local ASR"
                    .to_string(),
            ))
        }

        #[cfg(feature = "local-asr")]
        {
            let model_path = &config.model_path;
            if !model_path.exists() {
                return Err(AppError::Asr(format!(
                    "model file does not exist: {}",
                    model_path.display()
                )));
            }

            let model_path = model_path
                .to_str()
                .ok_or_else(|| AppError::Asr("model path is not valid UTF-8".to_string()))?;
            let context =
                WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
                    .map_err(|error| AppError::Asr(error.to_string()))?;

            Ok(Self {
                external: None,
                context: Some(context),
            })
        }
    }
}

pub fn run_asr_diagnostic(config: AsrRuntimeConfig) -> Result<AsrDiagnosticResult, AppError> {
    let speech_path = std::env::temp_dir().join(format!(
        "feelsay-asr-diagnostic-{}.wav",
        current_timestamp_ms()
    ));
    synthesize_diagnostic_wav(&speech_path)?;

    let Some(executable_path) = config.executable_path else {
        let _ = fs::remove_file(speech_path);
        return Err(AppError::Asr(
            "ASR diagnostic requires a whisper.cpp executable path".to_string(),
        ));
    };

    let engine = ExternalWhisperCliEngine::new(executable_path, config.model_path)?;
    let text = engine.transcribe_wav(&speech_path, config.caption_mode != CaptionMode::Captions)?;
    let _ = fs::remove_file(speech_path);

    Ok(AsrDiagnosticResult { text })
}

pub fn run_asr_file_diagnostic(
    config: AsrRuntimeConfig,
    audio_path: PathBuf,
) -> Result<AsrDiagnosticResult, AppError> {
    let engine = WhisperAsrEngine::load(&config)?;
    let audio = read_pcm16_wav(&audio_path)?;
    let samples = resample_to_16k_mono(&PcmAudioFrame {
        source_ids: vec!["diagnostic:file".to_string()],
        samples: audio.samples,
        sample_rate: audio.sample_rate,
        channels: audio.channels,
        timestamp_ms: current_timestamp_ms(),
    });
    let output = engine.transcribe(&samples, config.caption_mode, config.translation.as_ref())?;

    Ok(AsrDiagnosticResult {
        text: output.display_text,
    })
}

pub fn run_asr_pipeline_diagnostic(
    config: AsrRuntimeConfig,
) -> Result<AsrDiagnosticResult, AppError> {
    let speech_path = std::env::temp_dir().join(format!(
        "feelsay-asr-pipeline-diagnostic-{}.wav",
        current_timestamp_ms()
    ));
    synthesize_diagnostic_wav(&speech_path)?;

    let audio = read_pcm16_wav(&speech_path)?;
    let _ = fs::remove_file(speech_path);

    let caption_path = config.caption_path.clone();
    let mut runtime = AsrCaptureRuntime::new(config)?;
    let chunk_samples = ((audio.sample_rate / 5).max(1) as usize) * audio.channels as usize;
    for chunk in audio.samples.chunks(chunk_samples) {
        runtime.push_frame(
            &PcmAudioFrame {
                source_ids: vec!["diagnostic:generated-speech".to_string()],
                samples: chunk.to_vec(),
                sample_rate: audio.sample_rate,
                channels: audio.channels,
                timestamp_ms: current_timestamp_ms(),
            },
            true,
        );
    }

    let content = fs::read_to_string(&caption_path)?;
    let caption = serde_json::from_str::<CaptionRuntimeState>(&content)
        .map_err(|error| AppError::Asr(error.to_string()))?;

    Ok(AsrDiagnosticResult { text: caption.text })
}

pub fn run_system_audio_asr_diagnostic(
    config: AsrRuntimeConfig,
) -> Result<AsrDiagnosticResult, AppError> {
    let mut speech = play_diagnostic_speech_to_default_output()?;
    let frame = capture_default_loopback_frame(Duration::from_secs(7))?;
    let _ = speech.wait();

    if frame.samples.is_empty() {
        return Err(AppError::Audio(
            "system audio diagnostic captured no samples".to_string(),
        ));
    }

    let caption_path = config.caption_path.clone();
    let mut runtime = AsrCaptureRuntime::new(config)?;
    let mut vad = DiagnosticVoiceActivityDetector::new();
    let chunk_samples = ((frame.sample_rate / 5).max(1) as usize) * frame.channels as usize;
    for chunk in frame.samples.chunks(chunk_samples) {
        let chunk_frame = PcmAudioFrame {
            source_ids: frame.source_ids.clone(),
            samples: chunk.to_vec(),
            sample_rate: frame.sample_rate,
            channels: frame.channels,
            timestamp_ms: current_timestamp_ms(),
        };
        let speech_detected = vad.update_frame(&chunk_frame);
        runtime.push_frame(&chunk_frame, speech_detected);
    }

    let content = fs::read_to_string(&caption_path)?;
    let caption = serde_json::from_str::<CaptionRuntimeState>(&content)
        .map_err(|error| AppError::Asr(error.to_string()))?;

    Ok(AsrDiagnosticResult { text: caption.text })
}

impl AsrEngine for WhisperAsrEngine {
    fn engine_id(&self) -> &'static str {
        "whisper-rs"
    }

    fn transcribe(
        &self,
        samples: &[f32],
        mode: CaptionMode,
        translation: Option<&TranslationRuntimeConfig>,
    ) -> Result<AsrOutput, AppError> {
        if let Some(external) = &self.external {
            return external.transcribe(samples, mode, translation);
        }

        #[cfg(not(feature = "local-asr"))]
        {
            let _ = samples;
            Err(AppError::Asr(
                "local ASR feature is not enabled in this build".to_string(),
            ))
        }

        #[cfg(feature = "local-asr")]
        {
            if mode != CaptionMode::Captions
                && translation
                    .map(|config| config.target_language().is_english())
                    .unwrap_or(true)
            {
                return Err(AppError::Asr(
                    "translation mode currently requires a whisper.cpp executable path".to_string(),
                ));
            }

            let context = self
                .context
                .as_ref()
                .ok_or_else(|| AppError::Asr("local ASR engine is not configured".to_string()))?;
            if samples.is_empty() {
                return Ok(AsrOutput::captions(String::new()));
            }

            let mut state = context
                .create_state()
                .map_err(|error| AppError::Asr(error.to_string()))?;
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_language(Some("en"));
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);

            state
                .full(params, samples)
                .map_err(|error| AppError::Asr(error.to_string()))?;

            let text = state
                .as_iter()
                .map(|segment| segment.to_string())
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();

            if mode == CaptionMode::Captions {
                return Ok(AsrOutput::captions(text));
            }

            let translation = translation.ok_or_else(|| {
                AppError::Translation("translation runtime is not configured".to_string())
            })?;
            let translated_text = translation.translate_text(&text)?;

            match mode {
                CaptionMode::Captions => Ok(AsrOutput::captions(text)),
                CaptionMode::Translate => {
                    Ok(AsrOutput::translation_with_original(text, translated_text))
                }
                CaptionMode::OriginalAndTranslation => {
                    Ok(AsrOutput::original_and_translation(text, translated_text))
                }
            }
        }
    }
}

struct ExternalWhisperCliEngine {
    executable_path: PathBuf,
    model_path: PathBuf,
}

impl ExternalWhisperCliEngine {
    fn new(executable_path: PathBuf, model_path: PathBuf) -> Result<Self, AppError> {
        if !executable_path.exists() {
            return Err(AppError::Asr(format!(
                "whisper executable does not exist: {}",
                executable_path.display()
            )));
        }

        if !model_path.exists() {
            return Err(AppError::Asr(format!(
                "model file does not exist: {}",
                model_path.display()
            )));
        }

        Ok(Self {
            executable_path,
            model_path,
        })
    }

    fn transcribe(
        &self,
        samples: &[f32],
        mode: CaptionMode,
        translation: Option<&TranslationRuntimeConfig>,
    ) -> Result<AsrOutput, AppError> {
        if samples.is_empty() {
            return Ok(AsrOutput::captions(String::new()));
        }

        let id = format!("{}-{}", std::process::id(), current_timestamp_ms());
        let base_path = std::env::temp_dir().join(format!("feelsay-asr-{id}"));
        let wav_path = base_path.with_extension("wav");

        write_mono_wav(&wav_path, samples, TARGET_SAMPLE_RATE)?;

        let output = self.transcribe_wav_for_mode(&wav_path, mode, translation)?;
        let _ = fs::remove_file(wav_path);

        Ok(output)
    }

    fn transcribe_wav_for_mode(
        &self,
        wav_path: &std::path::Path,
        mode: CaptionMode,
        translation: Option<&TranslationRuntimeConfig>,
    ) -> Result<AsrOutput, AppError> {
        match mode {
            CaptionMode::Captions => Ok(AsrOutput::captions(self.transcribe_wav(wav_path, false)?)),
            CaptionMode::Translate => {
                if should_use_text_translation(translation) {
                    let original_text = self.transcribe_wav(wav_path, false)?;
                    let translated_text = translation
                        .expect("checked by should_use_text_translation")
                        .translate_text(&original_text)?;
                    Ok(AsrOutput::translation_with_original(
                        original_text,
                        translated_text,
                    ))
                } else {
                    Ok(AsrOutput::translation(self.transcribe_wav(wav_path, true)?))
                }
            }
            CaptionMode::OriginalAndTranslation => {
                let original_text = self.transcribe_wav(wav_path, false)?;
                let translated_text = if should_use_text_translation(translation) {
                    translation
                        .expect("checked by should_use_text_translation")
                        .translate_text(&original_text)?
                } else {
                    self.transcribe_wav(wav_path, true)?
                };
                Ok(AsrOutput::original_and_translation(
                    original_text,
                    translated_text,
                ))
            }
        }
    }

    fn transcribe_wav(
        &self,
        wav_path: &std::path::Path,
        should_translate: bool,
    ) -> Result<String, AppError> {
        let base_path = wav_path.with_extension("");
        let txt_path = base_path.with_extension("txt");
        let _ = fs::remove_file(&txt_path);

        let output = Command::new(&self.executable_path)
            .arg("-m")
            .arg(&self.model_path)
            .arg("-f")
            .arg(wav_path)
            .arg("-otxt")
            .arg("-of")
            .arg(&base_path)
            .arg("-nt")
            .arg("-np")
            .arg("-l")
            .arg("auto")
            .args(if should_translate {
                &["--translate"][..]
            } else {
                &[]
            })
            .output()?;

        let text = if txt_path.exists() {
            fs::read_to_string(&txt_path)?
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        };

        let _ = fs::remove_file(txt_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Asr(format!(
                "whisper executable failed: {}",
                stderr.trim()
            )));
        }

        Ok(text.trim().to_string())
    }
}

pub struct AsrCaptureRuntime {
    engine: WhisperAsrEngine,
    caption_path: PathBuf,
    transcript: Option<TranscriptRuntimeConfig>,
    caption_mode: CaptionMode,
    translation: Option<TranslationRuntimeConfig>,
    performance: PerformanceRuntimeConfig,
    source_label: Option<String>,
    speaker_labeler: ExperimentalSpeakerLabeler,
    samples: Vec<f32>,
    last_transcription: Option<Instant>,
    segment_start_ms: Option<u64>,
    last_transcript_text: String,
}

pub struct AsrCaptureWorker {
    sender: Option<SyncSender<AsrWorkItem>>,
    handle: Option<JoinHandle<()>>,
}

impl AsrCaptureWorker {
    pub fn start(config: AsrRuntimeConfig) -> Result<Self, AppError> {
        let queue_capacity = config.performance.asr_queue_capacity;
        let mut runtime = AsrCaptureRuntime::new(config)?;
        let (sender, receiver) = sync_channel::<AsrWorkItem>(queue_capacity);
        let handle = thread::Builder::new()
            .name("feelsay-asr-worker".to_string())
            .spawn(move || {
                while let Ok(item) = receiver.recv() {
                    runtime.push_frame(&item.frame, item.speech_detected);
                }
            })
            .map_err(|error| AppError::Asr(error.to_string()))?;

        Ok(Self {
            sender: Some(sender),
            handle: Some(handle),
        })
    }

    pub fn push_frame(&self, frame: &PcmAudioFrame, speech_detected: bool) {
        if !speech_detected {
            return;
        }

        let Some(sender) = &self.sender else {
            return;
        };

        match sender.try_send(AsrWorkItem {
            frame: frame.clone(),
            speech_detected,
        }) {
            Ok(()) | Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => {}
        }
    }
}

impl Drop for AsrCaptureWorker {
    fn drop(&mut self) {
        drop(self.sender.take());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct AsrWorkItem {
    frame: PcmAudioFrame,
    speech_detected: bool,
}

impl AsrCaptureRuntime {
    pub fn new(config: AsrRuntimeConfig) -> Result<Self, AppError> {
        write_labeled_caption_state(
            &config.caption_path,
            "",
            "Listening",
            config.source_label.as_deref(),
            None,
        )?;
        let performance = config.performance.clone();
        let speaker_labels_enabled = config.speaker_labels_enabled;

        Ok(Self {
            engine: WhisperAsrEngine::load(&config)?,
            caption_path: config.caption_path,
            transcript: config.transcript,
            caption_mode: config.caption_mode,
            translation: config.translation,
            source_label: config.source_label,
            speaker_labeler: ExperimentalSpeakerLabeler::new(speaker_labels_enabled),
            samples: Vec::with_capacity(performance.max_transcribe_samples),
            performance,
            last_transcription: None,
            segment_start_ms: None,
            last_transcript_text: String::new(),
        })
    }

    pub fn push_frame(&mut self, frame: &PcmAudioFrame, speech_detected: bool) {
        if !speech_detected {
            return;
        }

        if self.segment_start_ms.is_none() {
            self.segment_start_ms = Some(frame.timestamp_ms);
        }

        self.samples.extend(resample_to_16k_mono(frame));

        if self.samples.len() > self.performance.max_transcribe_samples {
            let extra = self.samples.len() - self.performance.max_transcribe_samples;
            self.samples.drain(0..extra);
        }

        if self.samples.len() < self.performance.min_transcribe_samples
            || !self.transcription_interval_elapsed()
        {
            return;
        }

        let samples = self.samples.clone();
        self.last_transcription = Some(Instant::now());

        match self
            .engine
            .transcribe(&samples, self.caption_mode, self.translation.as_ref())
        {
            Ok(output) => {
                let output = output.stabilized();
                if !output.display_text.is_empty() {
                    let speaker_label = self.speaker_labeler.label_for_samples(&samples);
                    let _ = write_labeled_caption_state(
                        &self.caption_path,
                        "",
                        &output.display_text,
                        self.source_label.as_deref(),
                        speaker_label.as_ref(),
                    );
                    self.append_transcript_output(&output, frame.timestamp_ms, speaker_label);
                }
            }
            Err(error) => {
                eprintln!("{error}");
                let _ = write_caption_state(&self.caption_path, "", "Transcription error");
            }
        }
    }

    fn append_transcript_output(
        &mut self,
        output: &AsrOutput,
        end_ms: u64,
        speaker_label: Option<SpeakerLabel>,
    ) {
        if output.display_text == self.last_transcript_text {
            return;
        }

        let Some(transcript) = &self.transcript else {
            return;
        };

        let start_ms = self.segment_start_ms.unwrap_or(end_ms);
        let _ = transcript.append_segment(TranscriptSegment {
            start_ms,
            end_ms,
            original_text: output.original_text.clone(),
            translated_text: output.translated_text.clone(),
            source_label: self.source_label.clone(),
            speaker_label: speaker_label.as_ref().map(|label| label.label.clone()),
            speaker_confidence: speaker_label.map(|label| label.confidence),
            is_final: true,
        });
        self.last_transcript_text = output.display_text.clone();
        self.segment_start_ms = Some(end_ms);
    }

    fn transcription_interval_elapsed(&self) -> bool {
        self.last_transcription
            .map(|last| last.elapsed().as_millis() >= self.performance.transcribe_interval_ms)
            .unwrap_or(true)
    }
}

impl AsrOutput {
    fn captions(original_text: String) -> Self {
        let display_text = original_text.clone();
        Self {
            original_text,
            translated_text: None,
            display_text,
        }
    }

    fn translation(translated_text: String) -> Self {
        Self {
            original_text: String::new(),
            display_text: translated_text.clone(),
            translated_text: Some(translated_text),
        }
    }

    fn translation_with_original(original_text: String, translated_text: String) -> Self {
        Self {
            original_text,
            display_text: translated_text.clone(),
            translated_text: Some(translated_text),
        }
    }

    fn original_and_translation(original_text: String, translated_text: String) -> Self {
        let display_text = [original_text.trim(), translated_text.trim()]
            .into_iter()
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        Self {
            original_text,
            translated_text: Some(translated_text),
            display_text,
        }
    }

    fn stabilized(self) -> Self {
        Self {
            original_text: stabilize_caption_text(&self.original_text),
            translated_text: self
                .translated_text
                .map(|text| stabilize_caption_text(&text))
                .filter(|text| !text.is_empty()),
            display_text: stabilize_caption_text(&self.display_text),
        }
    }
}

fn should_use_text_translation(translation: Option<&TranslationRuntimeConfig>) -> bool {
    translation
        .map(|config| config.target_language() != TranslationLanguage::English)
        .unwrap_or(false)
}

pub fn write_caption_state(
    path: &std::path::Path,
    committed_text: &str,
    provisional_text: &str,
) -> Result<(), AppError> {
    write_labeled_caption_state(path, committed_text, provisional_text, None, None)
}

pub fn write_labeled_caption_state(
    path: &std::path::Path,
    committed_text: &str,
    provisional_text: &str,
    source_label: Option<&str>,
    speaker_label: Option<&SpeakerLabel>,
) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let committed_text = stabilize_caption_text(committed_text);
    let provisional_text = stabilize_caption_text(provisional_text);
    let text = if provisional_text.is_empty() {
        committed_text.clone()
    } else {
        provisional_text.clone()
    };

    let content = serde_json::to_string(&CaptionRuntimeState {
        committed_text,
        provisional_text,
        text,
        is_provisional: true,
        updated_at_ms: current_timestamp_ms(),
        max_lines: CAPTION_MAX_LINES,
        source_label: source_label.and_then(normalize_source_label),
        speaker_label: speaker_label
            .map(|label| label.label.as_str())
            .and_then(normalize_source_label),
        speaker_confidence: speaker_label.map(|label| label.confidence),
    })
    .map_err(|error| AppError::Asr(error.to_string()))?;
    fs::write(path, content)?;
    Ok(())
}

fn normalize_source_label(label: &str) -> Option<String> {
    let label = label.split_whitespace().collect::<Vec<_>>().join(" ");
    (!label.is_empty()).then_some(label)
}

fn stabilize_caption_text(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_caption_lines(&collapsed, CAPTION_MAX_LINES, CAPTION_MAX_CHARS_PER_LINE)
}

fn truncate_caption_lines(text: &str, max_lines: usize, max_chars_per_line: usize) -> String {
    if text.is_empty() || max_lines == 0 || max_chars_per_line == 0 {
        return String::new();
    }

    let mut lines = Vec::<String>::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let next_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if next_len > max_chars_per_line && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.len() > max_lines {
        lines = lines[lines.len() - max_lines..].to_vec();
    }

    lines.join("\n")
}

fn default_caption_max_lines() -> usize {
    CAPTION_MAX_LINES
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn resample_to_16k_mono(frame: &PcmAudioFrame) -> Vec<f32> {
    let mono = downmix_to_mono(&frame.samples, frame.channels);
    resample_linear(&mono, frame.sample_rate, TARGET_SAMPLE_RATE)
}

fn write_mono_wav(
    path: &std::path::Path,
    samples: &[f32],
    sample_rate: u32,
) -> Result<(), AppError> {
    let data_len = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data_len as usize);

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());

    for sample in samples {
        let sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    fs::write(path, bytes)?;
    Ok(())
}

fn synthesize_diagnostic_wav(path: &std::path::Path) -> Result<(), AppError> {
    let script = "& {
        param($Path)
        Add-Type -AssemblyName System.Speech
        $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
        $synth.SetOutputToWaveFile($Path)
        $synth.Speak('hello world this is a local caption test')
        $synth.Dispose()
    }";

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .arg(path)
        .status()?;

    if !status.success() || !path.exists() {
        return Err(AppError::Asr(
            "could not generate local ASR diagnostic audio".to_string(),
        ));
    }

    Ok(())
}

fn play_diagnostic_speech_to_default_output() -> Result<Child, AppError> {
    let script = "& {
        Start-Sleep -Milliseconds 700
        Add-Type -AssemblyName System.Speech
        $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
        $synth.Speak('hello world this is a local caption test')
        $synth.Dispose()
    }";

    Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .spawn()
        .map_err(AppError::from)
}

struct DiagnosticAudio {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
}

fn read_pcm16_wav(path: &std::path::Path) -> Result<DiagnosticAudio, AppError> {
    let bytes = fs::read(path)?;
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(AppError::Asr(
            "diagnostic audio is not a WAV file".to_string(),
        ));
    }

    let mut offset = 12usize;
    let mut channels = None;
    let mut sample_rate = None;
    let mut data = None;

    while offset + 8 <= bytes.len() {
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        let chunk_start = offset + 8;
        let chunk_end = chunk_start.saturating_add(chunk_size).min(bytes.len());

        if chunk_id == b"fmt " {
            if chunk_size < 16 || chunk_end > bytes.len() {
                return Err(AppError::Asr(
                    "diagnostic WAV fmt chunk is invalid".to_string(),
                ));
            }

            let format = u16::from_le_bytes([bytes[chunk_start], bytes[chunk_start + 1]]);
            let parsed_channels =
                u16::from_le_bytes([bytes[chunk_start + 2], bytes[chunk_start + 3]]);
            let parsed_sample_rate = u32::from_le_bytes([
                bytes[chunk_start + 4],
                bytes[chunk_start + 5],
                bytes[chunk_start + 6],
                bytes[chunk_start + 7],
            ]);
            let bits_per_sample =
                u16::from_le_bytes([bytes[chunk_start + 14], bytes[chunk_start + 15]]);

            if format != 1 || bits_per_sample != 16 || parsed_channels == 0 {
                return Err(AppError::Asr(
                    "diagnostic WAV must be 16-bit PCM".to_string(),
                ));
            }

            channels = Some(parsed_channels);
            sample_rate = Some(parsed_sample_rate);
        } else if chunk_id == b"data" {
            data = Some(bytes[chunk_start..chunk_end].to_vec());
        }

        offset = chunk_end + (chunk_size % 2);
    }

    let channels =
        channels.ok_or_else(|| AppError::Asr("diagnostic WAV has no fmt chunk".to_string()))?;
    let sample_rate = sample_rate
        .ok_or_else(|| AppError::Asr("diagnostic WAV has no sample rate".to_string()))?;
    let data = data.ok_or_else(|| AppError::Asr("diagnostic WAV has no audio data".to_string()))?;
    let samples = data
        .chunks_exact(2)
        .map(|sample| i16::from_le_bytes([sample[0], sample[1]]) as f32 / i16::MAX as f32)
        .collect();

    Ok(DiagnosticAudio {
        samples,
        sample_rate,
        channels,
    })
}

fn downmix_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    let channels = channels.max(1) as usize;
    if channels == 1 {
        return samples.to_vec();
    }

    samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect()
}

fn resample_linear(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if samples.is_empty() || source_rate == 0 {
        return Vec::new();
    }

    if source_rate == target_rate {
        return samples.to_vec();
    }

    let ratio = source_rate as f32 / target_rate as f32;
    let output_len = (samples.len() as f32 / ratio).ceil() as usize;
    (0..output_len)
        .map(|index| {
            let source_position = index as f32 * ratio;
            let left = source_position.floor() as usize;
            let right = (left + 1).min(samples.len() - 1);
            let fraction = source_position - left as f32;
            samples[left] + (samples[right] - samples[left]) * fraction
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downmixes_stereo_to_mono() {
        let samples = downmix_to_mono(&[0.2, 0.4, -0.2, 0.2], 2);

        assert_eq!(samples, vec![0.3, 0.0]);
    }

    #[test]
    fn resamples_48k_to_16k() {
        let samples = vec![0.0; 48_000];
        let output = resample_linear(&samples, 48_000, 16_000);

        assert_eq!(output.len(), 16_000);
    }

    #[test]
    fn stabilizes_caption_text_to_two_wrapped_lines() {
        let text = stabilize_caption_text(
            "one two three four five six seven eight nine ten eleven twelve thirteen fourteen",
        );

        assert!(text.lines().count() <= CAPTION_MAX_LINES);
        assert!(text
            .lines()
            .all(|line| line.len() <= CAPTION_MAX_CHARS_PER_LINE));
    }

    #[test]
    fn caption_state_keeps_provisional_and_display_text() {
        let dir =
            std::env::temp_dir().join(format!("feelsay-caption-state-test-{}", std::process::id()));
        let path = dir.join("caption.json");

        write_caption_state(&path, "", " hello   world ").expect("write caption state");
        let content = std::fs::read_to_string(&path).expect("read caption state");
        let state =
            serde_json::from_str::<CaptionRuntimeState>(&content).expect("parse caption state");

        assert_eq!(state.text, "hello world");
        assert_eq!(state.provisional_text, "hello world");
        assert!(state.is_provisional);
        assert_eq!(state.max_lines, CAPTION_MAX_LINES);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn caption_state_keeps_source_label() {
        let dir = std::env::temp_dir().join(format!(
            "feelsay-caption-source-label-test-{}",
            std::process::id()
        ));
        let path = dir.join("caption.json");

        write_labeled_caption_state(&path, "", "hello", Some(" Default   Microphone "), None)
            .expect("write caption state");
        let content = std::fs::read_to_string(&path).expect("read caption state");
        let state =
            serde_json::from_str::<CaptionRuntimeState>(&content).expect("parse caption state");

        assert_eq!(state.source_label.as_deref(), Some("Default Microphone"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn caption_state_keeps_speaker_label_and_confidence() {
        let dir = std::env::temp_dir().join(format!(
            "feelsay-caption-speaker-label-test-{}",
            std::process::id()
        ));
        let path = dir.join("caption.json");
        let speaker_label = SpeakerLabel {
            label: "Speaker 1".to_string(),
            confidence: SpeakerLabelConfidence::Low,
        };

        write_labeled_caption_state(&path, "", "hello", None, Some(&speaker_label))
            .expect("write caption state");
        let content = std::fs::read_to_string(&path).expect("read caption state");
        let state =
            serde_json::from_str::<CaptionRuntimeState>(&content).expect("parse caption state");

        assert_eq!(state.speaker_label.as_deref(), Some("Speaker 1"));
        assert_eq!(state.speaker_confidence, Some(SpeakerLabelConfidence::Low));

        let _ = std::fs::remove_dir_all(dir);
    }
}
