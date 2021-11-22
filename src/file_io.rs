use super::errors;
use super::ProcessInfo;
use clap;
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::Path;

// TODO: this is not a good cross dependency; find fix.
use super::is_process_alive;

pub fn write_processes_to_state_file(processes: Vec<ProcessInfo>) -> io::Result<()> {
    let state_file_path = get_state_file_path();
    let updated_processes = serde_json::to_string(&processes)?;
    fs::write(state_file_path, updated_processes.as_bytes())?;

    Ok(())
}

pub fn get_processes_from_state_file() -> io::Result<Vec<ProcessInfo>> {
    let state_file_path = get_state_file_path();
    let contents = fs::read_to_string(state_file_path)?;
    let mut processes: Vec<ProcessInfo> = serde_json::from_str(&contents)?;
    processes.retain(|process| is_process_alive(&process.pid).unwrap());
    Ok(processes)
}

pub fn check_if_file_is_valid(filename: &str) -> Result<(), clap::Error> {
    match Path::new(filename).exists() {
        true => Ok(()),
        false => Err(errors::get_invalid_file_path_error()),
    }
}

pub fn create_state_file_if_not_exists() -> io::Result<()> {
    let state_file_path = get_state_file_path();
    if !Path::new(&state_file_path).exists() {
        File::create(&state_file_path)?;
    }
    Ok(())
}

fn get_state_file_path() -> String {
    let path_string = match env::var("EGGSECUTOR_STATE_FILE") {
        Ok(state_path) => state_path,
        Err(_) => "~/.eggsecutor.state".to_string(),
    };

    shellexpand::tilde(&path_string).to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn pass() {}
}
