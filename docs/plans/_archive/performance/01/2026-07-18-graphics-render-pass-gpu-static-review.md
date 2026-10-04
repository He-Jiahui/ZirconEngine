---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_command_lists.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/oit.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_sources:
  - dev/bevy/crates/bevy_core_pipeline/src/oit/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
tests:
  - GPU render-pass remaining leaf slice eight of eight Rust files reviewed, 1698 current lines
  - disabled forward volumetric parameter buffer source guard RED then GREEN
  - rustfmt and scoped git diff check passed
  - managed materialization Cargo reservation pending in FIFO; no test result claimed
  - current-source focused Cargo, F2 counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics GPU render-pass执行逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`render_pass_execution_context/gpu`中尚未覆盖的deferred、HZB occlusion、mesh command list/recording、OIT、particle、surface与tests当前8/8个Rust文件、1,698行；此前已覆盖的post-process、reports和resource lookup不在本计数内。借用的mesh streams、compiled resolver查询、particle/surface薄转发没有新增独立算法热点；确定热点集中在forward binding GPU对象生命周期、OIT每帧prepare以及diagnostics owned payload，编号PERF-MVP-368，并联动既有PERF-MVP-337/343。

## 已直接止损：恒定禁用参数缓冲不再按pass创建

`create_forward_shadow_receiver_bind_group`被GBuffer、depth prepass、TAA reactive mask、shadow atlas和velocity等路径调用。旧实现每次先用同一16-byte参数内容创建一个WGPU uniform buffer，再创建bind group；即使场景、视口与binding资源均未变化也重复产生driver对象。该参数只表达“volumetric disabled”，不依赖frame，因此本轮把它提升为`MeshPipelineCache`构造期唯一`forward_volumetric_disabled_params_buffer`，各pass仅借用。

源码守卫先确认旧状态cache field/constructor/binding reuse均为false且per-pass create为true，再确认GREEN为true/true/true且per-pass create为false；公开shader binding、bind group layout和pass顺序不变。该修复只消除了恒定buffer创建；bind group本身以及enabled forward参数仍按pass创建，继续由PERF-MVP-368负责。

## PERF-MVP-368：forward/OIT稳定帧仍创建GPU对象

`create_forward_receiver_bind_group_with_volumetric`每次用临时entries `Vec`聚合shadow/light-grid/reflection/lightmap/volumetric/transmission/cookie/irradiance bindings并调用`device.create_bind_group`。`create_forward_shading_bind_group`还为每次opaque/transparent/OIT pass新建参数buffer。正确owner应是Render02的forward binding bundle：以明确的resource-generation key描述这些可替换binding，固定禁用参数由cache持有，动态参数写入持久dynamic-uniform/ring；相同camera/resource generation的pass共享bind group与offset，资源替换只失效对应bundle。不得用裸地址或无generation的跨帧cache留下stale WGPU resource。

OIT路径每帧另创建settings uniform、fragment-store bind group，并再次调用forward shading binding；每个透明sprite还重新构建CPU vertices、texture bind group和独立vertex buffer。Render18应把OIT settings/heads/layers/counts binding放入持久per-device/per-view owner，动态参数打包上传；Render14应让OIT消费PERF-MVP-337的统一prepared sprite batches、image bind-group cache和持久instance/vertex arena，而不是第二次调用`build_sprite_vertices`。

Bevy OIT的`OitBuffers`长期持有`DynamicUniformBuffer<OrderIndependentTransparencySettings>`及容量型GPU buffers，prepare阶段只扩容并批量写camera offsets；Bevy sprite renderer以`RawBufferVec`持久化index/instance storage，并用`ImageBindGroups`按image缓存bind group。这些结构可作为生命周期参照，但Zircon必须保留自己的render-graph resource generation和transparent ordering合同。

`hzb_occlusion.rs`每次诊断记录为四个输出名分配`Vec<String>`，继续归PERF-MVP-343的diagnostics-off零owned-detail合同；`mesh_command_lists.rs`用借用slice/固定Option数组，transmission range为O(1)，无需另建问题。

## 验收

按forward/deferred、shadow、velocity、TAA reactive、OIT off/on，cameras 1/8、passes 1/8/32、sprites 0/1/100/10k、stable/1% resource changed记录uniform-buffer/bind-group create与destroy、entry Vec alloc、uniform upload calls/bytes、sprite vertex builds、texture bind-group creates、vertex-buffer creates和CPU record p95。当前止损要求cache构造后disabled params buffer create/frame=0；最终warm stable generation要求forward/OIT uniform-buffer与bind-group create=0、entry heap growth=0，同一resource tuple bundle build≤1/generation，动态参数至多一次packed upload/camera frame。OIT sprite额外CPU vertex build=0，且与普通transparent phase共享prepared artifact。

受管focused Cargo、F2产品counter、single/MSAA/forward/deferred/OIT像素对拍、GPU timestamp及DX12 RenderDoc对象/pass核对完成前，本模块只保留在`pending.md`，不得进入`review.md`。
