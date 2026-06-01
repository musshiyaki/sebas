# Sebas Project Architecture

## Intent

Sebas keeps the public CLI, docs, and orchestration surface in this umbrella
repository while preserving local engine checkouts as separate histories.

## Public Paths

- `sebas`: shared CLI entrypoint
- `rust`: optional Sebas agent runtime, TUI, tool execution, config surface
- `.workspace.example/manifest.json`: example local engine manifest
- `docs/qwen122b-runbook.md`: public 122B setup and benchmark runbook
- `docs/qwen122b-porting.md`: public architecture and measurement notes
- `engines/README.md`: policy for external engine checkout ownership

## Local-Only Paths

The following paths may exist in a working tree but are intentionally ignored by
the umbrella repository:

- `flash-moe-anemll-ios`: primary local inference engine checkout
- `flash-moe`: reference engine checkout
- `.workspace/manifest.json`: local runtime configuration copied from
  `.workspace.example/manifest.json`
- `.workspace/system-no-think.md`: local engine system prompt copied from
  `.workspace.example/system-no-think.md`

## Current Policy

- The root README must only link to files tracked in the public umbrella repo.
- Engine source redistribution stays out of the umbrella repo until upstream
  license and notice requirements are clarified.
- The local runtime still supports ignored engine checkouts at the paths named
  in `.workspace/manifest.json`.
- The runtime still keeps a legacy-compatible config / session / skill layout
  for existing users; public docs should present `sebas` as the supported
  entrypoint.
