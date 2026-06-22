---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshPassProcessor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshPassProcessor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PrimitiveSceneProxy.cpp
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
  - dev/bevy/crates/bevy_render/src/render_phase/draw.rs
  - dev/bevy/crates/bevy_render/src/render_phase/draw_state.rs
  - dev/bevy/crates/bevy_render/src/render_phase/rangefinder.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/bundle.rs
plan_sources:
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/ZirconEngine WGPU 渲染主链闭环计划.md
---

# 计划 02:MeshDrawCommand 绘制命令管线

## 目标

把"每帧从 extract 全量重建 `MeshDraw` 并逐条录制"的现状,升级为 UE 式三段结构:
mesh batch(extract 快照)→ 每 pass 的 MeshPassProcessor 转换 → 不可变 `MeshDrawCommand` 列表(静态部分跨帧缓存),
录制阶段只做排序 + 状态去重 + 重放。完成后:

1. 每个 RenderPhase(DepthPrepass/Shadow/Opaque3d/AlphaMask3d/Transparent3d)拥有独立的 pass processor,pass 专属逻辑(如 shadow 不需要 material 颜色绑定)收敛到各自 processor。
2. 静态网格的 draw command 跨帧缓存,可见性变化只影响筛选,不触发重建。
3. 排序键统一为打包整数(state bucket),录制时相邻同状态命令自动省略重复绑定。

## 现状与差距

- `mesh/build_mesh_draws/` 每帧重建全部 `MeshDraw`(含 batch key、skinning、LOD 决策),`prepared_queue.rs` 只输出统计性的 batch 候选,没有"命令"这一不可变中间产物。
- pass 间差异散落在 `builtin_scene_executors.rs` 的执行分支里(opaque/alpha-mask/transparent 三个 executor 共享同一份 draw 数据,各自再过滤),没有 per-pass processor 概念。
- 无跨帧缓存:静态场景每帧付出同样的 CPU 构建成本,这是与 UE cached mesh draw commands 最大的差距。
- 排序在 `phase_sort.rs` 有 phase 级排序,但没有 PSO/material 维度的 state bucket,录制时每条 draw 全量重绑。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshPassProcessor.h` | `FMeshPassProcessor::AddMeshBatch` 接口;`EMeshPass` 枚举;`FMeshPassProcessorRenderState`(per-pass 渲染状态);`FCachedMeshDrawCommandInfo` |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshPassProcessor.cpp` | `BuildMeshDrawCommands` 模板流程:shader 选择 → 绑定收集 → 排序键构造 → 提交到 DrawListContext |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.h/.cpp` | `FMeshDrawCommand` 的不可变结构(PSO id、ShaderBindings、draw args)、`SubmitMeshDrawCommands` 的状态去重重放、并行翻译任务 |
| `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PrimitiveSceneProxy.cpp` | proxy → `FMeshBatch` 的生成边界:哪些信息属于 extract 快照,哪些属于 pass 决策(对应本引擎 `scene_extract.rs` 与 processor 的分工) |

次参考:`dev/bevy/crates/bevy_render/src/render_phase/`(`PhaseItem`/`RenderCommand` 的 Rust trait 表达与排序键打包)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_render/src/render_phase/mod.rs` | per-phase 命令容器与跨帧缓存 | `BinnedRenderPhase` 两级 key(`BatchSetKey` → `BinKey` → entity)的 binned 组织;`cached_entity_bin_keys`/`CachedBinKey` 跨帧保留 bin 归属、变更才重 bin——与 `CachedMeshDrawCommands` 的保留/失效思想同源 |
| `dev/bevy/crates/bevy_render/src/render_phase/draw.rs` | MeshPassProcessor 的 pass 专属逻辑 | `Draw`/`RenderCommand` trait 与元组组合(如 `SetItemPipeline` + 各 bind 命令):bevy 把 per-pass 差异编码为命令元组类型;Zircon 改为 processor 在构建期固化进不可变命令,录制期不再做选择 |
| `dev/bevy/crates/bevy_render/src/render_phase/draw_state.rs` | 重放状态去重 | `DrawState` 跟踪当前 pipeline/bind group(含 dynamic offsets)/vertex/index buffer,`TrackedRenderPass::set_*` 相同即跳过——与重放器的 state_change 去重直接对应,移植时保留其 buffer slice 粒度判等 |
| `dev/bevy/crates/bevy_render/src/render_phase/rangefinder.rs` | 深度排序键 | `ViewRangefinder3d` 由 view 矩阵行算 view-space Z 距离;transparent 排序键的最小 Rust 表达 |
| `dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs` | sort_key 字段取舍 | `Opaque3dBatchSetKey`/`Opaque3dBinKey`(pipeline、draw_function、material/mesh id 进 key)与 `Transparent3d` 按距离排序;打包 u64 时该选哪些维度、opaque 与 transparent 语义差异的实例 |
| `dev/Fyrox/fyrox-impl/src/renderer/bundle.rs` | 批次聚合与排序 | `RenderDataBundleStorage` 以 persistent identifier 哈希聚 bundle、`sort()` 按 `sort_index` 单键排序;单线程简化形态,适合校对"批次身份 = 几何+材质+排序键"的最小判据 |

`不可变 MeshDrawCommand(命令自持全部绑定、静态部分跨帧缓存)` 无 Rust 同类参照(bevy 以泛型 RenderCommand 每帧重放、Fyrox 即时绑定),实现时以 UE 为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:`zircon_runtime/src/graphics/scene/scene_renderer/mesh/` 内部重组,新增 `mesh_pass/` 子模块;phase 契约仍在 `core/framework/render/core_pipeline/`。

核心类型:

- `MeshDrawCommand`(不可变):`pipeline_variant_id`(指向 `mesh_pipeline_cache` 条目)、bind set 引用(model/material/view 槽位句柄)、vertex/index buffer 句柄、draw args、`sort_key: u64`、可选 `instance_slot`(计划 03 接 GPUScene 后改为 instance index)。
- `trait MeshPassProcessor`:`fn add_mesh_batch(&mut self, batch: &MeshBatchRef, ctx: &PassBuildContext, out: &mut MeshDrawCommandList)`;每 phase 一个实现:`DepthPassProcessor`、`ShadowPassProcessor`、`OpaquePassProcessor`、`AlphaMaskPassProcessor`、`TransparentPassProcessor`。pass 相关选择(shader 变体、混合态、是否需要 material 绑定)只发生在这里。
- `CachedMeshDrawCommands`:按 (mesh instance id, phase) 索引的静态命令存储 + generation 计数;mesh/material/transform 静态性由 extract 标记,失效时按 id 重建单条。动态(skinned/morph/每帧变换)走每帧 dynamic 列表,与缓存列表合并提交。
- `sort_key` 打包:高位 phase 内排序语义(opaque:pipeline → material → 深度前向;transparent:深度后向优先),低位 pipeline/material id;`phase_sort.rs` 改为对 u64 排序的薄层。
- 重放器:按排序后命令流录制,跟踪当前 PSO/bind group,相同则跳过重绑;输出 `draw_call_count` 与 `state_change_count` 统计供测试断言。

