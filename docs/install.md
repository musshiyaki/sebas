# Installing Sebas

Sebas can be installed from source today. Prebuilt GitHub Releases and package
manager formulas are not published yet.

## Requirements

- macOS or Linux
- Rust toolchain with `cargo`
- Git

The CLI install does not download model weights or install the external
Flash-MoE engine checkout. The 122B inference path still requires the local
workspace setup in [qwen122b-runbook.md](qwen122b-runbook.md).

## Install From Source

```bash
git clone https://github.com/musshiyaki/sebas.git
cd sebas
tools/install-sebas
sebas --help
```

By default, the installer builds the release binary and copies it to
`~/.local/bin/sebas`.

If `~/.local/bin` is not on your `PATH`, add:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Install Somewhere Else

```bash
tools/install-sebas --prefix "$HOME/opt/sebas"
tools/install-sebas --bin-dir "$HOME/bin"
```

To reuse an already built binary:

```bash
cargo build --release --manifest-path rust/Cargo.toml -p sebas-cli --bin sebas
tools/install-sebas --no-build
```

## Verify

```bash
sebas --version
sebas --help
```

For engine work, run these from a Sebas workspace after copying the example
manifest:

```bash
mkdir -p .workspace
cp .workspace.example/manifest.json .workspace/manifest.json
cp .workspace.example/system-no-think.md .workspace/system-no-think.md

sebas engine doctor --engine qwen122b
```

## Uninstall

```bash
rm -f "$HOME/.local/bin/sebas"
```

Use the matching path if you installed with `--prefix` or `--bin-dir`.
