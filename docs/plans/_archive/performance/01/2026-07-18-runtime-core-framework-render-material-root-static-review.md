---
related_code:
  - zircon_runtime/src/core/framework/render/material/
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_pbr/src/material.rs
tests:
  - material root seventeen of seventeen Rust files reviewed
  - readiness report external test module one of one reviewed
  - resource streamer mesh draw and GPU uniform callers traced
  - layout membership temporary slot Vec and override field clone source guards RED to GREEN
  - rustfmt and scoped git diff check passed
  - material focused Cargo scale counters F2 trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render material root逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/material/*.rs`当前root 17/17个Rust文件、2,215行及`readiness_report/tests.rs` 1/1，并追踪resource streamer、mesh pending draw、command cache signature及GPU uniform创建链。三处低风险浪费已直接止损：material uniform layout membership由values×fields嵌套扫描改为一次名称索引，non-standard texture slot计数不再先克隆临时Vec，override field查找不再逐项深clone字段String。确认一个MVP产品热路径根因；`material/management/**`另见独立30/30证据，因此material目录当前48/48文件均已静态读完，但动态验收仍pending。

## PERF-MVP-359：稳定属性覆盖每帧重复payload clone/hash与GPU buffer创建

`extend_pending_draws_for_mesh_instance`对每个有property override的mesh entity调用`material_uniform_payload_with_overrides`。该入口每帧克隆基础payload的layout/bytes/unsupported并重放override；同一payload随后为direct/model/dynamic primitives多次clone。`material_uniform_override_signature`又对每个pending draw完整hash bytes和unsupported Strings，最终每个draw调用`GpuMaterialUniformResource::from_payload`，再次clone/pad bytes并执行`device.create_buffer_init`。因此稳定override也形成entities×primitives的CPU复制/哈希、GPU buffer分配和上传；command cache还因为override payload存在而放弃static state复用。

Render03/08/17应按`entity + material revision + override generation + layout hash`发布唯一prepared override uniform（payload signature、GPU buffer/bind group与diagnostics同owner），所有primitives/phase共享`Arc` handle；stable generation零encode/hash/create/upload，变化时只重写dirty range或从容量池复用buffer。override identity应进入static command cache generation，不以“有override”永久禁用缓存。Bevy把material binding与`Arc<MaterialProperties>`放入`PreparedMaterial`，在render asset prepare阶段生成并由draw按binding id复用；Zircon应采用同类prepared owner，而不是在draw构建期创建per-draw buffer。

## 其余root观察

`RenderMaterialReadinessReport::management_snapshot`会深cloneissue/prepared的多组Vec，resource streamer的management helpers还能先为全部material构造records与indices；当前尚未确认编辑器产品consumer，不能冒充已发生的帧热点。该调用族随`material/management/**`逐文件审查继续验证；一旦接入基础编辑器轮询，应复用generation snapshot/delta并回链Editor09/Render17。

## 验收要求

PERF-MVP-359按entities/materials/properties/primitives 1/1k/100k、stable/1% override changed、single/multi-camera记录payload encodes、bytes/layout/String clone、signature bytes hashed、GPU buffer creates/destroys、upload bytes、command-cache hits与CPU/GPU p95：stable generation encode/hash/create/upload=0；changed每唯一override generation encode/hash/create≤1；同entity全部primitive/camera共享相同prepared handle；buffer按capacity复用且只写dirty range。override类型/未知字段诊断、material hot reload、multi-primitive、像素、Cargo、F2与RenderDoc通过前，本切片留在`pending.md`。
