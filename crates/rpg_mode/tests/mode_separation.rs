use content_schema::{minimal_test_board, BallId, ContentId};
use feedback_events::map_game_event;
use game_rules::{promote_physics_event, GameEvent};
use physics_core::{simulate_shot, ShotInput};
use rpg_mode::{CharacterState, RPG_BALANCE_DIR, RPG_SAVE_DIR};
use run_mode::RunState;
use telemetry::{game_event_to_telemetry, physics_event_to_telemetry, shot_summary_to_telemetry};

#[test]
fn rpg_character_state_processes_board_outcome_without_mutating_run_state() {
    let board = minimal_test_board();
    let input = ShotInput {
        aim_angle_radians: std::f64::consts::FRAC_PI_2,
        launch_speed: 17.5,
        ball_id: BallId::new("balls/basic").unwrap(),
    };
    let run = RunState::act1_slice(0xC35E_u64);
    let original_run = run.clone();
    let mut character = CharacterState::act1_slice();

    let result = simulate_shot(run.rng_state, &board, &input);
    let promoted_events: Vec<GameEvent> = result
        .events
        .iter()
        .cloned()
        .map(promote_physics_event)
        .collect();
    let board_won = GameEvent::BoardWon {
        board: board.id.clone(),
        final_score: 1_000,
    };
    let feedback = map_game_event(&board_won);
    let physics_telemetry_count = result
        .events
        .iter()
        .filter_map(physics_event_to_telemetry)
        .count();

    character.xp += 25;
    character.level = 2;
    character
        .campaign_flags
        .push(ContentId::new("campaign/chapter_1/board_01_cleared").unwrap());

    let json = serde_json::to_string(&character).unwrap();
    let parsed: CharacterState = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed, character);
    assert_eq!(run, original_run);
    assert_ne!(run.run_id.as_str(), parsed.character_id.as_str());
    assert_eq!(RPG_SAVE_DIR, "saves/rpg/");
    assert_eq!(RPG_BALANCE_DIR, "content/balance/rpg/");
    assert!(!promoted_events.is_empty());
    assert!(feedback.is_some());
    assert!(physics_telemetry_count > 0);
    assert!(matches!(
        shot_summary_to_telemetry(board.id, 0, &result.summary),
        telemetry::TelemetryEvent::ShotResolved { .. }
    ));
    assert!(game_event_to_telemetry(&board_won).is_some());
}

#[test]
fn physics_core_has_no_mode_crate_dependencies_or_imports() {
    let manifest = include_str!("../../physics_core/Cargo.toml");
    let source = include_str!("../../physics_core/src/lib.rs");

    assert!(!manifest.contains("run_mode"));
    assert!(!manifest.contains("rpg_mode"));
    assert!(!source.contains("run_mode::"));
    assert!(!source.contains("rpg_mode::"));
}
