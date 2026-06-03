mod sebas_engine;

use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

use crate::sebas_engine::{
    ensure_engine_ready, is_engine_running, load_runtime, print_engine_doctor, print_engine_status,
    run_bench, run_chat_turn, run_demo, BenchOptions, EngineKind, EngineRuntime,
};

const PRIMARY_BINARY_NAME: &str = "sebas";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ENGINE: EngineKind = EngineKind::Qwen122b;
const STAR_PROMPT_TEXT: &str =
    "If sebas worked for you, a GitHub star helps others discover the project:";
const STAR_PROMPT_URL: &str = "https://github.com/musshiyaki/sebas";

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

struct CodexProxyArgs {
    engine: EngineKind,
    listen: String,
    backend: Option<String>,
    no_start: bool,
    max_tokens: String,
    malformed_log: String,
    tool_mode: String,
    context_mode: String,
    agent_mode: String,
    session_mode: String,
}

struct CodexConfigArgs {
    engine: EngineKind,
    listen: String,
    max_tokens: String,
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
        "codex" => handle_codex_command(&args[1..])?,
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

fn handle_codex_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let Some(action) = args.first().map(String::as_str) else {
        return Err(io::Error::other(
            "usage: sebas codex <proxy|config|doctor> [--engine <qwen35b|qwen122b>]",
        )
        .into());
    };

    match action {
        "proxy" => handle_codex_proxy_command(&args[1..]),
        "config" | "profile" => handle_codex_config_command(&args[1..]),
        "doctor" => handle_codex_doctor_command(&args[1..]),
        other => Err(io::Error::other(format!("unknown codex command: {other}")).into()),
    }
}

fn handle_codex_proxy_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let proxy = parse_codex_proxy_args(args)?;
    let root = workspace_root()?;
    let runtime = load_runtime(&root, proxy.engine).map_err(io::Error::other)?;
    let backend = proxy
        .backend
        .clone()
        .unwrap_or_else(|| runtime.http_base_url());
    let script = root.join("tools").join("sebas-codex-proxy");
    if !script.is_file() {
        return Err(
            io::Error::other(format!("missing Codex bridge script: {}", script.display())).into(),
        );
    }

    if !proxy.no_start {
        ensure_engine_ready(&runtime).map_err(io::Error::other)?;
    }

    println!(
        "Codex bridge\n  Engine           {}\n  Backend          {}\n  Listen           http://{}/v1\n  Model            {}\n",
        runtime.engine.as_cli_label(),
        backend,
        proxy.listen,
        runtime.model_id()
    );
    println!(
        "  Tool mode        {}\n  Context mode     {}\n  Agent mode       {}\n  Session mode     {}\n  Max tokens       {}\n  Malformed log    {}",
        proxy.tool_mode,
        proxy.context_mode,
        proxy.agent_mode,
        proxy.session_mode,
        proxy.max_tokens,
        proxy.malformed_log
    );
    println!(
        "In another terminal, use:\n  codex -p {}",
        codex_profile_name(runtime.engine)
    );
    println!();

    let status = Command::new("python3")
        .arg(&script)
        .arg("--listen")
        .arg(&proxy.listen)
        .arg("--backend")
        .arg(&backend)
        .arg("--model")
        .arg(runtime.model_id())
        .arg("--max-tokens")
        .arg(&proxy.max_tokens)
        .arg("--malformed-log")
        .arg(&proxy.malformed_log)
        .arg("--tool-mode")
        .arg(&proxy.tool_mode)
        .arg("--context-mode")
        .arg(&proxy.context_mode)
        .arg("--agent-mode")
        .arg(&proxy.agent_mode)
        .arg("--session-mode")
        .arg(&proxy.session_mode)
        .current_dir(&root)
        .status()
        .map_err(|error| io::Error::other(format!("failed to start python3: {error}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("Codex bridge exited with {status}")).into())
    }
}

fn handle_codex_config_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_codex_config_args(args)?;
    let runtime = load_runtime(&workspace_root()?, config.engine).map_err(io::Error::other)?;
    println!(
        "{}",
        render_codex_config_snippet(&runtime, &config.listen, &config.max_tokens)
    );
    Ok(())
}

