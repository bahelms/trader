use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::{fs, fs::File};

pub type Config = HashMap<String, String>;

pub fn init_config() -> Config {
    let mut config = HashMap::new();
    add_env_file(&mut config);
    add_access_key(&mut config);
    config
}

fn add_env_file(config: &mut Config) {
    let file = match File::open(".env") {
        Ok(file) => file,
        Err(_) => panic!("couldn't open file: .env"),
    };

    for line in BufReader::new(file).lines() {
        let key_values: Vec<String> = line.unwrap().split("::").map(str::to_string).collect();
        config.insert(key_values[0].clone(), key_values[1].clone());
    }
}

fn add_access_key(config: &mut Config) {
    let token = fs::read_to_string(".td_access_token").expect("couldn't open file: .env");
    config.insert("TD_ACCESS_TOKEN".to_string(), token);
}
