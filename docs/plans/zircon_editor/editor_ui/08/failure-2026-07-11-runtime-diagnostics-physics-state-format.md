---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: runtime-diagnostics-physics-state-format
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/tests/host/pane_presentation/first_wave_payloads.rs
  - zircon_editor/src/tests/host/pane_presentation/support.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_backend.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked tests::host::pane_presentation::first_wave_payloads::pane_payload_builders_emit_stable_body_metadata_for_first_wave_views -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked ui::layouts::windows::workbench_host_window::pane_payload_builders::runtime_diagnostics::tests::physics_state_display_normalizes_dynamic_backend_text_without_debug_quotes -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked ui::layouts::windows::workbench_host_window::pane_payload_builders::runtime_diagnostics::tests::physics_status_projects_human_readable_dynamic_backend_state -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked tests::host::pane_presentation -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked --no-run --message-format short --color never
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

### 2026-09-19 rolling source-contract snapshot (EditorUI08 owner retry)

- Retry Session `failure-roll-01a084c8-editorui08-physics-state-r2` transferred only this failure record, the Workbench `runtime_diagnostics.rs` formatter, and the hard-cut `first_wave_payloads.rs` test owner.
- Current source still trims dynamic physics state and normalizes ready/disabled/unavailable/unknown/blank values, while the exact pane assertion remains `Physics: jolt (Ready, 120 Hz)`. The current recursive `rustfmt --check` reports formatting diffs in these shared files (recorded as non-green); source hashes are `61092bddc7429f428e1378a7001439fc366a1f6766a0f97cbd80a7478c92a1b2` and `ee0f2fb8b8c866a6df34f87bc6feac907480cb0a2650d71ffdb341f1df0b11ae`.
- The 2026-09-11 focused Editor Cargo admission was rejected before ticket creation by `validation_ticket_external_worktree_dirty`; no dynamic result is reused. A fresh exact pane test, group rerun, and upward Editor M1 gate remain required.
- State remains `source_repair_recorded / static_source_contract / managed_validation_pending`; no fixed/return or closeout is justified until current-source behavior and independent Critical/Important/Moderate review complete.

## 2026-09-11 滚动修复记录

- 当前源码复核确认 `physics_state_display(...)` 已在唯一 Workbench formatter 对动态 `String` 做 trim/首字母规范化，并保留 ready、disabled、unavailable、unknown、空白回归；`rustfmt --edition 2024 --check` 与目标文件 `git diff --check` 通过。
- 精确快照 3414（本 failure 文档、`runtime_diagnostics.rs`、`first_wave_payloads.rs`）已冻结。提交请求 `editorui08-physics-state-format-20260911-r2` 使用 `cargo +1.94.1 test -p zircon_editor --locked --lib tests::host::pane_presentation::first_wave_payloads::pane_payload_builders_emit_stable_body_metadata_for_first_wave_views -- --exact --test-threads=1 --nocapture`。
- 协调器在 admission 因外部仓库 `E:\Git\zr_vm` dirty 返回 `validation_ticket_external_worktree_dirty`；未创建 ticket、未执行 Cargo、无动态通过证据。当前会话 `failure-roll-01a084c8-editorui08-physics-state-r1` 置为 `waiting_validation`；待外部 owner 提交清洁 revision 后，必须以同一快照重新提交并先取得本 exact 通过，再继续组级和向上验收。

### 2026-09-19 successor static source-contract receipt

Fixing Session `failure-roll-01a084c8-editorui08-physics-state-r2` sealed the
current formatter/test-owner snapshot in ticket
`d0f2a29ebee74e628c1f5f2678107f43`. Coordinator copy job
`f2a02f79eae541d4ada5b0cdca06901b` and run
`d0f2a29ebee74e628c1f5f2678107f43` exited 0 with
`EDITORUI08_PHYSICS_STATE_SOURCE_CONTRACT_PARSE_PASS`. The pre-receipt source
manifest was
`7fa79efb128010a66e3733c00b1ebbf6aa549759a971bcace7af3e1adf815ef8`.

This is parse/static evidence only; it does not reuse the earlier external-
worktree rejection and does not claim the exact pane test, pane-presentation
group, or upward Editor M1 gate. Current-source rustfmt parity, managed
Windows Cargo, external `E:/Git/zr_vm` admission, independent
Critical/Important/Moderate zero-finding review, canonical `failure return`,
and closeout remain pending. The appended receipt changes this document after
the ticket snapshot, so any dynamic successor must reseal the current bytes.

## 2026-09-21 independent current-source review receipt

- Reviewer Session `review-editorui08-physics-state-r2` inspected the full failure record,
  the UI-08 plan, the sole Workbench formatter, and the hard-cut pane presentation test.
  `EDITORUI08_RUSTFMT_PARSE_PASS` and `EDITORUI08_DIFF_CHECK_PASS` passed; strict
  `rustfmt --check` was run and its nonzero formatting diffs on both current shared files
  were intentionally retained as a blocker rather than reported green.
- Current-source probe `EDITORUI08_PHYSICS_STATE_CURRENT_SOURCE_REVIEW_PASS` confirmed the
  formatter trims dynamic backend text, handles ready/disabled/unavailable/custom/blank
  states without Debug quotes, preserves the exact pane assertion, and bounds the detail
  projection with `detail_item_capacity`/`Vec::with_capacity`. Independent C/I/M review is
  `0/0/0`; no foreign source change was absorbed.
