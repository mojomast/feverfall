use content_schema::{
    BallId, BoardDefinition, BoardId, ContentId, ObstacleDef, ObstacleId, ObstacleKind, PegKind,
    RelicId, Seed, ShapeDef, Vec2,
};
use feedback_events::{FeedbackEvent, FeedbackKind};
use game_rules::GameEvent;
use serde::{Deserialize, Serialize};

pub const META_SAVE_PATH: &str = "saves/roguelite/meta.json";

pub const ROGUELITE_SAVE_DIR: &str = "saves/roguelite/";
pub const ROGUELITE_BALANCE_DIR: &str = "content/balance/roguelite/";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
            id: RelicId::new("relics/act1/wide_cup_rim").expect("static id is valid"),
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
            Reward::Curse { reward_rarity } => {
                self.curse += 1;
                if matches!(reward_rarity, RewardRarity::Rare | RewardRarity::Boss) {
                    self.resources.coins += 12;
                } else {
                    self.resources.coins += 6;
                }
            }
        }
    }

    pub fn accept_curse(&mut self) -> RewardRarity {
        self.curse += 1;
        self.resources.sparks += 1;
        match self.curse {
            0 | 1 => RewardRarity::Uncommon,
            2 => RewardRarity::Rare,
            _ => RewardRarity::Boss,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelicInstance {
    pub id: RelicId,
    pub stacks: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunNode {
    pub id: ContentId,
    pub act: u8,
    pub kind: RunNodeKind,
    pub board: Option<BoardId>,
    #[serde(default)]
    pub path_choices: Vec<ContentId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunNodeKind {
    Board,
    Reward,
    Elite,
    Shop,
    Event,
    Forge,
    Camp,
    Boss,
}

pub trait RelicModifier {
    fn relic_id(&self) -> &RelicId;

    fn modify_board(&self, board: &mut BoardDefinition, state: &RunState) -> Option<FeedbackEvent>;

    fn on_event(&self, event: &GameEvent, state: &mut RunState) -> Option<FeedbackEvent>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardOffer {
    pub choices: Vec<Reward>,
    pub rarity: RewardRarity,
    pub source: ContentId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reward {
    Relic(RelicId),
    Ball(BallId),
    Coins(u32),
    Heal(u32),
    RemoveCurse(u32),
    Curse { reward_rarity: RewardRarity },
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
        "relics/act1/ghost_thread"
        | "relics/act1/rubber_glaze"
        | "relics/act1/heavy_core"
        | "relics/act1/splitter_charm" => Some(RelicCategory::Ball),
        "relics/act1/purple_pin"
        | "relics/act1/green_relay"
        | "relics/act1/blue_spark_wire"
        | "relics/act1/orange_lacquer" => Some(RelicCategory::Peg),
        "relics/act1/rim_ricochet"
        | "relics/act1/magnet_bell"
        | "relics/act1/wide_cup_rim"
        | "relics/act1/catch_streak_sash" => Some(RelicCategory::Basket),
        "relics/act1/cartographer_thread"
        | "relics/act1/bumper_seed"
        | "relics/act1/lane_lantern"
        | "relics/act1/stone_chisel" => Some(RelicCategory::Board),
        "relics/act1/risk_purse"
        | "relics/act1/combo_abacus"
        | "relics/act1/copper_interest"
        | "relics/act1/shopkeepers_stamp" => Some(RelicCategory::EconomyCombo),
        _ => None,
    }
}

pub fn all_relic_ids() -> Vec<RelicId> {
    [
        "relics/act1/ghost_thread",
        "relics/act1/rubber_glaze",
        "relics/act1/heavy_core",
        "relics/act1/splitter_charm",
        "relics/act1/purple_pin",
        "relics/act1/green_relay",
        "relics/act1/blue_spark_wire",
        "relics/act1/orange_lacquer",
        "relics/act1/rim_ricochet",
        "relics/act1/magnet_bell",
        "relics/act1/wide_cup_rim",
        "relics/act1/catch_streak_sash",
        "relics/act1/cartographer_thread",
        "relics/act1/bumper_seed",
        "relics/act1/lane_lantern",
        "relics/act1/stone_chisel",
        "relics/act1/risk_purse",
        "relics/act1/combo_abacus",
        "relics/act1/copper_interest",
        "relics/act1/shopkeepers_stamp",
    ]
    .into_iter()
    .map(|id| RelicId::new(id).expect("static id is valid"))
    .collect()
}

pub struct ContentRelicModifier {
    id: RelicId,
}

impl ContentRelicModifier {
    pub fn new(id: RelicId) -> Option<Self> {
        relic_category(&id).map(|_| Self { id })
    }

    fn trigger(&self, value: i64) -> FeedbackEvent {
        FeedbackEvent {
            kind: FeedbackKind::RelicTriggered,
            intensity: 0.65,
            position: Vec2::ZERO,
            combo: 0,
            value,
        }
    }
}

impl RelicModifier for ContentRelicModifier {
    fn relic_id(&self) -> &RelicId {
        &self.id
    }

    fn modify_board(&self, board: &mut BoardDefinition, state: &RunState) -> Option<FeedbackEvent> {
        let before = board.clone();
        match self.id.as_str() {
            "relics/act1/ghost_thread" => convert_first_peg(board, PegKind::Stone, PegKind::Ghost),
            "relics/act1/rubber_glaze" => board.bucket.catch_margin += 0.05,
            "relics/act1/heavy_core" => widen_first_peg(board, PegKind::Orange, 1.08),
            "relics/act1/splitter_charm" => {
                convert_first_peg(board, PegKind::Blue, PegKind::Splitter)
            }
            "relics/act1/orange_lacquer" => widen_first_peg(board, PegKind::Orange, 1.12),
            "relics/act1/wide_cup_rim" => board.bucket.width += 0.6,
            "relics/act1/magnet_bell" => {
                board.bucket.width += 0.25;
                board.bucket.catch_margin += 0.08;
            }
            "relics/act1/bumper_seed" => board.obstacles.push(ObstacleDef {
                id: ObstacleId::new(format!("relic_bumper_{}", state.node_index))
                    .expect("formatted id is valid"),
                kind: ObstacleKind::Rubber,
                shape: ShapeDef::Circle {
                    center: Vec2::new(1.2, 24.0),
                    radius: 0.45,
                },
            }),
            "relics/act1/stone_chisel" => {
                if let Some(index) = board.pegs.iter().position(|peg| peg.kind == PegKind::Stone) {
                    board.pegs.remove(index);
                }
            }
            "relics/act1/lane_lantern" | "relics/act1/cartographer_thread" => {
                let tag =
                    ContentId::new(format!("modifiers/{}", self.id.as_str().replace('/', ":")))
                        .expect("formatted id is valid");
                if !board.tags.contains(&tag) {
                    board.tags.push(tag);
                }
            }
            "relics/act1/risk_purse" if state.curse > 0 => {
                widen_first_peg(board, PegKind::Orange, 0.95)
            }
            _ => {}
        }
        (before != *board).then(|| self.trigger(1))
    }

    fn on_event(&self, event: &GameEvent, state: &mut RunState) -> Option<FeedbackEvent> {
        match (self.id.as_str(), event) {
            ("relics/act1/blue_spark_wire", GameEvent::PegScored { peg, .. })
                if peg.as_str().contains("blue") =>
            {
                state.resources.sparks += 1;
                Some(self.trigger(1))
            }
            ("relics/act1/green_relay", GameEvent::PegScored { peg, .. })
                if peg.as_str().contains("green") =>
            {
                state.resources.sparks += 2;
                Some(self.trigger(2))
            }
            ("relics/act1/purple_pin", GameEvent::PegScored { peg, .. })
                if peg.as_str().contains("purple") =>
            {
                state.resources.coins += 3;
                Some(self.trigger(3))
            }
            ("relics/act1/rim_ricochet", GameEvent::BucketCatchAwarded { .. }) => {
                state.resources.sparks += 1;
                Some(self.trigger(1))
            }
            ("relics/act1/catch_streak_sash", GameEvent::BucketCatchAwarded { .. }) => {
                state.resources.coins += 5;
                Some(self.trigger(5))
            }
            (
                "relics/act1/combo_abacus",
                GameEvent::ShotScoreResolved {
                    combo_multiplier, ..
                },
            ) if *combo_multiplier > 1 => {
                state.resources.sparks += *combo_multiplier;
                Some(self.trigger(i64::from(*combo_multiplier)))
            }
            ("relics/act1/copper_interest", GameEvent::BoardWon { .. }) => {
                let interest = (state.resources.coins / 10).min(8);
                state.resources.coins += interest;
                Some(self.trigger(i64::from(interest)))
            }
            ("relics/act1/risk_purse", GameEvent::BoardWon { .. }) if state.curse > 0 => {
                let payout = state.curse * 7;
                state.resources.coins += payout;
                Some(self.trigger(i64::from(payout)))
            }
            ("relics/act1/shopkeepers_stamp", GameEvent::ResourceChanged { resource, delta })
                if matches!(resource, game_rules::ResourceKind::Coins) && *delta < 0 =>
            {
                state.resources.coins += 2;
                Some(self.trigger(2))
            }
            _ => None,
        }
    }
}

pub fn apply_relic_board_modifiers(
    state: &RunState,
    board: &mut BoardDefinition,
) -> Vec<FeedbackEvent> {
    state
        .relics
        .iter()
        .filter_map(|relic| ContentRelicModifier::new(relic.id.clone()))
        .filter_map(|modifier| modifier.modify_board(board, state))
        .collect()
}

pub fn trigger_relics_on_event(event: &GameEvent, state: &mut RunState) -> Vec<FeedbackEvent> {
    let relics = state.relics.clone();
    relics
        .into_iter()
        .filter_map(|relic| ContentRelicModifier::new(relic.id))
        .filter_map(|modifier| modifier.on_event(event, state))
        .collect()
}

fn convert_first_peg(board: &mut BoardDefinition, from: PegKind, to: PegKind) {
    if let Some(peg) = board.pegs.iter_mut().find(|peg| peg.kind == from) {
        peg.kind = to;
    }
}

fn widen_first_peg(board: &mut BoardDefinition, kind: PegKind, multiplier: f64) {
    if let Some(peg) = board.pegs.iter_mut().find(|peg| peg.kind == kind) {
        match &mut peg.shape {
            ShapeDef::Circle { radius, .. } | ShapeDef::Capsule { radius, .. } => {
                *radius *= multiplier;
            }
            ShapeDef::Rect { half_extents, .. } => {
                half_extents.x *= multiplier;
                half_extents.y *= multiplier;
            }
            ShapeDef::Segment { .. } => {}
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunActPlan {
    pub act: u8,
    pub normal_boards: u8,
    pub elites: u8,
    pub bosses: u8,
}

pub fn full_run_act_plan() -> Vec<RunActPlan> {
    vec![
        RunActPlan {
            act: 1,
            normal_boards: 6,
            elites: 1,
            bosses: 1,
        },
        RunActPlan {
            act: 2,
            normal_boards: 7,
            elites: 2,
            bosses: 1,
        },
        RunActPlan {
            act: 3,
            normal_boards: 8,
            elites: 2,
            bosses: 1,
        },
    ]
}

pub fn full_run_nodes() -> Vec<RunNode> {
    let mut nodes = Vec::new();
    for plan in full_run_act_plan() {
        for index in 1..=plan.normal_boards {
            nodes.push(run_node(
                plan.act,
                index,
                RunNodeKind::Board,
                Some(format!("boards/act{}/normal_{index:02}", plan.act)),
            ));
        }
        for index in 1..=plan.elites {
            nodes.push(run_node(
                plan.act,
                plan.normal_boards + index,
                RunNodeKind::Elite,
                Some(format!("boards/act{}/elite_{index:02}", plan.act)),
            ));
        }
        nodes.push(run_node(plan.act, 20, RunNodeKind::Event, None));
        nodes.push(run_node(plan.act, 21, RunNodeKind::Shop, None));
        nodes.push(run_node(plan.act, 22, RunNodeKind::Forge, None));
        nodes.push(run_node(
            plan.act,
            plan.normal_boards + plan.elites + 1,
            RunNodeKind::Camp,
            None,
        ));
        nodes.push(run_node(
            plan.act,
            plan.normal_boards + plan.elites + 2,
            RunNodeKind::Boss,
            Some(format!("boards/act{}/boss_01", plan.act)),
        ));
    }

    for act in 1..=3 {
        let choices = [2_u8, 3_u8]
            .into_iter()
            .map(|index| ContentId::new(format!("runs/act{act}/node_{index:02}")).unwrap())
            .collect::<Vec<_>>();
        let first_id = format!("runs/act{act}/node_01");
        if let Some(node) = nodes
            .iter_mut()
            .find(|node| node.id.as_str() == first_id.as_str())
        {
            node.path_choices = choices;
        }
    }
    nodes
}

fn run_node(act: u8, index: u8, kind: RunNodeKind, board: Option<String>) -> RunNode {
    RunNode {
        id: ContentId::new(format!("runs/act{act}/node_{index:02}")).unwrap(),
        act,
        kind,
        board: board.map(|id| BoardId::new(id).unwrap()),
        path_choices: Vec::new(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaProgressionSave {
    pub total_runs: u64,
    pub total_oranges_cleared: u64,
    pub relics_seen: Vec<RelicId>,
    pub unlocked_starter_balls: Vec<BallId>,
    pub unlocked_starting_relics: Vec<RelicId>,
    pub unlocked_board_archetype_weights: Vec<ContentId>,
}

impl Default for MetaProgressionSave {
    fn default() -> Self {
        Self {
            total_runs: 0,
            total_oranges_cleared: 0,
            relics_seen: Vec::new(),
            unlocked_starter_balls: vec![BallId::new("balls/act1/basic_orb").unwrap()],
            unlocked_starting_relics: Vec::new(),
            unlocked_board_archetype_weights: Vec::new(),
        }
    }
}

impl MetaProgressionSave {
    pub fn record_run_end(&mut self, oranges_cleared: u64, relics: &[RelicInstance]) {
        self.total_runs += 1;
        self.total_oranges_cleared += oranges_cleared;
        for relic in relics {
            if !self.relics_seen.contains(&relic.id) {
                self.relics_seen.push(relic.id.clone());
            }
        }
    }

    pub fn next_unlock_offer(&self) -> [MetaUnlock; 3] {
        [
            MetaUnlock::StarterBall(BallId::new("balls/act1/rubber_orb").unwrap()),
            MetaUnlock::StartingRelic(RelicId::new("relics/act1/wide_cup_rim").unwrap()),
            MetaUnlock::BoardArchetypeWeight(ContentId::new("archetypes/act2/lanes_plus").unwrap()),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetaUnlock {
    StarterBall(BallId),
    StartingRelic(RelicId),
    BoardArchetypeWeight(ContentId),
}

pub fn act1_slice_nodes() -> Vec<RunNode> {
    vec![
        RunNode {
            id: ContentId::new("runs/act1/node_01").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Board,
            board: Some(BoardId::new("boards/feel_fan_01").expect("static id is valid")),
            path_choices: vec![
                ContentId::new("runs/act1/node_02").expect("static id is valid"),
                ContentId::new("runs/act1/node_03").expect("static id is valid"),
            ],
        },
        RunNode {
            id: ContentId::new("runs/act1/node_02").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Reward,
            board: None,
            path_choices: Vec::new(),
        },
        RunNode {
            id: ContentId::new("runs/act1/node_03").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Board,
            board: Some(BoardId::new("boards/feel_wave_01").expect("static id is valid")),
            path_choices: Vec::new(),
        },
        RunNode {
            id: ContentId::new("runs/act1/node_04").expect("static id is valid"),
            act: 1,
            kind: RunNodeKind::Boss,
            board: Some(BoardId::new("boards/feel_fortress_stone_01").expect("static id is valid")),
            path_choices: Vec::new(),
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
                    RelicId::new("relics/act1/wide_cup_rim").expect("static id is valid"),
                ),
            ],
            rarity: RewardRarity::Common,
            source: ContentId::new("runs/act1/node_01").expect("static id is valid"),
        },
        RewardOffer {
            choices: vec![
                Reward::Heal(1),
                Reward::Coins(18),
                Reward::Relic(
                    RelicId::new("relics/act1/orange_lacquer").expect("static id is valid"),
                ),
            ],
            rarity: RewardRarity::Uncommon,
            source: ContentId::new("runs/act1/node_02").expect("static id is valid"),
        },
        RewardOffer {
            choices: vec![
                Reward::Relic(
                    RelicId::new("relics/act1/stone_chisel").expect("static id is valid"),
                ),
                Reward::Ball(BallId::new("balls/heavy").expect("static id is valid")),
                Reward::RemoveCurse(1),
            ],
            rarity: RewardRarity::Rare,
            source: ContentId::new("runs/act1/node_03").expect("static id is valid"),
        },
        RewardOffer {
            choices: vec![
                Reward::Relic(
                    RelicId::new("relics/act1/copper_interest").expect("static id is valid"),
                ),
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
            RelicId::new("relics/act1/orange_lacquer").unwrap(),
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
            .any(|relic| relic.id.as_str() == "relics/act1/orange_lacquer" && relic.stacks == 1));
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
            RelicId::new("relics/act1/wide_cup_rim").unwrap(),
        ));
        state.apply_reward(&Reward::Relic(
            RelicId::new("relics/act1/wide_cup_rim").unwrap(),
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
        assert!(state
            .relics
            .iter()
            .any(|relic| { relic.id.as_str() == "relics/act1/wide_cup_rim" && relic.stacks == 3 }));
    }

    #[test]
    fn reward_application_covers_all_relic_categories() {
        let category_relics = [
            ("relics/act1/splitter_charm", RelicCategory::Ball),
            ("relics/act1/orange_lacquer", RelicCategory::Peg),
            ("relics/act1/wide_cup_rim", RelicCategory::Basket),
            ("relics/act1/stone_chisel", RelicCategory::Board),
            ("relics/act1/copper_interest", RelicCategory::EconomyCombo),
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

    #[test]
    fn full_run_plan_has_required_three_act_composition_and_branching() {
        let nodes = full_run_nodes();

        for plan in full_run_act_plan() {
            let act_nodes = nodes
                .iter()
                .filter(|node| node.act == plan.act)
                .collect::<Vec<_>>();
            assert_eq!(
                act_nodes
                    .iter()
                    .filter(|node| node.kind == RunNodeKind::Board)
                    .count(),
                usize::from(plan.normal_boards)
            );
            assert_eq!(
                act_nodes
                    .iter()
                    .filter(|node| node.kind == RunNodeKind::Elite)
                    .count(),
                usize::from(plan.elites)
            );
            assert_eq!(
                act_nodes
                    .iter()
                    .filter(|node| node.kind == RunNodeKind::Boss)
                    .count(),
                usize::from(plan.bosses)
            );
            assert!(act_nodes.iter().any(|node| node.kind == RunNodeKind::Shop));
            assert!(act_nodes.iter().any(|node| node.kind == RunNodeKind::Event));
            assert!(act_nodes.iter().any(|node| node.kind == RunNodeKind::Forge));
            assert!(act_nodes.iter().any(|node| node.kind == RunNodeKind::Camp));
            assert!(act_nodes.iter().any(|node| node.path_choices.len() >= 2));
        }
    }

    #[test]
    fn all_twenty_content_relics_are_categorized_and_construct_modifiers() {
        let relics = all_relic_ids();

        assert_eq!(relics.len(), 20);
        for relic in relics {
            assert!(
                relic_category(&relic).is_some(),
                "missing category for {relic}"
            );
            assert!(ContentRelicModifier::new(relic).is_some());
        }
    }

    #[test]
    fn relic_board_modifiers_cover_all_visible_categories() {
        let mut state = RunState::new(3);
        state.curse = 1;
        state.relics = vec![
            RelicInstance {
                id: RelicId::new("relics/act1/splitter_charm").unwrap(),
                stacks: 1,
            },
            RelicInstance {
                id: RelicId::new("relics/act1/orange_lacquer").unwrap(),
                stacks: 1,
            },
            RelicInstance {
                id: RelicId::new("relics/act1/wide_cup_rim").unwrap(),
                stacks: 1,
            },
            RelicInstance {
                id: RelicId::new("relics/act1/bumper_seed").unwrap(),
                stacks: 1,
            },
            RelicInstance {
                id: RelicId::new("relics/act1/risk_purse").unwrap(),
                stacks: 1,
            },
        ];
        let mut board = content_schema::minimal_test_board();
        let before = board.clone();

        let feedback = apply_relic_board_modifiers(&state, &mut board);

        assert_ne!(board, before);
        assert!(feedback
            .iter()
            .all(|event| event.kind == FeedbackKind::RelicTriggered));
        assert!(board.pegs.iter().any(|peg| peg.kind == PegKind::Splitter));
        assert!(board.bucket.width > before.bucket.width);
        assert!(board.obstacles.len() > before.obstacles.len());
    }

    #[test]
    fn relic_event_triggers_mutate_run_state_and_emit_feedback() {
        let mut state = RunState::new(4);
        state.resources.coins = 20;
        state.relics = vec![RelicInstance {
            id: RelicId::new("relics/act1/copper_interest").unwrap(),
            stacks: 1,
        }];
        let event = GameEvent::BoardWon {
            board: BoardId::new("boards/minimal_test").unwrap(),
            final_score: 1000,
        };

        let feedback = trigger_relics_on_event(&event, &mut state);

        assert_eq!(state.resources.coins, 22);
        assert_eq!(feedback.len(), 1);
        assert_eq!(feedback[0].kind, FeedbackKind::RelicTriggered);
    }

    #[test]
    fn curse_acceptance_adds_risk_and_improves_reward_rarity() {
        let mut state = RunState::new(5);

        assert_eq!(state.accept_curse(), RewardRarity::Uncommon);
        assert_eq!(state.accept_curse(), RewardRarity::Rare);
        assert_eq!(state.curse, 2);
        assert_eq!(state.resources.sparks, 2);
    }

    #[test]
    fn meta_progression_records_run_and_offers_three_unlock_types() {
        let mut save = MetaProgressionSave::default();
        let relics = vec![RelicInstance {
            id: RelicId::new("relics/act1/wide_cup_rim").unwrap(),
            stacks: 1,
        }];

        save.record_run_end(12, &relics);
        let offers = save.next_unlock_offer();

        assert_eq!(META_SAVE_PATH, "saves/roguelite/meta.json");
        assert_eq!(save.total_runs, 1);
        assert_eq!(save.total_oranges_cleared, 12);
        assert!(matches!(offers[0], MetaUnlock::StarterBall(_)));
        assert!(matches!(offers[1], MetaUnlock::StartingRelic(_)));
        assert!(matches!(offers[2], MetaUnlock::BoardArchetypeWeight(_)));
    }
}
