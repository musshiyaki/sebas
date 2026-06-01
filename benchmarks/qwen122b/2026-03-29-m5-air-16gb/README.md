# Qwen3.5-122B-A10B On 16 GB MacBook Air

This is the tracked baseline summary for the first Sebas 122B proof point.

The measurements were recorded during the Qwen3.5-122B-A10B bring-up on
2026-03-29 and summarized in
[`docs/qwen122b-porting.md`](../../../docs/qwen122b-porting.md). The raw logs
from that run were not preserved in the public repository, so this pack is a
summary-only baseline. The next benchmark should be captured with
[`tools/collect-qwen122b-repro-pack`](../../../tools/collect-qwen122b-repro-pack).

## Summary

| Field | Value |
|---|---|
| Hardware | MacBook Air |
| Chip | Apple M5 |
| Unified memory | 16 GB |
| Date measured | 2026-03-29 |
| Model source | `mlx-community/Qwen3.5-122B-A10B-4bit` |
| Runtime style | SSD-streamed routed MoE experts |
| Path | text-only |
| GGUF overlay | not required for this bring-up |

## Metrics

| Case | Result |
|---|---:|
| Short Japanese smoke prompt, prefill | ~3.0 tok/s |
| Short Japanese smoke prompt, decode | ~3.2 tok/s |
| Longer Japanese benchmark, TTFT | ~16.4 s |
| Longer Japanese benchmark, prefill | ~2.9 tok/s |
| Longer Japanese benchmark, generation | ~3.1 tok/s |

Per-token timing trace:

| Component | Time |
|---|---:|
| Dense / attention | ~79.6 ms/token |
| `o_proj` + shared path | ~32.3 ms/token |
| routed expert I/O | ~194.6 ms/token |
| routed expert compute | ~1.5 ms/token |
| total | ~314.7 ms/token |

The bottleneck was expert weight movement from SSD, not Metal math throughput.

## Limitations

- Summary-only pack; raw stdout/stderr logs are not available in this repo.
- Prepared model footprint was not captured as a release artifact.
- The engine was an external checkout, and its exact dirty state should be
  recaptured in the next pack.
- Results depend on local SSD behavior and cache state.
