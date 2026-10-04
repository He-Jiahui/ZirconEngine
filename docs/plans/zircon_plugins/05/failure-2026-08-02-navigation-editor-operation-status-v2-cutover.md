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

Open state: `source_v2_cutover_static_confirmed / managed_navigation_and_Runtime10_validation_pending`; no dynamic pass is claimed. 原始三处 V1 复现证据保留在上方，现行源码已在其后迁移。

### 2026-09-24 现行调用链核对

- `tests.rs` 的 gateway mock 返回 `ZrRuntimeOperationStatusV2`；`tests/operation_command.rs` 的 `RecordingGateway::poll_operation` 用 V2 `new` 创建状态，保留 foreign handle 与错误 ABI 的失败路径。当前导航 editor 目录运行本记录原 `git grep -n -E 'ZrRuntimeOperationProgressV1|ZrRuntimePollOperationFnV1'` 命令无匹配（退出码 1 表示无命中）；没有恢复 V1 类型或兼容 table slot。
- 直接生产调用链 `operation_command/command.rs` 用 `progress.phase()` 检查终态；共享 editor gateway `session/protocol.rs` 同时用 `status.phase()` 与 `status.detail_kind()` 拒绝未知 V2 字段。本项未修改这两个其他 owner 文件，且源码存在和静态 grep 不能替代运行 mock/命令测试。
- 初始 snapshot `3745` 中 `tests.rs` SHA-256 为 `55967f48b4dfe610ca156444d21d6d3a7cff30bff1b2897d8a1255eff2733d34`，`tests/operation_command.rs` 为 `f4598a34cf7d299483aab6a5860d27a2403e30fef1372e06d4c00760bc37b5fa`；两条路径从已归档原 owner 精确转交。复审发现的后续测试修补见下节。
- 仍须在匹配现行源码的受管验证中实际运行 Navigation editor package gate，再执行 Runtime10 status hard-cut consumer/upward audit；没有这些动态回执或 canonical return，故 failure 保持 `open`。初始快照的审查缺口与修补后复审见下节。

### 2026-09-24 独立审查缺口与下层测试修补

- 初始 snapshot `3745` 的独立只读审查为 `Critical=0 / Important=1 / Moderate=1`：失败 harvest result 的 mock 错误地返回 `Completed/None`，违背 Runtime operation service 的 `Failed/OwnerApplyFailed` 状态合同；foreign handle 和 terminal-failure 测试只断言 `Applied`，未断言真正错误链。原始 V1 编译失败证据和零动态通过状态均仍准确。
- 先新增下层 mock 回归，要求失败 poll 的 `status.phase()` 为 `Failed`、`status.detail_kind()` 为 `OwnerApplyFailed`，且 harvest 保留失败原因；编辑前源码级 RED 确认旧 mock 永远返回 `Completed/None`。再把 mock 失败分支对齐 runtime service 已有的 V2 状态，成功分支保留 `Completed/None`；上层路线验证成功状态的两个 V2 helper，两个失败路径各自明确验证 foreign handle 与 `runtime rejected the generated bake` 错误链。
- 仅 `tests/operation_command.rs` 有本轮源码编辑，现行 SHA-256 为 `992a3c27a8169b2c6e52b1417d38b2ccbde34eddfc353bf6532058cb85a28bcd`；`tests.rs` 仍为上述原 hash。源码级 GREEN 确认 phase/detail 分支及断言存在；Windows `rustfmt +1.94.1 --check --edition 2021`（两文件）、scoped `git diff --check` 与 825 份 handoff 结构检查均通过。这些都不是 Cargo 动态通过，完整 package/Runtime10 向上门禁仍待完成。
- Snapshot `3746` 的修补后独立只读复审为 `Critical=0 / Important=0 / Moderate=0`，确认上轮 I1/M1 均消除。未知 V2 detail 的 transport 拒绝属共享 gateway owner：`session/protocol.rs` 用 `detail_kind()` 判定，其 `tests/gateway/session/plugin_operations.rs` 有独立负向测试；无需在 Plugins05 mock 内复制协议校验。该外部 gateway 测试当前也没有可复用的受管动态回执，故仍只保留静态审查证据。
