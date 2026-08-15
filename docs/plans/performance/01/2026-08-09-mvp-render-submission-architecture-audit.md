---
related_code:
  - zircon_app/src/entry
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/core/runtime/diagnostics/profiling
  - zircon_runtime_interface/src
primary_reference:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderingThread.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
secondary_references:
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/Debugging/ProfilingSamplerWithCommandBufferTests.cs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/mod.rs
  - dev/Fyrox/examples/2d.rs
doc_type: architecture-audit-and-measurement-plan
status: proposed
measurement_status: pending_clean_current_source_profile
created_at: 2026-08-09
---

# MVP F2 渲染提交架构审计与测量计划

## 结论边界

本文件只记录已阅读到的结构事实、待测假设和后续门槛。尚未取得由当前源码构建的 Windows 产品 trace，因此没有 CPU/GPU p50、p95、p99、功耗、内存或规模结论；不得把本文的静态审计当作优化效果或验收证据。

目标是让 F2 的 `RenderableEmpty` 场景在不改变产品语义的前提下，先有可复现基线，再对确认的瓶颈实施结构优化。路径继续采用项目相对布局经统一物理解析，不引入虚拟 URI、多层前缀或 Windows 专用兼容分支。

## 范围与不变量

责任边界是 `zircon_runtime::graphics::runtime::render_framework`，其上游是 `zircon_app` 的进程/事件宿主，其输入输出只经过 runtime/framework 的中性 DTO。优化不得让应用宿主、编辑器类型或 F2 特判穿透到渲染框架。

必须保持：

1. `zircon_app` 只负责窗口、事件循环、动态会话和 profiling 生命周期；场景抽取与提交仍属于 runtime。
2. RHI/渲染可变状态只存在明确的单一所有者和顺序中，不能为了缩小锁而制造并发访问或重入。
3. F2 的动态 Camera、静态 Sun/Cube、输入响应、首帧和静态第二帧的视觉/反馈语义保持一致。
4. 所有新增采样、ETW、截图和报告工件写入 `E:\ZirconBuilds\mvp-perf\...`；受管构建 target 只使用获批的 `E:` target 根。不得把工件写入 `C:`。

## 当前源码提交图

当前路径为：

`zircon_app event/redraw -> runtime dynamic session -> WgpuRenderFramework::dispatch_runtime_frame_submission -> RenderSubmissionScheduler -> WgpuRenderFrameworkCore operation owner -> submit_runtime_frame_locked -> camera loop -> build submission context -> RenderFrameworkState lock -> prepare/render/feedback/output/history/stats`。

静态阅读确认的事实：

- `WgpuRenderFramework` 持有 scheduler；共享 `WgpuRenderFrameworkCore` 才持有 `state: Mutex<RenderFrameworkState>` 与 `operation_lock: Mutex<()>`。这两个 owner 不是可替换的通用 mutex，而是现有 RHI 顺序所有权的边界。
- 同步提交先取得 `operation_lock` 再执行；单槽 pipelined worker 也必须取得同一把操作锁，且 producer 等待 worker 已开始。因此它允许宿主与已启动 worker 的有限重叠，但不允许两个提交同时拥有渲染操作。
- `build_submission_context` 不是无锁准备：可见路径用短状态锁读取 budget、viewport snapshot、renderer asset manager 和 hydration-cache handle；history、pipeline compile 和自动 virtual-geometry helper 仍经 framework 状态取得输入。它不是可直接搬到任意 worker 的纯 DTO builder，每次 wait 的分布需要 trace 验证。
- `submit_runtime_frame_locked` 在取得 `RenderFrameworkState` 后完成 prepare、pipeline render、capture、feedback、output/history/stats 更新。锁覆盖实际范围需要 trace 验证，不能仅凭代码判断为瓶颈。
- runtime 已有 `render_framework.wait operation_lock`、`render_framework.wait state`、submission/context/prepare/render/feedback 等 profiling scope，可作为首轮 attribution 基础；`zircon_app` 的 `profiling` feature 会转发到 runtime。
- `CompiledGraphCache` 已保存 hit/miss/eviction/entry 统计，且现有第二帧测试要求稳定帧不新增 miss；基线必须导出这些计数，而不能将第二次查询先验视为重复编译。
- `RenderSubmissionConfig` 的默认值是同步，但动态 session profile 会显式写入 pipelined 配置；F2 产品基线必须记录其实际配置，并以同步配置作为同源码的对照，而不是混用两者的数据。

