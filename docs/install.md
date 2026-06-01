# Installing Sebas

Sebas can be installed from GitHub Releases when a prebuilt binary is available.
If a release asset is missing for your platform, the installer falls back to a
source build.

## Requirements

- macOS or Linux
- `curl` or `wget`

The CLI install does not download model weights or install the external
Flash-MoE engine checkout. The 122B inference path still requires the local
workspace setup in [qwen122b-runbook.md](qwen122b-runbook.md).

## Install Latest Release

```bash
curl -fsSL https://raw.githubusercontent.com/musshiyaki/sebas/main/install.sh | sh
sebas --help
```

By default, the installer downloads a prebuilt binary for your platform and
copies it to `~/.local/bin/sebas`.

Supported release assets:

- `sebas-aarch64-apple-darwin.tar.gz`
- `sebas-x86_64-apple-darwin.tar.gz`
- `sebas-aarch64-unknown-linux-gnu.tar.gz`
- `sebas-x86_64-unknown-linux-gnu.tar.gz`

## Install A Specific Version

```bash
curl -fsSL https://raw.githubusercontent.com/musshiyaki/sebas/main/install.sh | sh -s -- --version v0.1.1
```

You can also set the version with an environment variable:

```bash
SEBAS_VERSION=v0.1.1 sh -c "$(curl -fsSL https://raw.githubusercontent.com/musshiyaki/sebas/main/install.sh)"
```

## Source Build Fallback

The installer falls back to a source build when a prebuilt release asset is not
available. Source builds require:

- Rust toolchain with `cargo`
- Git

To force a source build:

```bash
git clone https://github.com/musshiyaki/sebas.git
cd sebas
tools/install-sebas
sebas --help
```

Or:

```bash
curl -fsSL https://raw.githubusercontent.com/musshiyaki/sebas/main/install.sh | sh -s -- --from-source
```

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

The top-level `install.sh` accepts the same destination options:

```bash
curl -fsSL https://raw.githubusercontent.com/musshiyaki/sebas/main/install.sh | sh -s -- --prefix "$HOME/opt/sebas"
curl -fsSL https://raw.githubusercontent.com/musshiyaki/sebas/main/install.sh | sh -s -- --bin-dir "$HOME/bin"
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

## Release Maintainers

Prebuilt CLI assets are produced by `.github/workflows/release.yml` when a tag
matching `v*` is pushed, or when the workflow is run manually with a tag input.
