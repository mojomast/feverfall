use content_schema::{ContentId, GearId, SkillId};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, io, path::Path};

pub const CAMPAIGN_SAVE_VERSION: u32 = 1;
pub const CHAPTER1_SAVE_PATH: &str = "saves/rpg/campaign.json";
pub const RPG_SAVE_DIR: &str = "saves/rpg/";
pub const RPG_BALANCE_DIR: &str = "content/balance/rpg/";
const LEVEL_THRESHOLDS: &[(u32, u64)] = &[(2, 200), (3, 500), (4, 900)];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterState {
    pub character_id: ContentId,
    pub level: u32,
    pub xp: u64,
    pub stats: Stats,
    pub gear: Vec<EquippedGear>,
    pub unlocked_skills: Vec<SkillState>,
    pub campaign_flags: Vec<ContentId>,
    #[serde(default)]
    pub available_stat_points: u32,
    #[serde(default)]
    pub inventory: GearInventory,
}

impl CharacterState {
    pub fn new(character_id: ContentId) -> Self {
        Self {
            character_id,
            level: 1,
            xp: 0,
            stats: Stats::default(),
            gear: Vec::new(),
            unlocked_skills: Vec::new(),
            campaign_flags: Vec::new(),
            available_stat_points: 0,
            inventory: GearInventory::default(),
        }
    }

    pub fn act1_slice() -> Self {
        let mut state = Self::new(ContentId::new("characters/ember").expect("static id is valid"));
        state.stats = Stats::act1_slice();
        state.gear = vec![
            EquippedGear {
                slot: GearSlot::Launcher,
                item: GearId::new("gear/act1/starter_launcher").expect("static id is valid"),
            },
            EquippedGear {
                slot: GearSlot::CoreBall,
                item: GearId::new("gear/act1/basic_core").expect("static id is valid"),
            },
        ];
        state.unlocked_skills.push(SkillState {
            id: SkillId::new("skills/act1/steady_shot").expect("static id is valid"),
            rank: 1,
            equipped: true,
            cooldown_boards: 0,
            cooldown_remaining: 0,
            used_this_board: false,
        });
        state
            .campaign_flags
            .push(ContentId::new("campaign/act1_slice_unlocked").expect("static id is valid"));
        state
    }

    pub fn chapter1() -> Self {
        let mut state = Self::new(ContentId::new("characters/ember").expect("static id is valid"));
        state.stats = Stats::chapter1_start();
        state.inventory = GearInventory {
            launchers: vec![
                GearId::new("gear/rpg_ch1/starter_launcher").expect("static id is valid"),
                GearId::new("gear/rpg_ch1/bankshot_launcher").expect("static id is valid"),
            ],
            core_balls: vec![
                GearId::new("gear/rpg_ch1/basic_core").expect("static id is valid"),
                GearId::new("gear/rpg_ch1/magnet_core").expect("static id is valid"),
            ],
        };
        state.gear = vec![
            EquippedGear {
                slot: GearSlot::Launcher,
                item: state.inventory.launchers[0].clone(),
            },
            EquippedGear {
                slot: GearSlot::CoreBall,
                item: state.inventory.core_balls[0].clone(),
            },
        ];
        state.unlocked_skills = vec![
            SkillState::ready("skills/rpg_ch1/zen_reroute"),
            SkillState::ready("skills/rpg_ch1/catch_magnet"),
        ];
        state
            .campaign_flags
            .push(ContentId::new("campaign/rpg_ch1_started").expect("static id is valid"));
        state
    }

    pub fn award_board_xp(&mut self, objective_tiers_met: u8) -> LevelUpResult {
        self.add_xp(100 + u64::from(objective_tiers_met) * 20)
    }

    pub fn add_xp(&mut self, amount: u64) -> LevelUpResult {
        let old_level = self.level;
        self.xp += amount;
        while let Some((level, _)) = LEVEL_THRESHOLDS
            .iter()
            .find(|(level, threshold)| self.level < *level && self.xp >= *threshold)
        {
            self.level = *level;
            self.available_stat_points += 1;
        }
        LevelUpResult {
            old_level,
            new_level: self.level,
            stat_points_gained: self.level.saturating_sub(old_level),
        }
    }

    pub fn allocate_stat_point(&mut self, stat: ChapterStat) -> Result<(), CharacterError> {
        if self.available_stat_points == 0 {
            return Err(CharacterError::NoStatPointsAvailable);
        }
        match stat {
            ChapterStat::Aim => self.stats.aim += 1,
            ChapterStat::Control => self.stats.control += 1,
            ChapterStat::Resonance => self.stats.resonance += 1,
            ChapterStat::Luck => self.stats.luck += 1,
        }
        self.available_stat_points -= 1;
        Ok(())
    }

