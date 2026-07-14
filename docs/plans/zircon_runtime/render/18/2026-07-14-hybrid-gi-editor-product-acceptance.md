# Render 18 HybridGI Editor 产品接入验收

Plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
Milestone: M5
Status: completed
Files: ["docs/plans/zircon_runtime/render/18/2026-07-09-advanced-lighting-features-output-records.md", "docs/plans/zircon_runtime/render/18/2026-07-14-hybrid-gi-editor-product-acceptance.md", "docs/zircon_plugins/hybrid_gi/usage.md", "docs/tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_20260714.md", "docs/tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_actual_20260714.png", "docs/tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_fallback_20260714.png", "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs"]

## Scope Delivered

- 真实 Editor 产品已显示 provider 解析后的 HybridGI profile、mode、quality、trace/card/voxel 预算和结构化 fallback。
- Custom actual 与 IndoorStatic missing-bake fallback 均通过非空 WGPU viewport 产品画面验证。
- Runtime Diagnostics 紧凑状态组将 Render viewport/frame 状态放在实际值、预算与 fallback 之后，避免被 active-probe 行挤出首屏。
- HybridGI 使用指南、Plan18 状态与证据报告已同步；broad/full runtime/workspace 验证仍归后续 M6，不在本里程碑内虚报完成。

## Fresh Testing Evidence

| Milestone | Testing stage | Status | Date | Evidence |
| --- | --- | --- | --- | --- |
| M5 | M5-T HybridGI Editor actual/fallback product testing | 通过 | 2026-07-14 | actual/fallback 产品截图、17/17 静态契约测试、1/1 scene velocity 测试 |

- 当前源码 r10 `target-editor-host` 构建用时 47m12s，产品 SHA-256 为 `B828F854EB342D661637E16A6944D59437937A74AD29F5B25C743899C911251F`。
- `python -m unittest tools.tests.test_hybrid_gi_m4_contract tools.tests.test_hybrid_gi_editor_profile`：17/17 通过。
- `hybrid_gi_resolve_accepts_external_or_transient_scene_velocity`：1/1 通过。
- scoped `rustfmt --check` 通过；两张 1688x980 PNG 已人工目检，viewport 非空且重定向 stderr 为空。
- actual PNG SHA-256：`35A8FF93D8C67E3EEBC6A59F9C251EE9FAB279BC00406753C0D6FD600511844E`；fallback PNG SHA-256：`9619D6E3B8B011CC5EB53D71505D5F19606B02C5FCDFEDDF5ECACEBB4F6BB7CF`。

## Review

- 待协调器托管验证后，由不同 reviewer Session 检查截图可读性、状态与实际值一致性、无 double-owner 宣称以及 manifest 只包含本 Session 归属路径。
- 完成条件：critical=0、important=0；不得把 M5 的 Editor 产品子门扩大为 M6 broad/full 或整个目标完成。
