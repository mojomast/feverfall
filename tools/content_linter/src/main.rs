use anyhow::{Context, Result};
use content_schema::{
    BallVariantDefinition, BoardDefinition, ContentId, GearDefinition, GearId, GearSlotDefinition,
    RelicDefinition, RpgSkillDefinition, ShopItemDefinition, SkillId,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::Path};

const CONTENT_DIR: &str = "game/assets/content";
const ROOT_CONTENT_DIR: &str = "content";

fn main() -> Result<()> {
    let mut ids = BTreeSet::new();
    let mut errors = Vec::new();
    lint_content_dir(Path::new(CONTENT_DIR), &mut ids, &mut errors)?;
    lint_optional_content_dir(Path::new(ROOT_CONTENT_DIR), &mut ids, &mut errors)?;

    if errors.is_empty() {
        println!("content lint passed: {} unique id(s)", ids.len());
        Ok(())
    } else {
        for error in &errors {
            println!("content lint error: {error}");
        }
        anyhow::bail!("content lint failed with {} error(s)", errors.len())
    }
}

fn lint_optional_content_dir(
    path: &Path,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) -> Result<()> {
    if path.exists() {
        visit(path, ids, errors)?;
    }
    Ok(())
}

fn lint_content_dir(
    path: &Path,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) -> Result<()> {
    if !path.exists() {
        println!("content directory missing; validating built-in minimal_test_board fallback");
        lint_board(
            &content_schema::minimal_test_board(),
            "minimal_test_board",
            ids,
            errors,
        );
        return Ok(());
    }

    visit(path, ids, errors)
}

fn visit(path: &Path, ids: &mut BTreeSet<String>, errors: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit(&path, ids, errors)?;
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("json" | "toml")
        ) {
            lint_content_file(&path, ids, errors)?;
        }
    }
    Ok(())
}

fn lint_content_file(
    path: &Path,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) -> Result<()> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if path
        .components()
        .any(|component| component.as_os_str() == "boards")
    {
        match serde_json::from_str::<BoardDefinition>(&source) {
            Ok(board) => lint_board(&board, &path.display().to_string(), ids, errors),
            Err(error) => errors.push(format!("{} schema violation: {error}", path.display())),
        }
    } else if path
        .components()
        .any(|component| component.as_os_str() == "relics")
    {
        match toml::from_str::<RelicDefinition>(&source) {
            Ok(relic) => lint_relic(&relic, &path.display().to_string(), ids, errors),
            Err(error) => errors.push(format!("{} schema violation: {error}", path.display())),
        }
    } else if path
        .components()
        .any(|component| component.as_os_str() == "balls")
    {
        match toml::from_str::<BallVariantDefinition>(&source) {
            Ok(ball) => lint_ball(&ball, &path.display().to_string(), ids, errors),
            Err(error) => errors.push(format!("{} schema violation: {error}", path.display())),
        }
    } else if path
        .components()
        .any(|component| component.as_os_str() == "shops")
    {
        match toml::from_str::<ShopItemDefinition>(&source) {
            Ok(item) => lint_shop_item(&item, &path.display().to_string(), ids, errors),
            Err(error) => errors.push(format!("{} schema violation: {error}", path.display())),
        }
    } else if has_path_component(path, "gear") || has_path_component(path, "rpg_gear") {
        lint_gear_source(&source, path, ids, errors);
    } else if has_path_component(path, "skills") || has_path_component(path, "rpg_skills") {
        lint_skill_source(&source, path, ids, errors);
    } else if has_path_component(path, "balance") {
        lint_balance_source(&source, path, ids, errors);
    } else if let Ok(value) = serde_json::from_str::<serde_json::Value>(&source) {
        lint_json_id_value(path, &value, ids, errors);
    } else if let Ok(value) = toml::from_str::<toml::Value>(&source) {
        if let Some(id) = value.get("id").and_then(|id| id.as_str()) {
            lint_id(id, &path.display().to_string(), ids, errors);
        }
    } else {
        errors.push(format!("{} is not valid content data", path.display()));
    }
    Ok(())
}

fn has_path_component(path: &Path, name: &str) -> bool {
    path.components()
        .any(|component| component.as_os_str() == name)
}

