---
handoff_kind: failure
status: open
created_at: 2026-08-02
summary_slug: navigation-editor-operation-status-v2-cutover
origin_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_runtime/runtime/10
fixing_child_dir: docs/plans/zircon_plugins/05
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/navigation/editor/src/tests.rs
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
tests:
  - git grep -n -E 'ZrRuntimeOperationProgressV1|ZrRuntimePollOperationFnV1' -- 'zircon_plugins/navigation/editor/**/*.rs'
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -ManifestPath zircon_plugins/navigation/editor/Cargo.toml -Package zircon_plugin_navigation_editor -Locked
---

# Plugins05: Navigation Editor Operation Status V2 Cutover

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 来源执行切片：Runtime10/Runtime11 operation status ABI hard cut and consumer audit.
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：Navigation editor owns the gateway mocks and operation-command tests that still consume the retired status DTO; Runtime10 must not retain a V1 interface alias to satisfy a plugin-local test harness.

## 失败现象与复现证据

Current-source `git grep` finds three retired V1 progress references under the Navigation editor owner:

- `zircon_plugins/navigation/editor/src/tests.rs` implements `EditorRuntimeGateway::poll_operation` with `ZrRuntimeOperationProgressV1`.
- `zircon_plugins/navigation/editor/src/tests/operation_command.rs` imports the retired V1 DTO and returns it from its `RecordingGateway` mock.
- The same mock constructs `ZrRuntimeOperationProgressV1::new(...)` with the former string progress detail.

Runtime10 has removed the V1 poll type/table slot in favor of fixed-layout `ZrRuntimeOperationStatusV2`; these test consumers therefore cannot compile against the current interface and must not cause a compatibility alias or parallel poll API to return.

## 最低共享层根因

Plugins05 Navigation editor's test gateway models the old JSON-shaped progress contract instead of the current fixed-layout operation status. The lowest owner is the plugin's editor test and operation-command boundary, not Runtime10's interface ABI.

## 架构修复验收

- Migrate the Navigation editor gateway mock and all operation-command status assertions to `ZrRuntimeOperationStatusV2`, including phase/detail access through the current V2 helpers.
- Remove all `ZrRuntimeOperationProgressV1` and `ZrRuntimePollOperationFnV1` references from `zircon_plugins/navigation/editor` without aliases, forwarding shims, or fallback table slots.
- Run the focused Navigation editor package gate through managed validation, then rerun Runtime10's status hard-cut source/consumer audit.

## 禁止临时方案

- Do not restore V1 types, a V1 poll function, a compatibility table slot, or a test-only cfg alias.
- Do not weaken the editor operation-command assertions by omitting phase/detail checks.

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
