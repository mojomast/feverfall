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
    pub ambience: BoardAmbience,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardAmbience {
    pub archetype: BoardArchetype,
    pub gradient_top: [f32; 3],
    pub gradient_bottom: [f32; 3],
    pub particle_density: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardArchetype {
    Fan,
    Wave,
    Clusters,
    Lanes,
    Spiral,
    Rings,
    Fortress,
    Boss,
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
            ambience: board_ambience(board),
        }
    }
}

pub fn board_ambience(board: &BoardDefinition) -> BoardAmbience {
    match board_archetype(board) {
        BoardArchetype::Fan => BoardAmbience {
            archetype: BoardArchetype::Fan,
            gradient_top: [0.05, 0.18, 0.34],
            gradient_bottom: [0.02, 0.07, 0.16],
            particle_density: 0.35,
        },
        BoardArchetype::Fortress => BoardAmbience {
            archetype: BoardArchetype::Fortress,
            gradient_top: [0.34, 0.09, 0.05],
            gradient_bottom: [0.13, 0.04, 0.03],
            particle_density: 0.22,
        },
        BoardArchetype::Spiral => BoardAmbience {
            archetype: BoardArchetype::Spiral,
            gradient_top: [0.16, 0.06, 0.33],
            gradient_bottom: [0.04, 0.02, 0.12],
            particle_density: 0.45,
        },
        BoardArchetype::Wave => BoardAmbience {
            archetype: BoardArchetype::Wave,
            gradient_top: [0.04, 0.22, 0.28],
            gradient_bottom: [0.02, 0.08, 0.12],
            particle_density: 0.4,
        },
        BoardArchetype::Clusters => BoardAmbience {
            archetype: BoardArchetype::Clusters,
            gradient_top: [0.12, 0.18, 0.11],
            gradient_bottom: [0.04, 0.07, 0.05],
            particle_density: 0.5,
        },
        BoardArchetype::Lanes => BoardAmbience {
            archetype: BoardArchetype::Lanes,
            gradient_top: [0.16, 0.14, 0.08],
            gradient_bottom: [0.06, 0.05, 0.03],
            particle_density: 0.28,
        },
        BoardArchetype::Rings => BoardAmbience {
            archetype: BoardArchetype::Rings,
            gradient_top: [0.12, 0.12, 0.24],
            gradient_bottom: [0.04, 0.04, 0.11],
            particle_density: 0.42,
        },
        BoardArchetype::Boss => BoardAmbience {
            archetype: BoardArchetype::Boss,
            gradient_top: [0.28, 0.06, 0.18],
            gradient_bottom: [0.08, 0.02, 0.06],
            particle_density: 0.6,
        },
    }
}

fn board_archetype(board: &BoardDefinition) -> BoardArchetype {
    let text = board
        .tags
        .iter()
        .map(|tag| tag.as_str())
        .chain(std::iter::once(board.id.as_str()))
        .collect::<Vec<_>>()
        .join(" ");

    if text.contains("fortress") {
        BoardArchetype::Fortress
    } else if text.contains("spiral") {
        BoardArchetype::Spiral
    } else if text.contains("wave") {
        BoardArchetype::Wave
    } else if text.contains("clusters") {
        BoardArchetype::Clusters
    } else if text.contains("lanes") {
        BoardArchetype::Lanes
    } else if text.contains("rings") {
        BoardArchetype::Rings
    } else if text.contains("boss") {
        BoardArchetype::Boss
    } else {
        BoardArchetype::Fan
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
        assert_eq!(render.ambience.archetype, BoardArchetype::Fan);
        assert!(render.ambience.particle_density > 0.0);
    }
}