fn lint_json_id_value(
    path: &Path,
    value: &serde_json::Value,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    if let Some(id) = value.get("id").and_then(|id| id.as_str()) {
        lint_id(id, &path.display().to_string(), ids, errors);
    }
}

fn lint_board(
    board: &BoardDefinition,
    source: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    lint_id(board.id.as_str(), source, ids, errors);
    let mut board_local_ids = BTreeSet::new();
    for tag in &board.tags {
        if let Err(error) = ContentId::new(tag.as_str()) {
            errors.push(format!("{source} invalid tag {}: {error}", tag.as_str()));
        }
    }
    for peg in &board.pegs {
        lint_local_id(peg.id.as_str(), source, &mut board_local_ids, errors);
    }
    for obstacle in &board.obstacles {
        lint_local_id(obstacle.id.as_str(), source, &mut board_local_ids, errors);
    }
    for objective in &board.objectives {
        lint_local_id(objective.id.as_str(), source, &mut board_local_ids, errors);
        if objective.target == 0 {
            errors.push(format!(
                "{source} objective {} target must be positive",
                objective.id
            ));
        }
    }
    if board.tags.iter().any(|tag| tag.as_str() == "boss") && board.boss_mechanic.is_none() {
        errors.push(format!("{source} boss board requires boss_mechanic"));
    }
    if let Some(mechanic) = &board.boss_mechanic {
        if mechanic.intensity == 0 || mechanic.cadence_shots == 0 {
            errors.push(format!(
                "{source} boss_mechanic intensity and cadence_shots must be positive"
            ));
        }
        for parameter in &mechanic.parameters {
            lint_reference_id(parameter.as_str(), source, errors);
        }
    }
}

fn lint_relic(
    relic: &RelicDefinition,
    source: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    lint_id(relic.id.as_str(), source, ids, errors);
    if relic.act == 0 {
        errors.push(format!("{source} relic act must be at least 1"));
    }
    if relic.name.trim().is_empty() || relic.description.trim().is_empty() {
        errors.push(format!("{source} relic requires name and description"));
    }
    for effect in &relic.effects {
        lint_reference_id(effect.as_str(), source, errors);
    }
}

fn lint_ball(
    ball: &BallVariantDefinition,
    source: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    lint_id(ball.id.as_str(), source, ids, errors);
    lint_reference_id(ball.family.as_str(), source, errors);
    if ball.name.trim().is_empty() || ball.description.trim().is_empty() {
        errors.push(format!("{source} ball requires name and description"));
    }
    if ball.stats.radius <= 0.0
        || ball.stats.mass <= 0.0
        || ball.stats.launch_speed_multiplier <= 0.0
        || ball.stats.restitution_multiplier <= 0.0
    {
        errors.push(format!("{source} ball stats must be positive"));
    }
    for effect in &ball.effects {
        lint_reference_id(effect.as_str(), source, errors);
    }
}

fn lint_shop_item(
    item: &ShopItemDefinition,
    source: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    lint_id(item.id.as_str(), source, ids, errors);
    lint_reference_id(item.grants.as_str(), source, errors);
    if item.act == 0 || item.price_coins == 0 || item.stock_weight == 0 {
        errors.push(format!(
            "{source} shop item act, price, and stock weight must be positive"
        ));
    }
    if item.name.trim().is_empty() || item.description.trim().is_empty() {
        errors.push(format!("{source} shop item requires name and description"));
    }
}

fn lint_gear_source(
    source_text: &str,
    path: &Path,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let source = path.display().to_string();
    if let Ok(gear) = toml::from_str::<GearDefinition>(source_text) {
        lint_gear(
            gear.id.as_str(),
            &gear.name,
            &gear.description,
            &gear.effects,
            &source,
            ids,
            errors,
        );
    } else if let Ok(gear) = serde_json::from_str::<RpgGearContent>(source_text) {
        lint_gear(
            gear.id.as_str(),
            &gear.name,
            &gear.description,
            &gear.effects,
            &source,
            ids,
            errors,
        );
    } else {
        errors.push(format!("{} RPG gear schema violation", path.display()));
    }
}

