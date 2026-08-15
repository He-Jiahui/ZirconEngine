---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-05
summary_slug: cli-diagnostics-editor-log-source-bridge
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/16
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/logging
tests:
  - GUI and headless operation diagnostics reach one canonical editor log sink
  - parse/operation failure stdout-stderr and diagnostics-file boundary
  - repeated commandlet observation does not create a second history
---

# Editor16: CLI diagnostics are outside the canonical EditorLog bridge

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.1 six-source logging aggregation.
- 修复责任计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 交接原因：CLI parse, GUI/headless bootstrap and commandlet output contracts belong to Editor16; Editor17 defines the existing canonical sink but must not create a parallel entry-runner parser.

## 失败现象与复现证据

`entry_runner/editor.rs` has separate diagnostic/startup/operation parsing and stdout JSON output, but it does not route post-bootstrap CLI and commandlet diagnostics into the `EditorContext` log service. The M3.1 source inventory consequently lists CLI as a missing producer.

## 最低共享层根因

Editor16 owns CLI bootstrap and operation identity, while post-bootstrap diagnostics have no route into the already-created editor context and its canonical log service.

## 2026-08-15 架构复核与边界裁决

本记录的原始验收把两种不同的生命周期混为一谈。当前 `run_editor_with_args_exit_code`
在 `EditorLaunchRoute::Commandlet` 分支直接序列化 JSON 并返回，`plugin-list` 与
`migrate-assets` 不创建 `CoreHandle`、`EditorManager` 或窗口。`EditorLogService` 则由
`EditorContextBuilder::build` 创建，并在 `RetainedEditorHost::new` 后通过 message bus 供
交互编辑器消费；它不是早期进程日志的全局替身。

本地 Unreal 参照支持这一分层：`LaunchEngineLoop.cpp` 先初始化 `GLog` 并挂接 stdout，随后
设置 commandlet 模式且跳过 Slate/交互 editor 初始化。Zircon 的等价早期层是既有
`zircon_runtime::diagnostic_log::initialize_process_log_with_config`，不是人为构造一个
`EditorContext`。

因此本 failure 继续为 `open`，但修复边界收敛为：

- context 之前的 parse 失败、`--run plugin-list`、`--run migrate-assets` 和其 JSON
  envelope 只使用现有 process diagnostic sink；不得建立 `LogChannel::Cli`、第二个
  commandlet history 或无窗口的完整 editor context。
- 已经获得 `EditorContext` 的交互 editor 启动、host 生命周期和 retained-host 诊断必须以
  `LogSource::editor()` 投影到该 context 的 `EditorLogService`；该投影应位于 host/composition
  边界，而不是早期 argv parser。
- `authoring-automation` 的 composition 生命周期若需要 UI 可见诊断，必须从其已打开的
  composition 取得同一个 context log service。不能为 report 序列化单独创建 log store。

后续实现必须先获得 `core/logging`、composition 和 host 边界的明确所有权，并分别用
pre-context JSON/process-log 与 post-context EditorLogService 测试验证，不得以“所有
headless 都写 editor UI history”为验收前提。

## 架构修复验收

- Once the editor host/context exists, Editor16 projects CLI startup, operation and commandlet diagnostics through its existing `EditorLogService` as `LogSource::editor()`; `LogChannel` stays the established six-value contract.
- Pre-context parse failures preserve CLI exit-code/stdout/stderr semantics and may use the existing diagnostics-file startup path, but do not pretend to have an `EditorContext` or create a global logger.
- A bounded operation identity/cursor prevents repeated status observation or retry from multiplying records. Headless and GUI consume the same core sink after bootstrap.
- Tests cover GUI bootstrap warning, headless operation failure, pre-context parse failure, diagnostics-file behavior and duplicate/retry prevention.

## 禁止临时方案

- Do not add `LogChannel::Cli`, a second commandlet history, or a direct retained-host sink.
- Do not change JSON result/exit-code contracts merely to make a log record appear, and do not import UI code into early argument parsing.

## 修复结果与回传

Open state: `source_contract_drift_recorded / no_local_rollback / target_validation_pending`. Editor17 did not modify entry parsing or CLI output; this record establishes the correct Editor16 bridge boundary.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Editor16 CLI diagnostics -> Editor17 M3.1 source bridge | `open_handoff_recorded` | Current source shows Editor16-owned diagnostic/operation output with no post-bootstrap canonical log projection. The six-channel model has no CLI channel, so the handoff fixes classification as `Editor` rather than extending the core enum. |
