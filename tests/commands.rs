use regex::Regex;
use rxedit::common::{LineStatus, MetaInfo};
use std::fs;
use std::process::Output;

static PY_SAMPLE: &str = "src/help/pysample.py";
static RS_SAMPLE: &str = "src/help/sample.rs";
static PY_IMPORT_SAMPLE: &str = "tests/fixtures/imports.py";
static RS_IMPORT_SAMPLE: &str = "tests/fixtures/imports.rs";
static PY_PROVIDERS_SAMPLE: &str = "tests/fixtures/providers.py";

pub fn extract_lines_between_patterns(
    file_path: &str,
    start_pattern: &str,
    end_pattern: &str,
) -> Result<Vec<LineStatus>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let start_regex = Regex::new(start_pattern)?;
    let end_regex = Regex::new(end_pattern)?;

    let mut result = Vec::new();
    let mut in_range = false;

    for (ix, line) in content.lines().enumerate() {
        if !in_range && start_regex.is_match(line) {
            in_range = true;
        }

        if in_range {
            result.push(LineStatus {
                line_number: ix + 1,
                visible: true,
                line: line.to_string(),
                meta: MetaInfo::default(),
            });

            if end_regex.is_match(line) {
                break;
            }
        }
    }

    Ok(result)
}

#[derive(Debug)]
pub struct RunResult {
    dbg_stdout: String,
    success: bool,
    dbg_stderror: String,
    stdout: Vec<String>,
    numlines: usize,
}

impl RunResult {
    fn build(res: &std::io::Result<Output>) -> Self {
        match res {
            Ok(output) => {
                let linesout: Vec<_> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|s| s.to_string())
                    .collect();

                let numlines = linesout.len();

                Self {
                    dbg_stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    success: output.status.success(),
                    dbg_stderror: String::from_utf8_lossy(&output.stderr).to_string(),
                    stdout: linesout,
                    numlines: numlines,
                }
            }
            Err(e) => Self {
                dbg_stdout: String::new(),
                success: false,
                dbg_stderror: e.to_string(),
                stdout: Vec::new(),
                numlines: 0,
            },
        }
    }
}

