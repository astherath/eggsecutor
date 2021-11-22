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
        Ok(state_path) => state_path,
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
    use uuid::Uuid;

    struct TestFile<'a> {
        path: &'a str,
    }

    impl<'a> Drop for TestFile<'a> {
        fn drop(&mut self) {
            fs::remove_file(self.path).expect("file could not be removed while getting dropped");
        }
    }

    fn generate_path_string() -> String {
        format!("{}.testfile", Uuid::new_v4().to_simple())
    }

    impl<'a> TestFile<'a> {
        fn touch(path: &'a str, data: &str) -> io::Result<Self> {
            fs::write(path, data)?;
            Ok(Self { path })
        }
    }

    #[test]
    fn file_valid_check_should_be_ok_with_existing_file() {
        let file_path = &generate_path_string();
        let empty_data = "";
        touch_file(file_path, empty_data).expect("test file couldnt be created");

        let result = check_if_file_is_valid(file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn state_file_should_be_created_if_not_exists() {
        // set path to a file that does not exist
        let file_path = &generate_path_string();
        set_path_to_use(file_path);

        // ensure file does not exists prior to call
        assert!(!Path::new(file_path).exists());

        create_state_file_if_not_exists().expect("state file check returned err");

        // check file was created and is empty
        assert!(Path::new(file_path).exists());
        let file_data = fs::read(file_path).expect("data could not be read from state file");
        assert!(file_data.is_empty());

        cleanup_file(file_path);
    }

    #[test]
    fn state_file_should_not_be_created_if_exists() {
        // create empty file and set path to point to it
        let file_path = "test-file.testfile";
        let test_data = "test data";

        touch_file(file_path, test_data).expect("state file path could not be created");
        set_path_to_use(file_path);

        create_state_file_if_not_exists().expect("state file check returned err");

        // check no data was overwritten
        let file_data = fs::read(file_path).expect("data could not be read from state file");
        assert_eq!(file_data, test_data.as_bytes());

        cleanup_file(file_path);
    }

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

        // we have to expand the tilde for the path
        let expected_path = shellexpand::tilde(&get_default_state_file_path_string()).to_string();

        assert_eq!(expected_path, state_file_path);
    }

    #[test]
    fn state_file_path_should_return_user_set_path_if_env_key_present() {
        let test_path_value = "test-dir";
        env::set_var(get_state_file_env_key(), test_path_value);

        let state_file_path = get_state_file_path();
        assert_eq!(test_path_value, state_file_path);
    }

    fn touch_file(path_str: &str, data: &str) -> io::Result<()> {
        fs::write(path_str, data)?;
        Ok(())
    }

    fn cleanup_file(path_str: &str) {
        fs::remove_file(path_str).unwrap();
    }

    fn set_path_to_use(path_str: &str) {
        env::set_var(get_state_file_env_key(), path_str);
    }
}
