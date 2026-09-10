use std::fmt::Write;

use rxedit::constants::FlagDef;
use rxedit::constants::{env_vars, grep_shortcodes};
use rxedit::user_messages;

use rxedit::commands::all::get_constant_definition as get_constant_definition_all;
use rxedit::commands::and::get_constant_definition as get_constant_definition_and;
use rxedit::commands::appendprepend::get_constant_definition as get_constant_definition_insert;
use rxedit::commands::change::get_constant_definition as get_constant_definition_change;
use rxedit::commands::declarations::get_constant_definition as get_constant_definition_declarations;
use rxedit::commands::delete::get_constant_definition as get_constant_definition_delete;
use rxedit::commands::imports::get_constant_definition as get_constant_definition_imports;
use rxedit::commands::invert::get_constant_definition as get_constant_definition_invert;
use rxedit::commands::less::get_constant_definition as get_constant_definition_less;
use rxedit::commands::lines::get_constant_definition as get_constant_definition_lines;
use rxedit::commands::more::get_constant_definition as get_constant_definition_more;
use rxedit::loader_for_constants;

pub fn format_grep_shortcodes_help() -> String {
    let key_width = grep_shortcodes::searchfield()
        .iter()
        .map(|grep_shortcode| grep_shortcode.key.len())
        .max()
        .unwrap_or(3);
    let shortcode_width = grep_shortcodes::searchfield()
        .iter()
        .map(|grep_shortcode| grep_shortcode.macrocode.len())
        .max()
        .unwrap_or(5);
    let expansion_width = grep_shortcodes::searchfield()
        .iter()
        .map(|grep_shortcode| grep_shortcode.expansion.len())
        .max()
        .unwrap_or(9);

    let mut text = String::new();
    writeln!(&mut text, "{}", include_str!("help/shortcodes.header.md")).unwrap();
    writeln!(
        &mut text,
        "{:<key_width$}  {:<shortcode_width$}  {:<expansion_width$}  Description",
        "Key",
        "Macro",
        "Expansion",
        key_width = key_width,
        shortcode_width = shortcode_width,
        expansion_width = expansion_width,
    )
    .unwrap();
    writeln!(
        &mut text,
        "{:-<key_width$}  {:-<shortcode_width$}  {:-<expansion_width$}  {:-<11}",
        "",
        "",
        "",
        "",
        key_width = key_width,
        shortcode_width = shortcode_width,
        expansion_width = expansion_width,
    )
    .unwrap();

    for grep_shortcode in grep_shortcodes::searchfield() {
        writeln!(
            &mut text,
            "{:<key_width$}  {:<shortcode_width$}  {:<expansion_width$}  {}",
            grep_shortcode.key,
            grep_shortcode.macrocode,
            grep_shortcode.expansion,
            grep_shortcode.description,
            key_width = key_width,
            shortcode_width = shortcode_width,
            expansion_width = expansion_width,
        )
        .unwrap();
    }

    writeln!(&mut text, "{}", include_str!("help/shortcodes.footer.md")).unwrap();

    text
}

const HDR_COL2_FLAGS: &str = "Format";

pub fn format_flag_help(section_title: &str, flags: &[FlagDef]) -> String {
    let name_width = flags.iter().map(|flag| flag.name.len()).max().unwrap_or(4);
    let value_width = flags.iter().map(|flag| flag.value.len()).max().unwrap_or(5);
    let example_width = flags
        .iter()
        .map(|flag| flag.example.len())
        .max()
        .unwrap_or(7);

    let mut text = String::new();
    writeln!(&mut text, "\n## {section_title}\n").unwrap();
    writeln!(
        &mut text,
        "{:<name_width$}  {:<value_width$}  {:<example_width$}  Description",
        "Name",
        HDR_COL2_FLAGS,
        "Example",
        name_width = name_width,
        value_width = value_width,
        example_width = example_width,
    )
    .unwrap();
    writeln!(
        &mut text,
        "{:-<name_width$}  {:-<value_width$}  {:-<example_width$}  {:-<11}",
        "",
        "",
        "",
        "",
        name_width = name_width,
        value_width = value_width,
        example_width = example_width,
    )
    .unwrap();

    for flag in flags {
        writeln!(
            &mut text,
            "{:<name_width$}  {:<value_width$}  {:<example_width$}  {}",
            flag.name,
            flag.value,
            flag.example,
            flag.help,
            name_width = name_width,
            value_width = value_width,
            example_width = example_width,
        )
        .unwrap();
    }

    text
}

fn print_flag_help(section_title: &str, flags: &[FlagDef]) {
    print!("{}", format_flag_help(section_title, flags));
}

pub fn format_config_help(config_path: &str) -> String {
    include_str!("help/config.md")
        .replace("{CONFIG_PATH}", config_path)
        .replace("{ENV_SPACER}", env_vars::SPACER)
        .replace("{ENV_LINE_NUMBER}", env_vars::LINE_NUMBER)
        .replace("{ENV_GREP_SHORTCODES}", env_vars::GREP_SHORTCODES)
    .replace("{ENV_PLUGINS_DIRECTORY}", env_vars::PLUGINS_DIRECTORY)
}

