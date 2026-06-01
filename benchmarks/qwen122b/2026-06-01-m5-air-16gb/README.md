# Qwen3.5-122B-A10B On 16 GB MacBook Air

This pack tracks a public-facing Sebas benchmark recapture with Japanese,
English, and Simplified Chinese prompts. It was measured on 2026-06-01 after
the long benchmark token budget was raised to avoid truncated answers.

## Summary

| Field | Value |
|---|---|
| Hardware | MacBook Air |
| Chip | Apple M5 |
| Unified memory | 16 GB |
| macOS | 26.5 |
| Date measured | 2026-06-01 |
| Model source | `mlx-community/Qwen3.5-122B-A10B-4bit` |
| Prepared model footprint | 64 GB |
| Runtime style | SSD-streamed routed MoE experts |
| Path | text-only |
| GGUF overlay | not required for this bring-up |

## Metrics

| Case | Prompt tokens | Generated tokens | TTFT | Prefill | Generation | Total |
|---|---:|---:|---:|---:|---:|---:|
| Japanese short smoke | 38 | 33 | 12.97 s | 2.9 tok/s | 3.40 tok/s | 22.4 s |
| Japanese long benchmark | 48 | 147 | 16.08 s | 3.0 tok/s | 2.90 tok/s | 66.4 s |
| English short smoke | 37 | 30 | 14.70 s | 2.5 tok/s | 3.32 tok/s | 23.4 s |
| English long benchmark | 44 | 155 | 15.63 s | 2.8 tok/s | 2.86 tok/s | 69.4 s |
| Chinese short smoke | 38 | 34 | 12.82 s | 3.0 tok/s | 3.33 tok/s | 22.7 s |
| Chinese long benchmark | 41 | 97 | 17.62 s | 2.3 tok/s | 3.07 tok/s | 48.9 s |

## Prompts

| Case | Prompt |
|---|---|
| Japanese short smoke | `日本語で、思考過程は出力せず、2文で自己紹介してください。` |
| Japanese long benchmark | `日本語で、富士山について6文で説明してください。思考過程は出力しないでください。箇条書きは禁止です。` |
| English short smoke | `In English, without showing your reasoning, introduce yourself in two short sentences.` |
| English long benchmark | `In English, explain Mount Fuji in six sentences. Do not show your reasoning. Do not use bullet points.` |
| Chinese short smoke | `请用简体中文回答，不要展示推理过程，用两句简短的话介绍你自己。` |
| Chinese long benchmark | `请用简体中文用六句话介绍富士山。不要展示推理过程，不要使用项目符号。` |

## Evidence

Raw stdout, stderr, and `/usr/bin/time -p` logs are tracked under [`raw/`](raw/).
The long runs used a 160-token generation budget; the Japanese, English, and
Chinese long answers completed without the visible truncation seen in the
earlier 96-token run.

The Sebas working tree was dirty at measurement time because the benchmark
language and token-budget CLI changes were being staged in the same patch. The
engine checkout was also a local external checkout with unrelated dirty files.
This pack is therefore a tracked launch evidence pack, not a clean-room
reproducibility artifact.
