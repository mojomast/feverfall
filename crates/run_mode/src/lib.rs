use content_schema::{BallId, BoardId, ContentId, RelicId, Seed};
use game_rules::GameEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunState {
    pub run_id: ContentId,
    pub act: u8,
    pub node_index: u16,
    pub resources: RunResources,
    pub relics: Vec<RelicInstance>,
    pub balls: Vec<BallId>,
    pub curse: u32,
    pub rng_state: Seed,
    pub visited_nodes: Vec<RunNode>,
}

impl RunState {
    pub fn new(seed: Seed) -> Self {
        Self {
            run_id: ContentId::new(format!("run/{seed:016x}")).expect("formatted run id is valid"),
            act: 1,
            node_index: 0,
            resources: RunResources::default(),
            relics: Vec::new(),
            balls: vec![BallId::new("balls/basic").expect("static id is valid")],
            curse: 0,
            rng_state: seed,
            visited_nodes: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunResources {
    pub shots: u32,
    pub hearts: u32,
    pub coins: u32,
    pub sparks: u32,
    pub keys: u32,
}

impl Default for RunResources {
    fn default() -> Self {
        Self {
            shots: 10,
            hearts: 3,
            coins: 0,
            sparks: 0,
            keys: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelicInstance {
    pub id: RelicId,
    pub stacks: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunNode {
    pub id: ContentId,
    pub act: u8,
    pub kind: RunNodeKind,
    pub board: Option<BoardId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunNodeKind {
    Board,
    EliteBoard,
    Shop,
    Event,
    Forge,
    Camp,
    Boss,
}

pub trait RelicModifier {
    fn relic_id(&self) -> &RelicId;

    fn on_event(&self, event: &GameEvent, state: &mut RunState);
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RewardOffer {
    pub choices: Vec<Reward>,
    pub rarity: RewardRarity,
    pub source: ContentId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Reward {
    Relic(RelicId),
    Ball(BallId),
    Coins(u32),
    Heal(u32),
    RemoveCurse(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardRarity {
    Common,
    Uncommon,
    Rare,
    Boss,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_state_round_trips_json() {
        let state = RunState::new(1234);

        let json = serde_json::to_string(&state).unwrap();
        let parsed: RunState = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, state);
    }
}
