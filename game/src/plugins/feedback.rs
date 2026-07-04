use content_schema::{BoardDefinition, PegId, PegKind, Score, Vec2};
use feedback_events::{AccessibilityFeedbackFlags, FeedbackEvent, FeedbackKind};
use physics_core::{PhysicsEvent, ShotInput, ShotResult};

use crate::plugins::{audio::MockAudioPlaybackState, vfx::MockVfxPlaybackState};

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
    let mut caught_bucket = false;
    let mut exited_board = false;

    for event in &result.events {
        match event {
            PhysicsEvent::BallHitPeg { peg, position, .. } => {
                if let Some(kind) = peg_kind(board, peg) {
                    if let Some(feedback) = peg_feedback(*kind, *position, combo + 1) {
                        combo += 1;
                        events.push(feedback);
                    }
                }
            }
            PhysicsEvent::BallEnteredBucket { .. } => {
                caught_bucket = true;
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
            PhysicsEvent::BallHitObstacle { .. } => {}
        }
    }

    if !caught_bucket && exited_board {
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
            ],
            obstacles: Vec::new(),
            bucket: BasketDef::spec_default(),
            tags: Vec::new(),
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
                FeedbackKind::PegHit,
                FeedbackKind::OrangeHit,
                FeedbackKind::BucketCatch
            ]
        );
        assert_eq!(playback.summaries.len(), 3);
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

        assert_eq!(playback.events.len(), 1);
        assert_eq!(playback.events[0].kind, FeedbackKind::Loss);
        assert!(playback.events[0].intensity <= 0.2);
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
}
