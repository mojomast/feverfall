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

    pub fn act1_slice(seed: Seed) -> Self {
        let mut state = Self::new(seed);
        state.resources = RunResources::act1_slice();
        state.relics.push(RelicInstance {
            id: RelicId::new("relics/act1/spark_catcher").expect("static id is valid"),
            stacks: 1,
        });
        state
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

impl RunResources {
    pub fn act1_slice() -> Self {
        Self {
            shots: 8,
            hearts: 3,
            coins: 10,
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

pub fn act1_slice_nodes() -> Vec<RunNode> {
    vec![
        RunNode {
            id: ContentId::new("runs/act1/node_01").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Board,
            board: Some(BoardId::new("boards/feel_fan_01").expect("static id is valid")),
        },
        RunNode {
            id: ContentId::new("runs/act1/node_02").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Board,
            board: Some(BoardId::new("boards/feel_wave_01").expect("static id is valid")),
        },
        RunNode {
            id: ContentId::new("runs/act1/node_03").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::EliteBoard,
            board: Some(BoardId::new("boards/feel_clusters_stone_01").expect("static id is valid")),
        },
        RunNode {
            id: ContentId::new("runs/act1/node_04").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Camp,
            board: None,
        },
        RunNode {
            id: ContentId::new("runs/act1/node_05").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Boss,
            board: Some(BoardId::new("boards/feel_fortress_stone_01").expect("static id is valid")),
        },
    ]
}

pub fn act1_slice_reward_offers() -> Vec<RewardOffer> {
    vec![
        RewardOffer {
            choices: vec![
                Reward::Coins(12),
                Reward::Ball(BallId::new("balls/spark").expect("static id is valid")),
                Reward::Relic(
                    RelicId::new("relics/act1/steady_bucket").expect("static id is valid"),
                ),
            ],
            rarity: RewardRarity::Common,
            source: ContentId::new("runs/act1/node_01").expect("static id is valid"),
        },
        RewardOffer {
            choices: vec![
                Reward::Heal(1),
                Reward::Coins(18),
                Reward::Relic(RelicId::new("relics/act1/orange_echo").expect("static id is valid")),
            ],
            rarity: RewardRarity::Uncommon,
            source: ContentId::new("runs/act1/node_02").expect("static id is valid"),
        },
        RewardOffer {
            choices: vec![
                Reward::Relic(
                    RelicId::new("relics/act1/stonebreaker").expect("static id is valid"),
                ),
                Reward::Ball(BallId::new("balls/heavy").expect("static id is valid")),
                Reward::RemoveCurse(1),
            ],
            rarity: RewardRarity::Rare,
            source: ContentId::new("runs/act1/node_03").expect("static id is valid"),
        },
        RewardOffer {
            choices: vec![
                Reward::Relic(RelicId::new("relics/act1/feverheart").expect("static id is valid")),
                Reward::Coins(40),
            ],
            rarity: RewardRarity::Boss,
            source: ContentId::new("runs/act1/node_05").expect("static id is valid"),
        },
    ]
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

    #[test]
    fn act1_slice_has_playable_path_and_rewards() {
        let state = RunState::act1_slice(42);
        let nodes = act1_slice_nodes();
        let rewards = act1_slice_reward_offers();

        assert_eq!(state.act, 1);
        assert_eq!(state.resources.shots, 8);
        assert_eq!(state.balls, vec![BallId::new("balls/basic").unwrap()]);
        assert_eq!(nodes.len(), 5);
        assert!(nodes.iter().all(|node| node.act == 1));
        assert!(nodes.iter().filter(|node| node.board.is_some()).count() >= 3);
        assert!(matches!(nodes.last().unwrap().kind, RunNodeKind::Boss));
        assert_eq!(rewards.len(), 4);
        assert!(rewards.iter().all(|offer| !offer.choices.is_empty()));
    }
}