pub fn format_user_messages_help() -> String {
    let key_width = user_messages::ALL
        .iter()
        .map(|message| message.key.len())
        .max()
        .unwrap_or(3);
    let code_width = user_messages::ALL
        .iter()
        .map(|message| message.code.len())
        .max()
        .unwrap_or(4);

    let mut text = String::new();
    writeln!(&mut text, "\n## User messages\n").unwrap();
    writeln!(
        &mut text,
        "{:<key_width$}  {:<code_width$}  Body",
        "Key",
        "Code",
        key_width = key_width,
        code_width = code_width,
    )
    .unwrap();
    writeln!(
        &mut text,
        "{:-<key_width$}  {:-<code_width$}  {:-<4}",
        "",
        "",
        "",
        key_width = key_width,
        code_width = code_width,
    )
    .unwrap();

    for message in user_messages::ALL {
        writeln!(
            &mut text,
            "{:<key_width$}  {:<code_width$}  {}",
            message.key,
            message.code,
            message.body,
            key_width = key_width,
            code_width = code_width,
        )
        .unwrap();
        writeln!(
            &mut text,
            "{:indent$}  {}",
            "",
            message.details,
            indent = key_width + code_width + 2
        )
        .unwrap();
        writeln!(&mut text).unwrap();
    }

    text
}

fn print_grep_shortcodes_help() {
    print!("{}", format_grep_shortcodes_help());
}

fn print_config_help(config_path: &str) {
    print!("{}", format_config_help(config_path));
}

fn print_user_messages_help() {
    print!("{}", format_user_messages_help());
}

const HELPTEXT_FLAGS_SEARCH: &str = "Search flags";

/// the main entry point for dealing with `rxedit --help <topic` on the command line
pub fn print_topic_help(topic: &str, config_path: &str) -> Result<(), String> {
    match topic {
        "explain" => {
            println!("{}", include_str!("help/commands/explain.md"));
            Ok(())
        }
        "all" => {
            println!("{}", include_str!("help/commands/all.md"));
            let command_info = get_constant_definition_all();
            print_flag_help(HELPTEXT_FLAGS_SEARCH, command_info.flags);
            println!("{}", include_str!("help/commands/_searches.footer.md"));
            Ok(())
        }
        "and" => {
            println!("{}", include_str!("help/commands/and.md"));
            print_flag_help("and flags", get_constant_definition_and().flags);
            println!("{}", include_str!("help/commands/_searches.footer.md"));
            Ok(())
        }
        "append" | "prepend" => {
            println!("{}", include_str!("help/append_prepend.md"));
            print_flag_help(
                "prepend/append flags",
                get_constant_definition_insert().flags,
            );
            println!("{}", include_str!("help/commands/_modifying.footer.md"));
            Ok(())
        }
        "chaining" => {
            println!("{}", include_str!("help/chaining.md"));
            Ok(())
        }
        "change" => {
            println!("{}", include_str!("help/commands/change.md"));
            print_flag_help(
                "prepend/append flags",
                get_constant_definition_change().flags,
            );
            println!("{}", include_str!("help/commands/_modifying.footer.md"));
            Ok(())
        }
        "config" => {
            print_config_help(config_path);
            Ok(())
        }
        "declarations" => {
            let flags_both = [
                get_constant_definition_declarations().flags,
                get_constant_definition_more().flags,
            ]
            .concat();

            println!("{}", include_str!("help/commands/declarations.md"));
            print_flag_help("Declarations", &flags_both);
            println!("{}", include_str!("help/commands/declarations.footer.md"));
            Ok(())
        }
        "delete" => {
            println!("{}", include_str!("help/commands/delete.md"));
            print_flag_help("delete flags", get_constant_definition_delete().flags);
            println!("{}", include_str!("help/commands/_modifying.footer.md"));
            Ok(())
        }
        "flags" => {
            println!("{}", include_str!("help/flags.md"));
            Ok(())
        }
        "imports" => {
            println!("{}", include_str!("help/commands/imports.md"));
            print_flag_help(
                HELPTEXT_FLAGS_SEARCH,
                get_constant_definition_imports().flags,
            );
            println!("{}", include_str!("help/commands/imports.footer.md"));
            Ok(())
        }
        "invert" => {
            println!("{}", include_str!("help/commands/invert.md"));
            print_flag_help("Invert flags:", get_constant_definition_invert().flags);
            println!("{}", include_str!("help/commands/invert.footer.md"));
            Ok(())
        }
        "languages" => {
            println!("{}", include_str!("help/languages.md"));
            Ok(())
        }
        "less" => {
            println!("{}", include_str!("help/commands/less.md"));
            let command_info = get_constant_definition_less();
            print_flag_help(HELPTEXT_FLAGS_SEARCH, command_info.flags);
            println!("{}", include_str!("help/commands/_searches.footer.md"));
            println!("{}", include_str!("help/commands/less.caveat.md"));
            Ok(())
        }
        "lines" => {
            println!("{}", include_str!("help/commands/lines.md"));
            print_flag_help("Lines flags", get_constant_definition_lines().flags);
            Ok(())
        }
        "macro" | "macros" => {
            println!("{}", include_str!("help/commands/macro.md"));
            Ok(())
        }
        "more" => {
            let command_info = get_constant_definition_more();
            println!("{}", include_str!("help/commands/more.md"));
            print_flag_help(HELPTEXT_FLAGS_SEARCH, command_info.flags);
            println!("{}", include_str!("help/commands/_searches.footer.md"));
            Ok(())
        }
        "noop" => {
            println!("{}", include_str!("help/commands/noop.md"));
            Ok(())
        }
        "sample" => {
            println!("{}", include_str!("help/sample.header.md"));
            println!("# Sample Rust file: sample.rs\n");
            println!("{}", include_str!("help/sample.rs"));
            println!("\n\n# Sample Python file: pysample.py\n");
            println!("{}", include_str!("help/pysample.py"));
            Ok(())
        }
        "shortcodes" => {
            print_grep_shortcodes_help();
            Ok(())
        }
        "user_messages" => {
            print_user_messages_help();
            Ok(())
        }
        _ => Err(format!(
            "Unknown help topic '{}'. Available topics: {}",
            topic,
            loader_for_constants::help_topics().join(", ")
        )),
    }
}
