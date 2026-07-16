---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
resolved_at: 2026-07-15
summary_slug: sdf-font-bake-cjk-loaded-font-count-regression
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/text/05
related_code:
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/sdf/font_bake/tests.rs
  - zircon_runtime/src/text/font/shared.rs
tests:
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::ui::sdf_font_bake::tests::sdf_font_bake_rasterizes_materialized_system_cjk_face --locked -- --exact --nocapture
  - cargo test -p zircon_runtime --lib scene:: --locked
---


# Text05：CJK SDF bake loaded-font 统计回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 fresh 默认特性 runtime scene 验收门禁
- 修复责任计划：`docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md`
- 交接原因：失败位于 Text05 的真实 Windows CJK face materialization、SDF cache 与 bake report 语义；Editor02 不拥有字体烘焙或统计合同。
- 受管证据：Windows job `acfc6c19219441e498a6af33ce4b5e7a`，日志 `E:\ZirconBuilds\editor02-m1-runtime-scene-final-20260714.log`。

## 失败现象与复现证据

Windows-only `sdf_font_bake_rasterizes_materialized_system_cjk_face` 成功解析 `Microsoft YaHei UI`、烘焙 `本` 且其 slot/visible/empty/pixel 断言均越过，最终报告断言失败：`loaded_font_count` 实际为 `2`，测试要求 `1`。同一默认 scene 门禁其余 Editor02 generation、inspection、5k 深链与 cycle-edge 合同全部通过。

## 最低共享层根因

最低 owner 是 Text05 的 SDF face materialization 与 bake report 统计。`SdfFontBakeCache` 已按 `FontFaceId` 去重，并不存在同一 face 重复插入；真正原因是测试先物化 Microsoft YaHei UI，却只把 family/locale 写入 atlas key，没有携带 production shaping 已提供的 authoritative `font_id`。Text01 默认 CompositeFont 生效后，family-only fallback 合法选择打包的 CJK face，因此 resident cache 中出现两个不同 face。旧 `loaded_font_count = fonts.len()` 又把 resident 总数模糊称为 loaded 数，最终得到 2。

## 架构修复验收

- 明确 `loaded_font_count` 的 typed 语义，并让预物化 authoritative CJK face 与 atlas build 共享同一 face identity/cache lineage。
- exact Windows CJK 测试重复运行稳定，仍证明真实非零像素、单 slot、零 empty glyph；若确实需要第二个 face，报告须分别暴露原因而不是模糊扩大一个计数。
- fresh 重跑 Editor02 默认 scene 门禁，不引入 native/SDF fallback 或 locale face 选择回归。

## 禁止临时方案

- 禁止仅把 `1` 改成 `2` 或放宽为 `>= 1` 而不定义报告语义。
- 禁止禁用 Windows/CJK 测试、换成 synthetic face 或跳过真实系统 face materialization。
- 禁止在 Editor02 或渲染上层添加字体特判。

## 修复结果与回传

- 根因：Windows CJK test preloaded Microsoft YaHei UI but omitted authoritative font_id; Text01 CompositeFont therefore correctly selected a second packaged CJK face, while loaded_font_count ambiguously reported total resident faces.
- 架构修复：SdfAtlasBakeReport now separates resident_font_count from per-build loaded_font_count, and the Windows system-face test carries the same authoritative face ID as production shaped atlas keys without changing CompositeFont fallback order.
- 验证：Current-source managed Windows lib-test: sdf_font_bake 13/13; sdf_render 44/44 plus one explicit exporter ignored. Upward scene gate: 1705 passed, 3 foreign renderer shadow-binding failures, 6 ignored; no Text/SDF/Layout/Editor02 scene failure.
- 回传：Text05 CJK SDF loaded-font semantics are fixed; Editor02 may resume its M1 scene gate while existing renderer-owner failures remain routed separately.
