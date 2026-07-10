---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: editor-m1-font-discovery
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/text/01
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_editor/src/tests/editing/state/render_frame_submission.rs
plan_sources:
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked tests::editing::state::render_frame_submission_hud_text_renders_through_runtime_glyph_capture -- --exact --test-threads=1
---

# Runtime Text 01：Editor M1 字体发现失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 Windows 全量失败聚类与 V2 公共契约闭环测试阶段
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：最低共享故障位于 Runtime Text 01 所有的字体数据库初始化与 screen-space renderer 字体发现策略，不属于 Editor HUD 调用点。

## 失败现象与复现证据

精确用例 `render_frame_submission_hud_text_renders_through_runtime_glyph_capture` 为 0/1。UI command、quad 与 text payload 均已生成，但最终 `changed_pixels=0`，说明失败发生在共享 glyph/font 支撑层而非 Editor frame submission 编排。

复现命令：

```text
cargo test -p zircon_editor --lib --locked tests::editing::state::render_frame_submission_hud_text_renders_through_runtime_glyph_capture -- --exact --test-threads=1
```

## 最低共享层根因

`ScreenSpaceUiTextSystem::new(...)` 先通过 `glyphon::FontSystem::new()` 取得系统字体，随后默认空的 `shared_font_database_snapshot()` 经 `sync_font_system(...)` 覆盖 backend；默认 `ProjectAssetManager` 又没有 `res://fonts/default.font.toml` 可回填。当前生产构造路径没有执行计划声明的 `SystemFontPolicy::Discover`。

## 架构修复验收

- 先增加 FontDatabase 初始化、system-font policy 与 renderer consumption 的下层回归。
- 统一生产构造路径的字体发现所有权，不允许默认空快照破坏已经发现的系统字体。
- Editor HUD 精确用例通过，随后重新运行 Editor M1 声明的完整门禁。

## 禁止临时方案

- 禁止增加 Editor 专用字体旁路、硬编码平台字体、旧字体兼容分支或测试专用 glyph 注入。
- 禁止跳过共享 `FontDatabase` 或弱化 `changed_pixels` 产品断言。

## 修复结果与回传

- 状态：`open / 待修复`。
- 当前不声明 Runtime Text 01 或 Editor M1 对此门禁通过。
- 修复验收后，修复者必须更新本文件、移动到 `docs/plans/zircon_editor/editor/01/`，并重命名为 `fixed-{resolved_at}-editor-m1-font-discovery.md`；Runtime Text 01 仅保留相对链接和已修复摘要。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| FR-M1 / Editor M1 | Screen-space system-font Discover → sync 初始化 | `实现已落地-HUD仍未通过` | 2026-07-11 | Runtime Text owner 已新增 `initialize_screen_space_ui_font_system(...)`，生产构造现在对共享 snapshot 执行 `SystemFontPolicy::Discover` 后再 `sync_font_system(...)`，并加入 Windows 下层测试 `screen_space_ui_font_initialization_discovers_system_faces_from_empty_snapshot`。该下层 Cargo 首次验证被 `graphics/tests/project_render/project_scenes/reflection_probe_product.rs:175` 的外部 E0282 阻断；随后包含 Discover 生产修复的当前 Editor binary `E:\cargo-targets\zircon-editor-assets-content-scroll-hover-validator-0710\debug\deps\zircon_editor-1ca47919e17744f1.exe` 直接执行完整路径 HUD exact 仍为 0/1，`changed_pixels=0`（2026-07-11 05:51）。因此“空共享 snapshot 覆盖 system fonts”并非全部根因，Text owner 后续需继续从 font family 解析、buffer glyph 生成与 glyphon render/atlas 提交的最低层逐级验证；状态保持 open，禁止在 Editor HUD 增加字体或像素旁路。 |