fn handle_codex_doctor_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_codex_config_args(args)?;
    let runtime = load_runtime(&workspace_root()?, config.engine).map_err(io::Error::other)?;
    let codex_version = Command::new("codex")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "not found on PATH".to_string());

    println!(
        "Codex bridge\n  Codex CLI        {}\n  Engine           {}\n  Engine status    {}\n  Backend URL      {}\n  Bridge URL       {}\n  Profile          {}\n  Config           sebas codex config --engine {}",
        codex_version,
        runtime.engine.as_cli_label(),
        if is_engine_running(&runtime) {
            "running"
        } else {
            "stopped"
        },
        runtime.http_base_url(),
        codex_base_url(&config.listen),
        codex_profile_name(runtime.engine),
        runtime.engine.as_cli_label()
    );
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
    let root = workspace_root()?;
    let runtime = load_runtime(&root, demo.engine).map_err(io::Error::other)?;
    run_demo(
        &root,
        &runtime,
        &demo.prompt,
        &demo.tokens,
        &demo.passthrough,
    )
    .map_err(io::Error::other)?;
    maybe_print_star_prompt(&root)?;
    Ok(())
}

fn handle_chat_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let chat = parse_chat_args(args)?;
    let root = workspace_root()?;
    let runtime = load_runtime(&root, chat.engine).map_err(io::Error::other)?;
    let mut star_prompt_pending = should_show_star_prompt(&root)?;

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
        if star_prompt_pending {
            if maybe_print_star_prompt(&root)? {
                println!();
            }
            star_prompt_pending = false;
        }
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

fn parse_codex_proxy_args(args: &[String]) -> Result<CodexProxyArgs, Box<dyn std::error::Error>> {
    let (explicit_engine, stripped) = extract_engine_arg(args).map_err(io::Error::other)?;
    let mut engine = explicit_engine;
    let mut listen = "127.0.0.1:61334".to_string();
    let mut backend = None;
    let mut no_start = false;
    let mut max_tokens: Option<String> = None;
    let mut malformed_log = "/tmp/sebas-codex-malformed.log".to_string();
    let mut tool_mode = "terminal".to_string();
    let mut context_mode = "compact".to_string();
    let mut agent_mode = "normal".to_string();
    let mut session_mode = "none".to_string();
    let mut index = 0;

    while index < stripped.len() {
        match stripped[index].as_str() {
            "--listen" => {
                listen.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --listen"))?,
                );
                index += 2;
            }
            flag if flag.starts_with("--listen=") => {
                listen = flag["--listen=".len()..].to_string();
                index += 1;
            }
            "--backend" => {
                backend = Some(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --backend"))?
                        .clone(),
                );
                index += 2;
            }
            flag if flag.starts_with("--backend=") => {
                backend = Some(flag["--backend=".len()..].to_string());
                index += 1;
            }
            "--max-tokens" => {
                let value = stripped
                    .get(index + 1)
                    .ok_or_else(|| io::Error::other("missing value for --max-tokens"))?;
                validate_positive_integer(value, "--max-tokens")?;
                max_tokens = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--max-tokens=") => {
                let value = flag["--max-tokens=".len()..].to_string();
                validate_positive_integer(&value, "--max-tokens")?;
                max_tokens = Some(value);
                index += 1;
            }
            "--malformed-log" => {
                malformed_log.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --malformed-log"))?,
                );
                index += 2;
            }
            flag if flag.starts_with("--malformed-log=") => {
                malformed_log = flag["--malformed-log=".len()..].to_string();
                index += 1;
            }
            "--no-start" => {
                no_start = true;
                index += 1;
            }
            "--tool-mode" => {
                tool_mode.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --tool-mode"))?,
                );
                validate_codex_tool_mode(&tool_mode)?;
                index += 2;
            }
            flag if flag.starts_with("--tool-mode=") => {
                tool_mode = flag["--tool-mode=".len()..].to_string();
                validate_codex_tool_mode(&tool_mode)?;
                index += 1;
            }
            "--context-mode" => {
                context_mode.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --context-mode"))?,
                );
                validate_codex_context_mode(&context_mode)?;
                index += 2;
            }
            flag if flag.starts_with("--context-mode=") => {
                context_mode = flag["--context-mode=".len()..].to_string();
                validate_codex_context_mode(&context_mode)?;
                index += 1;
            }
            "--agent-mode" => {
                agent_mode.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --agent-mode"))?,
                );
                validate_codex_agent_mode(&agent_mode)?;
                index += 2;
            }
            flag if flag.starts_with("--agent-mode=") => {
                agent_mode = flag["--agent-mode=".len()..].to_string();
                validate_codex_agent_mode(&agent_mode)?;
                index += 1;
            }
            "--one-shot-exec" => {
                agent_mode = "one-shot-exec".to_string();
                index += 1;
            }
            "--session-mode" => {
                session_mode.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --session-mode"))?,
                );
                validate_codex_session_mode(&session_mode)?;
                index += 2;
            }
            flag if flag.starts_with("--session-mode=") => {
                session_mode = flag["--session-mode=".len()..].to_string();
                validate_codex_session_mode(&session_mode)?;
                index += 1;
            }
            value if engine.is_none() => {
                engine = Some(EngineKind::parse(value).map_err(io::Error::other)?);
                index += 1;
            }
            other => {
                return Err(
                    io::Error::other(format!("unknown codex proxy option: {other}")).into(),
                );
            }
        }
    }

    Ok(CodexProxyArgs {
        engine: engine.unwrap_or(read_project_default_engine()?.unwrap_or(DEFAULT_ENGINE)),
        listen,
        backend,
        no_start,
        max_tokens: max_tokens.unwrap_or_else(|| {
            if agent_mode == "one-shot-exec" {
                "4096".to_string()
            } else {
                "128".to_string()
            }
        }),
        malformed_log,
        tool_mode,
        context_mode,
        agent_mode,
        session_mode,
    })
}

