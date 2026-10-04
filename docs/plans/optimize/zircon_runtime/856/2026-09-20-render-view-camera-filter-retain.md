---
title: Runtime856 render-view camera filter retain
category: zircon_runtime
report_id: Runtime856-render-view-camera-filter-retain-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime856 - render-view camera filter retain

## Scope

`World::build_render_view_extract` already builds the complete scene-camera
descriptor vector with a component-count capacity bound, then filters it to the
selected camera plus active cameras. The old iterator `filter(...).collect()`
started a second vector from zero and copied every retained descriptor. This
slice reuses the descriptor vector in place with `retain`, preserving sorted
camera order and the selected-inactive-camera exception.

## Optimization

- Bind the descriptor vector once and retain the selected entity or active
  descriptors in place.
- Keep the no-scene-camera fallback, camera order report, descriptor override,
  and layer-union semantics unchanged.
- Remove only the intermediate filter buffer; camera construction and the
  existing component-count reservation remain the same.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the lower
owner existed (one missing lower-file error), then GREEN at `4/4`. The lower
Rust regression models selected-inactive retention, active-camera order, and
empty input, and carries the ignored
`RUNTIME856_RENDER_VIEW_CAMERA_FILTER_RETAIN_BENCH_V1` marker. An 8,192-camera
model changes the filter collector's modeled growth events from `13` to `0`.

## Local validation

- `tools/tests/test_runtime_render_view_camera_filter_retain_performance_contract.py`:
  `4/4` after the RED run.
- The focused Runtime/Editor V2 batch (Runtime853/854/855/856 plus the two
  V2 capacity contracts and Editor856) passes `26/26` tests in `0.009s`, with
  zero failures, errors, or skips.
- Exact-file `rustfmt --edition 2021 --check` passes for `render.rs` and the
  lower Rust owner; the current expanded non-tooling source-contract loader
  (performance-or-contract filename filter, tooling/export/coordinator
  excluded) passes `4072/4072` across `962` files in `139.499s`, with zero load
  errors, failures, errors, or skips.
- No managed Windows Cargo/Release validation command was started locally.
  Allocator observations and Runtime render-view product p50/p95/p99 evidence
  remain pending behind the shared external worktree gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/scene/world/render.rs` | `9F29A79BE362941189774CE35C0B7E1610FF058CA55E877EAD7708F93217F641` |
| `zircon_runtime/src/scene/world/render/view_camera_filter_tests.rs` | `463E29C5CCC4FEFC9D1DFDF1EC65B30DDC99E93C0B27F2C936D6D97D3D0EDDF2` |
| `tools/tests/test_runtime_render_view_camera_filter_retain_performance_contract.py` | `6905D7D4D512B247D455D0FCFDE15288E94AA1527D5D5612E04FCB5D9D631CEA` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Runtime tree,
executes the lower regression and ignored marker, and supplies allocator plus
render-view product p50/p95/p99 measurements. Tooling production remains
deferred for the later Rust migration; coordinator status is not polled here.
