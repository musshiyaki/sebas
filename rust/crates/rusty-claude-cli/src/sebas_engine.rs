use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use serde_json::Value;

const HEALTH_POLL_ATTEMPTS: usize = 240;
const HEALTH_POLL_DELAY: Duration = Duration::from_millis(250);
const DEMO_SYSTEM_PROMPT: &str = "\
You are Sebas, a local AI demo running on this Mac with a 122B 4-bit MoE model. \
Reply in the user's language, stay concise, and finish within the token budget. \
Do not claim a maker unless asked.";
const BENCH_SHORT_PROMPT_EN: &str =
    "In English, without showing your reasoning, introduce yourself in two short sentences.";
const BENCH_LONG_PROMPT_EN: &str =
    "In English, explain Mount Fuji in six sentences. Do not show your reasoning. Do not use bullet points.";
const BENCH_SHORT_PROMPT_ZH: &str =
    "请用简体中文回答，不要展示推理过程，用两句简短的话介绍你自己。";
const BENCH_LONG_PROMPT_ZH: &str =
    "请用简体中文用六句话介绍富士山。不要展示推理过程，不要使用项目符号。";
const DEFAULT_BENCH_SHORT_TOKENS: &str = "48";
const DEFAULT_BENCH_LONG_TOKENS: &str = "160";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    Qwen35b,
    Qwen122b,
}

