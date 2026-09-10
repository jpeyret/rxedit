#![warn(missing_docs)]
//! # rxedit is a command line utility to search and edit text files with composable commands
//!
//! Example:
//! `rxedit main.rs 'd::main|run' more::/// less::Builds -N`
//! - show all declarations matching `main` or `run`
//! - also show `///` docstrings
//! - hide anything with `Builds` in it.
//! - `-N` suppresses line number display

use clap::{ArgAction, CommandFactory, Parser};
use log::debug;
use rxedit::config_utilities;
use rxedit::common::{
    AppConfig, CommandQualifier, dbg_, get_global_config, set_global_config, take_explain_requests,
    telemetry_events,
};
use rxedit::constants::{DEBUGGING, commandflags};
use rxedit::utilities;
use rxedit::utilities::vec_lines::digest_lines;
use rxedit::utilities::vec_lines::vec_lines_to_text;
use rxedit::{Command, CommandActions, commands_factory, common::LineStatus};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process;

mod constants;
mod help_utilities;
mod loader_for_constants;

enum BuildError {
    Message(String),
    HelpDisplayed,
}

/// CLI entry point.
fn main() {
    // env_logger::init();

    // suppress timestamp and level for now
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .format_level(false)
        .init(); // --snip--

    let args: Vec<String> = env::args().collect();

    let config = match Config::new(&args) {
        Ok(config) => config,
        Err(BuildError::HelpDisplayed) => return,
        Err(BuildError::Message(err)) => {
            println!("Problem parsing arguments: {err}");
            process::exit(1);
        }
    };

    if config.dump_json {
        if let Err(e) = dump_json_and_exit(&config.file_path) {
            println!("Application error: {e}");
            process::exit(1);
        }
        return;
    }

    if let Err(e) = run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}

