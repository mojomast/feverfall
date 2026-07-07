#![no_main]

mod common;

use arbitrary::Arbitrary;
use common::{finite_event_positions, FuzzShot};
use content_schema::BoardDefinition;
use libfuzzer_sys::fuzz_target;
use physics_core::{sample_shot_trajectory, simulate_shot};

const AUTHORED_BOARDS: &[&str] = &[
    include_str!("../../game/assets/content/boards/act1_boss_01/board.json"),
    include_str!("../../game/assets/content/boards/c4_rpg_ch1/board_01.json"),
    include_str!("../../game/assets/content/boards/c4_rpg_ch3/board_02.json"),
];

#[derive(Debug, Arbitrary)]
struct Input {
    board_index: u8,
    shot: FuzzShot,
}

fuzz_target!(|input: Input| {
    let json = AUTHORED_BOARDS[input.board_index as usize % AUTHORED_BOARDS.len()];
    let board: BoardDefinition = serde_json::from_str(json).expect("authored board fixture parses");
    let shot = input.shot.input();

    let result = simulate_shot(input.shot.seed, &board, &shot);
    assert_eq!(
        result.events.last(),
        Some(&physics_core::PhysicsEvent::ShotEnded {
            summary: result.summary.clone()
        })
    );
    assert!(result.events.iter().all(finite_event_positions));

    let samples = sample_shot_trajectory(&board, &shot, input.shot.sample_every_ticks());
    assert!(samples
        .iter()
        .all(|sample| common::finite_vec(sample.position)));
});
