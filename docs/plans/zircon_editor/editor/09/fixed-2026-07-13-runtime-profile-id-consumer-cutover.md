---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: runtime-profile-id-consumer-cutover
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/core/framework/project/runtime_profile_id.rs
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/plugin/runtime_profile/feature_presets.rs
  - zircon_runtime/src/plugin/runtime_profile/defaults.rs
  - zircon_runtime/src/plugin/runtime_profile/descriptor.rs
tests:
  - cargo test -p zircon_editor --lib --no-run --locked --jobs 1
resolved_at: 2026-07-13
---


# Frameworks 05：RuntimeProfileId owner 硬切后 plugin consumers 未原子迁移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1.4 与 ProjectAuthority fixture 当前源码重编译
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：Frameworks05 正在把 `RuntimeProfileId` 从 plugin 域硬切到 neutral `core/framework/project` owner；Editor09 不应恢复 plugin re-export 或修改 profile 领域导入。

## 失败现象与复现证据

Windows 受管复现 `cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 在 Runtime 编译报：

- `plugin/runtime_profile/feature_presets.rs` 的 `use super::RuntimeProfileId` 为 E0432；
- `plugin/runtime_profile/defaults.rs` 从 descriptor 私有 import `RuntimeProfileId` 为 E0603；
- canonical 类型已经存在于 `core::framework::project::RuntimeProfileId`，descriptor 本身已直接使用该路径。

完整日志：`.codex/tmp/editor09-m1-4-source-authority-compile-r7-20260713.log`。

## 最低共享层根因

最低边界是 Frameworks05 neutral project contract hard cut 已删除 plugin root re-export，但两个 plugin
consumer 仍依赖旧的 sibling/descriptor 导入。修复应让 consumers 直接依赖 canonical neutral owner，
不是恢复旧 `runtime_profile` facade。

## 架构修复验收

- feature presets 与 defaults 直接从 `crate::core::framework::project` 导入 `RuntimeProfileId`。
- plugin `runtime_profile.rs` 不重新导出该类型，不留 alias 或 private-import 借道。
- Frameworks05 focused guards 通过，Runtime production check 通过。
- 原始 Editor09 no-run gate 越过 E0432/E0603。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止在 plugin facade 恢复 `RuntimeProfileId` re-export。

## 修复结果与回传

- 根因：Frameworks05 moved RuntimeProfileId to the neutral core framework project owner while feature_presets and defaults still imported it through retired plugin sibling/descriptor paths.
- 架构修复：Both consumers now import crate::core::framework::project::RuntimeProfileId directly; plugin runtime_profile does not restore a re-export.
- 验证：Source inspection confirms both stale import forms are absent. The next Editor09 managed no-run gate will provide upward compile evidence.
- 回传：Canonical RuntimeProfileId consumer cutover is present without a compatibility facade; Editor09 resumes the managed no-run gate.
