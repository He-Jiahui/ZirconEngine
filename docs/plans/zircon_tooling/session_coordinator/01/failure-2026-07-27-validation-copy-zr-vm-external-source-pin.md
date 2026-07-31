---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: validation-copy-zr-vm-external-source-pin
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_workflow_node: M3
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_editor/Cargo.toml
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/workspace_copy.py
tests:
  - validation-copy materialize-cargo with a pinned zr_vm external source descriptor
  - managed Cargo test runs from the resulting immutable source copy
---

# Editor08: validation-copy 缺少 zr_vm 外部源码固定描述

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：M3 ToolScheduler bus adapter 的 source-bound Rust 验收
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Editor08 能声明固定 sibling Git 输入，但 immutable copy 的 external-source
  描述符校验、挂载和可审计返回值由协调器实现。没有该基础设施契约，Editor08 无法形成可重放的
  Cargo 输入证据。

## 失败现象与复现证据

2026-07-27，Editor08 M2/M3 的 source-bound Cargo 输入副本在未启动 Cargo 前被协调器拒绝：
`validation_copy_external_source_missing`。Cargo path dependency
`E:\Git\zr_vm\zr_vm_rust_binding\rust\zr_vm_rust_binding\Cargo.toml` 位于 sibling Git
仓库，未随 materialization 请求提供固定 external source descriptor，因此无法建立可审计的
不可变输入副本。

当前可解析的 sibling Git HEAD 为 `503fb72163cd20ddf32a38f8a330083712f5d648`；描述符必须以
`mountPath: zr_vm` 挂载，并覆盖以下两个 crate 根：

- `zr_vm_rust_binding/rust/zr_vm_rust_binding`
- `zr_vm_rust_binding/rust/zr_vm_rust_binding_sys`

## 最低共享层根因

`validation-copy materialize-cargo` 的外部源码输入契约要求调用方提供一个不可变 Git descriptor，
但当前拒绝信息只给出了缺失的绝对 manifest 路径。它没有在请求侧验证 descriptor 是否覆盖
Cargo path dependency，也没有把成功 materialization 的 external-source commit、挂载根和 source
manifest 作为后续 CPU reservation 的强制关联输入。因此 Editor08 无法从失败返回直接构造可审计的
重试，且无法证明 Cargo 读取的是固定 sibling 源。

## 架构修复验收

- Coordinator01 接受 `repoRoot`、不可变 `commit`、`mountPath: zr_vm` 与双 crate `includeRoots`
  描述符，并在 materialization 前验证其覆盖所需 path dependency；它只从 Git commit archive 取源，
  不读取 sibling checkout 的 dirty 工作树。
- 成功 materialization 返回 `source_copy_job_id`、输入 manifest hash、source root，以及可回读的
  external-source commit/mount metadata；CPU reservation 与 Cargo run 必须原子绑定这些精确输入。
- Editor08 用上述 descriptor 重新 materialize，并只在冻结副本的 managed Cargo 终态成功后回传
  M2/M3；当前缺 pin 的 pre-Cargo 拒绝不得被误报为 Rust 验收结果。

## 禁止临时方案

- 不得在普通工作树直接运行 Cargo 来绕过 immutable validation copy。
- 不得使用 `HEAD`、未固定分支名或外部仓库的 dirty 文件作为 external source commit。
- 不得因 validation-copy 前置失败将命令面板、SettingsRegistry 或 ToolScheduler failure 标记 fixed。

## 修复结果与回传

Open state: `待 Coordinator01 external-source descriptor 覆盖校验与返回绑定修复`。Editor08 已保留
固定 commit 和所需 crate 根，但尚未获得 materialized source copy、Cargo 终态、fixed return 或提交。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-27 | Editor08 M2/M3 source-bound validation-copy | open | 受管 `validation-copy materialize-cargo` 在 Cargo 启动前返回 `validation_copy_external_source_missing`；缺失 manifest 为 `E:\Git\zr_vm\zr_vm_rust_binding\rust\zr_vm_rust_binding\Cargo.toml`。已实读 sibling Git root `E:\Git\zr_vm` 与固定 commit `503fb72163cd20ddf32a38f8a330083712f5d648`，并将 descriptor 覆盖校验、immutable mount 和 reservation 绑定交接给 Coordinator01。 |
| 2026-07-27 | Editor08 M2 keymap signature-index source-copy retry | open | 受管 snapshot `1122` 已冻结 `key_chord.rs`=`ff3198d1...`, `keymap.rs`=`a29a0e23...`, `keymap/tests.rs`=`e0329e69...`；随后以 `cargo test -p zircon_editor --lib core::commands::keymap::tests --locked --jobs 1 --color never -- --test-threads=1` 请求 `validation-copy materialize-cargo`。协调器在 Cargo 启动前拒绝，原始错误为 `Cargo manifest path dependency has no pinned external source descriptor`；未创建 source copy、Cargo job 或 run，不能作为 Rust 验收。该重现确认同一 Coordinator01 external-source descriptor 覆盖契约仍是最低共享阻断。 |
