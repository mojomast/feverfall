use content_schema::{ContentId, GearId, SkillId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterState {
    pub character_id: ContentId,
    pub level: u32,
    pub xp: u64,
    pub stats: Stats,
    pub gear: Vec<EquippedGear>,
    pub unlocked_skills: Vec<SkillState>,
    pub campaign_flags: Vec<ContentId>,
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
        });
        state
            .campaign_flags
            .push(ContentId::new("campaign/act1_slice_unlocked").expect("static id is valid"));
        state
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub aim_control: u32,
    pub bucket_control: u32,
    pub combo_focus: u32,
    pub skill_charge: u32,
    pub resilience: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
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
            aim_control: 2,
            bucket_control: 2,
            combo_focus: 1,
            skill_charge: 1,
            resilience: 2,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_state_round_trips_json() {
        let mut state = CharacterState::new(ContentId::new("characters/tester").unwrap());
        state.unlocked_skills.push(SkillState {
            id: SkillId::new("skills/zen_reroute").unwrap(),
            rank: 1,
            equipped: true,
        });

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
}
