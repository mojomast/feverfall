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

    pub fn advance_to_node(&mut self, node: RunNode) {
        self.visited_nodes.push(node);
        self.node_index = self.visited_nodes.len() as u16;
    }

    pub fn apply_reward(&mut self, reward: &Reward) {
        match reward {
            Reward::Relic(id) => {
                if let Some(existing) = self.relics.iter_mut().find(|relic| relic.id == *id) {
                    existing.stacks += 1;
                } else {
                    self.relics.push(RelicInstance {
                        id: id.clone(),
                        stacks: 1,
                    });
                }
            }
            Reward::Ball(id) => self.balls.push(id.clone()),
            Reward::Coins(amount) => self.resources.coins += amount,
            Reward::Heal(amount) => self.resources.hearts += amount,
            Reward::RemoveCurse(amount) => {
                self.curse = self.curse.saturating_sub(*amount);
            }
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
    Reward,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelicCategory {
    Ball,
    Peg,
    Basket,
    Board,
    EconomyCombo,
}

pub fn relic_category(relic: &RelicId) -> Option<RelicCategory> {
    match relic.as_str() {
        "relics/act1/ball_splitter" => Some(RelicCategory::Ball),
        "relics/act1/orange_echo" => Some(RelicCategory::Peg),
        "relics/act1/steady_bucket" | "relics/act1/spark_catcher" => Some(RelicCategory::Basket),
        "relics/act1/stonebreaker" => Some(RelicCategory::Board),
        "relics/act1/feverheart" => Some(RelicCategory::EconomyCombo),
        _ => None,
    }
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
            kind: RunNodeKind::Reward,
            board: None,
        },
        RunNode {
            id: ContentId::new("runs/act1/node_03").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Board,
            board: Some(BoardId::new("boards/feel_wave_01").expect("static id is valid")),
        },
        RunNode {
            id: ContentId::new("runs/act1/node_04").expect("static id is valid"),
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
            source: ContentId::new("runs/act1/node_04").expect("static id is valid"),
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
        assert_eq!(nodes.len(), 4);
        assert!(nodes.iter().all(|node| node.act == 1));
        assert_eq!(nodes[0].kind, RunNodeKind::Board);
        assert_eq!(nodes[1].kind, RunNodeKind::Reward);
        assert_eq!(nodes[2].kind, RunNodeKind::Board);
        assert_eq!(nodes[3].kind, RunNodeKind::Boss);
        assert!(nodes.iter().filter(|node| node.board.is_some()).count() >= 3);
        assert!(matches!(nodes.last().unwrap().kind, RunNodeKind::Boss));
        assert_eq!(rewards.len(), 4);
        assert!(rewards.iter().all(|offer| !offer.choices.is_empty()));
    }

    #[test]
    fn two_board_run_state_round_trips_after_rewards_and_node_advancement() {
        let mut state = RunState::act1_slice(0xC2_12);
        let nodes = act1_slice_nodes();

        state.advance_to_node(nodes[0].clone());
        state.advance_to_node(nodes[1].clone());
        state.apply_reward(&Reward::Coins(12));
        state.advance_to_node(nodes[2].clone());
        state.apply_reward(&Reward::Relic(
            RelicId::new("relics/act1/orange_echo").unwrap(),
        ));

        let json = serde_json::to_string(&state).unwrap();
        let parsed: RunState = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, state);
        assert_eq!(parsed.node_index, 3);
        assert_eq!(
            parsed.visited_nodes,
            vec![nodes[0].clone(), nodes[1].clone(), nodes[2].clone()]
        );
        assert_eq!(
            parsed.visited_nodes[0].board.as_ref().unwrap().as_str(),
            "boards/feel_fan_01"
        );
        assert_eq!(
            parsed.visited_nodes[2].board.as_ref().unwrap().as_str(),
            "boards/feel_wave_01"
        );
        assert_eq!(
            parsed.resources.coins,
            RunResources::act1_slice().coins + 12
        );
        assert!(parsed
            .relics
            .iter()
            .any(|relic| relic.id.as_str() == "relics/act1/orange_echo" && relic.stacks == 1));
    }

    #[test]
    fn reward_application_handles_resources_balls_and_relic_stacks() {
        let mut state = RunState::act1_slice(99);
        state.curse = 2;

        state.apply_reward(&Reward::Coins(5));
        state.apply_reward(&Reward::Heal(1));
        state.apply_reward(&Reward::RemoveCurse(1));
        state.apply_reward(&Reward::Ball(BallId::new("balls/heavy").unwrap()));
        state.apply_reward(&Reward::Relic(
            RelicId::new("relics/act1/steady_bucket").unwrap(),
        ));
        state.apply_reward(&Reward::Relic(
            RelicId::new("relics/act1/steady_bucket").unwrap(),
        ));

        assert_eq!(state.resources.coins, RunResources::act1_slice().coins + 5);
        assert_eq!(
            state.resources.hearts,
            RunResources::act1_slice().hearts + 1
        );
        assert_eq!(state.curse, 1);
        assert!(state
            .balls
            .iter()
            .any(|ball| ball.as_str() == "balls/heavy"));
        assert!(state.relics.iter().any(|relic| {
            relic.id.as_str() == "relics/act1/steady_bucket" && relic.stacks == 2
        }));
    }

    #[test]
    fn reward_application_covers_all_relic_categories() {
        let category_relics = [
            ("relics/act1/ball_splitter", RelicCategory::Ball),
            ("relics/act1/orange_echo", RelicCategory::Peg),
            ("relics/act1/steady_bucket", RelicCategory::Basket),
            ("relics/act1/stonebreaker", RelicCategory::Board),
            ("relics/act1/feverheart", RelicCategory::EconomyCombo),
        ];

        for (id, category) in category_relics {
            let mut state = RunState::new(7);
            let relic_id = RelicId::new(id).unwrap();

            state.apply_reward(&Reward::Relic(relic_id.clone()));

            assert_eq!(relic_category(&relic_id), Some(category));
            assert_eq!(state.relics.len(), 1);
            assert_eq!(state.relics[0].id, relic_id);
            assert_eq!(state.relics[0].stacks, 1);
        }
    }

    #[test]
    fn node_map_advancement_records_ordered_path() {
        let nodes = act1_slice_nodes();
        let mut state = RunState::new(5);

        state.advance_to_node(nodes[0].clone());
        state.advance_to_node(nodes[1].clone());
        state.advance_to_node(nodes[2].clone());

        assert_eq!(state.node_index, 3);
        assert_eq!(state.visited_nodes.len(), 3);
        assert_eq!(state.visited_nodes[0].id.as_str(), "runs/act1/node_01");
        assert_eq!(state.visited_nodes[1].id.as_str(), "runs/act1/node_02");
        assert_eq!(state.visited_nodes[1].kind, RunNodeKind::Reward);
        assert!(state.visited_nodes[1].board.is_none());
        assert_eq!(state.visited_nodes[2].id.as_str(), "runs/act1/node_03");
        assert_eq!(
            state.visited_nodes[2].board.as_ref().unwrap().as_str(),
            "boards/feel_wave_01"
        );
    }
}
