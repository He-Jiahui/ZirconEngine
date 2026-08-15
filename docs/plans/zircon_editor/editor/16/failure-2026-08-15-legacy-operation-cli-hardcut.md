---
handoff_kind: failure
status: open
created_at: 2026-08-15
summary_slug: legacy-operation-cli-hardcut
origin_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/16
fixing_child_dir: docs/plans/zircon_editor/editor/16
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor/tests/cli_operation.rs
  - zircon_app/src/entry/cli/diagnostic_log_args.rs
  - zircon_app/src/entry/cli/launch_args.rs
tests:
  - cargo test -p zircon_app --lib --locked editor_gui_startup_parser_rejects_retired_operation_control_flags
  - cargo test -p zircon_app --lib --locked launch_args
  - zircon_editor commandlet subprocess fixture for --run only
---

# Editor16: legacy operation CLI must hard-cut to the commandlet entry

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 来源执行切片：M1 `EditorLaunchArgs` 收敛与 M2 Commandlet 框架
- 修复责任计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 交接原因：本轮架构要求不兼容旧路径；`--operation`、`--list-operations`、`--operation-history` 与 `--headless` 是独立 CLI parser 和 bootstrap，不是 `--run` 的可保留别名。

## 失败现象与复现证据

修复前，`zircon_app/src/entry/entry_runner/editor.rs` 在 GUI startup parser 后调用
`EditorCliOperationRequest::parse`，并公开 `--operation`、`--list-operations`、
`--operation-history` 和 `--headless`。这条路径创建 `EditorManager`、
`EditorHostEventController` 和 runtime session 后执行 operation-control；`--run` 则在
`EditorLaunchArgs` 中提前分派到 `core::commandlet`。二者的 parser、帮助、测试和
stdout JSON 合同彼此独立，继续并存会保留双重 headless 启动语义。

2026-08-15 ownership 复核时，`entry_runner/editor.rs` 是归档 session
`editor01-startup-metrics-r5-20260730` 的 mixed blob（current hash
`56a104b995fb325cb59f6005ffcb306000700f5ea7bf8060e1a9939f2e4a7ee5`），其中含非
Editor16 的 startup metrics/first-frame 语义。不得在未完成归属转交前整文件 transfer 以删除旧 CLI，更不得
重写或丢弃这些外部变更。

## 最低共享层根因

旧 operation-control CLI 在 command registry 合一之前直接承担脚本入口；当前
`--run` 已有 canonical registry projection，却没有完成 entry runner 的单一 dispatch
owner 切换。把 `--operation` 映射到 `--run` 会延续两种 payload/exit-code/history 模型，
不符合硬切和单一 commandlet JSON 合同。

## 架构修复验收

- 以可复核的 source-owner rotation 或文件级拆分接收 `entry_runner/editor.rs`，完整保留并
  分别归属其中的 startup metrics/first-frame 语义；不得以 whole-file overwrite 达成删除。
- 删除 `EditorCliOperationRequest`、`run_editor_operation` 及其 CLI-only parser、帮助、
  单测与文档入口；`EditorOperationControl*` DTO 仍可作为非 CLI typed control boundary，
  但不再接受旧命令行语法。
- `EditorLaunchArgs` 成为唯一 CLI 路由 owner：诊断组后只允许 GUI startup、`--run` commandlet
  或 hub protocol v1 参数组。未知旧 operation flag 必须走既有参数错误合同，而不是 silent
  fallback。
- commandlet 继续使用固定 JSON envelope 和 0/1/2/3 exit code；删除旧路径不得改变
  `--run` 输出或引入第二个 history/log sink。
- 更新所有 user-facing CLI 文档和 architecture plan，删除 `--headless --operation` 示例，
  不保留 deprecated alias、compatibility shim 或双写入口。

## 禁止临时方案

- 禁止将 `--operation` 翻译为 `--run`、将 operation history 伪装为 commandlet，或同时保留
  两种 parser。
- 禁止为了接收 archived mixed blob 而覆盖 startup metrics、first-frame capture、diagnostic
  log 或其他外部 owner 语义。
- 禁止通过 global logger、第二 command registry 或 retained-host UI import 让旧 CLI 看似可用。

## 修复结果与回传

Open / source repair complete / managed validation pending。已通过 ownership transfer-apply
`37acd88f9ac846379d8e2322e06ef0d4` 接收 `entry_runner/editor.rs`，完整保留 startup
metrics 与 first-frame 语义后删除 `EditorCliOperationRequest`、`run_editor_operation`、旧帮助
和专属测试；`EditorGuiStartupRequestArgs` 对旧 flag 走参数错误合同，`--run` 保持唯一无头
入口。已运行 scoped `rustfmt --check`、`git diff --check` 和残留符号扫描；未运行 Cargo、
子进程或产品验证，故本记录保持 open。

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-08-15 | legacy operation CLI hard-cut failure registered | `open / source-owner rotation required` | 复核到 `--operation`、`--list-operations`、`--operation-history`、`--headless` 的独立 parser/bootstrap/test/doc 路径；`entry_runner/editor.rs` 为 archived mixed blob，须先完成安全 rotation 或文件级拆分，再删除旧 CLI 并运行 focused Cargo 与 commandlet subprocess fixture。 |
| 2026-08-15 | source-owner rotation and legacy CLI deletion | `source repair complete / managed validation pending` | transfer-apply `37acd88f9ac846379d8e2322e06ef0d4` 后删除 parser、bootstrap、帮助及 `cli_operation.rs`；新增旧 flag 拒绝测试，并将日志样例和活跃计划文档切到 `--run <commandlet>`。scoped rustfmt/diff/residual-symbol checks 通过；共享 validation window 未分配给本会话，未启动 Cargo。 |
