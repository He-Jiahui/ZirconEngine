---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: mvp-performance-observability
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
tests:
  - profiling recorder full-capacity append ordering
  - task queue depth and enqueue-to-start latency pressure tests
  - diagnostics-off overhead comparison
---

# Runtime07：MVP 性能诊断存在自耗时与关键盲区

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F0 profiling recorder 与 task diagnostics 静态审查
- 来源证据：`docs/plans/performance/01/2026-07-17-task-system-static-review.md`
- 修复责任计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 交接原因：诊断数据模型和性能工具自耗时属于 Runtime07，共享消费者不能各自解释或修补错误指标。

## 失败现象与复现证据

profiling recorder 的默认 16,384 容量 ring 原以 `Vec::remove(0)` 淘汰，每次满容量追加都线性搬移；性能工具自身会在长采样中制造 CPU 开销。该点已在性能 Session 改为 `VecDeque::pop_front/push_back`，等待 Cargo 验收。

time diagnostics 原先每帧对四项稳定序列分别加锁，并重复分配 path/unit/tags 与排序 metadata。性能 Session 已增加 borrowed static-series fast path、storage-reuse 回归和单锁批量写入，等待 Cargo 与 allocation benchmark。

任务诊断目前没有 queued、active、enqueue-to-start delay、execution duration、panicked/cancelled；同时 `tasks.main_thread_wait_ms` 不校验调用线程。缺少这些指标时无法可靠区分主线程堆积、worker 饱和、依赖等待和历史失败。

## 最低共享层根因

profiling 容器和 task diagnostic schema 都由 runtime diagnostics owner 定义；只有在这一层修复，runtime、editor 与插件才能消费同一套准确且低开销的数据。

## 架构修复验收

- 接收并验证 profiling ring 修复，保留 snapshot 顺序与容量语义。
- 接收并验证 time static-series fast path；百万次 record 对照必须证明 metadata allocation 消失，动态 record 语义不变。
- 与 Runtime11 定义低开销 task 指标；诊断关闭/采样时不得引入每任务锁热点。
- 用 1/2/N worker、短/长任务、依赖 fanout 夹具输出 queue/active/lag 与 WPR 对照。

## 禁止临时方案

- 不得用 `scheduled-completed` 单一差值宣称 queue depth。
- 不得把缺失 GPU/CPU counter 写成零值或把旧 capture 当当前源码基线。

## 修复结果与回传

Open state: `profiling ring 已实现待验证；任务可观测性待 Runtime07/11 修复`。

## 2026-08-13 前向续作

- JobScheduler 现将 queue wait 与 execution duration 分开：仅成功开始的任务保存 execution 起点，只有正常或 panic 的终态累积 `tasks.execution_samples`/`tasks.execution_ms`；未启动取消与 diagnostics disabled 均不建立样本。
- queued、active、dependency wait、queue wait、execution、panic 与 cancel 已在同一 shard snapshot/report surface 输出。main-thread wait 归因仍需 Runtime11 的执行域边界定义，故 failure 不关闭。
- 新增真实 `schedule(...).wait()` 回归：一个成功 worker task 必须同时产生一个 completion 与一个 execution sample，断言不依赖机器相关耗时阈值。
- 未运行 Cargo 或性能矩阵；保持 open，等待受管 Windows 行为与压力证据。
