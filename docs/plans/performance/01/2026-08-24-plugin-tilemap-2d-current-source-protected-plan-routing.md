---
title: Plugin Tilemap 2D Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-tilemap-2d-current-source-performance-review.md
---

# Plugin Tilemap 2D Protected Plan Routing

## Review ledger status

`zircon_plugins/tilemap_2d` completed an E3 current-source static review over **11/11 Rust files**. This record does not edit protected `docs/plans/performance/review.md` or `docs/plans/performance/pending.md`; static review does not satisfy runtime, editor or GPU acceptance.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Package absent from linked catalogs; 0/2 resources; five operations and paint mode lack handlers/factory | `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md`, Editor34 and `docs/plans/zircon_plugins/10/failure-2026-08-01-terrain-tilemap-scene-mode-factories-missing.md` | Make readiness depend on selected executable providers, physical resources, document/controller and typed receipts. |
| Tiled `.json` diagnostic matcher conflicts at priority 0 with callable core Data JSON | plugin owner 06 and `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Remove broad matcher ownership; use explicit context/signature and one canonical Tiled build provider. |
| Dense layer DTO is cache/runtime payload and typed load deep-clones all cells | Runtime99e and `docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md` | Hard-cut source/document from sparse chunk artifact and shared runtime pages. |
| Validation lacks checked byte/chunk budget, stable identity, Tileset resolution and collider semantics | Runtime99e and `docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md` | Add semantic validation before allocation/cook and target-qualified artifact keys. |
| Scene world ignores load and saves `tilemap: None`; Tilemap render slot has no pass/consumer | Runtime99e and `docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md` | Deliver lossless typed component lifecycle and world-owned chunk execution before product qualification. |
| Paint helper is atomic/bounded but scans all cells once per stroke and is test-only | Editor34 and `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Preserve preflight semantics; move to coalesced reversible dirty-chunk transactions with cached indexes/stats and scheduled derived jobs. |
| Ignored 64 x 64 benchmark compares one scan with 128 scans but excludes product work | this performance review | Replace with current-source edit/chunk/frame/import WPR and RenderDoc evidence. |

## Acceptance routing

The module may move from static-reviewed to accepted only after M0-M6 in the source review close importer/product ownership, source/chunk/runtime identities, scene persistence, chunk-bounded edit/render/derived work and dynamic evidence. Minimum receipts include generic JSON determinism, no full-cell deep clone, bounded malformed import, one-cell dirty-chunk behavior, stationary zero rebuild/upload, WPR/ETW CPU/IO/power and RenderDoc Tilemap pixels/resources.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
