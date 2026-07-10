# 17-performance-and-profiling 产出记录归档

> 来源：[`17-performance-and-profiling.md`](../17-performance-and-profiling.md) 的 `## 状态与产出记录`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-23 | Render index 当前状态总览拆分 | PF-M1/PF-M3 部分完成,PF-M2/PF-M4 未启动 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 17 Performance/Profiling 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：CPU parallelization、compile stutter/perf baseline、预算降级阶梯；验收缺口：需要 GPU timestamp/profile hierarchy、perf fixtures、threshold policy、shader warmup reporting |
| 2026-06-15 | PF-M1 observation base | 部分完成: CPU profiling/markers/diagnostics 存在,GPU timestamp 未接 | `RenderStats`、runtime diagnostics、`profile_scope!`、debug markers 和 RenderDoc capture env hook 已存在并被各计划复用;但无 GPU pass timing、frame profile hierarchy 或 graph dump bundle。 | 本文件 `现状与差距` 实读代码列出 CPU profiling、debug marker、RenderDoc hook 已有,且 `timestamp_writes: None` 全面存在。 | 请求 timestamp feature、实现 `GpuPassTimer`、`RenderFrameProfile` 和 graph dump/profile 关联。 |
| 2026-06-15 | PF-M2 CPU parallelization | 未启动: 渲染提交仍串行 | submit/extract/prepare/present 仍在锁内串行,graph stage pass 逐个录制。 | 本文件 `现状与差距` 记录 `submit_frame_extract` 双锁串行和 `execute_graph_stage` for 循环。 | 设计 pipeline extract/prepare、rayon prepare jobs 和安全的并行 command recording。 |
| 2026-06-15 | PF-M3 memory and bandwidth budgets | 部分完成: 统计字段存在,预算/降级缺失 | 多计划已扩展 `RenderStats` 和 diagnostics,能看到 transient bytes、GPUScene upload、reactive mask、HZB readback 等;但无预算、超限降级或 OOM 阶梯。 | 本文件 `现状与差距` 明确统计扁平且无预算防回归;计划 03/04/06/07 状态表记录多项统计面已接入。 | 建立 budget policy、per-pass memory/bandwidth buckets 和 degrade ladder。 |
| 2026-06-15 | PF-M4 compile stutter and performance regression baseline | 未启动: 无性能 CI gate | CI 只有 build/test,无性能基线、无 shader warmup 统计、无 compile stutter 监控。 | 本文件 `现状与差距` 明确无预算与防回归机制;计划 08 MS-M4 也记录 shader variant cache/warmup 未启动。 | 建立 perf fixture、baseline artifacts、threshold policy 和 shader warmup reporting。 |

### 参考实现精读笔记

