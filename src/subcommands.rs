use clap::{App, Arg};
pub fn get_all_subcommands<'a>() -> Vec<App<'a>> {
    vec![
        get_hatch_subcommand(),
        get_list_processes_subcommand(),
        get_stop_process_subcommand(),
        get_clear_state_subcommand(),
    ]
}

fn get_clear_state_subcommand<'a>() -> App<'a> {
    const SUBCOMMAND_NAME: &str = "clear";
    const ABOUT: &str = "stops all of the processes being tracked and clears the tracking list";

    App::new(SUBCOMMAND_NAME).about(ABOUT).arg(
        Arg::new("only-clear")
            .long("--only-clear")
            .help("don't stop any processes, just clear the tracking list"),
    )
}

fn get_stop_process_subcommand<'a>() -> App<'a> {
    const SUBCOMMAND_NAME: &str = "stop";
    const ABOUT: &str = "stop a process by name or pid";

    App::new(SUBCOMMAND_NAME).about(ABOUT).arg(
        Arg::new("process identifier")
            .help("Name or pid of process to stop")
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
            .help("Sets the input file to use")
            .required(true)
            .takes_value(true)
            .value_name("INPUT"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    mod subcommand_testing_utils {
        use clap::App;
        pub fn test_subcommand_should_return_app_instance<'a, T>(
            subcommand_getter: T,
            expected_name: &str,
            expected_about: &str,
        ) where
            T: Fn() -> App<'a>,
        {
            let command = subcommand_getter();
            assert_eq!(command.get_name(), expected_name);
            assert_eq!(command.get_about().unwrap(), expected_about);
        }
    }
    mod stop_subcommand {
        use super::get_stop_process_subcommand;
        use super::subcommand_testing_utils as utils;

        #[test]
        fn subcommand_should_return_app_instance() {
            let expected_name = "stop";
            let expected_about = "stop a process by name or pid";
            utils::test_subcommand_should_return_app_instance(
                get_stop_process_subcommand,
                expected_name,
                expected_about,
            );
        }
    }

    mod list_subcommand {
        use super::get_list_processes_subcommand;
        use super::subcommand_testing_utils as utils;

        #[test]
        fn subcommand_should_return_app_instance() {
            let expected_name = "list";
            let expected_about = "list all managed processes";
            utils::test_subcommand_should_return_app_instance(
                get_list_processes_subcommand,
                expected_name,
                expected_about,
            );
        }
    }

    mod hatch_subcommand {
        use super::get_hatch_subcommand;
        use super::subcommand_testing_utils as utils;

        #[test]
        fn subcommand_should_return_app_instance() {
            let expected_name = "hatch";
            let expected_about = "start managing a binary process";
            utils::test_subcommand_should_return_app_instance(
                get_hatch_subcommand,
                expected_name,
                expected_about,
            );
        }
    }

    mod clear_subcommand {
        use super::get_clear_state_subcommand;
        use super::subcommand_testing_utils as utils;

        #[test]
        fn subcommand_should_return_app_instance() {
            let expected_name = "clear";
            let expected_about =
                "stops all of the processes being tracked and clears the tracking list";
            utils::test_subcommand_should_return_app_instance(
                get_clear_state_subcommand,
                expected_name,
                expected_about,
            );
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
            assert_eq!(arg.get_help().unwrap(), expected_arg_about);
        }
    }

    #[test]
    fn get_all_subcommands_return_should_be_foldable_into_app() {
        let all_subcommands = get_all_subcommands();
        let expected_count_of_subcommands = all_subcommands.len();

        // smoke check for safety
        assert!(expected_count_of_subcommands > 0);

        let count_of_subcommands_in_app = all_subcommands
            .into_iter()
            .fold(App::new("test-app"), |acc, sub| acc.subcommand(sub))
            .get_subcommands()
            .count();

        assert_eq!(count_of_subcommands_in_app, expected_count_of_subcommands);
    }
}
