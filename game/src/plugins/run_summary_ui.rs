use content_schema::{RelicId, Score};
use rpg_mode::CharacterState;
use run_mode::RunState;
use telemetry::TelemetryEvent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunSummaryScreen {
    pub final_score: Score,
    pub boards_cleared: u32,
    pub oranges_cleared: u32,
    pub bucket_catches: u32,
    pub relics_collected: Vec<RelicId>,
    pub xp_gained: u64,
    pub character_level: u32,
    pub run_duration_shots: u32,
    pub replay_hash: String,
}

impl RunSummaryScreen {
    pub fn from_run_end(
        run_state: &RunState,
        character_state: &CharacterState,
        initial_xp: u64,
        outcome: RunSummaryOutcome,
    ) -> Self {
        Self {
            final_score: outcome.final_score,
            boards_cleared: outcome.boards_cleared,
            oranges_cleared: outcome.oranges_cleared,
            bucket_catches: outcome.bucket_catches,
            relics_collected: run_state
                .relics
                .iter()
                .map(|relic| relic.id.clone())
                .collect(),
            xp_gained: character_state.xp.saturating_sub(initial_xp),
            character_level: character_state.level,
            run_duration_shots: outcome.run_duration_shots,
            replay_hash: outcome.replay_hash,
        }
    }

    pub fn display_line(&self) -> String {
        let relics = if self.relics_collected.is_empty() {
            String::from("none")
        } else {
            self.relics_collected
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };

        format!(
            "run_summary final_score={} boards_cleared={} oranges_cleared={} bucket_catches={} relics=[{}] xp_gained={} character_level={} duration_shots={} replay_hash={}",
            self.final_score,
            self.boards_cleared,
            self.oranges_cleared,
            self.bucket_catches,
            relics,
            self.xp_gained,
            self.character_level,
            self.run_duration_shots,
            self.replay_hash,
        )
    }

    pub fn to_telemetry_event(&self) -> TelemetryEvent {
        TelemetryEvent::RunEnded {
            final_score: self.final_score,
            boards_cleared: self.boards_cleared,
            oranges_cleared: self.oranges_cleared,
            bucket_catches: self.bucket_catches,
            relics_collected: self.relics_collected.clone(),
            xp_gained: self.xp_gained,
            character_level: self.character_level,
            run_duration_shots: self.run_duration_shots,
            replay_hash: self.replay_hash.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunSummaryOutcome {
    pub final_score: Score,
    pub boards_cleared: u32,
    pub oranges_cleared: u32,
    pub bucket_catches: u32,
    pub run_duration_shots: u32,
    pub replay_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{ContentId, RelicId};
    use run_mode::RelicInstance;

    #[test]
    fn run_summary_fields_are_populated_from_run_and_character_state() {
        let mut run_state = RunState::new(42);
        run_state.relics.push(RelicInstance {
            id: RelicId::new("relics/test/catcher").unwrap(),
            stacks: 1,
        });
        let mut character_state = CharacterState::new(ContentId::new("characters/tester").unwrap());
        character_state.level = 3;
        character_state.xp = 27;

        let summary = RunSummaryScreen::from_run_end(
            &run_state,
            &character_state,
            5,
            RunSummaryOutcome {
                final_score: 12_500,
                boards_cleared: 2,
                oranges_cleared: 17,
                bucket_catches: 3,
                run_duration_shots: 9,
                replay_hash: "sessionhash".to_owned(),
            },
        );

        assert_eq!(summary.final_score, 12_500);
        assert_eq!(summary.boards_cleared, 2);
        assert_eq!(summary.oranges_cleared, 17);
        assert_eq!(summary.bucket_catches, 3);
        assert_eq!(summary.relics_collected.len(), 1);
        assert_eq!(summary.xp_gained, 22);
        assert_eq!(summary.character_level, 3);
        assert_eq!(summary.run_duration_shots, 9);
        assert_eq!(summary.replay_hash, "sessionhash");
        assert!(summary
            .display_line()
            .contains("run_summary final_score=12500"));
    }
}