硬切换:`build_mesh_draws` 中被 processor 取代的 pass 分支逻辑删除;`prepared_queue.rs` 的 batch 候选统计保留但改为从命令列表派生。

## 里程碑

### MD-M1 MeshDrawCommand 类型与 per-pass processor

实施切片:
1. `MeshDrawCommand`/`MeshDrawCommandList` 与 sort_key 打包规则。
2. 五个内建 pass processor;executor 改为消费命令列表;删除 executor 内的临时过滤分支。
3. `phase_sort.rs` 收敛为 u64 排序。

2026-06-12 当前落地状态:已新增 `graphics/scene/scene_renderer/mesh/mesh_pass/` 命令层骨架,包含 `MeshDrawCommand`、`MeshDrawCommandList`、pipeline key/variant、command-owned WGPU bind/geometry handle、direct/indirect draw args、model-uniform/GPUScene instance source 和命令列表排序/统计测试。第一组 processor 壳层也已落地:`DepthPrepassProcessor`、`OpaqueBasePassProcessor`、`TransparentPassProcessor`、`ShadowPassProcessor`、`VelocityPassProcessor` 可从轻量 `MeshBatchRef` 生成命令。`MeshDraw::mesh_pass_batch_ref(sort_key, source_draw_index)` 已提供真实 draw 到命令适配器的入口,会 clone 当前 mesh、model bind group、材质纹理 bind group、base color 纹理 bind group、custom/standard material uniform bind group 和 indirect args buffer。`MeshPipelineCache` 现在持有 `MeshPipelineVariantRegistry`,以 `MeshPassPipelineKind + PipelineKey` 分配稳定非零 `MeshPipelineVariantId`;pass processor 通过 `MeshPassBuildContext` 解析 variant,固定 depth/shadow pipeline 继续保留 variant id 0。forward/base 与 motion-vector 录制已通过 `ensure_pipeline_for_variant(...)` / `ensure_motion_vector_pipeline_for_variant(...)` 由 variant id 回查 cache-backed WGPU pipeline,不再在 replay 阶段直接以命令 `PipelineKey` 查 pipeline。`phase_sort.rs` 已新增 `packed_sort_key_u64(...)`,命令生成会把 draw ordinal 作为 tie-breaker 打包;opaque/prepass/velocity 等非透明 phase 先按 pipeline variant + material discriminant 聚簇,transparent 忽略 pipeline/material 并保留深度/tie-breaker 主导。`MeshPassCommandBuffers` 现在会在 `SceneRendererCore::render_compiled_scene` 中、`assign_execution_owned_indirect_args` 之后由现有 `MeshDraw` 列表构建;旧 `RenderPassMeshDrawLists` 已收敛为 `RenderPassMeshCommandLists`,图执行上下文向 prepass/base/shadow/deferred gbuffer/object motion-vector 录制器传递命令切片。`partition_mesh_draws.rs` 与旧 `mesh_draw/render_pass_bindings.rs` 已删除,图执行和 overlay legacy mesh 入口均改为生成/消费命令,各录制循环通过 `MeshDrawCommandReplayer` 直接绑定命令携带的 WGPU 资源;`source_draw_for_command` 桥接已移除。`PreparedMeshQueueStats` 已携带命令计数、cache hit/rebuild/dynamic 计数与 replay `state_change_count`/`bind_skip_count`。当前 MD-M1 已完成 command-owned WGPU 重放、稳定 cache-backed variant registry、variant-id pipeline lookup、过渡 packed sort key 和基础 replay stats 汇总的核心迁移,但相机深度/queue 元数据接入与最终计划 09 位段仍未落地。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`
- `cargo test -p zircon_runtime mesh --locked` 与 `cargo test -p zircon_runtime phase --locked`、`render_product` 回归
- 验收证据:同场景渲染产物不变;每 phase 命令数与旧路径 draw 数一致的对拍断言。

### MD-M2 静态命令缓存与失效

实施切片:
1. extract 增加静态性标记(transform/material/geometry 三维度)。
2. `CachedMeshDrawCommands` 存储、generation 失效、单条重建。
3. 帧构建只为动态对象生成命令,静态对象走缓存 + 可见性筛选。

测试阶段:
- `cargo test -p zircon_runtime mesh --locked`(新增:静态场景第二帧命令重建数为 0 的断言;改材质后仅该 id 重建)
- 验收证据:静态重场景 prepare 阶段 CPU 时间显著下降(RenderStats 计数对比记入文档)。

2026-06-12 当前落地状态:`RenderMeshSnapshot` 已新增 `static_state: RenderMeshStaticState`,契约层仅携带 `transform_static`、`geometry_revision`、`material_revision`,无 WGPU 类型。`World::render_mesh_snapshots_for_camera` 从 `Mobility::Static` 填充 transform 静态性;renderer 在 `extend_pending_draws_for_mesh_instance` 中用 `ResourceStreamer::{mesh_revision,model_revision,material_revision}` 生成非零 revision signature,geometry signature 同时纳入 source mesh/model id 与 LOD 选择。`PendingMeshDraw` 与 `MeshDraw` 已携带 `(source_entity, source_draw_ordinal, static_state)`,`MeshBatchRef` 转交 `cache_identity` 和 static state 给命令构建。新增 `mesh_pass/cached_mesh_draw_commands.rs`,实现 `CachedMeshDrawKey { entity, draw_ordinal, phase }`、`lookup/store/retain_generation`、静态缓存资格判定和 cache stats;资格限制为 direct prepared non-transparent static 批次,phase 仅 Prepass/Shadow/Opaque3d/AlphaMask3d,transparent 与 velocity 明确拒绝。`SceneRendererCore` 新增 `cached_mesh_draw_commands` 字段,`render_compiled_scene` 使用当前 `model_uniform_generation` 作为缓存 generation,在 indirect args patch 后通过 `build_mesh_pass_command_buffers_cached(...)` 接入,随后 retain 当前 generation。`PreparedMeshQueueStats` 已从 command buffer stats 汇总 cache hit/rebuild/dynamic counts。当前实现仍保留每帧 `MeshDraw` 构建来更新 model/material bind 资源和判定 cache key,所以 CPU 收益还不是最终形态;M2-S3 后续仍需把"静态对象跳过 processor 和部分 draw 构造"继续下沉到批次构建阶段。

### MD-M3 状态去重重放

实施切片:
1. 重放器跟踪 PSO/bind group,跳过冗余绑定;统计 state_change_count。
2. 排序键调优:opaque 按 pipeline 聚簇,transparent 保持深度正确性。

测试阶段:
- `cargo test -p zircon_runtime mesh --locked`
- 验收证据:重复材质场景 state_change_count 明显低于命令数;RenderDoc 抓帧确认绑定序列收敛。

### MD-M4 GPUScene 衔接预留

实施切片:
1. 命令中 model 绑定槽位抽象为 `DrawInstanceSource`(uniform 路径 / instance index 路径二选一),为计划 03 切换做好接口,不留双实现。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked` + mesh 范围测试回归
- 验收证据:接口落地且 uniform 路径行为不变;计划 03 启动时无需再改命令结构。

