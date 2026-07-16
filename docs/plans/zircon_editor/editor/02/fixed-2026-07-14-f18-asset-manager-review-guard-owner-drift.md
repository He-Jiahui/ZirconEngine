---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: f18-asset-manager-review-guard-owner-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
related_code:
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/late_api_cleanup/f18_asset_manager_resolution.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/mod.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/handle.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
resolved_at: 2026-07-14
---


# Runtime15：F18 asset-manager 评审守卫仍读取退役 owner

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段 / Runtime02 exact core-min 上行门禁
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：失败发生在 Runtime15 的 `late_api_cleanup/f18_asset_manager_resolution.rs` 评审守卫；Editor02 world-sync 与 Runtime02 `SystemStage` 都不是该测试 owner。

## 失败现象与复现证据

协调器管理的 Windows jobs `8251688ab702482fba2a6f3ef49145bc`、`53d15ad02cac432e8f6d4b5b6b9384b9` 执行：

```text
cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
```

均以 Cargo `exit 101` 停在测试目标编译期。`f18_asset_manager_resolution.rs:4,6` 使用 `include_str!` 读取已经由 manager-service 硬切删除的：

- `asset/pipeline/manager/asset_manager/resolve_asset_manager.rs`
- `asset/pipeline/manager/asset_manager/asset_manager_handle.rs`

当前真实 owner 已收敛为 `asset_manager/handle.rs` 与通用 `core::manager` service resolution，守卫尚未同步。

## 最低共享层根因

最低已证明边界是 Runtime15 F18 review guard 的 owner-path 与形状断言漂移。生产代码正在执行不保留旧 facade 的硬切，测试仍把退役文件当作编译期输入，因此编译在进入 scene 测试前失败。

## 架构修复验收

- F18 守卫读取当前唯一 `asset_manager/handle.rs` 与通用 manager service owner。
- 守卫同时断言两个旧文件不存在，不恢复 compatibility module、旧 free function 或重复 handle 类型。
- F18 对外语义改为验证 typed `ManagerServiceHandle` + `resolve_manager_service` 的当前硬切边界。
- 原 exact core-min 上行命令至少越过本守卫并进入测试执行。

## 禁止临时方案

- 不恢复 `resolve_asset_manager.rs` 或 `asset_manager_handle.rs`。
- 不用空文件、re-export shim、兼容 free function 或放宽/删除整个 F18 守卫掩盖漂移。
- 不把 Runtime15 测试 owner 漂移写成 Editor02 或 Runtime02 功能失败。

## 修复结果与回传

- 根因：Runtime15 F18 review guard still included two manager-specific resolver files deleted by the generic ManagerServiceHandle hard cut.
- 架构修复：The guard now reads asset_manager/handle.rs and core/manager/service.rs, rejects both retired files and legacy Arc holder shape, and verifies asset_manager_handle plus resolve_manager_service at the dynamic project use point.
- 验证：Fresh standalone F18 1/1 and full code-review 80/80 passed; managed Windows job 1d651b687cf647fe8498321d7095c731 ran cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked with 596 passed and 0 failed.
- 回传：F18 owner-path and resolver-shape drift is fixed without a compatibility shim; Editor02 may resume its exact core-min gate.
