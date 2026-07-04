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
}
