fn main() {
    let board = content_schema::minimal_test_board();
    let report = board_gen::validate_board(&board);
    if report.is_valid() {
        println!("board_validator placeholder: {} ok", report.board_id);
    } else {
        println!(
            "board_validator placeholder: {} issues",
            report.issues.len()
        );
        std::process::exit(1);
    }
}
