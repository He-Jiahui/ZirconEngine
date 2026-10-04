---
title: Plugin Particles Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/particles
status: routing_pending_owner_absorption_and_dynamic_evidence
source_review:
  - docs/plans/performance/01/2026-08-24-plugin-particles-current-source-performance-review.md
---

# Plugin Particles Current Source Protected Plan Routing

The 50/50 Rust-file static review is complete, but product clock/backend ownership, authoring/artifact closure, algorithm scale and dynamic acceptance are not. This note routes findings without editing protected ledgers, shared source changes or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md` | Own the complete particle product: one clock/backend state machine, source/native parity, versioned source and compiled artifacts, CPU/GPU semantics, optional providers, preview and measured acceptance. |
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Admit Particles only when the selected source/native provider executes the advertised Runtime/Editor behavior; registration-only Dist and unsupported nested capabilities must fail closed. |
| `docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md` | Implement the particle document, module/curve authoring, diagnostics, exact-artifact preview, transport controls, scalability settings and source/artifact/preview generation truth. |
| `docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md` | Add a real `particles.system` importer/reimport/dependency/currentness path and remove template-only asset-kind success. |
| `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Own latest-wins particle compile/preview warmup jobs, debounce/coalescing, priority, cancellation, stale rejection, progress and bounded shutdown. |
| `docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md` | Define where particle component/system lifetime and frame progression enter world update; eliminate manual/test-only manager ticking. |
| `docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md` | Publish particle component/query/change access so simulation, transform input, visibility and extraction can be partitioned without one global mutex. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Own fixed/variable particle phases, dependency-aware CPU chunk jobs, backend activation/migration, thread budgets, completion/finalize and preview cancellation. |
| `docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md` | Make particle compute/render resources and fences graph-owned; support fence-safe artifact/buffer generation swaps and bounded external resources. |
| `docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md` | Own particle bounds, distance/significance/visibility admission, GPU-scene publication, previous-frame data and visible batch construction before sort/draw. |
| `docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md` | Replace no-op particle graph executors and prepare-time direct compute with real pass callbacks, dynamic dispatch extents, barriers, queue scheduling, culling and timestamps. |
| `docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md` | Compile particle shader modules/curves/layout/reflection/pipeline recipes into immutable cached artifacts; remove per-frame authoring-shaped compilation and preserve last-good generations. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Own deterministic particle source import, dependency hashing, CPU/GPU derived artifacts, cook/package and non-C DDC publication. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Store particle compiler/shader/pipeline artifacts by schema/compiler/dependency/target/backend/profile identity under configured non-C roots. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Track capacity-wide dispatch, per-slot emitter scan, manager-wide serialization, high-water CPU scans, snapshot/extract clones/sort, per-frame asset aggregation and unconditional readback as governed hot paths. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Define fixed particle scenes and WPR/ETW/RenderDoc receipts with p50/p95/p99, active/reserved counts, bytes/dispatches/readbacks, frame pacing and energy/frame. |

## Protected ledger disposition

Do not mark `docs/plans/performance/review.md` complete yet. Keep the module pending until the owner plans absorb the findings and a managed current-source binary passes clock/backend, CPU/GPU parity, scale, WPR/ETW, RenderDoc and power gates. The concise protected-ledger row should eventually use the module/folder name only, per the root performance-plan convention.

No milestone commit or WeCom completion notice is warranted by static review or the scoped local corrections alone.
