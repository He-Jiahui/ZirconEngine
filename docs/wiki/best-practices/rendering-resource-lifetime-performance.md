---
related_code:
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/graph/transient_allocation.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_device.rs
  - zircon_runtime/crates/zr_rhi/src/submission.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/create_texture_bundle.rs
implementation_files:
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/render_graph/graph/transient_allocation.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_device.rs
plan_sources:
  - user: 2026-09-09 补充 ZirconEngine 最佳实践、方案示例与详细 Wiki
tests:
  - zircon_runtime/src/render_graph/tests
  - zircon_runtime/crates/zr_rhi/src/tests
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests
doc_type: workflow-detail
---

# 渲染资源生命周期与性能实践

渲染扩展首先应描述资源访问与生命周期，再选择缓存或后端特化。Render Graph 把 pass、版本化资源、访问范围和 transient allocation 编译为计划；RHI 则以带 device/generation 的 typed handle、submission ticket 与 surface lease 约束真实 GPU 工作。相应实现见 [render graph builder](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/render_graph/builder.rs)、[transient allocation](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/render_graph/graph/transient_allocation.rs) 与 [RenderDevice](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_rhi/src/device/render_device.rs)。

## 按资源所有者建模

| 资源需求 | Render Graph 建模 | 生命周期 | 性能含义 |
| --- | --- | --- | --- |
| 只服务本帧中一段 pass 链 | `create_texture` / `create_buffer` | graph-owned transient | 编译器可在不重叠寿命且 descriptor/usage 兼容时 alias |
| 同一 texture 的 mip/layer/aspect 视图 | `create_texture_view_alias` | 依附 parent | 不新占 allocation slot；范围必须在 parent 内 |
| viewport、history、插件或宿主持有 | `import_external_*` | 外部所有者决定 | 声明 external binding 与保守 alias group |
| 最终显示目标 | `import_present_external_*` | surface frame lease | 是图存活根，完成后 present 或 discard |
| 跨帧历史数据 | external/imported 或明确 persistent 标记 | 调用方负责失效和设备重建 | 不可假装为可随时 alias 的 transient |

每次写入都会产生 `RenderGraphResourceVersionToken`。后续读取应指向正确版本并声明精确 buffer range、mip/layer/aspect、shader stage 和读写 intent；这让编译器推导依赖与 state transition，而不是依赖 pass 名称或人为排序。模型细节见 [渲染图](../graphics/render-graph.md)。

## 用资源声明表达真实数据流

**推荐，实际 API 的最小骨架：**

```rust
let mut graph = RenderGraphBuilder::new("outline");
let mask = graph.create_texture(mask_desc);
let geometry = graph.add_pass("outline-mask", QueueLane::Graphics);
let composite = graph.add_pass("outline-composite", QueueLane::Graphics);

// 生产代码还要用当前公开的 read/write authoring API
// 声明 mask 的写版本与 composite 对该版本的读取。
graph.add_dependency(geometry, composite)?;
let compiled = graph.compile()?;
```

上例故意不杜撰 access-authoring 简写；真实扩展必须使用当前公开的 `read_*`/`write_*` 方法为每次 GPU 使用登记版本和访问元数据。`compile()` 会拒绝循环、外来句柄、未定义读取、非法范围、外部类型冲突、alias 越界和 attachment 合同错误。接口契约与失败类型可从 [render graph 模块](https://github.com/He-Jiahui/ZirconEngine/tree/main/zircon_runtime/src/render_graph) 追溯。

**反模式，示意伪代码：**

```rust
// 错误：以资源名称和隐式 pass 顺序表达依赖。
pass_a.write("scene-color");
pass_b.read("scene-color"); // 没有版本、范围和访问 intent
```

这既可能读到错误写版本，也会迫使编译器保守同步，进而破坏裁剪、alias 和带宽审计的依据。

## 设备、提交与 surface 的边界

| 场景 | 正确步骤 | 禁止假设 |
| --- | --- | --- |
| 创建/使用 RHI 资源 | 从 `RenderDevice` 创建 typed handle；设备 generation 改变后重建派生资源 | 裸 index 或跨 device 的 handle 仍有效 |
| 上传 | 用 batch 收集高频写入，取得 `SubmissionTicket` | 每个小写入都拆成逻辑提交更快 |
| 销毁 | 考虑 in-flight 引用与 ticket 状态 | `destroy_*` 后 GPU 已立刻不再引用 |
| 呈现 | acquire 一个 `SurfaceFrameLease`，提交引用它的工作，再以 ticket present | present 会再次 queue submit |
| resize/outdated/零尺寸 | reconfigure 或跳过帧，等待宿主事件 | 无限 acquire 重试能恢复 |

RHI handle 含 device ID、device generation 与 slot generation；submission ticket 的终态查询也有保留上限。长期诊断应输出自己的聚合指标，不能要求任意旧 ticket 永久可查。surface lease 的严格 acquire/present/discard 路径和 WGPU 边界见 [RHI 与 WGPU 后端](../graphics/rhi-and-wgpu.md)。

## 性能决策与测量

```text
出现帧时间或显存回退
        |
        +-- 检查 graph dump：pass 是否被裁剪、资源版本/范围是否正确
        |
        +-- 检查 transient allocation：是否存在重叠寿命或 descriptor/usage 不兼容
        |
        +-- 检查 RenderDevice 统计：memory snapshot、allocator、submission
        |
        +-- 再检查产品缓存：resident/dirty/evictable 等专用统计
```

先用 `RenderGraphDump`、`CompiledRenderGraphStats`、`RenderGraphStoreLintReport` 发现无用 Store、无消费者资源或未预期 alias 禁用，再决定是否改变 cache。RHI 也提供 memory snapshot、transient allocator stats 和以 submission ticket 限定的诊断/readback；不要以一次 GPU query 的结果替代持续测量。[渲染图](../graphics/render-graph.md) 与 [RHI 与 WGPU 后端](../graphics/rhi-and-wgpu.md) 分别说明这些诊断面。

Hybrid GI 运行时反馈明确提供 resident、dirty、invalidated、evictable 和 cache generation 一类统计；这意味着缓存维护应由这些反馈驱动，而不是在每帧无条件重建。类型可见于 [runtime stats](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_stats.rs) 与 [runtime feedback](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_feedback.rs)。

## 失败定位与核对

| 症状 | 首先检查 | 修复 |
| --- | --- | --- |
| 黑屏或读到旧图像 | resource version、access range、present root | 声明写后读的正确 token，保留最终输出 |
| 显存持续增长 | external/persistent 资源的 owner、memory snapshot、eviction feedback | 给跨帧缓存定义失效/回收条件；临时资源交回 graph |
| resize 后 validation error | device/surface generation 与 derived resource cache | reconfigure 后废弃旧 lease/handle 并重建派生资源 |
| GPU 空转或 CPU 阻塞 | upload batch、submission ticket、readback 时机 | 合批上传，异步 poll/诊断，避免同步等待热路径 |
| alias 未发生或结果错误 | 生命周期重叠、descriptor/usage、external alias group | 先修正声明；不要强行复用不兼容资源 |

- [ ] 每个 pass 的 GPU 读写都有版本、范围与 intent，最终输出是存活根。
- [ ] transient、external、persistent/history 的 owner 和失效策略明确。
- [ ] 没有跨设备或跨 generation 缓存 RHI handle；surface lease 在一次帧流程内消费。
- [ ] 优化前后通过 graph/RHI 诊断和产品缓存统计比较，而不是只看单帧体感。
- [ ] 相关 render graph 或 RHI 测试覆盖资源访问、生命周期或 submission 失败分支。
