---
related_code:
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/environment.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler/gpu_resolution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ProfilingDebugging/RealtimeGPUProfiler.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneRendering.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/tracy_gpu.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs
  - dev/Fyrox/fyrox-impl/src/renderer/stats.rs
plan_sources:
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
---

# 计划 17:性能体系与优化(profiling / 并行 / 预算 / 防回归)

本计划是骨架层与能力层之上的横切层:为计划 01–16 的产出提供统一的观测底座、CPU 并行化骨架、内存/带宽预算治理与性能回归基线。跨计划契约名一律原样引用、只消费不重定义:计划 01 `RgTextureHandle`/`TransientResourcePool`/`CompiledGraphCache`;计划 02 `MeshDrawCommand`/`CachedMeshDrawCommands`;计划 03 `GpuScene`/`IndirectDrawBatcher`;计划 04 `ViewVisibilityContext`/`HzbBuilder`;计划 08 `ShaderVariantKey`;计划 09 `CameraRenderDescriptor`;计划 16 `ComputePassDescriptor`/`GpuReadbackQueue`。index.md §6 全局边界约束与 §8 全局工程约定全部适用;`sort_key` 位段归计划 09,本计划不触碰。

2026-08-30 standalone UI submission/completion owner增量：Runtime90 SUI-0至SUI-3已把standalone/offscreen device收敛到同一initial profile factory，native UI context必有typed `Arc<WgpuRenderDevice>`；standalone present删除raw submit并保留真实ticket，local frame在surface acquire前唯一poll，旧readback只作after-poll collect。UI image pin随native packet进入ticket-keyed有界退休表，由唯一submission completion callback或fault terminalization锁外释放；device poll error同批终结submission、diagnostic和surface frame。当前只有failing-first/static contract、精确rustfmt、scoped diff、locked metadata与结构证据；真实窗口、PNG/RDC、300帧profile、显存和功耗仍待完成，状态`render17_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`。

2026-08-30 product raw queue authority增量：Runtime90 PFO-4d4b删除scene resource/material frame preparation的无行为queue透传，并让product GPU timer从唯一`WgpuRenderDevice`读取启动时固化的timestamp period。该切片只收窄权限，不改变upload batch、native submit、query routing或缓存算法；failing-first 0/5转为扩展合同7/7。其余raw Device/Queue consumer、真实WGPU、PNG/RDC、300帧profile、显存和功耗仍待完成，状态`render17_pfo_4d4b_source_implemented_static_checks_passed_dynamic_validation_pending`。

## 目标

1. **观测底座(A)**:pass 级 wgpu timestamp query、分层 `RenderFrameProfile`(frame → pass → 子系统)、debug marker 与 render graph 节点名对齐、RenderDoc 抓帧钩子标准化、1080p 中档参考帧预算表。这是其它计划 stats 验收的公共依赖。
2. **CPU 并行化(B)**:extract 双缓冲与 sim/render 两帧重叠(bevy pipelined rendering 同型)、prepare/queue 的 rayon 并行、按 pass 分桶的多 `CommandEncoder` 并行录制与顺序合并提交;全部带回退开关。
3. **内存与带宽预算(C)**:瞬态池/staging 总预算与统计、超预算降级阶梯(顺序定稿)、attachment load/store lint 统计、G-buffer/HDR 中间格式带宽账本。
4. **编译卡顿治理与防回归(D)**:pipeline 异步编译与占位策略、计划 08 磁盘变体缓存预热衔接、wgpu `PipelineCache` 能力 gate、冷启动测量;`render_perf_*` 确定性计数断言进 `cargo test`,时间类指标只观测不断言。

## 现状与差距

基于实读代码的现状盘点:

- **统计是平铺的、CPU 侧的**:`RenderStats`(`zircon_runtime/src/core/framework/render/backend_types.rs`)是一个百余字段的扁平 `last_*` 结构(`last_mesh_draw_count`、`last_graph_executed_pass_count`、`last_graph_transient_texture_bytes_reserved` 等),由提交尾部的 `update_stats`(`submit_frame_extract/update_stats/update.rs`,旁挂 base/particle/hybrid_gi/virtual_geometry 等分文件)一次性写入,再经 `core/runtime/diagnostics/render_stats_store.rs` 镜像给诊断层。没有 frame → pass → 子系统的层级结构,没有任何 GPU 耗时维度。
- **完全没有 GPU timestamp**:Grep 全 `zircon_runtime/src`,所有 render/compute pass 创建点(`viewport_surface.rs`、`execute_lighting.rs`、`record_gbuffer_geometry.rs`、`mesh_motion_vector.rs` 等)一律 `timestamp_writes: None`;设备请求处未启用 `TIMESTAMP_QUERY` 系能力。pass 到底花了多少 GPU 时间,目前只能靠 RenderDoc 离线看。
- **CPU profiling 已有基础**:`profile_scope!` / `profile_dynamic_scope!` 宏(`core/runtime/diagnostics/profiling/scope.rs` 的 `ProfileScope::enter(stream, category, name)`)已埋进 `submit_frame_extract` 主路径(`build_submission_context`、`prepare_runtime_submission`、`render_frame_with_pipeline` 等)与 `execute_graph_stage` 的 stage 维度。本计划复用该体系,不另造 CPU 打点框架。
- **debug marker 已有但两套粒度**:`graphics/debug_markers.rs` 有固定 stage 常量(`zircon::Prepass`、`zircon::MainScene` 等,带 `REQUIRED_RENDERDOC_STAGE_MARKERS` 测试)与 graph 节点前缀 `zircon::RenderGraphPass::<pass_name>`(`marker_for_render_graph_pass`);`execute_graph_stage` 两者都打。计划 01 的 graph dump 落地后,marker 与 dump 的 pass 名对拍尚无测试闭环。
- **RenderDoc 钩子已存在**:`ZR_RENDERDOC_CAPTURE_NEXT` 环境变量(`graphics_debugger_capture/environment.rs`)触发单帧捕获,生命周期由 `begin_graphics_debugger_capture` / `finish_active_capture_and_relock` 管理(见 `submit/submit.rs`)。本计划收编该钩子:capture 帧自动附带 frame profile 与 graph dump,不改触发语义。
- **渲染全链单线程**:`submit_frame_extract` 在 `server.lock_operation()` + `lock_state()` 双锁下从 extract 到 present 串行执行;`execute_graph_stage` 对 stage 内 pass 逐个 for 循环录制。rayon 已在工作区内(`zircon_runtime/Cargo.toml` rayon 1.11;`core/runtime/tasks/pool.rs` 的 `TaskPool` 封装 `rayon::ThreadPool`,提供 `join`/`install`;ECS 有 `schedule_parallel_executor.rs`),但渲染路径零并行。
- **没有预算与防回归机制**:超预算无降级、无 OOM 阶梯;CI(`.github/workflows/ci.yml`)只跑 build + test,没有任何性能维度的回归围栏。

差距汇总:观测(无 GPU 耗时、统计无层级)→ 并行(单线程提交)→ 预算(无上限无降级)→ 防回归(无基线)四块全部缺位,而 01–16 各计划的 stats 验收都需要本计划的观测底座先行。

性能审查补充边界：`rhi_wgpu::WgpuRenderDevice`当前调用图只服务RHI契约测试，内部是全局mutex下的CPU资源模拟，并非本计划要测的产品GPU backend。其公开命名/capability与未来接入风险见`17/failure-2026-07-18-rhi-wgpu-submit-validation-and-copy-clones.md`；PF-M1前必须锁定唯一产品wgpu owner，并选择test-only硬收口或真实backend替换，禁止把模拟device wall-clock当GPU基线。

F4 native UI presenter的逐文件审查还发现per-present command sort、三层geometry Vec复制、GPU vertex-buffer重建、逐command Advanced text shaping、原每DrawOp一pass与surface/cache双录制；见`17/failure-2026-07-18-ui-surface-per-present-geometry-text-and-pass-rebuild.md`。PF-M1/PF-M2需把compiled UI generation、persistent upload、pass counters/timestamps与single authoritative projection纳入验收；局部连续non-text pass合并不能替代该收敛。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ProfilingDebugging/RealtimeGPUProfiler.h`(实现在同目录 `Private/.../RealtimeGPUProfiler.cpp`) | `FRealtimeGPUProfiler` 的 `BeginFrame/EndFrame` + `PushEvent/PopEvent` 帧内事件栈;`PendingFrames` 多帧 in-flight 查询池与 `FRealtimeGPUProfilerQuery::Discard`(pass 被 culling 时丢弃查询);`FRealtimeGPUProfilerHistoryItem` 64 帧滑动窗口与 `FetchPerfByDescription` 的 Avg/Min/Max 平滑 |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneRendering.h` | `FParallelCommandListSet`(line ~475):per-view 并行命令列表集合、`MinDrawsPerCommandList` 切分阈值、析构期 `Dispatch` 顺序合并;注意它已被标记 `UE_DEPRECATED(5.5, "Use GraphBuilder.AddDispatchPass instead")` —— UE 自己也把并行录制收编进 RDG pass 粒度,与 wgpu 的现实约束同向 |
| `dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs` | `ProfilingSampler` 单名字三联打点:主 marker 进 CommandBuffer(`cmd?.BeginSample(m_Marker)`,自动附 `MarkerFlags.SampleGPU`)+ `Inl_` 前缀 inline marker 记调用线程 CPU;recorder 懒分配零常驻开销 |
| `dev/bevy/crates/bevy_render/src/pipelined_rendering.rs` | `PipelinedRenderingPlugin` 的 sim/render 两帧重叠:`RenderAppChannels` 两条 bounded(1) 通道交换 `SubApp` 所有权,render 线程 loop(recv → update → send back),`renderer_extract` 在主线程等回 render app 再跑 extract;`Drop` 里 `recv_blocking` 保证 non-send 数据在正确线程析构 |
| `dev/bevy/crates/bevy_render/src/lib.rs` | `ExtractSchedule` 的定位:extract 是唯一同时摸两个 world 的同步点,extract 命令延迟到 render 侧应用以便主世界尽早开跑 |