/// Runs the line-processing Command pipeline and prints the resulting output.
fn dump_json_and_exit(file_path: &str) -> Result<(), Box<dyn Error>> {
    let content = fs::read(file_path)?;
    let extension = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let json = rxedit::treesitterparser::parse_content_to_hashtree_json(&content, extension);
    println!("{json}");
    Ok(())
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let global_config = get_global_config();

    if *DEBUGGING {
        eprintln!("\n\nRunning commands {:?}", config.commands);
        eprintln!("Commands file {:?}", config.commands_file);
        eprintln!("Commands file after {:?}", config.commands_file_after);
        eprintln!("grep_shortcodes {}", global_config.grep_shortcodes);
        eprintln!("In file {}\n\n", config.file_path);
    }

    let contents = fs::read_to_string(&config.file_path)?;
    let mut lines: Vec<LineStatus> = contents
        .lines()
        .enumerate()
        .map(|(idx, line)| LineStatus {
            line_number: idx + 1,
            visible: false,
            line: line.to_string(),
            ..Default::default()
        })
        .collect();

    // Get the file extension for tree-sitter parsing
    let extension = Path::new(&config.file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let commands = config.commands;
    let mut hashtree_cache: Option<HashMap<usize, rxedit::common::Parsed>> = None;
    let mut hashtree_digest: Option<String> = None;
    let mut previous_command_mutated_line_text = true;

    let show_line_numbers = global_config.line_number;
    let mut emitted_end_state_in_loop = false;

    // let's see if the user provided anything to do that was understood
    let count_active_commands = commands
        .iter()
        .filter(|command| {
            !matches!(
                command,
                Command::CMacro(_) | Command::CExplain(_) | Command::CNoop(_)
            )
        })
        .count();

    for (idx, command) in commands.iter().enumerate() {
        let current_digest = if hashtree_cache.is_none() || previous_command_mutated_line_text {
            Some(digest_lines(&lines))
        } else {
            None
        };

        let should_recompute_hashtree = if hashtree_cache.is_none() {
            true
        } else if previous_command_mutated_line_text {
            hashtree_digest.as_ref() != current_digest.as_ref()
        } else {
            false
        };

        if should_recompute_hashtree {
            let content = vec_lines_to_text(&lines);
            let hashtree =
                rxedit::treesitterparser::parse_content_to_hashtree(content.as_bytes(), extension);
            dbg_("run:treesitter recomputed, size={}", &hashtree.len());
            hashtree_cache = Some(hashtree);
            hashtree_digest = current_digest.or_else(|| Some(digest_lines(&lines)));
        } else {
            dbg_("run:treesitter reused cached hashtree", &"");
        }

        let hashtree = hashtree_cache
            .as_ref()
            .expect("hashtree cache should be initialized");

        if global_config.verbose {
            if let Some(qualifier) = command.qualifier() {
                let unrecognized = match qualifier {
                    CommandQualifier::GrepQualifier(grep) => &grep.unrecognized,
                    CommandQualifier::DeclareQualifier(declare) => &declare.unrecognized,
                };

                if !unrecognized.is_empty() {
                    eprintln!("\nunrecognized flag: {:?}\n", unrecognized);
                }
            }

            if let Command::CNoop(cnoop) = command {
                eprintln!(
                    "\nunrecognized or unavailable: {:?}, message: {:?}\n",
                    cnoop.text, cnoop.message
                );
            }
        }

        lines = command.apply(lines, hashtree);

        let explain_requests = take_explain_requests();
        if explain_requests > 0 {
            let events = telemetry_events();
            for _ in 0..explain_requests {
                print_visible_lines(&lines, &global_config, show_line_numbers);
                if events.is_empty() {
                    eprintln!("Explain: no telemetry events");
                } else {
                    eprintln!(
                        "\n\n-----------------------------------------\n[explain] / processing for these arguments...\n\n"
                    );
                    let global_config = get_global_config();
                    eprintln!("{}", global_config);
                    for event in &events {
                        eprintln!("{}", event);
                    }
                }
            }

            if idx + 1 == commands.len() {
                emitted_end_state_in_loop = true;
            }
        }
        previous_command_mutated_line_text = command.mutates_line_text();
    }

    if config.in_place_output || config.output_file.is_some() {
        let output_path = if config.in_place_output {
            config.file_path.as_str()
        } else {
            config
                .output_file
                .as_deref()
                .unwrap_or(config.file_path.as_str())
        };

        let output_contents = if lines.is_empty() {
            String::new()
        } else {
            let mut output = lines
                .iter()
                .map(|line_status| line_status.line.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            output.push('\n');
            output
        };

        fs::write(output_path, output_contents)?;
        return Ok(());
    }

    if !emitted_end_state_in_loop {
        print_visible_lines(&lines, &global_config, show_line_numbers);
    }

    // if there were no actual commands to run, notifiy the user, rather than leaving an empty stdout.
    if count_active_commands == 0 {
        eprintln!(
            r#"No active commands were found to run (`macro`, `explain` and malformed commands don't count).
Please specify commands like `all`, `more`, `less`.
Or consider adding a `explain` to show any errors/rejections in arguments.
"#
        );
        process::exit(1);
    }
    Ok(())
}

fn print_visible_lines(lines: &[LineStatus], global_config: &AppConfig, show_line_numbers: bool) {
    let mut counter_not_shown = 0;

    for line_status in lines {
        if line_status.visible {
            if global_config.verbose {
                debug!("meta.key={:?}", line_status.meta.key);
            }
            if counter_not_shown > 0 && global_config.spacer {
                println!();
            }
            if show_line_numbers {
                println!("{:06} {}", line_status.line_number, line_status.line);
            } else {
                println!("{}", line_status.line);
            }
            counter_not_shown = 0;
        } else {
            counter_not_shown += 1;
        }
    }
}

/// Runtime configuration produced from command-line arguments.
struct Config {
    commands: Vec<Command>,
    file_path: String,
    commands_file: Option<String>,
    commands_file_after: Option<String>,
    output_file: Option<String>,
    in_place_output: bool,
    dump_json: bool,
}

#[derive(Debug, Parser)]
#[command(disable_help_flag = true, about = include_str!("help/about.md"), after_help = include_str!("help/examples.help"))]
/// Command-line arguments accepted by the application.
struct CliArgs {
    #[arg(required_unless_present_any = ["help_topic", "gen_config"])]
    file_path: Option<String>,
    #[arg(
        short = 'h',
        long = "help",
        value_name = "TOPIC",
        num_args = 0..=1,
        default_missing_value = "",
        action = ArgAction::Set,
        help = "Print help (optionally for a topic)"
    )]
    help_topic: Option<String>,
    #[arg(
        short = 'f',
        long = "commands-file",
        help = "prepend commands from the given file"
    )]
    commands_file: Option<String>,
    #[arg(
        short = 'F',
        long = "commands-file-after",
        help = "append commands from the given file"
    )]
    commands_file_after: Option<String>,
    #[arg(
        short = 'o',
        long = "output-file",
        conflicts_with = "in_place_output",
        help = "write buffer to the given output file.  Note that everything gets written, not just the visible lines."
    )]
    output_file: Option<String>,
    #[arg(short = 'O', long = "in-place-output", action = ArgAction::SetTrue, conflicts_with = "output_file",help="modify file directly")]
    in_place_output: bool,
    #[arg(
        short = 's',
        long = "spacer",
        help = "separate blocks of displayed lines"
    )]
    spacer: bool,

    #[arg(
        short = 'n',
        long = "line-number",
        conflicts_with = "suppress_line_number",
        help = "show line numbers (those will diverge from the source file change on `delete`, `append` or `prepend` use)"
    )]
    line_number: bool,
    #[arg(
        short = 'N',
        long = "suppress-line-number",
        conflicts_with = "line_number",
        help = "don't show line numbers (those will diverge from the source file change on `delete`, `append` or `prepend` use)"
    )]
    suppress_line_number: bool,

    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
    #[arg(long = "debug", default_value_t = false, help = "debug messages")]
    debug: bool,
    #[arg(long = "dump-json", action = ArgAction::SetTrue, help = "print the parsed hashtree JSON and exit")]
    dump_json: bool,

    #[arg(
        short = 'g',
        long = "grep-shortcodes",
        conflicts_with = "no_grep_shortcodes",
        help = "expands shortcodes (`..`→`.*`,`~~s.`→` +` ...)"
    )]
    grep_shortcodes: bool,

    #[arg(
        short = 'G',
        long = "no-grep-shortcodes",
        conflicts_with = "grep_shortcodes",
        help = "does not expand grep shortcodes"
    )]
    no_grep_shortcodes: bool,

    #[arg(long = "gen-config", action = ArgAction::SetTrue, help = "generate user config file if it does not exist")]
    gen_config: bool,
    #[arg(short = 'c', long = "config", value_name = "FILE", help = "load config from the given file")]
    config: Option<String>,
    #[arg()]
    commands: Vec<String>,
}

