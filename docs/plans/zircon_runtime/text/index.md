---
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/text_geometry.rs
  - zircon_runtime/src/ui/surface/text_shape.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/candidate_line.rs
  - zircon_runtime/src/ui/text/layout_engine/direction.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/range_mapping.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/cache/mod.rs
  - zircon_runtime/src/graphics/text/cache/frame_dedup.rs
  - zircon_runtime/src/graphics/text/cache/layout_cache.rs
  - zircon_runtime/src/graphics/text/cache/measure_cache.rs
  - zircon_runtime/src/graphics/text/cache/shaped_cache.rs
  - zircon_runtime/src/graphics/text/cache/tests.rs
  - zircon_runtime/src/graphics/text/parallel/mod.rs
  - zircon_runtime/src/graphics/text/parallel/shape_pool.rs
  - zircon_runtime/src/graphics/text/parallel/raster_pool.rs
  - zircon_runtime/src/graphics/text/parallel/tests.rs
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/align.rs
  - zircon_runtime/src/graphics/text/layout/overflow.rs
  - zircon_runtime/src/graphics/text/layout/tab.rs
  - zircon_runtime/src/graphics/text/layout/line_break/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break/tests.rs
  - zircon_runtime/src/graphics/text/layout/line_break/glue.rs
  - zircon_runtime/src/graphics/text/layout/line_break/glyph_fallback.rs
  - zircon_runtime/src/graphics/text/layout/line_break/smart.rs
  - zircon_runtime/src/graphics/text/layout/line_break/soft_hyphen.rs
  - zircon_runtime/src/graphics/text/layout/line_break/wrap_space.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/tests/text_pipeline.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/graphics/text/raster/mod.rs
  - zircon_runtime/src/graphics/text/raster/policy.rs
  - zircon_runtime/src/graphics/text/raster/swash.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/host_requests.rs
  - zircon_runtime/src/graphics/text/font/mod.rs
  - zircon_runtime/src/graphics/text/atlas/mod.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/allocation.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/failure.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/placeholder.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/retry.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/staged_upload.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/staging.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/tests.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/types.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/upload.rs
  - zircon_runtime/src/graphics/text/atlas/bitmap_run/validation.rs
  - zircon_runtime/src/graphics/text/atlas/render_contract.rs
  - zircon_runtime/src/graphics/text/atlas/render_contract/tests.rs
  - zircon_runtime/src/graphics/text/atlas/render_plan.rs
  - zircon_runtime/src/graphics/text/atlas/render_plan/tests.rs
  - zircon_runtime/src/graphics/text/atlas/render_batch.rs
  - zircon_runtime/src/graphics/text/atlas/render_batch/tests.rs
  - zircon_runtime/src/graphics/text/atlas/render_gpu_plan.rs
  - zircon_runtime/src/graphics/text/atlas/render_submission.rs
  - zircon_runtime/src/graphics/text/atlas/render_submission/frame_driver.rs
  - zircon_runtime/src/graphics/text/atlas/render_submission/frame_state.rs
  - zircon_runtime/src/graphics/text/atlas/render_submission/tests.rs
  - zircon_runtime/src/graphics/text/atlas/shaders/glyph_atlas_sampling.wgsl
  - zircon_runtime/src/graphics/text/atlas/page_residency.rs
  - zircon_runtime/src/graphics/text/atlas/page_residency/tests.rs
  - zircon_runtime/src/graphics/text/atlas/upload.rs
  - zircon_runtime/src/graphics/text/atlas/upload/tests.rs
  - zircon_runtime/src/graphics/text/atlas/dirty.rs
  - zircon_runtime/src/graphics/text/atlas/dirty/tests.rs
  - zircon_runtime/src/graphics/text/atlas/raster_key/mod.rs
  - zircon_runtime/src/graphics/text/atlas/raster_key/tests.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/font/fallback.rs
  - zircon_runtime/src/graphics/text/font/fallback/tests.rs
  - zircon_runtime/src/graphics/text/font/coverage.rs
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/handoff.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/retry_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/source_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/resource.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_advances.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_char_run.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/draw_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/shader_contract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/layout_placement.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - docs/zircon_runtime/graphics/text.md
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/blend.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/blend/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/sync.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/sync/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/style/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/divider/geometry/label_bounds/horizontal.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/divider/horizontal.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/chip/geometry/label.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/chip/geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/alert/geometry/message.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/alert/geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/avatar/geometry/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/avatar/geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/badge/geometry/root_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/badge/geometry/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/badge/geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/actions/labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/actions/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
  - zircon_runtime/src/ui/tests/text_layout/mod.rs
  - zircon_runtime/src/ui/tests/text_layout/alignment.rs
  - zircon_runtime/src/ui/tests/text_layout/wrapping.rs
  - zircon_runtime/src/ui/tests/text_layout/overflow.rs
  - zircon_runtime/src/ui/tests/text_layout/direction.rs
  - zircon_runtime/src/ui/tests/text_layout/edit_state.rs
  - zircon_runtime/src/graphics/text/shaping/mod.rs
  - zircon_runtime/src/graphics/text/shaping/cosmic.rs
  - zircon_runtime/src/graphics/text/shaping/fallback_spans.rs
  - zircon_runtime/src/graphics/text/shaping/script_segment.rs
  - zircon_runtime/src/graphics/text/shaping/line_break.rs
  - zircon_runtime/src/core/framework/render/text/shaped_run.rs
  - zircon_runtime/src/graphics/text/font/backend.rs
  - zircon_runtime/src/graphics/text/font/shared.rs
  - zircon_runtime/src/core/framework/render/text/shaping_service.rs
  - zircon_runtime/src/core/framework/render/text/font/
  - zircon_runtime/src/graphics/text/layout/line_break/greedy.rs
  - zircon_runtime/src/graphics/text/atlas/page.rs
  - zircon_runtime/src/graphics/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/graphics/text/atlas/dirty.rs
  - zircon_runtime/src/graphics/text/font/descriptors.rs
  - zircon_runtime/src/graphics/text/font/matching.rs
  - zircon_runtime/src/graphics/text/font/default_families.rs
  - zircon_runtime/src/graphics/text/font/asset_registration.rs
  - zircon_runtime/src/ui/text/layout_engine/vertical.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/Cargo.toml
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/CompositeFont.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/godot/servers/text/text_server.h
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/slint/internal/core/textlayout/sharedparley.rs
plan_sources:
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - .codex/plans/UI SDF 字体真实 Bake 收束计划.md
status: in_progress
---

