---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: job-pump-budget-and-pending-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/jobs/system/state.rs
  - zircon_editor/src/core/jobs/tests/background_storm_contract.rs
tests:
  - 1000/10000 job promotion complexity benchmark
  - count/time-budgeted main-thread pump storm
  - progress coalescing preserves started and terminal edges
---

# Editor14：job pump 无帧配额且 pending admission 接近 O(n²)

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：Editor jobs 19 个生产 Rust 文件静态审查与 1,000-job storm 合同复核
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：job admission、worker→main 回流配额与 progress 合并属于 Editor14 调度契约。

## 失败现象与复现证据

`JobEventPump::pump` 对 unbounded MPSC 一直 drain 到空，并逐事件同步发布；retained host 每 tick 调用它。现有 1,000-job storm 明确输出 `numeric_budget=undefined`，未设置每帧 count/time SLA。

pending 使用 `Vec`；每次 promote 全量 `filter/min_by_key`，再 `remove(index)`，每个 job 完成又重跑。长队列累计扫描/搬移接近 O(n²)。

## 最低共享层根因

Editor job model 只有 active-category 并发配额，没有主线程 delivery 配额、queue age/peak 或按 priority/category 可增量准入的数据结构。上层 retained tick 无法安全决定何时停止 drain。

## 架构修复验收

- 1,000/10,000 job 分离测 submit、promotion、completion 和 pump p50/p95/allocations。
- 定义 count 与 time 双预算；Started/terminal 保序不可丢，Progress 可按 JobId latest-value 合并。
- 选择 priority/category/dependency-aware ready queues 或索引，证明大队列不再二次增长。
- 当前源码 editor WPR 验证主线程帧预算、worker 利用率、shutdown 和公平性。

## 禁止临时方案

- 不得通过降低 worker 并发掩盖主线程 pump；不得丢 terminal/cancel/error 事件。
- 不得把 wall-clock 观察值当通过阈值，必须冻结可复现预算。

## 修复结果与回传

Open state: `Editor14 源码已建立 pump SLA、progress 合并、索引化 admission 与 enum 自有扫描库存；current-source Cargo、性能/WPR 和独立复审尚未完成`。

2026-07-18 静态复核发现并修复一项后续扩展饥饿风险：priority/category enum 与各自 `ALL` 由同一 `define_job_enum!` 输入原子生成，扫描库存不再由 `pending.rs` 私有复制，最大 probe 数由库存长度派生；测试拒绝恢复本地常量、固定长度 `ALL` 或非宏声明源。`rustfmt --check`、atomic inventory guard 7/7 与 scoped diff check 已通过。

当前 managed Cargo 被 Coordinator01 的 `pending-cpu-reservation-absolute-expiry-not-enforced` failure 阻塞：两个 executable-owner reservation 在持久化绝对过期后仍保持 pending/jobless 并占用 FIFO head。本 failure 保持 `open`；Editor14 不释放 foreign reservation，也不在缺失 current-source Cargo、1k/10k 原始输出、WPR 与独立复审时生成 fixed return。

2026-07-22 current-source补充：indexed admission与64 events/1ms pump仍成立，本轮把`TOPIC_JOB` parse从每tick收为构造期一次。新的最低open条件是submit/lifecycle queue entry+bytes+age背压，以及event sink三锁/稳定label clone收敛；只证明progress coalesce或pending scan已修不能关闭PERF-MVP-020。

2026-07-22全量tests复核补充：`background_storm_contract`仍用“must accept every storm job”锁定1,000个请求全部无条件入队，并把wall-clock明确降为非pass/fail baseline；`admission_scaling_contract`只证明1k/10k probe线性，不记录payload/label bytes、oldest age或100k/1M重复请求。因此验收必须先把submit结果建模为accepted/merged/backpressured，并冻结entry+bytes+age/RSS与pump p95硬门；不得以线性promotion或60秒watchdog替代有界性。