次参考:`dev/tracy`(CPU profile 导出格式可对接,既有 `profiling/export.rs` 已具备);wgpu `Features::TIMESTAMP_QUERY` / `Queue::get_timestamp_period` 文档语义。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_render/src/diagnostic/internal.rs` | PF-M1 `GpuPassTimer` 直接对应:`TIMESTAMP_QUERY` 能力 gate、`QuerySet` 池、`get_timestamp_period` 换算、多帧 in-flight 槽轮转 + mapped 回调 | timestamps_query_set 创建分支、`supports_timestamps_inside_passes/encoders` 能力位区分(本计划只需 pass 边界档)、submitted_frames 轮转的非阻塞读取 |
| `dev/bevy/crates/bevy_render/src/diagnostic/mod.rs` | pass/time span 守卫式打点 API:同一 pass 名贯通 CPU/GPU 两面 | `RecordDiagnostics` trait、`PassSpanGuard`/`TimeSpanGuard` 的 begin/end 配对(对照 `pass_timestamp_writes` 挂载点) |
| `dev/bevy/crates/bevy_render/src/diagnostic/tracy_gpu.rs` | Tracy GPU 帧打点接入(注:`dev/tracy` 目录仅含 profiler 可执行文件、无 C++ 源码,Tracy client 概念经本文件的 `tracy_client` 集成参照) | `new_tracy_gpu_context` 的后端→`GpuContextType` 映射与 timestamp 校准 |
| `dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs` | 诊断值滑动窗口/EMA 平滑 —— 精读笔记"平滑放诊断层、契约层只传单帧"结论的 Rust 实绩 | `Diagnostic` 的 history 队列与 `ema_smoothing_factor` 增量更新 |
| `dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs` | frame time/fps 诊断源的最小形态(`render_stats_store` 镜像消费面的对照) | `diagnostic_system` 的逐帧测量写入 |
| `dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs` | PF-M4 `PipelineAsyncCompiler` 对应:异步 pipeline 编译状态机 + 同步回落档 | `CachedPipelineState::{Queued,Creating,Ok}` 状态推进、`create_pipeline_task`、`block_on_render_pipeline`(必须立即可用时的阻塞档)、关闭异步编译的开关位 |
| `dev/Fyrox/fyrox-impl/src/renderer/stats.rs` | 渲染统计分层聚合的 Rust 形态(对照 `RenderFrameProfile` 的子系统层) | `Statistics::begin_frame/end_frame/finalize` 帧圈定、`LightingStatistics`/`SceneStatistics` 子结构拆分 |

## 目标架构

### A. 观测底座(GpuPassTimer + RenderFrameProfile)

- `GpuPassTimer`(graphics 层,持 wgpu):`TIMESTAMP_QUERY` 能力 gate;每 graph pass 一对 query(begin/end)经 `RenderPassTimestampWrites`/`ComputePassTimestampWrites` 挂载 —— 不需要 `TIMESTAMP_QUERY_INSIDE_PASSES`,pass 边界粒度即可;帧末 `resolve_query_set` → copy 到 MAP_READ 小环(FRAMES_IN_FLIGHT=3 槽,N 帧延迟非阻塞读取)。该小环先独立实现,计划 16 `GpuReadbackQueue` 落地后在 PF-M1 收尾切片内迁移为其消费方(同型 staging 思路,迁移是硬切换)。
- `RenderFrameProfile`(framework 契约层,纯 POD 无 wgpu):frame → pass → 子系统三层;pass 条目带 `gpu_time_us`(能力缺失为 None)、draw 数、实例数、状态切换数(02 replayer 统计)、上传字节、dispatch 数;frame 级带瞬态内存峰值(01 `TransientResourcePool`)、staging 合计(03/13/16 ring)、`profile_latency_frames`。既有扁平 `last_*` 字段保留为兼容消费面,层级 profile 是新增权威,二者由同一 `update_stats` 写入,数值必须自洽(测试对拍)。
- marker 对齐:graph pass 的 marker 唯一来源是 `marker_for_render_graph_pass(pass_name)`,pass_name 与计划 01 `CompiledRenderGraph::dump()` 输出、`RenderFrameProfile.passes[].pass_name` 三方一致;固定 stage 常量保留为外层分组 marker。
- RenderDoc 收编:`ZR_RENDERDOC_CAPTURE_NEXT` 触发语义不变;capture 结果附带当帧 `RenderFrameProfile` 序列化文本 + graph dump,经既有 capture 查询路径返回。
- 帧预算表:`RenderFrameBudget::reference_1080p_mid()` 给出参考档位(见工程落地细化的观测点表),预算超标只产生 stats 告警计数,不在 PF-M1 触发降级(降级归 PF-M3)。

### B. CPU 并行化(pipelined extract + rayon prepare + 并行录制)

- **sim/render 两帧重叠**:bevy 通道模型同型。runtime 侧 `submit_frame_extract` 的调用退化为"把 `(viewport, RenderFrameExtract, Option<UiRenderExtract>)` 投入 bounded(1) 通道";render 线程独占 `WgpuRenderFramework` 状态执行现有提交体。所有权模型:extract 本来就是值传递快照(契约即 index.md §6 第 6 条),天然适合跨线程;feedback(`collect_runtime_feedback` 产物)变为滞后一帧回流,语义变化显式进契约注释与测试。回退开关:`pipelined_render` 关闭时通道退化为同步直调,行为与现状逐字节一致。
- **prepare/queue 并行**:02 的批次 → `MeshDrawCommand` 转换按 pass processor 维度 rayon 并行;04 落地后再叠加 per-`ViewVisibilityContext` 维度并行。线程池策略:统一走 `core/runtime/tasks/pool.rs` 的 `TaskPool`(与计划 04 并行剔除共享同一池),禁止渲染模块私建 `ThreadPoolBuilder`。
- **并行命令录制**:wgpu 没有 secondary command buffer,也不允许多线程往同一 `CommandEncoder` 写 —— 因此并行粒度固定为 pass:按 graph 拓扑分层把 pass 分桶,每桶一个 `CommandEncoder` 在 rayon 任务里录制,完成后按 graph 拓扑序合并成 `Vec<CommandBuffer>` 单次 `Queue::submit`。重 pass(base pass)内部 draw 循环保持串行,靠 02 重放器的低单 draw 开销兜底;这与 UE 把 `FParallelCommandListSet` 废弃、收编进 RDG dispatch pass 的方向一致。timestamp_writes 按 pass 挂,与并行录制天然正交。

### C. 内存与带宽预算

- 预算对象三类:瞬态池(01 `TransientResourcePool` 的纹理/缓冲峰值)、staging 总量(03 `GpuSceneStagingRing` + 13 上传环 + 16 `GpuReadbackQueue` ring 的合计上限)、attachment 带宽(load/store 实际行为)。
- 降级阶梯顺序定稿(固定、不可配置重排):① render scale 降档(1.0 → 0.85 → 0.7,走计划 07 动态分辨率);② 全局 mip bias +1(走计划 13);③ 关可选 feature(固定顺序 SSR → SSAO → contact shadow → bloom 高档,经 RenderFeature descriptor 关闭,compiled graph 即不含对应 pass)。升档迟滞 N 帧防抖。
- load/store lint:在计划 01 首写 ops 决策表校验之上,统计"本可 DontCare 却用了 Store/Load"的 pass-attachment 对(终读后仍 Store、首写前 Load 等),进 `RenderFrameProfile` 的 lint 计数,不阻断编译。
- 带宽账本:按计划 07 定稿的 G-buffer/HDR 中间格式逐 attachment 记每像素字节数与每帧读写次数,得出理论带宽;与 timestamp 实测互为印证(见观测点表)。

### D. 编译卡顿治理与性能回归基线

- pipeline 异步编译:`ensure_pipeline_for_variant`(02/08 已定)miss 时不阻塞当帧 —— 占位策略定稿为 `SkipDraw`(默认,该 draw 当帧不渲染)与 `DepthOnly`(仅深度,供 prepass 链)两档;绝不渲染错误材质;编译完成帧自动补回。
- 预热衔接:计划 08 `ShaderVariantCache` 磁盘缓存与 prewarm 清单是输入;本计划补"启动期预热钩子"(加载屏/Hub 启动阶段消费 prewarm 清单批量编译)与首帧 miss 计数验收。
- wgpu `PipelineCache`:能力 gate(目前仅 Vulkan 后端有效),命中时设备级缓存落盘随 08 磁盘缓存同目录;不可用平台静默跳过。
- 冷启动测量:计划 01 `CompiledGraphCache` 的冷/热编译耗时与命中计数进 profile,作为启动优化的观测面。
- 回归基线形态定稿:`render_perf_*` 测试只断言确定性计数(draw 数上限、状态切换数上限、上传字节上限、瞬态峰值上限、graph 编译次数),时间类(ms)指标一律只进观测导出不进断言;`render_perf_*` 确定性计数测试通过 focused 批次随里程碑一起验证（policy §3），计时型基线建议另开手动触发 workflow 导出 profile 文本工件，不设阈值门禁；全量 workspace 回归留给波次收口（policy §4）。

## 里程碑

依赖与并行性:PF-M1 不依赖其它计划,可与阶段 A(01/02)并行启动,且是各计划 stats 验收的依赖项,**最先做**;PF-M2 依赖 01/02 落地;PF-M3 依赖 01(池 stats)与 03(ring);PF-M4 依赖 08(变体缓存)与 01(graph 缓存),回归基线部分依赖 PF-M1。

### PF-M1 观测底座

实施切片:
1. `GpuPassTimer` + 能力 gate + 3 槽延迟读取小环;graph 执行循环挂 pass 级 `timestamp_writes`;`Queue::get_timestamp_period` 换算 us。
2. `RenderFrameProfile` 契约类型 + `update_stats` 写入层级结构(与扁平字段同源自洽);`render_stats_store` 镜像扩展。
3. marker 对齐收口:pass marker 统一经 `marker_for_render_graph_pass`,与 graph dump、profile pass 名三方一致;RenderDoc capture 附带 profile 文本(收编 `ZR_RENDERDOC_CAPTURE_NEXT`)。
4. `RenderFrameBudget::reference_1080p_mid()` 预算表 + 超标告警计数;(收尾,待 16 CN-M1)timer 读取环迁移到 `GpuReadbackQueue`。

测试阶段:
- `cargo test -p zircon_runtime render_perf --locked` + `cargo test -p zircon_runtime render_debugger --locked` 回归
- 验收证据:无 timestamp 能力的 adapter 上 profile 的 `gpu_time_us` 全 None 且零 panic;有能力平台 pass 耗时非零且 `profile_latency_frames <= 3`;capture 帧附带的 pass 名与 graph dump 完全一致(断言)。

### PF-M2 CPU 并行化

实施切片:
1. pipelined extract:render 线程 + bounded(1) 通道 + feedback 滞后一帧回流;`pipelined_render` 回退开关(关闭 = 同步直调)。
2. prepare/queue rayon 并行(pass processor 维度,经共享 `TaskPool`);并行前后 `MeshDrawCommandList` 排序结果逐元素一致(确定性归并)。
3. 并行命令录制:graph 拓扑分桶 → 每桶独立 `CommandEncoder` → 拓扑序合并单次 submit;`parallel_record` 开关与最小桶阈值。

测试阶段:
- `cargo test -p zircon_runtime render_perf --locked` + `cargo test -p zircon_runtime mesh_pass --locked` 回归
- 验收证据:三开关任意组合下 `render_product_*` 产物对拍逐像素一致;并行开启时提交的 command buffer 顺序与 graph 拓扑序一致(断言);pipelined 模式 feedback 滞后语义单测。

### PF-M3 内存与带宽预算

实施切片:
1. 预算配置类型 + 瞬态池/staging 合计统计接入 profile;超预算告警计数。
2. 降级阶梯(scale → mip bias → feature off)状态机 + 迟滞;接 07 动态分辨率与 13 mip bias 的既有入口。
3. load/store lint 统计(01 ops 校验之上);带宽账本(07 格式定稿后填实数值)。

测试阶段:
- `cargo test -p zircon_runtime render_perf --locked` + `cargo test -p zircon_runtime render_graph --locked` 回归
- 验收证据:人工压低预算后阶梯按固定顺序逐级触发且 stats 可解释;lint 在故意的"终读后 Store"用例上计数为 1;恢复预算后迟滞 N 帧再升档(断言)。

### PF-M4 编译治理与回归基线

实施切片:
1. `PipelineAsyncCompiler` + `SkipDraw`/`DepthOnly` 占位策略;首帧 miss 计数;补回帧验证。
2. 08 prewarm 清单的启动期预热钩子;wgpu `PipelineCache` 能力 gate 接入;01 `CompiledGraphCache` 冷启动计数进 profile。
3. `render_perf_*` 基线测试族定稿(计数断言上限进 `cargo test`);CI 接入建议文档化(现有 ci.yml 不动)。

测试阶段:
- `cargo test -p zircon_runtime render_perf --locked` 全族 + `cargo test -p zircon_runtime shader_variant --locked` 回归
- 验收证据:异步编译开启时首帧无错误材质画面(产物对拍:占位 = 无该 draw);预热后二次启动首帧变体 miss 计数为 0;基线测试在标准测试场景上全绿且对故意 +1 draw 的注入用例报红。

## 工程落地细化

本章是计划 17 的实施权威(index.md §8 第 7 条)。bind group 槽位、GPU 数据布局、测试命名等全局约定直接引用 index.md §8,不重定义;facade 固定 `zircon_runtime::core::framework::render`,契约层零 wgpu,全部硬切换,观测与并行不得绕过 render graph。

### 模块与文件落点

新增文件:

| 路径 | 职责(一行) |
|------|------------|
| `zircon_runtime/src/core/framework/render/frame_profile.rs` | `RenderFrameProfile`/`RenderPassProfileEntry`/`RenderSubsystemProfileEntry`/`RenderBudgetKey`/`RenderFrameBudget` 契约类型(纯 POD,无 wgpu),含 `#[cfg(test)] mod tests` |
| `zircon_runtime/src/graphics/backend/render_backend/gpu_pass_timer/mod.rs` | wiring:`gpu_pass_timer` 模块声明与受控导出 |
| `zircon_runtime/src/graphics/backend/render_backend/gpu_pass_timer/gpu_pass_timer.rs` | `GpuPassTimer`:QuerySet 池、resolve、3 槽 MAP_READ 延迟读取环(PF-M1 收尾迁 `GpuReadbackQueue`) |
| `zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs` | `FrameProfiler`:CPU 计数 + GPU timer 结果按 pass 聚合为 `RenderFrameProfile`,挂 `RenderFrameworkState` |
| `zircon_runtime/src/graphics/runtime/render_framework/pipelined/mod.rs` | wiring:pipelined 子模块声明 |
| `zircon_runtime/src/graphics/runtime/render_framework/pipelined/render_thread.rs` | render 线程生命周期、bounded(1) 通道、同步直调回退路径 |
| `zircon_runtime/src/graphics/runtime/render_framework/pipelined/feedback.rs` | `RenderThreadFeedback` 滞后一帧回流载体与排空语义 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/parallel_encoder_set.rs` | `ParallelEncoderSet`:graph 拓扑分桶、并行录制、拓扑序合并 |
| `zircon_runtime/src/graphics/runtime/render_framework/budget/mod.rs` | wiring:budget 子模块声明 |
| `zircon_runtime/src/graphics/runtime/render_framework/budget/memory_budget.rs` | `RenderMemoryBudget`:瞬态/staging 上限配置与超标判定 |
| `zircon_runtime/src/graphics/runtime/render_framework/budget/degrade_ladder.rs` | `BudgetDegradeLadder`:固定顺序降级状态机 + 迟滞 |
| `zircon_runtime/src/render_graph/store_lint.rs` | load/store lint:基于资源生命周期(first/last pass)的"可 DontCare 未 DontCare"统计(规划层,无 wgpu) |
| `zircon_runtime/src/graphics/pipeline/async_compile.rs` | `PipelineAsyncCompiler` + `PipelinePlaceholderPolicy`(SkipDraw/DepthOnly) |
| `zircon_runtime/src/graphics/pipeline/pipeline_cache_gate.rs` | wgpu `PipelineCache` 能力 gate 与落盘路径(随 08 磁盘缓存同目录) |
| `zircon_runtime/src/graphics/tests/render_perf_baseline.rs` | `render_perf_*` 基线测试族(标准测试场景计数断言) |

修改文件:

| 路径 | 改动点 |
|------|--------|
| `zircon_runtime/src/core/framework/render/mod.rs` | 仅 wiring:声明 `frame_profile` 模块并导出契约类型 |
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `RenderStats` 增 `last_frame_profile: RenderFrameProfile`、`last_budget_warning_count`、`last_store_lint_count`、`last_pipeline_async_pending_count`、`last_variant_first_frame_miss_count`;features 增 `allow_gpu_timing`/`allow_pipelined_render`/`allow_parallel_record` 三开关(沿用 `with_async_compute` 形态) |
| `zircon_runtime/src/graphics/backend/render_backend/request_device.rs` | 申请 `wgpu::Features::TIMESTAMP_QUERY`(可选位,失败降级);`RenderCapabilitySummary` 增 `supports_gpu_timestamp: bool` |
| `zircon_runtime/src/graphics/debug_markers.rs` | `marker_for_render_graph_pass` 保持唯一 pass marker 入口;固定 stage 常量降级为外层分组 marker(文档注释说明,不删常量与测试) |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs` | pass 循环挂 `GpuPassTimer::pass_timestamp_writes`;并行模式改经 `ParallelEncoderSet`(串行模式保留原循环) |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs` | 帧末 `gpu_pass_timer.resolve_and_copy(encoder)`;`try_collect` 结果交 `FrameProfiler` |
| `zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/render_framework_state.rs` | `RenderFrameworkState` 增 `frame_profiler: FrameProfiler`、`memory_budget: RenderMemoryBudget`、`degrade_ladder: BudgetDegradeLadder` 字段 |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs` | pipelined 开启时本函数体整体迁至 render 线程执行(入口只投递);串行模式路径不变 |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/update.rs` | 写入 `last_frame_profile` 与预算/lint/编译计数;层级与扁平字段同源自洽 |
| `zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/mod.rs`(及状态文件) | capture 帧附带 `RenderFrameProfile` 序列化文本 + graph dump |
| `zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs`(及分文件) | 镜像层级 profile 给诊断/编辑器消费 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs`(族) | miss 路径接 `PipelineAsyncCompiler`(占位策略),同步路径保留为 `allow_async_compile=false` 档 |
| `zircon_runtime/src/render_graph/mod.rs` | 仅 wiring:声明 `store_lint` 模块 |
| `tools/zircon_build.py` | 预热钩子:staged 启动脚本消费 08 prewarm 清单(衔接 08 的 `--prewarm-shaders`,不重复实现) |

