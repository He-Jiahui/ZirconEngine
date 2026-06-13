---
related_code:
  - zircon_runtime/src/graphics/extract/history.rs
  - zircon_runtime/src/graphics/types/viewport_motion_vector_object_history.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_motion_vector_camera/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_motion_vector_tile_max/mod.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/core/framework/render/camera.rs
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
- `history.rs` 有 history 槽位与 `history_resolve` 描述符,但无 jitter、无 resolve shader,history 内容"名义 scene color 实际混杂后处理"(P0 风险原文),重投影与 disocclusion 缺失。
- 相机契约(`camera.rs`)无 jitter 字段,上游 pass 无感知。

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
- `mesh/build_mesh_draws/build.rs` 的 GPUScene 同步阶段已改为双来源 previous:旧
  `ViewportMotionVectorObjectHistory` 命中时仍优先使用 CPU previous,缺失时读取 GPUScene 已滚动的
  previous transform 并将 `GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM`、motion params 与 MeshDraw 的
  `has_previous_motion_vector_transform` 一并置为有效。这样 TP-M1 后续删除 CPU object-history 路径前,
  unskinned 动态对象已经具备 GPUScene fallback;skinned GPU motion vector 仍要求 previous palette 存在。
- 仍未完成:旧 `ViewportMotionVectorObjectHistory` 字段、`update_motion_vector_history_after_success`、
  post_process 下的 motion-vector 三 pass、以及 velocity pass executor id 迁移仍保留到 TP-M1 后续硬切片;
  本切片不声称对象级 velocity 全覆盖已完成。
- 校验(2026-06-14):scoped `rustfmt --edition 2021 --check`、8 文件 source-contract scan、
  scoped tracked `git diff --check` 与未跟踪/忽略文件 trailing-whitespace scan 均通过
  (`git diff --check` 仅报告 LF/CRLF 提示)。锁定
  `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-prev-roll-0614 --message-format short --color never`
  在编译前被根 `Cargo.lock` 需要刷新阻塞;本切片未修改 lockfile。

