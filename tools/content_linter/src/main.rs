use anyhow::{Context, Result};
use content_schema::{
    BallVariantDefinition, BoardDefinition, ContentId, RelicDefinition, ShopItemDefinition,
};
use std::{collections::BTreeSet, fs, path::Path};

const CONTENT_DIR: &str = "game/assets/content";

fn main() -> Result<()> {
    let mut ids = BTreeSet::new();
    let mut errors = Vec::new();
    lint_content_dir(Path::new(CONTENT_DIR), &mut ids, &mut errors)?;

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
