extern crate clap;
extern crate shellexpand;

use clap::{App, Arg, Error, ErrorKind};
use serde::{Deserialize, Serialize};
use serde_json::{self};

use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::process::Command;

// wanted features:
//      - given path to a binary:
//          - execute it in a background process (not thread based)
//          - keep its info on how to start/stop/reset
//      - get list of all running processes "hatched eggs"
// fn main() -> Result<(), Error> {
fn main() {
    const PROGRAM_TITLE: &str = "eggsecutor";
    const VERSION: &str = "1.0";
    const AUTHOR: &str = "Felipe A. <farceriv@gmail.com>";
    const ABOUT: &str = "A background process task manager";

    let matches = App::new(PROGRAM_TITLE)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .subcommand(get_hatch_subcommand())
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(ref matches) = matches.subcommand_matches("hatch") {
        if let Some(filename) = matches.value_of("file") {
            process_file_input_for_hatch_subcommand(filename).unwrap();
        }
    }
}

fn get_hatch_subcommand<'a>() -> App<'a> {
    const SUBCOMMAND_NAME: &str = "hatch";
    const ABOUT: &str = "start managing a binary process";

    App::new(SUBCOMMAND_NAME).about(ABOUT).arg(get_hatch_arg())
}

fn get_hatch_arg<'a>() -> Arg<'a> {
    Arg::new("file")
        .about("Sets the input file to use")
        .required(true)
        .takes_value(true)
        .value_name("INPUT")
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
    let bin_path = format!("./{}", filename);
    let mut child = Command::new(bin_path)
        .spawn()
        .unwrap_or_else(|err| handle_spawn_error(err));

    let pid = child.id();
    let child_info = ProcessInfo {
        name: filename.to_string(),
        pid: pid.to_string(),
        status: ProcessStatus::Running,
    };

    add_subprocess_to_state_tracker(child_info)?;

    child.wait()?;

    println!("pid: {}", pid);

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

fn add_subprocess_to_state_tracker(process_info: ProcessInfo) -> io::Result<()> {
    let state_file_path = &shellexpand::tilde("~/.eggsecutor.state").to_string();
    if !Path::new(&state_file_path).exists() {
        File::create(state_file_path)?;
    }

    // read file contents
    let contents = fs::read_to_string(state_file_path)?;
    let mut processes: Vec<ProcessInfo> = serde_json::from_str(&contents)?;

    // add new process
    processes.push(process_info);

    // checkup on processes if dead
    for process in &processes {
        println!("process: {:?}", &process);
        is_process_alive(&process.pid)?;
    }

    // write info
    let updated_processes = serde_json::to_string(&processes)?;
    fs::write(state_file_path, updated_processes.as_bytes())?;

    Ok(())
}

fn kill_process_by_pid(pid: &str) -> io::Result<()> {
    let command = "kill";
    Command::new(command).arg("-9").arg(pid).spawn()?.wait()?;
    Ok(())
}

fn is_process_alive(pid: &str) -> io::Result<bool> {
    let command = "kill";
    let response = Command::new(command).arg("-0").arg(pid).spawn()?.wait()?;
    println!("response: {}", response);

    Ok(true)
}

fn handle_spawn_error(err_reason: io::Error) -> ! {
    Error::with_description(
        format!(
            "could not hatch subprocess: binary could not be executed. Details: {}",
            err_reason
        ),
        ErrorKind::Io,
    )
    .exit();
}