## 工程落地细化

本章是本计划的实施权威(见 index.md §8 第 7 条)。术语约定:**批次(MeshBatch)** = 现有 `MeshDraw` 改名后的产物,即 extract 经 `build_mesh_draws` 解析出 GPU 资源后的快照中间体;**命令(MeshDrawCommand)** = 各 pass processor 从批次转换出的不可变录制单元。bind group 槽位语义、storage buffer 布局、`RenderQueueValue` 数值段、sort_key 位段权威均直接引用 index.md §8,不在此重定义。

### 模块与文件落点

新增文件(全部在 `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/` 下,挂入既有 `mesh/mod.rs`):

| 新增文件 | 职责 |
|---|---|
| `mesh/mesh_pass/mod.rs` | 模块声明与 `pub(crate)` 导出(薄壳,无行为) |
| `mesh/mesh_pass/mesh_draw_command.rs` | `MeshDrawCommand`、`MeshDrawArgs`、`DrawInstanceSource`、`MeshBindHandle`、`MeshPipelineVariantId` 类型定稿 |
| `mesh/mesh_pass/mesh_draw_command_list.rs` | `MeshDrawCommandList`(排序/追加/统计)与 `MeshPassCommandBuffers`(六条 per-phase 列表容器) |
| `mesh/mesh_pass/mesh_pass_processor.rs` | `trait MeshPassProcessor`、`MeshBatchRef`、`PassBuildContext` |
| `mesh/mesh_pass/cached_mesh_draw_commands.rs` | `CachedMeshDrawCommands`、`CachedMeshDrawKey`、generation 失效与回收 |
| `mesh/mesh_pass/replay.rs` | `MeshDrawCommandReplayer` 状态去重重放器与 `MeshDrawReplayStats` |
| `mesh/mesh_pass/processors/mod.rs` | 五个内建 processor 的模块声明 |
| `mesh/mesh_pass/processors/depth_prepass.rs` | `DepthPrepassProcessor` |
| `mesh/mesh_pass/processors/opaque_base.rs` | `OpaqueBasePassProcessor`(同时产出 Opaque3d 与 AlphaMask3d 两条列表) |
| `mesh/mesh_pass/processors/transparent.rs` | `TransparentPassProcessor` |
| `mesh/mesh_pass/processors/shadow.rs` | `ShadowPassProcessor` |
| `mesh/mesh_pass/processors/velocity.rs` | `VelocityPassProcessor` |

修改/删除文件(均为真实现存路径):

| 文件 | 改动点 |
|---|---|
| `mesh/mod.rs` | 挂 `mesh_pass` 子模块;导出 `MeshBatch`/命令类型;删除 `MeshDraw` 旧导出 |
| `mesh/mesh_draw/mesh_draw.rs` | 改名为 `mesh_batch.rs`,`struct MeshDraw` → `struct MeshBatch`;字段保留,录制行为剥离 |
| `mesh/mesh_draw/render_pass_bindings.rs` | **整文件删除**:`bind_model`/`bind_texture`/`bind_base_color_texture`/`bind_material`/`bind_standard_material`/`bind_geometry_buffers`/`record_indexed_draw` 全部被 replayer 取代 |
| `mesh/mesh_draw/queue_profile.rs` | `MeshDrawQueueProfile`/`MeshDrawBatchKey` 保留,作为 processor 过滤与缓存资格判定的输入 |
| `mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs`(及 `mesh_pipeline_variant_registry.rs`、`ensure_pipeline.rs`、`ensure_motion_vector_pipeline.rs`) | 新增 variant 注册表:`resolve_variant(...) -> MeshPipelineVariantId` 已接入命令构建;cache-backed WGPU pipeline 查询已提供 `ensure_pipeline_for_variant(...)` / `ensure_motion_vector_pipeline_for_variant(...)` |
| `mesh/prepared_queue.rs` | `prepare_mesh_queue` 的 early-z 列表职责移除(归 `DepthPrepassProcessor`);`PreparedMeshQueueStats` 保留并新增命令/缓存/重放计数字段,输入改为批次+命令元数据 |
| `mesh/build_mesh_draws/build/build.rs` | `build_mesh_draws` → `build_mesh_batches`,输出 `Vec<MeshBatch>`(`BuiltMeshDraws` → `BuiltMeshBatches`);`phase_ordered_meshes` 排序逻辑保留 |
| `core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs` | 包装方法同步改名 |
| `core/scene_renderer_core/scene_renderer_core.rs`(及 `scene_renderer_core_construct/construct/construct.rs`) | `SceneRendererCore` 新增 `cached_mesh_draw_commands: CachedMeshDrawCommands` 字段与构造 |
| `core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs` | 改名 `build_compiled_scene_commands.rs`:批次构建 → 缓存查询/processor 转换 → `MeshPassCommandBuffers` 输出 |
| `core/scene_renderer_core_render_compiled_scene/render/partition_mesh_draws.rs` | **整文件删除**(`MeshDrawPartitions`/`partition_mesh_draws` 被 per-phase 命令列表取代) |
| `core/scene_renderer_core_render_compiled_scene/render/render.rs` | `render_compiled_scene` 集成点改造(见"帧时序与集成点") |
| `core/scene_renderer_core_render_compiled_scene/render/assign_execution_owned_indirect_args.rs` | 输入改 `&mut [MeshBatch]`,调用时序固定在命令转换之前(VG indirect args 补丁打在批次上) |
| `graph_execution/render_pass_execution_context/gpu.rs` | `RenderPassMeshDrawLists` → `RenderPassMeshCommandLists`(字段类型 `&[MeshDrawCommand]`);`record_depth_prepass_to_resources`/`record_mesh_stage_to_resources`/`record_shadow_map_to_resource` 改为 replayer 驱动 |
| `graph_execution/render_pass_execution_context/gpu/mesh_motion_vector.rs` | `record_mesh_motion_vectors_to_resource` 内的 `motion_vector_history_eligible` + `has_previous_motion_vector_transform` 过滤循环删除,改消费 velocity 命令列表 |
| `overlay/passes/base_scene_pass.rs` | `BaseScenePass::record_with_attachment_ops` 的逐 draw `ensure_pipeline` + 五连 bind 循环删除,改 replayer;`uses_builtin_fallback_shader` 分支前移到 `OpaqueBasePassProcessor` |
| `prepass/normal_prepass_pipeline/record.rs` | `NormalPrepassPipeline::record_with_attachment_ops` 改消费 depth-prepass 命令列表 |
| `shadow/shadow_map_renderer.rs` | `ShadowMapRenderer::record_with_attachment_ops` 中 per-draw `is_alpha_mask()` 切 pipeline 分支删除(由 `ShadowPassProcessor` 在构建期定 variant),录制改 replayer |
| `core/framework/render/scene_extract.rs` | `RenderMeshSnapshot` 新增 `static_state: RenderMeshStaticState`(契约层,无 wgpu)|
| `core/framework/render/core_pipeline/phase_sort.rs` | 新增唯一 u64 打包入口 `packed_sort_key_u64`(位段权威=计划 09,见下) |

