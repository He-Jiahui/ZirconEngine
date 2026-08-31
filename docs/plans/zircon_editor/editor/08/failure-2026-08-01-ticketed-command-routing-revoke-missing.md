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
