---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: multi-primitive-visibility-key-collapse
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/04-visibility-culling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
tests:
  - multi-primitive visibility stable-instance-key preservation test
  - multi-primitive BVH/relevance/batch/history parity test
  - multi-primitive GPU-scene slot parity test
---

# Render04：multi-primitive visibility 以 entity 为键发生覆盖

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F2 scene render extract、visibility input 与 batching 静态审查
- 修复责任计划：`docs/plans/zircon_runtime/render/04-visibility-culling.md`
- 共同责任：`docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md`、`03-gpu-scene-gpu-driven.md`
- 交接原因：Scene 已发出唯一 primitive instance key；丢失发生在 visibility/BVH/batching 共享层，不能由 Scene 简单去重掩盖。

## 失败现象与复现证据

`render_mesh_snapshots_for_camera` 会为同一 entity 的每个 primitive 发出独立
`RenderMeshSnapshot`。这些 snapshot 共享 `node_id`，但通过
`render_mesh_stable_instance_key(entity, primitive_ordinal)` 获得唯一 `stable_instance_key`；
GPU scene 也已经按该 key 注册和更新实例。

`VisibilityRenderableInput` 仅保存 entity。`build_visibility_input` 因而为每个 primitive 生成同
entity 的重复 row，排序后还派生三份重复 entity Vec。`collect_batching_result` 随后分别把 meshes、
phase inputs 和 visibility rows 收集为以 entity 为键的 `HashMap`/`BTreeMap`，后写 primitive 覆盖
前写 primitive。前面完成的 snapshot、layer clone、排序和 map insert 成为无效工作；更严重的是
frustum bounds、material batch、phase relevance、BVH/history 最终只代表一个 primitive。

## 最低共享层根因

scene extract 已经区分 authoring entity 与 render instance，但 visibility DTO 和 planning key 仍把
两者混为一谈。该边界与已经使用 `stable_instance_key` 的 GPU scene 不一致。

## 架构修复验收

- `VisibilityRenderableInput`、frustum candidate、relevance、batch member、BVH/history entry 使用
  明确的 render-instance stable key；entity 作为 authoring owner 单独保留。
- geometry/phase/visibility 的关联不得经 `HashMap<EntityId, _>` 覆盖；可以使用稳定 instance key，
  或在 validated aligned arrays 上使用 mesh index，但必须有 generation/ordering 合同。
- 两个 primitive 使用不同 mesh/material/bounds 的产品测试必须得到两个 cull/relevance/batch/history
  项和两个 GPU-scene slot；隐藏其中一个不能隐藏另一个。
- 1/10/100 primitive per entity 的构建计数证明没有覆盖式重复 insert，整体分配与 CPU 时间近线性。

## 禁止临时方案

- 不得在 Scene 层仅按 entity dedup visibility rows；这会降低分配但永久丢掉 primitive 可见性。
- 不得继续用 entity key、再依赖“最后一个 primitive 恰好代表整 entity”的顺序偶然性。

## 修复结果与回传

Render04 direct-mesh path now carries `stable_instance_key` through pending draw creation, GPU
scene sync, visibility/BVH/history, batching cache identity and command sorting; sibling primitive
regression coverage is present. The virtual-geometry indirect branch is separately owned by
Render03 and remains open at `../03/failure-2026-08-01-virtual-geometry-stable-instance-key-collapse.md`.
Managed multi-primitive validation and the Render03 forward repair remain pending.
