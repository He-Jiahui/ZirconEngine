---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dynamic-scene-session-bounded-async-io
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session/io
  - zircon_runtime/src/scene/dynamic_scene/session/path_mutation
  - zircon_runtime/src/scene/dynamic_scene/session/path_api
tests:
  - cargo test -p zircon_runtime --lib dynamic_scene_session --locked --jobs 1 -- --nocapture --test-threads=1
  - slow disk, write storm, cancellation and shutdown fixtures
---

# Runtime11：dynamic scene session有界异步I/O交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：dynamic scene session核心195/563逐Rust文件审查，PERF-MVP-475
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：Runtime11拥有统一job dependency、取消、队列预算与shutdown语义；Runtime04提供不可变archive artifact。
- 生命周期键：`dynamic-scene-session-bounded-async-io`

## 失败现象与复现证据

load/save/path mutation在调用线程执行完整`read_to_string`/parse/pretty String/`fs::write`；atomic save也先驻留完整payload再temp write/rename。每个小mutation重新加载并重写整个archive，没有in-flight bytes/count/time上限、同path合并、取消或shutdown结果合同。

## 最低共享层根因

session path facade直接拥有同步文件系统流程，没有经过Runtime11统一I/O lane，也没有path+generation ticket和bounded publication。

## 架构修复验收

- caller只提交Runtime04 immutable artifact ticket；I/O lane按path+generation single-flight，newer写合并/取消older未发布工作。
- streaming reader/writer避免完整pretty String常驻；temp write后flush/fsync/atomic rename，失败保留last-good且清理temp。
- read/write分别具count/bytes/time预算、backpressure和公平性；发布queue depth/bytes/age/drop/cancel/wait/service latency及RSS诊断。
- shutdown明确选择flush或cancel并可测试；任务failure/panic/cancel经唯一terminal observer回传，不靠poll storm。
- 1/64/512MiB、1/1k write burst和0/10/1000ms slow I/O下caller blocking I/O=0、pending bytes/RSS有界、每path同时发布≤1、stale write publish=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止每次save新建线程或绕过Runtime11线程预算。
- 禁止无界channel、DetachOnDrop fire-and-forget或仅在完成后丢弃stale结果。
- 禁止把serialize或fsync搬到主线程回调阶段。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
