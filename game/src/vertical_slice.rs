use std::fmt;

use board_gen::{authored_boards_dir, load_authored_boards, BoardLoadError};
use content_schema::{BallId, BoardDefinition, BoardId, ContentId, PegId, PegKind, Score, Seed};
use game_rules::{promote_physics_event, GameEvent, LossReason, ResourceKind};
use physics_core::{simulate_shot, ShotInput, ShotResult};
use rpg_mode::CharacterState;
use run_mode::{
    act1_slice_nodes, act1_slice_reward_offers, RewardOffer, RunNode, RunNodeKind, RunResources,
    RunState,
};

use crate::plugins::{
    feedback::play_shot_feedback,
    node_map_ui::{NodeMapInput, RunEndReason, RunScreen, RunSummary},
    run_summary_ui::{RunSummaryOutcome, RunSummaryScreen},
    ui::{RunSessionSummary, SliceCompletionSummary, SliceProgressionOutcome},
};

const SLICE_SEED: Seed = 0xC2A0_0000_0000_0002;
const AUTHORED_BOARD_ID: &str = "boards/feel_fan_01";
const LAUNCH_SPEED: f64 = 17.5;
const SHOT_AIM_RADIANS: f64 = std::f64::consts::FRAC_PI_2;

#[derive(Clone, Debug, PartialEq)]
pub struct VerticalSliceSession {
    pub seed: Seed,
    pub board: BoardDefinition,
    pub boards: Vec<BoardDefinition>,
    pub nodes: Vec<RunNode>,
    pub reward_offers: Vec<RewardOffer>,
    pub run_state: RunState,
    pub character_state: CharacterState,
    pub total_score: Score,
    pub shots: Vec<VerticalSliceShot>,
    pub game_events: Vec<GameEvent>,
    pub screen: RunScreen,
    pub run_summary: Option<RunSummary>,
    pending_reward_next_node: Option<u16>,
}

impl VerticalSliceSession {
    pub fn new(seed: Seed, board: BoardDefinition) -> Self {
        Self::with_boards(seed, board.clone(), vec![board])
    }

