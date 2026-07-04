use content_schema::{
    BallId, BoardId, ContentId, PegId, RelicId, Scalar, Seed, SkillId, Tick, Vec2,
};
use physics_core::PhysicsEvent;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

pub const TELEMETRY_SCHEMA_VERSION: &str = "telemetry/0.1.0-checkpoint1";

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
    BoardWon {
        board: BoardId,
        final_score: i64,
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
    PhysicsFeltUnfair,
    FirstBounceReadable,
    Custom(String),
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

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{minimal_test_board, BallId};
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
}
