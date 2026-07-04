mod plugins;
mod rpg_chapter1;
mod vertical_slice;

#[cfg(feature = "bevy_feel_test")]
mod feel_test;

const FULL_SMOKE_REPLAYS: &[&str] = &[
    "tests/golden_replays/minimal_test.replay.json",
    "tests/golden_replays/vertical_slice_feel_fan.replay.json",
    "tests/golden_replays/act1_twobboard_run.replay.json",
    "tests/golden_replays/rpg_chapter1_smoke.replay.json",
    "tests/golden_replays/rpg_ch1_smoke.replay.json",
    "tests/golden_replays/roguelite_3act_smoke.replay.json",
    "tests/golden_replays/roguelite_act1to3_smoke.replay.json",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if should_run_feel_test(&args) {
        run_feel_test_or_explain();
        return;
    }

    if args.iter().any(|arg| arg == "--smoke-full") {
        if let Err(error) = run_full_smoke() {
            eprintln!("smoke-full FAILED: {error}");
            std::process::exit(1);
        }
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

    let rpg_campaign = rpg_chapter1::run_rpg_campaign_smoke();
    for line in rpg_campaign.summary_lines() {
        println!("{line}");
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

    let full_roguelite = vertical_slice::run_roguelite_act1to4_smoke(0xC4A4_0000_0000_0004);
    println!("{}", full_roguelite.act4.display_line());
    println!("{}", full_roguelite.display_line());
}

fn run_full_smoke() -> Result<(), String> {
    println!("smoke-full start");

    let session = vertical_slice::run_checkpoint2_smoke_session()
        .map_err(|error| format!("C2 vertical slice smoke failed: {error}"))?;
    println!("{}", session.smoke_summary());
    println!("c2_run_summary_hash={}", session.replay_hash());

    let rpg = rpg_chapter1::run_chapter1_smoke()
        .map_err(|error| format!("RPG Chapter 1 smoke failed: {error}"))?;
    println!("{}", rpg.summary_line());
    println!("rpg_ch1_summary_hash={}", rpg.summary_hash());
    let campaign = rpg_chapter1::run_rpg_campaign_smoke();
    for line in campaign.summary_lines() {
        println!("{line}");
    }
    println!("rpg_campaign_summary_hash={}", campaign.summary_hash());

    let roguelite = vertical_slice::run_roguelite_act1to3_smoke(0xC3C0_0000_0000_0003);
    if roguelite.acts.len() != 3 || roguelite.final_state.act != 3 {
        return Err(format!(
            "roguelite base smoke expected Acts 1-3, got {} act(s) ending at Act {}",
            roguelite.acts.len(),
            roguelite.final_state.act
        ));
    }
    for act in &roguelite.acts {
        println!("{}", act.display_line());
    }
    println!("{}", roguelite.display_line());
    println!("roguelite_act1to3_summary_hash={}", roguelite.summary_hash);
    let full_roguelite = vertical_slice::run_roguelite_act1to4_smoke(0xC4A4_0000_0000_0004);
    if full_roguelite.acts.len() != 4 || full_roguelite.final_state.act != 4 {
        return Err(format!(
            "roguelite full smoke expected Acts 1-4, got {} act(s) ending at Act {}",
            full_roguelite.acts.len(),
            full_roguelite.final_state.act
        ));
    }
    println!("{}", full_roguelite.act4.display_line());
    println!("{}", full_roguelite.display_line());
    println!(
        "roguelite_full_run_summary_hash={}",
        full_roguelite.summary_hash
    );

    let feel = plugins::feel_test::run_smoke_scene()
        .map_err(|error| format!("feel-test smoke failed: {error}"))?;
    println!(
        "{} {}",
        feel.outcome_line(),
        plugins::feedback::c3_feedback_trigger_summary()
    );

    run_tool("content_linter", &[])?;
    run_tool("board_validator", &[])?;
    for replay in FULL_SMOKE_REPLAYS {
        run_tool("replay_runner", &["--replay", replay])?;
    }

    println!(
        "smoke-full summary: PASS checks={} replays={}",
        5 + FULL_SMOKE_REPLAYS.len(),
        FULL_SMOKE_REPLAYS.len()
    );
    Ok(())
}

fn run_tool(package: &str, args: &[&str]) -> Result<(), String> {
    let mut command = std::process::Command::new("cargo");
    command.args(["run", "-p", package, "--"]);
    command.args(args);
    let status = command
        .status()
        .map_err(|error| format!("failed to run {package}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{package} exited with status {status}"))
    }
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
