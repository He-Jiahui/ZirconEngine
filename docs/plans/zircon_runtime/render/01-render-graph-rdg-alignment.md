---
related_code:
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
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

现状基础:`render_graph/` 已有 `RgTextureHandle`/`RgBufferHandle`、`create_texture`/`create_buffer`/`import_external_resource`、读写声明、依赖推导、`cull_passes`、`resource_lifetimes`(first_pass/last_pass)与逻辑层 `transient_allocation_plan()`;执行层已有 `RenderGraphExecutionResources`(每帧新建 wgpu 资源,无池化)与 `RenderPassResourceResolver`(只解析元数据,不解析物理资源)。本计划的增量是:首写 ops 决策表校验、root 驱动的反向裁剪、物理瞬态池、编译缓存、句柄级物理解析收口。

新增文件:

| 路径 | 职责(一行) |
|------|------------|
| `zircon_runtime/src/render_graph/dump.rs` | `RenderGraphDump` 纯数据构建与文本序列化(pass 顺序、资源区间、culled 列表),无 wgpu |
| `zircon_runtime/src/render_graph/tests/attachment_ops.rs` | 首写 attachment ops 决策表与 usage flags 的编译期校验测试 |
| `zircon_runtime/src/render_graph/tests/culling.rs` | root 驱动反向可达裁剪的单测(从既有 `tests/ordering.rs` 拆出裁剪主题) |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs` | `TransientResourcePool`:描述符桶 + 区间着色的 wgpu 物理资源池,含 `#[cfg(test)] mod tests` |
| `zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs` | `CompiledGraphCache`/`CompiledGraphCacheKey`/统计与 LRU 驱逐,含 `#[cfg(test)] mod tests` |

修改文件:

| 路径 | 改动点 |
|------|--------|
| `zircon_runtime/src/render_graph/types.rs` | 新增 `RenderGraphResourceUsageFlags`;`RenderGraphResourceDeclaration`/`RenderGraphResourceLifetime` 增 `usage` 字段 |
| `zircon_runtime/src/render_graph/builder.rs` | `mark_persistent`/`mark_readback`/`import_external_resource_with_usage`;`compile()` 内首写 ops 校验;`cull_passes` 改为 usage-root 驱动并删除 `has_no_writes`/`writes_external` 兜底存活 |
| `zircon_runtime/src/render_graph/error.rs` | 新增 `FirstWriteMissingAttachmentOps`、`MissingCullRoot` 变体 |
| `zircon_runtime/src/render_graph/graph.rs` | `allocate_transient_lifetimes` 槽位分桶键从 kind 细化为完整描述符哈希;`CompiledRenderGraphTransientAllocation` 增 `bucket_key_hash: u64`;新增 `CompiledRenderGraph::dump()` |
| `zircon_runtime/src/render_graph/mod.rs` | 仅 wiring:声明 `dump` 模块、导出新类型 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs` | `RenderPassResourceResolver` 重命名为 `RgResourceResolver` 并增加物理解析(借用 `RenderGraphExecutionResources`),元数据 API 全量保留 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs` | `resource_resolver()` 改为按需组合 graph 元数据 + `gpu.resources` 物理表返回 `RgResourceResolver`;导出名同步 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs` | 删除 `materialize_transient_resources`(职责移入池);新增 `bind_pooled_texture`/`bind_pooled_buffer`;`require_texture_view`/`require_buffer` 可见性收紧为 `pub(in crate::graphics::scene::scene_renderer)`,执行器只许经 `RgResourceResolver` |
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
/// 跨计划契约名:RgResourceResolver。由 RenderPassResourceResolver 硬切换改名而来,
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
    // 元数据 API(原 RenderPassResourceResolver,签名不变):
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
4. `SceneRenderer::render_frame_with_pipeline` → `render_frame_with_pipeline_to_target` → `SceneRendererCore::render_compiled_scene`(`render/render.rs`):在 `let mut graph_resources = RenderGraphExecutionResources::new();` 与 `import_frame_targets(...)` 之后,**插入** `self.transient_pool.allocate_frame_resources(device, &pipeline.graph, &mut graph_resources)?`;**删除** `render.rs` 与 `execute_graph_stage.rs` 中对 `materialize_transient_resources` 的两处调用及该函数本体(硬切换)。history 纹理仍由 `prepare_history_textures`(`scene_renderer_history`)持久管理并以 import 方式进 `graph_resources`,池不接管(与计划 06 对齐的接缝就是 `mark_persistent` + import)。
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
  1. `resource_resolver.rs`:`RenderPassResourceResolver` → `RgResourceResolver`,加 `with_physical` 与句柄/按名物理解析 → 判据:cargo check(旧名引用全部报错即迁移清单)。
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

RG-M3(文件:`compiled_graph_cache.rs` 内联 tests;裁剪在 `render_graph/tests/culling.rs`;集成断言在 render_framework 既有测试目录):

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

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | RG-M1 resource handles and lifetime model | 部分完成: graph contract 已服务后续渲染切片,统一生命周期 owner 未完全收束 | 计划 04/06/07 已依赖 graph resource names、external imports、execution-owned resources 与 WGPU executor contract 承载 HZB、TAA、exposure、output-transfer 等资源;但真实 wgpu resource 仍有不少 executor 自持路径。 | 当前计划 04/06/07 的 `## 状态与产出记录` 均记录了 graph resource/executor 接入;`render_graph_execution_resources` 相关 alias/materialization 测试在会话记录中已有通过证据。 | 继续把 executor 私有资源迁入 graph materialization,并为 attachment clear/load 与 resource lifetime 建统一校验。 |
| 2026-06-15 | RG-M2 transient resource pool | 部分完成: transient texture/buffer pool 已被 TAA/HZB/postprocess 使用 | TAA history、HZB chain、postprocess HDR/tonemapped/output-transfer、reactive mask、exposure 等切片已通过 graph resource 名称和 transient materialization 复用资源。 | 计划 06/07 状态表记录 `R8Unorm`/`R16Float`/`Rg16Float`/`Rg11b10Ufloat` transient usage 修正和 core-min `cargo check` 通过;会话记录包含 transient pool reuse/alias tests。 | 补齐池容量预算、跨帧 eviction 诊断和 graph dump 中的 transient alias 可视化。 |
| 2026-06-15 | RG-M3 pass culling and compile cache | 部分完成: pass culling 已在多处生效,跨帧编译缓存仍未完成 | Postprocess optional pass filtering、TAA disabled stack culling、HZB occlusion capability gate、contact shadow feature disable gate 已落地;编译缓存仍以当前 pipeline asset 编译路径为主。 | 计划 04、06、07、05 状态表分别记录 HZB gate、TAA reactive mask culling、postprocess optional pass filtering、contact-shadow disabled graph absent。 | 建立 `CompiledRenderPipeline` cache key、增量 invalidation 和 graph dump 对拍测试。 |
| 2026-06-15 | RG-M4 diagnostics and RenderDoc bridge | 部分完成: debug marker/diagnostics 可用,自动化捕获验收仍待后续 | Graph pass marker、runtime diagnostics 与 RenderDoc capture env hook 已被后续计划使用;但缺少统一 graph dump artifact 和自动附带 frame profile 的 capture 包。 | 计划 04/06/07 状态表均记录 RenderDoc 仍无运行实例或待后续验收;当前只完成源码/测试级证据。 | 为每帧输出 graph dump、resource alias map、pass timings 和 RenderDoc marker 对拍。 |