    pub fn with_boards(seed: Seed, board: BoardDefinition, boards: Vec<BoardDefinition>) -> Self {
        let mut run_state = RunState::act1_slice(seed);
        let nodes = act1_slice_nodes();
        let reward_offers = act1_slice_reward_offers();
        if let Some((index, node)) = nodes
            .into_iter()
            .enumerate()
            .find(|(_, node)| node.board.as_ref() == Some(&board.id))
        {
            run_state.node_index = index as u16;
            run_state.visited_nodes.push(node);
        }
        let nodes = act1_slice_nodes();
        let screen = RunScreen::Board {
            node_index: run_state.node_index,
            board: board.id.clone(),
            balls: run_state.resources.shots,
            hearts: run_state.resources.hearts,
        };

        Self {
            seed,
            board,
            boards,
            nodes,
            reward_offers,
            run_state,
            character_state: CharacterState::act1_slice(),
            total_score: 0,
            shots: Vec::new(),
            game_events: Vec::new(),
            screen,
            run_summary: None,
            pending_reward_next_node: None,
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

        if won {
            self.complete_current_board();
        } else if self.run_state.resources.shots == 0 {
            self.fail_current_board();
        } else {
            self.screen = RunScreen::Board {
                node_index: self.run_state.node_index,
                board: self.board.id.clone(),
                balls: self.run_state.resources.shots,
                hearts: self.run_state.resources.hearts,
            };
        }
    }

    pub fn choose_reward_key(&mut self, key: char) -> Result<(), RunSessionError> {
        let choice_index = match key {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            _ => return Err(RunSessionError::InvalidRewardKey(key)),
        };
        self.choose_reward(choice_index)
    }

    pub fn choose_reward(&mut self, choice_index: usize) -> Result<(), RunSessionError> {
        let offer = match &self.screen {
            RunScreen::RewardChoice { offer, .. } => offer.clone(),
            _ => return Err(RunSessionError::NotOnRewardScreen),
        };
        let reward = offer
            .choices
            .get(choice_index)
            .cloned()
            .ok_or(RunSessionError::RewardChoiceOutOfRange(choice_index))?;

        self.run_state.apply_reward(&reward);
        self.screen =
            RunScreen::NodeMap(crate::plugins::node_map_ui::NodeMapScreen::from_run_state(
                &self.run_state,
                &self.nodes,
            ));
        Ok(())
    }

    pub fn advance_from_node_map_input(
        &mut self,
        input: NodeMapInput,
    ) -> Result<(), RunSessionError> {
        match (&self.screen, input) {
            (RunScreen::NodeMap(_), NodeMapInput::Enter | NodeMapInput::Space) => {
                let mut next_node = self
                    .pending_reward_next_node
                    .take()
                    .unwrap_or(self.run_state.node_index + 1);
                while self
                    .nodes
                    .get(next_node as usize)
                    .is_some_and(|node| node.kind == RunNodeKind::Reward)
                {
                    next_node += 1;
                }
                let node = self
                    .nodes
                    .get(next_node as usize)
                    .cloned()
                    .ok_or(RunSessionError::MissingNode(next_node))?;
                let board_id = node
                    .board
                    .clone()
                    .ok_or_else(|| RunSessionError::NodeHasNoBoard(node.id.clone()))?;
                self.board = self
                    .boards
                    .iter()
                    .find(|board| board.id == board_id)
                    .cloned()
                    .ok_or_else(|| RunSessionError::MissingBoard(board_id.clone()))?;
                self.run_state.node_index = next_node;
                self.run_state.resources.shots = shots_per_board(&self.run_state);
                self.run_state.visited_nodes.push(node);
                self.screen = RunScreen::Board {
                    node_index: next_node,
                    board: board_id,
                    balls: self.run_state.resources.shots,
                    hearts: self.run_state.resources.hearts,
                };
                Ok(())
            }
            (RunScreen::NodeMap(_), NodeMapInput::Other) => Ok(()),
            _ => Err(RunSessionError::NotOnNodeMapScreen),
        }
    }

    pub fn retry_failed_board(&mut self) -> Result<(), RunSessionError> {
        match self.screen {
            RunScreen::Failure {
                can_retry: true, ..
            } => self.load_node(self.run_state.node_index),
            _ => Err(RunSessionError::NotOnFailureScreen),
        }
    }

    pub fn continue_after_failure(&mut self) -> Result<(), RunSessionError> {
        match self.screen {
            RunScreen::Failure {
                can_continue: true, ..
            } => self.advance_to_node(self.run_state.node_index + 1),
            _ => Err(RunSessionError::NotOnFailureScreen),
        }
    }

    fn complete_current_board(&mut self) {
        let current = self.run_state.node_index;
        let current_kind = self
            .nodes
            .get(current as usize)
            .map(|node| node.kind)
            .unwrap_or(RunNodeKind::Board);

        if current_kind == RunNodeKind::Boss {
            self.finish_run(RunEndReason::BossCleared);
            return;
        }

        let next = current + 1;
        let Some(offer) = self.reward_offer_for_node(current) else {
            let _ = self.advance_to_node(next);
            return;
        };
        self.pending_reward_next_node = Some(
            if self
                .nodes
                .get(next as usize)
                .is_some_and(|node| node.kind == RunNodeKind::Reward)
            {
                next + 1
            } else {
                next
            },
        );
        if self
            .nodes
            .get(next as usize)
            .is_some_and(|node| node.kind == RunNodeKind::Reward)
        {
            self.run_state.node_index = next;
            self.run_state
                .visited_nodes
                .push(self.nodes[next as usize].clone());
        }
        self.screen = RunScreen::RewardChoice {
            node_index: self.run_state.node_index,
            offer,
        };
    }

    fn fail_current_board(&mut self) {
        self.run_state.resources.hearts = self.run_state.resources.hearts.saturating_sub(1);
        self.game_events.push(GameEvent::ResourceChanged {
            resource: ResourceKind::Hearts,
            delta: -1,
        });

        if self.run_state.resources.hearts == 0 {
            self.finish_run(RunEndReason::HeartsDepleted);
            return;
        }

        self.screen = RunScreen::Failure {
            board: self.board.id.clone(),
            hearts_remaining: self.run_state.resources.hearts,
            oranges_remaining: orange_count(&self.board),
            can_retry: true,
            can_continue: true,
        };
    }

    fn advance_to_node(&mut self, node_index: u16) -> Result<(), RunSessionError> {
        if node_index as usize >= self.nodes.len() {
            self.finish_run(RunEndReason::PathComplete);
            return Ok(());
        }
        if self.nodes[node_index as usize].kind == RunNodeKind::Reward {
            self.pending_reward_next_node = Some(node_index + 1);
            let offer = self
                .reward_offer_for_node(node_index.saturating_sub(1))
                .ok_or(RunSessionError::MissingRewardOffer(node_index))?;
            self.run_state.node_index = node_index;
            self.run_state
                .visited_nodes
                .push(self.nodes[node_index as usize].clone());
            self.screen = RunScreen::RewardChoice { node_index, offer };
            return Ok(());
        }
        self.load_node(node_index)
    }

    fn load_node(&mut self, node_index: u16) -> Result<(), RunSessionError> {
        let node = self
            .nodes
            .get(node_index as usize)
            .cloned()
            .ok_or(RunSessionError::MissingNode(node_index))?;
        let board_id = node
            .board
            .clone()
            .ok_or_else(|| RunSessionError::NodeHasNoBoard(node.id.clone()))?;
        self.board = self
            .boards
            .iter()
            .find(|board| board.id == board_id)
            .cloned()
            .ok_or_else(|| RunSessionError::MissingBoard(board_id.clone()))?;
        self.run_state.node_index = node_index;
        self.run_state.resources.shots = shots_per_board(&self.run_state);
        self.run_state.visited_nodes.push(node);
        self.screen = RunScreen::Board {
            node_index,
            board: board_id,
            balls: self.run_state.resources.shots,
            hearts: self.run_state.resources.hearts,
        };
        Ok(())
    }

    fn reward_offer_for_node(&self, node_index: u16) -> Option<RewardOffer> {
        let source = self.nodes.get(node_index as usize)?.id.clone();
        self.reward_offers
            .iter()
            .find(|offer| offer.source == source && offer.choices.len() >= 3)
            .cloned()
    }

    fn finish_run(&mut self, reason: RunEndReason) {
        let summary = RunSummary {
            reason,
            final_score: self.total_score,
            relics_collected: self
                .run_state
                .relics
                .iter()
                .map(|relic| relic.id.clone())
                .collect(),
            xp_gained: self.character_state.xp,
            boards_cleared: self
                .game_events
                .iter()
                .filter(|event| matches!(event, GameEvent::BoardWon { .. }))
                .count() as u32,
            hearts_remaining: self.run_state.resources.hearts,
            replay_hash: self.replay_hash(),
        };
        self.screen = RunScreen::Summary(summary.clone());
        self.run_summary = Some(summary);
    }

    pub fn replay_hash(&self) -> String {
        let mut hash = FNV_OFFSET;
        hash = fnv_u64(hash, self.seed);
        hash = fnv_i64(hash, self.total_score);
        hash = fnv_u64(hash, u64::from(self.run_state.node_index));
        hash = fnv_u64(hash, u64::from(self.run_state.resources.hearts));
        hash = fnv_u64(hash, u64::from(self.run_state.resources.shots));
        hash = fnv_u64(hash, self.character_state.xp);
        for shot in &self.shots {
            hash = fnv_str(hash, &shot.replay_hash);
            hash = fnv_i64(hash, shot.score);
        }
        for relic in &self.run_state.relics {
            hash = fnv_str(hash, relic.id.as_str());
            hash = fnv_u64(hash, u64::from(relic.stacks));
        }
        for ball in &self.run_state.balls {
            hash = fnv_str(hash, ball.as_str());
        }
        format!("{hash:016x}")
    }

    pub fn smoke_summary(&self) -> String {
        let last = self.shots.last();
        let run_session = RunSessionSummary::from_states(
            &self.run_state,
            &self.character_state,
            self.total_score,
        );
        format!(
            "checkpoint2 vertical_slice board={} seed={} shots={} pegs_hit={} score={} balls={} sparks={} xp={} events={} replay_hash={} {}",
            self.board.id,
            self.seed,
            self.shots.len(),
            last.map_or(0, |shot| shot.pegs_hit.len()),
            run_session.total_score,
            run_session.shots,
            run_session.sparks,
            run_session.xp,
            self.game_events.len(),
            last.map_or("<none>", |shot| shot.replay_hash.as_str()),
            last.map_or_else(|| "slice=<none>".to_owned(), |shot| shot.completion.display_line()),
        )
    }

    pub fn run_summary(&self) -> RunSummaryScreen {
        RunSummaryScreen::from_run_end(
            &self.run_state,
            &self.character_state,
            0,
            RunSummaryOutcome {
                final_score: self.total_score,
                boards_cleared: self.boards_cleared(),
                oranges_cleared: self.oranges_cleared(),
                bucket_catches: self.bucket_catches(),
                run_duration_shots: self.shots.len() as u32,
                replay_hash: self.replay_hash(),
            },
        )
    }

    fn boards_cleared(&self) -> u32 {
        self.game_events
            .iter()
            .filter(|event| matches!(event, GameEvent::BoardWon { .. }))
            .count() as u32
    }

    fn oranges_cleared(&self) -> u32 {
        self.shots
            .iter()
            .map(|shot| shot.completion.hit_oranges as u32)
            .sum()
    }

    fn bucket_catches(&self) -> u32 {
        self.shots.iter().filter(|shot| shot.caught_bucket).count() as u32
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

#[derive(Debug, PartialEq)]
pub enum RunSessionError {
    InvalidRewardKey(char),
    NotOnNodeMapScreen,
    NotOnRewardScreen,
    RewardChoiceOutOfRange(usize),
    NotOnFailureScreen,
    MissingNode(u16),
    MissingBoard(BoardId),
    MissingRewardOffer(u16),
    NodeHasNoBoard(ContentId),
}

impl fmt::Display for RunSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRewardKey(key) => write!(f, "invalid reward key {key}; use 1, 2, or 3"),
            Self::NotOnNodeMapScreen => write!(f, "not on node-map screen"),
            Self::NotOnRewardScreen => write!(f, "not on reward-choice screen"),
            Self::RewardChoiceOutOfRange(index) => {
                write!(f, "reward choice {index} is unavailable")
            }
            Self::NotOnFailureScreen => write!(f, "not on board-failure screen"),
            Self::MissingNode(index) => write!(f, "missing run node {index}"),
            Self::MissingBoard(board) => write!(f, "missing board {board}"),
            Self::MissingRewardOffer(index) => write!(f, "missing reward offer for node {index}"),
            Self::NodeHasNoBoard(node) => write!(f, "run node {node} has no board"),
        }
    }
}

