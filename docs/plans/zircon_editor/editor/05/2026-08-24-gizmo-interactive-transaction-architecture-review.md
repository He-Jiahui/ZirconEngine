---
record_kind: architecture_and_performance_review
status: open
created_at: 2026-08-24
owner_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
consumer_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
related_failure: failure-2026-08-19-gizmo-world-space-interactive-transaction.md
---

# Gizmo interactive transaction architecture and performance review

## Decision

The current viewport handle code must not be changed by substituting world transforms
for local transforms. It needs one Editor03-owned interactive batch transaction before
Editor05 can consume world-space deltas. This keeps the authority chain aligned with
Unreal's mode -> tool -> transaction model: the controller routes input, the active
tool derives deltas, and the transaction owner alone previews, commits, or rolls back
document changes.

## Current evidence

- `TransformHandleDragSession` stores one `node_id` and one local `initial_transform`.
- `ViewportTransformPreview` stores one `{ node_id, transform }` pair, so it cannot
  carry root-filtered multi-selection state or per-target rollback data.
- Existing handle selection helpers read `SceneNode.transform`, while runtime exposes
  `world_matrix`, `world_transform`, and `parent_of` for consumers that need world
  coordinates.
- Runtime persists TRS, not a general affine matrix. For a non-uniform or negative
  scale parent, `parent_inverse * desired_world` can contain shear and cannot be
  silently projected through TRS decomposition.

The preceding facts establish a structural correctness gap. They do not establish
runtime timing, allocation, power, or frame-time results.

## Required ownership model

`EditorTransactionEngine` must expose a single `InteractiveEditSession` (name may
change, responsibility may not) with this immutable begin snapshot:

| Frozen input | Owner and reason |
|---|---|
| document and world generation | Editor03 rejects stale commit or preview |
| tool kind, axis or plane, space, snap, pivot | Editor05 supplies semantics; Editor03 preserves them for one gesture |
| root-filtered selection | Editor03 deduplicates selected descendants before any write |
| local transform, world matrix, parent inverse per root | Editor03 owns before/after state and complete rollback |

The active HandleTool submits only a world-space delta to that session. The session
derives each desired world matrix, writes local TRS through the frozen parent inverse,
checks finite values and recomposition residual, and emits one root-keyed preview.
It either commits one batch command or restores every captured root. Capture loss,
cancel, rejected target, and conversion failure use the same rollback path.

## Math and error contract

For each affected root, the target local matrix is:

```text
local_target = parent_world_inverse * world_delta_about_pivot * frozen_world
```

After decomposition, the owner recomposes a TRS matrix and compares it with
`local_target`. A non-finite matrix, unavailable parent inverse, or residual beyond
the explicit transform-contract tolerance returns
`NonRepresentableTransform` and rolls back the complete session. This is required
until the runtime transform contract supports an affine representation; there is no
identity-inverse fallback or partial target update.

## Complexity and profile plan

The root filter may inspect the selected roots and their ancestor chains. Session
begin, update, preview, cancel, and commit must not enumerate `Scene::nodes()` or
clone a complete scene. For `k` selected roots, steady-state operations are `O(k)`;
the only hierarchy term is root filtering at begin.

Before and after implementation, the Windows-native managed profile must record the
following matrix in a report under this child plan:

| Scenario | Required measurements |
|---|---|
| 10,000 selected roots | begin/update/commit p50 and p95, allocation count, peak resident memory |
| deep parent hierarchy | root-filter cost and per-update work, proving no full-scene scan |
| 100 pointer updates then commit | one history entry and frame-budget impact |
| capture loss or typed transform rejection | rollback count and retained-session memory |

No profile command was run for this review: the managed Cargo dependency cache was
previously unavailable, and no valid current-source profiling receipt exists. The
implementation must not claim an optimization, power comparison, or bottleneck
elimination until that evidence is attached.

## Implementation boundary

The source owners for the transaction engine, viewport controller, feedback, and
workbench state are concurrently modified in the shared worktree. This review makes
no source edit in those paths. The next code change must begin at the Editor03
interactive-session boundary, then migrate the Editor05 DTO and tools as consumers;
adding a parallel Gizmo transaction in viewport or workbench code is explicitly out
of scope.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与后续 |
|---|---|---|---|
| 2026-08-24 | `open / architecture-and-performance-review-complete` | 完成当前 Gizmo 会话、preview、世界矩阵查询和 Editor03 transaction owner 的调用边界复核；确定批量交互事务、TRS 残差拒绝和 `O(k)` 约束。 | 静态源码与本地 Unreal/Fyrox 参考实现复核。未运行性能剖析，未修改并行变更中的 source owner；后续从 Editor03 interactive session 开始实现。 |
