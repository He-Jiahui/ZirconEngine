---
related_code:
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/compiled_graph_cache_tests.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/postprocess_context_guards.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/renderer_context_guards.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pass_executor_registry_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/core_contracts.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/postprocess_routes.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/external_compute_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pipeline_asset_compile_tests.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/default_pipelines.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/temporal_and_ops.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/compile_options.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/feature_descriptors.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/validation_core.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/validation_descriptors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pipeline_compile_monolith_tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphPass.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphUtils.h
  - dev/bevy/crates/bevy_render/src/texture/texture_cache.rs
  - dev/bevy/crates/bevy_render/src/texture/texture_attachment.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_render/src/renderer/mod.rs
  - dev/bevy/crates/bevy_render/src/renderer/render_context.rs
  - dev/Fyrox/fyrox-impl/src/renderer/cache/mod.rs
  - dev/godot/servers/rendering/rendering_device_graph.h
plan_sources:
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
---

# 计划 01:RenderGraph 向 RDG 对齐

## 目标

把 `zircon_runtime/src/render_graph/` 从"统计与依赖记录层"升级为真正承担资源生命周期、瞬态复用、pass 裁剪与编译缓存职责的 RDG 等价物。完成后:

1. 所有 pass 资源(scene color、depth、G-buffer、shadow atlas、history)由 graph 分配或注册,生命周期由 graph 推导。
2. 同一帧内不重叠生命周期的瞬态纹理/缓冲复用同一块物理资源(池化别名)。
3. 对最终输出无贡献的 pass 在编译期被裁剪。
4. feature descriptor → compiled graph 的解析编译结果按签名缓存,稳态帧零重编译。

## 现状与差距

- `render_graph/builder.rs`、`graph.rs` 已有 pass/资源声明与依赖推导,但风险清单(P1)明确指出它"更像统计层":真实 wgpu 资源仍由 `FrameSubmissionContext` 与各 executor 自行持有,生命周期固定。
- `graphics/pipeline/declarations/compiled_render_pipeline.rs` 的编译在帧时执行(pipeline asset → feature descriptors → pass contracts),没有跨帧缓存,feature 集不变时也重复付出解析成本。
- 部分 pass 缺显式 clear/load 语义(风险清单 P0 的一部分),attachment ops 虽存在但未被 graph 统一校验。
- 无 pass culling:禁用下游效果时,上游孤立 pass 仍然执行。

## 参考代码

主参考(UE RDG):

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h` | `FRDGBuilder` 的 CreateTexture/CreateBuffer/RegisterExternal*、AddPass、Execute 生命周期;`QueueTextureExtraction` 跨帧导出 |
| `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphPass.h` | pass 参数结构体如何声明资源访问(读/写/UAV/RT),执行 lambda 与参数分离 |
| `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphResources.h` | `FRDGTexture`/`FRDGBuffer` 句柄、transient 标记、生命周期区间(FirstPass/LastPass) |
| `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphUtils.h` | Clear/Copy/Resolve 等标准工具 pass 的形态 |

次参考:`dev/bevy/crates/bevy_render`(render graph 的 Rust 表达,节点/slot 风格,主要看 Rust 侧 API 人体工学,不照搬其无生命周期推导的设计)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_render/src/texture/texture_cache.rs` | `TransientResourcePool` 池化复用 | `TextureCache::get` 以完整 `TextureDescriptor` 做桶键、`frames_since_last_use < 3` 帧龄驱逐;与 Zircon 的描述符桶 + KEEP_FRAMES 驱逐同构,但 bevy 无生命周期区间着色,同帧别名需自实现 |
| `dev/bevy/crates/bevy_render/src/texture/texture_attachment.rs` | 首写 attachment ops 裁决 | `ColorAttachment`/`DepthAttachment` 以 `is_first_call: AtomicBool` 实现"首写 Clear、后续 Load";Zircon 把同一裁决前移到 `compile()` 期静态校验,不要照搬运行期原子标记 |
| `dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs` | 编译结果缓存 | `PipelineCache` 的 key 化缓存、异步编译与失效流程;注意它只缓存 PSO 粒度,graph 粒度的 `CompiledGraphCache` 键组成仍以 Unity RenderGraphCompilationCache 为准 |
| `dev/bevy/crates/bevy_render/src/renderer/mod.rs` | graph 执行驱动 | 新版 bevy 已删除 node 式 render graph,改为 `RenderGraph` schedule(Begin/Render/Submit/Finish)+ `render_system` 驱动;印证 Rust 侧不需要 trait-object 节点图,Zircon 的 compiled pass 序列 + executor registry 形态可行 |
| `dev/bevy/crates/bevy_render/src/renderer/render_context.rs` | pass 录制与提交分段 | `RenderContext`/`PendingCommandBuffers` 的 encoder 生命周期管理与分段 submit,wgpu 下命令录制组织的参考形态 |
| `dev/Fyrox/fyrox-impl/src/renderer/cache/mod.rs` | 池条目驱逐策略 | `TemporaryCache`/`TimeToLive` 按帧龄回收 GPU 资源,Fyrox 全部 renderer 缓存(texture/geometry/shader)共用该形态;印证瞬态池可独立于 graph 先落地 |
| `dev/godot/servers/rendering/rendering_device_graph.h` | 资源使用跟踪(C++) | `RenderingDeviceGraph` 按资源 usage 推导命令依赖并重排/插 barrier;wgpu 自动 barrier 下只借鉴其 usage 合并思路,barrier 部分不移植 |

`pass culling(反向可达性裁剪)` 与 `graph 粒度编译缓存` 无 Rust 同类参照,实现时以 UE/Unity 为唯一样板,按 index §8 第 8 条配对拍测试先行。

wgpu 适配要点:wgpu 没有显式 barrier 与 placed resource,RDG 的"自动 barrier"在本引擎对应为 (a) 资源池复用时机的正确性(同 encoder 内 usage 冲突由 wgpu 校验),(b) pass 间纹理 usage 推导(声明读写 → TextureUsages 合并),(c) load/store/clear ops 的统一裁决。瞬态别名退化为"池化复用",不做底层内存 alias。

## 目标架构

归属:全部在 `zircon_runtime/src/render_graph/`(基础设施层)与 `graphics/scene/scene_renderer/graph_execution/`(执行层),facade 不变。

核心类型(新增/改造):

- `RenderGraphBuilder`:帧内唯一声明入口。`create_texture(desc) -> RgTextureHandle`、`register_external_texture(...)`、`add_pass(decl, executor_id)`。executor 不再直接拿 `wgpu::TextureView`,改为在执行期通过 `RgResourceResolver` 以句柄解析。
- `TransientResourcePool`:按 (size, format, usage, sample_count) 键控的纹理/缓冲池;graph 编译出每个资源的 [first_pass, last_pass] 区间,区间不重叠的资源映射到同一池条目;帧末归还,LRU 收缩。
- `CompiledGraphCache`:键 = (pipeline asset 修订, 启用 feature 集, quality profile, viewport 尺寸/格式, capability 摘要) 的哈希;值 = 编译后的 pass 序列 + 资源计划。命中则跳过 descriptor 解析与拓扑排序。
- pass culling:从声明为 output/extraction 的资源反向遍历,未到达的 pass 标记 culled,不录制;统计中保留 culled 计数以便测试断言。
- attachment ops 裁决:每个纹理首次写入 pass 必须给出 Clear 或显式 Load(external);graph 编译期校验,违例返回编译诊断而不是默默通过。

执行层改造:`render_pass_executor_registry.rs` 的 executor 签名增加资源解析上下文;`render_graph_execution_record.rs` 改为按 compiled graph 的 pass 顺序驱动录制,删除 executor 自行查找全局纹理的旁路。

## 里程碑

### RG-M1 资源句柄与生命周期模型

实施切片:
1. `RgTextureHandle`/`RgBufferHandle` 与资源声明表;builder 收口所有内建 pass 的资源声明(mesh、deferred、shadow、post 系列全部经 builder 声明 IO)。
2. 生命周期区间推导与 attachment ops 校验;违例诊断类型。
3. executor 上下文改为句柄解析;删除直接持有 view 的旁路字段。

测试阶段:
- 编译:`cargo check -p zircon_runtime --lib --locked`
- 测试:`cargo test -p zircon_runtime render_graph --locked` 与 `cargo test -p zircon_runtime render_product --locked`
- 验收证据:全部内建 pass 的 IO 在 graph dump 中可见;缺 clear 的 pass 列表清零或逐个给出显式 Load 理由。
- 文档:更新 `docs/zircon_runtime` 下 render_graph 镜像文档。

### RG-M2 瞬态资源池

实施切片:
1. `TransientResourcePool` 与区间着色分配(线性扫描即可,资源数 < 百级)。
2. history/extraction 资源标记为持久,绕过池;与计划 06 的 history 管理对齐。
3. 池统计(峰值条目、复用率)进 RenderStats。

测试阶段:
- `cargo test -p zircon_runtime render_graph --locked`(新增:同尺寸两瞬态资源生命周期不重叠时物理资源数为 1 的断言)
- 验收证据:典型 deferred + post 场景的瞬态纹理物理数明显低于逻辑数;RenderDoc 抓帧确认画面不变。

### RG-M3 pass culling 与编译缓存

实施切片:
1. 反向可达性裁剪 + culled 统计。
2. `CompiledGraphCache` 键控缓存;feature 开关/分辨率变化触发重编译,稳态命中。
3. 帧时 descriptor 解析移出热路径(仅 miss 时执行)。

测试阶段:
- `cargo test -p zircon_runtime render_graph --locked` 与 `cargo test -p zircon_runtime compiled_render_pipeline --locked`
- 验收证据:关闭 bloom 后其上游独立 downsample pass 被裁剪(统计断言);连续两帧第二帧编译耗时为 0(缓存命中计数断言)。

### RG-M4 诊断与 RenderDoc 对接

实施切片:
1. graph dump(pass 顺序、资源区间、culled 列表)走 debug 接口输出。
2. 每个 pass 录制时打 debug marker(对接 `graphics_debugger_capture.rs` 与既有 RenderDoc 计划)。

测试阶段:
- `cargo test -p zircon_runtime --lib --locked` 范围回归
- 验收证据:开启 RenderDoc 捕获后,抓帧中 pass 名与 graph dump 一致。

## 工程落地细化

本章是计划 01 的实施权威(见 index.md §8 第 7 条)。bind group 槽位、GPU 数据布局、WGSL include、RenderQueueValue、sort_key、测试命名等全局约定直接引用 index.md §8,本章不重复定义。

### 模块与文件落点

现状基础:`render_graph/` 已有 `RgTextureHandle`/`RgBufferHandle`、`create_texture`/`create_buffer`/`import_external_resource`、读写声明、依赖推导、`cull_passes`、`resource_lifetimes`(first_pass/last_pass)与逻辑层 `transient_allocation_plan()`;执行层已有 `RenderGraphExecutionResources`(每帧新建 wgpu 资源,无池化)与 `RgResourceResolver`(pass-scoped graph declaration/access resolver)。本计划的增量是:首写 ops 决策表校验、root 驱动的反向裁剪、物理瞬态池、编译缓存、句柄级物理解析收口。

新增文件:

| 路径 | 职责(一行) |
|------|------------|
| `zircon_runtime/src/render_graph/dump.rs` | `RenderGraphDump` 纯数据构建与文本序列化(pass 顺序、资源区间、culled 列表),无 wgpu |
| `zircon_runtime/src/render_graph/tests/attachment_ops.rs` | 首写 attachment ops 决策表与 usage flags 的编译期校验测试 |
| `zircon_runtime/src/render_graph/tests/culling.rs` | root 驱动反向可达裁剪的单测(从既有 `tests/ordering.rs` 拆出裁剪主题) |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs` 与 `transient_resource_pool/` | `TransientResourcePool`:描述符桶 + 区间着色的物理资源池；子 owner 分离 generation-qualified allocation lease 与 focused tests |
| `zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs` | `CompiledGraphCache`/`CompiledGraphCacheKey`/统计与 LRU 驱逐,通过 `#[cfg(test)] mod tests;` 挂载 child tests |
| `zircon_runtime/src/graphics/pipeline/compiled_graph_cache/tests.rs` | compiled graph cache hit/miss/status/fingerprint/target-format/LRU 源码合同测试 |

修改文件:

