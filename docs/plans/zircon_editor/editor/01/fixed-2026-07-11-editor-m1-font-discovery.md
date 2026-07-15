---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: editor-m1-font-discovery
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/text/01
resolved_at: 2026-07-11
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/ui.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests/ui.rs
  - zircon_editor/src/tests/editing/state.rs
plan_sources:
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked render_frame_submission_hud_text_renders_through_runtime_glyph_capture --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib core::runtime::diagnostics::render_stats_store::product::tests::ui::render_product_diagnostics_record_ui_text_raster_stats --no-default-features --features target-client --locked -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::ui::text::tests::text_prepare_report_exposes_raster_upload_scroll_counters --no-default-features --features target-client --locked -- --exact --test-threads=1
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

- 根因：`ScreenSpaceUiTextSystem` 创建后用默认空的共享字体快照覆盖 glyphon 已发现的系统字体；默认工程又没有字体资产可以回填，生产路径因此没有可供 HUD glyph raster 使用的 face。
- 架构修复：Runtime Text 01 在 screen-space 文本系统的统一生产构造入口执行 `SystemFontPolicy::Discover`，再同步共享 `FontDatabase`；保留异步 raster、atlas upload 与最终 framebuffer 的真实路径，没有增加 Editor 专用字体、硬编码平台字体、测试像素注入或兼容旁路。
- 验证：2026-07-11 使用晚于相关源码的 Runtime 7492-test binary，字体发现、产品诊断与 prepare-report 三个 exact 均为 1/1；使用 09:01 Editor 2930-test binary 重跑 HUD 最终帧缓冲 exact 为 1/1（47.86s）。
- 回传：Runtime Text 01 的字体发现故障已关闭，Editor 01 可移除该 failure owner 并继续其余 M1 门禁；这只关闭本精确故障，不声明 Editor M1 全量通过。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| FR-M1 / Editor M1 | Screen-space system-font Discover → sync 初始化与异步 glyph 收敛 | `已修复-真实帧缓冲回归通过` | 2026-07-11 | Runtime Text owner 新增 `initialize_screen_space_ui_font_system(...)`，生产构造对共享 snapshot 执行 `SystemFontPolicy::Discover` 后再 `sync_font_system(...)`，并加入 Windows 下层测试 `screen_space_ui_font_initialization_discovers_system_faces_from_empty_snapshot`。首次下层 Cargo 被 Render 11 的外部 E0282 阻断；最终下层 exact 重跑在单体 lib-test 重新链接 1,204.1s 后超时且无 Rust 诊断、未执行断言，因此不声明该 filter green。05:51 的过渡 binary 在 12 帧收敛窗口下仍为 `changed_pixels=0`。Editor HUD 验证窗口随后与既有 Runtime 多语言产品门禁统一为 24 帧，覆盖字体发现、异步 raster worker、atlas upload 到最终 readback。当前源重新构建后，`cargo test -p zircon_editor --lib --locked render_frame_submission_hud_text_renders_through_runtime_glyph_capture --jobs 1 -- --test-threads=1` 实际运行 1 个测试并以 1 passed / 0 failed 结束（测试 54.49s，总命令 900.5s），证明 HUD 文本已写入最终帧缓冲。生产 `cargo check -p zircon_runtime --lib --no-default-features --features target-client --locked --jobs 1` 亦通过。未增加 Editor 专用字体、像素注入、硬编码平台字体或旧字体兼容分支。 |
| FR-M1 / Editor M1 | Fixed handoff 独立复验 | `已修复-复验通过` | 2026-07-11 | 使用 06:17 且晚于相关源码的 Editor test binary 执行 fully-qualified HUD exact，结果为 1 passed / 0 failed / 2927 filtered out，耗时 49.46s；24 帧收敛后最终 framebuffer 断言保持通过。 |
| FR-M1 / Editor M1 | Glyph/raster telemetry 诊断闭环 | `已修复-下层回归通过` | 2026-07-11 | 首次 Runtime lib-test 编译准确暴露 `ScreenSpaceUiTextRasterUploadReport` 测试 fixture 缺少 `worker_request_failed_count` 的 E0063；fixture 补入非零输入/输出传播断言后，修复后重新生成的 7469-test Runtime binary 执行 diagnostic-store exact 与 text prepare-report exact 均为 1 passed / 0 failed（各 0.01s）。 |
| FR-M1 / Editor M1 | Canonical fixed handoff 当前证据复验 | `已修复-上下层精确门禁通过` | 2026-07-11 | 10:44 且晚于字体相关源码的 Runtime 7492-test binary：`screen_space_ui_font_initialization_discovers_system_faces_from_empty_snapshot` 1/1（28.58s）、`render_product_diagnostics_record_ui_text_raster_stats` 1/1（0.05s）、`text_prepare_report_exposes_raster_upload_scroll_counters` 1/1（0.01s）；09:01 Editor 2930-test binary：`render_frame_submission_hud_text_renders_through_runtime_glyph_capture` 1/1（47.86s）。本记录同步硬切为 `handoff_kind: fixed` 并补齐规范修复结果字段。 |
| FR-M1 / Editor M1 | 2026-07-12 当前共享源码独立复验 | `已修复-下层与原始上层门禁通过` | 2026-07-12 | 晚于 `text.rs`/`database.rs` 的 current Runtime binary 执行 `screen_space_ui_font_initialization_discovers_system_faces_from_empty_snapshot` 为 1/1（26.89s，7,762 filtered）。Editor 当前 Cargo 重链 1,204.4s 无 Rust 诊断但在产出 binary 前超时，不计 green；随后使用同一共享工作区 19:58、晚于 HUD/字体相关源码的受管 Editor binary 确认 exact 存在并执行 `render_frame_submission_hud_text_renders_through_runtime_glyph_capture`，结果 1/1（47.37s，3,101 filtered）。原始 `changed_pixels` 最终 framebuffer 断言未弱化，仓库内未生成 target 输出。 |
| FR-M1 / Editor M1 | 2026-07-12 canonical fixed handoff 末次复验 | `已修复-当前受管二进制上下层4/4通过` | 2026-07-12 | Windows 托管 Runtime binary（23:49）执行 system-font Discover、render product raster diagnostics、prepare-report counters 三个 fully-qualified exact，均为 1/1（32.94s、0.01s、0.00s，7,795 filtered）；Editor binary（21:47）执行原始 HUD framebuffer exact 为 1/1（53.42s，3,107 filtered）。保留真实 `changed_pixels`、字体数据库、异步 raster/atlas/readback 路线；没有旧字体兼容、Editor 字体旁路或测试像素注入。 |
