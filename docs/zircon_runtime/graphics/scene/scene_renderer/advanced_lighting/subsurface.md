---
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/subsurface.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/material_control.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/subsurface_profile_extract.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/pipelines.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/shaders/setup.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/shaders/scatter.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/shaders/recombine.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_volumetric.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_subsurface.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_subsurface.wgsl
  - zircon_plugins/rendering/features/subsurface_scattering/runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/advanced_lighting/subsurface.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/material_control.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/subsurface_profile_extract.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/pipelines.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/shaders/setup.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/shaders/scatter.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/shaders/recombine.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_volumetric.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_subsurface.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_subsurface.wgsl
  - zircon_plugins/rendering/features/subsurface_scattering/runtime/src/lib.rs
plan_sources:
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessSubsurface.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/PostProcessSubsurface.usf
  - dev/UnrealEngine/Engine/Shaders/Private/SubsurfaceBurleyNormalized.ush
tests:
  - zircon_runtime/src/core/framework/render/advanced_lighting/subsurface.rs
  - zircon_runtime/src/asset/assets/material/material_control.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_subsurface.rs
  - zircon_plugins/rendering/features/subsurface_scattering/runtime/src/tests.rs
  - zircon_plugins/rendering/features/subsurface_scattering/editor/src/lib.rs
doc_type: module-detail
---

# Subsurface Scattering

## Purpose

Plan 18 的 SSS feature 为皮肤、蜡等不透明材质实现 screen-space normalized Burley diffusion。
它是默认关闭的可选插件，帧结构固定为 Deferred diffuse/retained 分量输出、8x8 tile 分类、GPU
indirect scatter 和 load/store recombine。

`SubsurfaceProfileData` 包含 profile id、RGB 毫米散射半径、RGB falloff 与毫米到世界单位的
换算。`resolve_subsurface_profile_table` 将 id 直接映射到 GPU 槽 0..15，允许稀疏槽位，通过
active mask 阻止空槽采样，并为重复或越界 id 产生诊断。`burley_radial_pdf` 是积分为一的 CPU
参考核。

## Material ABI

`rendering.subsurface_scattering` 注册 `custom:subsurface` shading model id 16。Forward include
复用 StandardPBR；Deferred 使用专用 G-buffer encoder。`subsurface_profile` 必须位于 0..15，
越界值进入材质 validation diagnostics，不再静默 clamp。材质可内嵌：

- `subsurface_scatter_radius = [r, g, b]`
- `subsurface_falloff = [r, g, b]`
- `subsurface_world_unit_scale = number`

提交阶段扫描当前 extract 中可见 mesh 的材质，收集实际使用的 profile id，并将内嵌 profile
并入 `AdvancedLightingExtract`；场景显式提供的同 id profile 优先。GPU material uniform 仍将 id
以 8-bit 编码写入 `data8.w`（byte offset 140），G-buffer normal alpha 保存同一个槽位 id。

## Graph Flow

descriptor 只有同时满足以下条件才进入图：DeferredGeometry 与 DeferredLighting 存在、当前视图
至少有一个 Subsurface 材质引用 active profile、graph sample count 为 1。否则三段 Pass 和全部
SSS transient resource 均不存在；MSAA 不会进入非法的单采样 compute binding。

1. SSS 激活时，`deferred-lighting` 通过 feature-owned MRT 扩展输出 `sss.diffuse` 与
   `sss.specular`。前者只含可散射 surface diffuse；后者保留 direct/environment specular、
   emissive 与 volumetric in-scattering。diffuse 在进入 Burley 前已乘体积透射率。
2. `sss.setup` 只解码 id 16 与 active profile mask，按 8x8 tile compact 到
   `sss.tile-list`，并在 `sss.indirect-args` 写 GPU 生成的 X group count。
3. `sss.scatter` 使用 `dispatch_workgroups_indirect`。RGB 每个通道各执行固定 64 个 stratified
   Burley sample，以当前 inverse view-projection 重建世界位置并计算 world-units-per-pixel；跨
   shading model、profile、normal 或重建深度不连续的 sample 被拒绝。`falloff_rgb` 作为每通道
   散射混合权重，通过 `mix(center_diffuse, scattered, falloff)` 保留未散射部分，不再把完整
   diffuse 直接乘 falloff 而造成统一能量损失。
4. `sss.recombine` 对非 SSS 像素 discard，对 id 16 写回 `scattered + retained`，随后进入透明段。

三张中间纹理使用 `Rgba16Float`。tile list 每个 8x8 tile 占 8 bytes，indirect buffer 为
16 bytes 且带 STORAGE、INDIRECT、COPY_DST。执行审计记录真实 indirect dispatch，但不伪造由
GPU 生成且未回读的 group count。

## Fallback And Diagnostics

SSS V1 只支持 Deferred 单采样。Forward 和 MSAA 使用 StandardPBR fallback；Forward resolver
返回显式诊断。插件未注册、没有 profile、profile 未被当前视图材质使用、或 feature 被关闭时，
compiled graph 与基线一致。

## Verification

单元测试覆盖 Burley 归一化、16 槽上限、稀疏/重复 id、材质越界诊断、视图使用集合、可选 MRT
扩展、GPU indirect workload audit、插件注册、Deferred/MSAA/Forward gate 与 feature-off 图一致性。

`render_product_subsurface.rs` 在内存中生成 UV sphere 与内嵌 profile 的 skin material。基线帧
不注册 id 16；正向帧通过正常提交路径自动抽取 profile。产品门除 Pass 名和像素差异外，还要求
同时存在亮化/暗化像素、可测量的 red-dominant gain，以及零 workload mismatch；统一降亮、错误
falloff 或只插入 Pass 名不能通过。

严格修复后的 WGPU 产品门记录 `27,821` 个变化像素、`581/657` 个亮化/暗化像素以及 `2,047`
个 red-dominant gain 像素。当前 PNG 为
`docs/tests/runtime/render/plan18_sss_skin_sphere_deferred_burley_wgpu_20260712.png`，SHA-256
`11A791EE707BBC7580914135B20409B2060E121204E31CB7716949178AABC344`。

早期 48,188 changed-pixel PNG 只保留为历史执行证据。严格审查证明该版本缺少生产 profile
来源、稀疏 ABI、真实 diffuse split、MSAA gate 与有效产品断言，因此不能继续作为完成证据；
该历史结论已由上述 current-source PNG、报告与哈希取代。

## Limits

- profile id 是直接 GPU 槽位，只允许 0..15；稀疏槽位合法，inactive 槽不可采样。
- 薄几何背面 transmission 与 dual-lobe skin specular 不属于当前 screen-space pass。
- 三段 Pass 依赖 compute、storage texture、单采样 attachment 和 Deferred；Forward/MSAA 保持
  StandardPBR fallback。