    pub fn equip_gear(&mut self, slot: GearSlot, item: GearId) -> Result<(), CharacterError> {
        let available = match slot {
            GearSlot::Launcher => self.inventory.launchers.iter().any(|gear| gear == &item),
            GearSlot::CoreBall => self.inventory.core_balls.iter().any(|gear| gear == &item),
            GearSlot::BasketRig | GearSlot::Charm | GearSlot::Trinket => true,
        };
        if !available {
            return Err(CharacterError::GearNotOwned(item));
        }
        if let Some(equipped) = self.gear.iter_mut().find(|gear| gear.slot == slot) {
            equipped.item = item;
        } else {
            self.gear.push(EquippedGear { slot, item });
        }
        Ok(())
    }

    pub fn unequip_gear(&mut self, slot: GearSlot) {
        self.gear.retain(|gear| gear.slot != slot);
    }

    pub fn use_skill(&mut self, skill_id: &SkillId) -> Result<SkillUse, CharacterError> {
        let skill = self
            .unlocked_skills
            .iter_mut()
            .find(|skill| skill.id == *skill_id && skill.equipped)
            .ok_or_else(|| CharacterError::SkillUnavailable(skill_id.clone()))?;
        if skill.cooldown_remaining > 0 || skill.used_this_board {
            return Err(CharacterError::SkillOnCooldown(skill_id.clone()));
        }
        skill.used_this_board = true;
        skill.cooldown_remaining = skill.cooldown_boards;
        Ok(SkillUse {
            skill_id: skill_id.clone(),
            timing_window: SkillTimingWindow::DuringShot,
            target: if skill_id.as_str().contains("catch_magnet") {
                SkillTarget::Bucket
            } else {
                SkillTarget::Board
            },
        })
    }

    pub fn finish_board_cooldowns(&mut self) {
        for skill in &mut self.unlocked_skills {
            skill.used_this_board = false;
            skill.cooldown_remaining = skill.cooldown_remaining.saturating_sub(1);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    #[serde(default)]
    pub aim: u32,
    #[serde(default)]
    pub control: u32,
    #[serde(default)]
    pub resonance: u32,
    #[serde(default)]
    pub luck: u32,
    pub aim_control: u32,
    pub bucket_control: u32,
    pub combo_focus: u32,
    pub skill_charge: u32,
    pub resilience: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            aim: 1,
            control: 1,
            resonance: 1,
            luck: 1,
            aim_control: 1,
            bucket_control: 1,
            combo_focus: 1,
            skill_charge: 1,
            resilience: 1,
        }
    }
}

impl Stats {
    pub fn act1_slice() -> Self {
        Self {
            aim: 2,
            control: 2,
            resonance: 1,
            luck: 1,
            aim_control: 2,
            bucket_control: 2,
            combo_focus: 1,
            skill_charge: 1,
            resilience: 2,
        }
    }

