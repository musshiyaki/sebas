# Sebas

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md)

[![Latest release](https://img.shields.io/github/v/release/musshiyaki/sebas?sort=semver)](https://github.com/musshiyaki/sebas/releases/latest)
[![Install](https://img.shields.io/badge/install-curl%20%7C%20sh-2ea44f)](#安装)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Sebas 可以在 16GB MacBook Air 上本地运行 Qwen3.5-122B-A10B。它不是把整个模型常驻进内存，而是从 SSD 按需流式读取 MoE expert weights。

Sebas 分为两层。核心是一套面向 Apple Silicon 的推理引擎，用于运行难以放进标准常驻式 runtime 的 Qwen3.5 MoE 模型。可选的 Rust CLI/agent 层则让这个本地引擎可以用于代码、搜索和实验工作流。

## 演示

[![Sebas running Qwen3.5-122B-A10B locally](docs/assets/sebas-demo-running.gif)](https://youtu.be/rn0uhik0bL0)

这是一段 cropped direct-camera demo，展示 Qwen3.5-122B-A10B 在 16GB MacBook Air 上本地运行。
[在 YouTube 上观看完整演示视频](https://youtu.be/rn0uhik0bL0)。

## 安装

安装最新的 prebuilt Sebas CLI：

```bash
curl -fsSL https://raw.githubusercontent.com/musshiyaki/sebas/main/install.sh | sh
sebas --help
```

安装器默认会把 `sebas` 复制到 `~/.local/bin`。release tag、自定义安装目录、PATH 设置和源码构建 fallback 请参考 [install.md](docs/install.md)。

这个命令只安装 CLI。122B path 仍然需要 prepared model 和外部 engine checkout，具体见 [qwen122b-runbook.md](docs/qwen122b-runbook.md)。

## 当前验证结果

![Sebas social preview](docs/assets/social-preview.png)

以下数据在 `MacBook Air (Apple M5, 16 GB)` 上测得，模型为完整本地准备后的 `mlx-community/Qwen3.5-122B-A10B-4bit`。最新 tracked run 位于 [`benchmarks/qwen122b/2026-06-01-m5-air-16gb`](benchmarks/qwen122b/2026-06-01-m5-air-16gb/)。

| Case | TTFT | Generation | Total |
|---|---:|---:|---:|
| Japanese short smoke | 12.97 s | 3.40 tok/s | 22.4 s |
| Japanese long benchmark | 16.08 s | 2.90 tok/s | 66.4 s |
| English short smoke | 14.70 s | 3.32 tok/s | 23.4 s |
| English long benchmark | 15.63 s | 2.86 tok/s | 69.4 s |
| Chinese short smoke | 12.82 s | 3.33 tok/s | 22.7 s |
| Chinese long benchmark | 17.62 s | 3.07 tok/s | 48.9 s |

当前瓶颈是从 SSD 移动 expert weights，而不是 Metal 计算吞吐。计时拆解和架构说明见 [qwen122b-porting.md](docs/qwen122b-porting.md)。

最早的 summary-only baseline 在 [`benchmarks/qwen122b/2026-03-29-m5-air-16gb`](benchmarks/qwen122b/2026-03-29-m5-air-16gb/)。如需采集新的 raw logs、环境元数据和 doctor output，请使用 [`tools/collect-qwen122b-repro-pack`](tools/collect-qwen122b-repro-pack)。

## 为什么能跑

Qwen3.5-122B-A10B 是 Mixture-of-Experts 模型。每个 token 只会激活一部分 routed experts。Sebas 会把模型准备成适合小机器运行的布局：dense weights 保持在可处理范围内，routed expert files 则从 SSD 按需读取。

这并不是说“整个 122B 模型可以放进 16GB RAM”。Sebas 是一个本地 streaming runtime：

- active path 使用 Apple Silicon Metal 计算
- routed experts 由 SSD-backed streaming 提供
- 针对 Qwen3.5 MoE shape 的 export 和 runtime config
- 面向 122B bring-up 的稳定 text-only inference path

## 为什么不是 Ollama

Sebas 并不是要替代 Ollama 成为通用本地模型运行器。对于 backend 支持且能装入常规布局的模型，Ollama 非常好用。但这个 122B bring-up 不是标准的“加载 GGUF 然后运行”路径。

第一个可工作的 122B path 从 MLX 4-bit safetensors 开始，不需要 GGUF overlay。推理之前需要 Qwen3.5 MoE-aware 的准备流程：

- 从 `config.json` 和 safetensors shapes 推导 architecture
- 把 routed experts repack 成按 layer 存放的文件
- 使用 `pread` 从 SSD 流式读取 active expert blocks
- 用匹配该布局的 C、Objective-C 和 Metal kernels 驱动推理

这个 custom engine 是 Sebas 存在的原因。CLI/agent runtime 是围绕引擎的 convenience layer，而不是项目的前提。

## 源码 checkout

公开入口是仓库顶层的 `./sebas` 命令。

```bash
./sebas --help
```

也可以从源码构建：

```bash
git clone https://github.com/musshiyaki/sebas.git
cd sebas
tools/install-sebas
sebas --help
```

## 本地 engine setup

Sebas 不会 vendor 或重新分发 Flash-MoE engine code。如果你选择使用兼容的外部
engine checkout，请先自行检查 upstream repository 及其 license status。

```bash
git clone https://github.com/Anemll/flash-moe flash-moe-anemll-ios
```

使用 local engine commands 前，先创建 local workspace manifest：

```bash
mkdir -p .workspace
cp .workspace.example/manifest.json .workspace/manifest.json
cp .workspace.example/system-no-think.md .workspace/system-no-think.md

./sebas engine doctor --engine qwen122b
./sebas engine bench --engine qwen122b
./sebas engine bench --engine qwen122b --lang all --case all --long-tokens 160
./sebas run engine-only --engine qwen122b
```

从 source MLX model 完整准备 122B path 的步骤见 [qwen122b-runbook.md](docs/qwen122b-runbook.md)。当前 public umbrella repo 跟踪 Sebas CLI 和文档。Flash-MoE engine checkout 暂时放在 tracked tree 外，直到再发布和 upstream license terms 完全明确。

## 仓库内容

| Path | Purpose |
|---|---|
| `sebas` | engine commands 和 optional agent workflow 的 main CLI entrypoint |
| `rust/` | optional Sebas agent runtime、TUI、tool execution、config、sessions |
| `.workspace.example/` | local engine manifest 示例；复制到 `.workspace/` 后使用 |
| `docs/qwen122b-runbook.md` | public 122B setup and benchmark runbook |
| `docs/qwen122b-porting.md` | public 122B architecture and measurement notes |
| `tools/` | thin operational wrappers |
| `docs/` | workspace architecture notes |
| `engines/` | external engine ownership and layout notes |

local build 可能仍包含 legacy compatibility alias，但公开支持的入口是 `sebas`。

## 当前状态

这是研究级 local runtime，还不是 polished consumer app。

目前可用：

- Qwen3.5-122B-A10B text-only inference path
- MacBook Air 16GB bring-up，以及实测 prefill/decode 数据
- 用于 local engine operation 的 `./sebas` CLI wrapper
- optional Rust code-first agent runtime and tool surface
- Qwen35B 和 Qwen122B engine selection paths
- benchmark and doctor commands

仍然 experimental：

- long-context latency
- fast mode / malloc-backed expert cache stability
- Qwen3.5 shape family 之外的 arbitrary MoE model support
- vision tensors
- prebuilt releases and package-manager installers

## Optional Agent Runtime

Sebas 还包含一个用 Rust 实现的 code-first AI coding runtime。这个层对 122B proof point 来说是 optional。它的目标是让本地 Qwen engine 不只是 inference demo，而是可以进入 developer workflow。

```bash
cd rust
cargo build --release

./target/release/sebas
./target/release/sebas "explain the current diff"
```

runtime 细节见 [rust/README.md](rust/README.md)。

## 背景

这项推理工作基于 Flash-MoE，以及一个核心想法：如果 expert weights 可以按需流式读取，非常大的 MoE 模型也可以在小型本地机器上运行。Anemll fork 把这个方向扩展到了 Apple Silicon 和 122B Qwen3.5 path。

相关文档：

- [Installing Sebas](docs/install.md)
- [Qwen3.5-122B porting notes](docs/qwen122b-porting.md)
- [Qwen3.5-122B runbook](docs/qwen122b-runbook.md)
- [Reproducibility pack workflow](docs/qwen122b-repro-pack.md)
- [Tracked benchmark summaries](benchmarks/qwen122b/)
- [Workspace architecture](docs/WORKSPACE_ARCHITECTURE.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)

## 限制

- 122B path 目前是 text-only。
- first-token experience 仍然比小型本地模型慢。
- model preparation 很大，也比较技术化。
- runtime 目前针对 Qwen3.5 MoE family assumptions。
- 可复现性依赖 Apple Silicon hardware、local SSD behavior 和 prepared model layout。

这些限制之所以明确写出来，是因为这个项目有趣的地方不是假装巨大模型“神奇地变轻了”，而是在小硬件上让巨大 local model 真实跑起来的工程约束。

## 关系说明

Sebas 是一个独立 research project，并不隶属于、获得背书于或由 OpenAI、Anthropic、Apple、Alibaba/Qwen、ANEMLL、Hugging Face 或 MLX Community 维护。第三方名称仅用于说明 compatibility、attribution 和 benchmark context。

## License

见 [LICENSE](LICENSE) 和 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