| 路径 | 改动点 |
|------|--------|
| `zircon_runtime/src/render_graph/types.rs` | 新增 `RenderGraphResourceUsageFlags`;`RenderGraphResourceDeclaration`/`RenderGraphResourceLifetime` 增 `usage` 字段 |
| `zircon_runtime/src/render_graph/builder.rs` | `mark_persistent`/`mark_readback`/`import_external_resource_with_usage`;`compile()` 内首写 ops 校验;`cull_passes` 改为 usage-root 驱动并删除 `has_no_writes`/`writes_external` 兜底存活 |
| `zircon_runtime/src/render_graph/error.rs` | 新增 `FirstWriteMissingAttachmentOps`、`MissingCullRoot` 变体 |
| `zircon_runtime/src/render_graph/graph.rs` | `allocate_transient_lifetimes` 槽位分桶键从 kind 细化为完整描述符哈希;`CompiledRenderGraphTransientAllocation` 增 `bucket_key_hash: u64`;新增 `CompiledRenderGraph::dump()` |
| `zircon_runtime/src/render_graph/mod.rs` | 仅 wiring:声明 `dump` 模块、导出新类型 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs` | `RgResourceResolver` 增加物理解析(借用 `RenderGraphExecutionResources`),元数据 API 全量保留 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs` | `resource_resolver()` 改为按需组合 graph 元数据 + `gpu.resources` 物理表返回 `RgResourceResolver`;导出名同步 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/` | 删除 `materialize_transient_resources`(职责移入池);新增 `bind_pooled_texture`/`bind_pooled_buffer`;`require_texture_view`/`require_buffer` 可见性收紧为 `pub(in crate::graphics::scene::scene_renderer)`,执行器只许经 `RgResourceResolver` |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs` | 仅 wiring:`transient_resource_pool` 模块声明与受控导出 |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs` | `SceneRendererCore` 增 `transient_pool: TransientResourcePool` 字段 |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs` | `RenderGraphExecutionResources::new()` 之后接 `transient_pool.allocate_frame_resources(...)`;帧末 `end_frame()` |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs` | 删除其内 `materialize_transient_resources` 调用;pass 循环改经 `RgResourceResolver` 注入 |
| `zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_asset.rs` | `RenderPipelineAsset` 增 `revision: u64` |
| `zircon_runtime/src/graphics/pipeline/declarations/mod.rs`、`pipeline/mod.rs` | 仅 wiring:`compiled_graph_cache` 导出 |
| `zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/render_framework_state.rs` | `RenderFrameworkState` 增 `compiled_graph_cache: CompiledGraphCache` 字段 |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs` | `compile_submission_pipeline` 改为 `CompiledGraphCache::get_or_compile`,原编译函数降级为 miss 路径闭包 |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs` | `compiled_pipeline` 持有类型改 `Arc<CompiledRenderPipeline>` |
| `zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs`、`set_pipeline_asset/set_pipeline_asset.rs`、`reload_pipeline/reload_pipeline.rs` | 资产注册/替换/重载时 bump `revision` 并调 `compiled_graph_cache.invalidate_pipeline(handle)` |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/`(update_stats 实现文件) | 写入池统计与缓存命中统计 |
| `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs` | 内建资源声明补 usage 标记(backbuffer→present、history→persistent、readback 输出→readback);违反首写决策表的声明就地修正 |
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `RenderStats` 增池/缓存统计字段(framework 契约层,纯 POD,无 wgpu) |
| `zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture.rs`(及 `graphics_debugger_capture/` 状态) | capture 帧附带 graph dump 文本,经既有 capture 查询路径返回 |

### 核心类型与接口

层归属约定:`render_graph/` 是无 wgpu 的规划层;`graphics/**` 是实现层,wgpu 类型只能出现在这里;`core::framework::render` 契约层只接收纯数据统计(`RenderStats` 字段)。`RgResourceResolver`、`TransientResourcePool` 携带 wgpu 类型,因此固定在 graphics 层,不出 graphics(不进 framework 契约、不进 `zircon_runtime_interface`)。

```rust
// ---- render_graph/types.rs(规划层,无 wgpu)----
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderGraphResourceUsageFlags {
    /// 写入它的 pass 链是裁剪 root(最终呈现面)
    pub present: bool,
    /// CPU 回读目标,裁剪 root(对接计划 16 GpuReadbackQueue)
    pub readback: bool,
    /// 跨帧持久(history 等),裁剪 root 且绕过瞬态池(对接计划 06)
    pub persistent: bool,
}
// RenderGraphResourceDeclaration / RenderGraphResourceLifetime 各增字段:
//     pub usage: RenderGraphResourceUsageFlags,

// ---- render_graph/builder.rs(规划层)----
impl RenderGraphBuilder {
    // 既有 API 原样保留(签名不变):
    // create_texture(desc: TextureDesc) -> RgTextureHandle
    // create_buffer(desc: BufferDesc) -> RgBufferHandle
    // import_external_resource(name) -> ExternalResource
    // add_pass_with_executor(name, queue, executor_id) -> RenderPassId
    // read_texture / write_texture / write_texture_with_ops / write_storage_texture
    // read_buffer / write_buffer / read_external / write_external(_with_ops)
    // compile(self) -> Result<CompiledRenderGraph, RenderGraphError>

    /// 新增:带 usage 的外部导入;既有 import_external_resource 等价于全 false。
    pub fn import_external_resource_with_usage(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
    ) -> ExternalResource;
    /// 新增:标记瞬态资源跨帧持久(history)。persistent 资源允许首写 Load,绕过池化。
    pub fn mark_persistent(&mut self, texture: RgTextureHandle) -> Result<(), RenderGraphError>;
    /// 新增:标记资源为回读 root。
    pub fn mark_readback(&mut self, resource: RenderGraphResource) -> Result<(), RenderGraphError>;
}

// ---- render_graph/error.rs 新增变体 ----
// FirstWriteMissingAttachmentOps { resource: String, pass: String }
// MissingCullRoot { graph_name: String }   // 全图无 present/readback/persistent/side-effect root

// ---- render_graph/dump.rs(规划层)----
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphDump {
    pub graph_name: String,
    pub pass_rows: Vec<RenderGraphDumpPassRow>,      // 执行序、queue、executor_id、culled、资源 IO
    pub resource_rows: Vec<RenderGraphDumpResourceRow>, // name、kind、usage、[first_pass, last_pass]、桶哈希
}
impl CompiledRenderGraph {
    pub fn dump(&self) -> RenderGraphDump;
}
impl RenderGraphDump {
    pub fn to_text(&self) -> String; // 行式文本,RenderDoc 抓帧对拍用
}

// ---- graphics/.../graph_execution/transient_resource_pool.rs(graphics 实现层)----
pub(in crate::graphics::scene::scene_renderer) struct TransientResourcePool {
    texture_buckets: HashMap<TransientTextureKey, Vec<PooledTextureEntry>>,
    buffer_buckets: HashMap<TransientBufferKey, Vec<PooledBufferEntry>>,
    frame_index: u64,
    last_frame_stats: TransientPoolFrameStats,
}
/// 描述符桶键:同键资源物理可互换。label 不参与哈希。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TransientTextureKey {
    width: u32, height: u32, depth: u32,
    mip_levels: u32, sample_count: u32,
    dimension: TextureDimension, format: TextureFormat, usage: TextureUsage,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TransientBufferKey { size_bytes: u64, usage: BufferUsage }
struct PooledTextureEntry {
    texture: wgpu::Texture,
    default_view: wgpu::TextureView,
    last_used_frame: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransientPoolFrameStats {
    pub logical_texture_count: usize,
    pub physical_texture_count: usize,
    pub reused_texture_count: usize,
    pub created_texture_count: usize,
    pub logical_buffer_count: usize,
    pub physical_buffer_count: usize,
    pub pool_entry_count: usize,
    pub evicted_entry_count: usize,
}
impl TransientResourcePool {
    pub fn new() -> Self;
    /// 帧首调用:对 graph 的瞬态生命周期做桶内区间着色,把逻辑资源绑定进
    /// RenderGraphExecutionResources(name → 池条目 view/buffer)。
    /// 跳过 imported、usage.persistent、is_sparse_reserved_texture() 的资源。
    pub fn allocate_frame_resources(
        &mut self,
        device: &wgpu::Device,
        graph: &CompiledRenderGraph,
        resources: &mut RenderGraphExecutionResources,
    ) -> Result<TransientPoolFrameStats, String>;
    /// 帧末调用:frame_index += 1;驱逐 last_used_frame 距今超过 KEEP_FRAMES(=8)的条目。
    pub fn end_frame(&mut self);
    pub fn last_frame_stats(&self) -> TransientPoolFrameStats;
}
```

`allocate_frame_resources` 分配算法(物理别名 = 池化复用,不做底层内存 alias):

1. 取 `graph.resource_lifetimes()`,过滤掉 `imported`、`usage.persistent`、`is_sparse_reserved_texture()`(稀疏纹理保留计划 13 的既有旁路)。
2. 按 `TransientTextureKey` / `TransientBufferKey` 分桶(描述符哈希做桶)。
3. 桶内按 `(first_pass, last_pass, name)` 排序后线性扫描区间着色:维护 `slot_last_passes: Vec<usize>`,若存在 slot 满足 `slot_last_pass < lifetime.first_pass`(区间不相交)则复用该 slot,否则新开 slot。算法与 `graph.rs::allocate_transient_lifetimes` 既有实现一致,唯一差异是桶键从 kind 细化为完整描述符(资源数 < 百级,线性扫描足够,与里程碑 RG-M2 原文一致)。
4. 桶内 slot i 映射到该桶池条目 i:条目已存在则复用(`reused_texture_count += 1`,刷新 `last_used_frame`);不足则 `device.create_texture`/`create_buffer` 新建(`created_texture_count += 1`)。物理创建复用 `render_graph_execution_resources.rs` 中既有的 `create_wgpu_texture`/`create_wgpu_buffer` 与 usage 推导函数(随删除 `materialize_transient_resources` 一并移入本文件)。
5. 对每个逻辑资源调 `resources.bind_pooled_texture(name, entry.default_view.clone())` / `bind_pooled_buffer(name, entry.buffer.clone())`;SSR coarse mip 别名沿用既有 `ssr_pyramid_mip_alias` 路径(父纹理来自池条目)。

```rust
// ---- graphics/pipeline/compiled_graph_cache.rs(graphics 实现层,无 wgpu)----
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompiledGraphCacheKey(u64); // 各成分 FNV-1a 折叠
impl CompiledGraphCacheKey {
    pub fn compose(
        pipeline_handle: RenderPipelineHandle,
        pipeline_revision: u64,                       // 资产修订,注册/重载时 bump
        options: &RenderPipelineCompileOptions,        // feature 集 + 能力开关 + msaa + post stack
        quality_profile: Option<&str>,                 // profile
        viewport_size: UVec2,                          // viewport class(尺寸;格式经 options.graph_msaa 与内建约定)
        extract_fingerprint: u64,                      // 编译期读取的 extract 字段指纹
        capabilities: &RenderCapabilitySummary,        // device caps 摘要
    ) -> Self;
}
/// 编译期读取面指纹:core pipeline 种类、effect stack 各效果 enable 位、
/// camera target 拓扑相关字段。逐帧变化的场景数据(变换/灯光参数)不进指纹。
pub(crate) fn extract_compile_fingerprint(extract: &RenderFrameExtract) -> u64;

pub struct CompiledGraphCache {
    entries: Vec<CompiledGraphCacheEntry>, // capacity 默认 16,LRU(对齐 Unity k_CachedGraphCount 量级)
    capacity: usize,
    stats: CompiledGraphCacheStats,
}
struct CompiledGraphCacheEntry {
    key: CompiledGraphCacheKey,
    pipeline_handle: RenderPipelineHandle, // invalidate_pipeline 用
    last_used_frame: u64,
    pipeline: Arc<CompiledRenderPipeline>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledGraphCacheStats { pub hits: u64, pub misses: u64, pub evictions: u64 }
impl CompiledGraphCache {
    pub fn with_capacity(capacity: usize) -> Self;
    /// 命中:刷新 last_used_frame 并克隆 Arc;miss:执行 compile 闭包,
    /// 满容量时按 last_used_frame 驱逐最旧条目(evictions += 1)。
    pub fn get_or_compile(
        &mut self,
        key: CompiledGraphCacheKey,
        pipeline_handle: RenderPipelineHandle,
        frame_index: u64,
        compile: impl FnOnce() -> Result<CompiledRenderPipeline, RenderFrameworkError>,
    ) -> Result<Arc<CompiledRenderPipeline>, RenderFrameworkError>;
    pub fn invalidate_pipeline(&mut self, pipeline: RenderPipelineHandle);
    pub fn clear(&mut self);
    pub fn stats(&self) -> CompiledGraphCacheStats;
}

// ---- graphics/.../render_pass_execution_context/resource_resolver.rs(graphics 实现层)----
/// 跨计划契约名:RgResourceResolver。
/// 元数据 API 全量保留,新增物理解析。不出 graphics 层。
pub struct RgResourceResolver<'a> {
    graph: &'a CompiledRenderGraph,
    pass_id: RenderPassId,
    physical: Option<&'a RenderGraphExecutionResources>, // 纯元数据上下文(无 GPU 测试)为 None
}
impl<'a> RgResourceResolver<'a> {
    pub fn new(graph: &'a CompiledRenderGraph, pass_id: RenderPassId) -> Self; // 元数据模式
    pub(in crate::graphics::scene::scene_renderer) fn with_physical(
        graph: &'a CompiledRenderGraph,
        pass_id: RenderPassId,
        physical: &'a RenderGraphExecutionResources,
    ) -> Self;
    // 物理解析:句柄 → declaration → 校验本 pass 声明了对应访问 → 物理表取值。
    // 未声明访问返回 Err(消息含 pass 名与资源名),这是"删除旁路"后的唯一访问面。
    pub fn texture_view(&self, handle: RgTextureHandle) -> Result<&'a wgpu::TextureView, String>;
    pub fn external_texture_view(&self, handle: ExternalResource) -> Result<&'a wgpu::TextureView, String>;
    pub fn buffer(&self, handle: RgBufferHandle) -> Result<&'a wgpu::Buffer, String>;
    // 内建 executor 以 PostProcessGraphResourceNames 常量寻址,按名解析同样过声明校验:
    pub fn texture_view_by_name(&self, name: &str, access: RenderGraphResourceAccessKind) -> Result<&'a wgpu::TextureView, String>;
    pub fn buffer_by_name(&self, name: &str, access: RenderGraphResourceAccessKind) -> Result<&'a wgpu::Buffer, String>;
    // 元数据 API(签名保持稳定):
    // resource_declaration / resource_declaration_by_name / resource_lifetime /
    // resource_lifetime_by_name / pass_declares_resource / pass_declares_resource_access /
    // pass_resource_access / pass_resource_access_by_name / pass_resource_declaration_by_name / pass_resources
}
```

首写 attachment ops 决策表(`compile()` 期对每个非 culled 瞬态纹理找到执行序中第一个写它的 pass 后裁决;违例返回 `RenderGraphError`,不默默通过):

