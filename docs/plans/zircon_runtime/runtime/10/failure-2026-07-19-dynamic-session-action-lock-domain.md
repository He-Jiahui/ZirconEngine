---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: dynamic-session-action-lock-domain
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/registry
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
tests:
  - slow same-session action concurrency
  - destroy quiescence and ordering parity
---

# Runtime10：dynamic session action锁域

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-430 dynamic session action lock-domain audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：dynamic session action 的 admission、ordered owner lane 与 publish 锁域由 Runtime10 所有。

## 失败现象与复现证据

registry mutex已缩短为slot lookup，这是健康项；但`with_session_activity`/`with_session`仍持slot session mutex跨tick、GPU capture/present、profile全量snapshot+JSON、plugin drain+JSON和operation poll。一个慢GPU/插件/导出会阻塞同session输入、控制及销毁推进。

## 最低共享层根因

slot session mutex 跨越 GPU、I/O、JSON 序列化与 plugin 慢操作，admission、执行和结果发布尚未拆成独立阶段。

## 架构修复验收

- 分成短锁admission、bounded ordered owner lane、短锁generation publish；same-session顺序继续确定。
- GPU/I/O/JSON/大复制不在session mutex内；只读结果以Arc snapshot发布。
- 0/10/1000ms慢动作×1/8/64 callers记录wait/hold、queue age/depth和clone-in-lock bytes；销毁quiescence、错误优先级、panic与wake合同保持，回传PERF-MVP-430。

## 禁止临时方案

采用Bevy bounded render channel、Godot CommandQueueMT与UE render command pipe的owner/有界原则。不得仅换`RwLock`、扩大临界区内并行或建立无界线程/队列。

## 修复结果与回传

Open state: `待 Runtime10 建立三阶段session action owner并回传锁域与压力证据`。
