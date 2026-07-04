use std::fmt;

use board_gen::{authored_boards_dir, load_authored_boards, BoardLoadError};
use content_schema::{BallId, BoardDefinition, PegId, PegKind, Score, Seed};
use game_rules::{promote_physics_event, GameEvent, LossReason, ResourceKind};
use physics_core::{simulate_shot, ShotInput, ShotResult};
use rpg_mode::CharacterState;
use run_mode::{act1_slice_nodes, RunState};

use crate::plugins::{
    feedback::play_shot_feedback,
    ui::{SliceCompletionSummary, SliceProgressionOutcome},
};

const SLICE_SEED: Seed = 0xC2A0_0000_0000_0002;
const AUTHORED_BOARD_ID: &str = "boards/feel_fan_01";
const LAUNCH_SPEED: f64 = 17.5;
const SHOT_AIM_RADIANS: f64 = std::f64::consts::FRAC_PI_2;

#[derive(Clone, Debug, PartialEq)]
pub struct VerticalSliceSession {
    pub seed: Seed,
    pub board: BoardDefinition,
    pub run_state: RunState,
    pub character_state: CharacterState,
    pub total_score: Score,
    pub shots: Vec<VerticalSliceShot>,
    pub game_events: Vec<GameEvent>,
}

impl VerticalSliceSession {
    pub fn new(seed: Seed, board: BoardDefinition) -> Self {
        let mut run_state = RunState::act1_slice(seed);
        if let Some(node) = act1_slice_nodes()
            .into_iter()
            .find(|node| node.board.as_ref() == Some(&board.id))
        {
            run_state.visited_nodes.push(node);
        }

        Self {
            seed,
            board,
            run_state,
            character_state: CharacterState::act1_slice(),
            total_score: 0,
            shots: Vec::new(),
            game_events: Vec::new(),
        }
    }

    pub fn fire_scripted_shot(&mut self) {
        let input = ShotInput {
            aim_angle_radians: SHOT_AIM_RADIANS,
            launch_speed: LAUNCH_SPEED,
            ball_id: BallId::new("balls/basic").expect("static id is valid"),
        };
        let result = simulate_shot(self.seed + self.shots.len() as u64, &self.board, &input);
        self.apply_shot(input, result);
    }

