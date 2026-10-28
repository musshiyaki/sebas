use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
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
    assert!(stdout.contains("sebas chat"));
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
    assert!(stdout.contains("sebas"));
    assert!(stdout.contains("sebas chat"));
    assert!(stdout.contains("sebas codex <proxy|config|doctor>"));
    assert!(stdout.contains("--agent-mode normal|one-shot-exec"));
    assert!(stdout.contains("--session-mode none|delta"));
    assert!(stdout.contains("sebas engine <start|doctor|status|bench>"));
    assert!(stdout.contains("sebas demo"));
    assert!(stdout.contains("sebas model"));
    assert!(stdout.contains("local-model demo"));
    assert!(!stdout.contains("prompt mode"));
    assert!(!stdout.contains("repl"));
}

#[test]
fn codex_config_prints_responses_profile() {
    let temp_dir = unique_temp_dir("codex-config");
    write_minimal_manifest(&temp_dir);

    let output = command_in(&temp_dir)
        .args([
            "codex",
            "config",
            "--engine",
            "qwen122b",
            "--listen",
            "127.0.0.1:61335",
            "--max-tokens",
            "256",
        ])
        .output()
        .expect("sebas should launch");

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("~/.codex/sebas-qwen122b.config.toml"));
    assert!(stdout.contains("model_provider = \"sebas-qwen122b\""));
    assert!(stdout.contains("base_url = \"http://127.0.0.1:61335/v1\""));
    assert!(stdout.contains("wire_api = \"responses\""));
    assert!(stdout.contains("model_max_output_tokens = 256"));

    fs::remove_dir_all(temp_dir).expect("cleanup");
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
fn chat_quit_exits_without_running_model() {
    let temp_dir = unique_temp_dir("chat-quit");
    write_minimal_manifest(&temp_dir);

    let mut child = command_in(&temp_dir)
        .arg("chat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sebas should launch");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"/quit\n")
        .expect("write quit");
    let output = child.wait_with_output().expect("wait");

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("Sebas local chat"));
    assert!(stdout.contains("Type /quit to exit."));
    assert!(stdout.contains("sebas>"));

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

#[cfg(unix)]
#[test]
fn demo_bench_and_chat_use_manifest_repo_path_for_both_engines() {
    for engine in ["qwen122b", "qwen35b"] {
        let temp_dir = unique_temp_dir(engine);
        let repo = write_engine_fixture(&temp_dir, engine);
        let size = engine.strip_prefix("qwen").expect("engine size");
        let infer_dir = repo.join("metal_infer").canonicalize().expect("infer dir");
        let model_dir = temp_dir.join("model");

        let demo = command_in(&temp_dir)
            .args([
                "demo",
                "--engine",
                engine,
                "--tokens",
                "7",
                "fixture prompt",
                "--",
                "fixture-extra",
            ])
            .env("SEBAS_DEMO_HARDWARE", "Fixture Mac")
            .output()
            .expect("demo should launch fixture helper");
        assert_success(&demo);
        let stdout = String::from_utf8(demo.stdout).expect("stdout should be utf8");
        assert!(stdout.contains(&format!("helper:run_{size}.sh")));
        assert!(stdout.contains(&format!("cwd:{}", infer_dir.display())));
        assert!(stdout.contains(&format!("model:{}", model_dir.display())));
        assert!(stdout.contains("prompt:fixture prompt"));
        assert!(stdout.contains("tokens:7"));
        assert!(stdout.contains("extra:fixture-extra"));

        let bench = command_in(&temp_dir)
            .args([
                "engine",
                "bench",
                "--engine",
                engine,
                "--lang",
                "en",
                "--case",
                "short",
                "--short-tokens",
                "7",
                "--long-tokens",
                "11",
                "--",
                "fixture-extra",
            ])
            .output()
            .expect("bench should launch fixture helper");
        assert_success(&bench);
        let stdout = String::from_utf8(bench.stdout).expect("stdout should be utf8");
        assert!(stdout.contains(&format!("helper:bench_{size}.sh")));
        assert!(stdout.contains(&format!("cwd:{}", infer_dir.display())));
        assert!(stdout.contains(&format!("model:{}", model_dir.display())));
        assert!(stdout.contains("lang:en"));
        assert!(stdout.contains("case:short"));
        assert!(stdout.contains("short:7"));
        assert!(stdout.contains("long:11"));
        assert!(stdout.contains("extra:fixture-extra"));

        let chat = output_with_input(
            command_in(&temp_dir).args(["chat", "--engine", engine, "--tokens", "9"]),
            b"fixture chat\n/quit\n",
        );
        assert_success(&chat);
        let stdout = String::from_utf8(chat.stdout).expect("stdout should be utf8");
        assert!(stdout.contains(&format!("helper:run_{size}.sh")));
        assert!(stdout.contains(&format!("cwd:{}", infer_dir.display())));
        assert!(stdout.contains(&format!("model:{}", model_dir.display())));
        assert!(stdout.contains("prompt:fixture chat"));
        assert!(stdout.contains("tokens:9"));

        fs::remove_dir_all(temp_dir).expect("cleanup");
    }
}

