use content_schema::{
    BallId, BoardId, ContentId, PegId, RelicId, Scalar, Score, Seed, SkillId, Tick, Vec2,
};
use game_rules::{GameEvent, LossReason};
use physics_core::{PhysicsEvent, ShotSummary};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

pub const TELEMETRY_SCHEMA_VERSION: &str = "telemetry/0.2.0-checkpoint2";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryEnvelope {
    pub schema_version: String,
    pub session_id: String,
    pub sequence: u64,
    pub event: TelemetryEvent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TelemetryEvent {
    ShotStarted {
        board: BoardId,
        ball: BallId,
        seed: Seed,
        aim_angle_radians: Scalar,
        launch_speed: Scalar,
    },
    PegHit {
        ball: BallId,
        peg: PegId,
        position: Vec2,
        speed: Scalar,
        tick: Tick,
    },
    BucketCatch {
        ball: BallId,
        tick: Tick,
    },
    ShotResolved {
        board: BoardId,
        shot_index: u32,
        ticks: Tick,
        pegs_hit: u32,
        caught_bucket: bool,
        exited_board: bool,
        replay_hash: String,
    },
    ShotScoreResolved {
        base_score: Score,
        fever_multiplier: u32,
        combo_multiplier: u32,
        final_score: Score,
    },
    BoardWon {
        board: BoardId,
        final_score: Score,
    },
    BoardLost {
        board: BoardId,
        reason: ContentId,
    },
    RelicChosen {
        relic: RelicId,
    },
    SkillUsed {
        skill: SkillId,
    },
    ReplayHash {
        board: BoardId,
        shot_index: u32,
        replay_hash: String,
    },
    ReplayTagged {
        tag: ReplayTag,
    },
    RunEnded {
        final_score: Score,
        boards_cleared: u32,
        oranges_cleared: u32,
        bucket_catches: u32,
        relics_collected: Vec<RelicId>,
        xp_gained: u64,
        character_level: u32,
        run_duration_shots: u32,
        replay_hash: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayTag {
    pub replay_hash: String,
    pub board: BoardId,
    pub seed: Seed,
    pub labels: Vec<ReplayLabel>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayLabel {
    DeterminismBaseline,
    FeelTooFloaty,
    FeelTooPinball,
    BucketCatchSatisfying,
    BucketCatchMissed,
    PhysicsFeltUnfair,
    FirstBounceReadable,
    VerticalSliceFailure,
    Custom(String),
}

pub fn shot_summary_to_telemetry(
    board: BoardId,
    shot_index: u32,
    summary: &ShotSummary,
) -> TelemetryEvent {
    TelemetryEvent::ShotResolved {
        board,
        shot_index,
        ticks: summary.ticks,
        pegs_hit: summary.pegs_hit.len() as u32,
        caught_bucket: summary.caught_bucket,
        exited_board: summary.exited_board,
        replay_hash: summary.replay_hash.clone(),
    }
}

pub struct JsonlTelemetryLogger<W> {
    writer: W,
    session_id: String,
    sequence: u64,
}

impl<W: Write> JsonlTelemetryLogger<W> {
    pub fn new(writer: W, session_id: impl Into<String>) -> Self {
        Self {
            writer,
            session_id: session_id.into(),
            sequence: 0,
        }
    }

    pub fn log(&mut self, event: TelemetryEvent) -> io::Result<()> {
        let envelope = TelemetryEnvelope {
            schema_version: TELEMETRY_SCHEMA_VERSION.to_string(),
            session_id: self.session_id.clone(),
            sequence: self.sequence,
            event,
        };
        serde_json::to_writer(&mut self.writer, &envelope).map_err(io::Error::other)?;
        self.writer.write_all(b"\n")?;
        self.sequence += 1;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

pub fn physics_event_to_telemetry(event: &PhysicsEvent) -> Option<TelemetryEvent> {
    match event {
        PhysicsEvent::BallHitPeg {
            ball,
            peg,
            position,
            speed,
            tick,
            ..
        } => Some(TelemetryEvent::PegHit {
            ball: ball.clone(),
            peg: peg.clone(),
            position: *position,
            speed: *speed,
            tick: *tick,
        }),
        PhysicsEvent::BallEnteredBucket { ball, tick } => Some(TelemetryEvent::BucketCatch {
            ball: ball.clone(),
            tick: *tick,
        }),
        _ => None,
    }
}

pub fn game_event_to_telemetry(event: &GameEvent) -> Option<TelemetryEvent> {
    match event {
        GameEvent::ShotScoreResolved {
            base_score,
            fever_multiplier,
            combo_multiplier,
            final_score,
        } => Some(TelemetryEvent::ShotScoreResolved {
            base_score: *base_score,
            fever_multiplier: *fever_multiplier,
            combo_multiplier: *combo_multiplier,
            final_score: *final_score,
        }),
        GameEvent::BoardWon { board, final_score } => Some(TelemetryEvent::BoardWon {
            board: board.clone(),
            final_score: *final_score,
        }),
        GameEvent::BoardLost { board, reason } => Some(TelemetryEvent::BoardLost {
            board: board.clone(),
            reason: loss_reason_id(*reason),
        }),
        GameEvent::SkillUsed { skill } => Some(TelemetryEvent::SkillUsed {
            skill: skill.clone(),
        }),
        _ => None,
    }
}

fn loss_reason_id(reason: LossReason) -> ContentId {
    let id = match reason {
        LossReason::OutOfShots => "loss/out_of_shots",
        LossReason::ObjectiveFailed => "loss/objective_failed",
        LossReason::Forfeit => "loss/forfeit",
    };
    ContentId::new(id).expect("loss reason ids are valid content ids")
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{minimal_test_board, BallId, BoardId};
    use game_rules::LossReason;
    use physics_core::{simulate_shot, ShotInput};

    fn shot_input() -> ShotInput {
        ShotInput {
            aim_angle_radians: 1.18,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        }
    }

    #[test]
    fn jsonl_logger_writes_parseable_envelopes() {
        let board = minimal_test_board();
        let mut logger = JsonlTelemetryLogger::new(Vec::new(), "local-session-001");

        logger
            .log(TelemetryEvent::ReplayHash {
                board: board.id,
                shot_index: 0,
                replay_hash: "abc123".to_string(),
            })
            .unwrap();

        let bytes = logger.into_inner();
        let line = std::str::from_utf8(&bytes).unwrap().trim_end();
        let envelope: TelemetryEnvelope = serde_json::from_str(line).unwrap();

        assert_eq!(envelope.schema_version, TELEMETRY_SCHEMA_VERSION);
        assert_eq!(envelope.session_id, "local-session-001");
        assert_eq!(envelope.sequence, 0);
    }

    #[test]
    fn logging_does_not_change_replay_hash_for_same_shot() {
        let board = minimal_test_board();
        let input = shot_input();
        let before = simulate_shot(123, &board, &input);

        let mut logger = JsonlTelemetryLogger::new(Vec::new(), "determinism-check");
        for event in &before.events {
            if let Some(telemetry_event) = physics_event_to_telemetry(event) {
                logger.log(telemetry_event).unwrap();
            }
        }
        logger
            .log(TelemetryEvent::ReplayHash {
                board: board.id.clone(),
                shot_index: 0,
                replay_hash: before.summary.replay_hash.clone(),
            })
            .unwrap();
        let _jsonl = logger.into_inner();

        let after = simulate_shot(123, &board, &input);

        assert_eq!(before.summary.replay_hash, after.summary.replay_hash);
    }

    #[test]
    fn telemetry_events_and_replay_tags_do_not_change_replay_hash() {
        let board = minimal_test_board();
        let input = shot_input();
        let before = simulate_shot(321, &board, &input);

        let mut logger = JsonlTelemetryLogger::new(Vec::new(), "run-session-determinism-check");
        logger
            .log(TelemetryEvent::ShotScoreResolved {
                base_score: 1_000,
                fever_multiplier: 1,
                combo_multiplier: 1,
                final_score: 1_000,
            })
            .unwrap();
        logger
            .log(TelemetryEvent::RelicChosen {
                relic: RelicId::new("relics/act1/orange_lacquer").unwrap(),
            })
            .unwrap();
        logger
            .log(TelemetryEvent::ReplayTagged {
                tag: ReplayTag {
                    replay_hash: before.summary.replay_hash.clone(),
                    board: board.id.clone(),
                    seed: 321,
                    labels: vec![
                        ReplayLabel::DeterminismBaseline,
                        ReplayLabel::Custom("run-session:act1-node-02".to_owned()),
                    ],
                    notes: Some("Two-board run-session QA tag".to_owned()),
                },
            })
            .unwrap();
        let _jsonl = logger.into_inner();

        let after = simulate_shot(321, &board, &input);

        assert_eq!(before.summary.replay_hash, after.summary.replay_hash);
    }

    #[test]
    fn telemetry_derivation_and_logging_do_not_mutate_physics_state() {
        let board = minimal_test_board();
        let input = shot_input();
        let board_before = board.clone();
        let input_before = input.clone();
        let result_before = simulate_shot(777, &board, &input);
        let events_before = result_before.events.clone();
        let hash_before = physics_core::stable_hash_events(&events_before);

        let mut logger = JsonlTelemetryLogger::new(Vec::new(), "physics-state-audit");
        logger
            .log(TelemetryEvent::ShotStarted {
                board: board.id.clone(),
                ball: input.ball_id.clone(),
                seed: 777,
                aim_angle_radians: input.aim_angle_radians,
                launch_speed: input.launch_speed,
            })
            .unwrap();
        for event in &result_before.events {
            if let Some(telemetry_event) = physics_event_to_telemetry(event) {
                logger.log(telemetry_event).unwrap();
            }
        }
        logger
            .log(shot_summary_to_telemetry(
                board.id.clone(),
                0,
                &result_before.summary,
            ))
            .unwrap();
        logger.flush().unwrap();

        let result_after = simulate_shot(777, &board, &input);

        assert_eq!(board, board_before);
        assert_eq!(input, input_before);
        assert_eq!(events_before, result_before.events);
        assert_eq!(
            hash_before,
            physics_core::stable_hash_events(&events_before)
        );
        assert_eq!(
            result_before.summary.replay_hash,
            result_after.summary.replay_hash
        );
        assert_eq!(events_before, result_after.events);
    }

    #[test]
    fn shot_summary_maps_to_vertical_slice_result_event() {
        let board = minimal_test_board();
        let input = shot_input();
        let result = simulate_shot(123, &board, &input);

        let event = shot_summary_to_telemetry(board.id.clone(), 2, &result.summary);

        assert_eq!(
            event,
            TelemetryEvent::ShotResolved {
                board: board.id,
                shot_index: 2,
                ticks: result.summary.ticks,
                pegs_hit: result.summary.pegs_hit.len() as u32,
                caught_bucket: result.summary.caught_bucket,
                exited_board: result.summary.exited_board,
                replay_hash: result.summary.replay_hash,
            }
        );
    }

    #[test]
    fn game_events_map_to_score_and_progression_telemetry() {
        let score_event = GameEvent::ShotScoreResolved {
            base_score: 100,
            fever_multiplier: 2,
            combo_multiplier: 3,
            final_score: 600,
        };
        let lost_event = GameEvent::BoardLost {
            board: BoardId::new("boards/minimal_test").unwrap(),
            reason: LossReason::OutOfShots,
        };

        assert_eq!(
            game_event_to_telemetry(&score_event),
            Some(TelemetryEvent::ShotScoreResolved {
                base_score: 100,
                fever_multiplier: 2,
                combo_multiplier: 3,
                final_score: 600,
            })
        );
        assert_eq!(
            game_event_to_telemetry(&lost_event),
            Some(TelemetryEvent::BoardLost {
                board: BoardId::new("boards/minimal_test").unwrap(),
                reason: ContentId::new("loss/out_of_shots").unwrap(),
            })
        );
    }

    #[test]
    fn run_ended_telemetry_event_serializes_summary_fields() {
        let event = TelemetryEvent::RunEnded {
            final_score: 42_000,
            boards_cleared: 2,
            oranges_cleared: 25,
            bucket_catches: 4,
            relics_collected: vec![RelicId::new("relics/act1/wide_cup_rim").unwrap()],
            xp_gained: 18,
            character_level: 3,
            run_duration_shots: 11,
            replay_hash: "abc123".to_owned(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: TelemetryEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, event);
        assert!(json.contains("RunEnded"));
        assert!(json.contains("final_score"));
        assert!(json.contains("relics/act1/wide_cup_rim"));
        assert!(json.contains("run_duration_shots"));
    }
}