### 核心类型与接口

命令与重放(归属 graphics 实现层 `mesh_pass/`,持 wgpu 句柄,**禁止上移 framework**):

```rust
// mesh_pass/mesh_draw_command.rs
pub(crate) struct MeshPipelineVariantId(pub(crate) u32); // MeshPipelineCache 条目稳定索引

pub(crate) enum MeshPassPipelineKind {
    DepthPrepass,          // NormalPrepassPipeline 固定 PSO
    Base,                  // mesh_pipelines: HashMap<PipelineKey, _> 路径
    ShadowDepth,           // ShadowMapRenderer.pipeline
    ShadowDepthAlphaMask,  // ShadowMapRenderer.alpha_mask_pipeline
    MotionVector,          // motion_vector_mesh_pipelines 路径
}

// bind group 引用:id 为创建点分配的单调递增 u64(ModelUniformCache / 材质缓存各持计数器),
// 重放去重按 id 比较,不依赖 wgpu 资源相等性。
pub(crate) struct MeshBindHandle { pub(crate) id: u64, pub(crate) bind_group: wgpu::BindGroup }

pub(crate) enum MeshDrawArgs {
    DirectIndexed { first_index: u32, index_count: u32, instance_count: u32 }, // 现路径 instance_count 恒 1
    IndexedIndirect { buffer: std::sync::Arc<wgpu::Buffer>, offset: u64 },     // stride = size_of::<IndexedIndirectArgs>()
}

// MD-M4 定稿;M1–M3 期间仅存在 ModelUniform 变体,不留双实现
pub(crate) enum DrawInstanceSource {
    ModelUniform { object_bind: MeshBindHandle },  // 现行:物理 group1(ModelUniform + joint palettes)
    GpuSceneInstance { instance_index: u32 },      // 计划 03 接管后:经 group3 instance buffer 寻址
}

pub(crate) struct MeshDrawCommand {
    pub(crate) phase: RenderPhase,                          // core_pipeline 契约枚举
    pub(crate) pipeline_variant_id: MeshPipelineVariantId,
    pub(crate) sort_key: u64,                               // 不透明值,升序提交;编码见 packed_sort_key_u64
    pub(crate) instance_source: DrawInstanceSource,
    pub(crate) material_textures: Option<MeshBindHandle>,   // 物理 group2;shadow 非 alpha-mask 为 None
    pub(crate) material: Option<MeshBindHandle>,            // 物理 group3;depth/shadow/velocity 走标准材质槽
    pub(crate) geometry: std::sync::Arc<GpuMeshResource>,   // vertex/index buffer 来源
    pub(crate) draw_args: MeshDrawArgs,
}
```

processor 契约与五个内建实现:

```rust
// mesh_pass/mesh_pass_processor.rs
pub(crate) struct MeshBatchRef<'a> {
    pub(crate) batch: &'a MeshBatch,
    pub(crate) entity: EntityId,
    pub(crate) draw_ordinal: u32,                       // 同 entity 多 primitive/LOD 切片的稳定序号
    pub(crate) static_state: &'a RenderMeshStaticState, // 来自 extract
}

pub(crate) struct PassBuildContext<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) streamer: &'a ResourceStreamer,
    pub(crate) mesh_pipelines: &'a mut MeshPipelineCache, // resolve_variant 需要可变借用
    pub(crate) frame: &'a ViewportRenderFrame,
}

pub(crate) trait MeshPassProcessor {
    fn phase(&self) -> RenderPhase;
    /// 过滤(本 pass 是否收此批次)→ 选 variant(MeshPassPipelineKind × PipelineKey)
    /// → 收集绑定槽 → 调 packed_sort_key_u64 → push 不可变命令。pass 专属决策只发生在这里。
    fn add_mesh_batch(
        &mut self,
        batch: &MeshBatchRef<'_>,
        ctx: &mut PassBuildContext<'_>,
        out: &mut MeshDrawCommandList,
    );
}
```

五个实现的逻辑差异(对照现执行代码逐条核对):

| processor | 输出 phase | 过滤条件(现实现出处) | variant 选择 | 绑定差异 |
|---|---|---|---|---|
| `DepthPrepassProcessor` | `Prepass` | `MeshDrawQueueProfile::early_z_eligible()`(原 `prepare_mesh_queue` early_z 列表) | `DepthPrepass` 固定 PSO | object + material_textures + 标准材质(对齐 `NormalPrepassPipeline::record_with_attachment_ops` 现序列) |
| `OpaqueBasePassProcessor` | `Opaque3d` / `AlphaMask3d` | phase = Opaque/AlphaMask(原 `partition_mesh_draws`) | `Base` × `PipelineKey` | `uses_fallback_shader()` 决定 material 槽取 `standard_material_uniform` 还是 `material_uniform`(原 `BaseScenePass` 内分支) |
| `TransparentPassProcessor` | `Transparent3d` | phase = Transparent | `Base` × `PipelineKey` | 同 opaque;sort_key 走深度后向段,**不做 PSO 聚簇** |
| `ShadowPassProcessor` | `Shadow` | `MeshBatch::casts_shadow()` | alpha-mask 批次 → `ShadowDepthAlphaMask`,否则 `ShadowDepth` | 非 alpha-mask:仅 object;alpha-mask:object + base_color 纹理 + 标准材质(对齐 `ShadowMapRenderer::record_with_attachment_ops` 现分支) |
| `VelocityPassProcessor` | `PostProcess` 前置 velocity 列表 | `motion_vector_history_eligible() && has_previous_motion_vector_transform()`(原 `record_mesh_motion_vectors_to_resource` 过滤) | `MotionVector` × `PipelineKey` | object + material_textures + 标准材质 |

缓存与失效(MD-M2):

