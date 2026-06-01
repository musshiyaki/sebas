# Engines

Sebas keeps the public CLI/orchestration surface separate from the external
engine checkouts.

- `flash-moe-anemll-ios`: primary local inference engine checkout for the 122B
  Apple Silicon bring-up
- `flash-moe`: reference implementation checkout

These engine directories are intentionally ignored by the umbrella repository
until redistribution and upstream license terms are clarified. For local runs,
copy `.workspace.example/manifest.json` to `.workspace/manifest.json` and keep
the engine checkout at the paths named in that manifest.
