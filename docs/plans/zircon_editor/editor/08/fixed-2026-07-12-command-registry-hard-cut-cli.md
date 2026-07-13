---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: command-registry-hard-cut-cli
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/16
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/commands/registry_handle.rs
  - zircon_editor/src/core/editor_operation.rs
tests:
  - cargo check -p zircon_app --locked
  - cargo test -p zircon_app --locked
  - cargo test -p zircon_editor --lib --locked
resolved_at: 2026-07-12
---


# Editor 16：命令注册表硬切后 CLI 仍引用已删除运行时与操作栈

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Plan08 M1.1 合一命令注册表硬切与 CLI/list/invoke 同源门
- 修复责任计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 交接原因：旧 CLI 启动宿主、参数到控制请求的映射及 commandlet 入口属于 Editor16；Plan08 只提供 Context 所有的唯一共享命令注册表，不能在 App 入口恢复旧 runtime 或兼容栈。

## 失败现象与复现证据

`zircon_app/src/entry/entry_runner/editor.rs` 仍保留三处已经被编辑器架构硬切删除的契约：

- 第 9 行导入 `core::editor_event::EditorEventRuntime`，第 108 行调用 `EditorEventRuntime::new(state, manager)`；当前宿主已经收敛为 Editor host/service 与 `EditorContext` 所有的共享服务，不再提供该聚合运行时。
- 第 283 行把 `--operation-stack` 映射为 `EditorOperationControlRequest::QueryOperationStack`，第 756 行测试继续断言该变体；当前 DTO 只有 `QueryOperationHistory`，旧操作标签栈已经由 Editor03 事务历史硬切删除。
- 第 280 行 `ListOperations` 名称仍是对外传输 DTO，可保留，但其消费必须进入 `EditorContext::commands()` 的唯一 `EditorCommandRegistryHandle`，不得在 CLI 构建第二注册表或恢复 `EditorEventRuntime`。

预期复现命令为 `cargo check -p zircon_app --locked`。当前共享里程碑禁止本切片运行 Cargo；以上旧符号由源树静态扫描精确定位，因此本交接不声明编译门通过。

## 最低共享层根因

Editor16 尚未把 `entry_runner/editor.rs` 从已删除的 `EditorEventRuntime`/`QueryOperationStack` 迁到新版 Editor host/service 入口和唯一 Context command registry。`QueryOperationHistory` 的真实结果依赖 Editor03 edit-command factory 与事务历史，但 CLI 参数解析、请求变体选择、宿主调用和输出合同的唯一接管 owner 是 Editor16。

## 架构修复验收

- `zircon_app` 不再引用 `EditorEventRuntime`、`QueryOperationStack` 或已删除的操作栈类型；启动路径使用当前 Editor host/service owner。
- `--list-operations` 与操作调用复用 `EditorContext::commands()` 的唯一共享 registry handle；App/CLI 不持有第二份 descriptor 图。
- 原 `--operation-stack` 入口硬切为 `QueryOperationHistory` 的诚实响应；Editor03 factory 未安装时透传 `OperationHistoryPendingFactory`，不得制造旧栈兼容结果。
- `cargo check -p zircon_app --locked`、`cargo test -p zircon_app --locked` 通过，并向上回跑 Plan08 的 CLI/list/invoke 同源断言。

## 禁止临时方案

- 禁止恢复 `EditorEventRuntime`、`EditorOperationStack`、`QueryOperationStack` 别名或 re-export。
- 禁止在 `zircon_app` 新建命令注册表、复制 built-in descriptor、静默 fallback 或为 CLI 加专用分派表。
- 禁止把 `QueryOperationHistory` 在 Editor03 factory 未就绪时伪装成成功的标签列表。
- 禁止弱化 CLI 测试或 Plan08 的唯一 registry 验收来隐藏编译失败。

## 修复结果与回传

- 根因：zircon_app editor CLI retained deleted EditorEventRuntime and QueryOperationStack contracts after Editor08 command registry hard cut
- 架构修复：Entry runner now uses EditorHostEventController backed by EditorManager context shared command registry; CLI hard-cuts --operation-stack to --operation-history and QueryOperationHistory with typed pending-factory propagation
- 验证：App stale-symbol scan 0；`cargo +nightly fmt --all -- --check` 通过；`target-editor-host` 下 `editor_cli_operation_` 目标测试 14/14 通过。并发 Runtime Text owner 完成 `RichTable*` 导出后，相同 Windows nightly locked/offline 六 profile 矩阵最终 6/6 通过；带真实 ZR VM MSVC 原生后端的 Runtime `--all-features` 也以 exit 0 通过。
- 回传：已回传 Editor08。CLI host 与 operation-history 硬切完成且无兼容 alias；本 failure 生命周期关闭。Frameworks03 仅保留分支 CI 实际全绿证据 pending。