impl std::error::Error for RunSessionError {}

#[derive(Debug)]
pub enum VerticalSliceError {
    BoardLoad(BoardLoadError),
    MissingAuthoredBoard,
    MissingAct1StartBoard,
    RunSession(RunSessionError),
    SmokeRunDidNotFinish,
}

impl fmt::Display for VerticalSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoardLoad(error) => error.fmt(f),
            Self::MissingAuthoredBoard => write!(f, "missing authored board {AUTHORED_BOARD_ID}"),
            Self::MissingAct1StartBoard => write!(f, "missing Act 1 slice start board"),
            Self::RunSession(error) => error.fmt(f),
            Self::SmokeRunDidNotFinish => write!(f, "checkpoint2 smoke run did not reach summary"),
        }
    }
}

impl std::error::Error for VerticalSliceError {}

impl From<BoardLoadError> for VerticalSliceError {
    fn from(value: BoardLoadError) -> Self {
        Self::BoardLoad(value)
    }
}

impl From<RunSessionError> for VerticalSliceError {
    fn from(value: RunSessionError) -> Self {
        Self::RunSession(value)
    }
}

pub fn run_smoke_session() -> Result<VerticalSliceSession, VerticalSliceError> {
    let mut session = VerticalSliceSession::new(SLICE_SEED, load_vertical_slice_board()?);
    session.fire_scripted_shot();
    let _ = session.choose_reward_key('x');
    let _ = session.choose_reward(99);
    let _ = session.advance_from_node_map_input(NodeMapInput::Other);
    let _ = session.retry_failed_board();
    let _ = session.continue_after_failure();
    let _ = session.run_summary();
    let _ = load_act1_run_session(SLICE_SEED.wrapping_add(1));
    Ok(session)
}