#[test]
fn doctor_success_reports_only_static_path_checks() {
    let temp_dir = unique_temp_dir("doctor-paths");
    write_engine_fixture(&temp_dir, "qwen122b");

    let output = command_in(&temp_dir)
        .args(["doctor", "qwen122b"])
        .output()
        .expect("doctor should launch");

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("OK  qwen122b: path checks only"));
    assert!(stdout.contains("Not verified: model layout, engine compatibility, or inference."));
    assert!(!stdout.contains("helper:"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("infer must not run"));

    fs::remove_dir_all(temp_dir).expect("cleanup");
}

#[test]
fn doctor_reports_missing_repo_binary_and_helpers_with_122b_guidance() {
    let temp_dir = unique_temp_dir("doctor-missing");
    let repo = write_engine_fixture(&temp_dir, "qwen122b");
    fs::remove_dir_all(&repo).expect("remove fixture repo");

    let output = command_in(&temp_dir)
        .args(["engine", "doctor", "--engine", "qwen122b"])
        .output()
        .expect("doctor should launch");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("engine repo is missing or not a directory:"));
    assert!(stdout.contains("infer binary is missing or not a regular file:"));
    assert!(stdout.contains("run_122b.sh"));
    assert!(stdout.contains("bench_122b.sh"));
    assert!(stdout.contains("docs/qwen122b-runbook.md"));
    assert!(stdout.contains("compatible 122B engine and model preparation"));
    assert!(!stdout.contains("OK  "));

    fs::remove_dir_all(temp_dir).expect("cleanup");
}

#[test]
fn doctor_rejects_directories_as_binary_or_helpers_and_files_as_model_dirs() {
    let temp_dir = unique_temp_dir("doctor-file-types");
    let repo = write_engine_fixture(&temp_dir, "qwen122b");
    let infer = repo.join("metal_infer/infer");
    fs::remove_file(&infer).expect("remove fixture infer");
    fs::create_dir(&infer).expect("infer directory");
    let run = repo.join("scripts/run_122b.sh");
    fs::remove_file(&run).expect("remove fixture run helper");
    fs::create_dir(&run).expect("run helper directory");
    let model = temp_dir.join("model");
    fs::remove_dir(&model).expect("remove empty model dir");
    fs::write(&model, "not a model directory").expect("model file");

    let output = command_in(&temp_dir)
        .args(["doctor", "qwen122b"])
        .output()
        .expect("doctor should launch");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("infer binary is missing or not a regular file:"));
    assert!(stdout.contains("model dir is missing or not a directory:"));
    assert!(stdout.contains(&format!(
        "engine helper script is missing or not a regular file: {}",
        run.canonicalize().expect("run helper path").display()
    )));
    assert!(!stdout.contains("OK  "));

    fs::remove_dir_all(temp_dir).expect("cleanup");
}

#[test]
fn missing_helpers_guide_demo_chat_and_bench_to_122b_runbook() {
    let temp_dir = unique_temp_dir("missing-helpers");
    let repo = write_engine_fixture(&temp_dir, "qwen122b");
    fs::remove_file(repo.join("scripts/run_122b.sh")).expect("remove run helper");
    fs::remove_file(repo.join("scripts/bench_122b.sh")).expect("remove bench helper");

    let demo = command_in(&temp_dir)
        .args(["demo", "--engine", "qwen122b", "fixture prompt"])
        .output()
        .expect("demo should launch");
    let bench = command_in(&temp_dir)
        .args(["bench", "qwen122b"])
        .output()
        .expect("bench should launch");
    let chat = output_with_input(
        command_in(&temp_dir).args(["chat", "--engine", "qwen122b"]),
        b"fixture prompt\n",
    );
    for (output, helper) in [
        (demo, "run_122b.sh"),
        (bench, "bench_122b.sh"),
        (chat, "run_122b.sh"),
    ] {
        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
        assert!(stderr.contains("engine helper script is missing or not a regular file:"));
        assert!(stderr.contains(helper));
        assert!(stderr.contains("docs/qwen122b-runbook.md"));
        assert!(stderr.contains("compatible 122B engine and model preparation"));
    }

    fs::remove_dir_all(temp_dir).expect("cleanup");
}

