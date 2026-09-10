/// simplistic test runner for the cli.  reads individual toml files for the command line arguments
/// and then checks for the presence of wanted words, and the absence
/// of unwanted words, in the output.

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct CliCase {
    file_path: String,
    commands: Vec<String>,
    wanted: Option<Vec<String>>,
    unwanted: Option<Vec<String>>,
}

fn cli_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_cases")
}

fn load_case(name: &str) -> CliCase {
    let contents = fs::read_to_string(cli_dir().join(name))
        .unwrap_or_else(|err| panic!("failed to read {name}: {err}"));
    toml::from_str(&contents).unwrap_or_else(|err| panic!("failed to parse {name}: {err}"))
}

fn run_cli(args: &[String]) -> std::process::Output {
    let bin = std::env::var("CARGO_BIN_EXE_rxedit")
        .expect("CARGO_BIN_EXE_rxedit is not set; run under cargo test");

    Command::new(bin)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("rxedit_line_number", "0")
        .args(args)
        .output()
        .expect("failed to execute rxedit binary")
}

fn assert_case(cli_name: &str) {
    let case = load_case(cli_name);
    let mut args = vec![case.file_path.clone()];
    args.extend(case.commands.clone());

    let output = run_cli(&args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "case {cli_name} failed; stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    for want in case.wanted.unwrap_or_default() {
        assert!(
            stdout.contains(&want),
            "case {cli_name} missing wanted text: {want}\nstdout:\n{stdout}"
        );
    }

    for unwanted in case.unwanted.unwrap_or_default() {
        assert!(
            !stdout.contains(&unwanted),
            "case {cli_name} unexpectedly contained unwanted text: {unwanted}\nstdout:\n{stdout}"
        );
    }
}

#[test]
fn cli_pysample_add() {
    assert_case("pysample_add.toml");
}

#[test]
fn cli_pymodify() {
    assert_case("pymodify.toml");
}


#[test]
fn cli_rust_signature() {
    assert_case("rust_signature.toml");
}

#[test]
fn cli_pycomplex() {
    assert_case("pycomplex.toml");
}

#[test]
fn cli_pyfixed() {
    assert_case("pyfixed.toml");
}

#[test]
fn cli_helpmain() {
    assert_case("helpmain.toml");
}

#[test]
fn cli_helpappend() {
    assert_case("helpappend.toml");
}

#[test]
fn cli_pymacro() {
    assert_case("pymacro.toml");
}