fn lint_skill_source(
    source_text: &str,
    path: &Path,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let source = path.display().to_string();
    if let Ok(skill) = toml::from_str::<RpgSkillDefinition>(source_text) {
        lint_skill(
            SkillLintInput {
                id: skill.id.as_str(),
                name: &skill.name,
                description: &skill.description,
                max_rank: skill.max_rank,
                effects: &skill.effects,
            },
            &source,
            ids,
            errors,
        );
    } else if let Ok(skill) = serde_json::from_str::<RpgSkillContent>(source_text) {
        lint_skill(
            SkillLintInput {
                id: skill.id.as_str(),
                name: &skill.name,
                description: &skill.description,
                max_rank: skill.max_rank.unwrap_or(1),
                effects: &skill.effects,
            },
            &source,
            ids,
            errors,
        );
    } else {
        errors.push(format!("{} RPG skill schema violation", path.display()));
    }
}

fn lint_balance_source(
    source_text: &str,
    path: &Path,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let source = path.display().to_string();
    match toml::from_str::<toml::Value>(source_text) {
        Ok(value) => lint_balance_value(&value, &source, ids, errors),
        Err(error) => errors.push(format!(
            "{} balance schema violation: {error}",
            path.display()
        )),
    }
}

fn lint_gear(
    id: &str,
    name: &str,
    description: &str,
    effects: &[ContentId],
    source: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    lint_id(id, source, ids, errors);
    if name.trim().is_empty() || description.trim().is_empty() {
        errors.push(format!("{source} gear requires name and description"));
    }
    for effect in effects {
        lint_reference_id(effect.as_str(), source, errors);
    }
}

struct SkillLintInput<'a> {
    id: &'a str,
    name: &'a str,
    description: &'a str,
    max_rank: u8,
    effects: &'a [ContentId],
}

fn lint_skill(
    skill: SkillLintInput<'_>,
    source: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    lint_id(skill.id, source, ids, errors);
    if skill.name.trim().is_empty() || skill.description.trim().is_empty() {
        errors.push(format!("{source} skill requires name and description"));
    }
    if skill.max_rank == 0 {
        errors.push(format!("{source} skill max_rank must be positive"));
    }
    for effect in skill.effects {
        lint_reference_id(effect.as_str(), source, errors);
    }
}

fn lint_balance_value(
    table: &toml::Value,
    source: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let Some(id) = table.get("id").and_then(|id| id.as_str()) else {
        errors.push(format!("{source} balance table requires id"));
        return;
    };
    lint_id(id, source, ids, errors);
    if table
        .get("version")
        .and_then(|version| version.as_str())
        .is_none_or(str::is_empty)
    {
        errors.push(format!("{source} balance table requires version"));
    }
    if table.as_table().is_none_or(|table| table.len() <= 2) {
        errors.push(format!("{source} balance table requires tunable entries"));
    }
}

#[derive(Deserialize)]
struct RpgGearContent {
    id: GearId,
    name: String,
    #[allow(dead_code)]
    slot: GearSlotDefinition,
    description: String,
    #[serde(default)]
    effects: Vec<ContentId>,
}

#[derive(Deserialize)]
struct RpgSkillContent {
    id: SkillId,
    name: String,
    description: String,
    #[serde(default)]
    max_rank: Option<u8>,
    #[serde(default)]
    effects: Vec<ContentId>,
}

fn lint_id(id: &str, source: &str, ids: &mut BTreeSet<String>, errors: &mut Vec<String>) {
    if let Err(error) = ContentId::new(id) {
        errors.push(format!("{source} invalid id {id}: {error}"));
    }
    if !ids.insert(id.to_string()) {
        errors.push(format!("duplicate content id {id} in {source}"));
    }
}

fn lint_local_id(id: &str, source: &str, ids: &mut BTreeSet<String>, errors: &mut Vec<String>) {
    if let Err(error) = ContentId::new(id) {
        errors.push(format!("{source} invalid id {id}: {error}"));
    }
    if !ids.insert(id.to_string()) {
        errors.push(format!("duplicate board-local id {id} in {source}"));
    }
}

fn lint_reference_id(id: &str, source: &str, errors: &mut Vec<String>) {
    if let Err(error) = ContentId::new(id) {
        errors.push(format!("{source} invalid reference id {id}: {error}"));
    }
}