```rust
// mesh_pass/cached_mesh_draw_commands.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CachedMeshDrawKey {
    pub(crate) entity: EntityId,
    pub(crate) draw_ordinal: u32,
    pub(crate) phase: RenderPhase,
}

pub(crate) struct CachedMeshDrawCommands { /* HashMap<CachedMeshDrawKey, CachedMeshDrawEntry> */ }
impl CachedMeshDrawCommands {
    /// 命中且 (geometry_revision, material_revision) 一致才返回;否则 None(单条重建)
    pub(crate) fn lookup(&mut self, key: &CachedMeshDrawKey,
        state: &RenderMeshStaticState, generation: u64) -> Option<&MeshDrawCommand>;
    pub(crate) fn store(&mut self, key: CachedMeshDrawKey,
        state: &RenderMeshStaticState, command: MeshDrawCommand, generation: u64);
    pub(crate) fn retain_generation(&mut self, generation: u64); // 复用 ModelUniformCache::retain_generation 同款回收策略
}

// core/framework/render/scene_extract.rs(契约层,无 wgpu)
pub struct RenderMeshStaticState {
    pub transform_static: bool,   // Mobility::Static 且变换未脏
    pub geometry_revision: u64,   // mesh/model 资产修订,换 LOD 选择也步进
    pub material_revision: u64,   // 材质资产修订(含 shader_revision 来源)
}
```

缓存资格(全部满足才入缓存):`mobility == Static` && 非 skinned/morph && `geometry_source == Prepared` && 非 indirect/VG && phase ∈ {Prepass, Opaque3d, AlphaMask3d, Shadow}。**Transparent 永不缓存**(sort_key 含相机相关深度,逐帧变化);**Velocity 永不缓存**(静态对象本就不进 velocity 列表)。失效维度:geometry_revision 变 → 该 entity 全 phase 重建;material_revision 变 → 同上(PipelineKey/绑定都可能变);transform_static 失守(static 转 dynamic)→ 删除条目走 dynamic 列表。静态对象的 transform/tint 微调不重建命令——`ModelUniformCache::cached_model_binding` 已保证 bind group 稳定、仅 `queue.write_buffer` 更新内容。

重放与排序键:

```rust
// mesh_pass/replay.rs
#[derive(Clone, Copy, Default)]
pub(crate) struct MeshDrawReplayStats {
    pub(crate) draw_call_count: u32,
    pub(crate) state_change_count: u32, // set_pipeline 实际调用次数
    pub(crate) bind_skip_count: u32,    // 因相邻同 id 跳过的 set_bind_group 次数
}

pub(crate) struct MeshDrawCommandReplayer {
    last_pipeline: Option<MeshPipelineVariantId>,
    last_bind_ids: [Option<u64>; 4],     // 槽位→最后 set 的 MeshBindHandle.id
    last_geometry: Option<usize>,        // Arc::as_ptr(&geometry) 指针身份
    stats: MeshDrawReplayStats,
}
impl MeshDrawCommandReplayer {
    /// pass 级前置(group0 scene、group4 forward shadow receiver)由调用方设置一次,不进命令。
    /// pipeline 变化时清空 bind/geometry 追踪(对齐 UE FMeshDrawCommandStateCache::SetPipelineState)。
    pub(crate) fn replay<'pass>(&mut self, pass: &mut wgpu::RenderPass<'pass>,
        commands: &'pass [MeshDrawCommand], pipelines: &'pass MeshPipelineCache);
    pub(crate) fn stats(&self) -> MeshDrawReplayStats;
}

// core/framework/render/core_pipeline/phase_sort.rs —— 唯一 u64 打包入口。
// 位段布局唯一由计划 09 定义;本计划只以不透明 u64 消费。MD-M1 落地过渡实现
// (把现 RenderPhaseSortKey::for_components 的 i128 分段量化压缩进 u64,保序),
// 计划 09 CO-M3 重排位段时只改本函数内部,调用方零改动。
pub fn packed_sort_key_u64(
    phase: RenderPhase,
    components: RenderPhaseSortComponents,
    pipeline_variant: u32,      // opaque 聚簇用;transparent 段忽略
    material_discriminant: u16, // 同 pipeline 内材质聚簇
) -> u64;
```

### GPU 数据布局与 WGSL 约定

本计划不新增 GPU buffer,只固化命令所引用的绑定布局。现行 mesh 链物理槽位(`fallback_mesh.wgsl` / `scene_renderer_core_construct/layouts/` 既有事实,逐 binding 核对自 `create_mesh_draw.rs`):

| 物理 group | 内容 | binding | 命令内归属 |
|---|---|---|---|
| 0 | SceneUniform(frame/view) | b0 | 不进命令,pass 录制器 set 一次 |
| 1 | ModelUniform(b0,176B:model 64 + tint 16 + shadow_params 16 + previous_model 64 + motion_params 16)+ joint palette(b1)+ prev joint palette(b2) | b0–b2 | `DrawInstanceSource::ModelUniform.object_bind` |
| 2 | 材质纹理 5 对 texture/sampler(base_color/normal/metallic_roughness/occlusion/emissive) | b0–b9 | `material_textures` |
| 3 | 材质 uniform(标准或自定义) | b0 | `material` |
| 4 | forward shadow receiver(shadow map + compare sampler + uniform,pass 级输入) | — | 不进命令,pass 录制器 set 一次 |

与 index.md §8 槽位语义(group0=frame/view、group1=pass、group2=material、group3=object)存在历史错位:现布局 group1 是 object 级、group3 是 material uniform、group4 承担 pass 级。**重排 WGSL 分组归计划 08 的 shader 模板拼接**;本计划以"命令记录语义槽 + replayer 持有唯一『语义槽→物理 index』常量表"兜住,计划 08 重排时只改该表与 shader 模板,命令结构与缓存不动。新写的 WGSL(若有)必须按 §8 布局。draw args 约定:`IndexedIndirect.offset` 必须是 `size_of::<IndexedIndirectArgs>()` 的整倍数(stride 定义在 `mesh/build_mesh_draws/indexed_indirect_args.rs`)。

### 帧时序与集成点

宿主链路(不变):`WgpuRenderFramework::submit_frame_extract`(`graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs`)→ `SceneRendererCore::render_compiled_scene`(`core/scene_renderer_core_render_compiled_scene/render/render.rs`)。该函数内的精确插入/替换序列:

