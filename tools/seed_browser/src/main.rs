fn main() {
    let params = board_gen::GenerationParams {
        act: 1,
        difficulty: 1,
        archetype: content_schema::ContentId::new("archetypes/seed_browser").unwrap(),
        seed: 1,
        peg_budget: 12,
        hazard_budget: 0,
    };
    let board = board_gen::generate_board(&params);
    println!("seed_browser placeholder: generated {}", board.id);
}
