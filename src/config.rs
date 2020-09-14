use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Env = HashMap<String, String>;

pub fn init_env() -> Env {
    let file = File::open(".env").expect("couldn't open file: .env");
    let mut env = HashMap::new();

    for line in BufReader::new(file).lines() {
        let key_values: Vec<String> = line.unwrap().split("=").map(str::to_string).collect();
        env.insert(key_values[0].clone(), key_values[1].clone());
    }
    env
}