1. **替换**:`build_compiled_scene_draws(...)` → `build_compiled_scene_commands(...)`。内部顺序固定:`build_mesh_batches`(原 `build_mesh_draws`,含 `phase_ordered_meshes` 材质 queue 偏移排序,逻辑不动)→ `assign_execution_owned_indirect_args`(对批次打 VG indirect 补丁,**必须在命令转换前**)→ 逐批次:缓存资格判定 → `CachedMeshDrawCommands::lookup` 命中即取,miss/失效则跑对应 processor 单条重建并 `store`;动态批次直接跑五个 processor → 各 `MeshDrawCommandList::sort_by_key`。
2. **删除**:`partition_mesh_draws(...)` 调用与 `prepare_mesh_queue(...)` 的 early_z 用途;`RenderPassMeshDrawLists { depth_prepass, opaque, alpha_mask, transparent, non_transparent, shadow_casters }` 字面构造替换为 `RenderPassMeshCommandLists::from_buffers(&command_buffers)`(`non_transparent` 字段由 opaque+alpha_mask 列表拼接派生,供 deferred gbuffer/velocity 消费)。
3. **保持**:graph 结构零改动——`mesh.opaque` / `mesh.alpha-mask` / `mesh.transparent`(`builtin_scene_executors.rs` 的 `mesh_executor`)、depth prepass、shadow、velocity 各 executor id 与 RenderGraph 节点/资源 IO 声明不变,满足全局约束 3;改动只发生在 executor 调到的 `RenderPassGpuExecutionContext::record_*` 内部(循环 → replayer)。
4. **缓存生命周期**:`CachedMeshDrawCommands` 挂 `SceneRendererCore` 字段,紧随现有 `self.model_uniform_cache.retain_generation(self.model_uniform_generation)` 同点调用 `retain_generation`,共用 `model_uniform_generation` 计数。
5. **统计出口**:`SceneRendererCompiledSceneOutputs::new(...)` 的 `prepared_mesh_queue.stats()` 参数继续存在,`PreparedMeshQueueStats` 新增 `command_count`、`cached_command_hit_count`、`command_rebuild_count`、`dynamic_command_count`、`state_change_count`、`bind_skip_count` 字段,从命令缓冲与 replayer 汇总。

硬切换删除清单(与上文修改表对应,同一里程碑内迁移调用方并删除):`partition_mesh_draws.rs` 整文件;`render_pass_bindings.rs` 整文件;`BaseScenePass::record_with_attachment_ops` 逐 draw 循环体;`ShadowMapRenderer::record_with_attachment_ops` 的 `is_alpha_mask` 运行时分支;`record_mesh_motion_vectors_to_resource` 的运行时过滤;`prepare_mesh_queue` 的 `early_z_draws` 路径。

### 实施切片细化

**MD-M1 命令类型与 per-pass processor**(切片期仅 `cargo check -p zircon_runtime --lib --locked`)

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M1-S1 类型层 | 新增 `mesh_pass/{mod,mesh_draw_command,mesh_draw_command_list,mesh_pass_processor}.rs`;`mesh/mod.rs`;`mesh_pipeline_cache/*` | 命令/列表/trait/ctx 定稿;`MeshPipelineCache` 增加 variant 注册表与 `resolve_variant`,并已由 pass processor 命令构建调用;cache-backed pipeline 可由 variant id 回查并确保创建 | check 通过;类型签名与本章一致 |
| M1-S2 批次改名 | `mesh/mesh_draw/mesh_draw.rs`→`mesh_batch.rs`、`mesh/mesh_draw/mod.rs`、全部 `MeshDraw` 引用点 | `MeshDraw`→`MeshBatch` 纯改名,行为零变化,编译器驱动清单 | check 通过;diff 内无逻辑改动 |
| M1-S3 processor 落地 | 新增 `processors/*`;`build_compiled_scene_draws.rs`→`build_compiled_scene_commands.rs`;删除 `partition_mesh_draws.rs`;`prepared_queue.rs`;`phase_sort.rs` | 五 processor 实现;`packed_sort_key_u64` 过渡实现已接入命令生成;命令缓冲产出并排序;stats 字段扩展 | check 通过;`MeshPassCommandBuffers` 成为唯一 per-phase 数据源 |
| M1-S4 录制硬切换 | `gpu.rs`、`gpu/mesh_motion_vector.rs`、`base_scene_pass.rs`、`prepass/.../record.rs`、`shadow_map_renderer.rs`;新增 `replay.rs`;删除 `render_pass_bindings.rs` | `RenderPassMeshCommandLists` 接管;四个录制函数改 replayer(本切片 replayer 直通重放、不去重,保证产物等价);executor 内残余过滤分支删除 | check 通过;无任何 `bind_model` 等旧调用残留 |

测试阶段:`cargo test -p zircon_runtime mesh --locked`、`cargo test -p zircon_runtime phase --locked`、`cargo test -p zircon_runtime render_product --locked`。验收:同场景渲染产物不变;每 phase 命令数与旧 draw 数一致(对拍断言见测试清单)。

**MD-M2 静态命令缓存与失效**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M2-S1 契约 | `scene_extract.rs` + runtime 侧 extract 填充点 | `RenderMeshStaticState` 三维度字段与修订号填充 | check 通过;契约层无 wgpu |
| M2-S2 缓存体 | 新增 `cached_mesh_draw_commands.rs`;`scene_renderer_core.rs` + construct | 缓存结构、lookup/store/retain_generation、资格判定函数 | check 通过 |
| M2-S3 接入 | `build_compiled_scene_commands.rs` | 静态批次走 lookup,miss/失效单条 processor 重建;动态批次照旧 | check 通过;静态第二帧 rebuild 计数为 0(本地验证) |

测试阶段:`cargo test -p zircon_runtime mesh --locked`。验收:静态重场景 prepare CPU 时间下降(`PreparedMeshQueueStats` 计数对比记入文档)。

**MD-M3 状态去重重放**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M3-S1 去重 | `replay.rs`、stats 出口 | last_pipeline/last_bind_ids/last_geometry 追踪;pipeline 变化清空追踪;统计三计数 | check 通过 |
| M3-S2 键调优 | `phase_sort.rs`(函数内部)、`opaque_base.rs`/`transparent.rs` | opaque 按 pipeline_variant→material_discriminant→深度前向聚簇;transparent 仅深度后向 | check 通过;透明排序回归不变 |

测试阶段:`cargo test -p zircon_runtime mesh --locked`;RenderDoc(`ZR_RENDERDOC_CAPTURE_NEXT=1`)抓帧确认绑定序列收敛。

**MD-M4 GPUScene 衔接预留**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M4-S1 | `mesh_draw_command.rs`、`replay.rs`、各 processor | object 绑定收敛进 `DrawInstanceSource`;`GpuSceneInstance` 变体只定义不实现路径;uniform 路径行为不变 | check + mesh 范围回归;计划 03 启动无需再改命令结构 |

### 测试与验收清单

单测就近放在各模块 `#[cfg(test)] mod tests`(与 `prepared_queue.rs`、`create_mesh_draw.rs` 现行风格一致);跑法 `cargo test -p zircon_runtime mesh --locked` / `phase` / `render_product` 过滤词。

