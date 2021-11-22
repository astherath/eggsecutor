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
    let path_string = match env::var(get_state_file_env_key()) {
        Ok(state_path) => {
            if let Err(file_err) = check_if_file_is_valid(&state_path) {
                // terminate with io error
                file_err.exit();
            }
            state_path
        }
        Err(_) => get_default_state_file_path_string(),
    };

    shellexpand::tilde(&path_string).to_string()
}

fn get_state_file_env_key() -> String {
    "EGGSECUTOR_STATE_FILE".to_string()
}

fn get_default_state_file_path_string() -> String {
    "~/.eggsecutor.state".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_file_env_key_should_be_default_value() {
        let default_env_key = "EGGSECUTOR_STATE_FILE";
        assert_eq!(default_env_key, &get_state_file_env_key());
    }

    #[test]
    fn default_state_file_path_string_should_be_set() {
        let expected_default_path = "~/.eggsecutor.state";
        assert_eq!(expected_default_path, &get_default_state_file_path_string());
    }

    #[test]
    fn state_file_path_should_return_default_if_env_not_set() {
        env::remove_var(get_state_file_env_key());

        let state_file_path = get_state_file_path();
        let expected_path = get_default_state_file_path_string();

        assert_eq!(expected_path, state_file_path);
    }

    #[test]
    fn state_file_path_should_return_user_set_path_if_env_key_present() {
        let test_path_value = "test-dir";
        env::set_var(get_state_file_env_key(), test_path_value);

        let state_file_path = get_state_file_path();
        assert_eq!(test_path_value, state_file_path);
    }
}
