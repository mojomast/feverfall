use anyhow::{anyhow, Context, Result};
use content_schema::BoardDefinition;
use physics_core::{PhysicsEvent, ShotInput};
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

const DEFAULT_REPLAY: &str = "tests/golden_replays/minimal_test.replay.json";

#[derive(Debug, Deserialize)]
struct ReplayFixture {
    name: String,
    board: BoardDefinition,
    #[serde(default)]
    seed: u64,
    shots: Vec<ShotInput>,
    expected_hash: Option<String>,
    #[serde(default)]
    pending_simulator: bool,
}

fn main() -> Result<()> {
    let path = replay_path()?;
    let fixture = load_fixture(&path)?;
    let events = run_replay(&fixture);
    let replay_hash = physics_core::stable_hash_events(&events);

    println!(
        "replay {} board={} shots={} hash={replay_hash}",
        fixture.name,
        fixture.board.id,
        fixture.shots.len()
    );
    println!(
        "rules={} physics={} mode=simulate_shot",
        game_rules::RULESET_VERSION,
        physics_core::PHYSICS_VERSION
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

fn run_replay(fixture: &ReplayFixture) -> Vec<PhysicsEvent> {
    let mut board = fixture.board.clone();
    let mut events = Vec::new();
    for (index, shot) in fixture.shots.iter().enumerate() {
        let result = physics_core::simulate_shot(fixture.seed + index as u64, &board, shot);
        board.pegs = result.remaining_pegs;
        events.extend(result.events);
    }
    events
}
