---
handoff_kind: failure
status: open
created_at: 2026-07-26
summary_slug: plugin-list-commandlet-registry-projection
origin_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
fixing_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_child_dir: docs/plans/zircon_editor/editor/16
fixing_child_dir: docs/plans/zircon_editor/editor/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commandlet/runner.rs
  - zircon_editor/src/core/plugin/projection.rs
tests:
  - cargo test -p zircon_editor --lib --locked commandlet
  - cargo test -p zircon_editor --lib --locked commands
---

# Editor08: plugin-list commandlet registry projection is missing

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 来源执行切片：M2.2 first headless commandlet wiring (`plugin-list`)
- 修复责任计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 交接原因：Plan16 owns the headless host, but Plan08 owns the only legal command
  descriptor and `--run` routing contract. A commandlet must not create a second registration
  table or infer a route from an unregistered CLI string.

## 失败现象与复现证据

`zircon_editor/src/core/commandlet/runner.rs` currently accepts only the literal
`--run migrate-assets` form and resolves only
`asset.migration.migrate_assets`. The canonical
`EditorCommandRegistry::default_workbench()` contains that remote-callable descriptor, but no
`plugin-list` command id, payload schema, required capability set, or typed headless action.

Plan16 M2.2 requires `--run plugin-list` to project the Plugin12 catalog directly. Adding an
`if command == "plugin-list"` branch in the runner would make the commandlet its own registry
and would violate the Plan08 three-entry single-source contract. The existing descriptor only
stores `Emit` or `Operation`; it cannot identify an executable headless route for a newly
registered command.

## 最低共享层根因

The canonical Plan08 command descriptor has no typed commandlet-action projection, and its
default registry has no `plugin-list` registration. Consequently, Plan16 cannot discover and
authorize a plugin-list command through `command(id)` before calling the existing Plugin12
catalog owner.

## 架构修复验收

- Plan08 adds a typed canonical headless-command route to the command descriptor contract and
  registers `plugin-list` exactly once with an operation id, payload schema, remote-callability,
  and explicit capability requirements.
- The registry rejects malformed, duplicate, or headless-callable descriptors that lack a typed
  route; the same descriptor remains the source for CLI, menu/palette visibility, and headless
  eligibility.
- Plan16 dispatches `--run plugin-list` by resolving the canonical descriptor and its typed route,
  then projects the existing Plugin12 catalog without creating a commandlet-local registry.
- Focused command and commandlet tests cover descriptor discovery, missing-capability exit code
  `3`, unknown command exit code `2`, and stable plugin-list JSON output; the existing
  `migrate-assets` commandlet regression remains green.

## 禁止临时方案

- Do not add a `plugin-list` string match, private command table, or special CLI parser branch in
  `core/commandlet` or `zircon_app`.
- Do not duplicate or rebuild the Plugin12 catalog in the commandlet; consume its existing
  catalog projection.
- Do not add compatibility aliases, silent fallback to an unregistered command, test-only
  dispatches, or weaken the remote-callability/capability gates.

## 修复结果与回传

Open state: `canonical descriptor route、plugin-list typed action、capability gate 与 Plugin12 shared
catalog projection 已在 current source 闭合；managed Cargo、独立复核、fixed return 与受管提交尚未完成。`

## 产出记录与时间

- 2026-07-26：Plan16 M2.2 read-only audit established that only
  `asset.migration.migrate_assets` is registered for remote headless execution. The missing
  `plugin-list` descriptor and typed route are handed to Plan08; Plan16 must not add a second
  registry while this record remains open.
- 2026-08-29：状态 `source-complete / static-contract-reviewed / managed-validation-pending`。
  current source 已在唯一 `EditorCommandRegistry::default_workbench` 中注册 `plugin.catalog.list`，
  descriptor 同时持有 `HeadlessPluginList` typed action、`commandlet.route.plugin_list` route、
  `plugin-list` CLI name、`editor.commandlet.plugin-list` schema、remote-callable 与
  `plugin.catalog.read` capability。parser 只经 canonical name lookup 生成 immutable request token，
  runner 不含 commandlet-local string dispatch table，并直接复用 `EditorPluginManager::builtin_shared`
  的共享 `Arc<EditorPluginCatalogProjection>`。现有聚焦回归覆盖唯一注册、unknown=2、missing
  capability=3、稳定 JSON、package-id 排序和共享投影指针复用；本轮未运行 Cargo，因此 handoff
  保持 open，不生成 `fixed-*` 或回传。