    fn apply_shot(&mut self, input: ShotInput, result: ShotResult) {
        if self.run_state.resources.shots > 0 {
            self.run_state.resources.shots -= 1;
            self.game_events.push(GameEvent::ResourceChanged {
                resource: ResourceKind::Balls,
                delta: -1,
            });
        }

        for event in result.events.iter().cloned() {
            self.game_events.push(promote_physics_event(event));
        }

        let mut base_score = 0;
        for peg in &result.summary.pegs_hit {
            let points = peg_score(&self.board, peg);
            if points > 0 {
                base_score += points;
                self.game_events.push(GameEvent::PegScored {
                    peg: peg.clone(),
                    points,
                });
            }
        }

        let mut shots_granted = 0;
        if result.summary.caught_bucket {
            shots_granted = 1;
            self.run_state.resources.shots += shots_granted;
            self.game_events.push(GameEvent::BucketCatchAwarded {
                ball: input.ball_id.clone(),
                points: 2_500,
                shots_granted,
            });
            self.game_events.push(GameEvent::ResourceChanged {
                resource: ResourceKind::Balls,
                delta: i64::from(shots_granted),
            });
            base_score += 2_500;
        }

        self.total_score += base_score;
        self.run_state.resources.sparks += result.summary.pegs_hit.len() as u32;
        self.character_state.xp += result.summary.pegs_hit.len() as u64;

        if !result.summary.pegs_hit.is_empty() {
            let delta = result.summary.pegs_hit.len() as i64;
            self.game_events.push(GameEvent::ResourceChanged {
                resource: ResourceKind::Sparks,
                delta,
            });
            self.game_events.push(GameEvent::ResourceChanged {
                resource: ResourceKind::Xp,
                delta,
            });
        }

        self.game_events.push(GameEvent::ShotScoreResolved {
            base_score,
            fever_multiplier: 1,
            combo_multiplier: 1,
            final_score: base_score,
        });

        let won = result
            .remaining_pegs
            .iter()
            .all(|peg| peg.kind != PegKind::Orange);
        let progression_outcome = if won {
            SliceProgressionOutcome::BoardWon
        } else if self.run_state.resources.shots == 0 {
            SliceProgressionOutcome::BoardLost
        } else {
            SliceProgressionOutcome::Continue
        };
        if won {
            self.game_events.push(GameEvent::BoardWon {
                board: self.board.id.clone(),
                final_score: self.total_score,
            });
        } else if self.run_state.resources.shots == 0 {
            self.game_events.push(GameEvent::BoardLost {
                board: self.board.id.clone(),
                reason: LossReason::OutOfShots,
            });
        }

        let feedback = play_shot_feedback(
            &self.board,
            &result,
            feedback_events::AccessibilityFeedbackFlags::DEFAULT,
        );
        let mut completion = SliceCompletionSummary::from_shot_summary(
            &self.board,
            &result.summary,
            self.total_score,
            self.run_state.resources.shots,
            feedback.summaries.len(),
        )
        .with_feedback_events(feedback.events.len());
        completion.progression_outcome = progression_outcome;

        let summary = VerticalSliceShot {
            input,
            pegs_hit: result.summary.pegs_hit.clone(),
            score: base_score,
            caught_bucket: result.summary.caught_bucket,
            replay_hash: result.summary.replay_hash.clone(),
            physics_ticks: result.summary.ticks,
            shots_granted,
            completion,
        };
        self.board.pegs = result.remaining_pegs;
        self.shots.push(summary);
    }

    pub fn smoke_summary(&self) -> String {
        let last = self.shots.last();
        format!(
            "checkpoint2 vertical_slice board={} seed={} shots={} pegs_hit={} score={} balls={} sparks={} xp={} events={} replay_hash={} {}",
            self.board.id,
            self.seed,
            self.shots.len(),
            last.map_or(0, |shot| shot.pegs_hit.len()),
            self.total_score,
            self.run_state.resources.shots,
            self.run_state.resources.sparks,
            self.character_state.xp,
            self.game_events.len(),
            last.map_or("<none>", |shot| shot.replay_hash.as_str()),
            last.map_or_else(|| "slice=<none>".to_owned(), |shot| shot.completion.display_line()),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VerticalSliceShot {
    pub input: ShotInput,
    pub pegs_hit: Vec<PegId>,
    pub score: Score,
    pub caught_bucket: bool,
    pub replay_hash: String,
    pub physics_ticks: u64,
    pub shots_granted: u32,
    pub completion: SliceCompletionSummary,
}

#[derive(Debug)]
pub enum VerticalSliceError {
    BoardLoad(BoardLoadError),
    MissingAuthoredBoard,
}

impl fmt::Display for VerticalSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoardLoad(error) => error.fmt(f),
            Self::MissingAuthoredBoard => write!(f, "missing authored board {AUTHORED_BOARD_ID}"),
        }
    }
}

impl std::error::Error for VerticalSliceError {}

impl From<BoardLoadError> for VerticalSliceError {
    fn from(value: BoardLoadError) -> Self {
        Self::BoardLoad(value)
    }
}

pub fn run_smoke_session() -> Result<VerticalSliceSession, VerticalSliceError> {
    let mut session = VerticalSliceSession::new(SLICE_SEED, load_vertical_slice_board()?);
    session.fire_scripted_shot();
    Ok(session)
}

