---
title: Plugin Asset Importers Current Source Protected Plan Routing
date: 2026-08-24
status: routing_only_protected_ledgers_unchanged
source_review: docs/plans/performance/01/2026-08-24-plugin-asset-importers-current-source-performance-review.md
---

# Plugin Asset Importers Current Source Protected Plan Routing

The protected `docs/plans/performance/review.md`, `pending.md` and numbered/main plans are intentionally unchanged. Route the reviewed findings to these canonical owners when their maintainers next update implementation scope:

| Owner | Routed responsibility |
|---|---|
| `docs/plans/optimize/zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md` | Remove aggregate/split WGSL overlap and join one target-specific shader compiler/artifact service. |
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Enforce one provider per matcher, linked catalog closure, duplicate-priority rejection and source/library/native truth. |
| `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Own immutable source receipts, recipes, import admission, subasset publication and provider hard cutover. |
| `docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md` | Absorb/remove aggregate audio descriptors and retain one cooked/streamed audio authority. |
| `docs/plans/optimize/zircon_plugins/17-first-party-virtual-geometry-source-runtime-editor-dist-catalog-asset-cook-cluster-page-streaming-culling-raster-product-integration-review.md` | Make virtual geometry an explicit independent derived recipe; reject unconditional importer cook. |
| `docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md` | Absorb/remove aggregate texture descriptors and resolve equal-priority PSD collision. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Admit parse/compile/cook jobs with memory-adjusted concurrency, priority, cancellation and shutdown fences. |
| `docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md` | Publish immutable generation-bound payload leases and coalesce same-key imports/loads. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Define canonical provider/source/recipe/target build identities and last-good artifact publication. |
| `docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md` | Replace raw-text-plus-generic-DOM and Model/Mesh duplicate payload schemas with typed references/products. |
| `docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md` | Own shader source graph, permutations, target compilation, reflection, cache and prewarm artifacts. |
| `docs/plans/optimize/zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md` | Own one mesh payload identity and optional LOD/collision/SDF/VG build products without full clones. |
| `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Surface import/compile queues and keep parse/cook work off the editor thread. |
| `docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md` | Own typed data schema, row keys, validation/migration diagnostics and lazy authoring projections. |
| `docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md` | Own model/mesh recipe UX, optional derived products, artifact preview and reimport receipts. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Capture end-to-end corpus, WPR/ETW and qualified RenderDoc receipts from current-source binaries. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Cache shader, mesh, data and optional derived products by deterministic input keys. |
| `docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md` | Enforce DOM/IR/geometry/compiler/artifact byte budgets and expose peak/lease accounting. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Track provider collisions, full-payload clone bytes, parser complexity, duplicate jobs and main-thread work. |

The implementation order is Plugins06/07 provider hard cut, Runtime85/86 identity/schema convergence, Runtime59/64 bounded jobs and leases, then Runtime91/93 plus the format-specific plugin owners. Editor24/32 and Tooling07/25 qualify the real product rather than isolated descriptor or helper microbenchmarks.

Dynamic acceptance remains pending. Do not mark the module accepted, create a milestone commit or send quantified WeCom results until the managed current-source executable completes the review gates.