pub fn run_checkpoint2_smoke_session() -> Result<VerticalSliceSession, VerticalSliceError> {
    let mut session = load_act1_run_session(SLICE_SEED)?;

    for _ in 0..16 {
        match &session.screen {
            RunScreen::Board { .. } => clear_current_smoke_board(&mut session),
            RunScreen::RewardChoice { .. } => session.choose_reward(0)?,
            RunScreen::NodeMap(_) => {
                session.advance_from_node_map_input(NodeMapInput::Enter)?;
            }
            RunScreen::Summary(_) => return Ok(session),
            RunScreen::Failure { .. } | RunScreen::RunComplete => {
                return Err(VerticalSliceError::SmokeRunDidNotFinish);
            }
        }
    }

    Err(VerticalSliceError::SmokeRunDidNotFinish)
}

pub fn load_act1_run_session(seed: Seed) -> Result<VerticalSliceSession, VerticalSliceError> {
    let boards = load_authored_boards(authored_boards_dir())?;
    let first_board_id = act1_slice_nodes()
        .into_iter()
        .find(|node| node.kind == RunNodeKind::Board)
        .and_then(|node| node.board)
        .ok_or(VerticalSliceError::MissingAct1StartBoard)?;
    let first_board = boards
        .iter()
        .find(|board| board.id == first_board_id)
        .cloned()
        .ok_or(VerticalSliceError::MissingAct1StartBoard)?;
    Ok(VerticalSliceSession::with_boards(seed, first_board, boards))
}

