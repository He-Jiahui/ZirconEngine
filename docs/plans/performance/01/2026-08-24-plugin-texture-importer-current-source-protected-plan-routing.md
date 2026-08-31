---
title: Plugin Texture Importer Current Source Protected Plan Routing
date: 2026-08-24
status: routing_only_protected_ledgers_unchanged
source_review: docs/plans/performance/01/2026-08-24-plugin-texture-importer-current-source-performance-review.md
---

# Plugin Texture Importer Current Source Protected Plan Routing

The protected `docs/plans/performance/review.md`, `pending.md` and numbered/main plans are intentionally unchanged. Route the reviewed findings to the following canonical owners when their maintainers next update implementation scope:

| Owner | Routed responsibility |
|---|---|
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Add the missing TextureImporter provider/dependency closure and make source/library/native capability fail closed. |
| `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Own canonical source/dependency receipts, import recipe, expansion admission and deterministic publication. |
| `docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md` | Own the provider hard cutover and the end-to-end Texture source/import/compiler/runtime/editor product. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Admit decode/mip/compress/artifact jobs with byte budgets, memory-adjusted concurrency, priority and cancellation. |
| `docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md` | Replace typed-load deep clones with immutable generation-bound leases and coalesce same-key requests. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Define versioned texture build definitions, platform cook keys, last-good publication and independently cacheable chunks. |
| `docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md` | Separate logical texture descriptors from validated physical subresource layouts and reject descriptor/payload disagreement. |
| `docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md` | Own format policy, qualified mip/compression compiler, chunk upload, residency, VRAM budget and eviction. |
| `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Surface texture build queue/progress/cancellation and prevent editor-thread decode, mip and compression work. |
| `docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md` | Own recipe editing, artifact-backed previews, source/artifact comparison and generation-qualified reimport diagnostics. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Capture corpus-scale cold/warm build, WPR/ETW scheduling/memory/power and RenderDoc format/mip/upload/pixel receipts. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Store deterministic platform texture chunks without full-payload staging or duplicate source/cooked owners. |
| `docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md` | Enforce source/decoded/scratch/derived/upload/resident budgets and expose peak/lease/VRAM accounting. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Track clone bytes, per-phase complexity, worker partitioning, cache/coalescing results and subresource transition costs. |

The implementation order is Plugins06/07/18 provider and source truth, Runtime85/86 schema/build identity, Runtime59/64 scheduling and ownership, then Runtime92/Editor35 compiler, residency and authoring convergence. Tooling07/25 supplies qualification evidence; it must not turn lower-bound static byte models into measured claims.

Dynamic acceptance remains pending. Do not mark this package accepted, create a milestone commit or send quantified WeCom results until a managed current-source executable completes the review gates.
