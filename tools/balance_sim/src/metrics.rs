use std::fmt::Write;

/// Stable RPG balance metrics emitted as deterministic JSON by the RPG simulator.
#[derive(Clone, Debug, PartialEq)]
pub struct RpgMetrics {
    pub schema_version: u32,
    pub seed_start: u64,
    pub seed_count: u32,
    pub cohorts: Vec<CohortMetrics>,
    pub content_coverage: ContentCoverage,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CohortMetrics {
    pub name: String,
    pub board_clear_rate: f64,
    pub objective_tier_rate: f64,
    pub retries_to_clear: f64,
    pub xp_level_curve: Vec<LevelPoint>,
    pub gear_acquisition_timing: Vec<GearTiming>,
    pub skill_unlock_rate: f64,
    pub skill_use_rate: f64,
    pub stat_dominance: StatDominance,
    pub chapter_completion: Vec<ChapterCompletion>,
    pub mastery_normalized_clear: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LevelPoint {
    pub level: u32,
    pub xp: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GearTiming {
    pub archetype: String,
    pub board_index: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatDominance {
    pub aim_share: f64,
    pub control_share: f64,
    pub resonance_share: f64,
    pub luck_share: f64,
    pub flags_outlier: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChapterCompletion {
    pub chapter: u8,
    pub completion_rate: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContentCoverage {
    pub chapters: u32,
    pub boards: u32,
    pub gear: u32,
    pub skills: u32,
}

pub fn stat_dominance(values: [f64; 4], outlier_ratio: f64) -> StatDominance {
    let total = values.iter().sum::<f64>().max(f64::EPSILON);
    let shares = values.map(|value| value / total);
    StatDominance {
        aim_share: shares[0],
        control_share: shares[1],
        resonance_share: shares[2],
        luck_share: shares[3],
        flags_outlier: shares.iter().any(|share| *share > outlier_ratio),
    }
}

impl RpgMetrics {
    pub fn to_stable_json(&self) -> String {
        let mut out = String::new();
        write!(
            out,
            "{{\"schema_version\":{},\"seed_start\":{},\"seed_count\":{},\"cohorts\":[",
            self.schema_version, self.seed_start, self.seed_count
        )
        .expect("writing to String cannot fail");
        for (index, cohort) in self.cohorts.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            cohort.write_json(&mut out);
        }
        write!(
            out,
            "],\"content_coverage\":{{\"chapters\":{},\"boards\":{},\"gear\":{},\"skills\":{}}}}}",
            self.content_coverage.chapters,
            self.content_coverage.boards,
            self.content_coverage.gear,
            self.content_coverage.skills
        )
        .expect("writing to String cannot fail");
        out
    }
}

impl CohortMetrics {
    fn write_json(&self, out: &mut String) {
        write!(
            out,
            "{{\"name\":\"{}\",\"board_clear_rate\":{:.4},\"objective_tier_rate\":{:.4},\"retries_to_clear\":{:.2},",
            escape(&self.name),
            self.board_clear_rate,
            self.objective_tier_rate,
            self.retries_to_clear
        )
        .expect("writing to String cannot fail");
        out.push_str("\"xp_level_curve\":[");
        for (index, point) in self.xp_level_curve.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            write!(out, "{{\"level\":{},\"xp\":{}}}", point.level, point.xp)
                .expect("writing to String cannot fail");
        }
        out.push_str("],\"gear_acquisition_timing\":[");
        for (index, timing) in self.gear_acquisition_timing.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            write!(
                out,
                "{{\"archetype\":\"{}\",\"board_index\":{}}}",
                escape(&timing.archetype),
                timing.board_index
            )
            .expect("writing to String cannot fail");
        }
        write!(
            out,
            "],\"skill_unlock_rate\":{:.4},\"skill_use_rate\":{:.4},\"stat_dominance\":{{\"aim_share\":{:.4},\"control_share\":{:.4},\"resonance_share\":{:.4},\"luck_share\":{:.4},\"flags_outlier\":{}}},\"chapter_completion\":[",
            self.skill_unlock_rate,
            self.skill_use_rate,
            self.stat_dominance.aim_share,
            self.stat_dominance.control_share,
            self.stat_dominance.resonance_share,
            self.stat_dominance.luck_share,
            self.stat_dominance.flags_outlier
        )
        .expect("writing to String cannot fail");
        for (index, chapter) in self.chapter_completion.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            write!(
                out,
                "{{\"chapter\":{},\"completion_rate\":{:.4}}}",
                chapter.chapter, chapter.completion_rate
            )
            .expect("writing to String cannot fail");
        }
        write!(
            out,
            "],\"mastery_normalized_clear\":{:.4}}}",
            self.mastery_normalized_clear
        )
        .expect("writing to String cannot fail");
    }
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpg_stat_dominance_metric_flags_outliers() {
        assert!(stat_dominance([80.0, 10.0, 5.0, 5.0], 0.45).flags_outlier);
        assert!(!stat_dominance([28.0, 26.0, 24.0, 22.0], 0.45).flags_outlier);
    }
}
