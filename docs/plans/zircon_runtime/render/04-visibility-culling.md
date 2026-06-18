---
related_code:
  - zircon_runtime/src/graphics/visibility/mod.rs
  - zircon_runtime/src/graphics/visibility/context/mod.rs
  - zircon_runtime/src/graphics/visibility/culling/mod.rs
  - zircon_runtime/src/graphics/visibility/planning/mod.rs
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/visibility_static_index.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_history.rs
  - zircon_runtime/src/core/framework/render/relevance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/tests/render_framework_visibility_submit.rs
  - zircon_runtime/src/graphics/tests/render_product_advanced.rs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/zr_hzb.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/hzb_build.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/hzb_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/execute_hzb_build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/hzb.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/hzb_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/hzb_source_texture_view.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/HZB.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneCulling/SceneCulling.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp
  - dev/bevy/crates/bevy_camera/src/visibility/mod.rs
  - dev/bevy/crates/bevy_light/src/lib.rs
  - dev/bevy/crates/bevy_render/src/occlusion_culling/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/mip_generation/experimental/depth.rs
  - dev/bevy/crates/bevy_core_pipeline/src/mip_generation/experimental/downsample_depth.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/occlusion_culling.wgsl
  - dev/Fyrox/fyrox-impl/src/renderer/occlusion/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs
plan_sources:
  - .codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
---

# 计划 04:可见性与剔除(InitViews 对齐)

## 目标

把可见性从"BVH 视锥剔除一步到位"升级为 UE InitViews 式多级流水:
并行 frustum cull → relevance 计算(对象参与哪些 pass)→ HZB 遮挡剔除(GPU)。完成后:

1. 每个 view(主相机、shadow view、自定义 RT 相机)拥有独立的可见性结果与 relevance 位集。
2. relevance 直接驱动计划 02 的 per-phase 命令筛选,不再每 phase 重复过滤。
3. 上一帧深度构建的 HZB 金字塔同时服务遮挡剔除、SSR、SSAO 与计划 03 的 GPU instance 剔除。
4. 剔除统计(各级剔除数量、最终可见数)可被测试断言。

## 现状与差距

- `graphics/visibility/` 已有 context/culling/planning 拆分与 BVH 视锥剔除,但只有单级:没有 relevance 概念,phase 参与判断散落在 mesh draw 构建里;没有遮挡剔除;shadow view 的剔除与主 view 复用同一份结果而不是按光源视锥独立计算。
- 无 HZB:SSR 已存在却各自做屏幕空间 march,没有共享深度金字塔。
- 剔除在单线程执行,大场景 prepare 受限于 CPU。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp` | `FSceneRenderer::ComputeViewVisibility`:FrustumCull(分块并行)→ relevance(`FPrimitiveViewRelevance` 位集如何决定 pass 参与)→ occlusion 的级联顺序;多 view 的结果隔离 |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/HZB.cpp` | 深度金字塔构建 compute(`BuildHZB`):mip 链 reduce、与遮挡查询/SSR 的消费接口 |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneCulling/SceneCulling.cpp` | 层级场景剔除结构(implicit grid):静态对象的空间索引如何增量维护 |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp` | GPU 侧 per-instance 剔除与 indirect args 改写(与计划 03 GS-M4 衔接) |

次参考:`dev/bevy/crates/bevy_render/src/view/visibility/`(Rust 并行可见性系统的任务划分与 `VisibleEntities` 表达)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_camera/src/visibility/mod.rs` | per-view 可见性与 CPU/GPU 双路径 | 新版 bevy 的可见性已移到 bevy_camera:`VisibleEntities` 按 view 实体存结果;`check_visibility_cpu_culling` 与 `check_visibility_gpu_culling` 两条系统并存(RenderLayers 过滤 + `Frustum::intersects_obb`),对应本计划"CPU 视锥 + GPU 遮挡"的双路径拆分 |
| `dev/bevy/crates/bevy_light/src/lib.rs` | shadow view 独立剔除 | `check_dir_light_mesh_visibility` 写 `CascadesVisibleEntities`:每个 cascade 用光源视锥独立剔除、与主 view 结果隔离(VC-M1 shadow view 的 Rust 对照) |
| `dev/bevy/crates/bevy_render/src/occlusion_culling/mod.rs` | 遮挡剔除开关与 subview | `OcclusionCulling` 组件作 per-view opt-in gate、`OcclusionCullingSubview` 把 shadow cascade 表达为遮挡剔除子视图;与本计划 capability/gate 设计对照 |
| `dev/bevy/crates/bevy_core_pipeline/src/mip_generation/experimental/depth.rs` | HZB 构建 pass | `ViewDepthPyramid::new`(`max_mips` 推导 mip 数、逐 mip storage view)与 SPD 单 pass min-reduce 下采样的 wgpu 管线组织;对照 `HzbBuilder` 的尺寸/mip 公式与逐 mip dispatch 取舍 |
| `dev/bevy/crates/bevy_core_pipeline/src/mip_generation/experimental/downsample_depth.wgsl` | depth pyramid reduce compute | SPD 风格 workgroup 共享内存 reduce(mips 0-1、2-5 分段);Zircon `hzb_build.wgsl` 当前为逐 mip 2x2 reduce,升级单 pass 时以此为样板 |
| `dev/bevy/crates/bevy_pbr/src/render/occlusion_culling.wgsl` | HZB 保守判定内核 | `get_aabb_size_in_pixels` 选 mip、`get_occluder_depth` 取 2x2 texel 最远深度再比较——VC-M3 "bounds 投影 → mip 选择 → 保守判定"的 WGSL 直接样板(配合计划 03 的 `mesh_preprocess.wgsl` 看 instance_count 改写) |
| `dev/Fyrox/fyrox-impl/src/renderer/occlusion/mod.rs` | 遮挡剔除替代方案 | `OcclusionTester` 走 GPU occlusion query + 异步回读 + `GridCache` 空间缓存;与 HZB 重投影是不同方案,读它了解 query 路线的延迟/抖动代价,不照搬 |
| `dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs` | cascade 视锥切分与剔除 | `FrustumSplitOptions` 切 cascade、每 cascade 由光源投影矩阵建 `Frustum` 并独立收集 `RenderDataBundleStorage`;shadow view 构建顺序的最小实现 |

`PrimitiveRelevance 位集(FPrimitiveViewRelevance 等价物)` 无 Rust 同类参照(bevy 以 per-material queue 系统分流 phase,无统一 relevance 位集),实现时以 UE 为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:`zircon_runtime/src/graphics/visibility/` 内部升级,新增 `relevance/` 与 `occlusion/` 子模块;HZB 构建 pass 注册为内建 RenderFeature(经计划 01 graph)。

核心类型:

- `ViewVisibilityContext`:按 view 隔离的可见性结果(主相机 / 每个 shadow cascade / 自定义 RT 相机各一份);view 集合由计划 09 的相机管理提供。
- `PrimitiveRelevance` 位集:`opaque/alpha_mask/transparent/casts_shadow/needs_velocity/needs_distortion/...`;在 extract 标记 + 材质域上一次性计算,缓存于静态对象(变更失效与计划 02 共用 generation)。
- 并行 frustum cull:对 extract 实例数组按块切分,rayon 并行;输出可见索引 + relevance 过滤后的 per-phase 候选集,直接喂给计划 02 的 pass processor。
- `HzbBuilder`:上一帧 scene depth → mip 金字塔(compute reduce);本帧用重投影保守测试,遮挡统计通过独立 GPU stats buffer readback 汇入 diagnostics。HZB 资源经 graph 声明为持久资源。
- GPU 遮挡剔除:HZB + 实例 bounds(读计划 03 GpuScene)→ compute 改写 indirect args 的 instance_count;CPU 路径保留 BVH 视锥结果作为回落。

## 里程碑

### VC-M1 relevance 与并行视锥剔除

实施切片:
1. `PrimitiveRelevance` 计算与缓存;mesh draw 构建中的零散 phase 判断删除,统一改读 relevance。
2. frustum cull 分块并行化;per-view 结果隔离(shadow view 独立剔除)。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`
- `cargo test -p zircon_runtime visibility --locked`、`cargo test -p zircon_runtime mesh --locked`、`render_product` 回归
- 验收证据:shadow view 剔除数与主 view 不同(光源背后对象只进 shadow);relevance 断言用例(透明物不出现在 opaque 候选)。

当前落地进度(2026-06-12):