impl EngineKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "qwen35b" | "35b" | "local-35b" => Ok(Self::Qwen35b),
            "qwen122b" | "122b" | "local-122b" => Ok(Self::Qwen122b),
            other => Err(format!("unknown engine: {other}")),
        }
    }

    pub fn as_manifest_key(self) -> &'static str {
        match self {
            Self::Qwen35b => "qwen35b",
            Self::Qwen122b => "qwen122b",
        }
    }

    pub fn as_cli_label(self) -> &'static str {
        match self {
            Self::Qwen35b => "qwen35b",
            Self::Qwen122b => "qwen122b",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineRuntime {
    pub engine: EngineKind,
    pub repo_path: PathBuf,
    pub system_prompt_file: PathBuf,
    pub infer_bin: PathBuf,
    pub model_dir: PathBuf,
    pub port: u16,
    pub health_path: String,
    pub pid_file: PathBuf,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
    pub serve_args: Vec<String>,
    pub validate_args: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct BenchOptions {
    pub case: Option<String>,
    pub lang: Option<String>,
    pub short_tokens: Option<String>,
    pub long_tokens: Option<String>,
}

impl EngineRuntime {
    pub fn health_url(&self) -> String {
        format!("http://127.0.0.1:{}{}", self.port, self.health_path)
    }

    pub fn openai_base_url(&self) -> String {
        format!("http://127.0.0.1:{}/v1", self.port)
    }

    pub fn model_id(&self) -> String {
        infer_model_id_from_path(&self.model_dir)
    }

    pub fn export_env(&self) {
        env::set_var("OPENAI_API_KEY", env::var("OPENAI_API_KEY").unwrap_or_else(|_| "dummy".to_string()));
        env::set_var("OPENAI_BASE_URL", self.openai_base_url());
    }
}

pub fn load_runtime(root_dir: &Path, engine: EngineKind) -> Result<EngineRuntime, String> {
    let manifest_path = root_dir.join(".workspace").join("manifest.json");
    let manifest = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let json: Value =
        serde_json::from_str(&manifest).map_err(|error| format!("invalid manifest JSON: {error}"))?;

    let engines = json
        .get("engines")
        .and_then(Value::as_object)
        .ok_or_else(|| "manifest missing engines section".to_string())?;
    let entry = engines
        .get(engine.as_manifest_key())
        .and_then(Value::as_object)
        .ok_or_else(|| format!("manifest missing engine {}", engine.as_manifest_key()))?;

    let repo_path = root_dir.join(required_string(entry, "repoPath")?);
    let infer_bin = root_dir.join(required_string(entry, "inferBin")?);
    let model_dir_env = required_string(entry, "modelDirEnv")?;
    let model_dir = expand_home(
        env::var(&model_dir_env)
            .ok()
            .as_deref()
            .unwrap_or(required_string(entry, "defaultModelDir")?),
    );
    let port_env = required_string(entry, "portEnv")?;
    let port = env::var(&port_env)
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(required_u16(entry, "defaultPort")?);
    let prefill_batch_env = required_string(entry, "prefillBatchEnv")?;
    let prefill_batch = env::var(&prefill_batch_env)
        .unwrap_or_else(|_| required_u64(entry, "defaultPrefillBatch").unwrap_or(0).to_string());
    let think_budget_env = required_string(entry, "thinkBudgetEnv")?;
    let think_budget = env::var(&think_budget_env)
        .unwrap_or_else(|_| required_u64(entry, "defaultThinkBudget").unwrap_or(0).to_string());
    let kv_quant_env = required_string(entry, "kvQuantEnv")?;
    let kv_quant = env::var(&kv_quant_env)
        .unwrap_or_else(|_| {
            required_string(entry, "defaultKvQuant")
                .unwrap_or("none")
                .to_string()
        });
    let values = [
        ("MODEL_DIR", model_dir.display().to_string()),
        ("PORT", port.to_string()),
        ("PREFILL_BATCH", prefill_batch),
        ("THINK_BUDGET", think_budget),
        ("KV_QUANT", kv_quant.clone()),
    ];

    let health_path = required_string(entry, "healthPath")?.to_string();
    let tmp_dir = env::var_os("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    Ok(EngineRuntime {
        engine,
        repo_path,
        system_prompt_file: root_dir.join(".workspace").join("system-no-think.md"),
        infer_bin,
        model_dir,
        port,
        health_path,
        pid_file: tmp_dir.join(required_string(entry, "pidFile")?),
        stdout_log: tmp_dir.join(required_string(entry, "stdoutLog")?),
        stderr_log: tmp_dir.join(required_string(entry, "stderrLog")?),
        serve_args: interpolate_args(required_string_array(entry, "serveArgs")?, &values),
        validate_args: interpolate_args(required_string_array(entry, "validateArgs")?, &values),
    })
}

pub fn ensure_engine_ready(runtime: &EngineRuntime) -> Result<(), String> {
    if health_check(runtime.port, &runtime.health_path) {
        return Ok(());
    }

    kill_stale_pid_file(&runtime.pid_file);
    validate_runtime_paths(runtime)?;

    if !runtime.validate_args.is_empty() {
        let status = Command::new(&runtime.infer_bin)
            .args(&runtime.validate_args)
            .current_dir(&runtime.repo_path)
            .status()
            .map_err(|error| format!("failed to validate {}: {error}", runtime.engine.as_cli_label()))?;
        if !status.success() {
            return Err(format!(
                "engine validation failed for {}",
                runtime.engine.as_cli_label()
            ));
        }
    }

    if let Some(parent) = runtime.stdout_log.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Some(parent) = runtime.stderr_log.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let stdout = File::options()
        .create(true)
        .append(true)
        .open(&runtime.stdout_log)
        .map_err(|error| format!("failed to open {}: {error}", runtime.stdout_log.display()))?;
    let stderr = File::options()
        .create(true)
        .append(true)
        .open(&runtime.stderr_log)
        .map_err(|error| format!("failed to open {}: {error}", runtime.stderr_log.display()))?;

    let mut serve_args = vec![
        "--model".to_string(),
        runtime.model_dir.display().to_string(),
        "--serve".to_string(),
        runtime.port.to_string(),
    ];
    serve_args.extend(runtime.serve_args.clone());
    let kv_quant = env::var(match runtime.engine {
        EngineKind::Qwen35b => "QWEN35B_KV_QUANT",
        EngineKind::Qwen122b => "QWEN122B_KV_QUANT",
    })
    .unwrap_or_else(|_| "none".to_string());
    if kv_quant != "none" {
        serve_args.push("--kv-quant".to_string());
        serve_args.push(kv_quant);
    }

    let mut command = Command::new(&runtime.infer_bin);
    command
        .args(&serve_args)
        .current_dir(&runtime.repo_path)
        .env("FLASH_MOE_SYSTEM_PROMPT_FILE", &runtime.system_prompt_file)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    #[cfg(unix)]
    command.process_group(0);

    let child = command
        .spawn()
        .map_err(|error| format!("failed to start {}: {error}", runtime.engine.as_cli_label()))?;

    fs::write(&runtime.pid_file, child.id().to_string())
        .map_err(|error| format!("failed to write {}: {error}", runtime.pid_file.display()))?;

    for _ in 0..HEALTH_POLL_ATTEMPTS {
        if health_check(runtime.port, &runtime.health_path) {
            return Ok(());
        }
        thread::sleep(HEALTH_POLL_DELAY);
    }

    Err(format!(
        "failed to start {} on :{}\n{}",
        runtime.engine.as_cli_label(),
        runtime.port,
        tail_file(&runtime.stderr_log, 40)
    ))
}

pub fn print_engine_doctor(runtime: &EngineRuntime) -> Result<(), String> {
    let mut failures = Vec::new();
    if !runtime.infer_bin.exists() {
        failures.push(format!("missing infer binary: {}", runtime.infer_bin.display()));
    }
    if !runtime.model_dir.is_dir() {
        failures.push(format!("missing model dir: {}", runtime.model_dir.display()));
    }

    if failures.is_empty() {
        println!(
            "OK  {}: {} -> :{}",
            runtime.engine.as_cli_label(),
            runtime.model_dir.display(),
            runtime.port
        );
        return Ok(());
    }

    for failure in failures {
        println!("FAIL  {}: {}", runtime.engine.as_cli_label(), failure);
    }
    Err(format!("doctor failed for {}", runtime.engine.as_cli_label()))
}

pub fn print_engine_status(runtime: &EngineRuntime) {
    let running = health_check(runtime.port, &runtime.health_path);
    println!(
        "Engine\n  Name             {}\n  Status           {}\n  Model dir        {}\n  Base URL         {}\n  Model id         {}\n  PID file         {}\n  Stdout log       {}\n  Stderr log       {}",
        runtime.engine.as_cli_label(),
        if running { "running" } else { "stopped" },
        runtime.model_dir.display(),
        runtime.openai_base_url(),
        runtime.model_id(),
        runtime.pid_file.display(),
        runtime.stdout_log.display(),
        runtime.stderr_log.display()
    );
}

pub fn run_bench(
    root_dir: &Path,
    runtime: &EngineRuntime,
    options: &BenchOptions,
    passthrough: &[String],
) -> Result<(), String> {
    match options.lang.as_deref().unwrap_or("ja") {
        "ja" => run_bench_once(root_dir, runtime, options, "ja", passthrough),
        "en" => run_bench_once(root_dir, runtime, options, "en", passthrough),
        "zh" => run_bench_once(root_dir, runtime, options, "zh", passthrough),
        "both" => {
            run_bench_once(root_dir, runtime, options, "ja", passthrough)?;
            run_bench_once(root_dir, runtime, options, "en", passthrough)
        }
        "all" => {
            run_bench_once(root_dir, runtime, options, "ja", passthrough)?;
            run_bench_once(root_dir, runtime, options, "en", passthrough)?;
            run_bench_once(root_dir, runtime, options, "zh", passthrough)
        }
        other => Err(format!("unknown bench language: {other}")),
    }
}

fn run_bench_once(
    root_dir: &Path,
    runtime: &EngineRuntime,
    options: &BenchOptions,
    lang: &str,
    passthrough: &[String],
) -> Result<(), String> {
    let script = match runtime.engine {
        EngineKind::Qwen35b => root_dir.join("flash-moe-anemll-ios").join("scripts").join("bench_35b.sh"),
        EngineKind::Qwen122b => root_dir.join("flash-moe-anemll-ios").join("scripts").join("bench_122b.sh"),
    };
    println!("== language suite: {lang} ==");
    let infer_dir = runtime.infer_bin.parent().unwrap_or(root_dir);
    let mut command = Command::new(&script);
    command.arg(&runtime.model_dir).args(passthrough).current_dir(infer_dir);
    if let Some(case) = options.case.as_deref() {
        command.env("BENCH_CASE", case);
    }
    command.env(
        "SHORT_TOKENS",
        options
            .short_tokens
            .clone()
            .or_else(|| env::var("SHORT_TOKENS").ok())
            .unwrap_or_else(|| DEFAULT_BENCH_SHORT_TOKENS.to_string()),
    );
    command.env(
        "LONG_TOKENS",
        options
            .long_tokens
            .clone()
            .or_else(|| env::var("LONG_TOKENS").ok())
            .unwrap_or_else(|| DEFAULT_BENCH_LONG_TOKENS.to_string()),
    );
    command.env("BENCH_LANG", lang);
    if lang == "en" {
        command.env("SHORT_PROMPT", BENCH_SHORT_PROMPT_EN);
        command.env("LONG_PROMPT", BENCH_LONG_PROMPT_EN);
    } else if lang == "zh" {
        command.env("SHORT_PROMPT", BENCH_SHORT_PROMPT_ZH);
        command.env("LONG_PROMPT", BENCH_LONG_PROMPT_ZH);
    }
    let status = command
        .status()
        .map_err(|error| format!("failed to run {}: {error}", script.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("bench failed for {}", runtime.engine.as_cli_label()))
    }
}

pub fn run_demo(
    root_dir: &Path,
    runtime: &EngineRuntime,
    prompt: &str,
    tokens: &str,
    passthrough: &[String],
) -> Result<(), String> {
    let script = match runtime.engine {
        EngineKind::Qwen35b => root_dir.join("flash-moe-anemll-ios").join("scripts").join("run_35b.sh"),
        EngineKind::Qwen122b => root_dir.join("flash-moe-anemll-ios").join("scripts").join("run_122b.sh"),
    };
    let infer_dir = runtime.infer_bin.parent().unwrap_or(root_dir);
    let mut child = Command::new(&script)
        .arg(&runtime.model_dir)
        .args(passthrough)
        .current_dir(infer_dir)
        .env(
            "SYSTEM_PROMPT",
            env::var("SYSTEM_PROMPT").unwrap_or_else(|_| DEMO_SYSTEM_PROMPT.to_string()),
        )
        .env("PROMPT", prompt)
        .env("TOKENS", tokens)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to run {}: {error}", script.display()))?;
    let stderr = child.stderr.take().map(|stderr| {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if !should_suppress_demo_stderr(&line) {
                    eprintln!("{line}");
                }
            }
        })
    });
    let status = child
        .wait()
        .map_err(|error| format!("failed to wait for {}: {error}", script.display()))?;
    if let Some(stderr) = stderr {
        let _ = stderr.join();
    }
    if status.success() {
        Ok(())
    } else {
        Err(format!("demo failed for {}", runtime.engine.as_cli_label()))
    }
}

