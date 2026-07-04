use anyhow::{anyhow, Context, Result};
use content_schema::{BoardObjective, BoardObjectiveKind, ContentId, PegKind, Seed};
use std::env;

fn main() -> Result<()> {
    let args = Args::parse()?;
    println!(
        "seed_browser mode={} act={} chapter={} archetype={} count={} seed_start={}",
        args.mode.as_str(),
        args.act,
        args.chapter,
        args.archetype.as_str(),
        args.count,
        args.seed_start
    );

    let mut valid = 0usize;
    for offset in 0..args.count {
        let seed = args.seed_start + u64::from(offset);
        let act = if args.mode == BrowserMode::Rpg {
            args.chapter
        } else {
            args.act
        };
        let params = board_gen::GenerationParams {
            act,
            difficulty: act.max(1),
            archetype: args.archetype.clone(),
            seed,
            peg_budget: 12 + u16::from(act) * 4,
            hazard_budget: u16::from(act.saturating_sub(1)),
        };
        let mut board = board_gen::generate_board(&params);
        if args.mode == BrowserMode::Rpg {
            board
                .tags
                .push(ContentId::new(format!("rpg/chapter{}", args.chapter)).unwrap());
            board.objectives.push(BoardObjective {
                id: ContentId::new(format!(
                    "objectives/rpg/chapter{}/clear_oranges",
                    args.chapter
                ))
                .unwrap(),
                kind: BoardObjectiveKind::ClearOrangePegs,
                target: board
                    .pegs
                    .iter()
                    .filter(|peg| peg.kind == PegKind::Orange)
                    .count() as u32,
                description: Some(format!("Clear Chapter {} orange pegs", args.chapter)),
            });
        }
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
            "seed={seed} board={} pegs={} orange={} objectives={} valid={} issues={}",
            board.id,
            board.pegs.len(),
            orange,
            board.objectives.len(),
            report.is_valid(),
            report.issues.len()
        );
    }

    println!("summary: {valid}/{} valid", args.count);
    Ok(())
}

struct Args {
    mode: BrowserMode,
    act: u8,
    chapter: u8,
    archetype: ContentId,
    count: u32,
    seed_start: Seed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BrowserMode {
    Roguelite,
    Rpg,
}

impl BrowserMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Roguelite => "roguelite",
            Self::Rpg => "rpg",
        }
    }
}

impl Args {
    fn parse() -> Result<Self> {
        let mut mode = BrowserMode::Roguelite;
        let mut act = 1u8;
        let mut chapter = 1u8;
        let mut archetype = ContentId::new("fan").expect("static id is valid");
        let mut count = 1u32;
        let mut seed_start = 1u64;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--mode" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow!("--mode requires a value"))?;
                    mode = match value.as_str() {
                        "roguelite" | "run" => BrowserMode::Roguelite,
                        "rpg" | "campaign" => BrowserMode::Rpg,
                        _ => return Err(anyhow!("--mode must be roguelite or rpg")),
                    };
                }
                "--act" => {
                    act = args
                        .next()
                        .ok_or_else(|| anyhow!("--act requires a value"))?
                        .parse()
                        .context("--act must be an integer")?;
                }
                "--chapter" => {
                    chapter = args
                        .next()
                        .ok_or_else(|| anyhow!("--chapter requires a value"))?
                        .parse()
                        .context("--chapter must be an integer")?;
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
                        "usage: cargo run -p seed_browser -- [--mode roguelite|rpg] [--act N] [--chapter N] [--archetype ID] [--count N] [--seed-start N]"
                    );
                    std::process::exit(0);
                }
                _ => return Err(anyhow!("unknown argument {arg}")),
            }
        }

        Ok(Self {
            mode,
            act,
            chapter,
            archetype,
            count,
            seed_start,
        })
    }
}