- 已新增 `core/framework/render/relevance.rs`,提供 renderer-neutral `PrimitiveRelevance` 位集。当前位集覆盖 render-layer match、main-view、opaque/alpha-mask/transparent、depth-prepass、shadow-caster、Core3d deferred geometry、dynamic opaque-like motion-vector candidate。
- 已新增 `graphics/visibility/declarations/visibility_relevance_entry.rs`,并在 `VisibilityContext::primitive_relevance` 中产出每 entity 的 relevance。`VisibilityContext` 已删除旧的 `visible_entities`、`culled_entities`、`visible_batches` 平铺字段;主视图可见实体、剔除实体和可见 batch 现在通过 `main_view_*` 派生方法从 `FrameVisibility + batches` 读取,history/upload plan 字段继续保留为独立计划输出。
- 已新增 `graphics/visibility/culling/parallel_frustum.rs`,对线性 `{ entity, VisibilityBounds }` 候选数组使用确定性 serial/parallel helper。当前阈值以下走串行,大场景走 rayon `par_iter`,返回顺序保持输入序。
- `is_mesh_visible.rs` 已收敛为 `is_bounds_visible(bounds, camera)` bounds 级内核。这让 frustum culling、shadow view 和后续静态空间索引可以共用同一剔除入口。
- `collect_batching_result.rs` 已改为同时计算 relevance 与 frustum 结果,并复用同一份预计算 bounds 写入 `VisibilityBvhInstance` 与 history entries;主视图可见性现在要求 `relevance.main_view()` 且 frustum 可见,因此相机 `RenderLayerSet` 会参与 `visible_entities`/`visible_batches` 过滤。layer mismatch 的 opaque-like mesh 仍保留 `shadow_caster` relevance,为后续 shadow view 独立剔除保留语义。
- 已新增 `graphics/visibility/view_context/mod.rs`,落地主相机版 `FrameVisibility` / `ViewVisibilityContext` / `VisibilityViewKey` / `ViewCullingStats`。`VisibilityContext::frame_visibility` 现在保存稳定的 frame primitive index space(`entities`/`bounds`/`relevance`)和主视图 visible indices/statistics,旧主视图平铺字段已经不再存储。
- 已新增 `graphics/visibility/view_context/build_views.rs`,并继续扩展为 Plan 05 atlas 所需的 shadow view key 集合:方向光生成 `ShadowCascade { light, cascade }`(shadow-casting 方向光 4 个 cascade key,legacy/default-shadow 路径保留 cascade 0),point 光生成 6 个 `ShadowPointFace { light, face }`,spot 光生成 `ShadowSpot { light }`。方向光 shadow view 现在使用 Plan 05 `shadow/cascade.rs` 的 split 与 camera frustum slice bounds 合成每级联正交 light camera;point/spot shadow view 仍由光源位置/方向/range 合成透视 camera。所有 shadow view 复用 `mesh_frustum_visibility(...)`,并以 `PrimitiveRelevance::shadow_caster()` 作为 relevance gate,因此主相机 layer mismatch 的 opaque-like mesh 仍可进入 shadow view。
- `ViewportRenderFrame` 现在携带 `FrameVisibility` sideband;`submit_frame_extract` 与 direct runtime-frame submit 两条路径都会把 `FrameSubmissionContext.visibility_context().frame_visibility` 传到 renderer。`FrameSubmissionContext::view_visibility(key)` 已提供 submit-time per-view 访问口。`build_mesh_draws(...)` 将 `FrameVisibility` 映射回 `MeshDraw`,并把 primitive relevance、main-view visibility、shadow-view visibility 透传到 `MeshBatchRef`。
- `MeshPassProcessor` 现在使用 relevance/view visibility 作为 phase gate:depth/opaque/alpha/transparent/velocity 需要 main-view 可见且对应 relevance 成立,shadow pass 需要 shadow-caster relevance 和 shadow view 可见。旧 queue/profile 仍负责材质 phase 与 pipeline variant 选择,但不再单独决定当前 view 是否参与该 pass。
- Hybrid GI 与 Virtual Geometry planning 已不再直接读取 `BatchingResult.visible_entities`;调用侧现在从 `FrameVisibility::main_view_visible_entity_set()` 派生主视图实体集合。`BatchingResult.visible_batches` 已删除,`construct.rs` 从 `batches + main_view_visible_entity_set()` 派生 `visible_batches`、`visible_instances`、draw commands 和 GPU instancing candidates。Virtual Geometry debug 的 node/cluster cull snapshot 也通过 `FrameSubmissionContext::view_visibility(MainCamera)` 读取相机,与运行时 view 权威保持一致。
- `RenderStats` 已新增 `last_visibility_view_count`、`last_visibility_input_count`、`last_visibility_layer_filtered_count`、`last_visibility_frustum_culled_count`、`last_visibility_occlusion_culled_count`、`last_visibility_visible_count`。`update_base_stats(...)` 从 `FrameVisibility.views[*].stats` 聚合这些字段,`render_stats_store::product` 记录到 `render.visibility.*`,运行时诊断 fixture 也覆盖这些路径。当前 occlusion 统计保持 0,等待 VC-M3 HZB/GPU occlusion 写入同一统计面。
- 2026-06-18 已接入 CO-M1/VC-M1 custom-target visibility payload bridge:`SortedRenderCamera` 现在保留 `ViewportCameraSnapshot`,scene-backed extract 会把选中主相机层与非 PrimarySurface scene cameras 的层合并到 mesh/sprite 候选集合,`FrameVisibility::from_frame_views(...)` 会为 Texture/Headless scene cameras 构建 `VisibilityViewKey::CustomTarget { camera }`。这只闭合 CPU visibility payload;WGPU 多相机 render loop、custom target 输出链、post/history/lighting per-camera ownership 仍属计划 09 后续。
- 尚未完成:custom render-target camera 的实际渲染提交链,以及 GPU 遮挡剔除 RenderDoc 验收仍按后续 VC-M3/CO-M1 推进。Directional multi-cascade view key 和 camera frustum slice bounds 已接入,后续阴影精度风险转为 caster expansion/receiver slice 覆盖、receiver 收束前的抓帧对拍与多光源稳定性验收。
- 验证状态: touched Rust 文件 `rustfmt --check` 通过;`git diff --check` 对本切片文件通过(仅 Git 行尾转换提示);`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` 在 relevance/frustum、bounds-kernel、主视图 `FrameVisibility`、shadow view 构建、mesh-pass relevance 消费、main-view planning accessor 迁移、visibility stats/diagnostics 接入后通过(现有 warning set)。focused lib-test 尚无结果:一次被共享 lib-test 的非 render 插件测试源阻塞(`runtime_plugin_package_manifest.rs` 缺少 `RuntimePluginDescriptor::with_target_mode`),最新一次在 304 秒编译窗口内超时。
- 2026-06-13 shadow atlas view-key 扩展后,`cargo fmt --all -- --check`、scoped `git diff --check` 与 core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` 通过。随后方向光 cascade slice bounds follow-up 复用同一 target dir:shared lib-test `--no-run` 编译完成;`cargo test ... render_shadow_ -- --nocapture` 通过 27 个 shadow 过滤测试;`cargo test ... visibility_context_builds_shadow_views_for_atlas_light_slots -- --nocapture` 通过 1 个 visibility 过滤测试。带 `--exact` 的同名尝试因未匹配完整测试路径运行 0 个测试,不作为覆盖证据。

### VC-M2 HZB 构建 pass

实施切片:
1. depth pyramid compute pass(内建 feature,graph 声明);持久 HZB 资源。
2. SSR/SSAO 切换为消费 HZB(删除各自的私有深度采样准备)。

测试阶段:
- `cargo test -p zircon_runtime render_graph --locked` + post 系列回归
- 验收证据:RenderDoc 抓帧可见 HZB mip 链;SSR 行为不回退。

当前落地进度(2026-06-13):

- 已新增 `graphics/visibility/occlusion/hzb_builder.rs`,作为 WGPU-free HZB 尺寸/mip/批次推导权威。当前公式为有效 render size 取各轴 `next_power_of_two >> 1`,下限 1;`1923x1081` 会得到 `1024x1024`、`11` 个 mip、`3` 个 4-mip reduce 批次。
- 已新增 `BuiltinRenderFeature::Hzb` 与 `feature_descriptors/hzb.rs`。默认 forward-plus/deferred 3D pipeline 现在把 `hzb-build` 放在 shadow 之后、clustered lighting 之前;pass 读 `scene-depth`,以 storage 写 `hzb-furthest`,executor 为 `visibility.hzb-build`,队列声明为 `AsyncCompute` 并可按 capability fallback 到 graphics。
- `compile.rs` 已把 `hzb-furthest` materialize 为 `Rgba16Float`、HZB builder 尺寸、完整 mip chain。`RenderGraphComputeWorkloadDispatchContext` 已新增 `HzbFurthest` extent,所以 graph execution audit 会用 HZB 资源大小而不是 viewport/cluster grid 计算期望 dispatch groups。
- runtime executor registry 已注册 `visibility.hzb-build`。executor 现在会校验 `scene-depth` 与 `hzb-furthest` 已绑定,按 HZB size 记录 `zircon-hzb-build-pipeline` 的 graph audit dispatch 与 storage write evidence,并通过 `post_process/shaders/hzb_build.wgsl` 对每个 mip 执行实际 WGPU compute reduce。mip0 从 scene depth 做 2x2 furthest-depth reduce,后续 mip 从上一层 HZB view 做 2x2 reduce。
- `ScenePostProcessResources` 已持有 HZB bind group layout、`HzbParams` uniform buffer、`zircon-hzb-build-pipeline` compute pipeline、mip0 用的 1x1 fallback HZB source view,并通过 `execute_hzb_build_mip(...)` 在 graph 执行时创建 per-mip bind group。当前 graph audit 仍保留单条聚合 dispatch record,真实 command encoder 则逐 mip dispatch,以便 target mip 可以独立绑定为 storage view。
- Frame history 已增加 `FrameHistorySlot::HzbFurthest`、`history.previous.hzb-furthest` 资源名、运行时 HZB history texture、mip-chain copy 到 history 的帧尾路径,并在 `RenderHistoryCopyReport`/diagnostics 中记录 `render.history.copy.hzb_furthest_copied`。
- `RenderStats` 已新增 `last_hzb_mip_count` 与 `last_hzb_graph_executed_pass_count`;`render_stats_store::product` 记录 `render.hzb.mip_count` 和 `render.hzb.graph_executed_pass_count`,运行时诊断 fixture 覆盖这些路径。
- SSAO descriptor 现在显式读 `hzb-furthest`;`record_ssao_to_resources(...)` 和 `execute_ssao(...)` 绑定共享 HZB full-mip view,`ssao.wgsl` 在局部 depth/normal AO 之外读取 HZB mip 1 做保守的大尺度 depth delta 调制。旧 depth/normal/previous AO/history 输出路径保持不变。
- SSR resolve pass 现在读共享 `hzb-furthest` 替代 `postprocess.screen-space-reflection.depth-pyramid` 与 `.depth-pyramid.coarse`。graph descriptor、`PostProcessStackDescriptor`、plugin test fixtures、pipeline compile tests、runtime executor bridge 都已切换到 HZB。`post_process_screen_space_reflection.wgsl` 的 binding 23 继续作为兼容名承载 HZB full-mip view;当 HZB 只有 1 个 mip 时 fallback 到 mip0,不再采样 binding 25 的私有 depth-coarse 纹理。
- SSR 私有 depth pyramid 生产链已删除:`PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid`/`DepthPyramidCoarse`、`PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID`/`_COARSE`、对应 feature pass、executor 注册、runtime record bridge、execute module、pipeline bundle 字段与 depth-pyramid pipeline 创建文件均已移除。反射颜色 pyramid 仍保留,因为它是 SSR 颜色粗糙度缓存,不是深度准备。
- 验证状态:HZB 相关 Rust 文件 `rustfmt --edition 2021 --check` 通过;切片文件尾随空白扫描为 clean;`git diff --check -- <HZB scoped files>` 退出 0(仅 Git LF→CRLF 提示)。`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 已在 HZB builder/descriptor/resource、dispatch audit、history mip-chain copy、diagnostics 接入后通过(现有 warning set),在真实 WGSL HZB build shader、pipeline、bind group、params buffer、fallback source view、per-mip dispatch 接入后再次通过。SSR/SSAO 共享 HZB 消费迁移、私有 SSR depth pyramid 代码删除后,隔离验证目录 `E:\cargo-targets\zircon-render-main-chain-verify` 的同一 `cargo check` 通过,最新一次报告 65 个现有 warnings。`cargo test -p zircon_runtime --lib hzb --locked ...` 尚未跑到 filtered HZB 测试,因为共享 lib-test 目标先被无关插件测试 `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs` 的缺失 bridge 类型导入挡住。
- 尚未完成:VC-M2 的 RenderDoc mip 链验收和 SSR/SSAO 视觉回归验收。共享 HZB 现在是 furthest-only `Rgba16Float` 链,旧 SSR 私有 depth pyramid 曾在 `.rg` 表达 min/max range;当前迁移以 HZB depth 同值写入 RGB,需要抓帧确认反射 hit gating 没有质量回退。

### VC-M3 GPU 遮挡剔除(依赖计划 03 GS-M4)

实施切片:
1. HZB 重投影遮挡 compute:实例 bounds 投影 → mip 选择 → 保守判定 → 改写 indirect instance_count。
2. capability gate 与 CPU 回落;遮挡剔除数进 RenderStats。

测试阶段:
- `cargo test -p zircon_runtime visibility --locked` 与 gpu_scene 范围测试
- 验收证据:遮挡场景(墙后大量实例)indirect 实际 instance 数下降(统计断言);画面无漏剔/误剔(对拍)。

当前落地进度(2026-06-13):

