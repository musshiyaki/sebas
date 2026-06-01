mod sebas_engine;

use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::sebas_engine::{
    ensure_engine_ready, is_engine_running, load_runtime, print_engine_doctor, print_engine_status,
    run_bench, run_chat_turn, run_demo, BenchOptions, EngineKind, EngineRuntime,
};

const PRIMARY_BINARY_NAME: &str = "sebas";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ENGINE: EngineKind = EngineKind::Qwen122b;

struct DemoArgs {
    engine: EngineKind,
    prompt: String,
    tokens: String,
    passthrough: Vec<String>,
}

struct ChatArgs {
    engine: EngineKind,
    tokens: String,
    passthrough: Vec<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}\n\nRun `{PRIMARY_BINARY_NAME} --help` for usage.");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().map(String::as_str) else {
        if should_start_chat()? {
            handle_chat_command(&[])?;
        } else {
            println!("{}", render_launcher()?);
        }
        return Ok(());
    };

    match command {
        "-h" | "--help" | "help" => print_help(),
        "-V" | "--version" | "version" => print_version(),
        "engine" => handle_engine_command(&args[1..])?,
        "doctor" => handle_doctor_shortcut(&args[1..])?,
        "bench" => handle_bench_shortcut(&args[1..])?,
        "chat" => handle_chat_command(&args[1..])?,
        "demo" => handle_demo_command(&args[1..])?,
        "run" if args.get(1).map(String::as_str) == Some("engine-only") => {
            handle_engine_only_command(&args[2..])?;
        }
        "model" => println!("{}", handle_model_command(&args[1..])?),
        "config" => println!("{}", render_config_report()?),
        "init" => println!("{}", initialize_workspace()?),
        other => {
            return Err(io::Error::other(format!(
                "unknown command: {other}. Sebas now ships only local model runner commands."
            ))
            .into());
        }
    }

    Ok(())
}

fn handle_engine_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let Some(action) = args.first().map(String::as_str) else {
        return Err(io::Error::other(
            "usage: sebas engine <start|doctor|status|bench> --engine <qwen35b|qwen122b>",
        )
        .into());
    };

    let (sub_args, passthrough) = split_passthrough_args(&args[1..]);
    let (runtime, remaining) = resolve_engine_runtime(&sub_args, None)?;

    match action {
        "start" => {
            ensure_engine_ready(&runtime).map_err(io::Error::other)?;
            print_engine_status(&runtime);
        }
        "doctor" => {
            print_engine_doctor(&runtime).map_err(io::Error::other)?;
        }
        "status" => print_engine_status(&runtime),
        "bench" => {
            let options = parse_bench_options(&remaining)?;
            run_bench(&workspace_root()?, &runtime, &options, &passthrough)
                .map_err(io::Error::other)?;
        }
        other => {
            return Err(io::Error::other(format!("unknown engine command: {other}")).into());
        }
    }

    Ok(())
}

fn handle_engine_only_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let (runtime, remaining) = resolve_engine_runtime(args, None)?;
    if !remaining.is_empty() {
        return Err(io::Error::other(format!(
            "unexpected arguments for run engine-only: {}",
            remaining.join(" ")
        ))
        .into());
    }
    ensure_engine_ready(&runtime).map_err(io::Error::other)?;
    print_engine_status(&runtime);
    Ok(())
}

fn handle_doctor_shortcut(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let default_engine = first_engine_arg(args);
    let range = if default_engine.is_some() {
        &args[1..]
    } else {
        args
    };
    let (runtime, remaining) = resolve_engine_runtime(range, default_engine)?;
    if !remaining.is_empty() {
        return Err(io::Error::other(format!(
            "unexpected doctor arguments: {}",
            remaining.join(" ")
        ))
        .into());
    }
    print_engine_doctor(&runtime).map_err(io::Error::other)?;
    Ok(())
}

fn handle_bench_shortcut(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let default_engine = first_engine_arg(args);
    let range = if default_engine.is_some() {
        &args[1..]
    } else {
        args
    };
    let (sub_args, passthrough) = split_passthrough_args(range);
    let (runtime, remaining) = resolve_engine_runtime(&sub_args, default_engine)?;
    let options = parse_bench_options(&remaining)?;
    run_bench(&workspace_root()?, &runtime, &options, &passthrough).map_err(io::Error::other)?;
    Ok(())
}

fn handle_demo_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let demo = parse_demo_args(args)?;
    let runtime = load_runtime(&workspace_root()?, demo.engine).map_err(io::Error::other)?;
    run_demo(
        &workspace_root()?,
        &runtime,
        &demo.prompt,
        &demo.tokens,
        &demo.passthrough,
    )
    .map_err(io::Error::other)?;
    Ok(())
}

