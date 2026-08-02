---
related_code:
  - zircon_runtime/src/graphics/extract/history.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/previous_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/joint_palette_storage.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/velocity_camera_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_camera.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/shaders/velocity_camera.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/asset/assets/material/material_control.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/taa_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_buffer_bundle/taa_resolve_params_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/taa_resolve_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/full_scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/profiled_scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/execute_taa_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/shaders/taa_resolve.wgsl
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/temporal_frame_index.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_reflection_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle_velocity.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_motion_vector_tile_max/mod.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/taa_quality.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VelocityRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalAA.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/MotionVectorRenderPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/MotionVectors.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/PostProcess/TemporalAntiAliasingPostProcessPass.cs
  - dev/bevy/crates/bevy_anti_alias/src/taa/mod.rs
  - dev/bevy/crates/bevy_anti_alias/src/taa/taa.wgsl
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/bevy/crates/bevy_core_pipeline/src/prepass/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/prepass/background_motion_vectors.wgsl
  - dev/bevy/crates/bevy_pbr/src/prepass/prepass.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/skin.rs
plan_sources:
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
---

# 计划 06:时域管线(velocity / jitter / TAA / history)

## 目标

补全时域链路并正面解决风险清单 P0(history ghosting):

1. 统一 velocity buffer:相机运动 + 动态对象运动全覆盖(读 GpuScene prev transform)。
2. 相机投影 jitter(Halton 序列)接入,所有上游 pass 感知 jitter,后处理前去 jitter。
3. TAA resolve:history 重投影 + neighborhood clamp + disocclusion 检测,history 资源语义定稿。
4. motion blur / DoF 等消费 velocity 的效果与 TAA 顺序定稿。

## 现状与差距

- motion vector 仅有相机部分(`execute_motion_vector_camera`)与 motion blur 用的 tile max;对象级 velocity 缺 prev transform 数据源。
- TP-M3-S1b..S1f 已新增真实 `taa_resolve.wgsl`、WGPU fullscreen resolve executor、`TemporalHistoryStore`、TAA 质量档、responsive/reactive suppression 与 offscreen 后端 TAA capability;TP-M4-S1 已删除 legacy `history_resolve` feature/effect/executor/resource 路径,并把公共 frame-history scene-color 槽位硬切为 `TaaSceneColor`;TP-M4-S2 已接入 `taa.reactive-mask` 图资源、默认清零 pass 与 TAA resolve shader 输入;TP-M4-S3 已接入透明 mesh reactive mask writer V1;TP-M4-S3b 已接入显式材质 authored strength;TP-M4-S3c 已接入 opaque/alpha-mask material flag writer;TP-M4-S4a 已新增静态空场景 TAA 多帧 product history 基线,并补齐 product 路径暴露的 WGPU sampled-texture limit 与 transient texture usage 能力映射;TP-M4-S4b 已新增动态遮挡 render_product 收敛基线,并补齐 mesh light-grid fallback、LightGridParams 128-byte ABI 与 DepthPrepass velocity-object 上下文注入;TP-M4-S4c 已把 authored material reactive-mask writer 计数暴露到 `RenderStats`/product diagnostics,并用真实材质资产的 WGPU submit 产品基线锁住 0 强度不写、1.0 强度写 1 条 command;TP-M4-S4d 已新增透明 Blend 材质 alpha writer 产品基线,显式对齐 `GeometryPhaseInput` 的 Blend phase 后锁住透明 draw 生成 1 条 reactive-mask command;TP-M4-S4e 已新增 TAA product 粒子透明 pass 基线,注册 `particle.transparent` 后锁住其在 `temporal.taa-resolve` 前执行,空帧/粒子帧均保持 TAA resolve,带 sprite 帧产生非零 RGBA delta 且 TAA-only 不计 particle velocity missing sprite;TP-M1-S9 已新增 `RenderParticlePreviousSpriteSnapshot` 与 `ParticleExtract.previous_sprites`,submit stats 改按 current sprite 减 matched previous-state 计算 particle velocity missing,并用 motion-blur/TAA 产品基线锁住 previous-state 帧 missing=0;TP-M1-S10..S23 已接入粒子 `scene-velocity` writer、stable sprite identity、renderer-owned previous rows、renderer-owned previous billboard basis、test-build velocity surface 像素读回、renderer-owned 二帧动态读回基线、same-entity nonzero-key multi-sprite 产品基线、key=0 anonymous stream ambiguity 诊断、keyed multi-sprite 三帧动态基线、32-sprite keyed stress field 产品基线、ambiguous key=0 hard enforcement、world-HUD producer nonzero-key 迁移,以及 CPU-morphed changing-shape previous source velocity。历史 pre-jitter hash artifact 已在 S21 审计为仓库内不可恢复的外部补证项;剩余缺口集中在 RenderDoc 验收。
- 相机契约已有 TP-M2-S1a/S1b 的 `temporal_jitter` 字段、Halton 序列、`ViewProjectionMatrixPair`、TAA 生效态 jitter 注入、scene uniform current jittered/current unjittered/previous unjittered ABI 与成功提交后的 temporal frame roll;TP-M2-S2 已完成上游投影矩阵消费审计;TP-M2-S3 已新增当前仓库可执行的 TAA/AA Off 产品对拍基线;TP-M3-S1a..S1f 已完成 TAA resolve graph/resource contract、最小 WGPU resolve/history flip、基础质量档/resolve 内核、TAA history store、responsive suppression 与 offscreen 后端 TAA 产品启用;TP-M4-S1 已完成旧 history 语义硬切;TP-M4-S2 已完成 reactive mask 图输入和默认零值路径;TP-M4-S3 已完成透明材质 alpha mask 写入路径;TP-M4-S3b 已完成材质显式强度到 reactive mask shader 的数据链路;TP-M4-S3c 已完成非透明材质强度到 dedicated reactive-material mask pipeline 的 CPU-gated 调度;TP-M4-S4a 用同一 viewport 连续 seed/history/history 产品提交锁住 TAA 多帧 history path 与空场景逐像素稳定性;TP-M4-S4b 用可见目标到遮挡目标的产品序列锁住动态遮挡收敛;TP-M4-S4c 用真实 `.zmaterial` authored strength 资产锁住 reactive mask writer 产品统计面;TP-M4-S4d 用真实 Blend `MaterialAsset` + Blend phase input 锁住透明 alpha writer 的产品统计面;TP-M4-S4e 用真实 particle feature/executor 产品提交锁住粒子透明 pass 顺序、可见贡献与 TAA-only velocity 诊断面;TP-M1-S9 用 previous-state 数据契约把粒子 velocity 诊断从“全部 sprite 缺口”收敛为“仅未匹配 previous-state 的 sprite 缺口”。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Renderer/Private/VelocityRendering.cpp` | velocity pass 的两种来源(base pass 输出 vs 独立 velocity pass)、哪些对象需要写 velocity(`needs_velocity` relevance) |
| `dev/UnrealEngine/.../Renderer/Private/PostProcess/TemporalAA.cpp` | TAA resolve 全套:history 重投影、YCoCg neighborhood clamp、disocclusion 权重、responsive AA 标记 |
| `dev/Graphics/.../Runtime/Passes/MotionVectorRenderPass.cs` + `MotionVectors.cs` | URP 的 per-object motion vector pass 组织与 prev 矩阵管理(`previousLocalToWorld`),数据面与本引擎规模匹配 |
| `dev/Graphics/.../Runtime/Passes/PostProcess/TemporalAntiAliasingPostProcessPass.cs` | TAA 与后处理链的插入位置、history RT 的分配与失效(分辨率/相机切换) |

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_anti_alias/src/taa/mod.rs` | jitter 注入与 `TemporalHistoryStore` | `prepare_taa_jitter`(Halton 序列写 `TemporalJitter`)、`prepare_taa_history_textures`/`TemporalAntiAliasHistoryTextures`(read/write 双缓冲)、`reset` 失效语义 |
| `dev/bevy/crates/bevy_anti_alias/src/taa/taa.wgsl` | `taa_resolve.wgsl` 全套算法 | `RGB_to_YCoCg`/`YCoCg_to_RGB`、`clip_towards_aabb_center`(AABB 中心线 clip)、history 置信度与 blend rate(`DEFAULT_HISTORY_BLEND_RATE`) |
| `dev/bevy/crates/bevy_render/src/camera.rs` | `ViewProjectionMatrixPair` 的 jitter 注入 | `TemporalJitter::jitter_projection`:像素偏移 → clip 空间投影矩阵修改的精确公式(含正交/透视分支) |
| `dev/bevy/crates/bevy_core_pipeline/src/prepass/mod.rs` | velocity prepass 组织与 prev 矩阵管理 | `MotionVectorPrepass` 组件、`PreviousViewData`/`PreviousViewUniforms`(上帧 unjittered 矩阵跨帧持有) |
| `dev/bevy/crates/bevy_core_pipeline/src/prepass/background_motion_vectors.wgsl` | `velocity_camera.wgsl` 相机重投影补底 | 全屏 pass:`previous_view.clip_from_world` 重投影差 `(curr - prev) * vec2(0.5, -0.5)` 的编码约定 |
| `dev/bevy/crates/bevy_pbr/src/prepass/prepass.wgsl` | `velocity_object.wgsl` 对象 velocity | 顶点双位置变换出 motion vector;`morph_prev_vertex`/`skin_prev_model` 即 prev 形变位置(衔接计划 08 `fetch_prev_position`) |
| `dev/bevy/crates/bevy_pbr/src/render/skin.rs` | prev skinning palette 双缓冲 | `SkinUniforms` 的 `current_buffer`/`prev_buffer` 双 buffer 滚动,即 `flip_skinned_prev_palettes` 的 Rust 同构 |

Fyrox 无 TAA/velocity/jitter 时域管线(仅 FXAA),bevy 是唯一 Rust/wgpu 同类参照;disocclusion 权重与 responsive AA 细节仍以 UE `TemporalAA.cpp` 为准,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:velocity 与 TAA 作为内建 RenderFeature 在 `scene_renderer/` 下新增 `temporal/` 模块;history 资源管理收口到计划 01 的持久资源;相机 jitter 进 `core/framework/render/camera.rs` 契约。

核心设计:

- prev transform:计划 03 GpuScene 的 primitive data 增加 prev transform 槽(帧末由本帧值滚动),骨骼对象另存 prev palette(衔接计划 08 GPU skinning)。
- velocity pass:深度 prepass 之后独立 pass,只绘制 `needs_velocity` relevance 的动态对象;静态部分由相机重投影在全屏 pass 补足(URP 做法)。输出 RG16F velocity。
- jitter:`ViewportCameraSnapshot` 增加 jitter 偏移(Halton 2,3 序列,周期 8/16 按质量档);投影矩阵加 jitter,velocity 计算用无 jitter 矩阵;TAA 关闭时 jitter 为零,upstream 无分支。
- `TaaResolveExecutor`:输入 scene color(post-lighting、pre-postprocess,语义定稿并重命名 history 资源)、velocity、depth、history;3x3 邻域 YCoCg AABB clamp;disocclusion 由 velocity 长度 + 深度差检测,失效像素回退当前帧;输出新 history(持久资源,分辨率/相机切换时失效重建)。
- 顺序定稿(与计划 07 共同遵守):velocity → (lighting/transparency) → TAA resolve → DoF → motion blur → 其余后处理。history ghosting 的 P0 风险在此关闭:history 只存 TAA 输入语义,后处理不再写回 history。

## 里程碑

### TP-M1 对象级 velocity 全覆盖

进度(2026-06-14):
- 已完成 TP-M1 第一段 GPUScene previous-transform 数据面(GPUScene previous-transform data surface):`graphics/scene/gpu_scene/prev_transform.rs`
  新增 `roll_prev_transforms_after_success()` 与 `previous_world_from_local(...)`。compiled-scene 与 legacy
  render_scene 两条提交成功路径在 `queue.submit(...)` 之后滚动当前 `GpuInstanceData.world_from_local` 到
  `prev_world_from_local`,并只在 previous 矩阵实际变化时把 instance span 标记为下一帧上传。`GpuSceneEntry`
  记录 `has_rolled_previous_transform`,所以新注册对象首帧不会伪造 velocity,下一帧才可被视为具备 previous。