- `RenderGraphComputeWorkload` 已新增 `IndirectArgs` dispatch extent,`RenderGraphComputeWorkloadDispatchContext` 会用本帧 mesh indirect args 数量审计 `hzb-occlusion-cull` 计划 dispatch。零 args 场景会被审计为零 dispatch groups。
- `MeshIndirectDrawExecution` 的 args buffer 已加入 `STORAGE` usage,并保留 `args_count` / `total_instances` 元数据。`MeshPassIndirectDrawExecutions` 与 `RenderPassMeshCommandLists` 现在能汇总 occlusion cull candidate arg/instance count,作为 graph audit 和 culler report 输入;同时提供 HZB occlusion 后的 phase-local indirect args readback helper,用于解析真实 WGPU replay args buffer 中 `instance_count == 0` 的条目数、剩余 instance 总量,以及 compact replay draw-count buffer 中实际提交的 compact draw 数。
- `graphics/visibility/occlusion/mod.rs` 已新增 `HzbOcclusionPhase::SingleFrameReproject`、预留 `TwoPhaseRetest`,以及 WGPU-free `HzbOcclusionCullReport`/`HzbOcclusionCullReadbackStats`/`HzbOcclusionIndirectArgsReadbackSummary`。V1 当前只实现单阶段 previous-frame HZB 重投影,不做当帧 retest/redraw。
- 已新增 `scene_renderer/hzb/` runtime 子模块。`HzbOcclusionCuller` 创建 `zircon-hzb-occlusion-cull-pipeline`,绑定 scene group0、previous HZB/params/indirect args/stats buffer group1、GpuScene group3,并按每个 mesh phase 的 indirect args buffer dispatch。每个 phase 的 cull params 现在通过 command encoder 内的 COPY_SRC upload 顺序写入 uniform buffer,避免多 phase 共享 params buffer 时所有 dispatch 读到最后一次 `args_count`。shader 读取 GPUScene instance/primitive bounds,用 `scene.previous_view_proj` 投影到上一帧 HZB,按屏幕半径选 mip,若 batch 内没有保守可见 instance,将对应 indexed-indirect args 的 `instance_count` 写 0,并用 storage atomic 统计 tested/culled arg 与 instance 数。
- `feature_descriptors/hzb.rs` 现在在 `DepthPrepass` stage 增加 `hzb-occlusion-cull` pass,executor 为 `visibility.hzb-occlusion-cull`,队列声明 `AsyncCompute`,读取 `history.previous.hzb-furthest`,把 execution-owned compaction metadata 声明为 external read,把 indirect args、visible-instance remap、draw-count、stats 声明为 external storage write,并携带 `RenderGraphComputeWorkload::indirect_args(...)`。原 `hzb-build` 保持在 AmbientOcclusion stage 构建本帧 HZB。
- runtime executor registry 已注册 `visibility.hzb-occlusion-cull`。`RenderPassGpuExecutionContext` 现在可持有 `HzbOcclusionCuller`,并在 depth-prepass graph stage 使用 previous HZB;无上一帧 HZB 时使用 post-process white fallback texture view,保持第一帧不失败。scene uniform bind group layout 已允许 compute 可见。
- `RenderCapabilitySummary::hzb_occlusion_culling_supported()` 已作为 VC-M3 runtime gate,要求 storage buffer 与 03 GS-M4 的 GPU-driven submission 条件同时成立。`compile_options_for_profile(...)` 会从实际 backend capability 派生 `enable_hzb_occlusion_culling`,运行时不再盲目把遮挡剔除 pass 放进 headless/低能力 backend 的 compiled graph。
- `RenderPipelineAsset::compile_with_options(...)` 现在只在 HZB occlusion gate 关闭时过滤 `visibility.hzb-occlusion-cull`,并保留 `hzb-build` 与 `hzb-furthest` history/resource 链。默认 asset compile 仍保留完整 HZB feature,用于 graph 资源验证和高能力 backend 预期。
- headless WGPU 当前以空 feature 集初始化,因此不满足 multi-draw indirect / first-instance / storage-driven 条件;运行时 compiled graph 会移除 `hzb-occlusion-cull`,CPU relevance/frustum 可见性结果保持最终结果,HZB build 仍继续为 SSR/SSAO 和下一步高能力路径服务。
- 默认 forward-plus/deferred pass 期望已同步:新 pass 出现在 `motion-vector-clear` 后、shadow/HZB build 前。graph execution record 覆盖了 `"mesh.indirect-args"`、`"mesh.visible-instance-index"`、`"mesh.indirect-draw-count"` 与 `"visibility.hzb-occlusion-stats"` storage side effects 和 indirect args workload audit。
- `HzbOcclusionCullReport` 现在从 `RenderPassGpuExecutionContext` 进入 `RenderGraphExecutionRecord`,再通过 `SceneRenderer::last_hzb_occlusion_cull_report()` 汇入 `RenderStats` 与产品 diagnostics。统计面包括 `last_hzb_occlusion_reported`、candidate arg/instance count、dispatch group count、dispatched phase count、history availability、GPU stats readback availability、tested arg/instance count、culled arg/instance count,以及 indirect args readback availability、readback arg count、compacted draw count、zero-instance arg count、remaining instance count;headless fallback 或未执行遮挡 pass 的帧会显式归零这些字段。带 stats readback 的 HZB report 会把 `RenderStats.last_visibility_occlusion_culled_count` 覆盖为 GPU culled instance 数。
- 产品 diagnostics 现在记录 `render.hzb.occlusion.reported`、`candidate_arg_count`、`candidate_instance_count`、`dispatch_group_count`、`dispatched_phase_count`、`history_available`、`readback_available`、`tested_*`、`culled_*`、`indirect_args_readback_available`、`readback_arg_count`、`compacted_draw_count`、`zero_instance_arg_count` 与 `remaining_instance_count`。其中 candidate/dispatch 是执行元数据,stats readback/tested/culled 来自 GPU stats buffer,indirect args readback 系列来自 HZB pass 后真实 phase-local replay args buffer 与 draw-count buffer snapshot。
- 2026-06-14 WGPU storage-buffer limit gate 已收束:新增 `graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE = 10`,代表当前 `hzb-occlusion-cull` compute pipeline 在 scene/HZB/GPUScene 三组 bind group 上合计需要的 per-stage storage-buffer 绑定数。真实 offscreen WGPU backend 会在 adapter 支持时申请该 limit;不支持时仍创建设备,但 `SceneRendererCore` 不构造 `HzbOcclusionCuller`,runtime capability summary 记录 `max_storage_buffers_per_shader_stage`,`compile_options_for_profile(...)` 关闭 HZB occlusion pass 并保留 HZB build/history 路径。产品 diagnostics 同步暴露 `render.capability.max_storage_buffers_per_shader_stage`,用于解释为何同一 pipeline 在低 limit 设备上走 CPU fallback。
- 源码覆盖已新增 capability policy、profile compile options gate、compiled graph 过滤 HZB occlusion 但保留 HZB build、headless runtime 不执行 `hzb-occlusion-cull`、execution record 保存 cull report、stats report/reset/readback helper、indirect args snapshot summary helper、compact draw-count summary helper、HZB diagnostics series、params encoder-order upload source guard 的断言。`hzb_occlusion_culls_fully_hidden_indirect_args_on_wgpu` 现在用真实 offscreen WGPU device 构造前景/墙后两个 one-instance indirect args,执行同一 `HzbOcclusionCuller` shader 后回读 stats buffer 与 args buffer,断言墙后 args word1 变 0、前景 args 保持 1。2026-06-13 已新增 `render_product_advanced.rs::render_product_hzb_occlusion_wall_scene` 产品级 source assertion:使用产品 `WgpuRenderFramework` 连续提交同一墙 + 64 个墙后静态实例场景,断言上一帧 HZB、stats/readback、indirect args readback summary、culled instance 覆盖数、compact draw-count 非零且不超过 readback args 容量、zero-instance arg 与 remaining instance 下降。该用例现已扩展为 capability-gated 产品对拍:同一场景在 HZB occlusion 支持开启与关闭路径各渲染两帧,对比第二帧 captured RGBA 完全一致,确保遮挡剔除只省工作不改像素。
- 2026-06-13 已补上 UE 式 clear + atomic compact ABI 与 replay:`mesh_pass/indirect_compaction.rs` 定义 `IndirectCompactionPlan` 与 `IndirectCompactionBatchMetadata`,从 phase-local `IndexedIndirectArgs` 生成 source arg index、visible remap base、source first_instance、source instance_count 的 per-arg metadata,并按 source instance_count 前缀预留 visible-instance index remap 容量。该切片还定义 metadata buffer、visible instance index buffer、draw count buffer 的字节大小常量和 unused-instance sentinel;`mesh_pass/indirect_compaction_resources.rs` 现在为每个 `MeshIndirectDrawExecution` 创建匹配的 WGPU metadata storage、visible-instance-index storage/copy buffer、storage/copy/indirect draw-count buffer 与 compacted indirect args buffer。`HzbOcclusionCuller` 在每个 phase dispatch 前清空 visible-instance remap、draw-count 与 compacted args 输出,descriptor/dispatch record 已声明这些 execution-owned external 资源。HZB shader 按 metadata 写 visible remap、compacted args 与 per-batch draw-count,mesh replay 在 opaque、alpha-mask、velocity phase 使用 `multi_draw_indexed_indirect_count` 和 visible-remap group3 读取 compact 结果。
- 验证状态:VC-M3 touched Rust 文件 `rustfmt --edition 2021 --check` 通过;`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3 --message-format short --color never` 通过(现有 warning set)。尝试 `cargo test -p zircon_runtime --lib hzb_occlusion --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-tests -- --nocapture` 在 904 秒后超时,该测试编译进程已停止,未返回 filtered test 结果。本次 capability gate/fallback follow-up 的 `rustfmt --edition 2021 --check` 通过;`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-gate --message-format short --color never` 通过,报告 66 个既有 warnings。report surface follow-up 的 `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-report --message-format short --color never` 通过,仍为 66 个既有 warnings;`cargo test -q -p zircon_runtime --lib update_hzb_occlusion_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-report-tests -- --test-threads=1 --nocapture` 通过 2 个过滤测试。exact stats readback follow-up 的 `rustfmt --edition 2021 --check` 通过;`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-readback --message-format short --color never` 通过,报告 66 个既有 warnings。`cargo test -q -p zircon_runtime --lib update_hzb_occlusion_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-readback -- --test-threads=1 --nocapture` 一次在输出 3/3 通过后因共享 lib-test 后续处理超时,重试则被无关插件半成品测试目标阻塞:`host_api_adapter.rs` 缺 `PluginInterfaceManifest` 导入,继续补临时导入后又被 `native_plugin_live_host.rs` 缺 `bridge_lifecycle` 模块阻塞。indirect args readback summary follow-up 的 `rustfmt --edition 2021 --check` 与 scoped `git diff --check` 通过(仅 Git LF→CRLF 提示);`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-args-readback-coremin --message-format short --color never` 已编译到 `zircon_runtime` 后被无关任务模块阻塞:`zircon_runtime/src/core/runtime/tasks/mod.rs` re-export 了私有 `JobSchedulerDiagnosticsState`。2026-06-13 indirect compaction ABI follow-up 的三个 touched Rust 文件 `rustfmt --edition 2021 --check` 通过,`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-coremin --message-format short --color never` 通过并报告 69 个既有 warnings;`cargo test -p zircon_runtime --lib indirect_compaction --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-coremin --message-format short --color never -- --test-threads=1 --nocapture` 在 424 秒后仍在编译共享 lib-test 目标而超时,本切片目标目录下残留 cargo/rustc 进程已停止,未返回 filtered test 结果。上述非 render 阻塞未在本切片中修复。
- 2026-06-13 indirect compaction resource follow-up 验证:`rustfmt --edition 2021 --check` 通过 `mesh_pass/indirect_compaction.rs`、`mesh_pass/indirect_compaction_resources.rs`、`mesh_pass/indirect_draw_execution.rs`、`mesh_pass/mod.rs`;scoped `git diff --check` 与尾随空白扫描通过(仅 Git LF→CRLF 提示);`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never` 通过,报告 68 个既有 warnings。`cargo test -p zircon_runtime --lib indirect_compaction_resources --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never -- --test-threads=1 --nocapture` 编译共享 lib-test 目标时被无关 UI 测试源阻塞:`zircon_runtime/src/ui/component/state_reducer/keyboard.rs:311` 触发 `String: Borrow<&str>` trait bound error;阻塞前未返回 render/visibility 错误。本切片没有修复该 UI 问题。
- 2026-06-13 compaction clear/resource declaration follow-up 状态:HZB descriptor 已把 compaction metadata、indirect args、visible-instance remap、draw-count 与 stats 声明为 execution-owned external resources;runtime dispatch record 已把 indirect args、visible-instance remap、draw-count 与 stats 作为 storage write evidence;`HzbOcclusionCuller` 已在每个非空 phase dispatch 前清空 visible-instance remap 与 draw-count 输出。source tests 覆盖 external resource 声明、dispatch write list 和 clear-before-dispatch 顺序。`rustfmt --edition 2021 --check` 通过本切片 touched Rust 文件;scoped `git diff --check` 与尾随空白扫描通过(仅 Git LF→CRLF 提示);`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never` 通过,报告 68 个既有 warnings。`cargo test -p zircon_runtime --lib hzb_occlusion_culler_clears_compaction_outputs_before_culling_dispatch --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never -- --test-threads=1 --nocapture` 在 304 秒后仍在编译 shared lib-test 目标而超时;该 target-dir 下残留 cargo/rustc 进程已停止,未返回 filtered test 结果。
- 2026-06-13 compact draw-count diagnostics follow-up 状态:HZB compact replay 的 phase-local readback 现在同时复制 replay args buffer 与 compaction draw-count buffer,`HzbOcclusionIndirectArgsReadbackSummary` 新增 compacted draw count,并映射到 `RenderStats.last_hzb_occlusion_compacted_draw_count` 与产品诊断 `render.hzb.occlusion.compacted_draw_count`。`render_product_hzb_occlusion_wall_scene` source assertion 已增加 compact draw-count 非零且不超过 readback args 容量的产品级断言。`rustfmt --edition 2021 --check` 通过本切片 touched Rust 文件;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` 通过,报告 68 个既有 warnings。`cargo test -p zircon_runtime --lib --no-default-features --features core-min hzb_occlusion_indirect_args_summary_saturates_totals --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never -- --nocapture` 两次仍在编译共享 lib-test 目标时超时(180 秒、600 秒),残留 cargo/rustc 进程已停止,未返回 filtered test 结果。RenderDoc 当前无运行实例,未执行抓帧验证。
- WGPU local wall/front follow-up 验证:`rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs` 通过;scoped `git diff --check` 与尾随空白扫描通过(仅 Git LF→CRLF 提示);`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-wgpu-local-coremin --message-format short --color never` 通过,报告 74 个既有 warnings。`cargo test -p zircon_runtime --lib hzb_occlusion_culls_fully_hidden_indirect_args_on_wgpu --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-wgpu-local-test --message-format short --color never -- --test-threads=1 --nocapture` 10 分钟超时,未返回 filtered test 结果;`cargo test -p zircon_runtime --lib hzb_occlusion_uploads_phase_params_in_encoder_order --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-wgpu-local-coremin --message-format short --color never -- --test-threads=1 --nocapture` 15 分钟超时,超时时仍在编译 `zircon_runtime` lib-test 目标。两次验证遗留的 cargo/rustc 进程均按 target-dir 清理。
- 2026-06-13 product wall-scene source assertion follow-up:`rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_advanced.rs` 通过;scoped `git diff --check` 通过(仅 Git LF→CRLF 提示)。`cargo test -p zircon_runtime render_product_hzb_occlusion_wall_scene --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc4-product-static-diagnostics` 编译到共享 lib-test 目标后被无关插件测试源阻塞:`zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs:265:30` 调用私有 `interface_exports_owned_by`。阻塞前未返回 render visibility 错误。后续像素对拍 source assertion 扩展完成后,`rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_advanced.rs`、scoped `git diff --check` 和尾随空白扫描通过;由于 editor UI 与 plugin reload 两条活跃 Cargo lane 正在编译 `zircon_runtime`,未启动第三条 Cargo 写入。
- 尚未完成:VC-M3 的产品墙场景 clean rerun 与 RenderDoc 验收仍待做。当前 capability-gated CPU fallback compiled-graph 路径、per-stage storage-buffer limit gate、report surface、exact stats readback 聚合、indirect args/readback compact draw-count summary、局部真实 WGPU 墙后 args 改写测试、产品级墙场景 source assertion 和遮挡关闭 captured-RGBA 对拍 source assertion 已收束。2026-06-14 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-hzb-gate-coremin --message-format short --color never` 通过并报告 66 个既有 warnings;`cargo test -p zircon_runtime --lib render_framework_stats_report_shadow_atlas_graph_execution --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-hzb-gate-coremin --message-format short --color never -- --exact --test-threads=1 --nocapture` 在 shared lib-test 编译/链接 20 分钟后超时,未返回 filtered test 结果,本 target-dir 残留 cargo/rustc 进程已停止。2026-06-18 focused rerun 修正 HZB culler clear-before-dispatch source contract 后,`cargo test -p zircon_runtime --lib hzb_occlusion_culler --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-culler-contract-0618 --message-format short --color never -- --test-threads=1 --nocapture` 通过 6/6;同一热 lib-test 二进制 `hzb_occlusion_dispatch` 通过 4/4、`indirect_compaction` 通过 8/8、`mesh_indirect_draw_execution` 通过 3/3、`multi_draw_indexed_indirect` 通过 1/1、`hzb_occlusion_cull_declares_execution_owned_external_buffers` 通过 1/1;热 target-dir `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-culler-contract-0618` 通过(既有 warning set)。

