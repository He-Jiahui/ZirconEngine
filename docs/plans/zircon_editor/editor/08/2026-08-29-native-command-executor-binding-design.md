---
plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
child_plan: docs/plans/zircon_editor/editor/08
status: lifecycle_source_complete_managed_validation_blocked
date: 2026-08-29
---

# Editor08 Native Command Executor Binding Design

## 目标

为 serialized editor command 建立一条不暴露 DLL function pointer/slot、不借用可撤销 edit factory、并能保留 native library generation 的执行 binding，作为后续 command definition/executor 原子注册的 Runtime 边界。

## 全链复审结论

- Editor `operation_factory` 只负责可撤销 `OperationCommand`、play-edit defer/route 与 transaction history；native callback 是外部 endpoint，不能包装成 no-op undo transaction。
- Runtime native loader 已拥有 command manifest slot table、host-owned bounded output sink、callback duration diagnostics、`NativePluginLibraryGenerationOwner` 与 per-call callback lease。
- Editor native registration 当前只从 `NativePluginLoadReport` 复制 serialized batch；registration report/ContributionBatch 没有 callback binding，load report 离开作用域后也没有 typed executable provenance。
- command manifest v4 只声明 input `payload_schema` 与 `max_output_bytes`，没有 versioned result codec；serialized command/3 现在补上可选 execution contract，但仍没有 surface policy、principal permission 或 endpoint kind。因此当前不能安全地直接打开 materializer execution。

## Runtime foundation 产出

- 新增公开 `NativePluginEditorCommandBinding` 与 typed `NativePluginEditorCommandBindingError`。
- `LoadedNativePlugin::bind_editor_command` 在 admission 时依次验证 editor behavior、`invoke_command` callback 与 command manifest 声明；未声明 command 不生成 binding。
- binding 私有持有 `NativePluginBehaviorSnapshot` 和 `NativePluginLibraryGenerationOwner`；调用只经过既有 manifest slot lookup、host-owned output sink 与 callback lease，Editor 不接触 FFI function pointer 或 slot。
- binding 可克隆以进入 immutable contribution generation；每次 `invoke` 都重新获取 callback lease，lifecycle transition active 时沿既有 typed report fail closed。
- 真实 `editor_contribution_fixture` manifest 新增 slot 0 `editor.contribution_fixture.open`，input schema 为 `zircon.editor.arguments-json/1`，output 上限 4096 bytes；callback 返回 bounded JSON 与成功诊断，不再用空 manifest/恒定 DENIED 伪装 command fixture。

## Editor executor 目标合同

1. `CommandDefinition` 明确 action kind：host event、transactional operation、native/external endpoint；三者不能靠“是否偶然存在 factory”推断。
2. `CommandExecutionRegistration` 与 definition ID 完全匹配，持有 endpoint binding、input/result codec、surface policy、resource budget 和 owner generation。
3. definition、executor、menu/default binding 必须同 `ContributionTicket` 候选提交；任一 resolver/bundle/codec/policy 失败则零发布。
4. dispatch 先做 principal/surface policy、context/capability、payload byte/depth/node 校验，再调用 endpoint；结果与诊断走 typed receipt，默认不把任意 payload写入长期 journal。
5. revoke/unload 先关闭该 owner 新调用，等待 in-flight callback lease，撤销同 generation definition/executor/menu projection，最后释放 library owner；旧 binding 返回 stale/owner-revoked，不可调用已卸载代码。

## 当前阻断与后续

materializer 的 `MissingExecutor` guard 必须保持，直到 Editor execution registration、codec/policy 与 ticket revoke 全部接通。下一切片把 `NativePluginEditorCommandBinding` 放入 command-specific registration map；不能直接把 binding 塞进 `OperationCommandFactoryRegistration`。

## 验证计划

