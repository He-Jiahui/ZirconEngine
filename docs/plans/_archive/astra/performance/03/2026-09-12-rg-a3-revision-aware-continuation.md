---
title: RG-A3 revision-aware RenderScene geometry replay continuation
plan_id: astra-geometry-replay-20260905
parent_plan: astra-full-domain-20260905
owner: zircon_runtime::graphics::scene::resources::resource_streamer
session_id: astra-rg-a3-revision-continuation-20260912
status: implemented_pending_validation
---

## Scope

Close the bounded replay gap where a newer revision of the same mesh/model arrives between
256-primitive chunks. The resource event batch now carries `(UntypedResourceHandle, revision)`
pairs into the projector. A partial stable-key continuation is reused only when its resource
revision snapshot and resource scope still match; a revision change restarts from the first
dependent. The existing `commit(..., !selection.truncated())` gate keeps the newer invalidation
pending until every restarted chunk succeeds, preserving atomic failure semantics and the
per-frame bound.

## Evidence

- Regression source: `render_scene_resource_geometry_replay_restarts_continuation_for_new_revision`
  in `zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay.rs` models
  300 dependents, commits a truncated revision 1 chunk, injects revision 2, asserts the first
  256 target keys are selected again, asserts revision 2 remains pending after that truncated
  chunk, then selects the final 44 and clears the pending set. The test has not been run because
  the managed Runtime/Editor Cargo gate is held by the dirty external `E:\Git\zr_vm` checkout.
- `resource_streamer_residency.rs` now passes the drained revision pairs to
  `prepare_geometry_replay_with_revisions`; the source guard verifies that the old revision-
  discarding map is absent.
- Reopened same-frame dependency regression:
  `render_scene_resource_geometry_replay_keeps_same_frame_declared_and_supplemental_dependency_events`
  publishes a model and an undeclared child-mesh event before admission. It proves that the
  direct artifact set does not contain the child mesh, while the model-backed supplemental
  admission predicate retains both geometry events; a direct-mesh artifact does not enable the
  fallback. The residency source guard verifies classification occurs before the bounded drain,
  the predicate is limited to `Mesh | Model`, and `MAX_GEOMETRY_REPLAY_PRIMITIVES` remains 256.
  After the committed journal indexes the resolver's supplemental child mesh,
  `retain_dependencies` keeps both revisions for replay acknowledgement.
- `rustfmt --edition 2021 --check` passed for the four owned Rust paths below.
- Scoped `git diff --check` passed for tracked owned changes; a PowerShell trailing-whitespace
  scan reported zero lines in all four owned Rust paths. No Cargo build/test/check or native
  validation result is claimed.
- Base checkout head at session registration: `c37155ba304740b3762b20585f77fb53a6da47fb`.

## Changed paths and hashes

The two resource-streamer files were already untracked modularized source in the shared checkout;
the session layered the revision-aware bridge and regression on that foreign baseline. The
projector and component-projector test file also contained pre-existing foreign changes and were
not reset.

| Path | SHA-256 after this slice |
|---|---|
| `zircon_runtime/src/graphics/scene/render_scene/component_projector/projector.rs` | `8fec75bab7c25f6fb0187394673ffb8afe577afe2aa8a9a4c3ff634b47df352b` |
| `zircon_runtime/src/graphics/scene/render_scene/component_projector/tests.rs` | `19e1386c1006862d7db5d28c52a33bbf9554c6391281c16497c94ddbc058fafd` |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay.rs` | `a63369cbf6cede885cd04edbd3827d23e9158fb7e33dc41047ba454d1bc6a162` |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_residency.rs` | `c9da1edab1d4355f1c09e99bbcc25d2e2a49caadc6b4815b42de96ed37d9be2b` |

## Remaining managed validation

Keep this record at `implemented_pending_validation` until the coordinator can issue the focused
Runtime/Editor Cargo regression batch and the Windows Release paired workload. Those checks must
verify resolver calls and scene journals across revision replacement, failure retry, resync, and
the p50/p95/p99 performance budget. The current coordinator baseline is degraded by unrelated
maintenance/Cargo blockers; this child does not claim acceptance.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| RG-A3/W0 gap | Revision-aware bounded geometry replay continuation, same-frame declared/supplemental dependency retention, and pending acknowledgement gate | `implemented_pending_validation` | 2026-09-12 | Focused rustfmt, whitespace, and bounded-drain source guards pass; revision replacement plus same-frame model/child-mesh regression sources added; managed Cargo, resolver/journal execution, and Windows Release performance remain pending. |
