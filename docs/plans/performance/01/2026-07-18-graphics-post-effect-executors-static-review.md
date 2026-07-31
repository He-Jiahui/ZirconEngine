---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_bloom
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_color_lut_bake
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_depth_of_field_prepare
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_exposure
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_smaa
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_core_pipeline/src/fullscreen_material.rs
  - dev/bevy/crates/bevy_anti_alias/src/smaa/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/SubpixelMorphologicalAA.cpp
tests:
  - remaining post-effect executor slice thirty-one of thirty-one Rust files reviewed, 2166 current lines
  - clustered-lighting CPU clear and full-light upload source guard RED then GREEN
  - clustered disabled clear uses command encoder and active lights upload only the prefix
  - rustfmt and scoped git diff check passed
  - current-source focused Cargo, F2 counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics post-effect执行器逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`post_process/resources`中除已覆盖HZB与`execute_post_process/**`外的全部当前`execute_*` Rust文件31/31、2,166行，覆盖bloom、blur、cluster、LUT、DoF、exposure、FXAA、motion blur/vector、output transfer、scene composite、SSR、SMAA、SSAO与upscale。普遍存在per-pass bind-group/参数buffer创建、禁用效果clear pass和SMAA graph外纹理创建，编号PERF-MVP-370（P0）。

## 已直接止损：cluster关闭不再CPU分配整块零buffer

clustered lighting旧disabled分支每帧`vec![0_u8; cluster_buffer_bytes]`，随后从CPU上传整个light-list buffer；大小随视口cluster数量增长。该buffer已有`COPY_DST`合同（旧`queue.write_buffer`的前提），本轮改为`encoder.clear_buffer`，删除size参数与caller传递，CPU allocation/upload bytes归零。enabled路径原始终上传`MAX_DIRECTIONAL_LIGHTS`完整数组，本轮也改为count>0才上传有效灯前缀；shader继续以`ClusterParams.counts.x`限制读取。

源码守卫先得到CPU clear=1、GPU clear=0、active prefix=0，再确认GREEN为0/1/1，旧size签名与caller参数均为0。cluster buffer清零、灯顺序、count和compute dispatch不变。

## PERF-MVP-370：效果pass的GPU对象与禁用清屏放大

bloom、cluster、color LUT、DoF prepare、exposure histogram/resolve、motion-vector、SSAO、upscale等在每次执行中创建bind group；blur/DoF/motion blur/scene composite和四个SSR阶段还各自创建完整`PostProcessParams` buffer与29-entry bind group。FXAA/output-transfer每pass新建terminal-region buffer；SMAA每frame创建3个region buffers、3个bind groups，并直接`device.create_texture`两张全尺寸edge/blend texture及views。PERF-MVP-369的persistent parameter ring/binding bundle必须扩展到所有这些effect ABI，而不是只修最终composite。

SMAA中间纹理必须成为Render01可见的logical graph resources，由transient pool/materialization提供physical backing和view bundle；executor只消费已声明views。Unreal `AddSMAAPasses`把edge/blend/stencil都注册为RDG textures后再添加三pass；Bevy SMAA通过`TextureCache`持有`CachedTexture`，参数则放在`DynamicUniformBuffer`。两者都避免由每帧pass函数直接创建无图可见的全尺寸纹理。

bloom、DoF prepare、motion-vector tile/neighbor和SSAO disabled分支当前仍录制合计至少6个clear render passes。compiled post graph应在feature关闭时裁掉producer和孤立consumer，并以显式neutral fallback/import或合法alias满足剩余读取；executor disabled branch仅保留debug invariant，不应成为产品常态。需用pass/resource计数证明关闭效果不是“仍执行但清零”。

color LUT节点只要tonemap/grading/LUT任一启用就每帧dispatch 32³ bake；auto exposure又作为storage buffer读入LUT，使稳定grading仍随曝光每帧重烘。Render07应把dynamic exposure应用移出generation LUT bake，LUT只在grading/tonemap/operator/user-LUT generation变化时重建；否则至少以明确exposure generation缓存并记录不可避免的rebake，不能无计数地固定每帧执行。

SSAO pipeline已用`OnceLock`按device资源延迟创建，属于正确方向；其它executor的pipeline也在resource constructor持久化，本轮未发现per-frame shader/pipeline创建。

## 验收

按effects逐项off/on、AA none/FXAA/SMAA、views 1/8、1080p/4K、lights 0/1/max、stable/1% changed记录GPU buffer/texture/view/bind-group create+destroy、uniform uploads、cluster clear CPU bytes、pass/dispatch/attachment bytes、LUT bakes和CPU/GPU p95。当前cluster disabled CPU alloc/upload=0且active light bytes=N×element；最终warm stable effect buffer/texture/view/bind-group create=0，SMAA中间physical resources由pool复用，disabled effect clear passes=0，LUT bake≤1/relevant generation且auto exposure不强迫32³重烘。

受管focused Cargo、post/AA/cluster像素与计数矩阵、GPU timestamp、DX12 RenderDoc对象/pass及Render01 transient dump完成前，只保留在`pending.md`。