    pub fn chapter1_start() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChapterStat {
    Aim,
    Control,
    Resonance,
    Luck,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GearInventory {
    pub launchers: Vec<GearId>,
    pub core_balls: Vec<GearId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquippedGear {
    pub slot: GearSlot,
    pub item: GearId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GearSlot {
    Launcher,
    CoreBall,
    BasketRig,
    Charm,
    Trinket,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillState {
    pub id: SkillId,
    pub rank: u8,
    pub equipped: bool,
    #[serde(default)]
    pub cooldown_boards: u8,
    #[serde(default)]
    pub cooldown_remaining: u8,
    #[serde(default)]
    pub used_this_board: bool,
}

impl SkillState {
    pub fn ready(id: &str) -> Self {
        Self {
            id: SkillId::new(id).expect("static id is valid"),
            rank: 1,
            equipped: true,
            cooldown_boards: 1,
            cooldown_remaining: 0,
            used_this_board: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillUse {
    pub skill_id: SkillId,
    pub timing_window: SkillTimingWindow,
    pub target: SkillTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTimingWindow {
    BeforeShot,
    DuringShot,
    AfterPegHit,
    ShotEnd,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SkillTarget {
    Board,
    Peg(ContentId),
    Ball,
    Bucket,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelUpResult {
    pub old_level: u32,
    pub new_level: u32,
    pub stat_points_gained: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharacterError {
    NoStatPointsAvailable,
    GearNotOwned(GearId),
    SkillUnavailable(SkillId),
    SkillOnCooldown(SkillId),
}

impl fmt::Display for CharacterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoStatPointsAvailable => write!(f, "no stat points available"),
            Self::GearNotOwned(gear) => write!(f, "gear not owned: {gear}"),
            Self::SkillUnavailable(skill) => write!(f, "skill unavailable: {skill}"),
            Self::SkillOnCooldown(skill) => write!(f, "skill on cooldown: {skill}"),
        }
    }
}

impl std::error::Error for CharacterError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignSave {
    pub version: u32,
    pub character: CharacterState,
}

impl CampaignSave {
    pub fn new(character: CharacterState) -> Self {
        Self {
            version: CAMPAIGN_SAVE_VERSION,
            character,
        }
    }
}

#[derive(Debug)]
pub enum CampaignSaveError {
    Io(io::Error),
    Json(serde_json::Error),
    UnknownVersion(u32),
}

impl fmt::Display for CampaignSaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Json(error) => error.fmt(f),
            Self::UnknownVersion(version) => {
                write!(f, "unknown RPG campaign save version {version}")
            }
        }
    }
}

impl std::error::Error for CampaignSaveError {}

impl From<io::Error> for CampaignSaveError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for CampaignSaveError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn load_campaign(path: impl AsRef<Path>) -> Result<Option<CharacterState>, CampaignSaveError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    let source = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&source)?;
    if let Some(version) = value.get("version").and_then(serde_json::Value::as_u64) {
        if version != u64::from(CAMPAIGN_SAVE_VERSION) {
            return Err(CampaignSaveError::UnknownVersion(version as u32));
        }
    }
    let save: CampaignSave = serde_json::from_str(&source)?;
    Ok(Some(save.character))
}

pub fn save_campaign(
    path: impl AsRef<Path>,
    character: &CharacterState,
) -> Result<(), CampaignSaveError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&CampaignSave::new(character.clone()))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_state_round_trips_json() {
        let mut state = CharacterState::new(ContentId::new("characters/tester").unwrap());
        state
            .unlocked_skills
            .push(SkillState::ready("skills/zen_reroute"));

        let json = serde_json::to_string(&state).unwrap();
        let parsed: CharacterState = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, state);
    }

    #[test]
    fn act1_slice_character_has_equipment_and_skill() {
        let state = CharacterState::act1_slice();

        assert_eq!(state.character_id.as_str(), "characters/ember");
        assert_eq!(state.level, 1);
        assert_eq!(state.stats.aim_control, 2);
        assert_eq!(state.gear.len(), 2);
        assert_eq!(state.unlocked_skills.len(), 1);
        assert!(state.unlocked_skills[0].equipped);
        assert_eq!(state.campaign_flags.len(), 1);
    }

    #[test]
    fn chapter1_awards_xp_levels_and_allocates_stats() {
        let mut state = CharacterState::chapter1();

        state.award_board_xp(3);
        let result = state.award_board_xp(2);

        assert_eq!(result.new_level, 2);
        assert_eq!(state.available_stat_points, 1);
        state.allocate_stat_point(ChapterStat::Aim).unwrap();
        assert_eq!(state.stats.aim, 2);
        assert_eq!(state.available_stat_points, 0);
    }

    #[test]
    fn gear_swaps_and_skills_are_board_cooldown_limited() {
        let mut state = CharacterState::chapter1();
        let launcher = GearId::new("gear/rpg_ch1/bankshot_launcher").unwrap();
        state
            .equip_gear(GearSlot::Launcher, launcher.clone())
            .unwrap();
        assert!(state.gear.iter().any(|gear| gear.item == launcher));

        let skill = SkillId::new("skills/rpg_ch1/zen_reroute").unwrap();
        assert!(state.use_skill(&skill).is_ok());
        assert!(matches!(
            state.use_skill(&skill),
            Err(CharacterError::SkillOnCooldown(_))
        ));
        state.finish_board_cooldowns();
        assert!(state.use_skill(&skill).is_ok());
    }

    #[test]
    fn campaign_save_load_round_trip_and_unknown_version_fails_gracefully() {
        let path =
            std::env::temp_dir().join(format!("feverfall-rpg-save-{}.json", std::process::id()));
        let state = CharacterState::chapter1();
        save_campaign(&path, &state).unwrap();

        assert_eq!(load_campaign(&path).unwrap(), Some(state));
        fs::write(&path, r#"{"version":99,"character":null}"#).unwrap();
        assert!(matches!(
            load_campaign(&path),
            Err(CampaignSaveError::UnknownVersion(99))
        ));
        let _ = fs::remove_file(path);
    }
}
