mod plugins;
mod rpg_chapter1;
mod vertical_slice;

#[cfg(feature = "bevy_feel_test")]
mod feel_test;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if should_run_feel_test(&args) {
        run_feel_test_or_explain();
        return;
    }

    let summary = plugins::register_placeholders();
    let _run_loop_api_count = vertical_slice::run_loop_registration_touchpoint();
    println!("Feverfall app skeleton ready. Enable --features bevy_feel_test for the playable Bevy feel-test scene, or pass --smoke to force this smoke path.");
    println!("{summary}");

    match vertical_slice::run_checkpoint2_smoke_session() {
        Ok(session) => {
            println!("{}", session.smoke_summary());
            let run_summary = session.run_summary();
            println!("{}", run_summary.display_line());
            let mut logger = telemetry::JsonlTelemetryLogger::new(
                std::io::stdout().lock(),
                format!("smoke-{:016x}", session.seed),
            );
            if let Err(error) = logger
                .log(run_summary.to_telemetry_event())
                .and_then(|_| logger.flush())
            {
                eprintln!("run summary telemetry unavailable: {error}");
            }
        }
        Err(error) => eprintln!("checkpoint2 vertical slice unavailable: {error}"),
    }

    match rpg_chapter1::run_chapter1_smoke() {
        Ok(session) => {
            println!("{}", session.summary_line());
            let mut logger =
                telemetry::JsonlTelemetryLogger::new(std::io::stdout().lock(), "rpg-ch1-smoke");
            for event in session.telemetry_events() {
                if let Err(error) = logger.log(event) {
                    eprintln!("rpg chapter1 telemetry unavailable: {error}");
                    break;
                }
            }
            if let Err(error) = logger.flush() {
                eprintln!("rpg chapter1 telemetry flush unavailable: {error}");
            }
        }
        Err(error) => eprintln!("rpg chapter1 smoke unavailable: {error}"),
    }

    match plugins::feel_test::run_smoke_scene() {
        Ok(scene) => println!(
            "{} {}",
            scene.outcome_line(),
            plugins::feedback::c3_feedback_trigger_summary()
        ),
        Err(error) => eprintln!("feel-test scene unavailable: {error}"),
    }

    let roguelite = vertical_slice::run_roguelite_act1to3_smoke(0xC3C0_0000_0000_0003);
    for act in &roguelite.acts {
        println!("{}", act.display_line());
    }
    println!("{}", roguelite.display_line());
}

#[cfg(feature = "bevy_feel_test")]
fn should_run_feel_test(args: &[String]) -> bool {
    !args.iter().any(|arg| arg == "--smoke")
        && (args.is_empty() || args.iter().any(|arg| arg == "--feel-test"))
}

#[cfg(not(feature = "bevy_feel_test"))]
fn should_run_feel_test(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--feel-test")
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
