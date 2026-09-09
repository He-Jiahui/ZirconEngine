---
related_code:
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/resource_schema.rs
implementation_files:
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/render_graph/builder/access_authoring.rs
  - zircon_runtime/src/render_graph/builder/resource_dependency_inference.rs
  - zircon_runtime/src/render_graph/graph/resource_state_plan.rs
  - zircon_runtime/src/render_graph/graph/transient_allocation.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
tests:
  - zircon_runtime/src/render_graph/tests
doc_type: module-detail
---

# 渲染图

## 概览

Render Graph 将一帧描述为 Pass、有类型资源和访问关系。它解决四类问题：由资源读写推导执行顺序；裁剪不会影响最终输出的工作；计算 transient texture/buffer 的物理别名；为执行层生成资源状态转换与访问分配表。

它不是命令编码器，也不直接执行 WGPU。`CompiledRenderGraph` 是计划；`SceneRenderer` 的 graph execution 层负责将 executor ID、外部绑定和物理资源连接起来。

## Builder 模型

### Pass

`add_pass(name, QueueLane)` 创建 Pass。可用 `add_pass_with_executor` 关联 executor ID；compute Pass 还可以设置 `RenderGraphComputeWorkload` 与 metadata。`set_pass_flags` 控制图裁剪等行为，`add_dependency` 添加资源关系之外的显式顺序。

### 资源

- `create_texture(TextureDesc)` / `create_buffer(BufferDesc)`：图拥有的 transient 资源。
- `create_texture_view_alias`：parent texture 某个 mip/layer/aspect 的逻辑视图，不另占 allocation slot。
- `import_external_*`：导入由 viewport、history、插件或宿主持有的物理资源。
- `import_present_external_*`：导入最终呈现目标，同时成为图存活根。
- `mark_persistent`：阻止瞬态回收语义用于该纹理。
- `mark_readback`：标记 CPU/诊断消费者需要的图输出。

外部资源可以携带 `RenderGraphExternalResourceBinding`，其中区分 resource type 和 required/optional/report-only 等要求。共享同一物理 allocation 的多个外部 view 应声明 alias group，使编译器保守合并访问历史。

## 资源版本与访问

每次写入会产生 `RenderGraphResourceVersionToken`。后续 Pass 读取特定版本，以避免“同一资源名字但读取了错误写入”的隐式依赖。Token 带 builder generation、producer Pass 和 access index；跨 builder 或资源不匹配会在 compile 前返回 `RenderGraphError`。

访问 metadata 可以限定：

- buffer byte range；
- texture mip、array layer、color/depth/stencil aspect；
- shader stage；
- read/write intent 和 attachment load/store 操作。

范围不相交时，编译器可避免虚假依赖；范围重叠时必须保持正确顺序。Texture view alias 的 parent 不能再是 alias，且 range 必须落在 parent descriptor 内。

## 编译结果

`RenderGraphBuilder::compile()` 返回 `CompiledRenderGraph`，包含：

- 拓扑排序后的 `CompiledRenderPass`；
- 被裁剪 Pass 和存活根统计；
- 每资源的版本/访问索引；
- `CompiledRenderGraphResourceStatePlan` 与 transition；
- transient allocation plan、slot reservation 和物理 allocation ID；
- compute binding/dispatch access packet；
- external access packet；
- `CompiledRenderGraphStats`。

编译失败包括循环依赖、未知/外来句柄、未定义读取、访问范围非法、外部类型冲突、视图 alias 越界和 attachment 合同无效。

## 最小示例

```rust
use zircon_runtime::render_graph::{QueueLane, RenderGraphBuilder};
use zircon_runtime::rhi::{
    TextureDesc, TextureFormat, TextureUsage,
};

fn build_graph() -> Result<(), zircon_runtime::render_graph::RenderGraphError> {
    let mut graph = RenderGraphBuilder::new("example");
    let color = graph.create_texture(TextureDesc::new(
        "scene-color",
        1280,
        720,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
    ));

    let geometry = graph.add_pass("geometry", QueueLane::Graphics);
    let composite = graph.add_pass("composite", QueueLane::Graphics);
    graph.add_dependency(geometry, composite)?;

    // 实际代码还必须用 write/read authoring API 声明 color 的版本访问。
    let compiled = graph.compile()?;
    let _stats = compiled.stats();
    let _ = color;
    Ok(())
}
```

示例刻意不虚构 access authoring 的简写；生产代码应查看 `RenderGraphBuilder` 当前公开的 `read_*`、`write_*` 和 metadata 方法，并为每个 GPU 使用声明精确访问。

## 调试与审计

`RenderGraphDump` 将 Pass、资源、descriptor、transient slot 和 Pass-resource 行转换为稳定诊断结构。`RenderGraphStoreLintReport` 审计 attachment store 行为和带宽账本，用于发现最终无人读取却仍 Store、或需要保留却被错误 Discard 的情况。

## 性能指南

- 资源声明尽量精确，但不要为了“优化”拆出大量无实际独立生命周期的小资源。
- transient alias 只复用生命周期不重叠且 descriptor/usage 兼容的物理存储。
- external alias group 选择保守正确性，不保证并行。
- Pass 名和 executor ID 应稳定；它们也用于诊断、缓存和 profiling。
- compute queue lane 只是图调度意图，最终是否异步取决于后端能力和编译降级。

## 实现状态

图编译与上述计划结构为**默认可用**。真正的多队列异步执行、所有后端通用状态屏障映射，不能仅从 `QueueLane` 类型存在推断；当前生产执行以 WGPU 能力和 SceneRenderer 路径为准。
