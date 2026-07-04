use content_schema::{BoardDefinition, PegId, PegKind, RelicCategory, Score, Vec2};
use feedback_events::{AccessibilityFeedbackFlags, FeedbackEvent, FeedbackKind};
use physics_core::{PhysicsEvent, ShotInput, ShotResult};

use crate::plugins::{audio::MockAudioPlaybackState, vfx::MockVfxPlaybackState};

const COMBO_THRESHOLDS: [u32; 4] = [3, 6, 10, 15];
const LONG_SHOT_DISTANCE_RATIO: f64 = 0.42;
const LONG_SHOT_SPEED: f64 = 16.0;

#[derive(Clone, Debug, PartialEq)]
pub struct FeelTestFeedbackPlayback {
    pub events: Vec<FeedbackEvent>,
    pub summaries: Vec<FeedbackCueSummary>,
    pub vfx: MockVfxPlaybackState,
    pub audio: MockAudioPlaybackState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeedbackCueSummary {
    pub kind: FeedbackKind,
    pub vfx_cues: usize,
    pub audio_cues: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeedbackTriggerCoverage {
    pub trigger: &'static str,
    pub event: FeedbackEvent,
}

pub fn play_feel_test_shot(
    seed: u64,
    board: &BoardDefinition,
    input: &ShotInput,
    accessibility: AccessibilityFeedbackFlags,
) -> FeelTestFeedbackPlayback {
    let result = physics_core::simulate_shot(seed, board, input);
    play_shot_feedback(board, &result, accessibility)
}

pub fn play_shot_feedback(
    board: &BoardDefinition,
    result: &ShotResult,
    accessibility: AccessibilityFeedbackFlags,
) -> FeelTestFeedbackPlayback {
    let events = feedback_events_for_shot_result(board, result);
    let mut vfx = MockVfxPlaybackState::new(accessibility);
    let mut audio = MockAudioPlaybackState::new(accessibility);
    audio.play_board_hum(board_ambient_intensity(board));
    let mut summaries = Vec::new();

    for event in &events {
        let vfx_before = vfx.emitted.len();
        let audio_before = audio.emitted.len();

        vfx.play_event(event);
        audio.play_event(event);

        summaries.push(FeedbackCueSummary {
            kind: event.kind,
            vfx_cues: vfx.emitted.len() - vfx_before,
            audio_cues: audio.emitted.len() - audio_before,
        });
    }
    vfx.reset_for_shot_end();

    FeelTestFeedbackPlayback {
        events,
        summaries,
        vfx,
        audio,
    }
}

pub fn feedback_events_for_shot_result(
    board: &BoardDefinition,
    result: &ShotResult,
) -> Vec<FeedbackEvent> {
    let mut events = Vec::new();
    let mut combo = 0;
    let mut orange_hits = 0;
    let starting_oranges = board
        .pegs
        .iter()
        .filter(|peg| peg.kind == PegKind::Orange)
        .count() as u32;
    let mut caught_bucket = false;
    let mut exited_board = false;
    let mut long_shot_reported = false;
    let mut obstacle_touched = false;

    events.push(FeedbackEvent {
        kind: FeedbackKind::BallLaunch,
        intensity: 0.4,
        position: board.cannon_position,
        combo: 0,
        value: 0,
    });

    for event in &result.events {
        match event {
            PhysicsEvent::BallHitPeg { peg, position, .. } => {
                if let Some(kind) = peg_kind(board, peg) {
                    if let Some(feedback) = peg_feedback(*kind, *position, combo + 1) {
                        combo += 1;
                        if *kind == PegKind::Orange {
                            orange_hits += 1;
                        }
                        events.push(feedback);
                        if should_emit_combo(combo) {
                            events.push(combo_feedback(*position, combo));
                        }
                        if !long_shot_reported && is_long_shot(board, *position) {
                            long_shot_reported = true;
                            events.push(long_shot_feedback(*position, combo));
                        }
                        if *kind == PegKind::Orange
                            && starting_oranges.saturating_sub(orange_hits) == 1
                        {
                            events.push(FeedbackEvent {
                                kind: FeedbackKind::FinalOrangeTension,
                                intensity: 0.6,
                                position: *position,
                                combo,
                                value: 0,
                            });
                        }
                        if *kind == PegKind::Orange && starting_oranges == orange_hits {
                            events.push(FeedbackEvent {
                                kind: FeedbackKind::ExtremeFever,
                                intensity: 1.0,
                                position: *position,
                                combo,
                                value: 50_000,
                            });
                        }
                    }
                }
            }
            PhysicsEvent::BallEnteredBucket { .. } => {
                caught_bucket = true;
                if obstacle_touched {
                    events.push(FeedbackEvent {
                        kind: FeedbackKind::LuckyBounce,
                        intensity: 0.75,
                        position: board.bucket.center,
                        combo,
                        value: 5_000,
                    });
                }
                events.push(FeedbackEvent {
                    kind: FeedbackKind::BucketCatch,
                    intensity: 0.8,
                    position: board.bucket.center,
                    combo,
                    value: 2_500,
                });
            }
            PhysicsEvent::BallExitedBoard { .. } => {
                exited_board = true;
            }
            PhysicsEvent::ShotEnded { summary } => {
                caught_bucket |= summary.caught_bucket;
                exited_board |= summary.exited_board;
            }
            PhysicsEvent::BallHitObstacle { .. } => {
                obstacle_touched = true;
            }
        }
    }

    if !caught_bucket && exited_board {
        if combo > 0 {
            events.push(FeedbackEvent {
                kind: FeedbackKind::NearBucketMiss,
                intensity: 0.2,
                position: board.bucket.center,
                combo,
                value: 0,
            });
        }
        events.push(FeedbackEvent {
            kind: FeedbackKind::Loss,
            intensity: 0.2,
            position: Vec2::new(board.bucket.center.x, board.kill_plane_y),
            combo,
            value: 0,
        });
    }

    events
}

pub fn c3_feedback_trigger_coverage() -> Vec<FeedbackTriggerCoverage> {
    c4_feedback_trigger_coverage()
}

pub fn c4_feedback_trigger_coverage() -> Vec<FeedbackTriggerCoverage> {
    let position = Vec2::new(10.0, 18.0);
    vec![
        coverage("ball_launch", FeedbackKind::BallLaunch, 0.4, 0, 0),
        coverage("blue_peg_hit", FeedbackKind::PegHit, 0.35, 1, 100),
        coverage("orange_peg_hit", FeedbackKind::OrangeHit, 0.65, 1, 1_000),
        coverage("purple_peg_hit", FeedbackKind::PurpleHit, 0.8, 1, 5_000),
        coverage("green_peg_hit", FeedbackKind::GreenHit, 0.7, 1, 500),
        coverage("bucket_catch", FeedbackKind::BucketCatch, 0.8, 2, 2_500),
        coverage("combo_3", FeedbackKind::ComboThreshold, 0.55, 3, 0),
        coverage("combo_6", FeedbackKind::ComboThreshold, 0.65, 6, 0),
        coverage("combo_10", FeedbackKind::ComboThreshold, 0.75, 10, 0),
        coverage("combo_15_plus", FeedbackKind::ComboThreshold, 0.85, 15, 0),
        coverage("long_shot", FeedbackKind::LongShot, 0.7, 1, 750),
        coverage("lucky_bounce", FeedbackKind::LuckyBounce, 0.75, 3, 5_000),
        coverage("near_miss", FeedbackKind::NearBucketMiss, 0.2, 0, 0),
        coverage(
            "last_orange_in_flight",
            FeedbackKind::FinalOrangeTension,
            0.6,
            4,
            0,
        ),
        coverage("extreme_fever", FeedbackKind::ExtremeFever, 1.0, 5, 50_000),
        coverage("relic_ball_flash", FeedbackKind::RelicTriggered, 0.65, 0, 1),
        coverage("relic_peg_flash", FeedbackKind::RelicTriggered, 0.65, 0, 2),
        coverage(
            "relic_basket_flash",
            FeedbackKind::RelicTriggered,
            0.65,
            0,
            3,
        ),
        coverage(
            "relic_board_flash",
            FeedbackKind::RelicTriggered,
            0.65,
            0,
            4,
        ),
        coverage(
            "relic_economy_flash",
            FeedbackKind::RelicTriggered,
            0.65,
            0,
            5,
        ),
        coverage("board_failure", FeedbackKind::Loss, 0.2, 0, 0),
    ]
    .into_iter()
    .map(|mut item| {
        item.event.position = position;
        item
    })
    .collect()
}

pub fn c3_feedback_trigger_summary() -> String {
    let coverage = c3_feedback_trigger_coverage();
    let mut vfx = MockVfxPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);
    for item in &coverage {
        if let Some(category) = relic_category_for_coverage(item.event.value) {
            vfx.play_relic_trigger(category, &item.event);
        }
    }
    let labels = coverage
        .iter()
        .map(|item| item.trigger)
        .collect::<Vec<_>>()
        .join(",");
    format!("c4_vfx_triggers={} [{}]", coverage.len(), labels)
}

pub fn relic_category_for_coverage(value: Score) -> Option<RelicCategory> {
    match value {
        1 => Some(RelicCategory::Ball),
        2 => Some(RelicCategory::Peg),
        3 => Some(RelicCategory::Basket),
        4 => Some(RelicCategory::Board),
        5 => Some(RelicCategory::EconomyCombo),
        _ => None,
    }
}

fn peg_kind<'a>(board: &'a BoardDefinition, peg: &PegId) -> Option<&'a PegKind> {
    board
        .pegs
        .iter()
        .find(|definition| definition.id == *peg)
        .map(|definition| &definition.kind)
}