- This is static evidence only. A current-source parse-only ticket, focused managed exact
  Cargo test, pane-presentation group, upward Editor M1 gate, product/performance evidence,
  canonical fixed return, and closeout remain pending. The failure remains `open`.

## 2026-09-21 current-source static validation receipt

- Ticket `d0faf9523e3641569c528b7d4f32bb7a` (request
  `editorui08-current-source-20260921-r2`) sealed the three-path current-source
  snapshot with manifest
  `e3fa6edd95fdc807833cf75094336709f75f35306affcaa3f0f6189270f774eb`.
- Managed copy job `8897d37742784124836c4d0048b74f67` and run
  `d0faf9523e3641569c528b7d4f32bb7a` completed with exit code 0 and terminal
  status `passed`. Its stdout markers were
  `EDITORUI08_PHYSICS_STATE_SOURCE_CONTRACT_CURRENT_PASS`,
  `STRICT_RUSTFMT_CHECK_REMAINS_DEFERRED`, and
  `ROOT_FAILURE_REMAINS_OPEN=true`.
- This receipt is parse/static evidence only. It does not claim strict rustfmt
  parity (the reviewer retained the nonzero check), the focused managed exact
  Cargo test, the pane-presentation group, the upward Editor M1 gate, or product
  performance evidence. The external blocker
  `validation_ticket_external_worktree_dirty:E:/Git/zr_vm` remains active;
  canonical fixed return and closeout are not authorized. The failure remains
  `open`.

## 2026-09-25 current-source rolling reconciliation (stable EditorUI08 Session)

- Because the EditorUI08 fixing plan already has an executable retained-window
  Session, coordinator WIP admission correctly rejected a second primary
  Session. The stable Session `failure-roll-01a084c8-editorui08-retained-window-r2`
  therefore retained its lifecycle identity and claimed this independent
  physics-state record plus the formatter and hard-cut test owner paths.
- Current source confirms the lowest owner remains
  `pane_payload_builders/runtime_diagnostics.rs::physics_status(...)`: the
  formatter consumes `RuntimePhysicsBackendDiagnostics.state` as a dynamic
  `String`, trims it, and title-cases the first character through
  `physics_state_display(...)`; it does not use Debug formatting. The exact
  regression assertion remains `Physics: jolt (Ready, 120 Hz)`, and the
  formatter tests cover ready, disabled, unavailable, custom, blank, unknown,
  and unavailable-backend branches. `support.rs` remains the fixture owner;
  `physics_backend.rs` is a read-only Runtime DTO dependency.
- Current SHA-256 manifest (all paths exist):
  `runtime_diagnostics.rs=d2f4a3d94699e4f52c15507ba271b88823536d0335eff352575f3fc7b2007380`,
  `first_wave_payloads.rs=ee0f2fb8b8c866a6df34f87bc6feac907480cb0a2650d71ffdb341f1df0b11ae`,
  `support.rs=809782375580adadc4c62f465beab7a625b05eeb7aae15a0eb8a9603b2179695`,
  `physics_backend.rs=2b98bb95d2d6726c905b87d0c59c4c605a39fa204fd31cb5b0aae91b10282de0`.
  The formatter path is dirty from pre-existing foreign detail-capacity and
  benchmark edits; no source line was changed or absorbed in this rolling
  reconciliation. The three other source paths are clean relative to HEAD.
- Exact-source checks: `git diff --check` reports no whitespace errors on the
  scoped paths (only the repository's LF-to-CRLF warning). Strict
  `rustfmt --edition 2024 --check` remains non-green on pre-existing formatting
  in the formatter and hard-cut test, so it is recorded as a blocker rather
  than reused as passing evidence. No Cargo command was run directly.
- Snapshot, managed Windows exact Cargo test, `pane_presentation` group,
  upward Editor M1 gate, external `E:/Git/zr_vm` admission, and independent
  C/I/M review of this refreshed manifest remain pending. This section is
  static reconciliation only; the failure remains `open` and no fixed return
  or closeout is claimed.

## 2026-09-25 refreshed independent static review

- Reviewer Session `review-editorui08-retained-window-r2` re-read the current
  document and snapshot 3788 after the source-manifest refresh. Critical /
  Important / Moderate findings are `0 / 0 / 0`.
- The current document hash is
  `6216822100426e014006907790039f56345751416af89cd9bb4a927d96dfa212`;
  snapshot 3788's three entries match byte-for-byte. The four frontmatter
  related paths all exist; `support.rs` and `physics_backend.rs` remain
  read-only dependencies, while the formatter and hard-cut test are the
  owned display/test paths. All five current filters resolve: three exact
  functions, the `pane_presentation` group, and the no-run compile gate.
- Review confirms the formatter trims dynamic state, title-cases the first
  character, covers blank/unknown/unavailable branches, and contains no Debug
  formatting; the first-wave `ready` fixture matches
  `Physics: jolt (Ready, 120 Hz)`. This is an independent static receipt only;
  managed Cargo, upward Editor M1, product evidence, canonical return, and
  closeout remain pending, so the failure stays `open`.
