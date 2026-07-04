use content_schema::{BoardId, RelicId, Score};
use run_mode::{act1_slice_nodes, RewardOffer, RunNode, RunNodeKind, RunState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeMapRegistrationSummary {
    pub visible_nodes: usize,
    pub current_node_highlighted: bool,
    pub hidden_future_nodes: usize,
}

pub fn register() -> NodeMapRegistrationSummary {
    let mut run_state = RunState::act1_slice(0xC2_C2);
    let nodes = act1_slice_nodes();
    let screen = NodeMapScreen::from_run_state(&run_state, &nodes);
    let mut controller = NodeMapController::after_reward_choice(false);
    let _ = controller.handle_input(NodeMapInput::Other, &mut run_state, &nodes);
    let _ = controller.handle_input(NodeMapInput::Enter, &mut run_state, &nodes);
    let _ = controller.handle_input(NodeMapInput::Space, &mut run_state, &nodes);
    let mut smoke_controller = NodeMapController::after_reward_choice(true);
    let _ = smoke_controller.tick(&mut run_state, &nodes);
    let _ = RunScreen::NodeMap(screen.clone());
    let _ = RunScreen::RunComplete;

    NodeMapRegistrationSummary {
        visible_nodes: screen.rows.len(),
        current_node_highlighted: screen
            .rows
            .iter()
            .any(|row| row.status == NodeMapNodeStatus::Current && row.highlighted),
        hidden_future_nodes: screen.hidden_future_nodes,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeMapScreen {
    pub title: String,
    pub current_index: u16,
    pub rows: Vec<NodeMapRow>,
    pub hidden_future_nodes: usize,
}

impl NodeMapScreen {
    pub fn from_run_state(run_state: &RunState, nodes: &[RunNode]) -> Self {
        let current_index = run_state.node_index;
        let max_visible_index = usize::from(current_index.saturating_add(1));
        let mut hidden_future_nodes = 0;
        let mut rows = Vec::new();

        for (index, node) in nodes.iter().enumerate() {
            if index > max_visible_index {
                hidden_future_nodes += 1;
                continue;
            }

            let status = if index < usize::from(current_index) {
                NodeMapNodeStatus::Completed
            } else if index == usize::from(current_index) {
                NodeMapNodeStatus::Current
            } else {
                NodeMapNodeStatus::Next
            };

            rows.push(NodeMapRow::from_node(index as u16, node, status));
        }

        Self {
            title: format!("Act {} Map", run_state.act),
            current_index,
            rows,
            hidden_future_nodes,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RunScreen {
    Board {
        node_index: u16,
        board: BoardId,
        balls: u32,
        hearts: u32,
    },
    RewardChoice {
        node_index: u16,
        offer: RewardOffer,
    },
    Failure {
        board: BoardId,
        hearts_remaining: u32,
        oranges_remaining: u32,
        can_retry: bool,
        can_continue: bool,
    },
    Summary(RunSummary),
    NodeMap(NodeMapScreen),
    RunComplete,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunSummary {
    pub reason: RunEndReason,
    pub final_score: Score,
    pub relics_collected: Vec<RelicId>,
    pub xp_gained: u64,
    pub boards_cleared: u32,
    pub hearts_remaining: u32,
    pub replay_hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunEndReason {
    BossCleared,
    HeartsDepleted,
    PathComplete,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeMapRow {
    pub index: u16,
    pub node_id: String,
    pub board: Option<BoardId>,
    pub label: NodeMapLabel,
    pub text: String,
    pub shape: PrimitiveNodeShape,
    pub status: NodeMapNodeStatus,
    pub highlighted: bool,
    pub opacity: f32,
}

impl NodeMapRow {
    fn from_node(index: u16, node: &RunNode, status: NodeMapNodeStatus) -> Self {
        let label = NodeMapLabel::from_kind(node.kind);
        Self {
            index,
            node_id: node.id.to_string(),
            board: node.board.clone(),
            label,
            text: label.as_str().to_owned(),
            shape: PrimitiveNodeShape::from_label(label),
            status,
            highlighted: status == NodeMapNodeStatus::Current,
            opacity: match status {
                NodeMapNodeStatus::Completed => 0.35,
                NodeMapNodeStatus::Current => 1.0,
                NodeMapNodeStatus::Next => 0.72,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeMapLabel {
    Board,
    Reward,
    Boss,
    Shop,
}

impl NodeMapLabel {
    fn from_kind(kind: RunNodeKind) -> Self {
        match kind {
            RunNodeKind::Board | RunNodeKind::EliteBoard => Self::Board,
            RunNodeKind::Shop => Self::Shop,
            RunNodeKind::Boss => Self::Boss,
            RunNodeKind::Reward | RunNodeKind::Event | RunNodeKind::Forge | RunNodeKind::Camp => {
                Self::Reward
            }
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::Reward => "Reward",
            Self::Boss => "Boss",
            Self::Shop => "Shop",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveNodeShape {
    Circle,
    RoundedRect,
    Diamond,
    Capsule,
}

impl PrimitiveNodeShape {
    const fn from_label(label: NodeMapLabel) -> Self {
        match label {
            NodeMapLabel::Board => Self::Circle,
            NodeMapLabel::Reward => Self::RoundedRect,
            NodeMapLabel::Boss => Self::Diamond,
            NodeMapLabel::Shop => Self::Capsule,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeMapNodeStatus {
    Completed,
    Current,
    Next,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeMapInput {
    Enter,
    Space,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeMapTransition {
    StayOnMap,
    LoadNextBoard {
        node_index: u16,
        board: Option<BoardId>,
    },
    RunComplete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeMapController {
    smoke_mode: bool,
    ticks_visible: u32,
}

impl NodeMapController {
    pub const fn after_reward_choice(smoke_mode: bool) -> Self {
        Self {
            smoke_mode,
            ticks_visible: 0,
        }
    }

    pub fn handle_input(
        &mut self,
        input: NodeMapInput,
        run_state: &mut RunState,
        nodes: &[RunNode],
    ) -> NodeMapTransition {
        match input {
            NodeMapInput::Enter | NodeMapInput::Space => advance_to_next_node(run_state, nodes),
            NodeMapInput::Other => NodeMapTransition::StayOnMap,
        }
    }

    pub fn tick(&mut self, run_state: &mut RunState, nodes: &[RunNode]) -> NodeMapTransition {
        self.ticks_visible = self.ticks_visible.saturating_add(1);
        if self.smoke_mode && self.ticks_visible >= 1 {
            advance_to_next_node(run_state, nodes)
        } else {
            NodeMapTransition::StayOnMap
        }
    }
}

pub fn advance_to_next_node(run_state: &mut RunState, nodes: &[RunNode]) -> NodeMapTransition {
    let current_index = usize::from(run_state.node_index);
    if current_index >= nodes.len() {
        return NodeMapTransition::RunComplete;
    }

    if let Some(current_node) = nodes.get(current_index) {
        if !run_state
            .visited_nodes
            .iter()
            .any(|visited| visited.id == current_node.id)
        {
            run_state.visited_nodes.push(current_node.clone());
        }
    }

    let next_index = current_index + 1;
    if next_index >= nodes.len() {
        run_state.node_index = nodes.len() as u16;
        return NodeMapTransition::RunComplete;
    }

    run_state.node_index = next_index as u16;
    NodeMapTransition::LoadNextBoard {
        node_index: run_state.node_index,
        board: nodes[next_index].board.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_map_renders_visible_act1_nodes_only() {
        let mut run_state = RunState::act1_slice(42);
        run_state.node_index = 1;
        let nodes = act1_slice_nodes();

        let screen = NodeMapScreen::from_run_state(&run_state, &nodes);

        assert_eq!(screen.title, "Act 1 Map");
        assert_eq!(screen.rows.len(), 3);
        assert_eq!(screen.hidden_future_nodes, nodes.len() - 3);
        assert_eq!(screen.rows[0].status, NodeMapNodeStatus::Completed);
        assert_eq!(screen.rows[0].opacity, 0.35);
        assert_eq!(screen.rows[1].status, NodeMapNodeStatus::Current);
        assert!(screen.rows[1].highlighted);
        assert_eq!(screen.rows[2].status, NodeMapNodeStatus::Next);
        assert!(screen.rows.iter().all(|row| matches!(
            row.label,
            NodeMapLabel::Board | NodeMapLabel::Reward | NodeMapLabel::Boss | NodeMapLabel::Shop
        )));
    }

    #[test]
    fn node_advance_increments_position_in_run_state() {
        let mut run_state = RunState::act1_slice(42);
        let nodes = act1_slice_nodes();

        let transition = advance_to_next_node(&mut run_state, &nodes);

        assert_eq!(run_state.node_index, 1);
        assert_eq!(run_state.visited_nodes, vec![nodes[0].clone()]);
        assert_eq!(
            transition,
            NodeMapTransition::LoadNextBoard {
                node_index: 1,
                board: nodes[1].board.clone(),
            }
        );
    }

    #[test]
    fn enter_and_space_advance_from_node_map() {
        let nodes = act1_slice_nodes();
        let mut run_state = RunState::act1_slice(42);
        let mut controller = NodeMapController::after_reward_choice(false);

        assert_eq!(
            controller.handle_input(NodeMapInput::Other, &mut run_state, &nodes),
            NodeMapTransition::StayOnMap
        );
        assert_eq!(run_state.node_index, 0);

        assert!(matches!(
            controller.handle_input(NodeMapInput::Enter, &mut run_state, &nodes),
            NodeMapTransition::LoadNextBoard { node_index: 1, .. }
        ));
        assert!(matches!(
            controller.handle_input(NodeMapInput::Space, &mut run_state, &nodes),
            NodeMapTransition::LoadNextBoard { node_index: 2, .. }
        ));
    }

    #[test]
    fn smoke_mode_advances_after_one_tick() {
        let nodes = act1_slice_nodes();
        let mut run_state = RunState::act1_slice(42);
        let mut controller = NodeMapController::after_reward_choice(true);

        let transition = controller.tick(&mut run_state, &nodes);

        assert_eq!(run_state.node_index, 1);
        assert!(matches!(
            transition,
            NodeMapTransition::LoadNextBoard { node_index: 1, .. }
        ));
    }
}
