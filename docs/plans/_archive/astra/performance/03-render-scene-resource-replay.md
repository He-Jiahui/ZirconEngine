---
title: RenderScene resource geometry replay
plan_id: astra-geometry-replay-20260905
parent_plan: astra-full-domain-20260905
owner: zircon_runtime::graphics::scene::resources::resource_streamer
status: implemented_pending_validation
---

# RenderScene Resource Geometry Replay

## Boundary

This slice owns the RenderScene component projector and ResourceStreamer bridge only. The
RenderGraph state tracker and GPUScene draw synchronization remain outside the slice.
W4 product acceptance remains outside this slice; this plan does not claim completion of that
milestone.

## Architecture

Source evidence: Optimize223 (`223-runtime-render-graph-gpu-scene-render-scene-frame-submission-current-working-tree-review.md`)
identifies the retained RenderScene/projector ownership; Optimize224
(`224-runtime-resource-authority-asset-residency-current-working-tree-review.md`) establishes resource
authority and residency ownership. The exact replay admission is in
`resource_streamer/resource_streamer_residency.rs`; component geometry reuse is in
`render_scene/component_projector/projection.rs`; the bounded event receiver and gap contract are
in `zircon_runtime/crates/zr_resource/src/event_stream.rs`.

The component journal remains the admission cursor for world changes. Each registered RenderScene
world owns an independent ResourceManager event receiver for prepared mesh/model revisions, so one
world cannot consume another world's invalidations. ResourceStreamer drains that cursor at frame
admission and records only changed geometry resource identities. The projector keeps
a reverse index from each referenced mesh/model identity to the stable primitive keys that use it.
An exact world+journal replay therefore resolves only indexed dependents; unrelated resource events
produce no geometry work. When the component journal also advances, its component upserts and the
indexed geometry replacements are merged into one RenderScene delta so continuous unrelated world
edits cannot starve resource replay. A resource event with the same revision is a no-op.

When the bounded resource event log reports a gap, the bridge requests a bounded geometry resync of
the indexed scene. The resync uses the same resolver and transaction as a targeted replay, so a
pending/missing/invalid resolve fails before any primitive or cursor is committed. Resource
invalidations are consumed only after the corresponding geometry transaction succeeds; a failure
keeps the invalidation pending for the next frame.
During a gap resync, the receiver stays at the gap frontier until all selected chunks commit. Events
published during resync therefore remain unread and trigger a later targeted replay. Each bounded
selection uses a maintained `BTreeSet` of live primitive keys with a range cursor; the same selection
feeds resource preparation and scene application, without a repeated whole-scene sort or scan.
Event filtering checks the reverse index before retaining a revision. Successful component commits
prune departed dependencies, bounding retained revisions by live scene dependencies even when asset
imports/deletions continue or geometry resolution repeatedly fails.

Both targeted replay and gap recovery process at most 256 primitive keys per frame. A stable-key
continuation advances only after a successful chunk; pending resource revisions remain unapplied
until the final chunk commits. Mesh topology changes restart the selection, while unrelated
component changes preserve continuation progress. New revisions arriving during a replay remain
pending so the completed selection cannot acknowledge geometry it has not resolved.

## Transaction and failure rules

1. Ensure changed mesh/model prepared resources before resolution.
2. Build replacement primitives from the existing descriptor and newly resolved all-LOD geometry.
3. Merge any component-journal delta with the replacement primitives, stage resource-reference
   deltas once, then commit the RenderScene delta and mark the drained resource revisions applied.
4. On resolution or residency staging failure, preserve the old primitive, scene generation, reverse
   index, and pending invalidation set. The bounded receiver cursor advances while draining, but the
   pending set retains those events until a successful replay makes that advance durable.

The reverse index is updated from committed RenderScene journals and is never consulted for unrelated
material/texture events. Shared assets map to multiple primitive keys and are replayed once per
dependent primitive. A gap resync is capped by the existing scene replay budget so a malformed or
unbounded event stream cannot turn one frame into an unbounded scan.

## Verification

Focused tests cover unchanged journal plus changed mesh/model revisions, shared assets, unrelated
assets, same-revision no-op events, gap resync, and failure preserving state. The combined batch
also covers a resource change while an unrelated component journal advances. It should filter
`render_scene_resource_geometry_replay` and the existing
`resource_streamer_residency` tests. A production-entry regression now covers a disconnected
resource cursor, the next-frame receiver installation, and a mesh update observed after reconnect:
`render_scene_admission_reconnects_geometry_replay_and_processes_mesh_update`. The tests also cover
more than 256 dependents, failed chunk retry and repeated revisions. Cargo execution and release
performance measurements remain pending.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1/M2 | RG-A3 定向 geometry replay、资源事件游标、primitive 反向索引与有界 resync | implemented_pending_validation | 2026-09-09 | Runtime 资源重放静态契约 `16/16` 通过；事件 drain、256 primitive/frame 上限、gap/disconnect resync、失败保留 pending、依赖裁剪、共享资产与稳定目标顺序回归均已落源。`projector.rs` 移除 BTreeSet 结果上的冗余 `sort_unstable`，避免每个 bounded chunk 重复排序；focused rustfmt/diff-check 通过。受管 Runtime/Editor Cargo 与 paired release p50/p95/p99 仍因外部 `E:/Git/zr_vm` 脏快照（112 tracked/submodule + 88 untracked = 200）待执行。 |
