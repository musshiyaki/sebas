# Third-Party Notices

This file records third-party or upstream material that this workspace keeps alongside the Sebas wrapper layer.

It is intentionally conservative:

- If a component has an upstream license file, follow that file first
- If a component does not show a clear license in this workspace copy, treat reuse and redistribution as unconfirmed until the upstream project is checked directly
- If you add copied code, docs, images, or prompts later, record the source next to that file as well

## Included Components

### `rust/`

- Upstream base: [`ultraworkers/claw-code-parity`](https://github.com/ultraworkers/claw-code-parity)
- Sebas の新しい agent/runtime/CLI 層は、この upstream parity 実装をベースに取り込んだ Rust workspace の上で構築している
- Upstream license and notice requirements should be checked against the imported upstream tree before redistribution
- If additional Sebas-specific patches are copied elsewhere, keep the upstream attribution with those files

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
