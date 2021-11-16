// our specific clap version doesn't use the new error syntax,
// but the beta version does, so we technically use the
// deprecated functions to the same effect.
#![allow(deprecated)]
extern crate clap;
extern crate shellexpand;

use clap::{App, AppSettings, Arg, Error, ErrorKind};
use serde::{Deserialize, Serialize};
use serde_json::{self};

use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, io};

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

    let subcommand_list = [
        get_hatch_subcommand(),
        get_list_processes_subcommand(),
        get_stop_process_subcommand(),
        get_clear_state_subcommand(),
    ];
    for subcommand in subcommand_list {
        app = app.subcommand(subcommand);
    }

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

fn get_clear_state_subcommand<'a>() -> App<'a> {
    const SUBCOMMAND_NAME: &str = "clear";
    const ABOUT: &str = "stops all of the processes being tracked and clears the tracking list";

    App::new(SUBCOMMAND_NAME).about(ABOUT).arg(
        Arg::new("only-clear")
            .long("--only-clear")
            .about("don't stop any processes, just clear the tracking list"),
    )
}

mod subcommand_tests {
    use super::*;
    mod clear_subcommand {
        use super::get_clear_state_subcommand;

        #[test]
        fn subcommand_should_return_app_instance() {
            let command = get_clear_state_subcommand();
            let expected_name = "clear";
            let expected_about =
                "stops all of the processes being tracked and clears the tracking list";
            assert_eq!(command.get_name(), expected_name);
            assert_eq!(command.get_about().unwrap(), expected_about);
        }

        #[test]
        fn subcommand_should_have_args() {
            let command = get_clear_state_subcommand();
            let expected_arg_name = "only-clear";
            let expected_arg_about = "don't stop any processes, just clear the tracking list";
            let arg = command
                .get_arguments()
                .into_iter()
                .filter(|x| x.get_name() == expected_arg_name)
                .next()
                .expect("arg iterator should return valid argument");
            assert_eq!(arg.get_name(), expected_arg_name);
            assert_eq!(arg.get_about().unwrap(), expected_arg_about);
        }
    }
}

fn get_stop_process_subcommand<'a>() -> App<'a> {
    const SUBCOMMAND_NAME: &str = "stop";
    const ABOUT: &str = "stop a process by name or pid";

    App::new(SUBCOMMAND_NAME).about(ABOUT).arg(
        Arg::new("process identifier")
            .about("Name or pid of process to stop")
            .required(true)
            .takes_value(true)
            .value_name("PROCESS_IDENTIFIER"),
    )
}

fn get_list_processes_subcommand<'a>() -> App<'a> {
    const SUBCOMMAND_NAME: &str = "list";
    const ABOUT: &str = "list all managed processes";

    App::new(SUBCOMMAND_NAME).about(ABOUT)
}

fn get_hatch_subcommand<'a>() -> App<'a> {
    const SUBCOMMAND_NAME: &str = "hatch";
    const ABOUT: &str = "start managing a binary process";

    App::new(SUBCOMMAND_NAME).about(ABOUT).arg(
        Arg::new("file")
            .about("Sets the input file to use")
            .required(true)
            .takes_value(true)
            .value_name("INPUT"),
    )
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
    println!(
        r#"Hatching process "{}" and starting to track..."#,
        filename
    );
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

    add_process_to_state_tracker(child_info).unwrap_or_else(|err| handle_process_boot_error(err));

    println!(r#"egg hatched, tracking process with pid: "{}""#, &pid);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
enum ProcessStatus {
    Running,
    Stopped,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProcessInfo {
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

    let display_str_for_processes = get_display_output_str_for_processes(processes);
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
            handle_no_such_process_error(process_identifier);
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

fn handle_process_boot_error(err_reason: io::Error) -> ! {
    Error::with_description(
        format!(
            "process abruptly exited after being hatched, details: {}",
            err_reason
        ),
        ErrorKind::Io,
    )
    .exit();
}

fn handle_no_file_data_error() -> ! {
    Error::with_description(
        format!("no state file data found. Add a process to track first",),
        ErrorKind::Io,
    )
    .exit();
}

fn handle_no_such_process_error(process_info: &str) -> ! {
    Error::with_description(
        format!(
            r#"couldn not stop process. no matching process with identifier: "{}""#,
            process_info
        ),
        ErrorKind::InvalidValue,
    )
    .exit();
}

fn get_display_output_str_for_processes(processes: Vec<ProcessInfo>) -> String {
    format!(
        "{}\n{}",
        get_display_header_string(),
        processes
            .iter()
            .map(|x| x.to_console_string())
            .collect::<Vec<String>>()
            .join("")
    )
}

fn get_display_header_string() -> String {
    format!(
        "{:<15} {:<7} {:<10}\n{:-<35}",
        "Process name", "pid", "status", ""
    )
}
