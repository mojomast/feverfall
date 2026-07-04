use content_schema::{minimal_test_board, BallId, BoardDefinition, BoardId, Vec2};
use physics_core::{predict_first_bounce, simulate_shot, PhysicsEvent, ShotInput};

use crate::plugins::ui::{AimHudState, CollisionDisplayKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DebugRegistrationSummary {
    pub collision_events: usize,
    pub first_bounce_predicted: bool,
    pub reused_aim_bounce_predicted: bool,
    pub f3_overlay_fields: usize,
}

pub fn register() -> DebugRegistrationSummary {
    let board = minimal_test_board();
    let input = ShotInput {
        aim_angle_radians: std::f64::consts::FRAC_PI_2,
        launch_speed: 17.5,
        ball_id: BallId::new("balls/basic").expect("static id is valid"),
    };
    let result = simulate_shot(123, &board, &input);
    let overlay = DebugOverlayState::mock_from_board_and_input(
        &board,
        &input,
        Some(result.summary.replay_hash),
        result.events,
    );
    let aim_hud = AimHudState::from_board_and_input(&board, &input);
    let reused_aim = DebugAimOverlay::from_aim_hud(&aim_hud);

    let mut f3 = F3DebugOverlay::from_runtime(
        120,
        overlay.replay_hash.clone(),
        overlay.board_id.clone(),
        3,
        overlay.event_log_summary.total_events,
    );
    f3.handle_f3();

    DebugRegistrationSummary {
        collision_events: overlay.collision_events.len(),
        first_bounce_predicted: overlay.aim.first_bounce.is_some(),
        reused_aim_bounce_predicted: reused_aim.first_bounce.is_some(),
        f3_overlay_fields: f3.visible_lines().len(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct F3DebugOverlay {
    pub visible: bool,
    pub physics_tick_rate_hz: u32,
    pub last_replay_hash: Option<String>,
    pub current_board_id: BoardId,
    pub active_relic_count: usize,
    pub telemetry_event_count: usize,
}

impl F3DebugOverlay {
    pub fn from_runtime(
        physics_tick_rate_hz: u32,
        last_replay_hash: Option<String>,
        current_board_id: BoardId,
        active_relic_count: usize,
        telemetry_event_count: usize,
    ) -> Self {
        Self {
            visible: false,
            physics_tick_rate_hz,
            last_replay_hash,
            current_board_id,
            active_relic_count,
            telemetry_event_count,
        }
    }

    pub fn handle_f3(&mut self) {
        self.visible = !self.visible;
    }

    pub fn visible_lines(&self) -> Vec<String> {
        if !self.visible {
            return Vec::new();
        }
        vec![
            format!("physics_tick_rate={}hz", self.physics_tick_rate_hz),
            format!(
                "last_replay_hash={}",
                self.last_replay_hash.as_deref().unwrap_or("<none>")
            ),
            format!("current_board_id={}", self.current_board_id),
            format!("active_relic_count={}", self.active_relic_count),
            format!("telemetry_event_count={}", self.telemetry_event_count),
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebugOverlayState {
    pub board_id: BoardId,
    pub replay_hash: Option<String>,
    pub collision_events: Vec<CollisionEventLogEntry>,
    pub event_log_summary: DebugEventLogSummary,
    pub aim: DebugAimOverlay,
}

impl DebugOverlayState {
    pub fn mock_from_board_and_input(
        board: &BoardDefinition,
        input: &ShotInput,
        replay_hash: impl Into<Option<String>>,
        events: impl IntoIterator<Item = PhysicsEvent>,
    ) -> Self {
        let events = events.into_iter().collect::<Vec<_>>();

        Self {
            board_id: board.id.clone(),
            replay_hash: replay_hash.into(),
            collision_events: events
                .iter()
                .cloned()
                .filter_map(CollisionEventLogEntry::from_physics_event)
                .collect(),
            event_log_summary: DebugEventLogSummary::from_events(&events),
            aim: DebugAimOverlay::from_board_and_input(board, input),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugEventLogSummary {
    pub total_events: usize,
    pub collision_events: usize,
    pub peg_hits: usize,
    pub obstacle_hits: usize,
    pub bucket_entries: usize,
    pub board_exits: usize,
}

impl DebugEventLogSummary {
    pub fn from_events(events: &[PhysicsEvent]) -> Self {
        let mut summary = Self {
            total_events: events.len(),
            collision_events: 0,
            peg_hits: 0,
            obstacle_hits: 0,
            bucket_entries: 0,
            board_exits: 0,
        };

        for event in events {
            match event {
                PhysicsEvent::BallHitPeg { .. } => {
                    summary.collision_events += 1;
                    summary.peg_hits += 1;
                }
                PhysicsEvent::BallHitObstacle { .. } => {
                    summary.collision_events += 1;
                    summary.obstacle_hits += 1;
                }
                PhysicsEvent::BallEnteredBucket { .. } => summary.bucket_entries += 1,
                PhysicsEvent::BallExitedBoard { .. } => summary.board_exits += 1,
                PhysicsEvent::ShotEnded { .. } => {}
            }
        }

        summary
    }

    pub fn display_line(&self) -> String {
        format!(
            "events={}, collisions={}, pegs={}, obstacles={}, bucket={}, exits={}",
            self.total_events,
            self.collision_events,
            self.peg_hits,
            self.obstacle_hits,
            self.bucket_entries,
            self.board_exits
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebugAimOverlay {
    pub ray_origin: Vec2,
    pub ray_direction: Vec2,
    pub first_bounce: Option<DebugFirstBounce>,
}

impl DebugAimOverlay {
    pub fn from_board_and_input(board: &BoardDefinition, input: &ShotInput) -> Self {
        let first_bounce = predict_first_bounce(board, input).map(DebugFirstBounce::from_event);

        Self {
            ray_origin: board.cannon_position,
            ray_direction: Vec2::new(input.aim_angle_radians.cos(), input.aim_angle_radians.sin()),
            first_bounce,
        }
    }

    pub fn from_aim_hud(aim: &AimHudState) -> Self {
        Self {
            ray_origin: aim.origin,
            ray_direction: Vec2::new(aim.aim_angle_radians.cos(), aim.aim_angle_radians.sin()),
            first_bounce: aim.first_bounce.as_ref().map(|bounce| DebugFirstBounce {
                impact: bounce.impact,
                normal: bounce.normal,
                kind: bounce.collision.clone(),
                tick: bounce.tick,
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebugFirstBounce {
    pub impact: Vec2,
    pub normal: Vec2,
    pub kind: CollisionDisplayKind,
    pub tick: u64,
}

impl DebugFirstBounce {
    fn from_event(event: PhysicsEvent) -> Self {
        match event {
            PhysicsEvent::BallHitPeg {
                peg,
                position,
                normal,
                tick,
                ..
            } => Self {
                impact: position,
                normal,
                kind: CollisionDisplayKind::Peg(peg.to_string()),
                tick,
            },
            PhysicsEvent::BallHitObstacle {
                obstacle,
                position,
                normal,
                tick,
                ..
            } => Self {
                impact: position,
                normal,
                kind: CollisionDisplayKind::Obstacle(obstacle.to_string()),
                tick,
            },
            PhysicsEvent::BallEnteredBucket { ball, tick } => Self {
                impact: Vec2::ZERO,
                normal: Vec2::ZERO,
                kind: CollisionDisplayKind::Bucket(ball.to_string()),
                tick,
            },
            PhysicsEvent::BallExitedBoard { ball, tick } => Self {
                impact: Vec2::ZERO,
                normal: Vec2::ZERO,
                kind: CollisionDisplayKind::Exit(ball.to_string()),
                tick,
            },
            PhysicsEvent::ShotEnded { summary } => Self {
                impact: Vec2::ZERO,
                normal: Vec2::ZERO,
                kind: CollisionDisplayKind::ShotEnd(summary.replay_hash),
                tick: summary.ticks,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollisionEventLogEntry {
    pub tick: u64,
    pub kind: CollisionDisplayKind,
    pub position: Option<Vec2>,
    pub normal: Option<Vec2>,
    pub speed: Option<f64>,
}

impl CollisionEventLogEntry {
    pub fn from_physics_event(event: PhysicsEvent) -> Option<Self> {
        match event {
            PhysicsEvent::BallHitPeg {
                peg,
                position,
                normal,
                speed,
                tick,
                ..
            } => Some(Self {
                tick,
                kind: CollisionDisplayKind::Peg(peg.to_string()),
                position: Some(position),
                normal: Some(normal),
                speed: Some(speed),
            }),
            PhysicsEvent::BallHitObstacle {
                obstacle,
                position,
                normal,
                tick,
                ..
            } => Some(Self {
                tick,
                kind: CollisionDisplayKind::Obstacle(obstacle.to_string()),
                position: Some(position),
                normal: Some(normal),
                speed: None,
            }),
            PhysicsEvent::BallEnteredBucket { ball, tick } => Some(Self {
                tick,
                kind: CollisionDisplayKind::Bucket(ball.to_string()),
                position: None,
                normal: None,
                speed: None,
            }),
            PhysicsEvent::BallExitedBoard { ball, tick } => Some(Self {
                tick,
                kind: CollisionDisplayKind::Exit(ball.to_string()),
                position: None,
                normal: None,
                speed: None,
            }),
            PhysicsEvent::ShotEnded { .. } => None,
        }
    }
}

#[cfg(test)]
mod f3_tests {
    use super::*;

    #[test]
    fn f3_overlay_toggles_required_fields() {
        let mut overlay = F3DebugOverlay::from_runtime(
            120,
            Some("abc123".to_owned()),
            BoardId::new("boards/test").unwrap(),
            7,
            11,
        );

        assert!(overlay.visible_lines().is_empty());
        overlay.handle_f3();
        let lines = overlay.visible_lines();

        assert_eq!(lines.len(), 5);
        assert!(lines.iter().any(|line| line == "physics_tick_rate=120hz"));
        assert!(lines.iter().any(|line| line == "last_replay_hash=abc123"));
        assert!(lines
            .iter()
            .any(|line| line == "current_board_id=boards/test"));
        assert!(lines.iter().any(|line| line == "active_relic_count=7"));
        assert!(lines.iter().any(|line| line == "telemetry_event_count=11"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{minimal_test_board, BallId};
    use physics_core::simulate_shot;

    #[test]
    fn debug_overlay_can_be_driven_without_game_loop() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let result = simulate_shot(123, &board, &input);

        let overlay = DebugOverlayState::mock_from_board_and_input(
            &board,
            &input,
            Some(result.summary.replay_hash.clone()),
            result.events.clone(),
        );

        assert_eq!(overlay.board_id, board.id);
        assert_eq!(overlay.replay_hash, Some(result.summary.replay_hash));
        assert!(overlay.aim.first_bounce.is_some());
        assert_eq!(
            overlay.event_log_summary.collision_events,
            overlay.collision_events.len()
        );
        assert!(overlay
            .collision_events
            .iter()
            .any(|event| matches!(event.kind, CollisionDisplayKind::Peg(_))));
    }

    #[test]
    fn debug_aim_overlay_can_reuse_aim_hud_state() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let aim_hud = AimHudState::from_board_and_input(&board, &input);

        let debug_aim = DebugAimOverlay::from_aim_hud(&aim_hud);

        assert_eq!(debug_aim.ray_origin, aim_hud.origin);
        assert_eq!(
            debug_aim.first_bounce.unwrap().impact,
            aim_hud.first_bounce.unwrap().impact
        );
    }
}