- `mesh/build_mesh_draws/build.rs` 的 GPUScene 同步阶段已硬切为 GPUScene-only previous transform:
  不再读取 `ViewportMotionVectorObjectHistory`;unskinned 对象的 previous 矩阵只来自 GPUScene 已滚动的
  `previous_world_from_local(...)`,并将 `GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM`、motion params 与
  MeshDraw 的 `has_previous_velocity_transform` 一并置为有效。新对象首帧因
  `GpuSceneEntry.has_rolled_previous_transform == false` 不会伪造 velocity。
- 已完成 TP-M1 第二段 velocity 双 pass 迁移核心面:新增 `BuiltinRenderFeature::Temporal` 与
  `feature_descriptors/temporal.rs`,默认 forward+/deferred 管线插入 `velocity-object`/`velocity-camera`
  两个 DepthPrepass 阶段 pass;旧 `post.motion-vector-clear`/`post.motion-vector-camera`/
  `post.motion-vector-object` executor id 与 pass 声明已删除,相机执行代码与参数类型迁入
  `scene_renderer/temporal/velocity/*`;`SCENE_MOTION_VECTOR` 硬切换为 `SCENE_VELOCITY = "scene-velocity"`,
  scene velocity transient 格式改为 `Rg16Float`,tile-max/neighbor-max 链改读 `SCENE_VELOCITY`。
- 已完成 TP-M1 第三段 CPU object-history 硬切:`ViewportMotionVectorObjectHistory`、viewport record
  object-history 字段、`ViewportRenderFrame.previous_motion_vector_object_history` 与
  `update_motion_vector_history_after_success` 已删除;成功提交后只更新 temporal camera history。
  submit stats 与 runtime diagnostics 也删除 `render.post_process.motion_vector.object.*_history_count`,
  对象 velocity 可用性改由 mesh queue 的 previous/missing transform draw 计数表达。
- 已完成 TP-M1 第四段 GPUScene previous skinned-palette roll:`gpu_scene/prev_skinned_palette.rs`
  新增 renderer-owned `GpuSceneSkinnedJointPaletteState { signature, uniform }`,GPUScene 在 mesh draw 同步阶段
  暂存本帧 current palette,并在 successful submit 后通过 `roll_prev_skinned_palettes_after_success()` 滚入下一帧
  previous 面。`build_mesh_draws` 只在非 `CpuMorphed` shader-skinning source、signature 一致且 joint count 一致时把
  previous palette 传入 `MeshDraw`;CPU-morphed direct-mesh 源的稳定 morph-shape 覆盖由 TP-M1-S7 追加。
- 已完成 TP-M1 第五段 temporal velocity 命名/文件所有权硬切:`graph_execution/.../gpu/mesh_motion_vector.rs`
  物理迁入 `scene_renderer/temporal/velocity/execute_velocity_object.rs`,mesh pipeline/cache 硬切为
  `create_velocity_mesh_pipeline.rs`、`ensure_velocity_pipeline.rs` 与 `MeshPassPipelineKind::Velocity`;
  draw/batch/stats/diagnostic 的 previous/missing transform 计数改为 `*_velocity_*` 命名。相机 velocity shader
  迁入 `temporal/velocity/shaders/velocity_camera.wgsl`,对应 post-process resource bundle 内部的 camera bind layout
  与 pipeline 字段也改为 `velocity_camera_*`。motion blur 的 tile-max/neighbor-max 链仍保留 motion-vector 命名。
- 已完成 TP-M1 第六段 CPU-morphed previous-shape velocity 诊断策略:`prepared_queue.rs` 把
  CPU-morphed GPU-skinning source 的 previous-shape 缺口从普通 `missing_velocity_transform_draw_count`
  中拆出,写入 `skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count` 与
  `render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count`。在真实 previous morph-shape
  buffer 落地前,该类 draw 仍不声明 object velocity。
- 已完成 TP-M1 第七段 CPU-morphed stable morph-shape velocity 覆盖:`PendingSkinnedGpuSource::CpuMorphed`
  携带 `morph_shape_signature`,GPUScene previous skinned-palette 状态同步记录该签名。current/previous
  morph-shape signature 一致时,CPU-morphed GPU-skinning source 可复用当前 morphed vertex source 与 previous
  skinned palette 写 object velocity;signature 缺失或变化时仍走 TP-M1-S6 的专用 previous-shape 缺口计数。
- 已完成 TP-M1 第八段粒子 velocity 缺口诊断:当前 `ParticleRenderer` 只从
  `RenderParticleSpriteSnapshot { position, size, aspect_ratio, billboard_offset, rotation, color }`
  展开当前帧 billboard 顶点,extract 侧没有稳定 per-sprite id、previous position 或 previous billboard basis。S8
  不伪造粒子速度,而是在 reconstructed velocity 被 motion blur/SSR 请求且 `particle.transparent` 执行时,
  通过 `RenderStats.last_particle_velocity_missing_sprite_count` 与
  `render.particle.velocity.missing_sprite_count` 记录本帧缺少粒子 velocity 的 sprite 数。S9-S13 已补齐
  previous-state DTO、stable identity、renderer-owned previous rows、`scene-velocity` writer 与 previous
  billboard basis,因此该段现在是缺口诊断的历史基线。
- 已完成 TP-M1 第九段粒子 previous-state velocity 诊断收敛:`RenderParticlePreviousSpriteSnapshot`
  与 `ParticleExtract.previous_sprites` 成为 neutral previous-state DTO;`previous_state_sprite_count()` 按 entity
  逐个消费 previous-state rows,submit stats 同时携带 current sprite
  数与 matched previous-state sprite 数。motion blur/SSR 请求 reconstructed velocity 且 `particle.transparent`
  执行时,`last_particle_velocity_missing_sprite_count` 只统计未匹配 previous-state 的 current sprite。
- 已完成 TP-M1 第十段粒子 `scene-velocity` writer V1:`particle.velocity` built-in executor 与
  `ParticleRenderer::record_velocity(...)` 写 `SCENE_VELOCITY`,matched current/previous 粒子 billboard 通过
  `particle_velocity.wgsl` 输出 xy screen velocity,缺 previous-state 的 sprite 保持 no-op 且继续进入缺口统计。
- 已完成 TP-M1 第十一段粒子 stable sprite identity 匹配合约:`RenderParticleSpriteIdentity { entity,
  stable_sprite_key }` 成为 previous-state 统计与 velocity writer 共享匹配键;`RenderParticlePreviousSpriteSnapshot::from_current(...)`
  复制 key;scene JSON 当前帧 sprite 可读取 `stable_sprite_key`/`sprite_key`;key=0 保留单 sprite 旧匿名 stream,同 entity 多 key=0 已在 S22 视为不可追踪并报告 ambiguity。
- 已完成 TP-M1 第十二段 renderer-owned 粒子 previous-state roll:`ViewportRecord` 持有上一成功帧粒子
  previous rows;submit/present/direct runtime-frame 成功路径把 current `RenderParticleSpriteSnapshot` 滚动为
  `RenderParticlePreviousSpriteSnapshot`;`FrameSubmissionContext` 在输入 extract 没有显式 previous rows 时注入
  viewport-owned rows,并把同一 effective rows 送进 runtime frame、缺口统计与 `particle.velocity` writer。
- 已完成 TP-M1 第十三段 renderer-owned 粒子 previous billboard basis:`RenderParticlePreviousSpriteSnapshot`
  携带可选 `RenderParticleBillboardBasisSnapshot`;renderer-owned roll 把提交帧 camera right/up 写入 previous
  rows;`build_particle_velocity_vertices(...)` 展开 previous corners 时优先使用 stored previous basis,显式
  legacy previous rows 没有 basis 时才 fallback 到 current camera basis。
- 已完成 TP-M1 第十四段粒子 velocity surface 像素读回证据:`RenderSceneVelocityReadbackReport`
  记录 test-build `scene-velocity` 读回是否可用、目标尺寸、字节数与非零像素数;compiled-scene 渲染在
  graph command submit 后、transient resource pool 释放前读取 graph-owned `Rg16Float` velocity surface;submit stats
  通过 `RenderStats.last_scene_velocity_readback_report` 暴露这份报告。产品测试证明 matched particle frame 会产生非零
  raw velocity pixels。
- 已完成 TP-M1 第十五段 renderer-owned 粒子 velocity 二帧读回基线:`render_product_particle_velocity_writer_uses_renderer_owned_previous_state_on_second_frame`
  现在在第一帧成功提交并滚动 viewport-owned previous rows 后,让第二个移动粒子帧断言 `last_particle_velocity_missing_sprite_count == 0`
  且 `RenderStats.last_scene_velocity_readback_report` 可用、尺寸/字节数匹配并产生非零 raw `scene-velocity` pixels。
- 已完成 TP-M1 第十六段 same-entity keyed 粒子 multi-sprite 产品基线:`render_product_particle_velocity_writer_matches_same_entity_renderer_owned_sprites_by_key_on_second_frame`
  提交同一 entity 下两个非零 `stable_sprite_key` 粒子,首帧缺 previous-state 时 missing=2,第二个移动帧复用 renderer-owned previous rows
  后 missing=0,并继续要求 velocity surface 读回可用且非零,锁住 stable identity 在产品 WGPU 路径中的多 sprite 边界。
- 已完成 TP-M1 第十七段 key=0 anonymous stream ambiguity 诊断:`ParticleExtract::anonymous_stream_ambiguity_sprite_count()`
  统计当前帧同一 entity 下多个 `stable_sprite_key == 0` 的 sprite 数;`FrameSubmissionContext` 传递该计数,`update_base_stats(...)`
  在 reconstructed velocity 被请求且 `particle.transparent` 执行时写入 `RenderStats.last_particle_velocity_anonymous_stream_ambiguity_count`,
  runtime diagnostics 镜像为 `render.particle.velocity.anonymous_stream_ambiguity_count`。产品测试保留单 sprite key=0 兼容行为,
  但 same-entity 两个 key=0 sprite 会报告 ambiguity=2,并由 S22 硬拒绝进入 previous-state/velocity matching。
- 已完成 TP-M1 第十八段 keyed multi-sprite 三帧动态基线:`render_product_particle_velocity_writer_rolls_keyed_multi_sprite_motion_across_three_frames`
  在同一 viewport 连续提交四个同 entity、非零 stable key 的粒子帧。首帧无 previous-state 时 missing=4 且 anonymous ambiguity=0;
  第二、第三个移动帧均复用 renderer-owned previous rows,保持 missing=0/ambiguity=0,并通过 `RenderSceneVelocityReadbackReport`
  断言 raw `scene-velocity` 继续产生非零像素,锁住 previous row 不只是二帧一次性 handoff。
- 已完成 TP-M1 第十九段 previous-palette gate 模块边界收束:`build_mesh_draws/build/previous_skinned_palette.rs`
  从 1000 行以上的 draw orchestration 文件中抽出 previous skinned-palette 选择逻辑与 CPU-morphed morph-shape
  compatibility 单测。`build.rs` 现在只调用 `previous_skinned_joint_palette_for_gpu_scene_entry(...)` 与
  `skinned_joint_palette_state_for_pending_draw(...)`,S23 后继续作为 changing morph-weight previous-source velocity 策略的单一承接模块。
- 已完成 TP-M1 第二十段压力规模 keyed 粒子产品基线:
  `render_product_particle_velocity_writer_rolls_keyed_stress_field_motion` 构造同一 entity 下 32 个非零 stable key sprite,
  首帧期望 missing=32/ambiguity=0,第二个移动帧期望 missing=0/ambiguity=0 并读回非零 raw `scene-velocity`。
  当前工作区补齐 UI 输入上下文对 `UiInputEvent::ToastTimer(_)` 的非刷新处理后,过滤产品测试已通过 8 个 `render_product_particle_velocity`
  用例,覆盖 stress-field 路径。
- 已完成 TP-M2/S21 historical pre-jitter hash artifact 审计:当前仓库文本、渲染计划、`.codex` 会话/计划、`examples` 与
  `zircon_runtime/src/graphics/tests` 未发现可用 golden/hash 产物;限定 git 历史检查显示 `0559abbc` 只在原始 Plan 06 文本中提出
  `render_product_temporal_off_matches_pre_jitter_baseline`,没有测试实现或入库 hash。当前可执行验收面保持 TP-M2-S3 的
  `render_product_temporal_off_matches_anti_alias_feature_disabled_product`;未来若外部 artifact 出现可追加补证,但当前设计不再把它列为仓库内待实现项。