### VC-M4 静态场景空间索引增量化

实施切片:
1. BVH/网格索引对静态对象增量更新(增删改单条更新,不全量重建),对齐 SceneCulling 的增量维护思路。

测试阶段:
- `cargo test -p zircon_runtime visibility --locked`(增删对象后索引一致性断言)
- 验收证据:静态场景帧间索引重建次数为 0。

当前落地进度(2026-06-13):

- 已新增 `graphics/visibility/static_index/mod.rs`,作为 WGPU-free 的静态对象 uniform-grid core。它复用现有 `VisibilityBvhInstance` bounds 行和 `VisibilityBvhUpdatePlan` diff,提供 `rebuild(...)`、`apply_update_plan(...)` 与 `query_bounds(...)` 三个核心入口。
- `VisibilityStaticIndexReport` 记录 full rebuild 次数、incremental update 次数、insert/update/remove 数量、indexed entity 数量、occupied cell 数量和主视图静态预筛证据。主视图证据包括是否启用预筛、静态输入数和 grid 粗筛候选数,并已映射到 `RenderStats.last_visibility_static_index_main_view_*` 与 `render.visibility.static_index.main_view_*` 产品诊断路径。
- 已接入 renderer persistent visibility state:每个 `ViewportRecord` 保存上一帧 `VisibilityStaticIndex`,`resolve_viewport_record_state(...)` 把上一帧索引交给 `VisibilityContext::from_extract_with_history_and_static_index(...)`,`record_history(...)` 在提交成功后写回本帧索引。主视图 culling 已接入 static/dynamic split,静态实例数达到 10_000 时先走 grid 粗筛,再进入既有 `mesh_frustum_visibility(...)` 精筛。
- 已新增局部一致性与状态测试:`visibility_static_index_incremental_update_matches_full_rebuild_queries` 覆盖增/删/移动对象后 query 结果与全量重建一致,`visibility_static_index_full_rebuild_strategy_replaces_existing_rows` 覆盖 full-rebuild 策略替换旧行,`visibility_context_reuses_static_index_without_frame_rebuild`/`visibility_context_uses_static_index_prefilter_above_threshold` 覆盖持久复用与 10_000 阈值预筛。`render_framework_reuses_static_index_and_reports_main_view_prefilter` 已新增渲染框架级静态大场景断言,连续两帧提交 10_001 个静态 mesh,要求第二帧 `full_rebuild_count == 0` 且 main-view candidate 数低于 static input 数。
- 2026-06-15 复验:`render_framework_reuses_static_index_and_reports_main_view_prefilter` 已在 core-min lib-test 路径通过,确认 debug assert 修正后第二帧可复用 persistent static index,且 main-view prefilter candidate 低于 static input。
- 验证状态:此前 `rustfmt`、core-min `cargo check`、`static_index` focused tests 与 `render_product_diagnostics_record_visibility_stats` 已通过;runtime diagnostics filtered test 仍被无关 light 诊断期望阻塞。本轮 main-view 预筛诊断 follow-up 中,限定 `rustfmt --edition 2021 --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc4-product-static-diagnostics --message-format short --color never` 曾在渲染统计/诊断新增字段后通过,报告 68 个既有 warnings。`update_visibility_static_index_stats_records_latest_report` 过滤测试通过 1 个测试,`render_product_diagnostics_record_visibility_stats` 通过 1 个过滤测试。`render_framework_reuses_static_index_and_reports_main_view_prefilter` 首次运行暴露了 mesh queue 统计层旧 debug assert:可见性预筛后 depth-prepass command count 为 1,而源 draw census 仍为 10_001;该断言已改为 command count 不超过对应源 draw census。2026-06-15 修正后的 scoped rerun 已通过 1 个过滤测试,不再被 plugin bridge 编译漂移阻塞。
- 2026-06-15 广义 visibility 直跑:复用已编译 core-min lib-test 二进制执行 `visibility --test-threads=1 --nocapture` 时先暴露 `graphics_surface_keeps_viewport_frame_and_icon_source_internal` 的源码形状守卫,要求 graphics root 保留独立 `pub(crate) use types::ViewportRenderFrame;` 内部边界。`graphics/mod.rs` 已拆分 `ViewportRenderFrame` 与 `ViewportRenderOutputTarget` 的 crate-private re-export;精确失败用例通过后,同一二进制 `visibility` filter 重跑通过 59/59。
- 尚未完成:完整 render-product 回归与 VC-M2/VC-M3/VC-M4 RenderDoc 可视化验收仍待后续。

## 工程落地细化

本章是计划 04 的实施权威(index.md §8 第 7 条)。bind group 槽位、GPU 数据布局、WGSL include 命名、RenderQueueValue、sort_key、测试命名全部直接引用 index.md §8,本章不重定义。跨计划契约名按 index 口径原样使用:01 `RgTextureHandle`/`RgBufferHandle`/`RgResourceResolver`/`TransientResourcePool`,02 `MeshDrawCommand`/`MeshPassProcessor`,03 `GpuScene`/`IndirectDrawBatcher`,09 `RenderLayer`,10 `RendererCommon`/`LodGroup`。

### 模块与文件落点

先给两个结论性裁决,后续落点据此展开:

1. **BVH 取舍:切线性数组,不保留"BVH 粗筛"。** 现状的 `VisibilityBvhInstance`/`VisibilityBvhUpdatePlan`(`declarations/`)只是平铺实例数组加帧间 diff 计划,真实剔除是 `collect_batching_result.rs` 里对 BTreeMap 的单线程逐实体 `is_mesh_visible` 调用——仓库中并不存在可遍历的 BVH 树。UE 自身默认也走线性分块(`GFrustumCullUseOctree` 默认 `false`,见 SceneVisibility.cpp:293),线性 SoA 数组对 rayon 分块与未来 SIMD 都更友好。`VisibilityBvhUpdatePlan` 的增量 diff 语义保留,VC-M4 把它转为静态空间索引(implicit grid)的维护输入;VC-M1 的并行视锥剔除直接在线性 bounds 数组上做。
2. **两阶段遮挡(prev-frame HZB 先行 + 当帧重测)不纳入 V1。** 两阶段需要当帧 depth 渲染后第二次剔除与补绘(re-test & redraw),牵动计划 02 命令重放的双次提交与 graph 的二段 pass 编排,收益集中在相机急速运动场景;V1 只做单阶段"上帧 furthest HZB + 重投影 + 保守 mip 偏置",误剔风险由偏置兜底(见风险节)。`HzbOcclusionPhase` 枚举预留 `TwoPhaseRetest` 变体作为 V2 扩展点,V1 不实现。

新增文件:

| 文件 | 内容 | 归属层 |
|------|------|--------|
| `zircon_runtime/src/core/framework/render/relevance.rs` | `PrimitiveRelevance` 位集与位常量(纯数据,无 wgpu) | framework 契约 |
| `zircon_runtime/src/graphics/visibility/view_context/mod.rs` | `ViewVisibilityContext`、`VisibilityViewKey`、`ViewCullingStats`、`FrameVisibility` | graphics 实现 |
| `zircon_runtime/src/graphics/visibility/view_context/build_views.rs` | 从 `RenderFrameExtract` 收集 view 集合(主相机 + shadow cascade + RT 相机) | graphics 实现 |
| `zircon_runtime/src/graphics/visibility/relevance/mod.rs` | relevance 计算入口与缓存(generation 失效) | graphics 实现 |
| `zircon_runtime/src/graphics/visibility/relevance/compute_relevance.rs` | 单实例 relevance 计算内核 | graphics 实现 |
| `zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs` | rayon 分块并行视锥剔除 | graphics 实现 |
| `zircon_runtime/src/graphics/visibility/occlusion/mod.rs` | `HzbBuilder`、`HzbOcclusionPhase`、occlusion 统计类型 | graphics 实现 |
| `zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs` | HZB 尺寸/mip 推导与 graph 资源声明参数 | graphics 实现 |
| `zircon_runtime/src/graphics/visibility/static_index/mod.rs` | VC-M4:静态对象 implicit grid 与增量维护 | graphics 实现 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs` | `BuiltinRenderFeature::Hzb` 的 descriptor(pass 声明经 01 graph) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/mod.rs` | HZB build compute executor 的 post-process resource 扩展入口 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/execute_hzb_build.rs` | `ScenePostProcessResources::execute_hzb_build_mip(...)` per-mip dispatch 封装 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/hzb.rs` | HZB build 的 depth/source/params/storage target bind group layout | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_buffer_bundle/hzb_params_buffer.rs` | HZB build uniform 参数 buffer | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/hzb_pipeline.rs` | `zircon-hzb-build-pipeline` compute pipeline 创建与 shader source 覆盖测试 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/hzb_source_texture_view.rs` | mip0 无 HZB source 时的 1x1 fallback texture view | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/hzb_params.rs` | `HzbParams` uniform ABI | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/hzb_build.wgsl` | 深度金字塔 reduce compute | shader |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/mod.rs` | HZB runtime renderer 子模块导出 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs` | VC-M3 occlusion compute pipeline、params buffer、per indirect args buffer bind/dispatch | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl` | 实例遮挡剔除 + indirect args 改写 compute | shader |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/zr_hzb.wgsl` | 共享 include:HZB 采样/矩形测试函数(SSR/SSAO/计划 12 消费,§8 第 3 条) | shader |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs` | graph executor 中把 previous HZB + mesh indirect args 接到 occlusion culler | graphics 实现 |
| `zircon_runtime/src/graphics/tests/render_hzb.rs` | HZB 系列单测(注册进 `graphics/tests/mod.rs`) | 测试 |

修改文件:

