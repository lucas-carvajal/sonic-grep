use crossbeam_channel::{bounded, unbounded};
use sonic_grep::config::Config;
use sonic_grep::search_case_insensitive;
use sonic_grep::utils::prepare_search_query;
use sonic_grep::utils::prepare_search_text;
use sonic_grep::utils::search;
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

    let shared_config = Arc::new(config);

    for _ in 0..shared_config.num_workers {
        let work_rx_clone = work_rx.clone();
        let result_tx_clone = result_tx.clone();
        let config_clone = shared_config.clone();

        std::thread::spawn(move || {
            for message in work_rx_clone {
                let search_text = prepare_search_text(config_clone, message.text);
                let search_query = prepare_search_query(config_clone, config_clone.query);
                if search_text.contains(search_query.as_str()) {
                    let _ = result_tx_clone.send(message);
                }
            }
            // Loop ends when work_rx_clone is disconnected
            // result_tx_clone dropped automatically here
        });
    }

    // TODO

    let contents = fs::read_to_string(shared_config.file_path.as_str())?;

    let results = if shared_config.clone().ignore_case {
        search_case_insensitive(shared_config.query.as_str(), &contents)
    } else {
        search(shared_config.query.as_str(), &contents)
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
