---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: level-manager-export-cutover-incomplete
origin_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/runtime/02
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/foundation/tests.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
resolved_at: 2026-07-14
---


# Frameworks05：LevelManager manager-surface 硬切未闭合

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 来源执行切片：`system-stage-owner-guard-drift` Failure 上行验证
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：失败发生在 Frameworks05 当前 M4 S4 manager handle/resolver 硬切的活动写入范围，且其 Session 正在修改同一 `core/manager` 与 `scene/module` 边界。

## 失败现象与复现证据

在协调器管理的 Windows core-min scene gate 执行：

```text
cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
```

首次编译期失败：`zircon_runtime/src/scene/module/mod.rs:4` 导入 `crate::core::manager::LevelManager`，但当前 `core/manager/mod.rs` 不再导出该类型；真实 trait owner 已位于 `core/framework/scene/mod.rs`，而 `core/manager/resolver.rs` 仍消费该 trait。工作区同时显示这些文件由活动 Session `frameworks05-core-contract-reverse-deps-20260713` 修改。

2026-07-14 09:35 后，活动 owner 已将 `scene/module` 消费方硬切为直接导入 `core::framework::scene::LevelManager`，原 `E0432` 已消失。来源 Session 随即在协调器受管 Windows test lane（job `96ad8a96b3494b93a4d90e6c4057713b`）重跑同一命令；编译继续推进后在 `zircon_runtime/src/foundation/tests.rs:71` 暴露下一处同属 M4 S4 的迁移未闭合：测试对 `Result<Arc<dyn ConfigManager>, CoreError>` 调用 `unwrap_err()`，从而要求成功值 `Arc<dyn ConfigManager>: Debug`，产生 `E0277`。这不是 `ConfigManager` 运行时契约应承担的 Debug 约束，而是 versioned manager handle 测试仍使用了会给成功类型附加 Debug 约束的断言写法。

## 最低共享层根因

最低已证明边界是 Frameworks05 M4 S4 的 manager vocabulary / handle-resolver 迁移尚未闭合：resolver 已指向中立 scene trait，`scene/module` consumer 的导入方向现已收敛，但 versioned handle 的失效代际测试仍通过 `unwrap_err()` 错误地把 `Debug` 约束传播给动态 manager trait。最终测试断言与 handle API 仍由该活动 owner Session定稿；来源 Session不得抢写。

## 架构修复验收

- `LevelManager` 只有一个中立 trait owner，manager handle/resolver 与 scene module consumer 都从定稿的公开边界导入。
- 不在 `core/manager` 恢复兼容 alias，除非该路径就是 Frameworks05 定稿的真实 manager vocabulary owner；不得同时保留两套真相。
- versioned manager handle 的 stale-generation 测试验证真实 `CoreError::StaleServiceHandle`，但不得为了测试便利给 `ConfigManager` 或其他 manager trait 添加 `Debug` 超集约束；使用不要求成功类型实现 `Debug` 的显式匹配。
- `cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked` 至少越过该导入错误。
- Runtime02 随后完成 SystemStage 精确结构守卫及原上行 gate 验收。

## 禁止临时方案

- 不由 Runtime02/Editor02 修改 Frameworks05 活动代码，不添加 call-site 私有补丁、compatibility shim、重复 trait 或 test-only bypass。
- 不放宽上行 scene gate来隐藏未完成的 manager 硬切。

## 修复结果与回传

- 根因：Manager Arc-holder hard cut removed the manager-side LevelManager export while scene registration and the stale-generation test still consumed retired assumptions.
- 架构修复：Kept LevelManager exclusively in core/framework/scene, registered it through RegisteredManagerService<dyn LevelManager>, converted stored manager state to ManagerServiceHandle<T> plus use-point resolution, and rewrote the stale-generation assertion to avoid adding Debug to dyn ConfigManager; no compatibility alias or duplicate trait owner remains.
- 验证：Fresh focused Frameworks05 manager hard-cut guard passed; scoped rustfmt and git diff check passed; Shader04 post-fix zircon_runtime Cargo build passed in 8m32s at E:/ZirconBuilds/shader04-executor-validation-20260714/zircon-runtime-build-after-frameworks05.log. Exact core-min scene test was requeued by Runtime02 in the shared managed lane after the assertion fix.
- 回传：Frameworks05 manager hard-cut blockers are removed; Runtime02 owns and is running the exact upward core-min scene gate, and Shader04 may continue its compile gate.
