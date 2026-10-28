use std::path::{Path, PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};
use itertools::Itertools;

trait Engine {
    fn start(temp_storage_path: &str);

    fn run(
        min_bet_to_not_fold: usize,
        turn_time_ms: usize,
        time_bank_ms: usize,
        game_params_path: &str,
        game_state_path: &str,
        game_actions_path: &str,
        temp_storage_path: &str,
    );
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExecutableFileEngine {
    file_path: PathBuf,
}

impl Engine for ExecutableFileEngine {
    fn start(temp_storage_path: &str) -> {
        todo!()
    }

    fn run(
        min_bet_to_not_fold: usize,
        turn_time_ms: usize,
        time_bank_ms: usize,
        game_params_path: &str,
        game_state_path: &str,
        game_actions_path: &str,
        temp_storage_path: &str,
    ) -> {
        todo!()
    }    
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(long, default_value_t = 400)]
    starting_funds: usize,

    #[arg(short, long, default_value_t = 10)]
    small_blind: usize,

    #[arg(short, long, default_value_t = 20)]
    big_blind: usize,

    #[arg(long, default_value_t = 0)]
    ante: usize,

    #[arg(long)]
    log_to: Option<String>,

    #[arg(long)]
    engines_dir: String,

    #[arg(short, long, default_value_t = 1_000)]
    turn_time_ms: usize,

    #[arg(long, default_value_t = 10_000)]
    time_bank_ms: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GameParameters {
    engines: Vec<String>,
    small_blind: usize,
    big_blind: usize,
    ante: usize,
    starting_funds: usize,
    starting_time_bank: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GameState {
    you_index: u8,
    funds: Vec<usize>,
    bets: Vec<usize>,
    folds: Vec<bool>,
    time_banks: Vec<usize>,
    your_hand: ((u8, u8), (u8, u8)),
    revealed_cards: Vec<(u8, u8)>,
    small_blind: usize,
    big_blind: usize,
}

fn get_engine_dirs(engines_dir: &Path) -> std::io::Result<impl Iterator<Item = PathBuf>> {
    Ok(std::fs::read_dir(engines_dir)?.filter_map(|di| {
        di.ok()
            .map(|di| di.path())
            .filter(|p| p.is_dir() && p.join("build.sh").exists() && p.join("run.sh").exists())
    }))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Outcome {
    FirstEngineWins,
    SecondEngineWins,
}

fn run_game(engine_1: impl Engine, engine_2: impl Engine) -> Outcome {
    todo!()
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let engine_dirs: Vec<_> = get_engine_dirs(args.engines_dir.as_ref())?.collect();
    for engine_dir in engine_dirs {
        // run the build.sh
    }
    let pairings = engine_dirs.into_iter().array_combinations::<2>();


    for [engine_1_dir, engine_2_dir] in pairings {
        run_game(engine_1_dir, engine_2_dir);
    }

    Ok(())
}
