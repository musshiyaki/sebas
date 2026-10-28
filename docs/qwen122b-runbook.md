# Qwen3.5-122B-A10B Runbook

## Start Here: The Public Setup Is Incomplete

**A fresh Sebas checkout cannot currently reproduce the 122B benchmark end to
end.** Sebas publishes the CLI and benchmark evidence, but a verified, publicly
obtainable 122B engine and preparation scripts have not been identified. Resolve
that prerequisite before downloading roughly 70 GB of model assets.

The original local 122B scripts and supporting source have now been located in
the maintainer's private archive and inspected statically. They are not a public
release. Their redistribution terms, complete build and benchmark reproducibility
remain unverified.

The model source is
[`mlx-community/Qwen3.5-122B-A10B-4bit`](https://huggingface.co/mlx-community/Qwen3.5-122B-A10B-4bit).
It is already quantized to MLX 4-bit safetensors. The missing step is packing
those weights for the custom Flash-MoE engine, **not conversion to GGUF**. The
recorded Sebas 122B path does not require a GGUF overlay.

This corrects the missing prerequisites reported in [Issue #3](https://github.com/musshiyaki/sebas/issues/3).

## Engine Availability And Provenance

Checked on 2026-10-28:

| Source | What is available | 122B setup status |
|---|---|---|
| Sebas | CLI, example manifest, benchmark logs | No engine source or model preparation scripts included |
| [Anemll `m5-nax` at `26cd7f8`](https://github.com/Anemll/flash-moe/tree/26cd7f841823ea255ad6996d96692102852fa591) | Public 397B engine and conversion guide | No `scripts/setup_122b.sh` or `scripts/prepare_122b.sh` |
| [Anemll `iOS-App` at `9d1d602`](https://github.com/Anemll/flash-moe/tree/9d1d602bceca11b2e1b9fd6cf89057858cfbe6e4) | Public fork branch | Neither 122B script present |
| Anemll `main` at `20f8591`, `ASTC-experiments` at `5e8ba26` | Other public branches inspected | Neither 122B script present |
| Maintainer private archive | 122B setup/prepare/run/bench helpers, modified converters and runtime source recovered | Source located; no public distribution or clean-build validation |
| [2026-06-01 benchmark metadata](../benchmarks/qwen122b/2026-06-01-m5-air-16gb/environment.json) | Engine SHA `87ca06bb6babadf3673b85a28ce64ecb20894abe`, dirty tree | SHA alone does not contain the recorded local modifications; the SHA was not retrievable from the Anemll Git tree API during this check |

The public Anemll
[conversion guide](https://github.com/Anemll/flash-moe/blob/26cd7f841823ea255ad6996d96692102852fa591/docs/model-download-and-convert.md)
is for 397B. Its
[weight extractor](https://github.com/Anemll/flash-moe/blob/26cd7f841823ea255ad6996d96692102852fa591/metal_infer/extract_weights.py)
hardcodes hidden size 4096, 60 layers, 512 experts and top-k 10. The 122B
[model config](https://huggingface.co/mlx-community/Qwen3.5-122B-A10B-4bit/blob/e9c67b08899964be5fdd069bb1b4bc8907fe68f5/config.json)
requires 3072, 48, 256 and 8 respectively. Changing only the download name or
`expert_index.json` source path is not a verified 122B port.

Do not treat a clone of the current default branch, the `iOS-App` branch, or
pinning the benchmark SHA as a complete installation recipe. The previously
listed `setup_122b.sh`, `prepare_122b.sh`, `run_122b.sh` and `bench_122b.sh` are
names from the recovered local workflow, not scripts distributed by Sebas.

The archive contains a Git commit object whose hash matches `87ca06bb…`, but its
`infer.m` was modified on June 2, after the June 1 measurement. Recovering the
commit does not prove that the archived working files match the measured dirty
tree. Comparing recovered source with the public `iOS-App` revision confirms
changes to config loading, expert packing and tokenizer output, but is not a
complete attribution or historical reconstruction.

No explicit engine redistribution license was identified in the inspected
upstream/fork trees. Sebas's MIT license and the model's Apache-2.0 metadata do
not establish permission to redistribute that engine. See
[Third-party notices](../THIRD_PARTY_NOTICES.md).

## Obtain The Source Model (Download Only)

These commands explain where to obtain the source. **They do not complete
Sebas setup.** For a new installation, wait until a compatible engine and its
preparation instructions are available. If you already have an independently
verified compatible setup, keep its source revision and output directory intact.

Use the Hugging Face `hf` CLI, following its official
[installation and download guide](https://huggingface.co/docs/huggingface_hub/guides/cli).
An existing `hf download --help` must list `--revision`, `--local-dir` and
`--dry-run`. No model execution or GGUF tool is needed to download the source.

Choose a dedicated source directory and a different prepared-output directory:

```bash
MODEL_REPO="mlx-community/Qwen3.5-122B-A10B-4bit"
MODEL_REVISION="e9c67b08899964be5fdd069bb1b4bc8907fe68f5"
SOURCE_MODEL_DIR="$HOME/Models/mlx-community-Qwen3.5-122B-A10B-4bit"
PREPARED_MODEL_DIR="$HOME/Models/flash_moe_qwen3.5_122b_4bit"

hf download "$MODEL_REPO" --revision "$MODEL_REVISION" \
  --local-dir "$SOURCE_MODEL_DIR" --dry-run
```

This revision identifies the publicly inspected source snapshot; it was not
recorded in the old benchmark and is not a newly validated engine/model pair.
The dry run lists files and download size without fetching the weight payloads.
When the engine prerequisite is resolved and disk space is sufficient, download
the complete snapshot (no `--include` filter):

```bash
hf download "$MODEL_REPO" --revision "$MODEL_REVISION" \
  --local-dir "$SOURCE_MODEL_DIR"
```

At this revision, the source contains `config.json`,
`model.safetensors.index.json`, 14 `model-00001-of-00014.safetensors` through
`model-00014-of-00014.safetensors` shards, and tokenizer assets. Keep the index,
all shards and configuration/tokenizer files together. This is still the MLX
source directory; do not pass it as Sebas's `MODEL_DIR`.

The recovered `prepare_122b.sh` already calls `hf download` internally, without
`--revision`. If a reviewed compatible release retains that script interface,
a separately downloaded pinned source must be used with `SKIP_DOWNLOAD=1` to
avoid another unpinned download. This flag skips only the download: the archived
script still requires an executable `HF_BIN` and then performs all conversion
steps. It is not a dry run. The archive is not yet a supported installation
route.

The Hub lists about 69.6 GB of source files. The historical prepared output was
reported as `64G`. The former 160 GB free-space recommendation was planning
headroom for both copies and temporary files, not a verified minimum or a
promise for other converters. Check source size, output requirements, temporary
space and any separate download cache on the actual destination volumes. Do not
delete an existing source or prepared model to make room for an unverified port.

## Preparation: What Must Be Supplied

There is no verified public conversion command to give at this time. Static
inspection of the recovered wrapper shows this sequence: download the MLX source,
extract non-expert weights, repack experts with `--model` and `--write-index`, then
export vocabulary and tokenizer binaries. Its converters read 122B configuration
and shard metadata instead of relying only on fixed 397B dimensions.

The archived scripts still need preparation for release: dependency versions and
the download revision are unpinned; source/output path separation and existing
output protection are not enforced. The archived repacker's `--dry-run` still
writes `layout.json` (and an index if requested), while the extractor's dry run
can create an output directory. Do not use those modes as read-only checks in an
existing model directory. None of these scripts was executed during this review.

A complete release needs all of the following:

- An obtainable engine revision, with the local 122B changes included, and clear
  terms for the engine, fork contributions and conversion helpers.
- Preparation scripts that derive 122B dimensions and MLX quantization metadata
  from the model, export the runtime configuration/tokenizer, and pack routed
  experts into the layout expected by that exact engine.
- Documented dependency versions, input model revision, output layout and
  checksums or another reproducible output validation method.
- Matching `metal_infer/infer`, `scripts/run_122b.sh` and
  `scripts/bench_122b.sh` interfaces for the Sebas manifest and CLI.
- A clean-checkout preparation and inference result, with hardware, storage and
  raw benchmark evidence. A static path check is insufficient.

The practical options are to finish reviewing the recovered local port for
release after its permissions are resolved, or to implement and validate a new
122B port with suitable rights. Until then, the published benchmark is historical
evidence, not a complete reproduction package. Do not substitute the 397B or
optional GGUF hybrid pipeline.

## Connect An Already Prepared Compatible Installation

The following applies **only** when you already have a verified 122B engine and
its matching prepared output. Run from the Sebas repository root. Keep existing
local configuration; copy the examples only if the destination files are absent:

```bash
mkdir -p .workspace
cp -n .workspace.example/manifest.json .workspace/manifest.json
cp -n .workspace.example/system-no-think.md .workspace/system-no-think.md
```

Edit `.workspace/manifest.json` so the `qwen122b` entry's `repoPath` points to the
compatible engine checkout and `inferBin` points to its binary. Relative paths
are relative to the Sebas workspace root; absolute paths are also accepted. The
example expects `flash-moe-anemll-ios/` under that root. Chat/demo and benchmark
helpers are resolved under the configured `repoPath`.

Set `MODEL_DIR` to the **prepared output**, separately from the download path:

```bash
export MODEL_DIR="$HOME/Models/flash_moe_qwen3.5_122b_4bit"
./sebas engine doctor --engine qwen122b
```

Doctor checks local paths and required CLI helpers without running the engine.
A successful result does not validate weight layout, engine/model compatibility,
or inference. The recovered run/bench wrappers also source `scripts/lib_qwen_chat.sh` and
invoke `make` on every call; the archived Makefile marks `infer` as phony, so it
rebuilds the binary even when one exists. They require a complete source checkout
and toolchain, and are not read-only probes of a prebuilt binary. Once all these
prerequisites have been independently established, the existing commands are:

```bash
./sebas chat
./sebas engine bench --engine qwen122b --lang all --case all --long-tokens 160
./sebas run engine-only --engine qwen122b
```

## Benchmark Evidence And Limits

The [2026-06-01 pack](../benchmarks/qwen122b/2026-06-01-m5-air-16gb/)
records a MacBook Air M5 with 16 GB unified memory and macOS 26.5, with raw
Japanese, English and Chinese runs. Both the Sebas and engine trees were dirty
at measurement time. Preserve those flags; do not represent the SHAs alone as
a reproducible release.

For a future validated release, record model revision, engine and Sebas revisions,
all required reviewed source changes, build/preparation versions, prepared-layout
validation, hardware and free disk space, cold/warm cache state, prompts, token
counts, TTFT, prefill/decode rates and raw logs. Review rights and private data
before publishing any local changes. See [reproducibility pack notes](qwen122b-repro-pack.md).

The recorded path is text-only, streams active MoE expert weights from SSD, and
does not fit the whole model in RAM. No download, preparation or inference result
is claimed by this documentation correction.
