# Diarization Evaluation

## Goal

Add speaker-turn labels without blocking real-time captions or making the base app heavy.

## Current P3-021 Implementation

FeelSay currently ships an opt-in experimental speaker-label scaffold:

- runs locally inside the ASR worker path
- adds no new runtime dependencies
- labels transcript/caption segments as `Speaker 1`, `Speaker 2`, etc.
- exposes uncertainty as `low confidence` or `medium confidence`
- never blocks normal caption capture when disabled

This is not production-grade diarization. It is a lightweight placeholder that keeps the
data model, settings, overlay, and transcript exports ready for a real local diarization
engine later.

## Local Options Reviewed

### pyannote.audio

Best current accuracy direction for real diarization. It is a Python/PyTorch toolkit with
pretrained diarization pipelines.

Tradeoffs:

- strong diarization quality
- adds Python/PyTorch model/runtime complexity
- may require gated model access depending on selected model
- likely too heavy for the default desktop utility path

Reference: https://github.com/pyannote/pyannote-audio

### WhisperX

Practical batch/offline pipeline that combines Whisper transcription, alignment, and
speaker diarization.

Tradeoffs:

- useful for transcript post-processing
- Python/GPU-oriented stack
- not ideal for low-latency always-on overlay captions

Reference: https://github.com/m-bain/whisperx

### WeSpeaker

Speaker embedding toolkit that can support speaker verification and diarization recipes.

Tradeoffs:

- useful direction for an embedding-based local pipeline
- more engineering required to integrate with the Rust desktop runtime
- likely better as a separate optional worker/model package

Reference: https://github.com/wenet-e2e/wespeaker

## Recommended Direction

Keep P3-021 as an experimental opt-in scaffold. For production diarization, add a later
PBI for an optional high-resource local worker:

- capture speech chunks with timestamps
- run diarization out-of-band
- update transcript speaker labels asynchronously
- keep the live overlay responsive even if diarization lags
- mark uncertain labels clearly

The default app should remain fast and local-first without requiring Python, GPU drivers,
or large speaker models.
