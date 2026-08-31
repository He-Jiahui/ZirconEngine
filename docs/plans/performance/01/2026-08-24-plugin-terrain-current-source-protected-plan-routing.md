---
title: Plugin Terrain Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-terrain-current-source-performance-review.md
---

# Plugin Terrain Protected Plan Routing

## Review ledger status

`zircon_plugins/terrain` completed an E3 current-source static review over **11/11 Rust files**. This record deliberately does not edit `docs/plans/performance/review.md` or `docs/plans/performance/pending.md`: those protected ledgers require the owner workflow, and static completion does not satisfy dynamic acceptance.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Package absent from linked catalogs; 0/3 declared resources; commands lack handlers; dist has no bridge | `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Make readiness depend on selected executable providers, resolved resources and typed operation receipts. |
| Inline `Vec<Real>` source reused as cache/runtime payload; typed loads deep-clone it | `docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md` and Runtime29 | Hard-cut to source/build-artifact/runtime-instance identities and shared generation leases. |
| Dimension-only admission, no byte/work budget or format semantics | `docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md` and Editor09 | Add header-first validation and budgeted cancellable import/build scheduling. |
| No Terrain scene/runtime consumer, tile/LOD/culling/residency/render/collision/nav chain | `docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md` | Implement the generation-qualified runtime architecture and dynamic evidence gates already owned there. |
| Create/open/import/sculpt are metadata; no dirty-region transaction or undo/job policy | `docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md` | Require bounded region edits, coalescing, reversible deltas and currentness receipts. |
| Built-in typed Terrain importer and diagnostic RAW/R16/PNG plugin importer have split authority; PNG resolves Texture | Runtime29 and plugin owner 06 | Establish one explicit Terrain build provider and deterministic contextual importer selection. |
| Extension-allocation microbenchmark does not exercise production work | `docs/plans/performance/01` Terrain review | Replace release evidence with corpus import/build, brush-region, residency/frame, WPR and RenderDoc measurements after an executable exists. |

## Acceptance routing

The module may move from static-reviewed to accepted only after M0-M6 in the source review provide current-source receipts. Minimum evidence is: selected provider/resource/handler closure; no deep clone of full samples on typed load; bounded corrupt/oversized import; per-view tile/LOD/culling counters; stationary zero-rebuild behavior; region-scaled brush work; generation-matched render/physics/navigation; WPR/ETW CPU/IO/power capture; and RenderDoc draw/resource/pixel capture.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
