---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: ticketed-command-routing-revoke-missing
origin_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
fixing_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_child_dir: docs/plans/zircon_editor/editor/06
fixing_child_dir: docs/plans/zircon_editor/editor/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/extension/store/model/contribution_store.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/live_actions.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration/ticketed_command_revoke.rs
tests:
  - cargo test -p zircon_editor --lib --locked
  - Editor12 PostWorkbench enable-disable lifecycle matrix
---

# Editor08: ticket-owned command routing must revoke with plugin contributions

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 来源执行切片：M1 `ContributionStore` ticket/revoke integration and Editor12 hot-disable
  lower-layer handoff.
- 修复责任计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 交接原因：`EditorCommandRegistry` is the live operation-routing authority and is owned by
  Editor08. Editor06 owns the immutable contribution snapshot, not a parallel mutable command
  registry.

## 失败现象与复现证据

`ContributionStore::revoke(ticket)` atomically removes the ticket's command descriptors from its
immutable snapshot, but `EditorHostEventController::register_editor_extension_owned` first clones
and mutates `self.commands()` and then publishes the same batch to the store. There is no inverse
owner/ticket operation for that independent `EditorCommandRegistry`. Consequently, a plugin can
be absent from the Store snapshot after revocation while its operations remain routable through
the live command registry.

The source is already integrated and must be repaired forward. Clearing only the Store or hiding
plugin rows is insufficient because command dispatch remains live.

## 最低共享层根因

Command registration does not retain `ContributionTicket` ownership. The command router has a
second mutable truth independent of `ContributionStore`, so it cannot atomically remove exactly
the commands and operation factories owned by a revoked plugin batch.

## 架构修复验收

- Editor08 makes each live command/operation-factory registration ticket-owned, or projects
  dispatch directly from the capability-filtered `ContributionSnapshot`; no duplicate mutable
  routing truth remains.
- Revoking one plugin ticket removes its commands, generated view-open operations, menu bindings,
  asset-write targets, and operation factories while preserving other tickets and old immutable
  readers.
- A failed registration or revoke publishes no partial router state.
- Focused command-routing tests and the Editor12 PostWorkbench enable-disable lifecycle matrix
  prove that a disabled plugin contribution is no longer routable.

## 禁止临时方案

- Do not make the plugin panel hide a disabled row while command dispatch remains enabled.
- Do not add an Editor06-side command cache, owner-id filter at one call site, compatibility
  alias, or test-only revoke path.
- Do not clear all commands when one ticket is revoked or weaken old-generation reader checks.

## 修复结果与回传

Open state: `Editor08 已将实时 command/operation routing 改为由 Store active tickets 单向投影，
并完成 runtime consumer、viewport overlay provider、scene-mode registration/active stack 的 typed
ticket teardown。剩余架构缺口是 view descriptor/layout/session/document-toolkit 的批量撤销，以及
native contribution provenance + callback lease quiescence；最终受控 Windows Cargo 与 review 也尚无
终态，因此完整 plugin hot-disable gate 继续保持 open。`

## 产出记录与时间

- 2026-08-01：状态 `open_handoff_recorded`。已证明 Store revoke 与 live command routing
  存在双重事实源；failure 已按最低共享层路由 Editor08，要求前向修复，不回滚已集成的
  Editor06 contribution snapshot。
- 2026-08-29：状态 `command-router-source-complete_static-verified_validation-pending`。
  `ContributionStore` 当前 active ticket batches 成为 command router 唯一投影输入；注册与撤销
  在同一 lifecycle gate 内构建私有 Store/command candidates；候选完全通过后才发布，拒绝候选
  不推进双 generation。router publication 从上一个 live generation 严格 `+1`，不再把候选构建
  期间的 descriptor mutation count 当作 revision。新增 2 个可精确过滤的回归，使用独立 asset
  type keys 覆盖双插件 command、generated view-open、factory、asset-write、menu、remaining ticket
  与 builtin preservation，并以唯一 command ID 冲突验证失败零发布。`rustfmt --check`、
  `git diff --check` 通过；静态计数 Store candidate clone `2`、live router clone `0`、gate
  acquisition site `2`。复审后明确撤回不安全的 native unload 自动接线：serialized native editor
  contribution 当前虽为 host-owned，通用注册报告仍可携带可执行 trait object，string owner id
  无法证明其 provenance。旧受控 Cargo 请求早于最终安全性与 generation 修订，不能作为最终
  snapshot 验收；最终 managed Cargo/review 及 view/mode/overlay/runtime-consumer teardown 未完成，
  failure 不关闭且不提交里程碑。