- 已完成 TP-M1/S22 key=0 ambiguous stream hard enforcement 与 producer migration:`ParticleExtract` 的 previous-state/ambiguity
  策略拆入 `frame_extract/particle_extract_policy.rs`;同 entity 多个 key=0 current sprite 不再匹配 previous rows,missing count
  保持为全部歧义 sprite;`build_particle_velocity_vertices(...)` 跳过歧义匿名 current/previous rows,不再生成不可信 motion vectors;
  renderer-owned `update_particle_previous_state_after_success(...)` 不再把歧义匿名 sprites 写入下一帧 previous cache;`World::collect_render_particles(...)`
  生成的 world-HUD bar 背景/填充 sprite 改用 bar index 派生的非零 stable keys。产品测试新增两帧 key=0 多 sprite hard-reject
  基线,过滤产品组已通过 9 个 `render_product_particle_velocity` 用例。
- 已完成 TP-M1/S23 CPU-morphed changing-shape previous source velocity:`GpuScene` 新增 previous skinned GPU source roll,
  CPU-morphed pending draws 在成功提交后保留上一帧 morphed-but-unskinned GPU mesh source;velocity pass 使用第二 vertex slot
  绑定 previous source position 到 `@location(8) previous_position`,从而在 morph weight 变化时用 previous source + previous palette
  写出实际对象速度。RenderDoc 验收仍未完成。
- 校验(2026-06-14):TP-M1-S8 scoped `rustfmt --edition 2021 --check`、`last_particle_velocity_missing_sprite_count`
  source scan 与锁定
  `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never`
  通过(65 个既有 warning)。过滤单测 `cargo test -p zircon_runtime --lib morph_shape_signature ...` 在 Windows
  lib-test 构建阶段 244s 超时,无可用测试输出;过滤单测
  `cargo test -p zircon_runtime --lib particle_velocity_gap_counts_sprites_only_when_reconstructed_velocity_is_requested ...`
  304s 超时且残留 cargo/rustc 进程已终止,未标记为通过。

实施切片:
1. GpuScene prev transform 滚动写入;骨骼 prev palette 槽位。
2. velocity pass(动态对象)+ 相机重投影补全屏;`needs_velocity` relevance 接计划 04。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime temporal --locked`(新模块)
- 验收证据:移动物体 velocity 非零、静止物体在移动相机下 velocity 等于相机重投影差(readback 断言);motion blur 切换消费新 velocity 后产物不回退。

### TP-M2 jitter 接入

进度(2026-06-14):
- 已完成 TP-M2-S1a 契约与矩阵对数据面:`core/framework/render/temporal_jitter.rs`
  新增 `TemporalJitterSample`、`TemporalJitterSequence` 与 Halton(2,3) 取样;`camera.rs`
  的 `ViewportCameraSnapshot` 新增 `#[serde(default)] temporal_jitter`;`view_matrix_pair.rs`
  新增 `ViewProjectionMatrixPair::from_camera(...)`,以 `translate(2*jx/w, 2*jy/h, 0) * unjittered`
  构造 jittered 矩阵。`SceneUniform::from_frame(...)` 已改经矩阵对生成当前 `view_proj`,并让
  previous camera fallback 使用 unjittered 当前矩阵;函数签名不再接受外部 aspect,由 frame viewport size 统一推导。
- 已完成 TP-M2-S1b jitter 注入与 scene uniform ABI 扩展:`FrameSubmissionContext` 按 `ViewportRecord`
  的 `temporal_frame_index` 生成 TAA jitter,TAA Off/Fxaa/Msaa/fallback 时强制零 jitter;`build_runtime_frame(...)`
  与 direct `submit_runtime_frame(...)` 都把有效 anti_alias 与 jitter 写回 frame extract。成功提交后
  `update_temporal_camera_history_after_success(...)` 保存去 jitter 的 previous camera,并推进 temporal frame index;
  失败/跳过提交不会推进。
- `SceneUniform` 已扩展为 `view_proj`、`view_proj_unjittered`、`inverse_view_proj`、
  `previous_view_proj_unjittered`、`motion_params` 与 `jitter_params`;velocity object shader 改读
  current/previous unjittered 矩阵,`VelocityCameraParams` 也基于 unjittered 矩阵重投影。deferred lighting、
  HZB occlusion cull 与 builtin PBR shader layout 已同步到新 ABI,避免 uniform offset 漂移。
- 已完成 TP-M2-S2 上游 pass 审计:`SceneUniform::inverse_view_proj` 改为 current unjittered inverse,
  让 deferred lighting 等屏幕空间反投影自动去 jitter;post-process SSR 参数仍以 `effect_projection`
  与 `effect_view_x/y/z` 表达无 jitter view-space 重建,并新增回归断言 jitter 不改变这些参数;反射探针投屏 helper
  改直接消费 `ViewProjectionMatrixPair::clip_from_world_unjittered`,删除本地重复的透视/正交投影 helper。审计清单:
  deferred geometry、fallback mesh、prepass、particle、sprite、overlay、shadow atlas 写入为光栅投影,保留 jittered
  `scene.view_proj`;velocity object、velocity camera、HZB occlusion、deferred lighting 反投影/重投影使用 unjittered;
  SSAO 当前只读 depth/normal/HZB,无投影矩阵消费。
- 已完成 TP-M2-S3 TAA/AA Off 产品对拍基线:`graphics/tests/render_product_anti_alias.rs`
  新增 `render_product_temporal_off_matches_anti_alias_feature_disabled_product`。同一 `World::new()`
  产品 extract 在 `AntiAliasSettings::off()` 下分别走“AA feature 仍启用但请求 Off”与“质量配置禁用 AA feature”
  两条 WGPU submit/capture 路径,断言 effective/requested mode 均为 Off、AA/FXAA pass 计数为 0、FXAA executor/node
  未执行,并逐像素比较两帧 RGBA 完全一致。该基线覆盖当前仓库可执行的 Off 产物不变性;旧计划要求的 pre-jitter
  hash 入库基线已在 S21 审计为仓库内不可恢复,未来仅作为外部 artifact 补证项。
- 校验(2026-06-14):TP-M2-S1a scoped `rustfmt --edition 2021 --check` 与 source scan
  `TemporalJitterSample|TemporalJitterSequence|ViewProjectionMatrixPair|temporal_jitter|SceneUniform::from_frame`
  通过。复跑 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never`
  被当前工作区无关 UI tree-view helper 缺失阻塞:`zircon_runtime/src/ui/surface/surface/default_interactions/tree_view.rs`
  中 `decode_tree_reorder_drag`、`encode_tree_reorder_drag`、`tree_reorderable` 等符号未定义;错误未指向本切片渲染文件。
- 校验(2026-06-14):TP-M2-S1b `cargo fmt --package zircon_runtime` 通过;`cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s1b-0614 --message-format short --color never` 通过(65 个既有 warning)。test-build 预检曾暴露 cfg(test) 漂移(缺 `temporal_jitter` literal、test-only helper 构造参数、私有 skinning 常量、`ExportBuildMode` re-export 与 stats 测试 import),本切片已补齐;最终 `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s1b-0614 --message-format short --color never` 仍被无关 UI table 测试/交互代码阻塞:`ui/tests/runtime_input_reply_routes/table_pointer_routes.rs` 引用已不存在的 `capture_started`/`capture_released`,`ui/surface/surface/default_interactions/table.rs` 存在 moved value 后借用。
- 校验(2026-06-14):TP-M2-S2 `cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s2-0614 --message-format short --color never` 通过(65 个既有 warning)。source scan `view_proj|inverse_view_proj|previous_view_proj|clip_from_world|world_from_clip` 覆盖 scene renderer WGSL/Rust 消费点;旧 post-process 本地 `orthographic_projection.rs`/`perspective_projection.rs` helper 已删除。
- 校验(2026-06-14):TP-M2-S3 `cargo fmt --package zircon_runtime` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s3-0614 --message-format short --color never` 通过(65 个既有 warning)。`cargo test -p zircon_runtime --lib render_product_temporal_off_matches_anti_alias_feature_disabled_product --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s3-0614 --message-format short --color never` 到达 lib-test 编译阶段后被无关 UI table 输入路由测试阻塞:`ui/tests/runtime_input_reply_routes/table_pointer_routes.rs` 仍引用已移除的 `UiInputDispatchDiagnostics.capture_started/capture_released`;无残留 cargo/rustc 进程。

实施切片:
1. 相机契约加 jitter;投影矩阵注入;velocity/重投影用无 jitter 矩阵。
2. 上游 pass 适配审计(SSR/SSAO/HZB 等使用投影矩阵处统一走契约字段)。

测试阶段:
- `cargo test -p zircon_runtime camera --locked` 与全量 `render_product` 回归
- 验收证据:TAA 关闭时产物与 jitter 改造前逐像素一致;开启时单帧画面按预期亚像素偏移。

### TP-M3 TAA resolve

实施切片:
1. resolve WGSL(重投影、clamp、disocclusion、blend 权重)与 executor;history 资源语义重命名定稿。
2. 质量档位(blend 系数、clamp 强度)接 quality profile。

当前进度:
- 已完成 TP-M3-S1a graph/contract 切片:`PostProcessEffectKind::TaaResolve`、`TAA_HISTORY_PREVIOUS` / `TAA_HISTORY_CURRENT` / `TAA_OUTPUT` 资源名、`temporal.taa-resolve` pass/executor id、`post-process` 可选 `TAA_OUTPUT` 输入、effective stack 过滤与 executor registry 覆盖已落地。
- TAA 生效时 final composite 改读 `TAA_OUTPUT`;TAA 关闭时 `taa-resolve` pass 与 TAA history/output resource 不进入 live graph。legacy history resolve stack node 已在 TP-M4-S1 删除。
- 编译过滤已拆分 raw `SCENE_VELOCITY` producer 与 tile/max/neighbor reconstructed motion-vector 链:TAA-only 只开启 `velocity-object`/`velocity-camera`,不误启 tile chain;history fallback 会移除 TAA-only velocity,但保留 motion blur/SSR 仍需的 velocity。
- 已完成 TP-M3-S1b 最小真实执行切片:`temporal/taa/shaders/taa_resolve.wgsl`、`TaaResolveParams`、`ScenePostProcessResources::execute_taa_resolve(...)`、TAA bind group layout/pipeline/buffer bundle、专用 `temporal.taa-resolve` executor、raw/fallback depth shader rewrite、`SceneFrameHistoryTextures` scene-color read/write 双缓冲与成功帧末 swap 已落地。
- 已完成 TP-M3-S1c TAA 质量档与 resolve 质量内核切片:`TaaQualityPreset::{Low, Medium, High}` 进入 `AntiAliasSettings` 与 `RenderQualityProfile::with_taa_quality(...)`;提交上下文把 quality profile 的 TAA 档位投到有效 view extract;`TaaResolveParams` 用 uniform 编码 history blend、motion rejection、variance clip gamma 与 depth disocclusion threshold,不进入 pipeline key。
- 已完成 TP-M3-S1d `TemporalHistoryStore` 基础硬切片:TAA scene-color history 改为独立 `Rgba16Float` 双缓冲 store,包含 `TemporalHistoryKey { size, format }`、valid/reset 状态、success-only flip、`build_frame_submission_context` 的 store-available seed 判定,以及无效首帧 history weight=0 的 resolve seed 语义;旧 `history_resolve` scene-color copy 与 TAA history 已分离。
- 已完成 TP-M3-S1e 资源内推导 responsive/reactive 抑制切片:`TaaResolveParams` 新增 luma threshold、velocity scale、full-responsive history multiplier 与 confidence cap;`taa_resolve.wgsl` 用当前/历史亮度差与速度推导响应度,压低 history 权重并限制响应像素的 history confidence 恢复。
- 已完成 TP-M3-S1f WGPU TAA 产品 gate 启用切片:`capability_summary(...)` 在 offscreen 后端报告 `supports_taa`,AA 统计把 `temporal.taa-resolve` 计入 anti-alias graph pass count,`render_product_anti_alias.rs` 新增 TAA seed-frame 产品测试源码。
- 已完成 TP-M4-S1 旧 history resolve 硬切片:`BuiltinRenderFeature::HistoryResolve`、`PostProcessEffectKind::HistoryResolve`、`feature_descriptors/history_resolve.rs`、`history.scene-color`/`post.history-resolve` executor id、`HISTORY_PREVIOUS/CURRENT/OUTPUT_SCENE_COLOR` 资源名、legacy scene-color history texture/copy/import 路径与 `with_history_resolve(*)` 测试夹具命名均已删除或替换;`FrameHistorySlot::SceneColor` 硬切为 `FrameHistorySlot::TaaSceneColor`。
- 2026-06-15 补充硬切复扫:editor viewport capture 测试夹具中残留的 `RenderQualityProfile::with_history_resolve(false)` 已迁移为现有 `with_temporal_history(false)`,保持关闭 temporal history 的测试意图,同时不恢复 legacy history-resolve public API。`zircon_runtime/src` 与 `zircon_editor/src` 源码中 `with_history_resolve|HistoryResolve|history_resolve` 已清零。
- 已完成 TP-M4-S2 TAA reactive mask 图输入/默认清零切片:`PostProcessGraphResourceNames::TAA_REACTIVE_MASK`、`temporal.taa-reactive-mask-clear`、`ScenePostProcessResources::execute_taa_reactive_mask_clear(...)` 与 TAA bind group binding 5 已落地;mask 使用 `R8Unorm` 中间纹理,默认 clear-zero,`taa_resolve.wgsl` 读取 authored mask 并与资源内推导 responsive rejection 取最大值。
- 已完成 TP-M4-S3 透明物 reactive mask writer V1 切片:`temporal.taa-reactive-mask-mesh` pass/executor、`MeshPassPipelineKind::TaaReactiveMask`、`TaaReactiveMaskPassProcessor`、`R8Unorm` mesh pipeline/cache 与 `fs_taa_reactive_mask` 已落地;透明 mesh 在 TAA 生效时按材质 base alpha/vertex tint/贴图 alpha 写入 `taa.reactive-mask`,完全透明 texel discard,mask pass 位于 clear 与 `taa-resolve` 之间并与 TAA 关闭过滤同步。
- 已完成 TP-M4-S3b 显式材质 reactive strength 切片:`StandardMaterialDescriptor::taa_reactive_mask_strength` 与材质控制覆盖落地,范围为 `0..=1`,默认 `0`;该字段为材质拥有属性,不会进入 shader custom property overrides。`MaterialRuntime`/capture seed 保留该强度,标准材质 uniform 扩展到 144 bytes 并把强度写入 `MaterialPropertyUniform.data8.x`;`fs_taa_reactive_mask` 对 `max(sampled_base_alpha, data8.x)` 写入 `taa.reactive-mask`。
- 已完成 TP-M4-S3c opaque material reactive flag writer 切片:`MeshPassPipelineKind::TaaReactiveMaterialMask`、`fs_taa_reactive_material_mask` 与独立 pipeline/cache 已落地;`PendingMeshDraw -> MeshDraw -> MeshBatchRef` 传递 clamped `taa_reactive_mask_strength`,`TaaReactiveMaskPassProcessor` 只为 visible opaque/alpha-mask 且强度非零的 batch 生成 reactive-material command,透明 batch 仍走 alpha-sampling writer。
- 当前 shader 使用速度重投影、closest-depth velocity dilation、视口/深度边界 rejection、YCoCg 3x3 variance AABB clip、深度差 disocclusion、confidence/motion-based history weight、资源内推导 responsive/reactive history suppression、`taa.reactive-mask` authored 输入、透明材质 alpha mask writer V1、显式材质 authored strength 与 opaque/alpha-mask material flag writer。产品级 seed、静态 history、动态遮挡、authored/transparent writer、粒子透明 pass、粒子 previous-state velocity 诊断收敛、粒子 `scene-velocity` writer、raw velocity surface 读回基线、key=0 hard enforcement/producer migration 与 pre-jitter artifact 审计已补;仍待后续切片:RenderDoc 验收与更复杂动态场景。

