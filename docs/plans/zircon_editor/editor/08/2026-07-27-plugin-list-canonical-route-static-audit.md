---
status: source_complete_static_green_validation_admission_pending
plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
source_failure: docs/plans/zircon_editor/editor/08/failure-2026-07-26-plugin-list-commandlet-registry-projection.md
origin_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
---

# Editor08 Plugin List Canonical Route Static Audit

## 范围与架构决定

Plan08 owns the single command-descriptor source for `plugin-list`. The commandlet parser now
looks up the exact CLI name from `EditorCommandRegistry`; it no longer converts arbitrary
`--run` input into an inferred route. The descriptor owns and validates all six bindings:

- command id: `plugin.catalog.list`
- CLI name: `plugin-list`
- typed route: `commandlet.route.plugin_list`
- typed action: `HeadlessPluginList`
- payload schema: `editor.commandlet.plugin-list`
- capability: `plugin.catalog.read`

`migrate-assets` uses the same descriptor-owned CLI-name contract. Registry registration rejects
missing headless names, duplicate names, duplicate typed routes, malformed names, and headless
metadata on a non-headless action. The runner resolves the registered descriptor before it reads
the route and action, so it has no commandlet-local table, compatibility alias, or unregistered
fallback. `plugin-list` projects `EditorPluginDescriptor::builtin_catalog_projection()` directly
and returns its deterministic package-id ordering.

## 当前验证边界

The source scope is:

- `zircon_editor/src/core/commands/defaults.rs`
- `zircon_editor/src/core/commands/descriptor.rs`
- `zircon_editor/src/core/commands/registry.rs`
- `zircon_editor/src/core/commandlet/runner.rs`
- `zircon_editor/src/core/commandlet/tests.rs`
- `zircon_editor/src/core/editor_plugin.rs`

Static checks are green: `rustfmt --edition 2021 --check` over the six paths and scoped
`git diff --check` both pass. The retired CLI-to-route inference function and literal
commandlet bindings are absent. A focused immutable source snapshot `1116` exists for the
pre-hardening six-path state; it is superseded by the current descriptor-name hardening and is
not used as acceptance evidence.

Current source-bound Cargo validation remains pending Coordinator01 admission repair. The pinned
external descriptor is `E:\Git\zr_vm` at commit
`503fb72163cd20ddf32a38f8a330083712f5d648`, mounted as `zr_vm`, with only
`zr_vm_rust_binding/rust/zr_vm_rust_binding` and
`zr_vm_rust_binding/rust/zr_vm_rust_binding_sys` included. The intended managed command is:

```text
cargo test -p zircon_editor --lib --locked commandlet --jobs 1 --color never -- --test-threads=1
```

No source-copy id, Cargo terminal result, independent review, fixed return, or commit is claimed
until a fresh immutable current-source copy is admitted and completes this command.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-27 | Plan16 M2.2 shared command contract | source_complete_static_green_validation_admission_pending | Added descriptor-owned CLI names and registry lookup; route, action, schema, remote-callability, and capability remain one canonical descriptor. Added focused tests for direct name lookup, missing name, duplicate name, duplicate route, stable JSON, and missing capability exit code `3`. |
| 2026-07-27 | Static source audit | green | Scoped `rustfmt --edition 2021 --check` and `git diff --check` pass. Static scan confirms no `commandlet_route`, `COMMANDLET_ROUTE_PREFIX`, `MIGRATE_ASSETS_COMMANDLET`, or `MIGRATE_ASSETS_OPERATION` implementation remains. |
| 2026-07-27 | Immutable validation admission | pending_coordinator01_repair | Coordinator01 accepted only the pre-hardening snapshot `1116`; current-source snapshot/materialize requests later timed out or returned `command_preflight_timeout` before submission. The final retry is request `357fd60e48024148a5777ab832e60c03` with `submission: not_submitted`; no Cargo process or ambiguous validation copy exists. Evidence is recorded in `docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-27-coordinator-health-preflight-validation-admission-timeout.md`. |
