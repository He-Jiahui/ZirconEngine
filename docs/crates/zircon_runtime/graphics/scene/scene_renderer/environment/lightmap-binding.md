---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/asset/assets/texture/lightmap_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_lightmap.wgsl
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/asset/assets/texture/lightmap_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_lightmap.wgsl
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/render/11/failure-2026-07-13-lightmap-forward-bind-group-integration-compile.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs
  - cargo test -p zircon_runtime --lib --locked --no-run
doc_type: module-detail
---

# Lightmap 与 Forward Bind Group 接线

## 职责与边界

`SceneLightmapResources` 是场景 light probe grid、lightmap atlas 和过滤采样器的唯一 GPU owner。它持有 fallback atlas texture，并通过 `LightmapGpuBindings` 复制资源的 `Arc` owner，向 Forward+ 与 Deferred 暴露同一组 bindings 23/24/28。binding 25/26/27 由 volumetric apply 使用，因此 lightmap sampler 固定在 28，跨 owner 测试禁止重叠。

该模块不拥有 lightmap 烘焙或已退役的全屏 baked-lighting 路径。Render 11 的生产边界保持为：场景抽取提供消费合同，资产层把 bake output 转为 raw RGBA16F array，ResourceStreamer 上传资源，`SceneLightmapResources` 校验并切换 GPU owner，mesh/deferred pipeline 只消费 bindings。

## 构造合同

fallback atlas 是 `Rgba16Float` 1x1 纹理，创建后必须通过 `wgpu::Queue::write_texture` 初始化。因此 `MeshPipelineCache::new` 的唯一构造 API 显式接收 `&wgpu::Queue`，并把它传给 `SceneLightmapResources::new`。`SceneRendererCore` 和所有 WGPU mesh-cache 测试调用点使用同一签名；不提供无 queue 的第二构造器，也不静默创建未初始化 atlas。

## Bind Group 生命周期

`SceneLightmapResources::bindings()` 返回拥有 buffer、view 和 sampler `Arc` 的 `LightmapGpuBindings`。`create_forward_shadow_receiver_bind_group` 必须先把该值绑定到局部 owner，再从它生成 `wgpu::BindGroupEntry`；局部 owner 的生命周期覆盖 `device.create_bind_group(...)`。禁止从临时 `self.lightmaps.bindings()` 直接借用 entries，因为语句结束后临时值会被销毁。

固定 ABI 为：

| Binding | 资源 |
|---|---|
| 23 | read-only light probe grid storage buffer |
| 24 | filterable `texture_2d_array<f32>` lightmap atlas |
| 28 | filtering sampler；25/26/27 为 volumetric apply |

Forward receiver layout、Forward bind group 和 Deferred resources 都从同一 lightmap binding owner 取得这三个槽位，不能删除槽位规避编译，也不能恢复第二套 baked-lighting 真相源。

## 验证状态

2026-07-13 的最低层修复已完成调用点迁移：一个生产调用点和六个 mesh pipeline WGPU 测试调用点都显式传递 queue；Forward bind group 保留持久 `LightmapGpuBindings` 局部 owner。随后修复了 sampler 25 与 volumetric params 25 的冲突，固定为 28 并增加不重叠断言。

当前 raw RGBA16F bake asset 转换、upload readiness、lightmap GPU ABI、GPUScene stable slot、Forward/Deferred shader 与真实 WGPU 产品门均已通过。GPUScene 按 stable instance key 写入 UV rect/page/generation，`zr_lightmap.wgsl` 由 Forward+ 与 Deferred geometry 共用，Deferred 通过现有 HDR emissive MRT 搬运逐实例 baked indirect。原 queue/lifetime failure 已回传 fixed。

单页 lightmap atlas 也必须向环境绑定公开 `D2Array` view；资源内部由 `lightmap_page_zero_bind_group_view_descriptor` 另建 page-zero `D2` view，只满足当前通用纹理 bind group，不暴露第二套 atlas 语义。当前二者回归 2/2、真实 GBuffer WGPU 1/1、外部 fixture 1/1、lightmap 聚焦 20/20、baked-indirect 2/2、产品导出 1/1。三联图位于 `docs/tests/runtime/render/plan11_lightmap_probe_forward_deferred_wgpu_20260713.png`，Forward/Deferred MAE 为 `0.0214`，最大通道误差为 `1`。EL-M3 已完成；后续由 HGI-M4 消费 generation 与 baked baseline。