# Zircon 文本与字体渲染子系统总体计划

本目录是 `zircon_runtime` **文本与字体渲染子系统的权威深度计划**。它把当前散落在 `ui/text`(启发式布局)、`graphics/scene/scene_renderer/ui`(glyphon bitmap + fontsdf SDF 双绘制后端)与 `asset`(字体资产)中的局部能力,收敛为一条**模块化、可测、高精度、高性能**的文本主链:

```
字体文件/资产 → FontFace/字体库/回退链 → shaping(Unicode/BIDI/竖排) →
换行/度量/布局 → 字形栅格(bitmap / SDF / MSDF) → 字形图集 →
渲染提取(quad/批) → 命中测试/光标/编辑 → IME
                       ↑                              ↑
                  多线程与缓存(贯穿全链)        富文本(HTML/BBCode 预处理)
```

## 0. 为什么单独立这条线

`render/14`(2D 栈)的 `TD-M1` 已经决定"把文本 shaping/字形图集**下沉为 `graphics/text/` 共享服务**,UI 与场景 2D 共用"。但 `render/14` 的篇幅集中在**2D 场景渲染器、sprite 批集成、UI 文本路径硬切换**,并未展开文本子系统**内部**的高精度细节(BIDI/竖排/MSDF/字体回退/富文本/IME/多线程)。`editor_layout/17` 是**编辑器侧排版规范**(度量=绘制、DPI 重栅格、换行自适应),`editor_ui/03` 是**编辑器文本栈定稿**(主链贯通)。三者都**消费**一个尚未深描的运行时文本服务。

本目录就是那个服务的实现权威:**`graphics/text/**` 共享服务内部 + `core/framework/render/text/**` 契约深化**,把用户要求的 15 项能力(glyphon、SDF/MSDF 动态与预生成、UE 风格度量算法、多语言 BIDI/竖排、换行规则、渲染规则、分辨率精度、字体文件处理、图集化、多线程、Unicode、FontFace、回退规则、富文本、多平台 IME、字体回退)逐项落到文件级实施权威。

## 1. 边界与归属(与三份既有计划的勾稽)

固定分工,**不重叠、不矛盾**:

