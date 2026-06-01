use std::io::{self, Write};

pub fn render_repl_help(
    slash_command_help: &str,
    primary_session_extension: &str,
) -> String {
    [
        "CODE AGENT".to_string(),
        "  /exit                Quit the session".to_string(),
        "  /quit                Quit the session".to_string(),
        "  Up/Down              Navigate input history".to_string(),
        "  Tab                  Complete commands, modes, and recent sessions".to_string(),
        "  Ctrl-C               Clear input (or exit on empty prompt)".to_string(),
        "  Shift+Enter/Ctrl+J   Insert a newline".to_string(),
        format!("  Auto-save            .codex/sessions/<session-id>.{primary_session_extension}"),
        "  Resume latest        /resume latest".to_string(),
        "  Browse sessions      /session list".to_string(),
        String::new(),
        slash_command_help.to_string(),
    ]
    .join("\n")
}

#[allow(clippy::too_many_arguments)]
pub fn print_help_to(
    out: &mut impl Write,
    primary_binary_name: &str,
    version: &str,
    primary_session_extension: &str,
    latest_session_reference: &str,
    slash_command_help: &str,
    resume_commands: &str,
) -> io::Result<()> {
    writeln!(out, "{primary_binary_name} v{version}")?;
    writeln!(out)?;
    writeln!(out, "Usage:")?;
    writeln!(out, "  {primary_binary_name} [PROMPT...]")?;
    writeln!(out, "      Start the code-first agent or run a one-shot coding task")?;
    writeln!(
        out,
        "  {primary_binary_name} engine <start|doctor|status|bench> --engine <qwen35b|qwen122b>"
    )?;
    writeln!(out, "      Manage local OpenAI-compatible inference engines")?;
    writeln!(
        out,
        "  {primary_binary_name} config [env|hooks|model|mcp|plugins|codex|import-qwen]"
    )?;
    writeln!(out, "      Inspect or migrate Codex configuration")?;
    writeln!(out, "  {primary_binary_name} search <query>")?;
    writeln!(out, "      Run built-in web search immediately")?;
    writeln!(out, "  {primary_binary_name} model set <qwen35b|qwen122b>")?;
    writeln!(out, "      Persist the project default model")?;
    writeln!(out, "  {primary_binary_name} session [list|latest]")?;
    writeln!(out, "      Inspect saved local sessions")?;
    writeln!(out, "  {primary_binary_name} mcp")?;
    writeln!(out, "      List configured MCP servers")?;
    writeln!(
        out,
        "  {primary_binary_name} --resume [SESSION.jsonl|session-id|latest] [/status] [/compact] [...]"
    )?;
    writeln!(
        out,
        "      Inspect or maintain a saved session without starting the interactive agent"
    )?;
    writeln!(out, "  {primary_binary_name} help")?;
    writeln!(out, "      Alias for --help")?;
    writeln!(out, "  {primary_binary_name} version")?;
    writeln!(out, "      Alias for --version")?;
    writeln!(out, "  {primary_binary_name} status")?;
    writeln!(out, "      Show the current local workspace status snapshot")?;
    writeln!(out, "  {primary_binary_name} sandbox")?;
    writeln!(out, "      Show the current sandbox isolation snapshot")?;
    writeln!(out, "  {primary_binary_name} dump-manifests")?;
    writeln!(out, "  {primary_binary_name} bootstrap-plan")?;
    writeln!(out, "  {primary_binary_name} agents")?;
    writeln!(out, "  {primary_binary_name} skills")?;
    writeln!(
        out,
        "  {primary_binary_name} system-prompt [--cwd PATH] [--date YYYY-MM-DD]"
    )?;
    writeln!(out, "  {primary_binary_name} login")?;
    writeln!(out, "  {primary_binary_name} logout")?;
    writeln!(out, "  {primary_binary_name} init")?;
    writeln!(out)?;
    writeln!(out, "Flags:")?;
    writeln!(out, "  --model MODEL              Override the active model")?;
    writeln!(
        out,
        "  --output-format FORMAT     Non-interactive output format: text or json"
    )?;
    writeln!(
        out,
        "  --permission-mode MODE     Set read-only, workspace-write, or danger-full-access"
    )?;
    writeln!(
        out,
        "  --dangerously-skip-permissions  Skip all permission checks"
    )?;
    writeln!(
        out,
        "  --allowedTools TOOLS       Restrict enabled tools (repeatable; comma-separated aliases supported)"
    )?;
    writeln!(
        out,
        "  --version, -V              Print version and build information locally"
    )?;
    writeln!(out)?;
    writeln!(out, "Interactive slash commands:")?;
    writeln!(out, "{slash_command_help}")?;
    writeln!(out)?;
    writeln!(out, "Resume-safe commands: {resume_commands}")?;
    writeln!(out)?;
    writeln!(out, "Session shortcuts:")?;
    writeln!(
        out,
        "  Code-agent turns auto-save to .codex/sessions/<session-id>.{primary_session_extension}"
    )?;
    writeln!(
        out,
        "  Use `{latest_session_reference}` with --resume, /resume, or /session switch to target the newest saved session"
    )?;
    writeln!(
        out,
        "  Use /session list in the interactive agent to browse managed sessions"
    )?;
    writeln!(out, "Examples:")?;
    writeln!(out, "  {primary_binary_name}")?;
    writeln!(out, "  {primary_binary_name} \"explain src/main.rs\"")?;
    writeln!(out, "  {primary_binary_name} engine doctor --engine qwen122b")?;
    writeln!(out, "  {primary_binary_name} search \"today's japan headlines\"")?;
    writeln!(out, "  {primary_binary_name} model set qwen122b")?;
    writeln!(out, "  {primary_binary_name} config import-qwen")?;
    writeln!(out, "  {primary_binary_name} --resume {latest_session_reference}")?;
    writeln!(
        out,
        "  {primary_binary_name} --resume {latest_session_reference} /status /diff /export notes.txt"
    )?;
    writeln!(out, "  {primary_binary_name} agents")?;
    writeln!(out, "  {primary_binary_name} /skills")?;
    writeln!(out, "  {primary_binary_name} login")?;
    writeln!(out, "  {primary_binary_name} init")?;
    Ok(())
}