fn handle_chat_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let chat = parse_chat_args(args)?;
    let root = workspace_root()?;
    let runtime = load_runtime(&root, chat.engine).map_err(io::Error::other)?;

    println!("Sebas local chat");
    println!("Model: {}", runtime.model_id());
    println!("Runtime: SSD-streamed MoE experts");
    println!("Type /quit to exit.");
    println!();

    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        print!("sebas> ");
        io::stdout().flush()?;
        input.clear();
        let bytes = stdin.read_line(&mut input)?;
        if bytes == 0 {
            println!();
            break;
        }
        let prompt = input.trim();
        if prompt.is_empty() {
            continue;
        }
        if matches!(prompt, "/quit" | "/exit" | "quit" | "exit") {
            break;
        }

        run_chat_turn(&root, &runtime, prompt, &chat.tokens, &chat.passthrough)
            .map_err(io::Error::other)?;
        println!();
    }

    Ok(())
}

fn resolve_engine_runtime(
    args: &[String],
    default_engine: Option<EngineKind>,
) -> Result<(EngineRuntime, Vec<String>), Box<dyn std::error::Error>> {
    let (explicit_engine, stripped) = extract_engine_arg(args).map_err(io::Error::other)?;
    let mut engine = explicit_engine.or(default_engine);
    let mut remaining = stripped;

    if engine.is_none() {
        if let Some((positional, index)) = first_engine_positional(&remaining) {
            engine = Some(positional);
            remaining.remove(index);
        }
    }

    let engine = engine.unwrap_or(read_project_default_engine()?.unwrap_or(DEFAULT_ENGINE));
    let runtime = load_runtime(&workspace_root()?, engine).map_err(io::Error::other)?;
    Ok((runtime, remaining))
}

fn parse_demo_args(args: &[String]) -> Result<DemoArgs, Box<dyn std::error::Error>> {
    let (explicit_engine, stripped) = extract_engine_arg(args).map_err(io::Error::other)?;
    let mut engine = explicit_engine;
    let mut tokens = "128".to_string();
    let mut prompt_parts = Vec::new();
    let mut passthrough = Vec::new();
    let mut index = 0;

    while index < stripped.len() {
        match stripped[index].as_str() {
            "--tokens" => {
                tokens.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --tokens"))?,
                );
                validate_positive_integer(&tokens, "--tokens")?;
                index += 2;
            }
            flag if flag.starts_with("--tokens=") => {
                tokens = flag["--tokens=".len()..].to_string();
                validate_positive_integer(&tokens, "--tokens")?;
                index += 1;
            }
            "--" => {
                passthrough.extend(stripped[index + 1..].iter().cloned());
                break;
            }
            value if engine.is_none() => {
                if let Ok(parsed) = EngineKind::parse(value) {
                    engine = Some(parsed);
                } else {
                    prompt_parts.push(value.to_string());
                }
                index += 1;
            }
            value => {
                prompt_parts.push(value.to_string());
                index += 1;
            }
        }
    }

    let prompt = if prompt_parts.is_empty() {
        "In English, without showing your reasoning, introduce Sebas in two short sentences."
            .to_string()
    } else {
        prompt_parts.join(" ")
    };

    Ok(DemoArgs {
        engine: engine.unwrap_or(read_project_default_engine()?.unwrap_or(DEFAULT_ENGINE)),
        prompt,
        tokens,
        passthrough,
    })
}

fn parse_chat_args(args: &[String]) -> Result<ChatArgs, Box<dyn std::error::Error>> {
    let (explicit_engine, stripped) = extract_engine_arg(args).map_err(io::Error::other)?;
    let mut engine = explicit_engine;
    let mut tokens = "128".to_string();
    let mut passthrough = Vec::new();
    let mut index = 0;

    while index < stripped.len() {
        match stripped[index].as_str() {
            "--tokens" => {
                tokens.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --tokens"))?,
                );
                validate_positive_integer(&tokens, "--tokens")?;
                index += 2;
            }
            flag if flag.starts_with("--tokens=") => {
                tokens = flag["--tokens=".len()..].to_string();
                validate_positive_integer(&tokens, "--tokens")?;
                index += 1;
            }
            "--" => {
                passthrough.extend(stripped[index + 1..].iter().cloned());
                break;
            }
            value if engine.is_none() => {
                engine = Some(EngineKind::parse(value).map_err(io::Error::other)?);
                index += 1;
            }
            other => {
                return Err(io::Error::other(format!("unknown chat option: {other}")).into());
            }
        }
    }

    Ok(ChatArgs {
        engine: engine.unwrap_or(read_project_default_engine()?.unwrap_or(DEFAULT_ENGINE)),
        tokens,
        passthrough,
    })
}

