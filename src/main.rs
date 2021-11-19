// our specific clap version doesn't use the new error syntax,
// but the beta version does, so we technically use the
// deprecated functions to the same effect.
#![allow(deprecated)]
extern crate clap;
extern crate shellexpand;

use clap::{App, AppSettings, Error, ErrorKind};
use serde::{Deserialize, Serialize};
use serde_json::{self};

use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, io};
mod output_display;
mod subcommands;

fn main() {
    const PROGRAM_TITLE: &str = "eggsecutor";
    const VERSION: &str = "1.0";
    const AUTHOR: &str = "astherath <me@felipearce.dev>";
    const ABOUT: &str = "A friendly background process task manager";

    let mut app = App::new(PROGRAM_TITLE)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .setting(AppSettings::ArgRequiredElseHelp);

    app = subcommands::get_all_subcommands()
        .into_iter()
        .fold(app, |acc, subcommand| acc.subcommand(subcommand));
    let matches = app.get_matches();

    // get matches and execute commands here
    if let Some(ref matches) = matches.subcommand_matches("hatch") {
        if let Some(filename) = matches.value_of("file") {
            process_file_input_for_hatch_subcommand(filename).unwrap();
        }
    } else if let Some(ref matches) = matches.subcommand_matches("stop") {
        if let Some(process_identifier) = matches.value_of("process identifier") {
            stop_process_by_process_identifier(process_identifier).unwrap();
        }
    } else if let Some(_) = matches.subcommand_matches("list") {
        print_list_of_processes().unwrap();
    } else if let Some(matches) = matches.subcommand_matches("clear") {
        if matches.is_present("only-clear") {
            clear_all_processes_from_file().unwrap();
        } else {
            stop_and_clear_all_processes().unwrap();
        }
    }
}

fn process_file_input_for_hatch_subcommand(filename: &str) -> io::Result<()> {
    if let Err(clap_err) = check_if_file_is_valid(filename) {
        clap_err.exit();
    }

    hatch_subprocess_from_file(filename)?;

    Ok(())
}

fn check_if_file_is_valid(filename: &str) -> Result<(), Error> {
    match Path::new(filename).exists() {
        true => Ok(()),
        false => Err(Error::with_description(
            String::from("invalid path to binary: file does not exist or is inaccessible"),
            ErrorKind::InvalidValue,
        )),
    }
}

