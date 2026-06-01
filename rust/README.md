# Sebas Rust CLI

This directory contains the minimal Rust CLI for Sebas. It is a local model
runner wrapper for local chat, demo, doctor, benchmark, and project default
engine selection.

## Build

```bash
cd rust
cargo build --release -p sebas-cli --bin sebas
```

## Commands

```bash
./target/release/sebas --help
./target/release/sebas chat
./target/release/sebas init
./target/release/sebas engine doctor --engine qwen122b
./target/release/sebas engine bench --engine qwen122b --lang all --case all --long-tokens 160
./target/release/sebas demo --tokens 96 "Explain why this local 122B demo is surprising."
./target/release/sebas model set qwen122b
```

## Workspace Layout

```text
rust/
├── Cargo.toml
├── Cargo.lock
└── crates/
    └── sebas-cli/
        ├── Cargo.toml
        ├── src/main.rs
        └── src/sebas_engine.rs
```

The older experimental runtime layer was removed so the public project can stay
focused on the 122B local inference proof point.

## License

See the repository root.
