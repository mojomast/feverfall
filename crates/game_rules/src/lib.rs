use content_schema::{BallId, BoardId, ContentId, PegId, RelicId, Score, Seed, SkillId};
use physics_core::PhysicsEvent;
use serde::{Deserialize, Serialize};

pub const RULESET_VERSION: &str = "game_rules/0.1.0-contracts";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GameEvent {
    Physics(PhysicsEvent),
    PegScored {
        peg: PegId,
        points: Score,
    },
    BucketCatchAwarded {
        ball: BallId,
        points: Score,
        shots_granted: u32,
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
        reason: LossReason,
    },
    ResourceChanged {
        resource: ResourceKind,
        delta: i64,
    },
    RelicTriggered {
        relic: RelicId,
        source_event: Option<Box<GameEvent>>,
    },
    SkillUsed {
        skill: SkillId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossReason {
    OutOfShots,
    ObjectiveFailed,
    Forfeit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Balls,
    Hearts,
    Coins,
    Sparks,
    Keys,
    Curse,
    Xp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayMetadata {
    pub ruleset_version: String,
    pub physics_version: String,
    pub content_manifest_hash: u64,
    pub seed: Seed,
    pub mode: ReplayMode,
    pub board_id: BoardId,
    pub initial_hash: u64,
    pub final_hash: Option<u64>,
    pub shot_hashes: Vec<u64>,
    pub input_commands: Vec<ReplayInput>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayMode {
    PhysicsTest,
    Roguelite,
    RpgCampaign,
    Tooling,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayInput {
    pub tick: u64,
    pub command: ContentId,
    pub payload_hash: u64,
}

pub fn promote_physics_event(event: PhysicsEvent) -> GameEvent {
    GameEvent::Physics(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_metadata_round_trips_json() {
        let metadata = ReplayMetadata {
            ruleset_version: RULESET_VERSION.to_string(),
            physics_version: physics_core::PHYSICS_VERSION.to_string(),
            content_manifest_hash: 42,
            seed: 7,
            mode: ReplayMode::PhysicsTest,
            board_id: BoardId::new("boards/minimal_test").unwrap(),
            initial_hash: 1,
            final_hash: Some(2),
            shot_hashes: vec![100, 200],
            input_commands: vec![ReplayInput {
                tick: 0,
                command: ContentId::new("input/shoot").unwrap(),
                payload_hash: 99,
            }],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: ReplayMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, metadata);
    }
}
