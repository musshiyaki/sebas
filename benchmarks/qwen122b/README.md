# Qwen3.5-122B-A10B Benchmarks

This directory contains small, tracked benchmark summaries for the Sebas 122B
proof point. Full raw benchmark packs should be generated with
[`tools/collect-qwen122b-repro-pack`](../../tools/collect-qwen122b-repro-pack)
and attached to GitHub Releases when they are too large or too local-specific
for the repository.

## Published Packs

| Pack | Hardware | Model | Status |
|---|---|---|---|
| [`2026-06-01-m5-air-16gb`](2026-06-01-m5-air-16gb/) | MacBook Air, Apple M5, 16 GB | `mlx-community/Qwen3.5-122B-A10B-4bit` | tracked Japanese, English, and Chinese raw logs |
| [`2026-03-29-m5-air-16gb`](2026-03-29-m5-air-16gb/) | MacBook Air, Apple M5, 16 GB | `mlx-community/Qwen3.5-122B-A10B-4bit` | summary only; raw logs pending recapture |

## What Counts As Reproducible

A complete pack should include:

- environment manifest
- Sebas commit SHA
- engine commit SHA and dirty state
- prepared model footprint
- `doctor` output
- raw benchmark logs
- prompt and token counts
- TTFT, prefill tok/s, decode tok/s, and disk footprint

See [qwen122b-repro-pack.md](../../docs/qwen122b-repro-pack.md) for the pack
format and collection workflow.
