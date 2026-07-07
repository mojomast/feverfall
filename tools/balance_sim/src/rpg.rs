use crate::metrics::{
    stat_dominance, ChapterCompletion, CohortMetrics, ContentCoverage, GearTiming, LevelPoint,
    RpgMetrics,
};
use std::{collections::BTreeMap, fs, path::Path};

const RPG_SCHEMA_VERSION: u32 = 1;
const DEFAULT_SEED_START: u64 = 0xC500_0000_0000_0001;

type ProgressionTables = (Vec<LevelPoint>, BTreeMap<String, u32>, [f64; 4], f64);

#[derive(Clone, Debug, PartialEq)]
pub struct RpgBalanceTables {
    pub cohorts: Vec<CohortConfig>,
    pub xp_curve: Vec<LevelPoint>,
    pub gear_timing: BTreeMap<String, u32>,
    pub stat_weights: [f64; 4],
    pub dominance_outlier_ratio: f64,
    pub content: ContentCoverage,
    pub gear_ids: Vec<String>,
    pub skill_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CohortConfig {
    pub name: String,
    pub level_offset: i32,
    pub skill_policy: SkillPolicy,
    pub gear_archetype: String,
    pub mastery_normalized: bool,
    pub retry_budget: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillPolicy {
    None,
    SkillAware,
}

pub fn load_tables(root: &Path) -> Result<RpgBalanceTables, String> {
    let balance_dir = root.join("content/balance/rpg");
    let cohorts = parse_cohorts(&read(&balance_dir.join("cohorts.toml"))?)?;
    let (xp_curve, gear_timing, stat_weights, dominance_outlier_ratio) =
        parse_progression(&read(&balance_dir.join("progression.toml"))?)?;
    let (content, gear_ids, skill_ids) =
        parse_content(&read(&balance_dir.join("content_coverage.toml"))?)?;
    Ok(RpgBalanceTables {
        cohorts,
        xp_curve,
        gear_timing,
        stat_weights,
        dominance_outlier_ratio,
        content,
        gear_ids,
        skill_ids,
    })
}

pub fn simulate_rpg(seed_start: u64, seed_count: u32, tables: &RpgBalanceTables) -> RpgMetrics {
    let cohorts = tables
        .cohorts
        .iter()
        .map(|cohort| simulate_cohort(seed_start, seed_count, tables, cohort))
        .collect();
    RpgMetrics {
        schema_version: RPG_SCHEMA_VERSION,
        seed_start,
        seed_count,
        cohorts,
        content_coverage: tables.content.clone(),
    }
}

pub fn simulate_default_json(root: &Path) -> Result<String, String> {
    let tables = load_tables(root)?;
    Ok(simulate_rpg(DEFAULT_SEED_START, 128, &tables).to_stable_json())
}

fn simulate_cohort(
    seed_start: u64,
    seed_count: u32,
    tables: &RpgBalanceTables,
    cohort: &CohortConfig,
) -> CohortMetrics {
    let mut boards_cleared = 0u32;
    let mut objective_tiers = 0u32;
    let mut retries = 0u32;
    let mut chapter_clears = [0u32; 5];
    let total_boards = tables.content.boards * seed_count;
    let total_tiers = total_boards * 3;

    for run in 0..seed_count {
        let mut rng = Lcg::new(seed_start.wrapping_add(u64::from(run)));
        let mut completed_chapters = 0usize;
        for (chapter, chapter_clear_count) in chapter_clears.iter_mut().enumerate() {
            let board_count = chapter_board_count(chapter + 1);
            let mut chapter_clear = true;
            for board in 0..board_count {
                let clear_score = clear_score(&mut rng, cohort, chapter as u32, board);
                let won = clear_score >= 0.54;
                if won {
                    boards_cleared += 1;
                    objective_tiers += objective_tiers_for(clear_score);
                } else {
                    let attempts = retry_attempts(clear_score, cohort.retry_budget);
                    retries += attempts;
                    if attempts == cohort.retry_budget {
                        chapter_clear = false;
                    } else {
                        boards_cleared += 1;
                        objective_tiers += 1;
                    }
                }
            }
            if chapter_clear {
                completed_chapters = chapter + 1;
                *chapter_clear_count += 1;
            }
        }
        if completed_chapters == 5 && cohort.mastery_normalized {
            chapter_clears[4] += 0;
        }
    }

    let skill_unlock_rate = match cohort.skill_policy {
        SkillPolicy::None => 0.0,
        SkillPolicy::SkillAware => (0.62 + f64::from(cohort.level_offset) * 0.05).clamp(0.0, 1.0),
    };
    let skill_use_rate = if cohort.skill_policy == SkillPolicy::None {
        0.0
    } else {
        0.48 + gear_bonus(&cohort.gear_archetype) * 0.35
    }
    .clamp(0.0, 1.0);

    CohortMetrics {
        name: cohort.name.clone(),
        board_clear_rate: rate(boards_cleared, total_boards),
        objective_tier_rate: rate(objective_tiers, total_tiers),
        retries_to_clear: f64::from(retries) / f64::from(seed_count.max(1)),
        xp_level_curve: adjusted_xp_curve(&tables.xp_curve, cohort.level_offset),
        gear_acquisition_timing: vec![GearTiming {
            archetype: cohort.gear_archetype.clone(),
            board_index: *tables.gear_timing.get(&cohort.gear_archetype).unwrap_or(&1),
        }],
        skill_unlock_rate,
        skill_use_rate,
        stat_dominance: stat_dominance(
            adjusted_stat_weights(tables.stat_weights, cohort),
            tables.dominance_outlier_ratio,
        ),
        chapter_completion: chapter_clears
            .iter()
            .enumerate()
            .map(|(index, clears)| ChapterCompletion {
                chapter: (index + 1) as u8,
                completion_rate: rate(*clears, seed_count),
            })
            .collect(),
        mastery_normalized_clear: if cohort.mastery_normalized {
            rate(chapter_clears[4], seed_count)
        } else {
            0.0
        },
    }
}

fn clear_score(rng: &mut Lcg, cohort: &CohortConfig, chapter: u32, board: u32) -> f64 {
    let level = f64::from(cohort.level_offset) * 0.045;
    let skill = if cohort.skill_policy == SkillPolicy::SkillAware {
        0.055
    } else {
        -0.075
    };
    let mastery = if cohort.mastery_normalized {
        -0.04
    } else {
        0.0
    };
    let difficulty = f64::from(chapter) * 0.045 + f64::from(board % 4) * 0.012;
    (0.58 + level + skill + gear_bonus(&cohort.gear_archetype) + mastery - difficulty
        + rng.next_unit() * 0.18)
        .clamp(0.0, 1.0)
}

fn objective_tiers_for(score: f64) -> u32 {
    if score >= 0.78 {
        3
    } else if score >= 0.66 {
        2
    } else {
        1
    }
}

fn retry_attempts(score: f64, retry_budget: u32) -> u32 {
    let needed = if score >= 0.48 {
        1
    } else if score >= 0.4 {
        2
    } else {
        3
    };
    needed.min(retry_budget)
}

fn gear_bonus(archetype: &str) -> f64 {
    match archetype {
        "starter" => -0.035,
        "balanced" => 0.0,
        "synergy" => 0.045,
        "optimized" => 0.075,
        "normalized" => -0.015,
        _ => 0.0,
    }
}

fn adjusted_xp_curve(curve: &[LevelPoint], level_offset: i32) -> Vec<LevelPoint> {
    curve
        .iter()
        .map(|point| LevelPoint {
            level: point.level,
            xp: if level_offset >= 0 {
                point.xp.saturating_add(level_offset as u32 * 120)
            } else {
                point.xp.saturating_sub(level_offset.unsigned_abs() * 90)
            },
        })
        .collect()
}

fn adjusted_stat_weights(mut weights: [f64; 4], cohort: &CohortConfig) -> [f64; 4] {
    if cohort.gear_archetype == "synergy" {
        weights[2] *= 1.25;
    }
    if cohort.gear_archetype == "optimized" {
        weights[0] *= 1.2;
        weights[1] *= 1.1;
    }
    if cohort.skill_policy == SkillPolicy::None {
        weights[0] *= 1.15;
    }
    weights
}

fn chapter_board_count(chapter: usize) -> u32 {
    match chapter {
        1 => 5,
        2 => 12,
        3 => 15,
        4 => 15,
        5 => 4,
        _ => 0,
    }
}

fn rate(numerator: u32, denominator: u32) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        f64::from(numerator) / f64::from(denominator)
    }
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))
}