fn print_general_help() -> Result<(), String> {
    let mut command = CliArgs::command();
    command
        // .print_long_help()
        .print_help()
        .map_err(|e| format!("Cannot print help: {}", e))?;
    println!();
    Ok(())
}

/// checks whether a given CLI flag has been set
fn arg_present(args: &[String], short: &str, long: &str) -> bool {
    args.iter()
        .any(|arg| arg == short || arg == long || arg.strip_prefix(&format!("{}=", long)).is_some())
}

impl Config {
    /// Builds the runtime configuration from raw CLI args.
    fn new(args: &[String]) -> Result<Config, BuildError> {
        let _prefix = "Config::new";

        let cli_args =
            CliArgs::try_parse_from(args).map_err(|e| BuildError::Message(e.to_string()))?;

        let user_config = if let Some(config_file) = cli_args.config.as_deref() {
            config_utilities::read_user_config_from_path(Path::new(config_file))
                .map_err(BuildError::Message)?
        } else {
            config_utilities::read_user_config().map_err(BuildError::Message)?
        };
        let env_config = config_utilities::read_env_config().map_err(BuildError::Message)?;

        if cli_args.gen_config {
            return match config_utilities::generate_user_config_file() {
                Ok(()) => Err(BuildError::HelpDisplayed),
                Err(e) => Err(BuildError::Message(e)),
            };
        }

        if let Some(help_topic) = cli_args.help_topic.as_deref() {
            let help_result = if help_topic.is_empty() {
                print_general_help()
            } else {
                help_utilities::print_topic_help(
                    help_topic,
                    &config_utilities::user_config_path_display(),
                )
            };

            return match help_result {
                Ok(()) => Err(BuildError::HelpDisplayed),
                Err(e) => Err(BuildError::Message(e)),
            };
        }

        let file_path = cli_args.file_path.clone().ok_or_else(|| {
            BuildError::Message("Missing required argument <FILE_PATH>".to_string())
        })?;

        let mut merged_command_args: Vec<String> = Vec::new();

        if let Some(commands_file_path) = &cli_args.commands_file {
            let file_commands =
                utilities::parse_commands_file(commands_file_path).map_err(BuildError::Message)?;
            merged_command_args.extend(file_commands);
        }

        merged_command_args.extend(cli_args.commands.clone());

        if let Some(commands_file_path) = &cli_args.commands_file_after {
            let file_commands =
                utilities::parse_commands_file(commands_file_path).map_err(BuildError::Message)?;
            merged_command_args.extend(file_commands);
        }

        let shortcodes_off = arg_present(
            args,
            &format!("-{}", commandflags::NO_GREP_SHORTCODES.value),
            "--no-grep-shortcodes",
        );
        let shortcodes_on = arg_present(
            args,
            &format!("-{}", commandflags::USE_GREP_SHORTCODES.value),
            "--grep-shortcodes",
        );

        let grep_shortcodes = if shortcodes_off || shortcodes_on {
            shortcodes_on && !shortcodes_off
        } else {
            env_config
                .grep_shortcodes
                .or(user_config.grep_shortcodes)
                .unwrap_or(cli_args.grep_shortcodes)
        };

        let spacer = if arg_present(args, "-s", "--spacer") {
            cli_args.spacer
        } else {
            env_config
                .spacer
                .or(user_config.spacer)
                .unwrap_or(cli_args.spacer)
        };

        // note that clap has those flags as mutually exclusing
        let no_line_number = arg_present(args, "-N", "--suppress-line-counters");
        let yes_line_number = arg_present(args, "-n", "--line-number");
        let line_number = if no_line_number || yes_line_number {
            yes_line_number && !no_line_number
        } else {
            env_config
                .line_number
                .or(user_config.line_number)
                .unwrap_or(cli_args.line_number)
        };

        let global_config = AppConfig {
            file_path: file_path.clone(),
            spacer,
            line_number,
            grep_shortcodes,
            verbose: cli_args.verbose,
            debug: cli_args.debug,
        };
        set_global_config(global_config);

        let commands = commands_factory(&merged_command_args);

        Ok(Config {
            commands,
            file_path,
            commands_file: cli_args.commands_file,
            commands_file_after: cli_args.commands_file_after,
            output_file: cli_args.output_file,
            in_place_output: cli_args.in_place_output,
            dump_json: cli_args.dump_json,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::config_utilities::user_config_path_display;
    use super::help_utilities::{
        format_config_help, format_grep_shortcodes_help, format_user_messages_help,
        print_topic_help,
    };

    #[test]
    fn grep_shortcodes_help_lists_registry_entries() {
        let help = format_grep_shortcodes_help();
        let header = include_str!("help/shortcodes.header.md").trim();

        assert!(help.contains(header));
        assert!(help.contains("wildcard"));
        assert!(help.contains(".."));
        assert!(help.contains(".*"));
        assert!(help.contains("~~s."));
        assert!(help.contains("~~o."));
    }

    #[test]
    fn config_help_mentions_path_and_keys() {
        let help = format_config_help(&user_config_path_display());

        assert!(help.contains("config.toml"));
        assert!(help.contains("spacer"));
        // assert!(help.contains("no_display_notshown"));
        assert!(help.contains("line_number"));
        assert!(help.contains("rxedit_spacer"));
        assert!(help.contains("rxedit_line_number"));
        assert!(help.contains("rxedit_plugins_directory"));
        assert!(help.contains("not overwritten"));
    }

    #[test]
    fn topic_help_accepts_config_topic() {
        assert!(print_topic_help("config", &user_config_path_display()).is_ok());
    }

    #[test]
    fn user_messages_help_lists_message_registry_entries() {
        let help = format_user_messages_help();

        assert!(help.contains("RXE-ERR-0001"));
        assert!(help.contains("RXE-WARN-0001"));
        assert!(help.contains("RXE-WARN-0002"));
        assert!(help.contains("hashtree_key={} not found"));
        assert!(help.contains("does not have an rxedit tree-sitter grammar defined yet"));
    }

    #[test]
    fn topic_help_accepts_user_messages_topic() {
        assert!(print_topic_help("user_messages", &user_config_path_display()).is_ok());
    }
}