实施切片:
1. GpuScene prev transform 滚动写入;骨骼 prev palette 槽位。
2. velocity pass(动态对象)+ 相机重投影补全屏;`needs_velocity` relevance 接计划 04。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime temporal --locked`(新模块)
- 验收证据:移动物体 velocity 非零、静止物体在移动相机下 velocity 等于相机重投影差(readback 断言);motion blur 切换消费新 velocity 后产物不回退。

### TP-M2 jitter 接入

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

测试阶段:
- `cargo test -p zircon_runtime temporal --locked`(静止场景收敛性:N 帧后帧间差趋零的 readback 断言)
- 验收证据:边缘锯齿收敛对比截图;快速遮挡切换无拖影(disocclusion 生效,RenderDoc 抓帧记录)。

### TP-M4 顺序整合与 P0 关闭

实施切片:
1. 后处理链按定稿顺序重排(与计划 07 同步);删除旧 history_resolve 路径。
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
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/shaders/velocity_camera.wgsl` | 相机重投影全屏 pass(entry `fs_velocity_camera`) | WGSL |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/shaders/velocity_object.wgsl` | 对象 velocity 模板(entry `vs_velocity_object` / `fs_velocity_object`) | WGSL |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/shaders/taa_resolve.wgsl` | TAA resolve(entry `fs_taa_resolve`) | WGSL |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/shaders/zr_motion.wgsl` | 共享 include:velocity 编解码与重投影函数,无 entry point(index.md §8 第 3 条,计划 08 模板消费) | WGSL |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs` | `temporal` feature descriptor(三个 pass 声明) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs` | prev transform / prev palette 滚动写入(模块归计划 03,本文件写入逻辑归本计划,见计划 03"风险与回退"第 3 条) | graphics 实现 |
| `zircon_runtime/src/graphics/tests/render_product_temporal.rs` | velocity/TAA 产物对拍测试 | 测试 |

修改文件:

| 修改文件 | 改动 |
|---|---|
| `zircon_runtime/src/core/framework/render/camera.rs` | `ViewportCameraSnapshot` 增加 `#[serde(default)] pub temporal_jitter: TemporalJitterSample`(默认零) |
| `zircon_runtime/src/core/framework/render/anti_alias/settings.rs` | `resolve()` 的 `history_available` 接 `TemporalHistoryStore` 真实状态(既有 `AntiAliasMode::Taa`/`UnsupportedTaa` fallback 机制不动) |
| `zircon_runtime/src/core/framework/render/post_process/stack.rs` | `PostProcessGraphResourceNames` 增加 `SCENE_VELOCITY`、`TAA_HISTORY_PREVIOUS`、`TAA_HISTORY_CURRENT`、`TAA_OUTPUT`;删除 `HISTORY_PREVIOUS_SCENE_COLOR`、`HISTORY_CURRENT_SCENE_COLOR`、`HISTORY_OUTPUT_SCENE_COLOR`;effect 链顺序定稿(TAA → DoF → motion blur → 其余,与计划 07 链定稿表一致) |
| `zircon_runtime/src/graphics/extract/history.rs` | `FrameHistorySlot::SceneColor` 重命名为 `TaaSceneColor`(语义定稿:只存 TAA resolve 输出,后处理不写回) |
| `zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs` | 增 `Temporal` 变体;删 `HistoryResolve` 变体 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs` | 删除 `motion-vector-clear` / `motion-vector-camera` / `motion-vector-object` 三个 pass(迁入 temporal feature);tile-max / neighbor-max 改读 `SCENE_VELOCITY` |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs` | 分发 `Temporal`,删除 `history_resolve` 分支 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` | 注册 `temporal.*` 三个 executor,注销 `post.motion-vector-*` 与 `history.scene-color` |
| `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs` + `from_frame.rs` | `view_proj` 改为 jittered;新增 `view_proj_unjittered`、`previous_view_proj_unjittered`(替代 `previous_view_proj`)、`jitter_params: [f32; 4]`(xy=本帧像素 jitter,zw=上帧) |
| `zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs` + `new.rs` | 增加 `temporal_frame_index: u64` 与 `previous_unjittered_view_proj`;`motion_vector_object_history` 字段删除 |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs` | jitter 注入与帧末翻转调用点(见"帧时序与集成点") |
| `zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs` + `prev_transform.rs` | `GpuSceneEntry` 记录已滚动 previous 状态;帧末把 current transform 滚动到 `prev_world_from_local`,并把变更 span 留给下一帧上传 |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs` + `scene_renderer_core_render_scene/render_scene.rs` | 成功 `queue.submit(...)` 后触发 GPUScene previous-transform 滚动,保持当帧 velocity 读取旧 previous、下一帧读取本帧 current |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs` | pending draw 同步 GPUScene 时优先使用 CPU previous history,缺失时 fallback 到 GPUScene rolled previous,并把有效 previous 传播到 primitive flags、motion params 与 MeshDraw |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_motion_vector_pipeline.rs` | 重命名/迁移为 temporal velocity pipeline 缓存,改用 GpuScene instance index ABI |

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

// taa/taa_resolve_params.rs —— 质量档映射
pub(crate) struct TaaQualityPreset {
    pub current_frame_weight: f32,     // Low 0.1 / Medium 0.05 / High 0.04
    pub variance_clamp_scale: f32,     // 对齐 URP Settings.varianceClampScale,默认 0.9
    pub velocity_disocclusion_threshold: f32, // NDC 单位,默认 0.02
}
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

TAA resolve pass binding(group0 = scene uniform;group1 = pass 级输入,遵循 index.md §8 第 1 条;现状 `execute_motion_vector_camera` 把 pass 输入塞 group0 属违例,迁移时一并纠正):

| group | binding | 资源 | WGSL 类型 |
|---|---|---|---|
| 1 | 0 | scene color(post-lighting) | `texture_2d<f32>` |
| 1 | 1 | scene depth | `texture_depth_2d` |
| 1 | 2 | scene velocity | `texture_2d<f32>` |
| 1 | 3 | TAA history previous | `texture_2d<f32>` |
| 1 | 4 | linear sampler | `sampler` |
| 1 | 5 | point sampler | `sampler` |
| 1 | 6 | `TaaResolveParams` | `var<uniform>` |

`TaaResolveParams` uniform(帧级小块,允许 uniform,index.md §8 第 2 条):

```text
viewport_and_flags  : vec4<u32>  // offset 0:  x,y=尺寸 z=camera_cut w=quality
weights             : vec4<f32>  // offset 16: x=current_frame_weight y=variance_clamp_scale
                                 //            z=velocity_disocclusion_threshold w=保留
```

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
→ temporal.taa-resolve       (RenderPassStage::PostProcess 头部;读 SCENE_COLOR/SCENE_DEPTH/
                              SCENE_VELOCITY/TAA_HISTORY_PREVIOUS,写 TAA_HISTORY_CURRENT + TAA_OUTPUT)
→ DoF → motion blur(tile-max/neighbor-max 改读 SCENE_VELOCITY)→ 其余后处理(读 TAA_OUTPUT 为 scene color)
```

