# Qwen3.5-122B-A10B Reproducibility Pack

The goal of a Sebas reproducibility pack is to make benchmark claims auditable
without shipping model weights. A useful pack should contain the exact hardware
and software context, raw command output, and benchmark logs for the 122B path.

## Generate A Pack

From the Sebas workspace root:

```bash
tools/collect-qwen122b-repro-pack --case all
tools/collect-qwen122b-repro-pack --case all --lang all --long-tokens 160
```

Common variants:

```bash
tools/collect-qwen122b-repro-pack --case short
tools/collect-qwen122b-repro-pack --skip-bench
tools/collect-qwen122b-repro-pack \
  --engine-dir ./flash-moe-anemll-ios \
  --model-dir "$HOME/Models/flash_moe_qwen3.5_122b_4bit"
```

Generated packs are written under `benchmarks/qwen122b/generated/`, which is
ignored by Git. Promote only curated, small artifacts into the tracked
`benchmarks/qwen122b/` tree or upload a full pack as a GitHub Release asset.

## Expected Contents

| File | Purpose |
|---|---|
| `README.md` | pack summary and commit references |
| `environment.txt` | raw machine, OS, memory, and disk information |
| `environment.json` | machine-readable environment manifest when Python is available |
| `git-state.txt` | Sebas and engine commits plus dirty state |
| `model-layout.txt` | prepared model directory presence, size, and top-level files |
| `logs/sebas-help.*` | CLI help command, stdout, stderr, and exit code |
| `logs/sebas-engine-doctor.*` | doctor command, stdout, stderr, and exit code |
| `raw/bench_122b.log` | raw benchmark output from the engine script |
| `raw/bench_122b.exit-code.txt` | benchmark exit code |

The pack must not include model weights, safetensors shards, packed expert
files, tokenizer assets, or private local paths beyond what is necessary for
reproducibility.

## External Engine Checkout

The current 122B engine is intentionally kept as an external checkout instead
of vendored source in this repository. That is the main reproducibility risk.

Every public benchmark pack should therefore capture:

- the engine checkout path
- the engine commit SHA
- the engine branch and dirty state from `git status --short --branch`
- the Sebas wrapper commit SHA
- the exact command output from `sebas engine doctor --engine qwen122b`

If the engine checkout has local changes, keep them in the pack rather than
editing the evidence away. For launch-quality results, also attach the generated
pack to a GitHub Release so reviewers can inspect the raw logs without cloning
private local state.

## Publish A Pack

For a launch or release, create a compressed artifact:

```bash
PACK_DIR="benchmarks/qwen122b/generated/<pack-id>"
tar -czf "sebas-qwen122b-repro-pack-<pack-id>.tar.gz" -C "$PACK_DIR" .
```

Attach that archive to a GitHub Release with:

```bash
gh release create qwen122b-repro-<date> \
  "sebas-qwen122b-repro-pack-<pack-id>.tar.gz" \
  --title "Qwen3.5-122B-A10B reproducibility pack" \
  --notes-file "$PACK_DIR/README.md"
```

## Current Published Summary

The tracked baseline summary is
[`benchmarks/qwen122b/2026-03-29-m5-air-16gb`](../benchmarks/qwen122b/2026-03-29-m5-air-16gb/).
It records the March 29, 2026 bring-up measurements from the porting notes.
Raw logs from that run were not preserved in the public repository, so the next
real benchmark run should be captured with `tools/collect-qwen122b-repro-pack`.

## Benchmark Protocol

Use this minimum protocol when producing public numbers:

- record cold or warm cache status
- keep the prompt text in the raw log
- report prompt tokens and generated tokens when available
- report TTFT, prefill tok/s, decode tok/s, and disk footprint
- include Sebas and engine commit SHAs
- include whether the engine checkout had uncommitted changes
- include the exact model source and prepared model path
- mention any failed cases rather than deleting them from the pack
