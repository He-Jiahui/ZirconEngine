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
  - zircon_editor/src/core/extension/store/model.rs
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

Open state: `待 Editor08 将实时命令/operation routing 收编为 ticket-owned contribution
lifecycle；Editor06 的 Store snapshot contract 保持集成，相关 hot-disable gate 不得宣称通过。`

## 产出记录与时间

- 2026-08-01：状态 `open_handoff_recorded`。已证明 Store revoke 与 live command routing
  存在双重事实源；failure 已按最低共享层路由 Editor08，要求前向修复，不回滚已集成的
  Editor06 contribution snapshot。
