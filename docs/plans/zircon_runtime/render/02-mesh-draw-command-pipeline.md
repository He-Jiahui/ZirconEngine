---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/lazy_rebuild_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/visibility_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams/types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs
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
| `mesh/mesh_pass/mesh_draw_command_list.rs` | `MeshDrawCommandList`(排序/追加/统计)、`MeshPassCommandBuffers`(七条 per-phase 列表容器)与 indirect batch 统计投影 |
| `mesh/mesh_pass/mesh_draw_command_list/builder.rs` | 批次到 command buffer 的 processor fan-out、静态命令缓存 lookup/rebuild、dynamic command 合并与 cache stats 汇总 |
| `mesh/mesh_pass/mesh_draw_command_list/tests.rs` | 命令列表排序/统计、batch→command buffer、indirect batch stats、variant 分配与静态缓存命中/失效行为测试 |
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
| 2026-06-24 | MD-M2/MD-M4 VG debug snapshot stream types owner split | render_plan02_vg_debug_snapshot_stream_types_owner_split_static_passed_cargo_deferred_active_compile_lane | VG debug snapshot stream types owner split 已把 `core/framework/render/virtual_geometry_debug_snapshot_streams.rs` 中的 readback/decoded stream DTO、decode error 与 summary 类型移入 `core/framework/render/virtual_geometry_debug_snapshot_streams/types.rs`；父 owner 从 850 行降到 703 行，只保留 decode/encode orchestration、packing helpers 和 diagnostics/metrics/types 子模块挂载。新增 `runtime_15_vg_debug_snapshot_stream_types_are_child_owner` 锁定 moved types 不回流、父/子 800 行预算和 docs/status 锚点。 | scoped rustfmt、static owner scan、line-count scan、docs-anchor scan、touched-file whitespace scan 和 scoped `git diff --check` 已通过；line-count 当前为 root 703、types 166、diagnostics 116、metrics 258。Cargo/WGPU/RenderDoc 因 active compile lane 暂缓，不计通过。 | Cargo lane 空闲后补跑 Plan 02 VG/debug snapshot stream focused guard 与产品组，并继续补 mesh-level VG indirect/execution buffer WGPU 与 RenderDoc evidence。 |
| 2026-06-24 | MD-M2/MD-M4 VG debug snapshot owner split | render_plan02_vg_debug_snapshot_owner_split_static_passed_cargo_deferred_active_compile_lane | `graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs` 从 955 行 owner 收敛为 169 行 orchestration root；新增 `graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/page.rs` 承接 page residency、request inspection 与 cull-input projection，`graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/node_cull.rs` 承接 node/cluster cull global state、dispatch setup、launch worklist、traversal replay 与 page request ids，`graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs` 承接 draw segment execution states、selected cluster expansion、visbuffer marks/64 entries 与 hardware rasterization records，`support.rs` 承接共享 saturation helper。结构守卫 `runtime_15_render_vg_debug_snapshot_is_child_owner_split` 锁定 moved owner 不回流、父子 800 行预算和 docs/status 锚点。 | scoped rustfmt、static owner scan、line-count scan、docs-anchor scan、touched-file whitespace scan 和 scoped `git diff --check` 已通过；line-count 当前为 root 169、page 138、node_cull 374、execution 361、support 3。Cargo/WGPU/RenderDoc 因 active compile lane 暂缓，不计通过。 | Cargo lane 空闲后补跑 Plan 02 VG/debug snapshot 相关 focused guard 与产品组，并继续补 mesh-level VG indirect/execution buffer WGPU 与 RenderDoc evidence。 |
| 2026-06-24 | MD-M2/MD-M4 mesh dead-code suppression cleanup | render_plan02_mesh_dead_code_suppression_cleanup_static_passed_cargo_deferred_active_editor_lane | 清理上一 VG stats 切片后剩余的 mesh 生产 dead-code suppression：删除 `MeshDraw.skinned_joint_count` 缓存和 `MeshDraw::skinned_joint_count()`，palette WGPU buffer 字段继续通过 prepared queue palette upload stats 保持 live；删除 `VirtualGeometryIndirectDrawRef.mesh_index_count` / `mesh_signature` scratch 字段和 `mesh_signature_for_pending_draw(...)`，VG GPU draw-ref input 的 `mesh_index_count` 仍由 pending draw index count 在 metadata buffer 构建时生成；同时移除 `mesh_pass/mod.rs` 的模块级 `allow(dead_code, unused_imports)` 与 `PreparedMeshQueueStats` 的 non-test `allow(dead_code)`。 | scoped rustfmt 通过；mesh 生产目录 `allow(dead_code)` 扫描零命中；focused stale scratch 扫描确认 `skinned_joint_count()`、`mesh_signature`、`mesh_signature_for_pending_draw(...)` 无残留，剩余 `mesh_index_count` 只存在于 GPU draw-ref input / raster path；`git diff --check` scoped 通过(仅 LF->CRLF 提示)。Cargo/WGPU/RenderDoc 因无关 active cargo/rustc lane 暂缓，不计通过。 | Cargo lane 空闲后补跑 Plan 02 prepared queue/VG/static-cache 产品组，并继续补 mesh-level VG indirect/execution buffer WGPU 与 RenderDoc evidence。 |
| 2026-06-24 | MD-M2/MD-M4 MeshDraw virtual geometry execution projection stats | render_plan02_virtual_geometry_execution_projection_stats_static_passed_cargo_timeout_no_result | `CompiledSceneDraws::virtual_geometry_execution_stats()` 现在从 `MeshDraw::virtual_geometry_execution_draw(...)` 生成 `PreparedMeshVirtualGeometryExecutionStats`；`PreparedMeshQueueStats` 承载 execution draw/segment/page/resident/pending/missing/repeated 计数，`render_compiled_scene(...)` 把这些 compiled-scene evidence 接入最终 prepared stats，`update_virtual_geometry_stats(...)` 用 mesh execution evidence 与既有 context/runtime 统计取 max 后发布到 public `RenderStats`。本切片同时移除 `mesh_draw/virtual_geometry_execution_projection.rs`、`VirtualGeometrySubmissionDetail` getter、`MeshDraw` VG submission 字段和 live pending input builder 上的 `#[allow(dead_code)]`；新增 focused stats-bridge guard 覆盖唯一/重复 VG execution segment，并确认普通 indirect draw 不进入 VG execution 口径。 | scoped rustfmt 通过；source-anchor 扫描确认 compiled-scene 已消费 MeshDraw VG execution projection，VG projection/detail 文件不再靠 `allow(dead_code)` 存活；mesh dead-code 扫描仅剩 draw-ref/skinned 既有残项；`git diff --check` 通过(仅 LF->CRLF 提示)。focused `cargo test -p zircon_runtime --lib prepared_queue_stats_carry_virtual_geometry_execution_counts --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-vg-execution-stats-0624 ...` 300s 超时无输出，额外等待 180s 后 cargo/rustc 残留仍未结束并已停止，不计 Cargo/WGPU/RenderDoc 通过。 | Cargo lane 空闲后补跑 prepared queue stats bridge focused guard、compiled-scene/VG product stats guard，并用 RenderDoc 对拍 mesh-level VG indirect/execution buffers；F12 后续继续处理 draw-ref `mesh_index_count`/`mesh_signature` 与 skinned palette/joint count suppressions。 |
| 2026-06-24 | MD-M2/MD-M4 virtual geometry compiled-scene indirect evidence | render_plan02_virtual_geometry_compiled_indirect_evidence_static_passed_cargo_deferred_active_lanes | `CompiledSceneDraws` 的 VG indirect segment/args/buffer getters 不再靠 `#[allow(dead_code)]` 存活，新增 `virtual_geometry_indirect_stats()` 把 args、segment 与五类 WGPU buffer 存在性投影成 `PreparedMeshVirtualGeometryIndirectStats`；`PreparedMeshQueueStats` 与 `prepared_queue/stats_bridge.rs` 承载该 evidence，`render_compiled_scene(...)` 在 GPUScene/replay stats 后接入，`update_virtual_geometry_stats(...)` 优先使用 compiled-scene evidence，旧 submission/execution segment evidence 继续作为兜底。新增 focused unit guard 覆盖 compiled-scene VG indirect count projection 与 prepared queue forwarding。 | scoped rustfmt 通过；`git diff --check` 通过(仅 LF->CRLF 提示)；静态扫描确认 `build_compiled_scene_draws.rs` 已无 `allow(dead_code)` 且 VG indirect buffer getters 由 `virtual_geometry_indirect_stats()` 消费。验证决策时仍有 cargo/rustc lane 活跃，focused Cargo/WGPU/RenderDoc 未启动，不计通过。 | Cargo lane 空闲后补跑 `render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache` 与 static-cache 产品组，并用 RenderDoc resource/marker 对拍五类 VG mesh-level indirect buffers；若产品路径显示 no-RT flagship 统计语义变化，更新对应产品断言与文档。 |
| 2026-06-23 | MD-M2/MD-M4 virtual geometry mesh-level indirect buffers | render_plan02_virtual_geometry_mesh_indirect_buffers_static_passed_cargo_deferred_active_lanes | 新增 `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_indirect.rs` 作为 351 行 child owner；`PendingMeshDraw`/`PendingMeshGeometry` 可复制，`build_mesh_draws(...)` 在 GPUScene sync 前用 `ViewportRenderFrame.virtual_geometry_debug_snapshot.execution_segments` 扩展 VG carrier draw，并为 executable segments 创建 WGPU indexed indirect args、submission、authority、draw-ref、segment buffers；per-draw `MeshDraw` 只在有 VG segment 时携带 indirect args buffer/offset/detail，非 VG draw 保持 direct path。`record_submission`/`record_present_submission` 不再把 VG indirect segment 统计写死为 0，`update_virtual_geometry_stats` 以提交记录中的 executable segment evidence 报告 indirect buffer count。产品守卫 `render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache` 增加 buffer/args/segment 非零断言，锁定 authored VG residual path 已从产品统计看到 mesh-level WGPU indirect 数据面。 | `rustfmt --edition 2021` 与 `rustfmt --edition 2021 --check` 覆盖本切片 Rust 文件；scoped `git diff --check` 通过(仅 LF->CRLF 提示)；行数检查：`build.rs` 968 行、`virtual_geometry_indirect.rs` 351 行、`render_product_mesh_cache.rs` 835 行。验证决策时仍有 cargo/rustc lane 活跃，focused Cargo/WGPU/RenderDoc 未启动，不计通过。 | Cargo lane 空闲后补跑 `render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache` 与 static-cache 产品组；随后把 `CompiledSceneDraws` 的 indirect buffer getter 接入更强的 render-path/debug snapshot evidence，并做 RenderDoc resource/marker 对拍。 |
| 2026-06-23 | MD-M2 static command cache virtual geometry residual product guard | render_plan02_static_cache_virtual_geometry_residual_product_guard_static_passed_cargo_deferred_active_lanes | `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs` 扩展为 766 行产品提交路径 static mesh cache owner，新增 `render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache`；测试使用带 advanced providers 的 WGPU 产品框架，开启 `virtual_geometry` quality profile，提交同一实体的 Dynamic 可见性承载 mesh + authored `RenderVirtualGeometryExtract` 两帧，断言 `last_virtual_geometry_payload_source == Authored` 与 `last_virtual_geometry_indirect_draw_count >= 1`，同时 pending static command-cache candidate、pre-MeshDraw skipped draw/phase、cache hit/miss/rebuild 保持 0，锁定 authored VG 产品路径不被 static mesh command cache 吸收。 | scoped rustfmt/source-anchor/line-count 已执行；focused Cargo 未启动，因为本切片验证决策时其他 cargo/rustc lane 正在占用编译通道；不计 Cargo/WGPU/RenderDoc 通过。 | Cargo lane 空闲后补跑 `render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse`、`render_product_static_mesh_material_revision_invalidates_pre_mesh_cache`、`render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path`、`render_product_static_transparent_mesh_stays_out_of_pre_mesh_cache`、`render_product_static_skinned_mesh_stays_out_of_pre_mesh_cache`、`render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache`；mesh-level indirect draw buffer 接入仍未关闭，继续推进 indirect residual path 与 RenderDoc evidence。 |
| 2026-06-23 | MD-M2 static command cache skinned residual product guard | render_plan02_static_cache_skinned_residual_product_guard_static_passed_cargo_deferred_active_lanes | `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs` 扩展为 622 行产品提交路径 static mesh cache owner，新增 `render_product_static_skinned_mesh_stays_out_of_pre_mesh_cache`；测试注册最小 static skinned mesh、root skeleton 和 pose，将 direct `RenderMeshSnapshot.mesh` 与 `RenderSkeletalPoseExtract` 送入 `WgpuRenderFramework::submit_frame_extract(...)` 两帧，断言 skinned draw 与 skinned GPU-source candidate 存在，但 pending static command-cache candidate、pre-MeshDraw skipped draw/phase、cache hit/miss/rebuild 保持 0，锁定 skinned/GPU-source 路径继续走 residual/dynamic command path。 | scoped rustfmt/source-anchor/line-count/diff-check 已执行；focused Cargo 未启动，因为本切片验证决策时其他 cargo/rustc lane 正在占用编译通道；不计 Cargo/WGPU/RenderDoc 通过。 | Cargo lane 空闲后补跑 `render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse`、`render_product_static_mesh_material_revision_invalidates_pre_mesh_cache`、`render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path`、`render_product_static_transparent_mesh_stays_out_of_pre_mesh_cache`、`render_product_static_skinned_mesh_stays_out_of_pre_mesh_cache`；继续推进 indirect/VG residual path 产品守卫和 RenderDoc evidence。 |
| 2026-06-23 | MD-M2 static command cache transparent residual product guard | render_plan02_static_cache_transparent_residual_product_guard_static_passed_cargo_deferred_active_lanes | `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs` 扩展为 396 行产品提交路径 static mesh cache owner，新增 `render_product_static_transparent_mesh_stays_out_of_pre_mesh_cache`；测试注册 `AlphaMode::Blend` 材质并用 `GeometryExtract::from_meshes_and_phase_inputs(...)` 把同一 static mesh 放入 `Transparent3d` phase，两帧产品提交均断言 pending static command-cache candidate、pre-MeshDraw skipped draw/phase、cache hit/miss/rebuild 为 0，同时透明 draw 和 dynamic command 仍存在，锁定 transparent 按相机深度排序的每帧命令路径不会被 static cache 吸收。 | scoped rustfmt/source-anchor/line-count 已执行；focused Cargo 未启动，因为本切片验证决策时已发现其他 cargo/rustc lane 正在占用编译通道；不计 Cargo/WGPU/RenderDoc 通过。 | Cargo lane 空闲后补跑 `render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse`、`render_product_static_mesh_material_revision_invalidates_pre_mesh_cache`、`render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path`、`render_product_static_transparent_mesh_stays_out_of_pre_mesh_cache`；继续推进 skinned/indirect residual path 产品守卫和 RenderDoc evidence。 |
| 2026-06-23 | MD-M2 static command cache TAA reactive residual product guard | render_plan02_static_cache_taa_reactive_residual_product_guard_static_passed_cargo_deferred_active_lanes | `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs` 扩展为 297 行产品提交路径 static mesh cache owner，新增 `render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path`；测试注册同一 static mesh 的 `taa_reactive_mask_strength = 1.0` 材质并开启 TAA/temporal history，两帧产品提交均断言 pre-MeshDraw skipped draw/phase 为 0，第二帧仍能在后续 command 层复用 ordinary cached static phases，同时 reactive-mask command 保持 per-frame dynamic command。 | scoped rustfmt/source-anchor/line-count 已执行；focused Cargo 未启动，因为本切片验证决策时已发现其他 cargo/rustc lane 正在占用编译通道；不计 Cargo/WGPU/RenderDoc 通过。 | Cargo lane 空闲后补跑 `render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse`、`render_product_static_mesh_material_revision_invalidates_pre_mesh_cache`、`render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path`；继续推进 skinned/indirect/transparent residual path 产品守卫和 RenderDoc evidence。 |
| 2026-06-23 | MD-M2 static command cache material revision product guard | render_plan02_static_cache_material_revision_product_guard_static_passed_cargo_deferred_active_lanes | `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs` 继续作为 206 行产品提交路径 static mesh cache owner，新增 `render_product_static_mesh_material_revision_invalidates_pre_mesh_cache`；测试用同一 material id/URI 通过 `ResourceRecord::with_source_hash(...)` 在两次产品提交之间推进 resource-manager revision，并断言第二帧不会走 pre-MeshDraw skip/cached hit，material-bound residual reason、material invalidation 与 command rebuild 计数可观测，transform/geometry invalidation 与 cache hit/miss 仍为 0。 | scoped rustfmt --check/source-anchor/line-count 已执行；focused Cargo 未启动，因为本切片验证决策时已发现其他 cargo/rustc lane 正在占用编译通道；不计 Cargo/WGPU/RenderDoc 通过。 | Cargo lane 空闲后补跑 `render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse` 与 `render_product_static_mesh_material_revision_invalidates_pre_mesh_cache`；继续推进 TAA reactive/skinned/indirect/transparent residual path 产品守卫和 RenderDoc evidence。 |
| 2026-06-23 | MD-M2 static command cache product stats guard | render_plan02_static_cache_product_stats_guard_static_passed_cargo_timeout_no_result | 新增 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs`(115 行)作为产品提交路径的静态 Mesh 二帧缓存复用守卫，`graphics/tests/mod.rs` 挂载新模块；`render_product_submit.rs` 只公开 mesh-cache snapshot/material fixture。守卫连续提交同一 eligible static mesh 两帧：首帧确认 `pending_static_command_cache_*` 候选存在且 cold hit 为 0，第二帧确认 `pre_mesh_draw_static_command_cache.skipped_*`、`cached_command_hit_count` 覆盖 skipped phases，且 miss/rebuild/residual 计数为 0。 | scoped rustfmt/source-anchor/line-count 已执行；focused locked `cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-static-cache-product-0623 render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse` 在约 184s 工具窗口超时无结果，匹配该 target 的 cargo/rustc 残留已停止；不计 Cargo/WGPU/RenderDoc 通过。 | 本切片关闭产品提交统计层的二帧 reuse source guard；仍需在 Cargo lane 可用时补跑 focused WGPU product result，并继续补静态材质变更产品对拍、TAA reactive/skinned/indirect/transparent 路径和 RenderDoc evidence。 |
| 2026-06-23 | MD-M2 pre-MeshDraw material-bound rebuild boundary guard | render_plan02_pre_mesh_draw_material_boundary_guard_static_passed_cargo_lock_blocked | 新增 `zircon_runtime/src/tests/runtime_absorption/structure_convention/render_pending_command_cache_material_boundary.rs`，锁定 `pending_command_cache_extract/non_material_rebuild.rs` 只能 pre-MeshDraw 重建 opaque `ShadowDepth`，并把 normal prepass、alpha-mask shadow、object velocity、TAA reactive mask replay 仍调用 `bind_standard_material_if_needed(...)` 的 WGPU material-bound 事实纳入结构守卫。`structure_convention.rs` 新挂载 `render_pending_command_cache_material_boundary`，避免后续在未改变 material bind group 生成时机前误把这些 phase 加进 `can_rebuild_non_material_command_phase(...)`。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 按本切片执行；focused locked `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-material-boundary-0623 runtime_15_pending_command_cache_material_bound_phases_stay_out_of_pre_mesh_rebuild` 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片不扩大重建面，只把 unsafe material-bound pass 边界固定为源码守卫；若后续要下沉 depth/alpha-mask/velocity/TAA reactive，必须先改变 WGPU material bind group 构造/携带策略并补产品对拍。 |
| 2026-06-23 | MD-M2 pre-MeshDraw second-frame extraction guards | render_plan02_pre_mesh_draw_second_frame_extract_guards_static_passed_cargo_timeout_no_result | 新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/second_frame_tests.rs`(188 行)作为 pre-MeshDraw 二帧/失效 focused owner，覆盖 full-hit 第二帧 `cached_command_hit_count == 3`、`command_rebuild_count == 0` 且不请求 rebuild batch，以及 shadow-only 材质 revision 改变时 opaque `ShadowDepth` 可在 `create_mesh_draw(...)` 前重建并记录 `cache_invalidated_material_count == 1`。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 同步锁定 `mod second_frame_tests;`、新 child owner、220 行预算和 docs/status 锚点。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 按本切片执行；focused locked `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-second-frame-extract-0623 pending_command_cache_extract` 在 184s 工具窗口超时无结果，且无本 target 残留 cargo/rustc；不计 Cargo/WGPU/RenderDoc 通过。 | 本切片补齐 pre-MeshDraw 抽取层的二帧 reuse 与 safe non-material material-invalidation source guard；静态缓存产品二帧/材质变更 WGPU 对拍、TAA reactive/skinned/indirect/transparent 路径和 RenderDoc evidence 仍未关闭。 |
| 2026-06-23 | Plan 02 prepared queue stats bridge tests owner split | render_plan02_prepared_queue_stats_bridge_tests_owner_split_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs` 从 769 行近阈值 queue behavior + bridge forwarding 混合 owner 降到 599 行，只保留 early-z/shadow phase、batch candidate、velocity/skinning/LOD 和 GPU-skinned batch eligibility 行为测试；新增 `graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs`(174 行)承接 `with_pending_command_cache_*`、mesh pass command buffer/replay 与 GPUScene stats forwarding 测试。`runtime_15_prepared_mesh_queue_is_folder_backed` 同步锁定 `mod stats_bridge_tests;`、moved test owner、tests 620 行/stats_bridge_tests 220 行预算和 docs/status 锚点。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 按本切片执行；focused locked `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-prepared-queue-stats-bridge-tests-0623 runtime_15_prepared_mesh_queue_is_folder_backed` 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片不改变渲染行为，只关闭 prepared queue test owner 继续膨胀风险；MD-M2 静态缓存产品二帧/材质变更对拍、TAA reactive/skinned/indirect/transparent 路径和 RenderDoc evidence 仍未关闭。 |
| 2026-06-23 | MD-M2 residual fallback owner split | render_plan02_residual_fallback_owner_split_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs` 从 331 行降到 294 行，只保留抽取入口、cache lookup/store、prebuilt command 汇总和 pending batch materialization；新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs`(58 行)承接 `PendingMeshCommandCacheResidualReason`、non-material rebuild 失败到 residual 统计的归因和 `record_residual_reason(...)`。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 同步锁定 `mod residual_fallback;`、child owner、root/child 行数预算和 docs/status 锚点。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 按本切片执行；focused locked `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-residual-owner-split-0623 pending_command_cache_extract` 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片不改变抽取行为，只关闭 residual reason accounting 回流 root 的结构风险；material-bound phase miss、TAA reactive、skinned/GPU-skinning、indirect/VG、transparent、静态第二帧/材质变更产品对拍和 RenderDoc 绑定序列 evidence 仍未关闭。 |
| 2026-06-23 | MD-M2 pre-MeshDraw residual fallback diagnostics | render_plan02_pre_mesh_draw_residual_fallback_diagnostics_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs` 新增带统计的 `commands_for_extract_item_with_stats(...)` 与 `PendingMeshCommandCacheResidualReason`，在 eligible static pending draw 无法 pre-MeshDraw 抽取时区分三类 residual 原因：material-bound phase miss/invalidated、shadow rebuild input 缺失、non-material rebuild 被策略拒绝。`PendingMeshCommandCacheExtractionStats`、`PreparedMeshQueueStats`、`RenderStats`、`update_stats/base_stats.rs` 与 `product/mesh_queue.rs` 输出 `render.mesh.queue.pre_mesh_draw_static_command_cache.residual_*_draw_count`。新增 `pending_command_cache_extract/fallback_tests.rs`(148 行)覆盖三类 fallback。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 通过；focused locked `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-residual-fallback-0623 pending_command_cache_extract` 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片只把 residual path 原因可观测化，不改变 material-bound phase 仍走 `MeshDraw` 构造的事实；后续仍需真正扩大 safe rebuild 面、静态第二帧/材质变更产品对拍和 RenderDoc 绑定序列 evidence。 |
| 2026-06-23 | Plan 02 prepared queue stats bridge owner split | render_plan02_prepared_queue_stats_bridge_owner_split_static_passed_cargo_timeout_no_result | `graphics/scene/scene_renderer/mesh/prepared_queue.rs` 从 330 行近阈值 stats owner 收敛为 241 行队列汇总/字段 owner；新增 `graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs`(93 行)承接 `with_pending_command_cache_*`、mesh pass command buffer/replay 与 GPUScene stats forwarding。`runtime_15_prepared_mesh_queue_is_folder_backed` 与 `runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 同步锁定 `mod stats_bridge;`、新 child owner、docs/status 锚点和父子行数预算。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 通过；focused locked `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-prepared-queue-stats-bridge-0623 runtime_15_prepared_mesh_queue_is_folder_backed` 180 秒超时无结果，且无本 target 残留 cargo/rustc；不计 Cargo/WGPU/RenderDoc 通过。 | 本切片不改变渲染行为，只为后续 MD-M2 cache diagnostics/skip path 继续扩展预留结构预算；material-bound phase miss、TAA reactive、skinned/GPU-skinning、indirect/VG、transparent 与产品二帧 evidence 仍未关闭。 |
| 2026-06-23 | MD-M2 visibility-pruned pre-MeshDraw diagnostics split | render_plan02_visibility_pruned_pre_mesh_draw_diagnostics_static_passed_cargo_timeout_no_result | `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs` 的抽取结果新增 `visibility_pruned` 标记，`PendingMeshCommandCacheExtractionStats` 新增 `visibility_pruned_mesh_draw_count`，并沿 `PreparedMeshQueueStats`、`RenderStats`、`update_stats/base_stats.rs` 和 `core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs` 输出 `render.mesh.queue.pre_mesh_draw_static_command_cache.visibility_pruned_draw_count`。新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/visibility_tests.rs` 作为 62 行 visibility focused owner；原综合 `tests.rs` 降到 220 行。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 通过；focused locked lib-test compile 180 秒超时无结果，且无本 target 残留 cargo/rustc；不计 Cargo/WGPU/RenderDoc 通过。 | 本切片只把零命令 visibility/relevance 裁剪从普通 skipped draw 中拆出产品诊断口径；material-bound phase miss、TAA reactive、skinned/GPU-skinning、indirect/VG、transparent 和静态第二帧/材质变更产品对拍仍未关闭。 |
| 2026-06-23 | MD-M2 pending command cache extract-item owner split | render_plan02_pending_command_cache_extract_item_owner_split_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs` 从 355 行降到 255 行，只保留抽取入口、cache lookup/store、non-material rebuild dispatch 与 pending batch materialization。新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs` 作为 111 行 extract item/eligibility/phase selection owner，承接 `PendingMeshCommandCacheExtractItem`、pending draw queue profile 投影、`can_skip_pending_mesh_draw_for_cached_commands(...)` 与 `cacheable_phases_for_extract_item(...)`。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 通过；focused locked lib-test compile 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片不改变抽取行为，只为后续扩大 MD-M2 skip/rebuild 面预留 owner 空间；material-bound phase miss、TAA reactive、skinned/GPU-skinning、indirect/VG、transparent 与产品证据仍未关闭。 |
| 2026-06-23 | MD-M2 visibility-pruned pre-MeshDraw empty extraction | render_plan02_visibility_pruned_pre_mesh_draw_empty_extract_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs` 现在把“可缓存但当前 visibility/relevance 裁掉所有 cacheable phases”的 static pending draw 视为成功抽取零条命令，而不是返回 residual path。这样 GPUScene 已同步但没有 main/shadow/prepass/base 输出的直接 prepared static draw 会在 `create_mesh_draw(...)` 前被移除，不创建 material bind groups，也不请求 non-material rebuild input。后续 diagnostics split 已将 focused guard 移入 `pending_command_cache_extract/visibility_tests.rs::pending_command_cache_extract_marks_visibility_pruned_static_draw`，锁定空命令抽取、cache stats default、`visibility_pruned` 标记和 rebuild batch 未请求。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 通过；focused locked lib-test compile 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片只覆盖 visibility 全裁剪的零命令安全路径；仍不重建 depth prepass、alpha-mask shadow、opaque/alpha material phase miss、TAA reactive、skinned/GPU-skinning、indirect/VG、transparent；后续仍需静态第二帧/材质变更 focused Cargo 与产品诊断对拍。 |
| 2026-06-23 | MD-M2 lazy pre-MeshDraw rebuild input | render_plan02_lazy_pre_mesh_draw_rebuild_input_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs` 不再为每个 candidate 预先查询 GPUScene span 并构造 `MeshBatchRef`；`commands_for_extract_item(...)` 现在只在 `non_material_rebuild::can_rebuild_non_material_command_phase(phase)` 允许且该 phase miss/invalidated 时，通过 `pending_mesh_command_cache_rebuild_batch_for_phase(...)` 惰性请求重建批次。新增 `pending_command_cache_extract/lazy_rebuild_tests.rs`，覆盖 full-hit 静态 draw 不请求 rebuild batch、material-bound phase miss 不物化 rebuild batch 两个边界，避免全命中第二帧仍支付多余 pending batch 构造成本。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 通过；focused locked `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-lazy-rebuild-0623 pending_command_cache_extract` 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片只去掉 full-hit 与 material-bound miss 路径的提前 rebuild input 物化；depth prepass、alpha-mask shadow、opaque/alpha material phase miss、TAA reactive、skinned/GPU-skinning、indirect/VG、transparent 仍走 residual `MeshDraw` 构造；后续仍需静态第二帧/材质变更 focused Cargo 与产品诊断对拍。 |
| 2026-06-23 | MD-M2 pre-MeshDraw opaque shadow cache rebuild | render_plan02_pre_mesh_draw_shadow_cache_rebuild_static_passed_cargo_timeout_no_result | 在 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs` 的全命中抽取路径上继续下沉 `RenderPhase::Shadow` 的 opaque static caster：新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs` 作为非材质重建子 owner，`build_mesh_draws` 将 GPUScene instance span 交给 extraction owner；当直接 prepared、非透明、非 skinned、无 reactive mask 的 static pending draw 只剩 opaque shadow miss/invalidated，或其它 material-bound phase 已命中时，可在 `create_mesh_draw(...)` 前重建并缓存 `ShadowDepth` 命令。实现显式拒绝 depth prepass 与 alpha-mask shadow 的 pre-MeshDraw 重建，因为现有 replay 会为这些路径绑定 standard material。新增 `pending_command_cache_extract/tests.rs` 覆盖 shadow-only miss、material-bound phase hit + opaque shadow miss、reactive/skinned 拒绝和 full-hit fallback。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 通过；focused locked `cargo test -p zircon_runtime --lib pending_command_cache_extract --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-shadow-cache-rebuild-0623` 首次 120 秒超时无结果，复跑 180 秒仍超时无结果；本轮 target 残留 cargo/rustc 已停止，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片只扩大 opaque shadow cache miss/invalidated 的安全重建面；depth prepass、alpha-mask shadow、opaque/alpha material phase miss、TAA reactive、skinned/GPU-skinning、indirect/VG 与 transparent 仍走 residual `MeshDraw` 构造。 |
| 2026-06-23 | MD-M2 pre-MeshDraw command cache extraction | render_plan02_pre_mesh_draw_command_cache_extraction_static_passed_cargo_lock_blocked | 新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs`，在 GPUScene 同步后、`create_mesh_draw(...)` 创建 WGPU material bind groups 之前，对直接 prepared、非透明、非 skinned、无 reactive mask 且所有 cacheable phase 全命中的静态 pending draw 直接抽取 `CachedMeshDrawCommands`。`BuiltMeshDraws` 现在携带 source prepared queue stats、prebuilt `MeshPassCommandBuffers` 与 extraction stats；compiled scene 将 prebuilt buffers 与 residual draw builder 输出合并，产品诊断新增 `pre_mesh_draw_static_command_cache.skipped_*`。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 扩展为同时锁定 plan owner 与 extraction owner。 | scoped rustfmt/source-anchor/docs-anchor/line-count/trailing-whitespace/diff-check 通过；focused locked `cargo test -p zircon_runtime --lib pending_command_cache_extract --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan02-pre-mesh-cache-extract-0623` 在编译前被当前根 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。 | 本切片只下沉“全 phase 命中”的保守静态路径；部分命中、TAA reactive mask、skinned/GPU-skinning、indirect/VG 与透明排序仍走 residual `MeshDraw` 构造，后续需继续扩大 cacheable path 并补二帧/材质变更产品对拍。 |
| 2026-06-23 | MD-M2 pending command cache plan diagnostics | render_plan02_pending_command_cache_plan_static_passed_cargo_lock_blocked | 新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs`，在 pending draw → `MeshDraw` 构造之前用 `RenderMeshStaticState`、`MeshDrawQueueProfile`、visibility/relevance 与 material shadow policy 统计静态命令缓存候选 draw/phase；`BuiltMeshDraws`、`CompiledSceneDraws`、`PreparedMeshQueueStats`、`RenderStats` 与产品诊断新增 `pending_static_command_cache_*` 计数，区分 pre-MeshDraw 候选、实际 command cache hit/rebuild/miss。新增 `runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 锁定 module owner、build 接入点、stats/diagnostics 桥接和 docs/status 锚点。 | scoped rustfmt 通过；source-anchor/docs-anchor scans、line-count guard 与 diff-check 已按本切片执行；focused locked Cargo test 尚未进入编译,当前根 `Cargo.lock` 更新需求会在 `--locked` 下阻断,不计 Cargo/WGPU/RenderDoc 通过。 | 本切片把 MD-M2 的下沉边界推进到 pending draw 阶段并可观测化；仍未真正跳过静态对象的 `MeshDraw`/WGPU bind 资源构造，后续需在二帧缓存命中时复用 command-owned resource handles，并补静态第二帧/材质变更 focused tests 与产品诊断对拍。 |
| 2026-06-23 | Plan 02 prepared queue tests owner split | render_plan02_prepared_queue_tests_owner_split_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/prepared_queue.rs` 从 944 行 inline-test stats owner 收敛为 272 行生产统计 owner，只保留 `PreparedMeshQueueStats`、`prepare_mesh_queue(...)`、`summarize_prepared_mesh_queue_items(...)` 与 stats bridge；新增 `graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs`(671 行)承接原 prepared queue behavior tests 和 fixture helpers，覆盖 early-z/shadow phase counts、static/dynamic/GPU instancing candidate grouping、skinned GPU/CPU-morphed stats、LOD stats、mesh pass command buffer/replay/GPUScene stats forwarding。新增 `runtime_15_prepared_mesh_queue_is_folder_backed` 锁定 parent 不再承载 inline tests、child test owner、docs/status 锚点和父子行数预算。 | scoped rustfmt/static/source-anchor/docs-anchor/diff-check 通过；focused locked Cargo test 尚未进入编译,当前根 `Cargo.lock` 更新需求会在 `--locked` 下阻断,不计 Cargo/WGPU/RenderDoc 通过。 | 本切片关闭 `prepared_queue.rs` 近千行文件预算和 stats test owner 回流风险；仍需在锁文件漂移清理后补跑 prepared queue focused tests，并继续推进 MD-M2 静态对象跳过 processor/部分 draw 构造下沉。 |
| 2026-06-23 | Plan 02 mesh pass processor tests owner split | render_plan02_mesh_pass_processor_tests_owner_split_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs` 从 280 行 inline-test root 收敛为 15 行 module declaration/re-export surface；新增 `graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs`(359 行)承接原 processor 行为测试，并补计划表 focused guards：depth prepass 过滤 transparent、opaque/base 命令保留 custom+standard material 槽供 fallback shader replay 选择、shadow 排除非 caster 且 alpha-mask 使用 `ShadowDepthAlphaMask` variant。新增 `runtime_15_mesh_pass_processors_are_folder_backed` 锁定 processor root 不再承载测试/fixtures、docs/status 锚点和父子行数预算。 | scoped rustfmt/source-anchor scans 通过；focused locked Cargo test 尚未进入编译,当前根 `Cargo.lock` 更新需求会在 `--locked` 下阻断,不计 Cargo/WGPU/RenderDoc 通过。 | 本切片关闭 processor root 结构漂移和 Plan 02 processor focused source-guard 缺口；仍需在锁文件漂移清理后补跑 processor focused tests,并继续补 replay 产物对拍与 RenderDoc 绑定序列确认。 |
| 2026-06-23 | MD-M3 replay state-dedup focused tests | render_plan02_replay_state_dedup_focused_tests_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs` 将 bind-group 与 geometry 状态判断抽成 `should_bind_raw_group(...)`、`should_bind_geometry(...)` 纯 helper,现有 `bind_raw_group_if_needed(...)` 与 `bind_geometry_if_needed(...)` 继续委派到同一逻辑。新增 focused tests 覆盖 pipeline change 计数、同 slot bind skip、pipeline 切换后 bind tracking reset、geometry 重复跳过与 pipeline 切换后重绑。 | `rustfmt --edition 2021 --check` 通过；source-anchor scan 通过；focused locked `cargo test -p zircon_runtime --lib mesh_draw_command_replayer --no-default-features --features core-min --locked --jobs 1` 被当前 `Cargo.lock` 更新需求在编译前阻断,不计 Cargo/WGPU/RenderDoc 通过。 | 仍需在锁文件漂移清理后补跑 focused tests,并继续补 replay 产物对拍与 RenderDoc 绑定序列确认。 |
| 2026-06-23 | Plan 02 mesh draw command list owner split | render_plan02_mesh_draw_command_list_owner_split_static_passed_cargo_lock_blocked | `graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs` 从 1027 行减压到 291 行,只保留 `MeshDrawCommandList`、`MeshPassCommandBuffers`、indirect batch stats 与排序/统计 helper；新增 `graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs`(297 行)承接 `build_mesh_pass_command_buffers*`、processor fan-out、静态 cache lookup/rebuild 与 dynamic command append；新增 `graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs`(443 行)承接原 inline 行为测试；新增 `runtime_15_mesh_draw_command_list_is_folder_backed` 锁定 owner 挂载、moved builder/test owner 不回流、docs/status 锚点和父子行数预算。 | scoped rustfmt/static scans、line-count/docs-anchor scans 与 scoped diff-check 通过；locked core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1` 被当前 `Cargo.lock` 更新需求在编译前阻断,不计 Cargo/WGPU/RenderDoc 通过。 | 本切片只关闭 `MeshDrawCommandList` 文件预算和 owner 边界；MD-M1/MD-M2/MD-M3 的功能验收缺口仍按本表既有后续项推进。 |
| 2026-06-23 | Render index 当前状态总览拆分 | MD-M4 基础 handoff 完成,MD-M2 command cache miss/失效诊断已接入,MD-M3 command sort input/state bucket depth 语义已接入,MD-M1~M2 仍部分完成 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 02 MeshDrawCommand 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：per-pass processor 收敛、静态命令缓存下沉到批次构建、replay 产物对拍；验收缺口：需要 focused tests 补跑、cached command 二帧/材质变更复用对拍、state-dedup replay 产物对拍和 RenderDoc 绑定序列确认 |
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