#[test]
fn missing_35b_helpers_do_not_show_122b_preparation_guidance() {
    let temp_dir = unique_temp_dir("missing-35b-helper");
    let repo = write_engine_fixture(&temp_dir, "qwen35b");
    fs::remove_file(repo.join("scripts/run_35b.sh")).expect("remove run helper");

    let output = command_in(&temp_dir)
        .args(["demo", "--engine", "qwen35b", "fixture prompt"])
        .output()
        .expect("demo should launch");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("run_35b.sh"));
    assert!(!stderr.contains("qwen122b-runbook.md"));

    fs::remove_dir_all(temp_dir).expect("cleanup");
}

fn command_in(cwd: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sebas"));
    command
        .current_dir(cwd)
        .env("SEBAS_WORKSPACE", cwd)
        .env_remove("SEBAS_RUNNER_TEST_MODEL_DIR");
    command
}

fn output_with_input(command: &mut Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sebas should launch");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input)
        .expect("write input");
    child.wait_with_output().expect("wait")
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

fn write_engine_fixture(root: &Path, engine: &str) -> PathBuf {
    write_minimal_manifest(root);
    let repo = root.join("custom engine");
    let infer = repo.join("metal_infer/infer");
    fs::create_dir_all(infer.parent().expect("infer parent")).expect("infer dir");
    fs::create_dir_all(repo.join("scripts")).expect("scripts dir");
    let model = root.join("model");
    fs::create_dir(&model).expect("empty model dir");

    write_test_script(
        &infer,
        "#!/bin/sh\nprintf 'infer must not run' >&2\nexit 99\n",
    );
    let helper = r#"#!/bin/sh
printf 'helper:%s\ncwd:%s\nmodel:%s\nextra:%s\nprompt:%s\ntokens:%s\nlang:%s\ncase:%s\nshort:%s\nlong:%s\n' "${0##*/}" "$(pwd -P)" "$1" "$2" "$PROMPT" "$TOKENS" "$BENCH_LANG" "$BENCH_CASE" "$SHORT_TOKENS" "$LONG_TOKENS"
"#;
    let size = engine.strip_prefix("qwen").expect("engine size");
    for action in ["run", "bench"] {
        write_test_script(&repo.join(format!("scripts/{action}_{size}.sh")), helper);
    }

    let manifest_path = root.join(".workspace/manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest"))
            .expect("valid fixture manifest");
    let engines = manifest["engines"].as_object_mut().expect("engines");
    let mut entry = engines.remove("qwen122b").expect("engine entry");
    entry["repoPath"] = serde_json::json!("custom engine");
    entry["inferBin"] = serde_json::json!("custom engine/metal_infer/infer");
    entry["modelDirEnv"] = serde_json::json!("SEBAS_RUNNER_TEST_MODEL_DIR");
    entry["defaultModelDir"] = serde_json::json!(model);
    engines.insert(engine.to_string(), entry);
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("fixture manifest");
    repo
}

fn write_test_script(path: &Path, contents: &str) {
    fs::write(path, contents).expect("fixture script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("executable script");
    }
}

fn write_minimal_manifest(root: &Path) {
    let workspace = root.join(".workspace");
    fs::create_dir_all(&workspace).expect("workspace dir");
    fs::write(
        workspace.join("manifest.json"),
        r#"{
  "workspaceVersion": 1,
  "engines": {
    "qwen122b": {
      "repoPath": "engine",
      "inferBin": "engine/metal_infer/infer",
      "modelDirEnv": "MODEL_DIR",
      "defaultModelDir": "~/Models/flash_moe_qwen3.5_122b_4bit",
      "portEnv": "QWEN122B_HTTP_PORT",
      "defaultPort": 61234,
      "prefillBatchEnv": "QWEN122B_PREFILL_BATCH",
      "defaultPrefillBatch": 32,
      "thinkBudgetEnv": "QWEN122B_THINK_BUDGET",
      "defaultThinkBudget": 0,
      "kvQuantEnv": "QWEN122B_KV_QUANT",
      "defaultKvQuant": "none",
      "healthPath": "/health",
      "pidFile": "qwen122b_http_server.pid",
      "stdoutLog": "qwen122b_http_server.stdout.log",
      "stderrLog": "qwen122b_http_server.stderr.log",
      "serveArgs": [],
      "validateArgs": []
    }
  }
}"#,
    )
    .expect("manifest");
}
