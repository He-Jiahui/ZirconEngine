---
status: in_progress
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
recorded_at: 2026-07-13
milestone: M1
stage: testing
related_code:
  - zircon_editor/src/core/asset/
  - zircon_editor/src/core/project/
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/
  - zircon_editor/src/tests/editor_asset_type_registry/
  - zircon_editor/src/tests/host/manager/
tests:
  - cargo check -p zircon_editor --lib --locked --jobs 1
  - cargo test -p zircon_editor --lib --no-run --locked --jobs 1
  - cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
---

# Editor09 M1 完整 Windows 验收记录

## 产出记录与时间

| 测试阶段 | 状态 | 更新日期 | 已完成证据与当前阻断 |
|---|---|---|---|
| M1 Windows production + lib-test acceptance | `IN_PROGRESS` | 2026-07-13 | production check 与当前源码 no-run 门自然退出 0；registry 24/24、descriptor 1/1、UI Asset Authoring plugin locked 2/2、ProjectAuthority-backed Manager 83/83。Runtime13 E0308/E0599 consumer hard-cut 已修复回传；第二轮完整门 job `e81ed19d256f40c28ddb2437e9a18460` 成功编译 3157-test binary，并执行到第 1755 项前已观察 130 个跨功能失败，随后在 export-capture test 触发 full-harness 停滞。子进程超过 10 分钟且 capture 文件保持 0 字节后人工终止，job 以 `-1` 释放；同一 binary 的该 exact 后续 1/1、19.65s 自然通过，故 Editor15 已做归属校正，缺失自然 summary 继续归 Runtime11/Editor14。M1 testing 保持进行中。 |

### 已完成的下层门禁

- `cargo check -p zircon_editor --lib --locked --jobs 1`：Windows 受管 job
  `2da44310d00e4ca39b24a163ee7a48d2`，退出码 0。
- `cargo test -p zircon_editor --lib --no-run --locked --jobs 1`：Windows 受管 job
  `c5f0129d36d8445c94820be243d70357`，自然退出码 0。
- M1.4 registry/dispatch 24 passed、0 failed；descriptor write-when 1 passed、0 failed；renderable
  template open + scan/import 1 passed、0 failed。
- 当前源码 Manager suite：`83 passed / 0 failed / 3073 filtered out`，日志
  `.codex/tmp/editor09-project-authority-manager-suite-shared-current-20260713.log`。
- Runtime13 generational HostRegistry consumer 已按 typed `Result`/`resolve` 合同硬切；第二轮完整门成功
  编译且 `editor_manager_registers_minimal_host_capabilities_as_vm_handles_when_script_is_available` 通过，
  fixed 回传为
  [`host-registry-generational-handle-consumer-cutover`](fixed-2026-07-13-host-registry-generational-handle-consumer-cutover.md)。
- 结构优先复核：`core/asset/type_registry/` 最大生产 owner 347 行，M1 新增 production files 均低于
  1000 行硬预算；旧 `AssetEditorDescriptor` 生产符号、`EditorAssetMetaDocument` 生产符号与
  `*.editor.meta.toml` 生产读取均为 0。retired UI asset audit unit tests 2/2 通过，日志
  `.codex/tmp/editor09-retired-ui-asset-audit-unit-20260713.log`。

### 当前完整门与失败归属

- 完整命令：`cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1`。
- 第一轮受管 job `5b3fce449e73469a857795e6027ef9f3` 在编译期自然退出 101；该 Runtime13
  consumer failure 已修复回传。
- 第二轮受管 job `e81ed19d256f40c28ddb2437e9a18460`：当前源码编译 32m47s 后开始串行执行
  3157 tests；日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。在第 1755 项前记录 130
  个失败名，但这只是停滞前的部分集合，不能作为最终 failed count。
- 停滞点：Editor15
  `ui::host::export_cargo_process::tests::cargo_capture_and_poll_complete_on_a_single_runtime_worker`。
  Windows `cmd /C` 子进程超过 10 分钟仍运行，两个 capture 文件持续 0 字节；前一轮独立完整门也停在
  同一测试。确认重复停滞后终止该测试进程树，受管 job 记录 `exit_code=-1` 并于 23:41:56 释放。
- 同一 current binary 的该 fully-qualified exact 随后 `1 passed / 0 failed / 3156 filtered out`，19.65s
  自然退出；独立 Windows 5000+5000 双流命令 14.8s 通过。故 Editor15 独立功能归属已校正并
  [`fixed 回传`](fixed-2026-07-13-export-cargo-single-worker-windows-output-hang.md)，未修改生产 capture 路径。
- 缺失完整自然 summary 继续归既有
  [`Runtime11 full-harness task budget/lifecycle`](../../../zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md)
  与 [`Editor14 full-gate resource closeout`](../08/fixed-2026-07-14-editor-full-gate-thread-exhaustion.md)。
- 停滞前跨功能失败已按既有 owner 去重补记：EditorUI05/06/08、Editor10、Editor12；新增交接为
  [`Editor07 动画索引夹具`](../07/failure-2026-07-13-animation-asset-open-index-fixture-cutover.md)、
  [`Editor03 editing structure guard`](fixed-2026-07-14-editing-operation-owner-structure-guard-drift.md) 与
  [`Render01 viewport resolver guard`](fixed-2026-07-14-editor-viewport-resolve-job-guard-drift.md)。

### 结构与旧架构复核交接

- `.zui` V2 authoring 仍经旧 `UiAssetDocument`/`UiAssetKind` 双向 projection 的生产事实，已补入既有
  [`EditorUI05 UI Asset V2 projection failure`](../../editor_ui/05/failure-2026-07-11-ui-asset-v2-projection-drift.md)，
  明确要求删除 `*_legacy_projection_*` 与旧 kind 映射；不在 Editor09 建第二条重复 failure。
- 聚合 plugin structure report fixture 因 descriptor 单源字段未同步产生 4 个 `KeyError`，已交接
  [`Plugins12`](fixed-2026-07-15-plugin-structure-audit-report-fixture-drift.md)。
- extension registry finalize coverage guard 已跟随当前 plan/apply 边界修复并返回本计划：
  [`fixed`](fixed-2026-07-15-extension-registry-finalize-coverage-guard-drift.md)。
- 两条结构守卫的失败日志：`.codex/tmp/editor09-m1-structure-static-tests-20260713.log`。这些失败不由
  Editor09 资产代码兜底，也不影响 retired UI asset audit 自身 2/2 证据；在 owner 回传前不宣称聚合
  structure gate 通过。

### 未完成门禁

Editor09 M1 尚未取得完整 lib-test 的自然测试 summary，因此 testing stage 与计划状态继续
`in_progress`；不得用已有二进制、filtered suite 或 compile-only 证据替代最终完整门，也不触发
session milestone closeout。Editor15 停滞与各功能 owner 失败修复回传后，必须从原完整命令重跑，
不能从第 1755 项续算或把人工终止当成 harness summary。
