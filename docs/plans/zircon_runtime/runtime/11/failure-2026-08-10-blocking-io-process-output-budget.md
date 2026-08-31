---
handoff_kind: failure
status: open
created_at: 2026-08-10
summary_slug: blocking-io-process-output-budget
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/
  - zircon_runtime/src/core/runtime/tasks/bounded_stream_io/
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/core/play/process_backend/child.rs
tests:
  - cargo test -p zircon_runtime --lib blocking_io --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib performance_source_guards --locked --jobs 1 -- --test-threads=1
---

# Runtime11: blocking-I/O Play process output byte budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：`failure-2026-07-22-play-process-output-byte-budget.md` / PERF-MVP-552。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：stdout/stderr 的 line decode、queue byte admission、reader 生命周期与 time/age drain budget 是跨 Play 调用的 blocking-I/O resource contract；Editor14 只应将已预算的记录消费到 UI diagnostics。

## 失败现象与复现证据

Editor14 当前 Play output 已有局部单行截断、queue-byte reservation 与每 poll count/byte/time drain 限制，但 `spawn_reader` 仍直接为每个 Play 创建 reader thread，Runtime11 的 `core/runtime/tasks/` 尚无可供该调用方复用的 bounded stream decoder/queue/lifecycle ticket。

因此 1/1K/1M lines、64B/1MiB/1GiB unterminated line、30/120Hz poll 与 stop/drop 场景无法由统一任务预算证明 reader threads、queued bytes、oldest age、drop/truncate 与 terminal cleanup 都受同一 owner 限制。现有 Editor14 保护只能止损，不能满足 PERF-MVP-552 的共享资源治理验收。

## 最低共享层根因

Runtime11 只有 keyed persistence-style bounded I/O lane；它没有用于 subprocess stdout/stderr 的 bounded streaming reader contract。line decode、entry/byte reservation、deadline/oldest-age drain、cancellation、terminal join 与 diagnostic counters 被留在 Editor14 process backend，导致每个 consumer 可重建一套 reader 生命周期和预算解释。

## 架构修复验收

- Runtime11 提供 folder-backed、可复用的 bounded stream I/O ticket：固定 read chunk、单行硬上限、queue entry/byte 上限、count/time/oldest-age drain、drop/truncate diagnostics，以及 cancel/terminal cleanup 语义。
- Reader worker 线程必须计入 Runtime11 I/O budget；同一 process output session 不能以 detached/private thread 绕过 admission、shutdown 或 observability。
- 对 `1/1K/1M` lines、`64B/1MiB/1GiB` line、`30/120Hz` polls 验证 buffered/queued bytes、reader count、oldest age、drop/truncate、format work、p95 与 RSS 都有明确上界；stop/drop 不 hang，stdout/stderr identity、UTF-8 lossy/error 与 terminal residual output 保持。
- Editor14 用该 contract 替换私有 reader lifecycle 后，重新运行原 Play output gate和独立 review；其 `failure-2026-07-22-play-process-output-byte-budget.md` 才可返回 fixed。

## 禁止临时方案

- 禁止仅增加 entry cap、行数 cap 或每 poll line cap 宣称内存/线程有界。
- 禁止为每次 Play 保留未记账的 raw `std::thread::spawn` reader，或在 active/controller lock 内 wait、join、serialize、fsync 或递归删除 snapshot 目录。
- 禁止在 Editor14 复制 Runtime11 的 streaming owner、别名/compatibility shim 或 test-only bypass。

## 修复结果与回传

Open state: `Runtime11 bounded-stream owner 源码与行为契约已实现；托管 Cargo、Editor14 迁移及产品性能/功耗验收仍保持 open`。

2026-08-26 Runtime11 新增 `tasks/bounded_stream_io/`：一次 capture 的全部 reader 在读取前原子预留，reader 上限同时受配置值和物理 Runtime `Io` worker 数限制；固定 chunk、line、queue entry/byte 与 drain count/byte/time 预算形成 `O(total input bytes)` decode 和 `O(1)` enqueue/drain-owner 路径。stdout/stderr identity、UTF-8 lossy、truncate/drop、oldest age、read failure、cancel/terminal 与 blocked-reader scope census 均有 typed contract。worker 使用 `ExecutionScope` 的 `FinishOnShutdown`，不会私建或 detach reader thread；宿主仍必须先终止子进程并关闭 pipe，Runtime 才能 drain residual tail 并证明 reader 退休。架构和待测矩阵见 `docs/plans/optimize/zircon_runtime/11/2026-08-26-bounded-stream-io-owner.md`。

该结果没有关闭 failure：当前 Editor14 仍使用自身 reader lifecycle，Runtime focused Cargo 尚未获得托管执行结果，`1/1K/1M` lines、极端 unterminated line、并发 session、P95/RSS/power 和共享 `Io` pool 饥饿风险也尚未完成产品数据验收。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-08-10 | `open / cross-plan-handoff-recorded` | 由 Editor14 PERF-MVP-552 失败前向路由至 Runtime11；记录局部止损不可替代共享 blocking-I/O owner。 | 来源 `docs/plans/zircon_editor/editor/14/failure-2026-07-22-play-process-output-byte-budget.md`；未声明任何验证通过。 |
| 2026-08-26 | `open / runtime-owner-source-implemented-focused-static-passed` | 实现 folder-backed bounded-stream lane、atomic multi-reader admission、fixed decode/queue/drain budgets、typed diagnostics、interrupted-read retry、RAII start abort、scope-accounted lifecycle 与 12 项行为契约；`job_system_boundary` 为 14 owner / 58 behavior anchors，missing/oversized/editor-dependency 均为空，聚焦审计 2/2 通过。 | `zircon_runtime/src/core/runtime/tasks/bounded_stream_io/`；`docs/plans/optimize/zircon_runtime/11/2026-08-26-bounded-stream-io-owner.md`；aggregate audit 仍被既有 mesh-builder direct-Rayon 路径阻塞；托管 Cargo、Editor14 migration、product perf/power 均未声明通过。 |
