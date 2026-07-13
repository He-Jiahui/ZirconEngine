---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: runtime-module-lifecycle-observer-import-cutover
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/frameworks/02
related_code:
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs
tests:
  - cargo test -p zircon_editor --lib --no-run --locked --jobs 1
resolved_at: 2026-07-13
---


# Frameworks 02：Runtime module lifecycle observer 硬切导入路径未原子收口

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1.4 source authority 与只读 command when/dispatch guard 的 Windows 编译门禁
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 交接原因：失败位于正在并发改造的 Runtime module lifecycle observer owner；Frameworks02 明确拥有模块内核与生命周期统一，Editor09 不应在资产管理切片中修改其公开路径或恢复 root facade。

## 失败现象与复现证据

Editor09 focused 行为测试发现并修正测试夹具后，Windows 受管门禁
`cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 在编译 `zircon_runtime` 时新增 5 个 E0432：

- `core/runtime/handle/core_handle.rs`、`handle/runtime_extensions.rs`、`runtime.rs`、
  `state/core_runtime_state.rs` 从 `crate::core` 导入
  `RuntimeModuleLifecycleObserver` 失败；
- `plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs` 从
  `crate::core` 导入 `RuntimeModuleLifecycleBlock` 与 observer 失败；
- rustc 指向当前 canonical `crate::core::runtime::{RuntimeModuleLifecycleBlock,
  RuntimeModuleLifecycleObserver}`。

完整日志：`.codex/tmp/editor09-m1-4-source-authority-compile-r5-20260713.log`。

## 最低共享层根因

最低边界是 Frameworks02 正在把 plugin bridge lifecycle 收编为 runtime module lifecycle observer，调用方已切换，`core/mod.rs` root 导出与下层导入路径处于非原子中间态。该类型已由 `core/runtime/mod.rs` 公开；功能 owner 必须选定并统一 canonical owner，不能依赖文件写入先后偶然通过。

## 架构修复验收

- lifecycle observer/block 的 canonical owner 与所有 Runtime 内部调用方一致；若遵守 core-spine 收口，应直接引用 `crate::core::runtime`，不要求 root facade 重导出。
- 全仓无该两类型的漂移导入路径，且不新增 alias、shim 或重复定义。
- `cargo check -p zircon_runtime --lib --locked` 通过。
- 原始 `cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 越过这 5 个 E0432。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止由 Editor09 恢复 `crate::core` 临时 facade，禁止复制 observer/block 合同。

## 修复结果与回传

- 根因：Runtime module lifecycle observer/block 已由 core::runtime 持有，但五处 Runtime 内部 consumer 仍从 crate::core root facade 导入，形成硬切中间态 E0432。
- 架构修复：Frameworks02 将全部 consumer 直接导向 canonical crate::core::runtime owner，未恢复 core root re-export、alias、shim 或重复合同。
- 验证：Windows 受管 job c5f0129d36d8445c94820be243d70357 执行 cargo test -p zircon_editor --lib --no-run --locked --jobs 1 自然退出 0，完整越过原五处 E0432；日志 .codex/tmp/editor09-m1-4-source-authority-compile-r6-20260713.log。
- 回传：Runtime lifecycle observer import hard cut 已原子收口到 core::runtime，Editor09 原编译门恢复。
