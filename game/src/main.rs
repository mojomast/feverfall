mod plugins;

#[cfg(feature = "bevy_feel_test")]
mod feel_test;

fn main() {
    if std::env::args().any(|arg| arg == "--feel-test") {
        run_feel_test_or_explain();
        return;
    }

    let summary = plugins::register_placeholders();
    println!("Feverfall app skeleton ready. Use --features bevy_feel_test -- --feel-test for the playable Bevy feel-test scene.");
    println!("{summary}");

    match plugins::feel_test::run_smoke_scene() {
        Ok(scene) => println!("{}", scene.outcome_line()),
        Err(error) => eprintln!("feel-test scene unavailable: {error}"),
    }
}

#[cfg(feature = "bevy_feel_test")]
fn run_feel_test_or_explain() {
    feel_test::run();
}

#[cfg(not(feature = "bevy_feel_test"))]
fn run_feel_test_or_explain() {
    eprintln!(
        "The Bevy feel-test scene is feature-gated. Run: cargo run -p feverfall_game --features bevy_feel_test -- --feel-test"
    );
}