| 首写场景 | ops | 裁决 |
|---------|-----|------|
| transient texture,attachment 写(`write_texture`/`write_texture_with_ops`) | `Clear/Store` 或 `Clear/Discard` | 合法 |
| 同上 | `Load/*` | `LoadBeforeProducer`(既有,保留) |
| 同上 | `None`(漏声明 ops) | `FirstWriteMissingAttachmentOps`(新增) |
| transient texture,storage 写(`write_storage_texture`,compute 整面覆写) | `None` | 合法(DontCare 语义,覆写责任在 executor) |
| `mark_persistent` 标记的瞬态纹理(history) | `Load/*` | 合法(跨帧内容);未标记则按上两行裁决 |
| external(backbuffer / 相机 RT) | `Load` 或 `Clear` | 合法(外部内容由导入方负责) |

root 驱动反向裁剪(替换 `cull_passes` 既有判据):live root = 写入 `usage.present || usage.readback || usage.persistent` 资源的 pass、`flags.has_side_effects`、`!flags.allow_culling`;然后沿"root 的读集合 + 显式 dependencies"逆执行序传播(算法骨架沿用既有 `cull_passes` 的逆序扫描)。同一变更内删除"写 external 一律存活"与"无写 pass 一律存活"两条兜底;全图无 root 时编译报 `MissingCullRoot`。`compile.rs` 在同一变更内给内建资源补标记:viewport 输出 external → `present`,history 槽 → `persistent`,picking/capture 输出 → `readback`。

### GPU 数据布局与 WGSL 约定(适用时)

本计划不新增任何 GPU buffer、bind group 或 WGSL include:瞬态池与编译缓存是 CPU 侧资源管理,`RgResourceResolver` 只是句柄到既有 wgpu 资源的查找面。唯一与 GPU 布局相关的规则是 usage 推导:graph 编译期把每个纹理的所有声明访问合并为 `TextureUsage`(被读 → `SAMPLED`,storage 写 → `STORAGE`,attachment 写 → `RENDER_ATTACHMENT`,history 拷贝 → `COPY_SRC|COPY_DST`),该合并结果进入 `TransientTextureKey`,保证池条目可被所有使用场景复用。新 pass 的 bind group 槽位遵守 index.md §8 第 1 条,本计划不涉及新增槽位。

### 帧时序与集成点

稳态一帧的精确时序(函数均为现存实现,括号内为本计划的插入/替换动作):

1. `submit_frame_extract`(`graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs`)→ `build_frame_submission_context`。
2. `compile_submission_pipeline`(`build_frame_submission_context/compile_pipeline.rs`):**替换** —— 由每帧无条件 `pipeline_asset().compile_with_options(extract, options)` 改为 `state.compiled_graph_cache.get_or_compile(key, ...)`,key 按上节 `compose` 组成;原编译调用降级为 miss 闭包,`validate_compiled_pipeline_capabilities` 只在 miss 路径执行(命中条目编译时已验过)。`FrameSubmissionContext::compiled_pipeline()` 改持 `Arc<CompiledRenderPipeline>`。缓存实例挂在 `RenderFrameworkState`(`render_framework_state.rs`),与 `pipelines`/`viewports` 同级。
3. 失效路径:`register_pipeline_asset`、`set_pipeline_asset`、`reload_pipeline`、`set_quality_profile` 在变更资产/选项时 bump `RenderPipelineAsset::revision` 并调 `invalidate_pipeline(handle)`;viewport 尺寸与能力摘要变化天然改变 key,无需显式失效。
4. `SceneRenderer::render_frame_with_pipeline` → `render_frame_with_pipeline_to_target` → `SceneRendererCore::render_compiled_scene`(`render/render.rs`):在 `let mut graph_resources = RenderGraphExecutionResources::new();` 之后先由 `bind_frame_graph_resources(...)` 绑定 live renderer-owned frame resources,再**插入** `self.transient_pool.allocate_frame_resources(device, &pipeline.graph, &mut graph_resources)?`;**删除** `render.rs` 与 `execute_graph_stage.rs` 中对 `materialize_transient_resources` 的两处调用及该函数本体(硬切换)。history 纹理仍由 `prepare_history_textures`(`scene_renderer_history`)持久管理并以 import 方式进 `graph_resources`,池不接管(与计划 06 对齐的接缝就是 `mark_persistent` + import)。
5. pass 循环(`execute_graph_stage.rs`):构造 `RenderPassExecutionContext` 时 `with_resource_resolver(&pipeline.graph, pass.id)` 语义升级为物理模式(`RgResourceResolver::with_physical`,physical 取 `gpu.resources`);各内建 executor(`builtin_scene_executors.rs`、`builtin_postprocess_executors.rs`、`preview_sky_executor.rs`)中所有 `resources.require_texture_view(name)` 直取改为 `context.resolver()?.texture_view_by_name(name, access)`;`RenderGraphExecutionResources::require_*` 可见性收紧,使绕过声明校验的直取在编译期不可达(删除旁路)。debug marker 仍由 `marker_for_render_graph_pass`(`graphics/debug_markers.rs`)在录制时打点,culled pass 不录制、不打 marker。
6. 帧末:`render_compiled_scene` 返回前调 `self.transient_pool.end_frame()`;`update_stats` 把 `transient_pool.last_frame_stats()` 与 `compiled_graph_cache.stats()` 写入 `RenderStats` 新字段(`backend_types.rs`):`last_graph_transient_pool_logical_texture_count`、`last_graph_transient_pool_physical_texture_count`、`last_graph_transient_pool_reused_texture_count`、`last_graph_transient_pool_created_texture_count`、`last_graph_transient_pool_entry_count`、`last_graph_compiled_cache_hits`、`last_graph_compiled_cache_misses`、`last_graph_compiled_cache_evictions`。
7. 诊断:`begin_graphics_debugger_capture`/`finish_active_capture_and_relock`(`graphics_debugger_capture.rs`)的 capture 帧附带 `pipeline.graph.dump().to_text()`,经既有 capture 查询路径返回给调用方与 RenderDoc 对拍。

### 实施切片细化

里程碑结构沿用正文 RG-M1..M4;每个切片是一次可评审的变更,切片期只跑 `cargo check -p zircon_runtime --lib --locked`,测试集中在各里程碑测试阶段。

**RG-M1 资源句柄与生命周期模型**

- 切片 1.1 usage flags 与声明扩展
  1. `render_graph/types.rs`:加 `RenderGraphResourceUsageFlags`;`RenderGraphResourceDeclaration`/`RenderGraphResourceLifetime` 加 `usage` 字段 → 判据:cargo check 通过,既有构造点以 `Default::default()` 补齐。
  2. `render_graph/builder.rs` + `mod.rs`:`import_external_resource_with_usage`/`mark_persistent`/`mark_readback`,`ResourceNode` 存 usage 并传入 declarations/lifetimes → 判据:`render_graph/tests` 既有用例不改语义即编译通过。
  3. `graphics/pipeline/render_pipeline_asset/compile.rs`:viewport 输出标 `present`、history 槽标 `persistent`、readback 输出标 `readback` → 判据:cargo check;graph dump(切片 4.1 前先用 `Debug` 输出人工确认)中标记可见。
- 切片 1.2 首写 attachment ops 校验
  1. `render_graph/error.rs`:加 `FirstWriteMissingAttachmentOps` → 判据:cargo check。
  2. `render_graph/builder.rs::compile()`:按决策表实现首写裁决(在 `infer_resource_dependencies` 之后、`cull_passes` 之前,作用于执行序首写);persistent 资源放行 Load → 判据:cargo check;以临时单测核对一例违例报错文案。
  3. `compile.rs` 与各 feature descriptor 声明修正:对新校验报出的缺 ops 写入逐个补 `write_texture_with_ops` 显式声明或给出 Load 理由(persistent)→ 判据:内建 forward+/deferred/core2d 三条默认管线 `compile()` 全部 Ok。
- 切片 1.3 RgResourceResolver 收口(本里程碑最大切片,波及全部内建 executor)
  1. `resource_resolver.rs`:`RgResourceResolver` 加 `with_physical` 与句柄/按名物理解析；transient texture/buffer 与 typed external texture/buffer 已优先走 exact access-ID binding，persistent/unknown report-only external 保留明确的 whole-resource 兼容分支 → 判据:源级 exact-binding contract 与后续 managed cargo check（旧 whole-resource transient 绕过需继续清零）。
  2. `render_pass_execution_context.rs`:`resource_resolver()` 返回物理模式 resolver(gpu 存在时);删除旧导出名 → 判据:cargo check。
  3. `render_graph_execution_resources.rs`:`require_texture_view`/`require_buffer`/`texture_view`/`buffer` 收紧可见性;`builtin_*_executors.rs`、`preview_sky_executor.rs`、`post_process` 执行路径全部改经 resolver;插件 executor(`zircon_plugins` 的 rendering/hybrid_gi/virtual_geometry/particles)同一变更内适配 → 判据:root workspace + `zircon_plugins` workspace cargo check 全绿,`grep require_texture_view` 在 executor 文件中无直调残留。
- 测试阶段(RG-M1):新增 `render_graph/tests/attachment_ops.rs`;跑 `cargo test -p zircon_runtime render_graph --locked`、`cargo test -p zircon_runtime render_product --locked`;验收证据按正文(graph dump 可见全部内建 pass IO、缺 clear 清零);更新 `docs/zircon_runtime` render_graph 镜像文档。

**RG-M2 瞬态资源池**

- 切片 2.1 池本体与桶化分配计划
  1. `graph.rs`:`allocate_transient_lifetimes` 桶键细化为描述符哈希,`CompiledRenderGraphTransientAllocation` 加 `bucket_key_hash` → 判据:cargo check,既有 `transient_allocation_plan` 统计字段语义保持(slot 字节=桶内描述符尺寸)。
  2. 新增 `transient_resource_pool.rs`:类型、分配算法、`end_frame` 驱逐;把 `create_wgpu_texture`/`create_wgpu_buffer`/usage 推导从 `render_graph_execution_resources.rs` 移入 → 判据:cargo check。
- 切片 2.2 帧路径接池(硬切换)
  1. `scene_renderer_core.rs` 加字段;`render.rs` 插入 `allocate_frame_resources` + 帧末 `end_frame`;删除 `materialize_transient_resources` 本体与 `execute_graph_stage.rs` 调用;`RenderGraphExecutionResources` 加 `bind_pooled_*` → 判据:cargo check;编辑器 viewport 手动冒烟画面正常。
  2. persistent/sparse 旁路核对:history 经 import、sparse 经计划 13 既有路径,池逻辑断言不接管 → 判据:cargo check + 旁路单测留到测试阶段。
- 切片 2.3 统计入口
  1. `backend_types.rs` 加池字段;`update_stats` 写入 → 判据:cargo check。
- 测试阶段(RG-M2):`cargo test -p zircon_runtime render_graph --locked`(含正文要求的"同尺寸两瞬态区间不重叠 → 物理资源数 1"断言);RenderDoc 抓帧确认画面不变;验收证据:deferred + post 典型场景 `physical_texture_count` 明显低于 `logical_texture_count`。

**RG-M3 pass culling 与编译缓存**

- 切片 3.1 root 驱动裁剪
  1. `builder.rs::cull_passes` 改 root 判据,删两条兜底;`error.rs` 加 `MissingCullRoot`;`compile.rs` 标记缺口补齐 → 判据:三条默认管线编译通过且 culled 集合不增(全启用时);关闭 bloom 后 `post.bloom-extract` 上游链 culled(临时断言)。
- 切片 3.2 revision 与 CompiledGraphCache
  1. `render_pipeline_asset.rs` 加 `revision`;注册/替换/重载路径 bump + invalidate → 判据:cargo check。
  2. 新增 `compiled_graph_cache.rs`;`render_framework_state.rs` 挂字段;`compile_pipeline.rs` 接 `get_or_compile`;`frame_submission_context.rs` Arc 化(调用方 `render_frame_with_pipeline` 形参 `&CompiledRenderPipeline` 不变,传 `&*arc`)→ 判据:cargo check;手动连续两帧日志确认第二帧走 hit。
- 切片 3.3 指纹固化与热路径清理
  1. `extract_compile_fingerprint` 清点 `compile_with_options` 的 extract 读取面并实现;debug_assert:命中帧重算指纹等于 key 成分 → 判据:cargo check + 编辑器冒烟无 assert。
  2. `update_stats` 写缓存统计 → 判据:cargo check。
- 测试阶段(RG-M3):`cargo test -p zircon_runtime render_graph --locked` 与 `cargo test -p zircon_runtime compiled_render_pipeline --locked`;验收证据按正文(关 bloom 裁剪统计断言、第二帧零编译 / 命中计数断言)。

**RG-M4 诊断与 RenderDoc 对接**

- 切片 4.1 graph dump
  1. 新增 `render_graph/dump.rs` + `CompiledRenderGraph::dump()`;`mod.rs` wiring → 判据:cargo check。
  2. `graphics_debugger_capture.rs`:capture 帧生成 dump 文本并随 capture 报告暴露 → 判据:`ZR_RENDERDOC_CAPTURE_NEXT=1` 手动抓帧能取到 dump。
- 切片 4.2 marker 对拍
  1. 审计 `execute_graph_stage.rs` 的 `marker_for_render_graph_pass` 打点覆盖全部非 culled pass(含 async 回落 pass);culled pass 零 marker → 判据:`executed_debug_markers()` 与 dump pass 行一一对应。
- 测试阶段(RG-M4):`cargo test -p zircon_runtime --lib --locked` 范围回归;验收证据:RenderDoc 抓帧 pass 名与 graph dump 一致。

### 测试与验收清单

命名遵守 index.md §8 第 6 条(`render_<topic>_*` 单测、`render_product_*` 产物对拍)。

RG-M1(文件:`render_graph/tests/attachment_ops.rs`;resolver 用例在 `resource_resolver.rs` 的 `#[cfg(test)] mod tests`):

