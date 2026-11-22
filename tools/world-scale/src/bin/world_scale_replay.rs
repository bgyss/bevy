//! Runs and compares versioned world-scale replays.

use bevy_world_scale::{compare_runs, run_replay, Replay};
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let replay = if let Some(index) = args.iter().position(|arg| arg == "--replay") {
        match fs::read_to_string(&args[index + 1])
            .and_then(|text| serde_json::from_str(&text).map_err(std::io::Error::other))
        {
            Ok(replay) => replay,
            Err(error) => {
                eprintln!("failed to read replay: {error}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        Replay::intersection_v1()
    };
    let expected = run_replay(&replay, false).expect("versioned replay");
    let fault = args
        .windows(2)
        .any(|pair| pair == ["--fault", "accept-duplicate"]);
    let actual = run_replay(&replay, fault).expect("versioned replay");
    if let Err(difference) = compare_runs(&expected, &actual) {
        eprintln!(
            "first difference: tick={} stable_id={} field={} expected={} actual={}",
            difference.tick,
            difference.stable_id.0,
            difference.field,
            difference.expected,
            difference.actual
        );
        return ExitCode::FAILURE;
    }
    println!("replay matched: {} ticks", actual.len());
    ExitCode::SUCCESS
}
