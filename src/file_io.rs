use super::errors;
use super::ProcessInfo;
use clap;
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::Path;

// TODO: this is not a good cross dependency; find fix.
use super::is_process_alive;

type Processes = Vec<ProcessInfo>;

pub fn write_processes_to_state_file(processes: Processes) -> io::Result<()> {
    let state_file_path = get_state_file_path();
    let updated_processes = serde_json::to_string(&processes)?;
    fs::write(state_file_path, updated_processes.as_bytes())?;

    Ok(())
}

pub fn get_running_processes_from_state_file() -> io::Result<Processes> {
    let mut processes = get_all_processes_from_state_file()?;
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

fn get_all_processes_from_state_file() -> io::Result<Processes> {
    let state_file_path = get_state_file_path();
    let contents = fs::read_to_string(state_file_path)?;
    let processes: Vec<ProcessInfo> = serde_json::from_str(&contents)?;
    Ok(processes)
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

    #[test]
    fn getting_processes_from_file_should_be_ok_given_valid_file() {
        let file_path = &generate_path_string();
        let process_data = get_valid_process_data();
        set_path_to_use(file_path);

        // let test_file = TestFile::touch(file_path, &process_data)
        // .expect("test file with process data could not be created");

        fs::write(file_path, &process_data).unwrap();

        let processes = get_all_processes_from_state_file()
            .expect("getting processes from file returned unexpected error");

        fs::remove_file(file_path).unwrap();

        assert!(processes.len() > 0);
    }

    #[test]
    fn get_processes_from_state_file_should_return_err_if_no_file() {
        let file_path = &generate_path_string();
        set_path_to_use(file_path);

        assert!(!Path::new(file_path).exists());

        let result = get_all_processes_from_state_file();
        assert!(result.is_err());
    }

    // pub fn get_processes_from_state_file() -> io::Result<Vec<ProcessInfo>> {

    #[test]
    fn file_valid_check_should_err_with_nonexistent_file_path() {
        let nonexistent_file_path = &generate_path_string();
        let result = check_if_file_is_valid(nonexistent_file_path);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, clap::ErrorKind::InvalidValue);
    }

    #[test]
    fn file_valid_check_should_be_ok_with_existing_file() {
        let file_path = &generate_path_string();
        let empty_data = "";
        let _test_file =
            TestFile::touch(file_path, empty_data).expect("test file couldnt be created");

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

        // start tracking file so we can cleanup after
        let _test_file = TestFile::track(file_path);

        create_state_file_if_not_exists().expect("state file check returned err");

        // check file was created and is empty
        assert!(Path::new(file_path).exists());
        let file_data = fs::read(file_path).expect("data could not be read from state file");
        assert!(file_data.is_empty());
    }

    #[test]
    fn state_file_should_not_be_created_if_exists() {
        // create empty file and set path to point to it
        let file_path = &generate_path_string();
        let test_data = "test data";

        let _test_file =
            TestFile::touch(file_path, test_data).expect("state file path could not be created");
        set_path_to_use(file_path);

        create_state_file_if_not_exists().expect("state file check returned err");

        // check no data was overwritten
        let file_data = fs::read(file_path).expect("data could not be read from state file");
        assert_eq!(file_data, test_data.as_bytes());
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

    struct TestFile<'a> {
        path: &'a str,
    }

    impl<'a> Drop for TestFile<'a> {
        fn drop(&mut self) {
            // we don't actually care if the file can't be removed because a
            // panic would mean an abort anyway, so the result can be ignored
            let _result = fs::remove_file(self.path);
        }
    }

    impl<'a> TestFile<'a> {
        fn track(path: &'a str) -> Self {
            Self { path }
        }

        fn touch(path: &'a str, data: &str) -> io::Result<Self> {
            fs::write(path, data)?;
            Ok(Self { path })
        }
    }

    fn generate_path_string() -> String {
        format!("{}.testfile", Uuid::new_v4().to_simple())
    }

    fn set_path_to_use(path_str: &str) {
        env::set_var(get_state_file_env_key(), path_str);
    }

    fn get_valid_process_data() -> String {
        r#"[{"name":"TEST_PROCES","pid":"0000","status":"Running"}]"#.to_string()
    }
}