| 测试函数 | 断言 |
|---------|------|
| `render_graph_first_write_without_ops_fails_for_attachment_texture` | attachment 用途瞬态纹理首写 ops=None → `Err(FirstWriteMissingAttachmentOps)`,错误文案含资源与 pass 名 |
| `render_graph_first_write_load_without_producer_fails` | 首写 `load_store` → `Err(LoadBeforeProducer)`(既有语义回归) |
| `render_graph_persistent_marked_texture_allows_load_first_write` | `mark_persistent` 后首写 Load → `compile()` Ok |
| `render_graph_storage_first_write_without_ops_is_valid` | `write_storage_texture` 首写 → Ok |
| `render_graph_resolver_rejects_undeclared_resource_access` | pass 未声明的句柄经 `texture_view`/`buffer` 解析 → Err |
| `render_graph_resolver_returns_view_for_declared_access` | 声明过写访问的句柄解析出与物理表同一 view(`RenderBackend::new_offscreen()` 驱动) |

RG-M2(文件:`transient_resource_pool.rs` 内联 tests,需 offscreen backend):

| 测试函数 | 断言 |
|---------|------|
| `render_graph_transient_pool_aliases_disjoint_lifetimes_to_one_texture` | 同描述符两资源区间不相交 → `physical_texture_count == 1 && logical_texture_count == 2` |
| `render_graph_transient_pool_separates_overlapping_lifetimes` | 区间相交 → 物理数 2 |
| `render_graph_transient_pool_buckets_by_descriptor_not_name` | 同名义不同 format → 不同桶、不复用 |
| `render_graph_transient_pool_reuses_entries_across_frames` | 第二帧 `created_texture_count == 0 && reused_texture_count > 0` |
| `render_graph_transient_pool_evicts_stale_entries_after_keep_frames` | 资源停用后第 9 帧 `evicted_entry_count > 0`、`pool_entry_count` 下降 |
| `render_graph_transient_pool_skips_persistent_and_sparse_resources` | persistent/sparse 生命周期不产生池条目(history/SVT 旁路) |

RG-M3(文件:`compiled_graph_cache/tests.rs` child tests;裁剪在 `render_graph/tests/culling.rs`;集成断言在 render_framework 既有测试目录):

| 测试函数 | 断言 |
|---------|------|
| `render_graph_culls_passes_unreachable_from_present_root` | 写孤立纹理的 pass culled=true,统计 `culled_pass_count` 增加 |
| `render_graph_readback_marked_buffer_keeps_producer_alive` | `mark_readback` 的 buffer 生产 pass 不被裁剪 |
| `render_graph_side_effect_pass_survives_culling` | `has_side_effects` pass 始终存活 |
| `render_graph_missing_cull_root_is_compile_error` | 全图无 root → `Err(MissingCullRoot)` |
| `compiled_render_pipeline_cache_hits_on_identical_key` | 同 key 第二次 `get_or_compile` 不执行闭包,`hits == 1` |
| `compiled_render_pipeline_cache_misses_on_feature_set_change` | options 中 feature 集变化 → key 不同 → miss |
| `compiled_render_pipeline_cache_misses_on_viewport_resize` | 尺寸变化 → miss |
| `compiled_render_pipeline_cache_invalidates_on_pipeline_revision_bump` | bump revision 后旧 key 条目失效 |
| `compiled_render_pipeline_cache_evicts_least_recently_used_entry` | 超容量插入驱逐 `last_used_frame` 最小者,`evictions == 1` |
| `render_graph_disabled_bloom_culls_bloom_extract_chain` | 关 bloom 编译选项 → `post.bloom-extract` 及其独立上游 culled(对应正文验收) |
| `render_graph_steady_state_second_frame_skips_recompile` | 连续两帧提交,`RenderStats::last_graph_compiled_cache_hits` 第二帧递增、misses 不变(对应"第二帧编译耗时为 0") |

RG-M4(文件:`render_graph/tests` 与 graph_execution 既有测试):

| 测试函数 | 断言 |
|---------|------|
| `render_graph_dump_lists_pass_order_resources_and_culled` | dump 行数 = pass 数,culled 标记与 `CompiledRenderPass::culled` 一致,资源行含 `[first_pass, last_pass]` |
| `render_graph_executed_markers_match_dump_pass_rows` | `executed_debug_markers()` 与 dump 非 culled pass 行一一对应(前缀 `zircon::RenderGraphPass::`) |

render_product 对拍场景(`cargo test -p zircon_runtime render_product --locked` + `ZR_RENDERDOC_CAPTURE_NEXT=1` 人工抓帧):

- `render_product_deferred_post_chain_transient_pool`:deferred + SSAO + bloom + tonemap 场景,池化后帧产物与既有 render_product 基线逐像素一致;同帧 `physical_texture_count < logical_texture_count` 写入断言。
- `render_product_forward_plus_cache_steady_state`:forward+ 场景连续三帧,第 2/3 帧缓存命中且产物与第 1 帧一致(排除缓存引入的拓扑漂移)。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 2026-08-29 Render01 terminal graph ownership source slice（2026-08-30 语义修正）：PrimarySurface 已由不可裁剪 `Present` pass 终结；sRGB Texture target 由只读最终输出的 `output-target-direct-import` pass 终结，不再声明不会执行的 self-copy/write hazard；只有线性 target 才由带精确尺寸/格式/usage 的 `output-target-writeback` pass 执行 sampled-to-attachment conversion，headless 不生成终端 pass。`FINAL_COLOR`/`VIEWPORT_OUTPUT` 外部资源补齐统一 schema，直接导入绑定真实 texture/view/descriptor；输出目标 readiness、graph import、writeback、final selection 与 executor 共享一次 immutable `OutputTargetFramePlan`，terminal packet 无旧式 writeback 与额外 submit。静态检查通过，受管 Cargo 因 `cargo_reuse_target_mismatch` 在启动前阻塞；真实 WGPU、PNG、RenderDoc、性能/功耗和协调器验收仍待完成，不能标记 RG-M4/M5 accepted。
- 2026-08-30 EnvironmentOnly fallback terminal ownership：确认轻量 `EnvironmentOnlyPbrPreview` 不创建 compiled scene graph，因此保留专用直录 recorder；`OutputTargetFramePlan` 一次解析后同时保存 compiled-graph 与 direct-submission 的终端产品，避免 sRGB direct-import 决策让 fallback 错误跳过 copy。其 Texture output transfer 由资源流在同一 frame encoder 中消费 `direct_submission_writeback_plan`；普通 FullScene/StandardPbrPreview 只消费 `compiled_graph_writeback_plan` 并走 Render01 terminal `Present` stage。PrimarySurface 的 direct fallback blit 明确属于 surface lease 的受控边界，未计入 RG-M4 compiled graph 验收。该切片未产生动态 GPU/PNG/RDC 证据，Render01 继续未验收。
- 同切片按模块预算将 terminal schema API 放入 52 行的 `render_feature_pass_descriptor/terminal_schema.rs`；2026-08-30 进一步把 25 个 external-resource builder API 原样迁入 355 行的 `external_resources.rs`，`construct.rs` 从 1280 行降到 936 行。该拆分只调整职责边界，公共 builder 方法名与资源语义不变，解决本次触及文件超过 1000 行的结构约束。
- 2026-08-30 P1-030 batch consumer foundation：`RenderGraphExecutionPacket` 新增不可变 compiled graph-order batch，按有效 `QueueLane`、裁剪间隙和队列切换形成连续 pass range，并在构造时验证所有 live pass 恰好覆盖一次、culled pass 不进入批次。packet 同时一次缓存 `RenderGraphExecutionBatchReport`，并预计算 stage-to-batch index；经 execution record、`RenderStats` 与 `render.graph.execution.batch.*` diagnostics 发布 planned batch/live-pass、队列分布、最大 pass 数和队列切换数。`execute_graph_stage` 与 sprite-stage discovery 现只遍历当前 stage 相关 batches，并以 `RenderPassStage` 过滤服务路由，使图顺序、culling gap、queue boundary 和 exact access identity 来自同一 packet，避免每个 stage 重扫无关 batch。history/readback/writeback epilogue、backend queue/barrier lowering、Cargo/WGPU、PNG/RDC 及性能/功耗证据仍待完成，Render01 不标记 accepted。
- 2026-08-30 batch consumer API closure and epilogue audit：`CompiledRenderPipeline` 已补齐对 packet stage-to-batch index 的只读转发，确保执行消费者不退回 authored pass 列表。复审确认 `copy_history_textures` 仍直接向 frame-owned encoder 编码跨帧纹理复制；在 typed device lease、barrier 和 completion owner 尚未下沉前，不把它伪装成普通 graph pass，避免产生错误的资源生命周期语义。history/readback/writeback graph epilogue 继续保持明确的结构性待办，当前仅计入 source-only foundation。
- 同次复审确认 stage-filtered batch consumer 仍受产品固定阶段顺序约束：若编译图包含跨 stage 交错，frame-local `admit_graph_pass` 会 fail closed，而不会错误重排；因此全局 batch 驱动的统一服务路由仍是 P1-030 未完成项。
- 2026-08-30 immutable stage-order cache：packet 构造期按 compiled graph 顺序缓存首次出现的 `RenderPassStage`，late-stage discovery 直接消费该只读序列，不再为发现 UI/Overlay/Debug 重新扫描 batch；该优化只消除重复扫描，不改变固定阶段执行边界或 admission fail-closed 规则。
- 2026-08-30 deferred lighting typed failure：`DeferredSceneResources::execute_lighting` 对 volumetric 参数和 subsurface MRT 目标的内部不变量改为 `Result` fail-closed，graph executor 传播错误，不再依赖生产 `expect`；有效路径的 bind-group、attachment 数量和绘制布局不变。
- 2026-08-30 clustered-lighting buffer window slice：deferred lighting 的 `LIGHT_GRID_PARAMS`、`LIGHT_ZBINS` 与 `LIGHT_TILE_MASKS` 迁移到 `require_buffer_binding_by_name`，transient 访问消费 exact compiler range，external/persistent 保留明确 full-buffer fallback；lighting bind-group 改用 `wgpu::BufferBinding`，其余非 compute buffer call sites 仍待按资源族分批迁移。
- 2026-08-30 post-process clustered-lighting binding slice：uber post-process 与四个 SSR consumer 的 `LIGHT_LIST` 沿共享 executor/bind-group 调用链传播 `wgpu::BufferBinding`，clustered-lighting producer 的 clear/bind 与 light-grid upload 也按同一窗口执行；transient 访问保留编译器证明的 byte window，external/persistent 继续使用显式 full-buffer compatibility path，probe 与其它 persistent buffers 未被扩大改造；exposure 族随后单独完成窗口化切片。
- 2026-08-30 clustered-lighting continuation：froxel volumetric light-scatter executor 及其两个已有 WGPU product request 也改为携带三条显式 `wgpu::BufferBinding`；light-grid 参数、z-bin 与 tile-mask 的 exact window 现在贯穿 deferred、uber/SSR、cluster producer、upload 和 volumetric scatter，仍保持 full-range test fixture 兼容路径。
- 2026-08-30 forward light-grid consumer slice：mesh recording、OIT fragment store 与 overlay BaseScenePass 的三条 light-grid 参数改用 `optional_buffer_binding_by_name` 和 `Option<wgpu::BufferBinding>`；transient exact byte windows 进入 fragment bind group，缺失/外部资源继续使用显式 full-range fallback。shadow/probe 及其它 full-buffer 族仍未扩大迁移；exposure 族随后单独完成窗口化切片。
- 2026-08-30 OIT transient window slice：OIT fragment-store 的 `layers/counts` 写入、atomic count clear 与 resolve readback 统一消费 `require_buffer_binding` 的 exact `BufferBinding`；容量门禁按实际 binding window 检查，clear 保留 compiler offset/size，避免 transient backing 的未使用尾部被错误纳入执行。
- 2026-08-30 OIT/forward error-terminal follow-up：OIT fragment-store/resolve 的管线初始化缺失与 BaseScene forward receiver 绑定缺失改为现有执行边界的 fail-closed 路径，不再依赖生产 `pipeline.as_ref().unwrap()` 或 generic receiver `expect`；正常 bind/draw ABI 不变。
- 2026-08-30 SSS buffer-window slice：subsurface setup/scatter 的 tile-list、indirect-args、profiles 与 params 统一消费 exact `wgpu::BufferBinding`；setup clear 保留 compiler `offset/size`，scatter indirect dispatch 使用 `buffer + offset`，所有 bind-group 不再隐式扩大为 `as_entire_binding()`。外部/persistent 兼容路径仍由 resolver 的 full-range 合同负责，未扩大到其它 buffer 族。
- 2026-08-30 compiled execution cursor wiring：`RenderGraphExecutionPacket` 的 immutable `begin/admit/finish` cursor 现接入 `RenderGraphStageExecution`；每个 live compiled pass 在阶段准备时必须按图序一次通过 admission，帧尾再由同一 cursor 完成性校验。固定 stage service routing 仍保持，但交错 graph 会在执行入口 typed fail-closed，不再只是 coverage 去重。
- 2026-08-30 compiled batch ownership index：`RenderGraphExecutionPacket` 在构造期为每个 live compiled pass 缓存 O(1) 的 `graph_pass -> batch` 反向索引，culled pass 明确为 `None`；stage consumer 直接携带全局 batch index，admission 会校验 pass 未跨 batch 路由。该切片只强化 immutable batch ownership 与 queue-boundary 契约，不改变执行顺序，也不代表 history/readback/barrier、typed external lease 或动态 WGPU/截图验收已完成。
- 2026-08-30 external access lease packet foundation：`CompiledRenderGraph` 为每个 live external access 生成 immutable `{access_id, versioned_key, external_binding, typed_desc}` packet；frame resource table 在所有 frame/plugin physical bindings 完成后按该 packet 建立 access-ID lease。generic-compute 与 resolver-backed non-compute consumer 的 external texture、external buffer 和 indirect dispatch 已切换为 exact access-ID 物理查询，schema buffer window 必须与 lease window 完全一致；report-only view-only external 仍允许 descriptor-less 兼容，但需要 pipeline descriptor 时会 typed fail-closed。仍有少量绕过 GPU resolver 的旧 non-compute helper 待迁移，且尚未取得动态 WGPU/PNG/RDC/性能证据。
- 2026-08-30 IBL graph output identity/window slice：环境 IBL bake 的 graph-backed PMREM/IEM texture readback 现通过每个 live `Write` access ID 取得 compiler physical allocation，PMREM 的多个 mip writer 必须归属同一 allocation；transient access table 每个 physical allocation 只保留一份 full WGPU texture handle。irradiance SH9 storage-buffer 输出与后续 readback 改用 exact compiler window；编码路径通过 `RenderPassGpuExecutionContext::require_buffer_binding(...)`/`StorageBufferRange` 保留 offset/size，staging copy 与 product diagnostic admission 继续消费同一 range，避免非零 transient window 从 offset 0 读取；backend 在注册读取前校验 descriptor byte length、`offset + size` 溢出和物理 buffer 越界，并新增 `64..64+SH9_SIZE` typed external fixture。直接 environment-capture target 仍明确使用 full-buffer compatibility variant。该切片收口 IBL 绕过 GPU resolver 的 texture/buffer readback 旁路，不改变 shader/layout 或 direct capture ownership，受管 Cargo/WGPU、PNG/RDC、性能/功耗与协调器验收仍待完成。
- 2026-08-30 P1-030/P1-031 history epilogue identity slice：`CompiledRenderPipeline` 现冻结 canonical GI、SSR、HZB 与 volumetric 输出的最终 live writer access ID、texture descriptor 和 `COPY_SRC` 合同；`copy_history_textures` 不再按 graph resource name 查找这些 WGPU texture，统一通过 exact graph-owned access lease 解析，lease 缺失在提交前 typed fail-closed。为覆盖 graph-owned persistent HZB/插件 GI，frame resource table 新增独立 persistent texture access-ID lease，每个逻辑 persistent resource 只保留一份 WGPU texture handle，并与 transient/external lease 一起在 backing 回池或 retirement 前清空。history copy 仍由现有 frame serial encoder 和 history transaction owner 执行，未伪装成普通 graph pass；queue/barrier/completion owner 下沉、受管 Cargo/WGPU、PNG/RDC、性能/功耗和协调器验收仍待完成，Render01 不标记 accepted。
- 2026-08-30 advanced-lighting error-terminal follow-up：Froxel media-inject/light-scatter/integrate、planar-filter 与 subsurface pipeline cache 在初始化后不再通过生产 `as_ref().unwrap()` 进入编码；缺失缓存统一返回带 executor/pass 身份的 typed `Result`，光散射固定三维 dispatch 也删除无必要的切片 `unwrap`。这是源码错误终态收敛，受管 Cargo/WGPU、PNG/RDC 与性能/功耗证据仍待完成。
- 2026-08-30 deferred ordering correction：旧公共 early list 会在 deferred GBuffer 前执行 `AmbientOcclusion`，与图声明的资源生产顺序冲突；现调整为 common early 只含 `DepthPrepass/Shadow`，forward 维持 `AmbientOcclusion -> Lighting -> scene`，deferred 改为 `Deferred -> AlphaMask3d -> Opaque2d -> AmbientOcclusion -> Lighting`。该修正通过源码顺序合同测试，尚未取得受管 WGPU、PNG/RDC 或性能证据。
- 2026-08-30 compiled execution coverage guard：阶段准备按 compiled pass index 维护 frame-local live-pass admission，重复、culled 或越界执行 fail closed，compiled scene 尾部要求所有 live pass 恰好一次；deferred 路径补齐 `AlphaMask3d`，阶段顺序固定为 `Deferred -> AlphaMask3d -> Opaque2d -> AmbientOcclusion -> Lighting`。这是 source-only correctness foundation，受管 Cargo/WGPU、截图/RenderDoc 与性能证据仍待完成。

