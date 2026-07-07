#![no_main]

mod common;

use arbitrary::Arbitrary;
use common::{finite_event_positions, FuzzBoard, FuzzShot};
use libfuzzer_sys::fuzz_target;
use physics_core::simulate_shot;

#[derive(Debug, Arbitrary)]
struct ReplaySequence {
    board: FuzzBoard,
    shots: [FuzzShot; 6],
}

fuzz_target!(|sequence: ReplaySequence| {
    let mut board = sequence.board.board_definition("multi_shot");

    for shot in sequence.shots {
        let input = shot.input();
        let first = simulate_shot(shot.seed, &board, &input);
        let second = simulate_shot(shot.seed, &board, &input);

        assert_eq!(first.summary.replay_hash, second.summary.replay_hash);
        assert_eq!(first.summary.ticks, second.summary.ticks);
        assert!(first.events.iter().all(finite_event_positions));

        board.pegs = first.remaining_pegs;
    }
});
