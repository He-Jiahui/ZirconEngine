---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: level-manager-name-core-error-import-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/scene/module/core_error.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
resolved_at: 2026-07-14
---


# Frameworks05：LevelManager name 的 core-error import 未随硬切收敛

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段 / Runtime02 exact core-min 上行门禁
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：失败位于 Frameworks05 正在迁移的 manager-service/LevelManager consumer 边界，不属于 Editor02 inspection 或 Runtime02 `SystemStage`。

## 失败现象与复现证据

协调器管理的 Windows job `53d15ad02cac432e8f6d4b5b6b9384b9` 执行：

```text
cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
```

Cargo `exit 101`，`scene/module/core_error.rs:3` 报 `E0432`：从 `super::LEVEL_MANAGER_NAME` 导入，但 `scene::module` 已按硬切收缩，不再拥有该常量。

门禁结束后工作区已出现直接从 `crate::core::manager::LEVEL_MANAGER_NAME` 导入的修正，说明实现修复正在 Frameworks05 活动 Session 中收敛；本 Failure 在 fresh exact gate 验证前保持 open。

## 最低共享层根因

最低已证明边界是 Frameworks05 LevelManager service identity owner 迁移后，一个错误构造 helper 仍依赖旧 module-local visibility。正确修复是从当前 manager identity owner 直接消费，不应把常量重新导回 `scene::module`。

## 架构修复验收

- `scene/module/core_error.rs` 直接使用当前唯一 `core::manager::LEVEL_MANAGER_NAME` owner。
- `scene::module` 不恢复兼容 re-export、重复常量或旧 facade。
- 原 exact core-min 上行命令越过该 `E0432` 并进入测试执行。

## 禁止临时方案

- 不在 `scene::module` 添加 `pub use LEVEL_MANAGER_NAME` 或复制常量。
- 不通过字符串字面量、test-only bypass 或弱化错误信息绕过 service identity。
- 不把 Frameworks05 consumer 漂移归到 Editor02/Runtime02。

## 修复结果与回传

- 根因：LevelManager service identity moved to core::manager, while scene/module/core_error.rs still imported the removed scene::module forwarding visibility.
- 架构修复：Hard-cut the consumer to crate::core::manager::LEVEL_MANAGER_NAME; scene::module keeps no compatibility export, copied constant, or facade.
- 验证：Static ownership guard passes and fresh managed Runtime gates no longer report E0432 at scene/module/core_error.rs; exact core-min execution is independently blocked before test execution by the active Plugins08 host_modules.rs relocation.
- 回传：Frameworks05 repaired the unique owner import without restoring the old scene-module API; return to Editor02 as fixed.