### 核心类型与接口

契约层(`core/framework/render/frame_profile.rs`,可序列化、无 wgpu):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderBudgetKey {
    Shadow, DepthPrepass, Hzb, GpuSceneUpdate, BasePass, LightGrid,
    DeferredLighting, Ssao, Transparent, PostProcess, TemporalAa, Ui, Other,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderPassProfileEntry {
    pub pass_name: String,            // = graph 节点名 = marker 后缀 = dump 名(三方一致)
    pub executor_id: String,
    pub budget_key: RenderBudgetKey,  // pass → 子系统聚合键
    pub gpu_time_us: Option<u64>,     // 能力缺失 None;数值滞后 profile_latency_frames 帧
    pub draw_count: u32,              // 02 MeshDrawReplayStats 注入
    pub instance_count: u32,          // 03 IndirectDrawBatcher 合批后实例数
    pub state_change_count: u32,      // pipeline + bind group 切换(02 重放器去重统计)
    pub upload_bytes: u64,            // 本 pass 归因的 staging/直写字节
    pub dispatch_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderSubsystemProfileEntry {
    pub key: RenderBudgetKey,
    pub gpu_time_us: Option<u64>,     // 同 key pass 求和
    pub budget_us: u64,               // 来自 RenderFrameBudget
    pub over_budget: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderFrameProfile {
    pub frame_generation: u64,
    pub gpu_frame_time_us: Option<u64>,
    pub cpu_submit_time_us: u64,            // submit_frame_extract 全程(profile_scope 同源)
    pub profile_latency_frames: u32,        // GPU 时间数据的滞后帧数(<= 3)
    pub passes: Vec<RenderPassProfileEntry>,
    pub subsystems: Vec<RenderSubsystemProfileEntry>,
    pub transient_texture_peak_bytes: u64,  // 01 TransientResourcePool stats
    pub transient_buffer_peak_bytes: u64,
    pub staging_total_bytes: u64,           // 03 GpuSceneStagingRing + 13 上传环 + 16 GpuReadbackQueue 合计
    pub compiled_graph_cache_hit: bool,     // 01 CompiledGraphCache
    pub variant_miss_count: u32,            // 08 ShaderVariantKey 解析 miss
    pub store_lint_count: u32,
    pub budget_warning_count: u32,
    pub degrade_step_active: u32,           // 0 = 未降级
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFrameBudget { /* Vec<(RenderBudgetKey, u64 /*us*/)> + total_us */ }
impl RenderFrameBudget {
    pub fn reference_1080p_mid() -> Self; // 见观测点表
    pub fn budget_us(&self, key: RenderBudgetKey) -> u64;
}
```

graphics 实现层(持 wgpu,不出 graphics):

```rust
// graphics/backend/render_backend/gpu_pass_timer/gpu_pass_timer.rs
pub(crate) struct GpuPassTimer {
    query_set: wgpu::QuerySet,        // TIMESTAMP,容量 = 2 * max_passes
    resolve_buffer: wgpu::Buffer,     // QUERY_RESOLVE | COPY_SRC
    readback_slots: [TimerSlot; 3],   // MAP_READ;N 帧延迟轮转(PF-M1 收尾迁 GpuReadbackQueue)
    period_ns_per_tick: f32,          // Queue::get_timestamp_period
    pass_names: Vec<String>,          // 帧内按挂载序登记,与 query 下标对应
}
impl GpuPassTimer {
    pub(crate) fn try_new(device: &wgpu::Device, queue: &wgpu::Queue, max_passes: u32) -> Option<Self>; // 能力 gate
    pub(crate) fn begin_frame(&mut self, frame_generation: u64);
    /// 每个 graph pass 录制前调用;返回 None 表示容量耗尽(只丢观测不丢渲染)
    pub(crate) fn render_pass_writes(&mut self, pass_name: &str) -> Option<wgpu::RenderPassTimestampWrites<'_>>;
    pub(crate) fn compute_pass_writes(&mut self, pass_name: &str) -> Option<wgpu::ComputePassTimestampWrites<'_>>;
    pub(crate) fn resolve_and_copy(&mut self, encoder: &mut wgpu::CommandEncoder);
    /// 非阻塞:返回最近一个已完成帧的 (frame_generation, Vec<(pass_name, gpu_time_us)>)
    pub(crate) fn try_collect(&mut self) -> Option<GpuTimerFrameResult>;
}

// graphics/runtime/render_framework/pipelined/render_thread.rs
pub(crate) struct RenderThreadHandle {
    submit_tx: std::sync::mpsc::SyncSender<RenderThreadFrame>, // bounded(1):sim 最多领先 1 帧
    feedback_rx: std::sync::mpsc::Receiver<RenderThreadFeedback>,
    join: Option<std::thread::JoinHandle<()>>,
}
pub(crate) struct RenderThreadFrame {
    pub(crate) viewport: RenderViewportHandle,
    pub(crate) extract: RenderFrameExtract,
    // UI paint products are generation-owned and shared across session/queue/render stages.
    pub(crate) ui: Option<Arc<UiRenderExtract>>,
}
// Drop 语义对齐 bevy:先关 submit_tx,recv 排空 feedback,join 线程,
// 保证 wgpu 资源在 render 线程析构。

// graphics/scene/scene_renderer/graph_execution/parallel_encoder_set.rs
pub(crate) struct ParallelEncoderSet {
    buckets: Vec<EncoderBucket>,      // 桶 = 连续拓扑层切片;桶间无资源写后读跨越
    min_passes_per_bucket: usize,     // 低于阈值退化为单 encoder(对齐 UE MinDrawsPerCommandList 思路)
}
impl ParallelEncoderSet {
    pub(crate) fn partition(compiled: &CompiledRenderGraph, min_passes_per_bucket: usize) -> Self;
    /// pool = core::runtime::tasks::TaskPool(与计划 04 共享);返回序 = graph 拓扑序
    pub(crate) fn record_parallel<F>(self, pool: &TaskPool, record_bucket: F) -> Vec<wgpu::CommandBuffer>
    where F: Fn(&EncoderBucket, &mut wgpu::CommandEncoder) + Sync;
}

// graphics/runtime/render_framework/budget/degrade_ladder.rs
pub(crate) enum DegradeStep {
    RenderScale(f32),                 // 1.0 → 0.85 → 0.7(07 动态分辨率入口)
    GlobalMipBias(i32),               // +1(13 入口)
    DisableFeature(&'static str),     // 固定序:ssr → ssao → contact_shadow → bloom_high
}
pub(crate) struct BudgetDegradeLadder {
    steps: Vec<DegradeStep>,          // 构造期定稿,运行期只走 active 指针
    active: usize,
    hysteresis_frames: u32,           // 升档迟滞(默认 120 帧)
    frames_under_budget: u32,
}
impl BudgetDegradeLadder {
    pub(crate) fn evaluate(&mut self, profile: &RenderFrameProfile, budget: &RenderMemoryBudget) -> Option<&DegradeStep>;
}

// graphics/pipeline/async_compile.rs
pub(crate) enum PipelinePlaceholderPolicy { SkipDraw, DepthOnly }
pub(crate) struct PipelineAsyncCompiler { /* pending: 按 MeshPipelineVariantId 去重;完成经通道回收 */ }
```

### GPU 数据布局与观测点表

GPU 侧仅 `GpuPassTimer` 一处新增资源:QuerySet(TIMESTAMP,2 × max_passes,默认 max_passes = 64)+ resolve buffer(`8 B × 2 × max_passes`,QUERY_RESOLVE | COPY_SRC)+ 3 个 MAP_READ 槽同尺寸。无新增 storage/uniform 布局,index.md §8 第 2 条不涉及。

观测点 → 来源对照表(profile 字段的唯一数据源,禁止旁路再统计):

| 观测点 | 来源(契约名) | 落点字段 |
|--------|--------------|---------|
| pass GPU 耗时 | `GpuPassTimer`(本计划) | `RenderPassProfileEntry.gpu_time_us` |
| draw 数 / 状态切换数 | 计划 02 `MeshDrawCommand` 重放统计(`CachedMeshDrawCommands` 命中亦计) | `draw_count` / `state_change_count` |
| 实例数 / indirect 批次 | 计划 03 `IndirectDrawBatcher` | `instance_count` |
| 上传字节 | 计划 03 `GpuScene` flush + 13 纹理上传 + 16 readback staging | `upload_bytes` / `staging_total_bytes` |
| 瞬态内存峰值 | 计划 01 `TransientResourcePool` stats | `transient_*_peak_bytes` |
| graph 编译命中 | 计划 01 `CompiledGraphCache` | `compiled_graph_cache_hit` |
| 变体 miss | 计划 08 `ShaderVariantKey` 解析路径 | `variant_miss_count` |
| 可见性/HZB 计数 | 计划 04 `ViewVisibilityContext`/`HzbBuilder`(per-view) | 04 自有扁平字段,PF 不重复 |
| 相机序列 | 计划 09 `CameraRenderDescriptor` 解析结果 | 既有 `last_scene_camera_*`,PF 不重复 |

帧预算参考档位 `reference_1080p_mid()`(1080p、中档独显、60 fps,GPU 总预算 14.0 ms 留 2.6 ms 余量;数值是观测基线非断言):

| RenderBudgetKey | 预算 ms | 说明 |
|---|---|---|
| Shadow | 2.2 | CSM + 本地光 atlas(05) |
| DepthPrepass | 0.7 | early-z(02) |
| Hzb | 0.25 | mip 金字塔 reduce(04) |
| GpuSceneUpdate | 0.4 | 上传 + GPU 剔除(03/04) |
| BasePass | 3.2 | forward 主 pass 或 G-buffer(07 格式定稿) |
| LightGrid | 0.35 | froxel 注入(05) |
| DeferredLighting | 2.2 | deferred 路径;forward 路径并入 BasePass |
| Ssao | 0.8 | 既有 SSAO feature |
| Transparent | 1.2 | |
| PostProcess | 1.6 | uber + bloom + tonemap(07) |
| TemporalAa | 0.7 | TAA resolve(06) |
| Ui | 0.4 | UI pass(既有闭环) |
| Other | 0.0 | 兜底归集,预算 0 即"出现即告警" |

带宽账本记法(07 格式定稿后填实):每 attachment 一行 `格式 × 每像素字节 × 读写次数`,frame 合计与 `gpu_frame_time_us` 实测互证;例:1080p RGBA16F scene color 一写一读 ≈ 2 M px × 8 B × 2 = 33 MB/帧。

### 帧时序与集成点

串行模式(回退档,行为 = 现状 + 观测):

```text
submit_frame_extract
 ├─ build_frame_submission_context(01 CompiledGraphCache 命中走缓存)
 ├─ begin_graphics_debugger_capture(ZR_RENDERDOC_CAPTURE_NEXT 语义不变)
 ├─ prepare_runtime_submission
 ├─ render_frame_with_pipeline
 │   ├─ gpu_pass_timer.begin_frame
 │   ├─ execute_graph_stage × N:每 pass 挂 timestamp_writes + marker_for_render_graph_pass
 │   └─ 帧末 encoder:gpu_pass_timer.resolve_and_copy → present
 ├─ collect_runtime_feedback
 ├─ gpu_pass_timer.try_collect →(滞后 ≤3 帧的)GpuTimerFrameResult
 └─ update_stats:FrameProfiler 聚合 → RenderStats.last_frame_profile + 扁平字段
```

pipelined 模式(PF-M2,bevy 同型):

```text
sim 线程:  | extract N+1 | 投递(bounded(1),满则阻塞=自然背压)| sim N+2 ...
render 线程:| 提交体(frame N):上图全流程 | feedback N 回流(sim 于 N+1 帧首排空)|
```

集成点定稿:
1. timestamp 挂载点唯一在 graph pass 录制处(`execute_graph_stage` 的 pass 循环与 compute 等价路径)——观测不绕过 graph(index.md §6 第 3 条的观测面延伸)。
2. 降级阶梯 `evaluate` 在 `update_stats` 之后、下一帧 `build_frame_submission_context` 之前消费(降级影响下一帧编译输入,不回写当帧)。
3. `PipelineAsyncCompiler` 完成回收在 `prepare_runtime_submission` 入口排空(补回的 variant 当帧生效)。
4. 并行录制桶边界 = 拓扑层边界,跨桶资源依赖天然满足;`ParallelEncoderSet::record_parallel` 输出直接喂单次 `Queue::submit`,提交序测试断言。
5. profile 经 `render_stats_store` 镜像 + 既有 `query_stats` 路径暴露给编辑器/诊断面板;capture 附件经 capture 查询路径返回。

### 实施切片细化

PF-M1(每切片末 `cargo check -p zircon_runtime --lib --locked`):
1. `frame_profile.rs` 契约类型 + `RenderStats.last_frame_profile` 字段 + `update_stats` 空聚合(纯 CPU 计数先通)。
2. `request_device.rs` 申请 TIMESTAMP_QUERY(可选)+ `supports_gpu_timestamp` 能力位;`GpuPassTimer` 实现与单元测试桩。
3. `execute_graph_stage` 挂 `timestamp_writes` + `resolve_and_copy` 接 `render.rs` 帧末;`try_collect` 接 `FrameProfiler`。
4. marker 三方一致收口 + capture 附带 profile/dump;预算表与告警计数。
5. (待 16 CN-M1)读取环迁 `GpuReadbackQueue`,删除私有 slots(硬切换)。

PF-M2:
1. `pipelined/` 模块 + 同步直调回退;`submit.rs` 入口投递化;feedback 滞后契约注释。
2. prepare/queue rayon 化(processor 维度;确定性归并排序断言先写)。
3. `ParallelEncoderSet` + `execute_graph_stage` 并行分支;三开关矩阵产物对拍。

PF-M3:
1. `memory_budget.rs` + profile 接入瞬态/staging 合计。
2. `degrade_ladder.rs` 状态机 + 07/13/feature 三入口接线 + 迟滞。
3. `store_lint.rs`(规划层,消费 01 生命周期数据)+ 带宽账本文档化。

PF-M4:
1. `async_compile.rs` + `ensure_pipeline_for_variant` miss 路径接入 + 占位两档。
2. 预热钩子 + `pipeline_cache_gate.rs` + 冷启动计数。
3. `render_perf_baseline.rs` 基线族 + 标准测试场景固化 + CI 建议文档。

### 测试与验收清单

全部进 `cargo test -p zircon_runtime --lib --locked`(过滤词 `render_perf`),计数断言、零时间断言:

| 测试名 | 断言 |
|--------|------|
| `render_perf_frame_profile_matches_flat_stats` | 层级 profile 的 draw/pass/上传合计与既有扁平 `last_*` 字段自洽 |
| `render_perf_pass_names_match_graph_dump_and_markers` | profile pass 名 = 01 graph dump = `marker_for_render_graph_pass` 后缀,三方一致 |
| `render_perf_gpu_timer_capability_gate` | 无 TIMESTAMP_QUERY 时 `try_new` 返回 None,profile 全 None 且渲染产物不变 |
| `render_perf_gpu_timer_latency_within_three_frames` | mock 设备下 `try_collect` 滞后 ≤ 3 帧且不阻塞 |
| `render_perf_budget_table_covers_all_builtin_passes` | 内建 pass 的 `budget_key` 无一落入 Other(新 pass 忘登记即红) |
| `render_perf_draw_count_baseline` | 标准场景 draw 数 ≤ 基线上限;注入 +1 draw 用例必须红(防失效) |
| `render_perf_state_change_baseline` | 02 重放去重后状态切换数 ≤ 基线上限 |
| `render_perf_upload_bytes_static_second_frame_zero` | 静态场景第 2 帧 `upload_bytes` = 0(03 增量上传验收的横切复验) |
| `render_perf_transient_peak_baseline` | 瞬态峰值 ≤ 基线上限(01 池统计) |
| `render_perf_pipelined_product_parity` | pipelined 开/关产物对拍逐像素一致;feedback 滞后一帧语义 |
| `render_perf_parallel_record_submission_order` | 并行录制提交序 = graph 拓扑序;开/关产物一致 |
| `render_perf_parallel_prepare_deterministic_sort` | rayon prepare 前后 `MeshDrawCommandList` 逐元素一致 |
| `render_perf_degrade_ladder_fixed_order` | 超预算按 scale → mip bias → feature 固定序触发;迟滞帧数内不升档 |
| `render_perf_store_lint_detects_dead_store` | 故意"终读后 Store"用例 lint 计数 = 1 |
| `render_perf_async_pipeline_placeholder_no_error_material` | 占位期产物 = 无该 draw(SkipDraw)或 depth-only;补回帧恢复 |
| `render_perf_prewarm_zero_first_frame_miss` | 预热清单消费后首帧 `variant_miss_count` = 0 |
| `render_perf_cold_start_graph_compile_once` | 冷启动 graph 编译 1 次,第 2 帧 `compiled_graph_cache_hit` = true |

产物对拍:`render_product_*` 系列在三开关(pipelined/parallel_record/async_compile)8 组合矩阵下全绿。CI 接入（policy §4 波次收口）：`render_perf_*` 确定性计数测试通过 focused 批次验证；全量 workspace 回归留给波次收口；计时型观测建议另开 `workflow_dispatch` 手动 job 导出 profile 文本工件，不设阈值门禁。

## 状态与产出记录

- open/待修复：[gpu-readback-queue-owner-missing](16/failure-2026-08-01-gpu-readback-queue-owner-missing.md)

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`17/2026-07-09-performance-and-profiling-output-records.md`](17/2026-07-09-performance-and-profiling-output-records.md)

## 性能审阅交接

- 2026-08-30 texture mip preservation typed boundary（P0-5源码闭合、动态采集待办）：resident-mip replacement的旧mip copy必须先于新mip queue write，现由`RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload`随success/failure frame receipt发布；copy/post tickets不误标。producer/reason配对由独立record owner校验，错误配对在transaction变更前返回typed error，receipt重建时再次fail closed。该切片没有重排submit。先用11.8的frame interval metrics采集同帧0/1/N个replacement时的physical submit、ticket、upload bytes和CPU/GPU时间，再决定是否把多个texture work收集为“all pre-copy commands -> one boundary -> merged uploads -> post commands”；没有profile前禁止实施该结构优化。状态：`render17_texture_mip_preservation_typed_boundary_source_implemented_static_checks_passed_dynamic_validation_pending`。
- 2026-08-30 frame submission interval观测（P0-5源码闭合、动态采集待办）：`RenderFrameSubmissionReceipt`现可选携带backend-neutral `RenderFrameSubmissionMetrics`，区分frame-owned logical packets、flush submitted tickets、physical backend submits及buffer/texture upload batch/write/payload bytes。compiled/legacy owner均在poll后且资源准备前取baseline，在terminal scene ticket后封口；owner改变或单调计数回退时不发布样本。该切片不新增flush/poll/wait/queue work，也未运行真实WGPU。Render17后续须对普通单viewport、resident-mip迁移、diagnostic/capture、environment capture、hit proxy和retained UI分别采集至少300个steady-state frame的计数分布，再以RenderDoc事件/提交对拍；当前没有“一帧一次submit”或性能收益结论。状态：`render17_frame_submission_interval_metrics_source_implemented_static_checks_passed_dynamic_validation_pending`。
- 2026-08-27 forward receiver binding观测（PFO-4d2e源码闭合、动态采集待办）：standard receiver与包含graph transient资源的full receiver现分别拥有逐成功帧native bind-group create counter和独立CPU scope；direct/compiled入口清零、唯一成功出口上报。该切片没有实现缓存或改变绑定行为。必须先采集environment-only/full deferred、1/4/16/64 shadow slot和transparent/OIT组合的至少300帧p50/p95/p99，再决定是否建立generation snapshot cache；源码调用次数不得作为瓶颈结论。
- 2026-08-27 RDG transient pool观测（PFO-4d3b源码闭合、动态采集待办）：completion collection和frame-end maintenance拥有独立CPU scope；texture/buffer completion status query、stale scan、budget accounting与over-budget sort candidate共8条work counter已进入既有RenderStats diagnostics。计数复用原遍历点且没有改变容器、淘汰顺序或submission语义。必须对deferred/forward/shadow-heavy/resize-device-recovery各采集至少300个steady-state frame的p50/p95/p99、allocator/RSS与submission数据，再判断ticket coalescing、age bucket、retained-byte ledger或bounded eviction queue；当前没有瓶颈或性能收益结论。

- 2026-07-18 core diagnostics交接：`render_stats_store/**` 30/30文件确认一次采集约写541条series；四类helper、5条遗漏product leaf与collect root 5条metric现均走static metadata快路。Render17仍须联动Runtime07交付RenderStats整体/domain generation、dense token、packed delta与editor同generation snapshot缓存，避免可见pane按UI刷新率重复全批；预算与证据见PERF-MVP-324及`docs/plans/performance/01/2026-07-18-runtime-core-render-stats-store-static-review.md`。
- 2026-07-18 profiler本身性能交接：capture inactive的recorder锁/静态name/动态payload和frame stream临时key已止损；active scope/frame/counter仍由单个全局Mutex串行，snapshot/hotspot/Perfetto/Markdown导出仍深clone、全排序并同步写盘。Render17需联动Runtime07把采集改为thread-local bounded chunks+static IDs，封存后后台聚合/导出，并记录observer overhead、drop和frame-thread I/O；见PERF-MVP-326及`docs/plans/performance/01/2026-07-18-runtime-core-profiling-static-review.md`。
- 2026-07-18 gizmo overlay交接：framework extract的per-endpoint matrix、circle/sphere temp points及line realloc已直接止损，但framework gizmos尚无生产caller，Editor05仍有独立interaction/gizmo extract。Render17需先会签唯一overlay owner，再以generation-compiled retained geometry、可复用line-list/strip buffers和transform instancing消除stable rebuild/upload；RenderDoc稳定帧必须记录line upload bytes、draw/pass与1/1k/100k instances，不以未接产品的microbench替代。见PERF-MVP-333及`docs/plans/performance/01/2026-07-18-runtime-core-framework-camera-gizmos-static-review.md`。
- 2026-07-18 sprite/light交接：sprite stats路径与真实record重复完整CPU vertex/batch构建，stable帧仍重算slices、上传并per-batch建buffer/pass；light diagnostics重扫列表且payload携带per-frame固定String。Render17联动Render14/05把prepared batch/light report作为唯一统计权威，并以RenderDoc验证stable generation的slice rebuild/upload/buffer create为0、stats额外vertex build为0、pass按phase增长、light reason alloc为0。见PERF-MVP-337/338。
- 2026-07-18 phase/static-batch/particle交接：phase span与summary O(1)、borrowed static key、一次性override build已局部止损；stable frame仍重建phase/static artifacts，particle counters重复树索引。Render17联动Render09/03/12发布queue build/sort、batch build、history index build与额外stats work counters，stable generation均应为0或1次共享report；见PERF-MVP-339/340/341。
- 2026-07-18 frame/report所有权交接：owned snapshot adapter深clone与fixed-stage `BTreeSet` allocation已直接止损；Render17联动Render10让stable generation extract rebuild=0，并联动Render01让sealed graph execution report成为alias/profile/stage统计唯一权威，`RenderStats`不得每帧再深clonealias/profile String+Vec。新增clone bytes、report build/sort/alloc与stats-extra-work counters，RenderDoc/capture导出才格式化alias String；见PERF-MVP-342/343及`docs/plans/performance/01/2026-07-18-runtime-core-framework-render-backend-frame-static-review.md`。
- 2026-07-18 camera/view性能交接：legacy layer fast path、lazy planar list、borrowed sequence与zero-jitter matrix已局部止损；Render17联动Render04/09/06发布mask alloc/clone、camera descriptor clone/find/sort和prepared matrix build/multiply counters。stable generation的camera sort、descriptor build与matrix build分别应为0/0/≤1 per camera-region，RenderDoc按1/8 cameras验证；见PERF-MVP-344/345/346。
- 2026-07-18 capture poll交接：generation-aware Wgpu capture已在stored generation相同时于RGBA clone前返回，editor/runtime stale poll copy=0；Render17仍须联动Editor01/EditorUI08交付短锁capture handle、GPU texture或bounded async readback ring，并统计GPU stall、RGBA clone/import bytes与capture age。`renderdoccmd.exe`本机command-not-found，真实capture仍pending；见PERF-MVP-023及capture/profile静态证据。
- 2026-07-18 sideband/particle资源交接：Virtual Geometry整份prepared readback重复clone/merge已止损，frame sideband现为唯一feedback owner；Render17联动Render03补sideband clone bytes、feedback duplicate和persistent page-request buffer create counters。particles neutral fallback仍每帧创建七类GPU buffer，联动Render12/Plugins01按capacity持久复用并记录create/destroy、dirty write bytes、temporary alloc与binding String alloc；stable/empty create=0。见PERF-MVP-347/348及sideband/relevance静态证据。
- 2026-07-18 TAA mask交接：0 reactive command仍固定执行全屏R8 clear pass，有command则clear+mesh为两个pass。Render17联动Render06记录mask pass/attachment bytes、reactive command count、resolve bind-group creates与resource-view generation；0-command mask pass/bytes=0，有command mask pass=1，只有generation稳定才允许缓存bind group。见PERF-MVP-350及anti-alias静态证据。
- 2026-07-18 environment 性能交接：Render17 联动 Runtime04、Render03/11发布 BRDF LUT integrate samples/CPU/upload、frame artifact fs/stat/bytes/decode/clone/stall、lightmap slot probes/index builds及 cubemap bake worker/tile/offset/scratch counters。验收要求同 device LUT create/upload≤1、稳定 IBL generation frame I/O=0、lightmap 查询近 O(meshes)、production serial bake entry=0，并补 F0/F2 trace 与 RenderDoc；见 PERF-MVP-351..354及 environment 静态证据。
- 2026-07-18 shader 性能交接：Render17联动Render03/08、Runtime04与Editor09发布variant resolves/key+String alloc/report clone、frame-thread shader fs/zstd/write/driver compile stall、async queue depth/latency/fallback draw、prewarm WGSL copies/hash visits/workers/RSS及include scan counters。稳定generation resolve/alloc=0、frame pipeline阻塞=0、source text唯一存储、changed source scan=1，并补F2与RenderDoc；见PERF-MVP-355..358及shader静态证据。
- 2026-07-18 material override性能交接：Render17联动Render03/08发布payload encode/clone、signature bytes hashed、override uniform buffer create/destroy/upload与static cache hit counters。稳定override generation全部归零，changed每唯一generation encode/hash/create≤1，multi-primitive/camera共享prepared handle并以RenderDoc验证无重复buffer；见PERF-MVP-359。
- 2026-07-18 material management性能交接：Render17联动Render08/Editor09发布management record builds、detail/row clone bytes、record visits、sort comparisons及summary/status/issue passes。稳定generation全量build/clone/sort=0、changed visits近changed+page、indices每generation≤1；在真实asset/material pane轮询trace而非test-only API上验收；见PERF-MVP-360。
- 2026-07-18 post effect统计交接：effect resource detection已由重复扫描止损为graph/executor各一轮；Render17联动Render07让sealed post-process report发布dense effect/resource bitset+counts，stats按generation共享，UI关闭时label String物化=0。记录visits、label alloc、report builds与clone bytes并以RenderDoc对齐真实pass；见PERF-MVP-361。
- 2026-07-18 post stack交接：Render17联动Render01/07记录descriptor builds、String/Vec alloc、dependency clones、graph validate/sort、stack/graph clone bytes与`Arc::make_mut` extract clone。稳定generation全部=0、changed每variant≤1且compile/extract/context/stats共享artifact identity；见PERF-MVP-362。
- 2026-07-18 post Volume交接：内建evaluator已改为进程复用，产品extract已在scene边界按priority排序并仅对乱序输入fallback sort。Render17联动Render07/Runtime07新增registry build、volume/shape visits、sort、component lookup、parameter alloc、resolved build与clone-byte counters；稳定generation registry/sort/override/clone=0、每camera submission resolved≤1且post/froxel/history/stats共享identity；见PERF-MVP-363/364。
- 2026-07-18 WGPU UI image/cache补充交接：单帧upload key现借用draw-list String，cache≤256时prune直接早退，常见present不再复制key或collect全cache Vec；但GPU chrome stream固定`include_image_bytes=true`，静态image/atlas仍可能随damage frame重复`queue.write_texture`。Render17联动EditorUI08/Render13以image generation只在首次/变更帧携bytes，并记录key alloc、prune visits、write calls/bytes；stable static image全部=0。见PERF-MVP-225。
- 2026-07-18 compiled graph cache交接：hit路径的重复fingerprint断言已删除；宽compile-options/post-stack key仍逐submission clone/hash，miss仍持framework state锁同步compile。Render17联动Render01新增key clone/hash bytes、fingerprint calls、cache lock wait/hold、compile queue/join/fallback counters；stable key owned bytes=0、fingerprint≤1、miss compile lock hold=0、同key compile≤1。见PERF-MVP-365。
- 2026-07-18 瞬态物化交接：lifetime validation tree已改复用compiled index。Render17联动Render01记录logical/physical resource、slot/index builds、BTree/String/Vec allocations、clone bytes、`create_view`、validation name lookup/bitset words、pool visits与candidate sorts。RG-M2后的dense plan/binding-mask/default-view/workspace切片要求warm grouping/String/workspace/default-view create及成功验证String lookup=0，候选sort仅超预算，并以F2/DX12 RenderDoc对拍physical backing少于logical resources且像素一致；见PERF-MVP-366。
- 2026-07-18 graph execution record交接：compute audit临时partition Vec已删除并锁定原有dispatch分组顺序。Render17联动Render01增加per-pass metadata clone bytes、detail-row builds、summary full scans、workspace growth、profile/alias clone counters；diagnostics-off要求compiled metadata clone/detail/full scans=0，diagnostics-on每pass detail≤1且workspace warm growth=0，stats额外deep clone=0。见PERF-MVP-343。
- 2026-07-18 HZB build交接：Render17联动Render04记录mip/reduce batch、texture-view/bind-group/upload-buffer create、buffer copy、compute pass/dispatch、CPU record与GPU timestamp。1923×1081必须从当前11个逐mippasses收敛到≤3个四mip批次；params Vec已删除，最终warm GPU object/create增长=0，并以single/MSAA HZB chain及DX12 RenderDoc对拍。见PERF-MVP-367。
- 2026-07-18 forward/OIT binding交接：恒定disabled volumetric uniform buffer已从per-pass create提升为`MeshPipelineCache`唯一owner。Render17联动Render02/14/18记录uniform-buffer/bind-group create+destroy、entry Vec alloc、packed upload calls/bytes、resource-generation bundle rebuild、OIT sprite vertex builds/texture bind-group/vertex-buffer creates。warm stable generation要求forward/OIT GPU object create=0、bundle≤1/resource tuple generation、动态参数≤1 packed upload/camera frame、OIT额外sprite build=0，并以F2像素、timestamp和DX12 RenderDoc核对；见PERF-MVP-368。
- 2026-07-18 post execute交接：probe active-prefix/zero-count upload已止损；Render17联动Render07/18记录post params buffer/bind-group create+destroy、29-binding bundle rebuild、params/probe upload calls+bytes、GI tree nodes/alloc、scene-row comparisons与camera projection。warm stable要求post GPU object create=0、bundle≤1/resource generation、params≤1 packed upload/camera frame；count0 probe writes=0，prepared join≤1/generation。见PERF-MVP-369。
- 2026-07-18 post effect executor交接：cluster CPU clear/full-light upload已止损；Render17联动Render01/05/07记录各effect buffer/texture/view/bind-group create+destroy、disabled clear passes/attachment bytes、cluster clear/upload bytes和color-LUT bake generations。warm stable effect GPU object create=0、SMAA backing由pool复用、feature-off clear passes=0、LUT bake≤1/relevant generation且auto exposure不强迫重烘；见PERF-MVP-370。
- 2026-07-18 post pipeline启动交接：同源split post WGSL transforms/shader modules/layouts已由9/9/9降至1/1/1。Render17联动Render07/08记录constructor pipeline creates、driver compile wall、frame-thread stall、pipeline queue depth/latency、first-use bypass与RSS；F0不得创建未请求optional pipeline，duplicate descriptor compile≤1，F2必需首帧前ready。见PERF-MVP-371。
- 2026-07-18 post pass-graph记录交接：normal executor-ID clone/String tree已用固定effect mask止损。Render17联动Render07记录graph clone bytes、executor/node label clone、fallback tree allocation、mask builds与diagnostics materialization；stable compiled generation要求graph/fallback set/node-name String build=0，compact executed bitset由stats共享，只有UI/capture/log导出才格式化label。见PERF-MVP-372。
- 2026-07-18 compiled-scene提交交接：HZB telemetry当前提交后同步wait GPU；frame/history/stage、fallback resources、indirect workspace和irradiance selection的四类重复工作已分别编号PERF-MVP-373..377。Render17必须记录blocking poll/wait、readback age/drop、frame clone、GPU object create/destroy、copy/upload、position/index visits；产品`wait_indefinitely`=0、diagnostics off readback=0、stable generation各类rebuild/create=0。详见compiled-scene render静态证据。
- 2026-07-18 stage/pass lookup交接：compiled-scene当前stage×pass全扫并按name再次查pass。Render17联动Render01新增stage-entry visits、pass-name comparisons与range/index build counters；PassId/dense range hard cut后frame visits近executed passes、name comparisons=0、range build≤1/compile generation。见PERF-MVP-378。
- 2026-07-18 scene renderer core补充交接：Render17联动Plugins01、Render03/12/18记录runtime collector callback wall、payload owned/copy bytes、binding Vec growth、queue age/drop，要求stable heavy prepare/copy/rebuild=0且submission callback有界；联动Runtime04、Render11/13记录cubemap frame-thread f16 conversions、temp bytes、write/copy calls和staging growth，要求stable全0、changed artifact build≤1/generation且upload batch≤1。见PERF-MVP-379/380及scene-renderer-core静态证据。
- 2026-07-18 prepared mesh stats交接：Render17联动Render02/03记录pending stats draw/key/GPU-entry visits、hash/entry probes、key clone bytes及VG execution DTO/HashSet alloc。单表止损后eligible draw entry≤1/key clone=0；最终stable extra scan/key/DTO/set=0、diagnostics off unique work=0、changed build≤1/generation。参考UE以`r.MeshDrawCommands.Stats`门控pass draw-data；见PERF-MVP-381。
- 2026-07-18 mesh batch projection交接：Render17联动Render02/03增加batch projection、Arc/wgpu/PipelineKey clone、per-draw command Vec alloc/grow及moved-command counters。stable 100% static cache hit必须资源/key clone=0、temp Vec=0；dynamic只允许frame arena warm reuse。见PERF-MVP-382。
- 2026-07-18 mesh command/indirect artifact交接：全command预排序和dynamic per-draw Vec已止损。Render17联动Render02/03记录phase Vec alloc/grow、sort calls/comparisons、camera-stack moved commands、cache retain visits、indirect batcher builds、key clone/args/metadata bytes；stable sort/partition/batcher/cache full-scan=0，stats extra build=0。见PERF-MVP-383。
- 2026-07-18 pending cache hit分配交接：Render17补phase/command/residual Vec alloc+capacity、cached command/resource clone bytes与moved-command counters。100% stable hit及visibility-pruned必须per-draw heap alloc=0、command/resource clone=0；只允许真实miss按changed generation重建。见PERF-MVP-382及pending-cache静态证据。
- 2026-07-18 mesh material/palette binding交接：Render17联动Render02/03/08记录material custom/standard与palette bind-group creates、13-entry builds、binding cache hit/miss、handle clone和RSS。stable create=0；changed material≤2/unique generation、palette≤1/unique buffer-pair generation，并以DX12 RenderDoc核对bindings。见PERF-MVP-384。
- 2026-07-18 build-mesh-draws完整模块交接：Render17新增phase-input visits/sort、morph delta/weight bytes、skeleton map/inverse/CPU vertex visits、VG draw/segment clone与5类buffer create、resident page upload以及dynamic `GpuMeshResource::from_asset`分类计数。PERF-MVP-385..389验收要求stable generation相应build/create/upload为0，changed近dirty delta，并分别以morph/skin/motion-vector/VG/LOD/HZB像素和DX12 RenderDoc对拍。
- 2026-07-18 mesh pipeline cache交接：Render17记录7类ensure的cache hit、variant/key clone、source assembly/hash bytes与module-key build，确认本轮stable hit均为0；另记录`MeshPipelineCache::new`的texture/buffer/sampler/layout creates、queue writes、RSS和constructor wall，minimal/all features及1/8 owners区分。见PERF-MVP-355/356/390。
- 2026-07-18 mesh descriptor driver-cache交接：Render17区分WGSL disk hit与driver pipeline cache hit，记录`RenderPipelineDescriptor.cache`使用、module/pipeline create、driver compile wall、artifact bytes/compatibility miss及frame stall；cold/warm process与driver变化矩阵验收PERF-MVP-356。当前7类descriptor均`cache: None`，不得把WGSL hit误报为pipeline warm。
- 2026-07-18 skin palette bytes补充交接：Render17记录active joint count、storage capacity、initialized/copied/uploaded bytes与current/previous slot dirty count；当前固定约16 KiB/palette，1k实例双面约32.8 MiB。PERF-MVP-386最终要求stable upload=0、changed bytes近active bones，禁止只报buffer count掩盖固定全块传输。
- 2026-07-18 shadow完整模块交接：Render17记录allocator dedup/hash/sort、preemption pair/free-rect visits、plan/matrix builds、slot/global upload ranges/bytes，以及per-slot BTreeSet/set probes、command visits、pass/uniform-buffer/bind-group/String creates；另计shadow renderer neutral环境对象。PERF-MVP-390..392要求stable plan/upload=0、atlas pass≤1、visits近visible commands、重复neutral=0。
- 2026-07-18 lighting/grid交接：Render17记录packed-light与cookie-plan builds、volumetric membership probes、grid alloc/zero bytes、tile/bin visits、stats cluster visits、upload calls/ranges/bytes及CPU/GPU p95；区分1/8 cameras、stable/1% changed和diagnostics off/on。PERF-MVP-393要求每lighting generation pack≤1、stable grid build/upload=0、off时Cartesian stats=0。
- 2026-07-18 HZB occlusion整目录补充交接：Render17联动Render04记录per-phase params upload buffer、8-entry bind-group、compute-pass、compaction clear、copy/map、blocking poll wall、readback age/drop，并区分history/execution generation与diagnostics gate。PERF-MVP-373/376要求产品wait=0、off时readback/copy=0、stable params upload/binding/workspace create=0；见HZB occlusion静态证据。
- 2026-07-18 Deferred整目录交接：Render17记录source assembly/module/layout/pipeline creates与driver wall，以及params buffer、大bind-group/entry alloc、GBuffer receiver group、lightmap handle clone、neutral资源和attachment alloc。当前source/module/layout 2→1且attachment Vec=0；PERF-MVP-356/368/390最终要求stable binding/clone=0、无SSS pipeline=0、shared neutral≤1/device/kind。
- 2026-07-18 transparent mixed交接：Render17联动Render09/02/14记录sprite presence scans、mesh/sprite item visits、mixed Vec alloc/grow/capacity、sort calls/comparisons/temp bytes与generation hits。当前growth/stable-sort temp=0；PERF-MVP-339最终要求stable mixed build/sort=0、changed linear merge近affected ranges。
- 2026-07-18 scene clear交接：Render17联动Render01/09记录region coverage、clear pass/draw、color uniform writes、first-write load ops、pipeline creates、attachment bytes与GPU timestamp。PERF-MVP-394要求full-target pass/draw=0、partial color+depth≤1、无clear=0且未请求partial clear pipeline=0。
- 2026-07-18 history资源交接：Render17记录per-slot texture/view/buffer creates/destroys、VRAM、CPU init alloc/upload、GPU clear passes、bind handle clones、resize reason和copy bytes。当前4K约365MB CPU init已降0且full pack clear≤2；PERF-MVP-395最终要求feature-off slot=0、stable rebuild/clone=0、changed只affected slot。
- 2026-07-18 temporal执行交接：Render17记录TAA mask/resolve、camera/object velocity pass/draw/clear bytes、matrix builds/inverses、params writes、bind-group creates及history view clones。当前empty LoadStore object pass=0；PERF-MVP-346/350/368/395要求0-reactive pass=0、stable matrix≤1/camera generation且binding/history clone=0。
- 2026-07-18 primitives/product overlay交接：Render17记录selection mesh/model probes、wire/gizmo/handle matrix+trig、Vec/HashSet alloc、vertex build bytes、GPU buffer creates/uploads/draws与generation hits。当前Shaded wire visits/buffer=0、WireOverlay selection hash=0；PERF-MVP-333最终要求selection近O(S)、stable rebuild/upload/create=0。
- 2026-07-18 sprite整目录补充交接：Render17联动Render14记录phase/fallback visits、slice/vertex/batch builds、stats额外work、artifact identity、upload bytes、buffer/bind-group/pass/draw与generation hit；分别标记2D、mixed transparent、overlay和OIT消费者。PERF-MVP-337要求stable artifact rebuild/upload/GPU create=0、stats额外vertex build=0、OIT无第二prepare owner且pass按phase增长。
- 2026-07-18 overlay整目录补充交接：Render17记录display/sky feature、selection probes、transparent sprite find probes、geometry/command artifact builds、GPU buffer/bind-group/pipeline create、upload、内部pass/draw与generation hit。当前Disabled sky params/bind=0、WireOnly LoadStore base/mixed work=0；PERF-MVP-333/337/368/383/390最终要求overlay pass≤1、stable rebuild/create=0、sprite lookup近O(S)、fallback extra command build=0、minimal未请求pipeline=0。
- 2026-07-18 legacy particle整目录交接：Render17联动Render12/03/06/08记录current/previous visits、anonymous/history tree nodes、sin/cos、CPU world-quad bytes、instance/dirty upload、buffer/pipeline/pass/draw、indirect args与artifact generation。PERF-MVP-396要求CPU quad=0、history index≤1/generation、stable create/upload=0、颜色pass≤1且particle-off pipeline=0。
- 2026-07-18 UI font/SDF upload交接：Render17联动Text01/05/09与Render13记录font manifest stat/read/parse/negative retry、page-key/set builds、page probes/spec/report clone、upload commands/calls/bytes与atlas generation。当前stable command=0、report clone=0、page table≤1/build；PERF-MVP-249/250最终要求stable atlas upload=0、page metadata≤1/generation、missing同generation I/O≤1且reload恢复。
- 2026-07-18 SDF advance与UI pipeline补充交接：Render17记录grapheme/char passes、advance temp Vec/sanitize visits及base UI pipeline driver wall/cache hit。当前fallback mapping单stream、grapheme Vec=0、sanitize visits=1；PERF-MVP-249最终同generation mapping=0，PERF-MVP-356要求warm base UI frame compile=0。
- 2026-07-18 scene UI image交接：Render17联动EditorLayout21、Render13/14记录image generation、texture Arc clone、bind-group/buffer create、CPU vertex/instance bytes、upload、scissor/bind switches、draw/batch。PERF-MVP-397要求stable prepare/create/upload=0、bind≤1/texture generation、CPU 6-vertex=0且draw≤ordered compatible runs。
- 2026-07-18 scene UI root交接：Render17联动EditorLayout21、Render14与Text09记录paint projection、serde/hash/debug-label bytes、text line/advance/style与prepare-report clone bytes、七组plan Vec alloc/grow、solid buffer create/upload及generation hit。当前decoration重复投影2→1、空LoadStore pass=0；PERF-MVP-398最终要求stable上述CPU/GPU工作全部为0，changed近dirty ranges，sealed report deep clone=0。
- 2026-07-18 scene UI render子目录补充交接：Render17另记录rich parse/run/prefix probes、vertical grapheme-range build/binary probes/overlap visits与background candidate/blocker visits。当前paint≤1/command、rich parse≤1/rich command、vertical无glyph×grapheme全扫、background query O(C+B)；PERF-MVP-398最终stable这些projection/index rebuild仍须为0。
- 2026-07-18 scene UI SDF/text核心交接：Render17联动Text05/09、Render13/14记录glyph key/string clone、set/map/shelf/slot/run builds、CPU run prepare次数、atlas bake/page bytes、material scratch/upload skip、decoration/glyph vertex bytes、buffer create及native buffer/text-area/submission Vec。当前identical material uniform upload=0、scalar count Vec=0；PERF-MVP-249要求stable其余build/create/upload全部为0。
- 2026-07-18 native atlas renderer/upload交接：Render17联动Text04、Render13/14记录per-storage prepared upload/frame-plan/pass-report builds、vertex-buffer create/bytes、draw-command clone、pipeline linear probes、texture binding Vec与writes。page generation/face validity失败单独计数；PERF-MVP-231要求stable上述prepare/create/clone/write=0且changed近dirty slots/ranges。
- 2026-07-18 graph executor dispatch交接：Render17联动Render01/Plugins01记录registry table builds、BTree/String probes、executor ID clone/hash bytes、dense slot resolves、post node/resource-name clones/lookups、SSR full/per-mip view creates及plugin generation rebind。PERF-MVP-399要求同generation built-in build≤1、stable String/BTree dispatch=0、slot O(1)、required/produced clone=0；view bundle同时服从PERF-MVP-366，并以F2/plugin reload、timestamp和DX12 RenderDoc对拍。
- 2026-07-18 reflection/planar probe交接：Render17联动Render11/18记录candidate visits/sorts、registry lock、slot hit/eviction、GPU row/matrix builds、buffer writes/bytes、texture/buffer/view creates、VRAM与generation hit。PERF-MVP-400当前locks≤1/frame；最终stable sort/build/write=0、changed近delta、feature-off真实probe/planar allocation=0、capacity按需且neutral≤1/device，并以F2/timestamp/DX12 RenderDoc核对。
- 2026-07-18 realtime IBL交接：Render17记录bake-key calls、variant graph build/compile、resource-name/hash/binding/validation、per-dispatch uniform/bind-group/sampler、duplicate cloud dispatch、timestamp query/readback buffer、pending/completed queue age/drop及CPU/GPU p95。PERF-MVP-401要求warm对象/String增长=0、first update受预算、off时query/readback=0、固定in-flight ring且产品`wait_indefinitely`=0，以F2和DX12 RenderDoc对拍。
- 2026-07-18 IBL bake/writeback交接：Render17记录per-request/per-pass command/shader plan builds、String/map/params/readback-copy Vec、name parse/find、params/bind/sampler/mip-view creates、readback buffer/map/wait、payload copy、cache file bytes及job age/drop。PERF-MVP-402当前sampler≤1/cache；最终10-pass build=1、warm对象=0、cache-hit GPU/readback/I/O=0、render线程wait/file I/O=0且队列有界，以artifact bytes/F2/timestamp/DX12 RenderDoc对拍。
- 2026-07-18 advanced-lighting全目录交接：Render17为froxel/cookie/irradiance/OIT/planar/SSS分别记录settings/table/plan/position visits、CPU alloc、buffer/view/bind creates、clear/upload/blit bytes、pass/draw/dispatch及artifact hit。PERF-MVP-403当前空froxel per-frame fallback create/upload=0、OIT layer clear=0、cookie slot唯一；最终stable heavy build/write/create=0、cookie clear/blit=0、irradiance prepare≤1/camera generation、SSS table≤1、planar views≤1/texture generation，并以F2、timestamp和DX12 RenderDoc核对。
- 2026-07-18 scene resource residency交接：Render17记录每frame/unique resource的registry locks/probes、asset load/decode/clone/hash/validation、job queue age/drop、CPU alloc、GPU create/upload bytes、fallback/last-good及output encoder/submit。PERF-MVP-404要求stable重活/create/upload=0、bulk snapshot≤1/generation、render线程I/O/decode=0、changed近dirty、upload队列有界、writeback额外submit=0，并以F2、规模counter、timestamp和DX12 RenderDoc核对。
- 2026-07-18 GPUScene全目录交接：Render17记录full/delta draw visits、live/history map probes与clone bytes、free-span probes/sort/fragmentation、dirty merge、parallel pack jobs、morph/VG/palette capacity、buffer/bind create及direct/scatter write/copy/dispatch bytes。PERF-MVP-405当前stable核心/morph/VG upload=0；最终stable全场/history visits=0、allocator无frame全排序、缩短不重建、palette近active joints、large update有界scatter，以F2规模、timestamp和DX12 RenderDoc核对。
- 2026-07-18 RenderProduct 测试吞吐交接：Render17 记录 backend/adapter/device 初始化次数、context key/hit、纯测试/GPU 测试数、setup/test wall time 与 error-scope 污染；`PERF-MVP-406` 当前为 35 tests / 35 backend init，目标为兼容 key 内不超过 1 次、纯测试为 0 次且失败不污染后续测试，以真实 Cargo filter/full 批次验收。
- 2026-07-18 graphics backend交接：Render17记录readback region/buffer/encoder/submit/map/wait与staging/output bytes，surface present的bind/pipeline/encoder/submit/pass/draw，以及offscreen各slot create/destroy/VRAM/extent generation。当前cube 128/8同步等待48→1但完整artifact仍约3次；PERF-MVP-023/402最终产品wait=0，PERF-MVP-407额外present submit=0，PERF-MVP-408 feature-off真实slot=0，并以F2、timestamp和DX12 RenderDoc核对。
- 2026-07-18 graphics module cold-start交接：Render17按extension类别记录catalog freeze/build/clone bytes与Vec alloc，按instance/adapter/device/renderer/pipeline/resource stage记录wall、caller blocked time、init lane、ticket age/retry和single-flight hit。PERF-MVP-409要求catalog每generation物化≤1、factory deep clone=0、主/UI线程device-init blocked=0，以F0/F2 cold/warm启动trace和RenderDoc对象核对。
- 2026-07-18 viewport camera state交接：Render17记录camera key builds/hash/layer alloc与clone bytes、7表 probes/capacity、active/retired slot数、history/provider/debug/particle bytes、prune/forget latency。PERF-MVP-410当前key clone layer deep copy=0；最终entries≤active+TTL budget、removed GPU history按期释放，并以F2多视口10k-frame内存曲线和RenderDoc资源核对。
- 2026-07-18 render framework锁域交接：Render17记录operation/state lock wait+hold、driver/GPU/large-clone-in-lock bytes、lifecycle ticket age/stale publish与per-viewport queue p95。PERF-MVP-411要求锁不跨surface/history/capture/stats/VG重活，独立viewport慢操作不阻塞query/submit，以并发slow-driver fixture、F2多视口和RenderDoc核对。
- 2026-07-18 pipeline control-plane交接：Render17记录pipeline asset clone bytes、validation compile/executor/capability calls、profile option String/set alloc、operation/state wait+hold与single-flight ticket hit/age。PERF-MVP-412当前graph compile state-lock hold=0、profile deep clone=0；最终register/revision compile≤1、set compile=0且锁不跨compile。
- 2026-07-18 submission prepare/record交接：Render17记录provider metadata clone、compiled pipeline/graph clone bytes、graph dump serialization、history snapshot clone bytes、VG set/segment/traversal visits、state lock hold与capture RGBA clone。PERF-MVP-413当前provider String/pipeline deep clone=0、stable dump serialization=0、record侧VG scan=0、hierarchy traversal=1；最终stable history clone=0、diagnostics-off format=0、sealed stats build≤1/generation且锁持有不随capture/snapshot大小增长，并以F2多camera、timestamp与DX12 RenderDoc核对。
- 2026-07-18 frame context交接：Render17记录per-camera compile lookup/build、wide key clone/hash、state wait/hold、history/pipeline snapshot clone bytes、material lineage/load/mesh scans、IBL/texture/model fs/stat/read/build及VG output clone。PERF-MVP-414当前descriptor与owned viewport payload二次clone=0、provider String/VG output clone=0、material lineage≤unique roots/camera；最终compile lookup≤1/camera、stable scene/material/environment/model build/I/O=0且multi-camera共享artifact，以F2规模、timestamp和DX12 RenderDoc核对。
- 2026-07-18 feedback/history交接：Render17记录readback copy/map/wait、renderer/sideband merge visits与Vec growth、feedback clone bytes、ticket age/drop、per-camera take/discard、particle ambiguity/history scans、result Vec realloc及history compare/rotation。PERF-MVP-415当前history compare=1/context、stable particle result Vec realloc=0；最终merge≤1/generation、payload clone=0、nonowner particle work=0、frame-thread wait=0且队列/RSS有界，以F2 camera-stack、timestamp和DX12 RenderDoc核对。
- 2026-07-18 VG diagnostics observer交接：Render17记录snapshot build/clone bytes、page/cluster/instance/node visits、selected sort comparisons、CPU traversal rows、detail materialization、poll rate/age、RSS及normal-frame overhead。PERF-MVP-416当前page nested lookup=0；最终diagnostics-off上述全部=0，on时report≤1/generation、多camera共享且detail近visible rows，并以F2 overlay/query、timestamp和DX12 RenderDoc核对真实execution。
- 2026-07-18 VG runtime overlay补充：visbuffer marks已改borrow snapshot slice，不再为overlay复制整Vec。Render17在PERF-MVP-416同一计数中另记BVH/visbuffer node/mark visits、retained line rebuild/bytes、cluster/node index builds与overlay draw/pass；stable debug generation rebuild=0，debug-off全部=0。
- 2026-07-18 camera-loop plan交接：Render17记录sequence/plan builds、descriptor/post/visibility clone bytes、target probes、planar lock wait/hold、per-camera context build及failure retry。PERF-MVP-417当前terminal submission DTO=0、planar camera comparisons=0；最终stable plan build/clone=0、每loop resolve≤1、large source clone=0、planar work近changed且已成功probe不重做，以F2多camera/probe和DX12 RenderDoc核对。
- 2026-07-18 submit execution补充交接：Render17分别记录Phase A snapshot、Phase B prepare/render/present/feedback与Phase C publish的operation/state lock wait/hold、render-owner queue age、queue submit、surface lease、VG Arc clone/deep-clone bytes及per-viewport p95。PERF-MVP-411要求锁不跨Phase B且独立viewport/query不被慢driver阻塞，当前VG owned query clone state-lock hold=0；PERF-MVP-416当前内部global/per-camera snapshot deep clone=0，debug-off构造仍须为0。以1/2/8/64 viewports、0/10/100 ms slow driver和DX12 RenderDoc核对。
- 2026-07-18 stats observer补充交接：Render17记录sealed report builds、pass/executor/visibility/light/VG visits、String/Vec clone+alloc、subscription hit、state lock hold、history bytes与observer CPU p95，分别对diagnostics off/on/capture。PERF-MVP-418当前coverage clone/sort=0、executor scans 8→1、visibility 5→1、UI 3→1、stable String alloc=0、VG node lookup均摊O(1)；最终off detail=0、on build≤1/generation、多consumer deep clone=0，以F2和DX12 RenderDoc/timestamp核对。
- 2026-07-18 submission root contract补充：Render17记录Phase A/Phase C viewport map probes、slot hits、generation mismatch/stale publish、context payload clone bytes与capture failure分支。PERF-MVP-411当前mutable helper内部lookup 2→1、owner总路径3→2；最终stable slot owner path map probe≤1/transaction且Phase C近O(1)，以destroy/recreate/resize race和F2多camera验证。
- 2026-07-18 graphics debugger owner补充：Render17记录capture request/overwrite、active/queued age、backend start/stop/poll wall、operation/state wait+hold、error merge与独立viewport submit p95。PERF-MVP-411要求stop/poll不持全局operation/state锁，PERF-MVP-023要求capture/readback固定in-flight且有age/drop；以request burst、destroy/error race和真实DX12 RenderDoc核对。
- 2026-07-18 temporal history ownership补充：Render17记录validation key build/compare/clone bytes、mesh/pose visits、bindings/visibility/static clone bytes、Arc hit与state lock hold。PERF-MVP-413当前per-record validation-key deep clone=0；最终stable generation key build/compare和全部history payload clone=0、changed近affected revisions，以1/8/64 cameras、100k meshes/poses和F2 history parity核对。
- 2026-07-18 offline reflection bake补充：Render17按meshes/lights/probe budget记录visits、eligible count、Vec alloc/growth、CPU wall、job queue/cancel与未来GPU capture work。当前zero-budget/empty-mesh light scan=0、有效Vec growth=0；预算扩大时要求UI/render-lock hold=0并按dirty generation/timeslice执行。
- 2026-07-18 runtime provider合同补充：Render17分别记录HGI/VG/Solari provider callback wall、framework state-lock overlap、per-camera mesh/model asset visits、filtered projection capacity/growth、readback copy/merge与reload generation。当前三类filtered projection known-input growth=0；PERF-MVP-379/414最终provider callback不持state锁且stable multi-camera scan/load/build=0，PERF-MVP-415最终payload merge≤1/generation并以F2、timestamp和DX12 RenderDoc核对。
- 2026-07-18 visibility全目录补充：Render17记录per-view matrix/tan、layer/relevance/frustum visits、candidate/result bytes、TaskPool/GPU queue；Context map/set/key/index/history clone bytes、static cells/candidates/RSS；VG ordinal scans/sorts、frontier moves、lineage visits/set alloc与request queue。PERF-MVP-419当前mesh matrix/tan=N→1/view、extra candidate builds=views→1；420当前bounds temp containers 2→0、renderable/history clone=0、known draw growth=0；421最终dense index与cull近O(candidates+edges)，以F2、timestamp和DX12 RenderDoc核对。
- 2026-07-18 pipeline compiler补充：Render17记录descriptor builds/clone bytes、resource analyses、post stack/effect scans、writer/reader visits、edge/BTree alloc、stage clone/move、compile queue age和framework lock hold。PERF-MVP-422当前active resource builds=filtered passes→1/descriptor、resource plan clone=0、default-handle asset build/frame=0；423最终compile近O(P×R+E+M)、full pass clone=0，以F2 reload、timestamp和DX12 RenderDoc核对。
- 2026-07-18 graphics shader全目录补充：Render17记录module registry rebuild、include extract/strip/hash bytes、assembly/token/hash clone、IDE stub candidate/edge visits、validation-source/Naga parse、preview S×V、prewarm workers/in-flight/RSS以及frame/UI disk/zstd/driver stall。当前preview index=S×V→1/batch、manifest clone=0；PERF-MVP-356/357/358最终stable generation全部为0、changed近dirty closure且队列有界。
- 2026-07-18 graphics material registry补充：Render17记录shading lookup token/String alloc、descriptor×pass×ready-record visits、path normalize/suffix bytes、shader loads/source clone与reload generation。当前builtin lookup alloc=0、duplicate witness Vec=0；PERF-MVP-358/404最终stable scan/normalize/load/clone=0，changed近affected plugin models。
- 2026-07-18 graphics types补充：Render17记录output-target plan builds/String alloc、scene/extract/post/visibility/RGBA clone bytes、camera/target数量、encoder/submit与CPU p95。当前生产plan诊断alloc=0；PERF-MVP-413/414/417最终stable large clone=0，PERF-MVP-404最终writeback额外submit=0，并以F2 camera target/history、timestamp及DX12 RenderDoc核对。
- 2026-07-22 runtime asset pipeline观测补充：Render17联动Runtime04/11与Editor09记录worker request/completion entries+bytes+age、payload owner/deep-clone bytes、generation/project锁wait+hold、artifact reads/resident bytes，以及management registry scans/sorts/String/asset/scene clone bytes。PERF-MVP-498要求同结果payload owner=1且慢consumer RSS有界；499要求发布锁持有近常数且startup只驻留MVP working set；500要求stable 60Hz management build/sort/deep clone=0。F1/F4 CPU/allocator/RSS trace为主，只有资源驻留影响真实GPU对象时再与DX12 RenderDoc资源列表对拍。
- 2026-07-22 asset watch观测补充：Render17联动Runtime04/11记录notify ingress/pending entries+bytes+age、fold/coalesce/overflow/reconcile、batch flush latency、watcher-thread callback wall、targeted scan/import与RSS。PERF-MVP-501要求continuous 60s仍有界并在max latency内发布、同URI burst≤1 effective generation、overflow最终可观测收敛；该路径用CPU/allocator/I/O trace，不把无GPU提交的watch风暴误列为RenderDoc证据。
- 2026-07-22 importer/cook观测补充：Render17联动Runtime04/11、Plugins12与Render11/13记录matcher/index visits+alloc、source opens/read/hash/parse/decode/cook、dependency/hierarchy visits、payload owner/deep-clone bytes、worker queue/RSS及artifact/upload generation。PERF-MVP-503要求stable select近O(candidates)且alloc=0；504要求同content source open/parse/decode/cook≤1、大payload owner=1、warm heavy work=0。CPU/I/O/allocator为主，只有生成GPU resident资源后才用DX12 RenderDoc核对对象/上传去重。
- 2026-07-22 artifact store观测补充：Render17联动Runtime04/11、Render13与Editor15记录asset/cache DTO/bincode/zstd/chunk owner数、clone/copy/encoded/decoded/read/write bytes、atomic publish、queue/caller blocked与RSS；IBL另记blob candidate/texel clone和requested mip/face chunks。PERF-MVP-505当前压缩临时整块owner=0；506最终要求大payload owner=1、峰值额外内存按bounded chunk、warm payload I/O/decode/upload=0。GPU资源仅对ready generation用DX12 RenderDoc核对，磁盘缓存本身以CPU/allocator/I/O trace验收。
- 2026-07-22 VG cook观测补充：Render17联动Runtime04/11与Plugins12记录feature request、cook calls、triangle/source/header visits、sort/clone bytes、job queue/RSS、artifact generation与caller blocked。PERF-MVP-508当前page offset visits O(P)、binary/BVH主排序各1；509最终要求feature-off cook=0、on≤1/content+config generation、warm=0且work近O(T+C+P)。该阶段以CPU/allocator/job trace为主，ready artifact GPU消费再用DX12 RenderDoc核对resident page/upload去重。
- 2026-07-22 model/mesh导入观测补充：Render17联动Runtime04记录model→mesh primitive/vertex/index clone bytes、normal/tangent triangle visits、derived scratch与峰值RSS。PERF-MVP-514当前整份primitive clone=0、derived index temp=0；PERF-MVP-385/386/389最终要求stable morph/skin/resident artifact展开与GPU create/upload=0，509要求VG warm cook=0。CPU阶段用allocator/RSS/counter验收，ready mesh仅在真实backend用DX12 RenderDoc核对resource/upload/draw parity。
- 2026-07-22 material asset观测补充：Render17联动Runtime04/Render08/Editor09记录material parent/property/layout/slot visits、dependency probes、descriptor/readiness/summary builds、clone bytes、prepared binding/key与RSS。PERF-MVP-515当前summary visits 9N→N；516最终要求stable effective generation全部build/scan/clone=0、changed prepare≤1/generation且工作近线性。CPU/allocator/Editor trace验证资产阶段，真实backend用DX12 RenderDoc核对material binding/pipeline/draw parity。
- 2026-07-22 shader asset观测补充：Render17联动Runtime04/Render08/Editor09记录property slot probes、schema/name/hash/WGSL bytes、variant define clones、readiness/detail builds与RSS。PERF-MVP-517当前summary visits 14N→N且stage parse alloc=0；518最终要求packing近O(P)、stable compiled generation work=0、define payload owner=1、compact polling不建wide report。真实backend再用DX12 RenderDoc核对pipeline/binding/draw parity。
- 2026-07-22 scene asset观测补充：Render17联动Runtime04/Editor09/Render03记录scene/entity/component/reference visits、overview/row/detail clone bytes、generation build/sort与RSS。PERF-MVP-519当前三类aggregate visits为N且entity list宽row clone=0；520最终要求stable scene projection build/scan/clone=0、changed近delta+page、reference count不物化Vec。CPU/Editor trace为主，真实backend再用DX12 RenderDoc核对scene resource/upload/draw parity。
- 2026-07-22 texture asset观测补充：Render17联动Runtime04/11、Render13与Editor09记录descriptor normalize/clone、container header parse、format String alloc、source/output/scratch owners、chunk queue age/RSS、GPU object creates及mip/face/layer upload bytes。PERF-MVP-521当前metadata第二次clone/normalize为0且Cube LUT defaults构造1次；522/523最终stable上述全部为0、changed近dirty chunks并受bytes/RSS预算。CPU/allocator/I/O trace验证准备阶段，真实backend用DX12 RenderDoc核对resource/upload/像素parity。
- 2026-07-22 root/project/UI/sound asset观测补充：Render17联动Runtime04/11、Editor09/10、EditorUI09与Plugins02记录TOML/Value/DTO/string owners、Data text/JSON owners、UI DOM/locator/sprite-index visits、audio source/PCM/decode-ring bytes及caller/audio-thread stall。PERF-MVP-524..526当前局部临时分配已止损；527..529最终stable build/parse/decode=0且峰值受预算。CPU/allocator/I/O/audio trace为主，只有sprite/preview真实resident与draw再用DX12 RenderDoc对拍。
- 2026-07-22 plugin control-plane观测补充：Render17联动Plugins01/11、Runtime06/11与Editor12记录bridge status/snapshot/String/key-resolve、extension family/owner scans、freeze/thaw/hash/key clone、world plan/system factory build、callback mutex wait/hold、availability reason/id clone与generation hit。PERF-MVP-530/531当前局部冗余为0；532..534最终stable control-plane build=0、per-run shared callback lock=0、changed近owner slots。该路径以CPU/lock/allocator/WPR为主，不误列无GPU工作的RenderDoc证据。
- 2026-07-31 PF-M2 prepare/queue 前置交接：当前 Render02 command builder 将 variant id 分配、cache mutation 与 command 生成耦合在可变循环中；Render17 已完成 bounded pipelined submission，但不会以 mutex 假装 rayon 并行。见 [`02/failure-2026-07-31-parallel-mesh-command-preparation-contract.md`](02/failure-2026-07-31-parallel-mesh-command-preparation-contract.md)（open/待修复）。

## 2026-08-27 FrameProfiler GPU Resolution Owner Split

状态：`runtime_17_15_frame_profiler_gpu_resolution_owner_split_static_passed_cargo_profile_deferred`。

PF-M1 的 `FrameProfiler` 当前结构切片把延迟 GPU timer/pipeline-statistics 结果归并、重复 pass
occurrence 匹配、subsystem GPU budget 投影和 warning 计数迁入 153 行的
`frame_profiler/gpu_resolution.rs`。796 行父 owner 继续唯一拥有 current-frame 组装、4-frame
pending ring 与 profile publication；`FrameProfileWrite` 只在既有 crate-private 范围由父 owner
精选投影，未形成第二条诊断或调度路径。7 个迁移项与 `HEAD` 规范化等价，结构/status guard
2/2、定向格式与 diff check 通过。

这只是使 profiling 基础设施可审查的 owner 收敛，不是 profiler 算法优化。pending ring、
late-result matching、copy-on-write 和 budget 计算均未修改，当前没有产品 CPU/GPU timestamp、
allocator/RSS、功耗、WGPU/RenderDoc 或像素数据。必须先完成本计划既定 product-observability
gate，才能据数据提出算法变更；当前不声明 PF-M1、Render17 或 Runtime15 acceptance。
