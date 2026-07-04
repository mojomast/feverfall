use anyhow::{anyhow, Context, Result};
use content_schema::{ContentId, PegKind, Seed};
use std::env;

fn main() -> Result<()> {
    let args = Args::parse()?;
    println!(
        "seed_browser act={} archetype={} count={} seed_start={}",
        args.act,
        args.archetype.as_str(),
        args.count,
        args.seed_start
    );

    let mut valid = 0usize;
    for offset in 0..args.count {
        let seed = args.seed_start + u64::from(offset);
        let params = board_gen::GenerationParams {
            act: args.act,
            difficulty: args.act.max(1),
            archetype: args.archetype.clone(),
            seed,
            peg_budget: 12 + u16::from(args.act) * 4,
            hazard_budget: u16::from(args.act.saturating_sub(1)),
        };
        let board = board_gen::generate_board(&params);
        let report = board_gen::validate_board(&board);
        let orange = board
            .pegs
            .iter()
            .filter(|peg| peg.kind == PegKind::Orange)
            .count();
        if report.is_valid() {
            valid += 1;
        }
        println!(
            "seed={seed} board={} pegs={} orange={} valid={} issues={}",
            board.id,
            board.pegs.len(),
            orange,
            report.is_valid(),
            report.issues.len()
        );
    }

    println!("summary: {valid}/{} valid", args.count);
    Ok(())
}

struct Args {
    act: u8,
    archetype: ContentId,
    count: u32,
    seed_start: Seed,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut act = 1u8;
        let mut archetype = ContentId::new("fan").expect("static id is valid");
        let mut count = 1u32;
        let mut seed_start = 1u64;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--act" => {
                    act = args
                        .next()
                        .ok_or_else(|| anyhow!("--act requires a value"))?
                        .parse()
                        .context("--act must be an integer")?;
                }
                "--archetype" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow!("--archetype requires a value"))?;
                    archetype =
                        ContentId::new(value).context("--archetype must be a valid content id")?;
                }
                "--count" => {
                    count = args
                        .next()
                        .ok_or_else(|| anyhow!("--count requires a value"))?
                        .parse()
                        .context("--count must be an integer")?;
                }
                "--seed-start" | "--seed" => {
                    seed_start = args
                        .next()
                        .ok_or_else(|| anyhow!("--seed-start requires a value"))?
                        .parse()
                        .context("--seed-start must be an integer")?;
                }
                "--help" | "-h" => {
                    println!(
                        "usage: cargo run -p seed_browser -- [--act N] [--archetype ID] [--count N] [--seed-start N]"
                    );
                    std::process::exit(0);
                }
                _ => return Err(anyhow!("unknown argument {arg}")),
            }
        }

        Ok(Self {
            act,
            archetype,
            count,
            seed_start,
        })
    }
}