| 计划 | 拥有 | 与本目录关系 |
|------|------|------------|
| 本目录 `text/**` | 文本服务内部:字体库/FontFace/回退、shaping/Unicode/BIDI/竖排、换行/度量算法、栅格/图集、SDF/MSDF、富文本预处理、IME 接口、多线程与缓存 | **实现权威**;`graphics/text/**` + `core/framework/render/text/**` |
| `render/14`(2D 栈) | 场景 `TextRenderer` 组件、glyph quad→sprite 批、UI 文本路径**硬切换**装配、2D 排序 | `TD-M1` 的"共享服务内部"**委托本目录**;`render/14` 持有"如何把 `ShapedGlyphRun` 变成场景顶点/批" |
| `editor_layout/17` | 编辑器排版**规范**:度量=绘制四规则、字形随 `scale_factor` 重栅格、换行自适应两阶段、shrink-to-fit | **消费方**:本目录服务必须满足其度量一致性与 DPI 重栅格契约(本目录 §6.2/04) |
| `editor_ui/03` | 编辑器文本栈**贯通**:Label/Field/Console/树表同一链、CJK 一等公民、编辑链与候选窗实机 | **消费方**:其"shaping 权威未定/栅格策略未书面化/字体注册表缺失/IME 不闭环"四缺口由本目录 01/02/04/06/08 正面补齐 |
| `runtime/15`(结构规范) | owner-module 模式、命名前缀、`module_convention_gate`/`large_file_ownership_gate` | 本目录所有新增文件遵其结构规则;`ui/text` 巨型文件拆分纳入其治理 |

**契约名权威**:`render/14` 已定稿契约层类型(`ShapedGlyphRun`/`ShapedGlyph`/`ShapedLine`/`TextShapingService`/`TextStyle`/`ShapedTextCacheKey`/`GlyphAtlasFormat`/`GlyphAtlasRef`/`RenderTextSnapshot`,见 `render/14` §核心类型与接口)。本目录**沿用并扩展**这些类型,不另造同义类型;扩展项(变量字体轴、竖排朝向、富文本 span、回退命中 face)以本目录各子计划"工程落地细化"为准,扩展后回填 `render/14` 契约定义。

**契约名勘误注(2026-07-02 评审收口)**:按代码核实,上述契约名与实际落地存在如下偏差,以本注为准:`ShapedLine` 实际落地名为 `ShapedTextLine`(`core/framework/render/text/shaped_run.rs`);`GlyphAtlasRef`/`RenderTextSnapshot`/`ShapedTextCacheKey`/`TextStyle` 尚未落地,属待建契约;`GlyphAtlasFormat` 已落地但位于实现层 `graphics/text/atlas/page.rs` 且为 `pub(crate)`,尚未提升到契约层;另有已落地但 `render/14` 未收录的契约名:`TextShapeRequest`/`TextOrientation`/`VerticalMode`/`ShapedGlyphScript`,需回填 `render/14` 契约定义。

## 2. 现状评审(按代码核实,2026-06-27)

### 2.1 已成立(取其能力,不推倒)

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

已成立能力的详细明细已迁入 Text 09 产出目录；此处只保留当前能力概述。

- 迁入记录：[`09/2026-07-09-index-output-records.md`](09/2026-07-09-index-output-records.md)

| 能力 | 当前概述 |
|------|----------|
| glyphon bitmap atlas 绘制后端 | 真实 runtime WGPU 产品帧缓冲已验证 CJK、RTL 与彩色 Emoji；后端实际 color face→`SwashContent::Color`→RGBA 字节合同已直证，完整 atlas 硬切仍由 Text 04 跟踪。 |
| fontsdf SDF 烘焙 + 图集 + 渲染 | 真实 runtime WGPU 产品帧已验证横排 SDF 与两列 VerticalRl CJK 标点；系统 face 经共享 `FontDatabase` 物化后，实际 shaped glyph id/face id 直接进入 atlas key 与 indexed fontsdf bake，backend vertical origin/advance/rotation 由生产 quad 消费。MSDF/MTSDF、horizontal shaped-SDF 全链与 native/SDF parity 仍由 Text 05 跟踪。 |
| 启发式布局/换行/对齐/省略 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| Unicode BIDI / 竖排 shaping | UAX#9 paragraph/line owner、TTB/BTT vertical backend、Vertical_Orientation、`vert`/`vrt2`、native `vmtx` 与 vertical origin 已落地；horizontal cosmic per-run `locl`、变量轴与更完整黄金 corpus 仍由 Text 02 跟踪。 |
| grapheme 边界/导航 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| 命中测试 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| 度量缓存 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| 字体回退链 | 后端实际 face-ID 与 shared/native CompositeFont span 已硬切；bounded 缺字诊断/partial cluster 已进入帧外报告，真实 CJK/Arabic/Hebrew/彩色 Emoji、zh-Hans/ja 同码点产品帧与 Arabic mark complex-cluster 单实际 face 均已验收；per-run `locl` 仍由 Text 02/06 跟踪。 |
| 富文本最小集 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| 字体资产/导入 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| IME 上下文/编辑链 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| 渲染 DTO | 已有基础能力，后续实现与验证记录见对应子计划。 |

