#!/usr/bin/env sh
set -eu

REPO="${SEBAS_REPO:-musshiyaki/sebas}"
VERSION="${SEBAS_VERSION:-latest}"
PREFIX="${SEBAS_INSTALL_PREFIX:-$HOME/.local}"
BIN_DIR="${SEBAS_INSTALL_BIN_DIR:-}"
FROM_SOURCE=0
PRINT_FOOTER=1

usage() {
  cat <<'USAGE'
Usage:
  install.sh [options]

Options:
  --version VERSION  Install a release tag such as v0.1.0 (default: latest)
  --prefix DIR       Install under DIR/bin (default: ~/.local)
  --bin-dir DIR      Install the sebas binary directly into DIR
  --from-source      Clone the repository and build from source
  -h, --help         Show this help

Environment:
  SEBAS_VERSION          Default release tag
  SEBAS_INSTALL_PREFIX   Default install prefix
  SEBAS_INSTALL_BIN_DIR  Default binary install directory
  SEBAS_REPO             GitHub repo override, default musshiyaki/sebas
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      VERSION="${2:?missing value for --version}"
      shift 2
      ;;
    --prefix)
      PREFIX="${2:?missing value for --prefix}"
      shift 2
      ;;
    --bin-dir)
      BIN_DIR="${2:?missing value for --bin-dir}"
      shift 2
      ;;
    --from-source)
      FROM_SOURCE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$BIN_DIR" ]; then
  BIN_DIR="$PREFIX/bin"
fi

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "ERROR: required command not found: $1" >&2
    exit 1
  fi
}

mktemp_dir() {
  mktemp -d 2>/dev/null || mktemp -d -t sebas-install
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Darwin:arm64|Darwin:aarch64)
      echo "aarch64-apple-darwin"
      ;;
    Darwin:x86_64)
      echo "x86_64-apple-darwin"
      ;;
    Linux:x86_64|Linux:amd64)
      echo "x86_64-unknown-linux-gnu"
      ;;
    Linux:aarch64|Linux:arm64)
      echo "aarch64-unknown-linux-gnu"
      ;;
    *)
      echo "unsupported"
      ;;
  esac
}

download() {
  url="$1"
  dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$url" -O "$dest"
  else
    echo "ERROR: curl or wget is required" >&2
    exit 1
  fi
}

verify_checksum() {
  archive="$1"
  checksum_file="$2"

  if [ ! -s "$checksum_file" ]; then
    echo "WARNING: checksum not found; continuing without checksum verification." >&2
    return 0
  fi

  checksum_dir="$(dirname "$checksum_file")"
  checksum_name="$(basename "$checksum_file")"
  if command -v shasum >/dev/null 2>&1; then
    (cd "$checksum_dir" && shasum -a 256 -c "$checksum_name")
  elif command -v sha256sum >/dev/null 2>&1; then
    (cd "$checksum_dir" && sha256sum -c "$checksum_name")
  else
    echo "WARNING: shasum or sha256sum not found; continuing without checksum verification." >&2
  fi

  if [ ! -s "$archive" ]; then
    echo "ERROR: downloaded archive is empty" >&2
    exit 1
  fi
}

install_from_source() {
  need_cmd git

  if ! command -v cargo >/dev/null 2>&1; then
    echo "ERROR: cargo was not found. Install Rust from https://rustup.rs/ and retry." >&2
    exit 1
  fi

  tmp_dir="$(mktemp_dir)"
  trap 'rm -rf "$tmp_dir"' EXIT INT TERM

  clone_dir="$tmp_dir/sebas"
  if [ "$VERSION" = "latest" ]; then
    git clone --depth 1 "https://github.com/$REPO.git" "$clone_dir"
  else
    git clone --depth 1 --branch "$VERSION" "https://github.com/$REPO.git" "$clone_dir"
  fi

  "$clone_dir/tools/install-sebas" --bin-dir "$BIN_DIR"
  PRINT_FOOTER=0
}

install_prebuilt() {
  target="$(detect_target)"
  if [ "$target" = "unsupported" ]; then
    echo "No prebuilt Sebas binary for $(uname -s)/$(uname -m); building from source." >&2
    install_from_source
    return 0
  fi

  need_cmd tar

  asset="sebas-$target.tar.gz"
  if [ "$VERSION" = "latest" ]; then
    url="https://github.com/$REPO/releases/latest/download/$asset"
  else
    url="https://github.com/$REPO/releases/download/$VERSION/$asset"
  fi

  tmp_dir="$(mktemp_dir)"
  trap 'rm -rf "$tmp_dir"' EXIT INT TERM

  archive="$tmp_dir/$asset"
  checksum="$tmp_dir/$asset.sha256"

  echo "Downloading $url"
  if ! download "$url" "$archive"; then
    echo "No prebuilt Sebas binary was found; building from source." >&2
    install_from_source
    return 0
  fi

  download "$url.sha256" "$checksum" || true
  verify_checksum "$archive" "$checksum"

  mkdir -p "$tmp_dir/extract"
  tar -xzf "$archive" -C "$tmp_dir/extract"

  if [ ! -x "$tmp_dir/extract/sebas" ]; then
    echo "ERROR: release archive did not contain an executable sebas binary" >&2
    exit 1
  fi

  mkdir -p "$BIN_DIR"
  install -m 0755 "$tmp_dir/extract/sebas" "$BIN_DIR/sebas"
  echo "Installed sebas to $BIN_DIR/sebas"
  "$BIN_DIR/sebas" --version
}

if [ "$FROM_SOURCE" = "1" ]; then
  install_from_source
else
  install_prebuilt
fi

if [ "$PRINT_FOOTER" = "1" ]; then
  case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
      echo
      echo "Add this to your shell profile if sebas is not found:"
      echo "  export PATH=\"$BIN_DIR:\$PATH\""
      ;;
  esac

  echo
  echo "Try:"
  echo "  sebas"
fi
