# Third-Party Notices

This file records third-party or upstream material that this workspace keeps alongside the Sebas wrapper layer.

It is intentionally conservative:

- If a component has an upstream license file, follow that file first
- If a component does not show a clear license in this workspace copy, treat reuse and redistribution as unconfirmed until the upstream project is checked directly
- If you add copied code, docs, images, or prompts later, record the source next to that file as well

## Included Components

### `rust/`

- Sebas includes a small Rust CLI wrapper for engine, demo, doctor, benchmark,
  and local configuration commands
- The previous experimental runtime layer was removed from the tracked workspace
- Sebas-specific Rust code is offered under the root MIT license

### External Flash-MoE engines (not included)

The historical local workspace used `flash-moe/` and
`flash-moe-anemll-ios/`. Neither source tree is distributed in this public Sebas
repository. Inspection on 2026-10-28 covered:

- [`danveloper/flash-moe` at `3601d418`](https://github.com/danveloper/flash-moe/tree/3601d41882a346d85041f7955c66429d248e8cd9).
- [`Anemll/flash-moe`, `m5-nax` at `26cd7f84`](https://github.com/Anemll/flash-moe/tree/26cd7f841823ea255ad6996d96692102852fa591),
  plus `iOS-App` (`9d1d602b`), `main` (`20f85917`) and
  `ASTC-experiments` (`5e8ba26c`).

No `LICENSE` or `COPYING` file was found in those trees. The inspected default
branch READMEs identify the projects and fork relationship, but do not supply an
explicit redistribution grant. Treat reuse and redistribution as unconfirmed;
public source availability alone does not resolve the terms. Obtain the relevant
permissions before copying the engine or its derivatives into a release.

The Anemll README also attributes cached-read fanout work to
[`ncdrone/rustane`](https://github.com/ncdrone/rustane). Rights for incorporated
contributions need separate review; no conclusion about that code's terms is
made here.

The local 122B helper scripts and supporting source have been located in the
maintainer's private archive and inspected statically. The archive's own notices
also mark the external engines' redistribution terms as unconfirmed. Recovery
does not establish a redistribution grant or a complete reproducible release.
Origin, permissions and private data still require review before publication.
This documentation fix adds no engine or conversion code. See the [122B runbook](docs/qwen122b-runbook.md).

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
by, or maintained by Apple, Alibaba/Qwen, ANEMLL, Hugging Face, the MLX
Community, Flash-MoE upstream projects, or any other third-party project named
here. Third-party names, model identifiers, product names, and repository names
are used only for descriptive compatibility, attribution, setup, and benchmark
context.
