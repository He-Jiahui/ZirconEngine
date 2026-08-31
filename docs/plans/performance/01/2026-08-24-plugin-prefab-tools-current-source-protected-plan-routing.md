---
title: Plugin Prefab Tools Current Source Protected Plan Routing
date: 2026-08-24
status: routing_only_protected_ledgers_unchanged
source_review: docs/plans/performance/01/2026-08-24-plugin-prefab-tools-current-source-performance-review.md
---

# Plugin Prefab Tools Current Source Protected Plan Routing

The protected `docs/plans/performance/review.md`, `pending.md` and numbered/main plans are intentionally unchanged. Route the reviewed findings to these canonical owners when their maintainers next update implementation scope:

| Owner | Routed responsibility |
|---|---|
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Link or absorb PrefabTools providers, remove duplicate importer metadata, resolve resources and enforce source/library/native truth. |
| `docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md` | Keep one prefab importer and publish immutable source/dependency/generation receipts. |
| `docs/plans/optimize/zircon_runtime/39-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-scalability-product-integration-review.md` | Own archetype/instance/override identity, nested graph, bounded instantiation, propagation, network/save and product acceptance. |
| `docs/plans/optimize/zircon_runtime/53-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-review.md` | Reconcile prefab source generations incrementally and preserve last-good instance state. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Admit prefab load/expand/reconcile/despawn jobs with priorities, cancellation and entity/byte budgets. |
| `docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md` | Publish prefab entities/components in bounded ECS batches and share immutable defaults. |
| `docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md` | Preserve stable nested hierarchy/transform identity and targeted propagation. |
| `docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md` | Replace string property paths with versioned component/property identities and typed deltas. |
| `docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md` | Share immutable prefab generation leases and remove full-scene clones per instance/load. |
| `docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md` | Make prefab apply/revert/break/save atomic, undoable and recoverable. |
| `docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md` | Own executable Prefab editor workflow and selection/instance editing behavior. |
| `docs/plans/optimize/zircon_editor/62-editor-inspector-property-grid-reflection-schema-multi-selection-edit-transaction-undo-prefab-override-customization-asset-reference-virtualization-product-integration-current-source-review.md` | Render and mutate typed override deltas without rebuilding/cloning complete indexes each frame. |
| `docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md` | Bind prefab operations to document/world generation, transactions, rollback and async completion. |
| `docs/plans/optimize/zircon_editor/65-editor-scene-object-creation-placement-palette-factory-asset-drag-drop-template-favorites-preview-transform-transaction-plugin-product-integration-current-source-review.md` | Implement create-from-selection, placement and artifact-backed preview through the canonical instance service. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Capture instance/reload/override scale receipts with WPR and qualified RenderDoc evidence. |
| `docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md` | Expose prefab shared/unique bytes, expansion reservations, peak RSS and lease accounting. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Track duplicate matcher/provider state, whole-scene clones, override-index rebuilds and propagation work. |

The implementation order is Plugins06/07 ownership closure, Runtime39/63 identity and product schema, Runtime59/60/64 bounded instantiation and sharing, then Runtime53 plus Editor02/03/62/63 transactional propagation and authoring. Tooling07/25 qualifies the complete product.

Dynamic acceptance remains pending. Do not mark this module accepted, create a milestone commit or send quantified WeCom results until the managed current-source executable completes the review gates.
