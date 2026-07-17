---
related_code:
  - zircon_runtime/src/core/framework/input/event_retention
  - zircon_runtime/src/core/framework/input/input_manager.rs
  - zircon_runtime/src/input/prelude.rs
  - zircon_runtime/src/input/runtime/event_buffer
  - zircon_runtime/src/input/runtime/action_evaluator.rs
  - zircon_runtime/src/input/runtime/action_evaluator/binding_index.rs
  - zircon_runtime/src/input/runtime/action_evaluator/frame_axis_index.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/recording.rs
  - zircon_runtime/src/input/tests/action_mapping.rs
  - zircon_runtime/src/input/tests/input_manager/event_buffer.rs
  - zircon_runtime/src/input/tests/recording_replay.rs
implementation_files:
  - zircon_runtime/src/core/framework/input/event_retention
  - zircon_runtime/src/core/framework/input/input_manager.rs
  - zircon_runtime/src/input/prelude.rs
  - zircon_runtime/src/input/runtime/event_buffer
  - zircon_runtime/src/input/runtime/action_evaluator/binding_index.rs
  - zircon_runtime/src/input/runtime/action_evaluator/frame_axis_index.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_plugins/02/fixed-2026-07-17-input-event-buffer-visibility.md
  - docs/plans/performance/01/fixed-2026-07-17-input-event-growth-and-frequency.md
tests:
  - python -m unittest tools.tests.test_runtime_input_stack_audit
  - input_stack_boundary_audit (runtime/framework/test 18/25/7, behavior anchors 21, risks empty)
  - managed Windows job d064840b0a8f40dcb405bab74b493ba1 / run 78454dffc1744c858bad697721992c7e
  - managed Windows job 586f1f84cf814180a1bc71c48a713a90 / run a101c9a710634fa386a5f50fb7f3b475
doc_type: milestone-detail
---

# Runtime12 M4 Input Event Bounds Current-Source Addendum

Plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md

Milestone: M4

Status: accepted

Files: [".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py", ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py", ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py", "docs/plans/performance/01/fixed-2026-07-17-input-event-growth-and-frequency.md", "docs/plans/zircon_plugins/02/fixed-2026-07-17-input-event-buffer-visibility.md", "docs/plans/zircon_runtime/runtime/12/2026-07-17-input-event-buffer-visibility-return.md", "docs/plans/zircon_runtime/runtime/12/2026-07-17-input-event-growth-and-frequency-return.md", "docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md", "docs/zircon_runtime/input/input_state.md", "tools/tests/test_runtime_input_stack_audit.py", "zircon_runtime/src/core/framework/input/event_retention/mod.rs", "zircon_runtime/src/core/framework/input/event_retention/queue_status.rs", "zircon_runtime/src/core/framework/input/event_retention/recording_config.rs", "zircon_runtime/src/core/framework/input/event_retention/recording_status.rs", "zircon_runtime/src/core/framework/input/input_manager.rs", "zircon_runtime/src/core/framework/input/mod.rs", "zircon_runtime/src/input/mod.rs", "zircon_runtime/src/input/prelude.rs", "zircon_runtime/src/input/runtime/action_evaluator.rs", "zircon_runtime/src/input/runtime/action_evaluator/binding_index.rs", "zircon_runtime/src/input/runtime/action_evaluator/frame_axis_index.rs", "zircon_runtime/src/input/runtime/default_input_manager.rs", "zircon_runtime/src/input/runtime/event_buffer/frame.rs", "zircon_runtime/src/input/runtime/event_buffer/mod.rs", "zircon_runtime/src/input/runtime/event_buffer/recorder.rs", "zircon_runtime/src/input/runtime/input_state.rs", "zircon_runtime/src/input/runtime/mod.rs", "zircon_runtime/src/input/runtime/recording.rs", "zircon_runtime/src/input/tests/action_mapping.rs", "zircon_runtime/src/input/tests/input_manager.rs", "zircon_runtime/src/input/tests/input_manager/event_buffer.rs", "zircon_runtime/src/input/tests/input_manager/frame_state.rs", "zircon_runtime/src/input/tests/input_manager/touch_gamepad.rs", "zircon_runtime/src/input/tests/recording_replay.rs", "zircon_runtime/src/prelude.rs", "zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs", "zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/module_sets.rs"]