## Unreal 为主的架构对照

Unreal 是本轮结构决策的主参考，而非把其实现直接移植：

- `RenderingThread.h` 的 `RenderCommandPipe::FCommandList` 在消费前显式 `Close()`；它 move command/function，按已关闭的链表消费，并在发布新 context 前让旧 context 由已经排定的 task 消费。这给出 Zircon 的最低类比：跨线程提交必须是已封口、单一所有者可消费的包，而不是指向可变 render state 的回调。
- `RenderGraphBuilder.h` 要求传入 pass 的 parameter struct 在加入后不可变，并明确 lambda 默认延迟到执行期；它用 workload、resource access 和 setup-task wait point 描述可并行的工作。`RenderGraphBuilder.cpp` 在 `Execute` 中于编译/执行边界等待 setup tasks。并行来自可证明独立的准备工作，而不是松开未知可变状态。

Graphics 的 `ProfilingScope` 为 command-buffer replay、calling-thread inline CPU 和 GPU recorder 使用不同采样面；其 `ProfilingSamplerWithCommandBufferTests` 先断言 inline sample，再在 `Graphics.ExecuteCommandBuffer` 后等待 render-thread CPU/GPU sample。`RenderGraph` 又将 `BeginRecording` 与 `EndRecordingAndExecute` 显式分开。Zircon 的 P1 也必须把 producer wait、worker replay、CPU submit 与 GPU 时间分开，不能把一个 wall-clock span 命名为全部渲染时间。

Fyrox 的 `Engine::render` 保留单一 `GraphicsContext::Initialized` mutable renderer，并在 `Renderer::render_frame -> render_and_swap_buffers` 内清理 frame cache、依次渲染 scene/UI、再 present。其 `examples/2d.rs` 将 game plugin、window configuration 与 executor 生命周期保持在产品入口。它是反例约束：若 P1 显示没有可重叠的 immutable 准备阶段，Zircon 应收窄已测 renderer 阶段，而不是虚构并行。Zircon 保持既有 app-host/runtime 边界，不移植 Fyrox 的事件循环。

## P1 后的内部边界候选

这不是实现授权。只有 H1 或 H3 被当前源码产品 profile 证实后，才评审下列内部结构：

1. `FrameSubmissionContext` 继续承载抽取与编译后的中性描述；先证明其中哪些字段、generation 与资源引用在 enqueue 后不可变。
2. 新的 runtime-internal sealed packet 只能持有不可变描述、显式 viewport generation 和由唯一 renderer owner 消费的准备输入；它不能暴露 RHI、`RenderFrameworkState`、编辑器对象或跨帧可变缓存。
3. renderer 保持唯一 operation owner，执行完成后只发布一个有 generation 的反馈/统计 delta；history、capture、viewport products 和 diagnostics 仍在顺序消费点提交，失败时不得部分发布。

实现测试必须先覆盖：一提交延迟 feedback、两 camera 的 generation/顺序、capture 失败、history invalidation、packet 取消和回收、feedback 不能回写已发布 output snapshot，以及同步/pipelined 两种配置下的视觉与统计一致性。只要其中一项需要共享可变 `RenderFrameworkState` 才能越过 worker 边界，该字段仍留在 renderer owner，不能通过 clone 或全局缓存伪造独立性。

## 待测假设与决策门

| 假设 | 可反驳指标 | 若证伪 | 若确认后的候选设计 |
|---|---|---|---|
| H1：`operation_lock` 或 `state` 等待主导提交尾延迟 | 两个 wait scope 的 p95/p99、ETW ready time 与 context switch | 保留锁边界，优先检查 prepare/pipeline/GPU | 先提取不可变提交描述，再维持单一 renderer mutation owner |
| H2：静态第二帧仍主要消耗 context/prepare/pipeline，而非等待 | 分 scope CPU 时间、GPU pass/timestamp、upload/copy 数 | 依数据定位真实阶段 | 仅优化被测阶段，禁止全链路重写 |
| H3：单槽调度的 backpressure 对 F2 产生可观 producer stall | `wait_previous_submission`、`wait_worker_start`、`wait_pending_submission`、`pending_depth`、frame latency，以及未来显式导出的 render worker utilization | 不改变队列模型 | 比较明确 pacing 或提交包边界，保留一提交延迟 feedback 契约 |
| H4：每帧两次 graph-cache 查询导致可观 CPU 或 cold miss 成本 | cache hit/miss/eviction/entry、context scope、cold/steady trace | 保留现有两阶段配置推导 | 仅在键/语义可证明等价时合并或延迟查询 |