- **UE `FRealtimeGPUProfiler`**(`RealtimeGPUProfiler.h`):单例 `Get()`,`BeginFrame/EndFrame(FRHICommandListImmediate&)` 圈帧;`PushEvent(GPUMask, Name, Stat, Description)` 返回 `FRealtimeGPUProfilerQuery`,查询对象延迟 `Submit(RHICmdList, bBegin)`,且专门提供 `Discard(bBegin)` 处理"RDG 建了 profiler event 但 pass 被 culling 从未提交"的情况 —— Zircon 对应:`GpuPassTimer` 的 query 槽按实际录制挂载而非按 graph 声明预分配,01 的 pass culling 天然不产生悬空查询。`ActiveFrame` + `TQueue<TUniquePtr<FRealtimeGPUProfilerFrame>> PendingFrames` + `FRenderQueryPoolRHIRef` 即多帧 in-flight 池,与本计划 3 槽延迟环同构;`FRealtimeGPUProfilerHistoryItem`(HistoryCount=64,`AccumulatedTime`)与 `FetchPerfByDescription` 的 Avg/Min/Max 说明 UI 展示要做滑动窗口平滑 —— 我们放诊断层(`render_stats_store`)做,契约层只传单帧。
- **UE `FParallelCommandListSet`**(`SceneRendering.h` line 475):`NewParallelCommandList/AddParallelCommandList` 聚 `QueuedCommandLists`,派生类析构调 `Dispatch` 顺序合并,`MinDrawsPerCommandList` 控制切分粒度;整个类已 `UE_DEPRECATED(5.5, "Use GraphBuilder.AddDispatchPass instead")`。结论直接吸收:不做 pass 内 draw 级并行拆分(wgpu 也没有 secondary command buffer),并行粒度 = graph pass 桶,阈值参数对齐 `min_passes_per_bucket`。
- **Unity `ProfilingSampler`/`ProfilingScope`**(`ProfilingScope.cs`):一个采样名同时驱动三条时间线 —— `cmd?.BeginSample(m_Marker)` 进 CommandBuffer(渲染线程执行时间 + 自动 `MarkerFlags.SampleGPU` 的 GPU 时间)与 `Inl_<name>` inline marker(调用线程 CPU);recorder 懒分配保证不开观测零常驻成本。Zircon 对应:pass_name 同名贯通 `profile_scope!`(CPU)、debug marker(RenderDoc)、`GpuPassTimer`(GPU)三面;`allow_gpu_timing=false` 时 `GpuPassTimer` 不创建任何 wgpu 资源。
- **bevy `pipelined_rendering.rs`**:`RenderAppChannels` 用两条 `async_channel::bounded(1)` 交换 `SubApp` 所有权,render 线程 `loop { recv → render_app.update() → send back }`;`renderer_extract` 在主线程收回 render app 后才跑 extract(extract 是唯一双世界同步点,对应 `ExtractSchedule`);`Drop for RenderAppChannels` 里 `recv_blocking` 等 render world 归还,保证 non-send 数据在正确线程析构。Zircon 简化:我们传的不是 world 而是 `RenderFrameExtract` 值快照(契约已禁渲染侧访问 ECS),无需 world 往返,单向 frame 通道 + 反向 feedback 通道即可;线程退出排空语义照抄。

## 风险与回退

| 风险 | 缓解 / 回退 |
|------|------------|
| TIMESTAMP_QUERY 在部分后端(GL/老驱动)不可用或精度差 | 能力 gate 已是 None 语义;profile 其余计数维度不受影响;验收明确"无能力平台零 panic" |
| timestamp 读取引入气泡或与 present 抢提交 | resolve/copy 合入帧末既有 encoder,读取走 3 槽非阻塞环;`allow_gpu_timing=false` 一键全关(不创建资源) |
| pipelined 模式暴露隐藏的线程亲和假设(surface/窗口句柄、Drop 顺序) | 回退开关默认关,逐平台开;Drop 排空语义照 bevy;feedback 滞后语义有契约测试,消费方(拾取/回读)按 N-1 帧语义改造在同一里程碑内完成 |
| 并行录制收益不足(pass 数少、单 pass 过重) | `min_passes_per_bucket` 阈值退化单 encoder;收益依赖 02 命令化后录制成本下降,故 PF-M2 排在 01/02 之后 |
| rayon 并行引入排序不确定性导致产物抖动 | 确定性归并(按 sort_key 稳定序,位段归 09)先于并行开启;产物对拍测试在开关矩阵上跑 |
| 降级阶梯与用户显式画质设置打架 | 阶梯只作用于"预算超标"运行态,且 `degrade_step_active` 进 profile 可解释;用户设置为底,阶梯只降不改底 |
| 异步编译占位造成首帧画面缺 draw 被误判为 bug | 占位策略仅两档且进 stats(`last_pipeline_async_pending_count`);`allow_async_compile=false` 回退同步编译 |
| 基线上限定得过松(永远绿)或过紧(噪声红) | 只断言确定性计数不断言时间;每个基线测试配一个"注入劣化必须红"的反向用例防失效 |
| `RenderStats` 双形态(扁平 + 层级)长期并存漂移 | `render_perf_frame_profile_matches_flat_stats` 自洽断言常驻;扁平字段新增项必须同步进层级聚合,否则该测试红 |

回退总开关:`allow_gpu_timing` / `allow_pipelined_render` / `allow_parallel_record` / `allow_async_compile` 四开关相互独立,任意组合下 `render_product_*` 产物一致是 PF-M2/M4 的硬验收;全关即与本计划落地前行为等价(观测字段为空,不影响渲染正确性)。
