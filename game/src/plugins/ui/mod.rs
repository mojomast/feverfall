use content_schema::{
    minimal_test_board, BallId, BoardDefinition, BoardId, ContentId, RelicId, Scalar, Score,
    SkillId, Vec2,
};
use physics_core::{predict_first_bounce, PhysicsEvent, ShotInput};
use rpg_mode::{CharacterState, SkillState};
use run_mode::{RelicInstance, RunState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiRegistrationSummary {
    pub first_bounce_predicted: bool,
    pub balls_remaining: u32,
    pub equipped_skill_count: usize,
    pub active_power_charge_percent: u8,
}

pub fn register() -> UiRegistrationSummary {
    let board = minimal_test_board();
    let input = ShotInput {
        aim_angle_radians: std::f64::consts::FRAC_PI_2,
        launch_speed: 17.5,
        ball_id: BallId::new("balls/basic").expect("static id is valid"),
    };
    let mut run_state = RunState::new(9);
    run_state.relics.push(RelicInstance {
        id: RelicId::new("relics/mock_focus").expect("static id is valid"),
        stacks: 2,
    });

    let mut character_state =
        CharacterState::new(ContentId::new("characters/mock").expect("static id is valid"));
    character_state.unlocked_skills.push(SkillState {
        id: SkillId::new("skills/zen_reroute").expect("static id is valid"),
        rank: 1,
        equipped: true,
    });

    let mut hud =
        HudState::mock_from_states(&board, &input, &run_state, &character_state, 99, 2, 3);
    hud.score.active_power = Some(ActivePowerDisplay {
        id: SkillId::new("skills/zen_reroute").expect("static id is valid"),
        label: String::from("Zen Reroute"),
        charge_percent: 75,
    });

    UiRegistrationSummary {
        first_bounce_predicted: hud.aim.first_bounce.is_some(),
        balls_remaining: hud.score.balls_remaining,
        equipped_skill_count: hud.progression.equipped_skill_count,
        active_power_charge_percent: hud
            .score
            .active_power
            .as_ref()
            .map_or(0, |power| power.charge_percent),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HudState {
    pub aim: AimHudState,
    pub score: ScoreHudState,
    pub progression: ProgressionHudState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeelTestHudState {
    pub board_id: BoardId,
    pub aim: AimHudState,
    pub replay_hash: Option<String>,
    pub balls_remaining: u32,
    pub shot_count: u32,
    pub mock_score: Score,
    pub collision_count: usize,
    pub event_log_summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeelTestHudParts {
    pub replay_hash: Option<String>,
    pub balls_remaining: u32,
    pub shot_count: u32,
    pub mock_score: Score,
    pub collision_count: usize,
    pub event_log_summary: String,
}

impl FeelTestHudState {
    pub fn from_scene_parts(
        board: &BoardDefinition,
        input: &ShotInput,
        parts: FeelTestHudParts,
    ) -> Self {
        Self {
            board_id: board.id.clone(),
            aim: AimHudState::from_board_and_input(board, input),
            replay_hash: parts.replay_hash,
            balls_remaining: parts.balls_remaining,
            shot_count: parts.shot_count,
            mock_score: parts.mock_score,
            collision_count: parts.collision_count,
            event_log_summary: parts.event_log_summary,
        }
    }

    #[cfg(test)]
    pub fn from_shot_summary(
        board: &BoardDefinition,
        input: &ShotInput,
        summary: &physics_core::ShotSummary,
        balls_remaining: u32,
        shot_count: u32,
        collision_count: usize,
        event_log_summary: impl Into<String>,
    ) -> Self {
        let mock_score =
            summary.pegs_hit.len() as Score * 100 + if summary.caught_bucket { 2_500 } else { 0 };

        Self::from_scene_parts(
            board,
            input,
            FeelTestHudParts {
                replay_hash: Some(summary.replay_hash.clone()),
                balls_remaining,
                shot_count,
                mock_score,
                collision_count,
                event_log_summary: event_log_summary.into(),
            },
        )
    }
}

impl HudState {
    pub fn mock_from_states(
        board: &BoardDefinition,
        shot_input: &ShotInput,
        run_state: &RunState,
        character_state: &CharacterState,
        score: Score,
        fever_multiplier: u32,
        combo_hits: u32,
    ) -> Self {
        Self {
            aim: AimHudState::from_board_and_input(board, shot_input),
            score: ScoreHudState::from_run_state(score, fever_multiplier, combo_hits, run_state),
            progression: ProgressionHudState::from_character_state(character_state),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AimHudState {
    pub origin: Vec2,
    pub aim_angle_radians: Scalar,
    pub launch_speed: Scalar,
    pub first_bounce: Option<FirstBounceAimLine>,
}

impl AimHudState {
    pub fn from_board_and_input(board: &BoardDefinition, input: &ShotInput) -> Self {
        Self {
            origin: board.cannon_position,
            aim_angle_radians: input.aim_angle_radians,
            launch_speed: input.launch_speed,
            first_bounce: predict_first_bounce(board, input).map(|event| {
                FirstBounceAimLine::from_collision_event(board.cannon_position, event)
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FirstBounceAimLine {
    pub start: Vec2,
    pub impact: Vec2,
    pub normal: Vec2,
    pub collision: CollisionDisplayKind,
    pub tick: u64,
}

impl FirstBounceAimLine {
    fn from_collision_event(start: Vec2, event: PhysicsEvent) -> Self {
        match event {
            PhysicsEvent::BallHitPeg {
                peg,
                position,
                normal,
                tick,
                ..
            } => Self {
                start,
                impact: position,
                normal,
                collision: CollisionDisplayKind::Peg(peg.to_string()),
                tick,
            },
            PhysicsEvent::BallHitObstacle {
                obstacle,
                position,
                normal,
                tick,
                ..
            } => Self {
                start,
                impact: position,
                normal,
                collision: CollisionDisplayKind::Obstacle(obstacle.to_string()),
                tick,
            },
            PhysicsEvent::BallEnteredBucket { ball, tick } => Self {
                start,
                impact: start,
                normal: Vec2::ZERO,
                collision: CollisionDisplayKind::Bucket(ball.to_string()),
                tick,
            },
            PhysicsEvent::BallExitedBoard { ball, tick } => Self {
                start,
                impact: start,
                normal: Vec2::ZERO,
                collision: CollisionDisplayKind::Exit(ball.to_string()),
                tick,
            },
            PhysicsEvent::ShotEnded { summary } => Self {
                start,
                impact: start,
                normal: Vec2::ZERO,
                collision: CollisionDisplayKind::ShotEnd(summary.replay_hash),
                tick: summary.ticks,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollisionDisplayKind {
    Peg(String),
    Obstacle(String),
    Bucket(String),
    Exit(String),
    ShotEnd(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreHudState {
    pub score: Score,
    pub balls_remaining: u32,
    pub fever_multiplier: u32,
    pub combo: ComboDisplay,
    pub active_power: Option<ActivePowerDisplay>,
    pub relics: Vec<RelicDisplay>,
}

impl ScoreHudState {
    pub fn from_run_state(
        score: Score,
        fever_multiplier: u32,
        combo_hits: u32,
        run_state: &RunState,
    ) -> Self {
        Self {
            score,
            balls_remaining: run_state.resources.shots,
            fever_multiplier,
            combo: ComboDisplay::from_hits(combo_hits),
            active_power: None,
            relics: run_state
                .relics
                .iter()
                .map(|relic| RelicDisplay {
                    id: relic.id.clone(),
                    stacks: relic.stacks,
                    ready: false,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboDisplay {
    pub hits: u32,
    pub label: String,
    pub visible: bool,
}

impl ComboDisplay {
    pub fn from_hits(hits: u32) -> Self {
        Self {
            hits,
            label: if hits == 0 {
                String::from("No Combo")
            } else {
                format!("Combo x{hits}")
            },
            visible: hits > 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivePowerDisplay {
    pub id: SkillId,
    pub label: String,
    pub charge_percent: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelicDisplay {
    pub id: RelicId,
    pub stacks: u32,
    pub ready: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressionHudState {
    pub character_id: ContentId,
    pub level: u32,
    pub xp: u64,
    pub equipped_skill_count: usize,
}

impl ProgressionHudState {
    pub fn from_character_state(character_state: &CharacterState) -> Self {
        Self {
            character_id: character_state.character_id.clone(),
            level: character_state.level,
            xp: character_state.xp,
            equipped_skill_count: character_state
                .unlocked_skills
                .iter()
                .filter(|skill| skill.equipped)
                .count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{minimal_test_board, BallId, ContentId, RelicId, SkillId};
    use rpg_mode::SkillState;
    use run_mode::RelicInstance;

    #[test]
    fn aim_hud_uses_physics_first_bounce_prediction() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };

        let hud = AimHudState::from_board_and_input(&board, &input);
        let predicted = predict_first_bounce(&board, &input);

        assert!(hud.first_bounce.is_some());
        assert_eq!(hud.origin, board.cannon_position);
        match (hud.first_bounce.unwrap().collision, predicted.unwrap()) {
            (CollisionDisplayKind::Peg(displayed), PhysicsEvent::BallHitPeg { peg, .. }) => {
                assert_eq!(displayed, peg.to_string());
            }
            other => panic!("unexpected first bounce mapping: {other:?}"),
        }
    }

    #[test]
    fn score_hud_can_be_driven_from_mock_run_state() {
        let mut run_state = RunState::new(7);
        run_state.resources.shots = 8;
        run_state.relics.push(RelicInstance {
            id: RelicId::new("relics/mock_focus").unwrap(),
            stacks: 2,
        });

        let hud = ScoreHudState::from_run_state(12_500, 3, 6, &run_state);

        assert_eq!(hud.score, 12_500);
        assert_eq!(hud.balls_remaining, 8);
        assert_eq!(hud.fever_multiplier, 3);
        assert_eq!(hud.combo.label, "Combo x6");
        assert!(hud.combo.visible);
        assert!(hud.active_power.is_none());
        assert_eq!(hud.relics.len(), 1);
        assert_eq!(hud.relics[0].stacks, 2);
    }

    #[test]
    fn full_hud_mock_state_uses_run_and_character_models() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let run_state = RunState::new(9);
        let mut character_state = CharacterState::new(ContentId::new("characters/mock").unwrap());
        character_state.level = 4;
        character_state.xp = 900;
        character_state.unlocked_skills.push(SkillState {
            id: SkillId::new("skills/zen_reroute").unwrap(),
            rank: 1,
            equipped: true,
        });

        let hud =
            HudState::mock_from_states(&board, &input, &run_state, &character_state, 99, 2, 3);

        assert!(hud.aim.first_bounce.is_some());
        assert_eq!(hud.score.balls_remaining, run_state.resources.shots);
        assert_eq!(hud.progression.level, 4);
        assert_eq!(hud.progression.equipped_skill_count, 1);
    }

    #[test]
    fn feel_test_hud_exposes_checkpoint_fields() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let result = physics_core::simulate_shot(123, &board, &input);

        let hud = FeelTestHudState::from_shot_summary(
            &board,
            &input,
            &result.summary,
            8,
            1,
            result.events.len(),
            "events logged",
        );

        assert_eq!(hud.board_id, board.id);
        assert_eq!(hud.shot_count, 1);
        assert_eq!(hud.balls_remaining, 8);
        assert!(hud.aim.first_bounce.is_some());
        assert_eq!(hud.replay_hash, Some(result.summary.replay_hash));
        assert!(hud.collision_count > 0);
    }
}
