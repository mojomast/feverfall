use anyhow::Result;
use board_gen::{generate_board, GenerationParams};
use content_schema::{BallId, BoardDefinition, ContentId, PegKind, Seed};
use physics_core::{simulate_shot, ShotInput};
use std::collections::BTreeMap;

const RUNS: usize = 1_000;
const START_SEED: Seed = 0xC3BA_0000_0000_0000;
const BOARDS_PER_ACT: u32 = 3;
const ACTS: u8 = 3;

fn main() -> Result<()> {
    let mut aggregate = Aggregate::default();

    for index in 0..RUNS {
        let seed = START_SEED + index as Seed;
        let run = simulate_run(seed);
        aggregate.record(&run);
    }

    println!(
        "balance_sim runs={RUNS} seed_start={START_SEED:#018x} seed_end={:#018x}",
        START_SEED + RUNS as Seed - 1
    );
    for act in 1..=ACTS {
        let started = aggregate.act_started[act as usize];
        let cleared = aggregate.act_cleared[act as usize];
        let win_rate = if started == 0 {
            0.0
        } else {
            cleared as f64 / started as f64
        };
        println!("act_{act}_win_rate={win_rate:.3} started={started} cleared={cleared}");
    }
    println!(
        "avg_oranges_cleared_per_board={:.2}",
        aggregate.oranges_cleared as f64 / aggregate.boards_played as f64
    );
    println!(
        "avg_relics_collected={:.2}",
        aggregate.relics_collected as f64 / RUNS as f64
    );
    println!(
        "avg_run_length_shots={:.2}",
        aggregate.shots_fired as f64 / RUNS as f64
    );
    println!("boards_played={}", aggregate.boards_played);
    println!("full_run_wins={}", aggregate.full_run_wins);
    println!("reward_choices={:?}", aggregate.reward_choices);

    Ok(())
}

#[derive(Default)]
struct Aggregate {
    act_started: [u32; 4],
    act_cleared: [u32; 4],
    boards_played: u32,
    oranges_cleared: u32,
    relics_collected: u32,
    shots_fired: u32,
    full_run_wins: u32,
    reward_choices: BTreeMap<&'static str, u32>,
}

impl Aggregate {
    fn record(&mut self, run: &RunResult) {
        for act in 1..=ACTS {
            if run.acts_started >= act {
                self.act_started[act as usize] += 1;
            }
            if run.acts_cleared >= act {
                self.act_cleared[act as usize] += 1;
            }
        }
        self.boards_played += run.boards_played;
        self.oranges_cleared += run.oranges_cleared;
        self.relics_collected += run.relics_collected;
        self.shots_fired += run.shots_fired;
        self.full_run_wins += u32::from(run.acts_cleared == ACTS);
        for reward in &run.reward_choices {
            *self.reward_choices.entry(reward).or_default() += 1;
        }
    }
}

struct RunResult {
    acts_started: u8,
    acts_cleared: u8,
    boards_played: u32,
    oranges_cleared: u32,
    relics_collected: u32,
    shots_fired: u32,
    reward_choices: Vec<&'static str>,
}

fn simulate_run(seed: Seed) -> RunResult {
    let mut rng = Lcg::new(seed);
    let mut hearts = 3u32;
    let mut relics = 0u32;
    let mut balls_bonus = 0u32;
    let mut result = RunResult {
        acts_started: 0,
        acts_cleared: 0,
        boards_played: 0,
        oranges_cleared: 0,
        relics_collected: 0,
        shots_fired: 0,
        reward_choices: Vec::new(),
    };

    for act in 1..=ACTS {
        result.acts_started = act;
        let mut act_cleared = true;
        for board_index in 0..BOARDS_PER_ACT {
            let board_seed = rng.next_u64();
            let board = generated_board(act, board_index, board_seed);
            let board_result = simulate_board(&board, board_seed, act, balls_bonus, &mut rng);
            result.boards_played += 1;
            result.oranges_cleared += board_result.oranges_cleared;
            result.shots_fired += board_result.shots_fired;
            if !board_result.won {
                hearts = hearts.saturating_sub(1);
                act_cleared = false;
                if hearts == 0 {
                    return result;
                }
            } else {
                let reward = choose_reward(act, board_index, relics, &mut rng);
                result.reward_choices.push(reward.name);
                match reward.reward {
                    SimReward::Relic => {
                        relics += 1;
                        result.relics_collected += 1;
                    }
                    SimReward::Ball => balls_bonus += 1,
                    SimReward::Heal(amount) => hearts += amount,
                    SimReward::Coins => {}
                }
            }
        }
        if act_cleared {
            result.acts_cleared = act;
        }
    }

    result
}