测试阶段:
- `cargo test -p zircon_runtime temporal --locked`(静止场景收敛性:N 帧后帧间差趋零的 readback 断言)
- 验收证据:边缘锯齿收敛对比截图;快速遮挡切换无拖影(disocclusion 生效,RenderDoc 抓帧记录)。

### TP-M4 顺序整合与 P0 关闭

实施切片:
1. 后处理链按定稿顺序重排(与计划 07 同步);删除旧 history_resolve 路径。当前 TP-M4-S1 已完成旧路径删除与公共 history 槽位命名硬切,TP-M4-S2 已补 TAA reactive mask 图输入与默认零值路径,TP-M4-S3 已补透明 mesh alpha mask writer,TP-M4-S3b 已补显式材质 reactive strength,TP-M4-S3c 已补 opaque/alpha-mask material flag writer,TP-M4-S4a/S4b 已补静态 history 与动态遮挡产品基线,TP-M4-S4c/S4d 已补 authored opaque 与 transparent alpha writer 的产品统计基线,TP-M4-S4e 已补粒子透明 pass + TAA resolve 产品顺序/贡献基线,TP-M1-S9..S23 已补粒子 previous-state velocity 诊断收敛、`scene-velocity` writer、stable identity、renderer-owned previous rows、previous billboard basis、velocity surface 像素读回、keyed stress-field、key=0 hard enforcement/producer migration 与 CPU-morphed changing-shape previous source velocity;RenderDoc 视觉验收与更复杂动态场景仍待后续切片。
2. 风险清单 P0 条目对照验收并在风险文档标记关闭。

测试阶段:
- `cargo test -p zircon_runtime --lib --locked` 范围回归 + RenderDoc 人工对拍
- 验收证据:ghosting 复现场景(高速移动 + 高对比)无可见拖影;`.codex/plans/Runtime 渲染风险清单` P0 条目附关闭证据。

## 工程落地细化

本章节为本计划的实施权威(见 index.md §8 第 7 条)。bind group 槽位、GPU 数据布局、WGSL include 前缀、RenderQueueValue、sort_key、测试命名全部直接引用 index.md §8 全局约定,本章不重复定义。

### 模块与文件落点

新增文件(全部位于根工作区 `zircon_runtime`,不新增 crate):

| 新增文件 | 内容 | 层 |
|---|---|---|
| `zircon_runtime/src/core/framework/render/temporal_jitter.rs` | `TemporalJitterSequence`、`TemporalJitterSample`、`halton(index, base)`;纯数学,无 wgpu | framework 契约 |
| `zircon_runtime/src/core/framework/render/view_matrix_pair.rs` | `ViewProjectionMatrixPair`(jittered/unjittered 矩阵对)与构造函数 | framework 契约 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/mod.rs` | 模块声明(thin) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/velocity_camera_params.rs` | `VelocityCameraParams`(替代 `MotionVectorCameraParams`,沿用其相机切变检测阈值) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_camera.rs` | 相机重投影全屏 velocity executor(executor id `temporal.velocity-camera`) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs` | 动态对象 velocity executor(executor id `temporal.velocity-object`),读 GpuScene prev transform | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_executor.rs` | `TaaResolveExecutor`(executor id `temporal.taa-resolve`) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_params.rs` | `TaaResolveParams` GPU uniform 与质量档映射 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/temporal_history_store.rs` | `TemporalHistoryStore`(history 双缓冲,走计划 01 持久资源 API) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/shaders/velocity_camera.wgsl` | 相机重投影全屏 pass(entry `fs_main`) | WGSL |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/shaders/velocity_object.wgsl` | 对象 velocity 模板(entry `vs_velocity_object` / `fs_velocity_object`) | WGSL |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/shaders/taa_resolve.wgsl` | TAA resolve(entry `fs_taa_resolve`) | WGSL |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/taa_reactive_mask.rs` | reactive mask draw 生成:visible transparent batches 写 alpha mask;visible opaque/alpha-mask 且材质强度非零时写 material mask | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs` | `R8Unorm` reactive mask mesh pipeline,入口 `vs_main` / `fs_taa_reactive_mask` / `fs_taa_reactive_material_mask` | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs` | reactive mask mesh pipeline cache,按 `MeshPassPipelineKind::TaaReactiveMask` / `TaaReactiveMaterialMask` 与 `PipelineKey` 缓存 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/shaders/zr_motion.wgsl` | 共享 include:velocity 编解码与重投影函数,无 entry point(index.md §8 第 3 条,计划 08 模板消费) | WGSL |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs` | `temporal` feature descriptor(三个 pass 声明) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs` | prev transform 滚动写入(模块归计划 03,本文件写入逻辑归本计划,见计划 03"风险与回退"第 3 条) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs` | renderer-owned skinned current/previous palette 状态滚动;成功提交后把 current 面切给下一帧 previous 面 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs` | object velocity mesh pipeline 创建,入口 `vs_velocity_object`/`fs_velocity_object` | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs` | object velocity mesh pipeline cache,按 `MeshPassPipelineKind::Velocity` 与 `PipelineKey` 缓存 | graphics 实现 |
| `zircon_runtime/src/graphics/tests/render_product_temporal.rs` | velocity/TAA 产物对拍测试 | 测试 |

修改文件:

| 修改文件 | 改动 |
|---|---|
| `zircon_runtime/src/core/framework/render/camera.rs` | `ViewportCameraSnapshot` 增加 `#[serde(default)] pub temporal_jitter: TemporalJitterSample`(默认零) |
| `zircon_runtime/src/core/framework/render/anti_alias/settings.rs` | `resolve()` 的 `history_available` 接 `TemporalHistoryStore` 真实状态(既有 `AntiAliasMode::Taa`/`UnsupportedTaa` fallback 机制不动) |
| `zircon_runtime/src/core/framework/render/material/standard_material.rs` | `StandardMaterialDescriptor` 增加 `taa_reactive_mask_strength`(默认 0,材质拥有的 TAA reactive 强度) |
| `zircon_runtime/src/asset/assets/material/material_control.rs` + `material_asset.rs` | `taa_reactive_mask_strength` 作为材质拥有 override 同步到标准材质 descriptor,校验有限数值且范围为 `0..=1`,不写入 shader custom property overrides |
| `zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs` + `resource_streamer/*material*.rs` | `MaterialRuntime` 与 capture seed 保留材质显式 reactive 强度,保证资源流与离线访问路径一致 |
| `zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs` | 标准材质 uniform 扩展到 144 bytes,`MaterialPropertyUniform.data8.x` 存 `taa_reactive_mask_strength` |
| `zircon_runtime/src/core/framework/render/post_process/stack.rs` | `PostProcessGraphResourceNames` 增加 `SCENE_VELOCITY`、`TAA_HISTORY_PREVIOUS`、`TAA_HISTORY_CURRENT`、`TAA_OUTPUT`、`TAA_REACTIVE_MASK`;删除 `HISTORY_PREVIOUS_SCENE_COLOR`、`HISTORY_CURRENT_SCENE_COLOR`、`HISTORY_OUTPUT_SCENE_COLOR`;`TAA_REACTIVE_MASK` 作为 TAA initial resource 进入 stack validation;effect 链顺序定稿(TAA → DoF → motion blur → 其余,与计划 07 链定稿表一致) |
| `zircon_runtime/src/graphics/extract/history.rs` | `FrameHistorySlot::SceneColor` 重命名为 `TaaSceneColor`(语义定稿:只存 TAA resolve 输出,后处理不写回) |
| `zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs` | 增 `Temporal` 变体;删 `HistoryResolve` 变体 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs` | 删除 `motion-vector-clear` / `motion-vector-camera` / `motion-vector-object` 三个 pass(迁入 temporal feature);tile-max / neighbor-max 改读 `SCENE_VELOCITY` |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs` | 分发 `Temporal`,删除 `history_resolve` 分支 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` | 注册 `temporal.*` executor(`velocity-object`、`velocity-camera`、`taa-reactive-mask-clear`、`taa-reactive-mask-mesh`、`taa-resolve`),注销 `post.motion-vector-*` 与 `history.scene-color` |
| `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs` + `from_frame.rs` | `view_proj` 改为 jittered;新增 `view_proj_unjittered`、`previous_view_proj_unjittered`(替代 `previous_view_proj`)、`jitter_params: [f32; 4]`(xy=本帧像素 jitter,zw=上帧) |
| `zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs` + `new.rs` | 增加 `temporal_frame_index: u64` 与 `previous_unjittered_view_proj`;`motion_vector_object_history` 字段删除 |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs` | jitter 注入与帧末翻转调用点(见"帧时序与集成点") |
| `zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs` + `prev_transform.rs` | `GpuSceneEntry` 记录已滚动 previous 状态;帧末把 current transform 滚动到 `prev_world_from_local`,并把变更 span 留给下一帧上传 |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs` + `scene_renderer_core_render_scene/render_scene.rs` | 成功 `queue.submit(...)` 后触发 GPUScene previous-transform 滚动,保持当帧 velocity 读取旧 previous、下一帧读取本帧 current |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs` | pending draw 同步 GPUScene 时只使用 GPUScene rolled previous transform,并把有效 previous 传播到 primitive flags、motion params 与 MeshDraw |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs` | temporal velocity pipeline 缓存,改用 GpuScene instance index ABI |

### 核心类型与接口

契约层(`core/framework/render`,无 wgpu):

```rust
// temporal_jitter.rs
/// Halton 低差异序列,base 取 2/3(对齐 URP HaltonSequence.Get 用法)。
pub fn halton(index: u32, base: u32) -> f32;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct TemporalJitterSample {
    /// 像素单位偏移,范围 [-0.5, 0.5];TAA 关闭时恒为 ZERO。
    pub offset_pixels: Vec2,
    /// 产生本样本的序列号(disocclusion 诊断与测试用)。
    pub sequence_index: u32,
}

