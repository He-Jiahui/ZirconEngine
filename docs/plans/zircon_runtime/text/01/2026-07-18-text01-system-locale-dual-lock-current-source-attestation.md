# Text01 system locale dual-lock current-source attestation

Plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md

Milestone: M2

Status: completed

Files: ["Cargo.lock", "zircon_plugins/Cargo.lock", "zircon_runtime/Cargo.toml", "zircon_runtime/src/text/language.rs", "zircon_runtime/src/text/parallel/raster_pool.rs", "docs/plans/zircon_plugins/02/fixed-2026-07-18-runtime-text-system-locale-lock-drift.md", "docs/plans/zircon_runtime/text/01/2026-07-18-runtime-text-system-locale-lock-drift-return.md", "docs/plans/zircon_runtime/text/01/2026-07-18-text01-system-locale-dual-lock-current-source-attestation.md"]

## Scope delivered

- `sys-locale` remains an optional dependency activated only by the existing Runtime `text`
  feature.
- The root and plugin canonical lockfiles now describe the same current `zircon_runtime`
  path-package dependency graph.
- The locale and raster source hard cut remains Text-owned; no compatibility shim, legacy
  re-export, or upper-layer workaround was introduced.
- The canonical cross-plan failure is returned as fixed through the coordinator receipt API.

## Fresh testing evidence

| Stage | Scope | Status |
| --- | --- | --- |
| M2 | Current-source dual-lock metadata testing | passed |

- Root lock generation: managed job `eff9b66f9e33468d9d5cbd625287abd5`, exit 0.
- Plugin lock generation: managed job `1e5fff515e294988b74a5f8bdd6cf656`, exit 0.
- Root locked gate: managed job `3b93428cb229499a85b4c05add531946`, exit 0.
- Plugin locked gate: managed job `ba72a46c16184e2fb853a2cad94cf43c`, exit 0.
- Final lock hashes are
  `9D7B95211BCC226D2BFA73B31DB7217C82A3660250EFB4C65BEB7A4D7A611A47`
  (root) and
  `6085AE284ACAD8C50C22DBE516E3886EB6D73328E66EB6E6845E8854903B134E`
  (plugin), unchanged after both `--locked` gates.

## Review

Independent current-source review accepted the exact five-path product scope with Critical 0,
Important 0, and Minor 0. The review confirmed each lock changed only by adding `sys-locale` to
the existing `zircon_runtime` dependency list and found no package, version, or checksum drift.