- 2026-08-29：状态 `runtime-object-teardown-source-complete_static-verified_validation-pending`。
  runtime consumer、viewport overlay provider 与 scene mode 均已保留 exact ticket/source；revoke
  先准备 Store/router/scene candidates，再在 runtime consumer lifecycle guard 内退休 active callback，
  随后以不可失败 built-in Select 回退发布实时 scene stack 与 registry。matching overlay/mode 的 exit
  与 Drop 均在 owner-aware 路径执行，trait object 延迟到 shell 解锁后销毁；其他 ticket 的 mode、
  provider 与 enabled state 保持不变。9 个精确源文件 `rustfmt --check` 与 scoped `git diff --check`
  通过；未运行 Cargo。view/layout/session/document-toolkit 与 native unload lease 仍是 open gate，
  failure 不转 fixed、不回传。
- 2026-09-08：状态 `command-contract-test-import-repaired_validation-pending`。
  受管 Windows 接口库验证 `c34098ad9f3740918f1415b061977924` 使用输入
  `interface-library-consumers-3128-20260908`，manifest
  `7b82a7b3c9f8b9d93ce1e184e44c984a5e2e8cae1b1cd72fa7331d555d69706b`，
  剩余 3 个编译错误且实际执行 0 个测试。其中
  `command_schema_v3_roundtrips_a_versioned_execution_contract` 缺失规范
  `EditorCommandExecutionContract` 的测试模块导入。该文件编辑前与 HEAD 一致，
  已通过 transfer `48931bcb4fb24f5f94b807917e97f54a` 接管；前置快照 `3133`，
  修复源码快照 `3134`，SHA-256
  `4d1bf7af9f70beefc105cbb9012dc0a11922245e2655b3d47a754856c9850d6c`。
  仅补齐类型导入，保留 command/3、codec、4096/8192/250 预算及往返断言。
  日志位于受管输入的 `results/interface-library-3128.log`；编译回执不是测试通过。
  其余两个编译错误归属 App07 模板反序列化测试。完整 command routing、Editor12
  enable-disable、view/layout/session/document-toolkit 和 native callback quiescence
  验收仍待完成；本条继续 open，未回传、未提交。
- 2026-09-08：状态 `command-contract-tests-passed_product-gates-pending`。
  后继受管 Windows 作业 `f95f64a6a06345d3940884140d9e3e50` 在
  `interface-library-consumers-3137-20260908`（manifest
  `484b58e5512bb5619941864b4906bbfaaeef0cff78f89267586fe6e4b00d2b63`）
  使用 locked/no-default-features/static 执行完整接口库测试。`3134` 源码被精确叠加，
  7 个 `editor_contribution::tests` 全部实际通过，包括 command/3 execution contract。
  全库为 739 passed、23 failed、101 ignored；原始证据位于该输入的
  `results/interface-library-3137.{json,log}`。本结果确认导入修复，未替代完整
  routing/revoke、Editor12、native quiescence、独立审查或正式 fixing-Session 票据。
- 2026-09-08：独立源码审查返回 Critical 0 / Important 0 / Moderate 0，报告
  `.codex/tmp/interface-app-editor-3137-review-20260908-result.txt`，所选源码与记录哈希
  审查前后匹配且无 reviewer ownership 冲突。实际 3133 -> 3134 的语义差异仅为测试模块
  `use super` 增加 `EditorCommandExecutionContract`；顶部 `EditorCommandId` 导入早已存在。
  报告顶部导入概述以此精确 diff 为准。完整产品验收与正式 closeout 仍未完成。