fn parse_bench_options(args: &[String]) -> Result<BenchOptions, Box<dyn std::error::Error>> {
    let mut options = BenchOptions::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--case" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| io::Error::other("missing value for --case"))?;
                validate_bench_case(value)?;
                options.case = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--case=") => {
                let value = &flag["--case=".len()..];
                validate_bench_case(value)?;
                options.case = Some(value.to_string());
                index += 1;
            }
            "--lang" | "--language" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| io::Error::other("missing value for --lang"))?;
                validate_bench_lang(value)?;
                options.lang = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--lang=") => {
                let value = &flag["--lang=".len()..];
                validate_bench_lang(value)?;
                options.lang = Some(value.to_string());
                index += 1;
            }
            flag if flag.starts_with("--language=") => {
                let value = &flag["--language=".len()..];
                validate_bench_lang(value)?;
                options.lang = Some(value.to_string());
                index += 1;
            }
            "--short-tokens" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| io::Error::other("missing value for --short-tokens"))?;
                validate_positive_integer(value, "--short-tokens")?;
                options.short_tokens = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--short-tokens=") => {
                let value = &flag["--short-tokens=".len()..];
                validate_positive_integer(value, "--short-tokens")?;
                options.short_tokens = Some(value.to_string());
                index += 1;
            }
            "--long-tokens" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| io::Error::other("missing value for --long-tokens"))?;
                validate_positive_integer(value, "--long-tokens")?;
                options.long_tokens = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--long-tokens=") => {
                let value = &flag["--long-tokens=".len()..];
                validate_positive_integer(value, "--long-tokens")?;
                options.long_tokens = Some(value.to_string());
                index += 1;
            }
            other => {
                return Err(io::Error::other(format!(
                    "unknown bench option: {other}. Use --case short|long|all, --lang ja|en|zh|both|all, or --long-tokens N"
                ))
                .into());
            }
        }
    }

    Ok(options)
}

fn handle_model_command(args: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    match args {
        [] => {
            let engine = read_project_default_engine()?.unwrap_or(DEFAULT_ENGINE);
            Ok(format!(
                "Model\n  Default engine   {}\n  Settings         {}\n  Usage            sebas model set <qwen35b|qwen122b>",
                engine.as_cli_label(),
                project_settings_path()?.display()
            ))
        }
        [action, value] if action == "set" => set_project_default_engine(value),
        _ => Err(io::Error::other("usage: sebas model set <qwen35b|qwen122b>").into()),
    }
}

fn set_project_default_engine(value: &str) -> Result<String, Box<dyn std::error::Error>> {
    let engine = EngineKind::parse(value).map_err(io::Error::other)?;
    let settings_path = project_settings_path()?;
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut root = match fs::read_to_string(&settings_path) {
        Ok(contents) if !contents.trim().is_empty() => {
            serde_json::from_str::<Value>(&contents).unwrap_or_else(|_| json!({}))
        }
        _ => json!({}),
    };
    if !root.is_object() {
        root = json!({});
    }
    root["defaultEngine"] = json!(engine.as_cli_label());
    fs::write(&settings_path, serde_json::to_string_pretty(&root)?)?;

    Ok(format!(
        "Model updated\n  Default engine   {}\n  Settings         {}",
        engine.as_cli_label(),
        settings_path.display()
    ))
}

fn render_config_report() -> Result<String, Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let settings = project_settings_path()?;
    let manifest = root.join(".workspace").join("manifest.json");
    let default_engine = read_project_default_engine()?.unwrap_or(DEFAULT_ENGINE);
    let mut lines = vec![
        "Config".to_string(),
        format!("  Workspace        {}", root.display()),
        format!("  Manifest         {}", path_state(&manifest)),
        format!("  Settings         {}", path_state(&settings)),
        format!("  Default engine   {}", default_engine.as_cli_label()),
    ];

    if manifest.is_file() {
        for engine in [EngineKind::Qwen122b, EngineKind::Qwen35b] {
            match load_runtime(&root, engine) {
                Ok(runtime) => lines.push(format!(
                    "  {:<15} model={} port={}",
                    engine.as_cli_label(),
                    runtime.model_dir.display(),
                    runtime.port
                )),
                Err(error) => lines.push(format!("  {:<15} {error}", engine.as_cli_label())),
            }
        }
    }

    Ok(lines.join("\n"))
}

