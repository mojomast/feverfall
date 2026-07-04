use feedback_events::{AccessibilityFeedbackFlags, FeedbackEvent, FeedbackKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioRegistrationSummary {
    pub cues: usize,
    pub high_frequency_cues: usize,
}

pub fn register() -> AudioRegistrationSummary {
    let mut state = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags::DEFAULT);
    state.play_board_hum(0.2);
    state.play_ui_confirmation();

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
    pub pitch_semitones: f32,
    pub chord_cluster: bool,
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
    SoftAirCannon,      // TODO(audio): awaiting asset
    PitchedPlink,       // TODO(audio): awaiting asset
    BrightPlink,        // TODO(audio): awaiting asset
    Thump,              // TODO(audio): awaiting asset
    Shimmer,            // TODO(audio): awaiting asset
    BassAccent,         // TODO(audio): awaiting asset
    RisingArpeggio,     // TODO(audio): awaiting asset
    CatchChord,         // TODO(audio): awaiting asset
    RimDing,            // TODO(audio): awaiting asset
    WhooshBell,         // TODO(audio): awaiting asset
    SoftDownbeat,       // TODO(audio): awaiting asset
    ComboPercussionHit, // TODO(audio): awaiting asset
    TensionRiser,       // TODO(audio): awaiting asset
    FeverSting,         // TODO(audio): awaiting asset
    SilenceGate,
    LossDownbeat,   // TODO(audio): awaiting asset
    BoardHum,       // TODO(audio): awaiting asset
    UiConfirmation, // TODO(audio): awaiting asset
    ChordCluster,   // TODO(audio): awaiting asset
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockAudioPlaybackState {
    pub accessibility: AccessibilityFeedbackFlags,
    pub emitted: Vec<MockAudioCue>,
    peg_hit_samples: u32,
}

impl MockAudioPlaybackState {
    pub fn new(accessibility: AccessibilityFeedbackFlags) -> Self {
        Self {
            accessibility,
            emitted: Vec::new(),
            peg_hit_samples: 0,
        }
    }

    pub fn play_event(&mut self, event: &FeedbackEvent) {
        let intensity = ethical_intensity(event.kind, event.intensity);

        match event.kind {
            FeedbackKind::BallLaunch => self.emit(
                event.kind,
                AudioBus::UiConfirmations,
                AudioLayer::SoftAirCannon,
                intensity,
                false,
                0.0,
                false,
            ),
            FeedbackKind::PegHit => self.emit_peg_hit(
                event.kind,
                AudioLayer::PitchedPlink,
                intensity,
                true,
                peg_pitch(event.combo),
            ),
            FeedbackKind::OrangeHit => {
                self.emit_peg_hit(
                    event.kind,
                    AudioLayer::BrightPlink,
                    intensity,
                    true,
                    peg_pitch(event.combo),
                );
                self.emit(
                    event.kind,
                    AudioBus::RewardStingers,
                    AudioLayer::Thump,
                    intensity,
                    false,
                    0.0,
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
                    0.0,
                    false,
                );
                self.emit(
                    event.kind,
                    AudioBus::RewardStingers,
                    AudioLayer::BassAccent,
                    intensity,
                    false,
                    0.0,
                    false,
                );
            }
            FeedbackKind::GreenHit => self.emit(
                event.kind,
                AudioBus::UiConfirmations,
                AudioLayer::RisingArpeggio,
                intensity,
                true,
                0.0,
                false,
            ),
            FeedbackKind::RelicTriggered => self.emit(
                event.kind,
                AudioBus::RewardStingers,
                AudioLayer::RisingArpeggio,
                intensity,
                false,
                0.0,
                false,
            ),
            FeedbackKind::BucketCatch => self.emit(
                event.kind,
                AudioBus::RewardStingers,
                AudioLayer::CatchChord,
                intensity,
                false,
                0.0,
                false,
            ),
            FeedbackKind::LongShot => self.emit(
                event.kind,
                AudioBus::RewardStingers,
                AudioLayer::WhooshBell,
                intensity,
                true,
                0.0,
                false,
            ),
            FeedbackKind::LuckyBounce => self.emit(
                event.kind,
                AudioBus::RewardStingers,
                AudioLayer::RimDing,
                intensity,
                true,
                0.0,
                false,
            ),
            FeedbackKind::NearBucketMiss => self.emit(
                event.kind,
                AudioBus::UiConfirmations,
                AudioLayer::SoftDownbeat,
                intensity,
                false,
                0.0,
                false,
            ),
            FeedbackKind::ComboThreshold => self.emit(
                event.kind,
                AudioBus::ComboPercussion,
                AudioLayer::ComboPercussionHit,
                intensity,
                false,
                combo_pitch(event.combo),
                false,
            ),
            FeedbackKind::FinalOrangeTension => self.emit(
                event.kind,
                AudioBus::AmbientBoardHum,
                AudioLayer::TensionRiser,
                intensity,
                true,
                0.0,
                false,
            ),
            FeedbackKind::ExtremeFever => {
                self.emit(
                    event.kind,
                    AudioBus::AmbientBoardHum,
                    AudioLayer::SilenceGate,
                    intensity,
                    false,
                    0.0,
                    false,
                );
                self.emit(
                    event.kind,
                    AudioBus::RewardStingers,
                    AudioLayer::FeverSting,
                    intensity,
                    false,
                    0.0,
                    false,
                );
            }
            FeedbackKind::Loss => self.emit(
                event.kind,
                AudioBus::UiConfirmations,
                AudioLayer::LossDownbeat,
                intensity,
                false,
                0.0,
                false,
            ),
        }
    }

    pub fn play_board_hum(&mut self, intensity: f32) {
        self.emit(
            FeedbackKind::BallLaunch,
            AudioBus::AmbientBoardHum,
            AudioLayer::BoardHum,
            intensity.clamp(0.0, 0.35),
            false,
            0.0,
            false,
        );
    }

    pub fn play_ui_confirmation(&mut self) {
        self.emit(
            FeedbackKind::BallLaunch,
            AudioBus::UiConfirmations,
            AudioLayer::UiConfirmation,
            0.35,
            false,
            0.0,
            false,
        );
    }

    fn emit_peg_hit(
        &mut self,
        kind: FeedbackKind,
        layer: AudioLayer,
        intensity: f32,
        high_frequency: bool,
        pitch_semitones: f32,
    ) {
        self.peg_hit_samples += 1;
        if self.peg_hit_samples > 12 {
            self.emit(
                kind,
                AudioBus::CollisionPlinks,
                AudioLayer::ChordCluster,
                intensity,
                false,
                peg_pitch(15),
                true,
            );
            return;
        }

        self.emit(
            kind,
            AudioBus::CollisionPlinks,
            layer,
            intensity,
            high_frequency,
            pitch_semitones,
            false,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn emit(
        &mut self,
        kind: FeedbackKind,
        bus: AudioBus,
        layer: AudioLayer,
        intensity: f32,
        high_frequency: bool,
        pitch_semitones: f32,
        chord_cluster: bool,
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
            pitch_semitones,
            chord_cluster,
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
        FeedbackKind::PegHit | FeedbackKind::BallLaunch => 0.45,
        FeedbackKind::FinalOrangeTension => 0.65,
        FeedbackKind::OrangeHit
        | FeedbackKind::PurpleHit
        | FeedbackKind::GreenHit
        | FeedbackKind::RelicTriggered
        | FeedbackKind::BucketCatch
        | FeedbackKind::ComboThreshold
        | FeedbackKind::LongShot
        | FeedbackKind::LuckyBounce => 0.85,
        FeedbackKind::ExtremeFever => 1.0,
    };

    requested.clamp(0.0, cap)
}

fn peg_pitch(combo: u32) -> f32 {
    combo.clamp(1, 15) as f32 * 0.35
}

fn combo_pitch(combo: u32) -> f32 {
    match combo {
        0..=3 => 0.0,
        4..=6 => 2.0,
        7..=10 => 5.0,
        _ => 7.0,
    }
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

    #[test]
    fn peg_pitch_caps_and_overloaded_hits_cluster() {
        let mut state = MockAudioPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);

        for combo in 1..=16 {
            let mut hit = event(FeedbackKind::PegHit, 0.35);
            hit.combo = combo;
            state.play_event(&hit);
        }

        let plinks = state
            .emitted
            .iter()
            .filter(|cue| cue.layer == AudioLayer::PitchedPlink)
            .count();
        assert_eq!(plinks, 12);
        assert!(state.emitted.iter().any(|cue| cue.chord_cluster));
        assert!(state
            .emitted
            .iter()
            .all(|cue| cue.pitch_semitones <= peg_pitch(15)));
    }
}
