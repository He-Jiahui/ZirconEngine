---
handoff_kind: fixed
status: fixed
created_at: 2026-08-17
summary_slug: managed-validation-asset-creation-include-path
origin_plan: docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
fixing_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/12
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus/asset_creation.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
tests:
  - cargo test -p zircon_editor --locked --test runtime_foreign_output_policy
resolved_at: 2026-08-17
---


# Editor UI12: managed validation asset-creation include path

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md`
- 来源执行切片：M2.4 host-output policy convergence 托管验证票据 `929673e5eec04b0791ccce5ae70e863f`
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 交接原因：最低层原因位于 UI12 已登记的 asset-creation menu 测试 owner；runtime-interface 切片未修改该文件，也不应通过复制资源或放宽闭包规划绕过它。

## 失败现象与复现证据

协调器接受了 45 文件精确源码清单，随后在 Cargo 执行前的 `closure_planning` 阶段终止。终端证据为 `validation_copy_compile_time_resource_missing`：引用源 `zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus/asset_creation.rs` 指向不存在的 `zircon_editor/src/tests/ui/host/editor_extension_registration.rs`。

失败命令为 `cargo test -p zircon_editor --locked --test runtime_foreign_output_policy`。工作树检查确认真实资源位于 `zircon_editor/src/ui/host/editor_extension_registration.rs`；同 crate 较浅目录中的既有测试使用可解析到该真实 owner 的相对路径。

## 最低共享层根因

`asset_creation.rs` 移入更深的 `workbench_window_menus` 目录后，`include_str!("../../../../ui/host/editor_extension_registration.rs")` 少回退一级。Rust integration target 本身不会编译 `#[cfg(test)]` 模块，但协调器的 Cargo 闭包规划会扫描 crate 的编译时资源引用，因此该失效路径会阻断任何需要 materialize `zircon_editor` 完整源码闭包的托管验证。

## 架构修复验收

- asset-creation source contract 必须读取唯一真实 owner `zircon_editor/src/ui/host/editor_extension_registration.rs`，不得复制一份测试资源。
- Cargo 闭包规划不得再报告 `validation_copy_compile_time_resource_missing`。
- 原始 `zircon_editor --test runtime_foreign_output_policy` 托管验证必须进入 Cargo 并通过。
- runtime-interface M2.4 的 Editor 上行验证可恢复，不把 host-only 替代票据冒充 Editor gate。

## 禁止临时方案

- 不得在 `zircon_editor/src/tests/ui/host` 创建别名、复制文件或兼容 shim。
- 不得放宽编译时资源闭包检查、移除测试或降低上行验收标准。
- 不得把该 source-contract 测试改成静默跳过缺失 owner。

## 修复结果与回传

- 根因：The deeply nested asset_creation.rs source contract retained a relative include path with one too few parent traversals after the test was moved.
- 架构修复：Point the source contract directly at the single production owner zircon_editor/src/ui/host/editor_extension_registration.rs; no copied resource, alias, shim, or relaxed closure rule was added.
- 验证：Resolve-Path resolves the corrected include to the production owner; managed ticket 7d2aaa8d545c4e68ab8ce2bf9dde355f passed compile-time resource closure planning and entered Cargo. The focused lib-test compile then exposed 243 unrelated existing Editor lib-test errors; the origin slice retains an independent non-cfg-test integration gate.
- 回传：Runtime Interface 01 managed materialization may resume; the missing compile-time resource blocker is removed while unrelated Editor lib-test compilation remains separately open.
