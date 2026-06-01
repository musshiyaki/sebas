# Qwen3.5-122B-A10B Runbook

This runbook documents the 122B proof point behind Sebas: running
`mlx-community/Qwen3.5-122B-A10B-4bit` on Apple Silicon by streaming routed MoE
expert weights from SSD.

## Current Public Repo Boundary

The tracked Sebas repository contains the CLI, docs, wrapper scripts, and
example local manifest. It does not currently vendor the Flash-MoE engine source
tree. Keep an engine checkout at `flash-moe-anemll-ios/` next to this README
when reproducing the 122B path locally.

That boundary is intentional until redistribution and upstream license terms
are fully clarified. See [Third-party notices](../THIRD_PARTY_NOTICES.md).

## Hardware And Disk

Measured target:

| Requirement | Value |
|---|---|
| Hardware | MacBook Air, Apple Silicon M5 |
| Unified memory | 16 GB |
| macOS | 26.4 during bring-up |
| Model source | `mlx-community/Qwen3.5-122B-A10B-4bit` |
| Recommended free disk | 160 GB or more |

The disk headroom covers source MLX assets, prepared Flash-MoE output, and
temporary files.

## Local Workspace Setup

Copy the example manifest into the local, ignored `.workspace/` directory:

```bash
mkdir -p .workspace
cp .workspace.example/manifest.json .workspace/manifest.json
cp .workspace.example/system-no-think.md .workspace/system-no-think.md
```

The default manifest expects:

```text
flash-moe-anemll-ios/metal_infer/infer
~/Models/flash_moe_qwen3.5_122b_4bit
```

You can override the prepared model path with:

```bash
export MODEL_DIR="$HOME/Models/flash_moe_qwen3.5_122b_4bit"
```

## Engine Preparation

With a compatible Flash-MoE engine checkout at `flash-moe-anemll-ios/`:

```bash
cd flash-moe-anemll-ios

./scripts/setup_122b.sh
source .venv/bin/activate

MODEL_DIR="$HOME/Models/mlx-community-Qwen3.5-122B-A10B-4bit"
OUT_DIR="$HOME/Models/flash_moe_qwen3.5_122b_4bit"

./scripts/prepare_122b.sh "$MODEL_DIR" "$OUT_DIR"
make -C metal_infer infer
```

## Doctor And Benchmark

From the Sebas workspace root:

```bash
./sebas engine doctor --engine qwen122b
./sebas engine bench --engine qwen122b
./sebas engine bench --engine qwen122b --lang all --case all --long-tokens 160
./sebas run engine-only --engine qwen122b
```

For direct engine execution, use the engine checkout scripts:

```bash
cd flash-moe-anemll-ios
./scripts/run_122b.sh "$HOME/Models/flash_moe_qwen3.5_122b_4bit"
./scripts/bench_122b.sh "$HOME/Models/flash_moe_qwen3.5_122b_4bit"
LONG_TOKENS=160 ./scripts/bench_122b.sh "$HOME/Models/flash_moe_qwen3.5_122b_4bit"
```

## Benchmark Artifacts To Publish

For reproducible public benchmarks, capture:

- commit SHA for Sebas and the engine checkout
- model source and prepared model path
- macOS version, chip, memory, and disk free space
- cold or warm cache status
- raw stdout and stderr logs
- prompt token count and generated token count
- TTFT, prefill tok/s, decode tok/s, disk footprint

Suggested table:

| Case | Prompt tokens | Generated tokens | TTFT | Prefill tok/s | Decode tok/s | Disk footprint | Notes |
|---|---:|---:|---:|---:|---:|---:|---|
| smoke-ja-short | 38 | 33 | 12.97s | 2.9 | 3.40 | 64G | 2026-06-01 run |
| benchmark-ja-long | 48 | 147 | 16.08s | 3.0 | 2.90 | 64G | `LONG_TOKENS=160` |
| smoke-en-short | 37 | 30 | 14.70s | 2.5 | 3.32 | 64G | English prompt |
| benchmark-en-long | 44 | 155 | 15.63s | 2.8 | 2.86 | 64G | `LONG_TOKENS=160` |
| smoke-zh-short | 38 | 34 | 12.82s | 3.0 | 3.33 | 64G | Chinese prompt |
| benchmark-zh-long | 41 | 97 | 17.62s | 2.3 | 3.07 | 64G | `LONG_TOKENS=160` |

You can collect the standard Sebas benchmark pack with:

```bash
tools/collect-qwen122b-repro-pack --case all
tools/collect-qwen122b-repro-pack --case all --lang all --long-tokens 160
```

See [qwen122b-repro-pack.md](qwen122b-repro-pack.md) for the pack format and
release workflow.

## Known Limits

- The 122B path is text-only.
- It is not a general Ollama replacement.
- It is not claiming the whole 122B model fits in RAM.
- It currently assumes Qwen3.5 MoE family tensor conventions.
- First-token latency and expert I/O are the main user-visible bottlenecks.
