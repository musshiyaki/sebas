# Tools

ここには運用・ベンチ・起動の wrapper を置きます。直接の起動ロジックは `sebas` にまとめ、個別スクリプト側では profile と engine を選ぶだけにします。

- `install-sebas`: builds the Rust CLI and installs `sebas` into a local bin directory.
- `collect-qwen122b-repro-pack`: collects environment, doctor, and benchmark evidence for the 122B path.

The repository root also has `install.sh`, which downloads a GitHub Release
binary when available and falls back to `install-sebas` for source builds.
