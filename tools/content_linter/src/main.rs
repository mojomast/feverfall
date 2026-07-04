use anyhow::{Context, Result};
use content_schema::{BoardDefinition, ContentId};
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
        } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
            lint_json_file(&path, ids, errors)?;
        }
    }
    Ok(())
}

fn lint_json_file(path: &Path, ids: &mut BTreeSet<String>, errors: &mut Vec<String>) -> Result<()> {
    let json =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if path
        .components()
        .any(|component| component.as_os_str() == "boards")
    {
        match serde_json::from_str::<BoardDefinition>(&json) {
            Ok(board) => lint_board(&board, &path.display().to_string(), ids, errors),
            Err(error) => errors.push(format!("{} schema violation: {error}", path.display())),
        }
    } else if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
        if let Some(id) = value.get("id").and_then(|id| id.as_str()) {
            lint_id(id, &path.display().to_string(), ids, errors);
        }
    } else {
        errors.push(format!("{} is not valid JSON", path.display()));
    }
    Ok(())
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
