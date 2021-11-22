use clap::{Error, ErrorKind};
use std::io;

pub fn handle_spawn_failure(err_reason: io::Error) -> ! {
    get_spawn_failure_error(err_reason).exit();
}

pub fn handle_no_file_data_error() -> ! {
    get_no_file_data_error().exit();
}

pub fn handle_no_such_process_error(process_info: &str) -> ! {
    get_no_such_process_error(process_info).exit();
}

pub fn handle_process_boot_error(err_reason: io::Error) -> ! {
    get_process_boot_error(err_reason).exit();
}

fn get_spawn_failure_error(err_reason: io::Error) -> Error {
    Error::with_description(
        format!(
            "could not hatch process: binary could not be executed. Details: {}",
            err_reason
        ),
        ErrorKind::Io,
    )
}

fn get_no_file_data_error() -> Error {
    Error::with_description(
        format!("no state file data found. Add a process to track first",),
        ErrorKind::Io,
    )
}

fn get_no_such_process_error(process_info: &str) -> Error {
    Error::with_description(
        format!(
            r#"couldn not stop process. no matching process with identifier: "{}""#,
            process_info
        ),
        ErrorKind::InvalidValue,
    )
}

fn get_process_boot_error(err_reason: io::Error) -> Error {
    Error::with_description(
        format!(
            "process abruptly exited after being hatched, details: {}",
            err_reason
        ),
        ErrorKind::Io,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn process_boot_error_should_return_io_clap_err() {
        let kind = clap::ErrorKind::Io;
        let err_msg = "test boot error";
        let io_err = get_io_error(err_msg);

        let clap_err_fn = || get_process_boot_error(io_err);

        check_err_matches_spec(err_msg, kind, clap_err_fn);
    }

    #[test]
    fn no_such_process_error_should_return_invalid_value_clap_err() {
        let process_err_msg = "test process not found error";
        let kind = clap::ErrorKind::InvalidValue;

        let clap_err_fn = || get_no_such_process_error(process_err_msg);
        check_err_matches_spec(process_err_msg, kind, clap_err_fn);
    }

    #[test]
    fn no_file_data_error_should_return_clap_io_err() {
        let process_err_msg = "no state file data found. Add a process to track first";
        let kind = ErrorKind::Io;

        let clap_err_fn = || get_no_file_data_error();
        check_err_matches_spec(process_err_msg, kind, clap_err_fn);
    }

    #[test]
    fn spawn_failure_error_should_return_clap_io_error() {
        let kind = ErrorKind::Io;
        let process_err_msg = "test spawn error";
        let io_err = get_io_error(process_err_msg);

        let clap_err_fn = || get_spawn_failure_error(io_err);

        check_err_matches_spec(process_err_msg, kind, clap_err_fn);
    }

    fn check_err_matches_spec<F>(err_msg: &str, error_kind: ErrorKind, err_factory: F)
    where
        F: FnOnce() -> clap::Error,
    {
        let clap_err = err_factory();

        assert!(clap_err.to_string().contains(err_msg));
        assert_eq!(clap_err.kind, error_kind);
    }

    fn get_io_error(err_msg: &str) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err_msg)
    }
}