fn hatch_subprocess_from_file(filename: &str) -> io::Result<()> {
    output_display::print_pre_hatch_message(filename);
    let bin_path = format!("./{}", filename);
    let child = Command::new(bin_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|err| handle_spawn_error(err));

    let pid = child.id();
    let child_info = ProcessInfo {
        name: filename.to_string(),
        pid: pid.to_string(),
        status: ProcessStatus::Running,
    };

    add_process_to_state_tracker(child_info)
        .unwrap_or_else(|err| errors::handle_process_boot_error(err));

    output_display::print_post_hatch_message(pid);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
enum ProcessStatus {
    Running,
    Stopped,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessInfo {
    name: String,
    pid: String,
    status: ProcessStatus,
}

impl ProcessInfo {
    fn to_console_string(&self) -> String {
        format!(
            "\
        {:<15} {:<7} {:<10?}\n",
            self.name, self.pid, self.status
        )
    }
}

fn get_state_file_path() -> String {
    let path_string = match env::var("EGGSECUTOR_STATE_FILE") {
        Ok(state_path) => state_path,
        Err(_) => "~/.eggsecutor.state".to_string(),
    };

    shellexpand::tilde(&path_string).to_string()
}

fn add_process_to_state_tracker(process_info: ProcessInfo) -> io::Result<()> {
    let state_file_path = get_state_file_path();
    if !Path::new(&state_file_path).exists() {
        File::create(&state_file_path)?;
    }

    let mut processes = get_processes_from_state_file()?;

    // add new process
    processes.push(process_info);

    // write info
    write_processes_to_state_file(processes)?;

    Ok(())
}

fn write_processes_to_state_file(processes: Vec<ProcessInfo>) -> io::Result<()> {
    let state_file_path = get_state_file_path();
    let updated_processes = serde_json::to_string(&processes)?;
    fs::write(state_file_path, updated_processes.as_bytes())?;

    Ok(())
}

fn print_list_of_processes() -> io::Result<()> {
    let processes = get_processes_from_state_file()
        .unwrap_or_else(|_| handle_no_file_data_error())
        .into_iter()
        .filter(|process| is_process_alive(&process.pid).unwrap())
        .collect();

    let display_str_for_processes = output_display::get_display_output_str_for_processes(processes);
    println!("{}", display_str_for_processes);
    Ok(())
}

fn get_processes_from_state_file() -> io::Result<Vec<ProcessInfo>> {
    let state_file_path = get_state_file_path();
    let contents = fs::read_to_string(state_file_path)?;
    let mut processes: Vec<ProcessInfo> = serde_json::from_str(&contents)?;
    processes.retain(|process| is_process_alive(&process.pid).unwrap());
    Ok(processes)
}

fn remove_process_from_state_tracker(pid: &str) -> io::Result<()> {
    if let Some(_) = find_process_by_pid(pid) {
        let mut processes = get_processes_from_state_file()?;
        processes.retain(|x| x.pid != pid);

        write_processes_to_state_file(processes)?;
    }
    Ok(())
}

fn stop_process_by_process_identifier(process_identifier: &str) -> io::Result<()> {
    // check if the process identfied passed is actually a pid
    let pid = &{
        if let Some(process) = find_process_by_name(process_identifier) {
            process.pid
        } else if is_existing_pid(process_identifier) {
            process_identifier.to_string()
        } else {
            errors::handle_no_such_process_error(process_identifier);
        }
    };

    stop_process_by_pid(pid)?;
    remove_process_from_state_tracker(pid)?;
    Ok(())
}

fn find_process_by_name(name: &str) -> Option<ProcessInfo> {
    for process in get_processes_from_state_file().unwrap() {
        if process.name == name {
            return Some(process);
        }
    }
    None
}

fn find_process_by_pid(pid: &str) -> Option<ProcessInfo> {
    for process in get_processes_from_state_file().unwrap() {
        if process.pid == pid {
            return Some(process);
        }
    }
    None
}

fn is_existing_pid(pid: &str) -> bool {
    [
        pid.parse::<i32>().is_ok(),
        is_pid_being_tracked(pid),
        is_process_alive(pid).unwrap(),
    ]
    .iter()
    .all(|x| *x)
}

fn is_pid_being_tracked(pid: &str) -> bool {
    get_processes_from_state_file()
        .unwrap()
        .iter()
        .map(|x| &x.pid)
        .find(|x| x == &pid)
        .is_some()
}

fn stop_process_by_pid(pid: &str) -> io::Result<()> {
    println!("stopping process with pid: {}", pid);
    let command = "kill";
    Command::new(command)
        .arg("15")
        .arg(pid)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;
    Ok(())
}

fn stop_and_clear_all_processes() -> io::Result<()> {
    get_processes_from_state_file()?
        .iter()
        .for_each(|x| stop_process_by_pid(&x.pid).unwrap());
    clear_all_processes_from_file()?;
    Ok(())
}

fn clear_all_processes_from_file() -> io::Result<()> {
    write_processes_to_state_file(vec![])?;
    Ok(())
}

fn is_process_alive(pid: &str) -> io::Result<bool> {
    let command = "kill";
    match Command::new(command)
        .arg("-0")
        .arg(pid)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?
        .code()
    {
        Some(code) => Ok(code == 0),
        None => Ok(false),
    }
}

fn handle_spawn_error(err_reason: io::Error) -> ! {
    Error::with_description(
        format!(
            "could not hatch process: binary could not be executed. Details: {}",
            err_reason
        ),
        ErrorKind::Io,
    )
    .exit();
}

pub fn handle_no_file_data_error() -> ! {
    Error::with_description(
        format!("no state file data found. Add a process to track first",),
        ErrorKind::Io,
    )
    .exit();
}

mod errors {
    use clap::{Error, ErrorKind};
    use std::io;

    pub fn handle_no_such_process_error(process_info: &str) -> ! {
        get_no_such_process_error(process_info).exit();
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

    pub fn handle_process_boot_error(err_reason: io::Error) -> ! {
        get_process_boot_error(err_reason).exit();
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

        struct ExpectedErrData<'a> {
            msg: &'a str,
            kind: clap::ErrorKind,
        }

        fn check_err_matches_spec(check_data: ExpectedErrData, err_factory: fn() -> clap::Error) {
            let err_msg = check_data.msg;
            let error_kind = check_data.kind;

            let clap_err = err_factory();

            assert!(clap_err.to_string().contains(err_msg));
            assert_eq!(clap_err.kind, error_kind);
        }

        #[test]
        fn process_boot_error_should_return_io_clap_err() {
            let err_msg = "test io error";
            let kind = clap::ErrorKind::Io;
            let io_err = io::Error::new(io::ErrorKind::Other, err_msg);

            let err_check_data = ExpectedErrData { kind, msg: err_msg };

            let clap_err_fn = move || get_process_boot_error(io_err);

            check_err_matches_spec(err_check_data, clap_err_fn);
        }

        #[test]
        fn no_such_process_error_should_return_invalid_value_clap_err() {
            let process_err_msg = "test process not found error";
            let clap_err = get_no_such_process_error(process_err_msg);
            assert!(clap_err.to_string().contains(process_err_msg));
            assert_eq!(clap_err.kind, clap::ErrorKind::InvalidValue);
        }
    }
}
