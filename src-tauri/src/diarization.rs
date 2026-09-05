use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerLabelConfidence {
    Low,
    Medium,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeakerLabel {
    pub label: String,
    pub confidence: SpeakerLabelConfidence,
}

#[derive(Debug, Clone)]
pub struct ExperimentalSpeakerLabeler {
    enabled: bool,
    speakers: Vec<SpeakerFingerprint>,
}

impl ExperimentalSpeakerLabeler {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            speakers: Vec::new(),
        }
    }

    pub fn label_for_samples(&mut self, samples: &[f32]) -> Option<SpeakerLabel> {
        if !self.enabled {
            return None;
        }

        let fingerprint = SpeakerFingerprint::from_samples(samples)?;
        let (speaker_index, confidence) = self.match_or_add_speaker(fingerprint);

        Some(SpeakerLabel {
            label: format!("Speaker {}", speaker_index + 1),
            confidence,
        })
    }

    fn match_or_add_speaker(
        &mut self,
        fingerprint: SpeakerFingerprint,
    ) -> (usize, SpeakerLabelConfidence) {
        let Some((index, distance)) = self.closest_speaker(&fingerprint) else {
            self.speakers.push(fingerprint);
            return (0, SpeakerLabelConfidence::Low);
        };

        if distance > 0.28 && self.speakers.len() < 4 {
            self.speakers.push(fingerprint);
            return (self.speakers.len() - 1, SpeakerLabelConfidence::Low);
        }

        self.speakers[index].update(fingerprint);
        let confidence = if self.speakers[index].observations >= 3 && distance < 0.12 {
            SpeakerLabelConfidence::Medium
        } else {
            SpeakerLabelConfidence::Low
        };

        (index, confidence)
    }

    fn closest_speaker(&self, fingerprint: &SpeakerFingerprint) -> Option<(usize, f32)> {
        self.speakers
            .iter()
            .enumerate()
            .map(|(index, speaker)| (index, speaker.distance(fingerprint)))
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpeakerFingerprint {
    rms: f32,
    zero_crossing_rate: f32,
    observations: u32,
}

impl SpeakerFingerprint {
    fn from_samples(samples: &[f32]) -> Option<Self> {
        if samples.len() < 160 {
            return None;
        }

        let rms = (samples
            .iter()
            .map(|sample| sample.clamp(-1.0, 1.0).powi(2))
            .sum::<f32>()
            / samples.len() as f32)
            .sqrt()
            .clamp(0.0, 1.0);

        if rms < 0.01 {
            return None;
        }

        let zero_crossings = samples
            .windows(2)
            .filter(|pair| pair[0].is_sign_positive() != pair[1].is_sign_positive())
            .count();
        let zero_crossing_rate = (zero_crossings as f32 / samples.len() as f32).clamp(0.0, 1.0);

        Some(Self {
            rms,
            zero_crossing_rate,
            observations: 1,
        })
    }

    fn distance(&self, other: &Self) -> f32 {
        ((self.rms - other.rms).abs() * 0.7)
            + ((self.zero_crossing_rate - other.zero_crossing_rate).abs() * 0.3)
    }

    fn update(&mut self, other: Self) {
        let weight = self.observations as f32;
        self.rms = ((self.rms * weight) + other.rms) / (weight + 1.0);
        self.zero_crossing_rate =
            ((self.zero_crossing_rate * weight) + other.zero_crossing_rate) / (weight + 1.0);
        self.observations = self.observations.saturating_add(1);
    }
}

pub fn speaker_confidence_label(confidence: SpeakerLabelConfidence) -> &'static str {
    match confidence {
        SpeakerLabelConfidence::Low => "low confidence",
        SpeakerLabelConfidence::Medium => "medium confidence",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speaker_labeler_is_opt_in() {
        let mut labeler = ExperimentalSpeakerLabeler::new(false);

        assert_eq!(labeler.label_for_samples(&vec![0.2; 320]), None);
    }

    #[test]
    fn speaker_labeler_generates_uncertain_label_when_enabled() {
        let mut labeler = ExperimentalSpeakerLabeler::new(true);

        let label = labeler
            .label_for_samples(&vec![0.2; 320])
            .expect("speaker label");

        assert_eq!(label.label, "Speaker 1");
        assert_eq!(label.confidence, SpeakerLabelConfidence::Low);
    }

    #[test]
    fn speaker_labeler_confidence_improves_after_repeated_observations() {
        let mut labeler = ExperimentalSpeakerLabeler::new(true);
        let samples = vec![0.2; 320];

        let _ = labeler.label_for_samples(&samples);
        let _ = labeler.label_for_samples(&samples);
        let label = labeler.label_for_samples(&samples).expect("speaker label");

        assert_eq!(label.confidence, SpeakerLabelConfidence::Medium);
    }
}