- 迁入记录：[`01/2026-07-09-render-graph-rdg-alignment-output-records.md`](01/2026-07-09-render-graph-rdg-alignment-output-records.md)
- fixed 已修复：[render-framework-pipeline-registration-test-double-migration](../../zircon_editor/editor/09/fixed-2026-07-13-render-framework-pipeline-registration-test-double-migration.md)
- fixed 已修复：[editor-viewport-resolve-job-guard-drift](../../zircon_editor/editor/09/fixed-2026-07-14-editor-viewport-resolve-job-guard-drift.md)
- 2026-08-15 Render01 状态：进行中，未验收。已完成静态实现：graph dump 为 transient allocation 输出完整 bucket identity；WGPU framework 新增从 compiled frame retained `scene_color` 读取线性 RGBA16F 的 HDR capture contract、native binding 和 focused regression；view-family 已收敛复用 camera viewport rect，facade 不再重复导出同名类型。待完成：受管 `zircon_runtime` 编译、GPU 真实运行与 PNG 证据、RenderDoc 对拍、协调器验收提交和量化消息；在这些完成前 RG-M4 不标记完成。
- 2026-08-16 Render01 M4 诊断切片：`RenderGraphDump` 现在仅在 debug dump 构造时以一次 `O(P+E)` 依赖投影输出每个非 culled pass 的 ready layer、全图 `topology_layers` 与 `topology_peak_width`；queue 信息仍在同一 pass 行，因此 RenderDoc marker 对拍可区分“可并行记录”与实际 queue fallback。新增回归覆盖两个独立 producer 的同层宽度、join 的下一层和 culled pass 不计入宽度。`rustfmt --check`、scoped `git diff --check` 与结构守卫通过；受管 Rust unit-test 通道目前因 368 个外部 `cfg(test)` 诊断而未执行该断言，尚无 GPU/RenderDoc/性能结论。
- 2026-08-16 Render01 M4 capture-consumer 回归：dump pass 行加入 `layer=` 后，`render_framework_graph_stats` 的 capture 对拍解析器会把该字段并入 pass 名，进而使 marker/profile 名称比较失真。已改为显式跳过 `layer=` 并新增含 live 与 culled 行的纯解析回归；`rustfmt --check` 与 scoped `git diff --check` 通过。该回归尚未进入受管 Rust test 执行，不能代替 RenderDoc 或产品捕获验收。
- 2026-08-27 Render01 / Runtime90 产品帧 owner 交界：PFO-0 第一身份切片已完成源码实施。frame-begin poll、compiled/direct scene submit与实际surface blit submit不再丢弃receipt/ticket；renderer用`RenderFrameSubmissionReceipt`校验同device generation与present sequence并发布到`RenderStats`。权威硬切设计和剩余库存记录在[`../../optimize/zircon_runtime/90/2026-08-27-neutral-product-frame-owner-hard-cut-design.md`](../../optimize/zircon_runtime/90/2026-08-27-neutral-product-frame-owner-hard-cut-design.md)。Render01仍只负责后续把surface blit/copy编为terminal graph pass并输出device-qualified packet，不接管device、queue、completion或fault owner。当前状态为`product_frame_identity_slice_source_implemented_static_checks_passed_dynamic_validation_pending`；稳定present仍为scene+blit两次提交，未新增PNG、RDC或性能结论。
- 2026-08-27 Render01 / Runtime90 PFO-1a：completion pump与frame receipt finalize已从direct/compiled core上移到`SceneRenderer`帧入口。outer owner按`poll -> resource/history prepare -> core scene ticket -> receipt finalize`排序，core不再拥有device poll。当前状态为`frame_boundary_completion_owner_source_implemented_static_checks_passed_dynamic_validation_pending`；producer ledger、失败帧cancel、terminal surface graph pass和device-qualified graph packet仍未完成。
- 2026-08-27 Render01 / Runtime90 PFO-1b 第一部分：frame transaction现在可记录有序pre-scene producer，history初始化真实ticket已进入成功frame receipt；非空记录以共享slice保留，steady无producer路径不创建记录数组。当前状态为`history_producer_identity_source_implemented_static_checks_passed_cancel_and_texture_producers_pending`；Render01后续terminal graph packet必须消费该ledger，而不是新增第二套ticket表。
- 2026-08-27 Render01 / Runtime90 PFO-1b 故障终态观察：coordinator的ticket status查询会先将fault gate同步为submission history终态，再返回原ticket的`DeviceLost`或`Failed`，不再因admission失败遮蔽ticket状态；submit、enqueue和poll仍fail closed。故障映射已由旧产品coordinator与neutral device共享唯一策略。PFO-0、PFO-1a和当前PFO-1b部分合计新增21个源内合同测试条目；transaction cancel、failure receipt、texture producers、Cargo/GPU/RenderDoc/PNG仍待完成。
- 2026-08-27 Render01 / Runtime90 PFO-1b pre-scene失败收敛：submission service现以一次queue/state锁、`O(P + T)`批量settle abandoned-frame tickets；Accepted转Cancelled，Submitted/terminal保持可观察。新的failure receipt拒绝残留Accepted，compiled history之后的prepare/core错误会携带原错误与settled producer identity，无producer错误不新增包装或分配。当前PFO合同测试条目合计27个；texture producer、scene-submit后错误身份和动态验收仍待完成。
- 2026-08-29 Render01 F4 提交失败闭环：advanced runtime plan 的 move-once 消费不再 `expect`，重复消费返回 typed `InvalidSubmissionState`；SceneLinear phase 缺失不再越过 frame transaction 直接 panic，而是在 resource producer 准备之后先调用统一失败结算，再以 typed `MissingViewFamilyPhase` 返回。framework boundary 保留该直接错误的 typed identity。两个 render-submit 生产目录与完整 graph execution 生产候选的 scoped `panic!`/`expect`/`unwrap` 扫描均为 0：preview-sky context 改为 fallible lookup；pending transient allocation 无 ticket 时 fail closed 丢弃且不查询 completion；pool byte diagnostics 饱和到 `u64`；generic-compute range 与 texture-view format 采用单次模式匹配。compiled scene resource binding 的 live scene-velocity backing 缺失改为 typed `MissingFrameGraphResourceBacking`，prepared GPU Scene upload 缺失改为 typed `MissingPreparedGpuSceneUpload`，并在后一错误返回前终止 pending realtime-IBL submission；framework boundary 保留两者 identity，compiled/direct scene core 的 production-only 候选扫描均为 0。随后复核整个 `scene_renderer/core`，将 hit-proxy readback 聚合、cubemap upload target 索引和 scene-uniform packed upload 的 7 个生产 `expect` 全部改为 `let-else`、已有 `MissingTarget` 或 `InvalidBufferUploadRange` 错误，目录 production-only 候选扫描仍为 0。focused source regression、精确格式、diff 与 metadata 静态检查通过；受管请求 `6d6bb68074fd4e3e8d6abaa787698b96` 因 `cargo.acquire` 超时未进入编译，当前状态为 `render_submit_typed_failure_source_implemented_dynamic_validation_pending`。该切片只处理错误终态，不改变 transient 淘汰算法，也不形成 WGPU、截图、RenderDoc、性能或功耗结论。
- 2026-08-29 Render01 F4 packet-lowering 收敛：`render_graph` 的访问索引缺失返回 `CompiledAccessIndexEntryMissing`，dispatch 声明缺失返回 `ResourceDeclarationMissing`，空 binding 候选返回既有 `ComputeBindingAccessMissing`；生产 `panic!`/`expect`/`unwrap` 扫描保持 0。该切片仅处理编译期错误终态，不改变访问候选选择、范围匹配或 dispatch 计算算法。
- 2026-08-29 Render01 F4 surface-format 收敛：`ViewportSurface` 的 BGRA/RGBA blit pipeline 格式转换接入 `create_viewport_surface -> Result`，固定格式契约失败返回 `GraphicsError::SurfaceStatus`，不再在生产构造器中 `expect`；blit shader、pipeline 选择和 surface transaction 顺序不变。
- 2026-08-29 Render01 F4 diagnostic decoder 收敛：RGBA32F product diagnostic 的定长 lane 转换改为 `chunks_exact(4)`，保留严格 16-byte 输入校验和 little-endian 结果，不再依赖生产 `expect`。
- 2026-08-29 Render01 F4 descriptor catalog 收敛：内建 texture allocation extent 的缺失 phase/target 改为 `Option` fail-closed，由既有 `missing_schema_error` `Result<String>` 边界报告；有效 pipeline 的尺寸/fallback 顺序不变。
- 2026-08-30 Render01 P1-031 exact resolver continuation：生产 `RenderPassGpuExecutionContext` 与 `RgResourceResolver` 对已有 compiler physical allocation 的 transient texture/buffer access，以及 typed external texture/buffer access，优先消费 exact access-ID WGPU view/buffer window；persistent/unknown report-only external 继续走明确的 declaration/name compatibility 路径。该切片通过 rustfmt、metadata、diff 与源级契约审计；仍待真正 non-compute 直接 helper 的彻底收口、受管 Cargo、WGPU、PNG/RDC、性能/功耗与协调器验收，Render01 不标记 accepted。
- 2026-08-30 Render01 non-compute buffer slice audit：静态盘点 15 个 executor 调用点仍使用 legacy `&wgpu::Buffer` 入口，资源族同时包含 full-buffer uniform/storage、indirect 参数、clustered-lighting 与 post-process。该问题需要先按资源族拆分 exact slice/full-range 合同并完成 typed external lease，再迁移调用面；本次仅记录结构性风险和实施顺序，不作性能或功耗结论。
- 2026-08-30 Render01 exposure buffer-window slice：exposure histogram/resolve 以及 color-LUT bake 的 graph buffer 入口现统一消费 `wgpu::BufferBinding`；transient histogram/exposure 访问保留编译器证明的 offset/size，histogram clear 也只清理该窗口，持久 fallback 显式构造 full-range binding。该切片未改变 shader/layout 或资源族生命周期，受管 Cargo/WGPU、PNG/RDC、性能/功耗与协调器验收仍待完成。
- 2026-08-30 Render01 Hybrid-GI transient handoff slice：`hybrid-gi-scene`/`hybrid-gi-trace` 三个插件 handoff executor 改用 `require_buffer_binding`，scene-depth packet 写入验证 compiler window 的相对 offset/size，trace schedule 与 resolve bind group 直接消费 exact `BufferBinding`，不再以 `as_entire_binding()` 扩大 transient backing。插件 ABI 与 shader binding index 不变；受管 plugin Cargo/WGPU、PNG/RDC、性能/功耗与协调器验收仍待完成。
- 2026-08-30 Render01 history direct-write receipt slice：TAA scene-color 与 exposure current history 属于 graph external destination，不执行 epilogue copy；它们现在只在对应 TAA render encoding / exposure compute dispatch 成功记录后生成 `SceneHistoryWriteIntent`。每个 `RecordedGraphPass` 将回执带回主执行 owner，串行与并行 pass 在 `RenderGraphStageExecution` 合并；history transaction 与 `RenderHistoryCopyReport` 不再因 compiled graph 仅声明 writer 就无条件标记成功。提交后才持久化 history domain 的既有边界不变；受管 Cargo/WGPU、PNG/RDC、性能/功耗和协调器验收仍 pending，Render01 不标记 accepted。
- 2026-08-30 Render01 history producer-receipt continuation：SSR resolve、HZB build 与 volumetric light-scatter 只在对应 render/compute 编码成功后发布 history write receipt；generic compute 在既有 storage-write 元数据单次扫描中识别 SSAO output，Hybrid GI 插件 resolve 在 lighting/temporal-metadata 双目标编码成功后通过公共 `record_frame_history_write(FrameHistorySlot)` 发布 GI 回执。epilogue 在复制 GI/AO/SSR/HZB/volumetric history 前必须看到该 domain 的本帧回执，因此 compiled writer declaration 或残留物理 backing 不再足以验证跨帧历史。queue/barrier/completion 下沉和全部动态证据继续 pending。
- 2026-08-30 workload audit descriptor closure：逐 pass compute workload audit 的 froxel grid 不再按 `VOLUMETRIC_SCATTERING` 名称查询 frame resource table，改为读取 `CompiledHistoryEpiloguePlan` 中与最终 writer access ID 同时冻结的 descriptor；`execute_graph_stage` 生产路径的直接 `owned_texture_desc` 旁路归零。同步补齐 `RecordedGraphPass` 测试夹具缺失的 texture upload、UI commit、writeback report 与 history receipt 字段。该切片仍只做身份/结构正确性，不声明 profile 收益。