fn load_py_sample(from_: &str, to_: &str) -> Result<Vec<LineStatus>, Box<dyn std::error::Error>> {
    let file_path = PY_SAMPLE;
    extract_lines_between_patterns(file_path, from_, to_)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::{Command, Output};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn run_cli(args: &[&str]) -> std::io::Result<Output> {
        let bin = std::env::var("CARGO_BIN_EXE_rxedit")
            .expect("CARGO_BIN_EXE_rxedit is not set; run as integration test via `cargo test`");
        Command::new(bin)
            .env("rxedit_line_number", "0")
            .args(args)
            .output()
    }

    fn run_cli_with_home(args: &[&str], home_dir: &std::path::Path) -> std::io::Result<Output> {
        let bin = std::env::var("CARGO_BIN_EXE_rxedit")
            .expect("CARGO_BIN_EXE_rxedit is not set; run as integration test via `cargo test`");
        Command::new(bin)
            .env("HOME", home_dir)
            .env("rxedit_line_number", "0")
            .args(args)
            .output()
    }

    fn run_cli_with_home_and_env(
        args: &[&str],
        home_dir: &std::path::Path,
        env_key: &str,
        env_value: &str,
    ) -> std::io::Result<Output> {
        let bin = std::env::var("CARGO_BIN_EXE_rxedit")
            .expect("CARGO_BIN_EXE_rxedit is not set; run as integration test via `cargo test`");
        Command::new(bin)
            .env("HOME", home_dir)
            .env("rxedit_line_number", "0")
            .env(env_key, env_value)
            .args(args)
            .output()
    }

    #[test]
    fn test_extract_lines_between_patterns_for_function() {
        let result = load_py_sample(r"def add\(", "result");

        assert!(result.is_ok());
        let lines = result.unwrap();
        assert!(!lines.is_empty());
        assert!(lines[0].line.contains("def add"));

        // Verify the extracted content doesn't include the next function definition
        let has_next_def = lines
            .iter()
            .skip(1)
            .any(|ls| ls.line.trim().starts_with("def "));
        assert!(!has_next_def);
    }

    #[test]
    fn test_extract_no_start_match_returns_empty() {
        let result = load_py_sample(r"def this_function_does_not_exist\(", r"result");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_extract_start_match_without_end_match_reads_to_eof() {
        let result = load_py_sample(r"def add\(", r"THIS_END_PATTERN_SHOULD_NOT_EXIST");
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert!(!lines.is_empty());
        assert!(lines[0].line.contains("def add"));
    }

    #[test]
    fn test_extract_includes_end_line() {
        let result = load_py_sample(r"def add\(", r"result");
        assert!(result.is_ok());
        let lines = result.unwrap();
        let last = lines.last().expect("expected at least one extracted line");
        assert!(last.line.contains("result"));
    }

    #[test]
    fn test_prepend_inserts_before_matching_line() {
        let out = run_cli(&[
            RS_SAMPLE,
            "all",
            "prepend::fn make_message::// inserted before make_message::F",
        ]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);

        let target_idx = res
            .stdout
            .iter()
            .position(|line| line.contains("fn make_message() -> String"))
            .expect("missing target function line");

        assert!(target_idx > 0, "inserted line should be before function");
        assert_eq!(
            res.stdout[target_idx - 1],
            "// inserted before make_message"
        );
    }

    #[test]
    fn test_append_with_indent_keep() {
        let out = run_cli(&[PY_SAMPLE, "all", "append::def add::# inserted doc::IF"]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);

        let target_idx = res
            .stdout
            .iter()
            .position(|line| line.contains("def add(self, a: int, b: int) -> int:"))
            .expect("missing python add definition line");

        assert!(
            target_idx + 1 < res.stdout.len(),
            "inserted line should follow target"
        );
        assert_eq!(res.stdout[target_idx + 1], "    # inserted doc");
    }

    #[test]
    fn test_rs_sample_declare_fixture_signature_with_colon_colon_in_body() {
        // Equivalent of: rxedit ../src/help/sample.rs d::fixture_signature_with_colon_colon_in_body
        // Should return only the function signature
        let out = run_cli(&[RS_SAMPLE, "d::fixture_signature_with_colon_colon_in_body"]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);

        assert_eq!(
            res.numlines, 1,
            "expected exactly one line (signature only)\nstdout={}",
            res.dbg_stdout
        );
        assert!(
            res.stdout[0].contains("fn fixture_signature_with_colon_colon_in_body"),
            "expected function signature in output\nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_basic_declare() {
        // Equivalent of: rxedit ../src/help/pysample.py d::add

        // Verify the binary exists and is executable
        let bin = std::env::var("CARGO_BIN_EXE_rxedit")
            .expect("CARGO_BIN_EXE_rxedit is not set; run as integration test via `cargo test`");

        assert!(
            std::path::Path::new(&bin).exists(),
            "Binary not found at: {}",
            bin
        );

        let py_sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(PY_SAMPLE);
        assert!(
            py_sample_path.exists(),
            "PY_SAMPLE file not found at: {}",
            py_sample_path.display()
        );

        assert!(
            std::path::Path::new(PY_SAMPLE).exists(),
            "PY_SAMPLE file not found at: {}",
            PY_SAMPLE
        );

        let out = run_cli(&[PY_SAMPLE, "d::add"]);
        let res = RunResult::build(&out);

        assert!(res.success == true, "{}", res.dbg_stderror);
        assert!(res.numlines == 1, "len:{}", res.numlines);

        assert!(
            res.stdout[0].contains("def add"),
            "stdout={}",
            res.dbg_stdout
        );
        assert!(res.stdout[0].ends_with("int:"), "stdout={}", res.dbg_stdout);
    }

    #[test]
    fn test_declare_parent_filter_supports_comma_or() {
        let out = run_cli(&[PY_SAMPLE, "d::Calculator,Nope/add"]);

        let res = RunResult::build(&out);
        assert!(res.success, "{}", res.dbg_stderror);
        assert!(
            res.stdout.iter().any(|line| line.contains("def add(")),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_declare_parent_filter_supports_double_dash_negation() {
        let out = run_cli(&[PY_SAMPLE, "d::Calculator--Calculator/add"]);

        let res = RunResult::build(&out);
        assert!(res.success, "{}", res.dbg_stderror);
        assert_eq!(res.numlines, 0, "stdout={}", res.dbg_stdout);
    }

    #[test]
    fn test_declare_parent_filter_url_obj_or_with_flags() {
        let out = run_cli(&[PY_PROVIDERS_SAMPLE, "d::url,obj/::io"]);

        let res = RunResult::build(&out);
        assert!(res.success, "{}", res.dbg_stderror);

        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("class UrlProvider:")),
            "stdout={}",
            res.dbg_stdout
        );
        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("class ObjProvider:")),
            "stdout={}",
            res.dbg_stdout
        );
        assert!(
            !res.stdout
                .iter()
                .any(|line| line.contains("class OtherProvider:")),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_declare_parent_filter_url_obj_with_negated_obj() {
        let out = run_cli(&[PY_PROVIDERS_SAMPLE, "d::url,obj--obj/::io"]);

        let res = RunResult::build(&out);
        assert!(res.success, "{}", res.dbg_stderror);

        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("class UrlProvider:")),
            "stdout={}",
            res.dbg_stdout
        );
        assert!(
            !res.stdout
                .iter()
                .any(|line| line.contains("class ObjProvider:")),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_declare_where_type_python_functions_only() {
        let out = run_cli(&[PY_SAMPLE, "d::::wt=f."]);

        let res = RunResult::build(&out);
        assert!(res.success, "{}", res.dbg_stderror);

        assert!(
            res.stdout
                .iter()
                .all(|line| !line.contains("class Calculator:")),
            "stdout={}",
            res.dbg_stdout
        );
        assert!(
            res.stdout.iter().any(|line| line.contains("def add(")),
            "stdout={}",
            res.dbg_stdout
        );
        assert!(
            res.stdout.iter().any(|line| line.contains("def multiply(")),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_declare_where_type_python_class_only() {
        let out = run_cli(&[PY_SAMPLE, "d::::wt=c."]);

        let res = RunResult::build(&out);
        assert!(res.success, "{}", res.dbg_stderror);

        assert_eq!(res.numlines, 1, "stdout={}", res.dbg_stdout);
        assert!(
            res.stdout[0].contains("class Calculator:"),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_declare_where_type_unsupported_is_ignored() {
        let baseline = RunResult::build(&run_cli(&[PY_SAMPLE, "d::"]));
        assert!(baseline.success, "{}", baseline.dbg_stderror);

        let filtered = RunResult::build(&run_cli(&[PY_SAMPLE, "d::::wt=z."]));
        assert!(filtered.success, "{}", filtered.dbg_stderror);

        assert_eq!(baseline.stdout, filtered.stdout);
    }

    #[test]
    fn test_declare_after_uses_signature_end_as_anchor() {
        let out = run_cli(&[PY_SAMPLE, "d::add::A2"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "{}", res.dbg_stderror);

        assert_eq!(res.numlines, 3, "stdout={}", res.dbg_stdout);
        assert!(
            res.stdout[0].contains("def add"),
            "first line should be declaration signature\nstdout={}",
            res.dbg_stdout
        );
        assert!(
            res.stdout[1].contains("\"\"\"Add two numbers"),
            "line after signature should be declaration-adjacent\nstdout={}",
            res.dbg_stdout
        );
        assert!(
            res.stdout[2].contains("result_add = a + b"),
            "second line after signature should be included\nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_declare_before_uses_signature_start_as_anchor() {
        let out = run_cli(&[PY_SAMPLE, "d::add::B2"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "{}", res.dbg_stderror);

        assert_eq!(res.numlines, 3, "stdout={}", res.dbg_stdout);
        assert!(
            res.stdout[2].contains("def add"),
            "last line should be declaration signature\nstdout={}",
            res.dbg_stdout
        );
        assert_eq!(res.stdout[1].trim(), "", "stdout={}", res.dbg_stdout);
    }

    #[test]
    fn test_more_showowner_prefers_innermost_python_method_owner() {
        let out = run_cli(&[PY_SAMPLE, "m::append::o"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "{}", res.dbg_stderror);

        let has_add_owner = res
            .stdout
            .iter()
            .any(|line| line.contains("def add(self, a: int, b: int) -> int:"));
        let has_multiply_owner = res
            .stdout
            .iter()
            .any(|line| line.contains("def multiply(self,"));
        let has_class_owner = res
            .stdout
            .iter()
            .any(|line| line.contains("class Calculator:"));

        assert!(has_add_owner, "stdout={}", res.dbg_stdout);
        assert!(has_multiply_owner, "stdout={}", res.dbg_stdout);
        assert!(!has_class_owner, "stdout={}", res.dbg_stdout);
    }

    #[test]
    fn test_more_showowner_keeps_multiline_owner_signature_lines() {
        let out = run_cli(&[PY_SAMPLE, "m::append::o"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "{}", res.dbg_stderror);

        let has_multiply_sig_line_1 = res
            .stdout
            .iter()
            .any(|line| line.contains("def multiply(self,"));
        let has_multiply_sig_line_2 = res
            .stdout
            .iter()
            .any(|line| line.contains("a: int, b: int) -> int:"));

        assert!(has_multiply_sig_line_1, "stdout={}", res.dbg_stdout);
        assert!(has_multiply_sig_line_2, "stdout={}", res.dbg_stdout);
    }

    #[test]
    fn test_import_python_matches_from_module_only() {
        let out = run_cli(&[PY_IMPORT_SAMPLE, "i::pathlib"]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);

        assert_eq!(res.numlines, 1, "stdout={}", res.dbg_stdout);
        assert!(
            res.stdout[0].contains("from pathlib import Path"),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_import_python_case_insensitive_flag_is_rightmost() {
        let out = run_cli(&[PY_IMPORT_SAMPLE, "i::PATHLIB::i"]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);

        assert_eq!(res.numlines, 1, "stdout={}", res.dbg_stdout);
        assert!(
            res.stdout[0].contains("from pathlib import Path"),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_import_rust_grouped_import_drops_group_side() {
        let out = run_cli(&[RS_IMPORT_SAMPLE, "i::collections"]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);

        assert_eq!(res.numlines, 1, "stdout={}", res.dbg_stdout);
        assert!(
            res.stdout[0].contains("use std::collections::HashMap;"),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_import_rust_matches_multiple_common_lines() {
        let out = run_cli(&[RS_IMPORT_SAMPLE, "i::common"]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);

        assert_eq!(res.numlines, 2, "stdout={}", res.dbg_stdout);
        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("use crate::common::CommandQualifier;")),
            "stdout={}",
            res.dbg_stdout
        );
        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("use crate::common::{GrepCommandQualifier,LineStatus};")),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_import_rust_pub_mod_and_pub_use_both_match_commands() {
        let out = run_cli(&[RS_IMPORT_SAMPLE, "i::commands"]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);

        assert_eq!(res.numlines, 2, "stdout={}", res.dbg_stdout);
        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("pub mod commands;")),
            "stdout={}",
            res.dbg_stdout
        );
        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("pub use commands::all::CAll;")),
            "stdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_import_rust_super_import_is_not_added_to_hashtree() {
        let out = run_cli(&[RS_IMPORT_SAMPLE, "i::preformat"]);

        let res = RunResult::build(&out);
        assert!(res.success, "stderr:\n{}", res.dbg_stderror);
        assert_eq!(res.numlines, 0, "stdout={}", res.dbg_stdout);
    }

    #[test]
    fn test_grepmacros_from_config_file_applies_without_g_flag() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let home_dir = std::env::temp_dir().join(format!("rxedit-config-test-{}", now));
        let config_dir = home_dir.join(".config").join("rxedit");

        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(config_dir.join("config.toml"), "grep_shortcodes = true\n")
            .expect("write config file");

        let out = run_cli_with_home(&[PY_SAMPLE, "m::~~pl."], &home_dir);
        let res = RunResult::build(&out);

        assert!(res.success, "stderr:\n{}", res.dbg_stderror);
        assert!(
            res.numlines > 0,
            "expected expanded shortcodes to match lines, stdout={} stderr={}",
            res.dbg_stdout,
            res.dbg_stderror
        );

        let _ = fs::remove_dir_all(&home_dir);
    }

    #[test]
    fn test_grep_shortcodes_env_overrides_config_file() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let home_dir = std::env::temp_dir().join(format!("rxedit-env-config-test-{}", now));
        let config_dir = home_dir.join(".config").join("rxedit");

        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(config_dir.join("config.toml"), "grep_shortcodes = false\n")
            .expect("write config file");

        let out = run_cli_with_home_and_env(
            &[PY_SAMPLE, "m::~~pl."],
            &home_dir,
            "rxedit_grep_shortcodes",
            "true",
        );
        let res = RunResult::build(&out);

        assert!(res.success, "stderr:\n{}", res.dbg_stderror);
        assert!(
            res.numlines > 0,
            "expected env var to override config and enable macro expansion, stdout={} stderr={}",
            res.dbg_stdout,
            res.dbg_stderror
        );

        let _ = fs::remove_dir_all(&home_dir);
    }

    #[test]
    fn pysample_declare_and_add_with_after2() {
        // Equivalent of: rxedit ../src/help/pysample.py d and::add::A2
        // which should return `def add(` and 2 lines after its definition
        //  1. find all declarations
        //  2. keep the visibles only if they match `add` and then show 2 lines after
        let out = run_cli(&[PY_SAMPLE, "d::", "and::add::A2"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        let exp = "a + b";
        assert!(
            res.stdout[res.numlines - 1].contains(exp),
            "last line should contain `{exp}` \nstdout={}",
            res.dbg_stdout
        );

        let exp = "def add(";
        assert!(
            res.stdout[0].contains(exp),
            "first line should contain `{exp}` \nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn pysample_declare_body_add() {
        // Equivalent of: rxedit ../src/help/pysample.py d::add::b
        // which should return `def add(` plus its body
        let out = run_cli(&[PY_SAMPLE, "d::add::b"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        let exp = "result_add";
        assert!(
            res.stdout[res.numlines - 1].ends_with(exp),
            "last line should end with `{exp}` \nstdout={}",
            res.dbg_stdout
        );

        let exp = "def add(";
        assert!(
            res.stdout[0].contains(exp),
            "first line should contain `{exp}` \nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn sample_rs_declare_body_make_message() {
        // Equivalent of: rxedit ../src/help/sample.rs d::make_message::b
        let out = run_cli(&[RS_SAMPLE, "d::make_message::b"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("fn make_message")),
            "expected declaration line in output\nstdout={} ",
            res.dbg_stdout
        );

        assert!(
            res.stdout
                .iter()
                .any(|line| line.contains("app.greet(\"world\")")),
            "expected function body line in output\nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_delete_basic() {
        // Equivalent of: rxedit ../src/help/sample.rs a:: delete::print_
        // Should show all lines except those containing "print_"
        let out = run_cli(&[RS_SAMPLE, "a::", "delete::print_"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        // Verify that lines with "print_" are not in the output
        assert!(
            !res.stdout.iter().any(|line| line.contains("print_")),
            "Expected no lines containing 'print_' in output\nstdout={}",
            res.dbg_stdout
        );

        // Verify that other lines still exist
        assert!(
            res.stdout.iter().any(|line| line.contains("fn ")),
            "Expected some function definitions in output\nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_invert_alone_flips_initial_hidden_to_visible() {
        let out = run_cli(&[RS_SAMPLE, "invert"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        let expected_lines = fs::read_to_string(RS_SAMPLE)
            .expect("cannot read RS_SAMPLE")
            .lines()
            .count();

        assert_eq!(
            res.numlines, expected_lines,
            "invert should make all lines visible when initial state is hidden\nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_invert_after_all_hides_hits_and_shows_non_hits() {
        let out = run_cli(&[RS_SAMPLE, "a::greet", "invert"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        assert!(
            res.numlines > 0,
            "expected non-empty output after inversion\nstdout={}",
            res.dbg_stdout
        );

        assert!(
            !res.stdout.iter().any(|line| line.contains("greet")),
            "expected all greet hits to be hidden after invert\nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_delete_case_insensitive() {
        // Equivalent of: rxedit ../src/help/sample.rs a:: delete::GREET::i
        // Should delete lines matching "greet" (case-insensitive)
        let out = run_cli(&[RS_SAMPLE, "a::", "delete::GREET::i"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        // Verify that lines with "greet" or "GREET" are deleted
        assert!(
            !res.stdout
                .iter()
                .any(|line| line.to_lowercase().contains("greet")),
            "Expected no lines containing 'greet' (case-insensitive) in output\nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_delete_preserves_line_numbers() {
        // Equivalent of: rxedit -n ../src/help/sample.rs a:: delete::print_
        // Should preserve line numbers for remaining lines
        let out = run_cli(&["-n", RS_SAMPLE, "a::", "delete::print_"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        // With -n flag, lines should have line numbers
        // Check that at least some lines have the line number format (6 digits + space)
        let lines_with_numbers = res
            .stdout
            .iter()
            .filter(|line| line.len() > 6 && line[0..6].chars().all(|c| c.is_ascii_digit()))
            .count();

        assert!(
            lines_with_numbers > 0,
            "Expected line numbers in output\nstdout={}",
            res.dbg_stdout
        );
    }

    #[test]
    fn test_delete_help() {
        // Verify that help for delete command is available
        let out = run_cli(&["--help", "delete"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        let stdout = res.dbg_stdout.to_lowercase();
        assert!(
            stdout.contains("delete"),
            "Expected 'delete' in help output\nstdout={}",
            stdout
        );
    }

    #[test]
    fn test_change_help() {
        // Verify that help for change command is available
        let out = run_cli(&["--help", "change"]);

        let res = RunResult::build(&out);
        assert!(res.success == true, "stderr:\n{}", res.dbg_stderror);

        let stdout = res.dbg_stdout.to_lowercase();
        assert!(
            stdout.contains("change"),
            "Expected 'change' in help output\nstdout={}",
            stdout
        );
    }

    #[test]
    fn test_cli_help() {
        let out = run_cli(&["--help"]).expect("failed to invoke rxedit --help");
        assert!(out.status.success());

        let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
        assert!(
            stdout.contains("usage") || stdout.contains("help"),
            "stdout was:\n{}",
            stdout
        );
    }
}
