---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-07-18
summary_slug: viewport-picking-visible-spatial-query
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_runtime/render/04-visibility-culling.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_runtime/render/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/visible_spatial_query.rs
  - zircon_editor/src/scene/viewport/interaction_extract/cache.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidates.rs
  - zircon_editor/src/scene/viewport/pointer/precision/renderer_visible_spatial_pick_source.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_visible_spatial_query.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
---

# Viewport picking visible spatial query

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：viewport shared interaction extract 与 pointer renderable broad-phase 审查
- 来源执行会话：`editor05-viewport-shared-interaction-extract-r2-20260718`
- 修复责任计划：`docs/plans/zircon_runtime/render/04-visibility-culling.md`
- 交接原因：可见集、`FrameVisibility`、静态空间索引和增量 BVH update plan 属于 Render04；Editor05 不应复制 runtime graphics 的空间事实源。

## 失败现象与复现证据

Editor05 已把 pointer renderable source 从 `Scene::nodes()` 全扫硬切到同代际 runtime `SceneViewportRenderPacket.scene.meshes`，并让 render/pointer 共享 handle、gizmo 与 render-mesh extract。该 packet 只执行 active hierarchy、camera layer 与 LOD 过滤，不携带 Render04 的 frustum-visible entity set，也没有 cursor/ray/bounds query 合同。因此 pointer 每次 generation 变化仍会为 packet 内全部 mesh 生成 coarse candidates，无法证明 1/1k/10k 场景访问量由 query hits 主导。

Render04 已有最低共享层原语，但当前不可被 editor 正确消费：

- `VisibilityContext::frame_visibility` 与 `main_view_visible_entities()` 持有 per-view 可见结果；
- `VisibilityStaticIndex::query_bounds` 已支持 uniform-grid bounds 查询，但类型与访问器为 `pub(crate)`，且只索引 static mobility；
- renderer persistent `ViewportRecord` 持有上一帧索引，`SceneViewportRenderPacket` 不携带其 generation/snapshot；
- dynamic instances 仍走线性精筛，现有 API 没有一个 renderer-neutral、不会泄漏 authoring token 的 picking query surface。

## 最低共享层根因

Render04 只发布“渲染提交内部的可见性结果”，没有发布可由 editor/runtime tooling 消费的不可变空间查询快照。直接把 `VisibilityStaticIndex` 公开会泄漏 renderer 内部维护策略，也不能覆盖 dynamic entities；让 Editor05 重建 grid/BVH 则会产生第二事实源与不同代际失效合同。

## 架构修复验收

- 发布 renderer-neutral 的 immutable query snapshot/handle；身份必须绑定 world/render generation 与 view key，禁止无 generation 的永久缓存。
- query 至少支持 viewport cursor ray 或保守 world bounds/frustum，并返回稳定排序的 `EntityId` 集合及 `visited_node/candidate/hit` 计数。
- static 与 dynamic 实体均覆盖；static 复用 persistent index，dynamic 走增量结构或有明确预算，禁止每次 pointer move 全量线性扫描。
- render culling 与 editor picking 消费同一 `FrameVisibility`/bounds generation；不得让 Editor05 读取 `VisibilityContext` 私有字段、复制 `VisibilityStaticIndex` 或反向依赖 graphics implementation modules。
- 1、1k、10k node 深测中，稳定 query 的 candidate visits 随空间命中/相交 cell 主导，不随 total nodes 无条件线性增长；返回顺序与全量参考逐实体一致。
- camera/resize/layer/world mutation、static↔dynamic mobility 变化、增删移对象均正确失效；无 stale entity、无上一帧错误命中。
- 接入后 Editor05 的 handle/gizmo/renderable priority、projected depth、hover/press/release/scroll 与 debug feed 等价。

## 禁止临时方案

- 不得仅把 `VisibilityStaticIndex` 改成 `pub` 后让 editor 依赖 graphics 私有结构。
- 不得只过滤 `main_view_visible_entities()` 后仍在每次 cursor move 全量遍历全部 visible entities，并把它宣称为 spatial query。
- 不得以降低可拾取对象数量、关闭 debug feed、缩小 ring/mesh pick 半径来伪造性能改善。
- 不得用 physics raycast 代替没有 collider 的 renderable picking；physics 可作为命中精筛之一，不能成为 render visibility 的事实源。

## 修复结果与回传

Render04 now publishes an immutable renderer-neutral snapshot bound to source world, viewport,
frame generation and main view. It reuses COW static and dynamic visibility indexes, returns sorted
entities with visit/candidate/hit stats, and applies bounded-cell queries with a conservative COW
overflow set for extreme indexed bounds plus a visible-entry fallback for extreme finite queries.
The contract also supports normalized cursor-ray queries through bounded grid traversal, so pointer
selection need not represent a long ray as a scene-wide sphere. Editor frame extracts now bind their
world snapshot handle to `Scene::world_generation()`, allowing stale query identities to be rejected.
Editor05 now consumes the facade only after a successful viewport submission: it adopts only the
matching world generation, maps query-hit owners through an immutable interaction-generation owner table,
and combines those candidates with UI handle/gizmo candidates in the existing runtime picking
resolver. Camera, resize, extract/world mutation and unavailable facade states clear the dynamic
source; rectangle selection retains its independent extract-owned candidate table. Managed 1/1k/10k
validation and render evidence remain pending.

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-07-18 | Editor05 → Render04 visible spatial query failure handoff | 运行时源码已修复（open） | immutable generation-bound query snapshot、COW static/dynamic index、bounded bounds/ray query 与 source-world generation binding 均归 Render04；Editor05 不会复制 runtime 索引。 |
| 2026-08-01 | Render04 immutable visible spatial query source slice | 修复中（resolving_failure） | runtime contract、viewport generation snapshot、COW static/dynamic index reuse and bounded fallback are implemented; Editor05 consumer and managed validation remain open. |
| 2026-08-02 | Render04 cursor-ray query and world-generation binding | 修复中（resolving_failure） | `RenderSpatialRay` 由 framework contract 承载，static/dynamic grid 共用一次归一化的 bounded traversal，ray result 保留 stable sorted entities 与 visits/candidates/hits；未声明运行时验收。 |
| 2026-08-02 | Editor05 renderer facade query consumer | 源码修复完成（open） | successful submit 后经 `RenderFramework::query_visible_spatial_snapshot` 取回同代 snapshot；pointer event 仅为 ray-query hit owners 构造 renderable candidates，并与 handle/gizmo runtime resolver 合并。错代/不可用 snapshot 清空动态 source，受管 1/1k/10k 与截图证据仍待执行。 |