### 2026-08-15 P1 产品测量边界复核

- 已完成：Render17 P1 基线采集/报告合同升级到 schema 4；每次 steady run 固定丢弃 60 个 `app/runtime_redraw` warmup frame，随后只统计连续 300 个同名 primary frame。span/counter 仅计入该时间窗口，避免 nested runtime frame、预热、进程启动和退出时间污染 p50/p95。相关 Pester 静态回归为 52/52；这只证明采集合同，不能替代实测。
- 已完成：UI12 复用了其受管 current-source `zircon_runtime -SkipTest` 验证，构建 exit 0（dev profile 11m37s，wrapper 801.6s）。该结果关闭了 ViewportSurface 编译阻塞，但不等同于 profiling-product 或 GPU 性能通过。
- 结构审查：`CapturedFrame` 已携带与 RGBA 同 generation 的 `graph_dump` 和 `frame_profile_json`，但 `zircon_app` 的显式产品导出仍只写 PNG。因而当前基线可关联 timeline/PNG，却不能把 PNG 与其 compiled graph/cache state 作为一个不可分割产物。该导出路径由 Editor16 租约持有，后续应以可选 sidecar 路径为契约一次性导出 PNG、graph dump 与 frame profile，保持默认产品路径零额外格式化/IO。
- 未完成且不得提前宣称：需要同一 source fingerprint 的 profiling runtime/editor artifact、每场景 3 次 steady capture、同帧 PNG + RenderDoc RDC + graph/profile sidecar；随后才记录 RenderGraph compile/cache、GPU frame、CPU render/RHI、RSS/VRAM 的 p50/p95，并只在内容、分辨率、AA、驱动和 warmup 一致时对照 Unreal。没有这些产物，不提交性能改善或功耗结论。

### 2026-08-15 P0 pass authoring 依赖 DAG 收敛

- 已完成（源实现，待受管验证）：`pass_authoring.rs` 已删除跨全部 stage/pass 的全局 `previous -> pass` 链，并且不再重复维护资源依赖事实。authoring 只声明资源访问并以 `(name, kind)` 做 unique-producer 前置排序；`RenderGraphBuilder::compile` 是唯一的 RAW/WAW/WAR history 与自环过滤所有者。无共享资源的 pass 不再因作者顺序产生依赖。
- 已完成（源实现，待受管验证）：同一合同已下沉至 `render_graph/builder/compile.rs`。编译从手工 WAW writer-pair/bitset closure 与仅 RAW 推导，收敛为以 manual 边为种子的去重 adjacency 和每资源访问历史；最终 RAW/WAW/WAR 边也是 culling 的唯一依赖回溯来源，防止仅因 WAR 保留的 reader 被错误裁剪。该模型对应 UE RDG 按 pass 参数枚举 texture/buffer access 的依赖事实，而不是按 authoring 顺序强行串行化。
- 已完成（源实现，待受管验证）：同阶段的 unique-producer 前置不再为每个 Write 重扫全部 pass/资源。现在一次建立 resource -> reader/writer index，再以稳定拓扑排序生成边，复杂度从嵌套全扫描收敛为一次访问索引加实际 edge 数；该步骤仍只负责 producer-before-reader 的声明顺序，资源危害边由统一访问历史负责。
- 已完成（focused regression 源码）：新增 `compile_keeps_independent_resource_producers_unordered_until_their_consumer`，要求两个独立 producer 均无人工 dependency，只有 join 依赖二者；新增 `compile_preserves_read_write_hazards_for_reused_resources`，要求 Write -> Read -> Write 保留 RAW/WAR/WAW 所需边；新增 `compile_rejects_side_effect_passes_without_declared_resources`，禁止未来将未声明的副作用重新隐藏到 authoring 顺序。静态 `rustfmt --check` 与 `git diff --check` 已通过，旧链模式匹配为 0。
- 已完成（focused regression 源码）：`render_graph/tests/resources.rs` 以 `seed -> sample -> overwrite -> present` 覆盖 RAW、WAW、WAR 与 culling 保活；`render_graph/tests/culling.rs` 约束编译期只保留一份去重 adjacency、reader history 和最终依赖图回溯。新增 `graph_rejects_attachment_load_after_discarded_transient_store` 锁定 `clear_discard -> load_store` 必须报 `ReadAfterDiscardedStore`，防止 attachment `Load` 被误当成纯写入。最终源码快照的两轮独立静态审查均为 critical 0、important 0、minor 0；`rustfmt --check`、`git diff --check` 和资源危害结构守卫通过。
- 已记录但不替代 Rust 验收：协调器 actions 验证 `40062d1acf3e4e4087bcf6784e575a9e` 已接受（44/44 coordinator workflow tests）；其执行面不包含 RenderGraph Rust test，且随后新增上述 discard-to-Load regression，故该回执不覆盖当前快照。
- 未完成且不得提前宣称：需在当前协调器 Cargo 通道释放后，以 source-matched 受管 `zircon_runtime` focused test 运行以上两条回归及默认 pipeline 编译；随后以 graph dump 的 ready-width、async-lane 可调度层和 GPU/CPU profile 量化吞吐改善。当前没有性能、功耗或并行执行收益数据，P0/RG-M4 仍为进行中。

### 2026-08-15 M4 schema/instance 切分前置审查

- 事实：当前 `CompiledGraphCacheKey` 把 `view_width/view_height/render_width/render_height` 与特性、quality、capability 放进同一 fingerprint；这不是可直接删除的冗余。`texture_desc_for` 用其决定 render/view/half-resolution 尺寸、HZB 与 SSR mip 链、froxel 3D 尺寸和 sample count，`buffer_desc_for` 用其决定 OIT 与像素相关 buffer 字节数。因此 resize 继续沿用 concrete `CompiledRenderGraph` 会复用错误的 WGPU 资源描述符，属于正确性错误而非缓存策略选择。
- 2026-08-15 静态架构审查：`CompiledGraphCache` 的 value 是 `Arc<CompiledRenderPipeline>`，而后者直接持有 concrete `CompiledRenderGraph`；现有 `compiled_render_pipeline_cache_misses_on_viewport_resize` 断言两次 miss/两份 entry，正是上述 descriptor 合同的回归保护。UE `FRDGBuilder::Compile()` 也在 dependency/culling 确定后才进入 execution；Zircon 的正确优化方向是复用无尺寸的 topology/schema，而不是让 concrete graph 越过 extent 生命周期。
- 后续结构：先发布只含 feature/capability/format/sample-class/pass-resource declaration 的 `PipelineGraphSchema`，再由 `(schema, extent, target binding, dynamic constants)` 生成持有 concrete texture/buffer descriptors 的 `RenderGraphInstance`；外层 schema cache 与内层 instance cache 的 key 分离，但 execution 只接收 immutable instance。不得通过删除现有尺寸字段、放宽全 graph equality 或让 WGPU materializer 在执行期猜测 extent 来制造 cache hit。
- 受管回归与量化计划：同一默认 pipeline 在 `64x64` 与 `128x64` 必须断言 schema identity 相同、instance identity 不同，且 HZB mip 数、half-resolution extent、OIT/pixel buffer bytes 均按新尺寸重算；相同 extent 重复三次才可断言 instance cache hit。P0 resource-DAG dynamic test 通过后，再采集 schema compile、instance materialization、CPU render/RHI 和 GPU frame 的 p50/p95；未获得 source-matched 产品 trace 前，不报告命中率、功耗或性能改善。

### 2026-08-30 persistent texture exact-view source slice

- 已完成（源实现，待受管验证）：`CompiledRenderGraph::persistent_texture_backing_resource(...)` 统一把 graph-owned texture-view alias 归一到持有 persistent lifetime/storage 的 parent。access scope 继续使用 compiler 已投影的 parent subresource range；执行期不重新组合 mip/layer，也不按逻辑名猜 backing。
- 已完成（源实现，待受管验证）：`persistent_texture_access_bindings.rs` 现在按 access ID 保存 exact `TextureView` lease，按 backing resource 保留唯一 `Texture` handle，并按 `(backing, range)` 复用创建过的 view。standard non-compute texture resolver/helper 先走 transient-or-persistent graph-owned exact lease；buffer helper 保持独立 exact byte-window 合同。
- 已完成（回归源码）：direct persistent mip materialization 和 persistent-parent alias resolver 用例覆盖 access/view/backing 计数、alias parent identity 与 pass-scoped exact lookup。精确 rustfmt、locked Cargo metadata、scoped diff/source-contract 和 texture/buffer helper 分类检查通过。状态为 `render_plan01_persistent_texture_exact_view_source_implemented_dynamic_validation_pending`。
- 未完成：其它 provider-owned/persistent buffer family 的 typed lease 收敛、sparse/provider-owned texture 的独立 typed lease、full-chain/selected-mip compatibility helper 的最终 access-scoped view packet、受管 Cargo/WGPU、真实 framebuffer PNG、RenderDoc RDC、固定硬件 CPU/GPU/VRAM/功耗量化与协调器验收。当前切片不关闭 RG-M2/RG-M4 或完整 Render01。

### 2026-08-30 persistent exposure external-buffer exact-lease source slice

- 已完成（源实现，待受管验证）：确认 exposure history 是 renderer-owned imported buffer，不新增 graph-owned persistent-buffer truth。`EXPOSURE_PREVIOUS/CURRENT` 现在共享精确 16-byte `RenderResourceSchema::Buffer`，usage 与物理双缓冲保持 `STORAGE | COPY_SRC | COPY_DST` 一致。
- 已完成（源实现，待受管验证）：builder 增加 versioned scoped external read/write/load API；pipeline asset authoring 删除已经过时的“等待 typed external lease packet”拒绝分支，并把 descriptor 的 range/intent 下沉到既有 access-ID external lease packet。`exposure-resolve` 使用 compute read/read-write，`scene-composite`、`color-lut-bake`、`uber` 使用真实 fragment/compute read intent，编译后 full range 归一为 `0..16`。
- 已完成（回归源码）：builder provenance/range、feature descriptor schema/access 和启用 color-LUT 后的五个 exposure access 都有源码回归；`construct.rs` 测试迁到 `external_resources.rs` owner，使被触及模块保持 1000 行以内。精确 rustfmt、locked Cargo metadata、scoped diff/source-contract 检查通过。状态为 `render_plan01_persistent_exposure_external_buffer_lease_source_implemented_dynamic_validation_pending`。
- 未完成：受管 Cargo/WGPU test、真实 framebuffer PNG、RenderDoc RDC、固定硬件帧时/显存/功耗、其它 provider-owned persistent buffer family，以及协调器 acceptance。当前没有动态渲染或性能改善结论。

### 2026-08-30 provider-owned external-texture exact-view source slice

