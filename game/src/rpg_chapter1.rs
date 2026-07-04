use std::fmt;

use board_gen::{authored_boards_dir, load_authored_boards, BoardLoadError};
use content_schema::{BoardDefinition, BoardId, ContentId, GearId, PegKind, Score, SkillId};
use game_rules::{GameEvent, ResourceKind};
use rpg_mode::{
    load_campaign, save_campaign, CampaignSaveError, ChapterStat, CharacterError, CharacterState,
    GearSlot, CHAPTER1_SAVE_PATH,
};
use telemetry::TelemetryEvent;

const COMPLETE_FLAG: &str = "campaign/rpg_ch1_smoke_complete";

#[derive(Clone, Debug, PartialEq)]
pub struct Chapter1Session {
    pub character: CharacterState,
    pub boards: Vec<ChapterBoardResult>,
    pub skill_uses: Vec<SkillId>,
    pub events: Vec<GameEvent>,
    pub loaded_from_save: bool,
}

impl Chapter1Session {
    pub fn summary_hash(&self) -> String {
        let mut hash = FNV_OFFSET;
        hash = fnv_u64(hash, u64::from(self.character.level));
        hash = fnv_u64(hash, self.character.xp);
        hash = fnv_u64(hash, u64::from(self.character.stats.aim));
        hash = fnv_u64(hash, u64::from(self.character.stats.control));
        hash = fnv_u64(hash, u64::from(self.character.stats.resonance));
        hash = fnv_u64(hash, u64::from(self.character.stats.luck));
        for gear in &self.character.gear {
            hash = fnv_str(hash, gear.item.as_str());
        }
        for board in &self.boards {
            hash = fnv_str(hash, board.board.as_str());
            hash = fnv_u64(hash, u64::from(board.objective_tiers_met));
            hash = fnv_i64(hash, board.score);
            hash = fnv_u64(hash, u64::from(board.oranges_cleared));
            hash = fnv_u64(hash, u64::from(board.balls_used));
        }
        for skill in &self.skill_uses {
            hash = fnv_str(hash, skill.as_str());
        }
        format!("{hash:016x}")
    }

