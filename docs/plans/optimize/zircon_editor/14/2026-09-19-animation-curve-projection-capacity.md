---
title: Editor14 animation curve projection capacity
category: zircon_editor
report_id: Editor814-animation-curve-projection-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor814 · animation curve projection capacity

## Scope

The selected-track curve projection used `filter_map(...).collect()` for the
component curves and `collect::<Option<Vec<_>>>()` for each component's keys.
Those collectors hid the known component/key bounds and paid iterator/Option
materialization overhead on every curve refresh.

## Implementation

- Reserve the exact scalar-component upper bound before ordered curve append,
  while retaining the existing `filter_map` skip behavior for malformed
  components.
- Reserve each channel-key input bound before direct key append.
- Preserve finite-value rejection, tangent projection, key IDs/points,
  interpolation, component names, and empty/discrete/quaternion zero-curve
  paths.
- Add a lower Rust component/order/invalid-input regression and the ignored
  `EDITOR814_ANIMATION_CURVE_PROJECTION_CAPACITY_BENCH_V1` Release marker.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the previous
collectors and GREEN after the bounded direct-append shape was added. A dense
four-component vector channel changes the modeled outer collector growth from
`1` event to `0`; explicit key reservation also makes the per-channel bound
visible without claiming an allocator or CPU result for the standard library's
iterator collector. This is allocation-shape evidence only, not product
curve-editor p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_animation_curve_projection_capacity_performance_contract.py`
  (`4/4`).
- The combined Runtime/Editor focused batch (Runtime804/807/808/809 and
  Editor805-814) passes `61/61` in one process with zero failures, errors, or
  skips.
- The strict non-tooling performance/pressure batch (tooling, export, and
  coordinator paths excluded) loads `683` files and passes `2634/2634` tests
  in `31.255s`, with zero failures, errors, or skips.
- Exact-file Rustfmt, Python compilation, and scoped diff checks pass.
- The lower Rust component-order/invalid-input regression and ignored Release
  marker are wired in
  `zircon_editor/src/ui/animation_editor/session/curve_foundation.rs`.
- Managed Cargo/Release and animation curve allocation/product percentile
  evidence remain pending; tooling production remains deferred for the later
  Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/animation_editor/session/curve_foundation.rs` | `D2BBC14954713DB3959E252A5C9342D59E53D893806D4660F82342E246AF17C6` |
| `tools/tests/test_editor_animation_curve_projection_capacity_performance_contract.py` | `D6CEF8832741165396639B504A6E93741C7BB034FAE82AC854BAFAAA8094D41C` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending`
until the owner-attributed Windows Release batch compiles the current Editor
tree, runs the lower regression and ignored marker, and supplies animation
curve allocation and product p50/p95/p99 evidence. No coordinator status is
polled by this session.
