---
title: Editor60 lossless scene node control identity
category: zircon_editor
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
related_code:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_fragment.rs
tests:
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/scene_node_identity.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
---

# Editor60 lossless scene node control identity

## Source finding and repair

ED60-P1-02/G07 remained partial: the product selection dispatcher already resolves authored controls through a lossless `control -> EntityId` map, but the `scene_node_id` property still used `entity.min(i64::MAX as u64) as i64`. Every entity above `i64::MAX` therefore exposed the same property value, and sparse hierarchy patches compared against that colliding value.

The retained hierarchy bridge now writes the full `u64` EntityId as a decimal `UiValue::String`. Sparse patch admission borrows that property from the control metadata, parses it as a checked `EntityId`, and compares it with the authoritative row identity without cloning the stored string. The saturating adapter is removed. Selection dispatch, Runtime scene IDs, and the fixed ten physical scene controls are unchanged. This patch only examines the changed rows when checking identity; it does not add a whole-tree scan. It does add decimal string storage for each materialized control and checked parsing for each changed control row, so no unmeasured allocation or latency win is claimed.

The new regression uses the real `SceneEntries` to bridge path for `i64::MAX`, `i64::MAX + 1`, and `u64::MAX`. It checks distinct exact properties and the existing `control -> EntityId` map, applies a successive-generation sparse content patch without reflow, then corrupts one high-ID control property to its neighbor's old colliding value and requires a resync rather than applying the wrong row.

## Evidence and remaining gates

Exact before-edit source and test-module bytes are saved under `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-v-editor1027-lossless-scene-node-id-preimage/`; the three new paths were absent. The existing one-line foreign comment in the test module is preserved. Lease request `99d75cde521e4c53aad40a5e869b98f4` acquired all five paths. Exact plan-write authorizations `2eb8023619a24d459e4909a4294bf0fc` and `0f9ba91d66ff4a88bc815dd320b7c33d` allowed the optimize and Astra records.

Scoped `rustfmt --edition 2021 --check`, tracked `git diff --check`, all-five-path whitespace/newline checks, and a source scan for the old clamp/Int adapter passed. These static checks do not execute the regression.

The focused Editor library regression, managed batch, true Editor window behavior, 100K loaded and 1M mixed hierarchy workloads, p95/p99 latency, RSS and allocation budgets, and same-workload reference benchmarks remain pending. Static source checks alone do not close those product gates or the full Editor60 plan.
