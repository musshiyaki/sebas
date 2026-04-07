# Sebas Workspace

Sebas は、ローカル実行用の Qwen 系エンジンと、それを使う Rust ベースの code-first AI coding agent runtime をまとめた親 workspace です。

正規の入口はトップレベルの `./sebas` です。現行の agent 層は `rust/` 配下の crate 群が本体です。

## What This Repo Contains

- `sebas`: 主要な code-first 実行コマンド
- `rust/`: Sebas agent runtime / CLI / tools / plugins
- `apps/`: 利用者向けの安定 entrypoint
- `tools/`: 起動・運用・ベンチ用 wrapper
- `docs/`: workspace の設計と運用方針
- `engines/`: エンジン構成の整理メモ
- `project-docs/`: 実験メモ、検証用プロンプト、関連資料

## Quick Start

```bash
./sebas
./sebas "explain the current diff"
./sebas engine doctor --engine qwen122b
./sebas engine doctor --engine qwen35b
./sebas run engine-only --engine qwen122b
./sebas run engine-only --engine qwen35b
./sebas config import-qwen
./sebas engine bench --engine qwen122b
```

## Current Layout

- `rust/`: claw-code-parity ベースで再構成した Sebas の agent layer
- `flash-moe/`: FlashMoE の実体
- `flash-moe-anemll-ios/`: iOS / 122B 系の実体
- `apps/`, `tools/`, `docs/`, `engines/`: Sebas の親レイヤー側の案内と wrapper

## Notes

- 実体の各リポジトリは、履歴保全のため既存パスのまま保持しています
- `.codex/` が canonical config / session / skill surface です。`.claw/` と `.claude/` は互換 discovery、`.qwen/` は import 対象です
- `.gitignore` で `.qwen/`、`.workspace/`、`node_modules/`、既存 repo 本体を除外しています
- 新しい変更は、まず親 workspace 側の案内や wrapper に寄せるのが基本です

## Third-Party Material

- まとめは [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md) を参照してください