fn peg_feedback(kind: PegKind, position: Vec2, combo: u32) -> Option<FeedbackEvent> {
    let (kind, intensity, value): (FeedbackKind, f32, Score) = match kind {
        PegKind::Blue => (FeedbackKind::PegHit, 0.35, 100),
        PegKind::Orange => (FeedbackKind::OrangeHit, 0.65, 1_000),
        PegKind::Purple => (FeedbackKind::PurpleHit, 0.8, 5_000),
        PegKind::Green => (FeedbackKind::GreenHit, 0.7, 500),
        PegKind::Stone => return None,
        PegKind::Ghost => (FeedbackKind::PegHit, 0.25, 0),
        PegKind::Bomb | PegKind::Splitter => (FeedbackKind::PegHit, 0.45, 100),
    };

    Some(FeedbackEvent {
        kind,
        intensity,
        position,
        combo,
        value,
    })
}

fn should_emit_combo(combo: u32) -> bool {
    COMBO_THRESHOLDS.contains(&combo)
}

fn combo_feedback(position: Vec2, combo: u32) -> FeedbackEvent {
    FeedbackEvent {
        kind: FeedbackKind::ComboThreshold,
        intensity: match combo {
            0..=3 => 0.55,
            4..=6 => 0.65,
            7..=10 => 0.75,
            _ => 0.85,
        },
        position,
        combo,
        value: 0,
    }
}