### 参考实现精读笔记

`dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h` — `FRDGBuilder`:

- `CreateTexture(const FRDGTextureDesc&, const TCHAR* Name, ERDGTextureFlags)` / `CreateBuffer(...)` / `RegisterExternalTexture(const TRefCountPtr<IPooledRenderTarget>&, ...)`:解决"声明与分配分离"——句柄先行,RHI 资源延迟到执行期。Zircon 对应 `RenderGraphBuilder::create_texture/create_buffer/import_external_resource_with_usage`(已有,本计划补 usage)。取舍:UE 的 `CreateBuffer(NumElementsCallback)` 延迟定容不移植,wgpu 无 placed resource,定容收益小。
- `AddPass(FRDGEventName&&, const ParameterStructType*, ERDGPassFlags, ExecuteLambdaType&&)`:参数结构体(\_RDG\_ 宏)即资源 IO 声明,lambda 与参数分离。Zircon 用显式 `read_*/write_*` 声明 + executor id 注册表替代 lambda(动态插件边界不能传闭包),等价性由 `validate_compiled_pipeline` 的 executor 存在性校验保证。
- `QueueTextureExtraction(FRDGTextureRef, TRefCountPtr<IPooledRenderTarget>*)` 与 `FRDGViewableResource::IsCullRoot() { return bExternal || bExtracted; }`(RenderGraphResources.h):extraction/external 即裁剪 root。Zircon 把它显式化为 `RenderGraphResourceUsageFlags{present, readback, persistent}`,比 UE 更细(UE 的 external 一律 root,Zircon 的 external 仅在标记后成 root,避免导入贴图把上游全保活)。
- `Compile()` / `SetupPassDependencies` / `MarkResourcesAsProduced`:编译期推导依赖与裁剪。Zircon 对应 `RenderGraphBuilder::compile()` 内的 `infer_resource_dependencies` + `cull_passes`(已有),本计划只改 root 判据。
- `IRHITransientResourceAllocator* TransientResourceAllocator` + `bSupportsTransientTextures` + `IsTransientInternal` + `FCollectResourceOp::Allocate/Deallocate`:UE 把生命周期区间编译成 Allocate/Deallocate 操作流,由 RHI transient allocator 做真实内存别名。wgpu 没有该能力,Zircon 退化为 `TransientResourcePool` 池化复用(正文既定取舍),Allocate/Deallocate 操作流简化为帧首一次性区间着色。
- `TickPoolElements()`:每帧池维护钩子 → Zircon `TransientResourcePool::end_frame` 的 KEEP_FRAMES 驱逐。

`dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphResources.h`:

- `FRDGViewableResource` 的位标记组:`bExternal / bExtracted / bProduced / bTransient / bForceNonTransient / bSkipLastTransition`,以及 `FirstPass / LastPasses`、`AcquirePass / DiscardPass`、`ReferenceCount`、`AliasingOverlaps(FRHITransientAliasingOverlap)`:资源自带生命周期端点与别名重叠表。Zircon 的 `RenderGraphResourceLifetime{first_pass, last_pass}` 已对应端点;`AliasingOverlaps`(别名内存的 acquire/discard 屏障)不移植——池化复用下 wgpu 自行管理底层内存,串台风险由首写 Clear 决策表兜底。
- `FRDGSubresourceState`(`FirstPass/LastPass` per pipeline、`IsTransitionRequired` / `IsMergeAllowed`):子资源粒度 barrier 合并。wgpu 自动 barrier,Zircon 不需要;mip 粒度的需求(SSR pyramid)用既有 `ssr_pyramid_mip_alias` 的 mip view 别名即可。
- `FRDGTexture::TransientTexture(FRHITransientTexture*)`:transient 分配的句柄回填位 → Zircon 对应 `RenderGraphExecutionResources::bind_pooled_texture` 的 name→view 绑定。

`dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs`:

- `Execute()` 中 `int graphHash = m_EnableCompilationCaching ? ComputeGraphHash() : 0; CompileNativeRenderGraph(graphHash);`:编译缓存以"图内容哈希"为 key,在录制后、编译前求值。`ComputeGraphHash()` 用 `HashFNV1A32` 对 `m_RenderPasses[i].ComputeHash(ref hash128, m_Resources)` 逐 pass 折叠。Zircon 的差异:Zircon 的图由 pipeline asset + options 决定、不每帧重录,所以 `CompiledGraphCacheKey` 直接由输入端(revision/options/profile/viewport/caps/extract 指纹)组成,省掉每帧全图哈希,这是比 Unity 更便宜的稳态路径;代价是必须用 `extract_compile_fingerprint` 的 debug_assert 守住"编译读取面都进 key"的不变量。
- pass 合并(`CompileNativeRenderGraph` 转入 NativeRenderPassCompiler 的 merge)不在本计划移植:wgpu 的 render pass 合并收益体现在 tile GPU,留待后续独立立项,不影响缓存接口。

`dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphCompilationCache.cs`:

- `HashEntry<T>{hash, lastFrameUsed, compiledGraph}`、`k_CachedGraphCount = 20`、`GetCompilationCache(hash, frameIndex, out CompilerContextData)`:命中刷新 `lastFrameUsed`;miss 时优先从 `m_NativeCompiledGraphPool` 取空条目,池空则按 `lastFrameUsed` QuickSort 复用最旧条目。Zircon `CompiledGraphCache` 采用同构 LRU(capacity 16,`last_used_frame` 驱逐),但条目直接持 `Arc<CompiledRenderPipeline>` 而非预分配对象池——Rust 下 Arc 复用即免 GC 压力,无需 Unity 的对象池形态;另补 `invalidate_pipeline`(Unity 靠图哈希天然失效,Zircon 输入端 key 需要资产修订显式失效)。

`dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs`(TransientResourcePool 的桶结构参照):

- `m_ResourcePool: Dictionary<int, SortedList<ulong, (Type resource, int frameIndex)>>` + `TryGetResource(hashCode)` / `ReleaseResource(hash, resource, currentFrameIndex)` + `kStaleResourceLifetime = 10` + `PurgeUnusedResources`:描述符哈希做桶、桶内按 `GetSortIndex` 排序保证跨帧取用稳定、按帧龄清陈旧条目。Zircon `TransientResourcePool` 对应:`HashMap<TransientTextureKey, Vec<PooledTextureEntry>>` 桶、桶内 slot 索引序(区间着色后 slot i ↔ 条目 i,天然稳定)、`KEEP_FRAMES = 8` 驱逐。取舍:Unity 是"借出/归还"动态池(pass 边界归还),Zircon 是"帧首整批绑定、帧末整批归还"——因为生命周期区间在编译期已知,整批绑定可保证同帧别名决策确定性,也避免执行期池锁。

## 风险与回退

- executor 签名改动波及全部内建 executor 与插件 executor:按硬切换原则一次迁完,以 `cargo check` 驱动清单;插件侧(rendering/hybrid_gi/virtual_geometry)在同一里程碑内适配。
- 池化复用引入串台风险(写后读到旧内容):靠 attachment ops 校验(首写必 Clear)兜底;出现画面异常时用 graph dump + RenderDoc 定位区间分配错误。
- 不做底层内存 alias,wgpu 池化即止;若未来需要更激进的显存控制,在本计划之上单独立项,不回改接口。