fn render_launcher() -> Result<String, Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let manifest = root.join(".workspace").join("manifest.json");
    let default_engine = read_project_default_engine()?.unwrap_or(DEFAULT_ENGINE);
    let mut lines = vec![
        format!("Sebas {VERSION}"),
        "Local 122B-class model runner for Apple Silicon".to_string(),
        String::new(),
        format!("Default engine   {}", default_engine.as_cli_label()),
        format!("Workspace        {}", root.display()),
    ];

    if manifest.is_file() {
        match load_runtime(&root, default_engine) {
            Ok(runtime) => {
                lines.push(format!("Model            {}", runtime.model_id()));
                lines.push(format!(
                    "Status           {}",
                    if is_engine_running(&runtime) {
                        "running"
                    } else {
                        "stopped"
                    }
                ));
                lines.push(format!("Model dir        {}", runtime.model_dir.display()));
                lines.push("Runtime          SSD-streamed MoE experts".to_string());
            }
            Err(error) => {
                lines.push(format!("Config           {error}"));
            }
        }
    } else {
        lines.push(format!("Manifest         {} (missing)", manifest.display()));
        lines
            .push("Setup            run from a Sebas workspace or set SEBAS_WORKSPACE".to_string());
    }

    lines.extend([
        String::new(),
        "Try:".to_string(),
        "  sebas chat".to_string(),
        "  sebas demo --tokens 96 \"Explain why running 122B locally on a 16GB MacBook Air is surprising.\"".to_string(),
        "  sebas bench qwen122b --lang all --case short".to_string(),
        "  sebas doctor qwen122b".to_string(),
        String::new(),
        "Use `sebas --help` for all commands.".to_string(),
    ]);

    Ok(lines.join("\n"))
}

fn should_start_chat() -> Result<bool, Box<dyn std::error::Error>> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(false);
    }
    Ok(workspace_root()?
        .join(".workspace")
        .join("manifest.json")
        .is_file())
}

fn initialize_workspace() -> Result<String, Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let workspace_dir = root.join(".workspace");
    fs::create_dir_all(&workspace_dir)?;

    let mut lines = vec![
        "Init".to_string(),
        format!("  Workspace        {}", root.display()),
    ];
    copy_example_if_missing(
        &root.join(".workspace.example").join("manifest.json"),
        &workspace_dir.join("manifest.json"),
        "manifest",
        &mut lines,
    )?;
    copy_example_if_missing(
        &root.join(".workspace.example").join("system-no-think.md"),
        &workspace_dir.join("system-no-think.md"),
        "system prompt",
        &mut lines,
    )?;

    let settings_path = project_settings_path()?;
    if settings_path.exists() {
        lines.push(format!(
            "  Settings         exists {}",
            settings_path.display()
        ));
    } else {
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            &settings_path,
            serde_json::to_string_pretty(
                &json!({ "defaultEngine": DEFAULT_ENGINE.as_cli_label() }),
            )?,
        )?;
        lines.push(format!(
            "  Settings         created {}",
            settings_path.display()
        ));
    }

    Ok(lines.join("\n"))
}

fn copy_example_if_missing(
    source: &Path,
    destination: &Path,
    label: &str,
    lines: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if destination.exists() {
        lines.push(format!("  {label:<15} exists {}", destination.display()));
        return Ok(());
    }
    fs::copy(source, destination).map_err(|error| {
        io::Error::other(format!(
            "failed to copy {} to {}: {error}",
            source.display(),
            destination.display()
        ))
    })?;
    lines.push(format!("  {label:<15} created {}", destination.display()));
    Ok(())
}

fn split_passthrough_args(args: &[String]) -> (Vec<String>, Vec<String>) {
    if let Some(index) = args.iter().position(|arg| arg == "--") {
        (args[..index].to_vec(), args[index + 1..].to_vec())
    } else {
        (args.to_vec(), Vec::new())
    }
}

fn extract_engine_arg(args: &[String]) -> Result<(Option<EngineKind>, Vec<String>), String> {
    let mut engine = None;
    let mut stripped = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--engine" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --engine".to_string())?;
                engine = Some(EngineKind::parse(value)?);
                index += 2;
            }
            flag if flag.starts_with("--engine=") => {
                engine = Some(EngineKind::parse(&flag["--engine=".len()..])?);
                index += 1;
            }
            other => {
                stripped.push(other.to_string());
                index += 1;
            }
        }
    }

    Ok((engine, stripped))
}

fn first_engine_arg(args: &[String]) -> Option<EngineKind> {
    args.first().and_then(|value| EngineKind::parse(value).ok())
}

