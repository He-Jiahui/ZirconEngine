---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: play-process-output-byte-budget
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/core/play/process_backend/child.rs
  - zircon_editor/src/core/play/process_backend/mod.rs
---

# Editor14 Play process output byte预算

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-552 Play process output byte budget
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Play output reader、drain budget 与 blocking-I/O job 资源配额由 Editor14 所有，并联动 Runtime11/Editor04。

## 失败现象与复现证据

stdout/stderr queue虽为1024 entries，但两个手建reader thread均用`read_until('\n')`，无换行单行可无限扩张临时Vec；poll过去每tickformat全部backlog。本轮只把live drain收为64 lines/poll并让terminal join/cleanup离开active mutex，仍没有max-line/max-bytes/time/oldest-age。

## 最低共享层根因

entry cap 不能约束无换行单行的临时 buffer bytes，手建 reader thread 与 poll backlog 也缺少统一 count/time/oldest-age admission owner。

## 架构修复验收

- Runtime11 blocking-I/O owner提供bounded line decoder、max line/queue bytes、count+time drain、oldest age及drop/truncate diagnostics；资源预算包含reader threads。
- 1/1k/1M lines、64B/1MiB/1GiB line、30/120Hz poll记录buffer/queue bytes、threads、age/drop/truncate/format/p95/RSS；单行和总queue硬有界，poll不越帧预算，stop/drop无hang。
- 保持stdout/stderr标识、UTF-8/lossy/error、terminal剩余输出与cleanup；Cargo/F4和独立review通过。

## 禁止临时方案

- 禁止仅靠1024 entry cap宣称内存有界；禁止为每次Play无限新增detached reader。
- 禁止在持active/controller锁时join、wait或递归删除snapshot目录。

## 修复结果与回传

Open state: `64 lines/poll与terminal锁外finish已止损；等待byte/time/age预算和统一blocking-I/O owner`。

2026-08-10 前向路由：共享 blocking-I/O 责任已登记到
`docs/plans/zircon_runtime/runtime/11/failure-2026-08-10-blocking-io-process-output-budget.md`。
Runtime11 必须先提供通用 bounded stream ticket；Editor14 仅在该 contract 落地后替换私有 reader lifecycle，
不得在当前 process backend 复制第二套 owner。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-22 | `open / handoff-recorded` | 由逐文件性能审查登记 PERF-MVP-552。 | 原始 reader `read_until` 无换行行缓冲不受 byte cap 约束；queue entry cap 不足以证明内存有界。 |
| 2026-08-10 | `open / Runtime11-forward-handoff-recorded` | 将跨调用的 stream decoder、reader admission、queue bytes、age/time drain 和 terminal cleanup 责任前向路由到 Runtime11；Editor14 保留 UI diagnostics 消费与后续接线责任。 | `docs/plans/zircon_runtime/runtime/11/failure-2026-08-10-blocking-io-process-output-budget.md`，其 `status: open`；未运行 Cargo、未生成 fixed return。 |
