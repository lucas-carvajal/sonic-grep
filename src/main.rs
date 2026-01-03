mod config;
mod utils;

use config::Config;
use crossbeam_channel::{bounded, unbounded};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;
use std::sync::Arc;
use utils::search_hit;

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
    let bound = config.num_workers.saturating_mul(8).max(100) as usize;
    let (work_tx, work_rx) = bounded::<Message>(bound);
    let (result_tx, result_rx) = unbounded::<Message>();

    let shared_config = Arc::new(config);

    let mut worker_handles = Vec::new();

    for _ in 0..shared_config.num_workers {
        let work_rx_clone = work_rx.clone();
        let result_tx_clone = result_tx.clone();
        let config_clone = shared_config.clone();

        let handle = std::thread::spawn(move || {
            for message in work_rx_clone {
                if search_hit(&config_clone, &message.text) {
                    if result_tx_clone.send(message).is_err() {
                        break;
                    };
                }
            }
            // Loop ends when work_rx_clone is disconnected
            // result_tx_clone dropped automatically here
        });

        worker_handles.push(handle);
    }

    // Read in lines from file and send to workers channel
    let file = File::open(shared_config.file_path.as_str())?;
    let reader = BufReader::new(file);
    let mut line_counter = 1;

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

    // Drop work and result channel sender and join all worker threads
    drop(work_tx);
    for handle in worker_handles {
        let _ = handle.join();
    }
    drop(result_tx);

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

#[derive(Debug, Clone)]
struct Message {
    text: String,
    line: usize,
}
