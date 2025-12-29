mod config;

use config::Config;
use crossbeam_channel::{bounded, unbounded};
use sonic_grep::search;
use sonic_grep::search_case_insensitive;
use std::convert::From;
use std::env;
use std::error::Error;
use std::fs;
use std::process;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let (work_tx, work_rx) = bounded::<Message>(usize::from(config.num_workers * 16));
    let (result_tx, result_rx) = unbounded::<Message>();

    let shared_config = Arc::new(&config);

    let contents = fs::read_to_string(config.file_path)?;

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{line}");
    }

    Ok(())
}

struct Message {
    text: String,
    line: u32,
}