prev 数据翻转时机(全部收口在 `submit/submit.rs` 的 present 成功路径,即今天 `update_motion_vector_history_after_success` 的调用点):

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

**TP-M1-S1 GpuScene prev 数据面**:触碰 `gpu_scene/prev_transform.rs`(新增)、`gpu_scene/mod.rs`(声明)、`submit/submit.rs`(帧末调用)。要点:prev transform 滚动 + skinned prev palette 双缓冲 + 翻转时机收口。完成判据:check 通过;prev 槽布局常量与计划 03 `primitive_data` SOA 偏移一致(代码注释互引)。

**TP-M1-S2 velocity 双 pass 与旧路径删除**:触碰 `temporal/velocity/*`(新增)、`feature_descriptors/temporal.rs`(新增)、`builtin_render_feature.rs`、`descriptor_for.rs`、`render_pass_executor_registry.rs`、删除表 TP-M1 行全部文件、`post_process.rs` descriptor、tile-max/neighbor-max 读取改名。要点:object pass 走 instance index ABI + 临时 `needs_velocity` 谓词;camera pass 沿用切变检测;`SCENE_VELOCITY` 资源名落地。完成判据:check 通过;`motion_vector` 旧符号 grep 清零(除 tile-max 系命名)。

**TP-M2-S1 契约与矩阵对**:触碰 `temporal_jitter.rs`、`view_matrix_pair.rs`(新增)、`camera.rs`、`render/mod.rs`(curated re-export)、`scene_uniform.rs`、`from_frame.rs`、`build_frame_submission_context/*`(jitter 注入)。要点:jittered/unjittered 同时下发;TAA 非生效时 jitter 恒零。完成判据:check 通过;`Mat4::perspective_rh` 直接消费方审计清单(grep `perspective_rh|orthographic_rh` in graphics)归零到矩阵对入口。

**TP-M2-S2 上游 pass 审计**:触碰 SSR/SSAO/HZB/deferred lighting 等读投影矩阵的 WGSL 与 params 构造(以 grep `view_proj|inverse_view_proj` 清单驱动)。要点:重投影/反投影类统一 unjittered,光栅类统一 jittered。完成判据:check 通过;审计清单逐文件勾销记录在 PR 描述。

**TP-M3-S1 history store 与 resolve executor**:触碰 `temporal/taa/*`(新增)、`taa_resolve.wgsl`、`zr_motion.wgsl`、`history/scene_frame_history_textures/*`(scene_color 面移除,SSR/AO/GI 面保留)。要点:双缓冲经 `register_external_texture` 注册;camera-cut 路径(`valid == false` → 输出=当前帧、history 直写)。完成判据:check 通过;taa-resolve pass 在 graph dump 中 IO 声明完整(资源四进二出)。

**TP-M3-S2 质量档**:触碰 `taa_resolve_params.rs`、quality profile 消费点(`RenderQualityProfile` 既有通道)。要点:`TaaQualityPreset` 三档;参数全走 uniform 不产生 shader 排列。完成判据:check 通过;档位切换不触发 pipeline 重建(`CompiledGraphCache` 键不含 TAA 参数)。

**TP-M4-S1 链序定稿与 history_resolve 删除**:触碰删除表 TP-M4 行全部文件、`stack.rs`(effect 顺序)、与计划 07 同步的 post 链重排。完成判据:check 通过;`history_resolve|HistoryResolve` grep 清零。

**TP-M4-S2 P0 关闭**:触碰 `.codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md`(P0 条目标记)、`docs/zircon_runtime/**` 镜像文档。完成判据:里程碑测试阶段全绿 + RenderDoc 抓帧证据归档。

### 测试与验收清单

命名遵循 index.md §8 第 6 条(`render_<topic>_*` 单测、`render_product_*` 对拍)。

