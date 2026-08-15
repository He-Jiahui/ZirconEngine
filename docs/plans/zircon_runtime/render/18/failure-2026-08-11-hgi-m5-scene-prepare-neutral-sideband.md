---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-08-11
summary_slug: hgi-m5-scene-prepare-neutral-sideband
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_runtime/render/18
failure_scope: local
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/prepared_runtime_sidebands.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/runtime_prepare_collector.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_render_framework_stats/product_wgpu.rs
  - zircon_runtime/src/graphics/runtime_prepare_mesh_geometry_seed.rs
tests:
  - focused Rust neutral-sideband and projection contracts
  - coordinator-managed DX12 WGPU Global SDF atlas readback and PNG under docs/tests/runtime/render
  - coordinator-managed DX12 RenderFramework scene-prepare product PNG and RenderDoc capture
---

# Render18 M5: scene-prepare neutral sideband is disconnected from WGPU execution

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：M5 scene-prepare neutral sideband WGPU execution gate
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：故障位于 Render18 自有的 prepared-runtime sideband、HGI provider 与 collector
  映射边界；同一编号计划是最低共享层 owner。

## 失败现象与复现证据

At discovery, the only production call of `HybridGiGpuResources::execute_prepare` was
`runtime_prepare_collector.rs:324`. Its `scene_prepare` argument is explicitly `None` at line
333. `collect_inputs` therefore creates empty card-capture, surface-cache-page, voxel-clipmap,
and voxel-cell descriptor vectors for every runtime-prepare GPU dispatch.

The provider does build a `HybridGiScenePrepareFrame` at `provider.rs:99`, but it converts that
frame into synthetic `RenderPluginRendererOutputs` at lines 108-109. The neutral
`RenderHybridGiPreparedFrame` in `prepared_runtime_sidebands.rs:82-95` carries radiance-cache and
probe payloads only; it does not carry a scene-prepare payload. No second production
`execute_prepare` caller exists.

Consequently the card/voxel scene-preparation source is visible to CPU feedback bookkeeping but
does not reach `create_buffers`, `dispatch`, or `dispatch_probe_trace_tiles` in the WGPU runtime
prepare path. This must be repaired before tuning card bounds or claiming a rendered M5 scene.

## 当前源码修复状态

The current source now carries a renderer-neutral scene frame in
`RenderHybridGiPreparedFrame`, maps the provider frame into that DTO, and projects it in the
collector before passing `Some(&scene_prepare)` to the single `execute_prepare` call. Card owner
stable instance keys cross the same boundary. The collector overlays card-capture and
surface-cache-page bounds from the authoritative prepared-geometry world-bounds projection;
missing geometry omits those descriptors instead of retaining the transform-scale sphere.

The collector now rebuilds voxel-cell occupancy and dominant-card identity from the same exact
prepared-geometry AABB projection for every clipmap. It retains an existing radiance payload only
when it maps to the same exact `(clipmap_id, cell_index)`; unresolved geometry contributes no
card, page, or voxel-cell descriptor. Clipmap topology may remain so the GPU can retain its
bounded resource layout, but it is not populated with geometry-derived occupancy.

Provider production output is empty and actual scene resources only return through collector
readback. The obsolete CPU scene-prepare synthesis helper has been removed, so it cannot be
reconnected as a false GPU feedback fallback. These are source-level forward repairs; they do not
constitute WGPU scene correctness without the managed runtime evidence below.

## 最低共享层根因

Add an owned, renderer-neutral scene-prepare DTO to the core prepared-runtime sideband rather
than passing a plugin-private `HybridGiScenePrepareFrame` through `zircon_runtime`. It must contain
the existing card-capture requests, persisted surface-cache contents, voxel clipmaps, and voxel
cells with only neutral math and scalar types.

The DTO must also preserve a `stable_instance_key` for each card owner. The collector already
projects authoritative prepared geometry into sorted `(stable_instance_key, RenderMeshBounds)` data
from `RuntimePrepareMeshGeometrySeed`; that identity is required to replace the old
`max(abs(scale)) * 0.5` card/voxel approximation with exact transformed local bounds. `card_id`
alone is not sufficient because collision resolution deliberately makes it a presentation-local
identifier.

The plugin owns two one-way mappings:

1. The provider maps `HybridGiScenePrepareFrame` into the neutral DTO when it builds
   `RenderHybridGiPreparedFrame`.
2. The runtime-prepare collector maps the DTO back into the plugin execution input, overlays
   bounds from the authoritative prepared-geometry projection by `stable_instance_key`, and passes
   `Some(&scene_prepare)` to `execute_prepare`.

When a mesh has no prepared geometry seed, this path must emit an explicit typed unavailable
outcome and omit that object from card/voxel GPU descriptors. It must not synthesize a sphere from
transform scale, because that can create false surface-cache occupancy and lighting occlusion. The
existing world-trace voxel fallback remains selectable through its capability graph.

