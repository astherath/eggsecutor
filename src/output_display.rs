
use super::ProcessInfo;

pub fn get_display_output_str_for_processes(processes: Vec<ProcessInfo>) -> String {
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

pub fn print_pre_hatch_message(filename: &str) {
    println!("{}", get_pre_hatch_message_string(filename));
}

pub fn print_post_hatch_message(pid: u32) {
    println!("{}", get_post_hatch_message_string(pid));
}

fn get_post_hatch_message_string(pid: u32) -> String {
    format!(r#"egg hatched, tracking process with pid: "{}""#, &pid)
}

fn get_pre_hatch_message_string(filename: &str) -> String {
    format!(
        r#"Hatching process "{}" and starting to track..."#,
        filename
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_output_str_for_empty_vec_should_just_be_header() {
        let empty_process_list = vec![];
        let display_string = get_display_output_str_for_processes(empty_process_list);

        // since no processes, should only be header
        let header_string = get_display_header_string();

        // trim both strings for consistency
        assert_eq!(display_string.trim(), header_string.trim());
    }

    #[test]
    fn display_header_string_should_be_non_empty() {
        let msg = get_display_header_string();
        assert!(msg.len() > 0);
    }

    #[test]
    fn pre_hatch_mesage_ok() {
        let filename = "test-filename";
        let message = get_pre_hatch_message_string(filename);
        assert!(message.contains(filename));

        // printing the message should work without error as well
        print_pre_hatch_message(filename);
    }

    #[test]
    fn post_hatch_message_ok() {
        let pid = 1234;
        let message = get_post_hatch_message_string(pid);
        assert!(message.contains(&pid.to_string()));

        // printing the message should work without error as well
        print_post_hatch_message(pid);
    }
}