- 已完成（源实现，待受管验证）：`CompiledRenderGraphExternalAccessPacket` 的 `Texture(range)` 不再只停留在 metadata。external access materializer 现在从 provider 发布的 physical `wgpu::Texture` backing 创建 access-scoped `TextureView`，并按 `(graph resource, compiler-canonical range)` 复用相同 scope 的 view；每个 live access ID 仍保留独立 lease identity。
- 已完成（源实现，待受管验证）：view-only compatibility 被收紧为可证明覆盖完整 mip/layer/aspect 的 scope。部分 mip/layer/plane access 若没有 physical texture backing 会在编码前 fail closed，`UnresolvedExternal` 旧资源仍明确复用 producer 默认 view；texture access 携带 buffer scope 同样立即拒绝。
- 已完成（回归源码）：新增 backing + mip2 exact lease、view-only partial scope 拒绝和 view-only canonical full scope 兼容用例；共享 texture-view descriptor validation 的 owner 名称已去除仅限 owned transient 的错误语义。精确 rustfmt、locked Cargo metadata、scoped source-contract/diff 检查完成后状态为 `render_plan01_provider_external_texture_exact_view_source_implemented_dynamic_validation_pending`。
- 未完成：受管 Cargo/WGPU test、真实 framebuffer PNG、RenderDoc RDC、固定硬件帧时/显存/功耗、view-only partial producer 的 backing/access-view 发布协议、sparse residency lease，以及协调器 acceptance。当前切片不关闭 RG-M2/RG-M4 或完整 Render01。

### 2026-08-30 TAA external-texture exact-lease source slice

- 已完成（源实现，待受管验证）：TAA previous/current history 不再只声明 report-only external texture。时序 feature descriptor 为两槽发布 View-sized `Rgba16Float`、单 mip、`SAMPLED | RENDER_ATTACHMENT` schema，并把 previous 固定为 fragment sampled full-texture read、current 固定为 color-attachment full-texture write。
- 已完成（源实现，待受管验证）：`TemporalHistoryStore` 与 `SceneFrameHistoryTextures` 公开同一 ping-pong owner 的 borrowed texture/view/descriptor；compiled-scene binder 通过 `import_borrowed_texture_with_identity(...)` 一次发布 physical texture、default view、descriptor 和稳定 identity。执行资源物化因此从上一切片的 external access packet 创建每个 access-ID 的精确 view，而不是按资源名克隆默认 view。
- 已完成（回归源码）：builder typed access 保留、TAA descriptor contract、compiled external packet range/intent 和 live binder physical backing/descriptor 均有源码回归。精确 rustfmt、locked Cargo metadata、scoped source-contract/diff 检查通过。状态为 `render_plan01_taa_external_texture_exact_lease_source_implemented_dynamic_validation_pending`。
- 未完成：受管 Cargo/WGPU test、真实 TAA/AO 连续帧 framebuffer PNG、RenderDoc RDC、history valid/reset 300 帧采样、固定硬件 CPU/GPU/VRAM/功耗，以及协调器 acceptance。SSAO previous 已由 SceneLinear `render_size` exact lease 独立收敛；动态 mip HZB 和动态 3D volumetric history 已完成 catalog-backed source slice，但各自动态验证仍 pending，不能套用本切片的固定 View schema。

### 2026-08-30 Hybrid GI and SSR exact-history source slice

- 已完成（源实现，待受管验证）：Hybrid GI lighting/temporal-metadata 两张 previous history 由插件声明 View-sized、单 mip `Rgba16Float` schema 与 fragment sampled full-texture access；renderer history owner 发布 backing texture/view/descriptor，compiled-scene binder 通过通用 physical-texture import 进入 external access materializer。
- 已完成（源实现，待受管验证）：SSR shader 既有 temporal reprojection 原先绕过图，resolve pass 未声明 previous history、GPU executor 直接读取 `SceneFrameHistoryTextures`。现在 resolve descriptor 声明同类 exact external read，执行器通过 pass-scoped resolver 取得 optional access-ID view；cold-start 未绑定时继续使用既有 fallback 且 `history_available=false` 禁止混合。shared bind-group 中与 SSR 辅助 pass 无关的 GI/SSR history 槽不再隐式读取 owner。
- 已完成（源实现，待受管验证）：uber 的 binding 9 仍保持“当前 Hybrid GI 输出优先、previous GI fallback”语义，但 previous fallback 改为声明同一 exact external access 并经 resolver 获取；root/SSR graph executor 中 `history.map(...global_illumination/screen_space_reflection...)` 已清零。
- 已完成（回归源码）：Hybrid GI 插件 descriptor、两张物理 binding、SSR/uber compiled external packet、resolver-only 执行路径与物理 binding 均有源码回归。精确 rustfmt、locked Cargo metadata、scoped source-contract/diff 检查通过。状态为 `render_plan01_hybrid_gi_ssr_exact_history_source_implemented_dynamic_validation_pending`。
- 未完成：受管 Cargo/WGPU、连续帧 GI/SSR PNG、RenderDoc RDC、300 帧 domain validity、GPU timestamp/VRAM/功耗和协调器 acceptance。SSR filter/denoiser 算法与 UE 独立 TAA/denoiser 分支没有在本切片重写；没有 profile 前不作算法性能结论。

### 2026-08-30 HZB dynamic external-history exact-lease source slice

- 已完成（架构复核）：上一帧 HZB 不能复用固定 View/单 mip schema。旧 feature 只声明 report-only external view，pipeline external authoring 不调用资源目录；而目录若按名称直接解析 previous HZB，会落入普通 2D/sRGB/单 mip 分支。运行时历史创建与 graph 编译均已确认从 `ViewFamilyPipeline::SceneLinear` allocation 进入同一 `HzbBuilder`，因此动态尺寸和 full mip chain 应由目录统一拥有。
- 已完成（源实现，待受管验证）：新增 catalog-backed persistent external texture access API，feature 只声明 compute sampled/full-texture intent，不复制动态 descriptor。external authoring 现在可从 `RenderResourceSchemaCatalog` 获取内建外部纹理描述；current/previous HZB 共享一个 `HzbBuilder` descriptor helper，current usage 收敛为 `SAMPLED | STORAGE | COPY_SRC`，previous graph contract 收敛为 `SAMPLED` 且固定单采样。
- 已完成（源实现，待受管验证）：`SceneFrameHistoryTextures` 发布 previous HZB 的 borrowed texture、view 与实际 `Rgba16Float`/full-mip/`SAMPLED | COPY_DST` descriptor；compiled-scene binder 改用 physical-texture import。执行器继续只经 pass-scoped resolver 读取 optional history view，首帧 white fallback 与 history-availability gate 不变。
- 已完成（回归源码）：覆盖无 schema 的 catalog-backed exact access、1923x1081 输入对应 1024x1024/11 mip 的 current/previous 几何一致性、compiled external packet 的 canonical full-mip range/compute intent，以及 live binder 的 16x16 render input 对应 8x8/4 mip 物理历史。精确 rustfmt、locked Cargo metadata、scoped source-contract/diff 检查通过。状态为 `render_plan01_hzb_dynamic_external_history_exact_lease_source_implemented_dynamic_validation_pending`。
- 已完成（结构预算）：新增 HZB 合同后 `compile_tests/core_contracts.rs` 达到 892 行并违反 Runtime15 的 `<800` owner 守卫；exposure/TAA/SSR/HZB/Hybrid GI 五组 external exact-lease tests 已整体迁入 `compile_tests/core_contracts/external_history_leases.rs`。父/子分别为 557/337 行，守卫锁定子模块挂载、测试归属与预算，不以扩大单文件承载新合同。
- 未完成：受管 Cargo/WGPU test、连续帧 HZB/遮挡产品 PNG、RenderDoc mip/resource/barrier RDC、墙场景误剔对拍、300 帧 CPU/GPU/VRAM/功耗量化和协调器 acceptance。未取得 profile 前不声明 cull 算法或功耗改善，也不关闭 RG-M2/RG-M4、VC-M2/VC-M3 或完整 Render01。

### 2026-08-30 Volumetric dynamic 3D external-history exact-lease source slice

- 已完成（架构复核）：volumetric light-scatter executor 已经通过 pass-scoped resolver 读取 previous history，但插件仍只声明无 scope/intent 的 report-only external，history binder 也只发布 cloned view，未发布 backing texture 与 descriptor。该历史不是固定 View 2D 资源，而是由 shader quality 决定 `160x90x48/64/96` 的 `Rgba16Float` D3 texture；因此不能复制 TAA/GI schema。UE 5.5.4 的 `VolumetricFog.cpp` 同样把有效 previous `LightScatteringHistory` 注册为 RDG external texture，并对本帧 scattering 结果排队 extraction。
- 已完成（源实现，待受管验证）：volumetric 插件改为声明 compute sampled/full-texture exact access 且不复制动态 schema；built-in catalog 的 current/previous volumetric descriptor 共用唯一 froxel geometry helper。`VOLUMETRIC_SCATTERING` usage 收敛为 `SAMPLED | STORAGE | COPY_SRC`，previous graph contract 为 `SAMPLED`；history provider 发布 `SAMPLED | COPY_DST` 的 physical D3 texture/view/descriptor，binder 使用 borrowed physical import，executor 的 media fallback、temporal availability gate 和 reprojection逻辑不变。
- 已完成（回归源码）：覆盖 feature exact metadata、High 档 current/previous `160x90x96` D3 descriptor 一致性、compiled external packet 的 canonical `mip=1/layer=1` compute sampled lease，以及 live binder 的 physical texture/descriptor。精确 rustfmt、locked Cargo metadata、scoped source-contract/diff 检查通过。状态为 `render_plan01_volumetric_dynamic_external_history_exact_lease_source_implemented_dynamic_validation_pending`。
- 未完成：受管 runtime/plugin Cargo、真实连续帧 volumetric WGPU PNG、RenderDoc 3D resource/access/barrier RDC、history cut/quality-change 产品对拍、300 帧 CPU/GPU/VRAM/功耗与协调器 acceptance。该切片不修改 temporal weight、jitter、history rejection、light injection 或 froxel layout 算法，不关闭 RG-M4、AF-M3 或 Runtime99。

### 2026-08-30 SSAO descriptor single-owner and AO history exact lease

- 已完成（架构复核）：runtime built-in 与 `rendering.ssao` plugin 原先各自复制完整 compute pass、shader、binding、dispatch 与资源声明；plugin 副本遗漏 runtime 已有的 `AMBIENT_OCCLUSION` `Rgba8Unorm` storage schema，导致同一 feature name 根据注册入口得到不同的物理资源合同。这违反内建渲染功能的单一 descriptor owner 边界，也使后续 exact resource lowering 无法拥有一个可信输入。
- 已完成（源实现，待受管验证）：runtime built-in descriptor 成为唯一 owner，并通过 `graphics::screen_space_ambient_occlusion_render_feature_descriptor()` 公共 facade 发布；SSAO plugin 只保留 plugin identity/capability/registration，descriptor 直接委托 runtime。plugin 回归同时锁定 async-compute workload、history binding、persistent resource 标记与 AO output schema，plugin 内重复 `ComputePassDescriptor` 和 WGSL include 已清零。状态为 `render_plan01_ssao_descriptor_single_owner_source_implemented_dynamic_validation_pending`。
- 已完成（AO exact lease）：`SceneFrameHistoryTextures` 在 SSAO active 时随 history owner 于 pre-scene 阶段分配 SceneLinear primary/render-sized `Rgba8Unorm` AO history，并通过 GPU clear 初始化为 1.0；descriptor 声明 Render extent、full subresource、compute sampled intent，history binder 发布真实 texture/view/physical descriptor。AO history 额外记录 `render_size`，不再复用 temporal/display `size`，因此 temporal upscaler 与动态分辨率不会把 AO history 绑定到 secondary/display extent。SSAO 专用 binder 不再以无尺寸语义的 1x1 white view 冒充 previous history；首帧仅由 `SceneHistoryDomain::AmbientOcclusion` availability 禁止 shader 采样，写回成功后才复制并提交 domain。
- 未完成：projection-aware GTAO/VBAO、有效 normal/depth 合同、edge-aware denoise、qualified temporal、indirect-light/specular-occlusion 合成，以及受管 Cargo/WGPU、PNG/RDC、profile/功耗和协调器 acceptance。该切片只收敛 AO history 资源身份与 ABI，不关闭 RG-M4、PP-M2/PP-M3、Runtime27 或 Runtime89。

### 2026-08-30 Fixed external-buffer physical-lease completion

- 已完成（复核修正）：exposure pass 已经编译出 16-byte `STORAGE | COPY_SRC | COPY_DST` exact external buffer descriptor/access packet，但 history binder 实际仍调用 `insert_buffer`，会移除 imported physical descriptor。history owner 现在从真实 allocation 发布 borrowed buffer 与 `BufferDesc`，binder 使用 `import_borrowed_buffer_with_physical_desc`；previous/current 两槽共享相同 16-byte ABI，仍按 history transaction 轮换。
- 已完成（源实现，待受管验证）：新增 frame-scoped `read_external_buffer_with_schema_and_access` authoring API。SSAO params 的 Rust ABI 从 post-process producer 移入 feature descriptor 单一 owner，compute descriptor 与 pass resource 共同声明 32-byte `UNIFORM | COPY_DST` schema、full-buffer range 和 compute uniform intent；producer 复用同一 `SsaoParams` 类型并发布实际 buffer size/usage，compiled-scene binder 物化 access-ID lease，不再以 descriptor-less buffer 进入 generic compute。状态为 `render_plan01_fixed_external_buffer_physical_leases_source_implemented_dynamic_validation_pending`。
- 已完成（回归源码）：builder API 锁定 frame-scoped typed buffer metadata；默认 pipeline 回归锁定 SSAO canonical `0..32` uniform access packet；SSAO producer/source guard 与 history binder 回归锁定 physical descriptor 路径，exposure previous/current 断言 16-byte physical descriptor。相关文件均低于 800 行结构预算；精确 rustfmt、locked Cargo metadata、source-contract 与 scoped diff checks 通过。
- 未完成：受管 Cargo、真实 generic-compute SSAO/exposure WGPU、PNG/RDC、连续帧 exposure/AO 产品对拍、profile/功耗和协调器 acceptance。该切片修正资源身份与 ABI，不修改 exposure adaptation、SSAO/GTAO 采样或 temporal 算法，也不解决 AO 1x1 historyless variant。

## 性能审阅交接

