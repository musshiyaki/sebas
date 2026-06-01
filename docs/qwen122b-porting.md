# Qwen3.5-122B-A10B Porting Notes

## Goal

Sebas targets a text-only Apple Silicon inference path for
`mlx-community/Qwen3.5-122B-A10B-4bit`. The goal is not to make the whole model
resident in 16 GB of unified memory. The goal is to keep the active dense path
small and stream routed MoE expert weights from SSD on demand.

## Model Shape

The 122B path was derived from the model `config.json` and safetensors metadata
rather than hardcoded 397B assumptions.

| Field | 122B-A10B |
|---|---:|
| `hidden_size` | 3072 |
| `num_hidden_layers` | 48 |
| `num_experts` | 256 |
| `num_experts_per_tok` | 8 |
| `moe_intermediate_size` | 1024 |
| `shared_expert_intermediate_size` | 1024 |
| `num_attention_heads` | 32 |
| `num_key_value_heads` | 2 |
| `head_dim` | 256 |
| `full_attention_interval` | 4 |
| `vocab_size` | 248320 |

The model keeps the Qwen3.5 MoE family structure, including alternating linear
and full attention on a 4-layer cadence.

## What The Engine Preparation Does

- reads MLX 4-bit safetensors
- derives layer, expert, and quantization layout from local model metadata
- repacks routed experts into per-layer files
- exports runtime config for the C, Objective-C, and Metal inference path
- streams active expert blocks from SSD during generation
- keeps the 122B path text-only for the first public proof point

No GGUF overlay is required for the initial 122B bring-up path.

## Measured Runtime

Measured on 2026-03-29 on a `MacBook Air (Apple M5, 16 GB)` after full local
preparation:

| Case | Result |
|---|---:|
| Short Japanese smoke prompt, prefill | ~3.0 tok/s |
| Short Japanese smoke prompt, decode | ~3.2 tok/s |
| Longer Japanese benchmark, TTFT | ~16.4 s |
| Longer Japanese benchmark, prefill | ~2.9 tok/s |
| Longer Japanese benchmark, generation | ~3.1 tok/s |

Per-token timing trace on the same target:

| Component | Time |
|---|---:|
| Dense / attention | ~79.6 ms/token |
| `o_proj` + shared path | ~32.3 ms/token |
| routed expert I/O | ~194.6 ms/token |
| routed expert compute | ~1.5 ms/token |
| total | ~314.7 ms/token |

The current bottleneck is expert weight movement from SSD, not Metal math
throughput.

## Current Limits

- The 122B path is text-only.
- The runtime still targets Qwen3.5 MoE family assumptions.
- First-token latency is high compared with small resident local models.
- A malloc-backed expert cache can improve short-run throughput, but it is not
  stable enough for the default path.
- Reproducibility depends on Apple Silicon hardware, local SSD behavior, and the
  prepared model layout.

## Distribution Status

The public Sebas umbrella repository currently tracks the CLI, documentation,
and local orchestration surface. The Flash-MoE engine checkout is intentionally
kept out of the tracked umbrella tree while redistribution and upstream license
terms are clarified. See [Third-party notices](../THIRD_PARTY_NOTICES.md).