没有一个假设在取得数据前可直接转化为代码变更。

## 受管 Windows 基线方案

1. 通过仓库 Windows 受管验证路径产出带 `target-client,profiling` 的当前源码运行时；记录提交、dirty diff 摘要、feature、target 根、二进制哈希和 GPU/驱动信息。不得执行直接 Cargo 命令，也不得使用历史或来源不明的二进制。
2. 用 `ZIRCON_PROFILE_CAPTURE=1`、`ZIRCON_PROFILE_OUTPUT_ROOT=E:\ZirconBuilds\mvp-perf\<session>`、有限 frame/span/counter 容量采集内建 timeline；需要 Perfetto 时显式启用对应 feature 和环境变量。
3. 同一 `RenderableEmpty` 项目至少进行 3 次冷启动/首帧和 3 次稳定帧窗口采样；产品路径使用实际 pipelined 配置，并采集同源码同步对照。冷启动使用现有 `ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME=1`；稳定窗口使用 `ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES=120` 且不设置首帧变量。后者只在 120 次成功 native/fallback present 后正常退出，确保 trace 覆盖连续真实帧而非单帧截图。
4. 同时用 WPR/xperf/WPA 在相同会话采集 CPU sampling、线程调度、context switch、等待、工作集和磁盘 I/O，ETL 也只写入同一 `E:` 会话目录。GPU pass、draw/dispatch/copy/upload 以 wgpu timestamp/marker 和可用的 RenderDoc capture 交叉验证。
5. 报告按中位数、p95、p99 和极差输出：frame、各 scope、锁等待、队列延迟、CPU/worker 利用率、工作集、I/O、GPU pass、copy/upload。功耗只在设备/驱动提供可校准能耗计数或外接功耗计时报告；否则明确写为“未测”，不得估算或伪造零值。

### P1 当前准备状态（非验收）