- 2026-07-18 graph execution diagnostics交接：每帧stage去重已改固定表、compute audit临时partition Vec也已删除；但每pass仍把compiled name/executor/dependencies/resources多次clone进context/profile/record，stats又对workload、dispatch、resource、stage、queue多轮扫描并clone alias/profile。RG-M4须让compiled graph保持metadata权威、always-on一次增量累计dense summary，详细String rows只在capture/profile启用并复用workspace，stats借用/Arc共享；见PERF-MVP-343及`docs/plans/performance/01/2026-07-18-graphics-render-graph-execution-record-static-review.md`。
- 2026-07-18 post-process graph性能交接：frame submission稳定配置仍每帧重建String/Vec stack、clone进compile options、validate/sort graph，再把stack/graph深clone到extract并另存context。Render01联动Render07/17按settings/history/AA/size/feature generations编译唯一dense-ID post artifact，所有消费者共享Arc；stable build/validate/sort/clone=0、changed每variant≤1；见PERF-MVP-362及post-process stack静态证据。
- 2026-07-18 RenderGraph当前源复核：14/14文件、3,767行已重读；typed lookup indices与transient allocation plan构造期缓存已存在，builder handle scan也已O(1)。剩余PERF-MVP-224聚焦`manual_reachability` HashSet clone、read-producer全图复扫、cull writes临时Vec、per-frame `stats()`多遍扫描、内部String join及realtime IBL稳定拓扑重复compile；核心文件由现有owner租约保护，本性能会话只更新验收预算。
- 2026-07-18 compiled graph cache交接：cache-hit立即重复fingerprint已删除；每submission仍两次clone compile options并以BTreeSet/plugin String/owned post stack作HashMap key，miss还在framework state锁内同步compile。Render01联动Render17发布generation-based compact key与锁外single-flight compile ticket，同key compile≤1、stable key owned/hash bytes=0、miss compile lock hold=0；见PERF-MVP-365及cache静态证据。
- 2026-07-18 瞬态物化性能交接：RG-M2已缓存allocation plan并收敛预算驱逐；materialization validation的per-frame lifetime `BTreeSet`也已改为复用compiled索引。但稳定graph仍重建slot/name maps、逻辑views并以String表扫描成功验证。保持当前RG-M2冻结manifest不变；提交后RG-M3首批改为compiled dense materialization plan/required-binding mask、handle-indexed workspace/bound bitset与pool-owned default view。warm grouping/String/workspace/default-view create及成功验证String lookup均为0；见PERF-MVP-366及两份materialization静态证据。
- 2026-07-18 post executor资源交接：SMAA executor当前绕开graph每帧直接创建edge/blend全尺寸textures/views，disabled bloom/DoF/MV/SSAO仍以至少6个clear passes物化中间资源。RG后续须把SMAA edge/blend/stencil声明为logical transient resources并复用pool/view bundle；feature-off producer与孤立consumer必须真正cull，以neutral import/合法alias满足剩余读取，不能把clear伪装成裁剪。保持RG-M2冻结manifest不变，在后续owner切片实施；见PERF-MVP-370。
- 2026-07-18 compiled-scene绑定交接：historyless路径仍在执行期clone frame并重建post stack/graph；light-grid/HZB无producer时每帧创建最多9个fallback buffers并格式化backing name。Render01联动Render07把history variant纳入compiled artifact，并把neutral resources纳入dense binding plan的device/resource-generation owner；见PERF-MVP-374/375及compiled-scene render静态证据。
- 2026-07-18 stage dispatch hard-cut交接：frame执行目前每stage全扫`pass_stages`，每entry再按String全扫graph passes，虽CompiledGraph已有PassId index却未被stage metadata使用。Render01后续把`CompiledRenderPipelinePassStage`改为PassId/dense index并编译按stage连续range，执行只遍历range且O(1)取pass；见PERF-MVP-378。当前`render_graph/graph.rs`活跃owner租约未被本性能会话越权修改。
- 2026-07-18 full-target clear融合交接：camera policy当前为split-view安全把scene color/depth首次Clear一律改Load，再额外录制region triangle；full-target也付出1 pass/1 draw。Render01联动Render09只对partial region保留draw clear，full-target首camera把intent融合进各资源首次attachment write；无partial clear时不创建三条region pipeline。见PERF-MVP-394及scene-clear静态证据。
- 2026-08-27 PFO-4d1l region-clear upload owner收敛：保留partial-region triangle与viewport/scissor策略，但16-byte color uniform不再由clear feature直接写queue；它随`RenderGraphStageExecution`在整图成功后合入唯一`FrameBufferUpload`，graph失败不发布参数side effect。源码静态检查通过，scene-renderer非测试direct write由本切片前12次/11文件降为11次/10文件。该结果不关闭PERF-MVP-394：full-target首camera仍需融合进attachment clear，并必须以真实WGPU/RenderDoc/profile证明pass/draw消失。
- 2026-08-27 PFO-4d3a RDG物理owner收敛：pool与`RenderGraphExecutionResources`之间不再交接裸texture/buffer，owned transient改存携带device epoch、descriptor key/descriptor、frame age与last-use ticket的move-only allocation lease；external imports仍在独立map，persistent extraction/history copy边界不变。pool key、compiler alias compatibility和WGPU native create同时纳入`view_formats`集合。pool主文件从1009行拆至707行，静态格式/source/diff检查通过；`BTreeMap`、全历史frame-end扫描和逐资源pending查询保持未优化，等待真实300帧profile后再决定hash bucket/ticket bucket/age wheel，当前无性能或动态验收结论。
- 2026-08-27 PFO-4d3b RDG瞬态池测量基础：completion collection与frame-end maintenance现有独立CPU scope；texture/buffer status query、stale scan、budget accounting与over-budget sort candidate共8个work counter进入既有graph report和render diagnostics。计数只复用原遍历点，没有增加扫描或修改复用/淘汰算法。静态格式、诊断映射与diff检查通过；真实300帧profile、WGPU、RenderDoc、PNG、显存和功耗仍pending，当前不批准ticket bucket、age wheel或其它优化结论。
- 2026-08-29 frame producer rejection settlement：backend新增统一的pre-scene producer登记边界；当history initialization、texture pre/copy/post upload或frame-resource upload的ledger校验失败时，刚由同一device接受的ticket立即经`settle_abandoned_submissions`结算，并把ticket与settled status保留在typed `FrameProducerRegistrationFailed`中，再回到既有frame transaction失败路径。生产调用不再直接绕过backend登记，避免失败结算遗漏已接受ticket；新增源守卫覆盖登记与结算顺序。精确rustfmt、scoped diff与locked Cargo metadata通过；受管Cargo、WGPU、RenderDoc、PNG、帧时与功耗证据仍pending，状态为`frame_producer_rejection_settlement_source_implemented_dynamic_validation_pending`。
- 2026-08-29 viewport history record typed failure：`record_history`的`(existing=None, allocated=None)`异常态不再使用生产`unreachable!`，而是返回`RenderFrameworkError::InvalidSubmissionState`；capture、present、runtime-frame与non-viewport camera四条记录路径显式传播，且history记录前移到pipeline/visibility/capture等其它viewport状态写入之前，避免失败时留下部分记录。对应生产子域`panic!/expect/unwrap`扫描为0，精确rustfmt、scoped diff与locked Cargo metadata通过；动态Cargo/WGPU与产品帧证据仍pending，状态为`viewport_history_record_typed_failure_source_implemented_dynamic_validation_pending`。
- 2026-07-18 history binding owner交接：六张full-res history初始化的CPU整图Vec/write已改同encoder两次GPU clear；但任一history feature仍创建全集，任一size/HZB/froxel变化替换整包，bind每frame clone handles。Render01发布`HistoryResourceMask`和per-slot generation dense binding plan，feature-off真实slot=0、stable clone/rebuild=0。见PERF-MVP-395。
- 2026-07-18 overlay pass融合交接：graph的单个overlay executor内部当前对相同color/depth依次开启selection/wire/grid/gizmo/handle最多5个LoadStore pass，掩盖真实pass成本。Render01联动Render10把它收敛为一个overlay pass内保持原draw顺序和pipeline切换，并让compiled report/timestamp呈现真实内部pass/draw；验收overlay pass≤1。见PERF-MVP-333及overlay静态证据。
- 2026-07-18 graph executor dispatch交接：`graph_execution/**`当前47/47个Rust文件、14,478行已静态读完。validation已有generation cache，但稳定pass仍以String ID查`BTreeMap`，每renderer owner重复注册built-in表；post product executor还clone required/produced资源名。RG-M4在compile/validation边界发布dense `ExecutorSlot`和declaration-handle ranges，执行期O(1)取slot且不回退String tree；插件generation变更只重绑受影响compiled artifact。Uber未使用resource-route扫描已直接删除。见PERF-MVP-399及`docs/plans/performance/01/2026-07-18-graphics-graph-execution-complete-static-review.md`。
- 2026-07-18 realtime IBL graph交接：每个活跃time slice仍动态格式化约70个A/B mip资源名、clone slot树、compile graph、建live-name HashSet并逐名绑定/验证。Render01联动Render11按request geometry×scheduler state/substep×slot预编译有限variant与dense binding plan，sky key变化只更新参数；同variant compile≤1、warm String/hash/map growth=0。初始full update也须有界，见PERF-MVP-401及realtime-IBL静态证据。
- 2026-07-18 IBL bake command交接：标准10-pass bake当前每pass重建完整10-command shader/command plan并线性匹配自身，随后逐pass创建params/bind/output mip view；runtime writeback还同步readback+文件写。Render01联动Render11把request generation编译为dense pass→command/params/view/readback ranges并复用graph binding plan，10-pass build=1、per-pass name parse/full-plan=0；见PERF-MVP-402及environment IBL bake静态证据。
- 2026-07-18 advanced-lighting artifact交接：froxel/cookie/irradiance/planar/SSS pass当前分别重做camera/scene筛选、metadata与逐pass GPU对象，irradiance甚至在draw前和graph pass内完整准备两次。Render01须让compiled pass只持`PreparedAdvancedLightingFrame`的dense handles，不在executor重建selection/table/plan；stable artifact resolve=0且irradiance prepare≤1/camera generation。OIT冗余layer clear、空froxel fallback及cookie slot已止损，见PERF-MVP-403与advanced-lighting静态证据。
- 2026-07-18 output-target writeback交接：resource streamer当前每写回帧创建conversion bind group和独立encoder，并额外`queue.submit`；Render01须把conversion/copy节点编入主graph与同一encoder，binding按source/target/layout generation复用，失败或未请求时零额外对象/提交。验收writeback额外submit=0、stable bind/encoder create=0，见PERF-MVP-404及scene-resources静态证据。
- 2026-07-18 GPUScene scatter upload交接：Render01联动Render03把large dirty update的预尺寸staging/scatter应用编入主graph，按阈值保留small direct write；graph节点消费dense dirty ranges并复用in-flight arena，禁止每range独立owner/submit。稳定节点cull，large update copy/compute批次有界，见PERF-MVP-405。
- 2026-07-18 backend surface/target交接：Render01把surface present blit编入主graph末端并复用frame encoder/submit，稳定帧额外bind/encoder/submit=0；同时发布`OffscreenResourceMask`与target extent generation，让final/depth/core和GBuffer/GI/bloom/AO/cluster按compiled usage独立slot化，resize只替换受影响slot。见PERF-MVP-407/408及graphics backend静态证据。
- 2026-07-18 framework lifecycle command交接：Render01让surface create/reconfigure/present与history release服从同一render-owner ordered lane；framework API只在短锁内预留viewport generation ticket，GPU/driver执行不持全局state lock，stale publish可丢弃且same-viewport顺序确定。见PERF-MVP-411。
- 2026-07-18 validated pipeline revision交接：Render01联动Render08在register/reload按pipeline revision+executor/capability generation发布immutable validation/compiled binding artifact，set viewport/profile只做handle与兼容性检查；锁外single-flight compile、短锁CAS发布且失败保留last-good。见PERF-MVP-412并复用PERF-MVP-365 compact key/cache。
- 2026-07-18 submission graph诊断交接：离屏record的stable compiled generation现可复用上一capture graph文本，不再逐帧遍历序列化；Render01仍须把graph dump作为compiled generation的lazy immutable diagnostics artifact，只有capture/debug请求才格式化并共享，normal present/diagnostics-off build=0。采用UE RDG的debug/request gate原则，不复制其API；见PERF-MVP-413。
- 2026-07-18 frame context双compile交接：同一camera当前先compile/cache lookup读取feature flags，AA/post/IBL options成形后再lookup/compile最终variant，且miss闭包仍持framework state锁。Render01让validated revision先发布descriptor feature mask与compact option template，只对最终camera key执行一次锁外single-flight compile并短锁发布；每camera lookup≤1、same key compile≤1、state锁内compile=0。见PERF-MVP-414并复用365/412。
- 2026-07-18 feedback/history graph交接：Render01把readback copy和history release/rotation纳入同一ordered graph/render-owner lane，输出generation-tagged ticket而非submission点同步take大payload。固定in-flight staging ring只在diagnostics/provider需要时录制copy，feature-off节点cull；frame-thread wait=0、same generation copy/merge≤1。见PERF-MVP-415并复用411/413。
- 2026-07-18 pipeline compiler交接：descriptor/resource analysis当前在validation与authoring间重复，stage/pass ordering又按write重扫writers/readers并多轮clone。Render01发布PERF-MVP-422的唯一compiled descriptor/resource artifact，并按PERF-MVP-423一次构建dense resource adjacency与stage ranges；stable generation analysis=0，单次compile近O(P×R+E+M)，pass full clone=0。
- 2026-07-18 output-target plan补充：types层graph-import/writeback状态判断已移除生产诊断String，但readiness、final selection与skip-writeback仍重复构造计划，writeback仍独立encoder/submit。Render01在PERF-MVP-404把external texture import/copy/conversion编入主graph并复用一次状态决策；额外submit=0，稳定binding/plan重建=0。
