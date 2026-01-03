use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub num_workers: u16,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Not enough arguments!");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        let num_workers = 8;

        Ok(Config {
            query,
            file_path,
            ignore_case,
            num_workers,
        })
    }
}
