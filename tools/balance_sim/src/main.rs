use anyhow::{anyhow, bail, Context, Result};
use content_schema::Seed;
use std::{env, path::Path};

pub mod metrics;
pub mod roguelite;
pub mod rpg;

const DEFAULT_ROGUELITE_SEED: Seed = 0xC43A_0000_0000_0000;
const DEFAULT_RPG_SEED: u64 = 0xC500_0000_0000_0001;
const DEFAULT_RPG_RUNS: u32 = 128;
const SMOKE_RUNS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    All,
    Roguelite,
    Rpg,
}

#[derive(Debug)]
struct Cli {
    mode: Mode,
    smoke: bool,
    runs: Option<usize>,
    seed: Option<u64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse(env::args().skip(1))?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root
        .canonicalize()
        .context("canonicalizing workspace root for balance tables")?;

    match cli.mode {
        Mode::All => {
            println!("{}", run_roguelite_json(&root, &cli)?);
            println!("{}", run_rpg_json(&root, &cli)?);
        }
        Mode::Roguelite => println!("{}", run_roguelite_json(&root, &cli)?),
        Mode::Rpg => println!("{}", run_rpg_json(&root, &cli)?),
    }

    Ok(())
}

fn run_roguelite_json(root: &Path, cli: &Cli) -> Result<String> {
    let tables =
        roguelite::RogueliteBalanceTables::load_from_dir(root.join("content/balance/roguelite"))
            .context("loading roguelite balance tables")?;
    let runs = cli.runs.unwrap_or(if cli.smoke { SMOKE_RUNS } else { 0 });
    let seed = cli.seed.unwrap_or(DEFAULT_ROGUELITE_SEED);
    Ok(roguelite::simulate_roguelite(&tables, runs, seed).to_json())
}

fn run_rpg_json(root: &Path, cli: &Cli) -> Result<String> {
    let tables = rpg::load_tables(root).map_err(|err| anyhow!(err))?;
    let seed_count = cli.runs.unwrap_or(if cli.smoke {
        SMOKE_RUNS
    } else {
        DEFAULT_RPG_RUNS as usize
    });
    let seed_count = u32::try_from(seed_count).context("RPG run count exceeds u32 range")?;
    let seed = cli.seed.unwrap_or(DEFAULT_RPG_SEED);
    Ok(rpg::simulate_rpg(seed, seed_count, &tables).to_stable_json())
}

impl Cli {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut cli = Self {
            mode: Mode::All,
            smoke: false,
            runs: None,
            seed: None,
        };
        let mut args = args.into_iter().peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--mode" => cli.mode = parse_mode(next_value(&mut args, "--mode")?)?,
                "--roguelite" | "roguelite" => cli.mode = Mode::Roguelite,
                "--rpg" | "rpg" => cli.mode = Mode::Rpg,
                "--all" | "all" => cli.mode = Mode::All,
                "--smoke" => cli.smoke = true,
                "--runs" => cli.runs = Some(parse_usize(next_value(&mut args, "--runs")?)?),
                "--seed" => cli.seed = Some(parse_seed(next_value(&mut args, "--seed")?)?),
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => bail!("unknown balance_sim argument `{other}`; try --help"),
            }
        }
        Ok(cli)
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{flag} requires a following value"))
}

fn parse_mode(value: String) -> Result<Mode> {
    match value.as_str() {
        "all" => Ok(Mode::All),
        "roguelite" => Ok(Mode::Roguelite),
        "rpg" => Ok(Mode::Rpg),
        other => bail!("unknown balance sim mode `{other}`; expected all, roguelite, or rpg"),
    }
}

fn parse_usize(value: String) -> Result<usize> {
    value
        .parse::<usize>()
        .with_context(|| format!("parsing run count `{value}`"))
}

fn parse_seed(value: String) -> Result<u64> {
    if let Some(hex) = value.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).with_context(|| format!("parsing hex seed `{value}`"))
    } else {
        value
            .parse::<u64>()
            .with_context(|| format!("parsing decimal seed `{value}`"))
    }
}

fn print_help() {
    println!(
        "balance_sim [--mode all|roguelite|rpg] [--smoke] [--runs N] [--seed SEED]\n\
         Emits deterministic JSON metrics. Default mode runs both C5 roguelite and RPG simulators."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_smoke_rpg_cli() {
        let cli = Cli::parse([
            "--mode".to_string(),
            "rpg".to_string(),
            "--smoke".to_string(),
            "--runs".to_string(),
            "4".to_string(),
            "--seed".to_string(),
            "0xc5".to_string(),
        ])
        .expect("cli parses");
        assert_eq!(cli.mode, Mode::Rpg);
        assert!(cli.smoke);
        assert_eq!(cli.runs, Some(4));
        assert_eq!(cli.seed, Some(0xc5));
    }
}
