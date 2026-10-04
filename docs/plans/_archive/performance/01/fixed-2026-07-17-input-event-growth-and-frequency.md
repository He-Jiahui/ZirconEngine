---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: input-event-growth-and-frequency
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/12
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/action_evaluator.rs
  - zircon_runtime/src/script/vm/gameplay_host/input.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
tests:
  - 1000 Hz pointer input bounded-memory pressure test
  - recording disabled/enabled queue semantics
  - 10/100/1000 binding action-evaluation benchmark
  - gameplay_key_query_reads_the_lightweight_snapshot_for_codes_and_names
resolved_at: 2026-07-17
---


# Runtime12：输入历史无界增长、高频同步和 action 线性扫描

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F2 input 22 个 Rust 文件与 app/script 消费路径静态审查
- 来源证据：`docs/plans/performance/01/2026-07-17-input-static-review.md`
- 修复责任计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 交接原因：帧队列、录制历史、频率合并和 action 索引必须共享同一 Runtime12 输入语义。

## 失败现象与复现证据

`DefaultInputManager::submit_event` 对每个事件取系统时间、复制并追加 `events` 与 `records`；产品路径未发现常规 drain，长会话或 500/1000 Hz pointer 可能持续增长。pointer moved 还会逐事件跨 ABI、解析 manager 并同步 world camera/selection。

`InputActionEvaluator` 每次求值重建集合并线性扫描 action bindings、axis mappings、transitions 与 gamepads。脚本单键查询原先还克隆完整 frame snapshot；该项已改读轻量 `InputSnapshot` 并补回归，等待 Cargo 验证。

## 最低共享层根因

输入 manager 没有把 frame-transient events、optional recording history 和高频连续值分成有界所有权；action mapping 也没有按稳定配置建立可复用查找结构。

## 架构修复验收

- 明确 events 是帧队列还是历史；recording 仅在启用时保存，并给所有历史设置上限、丢弃计数和可观测语义。
- 125/500/1000 Hz 压测中内存有界；按钮/触控 edge 不丢，pointer latest-value/raw-delta 可按帧合并。
- 对 10/100/1000 binding 建 criterion/计数基线，再按证据选择预索引或帧缓存。
- 接收 lightweight gameplay key query，并继续评估无 clone 的直接按钮查询接口。

## 禁止临时方案

- 不得简单丢弃所有 pointer 事件或破坏录制/回放确定性。
- 不得在 app/script 各自做私有缓存，输入帧语义必须由 Runtime12 owner 定义。

## 修复结果与回传

- 根因：Input events and recordings lacked explicit frame ownership, bounded retention, high-frequency coalescing, and stable action-to-binding evaluation indexes.
- 架构修复：Runtime12 now separates frame-transient events from opt-in bounded recording, preserves edge ordering, coalesces only compatible continuous values, and builds stable action and frame-axis indexes.
- 验证：Managed job d064840b0a8f40dcb405bab74b493ba1 passed 39/39 current-source input tests; input_stack audit passed 1/1 with 18/25/7 structure counts, 21 behavior anchors, and no risks; independent review was C0/I0.
- 回传：Returned Runtime12 bounded input retention and action-index repair to Performance01; upstream performance work may resume with its own product gates.