| 测试函数 | 断言内容 | 位置 |
|---|---|---|
| `render_mesh_draw_command_counts_match_legacy_partitions` | 同一批次集合下,五条命令列表长度 = 旧 `partition_mesh_draws` 各分区长度(M1 对拍,旧逻辑以测试内联重建) | `mesh_pass/mesh_draw_command_list.rs` |
| `render_mesh_draw_processor_depth_prepass_filters_transparent` | transparent 批次不产出 Prepass 命令;opaque/alpha-mask 各一条 | `processors/depth_prepass.rs` |
| `render_mesh_draw_processor_opaque_selects_material_slot_by_fallback_shader` | `uses_fallback_shader` 决定 material 槽取 standard 还是 custom uniform 的 bind id | `processors/opaque_base.rs` |
| `render_mesh_draw_processor_shadow_excludes_non_casters_and_picks_alpha_mask_variant` | `casts_shadow()==false` 无命令;alpha-mask 批次 variant 为 `ShadowDepthAlphaMask` 且带 base_color 绑定 | `processors/shadow.rs` |
| `render_mesh_draw_processor_velocity_requires_previous_transform` | 无 prev transform / 静态 mobility 的批次不产出 velocity 命令 | `processors/velocity.rs` |
| `render_mesh_draw_sort_key_orders_opaque_by_pipeline_then_depth_front_to_back` | 两 pipeline 三深度的命令排序后先按 variant 聚簇、簇内深度升序 | `mesh_pass/mesh_draw_command_list.rs` |
| `render_mesh_draw_sort_key_orders_transparent_back_to_front_ignoring_pipeline` | transparent 排序仅深度降序,variant 不参与 | 同上 |
| `render_mesh_draw_cache_static_second_frame_rebuilds_zero_commands` | 同 extract 连续两帧,第二帧 `command_rebuild_count == 0`、hit 数 = 静态命令数 | `mesh_pass/cached_mesh_draw_commands.rs` |
| `render_mesh_draw_cache_material_revision_invalidates_only_touched_entity` | 改单 entity material_revision 后仅该 entity 各 phase 条目重建 | 同上 |
| `render_mesh_draw_cache_rejects_skinned_and_transparent_entries` | skinned / transparent / indirect 批次资格判定为 false | 同上 |
| `render_mesh_draw_replay_skips_redundant_pipeline_and_bind_groups` | 同 variant 同 bind id 连续命令:`state_change_count == 1`,`bind_skip_count > 0`,draw 数不变 | `mesh_pass/replay.rs` |
| `render_mesh_draw_replay_resets_bind_tracking_on_pipeline_change` | pipeline 切换后即使 bind id 相同也重新 set | 同上 |
| `render_mesh_draw_instance_source_uniform_path_keeps_object_binding` | M4 后 `DrawInstanceSource::ModelUniform` 重放序列与 M3 基线一致 | `mesh_pass/mesh_draw_command.rs` |

`render_product_*` 场景:沿用既有产物对拍集(`cargo test -p zircon_runtime render_product --locked`),M1/M2/M3 每个测试阶段全量回归,断言像素产物不因命令化/缓存/去重改变;重复材质场景额外断言 `state_change_count` 显著小于 `draw_call_count`(阈值写死为 ≤ 命令数的 1/2,场景构造保证)。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | MD-M1 MeshDrawCommand and per-pass processors | 部分完成: 命令面已被 GPUScene/visibility/shadow/TAA 消费,完整 processor 收口仍待补齐 | `MeshDrawCommandList`、phase command buffers、source entity、view visibility filtering、TAA reactive mask processor 与 shadow atlas slot filtering 已在后续计划中使用;但旧 per-pass 判断仍有少量历史路径待清。 | 计划 03/04/05/06 状态表记录 mesh command replay、visibility view filtering、shadow atlas source guard、reactive mask command count 相关测试/检查。 | 回到计划 02 收口 per-pass processor 所有判断源,删除残余 scattered phase logic。 |
| 2026-06-15 | MD-M2 static command cache and invalidation | 部分完成: 静态索引与 GPUScene diff 已接入,静态命令缓存未形成独立 owner | 计划 03 的 GPUScene direct-write diff upload 和计划 04 的 static index 降低了静态场景重复工作;但 cached mesh draw command 仍未独立成跨帧缓存资产。 | 计划 03 GS-M3 与计划 04 VC-M4 状态表记录静态第二帧 0 上传、10,001 静态实例第二帧 `full_rebuild_count == 0` 和 `visibility` sweep 通过。 | 实施 command cache key、失效原因诊断和静态 draw command 跨帧复用。 |
| 2026-06-17 | MD-M2 command cache invalidation diagnostics | 部分完成: 静态命令缓存已具备可观察的 miss/失效原因计数,但静态对象跳过部分 draw 构建仍未下沉 | `CachedMeshDrawCommands::lookup_status` 区分 hit、cold miss 与 static-state mismatch;`MeshDrawCommandCacheStats`/`PreparedMeshQueueStats`/`RenderStats` 新增 command cache miss、transform/geometry/material invalidation、command hit/rebuild/dynamic 与 replay state/bind counters;runtime diagnostics 输出 `render.mesh.queue.command_*` 与 replay 指标。 | `rustfmt --edition 2021` passed for touched Rust files;`git diff --check -- <scoped files>` clean except LF-to-CRLF notices;`cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-md2-cache-diagnostics-0617` passed with existing warnings。Focused tests 按实现优先策略延后。 | 后续继续把静态对象跳过 processor/部分 draw 构造下沉到批次构建阶段,并补静态第二帧/材质变更 focused tests 与产品诊断对拍。 |
| 2026-06-15 | MD-M3 state-deduplicated replay | 部分完成: replay 路径已有 indirect/multi-draw 基础,state bucket 仍未完整定稿 | `MeshDrawCommandReplayer` 已支持 phase-local indirect args 和 `multi_draw_indexed_indirect`;shadow/velocity/base/depth 等 pass 可 replay eligible batches。 | 计划 03 GS-M4 状态表记录 WGPU fixed-count multi-draw replay、`cargo check` 与 source scans;focused lib-test 仍因 shared lib-test 编译超时未完成。 | 补 PSO/material state bucket 排序、set_bind_group 去重与产物对拍。 |
| 2026-06-17 | MD-M3 command sort input/state bucket depth semantics | 部分完成: command sort key 不再使用 draw ordinal 占位,真实 phase queue depth/render_queue/material_queue 已进入 MeshDrawCommand | 新增 `MeshCommandSortInput`;`PendingMeshDraw`/`MeshDraw` 携带 phase queue sort input;`MeshBatchRef::command` 重新用 `packed_sort_key_u64` 生成 opaque state bucket 与 transparent depth sort;新增源码测试覆盖 opaque state bucket before depth、transparent depth before bucket。 | `rustfmt --edition 2021` 已执行;`git diff --check -- <scoped files>` clean except LF-to-CRLF notices;`cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-md3-sort-input-0617` passed with existing warnings。Focused tests 按用户要求延后。 | 后续补 replay 产物对拍、RenderDoc 绑定序列确认、静态命令缓存更深下沉/失效诊断。 |
| 2026-06-15 | MD-M4 GPUScene handoff | 已完成(基础 handoff),更高阶 GPU count 提交待计划 19 | Mesh draw ABI 已从 model uniform 路径硬切到 GPUScene instance index;command-local skinned palette bind group 与 visible-instance remap 已被计划 03/04 使用。 | 计划 03 GS-M2/GS-M4 与计划 04 VC-M3 状态表记录 group3 GPUScene ABI、indirect args buffer、compact replay 和 visible remap。 | 计划 19 再升级 `multi_draw_indirect_count` 与 GPU-decided draw count。 |