pub fn run_loop_registration_touchpoint() -> usize {
    let run_state = RunState::act1_slice(SLICE_SEED);
    let node_map =
        crate::plugins::node_map_ui::NodeMapScreen::from_run_state(&run_state, &act1_slice_nodes());
    let screens = [RunScreen::NodeMap(node_map), RunScreen::RunComplete];
    let errors = [
        RunSessionError::InvalidRewardKey('x'),
        RunSessionError::NotOnNodeMapScreen,
        RunSessionError::NotOnRewardScreen,
        RunSessionError::RewardChoiceOutOfRange(9),
        RunSessionError::NotOnFailureScreen,
    ];
    let vertical_error = VerticalSliceError::MissingAct1StartBoard;
    let api = (
        VerticalSliceSession::choose_reward_key
            as fn(&mut VerticalSliceSession, char) -> Result<(), RunSessionError>,
        VerticalSliceSession::choose_reward
            as fn(&mut VerticalSliceSession, usize) -> Result<(), RunSessionError>,
        VerticalSliceSession::advance_from_node_map_input
            as fn(
                &mut VerticalSliceSession,
                crate::plugins::node_map_ui::NodeMapInput,
            ) -> Result<(), RunSessionError>,
        VerticalSliceSession::retry_failed_board
            as fn(&mut VerticalSliceSession) -> Result<(), RunSessionError>,
        VerticalSliceSession::continue_after_failure
            as fn(&mut VerticalSliceSession) -> Result<(), RunSessionError>,
        run_smoke_session as fn() -> Result<VerticalSliceSession, VerticalSliceError>,
        load_act1_run_session as fn(Seed) -> Result<VerticalSliceSession, VerticalSliceError>,
    );

    screens.len()
        + errors.len()
        + usize::from(matches!(
            vertical_error,
            VerticalSliceError::MissingAct1StartBoard
        ))
        + std::mem::size_of_val(&api)
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

fn shots_per_board(run_state: &RunState) -> u32 {
    RunResources::act1_slice().shots + run_state.balls.len().saturating_sub(1) as u32
}

fn orange_count(board: &BoardDefinition) -> u32 {
    board
        .pegs
        .iter()
        .filter(|peg| peg.kind == PegKind::Orange)
        .count() as u32
}

fn clear_current_smoke_board(session: &mut VerticalSliceSession) {
    let pegs_hit = session
        .board
        .pegs
        .iter()
        .filter(|peg| peg.kind == PegKind::Orange)
        .map(|peg| peg.id.clone())
        .collect::<Vec<_>>();
    let remaining_pegs = session
        .board
        .pegs
        .iter()
        .filter(|peg| peg.kind != PegKind::Orange)
        .cloned()
        .collect::<Vec<_>>();
    let input = ShotInput {
        aim_angle_radians: SHOT_AIM_RADIANS,
        launch_speed: LAUNCH_SPEED,
        ball_id: BallId::new("balls/basic").expect("static id is valid"),
    };
    let result = ShotResult {
        events: Vec::new(),
        summary: physics_core::ShotSummary {
            ticks: 120,
            pegs_hit,
            caught_bucket: false,
            exited_board: true,
            replay_hash: format!(
                "c2-smoke-clear-{}-{}",
                session.shots.len(),
                session.board.id.as_str()
            ),
        },
        remaining_pegs,
    };

    session.apply_shot(input, result);
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv_str(mut hash: u64, value: &str) -> u64 {
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn fnv_u64(mut hash: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn fnv_i64(hash: u64, value: i64) -> u64 {
    fnv_u64(hash, value as u64)
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
        let run_summary = session.run_summary();
        assert_eq!(run_summary.final_score, session.total_score);
        assert_eq!(
            run_summary.oranges_cleared,
            session.shots[0].completion.hit_oranges as u32
        );
        assert_eq!(
            run_summary.bucket_catches,
            u32::from(session.shots[0].caught_bucket)
        );
        assert_eq!(run_summary.character_level, session.character_state.level);
        assert_eq!(run_summary.run_duration_shots, 1);
        assert_eq!(run_summary.replay_hash, session.replay_hash());
        assert_eq!(
            session.shots[0].completion.replay_hash,
            session.shots[0].replay_hash
        );
    }

    #[test]
    fn checkpoint2_smoke_session_reaches_summary_after_multiple_boards() {
        let session = run_checkpoint2_smoke_session().unwrap();
        let run_summary = session.run_summary();

        assert!(matches!(session.screen, RunScreen::Summary(_)));
        assert!(run_summary.boards_cleared >= 2);
        assert!(run_summary.run_duration_shots >= 2);
        assert_eq!(run_summary.replay_hash, session.replay_hash());
        assert_eq!(session.run_summary.as_ref().unwrap().boards_cleared, 3);
        assert_eq!(session.shots.len(), 3);
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

    #[test]
    fn act1_run_progresses_board_reward_board_boss_and_ends_with_summary() {
        let mut session = load_act1_run_session(99).unwrap();

        assert_eq!(session.nodes[0].kind, RunNodeKind::Board);
        assert_eq!(session.nodes[1].kind, RunNodeKind::Reward);
        assert_eq!(session.nodes[2].kind, RunNodeKind::Board);
        assert_eq!(session.nodes[3].kind, RunNodeKind::Boss);

        clear_current_board(&mut session, "first-clear");
        match &session.screen {
            RunScreen::RewardChoice { node_index, offer } => {
                assert_eq!(*node_index, 1);
                assert_eq!(offer.choices.len(), 3);
            }
            screen => panic!("expected reward screen after first board, got {screen:?}"),
        }

        session.choose_reward_key('3').unwrap();
        assert!(session
            .run_state
            .relics
            .iter()
            .any(|relic| relic.id.as_str() == "relics/act1/steady_bucket"));
        assert!(matches!(session.screen, RunScreen::NodeMap(_)));
        session
            .advance_from_node_map_input(NodeMapInput::Enter)
            .unwrap();
        match &session.screen {
            RunScreen::Board {
                node_index, board, ..
            } => {
                assert_eq!(*node_index, 2);
                assert_eq!(board.as_str(), "boards/feel_wave_01");
            }
            screen => panic!("expected second board, got {screen:?}"),
        }

        clear_current_board(&mut session, "second-clear");
        assert!(matches!(session.screen, RunScreen::RewardChoice { .. }));
        session.choose_reward_key('1').unwrap();
        assert!(matches!(session.screen, RunScreen::NodeMap(_)));
        session
            .advance_from_node_map_input(NodeMapInput::Enter)
            .unwrap();
        match &session.screen {
            RunScreen::Board {
                node_index, board, ..
            } => {
                assert_eq!(*node_index, 3);
                assert_eq!(board.as_str(), "boards/feel_fortress_stone_01");
            }
            screen => panic!("expected boss board, got {screen:?}"),
        }

        clear_current_board(&mut session, "boss-clear");
        match &session.screen {
            RunScreen::Summary(summary) => {
                assert_eq!(summary.reason, RunEndReason::BossCleared);
                assert_eq!(summary.boards_cleared, 3);
                assert_eq!(summary.final_score, session.total_score);
                assert_eq!(summary.xp_gained, session.character_state.xp);
                assert_eq!(summary.replay_hash, session.replay_hash());
            }
            screen => panic!("expected run summary, got {screen:?}"),
        }
        let summary_screen = session.run_summary.as_ref().unwrap();
        assert_eq!(summary_screen.boards_cleared, 3);
        assert_eq!(summary_screen.replay_hash, session.replay_hash());
    }

    #[test]
    fn board_failure_deducts_hearts_and_allows_retry_or_continue() {
        let mut session = load_act1_run_session(100).unwrap();
        session.run_state.resources.shots = 1;

        miss_current_board(&mut session, "first-miss");
        match &session.screen {
            RunScreen::Failure {
                hearts_remaining,
                oranges_remaining,
                can_retry,
                can_continue,
                ..
            } => {
                assert_eq!(*hearts_remaining, 2);
                assert!(*oranges_remaining > 0);
                assert!(*can_retry);
                assert!(*can_continue);
            }
            screen => panic!("expected failure screen, got {screen:?}"),
        }

        session.retry_failed_board().unwrap();
        assert!(matches!(
            session.screen,
            RunScreen::Board {
                node_index: 0,
                balls: 8,
                hearts: 2,
                ..
            }
        ));
        session.run_state.resources.shots = 1;
        miss_current_board(&mut session, "second-miss");
        session.continue_after_failure().unwrap();
        assert!(matches!(
            session.screen,
            RunScreen::RewardChoice { node_index: 1, .. }
        ));

        let mut lethal = load_act1_run_session(101).unwrap();
        lethal.run_state.resources.hearts = 1;
        lethal.run_state.resources.shots = 1;
        miss_current_board(&mut lethal, "lethal-miss");
        assert!(matches!(
            lethal.screen,
            RunScreen::Summary(RunSummary {
                reason: RunEndReason::HeartsDepleted,
                ..
            })
        ));
    }

    #[test]
    fn identical_reward_and_board_transitions_emit_identical_replay_hashes() {
        let mut first = load_act1_run_session(202).unwrap();
        let mut second = load_act1_run_session(202).unwrap();

        for session in [&mut first, &mut second] {
            clear_current_board(session, "first-clear");
            session.choose_reward_key('2').unwrap();
            session
                .advance_from_node_map_input(NodeMapInput::Enter)
                .unwrap();
            clear_current_board(session, "second-clear");
            session.choose_reward_key('1').unwrap();
            session
                .advance_from_node_map_input(NodeMapInput::Enter)
                .unwrap();
            clear_current_board(session, "boss-clear");
        }

        assert_eq!(first.replay_hash(), second.replay_hash());
        assert_eq!(
            first.run_summary.as_ref().unwrap().replay_hash,
            second.run_summary.as_ref().unwrap().replay_hash
        );
    }

    fn clear_current_board(session: &mut VerticalSliceSession, replay_hash: &str) {
        let hit_oranges = session
            .board
            .pegs
            .iter()
            .filter(|peg| peg.kind == PegKind::Orange)
            .map(|peg| peg.id.clone())
            .collect::<Vec<_>>();
        let remaining_pegs = session
            .board
            .pegs
            .iter()
            .filter(|peg| peg.kind != PegKind::Orange)
            .cloned()
            .collect::<Vec<_>>();
        apply_synthetic_shot(session, hit_oranges, remaining_pegs, replay_hash);
    }

    fn miss_current_board(session: &mut VerticalSliceSession, replay_hash: &str) {
        apply_synthetic_shot(session, Vec::new(), session.board.pegs.clone(), replay_hash);
    }

    fn apply_synthetic_shot(
        session: &mut VerticalSliceSession,
        pegs_hit: Vec<PegId>,
        remaining_pegs: Vec<content_schema::PegDef>,
        replay_hash: &str,
    ) {
        let input = ShotInput {
            aim_angle_radians: SHOT_AIM_RADIANS,
            launch_speed: LAUNCH_SPEED,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let result = ShotResult {
            events: Vec::new(),
            summary: ShotSummary {
                ticks: 120,
                pegs_hit,
                caught_bucket: false,
                exited_board: true,
                replay_hash: replay_hash.to_owned(),
            },
            remaining_pegs,
        };

        session.apply_shot(input, result);
    }
}
