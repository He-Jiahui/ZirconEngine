---
title: Material Editor Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/material_editor
status: routing_pending_owner_absorption_and_dynamic_evidence
source_review:
  - docs/plans/performance/01/2026-08-24-material-editor-current-source-performance-review.md
---

# Material Editor Current Source Protected Plan Routing

The 6/6 Rust-file static review is complete, but product/compiler parity, algorithm scale and dynamic acceptance are not. This note routes findings without editing protected ledgers, shared source changes or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md` | Close Material Editor resources, document/toolkit consumers, operation factories, source/native behavior and typed registration/operation receipts. |
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Add an explicit Material Editor provider/profile only when executable behavior exists; registration-only native Dist must fail capability admission. |
| `docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md` | Own the versioned graph document/schema, typed connections, transactions, compiler generations, diagnostics, exact artifact preview and removal of base-color-only success semantics. |
| `docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md` | Make graph import/reimport run the same bounded semantic compiler, track dependencies and expose source/artifact currentness rather than accepting any output node. |
| `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Own latest-wins graph/shader compile jobs, debounce/coalescing, priority, cancellation, stale rejection, progress and bounded shutdown. |
| `docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md` | Define the executable material/shader artifact and bind graph output to Runtime material, variants, layouts, pipeline/PSO, fallback and hot-reload behavior. |
| `docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md` | Own one compiler authority, non-recursive O(V+E) lowering, typed IR, reflection/layout/variant/pipeline keys, cache/prewarm and removal/renaming of duplicate graph placeholders. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Connect Material Graph source/import/dependency generations to deterministic derived artifacts and packaging; invalid graphs must fail before resource readiness. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Provide bounded dependency-aware compiler execution, cancellation and current-generation publication without UI/render-thread accumulation. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Store versioned material/shader artifacts under explicit non-C DDC roots with integrity, dependency/compiler keys, bytes and cold/warm hit receipts. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Own managed scale fixtures, WPR/ETW CPU/scheduling/allocation/power evidence and RenderDoc preview shader/pipeline/binding/pixel parity. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Enforce unique-node O(V+E) graph work, non-recursive topology, no exponential DAG visits, bounded diagnostics/artifacts and zero stable-preview compile work. |

Protected `docs/plans/performance/review.md` and `pending.md` remain unchanged. Promotion requires owner absorption, one executable shared compiler product, managed tests and scale gates, current-source launchability, WPR evidence and RenderDoc correctness parity. Only then may this module receive a milestone commit and WeCom completion notification.
