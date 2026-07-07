//! Roguelite balance simulation core for Checkpoint 5.
//!
//! C5-INTEGRATE owns `main.rs`; this table-driven module is ready to be wired
//! there without preserving the previous hardcoded roguelite constants.

use anyhow::{anyhow, Context, Result};
use board_gen::{generate_board, GenerationParams};
use content_schema::{BallId, BoardDefinition, ContentId, PegDef, PegKind, Seed, ShapeDef, Vec2};
use physics_core::{simulate_shot, ShotInput};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const ACTS_FALLBACK: u8 = 3;
const DEFAULT_RUNS: usize = 1_000;
const DEFAULT_SEED_START: Seed = 0xC43A_0000_0000_0000;

#[derive(Clone, Debug, PartialEq)]
pub struct RogueliteBalanceTables {
    pub board_curve: BoardCurve,
    pub reward_pool: RewardPool,
    pub scoring_curve: ScoringCurve,
    pub sim_targets: SimTargets,
}

impl RogueliteBalanceTables {
    pub fn load_from_dir(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        Self::from_toml_strings(
            &fs::read_to_string(path.join("board_curve.toml"))?,
            &fs::read_to_string(path.join("reward_pool.toml"))?,
            &fs::read_to_string(path.join("scoring_curve.toml"))?,
            &fs::read_to_string(path.join("sim_targets.toml"))?,
        )
    }

