#![no_main]

mod common;

use common::{first_collision_signature, FuzzCase};
use libfuzzer_sys::fuzz_target;
use physics_core::{predict_first_bounce, simulate_shot};

fuzz_target!(|case: FuzzCase| {
    let board = case.board.board_definition("first_bounce");
    let input = case.shot.input();

    let predicted = predict_first_bounce(&board, &input)
        .as_ref()
        .and_then(first_collision_signature);
    let simulated = simulate_shot(case.shot.seed, &board, &input)
        .events
        .iter()
        .find_map(first_collision_signature);

    assert_eq!(predicted, simulated);
});