| 测试函数 | 位置 | 断言 |
|---|---|---|
| `render_velocity_prev_transform_rolls_after_present` | `gpu_scene/prev_transform.rs` `#[cfg(test)]` | 帧 N 写入的 transform 在帧 N+1 readback 出现在 prev 槽;新注册条目 prev==current |
| `render_gpu_scene_rolls_current_transform_into_previous_after_success` | `gpu_scene/prev_transform.rs` `#[cfg(test)]` | 成功提交后的 roll 把 current transform 写入 previous 槽,标记下一帧 instance span 上传,并让 entry 的 previous 状态变为有效 |
| `render_gpu_scene_roll_marks_previous_valid_without_dirty_upload_when_unchanged` | `gpu_scene/prev_transform.rs` `#[cfg(test)]` | current 与 previous 已一致时只标记 previous 可用,不产生下一帧脏上传 |
| `render_velocity_skinned_prev_palette_flips_at_frame_end` | 同上 | palette 双缓冲翻转后 read 面等于上帧 write 面 |
| `render_velocity_object_nonzero_for_moving_mesh` | `graphics/tests/render_product_temporal.rs` | 移动 Dynamic mesh 中心像素 readback `|velocity| > 0`;静止 Dynamic mesh 为 0(对齐 UE 0.0001 容差跳过) |
| `render_velocity_camera_matches_reprojection_for_static_scene` | 同上 | 移动相机+静止物体:velocity 等于双矩阵重投影差(readback 与 CPU 参考值逐像素容差 1e-3) |
| `render_velocity_clears_on_camera_cut` | `temporal/velocity/velocity_camera_params.rs` `#[cfg(test)]` | 沿用既有切变阈值用例(平移/旋转/FOV/clip 切变 → disabled) |
| `render_taa_jitter_sequence_is_periodic_and_centered` | `core/framework/render/temporal_jitter.rs` `#[cfg(test)]` | 周期 8/16 内均值趋零、每分量 ∈ [-0.5,0.5]、与 halton(2,3) 参考值一致 |
| `render_taa_jitter_zero_when_taa_inactive` | `build_frame_submission_context` 测试 | Off/Fxaa/Msaa/fallback 下 `temporal_jitter == default` |
| `render_taa_matrix_pair_jittered_unjittered_consistent` | `view_matrix_pair.rs` `#[cfg(test)]` | jittered == translate(2j/size) * unjittered;jitter 零时两者相等 |
| `render_taa_resolve_converges_on_static_scene` | `render_product_temporal.rs` | 静止场景 N=16 帧后连续两帧输出差的 max-abs 趋零(readback 断言,正文 TP-M3 验收) |
| `render_taa_resolve_rejects_history_on_disocclusion` | 同上 | 遮挡体快速移开后暴露区像素与无 history 单帧渲染差 < 阈值(无拖影) |
| `render_taa_history_invalidates_on_resize_and_camera_switch` | `temporal_history_store.rs` `#[cfg(test)]` | resize/target 切换后 `valid==false`,下一帧 camera-cut 路径 |
| `render_taa_pass_absent_when_disabled` | `graphics/tests/pipeline_compile.rs` 追加 | TAA off 时 compiled graph 无 `temporal.taa-resolve` 节点(culled/未声明断言) |

"关闭 TAA 逐像素一致"的 `render_product_*` 对拍(TP-M2 硬验收线):在 `render_product_temporal.rs` 中以既有 render_product 基线机制对拍——`render_product_temporal_off_matches_pre_jitter_baseline`:同一 extract 夹具在 `AntiAliasSettings::off()` 下渲染,产物 hash 必须等于 jitter 改造合入前录制的基线 hash(基线在 TP-M2-S1 合入前一提交录制并入库);另跑全量既有 `render_product` 系列(`cargo test -p zircon_runtime render_product --locked`)守护 SSR/SSAO/UI 等间接消费方。里程碑测试命令沿用正文:`cargo test -p zircon_runtime temporal --locked`、`cargo test -p zircon_runtime camera --locked`、TP-M4 全量 `--lib` 回归。

### 参考实现精读笔记

**UE `Renderer/Private/PostProcess/TemporalAA.cpp`**:着色器类 `FTemporalAA`,排列维 `FTAAPassConfigDim`(`ETAAPassConfig`:Main/SSR/DOF/Hair 等,history 输出名表 `kTAAOutputNames` 含 `TAA.History`/`SSR.TemporalAA`)与 `FTAAQualityDim`。`FParameters` 关键字段:`CurrentFrameWeight`(CVar `r.TemporalAACurrentFrameWeight` 默认 0.04)、`SampleWeights[9]`/`PlusWeights[5]`、`bCameraCut`、`HistoryBuffer[FTemporalAAHistory::kRenderTargetCount]`、`ScreenPosToHistoryBufferUV`。`SetupSampleWeightParameters` 把 3x3 采样偏移减去 `TemporalJitterPixels` 后过 `CatmullRom(x)` 或高斯 `exp(-2.29*d²)` 归一化——即"当前帧滤波核以 jitter 为中心"。`AddTemporalAAPass(GraphBuilder, View, Inputs, InputHistory, OutputHistory)`:`bCameraCut = !InputHistory.IsValid() || View.bCameraCut`;输出经 `QueueTextureExtraction` 进 `OutputHistory->RT[i]` 跨帧持有。Zircon 对应:`TaaResolveExecutor` + `TemporalHistoryStore.register`(QueueTextureExtraction ↔ 计划 01 持久资源注册);取舍:V1 不做多 RT history(`kRenderTargetCount` 面)、不做 SSR/DOF 专用 TAA 排列——SSR history 仍走自有 `SCREEN_SPACE_REFLECTION_HISTORY` 槽,只共享 `SCENE_VELOCITY`;jitter 加权滤波核 V1 简化为均匀 3x3(权重表接口预留在 `TaaResolveParams` 扩展位)。