    pub fn from_toml_strings(
        board: &str,
        rewards: &str,
        scoring: &str,
        targets: &str,
    ) -> Result<Self> {
        Ok(Self {
            board_curve: BoardCurve::parse(board)?,
            reward_pool: RewardPool::parse(rewards),
            scoring_curve: ScoringCurve::parse(scoring)?,
            sim_targets: SimTargets::parse(targets)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoardCurve {
    pub acts: Vec<ActCurve>,
}

impl BoardCurve {
    fn parse(src: &str) -> Result<Self> {
        let acts = array_tables(src, "acts")
            .into_iter()
            .map(|table| {
                Ok(ActCurve {
                    act: parse_required_u8(table, "act")?,
                    boards: parse_required_u32(table, "boards")?,
                    orange_min: parse_required_u32(table, "orange_min")?,
                    orange_max: parse_required_u32(table, "orange_max")?,
                    starting_balls: parse_required_u32(table, "starting_balls")?,
                    elite_orange_bonus: parse_required_u32(table, "elite_orange_bonus")?,
                    boss_orange_bonus: parse_required_u32(table, "boss_orange_bonus")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if acts.is_empty() {
            return Err(anyhow!("board_curve.toml must define [[acts]]"));
        }
        Ok(Self { acts })
    }

    fn act(&self, act: u8) -> &ActCurve {
        self.acts
            .iter()
            .find(|curve| curve.act == act)
            .unwrap_or_else(|| self.acts.last().unwrap())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActCurve {
    pub act: u8,
    pub boards: u32,
    pub orange_min: u32,
    pub orange_max: u32,
    pub starting_balls: u32,
    pub elite_orange_bonus: u32,
    pub boss_orange_bonus: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RewardPool {
    pub reward_type_weights: BTreeMap<String, u32>,
    pub board_clear_relic_drop_rate: f64,
    pub guaranteed_relic_until: u32,
}

impl RewardPool {
    fn parse(src: &str) -> Self {
        Self {
            reward_type_weights: parse_u32_section(src, "reward_type_weights"),
            board_clear_relic_drop_rate: parse_section_f64(src, "relic_drop_rates", "board_clear")
                .unwrap_or(0.0),
            guaranteed_relic_until: parse_section_u32(
                src,
                "reward_offer",
                "guaranteed_relic_choice_until_relic_count",
            )
            .unwrap_or(0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScoringCurve {
    pub free_ball_thresholds: Vec<u32>,
    pub repeat_step: u32,
    pub combo_multipliers: Vec<(u32, f64)>,
}

impl ScoringCurve {
    fn parse(src: &str) -> Result<Self> {
        let mut thresholds = ["first", "second", "third"]
            .iter()
            .filter_map(|key| parse_section_u32(src, "free_ball_score_thresholds", key))
            .collect::<Vec<_>>();
        thresholds.sort_unstable();
        let combo_multipliers = array_tables(src, "combo_multiplier_curve")
            .into_iter()
            .map(|table| {
                Ok((
                    parse_required_u32(table, "hits")?,
                    parse_required_f64(table, "multiplier")?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            free_ball_thresholds: thresholds,
            repeat_step: parse_section_u32(src, "free_ball_score_thresholds", "repeat_step")
                .unwrap_or(55_000),
            combo_multipliers,
        })
    }

    fn free_balls_for_score(&self, score: u32) -> u32 {
        let mut earned = self
            .free_ball_thresholds
            .iter()
            .filter(|threshold| score >= **threshold)
            .count() as u32;
        if let Some(last) = self.free_ball_thresholds.last() {
            if score > *last && self.repeat_step > 0 {
                earned += (score - last) / self.repeat_step;
            }
        }
        earned
    }

    fn combo_multiplier(&self, hits: u32) -> f64 {
        self.combo_multipliers
            .iter()
            .filter(|(needed, _)| hits >= *needed)
            .map(|(_, multiplier)| *multiplier)
            .next_back()
            .unwrap_or(1.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimTargets {
    pub acts: u8,
    pub default_runs: usize,
    pub seed_start: Seed,
    pub cohorts: Vec<CohortTarget>,
}

impl SimTargets {
    fn parse(src: &str) -> Result<Self> {
        let cohorts = array_tables(src, "cohorts")
            .into_iter()
            .map(CohortTarget::parse)
            .collect::<Result<Vec<_>>>()?;
        if cohorts.is_empty() {
            return Err(anyhow!("sim_targets.toml must define cohorts"));
        }
        Ok(Self {
            acts: parse_section_u32(src, "run_shape", "acts").unwrap_or(u32::from(ACTS_FALLBACK))
                as u8,
            default_runs: parse_section_u32(src, "run_shape", "default_runs")
                .unwrap_or(DEFAULT_RUNS as u32) as usize,
            seed_start: parse_section_u64(src, "run_shape", "seed_start")
                .unwrap_or(DEFAULT_SEED_START),
            cohorts,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CohortTarget {
    pub id: String,
    pub aim_model: AimModel,
    pub bucket_awareness: bool,
    pub orange_targeting: bool,
    pub meta_progression: bool,
    pub act1_clear_range: Option<(f64, f64)>,
    pub full_run_win_range: Option<(f64, f64)>,
    pub measured_by: Option<String>,
}

impl CohortTarget {
    fn parse(table: &str) -> Result<Self> {
        Ok(Self {
            id: parse_required_string(table, "id")?,
            aim_model: AimModel::parse(&parse_required_string(table, "aim_model")?)?,
            bucket_awareness: parse_required_bool(table, "bucket_awareness")?,
            orange_targeting: parse_required_bool(table, "orange_targeting")?,
            meta_progression: parse_required_bool(table, "meta_progression")?,
            act1_clear_range: parse_optional_range(table, "act1_clear_min", "act1_clear_max")?,
            full_run_win_range: parse_optional_range(
                table,
                "full_run_win_min",
                "full_run_win_max",
            )?,
            measured_by: parse_string(table, "measured_by"),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AimModel {
    Random,
    CenterBias,
    OrangeTargeting,
    BucketAware,
    HumanPlaytest,
}

impl AimModel {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "random" => Ok(Self::Random),
            "center_bias" => Ok(Self::CenterBias),
            "orange_targeting" => Ok(Self::OrangeTargeting),
            "bucket_aware" => Ok(Self::BucketAware),
            "human_playtest" => Ok(Self::HumanPlaytest),
            other => Err(anyhow!("unknown aim model {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RogueliteSimOutput {
    pub seed_start: Seed,
    pub runs_per_cohort: usize,
    pub cohorts: Vec<CohortMetrics>,
}

impl RogueliteSimOutput {
    pub fn to_json(&self) -> String {
        let mut json = format!(
            "{{\"mode\":\"roguelite\",\"seed_start\":{},\"runs_per_cohort\":{},\"cohorts\":[",
            self.seed_start, self.runs_per_cohort
        );
        for (index, cohort) in self.cohorts.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push_str(&cohort.to_json());
        }
        json.push_str("]}");
        json
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CohortMetrics {
    pub cohort_id: String,
    pub aim_model: AimModel,
    pub runs: usize,
    pub full_run_wins: u32,
    pub act_started: BTreeMap<u8, u32>,
    pub act_survival_clears: BTreeMap<u8, u32>,
    pub perfect_act_clears: BTreeMap<u8, u32>,
    pub boards_played: u32,
    pub boards_cleared: u32,
    pub oranges_cleared: u32,
    pub shots_fired: u32,
    pub bucket_catches: u32,
    pub score_free_balls: u32,
    pub starting_balls_samples: Vec<u32>,
    pub reward_choices: BTreeMap<String, u32>,
}

impl CohortMetrics {
    fn new(target: &CohortTarget, runs: usize) -> Self {
        Self {
            cohort_id: target.id.clone(),
            aim_model: target.aim_model,
            runs,
            full_run_wins: 0,
            act_started: BTreeMap::new(),
            act_survival_clears: BTreeMap::new(),
            perfect_act_clears: BTreeMap::new(),
            boards_played: 0,
            boards_cleared: 0,
            oranges_cleared: 0,
            shots_fired: 0,
            bucket_catches: 0,
            score_free_balls: 0,
            starting_balls_samples: Vec::new(),
            reward_choices: BTreeMap::new(),
        }
    }

    fn record(&mut self, run: &RunResult) {
        self.full_run_wins += u32::from(run.full_run_won);
        self.boards_played += run.boards_played;
        self.boards_cleared += run.boards_cleared;
        self.oranges_cleared += run.oranges_cleared;
        self.shots_fired += run.shots_fired;
        self.bucket_catches += run.bucket_catches;
        self.score_free_balls += run.score_free_balls;
        self.starting_balls_samples
            .extend(run.starting_balls_samples.iter().copied());
        for act in &run.acts_started {
            *self.act_started.entry(*act).or_default() += 1;
        }
        for act in &run.act_survival_clears {
            *self.act_survival_clears.entry(*act).or_default() += 1;
        }
        for act in &run.perfect_act_clears {
            *self.perfect_act_clears.entry(*act).or_default() += 1;
        }
        for reward in &run.reward_choices {
            *self.reward_choices.entry(reward.clone()).or_default() += 1;
        }
    }

    fn to_json(&self) -> String {
        format!("{{\"id\":\"{}\",\"aim_model\":\"{:?}\",\"runs\":{},\"full_run_wins\":{},\"act_started\":{},\"act_survival_clears\":{},\"perfect_act_clears\":{},\"boards_played\":{},\"boards_cleared\":{},\"oranges_cleared\":{},\"shots_fired\":{},\"bucket_catches\":{},\"score_free_balls\":{},\"reward_choices\":{}}}", json_escape(&self.cohort_id), self.aim_model, self.runs, self.full_run_wins, json_u8_map(&self.act_started), json_u8_map(&self.act_survival_clears), json_u8_map(&self.perfect_act_clears), self.boards_played, self.boards_cleared, self.oranges_cleared, self.shots_fired, self.bucket_catches, self.score_free_balls, json_string_map(&self.reward_choices))
    }
}

pub fn simulate_roguelite(
    tables: &RogueliteBalanceTables,
    runs: usize,
    seed_start: Seed,
) -> RogueliteSimOutput {
    let runs = if runs == 0 {
        tables.sim_targets.default_runs
    } else {
        runs
    };
    let mut cohorts = Vec::new();
    for target in &tables.sim_targets.cohorts {
        if matches!(target.aim_model, AimModel::HumanPlaytest) {
            continue;
        }
        let mut metrics = CohortMetrics::new(target, runs);
        for run_index in 0..runs {
            let seed = seed_start
                .wrapping_add((run_index as Seed).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                .wrapping_add(stable_hash(&target.id));
            metrics.record(&simulate_run(tables, target, seed));
        }
        cohorts.push(metrics);
    }
    RogueliteSimOutput {
        seed_start,
        runs_per_cohort: runs,
        cohorts,
    }
}

#[derive(Default)]
struct RunResult {
    full_run_won: bool,
    acts_started: Vec<u8>,
    act_survival_clears: Vec<u8>,
    perfect_act_clears: Vec<u8>,
    boards_played: u32,
    boards_cleared: u32,
    oranges_cleared: u32,
    shots_fired: u32,
    bucket_catches: u32,
    score_free_balls: u32,
    starting_balls_samples: Vec<u32>,
    reward_choices: Vec<String>,
}

fn simulate_run(tables: &RogueliteBalanceTables, target: &CohortTarget, seed: Seed) -> RunResult {
    let mut rng = Lcg::new(seed);
    let mut hearts = 3u32;
    let mut bonus_balls = 0u32;
    let mut relics = 0u32;
    let mut result = RunResult::default();
    for act in 1..=tables.sim_targets.acts.max(ACTS_FALLBACK) {
        let curve = tables.board_curve.act(act);
        result.acts_started.push(act);
        let mut perfect = true;
        for board_index in 0..curve.boards {
            let is_boss = board_index + 1 == curve.boards;
            let board_seed = rng.next_u64();
            let board = generated_board(curve, board_index, is_boss, board_seed);
            let starting_balls = curve.starting_balls + bonus_balls.min(3);
            let board_result =
                simulate_board(&board, board_seed, starting_balls, target, tables, &mut rng);
            result.boards_played += 1;
            result.oranges_cleared += board_result.oranges_cleared;
            result.shots_fired += board_result.shots_fired;
            result.bucket_catches += board_result.bucket_catches;
            result.score_free_balls += board_result.score_free_balls;
            result.starting_balls_samples.push(starting_balls);
            if board_result.won {
                result.boards_cleared += 1;
                let reward = choose_reward(&tables.reward_pool, is_boss, relics, &mut rng);
                result.reward_choices.push(reward.label.clone());
                match reward.kind {
                    RewardKind::Relic => relics += 1,
                    RewardKind::Ball => bonus_balls += 1,
                    RewardKind::Heal => hearts += 1,
                    RewardKind::Coins => {}
                }
            } else {
                hearts = hearts.saturating_sub(1);
                perfect = false;
                if hearts == 0 {
                    return result;
                }
            }
        }
        result.act_survival_clears.push(act);
        if perfect {
            result.perfect_act_clears.push(act);
        }
    }
    result.full_run_won = true;
    result
}

struct BoardResult {
    won: bool,
    oranges_cleared: u32,
    shots_fired: u32,
    bucket_catches: u32,
    score_free_balls: u32,
}

fn simulate_board(
    board: &BoardDefinition,
    seed: Seed,
    starting_balls: u32,
    target: &CohortTarget,
    tables: &RogueliteBalanceTables,
    rng: &mut Lcg,
) -> BoardResult {
    let mut board = board.clone();
    let mut shots = starting_balls;
    let mut fired = 0;
    let mut oranges_cleared = 0;
    let mut bucket_catches = 0;
    let mut score_free_balls = 0;
    let mut score = 0u32;
    let ball_id = BallId::new("balls/basic").expect("static id is valid");
    while shots > 0 && orange_count(&board) > 0 {
        shots -= 1;
        fired += 1;
        let input = ShotInput {
            aim_angle_radians: aim_for(&board, target, rng),
            launch_speed: launch_speed_for(target, rng),
            ball_id: ball_id.clone(),
        };
        let before = board.pegs.clone();
        let shot = simulate_shot(seed.wrapping_add(fired as Seed), &board, &input);
        let hit_count = shot.summary.pegs_hit.len() as u32;
        oranges_cleared += shot
            .summary
            .pegs_hit
            .iter()
            .filter(|peg| {
                before
                    .iter()
                    .any(|candidate| candidate.id == **peg && candidate.kind == PegKind::Orange)
            })
            .count() as u32;
        score = score.saturating_add(
            ((hit_count * 1_000) as f64 * tables.scoring_curve.combo_multiplier(hit_count)) as u32,
        );
        let earned = tables.scoring_curve.free_balls_for_score(score);
        if earned > score_free_balls {
            shots += earned - score_free_balls;
            score_free_balls = earned;
        }
        if shot.summary.caught_bucket {
            bucket_catches += 1;
            shots += 1;
        }
        board.pegs = shot.remaining_pegs;
    }
    BoardResult {
        won: orange_count(&board) == 0,
        oranges_cleared,
        shots_fired: fired,
        bucket_catches,
        score_free_balls,
    }
}

fn generated_board(
    curve: &ActCurve,
    board_index: u32,
    is_boss: bool,
    seed: Seed,
) -> BoardDefinition {
    let archetypes = [
        "fan", "wave", "clusters", "lanes", "spiral", "rings", "fortress",
    ];
    let archetype = archetypes[(seed as usize + board_index as usize) % archetypes.len()];
    let orange_budget =
        curve.orange_min + (seed as u32 % (curve.orange_max - curve.orange_min + 1));
    let bonus = if is_boss { curve.boss_orange_bonus } else { 0 };
    generate_board(&GenerationParams {
        act: curve.act,
        difficulty: curve.act + board_index as u8,
        archetype: ContentId::new(format!("archetypes/{archetype}")).unwrap(),
        seed,
        peg_budget: (orange_budget + bonus + 22).min(u16::MAX as u32) as u16,
        hazard_budget: u16::from(curve.act.saturating_sub(1)) + u16::from(is_boss),
    })
}

#[derive(Clone, Debug)]
struct ChosenReward {
    label: String,
    kind: RewardKind,
}
#[derive(Clone, Copy, Debug)]
enum RewardKind {
    Relic,
    Ball,
    Heal,
    Coins,
}

fn choose_reward(pool: &RewardPool, boss: bool, relics: u32, rng: &mut Lcg) -> ChosenReward {
    let relic_weight = *pool.reward_type_weights.get("relic").unwrap_or(&62) as f64 / 100.0;
    if boss
        || relics < pool.guaranteed_relic_until
        || rng.next_unit() < pool.board_clear_relic_drop_rate.max(relic_weight)
    {
        return ChosenReward {
            label: if boss {
                "relic:boss_feverheart"
            } else {
                "relic:weighted_pool"
            }
            .to_string(),
            kind: RewardKind::Relic,
        };
    }
    let roll = rng.next_unit();
    if roll < 0.45 {
        ChosenReward {
            label: "ball:extra_orb".to_string(),
            kind: RewardKind::Ball,
        }
    } else if roll < 0.70 {
        ChosenReward {
            label: "heal:heart".to_string(),
            kind: RewardKind::Heal,
        }
    } else {
        ChosenReward {
            label: "coins:payout".to_string(),
            kind: RewardKind::Coins,
        }
    }
}

fn aim_for(board: &BoardDefinition, target: &CohortTarget, rng: &mut Lcg) -> f64 {
    match target.aim_model {
        AimModel::Random => std::f64::consts::FRAC_PI_2 - (-0.62 + rng.next_unit() * 1.24),
        AimModel::CenterBias => std::f64::consts::FRAC_PI_2 - (-0.22 + rng.next_unit() * 0.44),
        AimModel::OrangeTargeting | AimModel::BucketAware | AimModel::HumanPlaytest => {
            let peg = best_orange(board, target.aim_model == AimModel::BucketAware)
                .unwrap_or(Vec2 { x: 0.0, y: 0.0 });
            let launcher = Vec2 { x: 0.0, y: -5.0 };
            (peg.y - launcher.y).atan2(peg.x - launcher.x)
                + if target.aim_model == AimModel::BucketAware {
                    -0.04
                } else {
                    0.0
                }
                + (rng.next_unit() - 0.5) * 0.16
        }
    }
}

fn best_orange(board: &BoardDefinition, prefer_bucket_lane: bool) -> Option<Vec2> {
    board
        .pegs
        .iter()
        .filter(|peg| peg.kind == PegKind::Orange)
        .min_by(|a, b| {
            orange_score(a, prefer_bucket_lane).total_cmp(&orange_score(b, prefer_bucket_lane))
        })
        .map(peg_center)
}

fn orange_score(peg: &PegDef, prefer_bucket_lane: bool) -> f64 {
    let position = peg_center(peg);
    let center = position.x.abs();
    let height = position.y;
    if prefer_bucket_lane {
        center * 0.75 + height * 0.25
    } else {
        center + height * 0.1
    }
}

fn peg_center(peg: &PegDef) -> Vec2 {
    match &peg.shape {
        ShapeDef::Circle { center, .. } | ShapeDef::Rect { center, .. } => *center,
        ShapeDef::Capsule { a, b, .. } | ShapeDef::Segment { a, b } => Vec2 {
            x: (a.x + b.x) * 0.5,
            y: (a.y + b.y) * 0.5,
        },
    }
}

fn launch_speed_for(target: &CohortTarget, rng: &mut Lcg) -> f64 {
    let base = if target.bucket_awareness { 17.4 } else { 16.5 };
    base + rng.next_unit() * if target.orange_targeting { 1.2 } else { 2.5 }
}

fn orange_count(board: &BoardDefinition) -> u32 {
    board
        .pegs
        .iter()
        .filter(|peg| peg.kind == PegKind::Orange)
        .count() as u32
}

#[derive(Clone, Copy)]
struct Lcg(u64);
impl Lcg {
    fn new(seed: Seed) -> Self {
        Self(seed ^ 0xa076_1d64_78bd_642f)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn next_unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

fn stable_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x1000_0000_01b3)
    })
}
fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("").trim()
}

fn array_tables<'a>(src: &'a str, name: &str) -> Vec<&'a str> {
    let marker = format!("[[{name}]]");
    let mut tables = Vec::new();
    let mut start = None;
    for (offset, _) in src.match_indices("[[") {
        if let Some(current) = start.take() {
            tables.push(&src[current..offset]);
        }
        if src[offset..].starts_with(&marker) {
            start = Some(offset + marker.len());
        }
    }
    if let Some(current) = start {
        tables.push(&src[current..]);
    }
    tables
}

fn section<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("[{name}]");
    let start = src.find(&marker)? + marker.len();
    let end = src[start..]
        .find("\n[")
        .map(|relative| start + relative)
        .unwrap_or(src.len());
    Some(&src[start..end])
}

fn parse_u32_section(src: &str, name: &str) -> BTreeMap<String, u32> {
    section(src, name)
        .unwrap_or("")
        .lines()
        .filter_map(|line| {
            let (key, value) = strip_comment(line).split_once('=')?;
            Some((key.trim().to_string(), value.trim().parse().ok()?))
        })
        .collect()
}

fn parse_section_u32(src: &str, section_name: &str, key: &str) -> Option<u32> {
    parse_u32(section(src, section_name)?, key)
}
fn parse_section_u64(src: &str, section_name: &str, key: &str) -> Option<u64> {
    parse_u64(section(src, section_name)?, key)
}
fn parse_section_f64(src: &str, section_name: &str, key: &str) -> Option<f64> {
    parse_f64(section(src, section_name)?, key)
}
fn parse_required_u8(src: &str, key: &str) -> Result<u8> {
    Ok(parse_required_u32(src, key)?.try_into()?)
}
fn parse_required_u32(src: &str, key: &str) -> Result<u32> {
    parse_u32(src, key).with_context(|| format!("missing u32 key {key}"))
}
fn parse_required_f64(src: &str, key: &str) -> Result<f64> {
    parse_f64(src, key).with_context(|| format!("missing f64 key {key}"))
}
fn parse_required_bool(src: &str, key: &str) -> Result<bool> {
    parse_bool(src, key).with_context(|| format!("missing bool key {key}"))
}
fn parse_required_string(src: &str, key: &str) -> Result<String> {
    parse_string(src, key).with_context(|| format!("missing string key {key}"))
}
fn parse_u32(src: &str, key: &str) -> Option<u32> {
    parse_value(src, key)?.parse().ok()
}
fn parse_u64(src: &str, key: &str) -> Option<u64> {
    parse_value(src, key)?.parse().ok()
}
fn parse_f64(src: &str, key: &str) -> Option<f64> {
    parse_value(src, key)?.parse().ok()
}
fn parse_bool(src: &str, key: &str) -> Option<bool> {
    parse_value(src, key)?.parse().ok()
}
fn parse_string(src: &str, key: &str) -> Option<String> {
    Some(parse_value(src, key)?.trim_matches('"').to_string())
}
fn parse_value<'a>(src: &'a str, key: &str) -> Option<&'a str> {
    src.lines().find_map(|line| {
        let (found, value) = strip_comment(line).split_once('=')?;
        (found.trim() == key).then_some(value.trim())
    })
}

fn parse_optional_range(src: &str, min_key: &str, max_key: &str) -> Result<Option<(f64, f64)>> {
    match (parse_f64(src, min_key), parse_f64(src, max_key)) {
        (Some(min), Some(max)) => Ok(Some((min, max))),
        (None, None) => Ok(None),
        _ => Err(anyhow!("range requires both {min_key} and {max_key}")),
    }
}

fn json_u8_map(map: &BTreeMap<u8, u32>) -> String {
    let mut out = String::from("{");
    for (index, (key, value)) in map.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        let _ = write!(out, "\"{}\":{}", key, value);
    }
    out.push('}');
    out
}
fn json_string_map(map: &BTreeMap<String, u32>) -> String {
    let mut out = String::from("{");
    for (index, (key, value)) in map.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        let _ = write!(out, "\"{}\":{}", json_escape(key), value);
    }
    out.push('}');
    out
}
fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_tables() -> RogueliteBalanceTables {
        RogueliteBalanceTables::from_toml_strings(
            include_str!("../../../content/balance/roguelite/board_curve.toml"),
            include_str!("../../../content/balance/roguelite/reward_pool.toml"),
            include_str!("../../../content/balance/roguelite/scoring_curve.toml"),
            include_str!("../../../content/balance/roguelite/sim_targets.toml"),
        )
        .unwrap()
    }

    #[test]
    fn roguelite_balance_tables_parse() {
        let tables = fixture_tables();
        assert_eq!(tables.board_curve.act(1).starting_balls, 12);
        assert!(tables.reward_pool.board_clear_relic_drop_rate >= 0.62);
        assert!(tables
            .sim_targets
            .cohorts
            .iter()
            .any(|cohort| cohort.id == "bucket_aware_base"));
    }

    #[test]
    fn roguelite_sim_seed_is_deterministic() {
        let tables = fixture_tables();
        assert_eq!(
            simulate_roguelite(&tables, 4, 99).to_json(),
            simulate_roguelite(&tables, 4, 99).to_json()
        );
    }

    #[test]
    fn roguelite_metrics_separate_act_survival_and_perfect_clear() {
        let tables = fixture_tables();
        let output = simulate_roguelite(&tables, 3, 7);
        let metrics = output.cohorts.first().unwrap();
        assert!(metrics
            .act_survival_clears
            .keys()
            .all(|act| metrics.act_started.contains_key(act)));
        assert!(metrics
            .perfect_act_clears
            .keys()
            .all(|act| metrics.act_survival_clears.contains_key(act)));
    }

    #[test]
    fn roguelite_sim_uses_starting_balls_from_toml() {
        let mut tables = fixture_tables();
        tables.board_curve.acts[0].starting_balls = 17;
        let output = simulate_roguelite(&tables, 1, 42);
        assert!(output.cohorts[0].starting_balls_samples.contains(&17));
    }

    #[test]
    fn roguelite_smoke_sim_emits_json() {
        let tables = fixture_tables();
        let json = simulate_roguelite(&tables, 1, 123).to_json();
        assert!(json.starts_with("{\"mode\":\"roguelite\""));
        assert!(json.contains("\"cohorts\""));
    }
}
