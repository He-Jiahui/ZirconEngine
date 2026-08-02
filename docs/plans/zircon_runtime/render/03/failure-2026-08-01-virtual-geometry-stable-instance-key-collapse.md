---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: virtual-geometry-stable-instance-key-collapse
origin_plan: docs/plans/zircon_runtime/render/04-visibility-culling.md
fixing_plan: docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
origin_child_dir: docs/plans/zircon_runtime/render/04
fixing_child_dir: docs/plans/zircon_runtime/render/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_virtual_geometry_draw_segment.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_indirect.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_snapshot_rebuild.rs
tests:
  - automatic virtual-geometry extraction retains original primitive ordinals after non-VG filtering
  - same-entity virtual-geometry visibility selection by stable instance key
  - same-entity virtual-geometry pending draws expand only their own stable-key segments
  - same-entity virtual-geometry execution statistics preserve distinct stable-key segments
  - legacy virtual-geometry key falls back only to the entity's primitive zero key
  - same-entity virtual-geometry execution-snapshot reconstruction by stable instance key
---

# Render03: virtual-geometry segment expansion collapses sibling primitives

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/04-visibility-culling.md`
- 来源执行切片：Render04 multi-primitive visibility failure forward repair second review
- 修复责任计划：`docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md`
- 交接原因：virtual-geometry extract, segment identity and GPU-driven indirect expansion own the missing render-instance identity; Render04 must not add an entity-keyed call-site filter.

## 失败现象与复现证据

Scene extraction emits a unique `stable_instance_key` for each primitive. Virtual-geometry cluster,
instance and visibility segment contracts retain only `EntityId`; `virtual_geometry_indirect.rs`
groups execution segments by entity and expands every matching segment into every pending draw for
that entity. Two primitives owned by one entity therefore cross-submit segments and duplicate
indirect work.

## 最低共享层根因

The Render03 virtual-geometry DTO boundary models authoring ownership but not render-instance
identity, unlike the mesh GPU-scene boundary. Entity-keyed grouping is therefore structurally
unable to distinguish sibling primitives.

## 架构修复验收

- Carry `stable_instance_key` from virtual-geometry extract through visibility draw segments and execution segments.
- Group and expand indirect segments by stable render-instance key; retain `EntityId` only as authoring ownership metadata.
- Add a same-entity/two-primitive regression where each primitive receives only its own segment and indirect draw count.

## 禁止临时方案

- Do not filter or deduplicate entity-keyed segment lists at the Render04 call site.
- Do not use primitive ordering as an implicit identity or duplicate a second mapping table.

## 修复结果与回传

Source repair is integrated forward: automatic extraction now emits the mesh-compatible key for
each model primitive; visibility plans and draw segments preserve that key; execution snapshots,
debug reconstruction, indirect expansion, and prepared-queue execution statistics use it as their
grouping identity. Legacy authored extracts without the field use the existing `(entity, primitive
0)` compatibility key.

## 二次审查

- 2026-08-01: 完成 P2 覆盖补齐后复审。原始 primitive ordinal 在 non-VG filter 后保持不变；生产 pending-draw expansion
  直接覆盖 same-entity sibling 与 legacy primitive-0 隔离。未发现可操作的 correctness、性能或模块边界问题。

Status remains `resolving_failure` until coordinator-managed Windows validation and the required
render screenshot/RDC evidence are accepted; no pass is claimed here.