- `tools/mvp/Build-RenderExtractProfilingInputs.ps1` 已将受管 runtime profiling build 收敛为单一物理输入目录中的同目录 EXE/DLL 对：只接受 `D:`、`E:` 或 `F:` 下的专用 `ZirconBuilds` 输出目录，经统一 Windows resolver 执行文件操作，并发布包含 source fingerprint、profile、feature、字节数和 SHA-256 的双产物测量输入清单。路径与源码预检不创建输出根；只有受管发布实际开始后才允许落盘，发布失败的现存工件不作推测性删除。该脚本不采集 trace、不生成截图，也不构成 P1 结果。
- `tools/mvp/Capture-RenderExtractBaseline.ps1` 只消费上述清单：预检要求 manifest、EXE 与 DLL 同处其声明的物理输入目录，固定三者路径和 SHA-256；在产品启动前，将已校验的三份字节以 create-new 语义冻结到本次 invocation 的 `inputs/` 副本，并且只从该副本启动。每次运行前仍重新核对原输入身份，任何输入替换都会中止采集而非混入新二进制。子进程以解析后的项目根为 working directory 并传 `--project .`，运行时库保持现有同目录相对名 `zircon_runtime.dll`。产品 project 必须是 F1 创建的项目或示例项目，脚本拒绝仓库 `templates/projects` 源模板，避免运行时 `.zircon` 缓存污染模板；所有路径仍由现有物理 resolver 处理。manifest/source 预检会在创建会话工件前完成；随后以 Windows 独占、close-delete 的会话租约防止两个遵循该协议的采集进程复用同一会话目录。每次调用还将日志、PNG、trace、profile 与冻结输入写入租约生成的随机 invocation 子目录，写入的 baseline summary schema 为 v2 并绑定该 `invocation_id` 和每个 run 的输入哈希；脚本写出的文本一律以 create-new 语义落盘，绝不覆盖外来同名文件。租约和随机目录用于防止意外复用与覆盖，不构成同一 Windows 身份下任意外部进程的安全隔离；实际产品运行开始后保留所有已写出的工件用于诊断，脚本不会猜测目录所有权再删除它们。它为 `runtime-pipelined` 首帧、`runtime-pipelined` 稳定窗口与同步 `runtime` 稳定窗口各安排至少 3 次产品运行，并要求每次导出非空 timeline、hotspot、counter、summary 和 PNG；传 `-UseWpr` 时同一 run 还必须导出非空 CPU ETL。所有会话工件写入单个 `E:\ZirconBuilds\mvp-perf\<session>` 目录。
- `tools/mvp/Write-RenderExtractBaselineReport.ps1` 在 raw summary 写入后自动执行，只从同一会话的 native timeline 归约 process、frame、span 与 counter 采样，并以 runtime hotspot 相同的上取整 percentile 规则输出 median/p95/p99、均值和极差。summary 和每份用于归约的 timeline 都只读一次：报告从同一 byte snapshot 同时获得 JSON 与 SHA-256，不会把一份内容的指标归因给另一份内容哈希。它只接受上述三种场景及其对应 profile、每种至少三次成功运行，并验证每条 run 的输入哈希与 capture summary 一致。报告对 raw summary、每个 profile 工件、每个 invocation-scoped PNG 及可选 WPR ETL 记录 SHA-256；报告 JSON 与 Markdown 同样以 create-new 语义发布，已有同名文件会使调用失败而不会被覆盖。锁等待只识别 `render_framework.wait/{operation_lock,state}`；队列回压只识别 `render_framework.scheduler` 的三个 wait span 和 `pending_depth` counter；worker utilization 只接受未来的 `render_framework.scheduler.worker_utilization` counter，因此当前为 `not_emitted`，不会把 ECS 指标误报为渲染数据。GPU 时间、系统功耗、工作集、I/O 和 WPR CPU 调度没有校准且已解析的采样时明确为未测，不从通用 counter 或未解析 ETL 推断。
- 当前源码已具备 `ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES` 的正整数退出上限，首帧开关仍映射为上限 `1`；staging 子进程会清除未显式传入的多帧变量，避免 inherited environment 污染 MVP 首帧证据。
- scheduler 已新增 `wait_previous_submission`、`wait_worker_start`、`wait_pending_submission` 与 `pending_depth` 观测点；它们不改变单槽容量、RHI 所有权或 feedback 时序。
- P1 现在还在 `app` timeline stream 记录每次 `runtime_entry.frame_pump`、
  `frame_pump_suppressed`、成功 `runtime_tick` 与实际 `redraw_request`，并记录
  native present、fallback capture request、fallback RGBA bytes、fallback CPU present、
  successful presented frame 及显式 PNG capture request/RGBA bytes。它们是事件计数和
  CPU-owned RGBA payload 证据，不改变 cadence、surface、readback 或 first-frame 的语义。
  `Write-RenderExtractBaselineReport.ps1` 将这些名称分别归约为 `app_cadence` 和
  `surface_presentation` 覆盖；其状态仍只能说明 counter 是否发出，不能推断 GPU 时间、
  WPR 调度或系统功耗。
- 既有 profiling-input/baseline-capture 工具契约已完成 Pester 检查；本轮 App counter/report
  扩展已完成 Rust 格式、PowerShell parse 与 scoped diff hygiene，但其 Pester 回归等待 UI12
  validation 窗口。尚无当前源码的受管 Windows 二进制、ETW、profile JSON、产品 PNG 或数值，
  因此 P1 仍为 pending，H1-H4 仍未裁决。

## 优化实施门槛

P0 已完成路径和参考源码阅读，P1 为当前源码的受管基线。只有 P1 能把热点归因到某一假设后，才选择下列一项并单独设计：

- 锁等待低：保持状态所有权，局部优化已测的 context/prepare/pipeline 工作。
- 状态锁主导且提交描述可证明不可变：建立 submission packet，锁外构造、锁内按顺序消费；补充重入、代际、history/feedback 一致性测试。
- 调度回压主导且锁不是根因：调整 pacing 或队列边界；验证一提交延迟 feedback、窗口退出和多 camera 顺序。

每项实现前需有独立设计审查，实施后对同一 workload 复测并与 P1 按绝对值和百分比对比。任何功能、截图、输入、第二帧稳定性或资源所有权回归都会否决优化。

