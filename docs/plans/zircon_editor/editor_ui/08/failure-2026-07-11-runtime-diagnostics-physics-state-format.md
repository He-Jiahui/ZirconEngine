---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: runtime-diagnostics-physics-state-format
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
related_code:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/tests/host/pane_presentation/
  - zircon_runtime/src/core/runtime/diagnostics/physics_backend.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked tests::host::pane_presentation::first_wave_payloads::pane_payload_builders_emit_stable_body_metadata_for_first_wave_views -- --exact --test-threads=1
---

# Editor UI 08：Runtime diagnostics physics state 格式失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 当前源码完整单线程门禁
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：失败位于 Workbench runtime-diagnostics pane payload builder 的显示投影，Plan 01 内核不拥有 pane body metadata 格式。

## 失败现象与复现证据

当前 08:31 源码 Editor binary 的完整门禁仍报告 `pane_payload_builders_emit_stable_body_metadata_for_first_wave_views` 失败；独立 fully-qualified exact 稳定复现为 0/1（0.04s）：

```text
left:  Physics: jolt ("ready", 120 Hz)
right: Physics: jolt (Ready, 120 Hz)
```

`RuntimePhysicsBackendDiagnostics.state` 当前是已经完成动态边界投影的 `String`（夹具值 `ready`），但 `pane_payload_builders/runtime_diagnostics.rs::physics_status(...)` 继续使用 `format!("{:?}", status.state)`；Debug 格式因此把字符串引号写入用户可见 metadata。该故障不是 physics backend 状态机失败，而是 Editor UI 显示层把字符串当枚举 Debug 输出。

## 最低共享层根因

最低共享 owner 是 EditorUI08 的 runtime-diagnostics pane payload formatter：Runtime diagnostics 已提供
动态边界字符串，Workbench 投影仍沿用旧枚举 Debug 呈现。修复应在唯一 pane formatter 统一用户可见
state 文案，而不是把 Runtime DTO 或 Editor kernel 改回旧 enum。

## 架构修复验收

- 在 Workbench runtime-diagnostics 显示投影层定义稳定的人类可读 state 文案规则，并覆盖 ready/disabled/unavailable/未知值；不得把 runtime 动态边界重新耦合到 Editor 私有枚举。
- 先让本 fully-qualified exact 通过，再重新运行 `pane_presentation` 组与 Editor M1 完整单线程门禁。
- 如决定 UI 保持小写 `ready`，应同步产品合同断言与模块文档；无论大小写选择，用户可见字符串都不得包含 Debug 引号。

## 禁止临时方案

- 禁止仅把本夹具改成带引号字符串、删除 physics status 断言或在测试里 trim 引号。
- 禁止恢复旧 diagnostics DTO、增加旧枚举兼容层，或在 Plan 01 内核增加 pane-specific 格式特例。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor UI 08 / Editor M1 | Runtime diagnostics physics state 人类可读投影 | `未通过-待-Editor-UI-08-修复` | 2026-07-11 | 当前完整门禁在 457/2928 报告该失败；独立 fully-qualified exact 为 0 passed / 1 failed（0.03s），`physics_status(...)` 对 `String state = "ready"` 使用 Debug 格式，实际输出 `Physics: jolt ("ready", 120 Hz)`。失败已从 Plan 01 回归入口交接给 Workbench shell/pane payload owner，完整门禁继续收集其余独立信号。 |
| Editor UI 08 / Editor M1 | 当前源码完整门禁复核 | `未通过-显示投影故障未变化` | 2026-07-11 | 08:31 当前源码 binary 完整执行 2930 项为 2763/133/34（2258.13s），133 个失败名相对 06:17 门禁 added=0、removed=0；本 exact 仍为 0/1（0.04s），实际 `Physics: jolt ("ready", 120 Hz)`、期望 `Physics: jolt (Ready, 120 Hz)`。 |
| Editor UI 08 / Editor03+08 M1 | 当前全量门 pane metadata 复现 | `未通过-继续由功能owner处理` | 2026-07-12 | 受管 job `520d85713df249afae31661a7697ad07` 的 `pane_payload_builders_emit_stable_body_metadata_for_first_wave_views` 再次失败；Physics03 的 `sleep_policy` 消费者已越过编译，因此当前信号仍位于 Workbench pane metadata 投影而非刚体字段兼容。原始日志 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`；修复后须先跑本 exact 与 `pane_presentation` 组，再向上复验全量门。 |
| Editor UI 08 / Editor09 M1 | 当前源码完整门停滞前复现 | `未通过-继续由功能owner处理` | 2026-07-13 | 当前源码 job `e81ed19d256f40c28ddb2437e9a18460` 完成编译并在第 1755 项外部停滞前再次记录 `pane_payload_builders_emit_stable_body_metadata_for_first_wave_views` 失败；日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。本 failure 保持 open，不另建重复记录。 |
| Editor UI 08 | physics state display formatter forward repair | `resolving_failure` | 2026-08-13 | 已在唯一 `runtime_diagnostics.rs` formatter 移除动态 `String` 的 Debug 格式化；新增 UI-only 文案标准化与模块内产品回归，覆盖 ready、disabled、unavailable、unknown、空白和 unavailable physics 分支。`rustfmt --edition 2024 --check`、scoped `git diff --check` 与反向源码合同均通过。 |
| Editor UI 08 | pane presentation test-owner hard cut | `resolving_failure` | 2026-08-14 | physics 产品断言随 `tests/host/pane_presentation.rs` 的完整 hard cut 迁入 `first_wave_payloads.rs`；HEAD/current 的 5 项测试均保留，fixture 收敛到 `support.rs`，旧 flat 文件不存在。精确选择器已更新为新叶模块路径，不保留 alias、wrapper 或 `#[path]` mount。`rustfmt --check`、`git diff --check` 与边界扫描通过。 |

## 修复结果与回传

- 状态：`open / 待修复`。
- 修复后更新本文件，并按交接规范移动到来源计划 `docs/plans/zircon_editor/editor/01/fixed-2026-07-11-runtime-diagnostics-physics-state-format.md`；Editor UI 08 仅保留相对回链与已修复摘要。
