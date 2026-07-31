---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: viewport-pointer-candidate-regeneration
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_sync.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidates.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/projected_ring_segments.rs
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
---

# Viewport pointer candidate regeneration

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/scene`当前源126/126 Rust文件
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：scene selection、gizmo、camera与picking generation属于Editor05，不应由pointer move临时重建第二份场景投影。

## 失败现象与复现证据

原实现每个稳定`PointerMoved`先全扫scene renderables、重建scene gizmos/handles与48段ring projection，再用完整layout相等发现没有变化；dispatch随后为debug feed再次hit-test/评分。性能计划已直接加入world/selection/settings/camera/viewport generation键、lazy handle closure、route/debug单pass和ring临时Vec删除，使稳定hover零候选重建、每事件单次评分。

## 最低共享层根因

generation变化时仍无共享camera projection context、runtime可见候选/空间索引与跨render/pointer复用的gizmo extract。`projected_point`为每个点重复构造projection×view矩阵；pointer与render snapshot分别调用`build_scene_gizmos`；changed scene仍按total nodes线性扫描。

## 架构修复验收

- world/camera/settings generation change最多构建一次camera view-projection context；per-point matrix build=0。
- render与pointer消费同generation的共享gizmo/candidate extract，不各自全场景扫描。
- 候选来自runtime visible set、BVH或等价空间索引；1/1k/10k nodes的精确测试访问由query hits主导，不随total nodes无条件线性增长。
- 1k stable moves保持candidate/handle/gizmo/renderable/projection/surface rebuild=0；changed move记录scan、matrix、trig、alloc和CPU p95。
- handle/gizmo/renderable priority、projected depth、selection、hover/press/release/scroll、camera/resize、debug feed和像素/命中等价。

## 禁止临时方案

- 不得使用无法由world mutation失效的永久cache。
- 不得通过减少可拾取节点、降低ring精度或关闭debug feed伪造性能改善。
- 不得让render与pointer各维护一份无generation合同的gizmo事实源。

## 修复结果与回传

Open state: `shared projection context与render/pointer同代际single interaction extract已落地；renderable已硬切到runtime camera/layer/active-state-filtered RenderMeshSnapshot并删除editor Scene::nodes()扫描。仍待空间/BVH或pick-id broad phase、1/1k/10k query-hit访问计数、changed-move p95与受管Cargo/独立复审，故failure不回传fixed`。

runtime最低共享层缺口已正式移交 Render04：[`../../../zircon_runtime/render/04/failure-2026-07-18-viewport-picking-visible-spatial-query.md`](../../../zircon_runtime/render/04/failure-2026-07-18-viewport-picking-visible-spatial-query.md)。Editor05 不会复制 `VisibilityStaticIndex` 或从 graphics 私有字段旁路取数；待 Render04 返回同 generation 的 renderer-neutral query snapshot 后再接 cursor broad phase。

2026-07-22 current-source增量：当前树为128/128静态覆盖。本轮用RED→GREEN源码合同让gizmo scan先按`NodeKind`过滤，仅Camera/DirectionalLight调用`active_in_hierarchy`，builder复用循环中的`&SceneNode`而不再`find_node`；Editor05 interaction-extract合同5/5通过。该止损只删除非gizmo active查询和重复lookup；total-node线性scan、render packet meshes→cache Arc整slice复制、空间query、Cargo、规模counter与F4/RenderDoc仍open。