## 2026-08-11 提交路径复审

本次在当前源码上重读了 `pipelined/queue.rs`、`wgpu_render_framework.rs` 与两条 runtime/extract submission 路径，并对照 Unreal `RenderGraphBuilder` 的依赖收敛方式、Lyra 的 project asset startup 以及 Fyrox 的完整游戏示例输入循环。结论仍然是先测量，不能据此直接改队列容量或锁粒度：

1. `PipelinedSubmissionQueue::submit` 会先消费此前 pending result，再向 bounded(1) sender 发送新 payload，随后等待 worker 的 started 信号。因此它只允许一个 renderer-owned submission 在途；“pipelined”表示宿主在 worker 执行时可能推进其他工作，不是积压多帧或允许 RHI 并发。
2. worker 只在取得 `WgpuRenderFrameworkCore::operation_lock` 后发送 started。因此 `wait_worker_start` 同时包含 worker 调度和 operation-lock 获得时间，不能单独作为线程创建或调度开销解释。
3. 两条 submission 路径都在 `build_submission_context` 后取得 `state`，并让该锁覆盖 prepare、renderer execution、history/feedback 和统计写入中的主要区间。锁竞争是否主导尾延迟必须由 `render_framework.wait/{operation_lock,state}` 的 raw span 数据判断；若不主导，拆锁只会扩大可变 renderer 状态的并发面。
4. Unreal 的 RDG 执行仍以已收敛的资源依赖和明确 owner 为前提，Lyra 先扫描 registry 再让关键 project asset fail-closed，Fyrox 示例将 OS event 规约为 input state 后由单一 scene update 消费。它们都不支持在 Zircon 尚未证明 immutable submission packet 的边界前，以更大队列或无主状态访问换取表面并行度。

因此 P1 报告必须优先输出 `wait_previous_submission`、`wait_worker_start`、`wait_pending_submission`、`render_framework.wait/{operation_lock,state}`、`pending_depth` 和 frame/span 分位数；仅在未来导出精确的 render worker counter 时才纳入 worker utilization。只有这些数据归因到某个假设后，才选择锁外 immutable packet、pacing/queue 边界或局部 renderer 阶段之一；当前没有性能数值或功耗数值，也没有优化效果可报告。

## 2026-08-12 产品输入身份传输测量

此项只审计 P1 输入清单的正确性与构建前置成本，不是渲染帧、GPU 或产品性能基线。Windows PowerShell/.NET Framework 在 `Process.StandardInput` 上构造默认 UTF-8 `StreamWriter` 时会向 Git 的 `hash-object --stdin-paths` 写入 preamble；第一个相对路径会被解释为带 BOM 的不存在路径。当前实现改为受限的 `git hash-object --no-filters -- <file...>` 参数批次：路径仍从 Git 的 NUL 清单读取，拒绝换行和双引号，且每个批次低于 Windows 命令行预算；不存在 shell 转义、虚拟 URI 或额外路径前缀。

回归夹具在 Windows PowerShell 中将 `.codex/config with spaces.toml` 作为首个变更路径，并用 320 个长路径强制跨批。该夹具的四次完整指纹计算实测约 104 秒，明显高于短路径夹具；它只证明当前内容身份和分批顺序正确，不能归因给单一 Git 子步骤，也不构成将批量预算、并发数或缓存策略改动的授权。

当前共享工作树在产品预检期间出现真实的动画 runtime 源文件变化，因此两次源码指纹不一致并按设计停止在 Cargo 启动前，未发布任何二进制。后续若产品输入身份成本成为已稳定源码快照下的瓶颈，必须分别测量 raw diff、name-only 清单、每个 `hash-object` 批次和未跟踪文件哈希的耗时，再提出唯一的改动候选；在此之前保持现有 fail-closed 语义，不把工具耗时冒充为 P1 渲染热点。

## 后续里程碑

- P1：受管 Windows 当前源码基线与产品截图/trace。
- P2：根据 P1 数据选择唯一的结构优化方案。
- P3：测试先行实施，执行源码和产品验证。
- P4：相同配置复测、二次代码审查、量化结果归档；达到验收条件后才创建 scoped milestone commit，并将量化数据发送至企微。