fn parse_cohorts(input: &str) -> Result<Vec<CohortConfig>, String> {
    let sections = sections(input);
    let mut cohorts = Vec::new();
    for (section, values) in sections {
        if let Some(name) = section.strip_prefix("cohorts.") {
            cohorts.push(CohortConfig {
                name: name.to_string(),
                level_offset: value(&values, "level_offset")?
                    .parse()
                    .map_err(|_| "bad level_offset")?,
                skill_policy: match unquote(value(&values, "skill_policy")?).as_str() {
                    "none" => SkillPolicy::None,
                    "skill_aware" => SkillPolicy::SkillAware,
                    other => return Err(format!("unknown skill policy {other}")),
                },
                gear_archetype: unquote(value(&values, "gear_archetype")?),
                mastery_normalized: value(&values, "mastery_normalized")? == "true",
                retry_budget: value(&values, "retry_budget")?
                    .parse()
                    .map_err(|_| "bad retry_budget")?,
            });
        }
    }
    cohorts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(cohorts)
}

type SectionMap = BTreeMap<String, BTreeMap<String, String>>;

fn parse_progression(input: &str) -> Result<ProgressionTables, String> {
    let sections = sections(input);
    let xp_section = sections.get("xp_curve").ok_or("missing xp_curve")?;
    let mut xp_curve = Vec::new();
    for level in 1..=5 {
        xp_curve.push(LevelPoint {
            level,
            xp: value(xp_section, &format!("level_{level}"))?
                .parse()
                .map_err(|_| "bad xp level")?,
        });
    }
    let gear_section = sections.get("gear_timing").ok_or("missing gear_timing")?;
    let gear_timing = gear_section
        .iter()
        .map(|(key, value)| Ok((key.clone(), value.parse().map_err(|_| "bad gear timing")?)))
        .collect::<Result<_, String>>()?;
    let stats = sections.get("stat_weights").ok_or("missing stat_weights")?;
    Ok((
        xp_curve,
        gear_timing,
        [
            value(stats, "aim")?.parse().map_err(|_| "bad aim")?,
            value(stats, "control")?
                .parse()
                .map_err(|_| "bad control")?,
            value(stats, "resonance")?
                .parse()
                .map_err(|_| "bad resonance")?,
            value(stats, "luck")?.parse().map_err(|_| "bad luck")?,
        ],
        value(stats, "dominance_outlier_ratio")?
            .parse()
            .map_err(|_| "bad dominance_outlier_ratio")?,
    ))
}

