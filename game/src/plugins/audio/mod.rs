use std::time::Duration;

use feedback_events::{AccessibilityFeedbackFlags, FeedbackEvent, FeedbackKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioRegistrationSummary {
    pub cues: usize,
    pub high_frequency_cues: usize,
}

pub fn register() -> AudioRegistrationSummary {
    debug_assert_eq!(RUNTIME_BUSES.len(), AudioBus::ALL.len());
    debug_assert_eq!(AUDIO_ASSET_MANIFEST.len(), AudioLayer::ALL_AUDIBLE.len());

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
pub struct AudioCue {
    pub kind: FeedbackKind,
    pub bus: AudioBus,
    pub layer: AudioLayer,
    pub asset_path: Option<&'static str>,
    pub intensity: f32,
    pub high_frequency: bool,
    pub pitch_semitones: f32,
    pub chord_cluster: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioBus {
    CollisionPlinks,
    ComboPercussion,
    RewardStingers,
    AmbientBoardHum,
    UiConfirmations,
}

impl AudioBus {
    pub const ALL: [Self; 5] = [
        Self::CollisionPlinks,
        Self::ComboPercussion,
        Self::RewardStingers,
        Self::AmbientBoardHum,
        Self::UiConfirmations,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioLayer {
    SoftAirCannon,
    PitchedPlink,
    BrightPlink,
    Thump,
    Shimmer,
    BassAccent,
    RisingArpeggio,
    CatchChord,
    RimDing,
    WhooshBell,
    SoftDownbeat,
    ComboPercussionHit,
    TensionRiser,
    FeverSting,
    SilenceGate,
    LossDownbeat,
    BoardHum,
    UiConfirmation,
    ChordCluster,
}

impl AudioLayer {
    pub const ALL_AUDIBLE: [Self; 18] = [
        Self::SoftAirCannon,
        Self::PitchedPlink,
        Self::BrightPlink,
        Self::Thump,
        Self::Shimmer,
        Self::BassAccent,
        Self::RisingArpeggio,
        Self::CatchChord,
        Self::RimDing,
        Self::WhooshBell,
        Self::SoftDownbeat,
        Self::ComboPercussionHit,
        Self::TensionRiser,
        Self::FeverSting,
        Self::LossDownbeat,
        Self::BoardHum,
        Self::UiConfirmation,
        Self::ChordCluster,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeAudioBus {
    pub bus: AudioBus,
    pub channel_name: &'static str,
    pub volume: f32,
    pub min_interval: Duration,
}

pub const RUNTIME_BUSES: [RuntimeAudioBus; 5] = [
    RuntimeAudioBus {
        bus: AudioBus::CollisionPlinks,
        channel_name: "collision_plinks",
        volume: 0.42,
        min_interval: Duration::from_millis(18),
    },
    RuntimeAudioBus {
        bus: AudioBus::ComboPercussion,
        channel_name: "combo_percussion",
        volume: 0.58,
        min_interval: Duration::from_millis(45),
    },
    RuntimeAudioBus {
        bus: AudioBus::RewardStingers,
        channel_name: "reward_stingers",
        volume: 0.70,
        min_interval: Duration::from_millis(120),
    },
    RuntimeAudioBus {
        bus: AudioBus::AmbientBoardHum,
        channel_name: "ambient_board_hum",
        volume: 0.28,
        min_interval: Duration::from_millis(250),
    },
    RuntimeAudioBus {
        bus: AudioBus::UiConfirmations,
        channel_name: "ui_confirmations",
        volume: 0.46,
        min_interval: Duration::from_millis(70),
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioAssetManifestEntry {
    pub layer: AudioLayer,
    pub path: &'static str,
}

pub const AUDIO_ASSET_MANIFEST: [AudioAssetManifestEntry; 18] = [
    asset(AudioLayer::SoftAirCannon, "audio/soft_air_cannon.wav"),
    asset(AudioLayer::PitchedPlink, "audio/pitched_plink.wav"),
    asset(AudioLayer::BrightPlink, "audio/bright_plink.wav"),
    asset(AudioLayer::Thump, "audio/thump.wav"),
    asset(AudioLayer::Shimmer, "audio/shimmer.wav"),
    asset(AudioLayer::BassAccent, "audio/bass_accent.wav"),
    asset(AudioLayer::RisingArpeggio, "audio/rising_arpeggio.wav"),
    asset(AudioLayer::CatchChord, "audio/catch_chord.wav"),
    asset(AudioLayer::RimDing, "audio/rim_ding.wav"),
    asset(AudioLayer::WhooshBell, "audio/whoosh_bell.wav"),
    asset(AudioLayer::SoftDownbeat, "audio/soft_downbeat.wav"),
    asset(
        AudioLayer::ComboPercussionHit,
        "audio/combo_percussion_hit.wav",
    ),
    asset(AudioLayer::TensionRiser, "audio/tension_riser.wav"),
    asset(AudioLayer::FeverSting, "audio/fever_sting.wav"),
    asset(AudioLayer::LossDownbeat, "audio/loss_downbeat.wav"),
    asset(AudioLayer::BoardHum, "audio/board_hum.wav"),
    asset(AudioLayer::UiConfirmation, "audio/ui_confirmation.wav"),
    asset(AudioLayer::ChordCluster, "audio/chord_cluster.wav"),
];

const fn asset(layer: AudioLayer, path: &'static str) -> AudioAssetManifestEntry {
    AudioAssetManifestEntry { layer, path }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioPlaybackState {
    pub accessibility: AccessibilityFeedbackFlags,
    pub emitted: Vec<AudioCue>,
    peg_hit_samples: u32,
}

/// Compatibility alias for feedback-layer contracts that still refer to the C1 mock adapter.
pub type MockAudioPlaybackState = AudioPlaybackState;

impl AudioPlaybackState {
    pub fn new(accessibility: AccessibilityFeedbackFlags) -> Self {
        Self {
            accessibility,
            emitted: Vec::new(),
            peg_hit_samples: 0,
        }
    }

    /// Converts nondeterministic presentation feedback into runtime audio commands only.
    /// No gameplay state is mutated here; callers can drop the emitted cues in headless runs.
    pub fn play_event(&mut self, event: &FeedbackEvent) {
        let intensity = ethical_intensity(event.kind, event.intensity);

        for cue in feedback_audio_mapping(event.kind, intensity, event.combo) {
            if cue.layer == AudioLayer::PitchedPlink || cue.layer == AudioLayer::BrightPlink {
                self.emit_peg_hit(cue);
            } else {
                self.emit(cue);
            }
        }
    }

    pub fn play_board_hum(&mut self, intensity: f32) {
        self.emit(cue(
            FeedbackKind::BallLaunch,
            AudioBus::AmbientBoardHum,
            AudioLayer::BoardHum,
            intensity.clamp(0.0, 0.35),
            false,
            0.0,
            false,
        ));
    }

    pub fn play_ui_confirmation(&mut self) {
        self.emit(cue(
            FeedbackKind::BallLaunch,
            AudioBus::UiConfirmations,
            AudioLayer::UiConfirmation,
            0.35,
            false,
            0.0,
            false,
        ));
    }

    fn emit_peg_hit(&mut self, cue: AudioCue) {
        self.peg_hit_samples += 1;
        if self.peg_hit_samples > 12 {
            self.emit(cue_with_pitch(
                cue.kind,
                AudioBus::CollisionPlinks,
                AudioLayer::ChordCluster,
                cue.intensity,
                false,
                peg_pitch(15),
                true,
            ));
            return;
        }

        self.emit(cue);
    }

    fn emit(&mut self, cue: AudioCue) {
        if self.accessibility.mute_high_frequency_layers && cue.high_frequency {
            return;
        }

        self.emitted.push(cue);
    }
}

pub fn play_mock_checkpoint1_scene(
    accessibility: AccessibilityFeedbackFlags,
) -> AudioPlaybackState {
    let mut state = AudioPlaybackState::new(accessibility);

    for event in crate::plugins::vfx::mock_checkpoint1_feedback_sequence() {
        state.play_event(&event);
    }

    state
}

pub fn feedback_audio_mapping(kind: FeedbackKind, intensity: f32, combo: u32) -> Vec<AudioCue> {
    let intensity = ethical_intensity(kind, intensity);
    match kind {
        FeedbackKind::BallLaunch => vec![cue(
            kind,
            AudioBus::UiConfirmations,
            AudioLayer::SoftAirCannon,
            intensity,
            false,
            0.0,
            false,
        )],
        FeedbackKind::PegHit => vec![cue(
            kind,
            AudioBus::CollisionPlinks,
            AudioLayer::PitchedPlink,
            intensity,
            true,
            peg_pitch(combo),
            false,
        )],
        FeedbackKind::OrangeHit => vec![
            cue(
                kind,
                AudioBus::CollisionPlinks,
                AudioLayer::BrightPlink,
                intensity,
                true,
                peg_pitch(combo),
                false,
            ),
            cue(
                kind,
                AudioBus::RewardStingers,
                AudioLayer::Thump,
                intensity,
                false,
                0.0,
                false,
            ),
        ],
        FeedbackKind::PurpleHit => vec![
            cue(
                kind,
                AudioBus::RewardStingers,
                AudioLayer::Shimmer,
                intensity,
                true,
                0.0,
                false,
            ),
            cue(
                kind,
                AudioBus::RewardStingers,
                AudioLayer::BassAccent,
                intensity,
                false,
                0.0,
                false,
            ),
        ],
        FeedbackKind::GreenHit => vec![cue(
            kind,
            AudioBus::UiConfirmations,
            AudioLayer::RisingArpeggio,
            intensity,
            true,
            0.0,
            false,
        )],
        FeedbackKind::RelicTriggered => vec![cue(
            kind,
            AudioBus::RewardStingers,
            AudioLayer::RisingArpeggio,
            intensity,
            false,
            0.0,
            false,
        )],
        FeedbackKind::BucketCatch => vec![cue(
            kind,
            AudioBus::RewardStingers,
            AudioLayer::CatchChord,
            intensity,
            false,
            0.0,
            false,
        )],
        FeedbackKind::LongShot => vec![cue(
            kind,
            AudioBus::RewardStingers,
            AudioLayer::WhooshBell,
            intensity,
            true,
            0.0,
            false,
        )],
        FeedbackKind::LuckyBounce => vec![cue(
            kind,
            AudioBus::RewardStingers,
            AudioLayer::RimDing,
            intensity,
            true,
            0.0,
            false,
        )],
        FeedbackKind::NearBucketMiss => vec![cue(
            kind,
            AudioBus::UiConfirmations,
            AudioLayer::SoftDownbeat,
            intensity,
            false,
            0.0,
            false,
        )],
        FeedbackKind::ComboThreshold => vec![cue(
            kind,
            AudioBus::ComboPercussion,
            AudioLayer::ComboPercussionHit,
            intensity,
            false,
            combo_pitch(combo),
            false,
        )],
        FeedbackKind::FinalOrangeTension => vec![cue(
            kind,
            AudioBus::AmbientBoardHum,
            AudioLayer::TensionRiser,
            intensity,
            true,
            0.0,
            false,
        )],
        FeedbackKind::ExtremeFever => vec![
            cue(
                kind,
                AudioBus::AmbientBoardHum,
                AudioLayer::SilenceGate,
                intensity,
                false,
                0.0,
                false,
            ),
            cue(
                kind,
                AudioBus::RewardStingers,
                AudioLayer::FeverSting,
                intensity,
                false,
                0.0,
                false,
            ),
        ],
        FeedbackKind::Loss => vec![cue(
            kind,
            AudioBus::UiConfirmations,
            AudioLayer::LossDownbeat,
            intensity,
            false,
            0.0,
            false,
        )],
    }
}

fn cue(
    kind: FeedbackKind,
    bus: AudioBus,
    layer: AudioLayer,
    intensity: f32,
    high_frequency: bool,
    pitch_semitones: f32,
    chord_cluster: bool,
) -> AudioCue {
    cue_with_pitch(
        kind,
        bus,
        layer,
        intensity,
        high_frequency,
        pitch_semitones,
        chord_cluster,
    )
}

fn cue_with_pitch(
    kind: FeedbackKind,
    bus: AudioBus,
    layer: AudioLayer,
    intensity: f32,
    high_frequency: bool,
    pitch_semitones: f32,
    chord_cluster: bool,
) -> AudioCue {
    AudioCue {
        kind,
        bus,
        layer,
        asset_path: asset_path_for_layer(layer),
        intensity,
        high_frequency,
        pitch_semitones,
        chord_cluster,
    }
}

pub fn asset_path_for_layer(layer: AudioLayer) -> Option<&'static str> {
    if layer == AudioLayer::SilenceGate {
        return None;
    }

    AUDIO_ASSET_MANIFEST
        .iter()
        .find(|entry| entry.layer == layer)
        .map(|entry| entry.path)
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
    use std::collections::HashSet;

    const ALL_FEEDBACK_KINDS: [FeedbackKind; 14] = [
        FeedbackKind::BallLaunch,
        FeedbackKind::PegHit,
        FeedbackKind::OrangeHit,
        FeedbackKind::PurpleHit,
        FeedbackKind::GreenHit,
        FeedbackKind::BucketCatch,
        FeedbackKind::NearBucketMiss,
        FeedbackKind::ComboThreshold,
        FeedbackKind::LongShot,
        FeedbackKind::LuckyBounce,
        FeedbackKind::FinalOrangeTension,
        FeedbackKind::ExtremeFever,
        FeedbackKind::RelicTriggered,
        FeedbackKind::Loss,
    ];

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
    fn all_feedback_kinds_have_audio_mapping() {
        for kind in ALL_FEEDBACK_KINDS {
            let cues = feedback_audio_mapping(kind, 1.0, 4);
            assert!(!cues.is_empty(), "{kind:?} lacks cue or silence gate");
            assert!(cues.iter().all(|cue| cue.kind == kind));
            assert!(cues
                .iter()
                .all(|cue| cue.layer == AudioLayer::SilenceGate || cue.asset_path.is_some()));
        }
    }

    #[test]
    fn all_audio_buses_have_runtime_channel() {
        let runtime_buses: HashSet<_> = RUNTIME_BUSES.iter().map(|bus| bus.bus).collect();
        assert_eq!(runtime_buses.len(), AudioBus::ALL.len());
        for bus in AudioBus::ALL {
            assert!(
                runtime_buses.contains(&bus),
                "{bus:?} lacks runtime channel"
            );
        }
        assert!(RUNTIME_BUSES.iter().all(|bus| !bus.channel_name.is_empty()));
    }

    #[test]
    fn loss_and_near_miss_do_not_use_victory_layers() {
        let mut state = AudioPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);

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
    fn high_frequency_mute_omits_plinks() {
        let normal = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags::DEFAULT);
        let muted = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags {
            reduce_shake: false,
            reduce_flash: false,
            mute_high_frequency_layers: true,
        });

        assert!(muted.emitted.len() < normal.emitted.len());
        assert!(!muted.emitted.iter().any(|cue| cue.high_frequency));
        assert!(!muted.emitted.iter().any(|cue| matches!(
            cue.layer,
            AudioLayer::PitchedPlink | AudioLayer::BrightPlink
        )));
    }

    #[test]
    fn extreme_fever_silence_gate_precedes_sting() {
        let cues = feedback_audio_mapping(FeedbackKind::ExtremeFever, 1.0, 0);
        assert_eq!(cues[0].layer, AudioLayer::SilenceGate);
        assert_eq!(cues[1].layer, AudioLayer::FeverSting);
        assert!(cues[0].asset_path.is_none());
        assert_eq!(cues[1].bus, AudioBus::RewardStingers);
    }

    #[test]
    fn audio_asset_manifest_paths_exist() {
        let manifest_layers: HashSet<_> = AUDIO_ASSET_MANIFEST
            .iter()
            .map(|entry| entry.layer)
            .collect();
        for layer in AudioLayer::ALL_AUDIBLE {
            assert!(
                manifest_layers.contains(&layer),
                "missing asset manifest entry for {layer:?}"
            );
        }

        for entry in AUDIO_ASSET_MANIFEST {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(entry.path);
            assert!(path.exists(), "missing audio asset {}", path.display());
        }
    }

    #[test]
    fn peg_pitch_caps_and_overloaded_hits_cluster() {
        let mut state = AudioPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);

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
