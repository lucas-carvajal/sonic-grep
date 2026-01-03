mod config;
mod utils;

use config::Config;
use crossbeam_channel::{bounded, unbounded};
use std::convert::From;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;
use std::sync::Arc;
use utils::prepare_search_query;
use utils::prepare_search_text;

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

    let shared_config = Arc::new(config);

    for _ in 0..shared_config.num_workers {
        let work_rx_clone = work_rx.clone();
        let result_tx_clone = result_tx.clone();
        let config_clone = shared_config.clone();

        std::thread::spawn(move || {
            for message in work_rx_clone {
                let search_text = prepare_search_text(&config_clone, &message.text);
                let search_query = prepare_search_query(&config_clone, &config_clone.query);
                if search_text.contains(search_query.as_str()) {
                    let _ = result_tx_clone.send(message);
                }
            }
            // Loop ends when work_rx_clone is disconnected
            // result_tx_clone dropped automatically here
        });
    }

    // Read in lines from file and send to workers channel
    let file = File::open(shared_config.file_path.as_str())?;
    let reader = BufReader::new(file);
    let mut line_counter = 0;

    for line in reader.lines() {
        let line = line?;
        if work_tx
            .send(Message {
                text: line,
                line: line_counter,
            })
            .is_err()
        {
            break;
        };
        line_counter += 1;
    }
    drop(work_tx);

    // Read results to vector, sort them and print them
    let mut results: Vec<Message> = result_rx.into_iter().collect();
    results.sort_by_key(|msg| msg.line);

    println!(
        "Found the following occurences of '{}' in file {}",
        shared_config.query, shared_config.file_path
    );
    for result in results {
        println!("Line {}: {}", result.line, result.text);
    }

    Ok(())
}

struct Message {
    text: String,
    line: u32,
}
