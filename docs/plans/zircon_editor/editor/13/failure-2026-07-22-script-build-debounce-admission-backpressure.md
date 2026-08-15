---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: script-build-debounce-admission-backpressure
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/13-script-compilation-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/13
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/script_build/orchestrator.rs
  - zircon_editor/src/core/script_build/request.rs
  - zircon_editor/src/core/jobs
tests:
  - continuous watch first-event max-latency fixture
  - command and play duplicate generation single-flight
  - queue entry byte age and cancel shutdown matrix
---

# Editor13：script build滑动debounce饥饿与无界准入

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-557 script-build debounce admission/backpressure audit
- 修复责任计划：`docs/plans/zircon_editor/editor/13-script-compilation-management.md`
- 交接原因：script generation、reason precedence、debounce deadline 与 compile request policy 由 Editor13 所有，Editor14 只提供共享 job admission。

## 失败现象与复现证据

300ms sliding debounce会被持续watch事件无限后推；unique path set原也随storm增长，本轮已在超过20条时切full-rebuild sentinel并清空set，内存收敛。Command/Play仍为每次调用分配三step request并进入无上限VecDeque，等价generation不合并，长build期间可无限积压；snapshot原clone最后失败String，本轮已改Arc共享。

## 最低共享层根因

Editor13 缺少 first-event max latency 与 source-generation single-flight，Command/Play 请求还会把等价 generation 以无界三步骤条目重复压入队列。

## 架构修复验收

Editor13定义first-event max latency、source generation与Command/Play reason precedence；Editor14提供single-flight typed ticket及entry+bytes+oldest-age预算。watch full-rebuild sentinel到期必须排队，后续变化进入下一generation；Play只保留latest resume intent且失败/取消有明确结果。1M storm下resident paths≤20+sentinel、同generation compile≤1、queue内存和max latency硬有界。

## 禁止临时方案

禁止只扩大 FIFO、把 compile 放回 watcher 线程，或以丢弃 Play/Command 完成结果换取表面有界。

## 修复结果与回传

Open：2026-08-11 已完成 Editor13 owner 的 first-event 1000ms max latency、20 路径/64KiB 双预算、单 pending generation、Command/Play typed-id single-flight、`Watch < Command < Play` precedence、full-rebuild expiry 后下一 generation 与显式 `Cancelled` outcome。独立 Rust 行为夹具 6/6（含 1M explicit storm）、Editor13 静态合同 8/8、scoped rustfmt/diff check GREEN。仍待 Editor14 共享 job ticket 的 entry/bytes/oldest-age 与取消接线、产品 caller F4 edit/build/Play trace，以及仓库级未登记 D/E/F artifact gate 解禁后的 current-source managed Cargo；failure 保持 open，回链 PERF-MVP-557。