fn should_suppress_demo_stderr(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("[config] K auto-set")
        || trimmed.starts_with("[config] K override")
        || trimmed.starts_with("bpe_load:")
        || trimmed.starts_with("Tokens (")
}

fn required_string<'a>(object: &'a serde_json::Map<String, Value>, key: &str) -> Result<&'a str, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("manifest missing string key {key}"))
}

fn required_u16(object: &serde_json::Map<String, Value>, key: &str) -> Result<u16, String> {
    let value = object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("manifest missing integer key {key}"))?;
    u16::try_from(value).map_err(|_| format!("manifest key {key} is out of range"))
}

fn required_u64(object: &serde_json::Map<String, Value>, key: &str) -> Result<u64, String> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("manifest missing integer key {key}"))
}

fn required_string_array(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Vec<String>, String> {
    let Some(values) = object.get(key).and_then(Value::as_array) else {
        return Err(format!("manifest missing array key {key}"));
    };
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("manifest array {key} contains a non-string"))
        })
        .collect()
}

fn interpolate_args(args: Vec<String>, values: &[(&str, String)]) -> Vec<String> {
    args.into_iter()
        .map(|arg| {
            if let Some(name) = arg.strip_prefix('$') {
                values
                    .iter()
                    .find_map(|(key, value)| (*key == name).then_some(value.clone()))
                    .unwrap_or_default()
            } else {
                arg
            }
        })
        .collect()
}

