---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: text-font-id-report-database-boundary-contract-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_text_font_id_report.rs
tests:
  - python -m unittest -v tools.tests.test_frameworks_05_text_boundary.Frameworks05TextBoundaryTests.test_graphics_does_not_own_cpu_text_service_state
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_15_screen_space_ui_text_font_id_report_is_child_owner --locked --jobs 1 -- --exact --test-threads=1
---

# Runtime15: Text font-id report database boundary contract drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text MVP foundation F1 的 Frameworks05 Text 边界静态门禁
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：最低共享原因是 Runtime15 所有的结构吸收测试仍要求 graphics 读取完整 `FontDatabase`，与当前 Frameworks05 所有权门禁互斥。

## 失败现象与复现证据

`test_graphics_does_not_own_cpu_text_service_state` 初始失败并报告
`zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs:327: text_state.font_database()`；Text01 已把生产出口收敛为 Text-owned 单值 `font_face_id` 查询后，该测试通过 1/1。

但是 `render_ui_text_font_id_report.rs` 仍在 `assert_contains_all` 中硬编码要求父 owner 包含
`text_state.font_database()`，并要求 `font_id_report.rs` 直接持有 `FontDatabase`。因此当前默认 lib-test 一旦执行该 Runtime15 测试，旧断言必然与新的边界契约冲突；尚未声明该 Cargo 测试通过。

## 最低共享层根因

Runtime15 旧契约把“font-id report 已拆为 graphics 子 owner”误等同于“graphics 可借用完整 CPU Text 数据库”。当前权威边界只允许 graphics 注入 backend-id 到中立 `FontFaceId` 的窄查询，字体库、fallback、raster 与 atlas 服务状态继续由 `text` owner 持有。

## 架构修复验收

- 更新 Runtime15 结构测试，使其要求 `TextRenderState::font_face_id` 窄出口和解析闭包，并明确拒绝 production graphics 的 `text_state.font_database()`。
- 原始 Frameworks05 Python 边界测试保持通过。
- 精确 Runtime15 结构测试与 Text01 default/UI lib-test 在同一 fresh current-source managed gate 中通过。

## 禁止临时方案

- 不得恢复 graphics 对完整 `FontDatabase` 的访问。
- 不得用注释、`cfg(test)` helper、兼容包装或弱化 Frameworks05 测试来满足旧字符串断言。
- 不得复制 font-id 映射表或在 graphics 建立第二份字体数据库真值。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
