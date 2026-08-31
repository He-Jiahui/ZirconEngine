---
title: Plugin Shader WGSL Importer Current Source Protected Plan Routing
date: 2026-08-24
status: routing_only
source_report:
  - docs/plans/performance/01/2026-08-24-plugin-shader-wgsl-importer-current-source-performance-review.md
---

# Plugin Shader WGSL Importer Current Source Protected Plan Routing

| Existing owner | Required adoption |
|---|---|
| Plugins05 shader importer owner | Own the unique frontend hard cut, provider/catalog truth, capability admission, source graph, validated IR/reflection, target artifact and NativeDynamic withdrawal/re-entry gates. |
| Runtime04 asset owner | Own generic asynchronous import operation, source/artifact identity, dependency invalidation, atomic publication and cache transaction boundaries. |
| Runtime09C shader/material owner | Own target/backend/device qualification, shader variant and pipeline/PSO artifact installation, last-good generation and GPU acceptance. |
| Plugins01 / Plugins04 | Own native import bridge, capability negotiation, generation leases, unload/reload and replay behavior; registration metadata alone cannot imply import availability. |
| Editor15 | Own non-blocking diagnostics/preview consumption, cancellation, stale-generation suppression and last-good shader presentation. |
| Tooling08 | Own shared DDC key/storage/telemetry, in-flight coalescing and cold/warm cache qualification. |

Protected `docs/plans/performance/review.md` and `docs/plans/performance/pending.md` remain untouched. Their owner may add one concise `shader_wgsl_importer` pending entry only after adopting the current-source report. The module cannot move to accepted until Cargo, worker/cache gates, WPR and rendered target-artifact validation pass.
