use board_gen::{authored_boards_dir, load_authored_boards, BoardLoadError};
use content_schema::{BallId, BoardDefinition, Scalar, Score};
use physics_core::{simulate_shot, ShotInput, ShotResult};

use crate::plugins::{
    debug::DebugOverlayState,
    render::FeelTestRenderState,
    ui::{FeelTestHudParts, FeelTestHudState},
};

pub const FEEL_TEST_SEED: u64 = 0xFEE1_FA11;
const DEFAULT_LAUNCH_SPEED: Scalar = 17.5;
const DEFAULT_BALLS: u32 = 9;
const AIM_STEP_RADIANS: Scalar = 2.5_f64.to_radians();

#[derive(Clone, Debug, PartialEq)]
pub struct FeelTestScene {
    pub board: BoardDefinition,
    pub input: ShotInput,
    pub seed: u64,
    pub balls_remaining: u32,
    pub shot_count: u32,
    pub mock_score: Score,
    pub last_result: Option<ShotResult>,
    pub hud: FeelTestHudState,
    pub debug: DebugOverlayState,
    pub render: FeelTestRenderState,
}

impl FeelTestScene {
    pub fn load_default_authored() -> Result<Self, FeelTestSceneError> {
        let mut boards = load_authored_boards(authored_boards_dir())?;
        let board = boards
            .drain(..)
            .next()
            .ok_or(FeelTestSceneError::NoAuthoredBoards)?;
        Ok(Self::new(board))
    }

    pub fn new(board: BoardDefinition) -> Self {
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: DEFAULT_LAUNCH_SPEED,
            ball_id: BallId::new("balls/basic").expect("static id is valid"),
        };
        let (hud, debug, render) = build_views(&board, &input, None, DEFAULT_BALLS, 0, 0, None);

        Self {
            board,
            input,
            seed: FEEL_TEST_SEED,
            balls_remaining: DEFAULT_BALLS,
            shot_count: 0,
            mock_score: 0,
            last_result: None,
            hud,
            debug,
            render,
        }
    }

    pub fn adjust_aim_steps(&mut self, steps: i32) {
        self.input.aim_angle_radians = clamp_feel_test_angle(
            self.input.aim_angle_radians + AIM_STEP_RADIANS * Scalar::from(steps),
        );
        self.refresh_views();
    }

    pub fn shoot(&mut self) {
        let result = simulate_shot(
            self.seed + u64::from(self.shot_count),
            &self.board,
            &self.input,
        );
        self.shot_count += 1;
        self.balls_remaining = self.balls_remaining.saturating_sub(1);
        self.mock_score += result.summary.pegs_hit.len() as Score * 100;
        if result.summary.caught_bucket {
            self.balls_remaining += 1;
            self.mock_score += 2_500;
        }
        self.last_result = Some(result);
        self.refresh_views();
    }

    pub fn outcome_line(&self) -> String {
        let replay_hash = self
            .last_result
            .as_ref()
            .map(|result| result.summary.replay_hash.as_str())
            .unwrap_or("<none>");
        format!(
            "feel-test board={} aim_deg={:.2} first_bounce={} shots={} balls={} score={} replay_hash={} {}",
            self.board.id,
            self.input.aim_angle_radians.to_degrees(),
            self.hud.aim.first_bounce.is_some(),
            self.shot_count,
            self.balls_remaining,
            self.mock_score,
            replay_hash,
            self.debug.event_log_summary.display_line()
        )
    }

    fn refresh_views(&mut self) {
        let replay_hash = self
            .last_result
            .as_ref()
            .map(|result| result.summary.replay_hash.clone());
        let events = self
            .last_result
            .as_ref()
            .map(|result| result.events.as_slice());
        let (hud, debug, render) = build_views(
            &self.board,
            &self.input,
            replay_hash,
            self.balls_remaining,
            self.shot_count,
            self.mock_score,
            events,
        );
        self.hud = hud;
        self.debug = debug;
        self.render = render;
    }
}

#[derive(Debug)]
pub enum FeelTestSceneError {
    BoardLoad(BoardLoadError),
    NoAuthoredBoards,
}

impl From<BoardLoadError> for FeelTestSceneError {
    fn from(error: BoardLoadError) -> Self {
        Self::BoardLoad(error)
    }
}

impl std::fmt::Display for FeelTestSceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoardLoad(error) => error.fmt(f),
            Self::NoAuthoredBoards => write!(f, "no authored boards found"),
        }
    }
}

impl std::error::Error for FeelTestSceneError {}

pub fn run_smoke_scene() -> Result<FeelTestScene, FeelTestSceneError> {
    let mut scene = FeelTestScene::load_default_authored()?;
    scene.adjust_aim_steps(-2);
    scene.shoot();
    Ok(scene)
}

fn build_views(
    board: &BoardDefinition,
    input: &ShotInput,
    replay_hash: Option<String>,
    balls_remaining: u32,
    shot_count: u32,
    mock_score: Score,
    events: Option<&[physics_core::PhysicsEvent]>,
) -> (FeelTestHudState, DebugOverlayState, FeelTestRenderState) {
    let debug = DebugOverlayState::mock_from_board_and_input(
        board,
        input,
        replay_hash.clone(),
        events.unwrap_or(&[]).iter().cloned(),
    );
    let hud = FeelTestHudState::from_scene_parts(
        board,
        input,
        FeelTestHudParts {
            replay_hash,
            balls_remaining,
            shot_count,
            mock_score,
            collision_count: debug.event_log_summary.collision_events,
            event_log_summary: debug.event_log_summary.display_line(),
        },
    );
    let render = FeelTestRenderState::from_board_and_debug(board, &debug);

    (hud, debug, render)
}

fn clamp_feel_test_angle(angle: Scalar) -> Scalar {
    angle.clamp(18.0_f64.to_radians(), 162.0_f64.to_radians())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feel_test_scene_can_aim_shoot_and_update_replay_hash() {
        let mut scene = FeelTestScene::load_default_authored().unwrap();
        let original_angle = scene.input.aim_angle_radians;

        scene.adjust_aim_steps(-1);

        assert!(scene.input.aim_angle_radians < original_angle);
        assert!(scene.hud.aim.first_bounce.is_some());

        scene.shoot();

        assert_eq!(scene.shot_count, 1);
        assert_eq!(scene.hud.shot_count, 1);
        assert!(scene.last_result.is_some());
        assert!(scene.hud.replay_hash.is_some());
        assert_eq!(
            scene.hud.replay_hash,
            scene
                .last_result
                .as_ref()
                .map(|result| result.summary.replay_hash.clone())
        );
    }

    #[test]
    fn feel_test_scene_exposes_first_bounce_debug_data() {
        let scene = FeelTestScene::load_default_authored().unwrap();

        assert_eq!(scene.hud.board_id, scene.board.id);
        assert!(scene.debug.aim.first_bounce.is_some());
        assert_eq!(scene.render.peg_primitives, scene.board.pegs.len());
        assert!(scene.outcome_line().contains("first_bounce=true"));
    }

    #[test]
    fn smoke_scene_runs_one_deterministic_shot() {
        let scene_a = run_smoke_scene().unwrap();
        let scene_b = run_smoke_scene().unwrap();

        assert_eq!(scene_a.shot_count, 1);
        assert_eq!(scene_a.hud.replay_hash, scene_b.hud.replay_hash);
        assert_eq!(
            scene_a.debug.event_log_summary,
            scene_b.debug.event_log_summary
        );
    }
}
