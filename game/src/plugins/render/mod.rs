use content_schema::{BoardDefinition, Vec2};

use crate::plugins::debug::DebugOverlayState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderRegistrationSummary {
    pub board_primitives: usize,
}

pub fn register() -> RenderRegistrationSummary {
    RenderRegistrationSummary {
        board_primitives: 0,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeelTestRenderState {
    pub board_size: Vec2,
    pub cannon_position: Vec2,
    pub peg_primitives: usize,
    pub obstacle_primitives: usize,
    pub aim_line_available: bool,
    pub event_markers: usize,
}

impl FeelTestRenderState {
    pub fn from_board_and_debug(board: &BoardDefinition, debug: &DebugOverlayState) -> Self {
        Self {
            board_size: board.size,
            cannon_position: board.cannon_position,
            peg_primitives: board.pegs.len(),
            obstacle_primitives: board.obstacles.len(),
            aim_line_available: debug.aim.first_bounce.is_some(),
            event_markers: debug.collision_events.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{minimal_test_board, BallId};
    use physics_core::{simulate_shot, ShotInput};

    use crate::plugins::debug::DebugOverlayState;

    #[test]
    fn render_state_reuses_board_and_debug_models() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let result = simulate_shot(123, &board, &input);
        let debug = DebugOverlayState::mock_from_board_and_input(
            &board,
            &input,
            Some(result.summary.replay_hash),
            result.events,
        );

        let render = FeelTestRenderState::from_board_and_debug(&board, &debug);

        assert_eq!(render.board_size, board.size);
        assert_eq!(render.peg_primitives, board.pegs.len());
        assert!(render.aim_line_available);
        assert_eq!(render.event_markers, debug.collision_events.len());
    }
}