fn parse_content(input: &str) -> Result<(ContentCoverage, Vec<String>, Vec<String>), String> {
    let gear_ids = parse_array(input, "gear")?;
    let skill_ids = parse_array(input, "skills")?;
    let chapters = parse_array(input, "chapters")?.len() as u32;
    let sections = sections(input);
    let boards = sections
        .get("boards")
        .ok_or("missing boards")?
        .values()
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| "bad board count".to_string())
        })
        .sum::<Result<u32, String>>()?;
    Ok((
        ContentCoverage {
            chapters,
            boards,
            gear: gear_ids.len() as u32,
            skills: skill_ids.len() as u32,
        },
        gear_ids,
        skill_ids,
    ))
}

fn sections(input: &str) -> SectionMap {
    let mut current = String::new();
    let mut out: SectionMap = BTreeMap::new();
    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(']') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current = line.trim_matches(['[', ']']).to_string();
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            if !current.is_empty() {
                out.entry(current.clone()).or_default().insert(
                    key.trim().to_string(),
                    val.trim().trim_end_matches(',').to_string(),
                );
            }
        }
    }
    out
}

fn parse_array(input: &str, key: &str) -> Result<Vec<String>, String> {
    let marker = format!("{key} = [");
    let start = input
        .find(&marker)
        .ok_or_else(|| format!("missing {key}"))?
        + marker.len();
    let rest = &input[start..];
    let end = rest
        .find(']')
        .ok_or_else(|| format!("unterminated {key}"))?;
    Ok(rest[..end]
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(unquote)
        .collect())
}

fn value<'a>(map: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    map.get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing {key}"))
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

#[derive(Clone, Copy)]
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0xa076_1d64_78bd_642f)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn next_unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn rpg_balance_tables_parse() {
        let tables = load_tables(&repo_root()).expect("RPG balance tables parse");
        assert_eq!(tables.cohorts.len(), 7);
        assert_eq!(tables.content.boards, 51);
    }

    #[test]
    fn rpg_sim_seed_is_deterministic() {
        let tables = load_tables(&repo_root()).expect("RPG balance tables parse");
        let a = simulate_rpg(DEFAULT_SEED_START, 16, &tables).to_stable_json();
        let b = simulate_rpg(DEFAULT_SEED_START, 16, &tables).to_stable_json();
        assert_eq!(a, b);
    }

    #[test]
    fn rpg_metrics_schema_is_stable() {
        let json = simulate_default_json(&repo_root()).expect("RPG sim emits JSON");
        assert!(json.starts_with("{\"schema_version\":1,\"seed_start\":"));
        assert!(json.contains("\"mastery_normalized_clear\""));
        assert!(json.contains("\"content_coverage\""));
    }

    #[test]
    fn rpg_sim_covers_all_c4_gear_and_skills() {
        let tables = load_tables(&repo_root()).expect("RPG balance tables parse");
        assert!(tables
            .gear_ids
            .iter()
            .any(|id| id == "gear/rpg_ch1/starter_launcher"));
        assert!(tables
            .gear_ids
            .iter()
            .any(|id| id == "gear/rpg_ch1/bankshot_launcher"));
        assert!(tables
            .gear_ids
            .iter()
            .any(|id| id == "gear/rpg_ch1/basic_core"));
        assert!(tables
            .gear_ids
            .iter()
            .any(|id| id == "gear/rpg_ch1/magnet_core"));
        assert!(tables
            .skill_ids
            .iter()
            .any(|id| id == "skills/rpg_ch1/zen_reroute"));
        assert!(tables
            .skill_ids
            .iter()
            .any(|id| id == "skills/rpg_ch1/catch_magnet"));
    }
}