fn parse_codex_config_args(args: &[String]) -> Result<CodexConfigArgs, Box<dyn std::error::Error>> {
    let (explicit_engine, stripped) = extract_engine_arg(args).map_err(io::Error::other)?;
    let mut engine = explicit_engine;
    let mut listen = "127.0.0.1:61334".to_string();
    let mut max_tokens = "128".to_string();
    let mut index = 0;

    while index < stripped.len() {
        match stripped[index].as_str() {
            "--listen" => {
                listen.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --listen"))?,
                );
                index += 2;
            }
            flag if flag.starts_with("--listen=") => {
                listen = flag["--listen=".len()..].to_string();
                index += 1;
            }
            "--max-tokens" => {
                max_tokens.clone_from(
                    stripped
                        .get(index + 1)
                        .ok_or_else(|| io::Error::other("missing value for --max-tokens"))?,
                );
                validate_positive_integer(&max_tokens, "--max-tokens")?;
                index += 2;
            }
            flag if flag.starts_with("--max-tokens=") => {
                max_tokens = flag["--max-tokens=".len()..].to_string();
                validate_positive_integer(&max_tokens, "--max-tokens")?;
                index += 1;
            }
            value if engine.is_none() => {
                engine = Some(EngineKind::parse(value).map_err(io::Error::other)?);
                index += 1;
            }
            other => {
                return Err(
                    io::Error::other(format!("unknown codex config option: {other}")).into(),
                );
            }
        }
    }

    Ok(CodexConfigArgs {
        engine: engine.unwrap_or(read_project_default_engine()?.unwrap_or(DEFAULT_ENGINE)),
        listen,
        max_tokens,
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
    let mut root = read_settings_json(&settings_path)?;
    root["defaultEngine"] = json!(engine.as_cli_label());
    write_settings_json(&settings_path, &root)?;

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

fn render_codex_config_snippet(runtime: &EngineRuntime, listen: &str, max_tokens: &str) -> String {
    let provider = codex_provider_name(runtime.engine);
    let profile = codex_profile_name(runtime.engine);
    format!(
        "\
# Save this as ~/.codex/{profile}.config.toml after starting `sebas codex proxy`.
model = \"{}\"
model_provider = \"{provider}\"
sandbox_mode = \"workspace-write\"
approval_policy = \"never\"
model_context_window = 32768
model_max_output_tokens = {max_tokens}

[model_providers.{provider}]
name = \"Sebas {}\"
base_url = \"{}\"
wire_api = \"responses\"
request_max_retries = 0
stream_max_retries = 0
",
        runtime.model_id(),
        runtime.engine.as_cli_label(),
        codex_base_url(listen)
    )
}

fn codex_provider_name(engine: EngineKind) -> &'static str {
    match engine {
        EngineKind::Qwen35b => "sebas-qwen35b",
        EngineKind::Qwen122b => "sebas-qwen122b",
    }
}

fn codex_profile_name(engine: EngineKind) -> &'static str {
    codex_provider_name(engine)
}