fn expand_home(value: &str) -> PathBuf {
    if value == "~" {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(value));
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(rest))
            .unwrap_or_else(|| PathBuf::from(value));
    }
    PathBuf::from(value)
}

fn validate_runtime_paths(runtime: &EngineRuntime) -> Result<(), String> {
    let mut failures = Vec::new();
    if !runtime.infer_bin.exists() {
        failures.push(format!("missing infer binary: {}", runtime.infer_bin.display()));
    }
    if !runtime.model_dir.is_dir() {
        failures.push(format!("missing model dir: {}", runtime.model_dir.display()));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn kill_stale_pid_file(pid_file: &Path) {
    let Ok(pid) = fs::read_to_string(pid_file) else {
        return;
    };
    let pid = pid.trim();
    if pid.is_empty() {
        let _ = fs::remove_file(pid_file);
        return;
    }
    let _ = Command::new("kill").arg(pid).status();
    let _ = fs::remove_file(pid_file);
}

fn health_check(port: u16, path: &str) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(250)) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
    let request = format!("GET {path} HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n");
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }
    response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200")
}

fn tail_file(path: &Path, max_lines: usize) -> String {
    let Ok(contents) = fs::read_to_string(path) else {
        return String::new();
    };
    let mut lines = contents.lines().rev().take(max_lines).collect::<Vec<_>>();
    lines.reverse();
    lines.join("\n")
}

fn infer_model_id_from_path(model_dir: &Path) -> String {
    let mut name = model_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    for prefix in [
        "flash_moe_",
        "mlx-community-",
        "mlx-community_",
        "mlxcommunity-",
        "mlxcommunity_",
    ] {
        if let Some(rest) = name.strip_prefix(prefix) {
            name = rest.to_string();
            break;
        }
    }
    name.chars()
        .map(|ch| match ch {
            '_' | ' ' => '-',
            value => value.to_ascii_lowercase(),
        })
        .collect()
}