    pub fn summary_line(&self) -> String {
        let gear = self
            .character
            .gear
            .iter()
            .map(|gear| format!("{:?}={}", gear.slot, gear.item))
            .collect::<Vec<_>>()
            .join(",");
        let skills = self
            .skill_uses
            .iter()
            .map(|skill| skill.as_str())
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "rpg_ch1 smoke boards={} xp={} level={} stats=aim:{}/control:{}/resonance:{}/luck:{} gear=[{}] skill_uses=[{}] hash={} save={}",
            self.boards
                .iter()
                .map(|board| board.board.as_str())
                .collect::<Vec<_>>()
                .join(","),
            self.character.xp,
            self.character.level,
            self.character.stats.aim,
            self.character.stats.control,
            self.character.stats.resonance,
            self.character.stats.luck,
            gear,
            skills,
            self.summary_hash(),
            CHAPTER1_SAVE_PATH,
        )
    }

    pub fn telemetry_events(&self) -> Vec<TelemetryEvent> {
        self.skill_uses
            .iter()
            .cloned()
            .map(|skill| TelemetryEvent::SkillUsed { skill })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChapterBoardResult {
    pub board: BoardId,
    pub objective: ChapterObjective,
    pub objective_tiers_met: u8,
    pub score: Score,
    pub oranges_cleared: u32,
    pub balls_used: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChapterObjective {
    ClearAllOrange,
    ScoreAtLeast(Score),
    CatchStreak(u32),
    ClearAllOrangeWithinBalls(u32),
}

pub fn chapter1_board_catalog() -> Vec<(&'static str, ChapterObjective)> {
    vec![
        ("boards/rpg_ch1_01", ChapterObjective::ClearAllOrange),
        ("boards/rpg_ch1_02", ChapterObjective::ScoreAtLeast(15_000)),
        ("boards/rpg_ch1_03", ChapterObjective::CatchStreak(3)),
        (
            "boards/rpg_ch1_04",
            ChapterObjective::ClearAllOrangeWithinBalls(8),
        ),
        ("boards/rpg_ch1_05", ChapterObjective::ClearAllOrange),
    ]
}

#[derive(Debug)]
pub enum Chapter1Error {
    BoardLoad(BoardLoadError),
    MissingBoard(&'static str),
    Save(CampaignSaveError),
    Character(CharacterError),
}

impl fmt::Display for Chapter1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoardLoad(error) => error.fmt(f),
            Self::MissingBoard(board) => write!(f, "missing Chapter 1 board {board}"),
            Self::Save(error) => error.fmt(f),
            Self::Character(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Chapter1Error {}

impl From<BoardLoadError> for Chapter1Error {
    fn from(value: BoardLoadError) -> Self {
        Self::BoardLoad(value)
    }
}

impl From<CampaignSaveError> for Chapter1Error {
    fn from(value: CampaignSaveError) -> Self {
        Self::Save(value)
    }
}

impl From<CharacterError> for Chapter1Error {
    fn from(value: CharacterError) -> Self {
        Self::Character(value)
    }
}

pub fn run_chapter1_smoke() -> Result<Chapter1Session, Chapter1Error> {
    run_chapter1_smoke_at(CHAPTER1_SAVE_PATH)
}

pub fn run_chapter1_smoke_at(save_path: &str) -> Result<Chapter1Session, Chapter1Error> {
    let _catalog = chapter1_board_catalog();
    let authored = load_authored_boards(authored_boards_dir())?;
    let board1 = find_board(&authored, "boards/rpg_ch1_01")?;
    let board5 = find_board(&authored, "boards/rpg_ch1_05")?;
    let loaded = load_campaign(save_path)?;
    let loaded_from_save = loaded.is_some();
    let mut character = loaded.unwrap_or_else(CharacterState::chapter1);
    let already_complete = character
        .campaign_flags
        .iter()
        .any(|flag| flag.as_str() == COMPLETE_FLAG);

    character.equip_gear(
        GearSlot::Launcher,
        GearId::new("gear/rpg_ch1/bankshot_launcher").expect("static id is valid"),
    )?;
    character.equip_gear(
        GearSlot::CoreBall,
        GearId::new("gear/rpg_ch1/magnet_core").expect("static id is valid"),
    )?;

    let mut session = Chapter1Session {
        character,
        boards: Vec::new(),
        skill_uses: Vec::new(),
        events: Vec::new(),
        loaded_from_save,
    };

    complete_smoke_board(
        &mut session,
        &board1,
        ChapterObjective::ClearAllOrange,
        3,
        20_000,
        1,
    )?;
    complete_smoke_board(
        &mut session,
        &board5,
        ChapterObjective::ClearAllOrange,
        1,
        28_000,
        8,
    )?;

    if !already_complete {
        if session.character.available_stat_points > 0 {
            session.character.allocate_stat_point(ChapterStat::Aim)?;
        }
        session
            .character
            .campaign_flags
            .push(ContentId::new(COMPLETE_FLAG).expect("static id is valid"));
    }
    save_campaign(save_path, &session.character)?;
    Ok(session)
}

fn complete_smoke_board(
    session: &mut Chapter1Session,
    board: &BoardDefinition,
    objective: ChapterObjective,
    objective_tiers_met: u8,
    score: Score,
    balls_used: u32,
) -> Result<(), Chapter1Error> {
    let use_skill = match board.id.as_str() {
        "boards/rpg_ch1_01" => Some("skills/rpg_ch1/zen_reroute"),
        "boards/rpg_ch1_05" => Some("skills/rpg_ch1/catch_magnet"),
        _ => None,
    };
    if let Some(skill) = use_skill {
        let skill = SkillId::new(skill).expect("static id is valid");
        let _ = session.character.use_skill(&skill)?;
        session.events.push(GameEvent::SkillUsed {
            skill: skill.clone(),
        });
        session.skill_uses.push(skill);
    }

    let oranges_cleared = board
        .pegs
        .iter()
        .filter(|peg| peg.kind == PegKind::Orange)
        .count() as u32;
    session.events.push(GameEvent::BoardWon {
        board: board.id.clone(),
        final_score: score,
    });
    session.events.push(GameEvent::ResourceChanged {
        resource: ResourceKind::Xp,
        delta: i64::from(100 + u32::from(objective_tiers_met) * 20),
    });
    session.boards.push(ChapterBoardResult {
        board: board.id.clone(),
        objective,
        objective_tiers_met,
        score,
        oranges_cleared,
        balls_used,
    });

    let already_complete = session
        .character
        .campaign_flags
        .iter()
        .any(|flag| flag.as_str() == COMPLETE_FLAG);
    if !already_complete {
        session.character.award_board_xp(objective_tiers_met);
    }
    session.character.finish_board_cooldowns();
    Ok(())
}

fn find_board(
    boards: &[BoardDefinition],
    id: &'static str,
) -> Result<BoardDefinition, Chapter1Error> {
    boards
        .iter()
        .find(|board| board.id.as_str() == id)
        .cloned()
        .ok_or(Chapter1Error::MissingBoard(id))
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

    #[test]
    fn chapter1_smoke_uses_boards_one_and_five_and_is_stable_after_save_load() {
        let path = std::env::temp_dir().join(format!(
            "feverfall-rpg-ch1-smoke-{}.json",
            std::process::id()
        ));
        let path = path.to_string_lossy().to_string();
        let first = run_chapter1_smoke_at(&path).unwrap();
        let second = run_chapter1_smoke_at(&path).unwrap();

        assert_eq!(first.boards[0].board.as_str(), "boards/rpg_ch1_01");
        assert_eq!(first.boards[1].board.as_str(), "boards/rpg_ch1_05");
        assert_eq!(first.character.xp, 280);
        assert_eq!(first.character.level, 2);
        assert_eq!(first.character.stats.aim, 2);
        assert_eq!(first.skill_uses.len(), 2);
        assert!(second.loaded_from_save);
        assert_eq!(first.summary_hash(), second.summary_hash());
        let _ = std::fs::remove_file(path);
    }
}
