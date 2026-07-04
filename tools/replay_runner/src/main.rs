use anyhow::{anyhow, Context, Result};
use content_schema::BoardDefinition;
use physics_core::{PhysicsEvent, ShotInput};
use rpg_mode::CharacterState;
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

const DEFAULT_REPLAY: &str = "tests/golden_replays/minimal_test.replay.json";

#[derive(Debug, Deserialize)]
struct ReplayFixture {
    name: String,
    board: Option<BoardDefinition>,
    board_path: Option<PathBuf>,
    boards: Option<Vec<ReplayBoardFixture>>,
    #[serde(default)]
    seed: u64,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    character_state: Option<CharacterState>,
    #[serde(default)]
    shots: Vec<ShotInput>,
    expected_hash: Option<String>,
    #[serde(default)]
    pending_simulator: bool,
}

#[derive(Debug, Deserialize)]
struct ReplayBoardFixture {
    board_path: PathBuf,
    #[serde(default)]
    character_state: Option<CharacterState>,
    shots: Vec<ShotInput>,
}

fn main() -> Result<()> {
    let path = replay_path()?;
    let fixture = load_fixture(&path)?;
    let replay = run_replay(&fixture)?;
    let events = replay.events;
    let replay_hash = physics_core::stable_hash_events(&events);
    let character_snapshots = fixture.character_snapshot_count();

    println!(
        "replay {} boards={} shots={} character_snapshots={} hash={replay_hash}",
        fixture.name, replay.board_count, replay.shot_count, character_snapshots
    );
    println!(
        "rules={} physics={} mode={}",
        game_rules::RULESET_VERSION,
        physics_core::PHYSICS_VERSION,
        fixture.mode.as_deref().unwrap_or("simulate_shot")
    );

    match fixture.expected_hash {
        Some(expected) if expected == replay_hash => {
            println!("golden hash match: {expected}");
            Ok(())
        }
        Some(expected) => Err(anyhow!(
            "golden hash mismatch for {}: expected {}, got {}",
            path.display(),
            expected,
            replay_hash
        )),
        None if fixture.pending_simulator => {
            println!(
                "golden hash pending: set expected_hash to the emitted hash once this replay is accepted"
            );
            Ok(())
        }
        None => Err(anyhow!(
            "{} has no expected_hash and is not marked pending_simulator",
            path.display()
        )),
    }
}

impl ReplayFixture {
    fn character_snapshot_count(&self) -> usize {
        usize::from(self.character_state.is_some())
            + self
                .boards
                .as_deref()
                .unwrap_or_default()
                .iter()
                .filter(|board| board.character_state.is_some())
                .count()
    }
}

fn replay_path() -> Result<PathBuf> {
    let mut args = env::args().skip(1);
    let mut path = PathBuf::from(DEFAULT_REPLAY);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--replay" | "-r" => {
                path = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow!("{arg} requires a file path"))?;
            }
            "--help" | "-h" => {
                println!("usage: cargo run -p replay_runner -- [--replay FILE]");
                std::process::exit(0);
            }
            _ => return Err(anyhow!("unknown argument {arg}")),
        }
    }

    Ok(path)
}

fn load_fixture(path: &PathBuf) -> Result<ReplayFixture> {
    let json = fs::read_to_string(path)
        .with_context(|| format!("failed to read replay fixture {}", path.display()))?;
    serde_json::from_str(&json)
        .with_context(|| format!("failed to parse replay fixture {}", path.display()))
}

fn load_board(fixture: &ReplayFixture) -> Result<BoardDefinition> {
    match (&fixture.board, &fixture.board_path, &fixture.boards) {
        (_, _, Some(_)) => Err(anyhow!(
            "replay fixture {} with boards must not set top-level board or board_path",
            fixture.name
        )),
        (Some(board), None, None) => Ok(board.clone()),
        (None, Some(path), None) => {
            let json = fs::read_to_string(path)
                .with_context(|| format!("failed to read board fixture {}", path.display()))?;
            serde_json::from_str(&json)
                .with_context(|| format!("failed to parse board fixture {}", path.display()))
        }
        (Some(_), Some(_), None) => Err(anyhow!(
            "replay fixture {} must set either board or board_path, not both",
            fixture.name
        )),
        (None, None, None) => Err(anyhow!(
            "replay fixture {} must set board or board_path",
            fixture.name
        )),
    }
}

struct ReplayRun {
    board_count: usize,
    shot_count: usize,
    events: Vec<PhysicsEvent>,
}

fn run_replay(fixture: &ReplayFixture) -> Result<ReplayRun> {
    if let Some(boards) = &fixture.boards {
        let mut events = Vec::new();
        let mut shot_count = 0;

        for replay_board in boards {
            let json = fs::read_to_string(&replay_board.board_path).with_context(|| {
                format!(
                    "failed to read board fixture {}",
                    replay_board.board_path.display()
                )
            })?;
            let mut board: BoardDefinition = serde_json::from_str(&json).with_context(|| {
                format!(
                    "failed to parse board fixture {}",
                    replay_board.board_path.display()
                )
            })?;
            for shot in &replay_board.shots {
                let result =
                    physics_core::simulate_shot(fixture.seed + shot_count as u64, &board, shot);
                board.pegs = result.remaining_pegs;
                events.extend(result.events);
                shot_count += 1;
            }
        }

        return Ok(ReplayRun {
            board_count: boards.len(),
            shot_count,
            events,
        });
    }

    let board = load_board(fixture)?;
    let mut board = board.clone();
    let mut events = Vec::new();
    for (index, shot) in fixture.shots.iter().enumerate() {
        let result = physics_core::simulate_shot(fixture.seed + index as u64, &board, shot);
        board.pegs = result.remaining_pegs;
        events.extend(result.events);
    }
    Ok(ReplayRun {
        board_count: 1,
        shot_count: fixture.shots.len(),
        events,
    })
}
