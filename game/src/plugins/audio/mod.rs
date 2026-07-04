use feedback_events::{AccessibilityFeedbackFlags, FeedbackEvent, FeedbackKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioRegistrationSummary {
    pub cues: usize,
    pub high_frequency_cues: usize,
}

pub fn register() -> AudioRegistrationSummary {
    let state = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags::DEFAULT);

    AudioRegistrationSummary {
        cues: state.emitted.len(),
        high_frequency_cues: state
            .emitted
            .iter()
            .filter(|cue| cue.high_frequency)
            .count(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockAudioCue {
    pub kind: FeedbackKind,
    pub bus: AudioBus,
    pub layer: AudioLayer,
    pub intensity: f32,
    pub high_frequency: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioBus {
    CollisionPlinks,
    ComboPercussion,
    RewardStingers,
    AmbientBoardHum,
    UiConfirmations,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioLayer {
    PitchedPlink,
    BrightPlink,
    Thump,
    Shimmer,
    BassAccent,
    RisingArpeggio,
    CatchChord,
    SoftDownbeat,
    ComboPercussionHit,
    TensionRiser,
    FeverSting,
    SilenceGate,
    LossDownbeat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockAudioPlaybackState {
    pub accessibility: AccessibilityFeedbackFlags,
    pub emitted: Vec<MockAudioCue>,
}

impl MockAudioPlaybackState {
    pub fn new(accessibility: AccessibilityFeedbackFlags) -> Self {
        Self {
            accessibility,
            emitted: Vec::new(),
        }
    }

    pub fn play_event(&mut self, event: &FeedbackEvent) {
        let intensity = ethical_intensity(event.kind, event.intensity);

        match event.kind {
            FeedbackKind::PegHit => self.emit(
                event.kind,
                AudioBus::CollisionPlinks,
                AudioLayer::PitchedPlink,
                intensity,
                true,
            ),
            FeedbackKind::OrangeHit => {
                self.emit(
                    event.kind,
                    AudioBus::CollisionPlinks,
                    AudioLayer::BrightPlink,
                    intensity,
                    true,
                );
                self.emit(
                    event.kind,
                    AudioBus::RewardStingers,
                    AudioLayer::Thump,
                    intensity,
                    false,
                );
            }
            FeedbackKind::PurpleHit => {
                self.emit(
                    event.kind,
                    AudioBus::RewardStingers,
                    AudioLayer::Shimmer,
                    intensity,
                    true,
                );
                self.emit(
                    event.kind,
                    AudioBus::RewardStingers,
                    AudioLayer::BassAccent,
                    intensity,
                    false,
                );
            }
            FeedbackKind::GreenHit => self.emit(
                event.kind,
                AudioBus::UiConfirmations,
                AudioLayer::RisingArpeggio,
                intensity,
                true,
            ),
            FeedbackKind::BucketCatch => self.emit(
                event.kind,
                AudioBus::RewardStingers,
                AudioLayer::CatchChord,
                intensity,
                false,
            ),
            FeedbackKind::NearBucketMiss => self.emit(
                event.kind,
                AudioBus::UiConfirmations,
                AudioLayer::SoftDownbeat,
                intensity,
                false,
            ),
            FeedbackKind::ComboThreshold => self.emit(
                event.kind,
                AudioBus::ComboPercussion,
                AudioLayer::ComboPercussionHit,
                intensity,
                false,
            ),
            FeedbackKind::FinalOrangeTension => self.emit(
                event.kind,
                AudioBus::AmbientBoardHum,
                AudioLayer::TensionRiser,
                intensity,
                true,
            ),
            FeedbackKind::ExtremeFever => {
                self.emit(
                    event.kind,
                    AudioBus::AmbientBoardHum,
                    AudioLayer::SilenceGate,
                    intensity,
                    false,
                );
                self.emit(
                    event.kind,
                    AudioBus::RewardStingers,
                    AudioLayer::FeverSting,
                    intensity,
                    false,
                );
            }
            FeedbackKind::Loss => self.emit(
                event.kind,
                AudioBus::UiConfirmations,
                AudioLayer::LossDownbeat,
                intensity,
                false,
            ),
        }
    }

    fn emit(
        &mut self,
        kind: FeedbackKind,
        bus: AudioBus,
        layer: AudioLayer,
        intensity: f32,
        high_frequency: bool,
    ) {
        if self.accessibility.mute_high_frequency_layers && high_frequency {
            return;
        }

        self.emitted.push(MockAudioCue {
            kind,
            bus,
            layer,
            intensity,
            high_frequency,
        });
    }
}

pub fn play_mock_checkpoint1_scene(
    accessibility: AccessibilityFeedbackFlags,
) -> MockAudioPlaybackState {
    let mut state = MockAudioPlaybackState::new(accessibility);

    for event in crate::plugins::vfx::mock_checkpoint1_feedback_sequence() {
        state.play_event(&event);
    }

    state
}

fn ethical_intensity(kind: FeedbackKind, requested: f32) -> f32 {
    let cap = match kind {
        FeedbackKind::Loss => 0.2,
        FeedbackKind::NearBucketMiss => 0.25,
        FeedbackKind::PegHit => 0.45,
        FeedbackKind::FinalOrangeTension => 0.65,
        FeedbackKind::OrangeHit
        | FeedbackKind::PurpleHit
        | FeedbackKind::GreenHit
        | FeedbackKind::BucketCatch
        | FeedbackKind::ComboThreshold => 0.85,
        FeedbackKind::ExtremeFever => 1.0,
    };

    requested.clamp(0.0, cap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::Vec2;

    fn event(kind: FeedbackKind, intensity: f32) -> FeedbackEvent {
        FeedbackEvent {
            kind,
            intensity,
            position: Vec2::ZERO,
            combo: 0,
            value: 0,
        }
    }

    #[test]
    fn loss_and_near_miss_audio_are_not_victory_like() {
        let mut state = MockAudioPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);

        state.play_event(&event(FeedbackKind::NearBucketMiss, 1.0));
        state.play_event(&event(FeedbackKind::Loss, 1.0));

        assert!(state.emitted.iter().all(|cue| cue.intensity <= 0.25));
        assert!(state.emitted.iter().all(|cue| {
            !matches!(
                cue.layer,
                AudioLayer::CatchChord | AudioLayer::FeverSting | AudioLayer::RisingArpeggio
            )
        }));
    }

    #[test]
    fn accessibility_flags_mute_high_frequency_audio_layers() {
        let normal = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags::DEFAULT);
        let muted = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags {
            reduce_shake: false,
            reduce_flash: false,
            mute_high_frequency_layers: true,
        });

        assert!(muted.emitted.len() < normal.emitted.len());
        assert!(!muted.emitted.iter().any(|cue| cue.high_frequency));
    }
}
