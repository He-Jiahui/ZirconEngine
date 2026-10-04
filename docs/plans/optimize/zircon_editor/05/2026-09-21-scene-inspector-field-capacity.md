---
title: Editor Scene Inspector Field Capacity
category: zircon_editor
report_id: Editor883-scene-inspector-field-capacity-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor883 Scene Inspector Field Capacity

## Finding

Scene edit-mode projection converts the selected entity's runtime inspection
fields into retained Editor Inspector fields. Unsupported reflected values are
filtered out, so the previous `filter_map(...).collect()` exposed a zero lower
size bound and could grow geometrically for a dense supported component. An
eager allocation would regress entities whose runtime fields are all
unsupported.

## Optimization

- Route runtime-field projection through one shared helper.
- Keep the output at zero capacity until the first supported field is
  materialized, then reserve the runtime field-count upper bound once.
- Append supported projections directly in runtime field order.
- Preserve unsupported-value filtering, component/field labels, property
  paths, reflected values, editability, selected-entity gating, and the empty
  default path.

## TDD and deterministic evidence

The Editor883 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. Lower regressions prove 64 `Null` fields retain zero capacity and mixed
unsupported/supported input preserves exact output equality and order.

For 4,096 supported runtime fields, the retired zero-lower-bound model performs
11 geometric capacity changes and the lazy upper-bound model performs zero. The
ignored 101-pair Release marker
`EDITOR883_SCENE_INSPECTOR_FIELD_CAPACITY_BENCH_V1` emits alternating
p50/p95/p99 samples, checks complete output equality, locks both growth counts,
and requires lazy-reserved p95 to remain within 10% of filtered collection.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- Editor879–883 plus the adjacent Asset Browser contract batch passes `43/43`.
- Lower Rust execution was submitted with Editor882 in asynchronous v12 (PID
  `28704`); no per-task Cargo run is launched.
- A later one-time v12 receipt read showed Runtime passing, while Editor stopped
  before Cargo because a compile-time include resource was unavailable; no
  Editor Rust diagnostic or acceptance evidence was produced.
- Local evidence does not establish Windows compilation, allocator behavior,
  or Scene Inspector product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/scene/viewport/edit_mode_projection/build.rs` | `A7DFDEC9439DD39069DE700C3470F66C60EE6111ED813FFB48F6F8BE0AB5314B` |
| `zircon_editor/src/scene/viewport/edit_mode_projection/build/field_capacity_tests.rs` | `AE2D5983DB17F0D491D825E3E569CFCA4992F5EF8F22278C138E6FD1D8203702` |
| `tools/tests/test_editor883_scene_inspector_field_capacity_performance_contract.py` | `282DCCC208931219E3B7BA6A8DB9A712046F64DA810CDFFDF33EA1B1ADA3C477` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regressions and ignored Release marker, and supplies allocator plus real Scene
Inspector product p50/p95/p99 evidence. The deterministic growth model is not
product acceptance.
