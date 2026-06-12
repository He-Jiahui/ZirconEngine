---
related_code:
  - zircon_runtime/src/graphics/visibility/mod.rs
  - zircon_runtime/src/graphics/visibility/context/mod.rs
  - zircon_runtime/src/graphics/visibility/culling/mod.rs
  - zircon_runtime/src/graphics/visibility/planning/mod.rs
  - zircon_runtime/src/core/framework/render/relevance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/HZB.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneCulling/SceneCulling.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp
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

## 目标架构

归属:`zircon_runtime/src/graphics/visibility/` 内部升级,新增 `relevance/` 与 `occlusion/` 子模块;HZB 构建 pass 注册为内建 RenderFeature(经计划 01 graph)。

核心类型:

- `ViewVisibilityContext`:按 view 隔离的可见性结果(主相机 / 每个 shadow cascade / 自定义 RT 相机各一份);view 集合由计划 09 的相机管理提供。
- `PrimitiveRelevance` 位集:`opaque/alpha_mask/transparent/casts_shadow/needs_velocity/needs_distortion/...`;在 extract 标记 + 材质域上一次性计算,缓存于静态对象(变更失效与计划 02 共用 generation)。
- 并行 frustum cull:对 extract 实例数组按块切分,rayon 并行;输出可见索引 + relevance 过滤后的 per-phase 候选集,直接喂给计划 02 的 pass processor。
- `HzbBuilder`:上一帧 scene depth → mip 金字塔(compute reduce);本帧用重投影保守测试(无 readback)。HZB 资源经 graph 声明为持久资源。
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
- 已新增 `graphics/visibility/declarations/visibility_relevance_entry.rs`,并在 `VisibilityContext::primitive_relevance` 中产出每 entity 的 relevance。旧的 `visible_entities`、`culled_entities`、batch/history/upload plan 字段暂时保留,便于后续消费者分阶段迁移。
- 已新增 `graphics/visibility/culling/parallel_frustum.rs`,对线性 `{ entity, VisibilityBounds }` 候选数组使用确定性 serial/parallel helper。当前阈值以下走串行,大场景走 rayon `par_iter`,返回顺序保持输入序。
- `is_mesh_visible.rs` 已收敛为 `is_bounds_visible(bounds, camera)` bounds 级内核。这让 frustum culling、shadow view 和后续静态空间索引可以共用同一剔除入口。
- `collect_batching_result.rs` 已改为同时计算 relevance 与 frustum 结果,并复用同一份预计算 bounds 写入 `VisibilityBvhInstance` 与 history entries;主视图可见性现在要求 `relevance.main_view()` 且 frustum 可见,因此相机 `RenderLayerSet` 会参与 `visible_entities`/`visible_batches` 过滤。layer mismatch 的 opaque-like mesh 仍保留 `shadow_caster` relevance,为后续 shadow view 独立剔除保留语义。
- 已新增 `graphics/visibility/view_context/mod.rs`,落地主相机版 `FrameVisibility` / `ViewVisibilityContext` / `VisibilityViewKey` / `ViewCullingStats`。`VisibilityContext::frame_visibility` 现在保存稳定的 frame primitive index space(`entities`/`bounds`/`relevance`)和主视图 visible indices/statistics,旧平铺字段继续保留。
- 已新增 `graphics/visibility/view_context/build_views.rs`,对每个 extracted directional light 生成 `ShadowCascade { cascade: 0 }` view。shadow view 使用帧 bounds 合成正交 light camera,复用 `mesh_frustum_visibility(...)`,并以 `PrimitiveRelevance::shadow_caster()` 作为 relevance gate,因此主相机 layer mismatch 的 opaque-like mesh 仍可进入 shadow view。
- `ViewportRenderFrame` 现在携带 `FrameVisibility` sideband;`submit_frame_extract` 与 direct runtime-frame submit 两条路径都会把 `FrameSubmissionContext.visibility_context().frame_visibility` 传到 renderer。`FrameSubmissionContext::view_visibility(key)` 已提供 submit-time per-view 访问口。`build_mesh_draws(...)` 将 `FrameVisibility` 映射回 `MeshDraw`,并把 primitive relevance、main-view visibility、shadow-view visibility 透传到 `MeshBatchRef`。
- `MeshPassProcessor` 现在使用 relevance/view visibility 作为 phase gate:depth/opaque/alpha/transparent/velocity 需要 main-view 可见且对应 relevance 成立,shadow pass 需要 shadow-caster relevance 和 shadow view 可见。旧 queue/profile 仍负责材质 phase 与 pipeline variant 选择,但不再单独决定当前 view 是否参与该 pass。
- Hybrid GI 与 Virtual Geometry planning 已不再直接读取 `BatchingResult.visible_entities`;调用侧现在从 `FrameVisibility::main_view_visible_entity_set()` 派生主视图实体集合。`BatchingResult.visible_batches` 已删除,`construct.rs` 从 `batches + main_view_visible_entity_set()` 派生 `visible_batches`、`visible_instances`、draw commands 和 GPU instancing candidates。Virtual Geometry debug 的 node/cluster cull snapshot 也通过 `FrameSubmissionContext::view_visibility(MainCamera)` 读取相机,与运行时 view 权威保持一致。
- `RenderStats` 已新增 `last_visibility_view_count`、`last_visibility_input_count`、`last_visibility_layer_filtered_count`、`last_visibility_frustum_culled_count`、`last_visibility_occlusion_culled_count`、`last_visibility_visible_count`。`update_base_stats(...)` 从 `FrameVisibility.views[*].stats` 聚合这些字段,`render_stats_store::product` 记录到 `render.visibility.*`,运行时诊断 fixture 也覆盖这些路径。当前 occlusion 统计保持 0,等待 VC-M3 HZB/GPU occlusion 写入同一统计面。
- 尚未完成:custom render-target camera view、directional light 多 cascade 切分、legacy `visible_entities`/`culled_entities` 字段删除,以及 GPU 遮挡剔除仍按后续 VC-M1/VC-M3 推进。`CustomTarget` 需要计划 09 把多相机投影/变换快照带入 `RenderFrameExtract`;directional multi-cascade split 属于计划 05 LS-M3 的 CSM/atlas/shader sampling 切片。
- 验证状态: touched Rust 文件 `rustfmt --check` 通过;`git diff --check` 对本切片文件通过(仅 Git 行尾转换提示);`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` 在 relevance/frustum、bounds-kernel、主视图 `FrameVisibility`、shadow view 构建、mesh-pass relevance 消费、main-view planning accessor 迁移、visibility stats/diagnostics 接入后通过(现有 warning set)。focused lib-test 尚无结果:一次被共享 lib-test 的非 render 插件测试源阻塞(`runtime_plugin_package_manifest.rs` 缺少 `RuntimePluginDescriptor::with_target_mode`),最新一次在 304 秒编译窗口内超时。

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
- runtime executor registry 已注册 `visibility.hzb-build`。当前 executor 会校验 `scene-depth` 与 `hzb-furthest` 已绑定,按 HZB size 记录 `zircon-hzb-build-pipeline` 的 compute dispatch 与 storage write evidence。此切片尚未提交实际 WGSL reduce shader,因此它是 graph/resource/diagnostic 闭环,不是最终 GPU 金字塔填充。
- Frame history 已增加 `FrameHistorySlot::HzbFurthest`、`history.previous.hzb-furthest` 资源名、运行时 HZB history texture、mip-chain copy 到 history 的帧尾路径,并在 `RenderHistoryCopyReport`/diagnostics 中记录 `render.history.copy.hzb_furthest_copied`。
- `RenderStats` 已新增 `last_hzb_mip_count` 与 `last_hzb_graph_executed_pass_count`;`render_stats_store::product` 记录 `render.hzb.mip_count` 和 `render.hzb.graph_executed_pass_count`,运行时诊断 fixture 覆盖这些路径。
- 验证状态:HZB 相关 Rust 文件 `rustfmt --edition 2021 --check` 通过;切片文件尾随空白扫描为 clean;`git diff --check -- <HZB scoped files>` 退出 0(仅 Git LF→CRLF 提示)。`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 已在 HZB builder/descriptor/resource、dispatch audit、history mip-chain copy、diagnostics 接入后通过(现有 warning set)。`cargo test -p zircon_runtime --lib hzb --locked ...` 尚未跑到 filtered HZB 测试,因为共享 lib-test 目标先被无关插件测试 `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs` 的缺失 bridge 类型导入挡住。
- 尚未完成:VC-M2 的真实 WGSL depth pyramid reduce、RenderDoc mip 链验收、SSR/SSAO 改为消费共享 `hzb-furthest`。现有 SSR 私有 pyramid 资源仍保留,等 HZB shader 产物稳定后再删除私有路径。

### VC-M3 GPU 遮挡剔除(依赖计划 03 GS-M4)

实施切片:
1. HZB 重投影遮挡 compute:实例 bounds 投影 → mip 选择 → 保守判定 → 改写 indirect instance_count。
2. capability gate 与 CPU 回落;遮挡剔除数进 RenderStats。

测试阶段:
- `cargo test -p zircon_runtime visibility --locked` 与 gpu_scene 范围测试
- 验收证据:遮挡场景(墙后大量实例)indirect 实际 instance 数下降(统计断言);画面无漏剔/误剔(对拍)。

### VC-M4 静态场景空间索引增量化

实施切片:
1. BVH/网格索引对静态对象增量更新(增删改单条更新,不全量重建),对齐 SceneCulling 的增量维护思路。

测试阶段:
- `cargo test -p zircon_runtime visibility --locked`(增删对象后索引一致性断言)
- 验收证据:静态场景帧间索引重建次数为 0。

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
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/mod.rs` | HZB build / occlusion cull 两个 compute executor | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_build.wgsl` | 深度金字塔 reduce compute | shader |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl` | 实例遮挡剔除 + indirect args 改写 compute | shader |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/zr_hzb.wgsl` | 共享 include:HZB 采样/矩形测试函数(SSR/SSAO/计划 12 消费,§8 第 3 条) | shader |
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
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl` | 私有 depth pyramid 采样准备删除,改 include `zr_hzb.wgsl` |
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
    CustomTarget { camera: EntityId },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViewCullingStats {
    pub input_count: usize,
    pub layer_filtered_count: usize,   // RenderLayer mask 不相交(09 语义)
    pub frustum_culled_count: usize,
    pub occlusion_culled_count: usize, // VC-M3 起非零(GPU readback 仅测试路径)
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

剔除内核复用现有 `perspective_visible`/`orthographic_visible`(`is_mesh_visible.rs` 拆出 `is_bounds_visible(bounds, camera)`,去掉对 `RenderMeshSnapshot` 整体的依赖)。shadow cascade 视锥用正交快照表达,方向光各级联先共享一次主方向粗剔结果再做距离细分(风险节既有口径)。

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
    /// graph 声明参数:R32Float、mip_count 级、STORAGE_BINDING | TEXTURE_BINDING,
    /// 经 FrameHistorySlot::HzbFurthest 标记为持久资源(绕过 TransientResourcePool)。
    pub fn texture_desc(&self) -> HzbTextureDesc;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HzbOcclusionPhase {
    /// V1:上帧 furthest HZB + 当帧重投影保守测试,无 readback。
    PrevFrameReprojection,
    /// V2 预留:当帧 HZB 重测 + 补绘。本计划不实现。
    TwoPhaseRetest,
}
```

GPU occlusion pass 的 CPU 侧封装(`occlusion/mod.rs`):输入为 03 `GpuScene` 的 instance 缓冲句柄与 `IndirectDrawBatcher` 的 args 缓冲 `RgBufferHandle`,输出 occlusion 统计缓冲;capability gate 复用 03 的 gpu_driven 档位,不支持时整个 occlusion feature 不进 compiled graph(§6 第 4 条),CPU 视锥结果即最终结果。

### GPU 数据布局与 WGSL 约定

**HZB mip 链**:格式 `R32Float`(furthest depth;Zircon 深度约定 0=near、1=far,reduce 取 2×2 `max`;若后续切 reversed-z,只改 `zr_hzb.wgsl` 内 `ZR_HZB_FAR_OP` 一处)。尺寸恒为 2 的幂(`next_pow2(view)/2`),因此 mip 链内部无奇数尺寸;**奇数只出现在源深度采样阶段**,处理方式对齐 UE `BuildHZB`:线程 UV = `(thread_id * 2 + 0.5) * inv_src_size`,采样前 clamp 到 `input_viewport_max_uv`(= `(view_size - 0.5) / src_size`),源边界外自然复制边缘 texel,保守性成立。V1 只建 furthest 链,不建 closest 链(UE 分两张纹理的理由是消费方通常只要其一,SceneTextureReductions.cpp 注释;Zircon 当前消费方 SSR/SSAO/遮挡剔除全部只需 furthest)。

`hzb_build.wgsl` binding 编号(§8 槽位:group1 = pass 级输入;本 pass 无 view/material/instance 数据,group0/2/3 空缺):

| group | binding | 资源 | WGSL 类型 |
|-------|---------|------|-----------|
| 1 | 0 | 源(scene depth 或上批末级 mip) | `texture_2d<f32>` |
| 1 | 1 | point clamp sampler | `sampler` |
| 1 | 2..5 | 目标 mip 0..3(批内) | `texture_storage_2d<r32float, write>` |
| 1 | 6 | `ZrHzbBuildParams` | `uniform` |

```wgsl
// hzb_build.wgsl —— workgroup 8×8×1,一次 dispatch 经 LDS 归约写 4 级 mip(对齐 UE kMaxMipBatchSize=4)
struct ZrHzbBuildParams {
    inv_src_size: vec2<f32>,
    input_viewport_max_uv: vec2<f32>, // 奇数/非整除视口的 clamp 上界
    dst_mip0_size: vec2<u32>,
    mip_batch_count: u32,             // 本 dispatch 实际写几级(1..4)
    src_is_depth: u32,                // 1 = 首批读 scene depth,0 = 读上批 HZB mip
};
```

mip 级数不足 4 的尾批用 `mip_batch_count` 截断;`dispatch_groups = ceil(dst_mip0_size / 8)`。pipeline label `"zircon-hzb-build-pipeline"`,workgroup 常量与 SSAO 一致放 `feature_descriptors/compute_workload.rs`(`HZB_BUILD_WORKGROUP_SIZE: [u32;3] = [8,8,1]`)。

**遮挡剔除 compute**(`hzb_occlusion_cull.wgsl`,两个 entry point):

| group | binding | 资源 | WGSL 类型 | 说明 |
|-------|---------|------|-----------|------|
| 0 | 0 | `ZrHzbCullView` | `uniform` | prev_view_proj(重投影)、hzb_size、mip_count、mip_bias |
| 1 | 0 | furthest HZB | `texture_2d<f32>` | 上帧资源,经 `RgResourceResolver` 解析 |
| 1 | 1 | point clamp sampler | `sampler` | |
| 1 | 2 | indirect args | `var<storage, read_write> array<u32>` | 03 `IndirectDrawBatcher` 的 args 缓冲 |
| 1 | 3 | `ZrInstanceCullMeta` | `var<storage, read> array<...>` | instance → indirect-arg 槽映射(03 batcher 生成) |
| 1 | 4 | visible instance id 输出 | `var<storage, read_write> array<u32>` | 压实 id 流,03 indirect 提交消费 |
| 1 | 5 | `ZrCullStats` | `var<storage, read_write>` | atomic 计数(occlusion_culled 等) |
| 3 | 0 | GpuScene instance 数据 | `var<storage, read> array<ZrGpuInstance>` | struct 由 03 的 `zr_gpu_scene.wgsl` 唯一定义,本计划只 include |

```wgsl
// hzb_occlusion_cull.wgsl —— workgroup 64×1×1,thread = instance(对齐 UE NUM_THREADS_PER_GROUP 线性派发)
struct ZrHzbCullView {
    prev_view_proj: mat4x4<f32>,   // 列主序(§8 第 2 条)
    hzb_size: vec2<u32>,
    mip_count: u32,
    mip_bias: u32,                 // 保守偏置,默认 1
};
struct ZrInstanceCullMeta {
    indirect_arg_slot: u32,        // 本 instance 所属 indirect batch 的 args 槽
    instance_output_offset: u32,   // 压实输出段基址
    _pad0: u32, _pad1: u32,        // std430,16B 对齐
};
```

测试流程(单 thread):读 `ZrGpuInstance` 的 world AABB → 8 角点经 `prev_view_proj` 投影为屏幕矩形与最近深度 → `mip = clamp(ceil(log2(max(rect_w_px, rect_h_px) / 2)) + mip_bias, 0, mip_count-1)`(2×2 footprint 内 4 采样,对齐 UE `IsVisibleHZB` 的 4×4 改为 2×2 + 偏置,保守且省带宽)→ 4 点取 `max`(furthest)与矩形最近深度比较,`nearest_depth <= hzb_far` 即可见 → 可见者 `atomicAdd(&indirect_args[slot * 5u + 1u], 1u)`(`instance_count` 位于 draw_indexed_indirect 5 词布局的 word 1,与 UE `INDIRECT_ARGS_NUM_WORDS` 口径一致)并按返回序号写压实 id。相机越界/AABB 跨近平面的实例直接判可见(保守)。第二 entry `zr_clear_indirect_instance_counts`:每 thread 清一个 args 槽的 instance_count(对齐 UE `ClearIndirectArgInstanceCountCS`),在 cull pass 前以独立 graph pass 执行;CPU 回落路径不跑 clear/cull,args 由 batcher 填满额 instance_count。

`zr_hzb.wgsl` include 只暴露 `zr_hzb_sample_furthest(uv, mip)` 与 `zr_hzb_rect_visible(rect, nearest_depth)` 两个函数和上述 struct,不含 entry point(§8 第 3 条);SSR/SSAO 与计划 12 的 GPU 粒子剔除统一经它消费 HZB。

### 帧时序与集成点

帧内顺序(全部在 `WgpuRenderFramework::submit_frame_extract` 既有骨架内,锚点为真实文件):

1. **Extract 后、Prepare 前**(`build_frame_submission_context/build.rs` L130 调用点):构建 `FrameVisibility`——`build_views.rs` 收集 view 集合(主相机来自 `extract.view.camera`;shadow cascade 视锥由 `LightingExtract.directional_lights` 等光源快照合成;RT 相机入口预留,计划 09 落地相机集合后接入)→ relevance 计算/缓存命中 → 每 view 走 `parallel_frustum_cull`。`FrameSubmissionContext`(`frame_submission_context.rs`)持有 `FrameVisibility`,`visibility_context()` 访问器旁新增 `view_visibility(key)`。
2. **Queue/Sort**:`create_mesh_draw.rs` 与 `prepared_queue.rs` 改读 `relevance[i]` 决定 phase 参与;`phase_queue.rs` 的 `into_phase_item` 不再从 `RenderMaterialAlphaMode` 现场推导 phase 位(输入侧已带 relevance)。shadow 命令列表(`builtin_scene_executors.rs::shadow_map_executor`)从 `views[ShadowCascade{..}]` 的 `visible` 取候选,不再复用主 view 结果。
3. **Graph 编译期**(01 `CompiledGraphCache` 之内):`BuiltinRenderFeature::Hzb` descriptor 声明 `read_texture(SCENE_DEPTH)` + `FrameHistoryBinding::read_write(FrameHistorySlot::HzbFurthest)` + `reduce_pass_count()` 个 compute pass(executor id `"visibility.hzb-build"`,`QueueLane::AsyncCompute`,与 SSAO descriptor 同构)。
4. **Execute**:HZB build 在 depth 写完后、SSR/SSAO 之前由 graph 依赖序保证;occlusion cull pass(executor id `"visibility.hzb-occlusion-cull"`)依赖上帧 `HzbFurthest` 与 03 args 缓冲,排在 indirect 提交前。
5. **Present 后**:`HzbFurthest` history 槽位轮转(复用既有 frame history 机制,`graphics/extract/history.rs`)。

硬切换删除清单(同一里程碑内迁移调用方并删除,§6 第 5 条):

| 删除项 | 位置 | 时机 |
|--------|------|------|
| `collect_batching_result.rs` 内单线程 BTreeMap 剔除循环 | `visibility/context/from_extract_with_history/` | VC-M1 |
| `VisibilityContext.visible_entities`/`culled_entities` 单 view 平铺字段(内部 `construct.rs` 已从 main-view 派生 visible batches、draw commands、GPU instancing、VG/HGI 输入;公开兼容字段最终删除仍待外部/test 消费点收束) | `declarations/visibility_context.rs` | VC-M1 |
| `create_mesh_draw.rs`/`prepared_queue.rs` 中 alpha mode→phase 散落判断 | `scene_renderer/mesh/` | VC-M1(若计划 02 的 `MeshPassProcessor` 已落地,则该判断已被 processor 收口,本计划只把 processor 的判断源切到 relevance) |
| `SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID`/`_COARSE` 资源名与 SSR 私有金字塔构建 | `post_process/stack.rs`、SSR descriptor、`post_process_screen_space_reflection.wgsl` | VC-M2 |
| shadow 执行链对主 view 可见集的复用读取 | `frame_submission_context.rs`、`builtin_scene_executors.rs` | VC-M1 |

### 实施切片细化

**VC-M1a relevance 契约与计算**:触碰 `core/framework/render/relevance.rs`(新增)、`visibility/relevance/*`(新增)、`mod.rs` 导出。要点:位表落地;`compute_relevance` 以 `RenderMeshSnapshot` 材质域 + `Mobility` + sprite 源为输入;缓存键 = (entity, material generation),generation 与 02 命令缓存同源。完成判据:`cargo check -p zircon_runtime --lib --locked` 过;relevance 单测绿。

**VC-M1b 并行视锥与 per-view 隔离**:触碰 `culling/parallel_frustum.rs`、`is_mesh_visible.rs` 内核拆分、`view_context/*`、`collect_batching_result.rs` 重写、`visibility_context.rs` 字段切换、`build.rs`/`frame_submission_context.rs` 调用点、shadow executor、`backend_types.rs` 统计字段。要点:线性数组 + 1024 chunk;并行结果与串行参考逐元素一致(测试断言);shadow cascade 独立剔除。完成判据:切片期 `cargo check`;里程碑末 `cargo test -p zircon_runtime visibility --locked`、`cargo test -p zircon_runtime mesh --locked`、`render_product` 回归全绿;shadow view 与主 view 剔除数可在统计中区分。

**VC-M2 HZB 构建 pass**:触碰 `occlusion/hzb_builder.rs`、`feature_descriptors/hzb.rs`、`builtin_render_feature.rs`、`descriptor_for.rs`、`compute_workload.rs` 常量、`scene_renderer/hzb/*`(executor + WGSL)、`history.rs` 槽位、`stack.rs` 资源名、SSR/SSAO descriptor 与 shader 切换。要点:尺寸/mip 公式按 `HzbBuilder` 文档化口径;首批 pass 读 depth、后续批读上批末级 mip;SSR 私有金字塔删除。完成判据:`cargo test -p zircon_runtime render_graph --locked` + post 系列回归;compiled graph dump 中 HZB pass 数 = `reduce_pass_count()`;`render_hzb_*` 单测绿;RenderDoc 抓帧可见 mip 链。

**VC-M3 GPU 遮挡剔除**(前置:03 GS-M4 的 args 缓冲与 `ZrInstanceCullMeta` 就绪):触碰 `occlusion/mod.rs`、`hzb_occlusion_cull.wgsl`、hzb executor 注册、capability gate 接线、`RenderStats` occlusion 字段。要点:clear → cull 两 pass 经 graph 声明;gate 关闭时 feature 不进 compiled graph;统计经 readback 仅在测试路径开启。完成判据:`cargo test -p zircon_runtime visibility --locked` 与 gpu_scene 范围测试;墙后实例场景 indirect instance 数下降断言;回落路径产物与 CPU 基线一致。

**VC-M4 静态空间索引增量化**:触碰 `static_index/*`(新增)、`build_bvh_update_plan.rs`(diff 输出改喂 static_index)、`parallel_frustum.rs`(静态集走 grid 粗筛 + 线性精筛,仅当静态实例数超过阈值 10_000 时启用)。要点:implicit grid 对齐 `FSceneCullingBuilder` 的 temp-cell 增量更新思路,增删改单条维护;动态集恒走线性。完成判据:`cargo test -p zircon_runtime visibility --locked`;增删对象后索引与全量重建结果一致断言;静态场景帧间索引重建次数为 0。

### 测试与验收清单

单测(§8 第 6 条命名;mesh/sprite 构造复用 `graphics/tests/visibility.rs` 既有 fixture helper):

| 测试函数 | 断言要点 | 位置 |
|----------|----------|------|
| `render_visibility_relevance_bits_match_material_alpha_mode` | Opaque/Mask/Blend 分别置 OPAQUE/MASKED/TRANSLUCENT,且 RENDER_IN_DEPTH_PASS = OPAQUE\|MASKED | `graphics/tests/visibility.rs` |
| `render_visibility_relevance_translucent_excluded_from_opaque_candidates` | Blend 材质实例不出现在 opaque phase 候选 | 同上 |
| `render_visibility_relevance_cache_invalidates_with_material_generation` | 改材质域后 relevance 与 02 命令缓存同帧失效(共用 generation) | 同上 |
| `render_visibility_parallel_frustum_matches_serial_results` | 3000+ 实例下并行与串行内核输出逐元素相等(确定性) | 同上 |
| `render_visibility_shadow_view_culls_independently_from_main` | 光源背后实例:主 view 剔除、shadow view 可见;两 view stats 不同 | 同上 |
| `render_visibility_stats_partition_input_count` | 每 view `layer_filtered + frustum_culled + occlusion_culled + visible == input` | 同上 |
| `render_hzb_size_and_mip_count_for_odd_viewport` | 1923×1081 → hzb 1024×1024、mip_count 11;1×1 视口不崩 | `graphics/tests/render_hzb.rs` |
| `render_hzb_reduce_pass_batches_cover_all_mips` | `reduce_pass_count()` × 批宽 ≥ mip_count,尾批截断正确 | 同上 |
| `render_hzb_graph_declares_persistent_history_resource` | compiled graph 中 `hzb-furthest` 标记持久、不进 transient 池 | 同上 |
| `render_hzb_ssr_consumes_shared_pyramid` | SSR pass 的读集合含 `hzb-furthest`,且 `screen-space-reflection` 私有 pyramid 资源名不存在 | 同上 |
| `render_visibility_occlusion_rewrites_indirect_instance_count` | 全遮挡 batch 的 args word1 == 0(readback);无遮挡 batch 不变 | `graphics/tests/render_hzb.rs`(VC-M3) |
| `render_visibility_occlusion_gate_falls_back_to_cpu_results` | gate 关闭时 compiled graph 无 cull pass,产物与 CPU 基线一致 | 同上 |
| `render_visibility_static_index_incremental_matches_full_rebuild` | 增/删/移对象后 grid 查询结果 == 全量重建结果 | `graphics/tests/visibility.rs`(VC-M4) |

产物对拍:`render_product_shadows.rs` 既有用例做 VC-M1 shadow 隔离回归;VC-M3 在 `render_product_advanced.rs` 增 `render_product_hzb_occlusion_wall_scene`(墙后 64 实例,断言可见 instance 统计下降且最终图像与遮挡关闭时一致——遮挡剔除只省工作不改像素)。

`RenderStats` 新增字段(`backend_types.rs`,命名延续 `last_` 前缀惯例):`last_visibility_view_count`、`last_visibility_frustum_culled_count`、`last_visibility_occlusion_culled_count`、`last_visibility_visible_count`、`last_hzb_mip_count`、`last_hzb_graph_executed_pass_count`。

命令基线:切片期 `cargo check -p zircon_runtime --lib --locked`;里程碑末 `cargo test -p zircon_runtime visibility --locked`、`cargo test -p zircon_runtime render_hzb --locked`、`cargo test -p zircon_runtime render_graph --locked`、`render_product` 回归(§7)。

### 参考实现精读笔记

| 参考符号(真实读到) | 要点 | Zircon 对应物与取舍 |
|----------------------|------|---------------------|
| `FrustumCull(Scene, View, Flags, ..., TaskConfig, TaskIndex)`(SceneVisibility.cpp:756) | 按位字数组分块(`TaskWordOffset = TaskIndex * NumWordsPerTask`)在任务间切分 primitive 区间;`GFrustumCullUseOctree` 默认 false、`GFrustumCullUseFastIntersect` 默认 true——UE 默认线性数组 + 快速平面测试 | `parallel_frustum_cull` 的 rayon 1024-chunk 等价于 word 分块;不引入 octree,印证"切线性数组"裁决;bit array 改为升序索引 Vec(Rust 侧更易保序断言) |
| `IsPrimitiveVisible(View, PermutedPlanePtr, ...)`(SceneVisibility.cpp:548) | 预置换平面布局做 SIMD 球/盒测试 | V1 复用现有 `perspective_visible`/`orthographic_visible` 标量内核;SIMD 化留作性能切片,不进本计划验收 |
| `FRelevancePacket::LaunchComputeRelevanceTask` / `Finalize`(SceneVisibility.cpp:1252/1287) | relevance 在 packet 任务内计算、`Finalize` 单线程合并进 view(`ShadingModelMaskInView |=` 等);`NotDrawRelevant` 反向清可见位 | Zircon 的 relevance 是材质域纯函数,V1 在 extract 后一次性算 + 缓存,不需要 packet 级合并;chunk 局部 stats 按序归并即 Finalize 等价 |
| `FPrimitiveViewRelevance` 位字段(PrimitiveViewRelevance.h:20-70):`bOpaqueRelevance`、`bMaskedRelevance`、`bShadowRelevance`、`bVelocityRelevance`、`bRenderInDepthPass`、`bRenderInMainPass`、`bRenderCustomDepth` | UE 以 view 级 union 聚合驱动 pass 启停 | `PrimitiveRelevance` 位表直接对位;UE 的编辑器位(`bEditorPrimitiveRelevance` 等)不引入,2D 用 `TWO_D` 自有位 |
| `BuildHZB(GraphBuilder, SceneDepth, ...)`(SceneTextureReductions.cpp:116):`HZBSize = RoundUpToPowerOfTwo(ViewRect) >> 1`、`NumMips = FloorToInt(Log2(max))`、`FHZBBuildCS::kMaxMipBatchSize = 4`、`DispatchThreadIdToBufferUV` + `InputViewportMaxBound` | pow2 尺寸 + 每 dispatch 批量写 4 mip + 源采样 UV clamp 处理非整除视口;furthest/closest 分两张纹理省缓存 | `HzbBuilder` 公式逐项对齐;V1 只建 furthest;`MaxSimultaneousUAVs` 退批逻辑(首批降为 3 mip)wgpu 下无对应限制,不引入 |
| `GetHZBParameters(GraphBuilder, View, ...)`(HZB.cpp:53) | 消费侧参数统一打包(UV factor、extent),消费方不自行换算 | `zr_hzb.wgsl` 的函数式 include 承担同职责,SSR/SSAO/粒子统一入口 |
| `FInstanceCullingManager::RegisterView`/`FlushRegisteredViews`(InstanceCullingManager.cpp:50/98) | 多 view 注册后合批剔除,deferred context 延迟到 graph 执行 | V1 每 view 独立 cull dispatch(view 数少);多 view 合批列为 VC-M3 后优化项 |
| `BuildInstanceDrawCommands.usf:312/338`:`InterlockedAdd(DrawIndirectArgsBufferOut[IndirectArgIndex * INDIRECT_ARGS_NUM_WORDS + 1], ...)`;`ClearIndirectArgInstanceCountCS`(:358) | instance_count 在 args word1 原子累加;独立 clear pass 先置零 | `zr_hzb_occlusion_cull` + `zr_clear_indirect_instance_counts` 同构;word1 口径写进 03 `IndirectDrawBatcher` 的布局契约 |
| `InstanceCullingOcclusionQuery.usf:113`:`IsVisibleHZB(Rect, bSample4x4)`;BuildInstanceDrawCommands.usf:199-202 prev-frame `HZBTestViewRect` + 自遮挡精度注释 | 屏幕矩形 → mip 选择 → 多点采样保守判定;上帧 HZB 测试需防自遮挡精度误差 | mip 选择公式 + `mip_bias`(默认 1)即自遮挡防线;2×2 采样代替 4×4,以偏置换带宽 |
| `FSceneCullingBuilder` temp-cell 增量更新(SceneCulling.cpp:803,1149-1150 注释) | grid cell 首次触碰时建 temp cell 记录增删,结束统一回写,避免全量重建 | VC-M4 `static_index` 采用同思路:`VisibilityBvhUpdatePlan` diff → temp-cell 合并回写;V1 不做 UE 的层级 implicit grid,单层均匀 grid 即可满足"帧间零重建"验收 |

## 风险与回退

- HZB 重投影在相机剧烈运动时保守性不足导致误剔:采用一帧延迟 + 保守 mip 偏置;出现闪烁时先放宽偏置再查重投影矩阵。
- relevance 缓存失效与计划 02 缓存失效耦合:两者共用同一 generation 来源,单测覆盖"改材质域后 relevance 与命令同时失效"。
- 多 view 剔除成本上升:shadow cascade 共享中间结果(方向光各级联只做距离细分),避免每级全量剔除。
