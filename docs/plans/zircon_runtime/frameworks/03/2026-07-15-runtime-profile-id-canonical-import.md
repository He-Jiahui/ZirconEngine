---
related_code:
  - zircon_runtime/tests/frameworks_03_profile_feature_presets.rs
  - zircon_runtime/src/core/framework/project/runtime_profile_id.rs
  - zircon_runtime/src/plugin/runtime_profile/feature_presets.rs
implementation_files:
  - zircon_runtime/tests/frameworks_03_profile_feature_presets.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - managed Windows cargo test -p zircon_runtime --test frameworks_03_profile_feature_presets --locked (job 7d0c3e3ddfa148ce98e2350d8a3cc939; 2 passed)
  - rustfmt --edition 2021 --check zircon_runtime/tests/frameworks_03_profile_feature_presets.rs
  - git diff --check -- zircon_runtime/tests/frameworks_03_profile_feature_presets.rs
  - git grep -n -F .feature_preset() -- *.rs (no matches)
doc_type: milestone-detail
---

# Frameworks03 RuntimeProfileId Canonical Import And Explicit Preset Lookup

Plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
Milestone: M4
Status: completed
Date: 2026-07-15
Files: ["docs/plans/zircon_runtime/frameworks/03/2026-07-15-runtime-profile-id-canonical-import.md", "zircon_runtime/tests/frameworks_03_profile_feature_presets.rs"]

## Scope Delivered

| 切片 | 状态 | 完成证据 |
|---|---|---|
| RuntimeProfileId canonical owner import | `completed` | 测试从 `core::framework::project` 导入 `RuntimeProfileId`，preset table 继续从 plugin owner 导入，没有恢复 plugin façade re-export。 |
| 删除 enum convenience method 后的显式 preset 消费 | `completed` | 测试通过局部 `feature_preset_for` 对 `RUNTIME_PROFILE_FEATURE_PRESETS` 做穷举查找；受管聚焦门 job `7d0c3e3ddfa148ce98e2350d8a3cc939` 2/2 GREEN。 |

## 根因与修复

`RuntimeProfileId` 已硬切到 runtime project contract owner，但
`frameworks_03_profile_feature_presets.rs` 仍从 `zircon_runtime::plugin` 导入该类型。preset
数据仍由 plugin runtime-profile owner 提供，因此正确边界是分别从 project contract 与 plugin
preset owner 导入，而不是把类型重新导回 plugin。

随后 `RuntimeProfileId::feature_preset()` convenience method 被 hard-cut 删除，而该 integration
contract 仍保留九处旧方法调用。preset 数据的显式 owner 是
`RUNTIME_PROFILE_FEATURE_PRESETS`，因此测试现在通过局部、只读的 `feature_preset_for` 查表函数
按 `RuntimeProfileId` 解析 `RuntimeProfileFeaturePreset`。缺少任何内建 profile 时测试立即 panic，
不会静默回退。

修复不改变 profile 顺序、feature preset 数据或测试断言；没有增加 compatibility re-export、
alias、shim、重复类型或恢复被删除的 enum method。

## Fresh Testing Evidence

- 前序 job `e72bbd72c30c4678bafb7ce87e7624d2` 提供 canonical import 红态：plugin 中不再存在
  `RuntimeProfileId`；Shader06 上层受管 package 门随后提供删除方法后的 E0599 x9 红态。
- Windows 受管 test lane job `7d0c3e3ddfa148ce98e2350d8a3cc939` 在 841a 复用池运行
  `cargo test -p zircon_runtime --test frameworks_03_profile_feature_presets --locked`：2 passed，
  0 failed，exit 0。
- scoped rustfmt、diff check 通过；全仓 Rust source scan 确认 `.feature_preset()` 零命中。
- Shader06 负责重跑 external cubemap 3/3 与上层 package 门；Frameworks03 与 Runtime15 总计划
  仍为 `in_progress`。

## Review

独立 reviewer 通过协调器 M4 review gate 记录 Critical/Important findings；只有两项均为 0
时允许本里程碑提交。review evidence 由协调器持有，本文不复制可漂移的 reviewer 状态。

## 剩余范围

本记录只覆盖 preset integration test 的 canonical import 与显式 preset-table 消费，不改变
Frameworks03 的完整 profile matrix、CI target 或 optional-feature 完成状态。