pub struct TemporalJitterSequence {
    period: u32, // 质量档:Low=8、Medium/High=16
}
impl TemporalJitterSequence {
    pub fn new(period: u32) -> Self;
    /// sample(i) = (halton(i % period + 1, 2) - 0.5, halton(i % period + 1, 3) - 0.5)
    /// 对齐 URP TemporalAA.CalculateJitter 的 (frameIndex & 1023) + 1 取样(避开 index 0)。
    pub fn sample(&self, frame_index: u64) -> TemporalJitterSample;
}

// view_matrix_pair.rs
/// jittered/unjittered 矩阵对:同帧同时下发,velocity/重投影/HZB/SSR 一律取 unjittered,
/// 光栅化取 jittered。唯一构造入口,杜绝散落的"自行加 jitter"。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewProjectionMatrixPair {
    pub clip_from_world_jittered: Mat4,
    pub clip_from_world_unjittered: Mat4,
}
impl ViewProjectionMatrixPair {
    /// jittered = translate(2*jx/w, 2*jy/h, 0) * unjittered(对齐 URP CalculateJitterMatrix)。
    pub fn from_camera(
        camera: &ViewportCameraSnapshot, // 读 camera.temporal_jitter
        viewport_size: UVec2,
    ) -> Self;
}
```

实现层(`graphics/scene/scene_renderer/temporal`):

```rust
// taa/temporal_history_store.rs —— 持久资源经计划 01 API,不进 TransientResourcePool
pub(crate) struct TemporalHistoryStore {
    /// (size, format) 失配或相机 target/projection 切换 → invalidate 重建。
    key: TemporalHistoryKey,
    /// 双缓冲:read = 上帧 resolve 输出,write = 本帧输出;帧末 flip()。
    textures: [wgpu::Texture; 2],
    read_index: usize,
    valid: bool, // 首帧/失效后为 false → resolve 走 camera-cut 路径
}
impl TemporalHistoryStore {
    /// 每帧把 read/write 两面经 RenderGraphBuilder::register_external_texture 注册为
    /// TAA_HISTORY_PREVIOUS / TAA_HISTORY_CURRENT(RgTextureHandle),executor 内只用句柄解析。
    pub(crate) fn register(&self, builder: &mut RenderGraphBuilder) -> TemporalHistoryHandles;
    pub(crate) fn ensure(&mut self, device: &wgpu::Device, key: TemporalHistoryKey);
    pub(crate) fn flip(&mut self);
    pub(crate) fn invalidate(&mut self);
}

// taa/taa_resolve_executor.rs
pub(crate) struct TaaResolveExecutor {
    pipeline: wgpu::RenderPipeline, // 全屏三角形 raster pass,与既有 post 系执行器同形态
    params_buffer: wgpu::Buffer,
}
// 执行步骤(fs_taa_resolve 算法,详见 WGSL 节):
// 1. 3x3 深度最近邻 velocity dilate(取邻域 depth 最近像素的 velocity);
// 2. history_uv = uv - velocity;越界 → disocclusion;
// 3. 当前帧 3x3 邻域采样,YCoCg 域构建 AABB(均值±方差*variance_clamp_scale);
// 4. history 双线性采样 → YCoCg → clip 到 AABB(zr_clip_aabb);
// 5. disocclusion 检测:|velocity| > velocity_disocclusion_threshold 或 history_uv 越界
//    或 clamp 截断量超阈 → blend_weight 提升至 1.0(回退当前帧);
// 6. output = mix(history_clamped, current_filtered, blend_weight),
//    blend_weight 基线 = current_frame_weight(默认 0.04,对齐 UE CVarTemporalAACurrentFrameWeight);
// 7. 输出同时写 TAA_HISTORY_CURRENT 与 TAA_OUTPUT(后处理链的 scene color 输入)。

// anti_alias/taa_quality.rs + taa/taa_resolve_params.rs —— 质量档映射
pub enum TaaQualityPreset {
    Low,
    Medium,
    High,
}
// Low/Medium/High 通过 TaaResolveParams uniform 映射到 history_blend_weight、
// motion_rejection_scale、variance_clip_gamma、depth_disocclusion_threshold;
// 不进入 shader define 或 pipeline cache key。
```

velocity 输入侧(写入逻辑归本计划,缓冲归属计划 03 `GpuScene`):

```rust
// graphics/scene/gpu_scene/prev_transform.rs
impl GpuScene {
    /// 帧末(present 成功后)调用:本帧 transform 段整体滚动为 prev 段。
    /// 实现为 prev 槽与 current 槽的 buffer 内 copy(encoder copy_buffer_to_buffer)
    /// 或双段索引翻转,新注册条目 prev = current(首帧零 velocity)。
    pub(crate) fn roll_prev_transforms(&mut self, encoder: &mut wgpu::CommandEncoder);
    /// 骨骼对象:prev palette 双缓冲 storage buffer,与 skinning palette 同布局;
    /// flip 时机与 roll_prev_transforms 相同。衔接计划 08 GPU skinning;08 落地前
    /// fallback skinning 路径同样从该缓冲读 prev palette。
    pub(crate) fn flip_skinned_prev_palettes(&mut self);
}
```

对象筛选消费计划 04 的 `PrimitiveRelevance`:velocity object pass 只绘制 `needs_velocity` 位为真的非透明对象(本计划 06 正文与计划 04 §目标架构均使用 `needs_velocity` 命名,本章沿用)。计划 04 未落地期间,TP-M1 以临时谓词 `mobility == Dynamic && transform != prev_transform`(对齐 UE `PrimitiveHasVelocityForFrame` 的 0.0001 容差比较)内联在 temporal 模块,04 落地时同变更切换到 relevance 位并删除临时谓词。

AA 模式选择与计划 07 协调:`VolumeComponentDescriptor` 注册的 AA 覆写最终收敛为 `AntiAliasSettings.mode`;本计划只消费 `AntiAliasMode::Taa` 判定,互斥/共存策略(TAA vs FXAA/SMAA/MSAA)按计划 07 定稿,不在本计划重定义。

### GPU 数据布局与 WGSL 约定

velocity 纹理:`SCENE_VELOCITY`,格式 `Rg16Float`(`render_graph` 的 `TextureFormat::Rg16Float` 已存在映射;对齐 URP `k_TargetFormat = R16G16_SFloat`、UE `PF_G16R16`),编码为 NDC 偏移 `(ndc_curr - ndc_prev).xy * 0.5`(即 UV 位移),清除值黑。history 纹理:`Rgba16Float`(HDR scene color 语义,对齐 URP `AccumulationFormatList[0] = R16G16B16A16_SFloat`),不进瞬态池。

scene uniform(group0,既有缓冲扩展,std430 偏移注释):

```text
view_proj                  : mat4x4<f32>  // offset 0    —— jittered,所有光栅化 pass 使用
view_proj_unjittered       : mat4x4<f32>  // offset 64
previous_view_proj_unjittered : mat4x4<f32> // offset 128 —— 替代既有 previous_view_proj
jitter_params              : vec4<f32>    // offset 192  —— xy 本帧像素 jitter,zw 上帧
// 其余既有字段顺延,矩阵列主序(index.md §8 第 2 条)
```

TAA resolve pass binding(TP-M4-S2 起使用 post-process resource-local group0,不绑定 scene uniform;shader 通过 `textureLoad` 做 point sampling,depth fallback 由 `PostProcessDepthSamplingMode` 重写 WGSL 声明与 load 语句):

| group | binding | 资源 | WGSL 类型 |
|---|---|---|---|
| 0 | 0 | scene color(post-lighting) | `texture_2d<f32>` |
| 0 | 1 | scene depth | `texture_depth_2d` 或 fallback `texture_2d<f32>` |
| 0 | 2 | scene velocity | `texture_2d<f32>` |
| 0 | 3 | TAA history previous | `texture_2d<f32>` |
| 0 | 4 | `TaaResolveParams` | `var<uniform>` |
| 0 | 5 | TAA reactive mask | `texture_2d<f32>` |

`TaaResolveParams` uniform(帧级小块,允许 uniform,index.md §8 第 2 条):

```text
viewport_and_flags  : vec4<u32>  // offset 0:  x,y=尺寸 z=TAA enabled flag w=保留
blend_and_clamp     : vec4<f32>  // offset 16: x=history blend weight y=motion rejection scale
                                 //            z=neighborhood clamp strength w=保留
```

Reactive mask 纹理:`TAA_REACTIVE_MASK`,格式 `R8Unorm`。TP-M4-S2 先由 `temporal.taa-reactive-mask-clear` 清零,TP-M4-S3 在 clear 与 resolve 之间追加 `temporal.taa-reactive-mask-mesh`,对 visible transparent mesh 使用 `fs_taa_reactive_mask` 写入采样 base alpha 并与材质 explicit strength 取最大。TP-M4-S3c 同一 mesh pass 追加 `TaaReactiveMaterialMask` variant:opaque/alpha-mask batch 只有在 CPU 侧检测到 `taa_reactive_mask_strength > 0` 时才进入 `fs_taa_reactive_material_mask`,直接写材质强度。完全透明 texel discard,其余 alpha/strength 作为 authored mask 与 resolve shader 内资源推导 responsive 值取最大。

velocity camera pass:group1 binding0 = depth、binding1 = `VelocityCameraParams`(矩阵对来自 scene uniform group0,参数仅保留相机切变 flags)。velocity object pass:group0 + group3(GpuScene instance index,index.md §8 第 1 条),无 group2 材质绑定(默认材质路径,对齐 UE velocity pass 的 `UseDefaultMaterial` 简化)。

`zr_motion.wgsl` 共享 include 关键函数(只暴露函数与 struct,无 entry point):

```wgsl
fn zr_encode_velocity(ndc_curr: vec2<f32>, ndc_prev: vec2<f32>) -> vec2<f32>;
fn zr_decode_velocity_uv(encoded: vec2<f32>) -> vec2<f32>;   // velocity → UV 位移
fn zr_reproject_uv(uv: vec2<f32>, velocity_uv: vec2<f32>) -> vec2<f32>;
fn zr_rgb_to_ycocg(color: vec3<f32>) -> vec3<f32>;
fn zr_ycocg_to_rgb(color: vec3<f32>) -> vec3<f32>;
// AABB 中心线 clip(UE TAAShader 同型做法),返回 clip 后 history 色
fn zr_clip_aabb(aabb_min: vec3<f32>, aabb_max: vec3<f32>, history: vec3<f32>, current: vec3<f32>) -> vec3<f32>;
fn zr_closest_depth_velocity(depth_tex: texture_depth_2d, velocity_tex: texture_2d<f32>, point_sampler: sampler, uv: vec2<f32>, texel: vec2<f32>) -> vec2<f32>;
```

entry point:`velocity_camera.wgsl::fs_velocity_camera`(depth 反投影 unjittered 矩阵对求重投影差,沿用既有 `execute_motion_vector_camera` 的 enabled/camera-cut flag 协议)、`velocity_object.wgsl::vs_velocity_object`(双位置变换:`clip_from_world_jittered * world_pos` 进光栅,`unjittered 当前/prev` 进 varying 求差)、`taa_resolve.wgsl::fs_taa_resolve`。

### 帧时序与集成点

pass 链精确位置(compiled graph 顺序,与计划 07 共同遵守):

```text
depth prepass
→ temporal.velocity-object   (RenderPassStage::DepthPrepass 尾部;只画 needs_velocity 对象,
                              depth load、SCENE_VELOCITY clear_store)
→ temporal.velocity-camera   (全屏补静态部分;SCENE_VELOCITY load_store,只写 velocity==0 处
                              ——既有 motion-vector-clear pass 删除,clear 语义并入 object pass 的 attachment ops)