- Runtime unit：manifest declared/undeclared lookup、missing callback/behavior typed binding error、callback lease active/transition race、bounded output。
- Real DLL：构建 editor contribution fixture，bind slot 0，输入 `{}`，验证 status OK、payload `{"opened":true}`、diagnostic 和 callback计数；undeclared ID必须 admission fail。
- Editor E2E：discovery -> atomic materialization -> menu/remote policy -> execute -> typed result -> ticket revoke -> stale invoke deny -> unload；失败批次所有 projection count 均为 0。
- 性能不在本切片宣称。完成后以既有 callback diagnostics 采集 1/64/1024 command owner、p50/p95、active callbacks、mutex acquisitions 和 unload wait，再写入 optimize 报告。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-29 | Runtime generation-owned editor command binding | `source-complete` | 新增公开 binding/error；`LoadedNativePlugin::bind_editor_command` admission 检查 behavior/callback/manifest，binding invoke 复用 generation owner + callback lease + bounded sink。 |
| 2026-08-29 | real native command fixture | `source-complete` | fixture command manifest 从 0 项变为 1 个 dense slot；canonical command ID、arguments-json/1、4096-byte output cap、OK JSON output 与 diagnostic 已落地。 |
| 2026-08-30 | Runtime editor binding admission/lifecycle unit coverage | `source-complete / static-verified / managed-validation-blocked` | `native_live_host/tests/callback_lease.rs` 新增已声明命令绑定并调用、未声明命令准入拒绝、缺失 callback、缺失 editor behavior、lifecycle transition 中 binding 调用 fail-closed 五项测试；只复用现有 generation owner、manifest table 与 callback lease，未创建第二执行注册表。首次受管 Runtime 聚焦验证未进入 Cargo，协调器返回 `cargo_reuse_pool_busy`，兼容 Cargo 池由 job `45a8421d8bbc448ebb792821242243a9` 占用；未将门禁拒绝误报为测试结果。 |
| 2026-08-30 | Versioned result codec and resource budget contract | `source-complete / static-verified / managed-validation-blocked` | `zircon_runtime_interface/src/editor_command_execution.rs` 成为共享合同 owner，`core/commands/execution.rs` 仅 re-export；serialized editor command 硬切到 `/3` 并携带可选合同，SDK builder 与 editor contribution fixture 已同步。`EditorCommandDescriptor` 与唯一 `EditorCommandRegistry` 支持 round-trip 保存合同元数据；registry/materializer 回归确认带合同的 operation descriptor 仍不生成 `operation_factory`，`MissingExecutor` guard 继续生效，未知合同字段与非法预算 fail closed。受管 DTO 聚焦验证未进入 Cargo，协调器返回 `cargo_reuse_target_mismatch`（指定 `E:\cargo-targets\editor08-command-contract-20260830` 与现有兼容键 primary pool 不一致）；未将门禁拒绝误报为测试结果。 |
| 2026-08-30 | Command-specific native executor registry foundation | `source-complete / static-verified / managed-validation-blocked` | `EditorCommandExecutorRegistry` 按 canonical command id 保存 generation-owned native binding、execution contract 与 admission lease；`EditorCommandRegistry` 暴露 register/get/revoke/count，clone/serde projection 不复制运行时 binding。`EditorCommandAction::NativeEndpoint` 成为独立 action kind，native descriptor 无合同、非 native descriptor、未注册命令、重复 executor 或 manifest command-name 不匹配均拒绝注册；invoke 在 revoke、输入超预算和输出超预算时 fail closed，并返回 typed `EditorCommandExecutionReceipt`。新增 action-kind/contract admission、receipt bounded-success/oversized-output 行为测试；尚未接入 ContributionTicket 原子发布、principal/surface policy、真实 dispatcher 或 managed Cargo。2026-08-30 managed 聚焦测试请求在 `cargo.acquire` 响应阶段超时（request `a2bdac1e982f4ee39c7dbac978883fd1`），恢复查询显示无测试进程与无 exit code，故不声明编译或测试通过。 |
| 2026-08-30 | Native binding contribution lifecycle and ticket revoke | `source-complete / static-verified / managed-validation-blocked` | native serialized materializer 在候选 registry 中原子生成 `NativeEndpoint` descriptor 与 `NativePluginEditorCommandBinding`；`EditorPluginRegistrationReport`、`ContributionBatch`、projected command registry 传递同一 binding。注册失败回滚 extension/binding，ticket revoke 重建剩余 active generations 的 command registry，旧 executor 通过 registry drop/revoke 关闭；direct/builtin registrations 保持空 binding。`EditorPluginRegistrationReport` 所有测试构造器已补齐默认 map。Windows managed `zircon_editor` check 在 validator 获取/执行阶段无输出超时，未获得 Cargo 进程或退出码，故不声明编译通过。 |
| 2026-08-30 | Native endpoint dispatcher and manifest metadata convergence | `source-complete / static-verified / managed-validation-blocked` | native binding 携带 manifest `payload_schema` 与 `max_output_bytes`；executor admission 校验 descriptor schema、插件声明和 execution contract 输出上限。`invoke_operation` 对 `NativeEndpoint` 执行 remote/capability gate、JSON/1 payload 编码、typed receipt 解码和 `NativeCommandExecuted` journal，不进入 undo transaction；执行器缺失、拒绝、codec 不支持、结果缺失或解码失败均记录 `ControlFailure`。新增事件执行分支保持 presentation effect。第二次 Windows managed check 在 `cargo.acquire` post-response 超时（无 Cargo 进程/退出码），已释放协调器 job，不声明编译/测试通过。 |
| 2026-08-30 | Native observation result and execution-budget hardening | `source-complete / static-verified / managed-validation-blocked` | native completion is now an observation (`begin_observation`, unchanged revision, no scene-authoring transaction); the returned control response retains the bounded decoded value while the durable event journal drops native arguments and stores an empty typed result, so arbitrary plugin payloads do not become replay state. A zero-output contract decodes a missing payload as JSON `null`; non-zero contracts still fail closed on missing payload, unsupported codec, or malformed JSON. Executor invocation measures callback wall time against the versioned execution budget and converts overruns to bounded typed rejection. Latest managed `zircon_editor` validation request `544dbf1aee2d47ac8adc1721896d00ec` was reconciled as failed before Cargo because coordinator preflight detected unmanaged artifact `D:\ZirconBuilds\mvp-test-fixtures-36724`; no Cargo exit code was available, so compilation/test success is not claimed. |
| 2026-08-29 | managed Cargo/real DLL validation and independent review | `pending` | Runtime binding source 晚于先前受管作业，旧 receipt 不可复用；不声明 test、C/I/M、commit、企微或性能完成。 |
