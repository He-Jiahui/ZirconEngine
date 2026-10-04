---
related_code:
  - zircon_editor/src/scene/viewport/interaction_extract
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
reference_sources:
  - dev/bevy/crates/bevy_picking/src/mesh_picking/mod.rs
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
tests:
  - tools.tests.test_editor05_viewport_interaction_extract_contract 5/5
  - current-source Windows Cargo and viewport product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor scene当前增量性能复核（2026-07-22）

## 覆盖与结论

旧证据覆盖126个文件；当前树新增`interaction_extract` 4个owner并硬删除2个重复owner，净增为 **128/128**。本轮复核共享cache/extract/key、render/pointer controller、render packet与projection调用链，没有重复宣称既有generation cache、single extract和shared projection工作。

changed-generation gizmo扫描原对每个scene node先调用`active_in_hierarchy`，再`find_node`，即使绝大多数node不是Camera/DirectionalLight。本轮先按`NodeKind`过滤，仅两类gizmo owner查询active hierarchy，并把循环已有`&SceneNode`传给builder，删除重复lookup。源码合同先RED后GREEN，Editor05 interaction-extract守卫 **5/5** 通过，rustfmt与scoped diff check通过。

## 保持open

`build_scene_gizmos`仍在generation变化时线性访问全部node；runtime render packet的`Vec<RenderMeshSnapshot>`又被完整复制进cache的`Arc<[...]>`，大场景会产生第二份mesh DTO bytes。PERF-MVP-222/Editor05/Render04必须改为runtime visible/BVH query与generation-owned shared mesh snapshot，不能把Camera/DirectionalLight过滤当作空间索引完成。1/1k/10k/100k nodes的visited/active-query/find/mesh-clone bytes、changed/stable move p95、current-source Cargo、F4产品hit与RenderDoc像素完成前不进入`review.md`。
