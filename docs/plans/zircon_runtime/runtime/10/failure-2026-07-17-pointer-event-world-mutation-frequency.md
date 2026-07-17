---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: pointer-event-world-mutation-frequency
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
tests:
  - pointer event world-mutation count at 125/500/1000 Hz
  - pointer latest-value frame application preserves button and touch edges
---

# Runtime10：pointer 事件仍逐次锁 world 并更新相机

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 拆分来源：Runtime12 `input-event-growth-and-frequency` 修复
- 来源执行切片：Runtime12 输入 retention/action-index 修复后的上层频率复核
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：dynamic session 的 ABI 摄取与逐事件 world mutation 属于 Runtime10 session 生命周期边界。

## 已由 Runtime12 收束的下层问题

Runtime12 已把 manager 事件口定义为帧瞬态队列，连续 cursor 只保留 latest、连续 raw motion 累加；录制默认关闭且显式有界，action map 在配置变更时建立稳定索引。该修复消除默认双历史增长和 `O(A*B)` binding 扫描，但不会自动减少进入 dynamic session 之前的 ABI 调用次数。

## 失败现象与复现证据

`RuntimeDynamicSession::push_event` 对每个 `ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1` 都调用 `submit_input_event(...)`，随后 `handle_cursor_moved(...)` 立即进入 `level.with_world_mut(...)` 调用 camera controller。125/500/1000Hz OS pointer 采样率仍直接决定 manager resolve、mutex 与 world mutation 次数。

## 最低共享层根因

dynamic session 同时承担 ABI 事件摄取与 gameplay preview 相机应用，没有把连续输入的 manager 状态归约和 frame update 应用分开。该 owner 属于 Runtime10 session/lifecycle 边界，不应在 app 建私有 cache，也不应由 Runtime12 manager 越权修改 world。

## 架构修复验收

- ABI 摄取继续把原始事件交给 Runtime12 manager；button/touch/keyboard 等 edge 不得丢失。
- pointer camera/world 应用改为帧边界消费 manager 的 latest cursor / accumulated raw delta，或等价的 Runtime10 session-owned batch；禁止每个连续采样都锁 world。
- 125/500/1000Hz 同一帧压力测试中，world mutation 次数由事件数收束为每帧固定上限，并记录 manager resolve、mutex wait 与 world mutation 计数。
- 点击、拖拽、touch started/moved/ended 与 selection orbit target 的顺序语义保持一致。

## 禁止临时方案

- 不得在 `zircon_app` 或 script 侧增加私有 pointer cache。
- 不得简单丢弃所有 pointer/touch 事件，或绕过 Runtime12 manager 状态机。
- 不得保留逐事件 world 路径作为 fallback/compat shim。

## 当前状态

Open：Runtime12 下层队列/录制/action index 已实现并等待受管验证；本文件只接管剩余的 Runtime10 dynamic-session world mutation 频率。

## 修复结果与回传

Open state：等待 Runtime10 owner 将连续 pointer 的 world 应用收束到帧边界，并提供 125/500/1000Hz world-mutation 计数证据。