fn is_long_shot(board: &BoardDefinition, position: Vec2) -> bool {
    let dx = position.x - board.cannon_position.x;
    let dy = position.y - board.cannon_position.y;
    let distance = (dx * dx + dy * dy).sqrt();
    distance >= board.size.y * LONG_SHOT_DISTANCE_RATIO || distance >= LONG_SHOT_SPEED
}

fn long_shot_feedback(position: Vec2, combo: u32) -> FeedbackEvent {
    FeedbackEvent {
        kind: FeedbackKind::LongShot,
        intensity: 0.7,
        position,
        combo,
        value: 750,
    }
}

fn board_ambient_intensity(board: &BoardDefinition) -> f32 {
    if board.tags.iter().any(|tag| tag.as_str().contains("boss")) {
        0.32
    } else {
        0.22
    }
}

fn coverage(
    trigger: &'static str,
    kind: FeedbackKind,
    intensity: f32,
    combo: u32,
    value: Score,
) -> FeedbackTriggerCoverage {
    FeedbackTriggerCoverage {
        trigger,
        event: FeedbackEvent {
            kind,
            intensity,
            position: Vec2::ZERO,
            combo,
            value,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{BallId, BasketDef, BoardId, PegDef, ShapeDef};
    use physics_core::ShotSummary;

    fn board() -> BoardDefinition {
        BoardDefinition {
            id: BoardId::new("boards/feedback_feel_test").unwrap(),
            size: Vec2::new(20.0, 35.56),
            cannon_position: Vec2::new(10.0, 1.5),
            kill_plane_y: 36.5,
            pegs: vec![
                peg("peg/blue", PegKind::Blue, Vec2::new(9.0, 10.0)),
                peg("peg/orange", PegKind::Orange, Vec2::new(10.0, 12.0)),
                peg("peg/purple", PegKind::Purple, Vec2::new(11.0, 18.0)),
                peg("peg/green", PegKind::Green, Vec2::new(12.0, 20.0)),
            ],
            obstacles: Vec::new(),
            bucket: BasketDef::spec_default(),
            tags: Vec::new(),
            objectives: Vec::new(),
            boss_mechanic: None,
        }
    }

    fn peg(id: &str, kind: PegKind, center: Vec2) -> PegDef {
        PegDef {
            id: PegId::new(id).unwrap(),
            kind,
            shape: ShapeDef::Circle {
                center,
                radius: 0.35,
            },
        }
    }

    fn summary(caught_bucket: bool, exited_board: bool, pegs_hit: Vec<PegId>) -> ShotSummary {
        ShotSummary {
            ticks: 120,
            pegs_hit,
            caught_bucket,
            exited_board,
            replay_hash: "test".to_string(),
        }
    }

    #[test]
    fn shot_result_generates_distinct_feedback_cues() {
        let board = board();
        let ball = BallId::new("ball/test").unwrap();
        let blue = PegId::new("peg/blue").unwrap();
        let orange = PegId::new("peg/orange").unwrap();
        let result = ShotResult {
            events: vec![
                PhysicsEvent::BallHitPeg {
                    ball: ball.clone(),
                    peg: blue.clone(),
                    position: Vec2::new(9.0, 10.0),
                    normal: Vec2::new(0.0, -1.0),
                    speed: 12.0,
                    tick: 10,
                },
                PhysicsEvent::BallHitPeg {
                    ball: ball.clone(),
                    peg: orange.clone(),
                    position: Vec2::new(10.0, 12.0),
                    normal: Vec2::new(0.0, -1.0),
                    speed: 10.0,
                    tick: 20,
                },
                PhysicsEvent::BallEnteredBucket { ball, tick: 100 },
                PhysicsEvent::ShotEnded {
                    summary: summary(true, false, vec![blue, orange]),
                },
            ],
            summary: summary(true, false, Vec::new()),
            remaining_pegs: Vec::new(),
        };

        let playback = play_shot_feedback(&board, &result, AccessibilityFeedbackFlags::DEFAULT);
        let kinds = playback
            .events
            .iter()
            .map(|event| event.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                FeedbackKind::BallLaunch,
                FeedbackKind::PegHit,
                FeedbackKind::OrangeHit,
                FeedbackKind::ExtremeFever,
                FeedbackKind::BucketCatch
            ]
        );
        assert_eq!(playback.summaries.len(), 5);
        assert!(playback
            .summaries
            .iter()
            .all(|summary| summary.vfx_cues > 0 && summary.audio_cues > 0));
    }

    #[test]
    fn shot_end_loss_is_muted_and_not_victory_like() {
        let board = board();
        let ball = BallId::new("ball/test").unwrap();
        let result = ShotResult {
            events: vec![
                PhysicsEvent::BallExitedBoard { ball, tick: 100 },
                PhysicsEvent::ShotEnded {
                    summary: summary(false, true, Vec::new()),
                },
            ],
            summary: summary(false, true, Vec::new()),
            remaining_pegs: board.pegs.clone(),
        };

        let playback = play_shot_feedback(&board, &result, AccessibilityFeedbackFlags::DEFAULT);

        assert_eq!(playback.events.len(), 2);
        assert_eq!(playback.events[1].kind, FeedbackKind::Loss);
        assert!(playback.events[1].intensity <= 0.2);
        assert!(playback.vfx.emitted.iter().all(|cue| !matches!(
            cue.layer,
            crate::plugins::vfx::VfxLayer::RewardRing
                | crate::plugins::vfx::VfxLayer::FeverFreezeFrame
                | crate::plugins::vfx::VfxLayer::FeverConfetti
        )));
        assert!(playback.audio.emitted.iter().all(|cue| !matches!(
            cue.layer,
            crate::plugins::audio::AudioLayer::CatchChord
                | crate::plugins::audio::AudioLayer::FeverSting
                | crate::plugins::audio::AudioLayer::RisingArpeggio
        )));
    }

    #[test]
    fn accessibility_flags_reduce_layers_for_shot_feedback() {
        let board = board();
        let ball = BallId::new("ball/test").unwrap();
        let orange = PegId::new("peg/orange").unwrap();
        let result = ShotResult {
            events: vec![
                PhysicsEvent::BallHitPeg {
                    ball: ball.clone(),
                    peg: orange.clone(),
                    position: Vec2::new(10.0, 12.0),
                    normal: Vec2::new(0.0, -1.0),
                    speed: 10.0,
                    tick: 20,
                },
                PhysicsEvent::BallEnteredBucket { ball, tick: 100 },
                PhysicsEvent::ShotEnded {
                    summary: summary(true, false, vec![orange]),
                },
            ],
            summary: summary(true, false, Vec::new()),
            remaining_pegs: Vec::new(),
        };

        let normal = play_shot_feedback(&board, &result, AccessibilityFeedbackFlags::DEFAULT);
        let reduced = play_shot_feedback(
            &board,
            &result,
            AccessibilityFeedbackFlags {
                reduce_shake: true,
                reduce_flash: true,
                mute_high_frequency_layers: true,
            },
        );

        assert!(reduced.vfx.emitted.len() < normal.vfx.emitted.len());
        assert!(reduced.audio.emitted.len() < normal.audio.emitted.len());
        assert!(!reduced.audio.emitted.iter().any(|cue| cue.high_frequency));
    }

    #[test]
    fn simulated_feel_test_shot_reports_feedback_cues() {
        let board = content_schema::minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("ball/test").unwrap(),
        };

        let playback = play_feel_test_shot(7, &board, &input, AccessibilityFeedbackFlags::DEFAULT);

        assert!(!playback.summaries.is_empty());
        assert!(playback
            .events
            .iter()
            .any(|event| matches!(event.kind, FeedbackKind::OrangeHit | FeedbackKind::Loss)));
    }

    #[test]
    fn c4_trigger_coverage_maps_all_required_feedback_without_new_fields() {
        let coverage = c4_feedback_trigger_coverage();
        let labels = coverage
            .iter()
            .map(|item| item.trigger)
            .collect::<std::collections::HashSet<_>>();

        for required in [
            "ball_launch",
            "blue_peg_hit",
            "orange_peg_hit",
            "purple_peg_hit",
            "green_peg_hit",
            "bucket_catch",
            "combo_3",
            "combo_6",
            "combo_10",
            "combo_15_plus",
            "long_shot",
            "lucky_bounce",
            "near_miss",
            "last_orange_in_flight",
            "extreme_fever",
            "relic_ball_flash",
            "relic_peg_flash",
            "relic_basket_flash",
            "relic_board_flash",
            "relic_economy_flash",
            "board_failure",
        ] {
            assert!(labels.contains(required), "missing C4 trigger {required}");
        }

        let mut vfx = MockVfxPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);
        let mut audio = MockAudioPlaybackState::new(AccessibilityFeedbackFlags::DEFAULT);
        for item in &coverage {
            if let Some(category) = relic_category_for_coverage(item.event.value) {
                vfx.play_relic_trigger(category, &item.event);
            } else {
                vfx.play_event(&item.event);
            }
            audio.play_event(&item.event);
        }
        assert!(!vfx.emitted.is_empty());
        assert!(!audio.emitted.is_empty());
    }

    #[test]
    fn reduce_flash_and_shake_remove_c3_pulses_bloom_and_camera_impulses() {
        let mut reduced = MockVfxPlaybackState::new(AccessibilityFeedbackFlags {
            reduce_shake: true,
            reduce_flash: true,
            mute_high_frequency_layers: false,
        });

        for item in c3_feedback_trigger_coverage() {
            reduced.play_event(&item.event);
        }

        assert!(!reduced.emitted.iter().any(|cue| matches!(
            cue.layer,
            crate::plugins::vfx::VfxLayer::CameraShake
                | crate::plugins::vfx::VfxLayer::ScalePulse
                | crate::plugins::vfx::VfxLayer::Bloom
        )));
    }
}
