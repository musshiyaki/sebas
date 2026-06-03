# Codex CLI Bridge

Sebas can expose the local Flash-MoE engine to Codex CLI through a small
Responses API bridge.

This does not vendor Codex CLI or modify Codex itself. Codex still owns the
agent loop, workspace access, terminal execution, and file editing tools. Sebas
only supplies a local model provider that translates Codex Responses requests to
the local Chat Completions engine.

## Start The Bridge

From a prepared Sebas workspace:

```bash
./sebas codex proxy --engine qwen122b
```

That command starts the qwen122b engine if needed, then listens at
`http://127.0.0.1:61334/v1`.

By default the bridge runs in compact mode and exposes only `exec_command`.
That keeps qwen122b prompt prefill practical on a 16 GB machine while still
allowing Codex to read files, edit files, and run shell commands through shell
commands. To expose every function tool Codex sends, use:

```bash
./sebas codex proxy --engine qwen122b --tool-mode all --context-mode full
```

For demos and one-off file edits, use one-shot exec mode:

```bash
./sebas codex proxy --engine qwen122b --one-shot-exec --session-mode delta
```

That mode biases the local model toward exactly one `exec_command` call for the
current Codex turn, then asks it to stop after the tool result. It avoids many
second and third Codex tool-loop prefills. The mode also uses a larger default
completion budget because the shell command can contain a complete heredoc.

`--session-mode delta` is experimental. With a Sebas engine that supports
`session_id`, `session_mode=append`, and `sebas_prompt`, the bridge sends only
new Codex input after the first turn and asks the engine to reuse its active KV
cache. Without that engine support, leave session mode at the default `none`.

While the local model is prefilling, the bridge sends the initial Responses API
stream event immediately and logs request size, exposed tools, and backend
elapsed time to the proxy terminal.

To print the Codex profile snippet:

```bash
./sebas codex config --engine qwen122b
```

Save the printed TOML as `~/.codex/sebas-qwen122b.config.toml`, then run Codex
with the profile:

```bash
codex -p sebas-qwen122b
```

For one-off validation without editing your normal Codex config, create a
temporary `CODEX_HOME` and put the printed profile file there.

## What Gets Bridged

Codex CLI sends:

- instructions and conversation input
- function tool schemas such as terminal execution
- tool results after Codex runs a tool locally

The bridge sends those to the Sebas engine as Chat Completions. If the local
model requests a tool, the bridge converts the result back into Responses API
`function_call` events so Codex CLI can execute the tool.

## Status

This bridge is experimental. It is intended for local research and demos:

- works with qwen122b and qwen35b engine profiles
- uses the existing local engine HTTP server
- supports Codex CLI tool calls through Responses API events
- keeps model calls non-streaming while preserving Codex-side streaming events
- defaults to compact `exec_command`-only tool exposure for qwen122b latency
- supports `--one-shot-exec` for single-tool demo/edit turns
- logs bridge request size and backend latency for long local-prefill turns
- can send delta append prompts to Sebas engines with experimental session-cache support

The quality of tool use depends on the local model following the tool protocol.
For reliable demos, ask the model explicitly to use the terminal or inspect a
file before answering.
