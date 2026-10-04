---
related_code:
  - zircon_app/src
  - zircon_runtime/src
  - zircon_editor/src
  - zircon_plugins
  - zircon_runtime_interface/src
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityQuery.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CpuProfilerTrace.h
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
  - dev/godot/core/object/worker_thread_pool.h
doc_type: performance-review-ledger
status: in_progress
---

# ZirconEngine 性能审查与结构优化计划

## Context

本计划以 MVP `F0-F5` 为最高优先级，先建立 current-source 的逐 `.rs` 文件审查、可复现 CPU/线程/I/O/GPU/内存基线，再按 Unreal 对齐的 owner、generation、阶段屏障和任务调度模型实施结构性优化。现有 `docs/plans/performance/01-*` 与 `02-*` 已记录大量静态发现；本计划不把静态发现伪装成动态性能结论，也不重复复制其他 owner 的实现正文。

当前 MVP 仍受 `docs/plans/mvp/index.md` 的 `00 -> F0 -> F5` 顺序约束。高级渲染、AI、网络和非 MVP 插件能力只做审查与路由，不能抢占 MVP 实现资源。

## 目标

- 证明启动/退出、最小场景帧循环、输入、项目资产、编辑器选择/修改/保存/重开路径的真实瓶颈。
- 定位冗余全量扫描、错误复杂度、主线程同步 I/O/锁等待、未使用并行调度、过高触发频率和重复分配。
- 仅在有 E4 复现证据且不违背 `performance/02` hard-cut 方向时直接修复简单叶子问题；结构性问题转交 canonical owner 计划。
- 所有报告、Cargo target、trace、RenderDoc capture 写入 `E:/D:/F:`，禁止 C 盘和仓库内 `target/`。

## 参考依据

- Unreal Mass query 明确以 archetype cache、chunk query 和 `ParallelForEachEntityChunk` 作为可并行边界，Zircon 的 ECS 优化必须验证 access plan/chunk 并行，而非仅替换容器。
- Bevy multi-threaded executor 以 system access conflict、dependency、exclusive/non-Send 状态决定并行，作为 Runtime schedule 的算法依据。
- Unreal CPU profiler 明确 static marker 低开销、per-thread event 语义；Zircon 的全局 recorder/动态字符串只可作为待测风险，不能直接宣称开销。
- Godot WorkerThreadPool 以 worker、group、任务分页/等待管理后台工作；Editor jobs 与 Runtime TaskGraph 应验证是否重复建立调度 authority。

## 里程碑

### M0：基线与账本

- [ ] 登记全部模块到 `pending.md`，逐 Rust 文件记录覆盖状态；已验收模块才移入 `review.md`。
- [ ] 固定源码 fingerprint、toolchain、Windows/GPU/分辨率/vsync、fixture 和 E/D/F 产物根目录。
- [ ] 执行最小 Runtime/App/Editor/Plugin focused Cargo batch；失败记录 current-source 根因，不能用旧 binary 或源码测试冒充通过。
- [ ] 若产品可启动，再执行 3 次 F0/F2/F4 脚本，记录 p50/p95/p99、主线程/worker、RSS、I/O、线程峰值、队列/锁等待；无 GPU/功耗工具时明确 unavailable。

### M1：Runtime/App MVP 热路径

- [ ] `zircon_app/src/entry/**`：帧 cadence、redraw/request、input/event、surface/present、host request drain。
- [ ] `zircon_runtime/src/core/runtime/**`：TaskGraph、pool、events/messages、diagnostics、config、shutdown。
- [ ] `zircon_runtime/src/scene/**`：World/ECS schedule、change detection、hierarchy、DynamicScene、render extract。
- [ ] `zircon_runtime/src/asset/**` 与 MVP graphics：项目/资产加载、CPU prepare、GPU upload/submission、frame pacing。
- [ ] 重点验证全量 World/DTO clone、`O(N×T)` reflection capture、domain-global dirty、每帧 schedule/rebuild、主线程同步等待。

### M2：Editor MVP 与 Jobs/Asset

- [ ] `zircon_editor/src/core/jobs/**`：merge/cancel、bounded journal、shutdown quiescence、blocking lane、UI pump。
- [ ] `zircon_editor/src/core/asset/**` 与 `ui/host/editor_asset_manager/**`：registry/catalog authority、import/reimport、preview、save/autosave。
- [ ] `zircon_editor/src/scene/**` 与 MVP `ui/**`：selection/Inspector/command、layout/recompute、pointer/scroll、重复 projection。
- [ ] 优先建立规模夹具和分配/锁计数；不为“验收队列”停止非验证性的落地工作。

### M3：插件与 ABI

- [ ] `zircon_runtime/src/plugin/**`、`zircon_plugins/**`、native loader、App/Editor discovery/load/selection。
- [ ] 先验证 disabled/未选择插件零代码执行、ABI/版本/卸载 barrier，再测 discovery/parse/registration/replay/hot reload/callback 成本。
- [ ] 只选择一个真实、MVP 可达插件验证 static/source/native parity；不批量扩展 metadata-only dist shell。

### M4：结构优化顺序

1. 先修复影响测量可信度与 MVP 生命周期的 P0：统一 TaskGraph/job shutdown、profiler generation/per-thread buffer、插件 load admission/ABI boundary、current-source 编译阻塞。
2. Runtime 采用 compiled access/affinity plan、World commit generation、dirty frontier、camera-neutral sealed extract；DynamicScene 按 archetype/serializable columns 后台分页处理。
3. Editor 采用 Runtime registry 单一真值、immutable/delta projection、bounded import/save/preview job、bounded event journal。
4. Graphics 只有在 CPU/GPU timestamp 与 RenderDoc current MVP capture 具备后，才调整 upload reuse、view-independent payload 和 pass/queue。
5. Plugin 采用单一 catalog/version/ABI generation 与 lifecycle ticket，再优化回调/加载成本。

## 验证门

- 静态 E2；有 `dev/` 对应实现证据为 E3；真实 focused test/benchmark/product trace 为 E4。
- 同一 fixture、profile、硬件和显示设置对比优化前后，报告样本数、warmup、方差与 p50/p95/p99。
- 稳态无变化时 full World extract、全量 GPU upload、全量 editor recompute、plugin rediscovery 应为 0；若无法证明，保持 pending。
- RenderDoc 只捕获当前可运行 MVP 产品；高级旧 capture 不作为结论。
- git commit 只在里程碑证据完成并得到用户确认后执行；企微发送需用户提供受控接口并在发送前确认摘要。

## 跨计划路由

- `docs/plans/mvp/`：F0-F5 产品门与连续运行证据。
- `docs/plans/zircon_runtime/runtime/{03,04,07,08,10,11,12}/`：调度、资产、热路径、ECS、ABI、任务、输入。
- `docs/plans/zircon_runtime/render/{01,02,03,04,17}/`：RenderGraph、GPUScene、visibility、culling、profiling。
- `docs/plans/zircon_editor/editor/{01,02,05,08,09,11,14}/` 与 `editor_ui/{02,08}`：编辑器 authority、资产、作者路径、任务、UI。
- `docs/plans/optimize/zircon_plugins/01-*`：插件 SDK/发现/装载/卸载/发行。

## 状态

- [ ] M0 基线与账本
- [ ] M1 Runtime/App MVP
- [ ] M2 Editor MVP
- [ ] M3 Plugins/ABI
- [ ] M4 结构优化与回归

详细发现使用既有编号计划；本文件只拥有性能优先级、测量门和跨计划路由。
