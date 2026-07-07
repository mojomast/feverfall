#![no_main]

mod common;

use common::{finite_vec, FuzzCase};
use libfuzzer_sys::fuzz_target;
use physics_core::sample_shot_trajectory;

fuzz_target!(|case: FuzzCase| {
    let board = case.board.board_definition("trajectory");
    let input = case.shot.input();
    let samples = sample_shot_trajectory(&board, &input, case.shot.sample_every_ticks());

    assert!(!samples.is_empty());
    assert_eq!(samples.first().unwrap().tick, 0);
    assert!(samples.iter().all(|sample| finite_vec(sample.position)));
    assert!(samples.windows(2).all(|pair| pair[0].tick < pair[1].tick));
});