struct BoardResult {
    won: bool,
    oranges_cleared: u32,
    shots_fired: u32,
}

fn simulate_board(
    board: &BoardDefinition,
    seed: Seed,
    act: u8,
    balls_bonus: u32,
    rng: &mut Lcg,
) -> BoardResult {
    let mut board = board.clone();
    let mut shots = (11u32.saturating_sub(u32::from(act))).max(7) + balls_bonus.min(3);
    let mut fired = 0;
    let mut oranges = 0;
    let ball_id = BallId::new("balls/basic").expect("static id is valid");

    while shots > 0 && orange_count(&board) > 0 {
        shots -= 1;
        fired += 1;
        let aim = random_aim(rng);
        let input = ShotInput {
            aim_angle_radians: aim,
            launch_speed: 16.5 + rng.next_unit() * 2.5,
            ball_id: ball_id.clone(),
        };
        let shot = simulate_shot(seed.wrapping_add(fired as Seed), &board, &input);
        oranges += shot
            .summary
            .pegs_hit
            .iter()
            .filter(|peg| {
                board
                    .pegs
                    .iter()
                    .any(|candidate| candidate.id == **peg && candidate.kind == PegKind::Orange)
            })
            .count() as u32;
        if shot.summary.caught_bucket {
            shots += 1;
        }
        board.pegs = shot.remaining_pegs;
    }

    BoardResult {
        won: orange_count(&board) == 0,
        oranges_cleared: oranges,
        shots_fired: fired,
    }
}

struct ChosenReward {
    name: &'static str,
    reward: SimReward,
}

enum SimReward {
    Relic,
    Ball,
    Heal(u32),
    Coins,
}

fn choose_reward(act: u8, board_index: u32, relics: u32, rng: &mut Lcg) -> ChosenReward {
    let rarity = if board_index == BOARDS_PER_ACT - 1 {
        SimRarity::Boss
    } else if rng.next_unit() < 0.18 {
        SimRarity::Rare
    } else if rng.next_unit() < 0.43 {
        SimRarity::Uncommon
    } else {
        SimRarity::Common
    };
    let roll = rng.next_unit();
    if relics < 2 || matches!(rarity, SimRarity::Rare | SimRarity::Boss) && roll < 0.62 {
        let name = match (act, rarity) {
            (_, SimRarity::Boss) => "relic:boss_feverheart",
            (_, SimRarity::Rare) => "relic:rare_orange_echo",
            (_, SimRarity::Uncommon) => "relic:uncommon_steady_bucket",
            _ => "relic:common_spark_catcher",
        };
        ChosenReward {
            name,
            reward: SimReward::Relic,
        }
    } else if roll < 0.78 {
        ChosenReward {
            name: "ball:extra_orb",
            reward: SimReward::Ball,
        }
    } else if roll < 0.9 {
        ChosenReward {
            name: "heal:heart",
            reward: SimReward::Heal(1),
        }
    } else {
        ChosenReward {
            name: "coins:payout",
            reward: SimReward::Coins,
        }
    }
}

#[derive(Clone, Copy)]
enum SimRarity {
    Common,
    Uncommon,
    Rare,
    Boss,
}

fn generated_board(act: u8, board_index: u32, seed: Seed) -> BoardDefinition {
    let archetypes = [
        "fan", "wave", "clusters", "lanes", "spiral", "rings", "fortress",
    ];
    let archetype = archetypes[(seed as usize + board_index as usize) % archetypes.len()];
    let peg_budget = match act {
        1 => 42,
        2 => 47,
        _ => 52,
    };
    let hazard_budget =
        u16::from(act.saturating_sub(1)) + u16::from(board_index == BOARDS_PER_ACT - 1);
    generate_board(&GenerationParams {
        act,
        difficulty: act + board_index as u8,
        archetype: ContentId::new(format!("archetypes/{archetype}")).unwrap(),
        seed,
        peg_budget,
        hazard_budget,
    })
}

fn orange_count(board: &BoardDefinition) -> u32 {
    board
        .pegs
        .iter()
        .filter(|peg| peg.kind == PegKind::Orange)
        .count() as u32
}

fn random_aim(rng: &mut Lcg) -> f64 {
    let offset = -0.62 + rng.next_unit() * 1.24;
    std::f64::consts::FRAC_PI_2 - offset
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
