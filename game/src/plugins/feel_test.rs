use board_gen::{authored_boards_dir, load_authored_boards, BoardLoadError};
use content_schema::{BallId, BoardDefinition, Scalar, Score};
use physics_core::{
    sample_shot_trajectory, simulate_shot, ShotInput, ShotResult, TrajectorySample,
};

use crate::plugins::{
    debug::DebugOverlayState,
    feedback::{play_shot_feedback, FeelTestFeedbackPlayback},
    render::FeelTestRenderState,
    ui::{FeelTestHudParts, FeelTestHudState, SliceCompletionSummary},
};
use feedback_events::AccessibilityFeedbackFlags;

pub const FEEL_TEST_SEED: u64 = 0xFEE1_FA11;
const DEFAULT_LAUNCH_SPEED: Scalar = 17.5;
const DEFAULT_BALLS: u32 = 9;
const AIM_STEP_RADIANS: Scalar = 2.5_f64.to_radians();
const TRAJECTORY_SAMPLE_EVERY_TICKS: u64 = 6;
const PLAYBACK_TICKS_PER_SECOND: Scalar = 480.0;
const SMOKE_PLAYBACK_DELTA_SECONDS: Scalar = 1.0 / 30.0;

#[derive(Clone, Debug, PartialEq)]
pub struct TrajectoryPlaybackCursor {
    samples: Vec<TrajectorySample>,
    ticks_per_second: Scalar,
    elapsed_ticks: Scalar,
    visible_sample_count: usize,
    complete: bool,
}

impl TrajectoryPlaybackCursor {
    pub fn new(samples: Vec<TrajectorySample>, ticks_per_second: Scalar) -> Self {
        let complete = samples.len() <= 1;
        Self {
            samples,
            ticks_per_second: ticks_per_second.max(1.0),
            elapsed_ticks: 0.0,
            visible_sample_count: 1,
            complete,
        }
    }

    pub fn advance(&mut self, delta_seconds: Scalar) -> bool {
        if self.complete || self.samples.is_empty() {
            return false;
        }

        self.elapsed_ticks += delta_seconds.max(0.0) * self.ticks_per_second;
        let last_tick = self.samples.last().map(|sample| sample.tick).unwrap_or(0) as Scalar;
        if self.elapsed_ticks >= last_tick {
            self.elapsed_ticks = last_tick;
            self.visible_sample_count = self.samples.len();
            self.complete = true;
            return true;
        }

        while self.visible_sample_count < self.samples.len()
            && self.samples[self.visible_sample_count].tick as Scalar <= self.elapsed_ticks
        {
            self.visible_sample_count += 1;
        }

        true
    }

    pub fn current_position(&self) -> Option<content_schema::Vec2> {
        let first = self.samples.first()?;
        if self.samples.len() == 1 || self.complete {
            return self.samples.last().map(|sample| sample.position);
        }

        let next_index = self
            .samples
            .iter()
            .position(|sample| sample.tick as Scalar >= self.elapsed_ticks)
            .unwrap_or(self.samples.len() - 1);
        if next_index == 0 {
            return Some(first.position);
        }

        let previous = self.samples[next_index - 1];
        let next = self.samples[next_index];
        let tick_span = (next.tick - previous.tick).max(1) as Scalar;
        let t = ((self.elapsed_ticks - previous.tick as Scalar) / tick_span).clamp(0.0, 1.0);

        Some(content_schema::Vec2::new(
            previous.position.x + (next.position.x - previous.position.x) * t,
            previous.position.y + (next.position.y - previous.position.y) * t,
        ))
    }

    pub fn trail_points(&self) -> Vec<content_schema::Vec2> {
        let mut points: Vec<_> = self
            .samples
            .iter()
            .take(self.visible_sample_count.min(self.samples.len()))
            .map(|sample| sample.position)
            .collect();

        if !self.complete {
            if let Some(position) = self.current_position() {
                if points.last().is_none_or(|last| *last != position) {
                    points.push(position);
                }
            }
        }

        points
    }