Date: 2026-07-17

## Scope delivered

| 范围 | 状态 | 完成证据 |
|---|---|---|
| event-buffer owner/visibility | `fixed` | `FrameEventBuffer` / `InputEventRecorder` 保持最小 `pub(in crate::input::runtime)`，由 `event_buffer` 以 `pub(super)` 暴露给同级 runtime owner；最终 broad 编译 E0365/E0603 为 0。 |
| frame-transient retention | `fixed` | pointer latest-value/raw-delta 只在相邻连续事件间合并，button/touch/其他 edge 保序并阻断合并；`begin_frame` 清理未 drain 的瞬态事件。 |
| optional bounded recording | `fixed` | recording 默认关闭，启用时使用有界容量并报告 discard/completeness；disabled capture 不再误报完整。 |
| action evaluation complexity | `fixed` | action-to-binding index 按配置建立；每帧 axis state/transition 各索引一次，10/100/1000 button 与 axis 回归均验证实际 visit 数。 |
| framework/prelude owner | `fixed` | `InputManager` 契约、event-retention DTO 与 input prelude 均为 folder-backed owner；crate prelude wiring 由结构审计显式验证。 |
| structure/doc mirror | `fixed` | runtime/framework/test 为 18/25/7，21 个行为锚，新增 prelude/manager/axis-index wiring 缺失项均为空，`risks = []`。 |
| independent review | `accepted` | 最终只读复审 Critical 0 / Important 0。 |

## Fresh testing evidence

Source-manifest-bound Windows job `d064840b0a8f40dcb405bab74b493ba1`（run `78454dffc1744c858bad697721992c7e`）执行：

```text
cargo test -p zircon_runtime --lib input::tests:: --locked --jobs 1 -- --nocapture --test-threads=1
```

结果为 `39 passed; 0 failed; 0 ignored; 8202 filtered`，测试执行 0.37s，总构建 16m05s。通过项包含 bounded pointer/event streams、edge ordering、recording discard/completeness、recording replay、input manager frame state、gamepad bridge、action contexts、consumed axis，以及 button/axis 10/100/1000 index scaling。

原失败作业 `54e50eb7fdf649dcb2c69e667ede841c` 的同名 plan-status 精确门在更新后 source manifest 上由 job `586f1f84cf814180a1bc71c48a713a90`（run `a101c9a710634fa386a5f50fb7f3b475`）重新执行，结果为 `1 passed; 0 failed; 8240 filtered`；`Runtime 12 输入契约/runtime/tests` 索引锚点恢复，作业 exit `0`。

Python regression `tools.tests.test_runtime_input_stack_audit` 为 1/1；direct audit 报告 runtime/framework/test `18/25/7`、behavior anchors `21`、unexpected/missing/wiring/risk lists 全空。`git diff --check` 无 whitespace error（仅仓库既有 LF/CRLF 提示）。

## Review

独立最终复审为 Critical 0 / Important 0；复审覆盖 d064 source-manifest hashes、两个 canonical fixed return、M5 状态镜像、最小 event-buffer 可见性，以及 action/event-retention 语义。

## 边界与剩余范围

本记录是 Runtime12 M4 的 current-source addendum，只验收输入事件保留与动作求值频率收束。Runtime10 继续拥有 per-pointer dynamic-session world mutation 频率；`zircon_app` 全包门和 Runtime12 其他历史里程碑不因本次 input owner green 自动提升。Sound02 必须在此 owner 提交后重新运行其 Kira focused/broad 门，不能把 Runtime12 green 冒充为 Sound acceptance。
