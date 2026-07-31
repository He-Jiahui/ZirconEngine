---
plan_source: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
failure_source: docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md
related_code:
  - zircon_editor/src/core/jobs/mod.rs
  - zircon_editor/src/core/jobs/event_sink.rs
  - zircon_editor/src/core/jobs/progress.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/state.rs
tests:
  - zircon_editor/src/core/jobs/tests/pump_contract.rs
  - zircon_editor/src/core/jobs/tests/background_storm_contract.rs
  - zircon_editor/src/core/jobs/tests/scheduling_contract.rs
status: implementation-landed-managed-validation-pending
---

# Editor14 有界 job pump 与索引化准入

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | W1 job pump 帧预算与 progress 合并 | `source_landed-static-green` | test-first 增加 count/time budget 延后 8 个生命周期边缘且不丢失、100 次 progress 只投递 latest value 的合同。生产实现以 `JobEventQueue` 保存不可丢的 Started/terminal 边缘，按 JobId 合并尚未消费的 Progress；`pump_events()` 固定默认 `64 events / 1 ms`，`pump_events_with_budget()` 提供显式测试/诊断预算。`background_storm_contract` 的机器可读输出从 `numeric_budget=undefined` 硬切为 `pump_count_budget=64 pump_time_budget_us=1000`。 | 待 managed Windows `pump_contract` 与 1000-job storm；并发生产/消费下的边缘顺序由独立复审确认。 |
| 2026-07-17 | W1 pending admission 索引化 | `source_landed-static-green` | `Vec + 全量 filter/min/remove` 已硬切为 `PendingJobQueue`：JobId owner map、priority/category ready bucket、dependency waiting count、reverse dependency index 与 dependency pin count。每轮最多探测 3 priority x 7 category = 21 bucket；同 priority 跨类别仍按最小 JobId，类别配额与 after/mutex handle 合同保留。新增 1,000/10,000 blocked pending probe，要求 10k probe 不超过 1k 的 11 倍；新增终态 dependency pin 回归。文件级 `rustfmt --check` 与 scoped `git diff --check` 通过。 | 待 managed Windows scheduling/full jobs 聚焦门、1k/10k 原始输出、WPR 帧预算与 worker 利用率；通过前 failure 保持 open。 |
| 2026-07-18 | W1 admission inventory 复审硬化 | `source_landed-coordinator-validation-blocked` | 独立源码复核发现 `pending.rs` 私有复制 priority/category 数组会让未来 enum 变体编译通过但永远不被准入。test-first 先证明手写 enum/`ALL` 仍非穷尽，再把两个 enum 与各自 `ALL` 硬切为同一 `define_job_enum!` 输入原子生成；pending 只消费 `JobPriority::ALL` / `JobCategory::ALL`，21-probe 上限由两者长度计算。静态合同拒绝本地库存、固定长度 `ALL` 和非宏声明源；三文件 `rustfmt --check`、atomic inventory guard 7/7 与 scoped `git diff --check` 通过。 | managed Cargo 尚未启动：Coordinator01 的 executable-owner pending reservation 在绝对过期后仍占 FIFO head，已登记 `pending-cpu-reservation-absolute-expiry-not-enforced` failure。待该 failure fixed return 后重建 current-source reservation，再执行 focused/full jobs、1k/10k 原始输出、WPR 与独立复审。 |

## 未完成项目

- `failure-2026-07-17-job-pump-budget-and-pending-scan.md` 仍为 open；只有 current-source Cargo、性能证据和独立复审全部通过后才可生成 canonical fixed return。
- 当前切片不降低 worker 并发、不创建 Editor 私有线程池，也不恢复旧 MPSC 无界 drain 兼容入口。
- WPR/真实 retained frame 仍需记录 pump p50/p95、队列峰值和 worker 利用率；墙钟观测不能替代固定 count/time 预算。
- Coordinator01 的 `pending-cpu-reservation-absolute-expiry-not-enforced` 是当前验证基础设施阻塞，不属于 Editor14 业务修复；本计划不释放、续约或改写任何 foreign reservation 来绕过 FIFO。
