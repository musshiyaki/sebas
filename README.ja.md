# Sebas

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md)

[![Latest release](https://img.shields.io/github/v/release/musshiyaki/sebas?sort=semver)](https://github.com/musshiyaki/sebas/releases/latest)
[![Install](https://img.shields.io/badge/install-curl%20%7C%20sh-2ea44f)](#インストール)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Qwen3.5-122B-A10B を 16GB MacBook Air 上でローカル実行します。モデル全体をメモリに常駐させるのではなく、MoE の expert weight を SSD からオンデマンドにストリーミングします。

Sebas は、標準的な常駐型ランタイムには収まりにくい Qwen3.5 MoE モデルを Apple Silicon で動かすためのローカル推論 path に集中しています。Rust CLI は local chat、demo、doctor、benchmark のための薄い operational wrapper です。

## デモ

[![Sebas running Qwen3.5-122B-A10B locally](docs/assets/sebas-demo-running.gif)](https://youtu.be/rn0uhik0bL0)

Qwen3.5-122B-A10B を 16GB MacBook Air 上で動かしている、cropped direct-camera demo です。
[YouTube でデモ動画全体を見る](https://youtu.be/rn0uhik0bL0)。

## インストール

最新の prebuilt Sebas CLI をインストールします。

```bash
curl -fsSL https://raw.githubusercontent.com/musshiyaki/sebas/main/install.sh | sh
sebas
```

デフォルトでは `sebas` は `~/.local/bin` にコピーされます。release tag、インストール先の変更、PATH 設定、ソースビルド fallback については [install.md](docs/install.md) を参照してください。

この手順で入るのは CLI のみです。122B path には、[qwen122b-runbook.md](docs/qwen122b-runbook.md) に書かれている prepared model と外部 engine checkout が必要です。

## 現在の実証結果

![Sebas social preview](docs/assets/social-preview.png)

`MacBook Air (Apple M5, 16 GB)` 上で、`mlx-community/Qwen3.5-122B-A10B-4bit` を完全にローカル準備した状態で測定しました。最新の tracked run は [`benchmarks/qwen122b/2026-06-01-m5-air-16gb`](benchmarks/qwen122b/2026-06-01-m5-air-16gb/) です。

| Case | TTFT | Generation | Total |
|---|---:|---:|---:|
| Japanese short smoke | 12.97 s | 3.40 tok/s | 22.4 s |
| Japanese long benchmark | 16.08 s | 2.90 tok/s | 66.4 s |
| English short smoke | 14.70 s | 3.32 tok/s | 23.4 s |
| English long benchmark | 15.63 s | 2.86 tok/s | 69.4 s |
| Chinese short smoke | 12.82 s | 3.33 tok/s | 22.7 s |
| Chinese long benchmark | 17.62 s | 3.07 tok/s | 48.9 s |

現在のボトルネックは Metal の計算性能ではなく、SSD からの expert weight 移動です。計測の内訳とアーキテクチャの詳細は [qwen122b-porting.md](docs/qwen122b-porting.md) を参照してください。

最初の summary-only baseline は [`benchmarks/qwen122b/2026-03-29-m5-air-16gb`](benchmarks/qwen122b/2026-03-29-m5-air-16gb/) にあります。新しい実行の raw logs、環境情報、doctor output は [`tools/collect-qwen122b-repro-pack`](tools/collect-qwen122b-repro-pack) で収集できます。

## なぜ動くのか

Qwen3.5-122B-A10B は Mixture-of-Experts モデルです。各 token で使われる routed experts は一部だけです。Sebas は dense weights を小さなマシンで扱える範囲に保ちつつ、routed expert files を SSD から必要に応じて読み出せるようにモデルを準備します。

これは「122B モデル全体が 16GB RAM に入る」という主張ではありません。Sebas はローカル streaming runtime です。

- active path は Apple Silicon Metal で計算
- routed experts は SSD backed streaming
- Qwen3.5 MoE の形状を理解した export と runtime config
- 122B bring-up 向けの安定した text-only inference path

## なぜ Ollama ではないのか

Sebas は Ollama を置き換える汎用ローカル model runner ではありません。Ollama は対応 backend に収まるモデルでは非常に優れています。一方、この 122B bring-up は標準的な「GGUF を読み込んで実行する」path ではありません。

最初に動いた 122B path は MLX 4-bit safetensors から始まり、GGUF overlay を必要としません。推論の前に Qwen3.5 MoE-aware な準備が必要です。

- `config.json` と safetensors shape から architecture を導出
- routed experts を layer ごとのファイルへ repack
- active expert blocks を `pread` で SSD から stream
- その layout に合わせた C、Objective-C、Metal kernels を駆動

この custom engine こそが Sebas の存在理由です。CLI は engine と benchmark workflow のための operational wrapper に絞っています。

## ソース checkout

公開 entrypoint は top-level の `./sebas` command です。

```bash
./sebas
```

ソースからビルドする場合:

```bash
git clone https://github.com/musshiyaki/sebas.git
cd sebas
tools/install-sebas
sebas
```

## ローカル engine setup

Sebas は Flash-MoE engine code を同梱・再配布しません。互換性のある外部
engine checkout を使う場合は、利用前に upstream repository と license status
を自分で確認してください。

```bash
git clone https://github.com/Anemll/flash-moe flash-moe-anemll-ios
```

local engine commands を使うには、まず local workspace manifest を作成します。

```bash
mkdir -p .workspace
cp .workspace.example/manifest.json .workspace/manifest.json
cp .workspace.example/system-no-think.md .workspace/system-no-think.md

./sebas engine doctor --engine qwen122b
./sebas chat
./sebas engine bench --engine qwen122b
./sebas engine bench --engine qwen122b --lang all --case all --long-tokens 160
./sebas run engine-only --engine qwen122b
```

source MLX model からの full 122B setup は [qwen122b-runbook.md](docs/qwen122b-runbook.md) を参照してください。現在の public umbrella repo は Sebas CLI と documentation を追跡しています。Flash-MoE engine checkout は、再配布と upstream license terms が十分に整理されるまで tracked tree の外に置いています。

## この repository に含まれるもの

| Path | Purpose |
|---|---|
| `sebas` | chat、engine、demo、doctor、benchmark commands の main CLI entrypoint |
| `rust/` | 最小構成の Sebas runner CLI |
| `.workspace.example/` | local engine manifest の例。`.workspace/` にコピーして使います |
| `docs/qwen122b-runbook.md` | public 122B setup and benchmark runbook |
| `docs/qwen122b-porting.md` | public 122B architecture and measurement notes |
| `tools/` | thin operational wrappers |
| `docs/` | workspace architecture notes |
| `engines/` | external engine ownership and layout notes |

## 現在の状態

これは研究開発段階の local runtime であり、完成した consumer app ではありません。

現在動いているもの:

- Qwen3.5-122B-A10B text-only inference path
- MacBook Air 16GB bring-up と prefill/decode の実測値
- local chat と engine operation 用の `./sebas` CLI wrapper
- Qwen35B / Qwen122B engine selection paths
- benchmark and doctor commands

まだ experimental なもの:

- long-context latency
- fast mode / malloc-backed expert cache stability
- Qwen3.5 shape family を超えた arbitrary MoE model support
- vision tensors
- package-manager installers

## 背景

この推論 work は Flash-MoE と、「expert weights をオンデマンドに stream すれば、非常に大きな MoE models も小さな local machines で動かせる」という考え方に基づいています。Anemll fork はその方向性を Apple Silicon と 122B Qwen3.5 path に拡張しています。

関連 docs:

- [Installing Sebas](docs/install.md)
- [Qwen3.5-122B porting notes](docs/qwen122b-porting.md)
- [Qwen3.5-122B runbook](docs/qwen122b-runbook.md)
- [Reproducibility pack workflow](docs/qwen122b-repro-pack.md)
- [Tracked benchmark summaries](benchmarks/qwen122b/)
- [Workspace architecture](docs/WORKSPACE_ARCHITECTURE.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)

## 制限

- 122B path は現在 text-only です。
- first-token experience は小さな local models と比べるとまだ遅いです。
- model preparation は大きく、技術的です。
- engine は現在 Qwen3.5 MoE family assumptions を対象にしています。
- 再現性は Apple Silicon hardware、local SSD behavior、prepared model layout に依存します。

これらの制限を明示しているのは、このプロジェクトの面白さが「巨大モデルが魔法のように軽くなった」と見せることではなく、小さな hardware 上で巨大な local model を成立させる engineering constraint にあるからです。

## 関係性

Sebas は独立した research project です。Apple、Alibaba/Qwen、ANEMLL、Hugging Face、MLX Community、Flash-MoE upstream projects とは提携、承認、保守関係にありません。第三者名は compatibility、attribution、setup、benchmark context を説明する目的でのみ使用しています。

## License

[LICENSE](LICENSE) と [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) を参照してください。
