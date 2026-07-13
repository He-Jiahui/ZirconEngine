---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: subsurface-profile-mask-test-inference
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/subsurface.rs
tests:
  - ZR_EXPORT_CONTRACT_PLATFORM=windows cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --offline -- --nocapture
resolved_at: 2026-07-13
---


# Render 18：Subsurface profile mask 测试类型推断失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行切片：Editor 15 M1 Windows 平台策略合同
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：编译错误位于 Render 18 AF-M4 subsurface profile-table 测试 owner，Editor 15 不拥有 SSS 类型或断言。

## 失败现象与复现证据

平台策略合同编译 `zircon_runtime` lib-test 时，
`zircon_runtime/src/core/framework/render/advanced_lighting/subsurface.rs:167` 的
`assert_eq!(table.active_profile_mask, u16::MAX.into())` 产生 2 个 E0283。`u16::MAX.into()` 的目标类型在
`assert_eq!` 与 `serde_json::Value` 比较实现共同存在时无法唯一推断；520.3 秒后编译失败，平台策略测试尚未执行。

## 最低共享层根因

最低根因是 Render 18 测试边界使用无目标类型的 `.into()` 构造 profile mask，而不是显式构造
`serde_json::Value` 可比较的整数。该错误只属于 SSS profile-table 测试合同。

## 架构修复验收

- Render 18 SSS profile-table focused test 编译并通过。
- 原始 `ZR_EXPORT_CONTRACT_PLATFORM=windows` 平台策略命令执行到测试体并通过。
- Editor 15 M1 平台策略门禁取得可审计结果。

## 禁止临时方案

- 禁止跳过 runtime lib-test、删除断言或在 Editor 15 添加绕过。
- 禁止扩大 SSS public API 或恢复旧类型兼容层来影响推断。

## 产出记录与时间

| 时间 | 状态 | 产出 |
| --- | --- | --- |
| 2026-07-12 22:24 +08:00 | 未通过，已转交 | Render 18 SSS profile-mask 断言产生 2 个 E0283；归档到 AF-M4 owner，Editor 15 未绕过。 |

## 修复结果与回传

- 根因：Render18 SSS 测试使用无目标类型的 u16::MAX.into()，在 serde_json::Value 比较实现下产生 E0283。
- 架构修复：Render18 owner 将 mask 期望显式构造为 u32::from(u16::MAX)，不改变 SSS public API。
- 验证：当前 zircon_runtime test binary 晚于源码修复；render_sss_profile_table_caps_at_16 1/1，ZR_EXPORT_CONTRACT_PLATFORM=windows 的 platform_target_policy_matches_host_resource_and_plugin_strategy 1/1。
- 回传：Editor15 M1 Windows 平台策略门已恢复，可继续 full editor gate。