| 文件 | 改动 |
|------|------|
| `zircon_runtime/src/graphics/visibility/mod.rs` | 导出新类型;删除被替代类型的导出 |
| `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs` | 单线程剔除循环改为调用 `parallel_frustum` + relevance;BTreeMap 改线性数组(extract 序保序) |
| `zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs` | `visible_entities`/`culled_entities` 等单 view 平铺字段删除,改持 `FrameVisibility`(含 `views`) |
| `zircon_runtime/src/graphics/visibility/culling/is_mesh_visible.rs` | 拆出按 `VisibilityBounds` + 相机的剔除内核(`is_bounds_visible`),供并行路径与 shadow view 复用 |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs` | L130 起的 `VisibilityContext::from_extract_with_history` 调用点切到新构建入口 |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs` | `visibility_context()` 访问器旁增加 `view_visibility(&VisibilityViewKey)`;shadow/mesh 执行链改读 per-view 结果 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs` | alpha mode→phase 的散落判断删除,改读 `PrimitiveRelevance`(与计划 02 `MeshPassProcessor` 同一落点,见切片) |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs` | `shadow_caster_draw_count` 等统计来源改为 relevance 派生,字段名不变 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs` | `shadow_map_executor`(L127)消费 `ShadowCascade` view 的可见集 |
| `zircon_runtime/src/core/framework/render/post_process/stack.rs` | `PostProcessGraphResourceNames` 增加 `HZB_FURTHEST = "hzb-furthest"`;删除 `SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID`/`_COARSE` 两常量 |
| `zircon_runtime/src/graphics/extract/history.rs` | `FrameHistorySlot` 增加 `HzbFurthest` 变体(跨帧持久,绕过 `TransientResourcePool`,对齐 01 RG-M2) |
| `zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs` + `dispatch/descriptor_for.rs` | 增加 `BuiltinRenderFeature::Hzb` 分支 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs` | `read_texture(SCENE_DEPTH)` 之外增读 `HZB_FURTHEST`;SSR descriptor 删除私有 pyramid 声明 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl` | 私有 depth coarse fallback 采样删除;binding 23 现在承载共享 HZB full-mip view,保留旧变量名作为 bind group 兼容别名 |
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `RenderStats` 增加 visibility/HZB 统计字段(见测试节) |
| `zircon_runtime/src/graphics/tests/visibility.rs` | 旧 `visibility_context_filters_visible_batches_through_camera_frustum` 等用例改写为 per-view 断言;新增 `render_visibility_*` 用例 |

### 核心类型与接口

`PrimitiveRelevance`(framework 契约,`core/framework/render/relevance.rs`):

```rust
/// 每 primitive 一份的 pass 参与位集;extract 标记 + 材质域一次性计算。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveRelevance(pub u32);

impl PrimitiveRelevance {
    pub const OPAQUE: u32                = 1 << 0;
    pub const MASKED: u32                = 1 << 1;  // RenderMaterialAlphaMode::Mask
    pub const TRANSLUCENT: u32           = 1 << 2;  // RenderMaterialAlphaMode::Blend
    pub const CASTS_SHADOW: u32          = 1 << 3;  // 材质/实例 casts_shadow 且 phase 政策允许
    pub const VELOCITY_RELEVANT: u32     = 1 << 4;  // Mobility::Dynamic 或骨骼/morph(计划 06 消费)
    pub const DISTORTION: u32            = 1 << 5;  // 预留,计划 07 折射/扭曲
    pub const TWO_D: u32                 = 1 << 6;  // sprite 源(RenderPhaseMeshSource::SpriteIndex)
    pub const RENDER_IN_DEPTH_PASS: u32  = 1 << 7;  // early-z 参与 = OPAQUE | MASKED
    pub const RENDER_IN_MAIN_PASS: u32   = 1 << 8;
    pub const CUSTOM_DEPTH: u32          = 1 << 9;  // 预留(对齐 UE bRenderCustomDepth)
    pub const UI_OVERLAY: u32            = 1 << 10; // 预留 overlay phase
    pub const STATIC_CACHED: u32         = 1 << 11; // 02 CachedMeshDrawCommands 候选
    pub const DYNAMIC_REBUILD: u32       = 1 << 12; // 每帧重建命令(skinned/morph/动态变换)
    // bit 13..31 保留;新增位必须同步更新本表与 render_visibility_relevance_bits_* 测试。

