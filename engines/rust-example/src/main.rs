use std::io::Write;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        println!("unknown invocation");
        return;
    }
    if args[1] == "START" {
        println!("Rust example bot");
        return
    }
    if args.len() < 8 {
        println!("unknown invocation");
        return;
    }
    let [
        executable_name,
        min_bet_to_not_fold,
        turn_time_ms,
        time_bank_ms,
        game_params_path,
        game_state_path,
        game_actions_path,
        temp_dir_path,
    ] = &args[..8] else { unreachable!() };
    let min_bet_to_not_fold: usize = min_bet_to_not_fold.parse().unwrap();
    let turn_time_ms: usize = turn_time_ms.parse().unwrap();
    let time_bank_ms: usize = time_bank_ms.parse().unwrap();

    // decide on move
    println!("FOLD");
    std::io::stdout().flush();

    // cleanup, logging
    eprintln!("long-term logging");
}