### 参考实现精读笔记

`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshPassProcessor.h`:
- `FMeshDrawCommand`(字段:`ShaderBindings`、`VertexStreams`、`IndexBuffer`、`CachedPipelineId`、`FirstIndex`/`NumPrimitives`/`NumInstances`、`VertexParams`/`IndirectArgs` union、`PrimitiveIdStreamIndex`、`StencilRef`)——解决"不可变命令最小集"问题。Zircon 对应 `MeshDrawCommand`;取舍:wgpu 无 PSO id 与 stencil ref 旁路,以 `MeshPipelineVariantId` + `MeshBindHandle.id` 替代;union 以 `MeshDrawArgs` 枚举表达。
- `FMeshDrawCommand::MatchesForDynamicInstancing` / `GetDynamicInstancingHash`——state bucket 等价判定与哈希。Zircon V1 不做跨 entity state bucket(`DrawInstanceSource` 留给计划 03 的 instancing),只取其"以 pipeline+绑定+几何为同态键"的思路供 `MeshDrawBatchKey` 统计沿用。
- `FMeshDrawCommandSortKey`(BasePass:`VertexShaderHash:16 | PixelShaderHash:32 | Background:1 | Masked:15`;Translucent:`MeshIdInPrimitive:16 | Distance:32 | Priority:16`)——证明 opaque 按 shader 聚簇、translucent 按距离主导是位段级决策。Zircon 对应 `packed_sort_key_u64` 的输入参数设计;位段摆放让渡给计划 09。
- `FMeshPassProcessor::AddMeshBatch`(纯虚)与 `BuildMeshDrawCommands` 模板(shader 选择→绑定收集→排序键→提交 DrawListContext)、`FMeshPassProcessorRenderState`(BlendState/DepthStencilState/StencilRef 的 per-pass 覆写)——`MeshPassProcessor::add_mesh_batch` 直接对应;Zircon 把 render state 折叠进 `MeshPassPipelineKind` × `PipelineKey`(wgpu PSO 含全部状态),不单独建 RenderState 类型。
- `FCachedMeshDrawCommandInfo`(`SortKey`/`CommandIndex`/`StateBucketId`/`MeshPass`/`MeshFillMode`/`MeshCullMode`)与 `FCachedPassMeshDrawList`(`TSparseArray<FMeshDrawCommand>` + `LowestFreeIndexSearchStart`)、`FStateBucketMap`——UE 把缓存索引与命令本体分离以保 InitViews cache 友好。Zircon 场景规模小一个量级,取舍:单 `HashMap<CachedMeshDrawKey, _>` 直存命令,不做 sparse 索引与 state bucket 去重存储,失效粒度按 entity×phase。
- `FMeshDrawCommandStateCache`(`PipelineId`/`StencilRef`/`ShaderBindings[]`/`VertexStreams[]`;`SetPipelineState` 重置全部追踪)——replayer 的直接蓝本;`MeshDrawCommandReplayer.last_*` 字段与"换 PSO 必清绑定追踪"规则照搬。

`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshPassProcessor.cpp`:
- `FMeshDrawCommand::SubmitDrawBegin`(`CachedPipelineId.GetId() != StateCache.PipelineId` 才 set PSO;vertex stream 逐槽与 `StateCache.VertexStreams` 比对跳过;`ShaderBindings.SetOnCommandList` 带 binding state 过滤)与 `SubmitDrawEnd`(`DrawIndexedPrimitive` vs `DrawIndexedPrimitiveIndirect` 按 `NumPrimitives>0` 与 override args 分流)——replay 的 set 与 draw 分离、间接/直接分流由 `MeshDrawArgs` 枚举承接。
- `SubmitMeshDrawCommandsRange`(局部 `FMeshDrawCommandStateCache` 生命周期 = 一次提交范围)——Zircon replayer 同样以单 render pass 为追踪作用域,跨 pass 不保留状态。
- `FCachedPassMeshDrawListContext::FinalizeCommandCommon` / `GetCommandInfoAndReset`——命令构建完成点收口 sort key 与缓存信息;Zircon 在 `add_mesh_batch` 末尾一次性构造不可变命令,无二段 finalize。
- `ApplyViewOverridesToMeshDrawCommands`(`bReverseCulling`/`bRenderSceneTwoSided` 时整批重建命令)——提示 per-view 覆写与缓存冲突的逃生通道;Zircon 现无对应 view 覆写,记录为未来风险,不实现。

`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.h`:
- `FParallelMeshDrawCommandPass`(`DispatchPassSetup` → `BuildRenderingCommands` → `Draw`)与 `FMeshDrawCommandPassSetupTaskContext`——UE 的并行翻译任务编排。取舍:Zircon V1 单线程构建(批次量级远小),保留 `MeshPassCommandBuffers` 为未来任务化的切分边界,不引入任务系统。
- `FPrimitiveIdVertexBufferPool`——GPUScene primitive id 流的池化,归计划 03 消费,本计划仅在 `DrawInstanceSource::GpuSceneInstance` 留接口。

`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp`:
- `GenerateDynamicMeshDrawCommands`(`FDynamicPassMeshDrawListContext` + 对 `DynamicMeshElements` 与 `DynamicMeshCommandBuildRequests` 两类输入循环调 `AddMeshBatch`)——"动态对象每帧走 processor、静态走缓存"的双输入合流即 Zircon `build_compiled_scene_commands` 的批次分流蓝本。
- `UpdateTranslucentMeshSortKeys`——透明命令生成后按视距重写 sort key;对应 Zircon transparent 不缓存、每帧以相机深度重算 sort_key 的决策依据。

## 风险与回退

- 缓存失效漏判导致画面滞留旧状态:失效维度(transform/material/geometry)各自有单测;调试时可用环境开关强制全动态路径定位(开关仅诊断用,不作为长期双路径保留)。
- skinned/morph 对象进缓存会放大复杂度:V1 明确排除,始终走 dynamic 列表;计划 08 的 GPU 形变落地后再评估。
- transparent 排序与状态聚簇冲突:transparent phase 不做 PSO 聚簇,保持深度优先,接受其状态切换成本。