fn load_vertical_slice_board() -> Result<BoardDefinition, VerticalSliceError> {
    load_authored_boards(authored_boards_dir())?
        .into_iter()
        .find(|board| board.id.as_str() == AUTHORED_BOARD_ID)
        .ok_or(VerticalSliceError::MissingAuthoredBoard)
}

fn peg_score(board: &BoardDefinition, peg_id: &PegId) -> Score {
    match board
        .pegs
        .iter()
        .find(|peg| peg.id == *peg_id)
        .map(|peg| peg.kind)
    {
        Some(PegKind::Orange) => 1_000,
        Some(PegKind::Purple) => 5_000,
        Some(PegKind::Green) => 500,
        Some(PegKind::Blue | PegKind::Bomb | PegKind::Splitter) => 100,
        Some(PegKind::Ghost | PegKind::Stone) | None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{minimal_test_board, Vec2};
    use physics_core::{PhysicsEvent, ShotSummary};

    #[test]
    fn scripted_smoke_session_hits_authored_board_and_updates_modes() {
        let session = run_smoke_session().unwrap();

        assert_eq!(session.board.id.as_str(), AUTHORED_BOARD_ID);
        assert_eq!(session.shots.len(), 1);
        assert!(!session.shots[0].pegs_hit.is_empty());
        assert!(session.total_score > 0);
        assert_eq!(
            session.run_state.resources.shots,
            7 + session.shots[0].shots_granted
        );
        assert_eq!(
            session.run_state.resources.sparks as usize,
            session.shots[0].pegs_hit.len()
        );
        assert_eq!(
            session.character_state.xp as usize,
            session.shots[0].pegs_hit.len()
        );
        assert!(session
            .game_events
            .iter()
            .any(|event| matches!(event, GameEvent::Physics(_))));
        assert!(session
            .game_events
            .iter()
            .any(|event| matches!(event, GameEvent::ShotScoreResolved { .. })));
        assert_eq!(session.shots[0].completion.score, session.total_score);
        assert_eq!(
            session.shots[0].completion.hit_pegs,
            session.shots[0].pegs_hit.len()
        );
        assert!(session.smoke_summary().contains("hit_oranges="));
        assert!(session.smoke_summary().contains("progression="));
    }

    #[test]
    fn applying_shot_emits_scoring_and_resource_events() {
        let board = minimal_test_board();
        let orange = board.pegs[0].id.clone();
        let mut session = VerticalSliceSession::new(7, board);
        let input = ShotInput {
            aim_angle_radians: SHOT_AIM_RADIANS,
            launch_speed: LAUNCH_SPEED,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let result = ShotResult {
            events: vec![PhysicsEvent::BallHitPeg {
                ball: input.ball_id.clone(),
                peg: orange.clone(),
                position: Vec2::new(10.0, 14.0),
                normal: Vec2::new(0.0, -1.0),
                speed: 10.0,
                tick: 20,
            }],
            summary: ShotSummary {
                ticks: 120,
                pegs_hit: vec![orange.clone()],
                caught_bucket: false,
                exited_board: true,
                replay_hash: "unit".to_owned(),
            },
            remaining_pegs: session.board.pegs[1..].to_vec(),
        };

        session.apply_shot(input, result);

        assert_eq!(session.total_score, 1_000);
        assert_eq!(session.run_state.resources.shots, 7);
        assert_eq!(session.run_state.resources.sparks, 1);
        assert_eq!(session.character_state.xp, 1);
        assert_eq!(session.shots[0].completion.hit_oranges, 1);
        assert_eq!(
            session.shots[0].completion.progression_outcome,
            SliceProgressionOutcome::BoardWon
        );
        assert!(session.game_events.iter().any(|event| {
            matches!(event, GameEvent::PegScored { peg, points } if *peg == orange && *points == 1_000)
        }));
        assert!(session.game_events.iter().any(|event| {
            matches!(
                event,
                GameEvent::ResourceChanged {
                    resource: ResourceKind::Balls,
                    delta: -1
                }
            )
        }));
    }
}
