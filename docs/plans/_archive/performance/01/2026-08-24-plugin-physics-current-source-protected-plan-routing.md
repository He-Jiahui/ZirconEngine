---
title: Plugin Physics Current Source Protected Plan Routing
date: 2026-08-24
scope:
  - zircon_plugins/physics
status: routing_pending_owner_absorption_and_dynamic_evidence
source_review:
  - docs/plans/performance/01/2026-08-24-plugin-physics-current-source-performance-review.md
---

# Plugin Physics Current Source Protected Plan Routing

The 86/86 Rust-file static review and local shared-snapshot fix are complete, but product and dynamic acceptance are not. This note routes findings without editing protected ledgers or independently owned plans.

| Owner plan | Required absorption |
|---|---|
| `docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md` | Own the complete Physics source/runtime/editor/dist product, selected backend, native behavior parity, persistent world bridge, native queries/events/constraints, cooked assets and ragdoll compile/runtime contract. |
| `docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md` | Add explicit Physics provider/profile classification and fail closed when required backend/system behavior is absent; source and native Dist must resolve the same product. |
| `docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md` | Retire full snapshots, duplicate all-pairs event scans, Rust-projected Jolt constraints and linear mirror queries; define backend-owned broadphase/query/event/constraint behavior. |
| `docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md` | Make Runtime scheduling the single fixed-step authority and publish dt/substep/remainder/drop/interpolation receipts used by Physics and replay. |
| `docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md` | Own per-world physics jobs, bounded command/result queues, dependencies, cancellation, thread budgets, replacement/shutdown and no unbounded pending work. |
| `docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md` | Provide component change generations, dirty extraction and active/changed writeback so Physics work scales with mutations rather than all scene nodes. |
| `docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md` | Own subscription-aware physics event delivery, bounded backlogs and overflow receipts instead of unconditional universal pair/event materialization. |
| `docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md` | Own cook/ragdoll/debug authoring jobs, progress/cancellation/stale generation, incremental overlay data and the same provider/artifact generation used by Runtime. |
| `docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md` | Surface phase timings, counts, queue depth, fixed-step misses and current backend/world generations rather than one aggregate physics duration. |
| `docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md` | Store versioned backend-native mesh/height-field/ragdoll cook artifacts in explicit non-C DDC roots with integrity, cache and cold/warm receipts. |
| `docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md` | Own reproducible WPR/ETW CPU, scheduling, allocation and power captures for current-source Physics product scenes; RenderDoc remains a visual/GPU-only gate. |
| `docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md` | Enforce complexity/data-movement gates for broadphase candidates, dirty sync, active writeback, dense ragdoll buffers and zero steady-state full snapshots. |

Protected `docs/plans/performance/review.md` and `pending.md` remain unchanged. Promotion requires owner absorption, managed Windows tests, one resolved product/backend graph, current-source launchability, correctness-matched WPR evidence, bounded scale results and power data. Only then may this module receive a milestone commit and WeCom completion notification.