    pub const fn is_complete(&self) -> bool {
        self.complete
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrajectoryPlaybackSnapshot {
    pub trail_point_count: usize,
    pub current_position: Option<content_schema::Vec2>,
    pub complete: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeelTestScene {
    pub board: BoardDefinition,
    pub input: ShotInput,
    pub seed: u64,
    pub balls_remaining: u32,
    pub shot_count: u32,
    pub mock_score: Score,
    pub last_result: Option<ShotResult>,
    pub last_feedback: Option<FeelTestFeedbackPlayback>,
    pub playback_snapshot: Option<TrajectoryPlaybackSnapshot>,
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
        let (hud, debug, render) = build_views(
            &board,
            &input,
            FeelTestHudParts {
                replay_hash: None,
                balls_remaining: DEFAULT_BALLS,
                shot_count: 0,
                mock_score: 0,
                collision_count: 0,
                event_log_summary: String::new(),
                completion: None,
            },
            None,
        );

        Self {
            board,
            input,
            seed: FEEL_TEST_SEED,
            balls_remaining: DEFAULT_BALLS,
            shot_count: 0,
            mock_score: 0,
            last_result: None,
            last_feedback: None,
            playback_snapshot: None,
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
        let feedback =
            play_shot_feedback(&self.board, &result, AccessibilityFeedbackFlags::DEFAULT);
        self.playback_snapshot = Some(build_playback_snapshot(&self.board, &self.input));
        self.last_result = Some(result);
        self.last_feedback = Some(feedback);
        self.refresh_views();
    }

    pub fn outcome_line(&self) -> String {
        let replay_hash = self
            .last_result
            .as_ref()
            .map(|result| result.summary.replay_hash.as_str())
            .unwrap_or("<none>");
        format!(
            "feel-test board={} aim_deg={:.2} first_bounce={} shots={} balls={} score={} replay_hash={} {} {} {}",
            self.board.id,
            self.input.aim_angle_radians.to_degrees(),
            self.hud.aim.first_bounce.is_some(),
            self.shot_count,
            self.balls_remaining,
            self.mock_score,
            replay_hash,
            self.debug.event_log_summary.display_line(),
            self.hud
                .completion
                .as_ref()
                .map_or_else(|| "slice=<none>".to_owned(), |completion| completion.display_line()),
            self.playback_snapshot_line()
        )
    }

    fn playback_snapshot_line(&self) -> String {
        self.playback_snapshot
            .as_ref()
            .map(|snapshot| {
                format!(
                    "playback_points={} playback_position={} playback_complete={}",
                    snapshot.trail_point_count,
                    snapshot.current_position.is_some(),
                    snapshot.complete
                )
            })
            .unwrap_or_else(|| {
                "playback_points=0 playback_position=false playback_complete=false".to_owned()
            })
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
        let completion = self.last_result.as_ref().map(|result| {
            SliceCompletionSummary::from_shot_summary(
                &self.board,
                &result.summary,
                self.mock_score,
                self.balls_remaining,
                self.last_feedback
                    .as_ref()
                    .map_or(0, |feedback| feedback.summaries.len()),
            )
            .with_feedback_events(
                self.last_feedback
                    .as_ref()
                    .map_or(0, |feedback| feedback.events.len()),
            )
        });
        let (hud, debug, render) = build_views(
            &self.board,
            &self.input,
            FeelTestHudParts {
                replay_hash,
                balls_remaining: self.balls_remaining,
                shot_count: self.shot_count,
                mock_score: self.mock_score,
                collision_count: 0,
                event_log_summary: String::new(),
                completion,
            },
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
    mut parts: FeelTestHudParts,
    events: Option<&[physics_core::PhysicsEvent]>,
) -> (FeelTestHudState, DebugOverlayState, FeelTestRenderState) {
    let debug = DebugOverlayState::mock_from_board_and_input(
        board,
        input,
        parts.replay_hash.clone(),
        events.unwrap_or(&[]).iter().cloned(),
    );
    parts.collision_count = debug.event_log_summary.collision_events;
    parts.event_log_summary = debug.event_log_summary.display_line();
    let hud = FeelTestHudState::from_scene_parts(board, input, parts);
    let render = FeelTestRenderState::from_board_and_debug(board, &debug);

    (hud, debug, render)
}

fn clamp_feel_test_angle(angle: Scalar) -> Scalar {
    angle.clamp(18.0_f64.to_radians(), 162.0_f64.to_radians())
}

fn build_playback_snapshot(
    board: &BoardDefinition,
    input: &ShotInput,
) -> TrajectoryPlaybackSnapshot {
    let samples = sample_shot_trajectory(board, input, TRAJECTORY_SAMPLE_EVERY_TICKS);
    let mut cursor = TrajectoryPlaybackCursor::new(samples, PLAYBACK_TICKS_PER_SECOND);
    cursor.advance(SMOKE_PLAYBACK_DELTA_SECONDS);

    TrajectoryPlaybackSnapshot {
        trail_point_count: cursor.trail_points().len(),
        current_position: cursor.current_position(),
        complete: cursor.is_complete(),
    }
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
        assert!(scene.last_feedback.is_some());
        assert!(scene.hud.completion.is_some());
        assert_eq!(
            scene.hud.replay_hash,
            scene
                .last_result
                .as_ref()
                .map(|result| result.summary.replay_hash.clone())
        );
        let completion = scene.hud.completion.as_ref().unwrap();
        assert_eq!(completion.score, scene.mock_score);
        assert_eq!(
            completion.hit_pegs,
            scene.last_result.as_ref().unwrap().summary.pegs_hit.len()
        );
        assert_eq!(
            completion.replay_hash,
            scene.hud.replay_hash.as_ref().unwrap().as_str()
        );
        assert_eq!(
            completion.feedback_cues,
            scene.last_feedback.as_ref().unwrap().summaries.len()
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
        assert!(scene_a.outcome_line().contains("hit_oranges="));
        assert!(scene_a.outcome_line().contains("progression="));
        assert_eq!(
            scene_a.debug.event_log_summary,
            scene_b.debug.event_log_summary
        );
    }

    #[test]
    fn trajectory_playback_cursor_reveals_trail_progressively() {
        let samples = vec![
            TrajectorySample {
                tick: 0,
                position: content_schema::Vec2::new(0.0, 0.0),
            },
            TrajectorySample {
                tick: 10,
                position: content_schema::Vec2::new(10.0, 0.0),
            },
            TrajectorySample {
                tick: 20,
                position: content_schema::Vec2::new(20.0, 0.0),
            },
        ];
        let mut cursor = TrajectoryPlaybackCursor::new(samples, 10.0);

        assert_eq!(
            cursor.trail_points(),
            vec![content_schema::Vec2::new(0.0, 0.0)]
        );
        assert!(!cursor.is_complete());

        cursor.advance(0.5);

        assert_eq!(
            cursor.current_position(),
            Some(content_schema::Vec2::new(5.0, 0.0))
        );
        assert_eq!(
            cursor.trail_points(),
            vec![
                content_schema::Vec2::new(0.0, 0.0),
                content_schema::Vec2::new(5.0, 0.0)
            ]
        );
        assert!(!cursor.is_complete());
    }

    #[test]
    fn trajectory_playback_cursor_completes_on_final_sample() {
        let samples = vec![
            TrajectorySample {
                tick: 0,
                position: content_schema::Vec2::new(0.0, 0.0),
            },
            TrajectorySample {
                tick: 4,
                position: content_schema::Vec2::new(1.0, 1.0),
            },
        ];
        let mut cursor = TrajectoryPlaybackCursor::new(samples, 4.0);

        cursor.advance(1.0);

        assert!(cursor.is_complete());
        assert_eq!(
            cursor.current_position(),
            Some(content_schema::Vec2::new(1.0, 1.0))
        );
        assert_eq!(
            cursor.trail_points(),
            vec![
                content_schema::Vec2::new(0.0, 0.0),
                content_schema::Vec2::new(1.0, 1.0)
            ]
        );
    }
}
