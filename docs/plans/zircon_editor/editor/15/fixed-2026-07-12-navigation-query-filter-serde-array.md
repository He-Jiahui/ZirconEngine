---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
resolved_at: 2026-07-12
summary_slug: navigation-query-filter-serde-array
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_plugins/05
related_code:
  - zircon_runtime/src/core/framework/navigation/query.rs
  - zircon_runtime/src/core/framework/navigation/tests.rs
tests:
  - cargo test -p zircon_plugin_navigation_recast --locked
  - cargo test -p zircon_plugin_navigation_runtime --locked
---

# Navigation 05：NavQueryFilter 固定数组 serde 编译失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行切片：Editor 15 M1.2 Windows focused compile gate
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：失败来自 Navigation M4-T2 新增的 `[Real; 64]` 查询过滤合同，最低共享原因归 Navigation framework owner。

## 失败现象与复现证据

Editor 15 的 Windows focused check 曾在 `NavQueryFilter.area_costs: [Real; 64]` 上触发 3 个 serde `E0277`，因为 serde 默认派生不覆盖该固定数组长度。预期是 framework 查询过滤 DTO 可序列化，且固定 64-area ABI 不变。

## 最低共享层根因

`NavQueryFilter` 直接派生 `Serialize` / `Deserialize`，但 `[Real; 64]` 缺少对应默认实现。问题位于 framework query DTO，而不是 Editor export 调用点。

## 架构修复验收

- 固定 64 项 cost table 能完整 serde round-trip。
- 长度不等于 64、非有限或非正 cost 必须拒绝。
- Navigation native/runtime 包级 Windows 验证通过，Editor 可恢复其原 focused gate。

## 禁止临时方案

- 不允许改成可变长度并在调用点截断或补齐。
- 不允许在 Editor 侧加别名、兼容 shim、静默 fallback 或 test-only bypass。
- 不允许削弱固定 64-area ABI 或反序列化校验。

## 修复结果与回传

- 根因：serde 默认派生不支持 `[Real; 64]`，导致 Navigation 查询过滤 DTO 阻断上层 Editor 编译。
- 架构修复：Navigation owner 为 cost table 增加显式序列 visitor，保持固定数组、严格长度与有限正数约束，并补齐 round-trip/非法输入测试。
- 验证：managed validator job `5c1a96ab19e54cb1bb47d091979e17d7` 通过 native 31 unit + 4 integration；job `567fc95691f44ec9a43ca895aabcfcc3` 通过 runtime 50 unit；均 0 failed。
- 回传：Editor 15 可恢复 M1.2 focused compile/test gate，不再需要绕过 Navigation query filter。
