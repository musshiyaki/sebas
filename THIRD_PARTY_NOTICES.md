# Third-Party Notices

This file records third-party or upstream material that this workspace keeps alongside the Sebas wrapper layer.

It is intentionally conservative:

- If a component has an upstream license file, follow that file first
- If a component does not show a clear license in this workspace copy, treat reuse and redistribution as unconfirmed until the upstream project is checked directly
- If you add copied code, docs, images, or prompts later, record the source next to that file as well

## Included Components

### `rust/`

- Sebas includes an optional Rust code-agent runtime and API/tooling layer
- This layer incorporates work from the UltraWorkers Claw Code lineage:
  [`ultraworkers/claw-code`](https://github.com/ultraworkers/claw-code) and the
  archived historical repository
  [`ultraworkers/claw-code-parity`](https://github.com/ultraworkers/claw-code-parity)
- The current `ultraworkers/claw-code` repository exposes an MIT license
- The archived `ultraworkers/claw-code-parity` repository did not expose a
  GitHub-detected license at the time of review; keep this attribution and
  re-check provenance before redistributing this runtime independently
- Sebas-specific changes are offered under the root MIT license where the
  authors have the right to license them
- Sebas does not claim ownership of upstream Anthropic or Claude Code source
  material, and is not affiliated with Anthropic

### `flash-moe/`

- Upstream repository: [`danveloper/flash-moe`](https://github.com/danveloper/flash-moe)
- This workspace keeps the repository at its existing path so the upstream history stays intact
- The upstream README describes the Flash-MoE laptop inference project and the paper-backed implementation details
- No separate top-level license file is tracked in this workspace copy
- The current GitHub page snapshot does not show a LICENSE file in the repository root
- Treat reuse or redistribution as license-unclear until the upstream project is checked directly

### `flash-moe-anemll-ios/`

- Upstream repository: [`Anemll/flash-moe`](https://github.com/Anemll/flash-moe)
- The workspace copy preserves the fork history and the existing path layout
- The README identifies this tree as a fork of [`danveloper/flash-moe`](https://github.com/danveloper/flash-moe)
- The README also cites `ncdrone/rustane` as the source of the `--cache-io-split` fanout idea
- No separate top-level license file is tracked in this workspace copy
- The current GitHub page snapshot does not show a LICENSE file in the repository root
- Treat reuse or redistribution as license-unclear until the upstream and fork-history terms are checked directly

### Qwen model sources

- Upstream model: [`Qwen/Qwen3.5-122B-A10B`](https://huggingface.co/Qwen/Qwen3.5-122B-A10B)
- The upstream Hugging Face model page currently lists the license as Apache-2.0
- Sebas does not redistribute the upstream model weights
- Users are responsible for obtaining model assets from upstream sources and
  following the license and terms that apply to those assets

### MLX Community converted model

- Converted model used for the current 122B proof point:
  [`mlx-community/Qwen3.5-122B-A10B-4bit`](https://huggingface.co/mlx-community/Qwen3.5-122B-A10B-4bit)
- The MLX Community Hugging Face model page currently lists the license as Apache-2.0
- The model card states that it was converted to MLX format from
  `Qwen/Qwen3.5-122B-A10B`
- Sebas records benchmark metadata and local preparation notes, but does not
  redistribute MLX model weights or prepared expert files

### Demo media and preview assets

- `docs/assets/sebas-demo-running.gif` is cropped direct-camera footage created
  by the Sebas maintainer for this repository
- The GIF is included only as public project demo media; it does not include
  model weights, external source code, or third-party video footage
- `docs/assets/social-preview.svg` and `docs/assets/social-preview.png` are
  project preview assets created for Sebas

### `project-docs/ollama-web-search-mcp/`

- Local MCP server built on `@modelcontextprotocol/sdk` and `zod`
- Direct dependencies in `package.json`: `@modelcontextprotocol/sdk@^1.0.0` and `zod@^4.1.12`
- `package-lock.json` records the transitive dependency tree and bundled license metadata
- The current lockfile scan shows MIT, BSD-3-Clause, BSD-2-Clause, and ISC licenses in the dependency graph
- If this component is redistributed independently, include the dependency notices that apply to the installed npm packages

### `project-docs/`

- Contains experiments, prompts, notes, and supporting materials
- No copied external source has been identified in the current local scan
- If any file quotes, adapts, or reproduces external material, keep attribution close to the copied content

## Workspace-Level License

- The Sebas workspace itself is distributed under the root [`LICENSE`](./LICENSE)
- If a component has stronger or more specific upstream license terms, those terms take precedence for that component
- This notice file is informational and does not replace the license text shipped with each component

## Trademarks and Affiliation

Sebas is an independent research project. It is not affiliated with, endorsed
by, or maintained by OpenAI, Anthropic, Apple, Alibaba/Qwen, ANEMLL, Hugging
Face, the MLX Community, or any other third-party project named here.
Third-party names, model identifiers, product names, and repository names are
used only for descriptive compatibility, attribution, setup, and benchmark
context.
