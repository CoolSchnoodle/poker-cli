use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use clap::Parser;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
struct RunFailed;

trait Engine {
    fn start(&mut self, temp_storage_path: &Path) -> Result<Output, RunFailed>;

    fn run(
        &mut self,
        min_bet_to_not_fold: usize,
        turn_time_ms: usize,
        time_bank_ms: usize,
        game_params_path: &Path,
        game_state_path: &Path,
        game_actions_path: &Path,
        temp_storage_path: &Path,
    ) -> Result<Output, RunFailed>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExecutableFileEngine {
    file_path: PathBuf,
}

impl ExecutableFileEngine {
    fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl Engine for ExecutableFileEngine {
    fn start(&mut self, temp_storage_path: &Path) -> Result<Output, RunFailed> {
        Command::new(&self.file_path)
            .args(&[OsStr::new("START"), temp_storage_path.as_ref()])
            .output()
            .map_err(|_| RunFailed)
    }

    fn run(
        &mut self,
        min_bet_to_not_fold: usize,
        turn_time_ms: usize,
        time_bank_ms: usize,
        game_params_path: &Path,
        game_state_path: &Path,
        game_actions_path: &Path,
        temp_storage_path: &Path,
    ) -> Result<Output, RunFailed> {
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

#[derive(Debug)]
enum RunGameError {
    RunFailed,
    IoError(std::io::Error),
}

impl From<std::io::Error> for RunGameError {
    fn from(value: std::io::Error) -> Self {
        RunGameError::IoError(value)
    }
}

fn run_game(
    mut engine_1: impl Engine,
    mut engine_2: impl Engine,
    parameters: &Args,
    log_path_1: &Path,
    log_path_2: &Path,
) -> Result<Outcome, RunGameError> {
    let game_data_path = PathBuf::from(".").canonicalize()?.join("tmp");
    let game_parameters_path = game_data_path.clone().join("game_parameters.json");
    let game_state_path = game_data_path.clone().join("game_state.json");
    let game_history_path = game_data_path.clone().join("game_history.txt");
    let tmp_path_1 = game_data_path.clone().join("engine1");
    let tmp_path_2 = game_data_path.clone().join("engine2");

    let log_open_options = std::fs::OpenOptions::new()
        .create_new(true)
        .append(true)
        .clone();
    let mut logs_1 = log_open_options.open(log_path_1)?;
    let mut logs_2 = log_open_options.open(log_path_2)?;

    let start_output_1 = engine_1
        .start(&tmp_path_1)
        .map_err(|_| RunGameError::RunFailed)?;
    logs_1.write(&start_output_1.stderr)?;
    let name_1 = String::from_utf8_lossy(&start_output_1.stdout);
    let start_output_2 = engine_2
        .start(&tmp_path_2)
        .map_err(|_| RunGameError::RunFailed)?;
    logs_2.write(&start_output_2.stderr)?;
    let name_2 = {
        let mut name_2 = String::from_utf8_lossy(&start_output_2.stdout).into_owned();
        if name_2 == name_1 {
            name_2.push('2');
        }
        name_2
    };

    let params = GameParameters {
        engines: vec![name_1.into_owned(), name_2],
        small_blind: parameters.small_blind,
        big_blind: parameters.big_blind,
        ante: parameters.ante,
        starting_funds: parameters.starting_funds,
        starting_time_bank: parameters.time_bank_ms,
    };

    todo!()
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let engines: Vec<_> = get_engine_dirs(args.engines_dir.as_ref())?
        .map(ExecutableFileEngine::new)
        .enumerate()
        .collect();
    let pairings = engines.into_iter().array_combinations::<2>();

    let log_paths_base = PathBuf::from(".").canonicalize()?.join("logs");

    let mut results = vec![];

    for [(i1, engine_1), (i2, engine_2)] in pairings {
        match run_game(
            engine_1,
            engine_2,
            &args,
            &log_paths_base.clone().join(i1.to_string()),
            &log_paths_base.clone().join(i2.to_string()),
        ) {
            Ok(outcome) => results.push((i1, i2, outcome)),
            Err(RunGameError::IoError(e)) => Err(e)?,
            Err(RunGameError::RunFailed) => eprintln!("game between {i1} and {i2} failed"),
        }
    }

    Ok(())
}