→ lighting(forward+/deferred)→ transparency
→ temporal.taa-reactive-mask-clear (TAA 生效时清零 R8Unorm `taa.reactive-mask`)
→ temporal.taa-reactive-mask-mesh  (visible transparent mesh 按 base alpha 写 authored mask)
→ temporal.taa-resolve       (RenderPassStage::PostProcess;读 SCENE_COLOR/SCENE_DEPTH/
                              SCENE_VELOCITY/TAA_HISTORY_PREVIOUS/TAA_REACTIVE_MASK,
                              写 TAA_HISTORY_CURRENT + TAA_OUTPUT)
→ DoF → motion blur(tile-max/neighbor-max 改读 SCENE_VELOCITY)→ 其余后处理(读 TAA_OUTPUT 为 scene color)
```

prev 数据翻转时机(全部收口在 `submit/submit.rs` 的 present 成功路径;相机历史当前由 `update_temporal_camera_history_after_success` 更新):

1. `GpuScene::roll_prev_transforms` + `flip_skinned_prev_palettes`(GPU 侧);
2. `TemporalHistoryStore::flip`;
3. `ViewportRecord` 记录本帧 unjittered view-proj 与相机快照(供下帧 velocity-camera 与 resolve);
4. `temporal_frame_index += 1`。
提交失败/跳帧:不翻转、不递增(对齐 URP `isNewFrame`/`GetAccumulationVersion` 的重绘保护:同帧重渲染时 velocity 置黑、history 不前进)。

jitter 注入点:`build_frame_submission_context` 在 `apply_viewport_size` 同一位置,按 `AntiAliasSettings.resolve(capabilities, history_store.valid)` 结果写 `camera.temporal_jitter = sequence.sample(temporal_frame_index)`;TAA 非生效(Off/fallback/任何非 Taa 模式)时强制 `TemporalJitterSample::default()`(零),且 `temporal` feature 的 taa-resolve pass 不进 compiled graph(velocity 两 pass 由 motion blur/SSR 消费方决定保留,feature 关闭即整体剔除,index.md §6 第 4 条)。上游 pass(SSR/SSAO/HZB/重投影)一律改读 scene uniform 的 `view_proj_unjittered` / `previous_view_proj_unjittered`,TP-M2 以 grep 审计清单驱动。

硬切换删除项(index.md §6 第 5 条,各项在对应切片同变更内完成):

| 删除项 | 替代 | 切片 |
|---|---|---|
| `types/viewport_motion_vector_object_history.rs`(CPU 对象 history)+ `submit/update_motion_vector_history.rs` + `viewport_record/motion_vector_object_history.rs` | GpuScene prev transform / prev palette(相机快照保留迁入 `viewport_record`) | TP-M1 |
| `post_process/resources/execute_motion_vector_camera/`、`params/motion_vector_camera_params.rs` | `temporal/velocity/execute_velocity_camera.rs` + `velocity_camera_params.rs` | TP-M1 |
| `gpu/mesh_motion_vector.rs` 的 `record_mesh_motion_vectors_to_resource`、`has_previous_motion_vector_transform` 谓词、`ensure_motion_vector_pipeline` 旧 ABI | `temporal/velocity/execute_velocity_object.rs`(instance index ABI) | TP-M1 |
| `feature_descriptors/post_process.rs` 的 `motion-vector-clear`/`motion-vector-camera`/`motion-vector-object` 三个 pass 声明 | `feature_descriptors/temporal.rs` | TP-M1 |
| scene uniform `previous_view_proj` + `motion_params` 旧字段语义 | 矩阵对 + `jitter_params` | TP-M2 |
| `feature_descriptors/history_resolve.rs`、executor `history.scene-color`、`BuiltinRenderFeature::HistoryResolve`、`FrameHistorySlot::SceneColor`、`HISTORY_PREVIOUS/CURRENT/OUTPUT_SCENE_COLOR` 资源名、`PostProcessEffectKind::HistoryResolve` | `temporal.taa-resolve` + `TaaSceneColor` 槽 + `TAA_*` 资源名 | TP-M4 |
| 测试夹具 `with_history_resolve(*)`(render_framework_bridge / render_product_ui / render_product_anti_alias 等) | `with_temporal(*)` 等价开关 | TP-M4 |

### 实施切片细化

里程碑-切片对应正文 TP-M1..M4;切片期只 `cargo check -p zircon_runtime --lib --locked`(index.md §7)。

**TP-M1-S1 GpuScene prev 数据面**:触碰 `gpu_scene/prev_transform.rs`(新增)、`gpu_scene/mod.rs`(声明)、`submit/submit.rs`(帧末调用)。要点:prev transform 滚动 + 翻转时机收口。完成判据:check 通过;prev 槽布局常量与计划 03 `primitive_data` SOA 偏移一致(代码注释互引)。

**TP-M1-S2 velocity 双 pass 与旧路径删除**:触碰 `temporal/velocity/*`(新增)、`feature_descriptors/temporal.rs`(新增)、`builtin_render_feature.rs`、`descriptor_for.rs`、`render_pass_executor_registry.rs`、删除表 TP-M1 行全部文件、`post_process.rs` descriptor、tile-max/neighbor-max 读取改名。要点:object pass 走 instance index ABI + 临时 `needs_velocity` 谓词;camera pass 沿用切变检测;`SCENE_VELOCITY` 资源名落地。完成判据:check 通过;`motion_vector` 旧符号 grep 清零(除 tile-max 系命名)。

**TP-M1-S4 GPUScene previous skinned palette roll**:触碰 `gpu_scene/prev_skinned_palette.rs`(新增)、`gpu_scene/gpu_scene.rs`、compiled/legacy successful-submit render paths、`mesh/build_mesh_draws/build.rs`、`extend_pending_draws_for_mesh_instance.rs`、`pending_mesh_draw.rs`、`mesh/skinning/*`。要点:GPUScene 持久化 current/previous skinned palette 状态;成功提交后滚动;draw 构建只在非 CPU-morphed 源且 skeleton signature/joint count 匹配时启用 previous palette。完成判据:rustfmt/source scan/check 通过;core-min cargo check 通过。

**TP-M1-S5 temporal velocity 命名/文件所有权硬切**:触碰 `temporal/velocity/execute_velocity_object.rs`(新增)、`temporal/velocity/shaders/velocity_camera.wgsl`(新增)、`mesh_pipeline/create_velocity_mesh_pipeline.rs`(新增)、`mesh_pipeline_cache/ensure_velocity_pipeline.rs`(新增)、`MeshPassPipelineKind`、mesh queue stats、RenderStats/diagnostics、post-process camera pipeline/bind layout resource names。要点:删除 object pass 的 `motion_vector` 文件/函数/field 旧名,只保留 motion blur tile-max/neighbor-max 链的 motion-vector 命名;相机 shader 物理归 temporal velocity。完成判据:rustfmt/source scan/check 通过。

**TP-M1-S6 CPU-morphed previous-shape velocity policy**:触碰 `mesh/prepared_queue.rs`、`backend_types.rs`、`update_stats/base_stats.rs`、`render_stats_store/product.rs`、`tests/runtime_diagnostics/*`。要点:CPU-morphed GPU-skinning source 在 previous shape buffer 落地前不归入普通 missing transform,而记录 `skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count` / `render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count`;S7 之后只有 current/previous morph-shape signature 匹配的源可进入 object velocity,签名缺失或变化仍走该缺口计数。完成判据:rustfmt/source scan/check 通过;S23 已用 renderer-owned previous GPU source 关闭 changing-shape previous velocity 缺口。

**TP-M1-S7 CPU-morphed stable morph-shape velocity 覆盖**:触碰 `pending_mesh_draw.rs`、`extend_pending_draws_for_mesh_instance.rs`、`build.rs`、`gpu_scene/prev_skinned_palette.rs`。要点:CPU-morphed shader-skinning source 记录 mesh id + morph weights 组成的 `morph_shape_signature`;GPUScene previous skinned-palette state 记录该签名;签名一致时允许 previous palette 进入 draw,签名变化仍保留 S6 missing previous-shape 诊断。完成判据:rustfmt/source scan/check 通过;过滤单测可运行时补证。

**TP-M1-S8 粒子 velocity 缺口诊断**:触碰 `frame_submission_context.rs`、`build_frame_submission_context/build.rs`、`update_stats/base_stats.rs`、`backend_types.rs`、`render_stats_store/particle.rs`、`tests/runtime_diagnostics/*`。要点:当时 particle extract 只有当前 billboard 状态,无稳定 sprite id/previous position/previous billboard basis,因此本切片不写错误 velocity;当 reconstructed velocity 被 motion blur/SSR 请求且 `particle.transparent` 执行时,按当前 sprite 数记录 `last_particle_velocity_missing_sprite_count` / `render.particle.velocity.missing_sprite_count`。完成判据:rustfmt/source scan/check 通过;S9-S13 已补 previous-state DTO、真实粒子 `scene-velocity` writer、stable identity、renderer-owned previous rows 与 previous billboard basis。

**TP-M1-S9 粒子 previous-state velocity 诊断收敛**:触碰 `scene_extract.rs`、`frame_extract.rs`、`render/mod.rs`、`frame_submission_context.rs`、`build_frame_submission_context/build.rs`、`update_stats/base_stats.rs`、`render_particles.rs` 与 `render_product_anti_alias.rs`。要点:`RenderParticlePreviousSpriteSnapshot` 和 `ParticleExtract.previous_sprites` 携带 previous-state DTO,按 entity 与 current sprite 匹配并逐个消费 previous-state rows;submit stats 用 current sprite count 减 matched previous-state count,避免已有 previous-state 的粒子继续被计为 velocity 缺口。完成判据:focused tests/check/rustfmt 通过;S10-S13 已补 writer、stable identity、renderer-owned rows 与 previous billboard basis。

**TP-M1-S10 粒子 `scene-velocity` writer V1**:触碰 `particle_renderer`、`build_particle_velocity_vertices`、`particle_velocity_vertex`、`particle_velocity.wgsl`、`graph_execution` 注册与产品 fixture。要点:`particle.velocity` 成为 built-in executor,向 graph `SCENE_VELOCITY` 写 matched current/previous billboard corner 的 xy screen velocity;缺 previous-state 的 sprite no-op,不伪造零 motion。完成判据:focused builder/product tests/check/rustfmt 通过;后续由 S11-S13 收敛 identity、previous-state owner 与 previous basis。

**TP-M1-S11 粒子 stable sprite identity 匹配**:触碰 `scene_extract.rs`、`frame_extract.rs`、`render_particles.rs`、`build_particle_velocity_vertices.rs` 与产品 fixtures。要点:`RenderParticleSpriteIdentity { entity, stable_sprite_key }` 成为 previous-state 统计和 velocity writer 的共享匹配键;current JSON 可读 `stable_sprite_key`/`sprite_key`;key=0 继续表示旧匿名 FIFO stream。完成判据:rustfmt/constructor scans 通过;当时 filtered Cargo 被无关 UI 编译错误阻塞,后续 S13 core-min check 已恢复通过。

**TP-M1-S12 renderer-owned 粒子 previous-state roll**:触碰 `ViewportRecord`、`FrameSubmissionContext`、submit/present/direct runtime-frame 成功路径和 `render_product_particle_velocity.rs`。要点:成功提交后把 current particle snapshots 滚动为 viewport-owned previous rows;下一帧若 extract 没有显式 previous rows,submit context 注入 renderer-owned rows 并供缺口统计与 `particle.velocity` writer 使用。完成判据:rustfmt 通过;当时 Cargo 被无关 UI 编译错误阻塞,后续 S13 direct lib-test 与 core-min check 已覆盖成功提交 roll 路径。

**TP-M1-S13 粒子 previous billboard basis ownership**:触碰 `scene_extract.rs`、`render/mod.rs`、`update_particle_previous_state.rs` 与 `build_particle_velocity_vertices.rs`。要点:`RenderParticlePreviousSpriteSnapshot` 携带可选 `RenderParticleBillboardBasisSnapshot`;renderer-owned roll 存储提交帧 camera right/up;velocity builder 使用 stored previous basis 展开 previous corners,显式 legacy rows 无 basis 时 fallback 到 current camera basis。完成判据:rustfmt/constructor scan 通过;`previous_billboard_basis` filtered Cargo test、direct lib-test `successful_submit_records_particle_previous_state_for_next_frame` 与 core-min `cargo check` 均通过。

**TP-M1-S14 粒子 velocity surface readback evidence**:触碰 `backend_types.rs`、`render_graph_execution_record.rs`、`render_graph_execution_resources.rs`、`scene_renderer_core_render_compiled_scene/render.rs`、`scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs`、`update_stats/base_stats.rs` 与 `render_product_particle_velocity.rs`。要点:`RenderSceneVelocityReadbackReport` 成为 test-build raw `scene-velocity` 读回报告;compiled-scene 在 graph submit 后、transient backings 释放前读取 graph-owned `Rg16Float` velocity target,并把 available/size/byte_len/nonzero_pixel_count 写入执行记录和 `RenderStats`;产品测试断言 matched 粒子帧产生非零 velocity pixels。完成判据:rustfmt/check 通过;执行记录 filtered Cargo test 通过;直接运行产品测试二进制覆盖新读回测试和完整 `render_product_particle_velocity` 组。

**TP-M2-S1a 契约与矩阵对数据面**:触碰 `temporal_jitter.rs`、`view_matrix_pair.rs`(新增)、`camera.rs`、`render/mod.rs`(curated re-export)、`scene_uniform/from_frame.rs`、`write_scene_uniform.rs`。要点:Halton jitter sample 与 `ViewportCameraSnapshot::temporal_jitter` 进入框架契约;`ViewProjectionMatrixPair` 成为 scene uniform 当前矩阵构造入口;`SceneUniform::from_frame` 不再从调用方接收 aspect。完成判据:rustfmt/source scan/check 通过;当前 check 被无关 UI tree-view helper 缺失阻塞,需在 UI 工作区恢复后补跑。

**TP-M2-S1b jitter 注入与 scene uniform ABI 扩展**:触碰 `viewport_record/temporal_frame_index.rs`、`build_frame_submission_context/*`、`frame_submission_context.rs`、`submit/build_runtime_frame.rs`、`submit/submit_runtime_frame.rs`、`submit/update_temporal_camera_history.rs`、`scene_uniform.rs`、`from_frame.rs`、velocity camera/object WGSL、deferred/HZB/builtin PBR uniform layout。要点:TAA 非生效时 jitter 恒零;生效时按 temporal frame index 写 jitter;scene uniform 暴露 current jittered/current unjittered/previous unjittered 与 jitter params;velocity/重投影统一无 jitter 矩阵;成功提交才推进 temporal frame index。完成判据:check 通过;关闭 TAA 产物对拍不变(产品对拍仍是 TP-M2 硬验收线)。

**TP-M2-S2 上游 pass 审计**:触碰 SSR/SSAO/HZB/deferred lighting 等读投影矩阵的 WGSL 与 params 构造(以 grep `view_proj|inverse_view_proj` 清单驱动)。要点:重投影/反投影类统一 unjittered,光栅类统一 jittered。完成判据:check 通过;审计清单逐文件勾销记录在 PR 描述。

**TP-M2-S3 TAA Off 产品对拍基线**:触碰 `graphics/tests/render_product_anti_alias.rs`。要点:在当前仓库内用同一产品 extract 对拍 `AntiAliasSettings::off()` 的 AA feature-enabled 与 feature-disabled 两条 WGPU submit/capture 路径,断言 AA graph pass 不执行且 RGBA 逐像素一致。完成判据:format/check 通过;filtered lib-test build 若被无关 UI test drift 阻塞,记录具体阻塞并后续补跑。历史 pre-jitter hash 基线已由 S21 审计为仓库内不可恢复,不再作为当前仓库完成判据。

**TP-M3-S1 history store 与 resolve executor**:触碰 `temporal/taa/*`(新增)、`taa_resolve.wgsl`、`zr_motion.wgsl`、`history/scene_frame_history_textures/*`(scene_color 面移除,SSR/AO/GI 面保留)。要点:双缓冲经 `register_external_texture` 注册;camera-cut 路径(`valid == false` → 输出=当前帧、history 直写)。完成判据:check 通过;taa-resolve pass 在 graph dump 中 IO 声明完整(资源四进二出)。

**TP-M3-S2 质量档**:触碰 `taa_resolve_params.rs`、quality profile 消费点(`RenderQualityProfile` 既有通道)。要点:`TaaQualityPreset` 三档;参数全走 uniform 不产生 shader 排列。完成判据:check 通过;档位切换不触发 pipeline 重建(`CompiledGraphCache` 键不含 TAA 参数)。

**TP-M4-S1 链序定稿与 history_resolve 删除**:触碰删除表 TP-M4 行全部文件、`stack.rs`(effect 顺序)、与计划 07 同步的 post 链重排。完成判据:check 通过;`history_resolve|HistoryResolve` grep 清零。

**TP-M4-S2 TAA reactive mask 图输入/default clear**:触碰 `stack.rs`、`feature_descriptors/temporal.rs`、`builtin_postprocess_executors.rs`、`taa_resolve.wgsl` 与 post-process resource bundle。要点:`TAA_REACTIVE_MASK = "taa.reactive-mask"` 作为 TAA-only `R8Unorm` 图资源进入 stack/compiled graph,先由 `temporal.taa-reactive-mask-clear` 提供默认零值,resolve binding 5 读取 authored mask 并与资源内推导 responsive 值取最大。完成判据:check 通过;TAA disabled 时 pass/resource 被 compiler 过滤。

**TP-M4-S3 透明 mesh reactive mask writer V1**:触碰 `mesh_pass/processors/taa_reactive_mask.rs`、mesh command buffer/indirect/stats 路径、`create_taa_reactive_mask_mesh_pipeline.rs`、`ensure_taa_reactive_mask_pipeline.rs`、`fallback_mesh.wgsl`、`graph_execution/render_pass_execution_context/gpu.rs` 与 `feature_descriptors/temporal.rs`。要点:只绘制 visible transparent mesh;depth 读 `SCENE_DEPTH`,color 写 `TAA_REACTIVE_MASK` load/store;`fs_taa_reactive_mask` 采样 base color alpha,alpha<=epsilon discard,否则写 alpha。完成判据:check 通过;processor/shader/executor/compile tests 覆盖;显式材质 strength 与 opaque writer 由 S3b/S3c 覆盖。

**TP-M4-S3b/S3c 材质 authored reactive strength 与 opaque material flag writer**:触碰标准材质 descriptor/control/asset/runtime/uniform、`mesh/build_mesh_draws/*`、`mesh_draw/*`、`mesh_pass/mesh_pass_processor.rs`、`mesh_pass/processors/taa_reactive_mask.rs`、`create_taa_reactive_mask_mesh_pipeline.rs`、`ensure_taa_reactive_mask_pipeline.rs` 与 `fallback_mesh.wgsl`。要点:`taa_reactive_mask_strength` 是材质拥有属性,默认 0,范围 `0..=1`;透明 writer 写 `max(sampled_base_alpha, data8.x)`,opaque/alpha-mask writer 只有强度非零时由 CPU 侧生成 `TaaReactiveMaterialMask` command 并用 `fs_taa_reactive_material_mask` 写强度。完成判据:check 通过;processor/command-buffer/material/shader tests 覆盖;默认强度 0 不产生 opaque/alpha-mask reactive command。

**TP-M4-S4 P0 关闭**:触碰 `.codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md`(P0 条目标记)、`docs/zircon_runtime/**` 镜像文档。完成判据:里程碑测试阶段全绿 + RenderDoc 抓帧证据归档。

### 测试与验收清单

命名遵循 index.md §8 第 6 条(`render_<topic>_*` 单测、`render_product_*` 对拍)。

| 测试函数 | 位置 | 断言 |
|---|---|---|
| `render_velocity_prev_transform_rolls_after_present` | `gpu_scene/prev_transform.rs` `#[cfg(test)]` | 帧 N 写入的 transform 在帧 N+1 readback 出现在 prev 槽;新注册条目 prev==current |
| `render_gpu_scene_rolls_current_transform_into_previous_after_success` | `gpu_scene/prev_transform.rs` `#[cfg(test)]` | 成功提交后的 roll 把 current transform 写入 previous 槽,标记下一帧 instance span 上传,并让 entry 的 previous 状态变为有效 |
| `render_gpu_scene_roll_marks_previous_valid_without_dirty_upload_when_unchanged` | `gpu_scene/prev_transform.rs` `#[cfg(test)]` | current 与 previous 已一致时只标记 previous 可用,不产生下一帧脏上传 |
| `render_gpu_scene_rolls_current_skinned_palette_after_success` | `gpu_scene/prev_skinned_palette.rs` `#[cfg(test)]` | successful-submit roll 后 previous 面等于上一帧 staged current palette,下一帧 staged current 不会提前污染 previous |
| `render_gpu_scene_drops_previous_skinned_palette_when_current_is_missing` | `gpu_scene/prev_skinned_palette.rs` `#[cfg(test)]` | 当前帧缺失 current palette 的 live key 在 roll 后从 previous 面移除 |
| `previous_skinned_joint_palette_requires_matching_signature_and_joint_count` | `mesh/build_mesh_draws/build/previous_skinned_palette.rs` `#[cfg(test)]` | previous palette 只有在 signature 与 joint count 都匹配时才进入 draw |
| `previous_skinned_joint_palette_requires_matching_cpu_morphed_shape` | `mesh/build_mesh_draws/build/previous_skinned_palette.rs` `#[cfg(test)]` | CPU-morphed shader-skinning source 只有在 current/previous morph-shape signature 匹配时复用 previous palette |
| `morph_shape_signature_tracks_mesh_and_weights` | `mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs` `#[cfg(test)]` | morph-shape signature 随 mesh id 与 morph weights 变化,相同输入稳定 |
| `prepared_queue_stats_count_cpu_morphed_gpu_skinning_source_as_dynamic_geometry` | `mesh/prepared_queue.rs` `#[cfg(test)]` | CPU-morphed GPU-skinning source 缺 previous shape 时进入专用 previous-shape velocity 缺口计数,不污染普通 missing velocity transform |
| `particle_velocity_gap_counts_sprites_only_when_reconstructed_velocity_is_requested` | `submit_frame_extract/update_stats/base_stats.rs` `#[cfg(test)]` | 只有 reconstructed velocity 被 motion blur/SSR 请求且 `particle.transparent` 执行时,粒子 sprite 数才进入 `last_particle_velocity_missing_sprite_count` |
| `particle_extract_counts_previous_state_by_entity` | `core/framework/render/frame_extract.rs` `#[cfg(test)]` | `ParticleExtract.previous_sprites` 只为 entity 匹配的 current sprites 提供 previous-state 计数 |
| `particle_extract_consumes_duplicate_entity_previous_state_once_per_row` | `core/framework/render/frame_extract.rs` `#[cfg(test)]` | 同一 entity 多个 current sprites 时,一个 previous-state row 只匹配一个 current sprite |
| `particle_velocity_gap_excludes_sprites_with_previous_state` | `submit_frame_extract/update_stats/base_stats.rs` `#[cfg(test)]` | reconstructed velocity 请求下 missing sprite count 按 current sprite 数减 matched previous-state 数计算并 saturate |
| `render_product_particle_previous_state_suppresses_velocity_gap_stats` | `graphics/tests/render_product_anti_alias.rs` | motion-blur/TAA 粒子产品帧无 previous-state 时 missing=1,带 previous-state 时 missing=0,且粒子透明 pass 仍在 TAA resolve 前执行 |
| `render_velocity_object_nonzero_for_moving_mesh` | `graphics/tests/render_product_temporal.rs` | 移动 Dynamic mesh 中心像素 readback `|velocity| > 0`;静止 Dynamic mesh 为 0(对齐 UE 0.0001 容差跳过) |
| `render_velocity_camera_matches_reprojection_for_static_scene` | 同上 | 移动相机+静止物体:velocity 等于双矩阵重投影差(readback 与 CPU 参考值逐像素容差 1e-3) |
| `render_velocity_clears_on_camera_cut` | `temporal/velocity/velocity_camera_params.rs` `#[cfg(test)]` | 沿用既有切变阈值用例(平移/旋转/FOV/clip 切变 → disabled) |
| `render_taa_halton_matches_reference_values` | `core/framework/render/temporal_jitter.rs` `#[cfg(test)]` | Halton base 2/3 的首批参考值与 URP jitter 序列输入一致 |
| `render_taa_jitter_sequence_is_periodic_and_avoids_zero_index` | `core/framework/render/temporal_jitter.rs` `#[cfg(test)]` | 周期化取样使用 `(frame % period) + 1`,避开 Halton index 0 且周期重复 |
| `render_taa_jitter_zero_when_taa_inactive` | `frame_submission_context.rs` `#[cfg(test)]` | Off/Fxaa/Msaa/fallback 下 `temporal_jitter == default` |
| `successful_submit_records_camera_history_for_next_frame` | `submit/update_temporal_camera_history.rs` `#[cfg(test)]` | 成功提交保存去 jitter previous camera,并只在成功路径推进 viewport temporal frame index |
| `scene_uniform_exposes_jittered_and_unjittered_current_matrices` | `scene_uniform/from_frame.rs` `#[cfg(test)]` | `SceneUniform` 同时暴露 current jittered/current unjittered、previous unjittered 与 jitter params |
| `render_velocity_camera_params_use_unjittered_camera_matrices` | `temporal/velocity/velocity_camera_params.rs` `#[cfg(test)]` | 相机重投影参数使用 unjittered 当前/历史矩阵,避免 TAA jitter 污染 velocity |
| `render_taa_matrix_pair_is_identical_without_jitter` | `view_matrix_pair.rs` `#[cfg(test)]` | jitter 为零时 jittered 与 unjittered 矩阵一致 |
| `render_taa_matrix_pair_applies_pixel_jitter_in_clip_space` | `view_matrix_pair.rs` `#[cfg(test)]` | jittered == translate(2j/size) * unjittered |
| `render_product_temporal_off_matches_anti_alias_feature_disabled_product` | `graphics/tests/render_product_anti_alias.rs` | `AntiAliasSettings::off()` 下 AA feature-enabled 与 feature-disabled 产品捕获帧逐像素一致,且 AA/FXAA pass 未执行 |
| `render_taa_resolve_converges_on_static_scene` | `render_product_temporal.rs` | 静止场景 N=16 帧后连续两帧输出差的 max-abs 趋零(readback 断言,正文 TP-M3 验收) |
| `render_taa_resolve_rejects_history_on_disocclusion` | 同上 | 遮挡体快速移开后暴露区像素与无 history 单帧渲染差 < 阈值(无拖影) |
| `render_taa_history_invalidates_on_resize_and_camera_switch` | `temporal_history_store.rs` `#[cfg(test)]` | resize/target 切换后 `valid==false`,下一帧 camera-cut 路径 |
| `render_taa_pass_absent_when_disabled` | `graphics/tests/pipeline_compile.rs` 追加 | TAA off 时 compiled graph 无 `temporal.taa-resolve` 节点(culled/未声明断言) |
| `taa_reactive_mask_processor_draws_visible_main_view_batches_by_mask_semantics` | `mesh_pass/processors/mod.rs` `#[cfg(test)]` | transparent batch 生成 `TaaReactiveMask`;opaque/alpha-mask 只有 visible 且材质强度非零时生成 `TaaReactiveMaterialMask`;未标记或不可见 batch 不写 mask |
| `mesh_pass_command_buffers_*` reactive-mask 覆盖 | `mesh_pass/mesh_draw_command_list.rs` `#[cfg(test)]` | reactive-material command 计入 `taa_reactive_mask` stream/indirect stats;static batch 的 material mask 走 uncached postprocess phase,不被 base/shadow/prepass cache 吃掉 |
| `fallback_mesh_shader_exposes_taa_reactive_mask_entry` | `mesh_pipeline/fallback_mesh_shader_source.rs` `#[cfg(test)]` | fallback mesh shader 暴露 `fs_taa_reactive_mask` 与 `fs_taa_reactive_material_mask`,分别写 sampled alpha/explicit strength 与 material strength |
| `taa_reactive_mask_mesh_executor_requires_graph_resources_instead_of_nooping` | `render_pass_executor_registry/tests.rs` | TAA 生效时 mesh writer executor 必须解析 `taa.reactive-mask` 与 `scene-depth`,缺资源报错而非静默跳过 |

"关闭 TAA 逐像素一致"的 `render_product_*` 对拍(TP-M2 硬验收线):当前已落地 `render_product_temporal_off_matches_anti_alias_feature_disabled_product`,用同一 extract 在 `AntiAliasSettings::off()` 下对拍 AA feature-enabled 与 feature-disabled 两条产品路径,作为当前仓库可执行的 Off 产物不变性基线。原设计中的 `render_product_temporal_off_matches_pre_jitter_baseline` 历史 hash 已由 S21 审计为仓库内不可恢复:`0559abbc` 只包含计划文本,没有测试实现或入库 hash。若未来外部 artifact 出现,可作为额外补证追加;当前完成判据改为当前 Off 对拍基线与全量既有 `render_product` 系列(`cargo test -p zircon_runtime render_product --locked`)守护 SSR/SSAO/UI 等间接消费方。里程碑测试命令沿用正文:`cargo test -p zircon_runtime temporal --locked`、`cargo test -p zircon_runtime camera --locked`、TP-M4 全量 `--lib` 回归。

### 参考实现精读笔记

**UE `Renderer/Private/PostProcess/TemporalAA.cpp`**:着色器类 `FTemporalAA`,排列维 `FTAAPassConfigDim`(`ETAAPassConfig`:Main/SSR/DOF/Hair 等,history 输出名表 `kTAAOutputNames` 含 `TAA.History`/`SSR.TemporalAA`)与 `FTAAQualityDim`。`FParameters` 关键字段:`CurrentFrameWeight`(CVar `r.TemporalAACurrentFrameWeight` 默认 0.04)、`SampleWeights[9]`/`PlusWeights[5]`、`bCameraCut`、`HistoryBuffer[FTemporalAAHistory::kRenderTargetCount]`、`ScreenPosToHistoryBufferUV`。`SetupSampleWeightParameters` 把 3x3 采样偏移减去 `TemporalJitterPixels` 后过 `CatmullRom(x)` 或高斯 `exp(-2.29*d²)` 归一化——即"当前帧滤波核以 jitter 为中心"。`AddTemporalAAPass(GraphBuilder, View, Inputs, InputHistory, OutputHistory)`:`bCameraCut = !InputHistory.IsValid() || View.bCameraCut`;输出经 `QueueTextureExtraction` 进 `OutputHistory->RT[i]` 跨帧持有。Zircon 对应:`TaaResolveExecutor` + `TemporalHistoryStore.register`(QueueTextureExtraction ↔ 计划 01 持久资源注册);取舍:V1 不做多 RT history(`kRenderTargetCount` 面)、不做 SSR/DOF 专用 TAA 排列——SSR history 仍走自有 `SCREEN_SPACE_REFLECTION_HISTORY` 槽,只共享 `SCENE_VELOCITY`;jitter 加权滤波核 V1 简化为均匀 3x3(权重表接口预留在 `TaaResolveParams` 扩展位)。

**UE `Renderer/Private/VelocityRendering.cpp`**:格式 `FVelocityRendering::GetFormat` = `PF_G16R16`(需 velocity-depth 时 `PF_A16B16G16R16`);三种输出位置 `BasePassCanOutputVelocity` / `DepthPassCanOutputVelocity` / 独立 velocity pass,Zircon 取独立 pass(URP 同款,改造面最小)。对象筛选两级:`FOpaqueVelocityMeshProcessor::PrimitiveHasVelocityForFrame`——`AlwaysHasVelocity()` 否则取 `Scene->VelocityData.GetComponentPreviousLocalToWorld` 与当前 `LocalToWorld.Equals(..., 0.0001f)` 比较,未动即跳过;`PrimitiveHasVelocityForView`——camera cut 跳过、屏占比小于 `MotionBlurPerObjectSize` 推出的 `MinScreenRadiusForVelocityPass` 跳过。Zircon 对应:前者即 TP-M1 临时谓词→计划 04 `needs_velocity` 位;后者(屏占比剔除)V1 不做,记为计划 04 HZB 切片的后续项。`UseDefaultMaterial` 简化(无遮罩/无顶点修改材质换默认材质)对应 velocity object pass 不绑 group2。

**URP `Runtime/Passes/MotionVectorRenderPass.cs`(含 `MotionVectors.cs`)**:`k_TargetFormat = GraphicsFormat.R16G16_SFloat` 直接背书 RG16F;执行序"先 `DrawCameraMotionVectors`(全屏 procedural 3 顶点三角形 + depth 重建)后 `DrawObjectMotionVectors`(LightMode tag `MotionVectors` 的 renderer list,`PerObjectData.MotionVectors` 提供 `previousLocalToWorld`)"——Zircon 取反序(先对象后相机补底),因对象 pass 带 depth test 写 velocity,相机 pass 只补 velocity==0 处,语义等价且省一次全屏混合判断;`camera.depthTextureMode |= MotionVectors | Depth` 的"prev 矩阵由引擎侧每帧维护"对应 GpuScene prev 槽帧末滚动。

**URP `Runtime/TemporalAA.cs`**:`CalculateJitter`:`HaltonSequence.Get((frameIndex & 1023) + 1, 2/3) - 0.5` ——`TemporalJitterSequence::sample` 照搬(含 +1 避 index 0);`CalculateJitterMatrix`:`Matrix4x4.Translate(2*jitter.x/width, 2*jitter.y/height, 0)` 左乘投影——`ViewProjectionMatrixPair::from_camera` 同式;`Settings.m_FrameInfluence`(原 frameInfluence)与 `varianceClampScale` ↔ `TaaQualityPreset` 两参数;`jitterFrameCountOffset` 用于测试确定性 ↔ 我们直接用 `temporal_frame_index` 注入测试;`TemporalAADescFromCameraDesc` 强制 msaa=1、无 mip、`AccumulationFormatList` 首选 `R16G16B16A16_SFloat` ↔ `TemporalHistoryKey` 约束;`Render()` 的 `isNewFrame = GetAccumulationVersion(multipassId) != Time.frameCount`、重绘帧用 `blackTexture` 当 motion 源——对应本计划"提交失败/跳帧不翻转 + velocity 置黑"规则;`ValidateAndWarn` 的 MSAA/动态分辨率/相机栈禁用矩阵 ↔ `AntiAliasSettings::resolve` 的 fallback 报告(`UnsupportedTaa`),动态分辨率与 TAA 共存推迟到计划 07 动态分辨率定稿,V1 共存时按 URP 行为禁用 TAA 并出 fallback 报告。

- 2026-07-18 reactive-mask性能交接：当前TAA每帧固定用独立pass清写整张R8 mask，0 reactive command仍产生pass与全屏写；有command再开mesh pass。TP-M4后续须让0-command绑定共享black mask并零mask pass/bytes，有command由唯一mesh writer以clear_store清零+绘制；同时发布resolve bind-group create与resource-view generation counter，只有lifetime稳定才缓存。见PERF-MVP-350及anti-alias静态证据。

## 状态与产出记录

- 2026-07-18 history validation性能交接：`FrameHistoryValidationKey`在submission context与viewport history之间已改Arc共享，per-record wide deep clone=0；Render06仍须以scene/camera/post/particle/feature component revisions形成compact token，复用shared bindings/visibility/static handles，stable generation不重建/深比较，changed保持现有五类invalidation reason优先级。见PERF-MVP-413/414。

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`06/2026-07-09-temporal-pipeline-output-records.md`](06/2026-07-09-temporal-pipeline-output-records.md)
- 2026-07-18 prepared-camera矩阵交接：zero-jitter路径的identity jitter matrix乘法已直接删除；scene uniform、froxel、postprocess、velocity、subsurface与Hybrid GI仍各自重建同camera projection/view pair。TP-M1/M3应产出per-camera/render-region generation的jittered/unjittered/inverse/previous prepared matrices供全部pass借用，最终pair build≤1/generation；见PERF-MVP-346及`docs/plans/performance/01/2026-07-18-runtime-core-framework-render-camera-view-static-review.md`。
- 2026-07-18 TAA history owner交接：TAA双纹理CPU整图初始化已改GPU clear；TP-M1仍须让TAA pair按TAA feature+size generation独立创建/resize/flip，HZB或froxel质量变化不得重建TAA，TAA-off真实texture=0且stable view clone=0。见PERF-MVP-395。
- 2026-07-18 temporal执行补充交接：object velocity空LoadStore pass已删除；TAA resolve与camera velocity仍每camera建bind group，camera params又重建current/previous matrix pair+inverse。TP-M1/M3消费prepared-camera artifact与resource-generation bindings，stable matrix build≤1/camera、bind create=0；0 reactive归PERF-MVP-350。见temporal静态证据。
- 2026-07-18 particle velocity hard-cut交接：legacy velocity路径每frame重建anonymous ambiguity与previous identity树并CPU展开current+previous六顶点；compiled graph已保证object velocity先写、particle固定LoadStore。TP-M1只消费Render12/PERF-MVP-341发布的matched current/previous instance ranges，在同一velocity artifact/pass写入，禁止第二history索引和CPU quad owner。见PERF-MVP-396及particle静态证据。