fn codex_base_url(listen: &str) -> String {
    let normalized = if let Some(port) = listen.strip_prefix("0.0.0.0:") {
        format!("127.0.0.1:{port}")
    } else if let Some(port) = listen.strip_prefix("[::]:") {
        format!("127.0.0.1:{port}")
    } else {
        listen.to_string()
    };
    format!("http://{normalized}/v1")
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
        "  sebas codex config".to_string(),
        "  sebas codex proxy --engine qwen122b".to_string(),
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
        write_settings_json(
            &settings_path,
            &json!({ "defaultEngine": DEFAULT_ENGINE.as_cli_label() }),
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
    Ok(project_settings_path_for_root(&workspace_root()?))
}

fn project_settings_path_for_root(root: &Path) -> PathBuf {
    root.join(".sebas").join("settings.json")
}

fn read_settings_json(settings_path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut root = match fs::read_to_string(settings_path) {
        Ok(contents) if !contents.trim().is_empty() => {
            serde_json::from_str::<Value>(&contents).unwrap_or_else(|_| json!({}))
        }
        _ => json!({}),
    };
    if !root.is_object() {
        root = json!({});
    }
    Ok(root)
}

fn write_settings_json(
    settings_path: &Path,
    root: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(settings_path, serde_json::to_string_pretty(root)?)?;
    Ok(())
}

fn should_show_star_prompt(root: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let settings = read_settings_json(&project_settings_path_for_root(root))?;
    Ok(!settings
        .get("starPromptShown")
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

fn maybe_print_star_prompt(root: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if !io::stdout().is_terminal() {
        return Ok(false);
    }
    if !should_show_star_prompt(root)? {
        return Ok(false);
    }

    println!("{STAR_PROMPT_TEXT}");
    println!("{STAR_PROMPT_URL}");

    let settings_path = project_settings_path_for_root(root);
    let mut settings = read_settings_json(&settings_path)?;
    settings["starPromptShown"] = json!(true);
    write_settings_json(&settings_path, &settings)?;
    Ok(true)
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

fn validate_codex_tool_mode(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        "terminal" | "all" => Ok(()),
        _ => Err(io::Error::other("codex tool mode must be one of: terminal, all").into()),
    }
}

fn validate_codex_context_mode(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        "compact" | "full" => Ok(()),
        _ => Err(io::Error::other("codex context mode must be one of: compact, full").into()),
    }
}

fn validate_codex_agent_mode(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        "normal" | "one-shot-exec" => Ok(()),
        _ => Err(io::Error::other("codex agent mode must be one of: normal, one-shot-exec").into()),
    }
}

fn validate_codex_session_mode(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        "none" | "delta" => Ok(()),
        _ => Err(io::Error::other("codex session mode must be one of: none, delta").into()),
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
  sebas codex <proxy|config|doctor> [--engine <qwen35b|qwen122b>]
      Bridge Codex CLI to the local Sebas engine through the Responses API
      proxy supports --tool-mode terminal|all, --context-mode compact|full,
      --agent-mode normal|one-shot-exec, --session-mode none|delta,
      and --malformed-log PATH
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
  sebas codex config --engine qwen122b
  sebas codex proxy --engine qwen122b --one-shot-exec --session-mode delta
  sebas engine doctor --engine qwen122b
  sebas engine bench --engine qwen122b --lang all --case all --long-tokens 160
  sebas demo --tokens 96 \"Explain why running 122B locally on a 16GB MacBook Air is surprising.\"
"
    );
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;

    use super::{
        extract_engine_arg, parse_codex_proxy_args, project_settings_path_for_root,
        read_settings_json, should_show_star_prompt, split_passthrough_args, write_settings_json,
        EngineKind,
    };

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

    #[test]
    fn one_shot_codex_proxy_defaults_to_larger_output_budget() {
        let args = vec![
            "--engine".to_string(),
            "qwen122b".to_string(),
            "--one-shot-exec".to_string(),
        ];

        let parsed = parse_codex_proxy_args(&args).expect("proxy args should parse");

        assert_eq!(parsed.engine, EngineKind::Qwen122b);
        assert_eq!(parsed.agent_mode, "one-shot-exec");
        assert_eq!(parsed.session_mode, "none");
        assert_eq!(parsed.max_tokens, "4096");
        assert_eq!(parsed.malformed_log, "/tmp/sebas-codex-malformed.log");
    }

    #[test]
    fn explicit_codex_proxy_output_budget_wins() {
        let args = vec![
            "--engine".to_string(),
            "qwen122b".to_string(),
            "--agent-mode".to_string(),
            "one-shot-exec".to_string(),
            "--max-tokens".to_string(),
            "256".to_string(),
            "--malformed-log".to_string(),
            "/tmp/custom-sebas.log".to_string(),
        ];

        let parsed = parse_codex_proxy_args(&args).expect("proxy args should parse");

        assert_eq!(parsed.agent_mode, "one-shot-exec");
        assert_eq!(parsed.session_mode, "none");
        assert_eq!(parsed.max_tokens, "256");
        assert_eq!(parsed.malformed_log, "/tmp/custom-sebas.log");
    }

    #[test]
    fn codex_proxy_accepts_delta_session_mode() {
        let args = vec![
            "--engine".to_string(),
            "qwen122b".to_string(),
            "--session-mode".to_string(),
            "delta".to_string(),
        ];

        let parsed = parse_codex_proxy_args(&args).expect("proxy args should parse");

        assert_eq!(parsed.session_mode, "delta");
        assert_eq!(parsed.max_tokens, "128");
    }

    #[test]
    fn star_prompt_defaults_to_unshown() {
        let root = unique_temp_dir("star-prompt-default");
        fs::create_dir_all(&root).expect("temp root");

        assert!(should_show_star_prompt(&root).expect("star prompt should be readable"));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn write_settings_preserves_default_engine_when_marking_star_prompt() {
        let root = unique_temp_dir("star-prompt-persist");
        let settings_path = project_settings_path_for_root(&root);
        write_settings_json(&settings_path, &json!({ "defaultEngine": "qwen122b" }))
            .expect("settings should write");

        let mut settings = read_settings_json(&settings_path).expect("settings should read");
        settings["starPromptShown"] = json!(true);
        write_settings_json(&settings_path, &settings).expect("settings should update");

        let persisted = read_settings_json(&settings_path).expect("settings should re-read");
        assert_eq!(
            persisted
                .get("defaultEngine")
                .and_then(|value| value.as_str()),
            Some("qwen122b")
        );
        assert_eq!(
            persisted
                .get("starPromptShown")
                .and_then(|value| value.as_bool()),
            Some(true)
        );

        fs::remove_dir_all(root).expect("cleanup");
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        env::temp_dir().join(format!("sebas-cli-{label}-{unique}"))
    }
}
