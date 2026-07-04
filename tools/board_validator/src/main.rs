use anyhow::{Context, Result};
use content_schema::BoardDefinition;

fn main() -> Result<()> {
    let boards = load_boards()?;
    let mut failures = 0usize;

    for board in boards {
        let report = board_gen::validate_board(&board);
        if report.is_valid() {
            println!("PASS {}", report.board_id);
        } else {
            failures += 1;
            println!("FAIL {}", report.board_id);
            for issue in report.issues {
                println!("  reason: {issue:?}");
            }
        }
    }

    if failures == 0 {
        Ok(())
    } else {
        anyhow::bail!("{failures} board(s) failed validation")
    }
}

fn load_boards() -> Result<Vec<BoardDefinition>> {
    board_gen::load_authored_boards(board_gen::authored_boards_dir())
        .context("failed to load authored boards")
}
