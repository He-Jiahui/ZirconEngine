---
title: Plugin UI Document Importer Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/ui_document_importer
status: routing_pending_owner_absorption_and_dynamic_evidence
source_review:
  - docs/plans/performance/01/2026-08-24-plugin-ui-document-importer-current-source-performance-review.md
---

# Plugin UI Document Importer Current Source Protected Plan Routing

The 4/4 package Rust-file review and related product-path trace are statically complete. Cooked-artifact ownership, root-driven loading, compiler scale, editor/runtime convergence, native execution and dynamic acceptance remain open. This note routes the findings without editing protected ledgers or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Admit UI document import only when the selected source/library/native provider executes it; registration-only Dist metadata must not qualify as native behavior. |
| `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Own `.zui` semantic import, typed dependency closure, deterministic compiled product artifact, schema/compiler identity and source/native parity. |
| `docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md` | Remove registry-wide synchronous UI artifact loading from session creation; install requested roots asynchronously by matching dependency generation. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Schedule bounded UI dependency load/compile jobs with priority, cancellation, deadlines, latest-wins rejection and no global-lock I/O/compile region. |
| `docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md` | Provide root-driven dependency-closure handles, immutable compiled artifact leases, generation swaps, last-good retention and bounded reload. |
| `docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md` | Compile imports/components/slots/params/events to stable indices, validate prototype graphs once and remove whole-document-per-component clones and per-occurrence string scans. |
| `docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md` | Version component catalog semantics/feature bits in the compiled artifact so Runtime instances do not rediscover component capabilities. |
| `docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md` | Replace AST-to-TOML artifact round trips with a cooked UI package keyed by source/dependency/compiler/target/profile generations and usable by cook/package/runtime. |
| `docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md` | Define the compiled UI package schema, source-map/diagnostic sidecars, typed dependencies and migration/version rules; invalid references must fail rather than disappear. |
| `docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md` | Converge editor retained UI construction on reusable compiled products and bounded instance state rather than caller-thread source parsing and repeated graph copies. |
| `docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md` | Publish stable UI asset ID-to-URI/dependency indices and remove the fallback recursive asset-root parse scan. |
| `docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md` | Move editor UI load/compile/cache publication out of global mutex/UI paths into coalesced latest-wins jobs with last-good installation. |
| `docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md` | Make UI authoring, validation and preview consume the exact cooked artifact/compiler contract used by game Runtime and expose source/artifact/preview generations. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Store compiled UI packages by content/dependency/compiler/schema/target/profile identity under configured non-C roots; mtime/length alone is not currentness. |
| `docs/plans/optimize/zircon_tooling/31-declarative-project-asset-ui-scene-manifest-schema-generated-artifact-physical-authority-review.md` | Enforce authoring source versus generated/cooked UI artifact authority and remove duplicate parser/validator/product paths. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Track all-artifact startup loads, TOML round trips, O(C x D) document clones, repeated prototype validation, linear import lookup and global-lock compile/cache work. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Define cold/warm/fan-out/depth/multi-root/edit-burst WPR/ETW receipts and rendered UI RenderDoc parity with p50/p95/p99, bytes, visits, allocations and energy/frame. |

## Protected ledger disposition

Do not mark `docs/plans/performance/review.md` complete yet. Keep the module pending until owner plans absorb the findings and a managed current-source binary passes provider truth, root-closure loading, compiled-artifact, algorithm-scale, editor responsiveness, WPR/ETW, rendered UI parity and power gates. The eventual protected-ledger entry should remain module/folder-only per the root performance-plan convention.

No milestone commit or WeCom completion notice is warranted by static review alone.