### 2.2 关键缺口(本目录正面补齐)

| 缺口 | 用户需求项 | 承接子计划 |
|------|-----------|-----------|
| shaping 权威未定(glyphon 挂名未接布局、无 GSUB/GPOS) | glyphon 方案、Unicode | 02 |
| 无完整 UAX#9 BIDI、无竖排 | 多语言 BIDI、竖排 | 02 |
| 度量=启发式等宽,非真实字形 | UE 文本长度算法 | 03 |
| 换行非 UAX#14、无 CJK 行首尾禁则、无连字符 | 换行规则 | 03 |
| 栅格器选型未定、图集策略未书面化、无 DPI 重栅格契约 | 图集化、分辨率精度 | 04 |
| 仅单通道 SDF,无 MSDF、无离线预生成 | SDF/MSDF 动态与预生成、渲染规则 | 05 |
| 字体回退硬编码、非脚本感知、单 face | 回退规则、字体回退、FontFace | 01、06 |
| 富文本仅 markdown 三标记 | 富文本 HTML/BBCode | 07 |
| IME 候选窗未实机、多平台未抽象、竖排/BIDI affinity 未接候选锚定 | 多平台 IME 接口 | 08 |
| 全链单线程、无异步栅格、缓存契约缺失 | 多线程处理 | 09 |
| 字体文件仅单 face TOML、无 TTC/WOFF2/变量字体/系统字体发现 | 字体文件处理、FontFace | 01 |

## 3. 子计划地图与执行顺序

| 计划 | 文档 | 主题 | 依赖 |
|------|------|------|------|
| 01 | `01-font-resource-faces-and-database.md` | 字体文件处理 / FontFace / CompositeFont / 字体库 / 系统字体发现 / 资产与导入 | 无(最先) |
| 02 | `02-shaping-unicode-and-bidi.md` | shaping 后端 / Unicode / UAX#9 BIDI / 脚本分段 / 竖排 / cluster 映射(备注:整形期 fallback 先用 cosmic-text 内置,FB-M1 后切 FallbackResolver 配置的 fontdb 候选序(见 06),消除 02↔06 表面环。2026-07-02 评审收口) | 01 |
| 03 | `03-line-breaking-measure-and-layout.md` | UAX#14 换行 + CJK 禁则 / UE 风格度量算法 / 对齐与两端对齐 / 竖排布局 / 行高 | 02 |
| 04 | `04-glyph-atlas-and-rasterization.md` | swash 栅格 / shelf 图集 / 页 LRU / 脏矩形上传 / DPI 重栅格 / subpixel / hinting | 02 |
| 05 | `05-sdf-msdf-pipeline.md` | SDF(动态)/ MSDF(动态 fdsm + 离线预生成)/ 渲染规则(着色/阈值/outline/阴影)/ 分辨率无关 | 04 |
| 06 | `06-font-fallback.md` | 脚本感知回退 / Unicode 范围 / CompositeFont 回退链 / 深度限制 / tofu | 01、02 |
| 07 | `07-rich-text-html-bbcode.md` | BBCode + HTML 子集解析 / 装饰器 schema / 内联对象 / 样式 run 合并 | 02、03 |
| 08 | `08-ime-and-text-input.md` | 多平台 IME(TSF/IMM32/macOS/IBus/fcitx/Web)/ preedit/composition / 候选窗定位 / 编辑链 | 03、04 |
| 09 | `09-threading-caching-and-performance.md` | 并行 shaping / 异步栅格 / worker pool / 缓存体系 / 精度与性能预算 / 性能计数 | 01–05 |

**阶段划分**:

- **阶段 A(底座):01 → 02 → 03。** 字体库立起来,shaping 接真实后端,度量=绘制根治错位。这一段即 `render/14 TD-M1 切片 1a` 与 `editor_layout/17 G1` 的共同地基。
- **阶段 B(像素质量):04 → 05。** 栅格与图集随 DPI 重栅格(`editor_layout/17 G2`),SDF 升级 MSDF + 离线预生成。
- **阶段 C(国际化与交互):06 + 07 + 08 并行。** 回退链脚本感知、富文本 HTML/BBCode、IME 多平台实机。
- **阶段 D(性能收敛):09。** 把 A–C 的同步实现并行化、异步化,缓存契约定稿,性能计数进测试。

`render/14 TD-M1 切片 1c`(UI 文本路径硬切换)在阶段 A 末执行——服务接口(`TextShapingService`)稳定后,按 `render/14` 硬切换清单一次性迁移 `ui/text` 调用方并删除启发式路径。

## 4. 参考引擎分工(对齐 zr-reference-engine-routing)

| 引擎 | 主导领域 | 关键源码 |
|------|---------|---------|
| **UnrealEngine / Slate** | 字体缓存/CompositeFont 回退/shaped run 缓存键/HarfBuzz 双向整形/SDF 生成/度量/富文本 marshaller/平台 TextField | `SlateCore/.../Fonts/{FontCache,CompositeFont,FontCacheHarfBuzz,SlateTextShaper,SlateSdfGenerator,FontGeometryPreprocessing,SlateFontRenderer,FontMeasure}`、`Slate/.../Framework/Text/{TextLayout,ShapedTextCache,RichTextLayoutMarshaller,TextDecorators,PlatformTextField}` |
| **godot / TextServerAdvanced** | TextServer API 形态/HarfBuzz+ICU 整形/UAX#9 BIDI/UAX#14 行断/竖排朝向/MSDF 导入(msdfgen)/字体回退 | `servers/text/text_server.{h,cpp}`、`modules/text_server_adv/{text_server_adv.cpp,script_iterator.cpp}`、`editor/import/{resource_importer_dynamic_font,dynamic_font_import_settings}.cpp`、`thirdparty/msdfgen` |
| **bevy / bevy_text** | Rust/wgpu 落地形态(parley 0.8 后端):shaping→图集→quad 全链、`FontAtlasKey`、`PositionedGlyph` | `crates/bevy_text/src/{pipeline,font_atlas,font_atlas_set,parley_context,glyph,font,text_edit,cursor}.rs` |
| **slint / textlayout** | 轻量 Rust 文本布局:shaping trait、Unicode/简单双行断器、glyph cluster | `internal/core/textlayout/{sharedparley,shaping,linebreaker,linebreak_unicode,linebreak_simple,fragments,glyphclusters}.rs` |
| **Fyrox / fyrox-ui** | 极简 Rust 字体/回退(MAX_FALLBACK_DEPTH)/RectPacker 图集/换行 | `fyrox-ui/src/font/{mod,loader}.rs`、`formatted_text.rs`、`formatted_text/textwrapper.rs` |

**纪律(防凭空实现,继承 `render/14` §8.8)**:每个机制动手前先读对应子计划"参考代码"表——UE/godot 提供算法与设计样板(BIDI/行断禁则/MSDF/回退/IME 平台抽象),bevy/slint/Fyrox 提供 Rust/wgpu 落地形态(所有权、缓存键、wgpu 资源)。两类都读,不得只凭记忆。无 Rust 同类参照的机制(如完整竖排、MSDF 离线预生成格式)必须对拍测试先行。

## 5. 选型决策(全目录共享,子计划引用不重定)