    pub fn contains(self, bits: u32) -> bool { self.0 & bits == bits }
    pub fn intersects(self, bits: u32) -> bool { self.0 & bits != 0 }
}
```

位与现状的换算关系:`OPAQUE/MASKED/TRANSLUCENT` 来自 `RenderMaterialAlphaMode`(`phase_queue.rs` 的 `into_phase_item` 当前在做的判断);`CASTS_SHADOW` 收编 `MeshDrawQueuePhase::casts_shadow()` 政策(opaque/alpha-mask 可投影、transparent 不投影)加材质开关;`TWO_D` 区分 sprite 与 mesh 源。relevance 落地后这些判断点只允许出现在 `relevance/compute_relevance.rs` 一处。

view 与帧级容器(graphics 实现,`visibility/view_context/`):

```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VisibilityViewKey {
    MainCamera,
    ShadowCascade { light: EntityId, cascade: u8 },
    ShadowPointFace { light: EntityId, face: u8 },
    ShadowSpot { light: EntityId },
    CustomTarget { camera: EntityId },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViewCullingStats {
    pub input_count: usize,
    pub layer_filtered_count: usize,   // RenderLayer mask 不相交(09 语义)
    pub frustum_culled_count: usize,
    pub occlusion_culled_count: usize, // VC-M3 起可由 HZB GPU stats readback 覆盖
    pub visible_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewVisibilityContext {
    pub view: VisibilityViewKey,
    pub camera: ViewportCameraSnapshot,      // shadow view 为光源视锥合成的快照
    pub visible: Vec<u32>,                   // 索引进 FrameVisibility 线性数组,升序保序
    pub stats: ViewCullingStats,
}

/// 帧级可见性产物:线性 SoA 源 + relevance + per-view 结果。
/// 替代 VisibilityContext 中 visible_entities/culled_entities 的单 view 平铺语义。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameVisibility {
    pub entities: Vec<EntityId>,             // extract 序,稳定索引空间
    pub bounds: Vec<VisibilityBounds>,
    pub relevance: Vec<PrimitiveRelevance>,  // 与 entities 等长;view 无关(V1 不做 per-view LOD,LodGroup 归计划 10)
    pub relevance_generation: u64,           // 与计划 02 命令缓存共用同一 generation 来源
    pub views: Vec<ViewVisibilityContext>,   // views[0] 恒为 MainCamera
}
```

并行视锥剔除(`culling/parallel_frustum.rs`):

```rust
/// chunk 策略:CHUNK = 1024;len < 2048 走串行(避免小场景调度开销)。
/// 各 chunk 输出本地 (Vec<u32>, ViewCullingStats),按 chunk 序归并 => 结果确定性。
pub(crate) fn parallel_frustum_cull(
    bounds: &[VisibilityBounds],
    layer_masks: &[u32],
    camera: &ViewportCameraSnapshot,
    view_layer_mask: u32,
) -> (Vec<u32>, ViewCullingStats)
```

剔除内核复用现有 `perspective_visible`/`orthographic_visible`(`is_mesh_visible.rs` 拆出 `is_bounds_visible(bounds, camera)`,去掉对 `RenderMeshSnapshot` 整体的依赖)。shadow cascade 视锥用正交快照表达;方向光各级联通过 Plan 05 `shadow/cascade.rs` 的 split 与 camera frustum slice bounds 合成独立 light camera,再走同一 bounds 级 frustum 内核。

`HzbBuilder`(`occlusion/hzb_builder.rs`,CPU 侧参数推导;pass 声明走 feature descriptor,执行走 executor,均经 01 graph):

```rust
pub struct HzbBuilder {
    pub view_size: UVec2,
    pub hzb_size: UVec2,   // (next_pow2(view_size.x) >> 1).max(1) × 同 y —— 对齐 UE BuildHZB
    pub mip_count: u32,    // floor(log2(max(hzb_size.x, hzb_size.y))) + 1,且 >= 1
}

impl HzbBuilder {
    pub fn for_view(view_size: UVec2) -> Self;
    /// 每 dispatch 写 MIP_BATCH(=4)级 mip;返回 pass 数 = ceil(mip_count / 4)。
    pub fn reduce_pass_count(&self) -> u32;
    /// 第 pass_index 个 reduce 的 dispatch groups:ceil(dst_mip0_size / 8) per 维。
    pub fn dispatch_groups(&self, pass_index: u32) -> [u32; 3];
    /// graph 声明参数:Rgba16Float、mip_count 级、STORAGE_BINDING | TEXTURE_BINDING,
    /// 经 FrameHistorySlot::HzbFurthest 标记为持久资源(绕过 TransientResourcePool)。
    pub fn texture_desc(&self) -> HzbTextureDesc;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HzbOcclusionPhase {
    /// V1:上帧 furthest HZB + 当帧重投影保守测试;统计由 GPU stats buffer readback。
    PrevFrameReprojection,
    /// V2 预留:当帧 HZB 重测 + 补绘。本计划不实现。
    TwoPhaseRetest,
}
```

GPU occlusion pass 的 CPU 侧封装(`occlusion/mod.rs`):输入为 03 `GpuScene` 的 instance 缓冲句柄与 `IndirectDrawBatcher` 的 args 缓冲 `RgBufferHandle`,输出 occlusion 统计缓冲;capability gate 复用 03 的 gpu_driven 档位,不支持时整个 occlusion feature 不进 compiled graph(§6 第 4 条),CPU 视锥结果即最终结果。

### GPU 数据布局与 WGSL 约定

**HZB mip 链**:当前落地格式为 `Rgba16Float`(furthest depth 写入 rgb,alpha 固定 1.0;Zircon 深度约定 0=near、1=far,reduce 取 2×2 `max`)。尺寸恒为 2 的幂(`next_pow2(view)/2`),因此 HZB 链内部无奇数尺寸;源 scene depth 和上一层 HZB 的越界采样统一返回 far depth `1.0`,保证 power-of-two padding 对 furthest 链保持保守。V1 只建 furthest 链,不建 closest 链。`R32Float` 仍是后续格式收敛候选,但必须在 storage texture 支持、fallback source view、history copy、SSR/SSAO consumer 都完成验证后再切换,不能让计划先于代码声明已完成。

`hzb_build.wgsl` binding 编号(当前 post-process compute pipeline 使用 group0):

| group | binding | 资源 | WGSL 类型 |
|-------|---------|------|-----------|
| 0 | 0 | scene depth | `texture_depth_2d` |
| 0 | 1 | 源 HZB mip(mip0 使用 1x1 fallback view) | `texture_2d<f32>` |
| 0 | 2 | `HzbParams` | `uniform` |
| 0 | 3 | 目标 HZB mip | `texture_storage_2d<rgba16float, write>` |

```wgsl
// hzb_build.wgsl - workgroup 8x8x1,每次 dispatch 写一个 mip。
struct HzbParams {
    target_size: vec2<u32>,
    target_mip_level: u32,
    _pad0: u32,
};
```

mip0 读取 `scene_depth_tex` 的 2x2 depth texels;`target_mip_level > 0` 时读取上一层 HZB mip 的 2x2 texels。每层 dispatch groups 为 `ceil(target_mip_size / 8)`。pipeline label `"zircon-hzb-build-pipeline"`,workgroup 常量与 graph workload 审计口径一致为 `HZB_BUILD_WORKGROUP_SIZE: [u32;3] = [8,8,1]`。后续若重新采用 UE 风格的一次 dispatch 写 4 个 mip,需要同时更新 bind group layout、shader ABI、graph audit 记录和 RenderDoc 验收。

**遮挡剔除 compute**(`scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl`,V1 单 entry point):

| group | binding | 资源 | WGSL 类型 | 说明 |
|-------|---------|------|-----------|------|
| 0 | 0 | `SceneUniform` | `uniform` | 使用 `previous_view_proj` 做 previous-frame HZB 重投影 |
| 1 | 0 | previous furthest HZB | `texture_2d<f32>` | 读取 `history.previous.hzb-furthest`;第一帧 runtime 使用 white fallback view |
| 1 | 1 | `HzbOcclusionCullParams` | `uniform` | `counts.x = args_count`;`values.x/y = depth_bias/radius_scale` |
| 1 | 2 | indirect args | `var<storage, read_write> array<IndexedIndirectArgs>` | 03/GS-M4 已生成的 phase-local indexed indirect args 缓冲 |
| 3 | 0 | GpuScene primitive 数据 | `var<storage, read> array<ZrGpuPrimitiveData>` | 由 `zr_gpu_scene.wgsl` 唯一定义 |
| 3 | 1 | GpuScene instance 数据 | `var<storage, read> array<ZrGpuInstanceData>` | 读取 current/previous world transform 与 primitive index |

```wgsl
// hzb_occlusion_cull.wgsl - workgroup 64x1x1,thread = indirect arg.
struct HzbOcclusionCullParams {
    counts: vec4<u32>,
    values: vec4<f32>,
};
struct IndexedIndirectArgs {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};
```

V1 测试流程:每个 thread 处理一个 indirect args 记录;从 `first_instance..first_instance+instance_count` 遍历 batch 内 instance。对每个 instance,读取 `ZrGpuInstanceData` 和 `ZrGpuPrimitiveData`,使用 previous transform(若 primitive flag 标记存在)或 current transform 计算 bounds sphere,经 `SceneUniform.previous_view_proj` 投影到上一帧 clip/NDC。跨近平面、屏幕外或深度范围不稳定的实例直接判可见(保守)。其余实例按屏幕半径选择 HZB mip,通过 `zr_hzb_load_furthest(uv, mip)` 读取 previous furthest depth,以 `nearest_depth - radius - depth_bias <= hzb_far` 判可见。若一个 indirect args 记录内没有保守可见 instance,shader 将 `indirect_args[arg_index].instance_count = 0u`。

`zr_hzb.wgsl` 目前只暴露 `zr_hzb_mip_for_radius(...)` 与 `zr_hzb_load_furthest(...)`,不含 entry point。为避免 WGSL handle 参数兼容风险,`zr_hzb_load_furthest` 直接读取当前 shader 模块的 `previous_hzb` 全局 texture。V1 已加入 stats storage buffer 与 CPU readback,用于把 exact culled instance 数覆盖到 `RenderStats.last_visibility_occlusion_culled_count`;同时已在提交点追加 phase-local replay args 与 draw-count buffer readback summary,用于验证 compact replay 的 zero-instance arg、remaining instance 与实际 submitted compact draw count。`HzbOcclusionCuller` 的 per-phase params upload 必须留在 command encoder 内,保持 params copy → dispatch 的命令顺序。UE 风格 clear + atomic compact ABI 已在 `mesh_pass/indirect_compaction.rs` 与 `mesh_pass/indirect_compaction_resources.rs` 落地:每个 source arg 记录 visible-instance remap base 与原始 `first_instance/count`,`MeshIndirectDrawExecution` 侧保存 plan 并拥有 metadata、visible remap、draw-count 与 compacted args WGPU buffers。graph resource 声明、clear-before-dispatch、atomic compact shader、group3 visible remap 消费、compact replay readback summary 和产品墙场景 source assertions 已接入;剩余为 clean Cargo 执行与 RenderDoc 抓帧验收。

### 帧时序与集成点

帧内顺序(全部在 `WgpuRenderFramework::submit_frame_extract` 既有骨架内,锚点为真实文件):

1. **Extract 后、Prepare 前**(`build_frame_submission_context/build.rs` L130 调用点):构建 `FrameVisibility`——`build_views.rs` 收集 view 集合(主相机来自 `extract.view.camera`;directional cascade 视锥由 camera frustum slice bounds + 方向光合成,point face、spot shadow 视锥由 `LightingExtract` 光源快照合成;RT 相机入口预留,计划 09 落地相机集合后接入)→ relevance 计算/缓存命中 → 每 view 走 `parallel_frustum_cull`。`FrameSubmissionContext`(`frame_submission_context.rs`)持有 `FrameVisibility`,`visibility_context()` 访问器旁新增 `view_visibility(key)`。
2. **Queue/Sort**:`create_mesh_draw.rs` 与 `prepared_queue.rs` 改读 `relevance[i]` 决定 phase 参与;`phase_queue.rs` 的 `into_phase_item` 不再从 `RenderMaterialAlphaMode` 现场推导 phase 位(输入侧已带 relevance)。shadow atlas slot replay 从 `views[ShadowCascade{..} / ShadowPointFace{..} / ShadowSpot{..}]` 的 `visible` 取候选,不再复用主 view 结果。
3. **Graph 编译期**(01 `CompiledGraphCache` 之内):`BuiltinRenderFeature::Hzb` descriptor 声明一个 `hzb-build` compute pass,读 `SCENE_DEPTH`、写 `HZB_FURTHEST`,并声明 `FrameHistoryBinding::read_write(FrameHistorySlot::HzbFurthest)`。executor id 为 `"visibility.hzb-build"`,队列为 `QueueLane::AsyncCompute`;真实 WGPU 命令在该 pass 内逐 mip dispatch,graph audit 保留一条聚合 workload 记录。
4. **Execute**:HZB build 在 depth 写完后、SSR/SSAO 之前由 graph 依赖序保证;occlusion cull pass(executor id `"visibility.hzb-occlusion-cull"`)依赖上帧 `HzbFurthest` 与 03 args 缓冲,排在 indirect 提交前。
5. **Present 后**:`HzbFurthest` history 槽位轮转(复用既有 frame history 机制,`graphics/extract/history.rs`)。

硬切换删除清单(同一里程碑内迁移调用方并删除,§6 第 5 条):

| 删除项 | 位置 | 时机 |
|--------|------|------|
| `collect_batching_result.rs` 内单线程 BTreeMap 剔除循环 | `visibility/context/from_extract_with_history/` | VC-M1 |
| `VisibilityContext.visible_entities`/`culled_entities`/`visible_batches` 单 view 平铺字段(已删除;公开读取改为 `main_view_visible_entities()`、`main_view_culled_entities()`、`main_view_visible_batches()`,内部 draw commands、GPU instancing、VG/HGI 输入继续从 main-view set 派生) | `declarations/visibility_context.rs` | VC-M1 |
| `create_mesh_draw.rs`/`prepared_queue.rs` 中 alpha mode→phase 散落判断 | `scene_renderer/mesh/` | VC-M1(若计划 02 的 `MeshPassProcessor` 已落地,则该判断已被 processor 收口,本计划只把 processor 的判断源切到 relevance) |
| `SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID`/`_COARSE` 资源名与 SSR 私有金字塔构建 | `post_process/stack.rs`、SSR descriptor、`post_process_screen_space_reflection.wgsl` | VC-M2 |
| shadow 执行链对主 view 可见集的复用读取 | `frame_submission_context.rs`、`builtin_scene_executors.rs` | VC-M1 |

### 实施切片细化

**VC-M1a relevance 契约与计算**:触碰 `core/framework/render/relevance.rs`(新增)、`visibility/relevance/*`(新增)、`mod.rs` 导出。要点:位表落地;`compute_relevance` 以 `RenderMeshSnapshot` 材质域 + `Mobility` + sprite 源为输入;缓存键 = (entity, material generation),generation 与 02 命令缓存同源。完成判据:`cargo check -p zircon_runtime --lib --locked` 过;relevance 单测绿。

**VC-M1b 并行视锥与 per-view 隔离**:触碰 `culling/parallel_frustum.rs`、`is_mesh_visible.rs` 内核拆分、`view_context/*`、`collect_batching_result.rs` 重写、`visibility_context.rs` 字段切换、`build.rs`/`frame_submission_context.rs` 调用点、shadow executor、`backend_types.rs` 统计字段。要点:线性数组 + 1024 chunk;并行结果与串行参考逐元素一致(测试断言);shadow cascade 独立剔除。完成判据:切片期 `cargo check`;里程碑末 `cargo test -p zircon_runtime visibility --locked`、`cargo test -p zircon_runtime mesh --locked`、`render_product` 回归全绿;shadow view 与主 view 剔除数可在统计中区分。

**VC-M2 HZB 构建 pass**:触碰 `occlusion/hzb_builder.rs`、`feature_descriptors/hzb.rs`、`builtin_render_feature.rs`、`descriptor_for.rs`、`compute_workload.rs` 常量、`scene_renderer/hzb/*`(executor + WGSL)、`history.rs` 槽位、`stack.rs` 资源名、SSR/SSAO descriptor 与 shader 切换。要点:尺寸/mip 公式按 `HzbBuilder` 文档化口径;首批 pass 读 depth、后续批读上批末级 mip;SSR 私有金字塔删除。完成判据:`cargo test -p zircon_runtime render_graph --locked` + post 系列回归;compiled graph dump 中 HZB pass 数 = `reduce_pass_count()`;`render_hzb_*` 单测绿;RenderDoc 抓帧可见 mip 链。

**VC-M3 GPU 遮挡剔除**(前置:03 GS-M4 的 args 缓冲就绪):已触碰 `occlusion/mod.rs`、`scene_renderer/hzb/*`、`hzb_occlusion_cull.wgsl`、hzb executor 注册、graph workload audit、mesh indirect args buffer usage/readback、runtime compile options、compiled graph HZB occlusion pass 过滤,以及 `mesh_pass/indirect_compaction.rs` / `mesh_pass/indirect_compaction_resources.rs` 的 visible-instance remap/compact ABI。capability gate compiled-graph 开关、headless CPU fallback 图路径、GPU stats buffer readback、phase-local replay args/draw-count readback summary 与 `RenderStats.last_visibility_occlusion_culled_count` exact 聚合已实现。2026-06-13 已新增并扩展 `render_product_hzb_occlusion_wall_scene` source assertion,使用产品 `WgpuRenderFramework` 连续提交同一墙 + 64 个墙后静态实例场景,断言 HZB history/readback、culled instance 覆盖墙后实例、compacted draw count 非零且不超过 readback args 容量、zero-instance arg 与 remaining instance 下降,并把 HZB occlusion 开启路径第二帧 captured RGBA 与 capability-gated CPU fallback 基线逐像素对拍。2026-06-18 clean rerun 暴露并修复 `post.uber` 对 `light-list` 的缺失 resource declaration:`PostProcessStackDescriptor` 现在把 `LIGHT_LIST` 视为基础 frame resource,`post.uber` descriptor 显式 read buffer,默认 forward+ 下读取 clustered-lighting graph buffer,禁用 clustered-lighting 时读取 renderer-owned frame external buffer。产品墙场景 clean rerun 与热重跑均通过。本次 compact replay follow-up 已让每个 `MeshIndirectDrawExecution` 拥有 source args、metadata、visible-remap、per-batch draw-count 与 compacted args buffers;`IndirectCompactionBatchMetadata` 记录 source/output arg、source instance span、visible remap base 和 draw-count slot;HZB shader 按 metadata 做 per-instance 保守判定,把可见 source instance 写入 remap,再通过 per-batch atomic draw-count 写 compacted indirect args。回放侧在 HZB cullable phase(opaque、alpha-mask、velocity) 使用 `multi_draw_indexed_indirect_count`,并为 group3 绑定 visible-instance remap;depth-prepass 已在当前 HZB pass 前执行、shadow 不使用主相机 HZB、transparent 需要排序稳定性,所以暂不进入 compact replay。command-local GPUScene/palette bind group 的 draw 仍留在 direct path,避免 compact replay 覆盖逐 draw palette。测试源覆盖 metadata 前缀容量、per-batch output/draw-count simulation、capacity overflow 拒绝、resource usage/zero-capacity allocation guard、execution plan/resource 构建、external resource 声明、clear-before-dispatch、WGSL binding/source 断言、count replay source 断言、compact readback source 断言、diagnostics compact draw-count series、`post.uber` light-list declaration 和 command-local GPUScene 直绘策略。尚未完成更宽 diagnostics focused sweep 和 RenderDoc 验收。完成判据仍为:`cargo test -p zircon_runtime visibility --locked` 与 gpu_scene/mesh 范围测试;墙后实例场景 indirect instance 数下降断言;回落路径产物与 CPU 基线一致。

**VC-M4 静态空间索引增量化**:已新增 `static_index/mod.rs` 的 WGPU-free uniform-grid core,支持 `VisibilityBvhUpdatePlan` diff 驱动的增删改单条维护,并提供 query 结果与全量重建对拍测试源。2026-06-13 已接入 renderer persistent visibility state:每个 `ViewportRecord` 保存上一帧 `VisibilityStaticIndex`,`resolve_viewport_record_state(...)` 把上一帧索引交给 `VisibilityContext::from_extract_with_history_and_static_index(...)`,`record_history(...)` 在提交成功后写回本帧索引。`VisibilityContext` 同步输出 `VisibilityStaticIndexReport`,其本帧 rebuild/update 计数、索引规模和 main-view 预筛证据已进入 `RenderStats` 与 `render.visibility.static_index.*` 产品诊断。主视图 culling 已接入静态/动态 split:静态实例数达到阈值 10_000 时先用静态 grid 对相机保守 bounds 做粗筛,再把粗筛结果与动态实例一起送入既有 `mesh_frustum_visibility(...)` 线性精筛;动态集恒走线性。`render_framework_reuses_static_index_and_reports_main_view_prefilter` 已新增 source-level 渲染框架静态大场景断言,覆盖连续两帧静态提交、第二帧零 full rebuild 和 prefilter candidate 下降。首次运行该测试发现 scene-renderer mesh queue 统计 debug assert 仍假设"命令数 == 源 draw 数";VC-M1/VC-M4 可见性裁剪后该假设不成立,现已收敛为"命令数 <= 源 draw census"。完成判据剩余:完整 render-product 回归与 RenderDoc 可视化验收。

2026-06-15 更新:VC-M4 的 clean scoped rerun 已通过 `render_framework_reuses_static_index_and_reports_main_view_prefilter`;随后复用同一 core-min lib-test 二进制直跑 `visibility` filter,修复 graphics root 内部 re-export 源码形状守卫后通过 59/59。剩余验收集中在完整 render-product 回归与 RenderDoc 可视化验收。

### 测试与验收清单

单测(§8 第 6 条命名;mesh/sprite 构造复用 `graphics/tests/visibility.rs` 既有 fixture helper):

| 测试函数 | 断言要点 | 位置 |
|----------|----------|------|
| `render_visibility_relevance_bits_match_material_alpha_mode` | Opaque/Mask/Blend 分别置 OPAQUE/MASKED/TRANSLUCENT,且 RENDER_IN_DEPTH_PASS = OPAQUE\|MASKED | `graphics/tests/visibility.rs` |
| `render_visibility_relevance_translucent_excluded_from_opaque_candidates` | Blend 材质实例不出现在 opaque phase 候选 | 同上 |
| `render_visibility_relevance_cache_invalidates_with_material_generation` | 改材质域后 relevance 与 02 命令缓存同帧失效(共用 generation) | 同上 |
| `render_visibility_parallel_frustum_matches_serial_results` | 3000+ 实例下并行与串行内核输出逐元素相等(确定性) | 同上 |
| `render_visibility_shadow_view_culls_independently_from_main` | 光源背后实例:主 view 剔除、shadow view 可见;两 view stats 不同 | 同上 |
| `visibility_context_builds_shadow_views_for_atlas_light_slots` | Plan 05 atlas 所需方向光 4 cascade、point 6 face、spot 1 view key 全部生成,且方向光 cascade camera 随 slice 深度产生不同 ortho size/transform | `visibility/context/from_extract_with_history/construct.rs` |
| `render_visibility_stats_partition_input_count` | 每 view `layer_filtered + frustum_culled + occlusion_culled + visible == input` | 同上 |
| `render_hzb_size_and_mip_count_for_odd_viewport` | 1923×1081 → hzb 1024×1024、mip_count 11;1×1 视口不崩 | `graphics/tests/render_hzb.rs` |
| `render_hzb_reduce_pass_batches_cover_all_mips` | `reduce_pass_count()` × 批宽 ≥ mip_count,尾批截断正确 | 同上 |
| `render_hzb_graph_declares_persistent_history_resource` | compiled graph 中 `hzb-furthest` 标记持久、不进 transient 池 | 同上 |
| `render_hzb_ssr_consumes_shared_pyramid` | SSR pass 的读集合含 `hzb-furthest`,且 `screen-space-reflection` 私有 pyramid 资源名不存在 | 同上 |
| `render_visibility_occlusion_rewrites_indirect_instance_count` | 局部 WGPU 测试已升级到 compact 模式:全遮挡 source arg 不进入 draw-count,无遮挡 arg 被写入 compacted args,draw-count == 1,未用 compacted arg 槽保持 zero instance;底层 readback summary helper 现在读取 compact replay args buffer 与 draw-count buffer;产品级 `render_product_hzb_occlusion_wall_scene` source assertion 已接入 64 个墙后实例并消费 stats/readback/indirect args summary,包括 compacted draw count 非零且不超过 readback args 容量;2026-06-18 clean 运行和同一热二进制重跑均通过,并覆盖 `post.uber` 读取 `light-list` 后的 graph resolver/resource declaration 正常路径 | `scene_renderer/hzb/hzb_occlusion_culler.rs::hzb_occlusion_culls_fully_hidden_indirect_args_on_wgpu`、`graphics/tests/render_product_advanced.rs::render_product_hzb_occlusion_wall_scene`(VC-M3) |
| `render_visibility_occlusion_builds_indirect_compaction_abi` | visible-instance remap metadata 按 source args 前缀分配输出容量;CPU simulation 会把 args 改写到 remap base/count 并按 draw batch 写 per-batch draw count;overflow 被拒绝;`MeshIndirectDrawExecution` 上传 args 时保存同一 compaction plan 并创建 metadata/visible-remap/draw-count/compacted-args WGPU buffers;HZB feature descriptor 声明 execution-owned external resources;runtime dispatch record 暴露 source args 读取与 compacted args、visible-remap、draw-count、stats 写入;每个 phase dispatch 前清空 visible-remap、draw-count 与 compacted args 输出;replay 在 HZB 完成后使用 `multi_draw_indexed_indirect_count` 与 group3 visible remap,command-local GPUScene draw 保持 direct | `mesh_pass/indirect_compaction.rs::tests::*`、`mesh_pass/indirect_compaction_resources.rs::tests::*`、`mesh_pass/indirect_draw_execution.rs::tests::*`、`mesh_pass/replay.rs::tests::*`、`gpu_scene/binding.rs::tests::*`、`feature_descriptors/hzb.rs::tests::hzb_occlusion_cull_declares_execution_owned_external_buffers`、`scene_renderer/hzb/hzb_occlusion_culler.rs::tests::*`、`graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs::tests::hzb_occlusion_dispatch_record_reports_compaction_output_writes`(VC-M3 compact ABI/replay) |
| `render_visibility_occlusion_gate_falls_back_to_cpu_results` | source 覆盖已落地为 capability gate、compiled graph 无 cull pass 且保留 `hzb-build`、headless runtime 不执行 occlusion;产品墙场景已把 HZB occlusion 开启路径第二帧 captured RGBA 与 capability-gated CPU fallback 基线逐像素对拍 | 同上 |
| `render_visibility_static_index_incremental_matches_full_rebuild` | 增/删/移对象后 grid 查询结果 == 全量重建结果;`static_index` 过滤测试已通过 core-min lib-test,覆盖增量查询对拍、full rebuild 替换、上一帧索引复用不触发本帧 full rebuild、上一帧索引缺失时安全重建、10_000 静态实例阈值预筛启用且粗筛候选数下降、统计字段映射;渲染框架级静态大场景 source test 覆盖第二帧零 full rebuild 和 main-view prefilter candidate 下降 | `graphics/visibility/static_index/mod.rs`、`context/from_extract_with_history/construct.rs`、`update_stats/base_stats.rs`、`graphics/tests/render_framework_visibility_submit.rs`(VC-M4) |

产物对拍:`render_product_shadows.rs` 既有用例做 VC-M1 shadow 隔离回归;VC-M3 已在 `render_product_advanced.rs` 增 `render_product_hzb_occlusion_wall_scene` source assertion(墙后 64 实例,断言 HZB history/readback 可用、GPU culled instance 覆盖墙后实例、zero-instance arg 出现且 remaining instance 下降)。该用例现在同时创建 HZB occlusion 支持开启框架和 capability-gated CPU fallback 框架,对第二帧 captured RGBA 做逐像素一致性断言,锁定“遮挡剔除只省工作不改像素”。

`RenderStats` 新增字段(`backend_types.rs`,命名延续 `last_` 前缀惯例):`last_visibility_view_count`、`last_visibility_frustum_culled_count`、`last_visibility_occlusion_culled_count`、`last_visibility_visible_count`、`last_visibility_static_index_*` 本帧 rebuild/update/change/规模字段和 main-view prefilter 证据字段、`last_hzb_mip_count`、`last_hzb_graph_executed_pass_count`、`last_hzb_occlusion_*` report/stats readback 字段,以及 `last_hzb_occlusion_indirect_args_readback_available`、`last_hzb_occlusion_readback_arg_count`、`last_hzb_occlusion_compacted_draw_count`、`last_hzb_occlusion_zero_instance_arg_count`、`last_hzb_occlusion_remaining_instance_count`。

命令基线:切片期 `cargo check -p zircon_runtime --lib --locked`;里程碑末 `cargo test -p zircon_runtime visibility --locked`、`cargo test -p zircon_runtime render_hzb --locked`、`cargo test -p zircon_runtime render_graph --locked`、`render_product` 回归(§7)。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | VC-M1 relevance/frustum and per-view visibility | 已完成(核心接入;旧公开字段由 2026-06-17 行收束;CustomTarget payload bridge 由 2026-06-18 行记录) | 新增 relevance 位表与线性 bounds 剔除路径;`FrameVisibility` 承载 main view、directional cascade、point face、spot shadow view;mesh pass、HGI/VG、shadow pass 与统计/诊断改从 view visibility 读取;directional multi-cascade view key 与 camera frustum slice bounds 已接入。 | 多轮 `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` 通过;`rustfmt --edition 2021 --check`、尾随空白与 scoped `git diff --check` 通过;`primitive_relevance` 过滤测试曾被无关 plugin test 编译漂移/共享 lib-test 超时阻塞,未计为通过。 | 完整 custom-target WGPU 渲染提交链仍依赖计划 09;旧 `visible_entities`/`culled_entities`/`visible_batches` 公开字段删除由 2026-06-17 行记录。 |
| 2026-06-15 | VC-M2 shared HZB and SSR/SSAO consumers | 已完成(构建链与消费迁移完成;视觉验收待后续) | 新增 HZB builder/descriptor/resource、WGSL reduce、bind group layout、params buffer、fallback source view 与 per-mip dispatch;HZB history mip-chain copy、dispatch audit、诊断与 graph IO 已接入;SSR/SSAO 迁到共享 `hzb-furthest`,旧 SSR 私有 depth pyramid 生产链删除。 | HZB 相关 `rustfmt --edition 2021 --check`、trailing-whitespace scan 与 scoped `git diff --check` 通过;`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-verify --message-format short --color never` 通过(65 个既有 warning);focused HZB lib-test 被无关 plugin extension bridge imports 阻塞。 | RenderDoc mip 链验收、SSR/SSAO 视觉回归和反射 hit gating 质量确认仍待后续。 |
| 2026-06-15 | VC-M3 HZB occlusion, readback and compact replay | 部分完成(核心 GPU 遮挡链路已接入;RenderDoc/宽 diagnostics 验收未完成) | `visibility.hzb-occlusion-cull` executor、`HzbOcclusionCuller`、capability gate/fallback、per-stage storage-buffer limit gate、GPU stats/readback、indirect args readback summary、compact draw-count/visible-remap ABI、execution-owned graph resource、dispatch clear/write audit与 group3 visible-instance remap 已接入;`render_product_hzb_occlusion_wall_scene` source assertion 覆盖墙后 64 实例与 capability fallback RGBA 对拍。 | 多个 scoped `cargo check` 通过:主 runtime、gate、report、readback、indirect-compaction 与 local WGPU core-min targets;`update_hzb_occlusion_stats` 过滤测试一次通过 2/2,readback 过滤测试输出 3/3 后工具在 shared lib-test tail 超时;2026-06-18 `hzb_occlusion_culler` 通过 6/6,覆盖真实 offscreen WGPU 墙后 args 改写与 clear-before-dispatch 合同;同一热二进制 `hzb_occlusion_dispatch` 4/4、`indirect_compaction` 8/8、`mesh_indirect_draw_execution` 3/3、`multi_draw_indexed_indirect` 1/1、HZB descriptor external 1/1 均通过;core-min `cargo check` 热 rerun 通过。2026-06-18 产品 clean lane 先复现 `post.uber` 缺 `light-list` declaration,修复后 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-postprocess-light-list-check-0618 --message-format short --color never` 通过;`cargo test -p zircon_runtime --lib render_product_hzb_occlusion_wall_scene --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-product-light-list-0618 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1;同一热二进制 `render_product_hzb_occlusion_wall_scene` 重跑通过 1/1,`light_list` 过滤通过 4/4。 | 更宽 diagnostics focused tests 与 RenderDoc 验收仍待后续。 |
| 2026-06-15 | VC-M4 static-index scoped clean rerun | 已完成(聚焦复验通过;直跑 sweep 另行记录) | 复验 `render_framework_reuses_static_index_and_reports_main_view_prefilter`,确认可见性预筛后 mesh queue debug assert 修正已生效;连续两帧 10,001 静态 mesh 场景第二帧复用 persistent static index,`full_rebuild_count == 0`,main-view static candidate 数低于 static input 数。 | `cargo test -p zircon_runtime render_framework_reuses_static_index_and_reports_main_view_prefilter --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1 个过滤测试(44 个既有 warning)。RenderDoc MCP 当前无运行实例。 | 衔接下一行 direct visibility sweep;之后继续完整 render-product 回归与 VC-M2/VC-M3/VC-M4 RenderDoc 视觉验收。 |
| 2026-06-15 | VC-M4 visibility direct sweep | 已完成(直跑 sweep 通过;render-product/RenderDoc 仍后续) | 修复 graphics root 源码形状守卫:将 `ViewportRenderFrame` 与 `ViewportRenderOutputTarget` 的 crate-private re-export 拆成独立行,保留 `ViewportRenderFrame` 内部边界,不扩大公开 API。 | `D:\cargo-targets\zircon-runtime-temporal-s4d-0614\debug\deps\zircon_runtime-5d2828c2001649f6.exe tests::graphics_surface::internal_visibility::graphics_surface_keeps_viewport_frame_and_icon_source_internal --exact --test-threads=1 --nocapture` 通过 1 个测试;同一二进制 `visibility --test-threads=1 --nocapture` 重跑通过 59/59。 | 完整 render-product 回归与 VC-M2/VC-M3/VC-M4 RenderDoc 视觉验收仍待后续;RenderDoc MCP 当前无运行实例。 |
| 2026-06-17 | VC-M3 phase-local HZB dispatch diagnostics | 部分完成(诊断口径收口;产品/RenderDoc 验收仍待后续) | `HzbOcclusionCuller` 的 `dispatch_group_count` 从 `ceil(total_candidate_args / 64)` 改为逐 opaque/alpha-mask/velocity phase 分别 `ceil(args_count / 64)` 后累加;`RenderGraphComputeWorkloadDispatchContext` 增加 phase-local indirect dispatch override,`execute_graph_stage(...)` 用 `HzbOcclusionCullReport.dispatch_group_count` 审计 `IndirectArgs` workload,避免 compact replay 的多 phase dispatch 被误判为 mismatch;同步更新可见性与 submit 诊断文档。 | `rustfmt --edition 2021` 通过本切片 3 个 Rust 文件;`git diff --check -- <scoped files>` 通过(仅 Git LF→CRLF 提示);首次 120s `cargo check` 超时未计入证据,随后 `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-dispatch-audit-0617` 通过(既有 warning set)。新增 source tests 覆盖 phase-local group 累加与 graph audit override,按当前“功能优先”要求未跑 full test。 | 继续 clean 跑 `render_product_hzb_occlusion_wall_scene`、`hzb_occlusion`/compact replay focused tests 与 RenderDoc 抓帧;若再次编辑 `hzb_occlusion_culler.rs`,优先拆出 phase report/dispatch helper,避免接近 1000 行文件继续堆职责。 |
| 2026-06-17 | VC-M3 HZB phase dispatch helper split | 部分完成(模块边界收口;产品/RenderDoc 验收仍待后续) | 新增 `scene_renderer/hzb/phase_dispatch.rs`,把 phase-local args/workgroup 派生与 `HzbOcclusionPhaseDispatchSummary` 从 `hzb_occlusion_culler.rs` 拆出;`HzbOcclusionCuller` 现在只消费 typed phase dispatch 执行 params upload、bind group、compute dispatch、compaction-ready 标记和 report 汇总,主文件从 904 行降到 864 行。 | `rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/hzb/{hzb_occlusion_culler,phase_dispatch,mod}.rs` 通过;`git diff --check -- <HZB phase-dispatch scoped files/docs>` 通过(仅 Git LF→CRLF 提示);首次 `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-phase-dispatch-0617` 304 秒超时无诊断并已停止残留进程,随后同命令 600 秒窗口复跑通过(153.4s,仅既有 warning)。 | 继续 clean 跑 `render_product_hzb_occlusion_wall_scene`、`hzb_occlusion`/compact replay focused tests 与 RenderDoc 抓帧;若后续实现 `TwoPhaseRetest`,优先在 `phase_dispatch.rs` 或新的 report helper 中扩展,不要把策略逻辑堆回 culler。 |
| 2026-06-17 | VC-M1 legacy visibility field cutover | 已完成(字段收束;测试执行按用户要求后置) | 删除 `VisibilityContext.visible_entities`、`culled_entities`、`visible_batches` 三个主视图平铺字段;新增 `main_view_visible_entities()`、`main_view_visible_entity_set()`、`main_view_culled_entities()`、`main_view_visible_batches()` 派生访问器,由 `FrameVisibility + batches` 统一提供主视图查询;构造器继续用同一 main-view set 派生 draw commands、visible instances、GPU instancing、HGI/VG 输入;现有可见性断言改到新 API。 | `rustfmt --edition 2021 --check zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs zircon_runtime/src/graphics/tests/visibility.rs` 通过;`git diff --check -- <visibility legacy cutover scoped files/docs>` 通过(仅 Git LF→CRLF 提示);`cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-visibility-legacy-fields-0617` 通过(既有 warning set);跨 `zircon_app`/`zircon_editor`/`zircon_runtime_interface`/`zircon_runtime/tests` 扫描未发现旧字段读取。 | Full `visibility`/render-product 测试和 RenderDoc 验收后置;CustomTarget visibility payload bridge 由 2026-06-18 行记录,完整渲染提交仍归计划 09。 |
| 2026-06-18 | VC-M1/CO-M1 custom target visibility payload bridge | 部分完成(visibility payload 已接入;WGPU 多相机输出未接入) | `SortedRenderCamera` 保留每个 active scene camera 的 `ViewportCameraSnapshot`;scene extract 为 mesh/sprite 候选合并主相机层与 Texture/Headless scene camera 层;`FrameVisibility` 增加 `render_layer_masks` 并为非 PrimarySurface scene cameras 构建 `VisibilityViewKey::CustomTarget`;`PrimitiveRelevance::view_visible_for_layers(...)` 让 custom target 使用自己的 layer mask 过滤 opaque-like primitive。 | `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-custom-target-visibility-0618 --message-format short --color never` 通过;`cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-custom-target-visibility-0618 --message-format short --color never` 通过并产出 lib-test binary;直接二进制 exact tests 通过:`render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index`,`render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views`,`visibility_context_builds_custom_target_view_from_scene_camera_payload`,`render_frame_extract_carries_scene_camera_order_report_for_scene_camera`,`render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras`,`visibility_context_records_relevance_and_filters_main_view_layers`,`visibility_context_builds_shadow_view_independent_from_main_layers`,`visibility_context_builds_shadow_views_for_atlas_light_slots`。同轮 diagnostics filters `render_product_diagnostics_record_hzb_stats`,`runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins`,`render_product_diagnostics_record_visibility_stats` 通过。RenderDoc MCP 无运行实例。 | 继续计划 09 `CameraRenderDescriptor`/Base+Overlay/多相机 render loop/custom target 输出链;补 RenderDoc 抓帧与更宽 HZB diagnostics sweep。 |
| 2026-06-18 | VC-M1/CO-M1 custom target visibility descriptor consumer | 部分完成(visibility consumer 已改读 descriptor;WGPU 多相机输出未接入) | `FrameVisibility::from_frame_views(...)` 改为接收 `RenderViewExtract.cameras` descriptor slice;Texture/Headless scene cameras 通过 `CameraRenderDescriptor.target` 与 `CameraRenderDescriptor.culling_mask` 构建 `VisibilityViewKey::CustomTarget`;`RenderCameraOrderReport` 保留 ordering diagnostics,不再作为 custom-target visibility payload source。 | `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-descriptor-visibility-0618 --message-format short --color never` 通过;lib-test `--no-run` 通过;直接二进制 exact tests 通过:`visibility_context_builds_custom_target_view_from_camera_descriptors`,`render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views`,`render_frame_extract_carries_scene_camera_order_report_for_scene_camera`。首次并行 `cargo test` 同 target dir 超时未产出 binary,不计为通过。 | 继续计划 09 descriptor hard cutover、WGPU 多相机 target loop 与 custom target 输出链;补 RenderDoc 抓帧与更宽 HZB diagnostics sweep。 |

### 参考实现精读笔记

| 参考符号(真实读到) | 要点 | Zircon 对应物与取舍 |
|----------------------|------|---------------------|
| `FrustumCull(Scene, View, Flags, ..., TaskConfig, TaskIndex)`(SceneVisibility.cpp:756) | 按位字数组分块(`TaskWordOffset = TaskIndex * NumWordsPerTask`)在任务间切分 primitive 区间;`GFrustumCullUseOctree` 默认 false、`GFrustumCullUseFastIntersect` 默认 true——UE 默认线性数组 + 快速平面测试 | `parallel_frustum_cull` 的 rayon 1024-chunk 等价于 word 分块;不引入 octree,印证"切线性数组"裁决;bit array 改为升序索引 Vec(Rust 侧更易保序断言) |
| `IsPrimitiveVisible(View, PermutedPlanePtr, ...)`(SceneVisibility.cpp:548) | 预置换平面布局做 SIMD 球/盒测试 | V1 复用现有 `perspective_visible`/`orthographic_visible` 标量内核;SIMD 化留作性能切片,不进本计划验收 |
| `FRelevancePacket::LaunchComputeRelevanceTask` / `Finalize`(SceneVisibility.cpp:1252/1287) | relevance 在 packet 任务内计算、`Finalize` 单线程合并进 view(`ShadingModelMaskInView |=` 等);`NotDrawRelevant` 反向清可见位 | Zircon 的 relevance 是材质域纯函数,V1 在 extract 后一次性算 + 缓存,不需要 packet 级合并;chunk 局部 stats 按序归并即 Finalize 等价 |
| `FPrimitiveViewRelevance` 位字段(PrimitiveViewRelevance.h:20-70):`bOpaqueRelevance`、`bMaskedRelevance`、`bShadowRelevance`、`bVelocityRelevance`、`bRenderInDepthPass`、`bRenderInMainPass`、`bRenderCustomDepth` | UE 以 view 级 union 聚合驱动 pass 启停 | `PrimitiveRelevance` 位表直接对位;UE 的编辑器位(`bEditorPrimitiveRelevance` 等)不引入,2D 用 `TWO_D` 自有位 |
| `BuildHZB(GraphBuilder, SceneDepth, ...)`(SceneTextureReductions.cpp:116):`HZBSize = RoundUpToPowerOfTwo(ViewRect) >> 1`、`NumMips = FloorToInt(Log2(max))`、`FHZBBuildCS::kMaxMipBatchSize = 4`、`DispatchThreadIdToBufferUV` + `InputViewportMaxBound` | pow2 尺寸 + 每 dispatch 批量写 4 mip + 源采样 UV clamp 处理非整除视口;furthest/closest 分两张纹理省缓存 | `HzbBuilder` 的尺寸/mip 公式对齐;V1 只建 furthest;当前 WGPU 落地选择每 dispatch 写 1 个 mip,用单 mip texture view 明确读上一层/写当前层,后续若批量写 4 mip 需重新验证 WGSL storage binding 与 graph audit |
| `GetHZBParameters(GraphBuilder, View, ...)`(HZB.cpp:53) | 消费侧参数统一打包(UV factor、extent),消费方不自行换算 | `zr_hzb.wgsl` 的函数式 include 承担同职责,SSR/SSAO/粒子统一入口 |
| `FInstanceCullingManager::RegisterView`/`FlushRegisteredViews`(InstanceCullingManager.cpp:50/98) | 多 view 注册后合批剔除,deferred context 延迟到 graph 执行 | V1 每 view 独立 cull dispatch(view 数少);多 view 合批列为 VC-M3 后优化项 |
| `BuildInstanceDrawCommands.usf:312/338`:`InterlockedAdd(DrawIndirectArgsBufferOut[IndirectArgIndex * INDIRECT_ARGS_NUM_WORDS + 1], ...)`;`ClearIndirectArgInstanceCountCS`(:358) | instance_count 在 args word1 原子累加;独立 clear pass 先置零 | Zircon 当前按 draw batch 拆分 draw-count slot,避免一个全局 atomic count 跨 pipeline/material/geometry batch 重排;HZB shader 从 source args + metadata compact 到独立 compacted args buffer,group3 通过 visible remap 把 compacted `first_instance` 还原到源 GPUScene instance;透明/阴影/command-local palette draw 暂不进入该路径 |
| `InstanceCullingOcclusionQuery.usf:113`:`IsVisibleHZB(Rect, bSample4x4)`;BuildInstanceDrawCommands.usf:199-202 prev-frame `HZBTestViewRect` + 自遮挡精度注释 | 屏幕矩形 → mip 选择 → 多点采样保守判定;上帧 HZB 测试需防自遮挡精度误差 | mip 选择公式 + `mip_bias`(默认 1)即自遮挡防线;2×2 采样代替 4×4,以偏置换带宽 |
| `FSceneCullingBuilder` temp-cell 增量更新(SceneCulling.cpp:803,1149-1150 注释) | grid cell 首次触碰时建 temp cell 记录增删,结束统一回写,避免全量重建 | VC-M4 `VisibilityStaticIndex` 已落 WGPU-free 单层 uniform grid:`VisibilityBvhUpdatePlan` diff → entity/cell 关系增量维护;renderer 持久 owner 已接入 `ViewportRecord`,frame-to-frame 零重建和 main-view prefilter candidate 下降已有渲染框架级 source test 覆盖,仍需 scoped/broad 验证 |

## 风险与回退

- HZB 重投影在相机剧烈运动时保守性不足导致误剔:采用一帧延迟 + 保守 mip 偏置;出现闪烁时先放宽偏置再查重投影矩阵。
- relevance 缓存失效与计划 02 缓存失效耦合:两者共用同一 generation 来源,单测覆盖"改材质域后 relevance 与命令同时失效"。
- 多 view 剔除成本上升:方向光现在按每级联 camera frustum slice 独立剔除,正确性优先;若后续大场景 shadow prepare 成本过高,再引入共享候选粗筛或 caster cache,不能退回主视图结果复用。