fn first_engine_positional(args: &[String]) -> Option<(EngineKind, usize)> {
    args.iter()
        .enumerate()
        .find_map(|(index, value)| EngineKind::parse(value).ok().map(|engine| (engine, index)))
}

fn read_project_default_engine() -> Result<Option<EngineKind>, Box<dyn std::error::Error>> {
    let settings_path = project_settings_path()?;
    let Ok(contents) = fs::read_to_string(settings_path) else {
        return Ok(None);
    };
    let json: Value = serde_json::from_str(&contents)?;
    let Some(value) = json.get("defaultEngine").and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(EngineKind::parse(value).map_err(io::Error::other)?))
}

fn project_settings_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(workspace_root()?.join(".sebas").join("settings.json"))
}

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = env::var_os("SEBAS_WORKSPACE") {
        return Ok(PathBuf::from(path).canonicalize()?);
    }

    let cwd = env::current_dir()?.canonicalize()?;
    for candidate in cwd.ancestors() {
        if candidate.join(".workspace").is_dir() || candidate.join(".workspace.example").is_dir() {
            return Ok(candidate.to_path_buf());
        }
    }

    Ok(cwd)
}

fn validate_bench_case(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        "short" | "long" | "all" => Ok(()),
        _ => Err(io::Error::other("bench case must be one of: short, long, all").into()),
    }
}

fn validate_bench_lang(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        "ja" | "en" | "zh" | "both" | "all" => Ok(()),
        _ => Err(io::Error::other("bench language must be one of: ja, en, zh, both, all").into()),
    }
}

fn validate_positive_integer(value: &str, flag: &str) -> Result<(), Box<dyn std::error::Error>> {
    match value.parse::<u32>() {
        Ok(number) if number > 0 => Ok(()),
        _ => Err(io::Error::other(format!("{flag} must be a positive integer")).into()),
    }
}

fn path_state(path: &Path) -> String {
    if path.exists() {
        format!("{} (present)", path.display())
    } else {
        format!("{} (missing)", path.display())
    }
}

fn print_version() {
    println!("sebas {VERSION}");
}

fn print_help() {
    println!(
        "\
sebas {VERSION}

Usage:
  sebas
      Start local chat when run from a configured Sebas workspace
  sebas chat [--engine <qwen35b|qwen122b>] [--tokens N]
      Start an interactive local chat loop
  sebas engine <start|doctor|status|bench> [--engine <qwen35b|qwen122b>]
      Manage the local Flash-MoE inference engine
  sebas demo [--engine <qwen35b|qwen122b>] [--tokens N] [PROMPT...]
      Run a direct local-model demo prompt
  sebas run engine-only [--engine <qwen35b|qwen122b>]
      Start the local HTTP inference engine and print its status
  sebas doctor [qwen35b|qwen122b]
      Shortcut for engine doctor
  sebas bench [qwen35b|qwen122b] [--lang ja|en|zh|both|all] [--case short|long|all]
      Shortcut for engine bench
  sebas model [set <qwen35b|qwen122b>]
      Show or persist the project default engine
  sebas config
      Show local Sebas runner configuration
  sebas init
      Create local .workspace and .sebas files from examples
  sebas --help
  sebas --version

Examples:
  sebas
  sebas chat --tokens 128
  sebas init
  sebas engine doctor --engine qwen122b
  sebas engine bench --engine qwen122b --lang all --case all --long-tokens 160
  sebas demo --tokens 96 \"Explain why running 122B locally on a 16GB MacBook Air is surprising.\"
"
    );
}

#[cfg(test)]
mod tests {
    use super::{extract_engine_arg, split_passthrough_args, EngineKind};

    #[test]
    fn extracts_engine_flags() {
        let args = vec![
            "--engine".to_string(),
            "qwen122b".to_string(),
            "--case".to_string(),
            "short".to_string(),
        ];

        let (engine, rest) = extract_engine_arg(&args).expect("engine should parse");

        assert_eq!(engine, Some(EngineKind::Qwen122b));
        assert_eq!(rest, vec!["--case".to_string(), "short".to_string()]);
    }

    #[test]
    fn splits_passthrough_args() {
        let args = vec![
            "--lang".to_string(),
            "en".to_string(),
            "--".to_string(),
            "--raw-engine-flag".to_string(),
        ];

        let (own, passthrough) = split_passthrough_args(&args);

        assert_eq!(own, vec!["--lang".to_string(), "en".to_string()]);
        assert_eq!(passthrough, vec!["--raw-engine-flag".to_string()]);
    }
}