| 关注点 | 选型 | 理由 | 备选 |
|--------|------|------|------|
| shaping + Unicode + BIDI + 行断 | **cosmic-text**(承接 `render/14` 既定选型) | 纯 Rust,一库聚合 rustybuzz(整形)+ swash(栅格)+ unicode-bidi(UAX#9)+ unicode-linebreak(UAX#14)+ unicode-script + fontdb(字体库/回退);CJK 与混排开箱;`render/14` 已定 | parley 0.8(bevy 现用,API 更分层但需自管 fontique) |
| 字形栅格(bitmap) | **swash**(cosmic-text 内置,亦可直用) | 彩色 emoji(ColorBitmap/ColorOutline)+ outline alpha + subpixel;bevy 同源 | fontdue(无彩色)、ab_glyph |
| 动态 SDF | **fontsdf**(既有,保留) | 已全链落地,单通道 R8 | swash outline + 自研 SDF |
| MSDF(动态 + 离线) | **fdsm**(纯 Rust MSDF/MTSDF 生成器) | 纯 Rust、无 C++ msdfgen 依赖;离线产物格式对齐 godot msdfgen 语义 | C++ msdfgen FFI(godot 路线,引入构建复杂度) |
| 字体数据库 / 系统字体发现 | **fontdb**(cosmic-text 内置) | 系统字体枚举 + family/weight/style 索引 + 回退候选 | font-kit(更重) |
| 富文本解析 | **自研 BBCode + HTML 子集解析器** | 标签 schema 受控、对齐 godot `RichTextLabel` BBCode 与 UE marshaller | html5ever(过重,安全面大) |
| IME 平台层 | **winit IME 事件 + 平台扩展抽象** | 复用既有 `ime_context` 出入站契约;平台特化经 `zircon_app` 注入 | 直接绑 TSF/IBus(放入平台插件) |

**隔离硬规则**:`cosmic_text`/`fontsdf`/`fdsm`/`fontdb`/`swash` 等第三方类型**只允许**出现在 `graphics/text/` 实现层的指定隔离文件内(见各子计划),出口一律契约层类型(`ShapedGlyphRun` 等 serde 可序列化、无 wgpu/无第三方句柄)。`core/framework/render/text/**` 与 `zircon_editor`/`zircon_app` 不得直接 import 上述库。

**现状违例记录(2026-07-02)**:按代码核实,当前存在以下隔离硬规则违例——`zircon_editor` 直接依赖并 import `fontdb`/`fontdue`/`swash`(`ui/retained_host/host_contract/paint_text/{font,raster}.rs`);`rhi_wgpu/ui_surface/text.rs` 直接 import `glyphon`;`cosmic-text` 实为经 glyphon re-export 使用而非直接依赖;`fdsm` 尚未引入。这些违例的收束路径:editor retained-host 路径迁移到 runtime 文本服务后删除直依;收束前该违例冻结不扩大(不得新增直接 import 点)。


> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Text 总索引中 2026-07 的 editor retained-host、runtime text cache、native atlas 与验证证据明细已迁入 Text 09 产出目录；此处只保留选型、边界与全局工程约定。

- 迁入记录：[`09/2026-07-09-index-output-records.md`](09/2026-07-09-index-output-records.md)

## 6. 全局工程约定(各子计划"工程落地细化"共享,不重定)

1. **模块归属**:契约层 `zircon_runtime::core::framework::render::text`(serde、无 wgpu);实现层 `zircon_runtime::graphics::text`(共享服务,持缓存/图集/隔离层);场景渲染器 `graphics/scene/scene_renderer/text`(归 `render/14`);UI 消费方 `ui/text`(硬切换为服务适配器)。不新增 crate。
2. **度量=绘制(继承 `editor_layout/17` 四规则)**:任何布局几何(advance/kerning/ascent/descent/换行点)必须来自 shaping 服务的真实字形度量,与绘制端**同一来源**。禁止任何路径回退到等宽近似。
3. **字形随 DPI 重栅格**:栅格输入 `physical_px = logical_px × scale_factor`;图集 key 含 `scale_factor` 量化桶(接 `editor_layout/17 §3.4`、`render/14`)。SDF/MSDF 因分辨率无关,bake 尺寸固定、运行时按 `font_size` 缩放采样。
4. **缓存键不持引用**:shaped run / measure / atlas 的缓存键一律用 `font_id + size_bits(f32::to_bits) + features_hash + 文本 hash`,不持文本/字体对象引用(改造自 UE `FCachedShapedTextKey`,见 `render/14` §核心类型)。features_hash 的来源见 02 `TextShapeRequest.features`(规范化=按 tag 排序后 hash)(2026-07-02 评审收口)。命中后必须等值比较文本副本防 hash 碰撞(缓存值持 `Arc<str>` 副本 verify,见 D6/09)(2026-07-02 评审收口)。
5. **图集格式**:`R8Unorm`(alpha mask、SDF)/ `Rgba8Unorm`(彩色 emoji、MSDF);格式分组分页(`GlyphAtlasFormat`);页级 LRU,本帧引用页不可逐出(`render/14` §目标架构同款)。
6. **测试命名(继承 `render/14` index §8.6)**:`text_<topic>_*` 单测(布局/度量/回退/富文本/缓存的确定性断言)、`render_text_*` 服务集成、`render_product_text_*` 抓帧对拍、`render_perf_text_*` 性能计数(shape 次数/栅格次数/图集上传字节,确定性计数断言;时间类只观测)。
7. **里程碑优先(milestone-first)**:切片期 `cargo check -p zircon_runtime --lib --locked`;里程碑末 `cargo test -p zircon_runtime <过滤词> --locked`。UI 文本硬切换以 `ui/tests` 全量文本测试为闸门(清单见 `render/14` §UI 文本路径硬切换清单)。
8. **实施权威**:各子计划"## 工程落地细化"是该计划实施权威——文件落点、类型签名、算法、切片步骤、测试清单以该章节为准,与正文概述冲突时以细化章节为新。
9. **结构纪律(遵 `runtime/15`)**:owner-module 模式,root `mod.rs` 留薄 façade;`ui/text/layout_engine.rs`/`graphics/.../ui/text.rs` 等大文件按 owner 叶子拆分;新增文件按命名前缀词表去冗余前缀。

## 7. 能力覆盖矩阵(用户需求 → 承接子计划)

| 用户需求项 | 承接子计划 | 备注 |
|-----------|-----------|------|
| glyphon 方案 | 02、04 | 保留为 bitmap atlas 绘制后端;布局接 cosmic-text(glyphon 同源) |
| SDF 方案(动态/预生成) | 05 | 动态 fontsdf 保留;离线预烘焙产物格式定稿 |
| MSDF 方案(动态/预生成) | 05 | fdsm 动态生成 + 离线预生成;多通道 + 中线通道(MTSDF) |
| 文本长度计算算法(UE 对齐、BIDI、竖排) | 03、02 | `FShapedGlyphSequence::GetMeasuredWidth` 子范围度量对齐 |
| 多语言国际化 BIDI | 02 | UAX#9(cosmic-text/unicode-bidi);run 重排 + 镜像字符 |
| 竖排等模式 | 02、03 | 朝向枚举(对齐 godot `Orientation`);竖排 advance/baseline |
| 换行规则 | 03 | UAX#14 + word/glyph + CJK 行首尾禁则 + 连字符 + 两端对齐 |
| 渲染规则 | 05、04 | bitmap alpha 混合 / SDF/MSDF 阈值 + `fwidth` 抗锯齿 + outline/阴影/下划线 |
| 分辨率精度 | 04 | 物理像素栅格、subpixel 定位、hinting、scale 量化桶 |
| 字体文件处理 | 01 | TTF/OTF/TTC/WOFF2、变量字体轴、face 索引 |
| 图集化生成 | 04 | shelf 分配、多页、脏矩形增量上传、LRU 逐出 |
| 多线程处理 | 09 | 并行 shaping、异步栅格上传、worker pool |
| Unicode 支持 | 02 | grapheme/script/规范化/组合字符/emoji 序列 |
| FontFace | 01 | `FontFace`(单 face)/ `CompositeFont`(family+回退)分层 |
| 回退规则 / 文本字体回退 | 06 | 脚本感知 + Unicode 范围 + 链式 + 深度限 + tofu |
| 富文本(HTML/BBCode) | 07 | BBCode 全集 + HTML 受控子集 + 装饰器 + 内联对象 |
| 多平台 IME 输入法接口 | 08 | TSF/IMM32(Win)、NSTextInputClient(mac)、IBus/fcitx(Linux)、Web |
| letter/word-spacing 与 OT features(2026-07-02 评审收口) | 02、03、07 | `TextShapeRequest.features`(tnum/smcp/liga 等)进 features_hash;spacing 在布局层应用 |
| language/locale 与 Han 消歧(2026-07-02 评审收口；2026-07-10 数据面完成) | 02、06 | `UiResolvedStyle.language` 段落/run 级 BCP 47 字段已进入 layout/shaped/SDF cache 与 native/SDF fallback；`locl` 因 cosmic-text `Attrs` 能力仍待后端支持 |
| 混 face 行度量与 baseline(2026-07-02 评审收口) | 03 | 行 ascent/descent 取行内各 run face 度量 max;baseline 统一 alphabetic(D7) |
| 字体失效级联(2026-07-02 评审收口) | 01、09 | face 失效 → 缓存/图集/SDF bake 级联剔除;09 缓存契约表持"失效来源"列 |
| 文本选择/caret affinity/双击选词(2026-07-02 评审收口) | 03、08 | `CaretAffinity` 模型、软换行行尾归属、grapheme/word 边界导航 |

### 7.1 已识别全局缺口与承接(2026-07-02 评审收口)

以下缺口在本轮评审中识别,登记归属,防止落入无主区:

- **gamma/linear 混合策略**:归 05 渲染规则;V1 观测不动(维持现有混合行为),仅记录对拍差异。
- **native/SDF paragraph parity 闸门**:归 05 新增里程碑 SM-M5;验收=同串同布局两路径 bbox/advance 逐项断言。
- **超长文本分段与虚拟化**:归 09 PF-M5(段落级脏跟踪、可视区增量 shape)。
- **泰/老/高棉字典断行**:归 03;V1 豁免按 glyph 断,后续以 `icu_segmenter` 可选 feature 立项。
- **合成 bold/oblique 应用层**:归 04(`GlyphRasterKey.synthetic: SyntheticFlags`,embolden/shear)。
- **emoji 肤色/旗帜/CBDT strike**:归 02(cluster 语义)/04(strike 选择与下采样)。
- **编辑器窗口级字体一致性视觉验收**:归本目录 runtime text 服务 + `editor_layout/17` 消费方联合验收;用户截图暴露的 Workbench/Asset Browser/Component Atlas 字号、字重、字体族、line-height 与 code/mono lane 混用问题不得只用底层字段传递视为完成。验收=真实 editor 截图与 runtime text 证据图同时写入 `docs/tests/runtime/text` 或 `docs/tests/editor`,并在状态表明确说明是否关闭窗口级 QA。
- **ruby/tcy(縦中横)/纵中横**:显式 V2 范围外,不在本目录 V1 承诺内。
- **IME 失焦提交/preedit clause/密码框**:归 08(blur→commit、`clauses` 字段、secure 标志)。
- **装饰线度量来源**:归 05(SM-M4 消费)/01(`post`/`OS/2` 表解析)。

## 8. 全局验收与测试基线

- 切片期:`cargo check -p zircon_runtime --lib --locked`。
- 里程碑测试:`cargo test -p zircon_runtime text --locked` + UI 文本全量回归(`ui/tests/{text_shaper,text_layout,text_hit_testing,render_text_fields,widget_text_input_pointer,surface_dirty_mui,boundary}`)。
- 产物对拍:`render_product_text_*`(中英混排/CJK/阿拉伯 RTL/竖排/SDF/MSDF/emoji)+ `ZR_RENDERDOC_CAPTURE_NEXT=1` 抓帧对照 UE/godot 同串。
- 多语料黄金集:维护 `text_corpus`(拉丁/CJK/阿拉伯/希伯来/天城文/泰文/emoji/混排)驱动度量与换行对照表,期望值以参考引擎或 Unicode 标准用例标定。
- 文档镜像:每里程碑后更新 `docs/zircon_runtime/ui/text*`、`docs/zircon_runtime/asset/assets/font.md` 镜像,并回填本目录状态表。

## 9. 风险与回退

- **shaping 库 CJK 排版细节不足**:以 `ShapedGlyphRun` 隔离选型,cosmic-text 不足时可换 parley/自研 rustybuzz 绑定而不动调用方(`render/14` 同款风险与对策)。
- **UI 文本硬切换面大**:阶段 A 末以 UI 全量文本测试为闸门,失败修服务不回退双路径。
- **MSDF 离线产物无 Rust 同类参照**:对拍测试先行,产物格式逐字节断言对照 godot msdfgen 语义。
- **竖排为长尾**:V1 已落 TTB/BTT、朝向枚举、`vert`/`vrt2`、原生 `vmtx`/vertical origin、标点与右到左多列；纵中横(tcy)、ruby 与复杂双向竖排仍明确留在 V2。
- **IME 平台实机**:winit 事件为基线,平台特化(TSF/IBus)落 `zircon_app` 平台层,运行时只持中立 `ime_context` 契约。

## 10. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Runtime Text 总索引中的状态与验证明细已迁入 Text 09 产出目录。

- 迁入记录：[`09/2026-07-09-index-output-records.md`](09/2026-07-09-index-output-records.md)
