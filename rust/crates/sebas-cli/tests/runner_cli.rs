use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn no_args_shows_launcher_instead_of_full_help_wall() {
    let temp_dir = unique_temp_dir("launcher");
    fs::create_dir_all(temp_dir.join(".workspace.example")).expect("workspace marker");

    let output = command_in(&temp_dir).output().expect("sebas should launch");

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("Local 122B-class model runner"));
    assert!(stdout.contains("Default engine   qwen122b"));
    assert!(stdout.contains("Try:"));
    assert!(stdout.contains("sebas --help"));
    assert!(!stdout.contains("Usage:"));

    fs::remove_dir_all(temp_dir).expect("cleanup");
}

#[test]
fn help_describes_runner_surface_without_old_branding() {
    let output = Command::new(env!("CARGO_BIN_EXE_sebas"))
        .arg("--help")
        .output()
        .expect("sebas should launch");

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("sebas engine <start|doctor|status|bench>"));
    assert!(stdout.contains("sebas demo"));
    assert!(stdout.contains("sebas model"));
    assert!(stdout.contains("local-model demo"));
    assert!(!stdout.contains("prompt mode"));
    assert!(!stdout.contains("repl"));
}

#[test]
fn model_set_writes_sebas_settings() {
    let temp_dir = unique_temp_dir("model-set");
    fs::create_dir_all(temp_dir.join(".workspace.example")).expect("workspace marker");

    let output = command_in(&temp_dir)
        .args(["model", "set", "qwen35b"])
        .output()
        .expect("sebas should launch");

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("Default engine   qwen35b"));

    let settings =
        fs::read_to_string(temp_dir.join(".sebas").join("settings.json")).expect("settings");
    assert!(settings.contains("\"defaultEngine\""));
    assert!(settings.contains("\"qwen35b\""));

    fs::remove_dir_all(temp_dir).expect("cleanup");
}

#[test]
fn arbitrary_prompt_no_longer_starts_prompt_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_sebas"))
        .arg("explain this repo")
        .output()
        .expect("sebas should launch");

    assert!(
        !output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("local model runner commands"));
}

fn command_in(cwd: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sebas"));
    command.current_dir(cwd);
    command
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("sebas-{label}-{millis}-{counter}"))
}
