use content_schema::RelicCategory;
use feedback_events::{AccessibilityFeedbackFlags, FeedbackEvent, FeedbackKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VfxRegistrationSummary {
    pub events: usize,
    pub cues: usize,
    pub camera_shake_cues: usize,
}

pub fn register() -> VfxRegistrationSummary {
    let events = mock_checkpoint1_feedback_sequence();
    let mut state = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags::DEFAULT);
    let relic_event = FeedbackEvent {
        kind: FeedbackKind::RelicTriggered,
        intensity: 0.65,
        position: content_schema::Vec2::ZERO,
        combo: 0,
        value: 1,
    };
    for category in [
        RelicCategory::Ball,
        RelicCategory::Peg,
        RelicCategory::Basket,
        RelicCategory::Board,
        RelicCategory::EconomyCombo,
    ] {
        state.play_relic_trigger(category, &relic_event);
    }

    VfxRegistrationSummary {
        events: events.len(),
        cues: state.emitted.len(),
        camera_shake_cues: state
            .emitted
            .iter()
            .filter(|cue| cue.layer == VfxLayer::CameraShake)
            .count(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockVfxCue {
    pub kind: FeedbackKind,
    pub layer: VfxLayer,
    pub intensity: f32,
    pub color: Option<VfxColor>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfxColor {
    Blue,
    Orange,
    Green,
    Purple,
    Gold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfxLayer {
    MuzzleFlash,
    AimLineCollapse,
    CannonRecoil,
    PegFlash,
    BurstParticles,
    RewardRing,
    PowerRing,
    ScoreBeam,
    BucketSnap,
    ComboRail,
    StreakTrail,
    KineticText,
    BucketEdgeSpark,
    NearMissMarker,
    FinalOrangeSpotlight,
    ScalePulse,
    Bloom,
    CameraShake,
    FeverFreezeFrame,
    FeverConfetti,
    LossFade,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockVfxPlaybackState {
    pub accessibility: AccessibilityFeedbackFlags,
    pub emitted: Vec<MockVfxCue>,
    pub combo_rail: ComboRailState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComboRailState {
    pub visible: bool,
    pub pulses: Vec<u32>,
}

impl MockVfxPlaybackState {
    pub fn new(accessibility: AccessibilityFeedbackFlags) -> Self {
        Self {
            accessibility,
            emitted: Vec::new(),
            combo_rail: ComboRailState::default(),
        }
    }

    pub fn play_event(&mut self, event: &FeedbackEvent) {
        let intensity = ethical_intensity(event.kind, event.intensity);

        match event.kind {
            FeedbackKind::BallLaunch => {
                self.emit(event.kind, VfxLayer::MuzzleFlash, intensity);
                self.emit(event.kind, VfxLayer::AimLineCollapse, intensity);
                self.emit(event.kind, VfxLayer::CannonRecoil, intensity);
            }
            FeedbackKind::PegHit => {
                self.emit(event.kind, VfxLayer::PegFlash, intensity);
                self.emit(event.kind, VfxLayer::BurstParticles, intensity);
            }
            FeedbackKind::OrangeHit => {
                self.emit(event.kind, VfxLayer::PegFlash, intensity);
                self.emit(event.kind, VfxLayer::RewardRing, intensity);
                self.emit(event.kind, VfxLayer::ScalePulse, intensity);
                self.emit(event.kind, VfxLayer::BurstParticles, intensity);
                self.emit(event.kind, VfxLayer::CameraShake, intensity);
            }
            FeedbackKind::PurpleHit => {
                self.emit(event.kind, VfxLayer::RewardRing, intensity);
                self.emit(event.kind, VfxLayer::ScoreBeam, intensity);
                self.emit(event.kind, VfxLayer::Bloom, intensity);
                self.emit(event.kind, VfxLayer::BurstParticles, intensity);
            }
            FeedbackKind::GreenHit => {
                self.emit(event.kind, VfxLayer::PowerRing, intensity);
                self.emit(event.kind, VfxLayer::ScalePulse, intensity);
                self.emit(event.kind, VfxLayer::BurstParticles, intensity);
            }
            FeedbackKind::RelicTriggered => {
                self.emit(event.kind, VfxLayer::PowerRing, intensity);
                self.emit(event.kind, VfxLayer::RewardRing, intensity);
            }
            FeedbackKind::BucketCatch => {
                self.emit(event.kind, VfxLayer::BucketSnap, intensity);
                self.emit(event.kind, VfxLayer::RewardRing, intensity);
                self.emit(event.kind, VfxLayer::ScalePulse, intensity);
                self.emit(event.kind, VfxLayer::CameraShake, intensity * 0.5);
            }
            FeedbackKind::NearBucketMiss => {
                self.emit(event.kind, VfxLayer::NearMissMarker, intensity);
            }
            FeedbackKind::ComboThreshold => {
                self.combo_rail.visible = event.combo >= 3;
                if [3, 6, 10].contains(&event.combo) || event.combo >= 15 {
                    self.combo_rail.pulses.push(event.combo);
                }
                self.emit(event.kind, VfxLayer::ComboRail, intensity);
                self.emit(event.kind, VfxLayer::BurstParticles, intensity);
            }
            FeedbackKind::LongShot => {
                self.emit(event.kind, VfxLayer::StreakTrail, intensity);
                self.emit(event.kind, VfxLayer::KineticText, intensity);
            }
            FeedbackKind::LuckyBounce => {
                self.emit(event.kind, VfxLayer::BucketEdgeSpark, intensity);
                self.emit(event.kind, VfxLayer::BurstParticles, intensity);
                self.emit(event.kind, VfxLayer::KineticText, intensity);
            }
            FeedbackKind::FinalOrangeTension => {
                self.emit(event.kind, VfxLayer::FinalOrangeSpotlight, intensity);
            }
            FeedbackKind::ExtremeFever => {
                self.emit(event.kind, VfxLayer::FeverFreezeFrame, intensity);
                self.emit(event.kind, VfxLayer::FeverConfetti, intensity);
                self.emit(event.kind, VfxLayer::Bloom, intensity);
                self.emit(event.kind, VfxLayer::CameraShake, intensity);
            }
            FeedbackKind::Loss => {
                self.emit(event.kind, VfxLayer::LossFade, intensity);
            }
        }
    }

    pub fn play_relic_trigger(&mut self, category: RelicCategory, event: &FeedbackEvent) {
        let intensity = ethical_intensity(event.kind, event.intensity);
        let color = relic_color(category);
        self.emit_colored(event.kind, VfxLayer::PowerRing, intensity, color);
        self.emit_colored(event.kind, VfxLayer::RewardRing, intensity, color);
    }

    pub fn reset_for_shot_end(&mut self) {
        self.combo_rail.visible = false;
    }

    fn emit(&mut self, kind: FeedbackKind, layer: VfxLayer, intensity: f32) {
        if self.accessibility.reduce_shake && layer == VfxLayer::CameraShake {
            return;
        }

        if self.accessibility.reduce_flash
            && matches!(
                layer,
                VfxLayer::PegFlash
                    | VfxLayer::RewardRing
                    | VfxLayer::FeverFreezeFrame
                    | VfxLayer::ScalePulse
                    | VfxLayer::Bloom
            )
        {
            return;
        }

        self.emitted.push(MockVfxCue {
            kind,
            layer,
            intensity,
            color: None,
        });
    }

    fn emit_colored(
        &mut self,
        kind: FeedbackKind,
        layer: VfxLayer,
        intensity: f32,
        color: VfxColor,
    ) {
        if self.accessibility.reduce_flash && layer == VfxLayer::RewardRing {
            return;
        }

        self.emitted.push(MockVfxCue {
            kind,
            layer,
            intensity,
            color: Some(color),
        });
    }
}

pub fn relic_color(category: RelicCategory) -> VfxColor {
    match category {
        RelicCategory::Ball => VfxColor::Blue,
        RelicCategory::Peg => VfxColor::Orange,
        RelicCategory::Basket => VfxColor::Green,
        RelicCategory::Board => VfxColor::Purple,
        RelicCategory::EconomyCombo => VfxColor::Gold,
    }
}

pub fn mock_checkpoint1_feedback_sequence() -> Vec<FeedbackEvent> {
    use content_schema::Vec2;

    vec![
        event(FeedbackKind::BallLaunch, 0.4, Vec2::new(10.0, 1.5), 0, 0),
        event(FeedbackKind::PegHit, 0.35, Vec2::new(5.0, 8.0), 1, 100),
        event(
            FeedbackKind::OrangeHit,
            0.65,
            Vec2::new(6.0, 10.0),
            2,
            1_000,
        ),
        event(FeedbackKind::PurpleHit, 0.8, Vec2::new(7.0, 12.0), 3, 5_000),
        event(FeedbackKind::GreenHit, 0.7, Vec2::new(8.0, 14.0), 4, 500),
        event(
            FeedbackKind::BucketCatch,
            0.8,
            Vec2::new(10.0, 34.4),
            4,
            2_500,
        ),
        event(
            FeedbackKind::NearBucketMiss,
            0.2,
            Vec2::new(12.0, 34.5),
            0,
            0,
        ),
        event(
            FeedbackKind::ComboThreshold,
            0.75,
            Vec2::new(9.0, 16.0),
            6,
            0,
        ),
        event(FeedbackKind::LongShot, 0.7, Vec2::new(3.0, 25.0), 6, 750),
        event(
            FeedbackKind::LuckyBounce,
            0.75,
            Vec2::new(10.0, 34.4),
            6,
            5_000,
        ),
        event(
            FeedbackKind::FinalOrangeTension,
            0.6,
            Vec2::new(11.0, 18.0),
            0,
            0,
        ),
        event(
            FeedbackKind::ExtremeFever,
            1.0,
            Vec2::new(10.0, 20.0),
            0,
            50_000,
        ),
        event(FeedbackKind::Loss, 0.2, Vec2::new(10.0, 34.8), 0, 0),
    ]
}

pub fn play_mock_checkpoint1_scene(
    accessibility: AccessibilityFeedbackFlags,
) -> MockVfxPlaybackState {
    let mut state = MockVfxPlaybackState::new(accessibility);

    for event in mock_checkpoint1_feedback_sequence() {
        state.play_event(&event);
    }

    state
}

fn event(
    kind: FeedbackKind,
    intensity: f32,
    position: content_schema::Vec2,
    combo: u32,
    value: content_schema::Score,
) -> FeedbackEvent {
    FeedbackEvent {
        kind,
        intensity,
        position,
        combo,
        value,
    }
}

fn ethical_intensity(kind: FeedbackKind, requested: f32) -> f32 {
    let cap = match kind {
        FeedbackKind::Loss => 0.2,
        FeedbackKind::NearBucketMiss => 0.25,
        FeedbackKind::PegHit => 0.45,
        FeedbackKind::BallLaunch => 0.5,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn checkpoint1_mock_sequence_covers_required_feedback() {
        let kinds = mock_checkpoint1_feedback_sequence()
            .into_iter()
            .map(|event| event.kind)
            .collect::<HashSet<_>>();

        assert_eq!(kinds.len(), 13);
        assert!(kinds.contains(&FeedbackKind::BallLaunch));
        assert!(kinds.contains(&FeedbackKind::PegHit));
        assert!(kinds.contains(&FeedbackKind::OrangeHit));
        assert!(kinds.contains(&FeedbackKind::PurpleHit));
        assert!(kinds.contains(&FeedbackKind::GreenHit));
        assert!(kinds.contains(&FeedbackKind::BucketCatch));
        assert!(kinds.contains(&FeedbackKind::NearBucketMiss));
        assert!(kinds.contains(&FeedbackKind::ComboThreshold));
        assert!(kinds.contains(&FeedbackKind::LongShot));
        assert!(kinds.contains(&FeedbackKind::LuckyBounce));
        assert!(kinds.contains(&FeedbackKind::FinalOrangeTension));
        assert!(kinds.contains(&FeedbackKind::ExtremeFever));
        assert!(kinds.contains(&FeedbackKind::Loss));
    }

    #[test]
    fn loss_and_near_miss_feedback_are_muted() {
        let mut state = MockVfxPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);

        state.play_event(&event(
            FeedbackKind::NearBucketMiss,
            1.0,
            content_schema::Vec2::ZERO,
            0,
            0,
        ));
        state.play_event(&event(
            FeedbackKind::Loss,
            1.0,
            content_schema::Vec2::ZERO,
            0,
            0,
        ));

        assert!(state.emitted.iter().all(|cue| cue.intensity <= 0.25));
        assert!(state.emitted.iter().all(|cue| !matches!(
            cue.layer,
            VfxLayer::RewardRing | VfxLayer::FeverFreezeFrame | VfxLayer::FeverConfetti
        )));
    }

    #[test]
    fn accessibility_flags_reduce_vfx_layers() {
        let normal = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags::DEFAULT);
        let reduced = play_mock_checkpoint1_scene(AccessibilityFeedbackFlags {
            reduce_shake: true,
            reduce_flash: true,
            mute_high_frequency_layers: false,
        });

        assert!(reduced.emitted.len() < normal.emitted.len());
        assert!(!reduced
            .emitted
            .iter()
            .any(|cue| cue.layer == VfxLayer::CameraShake));
        assert!(!reduced.emitted.iter().any(|cue| matches!(
            cue.layer,
            VfxLayer::PegFlash
                | VfxLayer::RewardRing
                | VfxLayer::FeverFreezeFrame
                | VfxLayer::ScalePulse
                | VfxLayer::Bloom
        )));
    }

    #[test]
    fn combo_rail_pulses_at_thresholds_and_resets_on_shot_end() {
        let mut state = MockVfxPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);

        for combo in [3, 6, 10, 15] {
            state.play_event(&event(
                FeedbackKind::ComboThreshold,
                0.75,
                content_schema::Vec2::ZERO,
                combo,
                0,
            ));
        }

        assert!(state.combo_rail.visible);
        assert_eq!(state.combo_rail.pulses, vec![3, 6, 10, 15]);
        state.reset_for_shot_end();
        assert!(!state.combo_rail.visible);
    }

    #[test]
    fn relic_trigger_color_is_distinct_by_category() {
        assert_eq!(relic_color(RelicCategory::Ball), VfxColor::Blue);
        assert_eq!(relic_color(RelicCategory::Peg), VfxColor::Orange);
        assert_eq!(relic_color(RelicCategory::Basket), VfxColor::Green);
        assert_eq!(relic_color(RelicCategory::Board), VfxColor::Purple);
        assert_eq!(relic_color(RelicCategory::EconomyCombo), VfxColor::Gold);
    }
}