**UE `Renderer/Private/VelocityRendering.cpp`**:格式 `FVelocityRendering::GetFormat` = `PF_G16R16`(需 velocity-depth 时 `PF_A16B16G16R16`);三种输出位置 `BasePassCanOutputVelocity` / `DepthPassCanOutputVelocity` / 独立 velocity pass,Zircon 取独立 pass(URP 同款,改造面最小)。对象筛选两级:`FOpaqueVelocityMeshProcessor::PrimitiveHasVelocityForFrame`——`AlwaysHasVelocity()` 否则取 `Scene->VelocityData.GetComponentPreviousLocalToWorld` 与当前 `LocalToWorld.Equals(..., 0.0001f)` 比较,未动即跳过;`PrimitiveHasVelocityForView`——camera cut 跳过、屏占比小于 `MotionBlurPerObjectSize` 推出的 `MinScreenRadiusForVelocityPass` 跳过。Zircon 对应:前者即 TP-M1 临时谓词→计划 04 `needs_velocity` 位;后者(屏占比剔除)V1 不做,记为计划 04 HZB 切片的后续项。`UseDefaultMaterial` 简化(无遮罩/无顶点修改材质换默认材质)对应 velocity object pass 不绑 group2。

**URP `Runtime/Passes/MotionVectorRenderPass.cs`(含 `MotionVectors.cs`)**:`k_TargetFormat = GraphicsFormat.R16G16_SFloat` 直接背书 RG16F;执行序"先 `DrawCameraMotionVectors`(全屏 procedural 3 顶点三角形 + depth 重建)后 `DrawObjectMotionVectors`(LightMode tag `MotionVectors` 的 renderer list,`PerObjectData.MotionVectors` 提供 `previousLocalToWorld`)"——Zircon 取反序(先对象后相机补底),因对象 pass 带 depth test 写 velocity,相机 pass 只补 velocity==0 处,语义等价且省一次全屏混合判断;`camera.depthTextureMode |= MotionVectors | Depth` 的"prev 矩阵由引擎侧每帧维护"对应 GpuScene prev 槽帧末滚动。

**URP `Runtime/TemporalAA.cs`**:`CalculateJitter`:`HaltonSequence.Get((frameIndex & 1023) + 1, 2/3) - 0.5` ——`TemporalJitterSequence::sample` 照搬(含 +1 避 index 0);`CalculateJitterMatrix`:`Matrix4x4.Translate(2*jitter.x/width, 2*jitter.y/height, 0)` 左乘投影——`ViewProjectionMatrixPair::from_camera` 同式;`Settings.m_FrameInfluence`(原 frameInfluence)与 `varianceClampScale` ↔ `TaaQualityPreset` 两参数;`jitterFrameCountOffset` 用于测试确定性 ↔ 我们直接用 `temporal_frame_index` 注入测试;`TemporalAADescFromCameraDesc` 强制 msaa=1、无 mip、`AccumulationFormatList` 首选 `R16G16B16A16_SFloat` ↔ `TemporalHistoryKey` 约束;`Render()` 的 `isNewFrame = GetAccumulationVersion(multipassId) != Time.frameCount`、重绘帧用 `blackTexture` 当 motion 源——对应本计划"提交失败/跳帧不翻转 + velocity 置黑"规则;`ValidateAndWarn` 的 MSAA/动态分辨率/相机栈禁用矩阵 ↔ `AntiAliasSettings::resolve` 的 fallback 报告(`UnsupportedTaa`),动态分辨率与 TAA 共存推迟到计划 07 动态分辨率定稿,V1 共存时按 URP 行为禁用 TAA 并出 fallback 报告。

## 风险与回退

- 透明物 velocity 缺失导致 TAA 拖影:V1 接受(UE/URP 同样默认不写透明 velocity),responsive 标记作为后续项记录。
- jitter 波及面大(一切投影矩阵消费方):TP-M2 用全量 render_product 对拍守护,"关闭 TAA 时逐像素一致"是硬验收线。
- 上采样类 TAA(TSR/STP 风格)不在本计划:接口预留 resolve 输入输出分辨率不等的可能,实施另立计划。