The provider must stop treating CPU-generated scene-prepare samples as if they were renderer
readback results. Actual scene-prepare resource feedback continues to arrive from the collector's
GPU readback and is applied through the existing completion path.

## 架构修复验收

- A source contract proves the collector obtains the neutral scene frame and passes
  `Some(&scene_prepare)` to its sole `execute_prepare` call.
- A non-uniform, rotated, off-centre local bounds regression proves the GPU descriptor uses
  `RuntimePrepareMeshGeometrySeed.local_bounds.transformed(mesh.transform)`, not transform-scale
  radius.
- A missing prepared resource regression proves no geometry-bound card, page, or voxel-cell
  descriptor is emitted for that instance and records the typed fallback reason.
- A scene with colliding node IDs proves the stable-instance-key mapping applies the correct bounds
  to both card owners.
- Coordinator validation captures the actual WGPU scene-prepare readback and writes a product PNG
  beneath `docs/tests/runtime/render`; the run also records a RenderDoc `.rdc` for the same source
  revision. This session has not produced either dynamic artifact.

## 禁止临时方案

- Do not make `HybridGiScenePrepareFrame` a public `zircon_runtime` dependency.
- Do not add a second GPU scene-preparation entry point or duplicate the collector's resources.
- Do not feed the provider's synthetic CPU readback records back into the runtime as GPU evidence.
- Do not retain the transform-scale sphere as a fallback for unresolved geometry.

## 修复结果与回传

- 根因：renderer-neutral prepared sideband 未承载 scene-prepare payload，唯一 WGPU
  `execute_prepare` 调用因此始终收到 `None`。
- 架构修复：核心 sideband 增加中性 scene frame；provider 单向投影该 DTO；collector 以
  stable-instance-key 覆盖精确 world AABB 后将它传给唯一 WGPU prepare dispatch。
- 验证：本 session 已完成 focused static contracts、路径/模块解析、格式和独立源码复审；
  未运行 Cargo、WGPU、RenderDoc 或性能命令。
- 回传：待 coordinator-managed Windows WGPU readback、产品 PNG、RenderDoc `.rdc` 与同源
  性能数据完成后，Render18 M5 动态验收 gate 才能恢复。

2026-08-13 second static review：0 Critical / 0 Important。复审逐项确认：实际 selected
Mesh primitive 的 dependency revision 与 conservative morph/skinning bounds 进入 prepared
geometry；missing/invalid/deforming/packing-overflow Mesh SDF 均保持 typed voxel fallback，不会
发布空 Global SDF page；page influence 扩张覆盖相邻页且 bounded candidate overflow 转 fallback；
GPU completion 携带 requested generation 并 compare-and-commit；probe trace ABI/diagnostics 输出
实际 source、distance、confidence、fallback reason 与 bounded cost counters。DXF cook settings、
外部 Mesh 引用、artifact v4、累计 cook budget、collector enqueue error propagation 和模块边界等
先前复审项也在当前树中闭环。

Open state: `source complete; dynamic acceptance pending`; no WGPU/product pass is claimed.

## 当前状态

Source complete, dynamic validation pending. The neutral sideband, card/page bounds, exact AABB
voxel-cell reconstruction, and removal of production synthetic feedback have source-level forward
repairs and focused static contracts. The
coordinator-managed WGPU readback, product PNG, RenderDoc capture, and performance evidence remain
required before this handoff can be returned as fixed. No Cargo, WGPU, RenderDoc, screenshot,
power, or performance command has been run by this session.

The dynamic evidence set is intentionally split by ownership. The ignored
`export_global_sdf_build_wgpu_png` test requests DX12, executes the production
`cs_build_global_sdf` compute pipeline, validates signed atlas samples and its completion word, and
only then writes the Global SDF slice PNG. The ignored
`export_hybrid_gi_m5_global_sdf_trace_wgpu_png` test likewise requires DX12 and writes the trace
result after building and publishing the same fixture page. The existing ignored RenderFramework
product exporter continues to validate the neutral scene-prepare sideband through the stateful
runtime collector to a captured final frame. None has run on the current source, and no individual
exporter replaces the required same-source product PNG, RenderDoc capture, or performance sample
set.

2026-08-14 forward correction and second source review: the voxel fallback no longer scans the
packed descriptor range or applies distance to a flattened cell id. Its frame-owned `8 x 64`
lookup maps declared clipmaps to canonical cell slots, rejects reserved IDs, duplicate clipmaps or
cells, undeclared cells, and over-capacity inputs, and disables only the voxel capability for an
invalid frame. Rust and WGSL now agree on the lookup-count uniform field and binding 11. The mixed
Global SDF/voxel fixture supplies the same fixed table rather than a test-only empty fallback;
the shader root is split from the voxel leaf so both owners remain below the 800-line review
threshold. Focused source guards, ABI-order comparison, rustfmt, and scoped diff checks pass; the
post-fix second review found no remaining Critical or Important issue in this route. This is
source evidence only: this failure remains open with dynamic acceptance pending.
