---
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/text_geometry.rs
  - zircon_runtime/src/ui/surface/text_shape.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs
  - zircon_runtime/src/ui/text/layout_engine/candidate_line.rs
  - zircon_runtime/src/ui/text/layout_engine/direction.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/range_mapping.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/cache/mod.rs
  - zircon_runtime/src/text/cache/frame_dedup.rs
  - zircon_runtime/src/text/cache/layout_cache.rs
  - zircon_runtime/src/text/cache/measure_cache.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/text/cache/tests.rs
  - zircon_runtime/src/text/parallel/mod.rs
  - zircon_runtime/src/text/parallel/shape_pool.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/text/parallel/tests.rs
  - zircon_runtime/src/text/layout/mod.rs
  - zircon_runtime/src/text/layout/align.rs
  - zircon_runtime/src/text/layout/overflow.rs
  - zircon_runtime/src/text/layout/tab.rs
  - zircon_runtime/src/text/layout/line_break/mod.rs
  - zircon_runtime/src/text/layout/line_break/tests.rs
  - zircon_runtime/src/text/layout/line_break/glue.rs
  - zircon_runtime/src/text/layout/line_break/glyph_fallback.rs
  - zircon_runtime/src/text/layout/line_break/smart.rs
  - zircon_runtime/src/text/layout/line_break/soft_hyphen.rs
  - zircon_runtime/src/text/layout/line_break/wrap_space.rs
  - zircon_runtime/src/text/layout/kinsoku.rs
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/text/layout/rich_source.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/font_dependencies.rs
  - zircon_runtime/src/ui/tests/text_pipeline
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/text/raster/mod.rs
  - zircon_runtime/src/text/raster/policy.rs
  - zircon_runtime/src/text/raster/swash/mod.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui/font_admission.rs
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
  - zircon_runtime/src/text/font/mod.rs
  - zircon_runtime/src/text/font/runtime_asset.rs
  - zircon_runtime/src/text/atlas/mod.rs
  - zircon_runtime/src/text/atlas/bitmap_run.rs
  - zircon_runtime/src/text/atlas/bitmap_run/allocation.rs
  - zircon_runtime/src/text/atlas/bitmap_run/failure.rs
  - zircon_runtime/src/text/atlas/bitmap_run/placeholder.rs
  - zircon_runtime/src/text/atlas/bitmap_run/retry.rs
  - zircon_runtime/src/text/atlas/bitmap_run/staged_upload.rs
  - zircon_runtime/src/text/atlas/bitmap_run/staging.rs
  - zircon_runtime/src/text/atlas/bitmap_run/tests.rs
  - zircon_runtime/src/text/atlas/bitmap_run/types.rs
  - zircon_runtime/src/text/atlas/bitmap_run/upload.rs
  - zircon_runtime/src/text/atlas/bitmap_run/validation.rs
  - zircon_runtime/src/text/atlas/render_contract.rs
  - zircon_runtime/src/text/atlas/render_contract/tests.rs
  - zircon_runtime/src/text/atlas/render_plan.rs
  - zircon_runtime/src/text/atlas/render_plan/tests.rs
  - zircon_runtime/src/text/atlas/render_batch.rs
  - zircon_runtime/src/text/atlas/render_batch/tests.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan.rs
  - zircon_runtime/src/text/atlas/render_submission.rs
  - zircon_runtime/src/text/atlas/render_submission/frame_driver.rs
  - zircon_runtime/src/text/atlas/render_submission/frame_state.rs
  - zircon_runtime/src/text/atlas/render_submission/tests.rs
  - zircon_runtime/src/text/atlas/shaders/glyph_atlas_sampling.wgsl
  - zircon_runtime/src/text/atlas/page_residency.rs
  - zircon_runtime/src/text/atlas/page_residency/tests.rs
  - zircon_runtime/src/text/atlas/upload.rs
  - zircon_runtime/src/text/atlas/upload/tests.rs
  - zircon_runtime/src/text/atlas/dirty.rs
  - zircon_runtime/src/text/atlas/dirty/tests.rs
  - zircon_runtime/src/text/atlas/raster_key/mod.rs
  - zircon_runtime/src/text/atlas/raster_key/tests.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/fallback.rs
  - zircon_runtime/src/text/font/fallback/tests.rs
  - zircon_runtime/src/text/font/coverage.rs
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/handoff.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/resource.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_advances.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/draw_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/shader_contract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/layout_placement.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/zr_text_sdf.wgsl
  - docs/zircon_runtime/graphics/text.md
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs
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
  - zircon_runtime/src/ui/surface/render/dialog.rs
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
  - zircon_runtime/src/text/shaping/mod.rs
  - zircon_runtime/src/text/shaping/cosmic.rs
  - zircon_runtime/src/text/shaping/fallback_spans.rs
  - zircon_runtime/src/text/shaping/script_segment.rs
  - zircon_runtime/src/text/shaping/line_break.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/text/font/backend.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/core/framework/text/layout_service.rs
  - zircon_runtime/src/text/model/font/
  - zircon_runtime/src/text/layout/line_break/greedy.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/text/font/descriptors.rs
  - zircon_runtime/src/text/font/matching.rs
  - zircon_runtime/src/text/font/default_families.rs
  - zircon_runtime/src/text/font/asset_registration.rs
  - zircon_runtime/src/ui/text/layout_engine/vertical.rs
  - zircon_runtime/src/text/sdf/params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
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

`render/14`(2D 栈)的 `TD-M1` 已经决定"把文本 shaping/字形图集**下沉为 `text/` 共享服务**,UI 与场景 2D 共用"。但 `render/14` 的篇幅集中在**2D 场景渲染器、sprite 批集成、UI 文本路径硬切换**,并未展开文本子系统**内部**的高精度细节(BIDI/竖排/MSDF/字体回退/富文本/IME/多线程)。`editor_layout/17` 是**编辑器侧排版规范**(度量=绘制、DPI 重栅格、换行自适应),`editor_ui/03` 是**编辑器文本栈定稿**(主链贯通)。三者都**消费**一个尚未深描的运行时文本服务。

本目录就是那个服务的实现权威:**`text/**` 共享服务内部 + `core/framework/text/**` 契约深化**,把用户要求的 15 项能力(glyphon、SDF/MSDF 动态与预生成、UE 风格度量算法、多语言 BIDI/竖排、换行规则、渲染规则、分辨率精度、字体文件处理、图集化、多线程、Unicode、FontFace、回退规则、富文本、多平台 IME、字体回退)逐项落到文件级实施权威。

## 1. 边界与归属(与三份既有计划的勾稽)

固定分工,**不重叠、不矛盾**:

| 计划 | 拥有 | 与本目录关系 |
|------|------|------------|
| 本目录 `text/**` | 文本服务内部:字体库/FontFace/回退、shaping/Unicode/BIDI/竖排、换行/度量算法、栅格/图集、SDF/MSDF、富文本预处理、IME 接口、多线程与缓存 | **实现权威**;`text/**` + `core/framework/text/**` |
| `render/14`(2D 栈) | 场景 `TextRenderer` 组件、glyph quad→sprite 批、UI 文本路径**硬切换**装配、2D 排序 | `TD-M1` 的"共享服务内部"**委托本目录**;`render/14` 持有"如何把 `ShapedGlyphRun` 变成场景顶点/批" |
| `editor_layout/17` | 编辑器排版**规范**:度量=绘制四规则、字形随 `scale_factor` 重栅格、换行自适应两阶段、shrink-to-fit | **消费方**:本目录服务必须满足其度量一致性与 DPI 重栅格契约(本目录 §6.2/04) |
| `editor_ui/03` | 编辑器文本栈**贯通**:Label/Field/Console/树表同一链、CJK 一等公民、编辑链与候选窗实机 | **消费方**:其"shaping 权威未定/栅格策略未书面化/字体注册表缺失/IME 不闭环"四缺口由本目录 01/02/04/06/08 正面补齐 |
| `runtime/15`(结构规范) | owner-module 模式、命名前缀、`module_convention_gate`/`large_file_ownership_gate` | 本目录所有新增文件遵其结构规则;`ui/text` 巨型文件拆分纳入其治理 |

**契约名权威**:`render/14` 已定稿契约层类型(`ShapedGlyphRun`/`ShapedGlyph`/`ShapedLine`/`TextShapingService`/`TextStyle`/`ShapedTextCacheKey`/`GlyphAtlasFormat`/`GlyphAtlasRef`/`RenderTextSnapshot`,见 `render/14` §核心类型与接口)。本目录**沿用并扩展**这些类型,不另造同义类型;扩展项(变量字体轴、竖排朝向、富文本 span、回退命中 face)以本目录各子计划"工程落地细化"为准,扩展后回填 `render/14` 契约定义。

**契约名勘误注(2026-07-02 评审收口；2026-08-27术语硬切同步)**:按代码核实,上述契约名与实际落地存在如下偏差,以本注为准:`ShapedLine` 当前落地名为 `ShapedHardLine`(`text/model/shaped_run.rs`)，只表示shaping阶段hard-line投影，不是visual/layout line；`GlyphAtlasRef`/`RenderTextSnapshot`/`ShapedTextCacheKey`/`TextStyle` 尚未落地,属待建契约;`GlyphAtlasFormat` 已落地但位于实现层 `text/atlas/page.rs` 且为 `pub(crate)`,尚未提升到契约层;另有已落地但 `render/14` 未收录的契约名:`TextShapeRequest`/`TextOrientation`/`VerticalMode`/`ShapedGlyphScript`,需回填 `render/14` 契约定义。

## 2. 现状评审(按代码核实,2026-06-27)

### 2.1 已成立(取其能力,不推倒)

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

已成立能力的详细明细已迁入 Text 09 产出目录；此处只保留当前能力概述。

- 2026-08-30 rich source contract validation：`RichTextLayoutSource` 统一由
  `validate_rich_text_layout_source` 做 run identity、checked range 与 UTF-8 admission；
  `RichAdvanceIndex::source_spans` 改为 typed `Result` owner，append 阶段保留
  `TextShapingOutcome::Deferred` generation retry，合法 source gap 继续由 base style 填充。
  measurement shaped geometry 与 VerticalRl rich columns 同步 fail closed，table/source-slice
  复用 checked owner。静态文本契约 13/13、文本相关静态回归 18/18、定向 rustfmt/diff-check 通过；
  受管 Cargo check 因共享 target 上既存 cargo/rustc 活动 184s 无输出超时，未生成 PNG，WGPU、
  profile/RSS/power 和 Unreal 同负载验证仍待后续里程碑。状态：
  `rich_source_contract_fail_closed_static_implemented / rich_advance_index_result_owner_preserves_deferred /
  base_style_gap_fill_preserved / managed_validation_pending`。

  同一切片又关闭横排/VerticalRl forced-line range 的 sentinel 恢复：所有 forced、chunk、trim、
  fallback source range 统一经过 `rich_source.rs::checked_source_range`，转换失败返回 typed
  outcome。静态 rich-source 契约 13/13 与文本相关回归 18/18 继续通过；未生成截图，受管 Cargo、
  WGPU、profile/RSS/power 与 Unreal 对照仍待验收。

  第二次使用 E 盘独立 target 的 `zircon_runtime` `cargo check --lib --offline --locked` 也在 184s
  无诊断超时，工作区编译进程保持运行且未被终止；该结果仅记录为受管编译未完成，不改变静态实现状态。

  本轮第三次使用 `target/codex_text_check_final` 的 E 盘独立 target 运行同一检查，304s
  无诊断后达到有界超时；其 rustc 仍在后台运行并未被终止，结果仍仅记为受管验证未完成。
  随后的增量输出记录了既存工作区错误（Rust 2024 let-chain、缺失 `zr_contracts`、图形导出/可见性
  冲突及其他 text font/glyph 模块），未指向本切片 rich source/layout 文件；因此不能把 Cargo gate
  标记为通过。

  修复 text 基础设施后再次增量检查，`text/glyph_artifact`、`text/font`、`text/sdf/font_bake`、
  `text/shaping/cosmic` 已无诊断；剩余错误收敛在未触碰的 graphics/core/plugin/dynamic-api 模块。
  因此本切片 text-module 编译诊断已关闭，但 workspace Cargo gate 仍因无关模块保持失败。
  新增的 text infrastructure compile-contract 静态回归为 6/6，覆盖 glyph projection owner、
  fallback hash 分支、font-family dedupe 类型、SDF face-cache 调用形状、default-face 可见性和
  Cosmic snapshot clone/revision 访问。

  forced range 生成又改为 `hard_line_count` 预分配加 `visit_hard_lines` 填充，避免中间
  `HardLine` 列表；validated source pass 与 checked u32 publication owner 复用同一边界，
  静态 rich-source 契约 13/13、文本回归 18/18 全部通过；text-module compile diagnostics 已关闭，
  workspace 级 Cargo gate 仍待无关模块修复。

2026-08-30 rich bidi-control authoring/trust gate：四种 versioned rich format 现在把 raw scalar、
HTML numeric entity 与 BBCode literal control 归一到 source-ranged bounded diagnostic；mark、embedding、
override、isolate 使用四个稳定 code，compiled logical text 保持不变。默认 untrusted 只允许 mark 与
balanced isolate，legacy embedding/pop/override 需要显式 trusted authoring 且仍须平衡；trust 已进入
cache/compiled identity，专用显式栈深度上限为 125。状态：
`RRT-P1-041_trust_gate_and_balanced_isolation_static_complete /
managed_copy_a11y_render_and_profile_pending`；静态集合 38/38（0.090 s），managed Rust/copy/a11y/render/profile
待办。详见
[`07/2026-08-30-rich-bidi-control-authoring-diagnostics.md`](07/2026-08-30-rich-bidi-control-authoring-diagnostics.md)。

- 迁入记录：[`../../_archive/zircon_runtime/text/09/2026-07-09-index-output-records.md`](../../_archive/zircon_runtime/text/09/2026-07-09-index-output-records.md)

2026-08-30 rich source contract guard：`RichTextLayoutSource` 的 rich index、glyph artifact 和 UI
prewarm 消费入口统一校验 run 是否可取、父级 source index 是否为非哨兵且严格递增、范围是否单调不重叠、
是否位于 source 文本的合法 UTF-8 边界，并保留合法空洞由 base style 补齐的既有语义。结构损坏统一
fail closed 为 `LayoutFailed`，不再由 rich materializer 静默跳过缺失 run/范围后发布空几何。source-contract
回归覆盖空文本、部分覆盖、重叠、空范围、越界及 source index 错误；rich 横排 glyph/word 与 VerticalRl word
范围提取也会对非法 hard-line slice 返回 `LayoutFailed`；rich UI item projection 对坏 run identity/range
同样 fail closed，VerticalRl item projection 也不再以 `Ready(None)` 吞掉坏的 source range；本切片不改变正常换行或测量算法。
rich table layout 同时对绝对 table/cell range 做 checked arithmetic、父表包含和 UTF-8 边界校验，合法空
cell 保留，同一 table 内 cell range 按 source 顺序非重叠，坏 range 不再被 `min/max` 夹成空 cell；表格
前后段 source slicing 拒绝反向、越界和非 UTF-8 边界，并复用 canonical hard-line separator owner。
`UiParsedText::project_range` 作为唯一 checked projection owner 返回 typed `Result`，适配器只传播失败。状态：
`rich_source_contract_fail_closed_static_implemented / base_style_gap_fill_preserved /
managed_validation_pending`。

2026-08-30 布局边界 fail-closed：line-break hard-line/glyph source range 与 UI glyph-wrap 的 metric
slice 不再静默 `continue`。非法 UTF-8 边界、越界或非单调 cluster 统一返回 typed
`Failed(LayoutFailed/BidiInvariant)`；合法零宽 virtual anchor 保留。静态架构与 malformed-metric 回归
通过 18/18，生产稳定顺序仅做 O(B) 检测，视觉顺序异常时才做 O(B log B) 归一化；Cargo 两次尝试均在 E 盘依赖 target 写入阶段失败，WGPU/PNG、profile/RSS/power 仍待受管执行。状态：
`range_invariant_fail_closed_static_implemented / partial_layout_suppression_removed /
managed_product_validation_pending`。

2026-08-29 NumberField MVP 状态：`Float value` 与活动 `String value_text` 已分离，input/render 共用
`number_edit_active` authority；默认编辑不逐键发布 typed value，显式策略只对完整 finite/range-valid
文本发布 Float Change。Enter/blur/Escape 复用同一 parse/clamp/optional-snap transaction，typed Commit
不再携带 String；128-byte buffer 上限与 `TooLong` receipt 已落地。对齐 Unreal 外层 SpinBox 路由的
Up/Down canonical step 在通用光标动作前执行，成功规范化并退出编辑态，坏 policy 零写入；versioned
diagnostics receipt 无正文。源码回归和静态格式/diff 检查已加入，managed
Cargo、平台 IME/a11y/clipboard、WGPU/PNG、profile/power、locale type interface 的动态验收和 format
cache 仍开放；focused numeric refresh 的 revision-qualified Float gateway 已完成静态实现但未通过
managed Runtime。model key 现要求 finite TOML Float 和完整 revision/edit authority，并按 owner
insertion incarnation 精确失效，避免 retained node pool 复用同一 `UiNodeId` 造成 ABA，同时不让
无关节点增删破坏稳定 binding key；Text08 不关闭。

2026-08-29 Text Core/font admission 状态：`TextModule` 作为独立 Core Services module 注册
`FontCollectionService`；Graphics、动态 Runtime UI surface 与 screen-space renderer 通过同一 Core
collection 解析。动态 UI 加载改为两阶段：先构建 retained surface 并汇总 `font` asset 依赖，再在
首次 `compute_layout` 前统一 admission/publish，随后布局与渲染准备均消费相同 `(collection_id,
generation)`。renderer fallback 不再在 plan 阶段调用进程级布局服务，raw batch 仅在 font dependency
ready 后由 renderer-owned collection canonical shape；`UiSurface::compute_layout` 已接通未使用的
font-generation invalidation hook。该切片仅完成静态实现、rustfmt 与 scoped diff 检查；managed Cargo、
WGPU/PNG、window/PIE 注入、profile/RSS/power 与 Unreal 对拍仍开放，不能据此宣称产品渲染验收。

该处历史 owner-lifecycle 门禁已由 2026-08-29 的 `RuntimeFontAssetClaimScope` 静态实现关闭：
`FontCollectionService` 统一管理 scope claim/release 与代际发布，renderer 和动态 Runtime UI 共享聚合
认领账本。对应实现与仍开放的 project switch/hot reload、fallback 隔离、驻留上限及稳定帧证据见本页
后续 FontObject session 生命周期记录；不得再把“缺少 lease”作为当前缺口重复规划。

| 能力 | 当前概述 |
|------|----------|
| glyphon bitmap atlas 绘制后端 | 真实 runtime WGPU 产品帧缓冲已验证 CJK、RTL 与彩色 Emoji；后端实际 color face→`SwashContent::Color`→RGBA 字节合同已直证，完整 atlas 硬切仍由 Text 04 跟踪；Core collection/首帧 admission 静态实现已接线，当前源码产品帧待受管重跑。 |
| fontsdf SDF 烘焙 + 图集 + 渲染 | 真实 runtime WGPU 产品帧已验证横排 SDF 与两列 VerticalRl CJK 标点；系统 face 经共享 `FontDatabase` 物化后，实际 shaped glyph id/face id 直接进入 atlas key 与 indexed fontsdf bake，backend vertical origin/advance/rotation 由生产 quad 消费。MSDF/MTSDF、horizontal shaped-SDF 全链与 native/SDF parity 仍由 Text 05 跟踪。 |
| 启发式布局/换行/对齐/省略 | canonical layout/cache 已消费显式 FontCollection；动态 UI 首次布局前完成字体 admission，后续实现与验证记录见对应子计划。 |
| Unicode BIDI / 竖排 shaping | UAX#9 paragraph/line owner、TTB/BTT vertical backend、Vertical_Orientation、`vert`/`vrt2`、native `vmtx` 与 vertical origin 已落地；horizontal RustyBuzz leaf 已接入 per-run `locl` 与变量轴，真实 Windows exact 待运行，更完整黄金 corpus 仍由 Text 02 跟踪。 |
| grapheme 边界/导航 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| 命中测试 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| 度量缓存 | 已有基础能力，后续实现与验证记录见对应子计划。 |
| 字体回退链 | 后端实际 face-ID 与 shared/native CompositeFont span 已硬切；bounded 缺字诊断/partial cluster 已进入帧外报告，真实 CJK/Arabic/Hebrew/彩色 Emoji、zh-Hans/ja 同码点产品帧与 Arabic mark complex-cluster 单实际 face 均已验收；per-run `locl` 实现已由 Text 02 horizontal RustyBuzz leaf 承接，focused exact 待运行。 |
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

**隔离硬规则**:`cosmic_text`/`fontsdf`/`fdsm`/`fontdb`/`swash` 等第三方类型**只允许**出现在 `zircon_runtime/src/text/` 实现域的指定隔离文件内(见各子计划),出口一律使用 `zircon_runtime::text` 的实现 DTO 或 `core/framework/text` 的中立服务契约,不向 UI/graphics 暴露第三方句柄。退役的 `graphics/text/**`、`core/framework/render/text/**` 以及 `zircon_editor`/`zircon_app` 不得直接 import 上述库。

**现状违例记录(2026-07-02)**:按代码核实,当前存在以下隔离硬规则违例——`zircon_editor` 直接依赖并 import `fontdb`/`fontdue`/`swash`(`ui/retained_host/host_contract/paint_text/{font,raster}.rs`);`rhi_wgpu/ui_surface/text.rs` 直接 import `glyphon`;`cosmic-text` 实为经 glyphon re-export 使用而非直接依赖;`fdsm` 尚未引入。这些违例的收束路径:editor retained-host 路径迁移到 runtime 文本服务后删除直依;收束前该违例冻结不扩大(不得新增直接 import 点)。


> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Text 总索引中 2026-07 的 editor retained-host、runtime text cache、native atlas 与验证证据明细已迁入 Text 09 产出目录；此处只保留选型、边界与全局工程约定。

- 迁入记录：[`../../_archive/zircon_runtime/text/09/2026-07-09-index-output-records.md`](../../_archive/zircon_runtime/text/09/2026-07-09-index-output-records.md)

## 6. 全局工程约定(各子计划"工程落地细化"共享,不重定)

1. **模块归属**:中立服务契约层为 `zircon_runtime::core::framework::text`；完整实现层为 `zircon_runtime::text`(共享服务,持模型/字体/shaping/layout/raster/SDF/atlas/cache/parallel/rich owner)；场景/UI renderer 仅在 `graphics/scene/scene_renderer/ui` 消费；UI 消费方 `ui/text` 负责 widget/layout/hit-test 适配。`graphics/text` 与 `core/framework/render/text` 已于 Frameworks05 M3 hard cut 删除，不保留兼容模块；后续由 Frameworks01 M3 抽取 `zr_text`，当前不新增中间 crate。
2. **度量=绘制(继承 `editor_layout/17` 四规则)**:任何布局几何(advance/kerning/ascent/descent/换行点)必须来自 shaping 服务的真实字形度量,与绘制端**同一来源**。禁止任何路径回退到等宽近似。
3. **字形随 DPI 重栅格**:栅格输入 `physical_px = logical_px × scale_factor`;图集 key 含 `scale_factor` 量化桶(接 `editor_layout/17 §3.4`、`render/14`)。SDF/MSDF 因分辨率无关,bake 尺寸固定、运行时按 `font_size` 缩放采样。
4. **缓存键不持引用**:shaped run / measure / atlas 的缓存键一律用 `font_id + size_bits(f32::to_bits) + features_hash + EphemeralCacheHash`,不持文本/字体对象引用(改造自 UE `FCachedShapedTextKey`,见 `render/14` §核心类型)。该hash只能进程内选桶，不可序列化或写入artifact/replay；features_hash 的来源见 02 `TextShapeRequest.features`(规范化=按 tag 排序后 hash)(2026-07-02 评审收口)。language 先由 `text/language.rs` 唯一 owner 验证并 canonicalize，cache 只做 exact identity，不得自建 lowercase/separator policy。命中后必须等值比较文本副本防 hash 碰撞(缓存值持 `Arc<str>` 副本 verify,见 D6/09)；持久内容身份使用格式/domain版本限定的`StableContentDigest`(2026-08-26 评审收口)。
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
| Unicode 支持(2026-08-26 compiled provider snapshot 基础设施) | 02 | 十二个 locale/Unicode provider role 的独立实现/数据版本已形成 16-byte request-bound identity；Word/Grapheme 与 Emoji/GeneralCategory 虽分别共享当前实现包，仍以独立能力进入 schema v4/generation 4 指纹，JoiningType 也作为独立能力进入身份。identity 已进入 script/emoji/word analysis、WordSmart、Bidi paragraph/line、line-break map、shaped artifact/cache 与 diagnostic；dynamic generation、完整 layout/document artifact 贯通、corpus 与产品验收仍开放 |
| Common/Inherited script-run 策略(2026-08-26 current-source 校准) | 02、09 | paragraph owner 的 `ScriptExtension` 定长位集已直接表达中性字符策略：前导归首个 specific script，行内/尾随归前一个兼容 run，纯 Common 保持 `Zyyy`，paired bracket 由同一分析栈覆盖。旧 `pending_common_start/end` 问题项已失效；聚焦回归已补，生产算法与复杂度未改，managed text test 待验收 |
| Arabic Joining_Type / Kashida safety(2026-08-26 非验收实现) | 02、03、09 | `text/joining_type.rs` 以单一 ICU4X compiled trie 替换手写 Arabic ranges；`arabic_justification.rs` 再以同一次 candidate shape 验证独立非零 Tatweel cluster、正 advance、RTL 邻接及左右同 face/instance，unsafe candidate fail closed。行级低基数 profile 已聚合 requested/probe/bytes/safe/accepted/rejection，普通 build 为零字段；font/language justification API、32-candidate/5-probe 的 31-sample 数据与真实产品渲染仍开放，`RTS-P1-036` 未关闭 |
| shaping failure receipt 与 horizontal hybrid composition(2026-08-27 非验收实现) | 02、06、09 | request owner发布14-code receipt：12个direct/backend cause加missing primary与generation changed；Bidi/source/itemization/font-budget fail closed。horizontal direct保留成功segments与有序holes，一次whole Cosmic candidate仅在identity/topology/source/cluster/coverage资格成立时填洞，否则完整回退；hybrid artifact保留absolute alternate ranges/首因receipt。组合为`O(lines+holes+glyphs)`，managed fault/profile/power/WGPU/PNG待验收 |
| direct backend typed failure(2026-08-26 current-source 校准) | 02 | horizontal/vertical direct 与 RustyBuzz adapter 已统一返回 typed Result；font access/index/parse、empty glyph、source/cluster/metric invariant 进入 12-code receipt。alternate backend 只由 horizontal capability receipt 决定，vertical 与 source/BiDi/budget fail closed；canonical language + RustyBuzz 0.20.1 非空接受域证明 optional projection 不擦除可达失败。managed fault injection/corpus 仍开放 |
| cluster break safety(2026-08-26 typed provenance 基础设施) | 02、03、09 | direct horizontal/vertical 从 RustyBuzz 发布 cluster-head `Safe/RequiresReshape`，Cosmic/旧 artifact 为 `Unknown`；receipt 已保留到 measured cluster 与 advance index，并以单调 `O(boundaries+clusters)` owner 聚合实际候选边界的 safe/requires/unknown 分布。该值不删除 UAX#14 break；final-line exact two-sided reshape/context plan、managed corpus/profile、shape-call 与性能/功耗验证仍开放 |
| vertical Tr substitution comparison(2026-08-26 pre-optimization receipt) | 02、09 | `TransformOrRotate` 仍以 enabled/disabled `vert/vrt2` glyph-sequence 差分发布可证明的 cluster decision；request-local profile 将 comparison call/input bytes/output glyphs/changed clusters 与总 backend call 分开聚合，普通 build 不观测。managed Tr 1/100/1k/10k 31-sample counter/timing/RSS/功耗前不改算法，`RTS-P1-019/020` 保持开放 |
| vertical cluster decision(2026-08-26 typed receipt) | 02、09 | cluster head 统一保留 Unicode orientation、effective `vert/vrt2` set、substitution proof 与 typed fallback；完整 shaped/neutral accessor 组合已有 rotation 和 selected face/instance，不复制字体身份。direct Tr 有 observed/not-observed proof，compat Tr 显式 provenance unavailable；不新增 shape call/查询/分配，managed size/RSS/profile/power/WGPU/PNG 待验收 |
| shaped source lifetime(2026-08-26 pre-optimization receipt) | 02、09 | current run 仍持 exact `Arc<str>`；同步无 owner 会分配，parallel hard-line split 当前每段独立 owner。已发布 materialization/reuse/allocation bytes 与 batch lease/unique-owner bytes 低基数回执；Unreal sequence 的外部文本 owner + range/index map 作为目标参考。managed 1/100/1k/10k cold/warm/edit/hybrid profile 前不实施 document snapshot/range lease，glyph SoA 独立评估 |
| ephemeral cache hash / stable artifact digest(2026-08-26 typed boundary) | 02、05、09 | shaped/rich/measure/layout/physical-line/document revision lookup统一不可序列化`EphemeralCacheHash`并保留完整key+exact source复核；`.zsdf` generation/offline identity统一`StableContentDigest`，BLAKE3字节与v1格式不变。禁止每viewport全文稳定hash及临时hash跨artifact/replay边界；managed Cargo/golden/collision/profile仍开放 |
| paragraph/hard-line/shaped/layout lifetime(2026-08-26 architecture audit) | 02、03、09 | session 与 shaped/hard-line/layout cache 已分层；未证实 direct→Cosmic fallback、rich advance-index、physical-line/viewport 投影的重复分析是主瓶颈。先做 plain/rich × fallback × cold/warm × 1/100/1k/10k + scroll/edit 31-sample 量化，再决定 document-revision-owned paragraph artifact；dirty-range、source lease、glyph SoA、renderer artifact 不捆绑 |
| stable text layout diagnostic code/catalog(2026-08-26 non-validation implementation) | 02、09 | `TextLayoutError`逐 variant 发布`ZR-TEXT-LAYOUT-*`稳定code与`text.layout.*`catalog key；`Display`仅作人类可读投影，Editor/telemetry禁止解析英文字符串。core仍不依赖Runtime Text face/range receipt；focused behavior test与rustfmt完成，managed Cargo/integration仍开放 |
| UI shaper facade hard cut(2026-08-26 non-validation implementation) | 02、09 | 删除无backend set/policy/receipt的单成员`UiTextShaperStack`；public/provider/viewport/measure/source-range入口直接消费唯一`UiSharedTextShaper`，真实direct/Cosmic组合保持在Runtime Text shaping owner。source guard拒绝旧wrapper恢复，managed Cargo/integration仍开放 |
| serializable DTO / renderer batch residency(2026-08-26 pre-migration receipt) | 02、09 | layout cache已统计serializable line/run/advance owned bytes；prepare report在Auto路由后对最终native/SDF batch各计一次count/text bytes/advance bytes下界，不记录raw text。`UiTextPaint`中间复制与真实serialization materialization仍开放，managed profile证明主导前不切String/Arc/range/lease |
| Runtime Text owner-local budget snapshot(2026-08-27 非验收基础设施) | 02、04、05、09 | 8-grapheme correctness context、32 tatweel/5 measurement单行工作量、16-entry/32 MiB hard-line cache与SDF/page-shadow内存预算由各自owner发布不可变快照，统一只读投影到`text.runtime_budget.*`；page-shadow补resident/max/rejection收据，同commit已知拒绝不重复admission。默认值与算法未改，managed规模/profile/power/WGPU/PNG前不调参 |
| SDF atlas iterator-owner cutover(2026-08-28 编译修正) | 04、09 | retained segment、cache discard与standalone plan统一直接消费`collect_sdf_atlas_text_keys_iter`；旧slice helper保持`cfg(test)`，不恢复production facade或flattened batch临时vector。结构守卫与静态检查通过；受管Runtime仍有其它共享错误，WGPU/PNG未重新验收 |
| session-owned text diagnostics(2026-08-27 非验收基础设施) | 02、09 | layout fallback/shaping failure两个process-global Mutex owner已删除；retained/operation-local session逐帧持有固定code与direct/whole-alternate/hybrid/deferred/terminal route及font-resolution work计数，parallel prewarm完成项合并回同一owner，cache hit不冒充backend work。35个profile名不含raw text/document id/dynamic label；layout-resolve总66项低于focused 128容量，完整集成capture为160，具体document drill-down等待统一有界owner，managed Cargo/fault/profile/power/WGPU/PNG仍开放 |
| shaping hard-line/range terminology(2026-08-27 源码硬切) | 02、09 | shaping阶段容器统一为`ShapedHardLine`，provider/session入口统一为horizontal/vertical `shape_*_range(_with_kerning)`；最终visual line继续由`CandidateLine`/`UiResolvedTextLine`拥有。41个Rust文件无alias/shim迁移，serde字段、request/budget/cache/backend/layout算法不变；旧Rust符号0命中，managed Cargo/serde golden/WGPU/PNG仍开放 |
| font capability non-ready causes(2026-08-27 非验收实现) | 02、06、09 | missing primary发布terminal `FontPrimaryUnavailable`；generation retry/stale cache/stale worker发布deferred `FontGenerationChanged`，session/parallel不再记成terminal。同步resolver已有request-owned固定聚合，区分cache hit-miss、真实coverage probe、candidate visit/reject、选择分类及generation attempt/restart；不进入glyph cache artifact。公开service仍为neutral error；exact candidate/face、pending、collection generation、policy/budget和backend组合trace继续开放，managed验证待执行 |
| shape-request analysis construction profile(2026-08-27 pre-optimization) | 02、09 | canonical shape owner以profiling-only TLS记录request count/bytes，以及Bidi、script/emoji、line-break各自build count/input bytes/nanos，共11个固定名称；inactive/normal build无`Instant`。current horizontal whole-alternate可观测到第二次line-break构造，但dynamic 31-sample route/profile前不hoist、不建立retained paragraph artifact，`RTS-P1-024`保持开放 |
| line-break profile/opportunity receipt(2026-08-26 非验收基础设施) | 02、03 | run-level Unicode snapshot 已固定 line-break provider/data version；cluster-head 以 2-byte receipt 区分 Unicode default allowed/mandatory、mandatory control 与 unknown。locale tailoring、UAX rule number、官方 corpus、动态验证仍开放 |
| soft-hyphen virtual fragment hard cut(2026-08-26 非验收实现) | 03、09 | typed decision 已发布 consumed range/marker mode/zero-width anchor；Plain、horizontal rich 与 VerticalRl rich 均由 retained logical-display sidecar 生成 canonical virtual glyph artifact。rich 按 cluster source mapping 恢复 style span，typed `DiscretionaryHyphen` role 进入 rebuild identity，被消费 U+00AD 保留在 replacement receipt，accessibility 保持原始 source value。capture 已从 per-grapheme receipt `.find` 收敛为 `O(clusters + virtual receipts + external ranges)` 单调游标；managed 动态验证与产品证据仍开放，`RTS-P1-031` 未关闭 |
| rich immutable glyph artifact(2026-08-26 非验收实现) | 03、07、09 | private composite handle 同时强持有 compiled metadata、style-aware glyph artifact、精确 layout-line snapshot 与 run slice directory；整行 glyph 只存一份，renderer run 借用切片，ligature continuation/intentional fallback 有显式空或 negative receipt。horizontal/VerticalRl soft-hyphen、text-only ellipsis virtual glyph 与 compiled inline external block 已接入；inline 普通行/省略行/inline-only 行不再把 U+FFFC 当字体 glyph，paint run 发布显式空 slice。ordinary styled VerticalRl、inline external block、U+2026 ellipsis 与 typed discretionary hyphen 已接入 canonical vertical provider；不支持或 role/text/source receipt 不匹配的 marker fail closed。renderer run publication 已从最坏 `O(R^2)` 双重查找硬切为 `O(lines+runs)` 单调目录投影，并区分 Artifact/VisualOnly/Missing/Stale/Incomplete；Rejected 仅在 exact source-isomorphic 时可 reshape，非同构路径不发布猜测 batch。generated cluster receipt 还保留可选 replaced-source identity，glyph owner 已覆盖 caret/hit-test/selection，accessibility 保持 source-owned。固定尺寸 direct-child widget 已通过 canonical frame 进入普通 child arrange/render/hit-test，renderer placeholder 已删除；artifact 使用 typed owner-local slot，Surface 只在当前树布局期解析 direct child且不保留 binding。desired-size retained session/incarnation lease、managed Cargo/profile/power/WGPU/PNG 仍开放 |
| UI text cache byte residency(2026-08-26 pre-cap 基础设施) | 09 | 4096-entry measure 与 2048-entry layout LRU 已在最低 cache owner 发布 source/DTO-owned heap 的 current/peak 下界，update/evict/trim/clear 同步记账，profile 只暴露四个低基数聚合计数；cache 行为和 entry cap 未改变。共享 glyph artifact、hash bucket/allocator/RSS 仍不在该下界内，必须完成唯一 artifact owner 归因和 managed 规模 profile 后才可设 byte cap/admission/eviction policy，`RTS-P1-045` 未关闭 |
| shaping work-budget receipt(2026-08-26 pre-scheduler 基础设施) | 02、09 | 默认 64 KiB 阈值已在 retained session cache miss 与 parallel unique pending job 记录 inline/oversized-synchronous 请求数、总输入字节与最大请求字节；cache hit、batch duplicate、invalid request 不计费。超阈值请求仍保留完整 source/context 同步执行，不切 line/run/cluster；typed defer/cancel、deadline、CPU/内存预算、managed 规模 profile/power/WGPU/PNG 仍开放，`RTS-P0-001`、`RTS-P1-015/016` 未关闭 |
| EndWord Unicode boundary(2026-08-26 非验收实现) | 02、03 | whitespace rollback 已删除；零拷贝、snapshot-bound `WordBoundaryMap` 成为 layout 与 UI navigation 的唯一 UAX #29 word owner，连字符/CJK/apostrophe 走同一完整词选择。horizontal text-only rich marker 已有 current-run style/canonical glyph owner，并以 single-gap fail-closed receipt 覆盖省略区间 geometry；compiled inline external block 已与 marker 共享 visual geometry owner。locale dictionary、WordBreakTest、VerticalRl 与动态证据仍开放 |
| ligature caret/advance(2026-08-27 partial infrastructure) | 02、03 | `text/cluster_geometry.rs` 是 shaping/renderer artifact 的共享 backend-cluster owner；measurement/index/fragment 保留 typed `AtomicCluster` receipt，plain/rich/UI glyph-wrap 不拆 cluster。canonical artifact 统一 caret/hit/selection；缺失或 stale 时，严格 source-congruent 单一 LTR horizontal 行以一次完整 shape 复用同一 index，caret/hit 只返回 cluster 两端、selection 扩为完整 cluster，editable pointer 复用 command text/style。rich/BiDi/vertical missing-artifact、跨-run continuation、任意 source-range 与 GDEF caret provider 仍开放，`RTS-P1-034` 未关闭 |
| invalid resolved advance geometry(2026-08-27 hard cut) | 03、09 | neutral DTO 只有在“一 visual grapheme 一 finite non-negative advance”成立时使用 exact-prefix geometry；invalid/legacy cardinality、NaN、负值不再等分总行宽。Runtime no-source hit-test 也已删除默认style临时重塑：严格source route复用cluster index，否则按aggregate midpoint只选整行端点；有效tab/BiDi/vertical DTO不变。rich/virtual backend cluster map与managed验证开放，`RTS-P1-012`未关闭 |
| non-empty whitespace layout admission(2026-08-27 MVP hard cut) | 03、09 | render resolution与owner-overlap prewarm不再以`trim().is_empty()`判断布局是否存在；spaces/tab/hard separator保留advance、tab-stop、line box及caret/selection/IME几何，真正空display source仍跳过，editable空source特例与whitespace Justify拒绝不变。managed Cargo、corpus、WGPU/PNG仍开放 |
| stable document hard-line model(2026-08-27 非验收基础设施) | 03、09 | revisioned document现有separator-aware stable line ID；edit仅重扫带前后context的局部line envelope，保留unchanged prefix/suffix ID，split新增ID、merge保留左ID，并发布old/new reanalyzed ordinal span。grapheme index全文重建、Vec suffix move、产品session/reflow、managed profile/WGPU/PNG仍开放 |
| retained document typed no-op edit(2026-08-27 非验收基础设施) | 03、09 | 内部`replace`以跨piece allocation-free range equality返回typed `Unchanged/Changed`；相同bytes不推进revision、不追加addition chunk、不重扫hard line、不失效grapheme index，MAX revision下no-op仍合法。真实late-mismatch额外比较成本、产品gateway/history/rebase、managed profile/WGPU/PNG仍开放 |
| physical-line content/placement geometry(2026-08-29 非验收结构修正) | 03、09 | 保留超宽center/right与Unreal `max(DrawWidth, ViewSize)`一致的origin；`UiResolvedTextLine.frame`硬切为natural content geometry，required `placement_frame`独立承载paragraph/rich-cell line slot。Plain/rich horizontal与VerticalRl、table translation、line selection/content hit、serde和frame extent回归已迁移；每行增加16 B且无新shaping/wrap/allocation/search loop。managed Cargo、cold/warm/profile/allocation/power、产品WGPU/PNG仍开放 |
| surface text layout revision exhaustion(2026-08-27 MVP hard cut) | 03、09 | `UiLayoutCache`不再让layout revision环回旧值；`u64::MAX`作为不可发布的耗尽哨兵，surface两处key构造仅通过`retained_text_layout_revision()`取得identity。耗尽后普通layout、editable与unretained viewport继续工作，仅禁用跨帧retained document reuse；产品`TextDocumentId + Revision` authority、managed Cargo/WGPU/PNG仍开放 |
| typed text-input constraint receipt(2026-08-27 MVP基础设施) | 08、09 | filter、single-line canonical separator移除与max-grapheme截断发布共享低基数`UiTextInputConstraintReceipt`，keyboard/text/IME/a11y不再靠字符串比较推断约束；CRLF计一次，catalog默认`max_length=0`恢复不限长。constrained preedit以requested-boundary UTF-8 edit map迁移cursor/clause并记录adjust/drop；single-line Enter为零属性写的handled Submit，repeat不重复commit。retained prefix/suffix仍全文grapheme计数，平台clause producer、managed Cargo/profile/power/WGPU/PNG仍开放 |
| secure text policy/event projection(2026-08-28 M0前置) | 08、09 | TextField catalog声明typed `input_kind` enum；WOC password/type/secure aliases进入唯一internal policy且畸形/未知fail-closed。secure Change/Submit发布surface-owned latest opaque reference，dispatch/direct-reply统一redact input、binding、effect、host/component report与action payload，lease不随clone/serde传播。文本 Change/Submit 与 focus-loss Commit 已按 matching binding 保留 authored route/action identity，compiled surface 复用事件索引且无 binding 不签 lease；plaintext resolver 已硬切为 Runtime crate-local。dynamic Runtime 已用typed `UiAction` Host Request关闭template-action降为bool的丢失点：256-row、240 KiB aggregate、64 KiB row与depth reserve，secure Change精确合并，拒绝立即撤销lease，Host page rollback可重试；七类非文本服务 `FReply` 风格宿主操作也已进入typed `UiHost`有界队列，IME/clipboard不重复投影，App校验viewport并做内容无关默认消费。binding mutation receipt 的产品版本/consumer/安全策略仍开放。window/seat/surface/route-qualified trusted session、一次性 consume、WOC adapter、retained state/history/export/crash/zeroization、secure IME/platform session、managed WOC/Cargo/IME/capture仍开放，M0/P0未关闭 |
| secure keyboard word-boundary policy(2026-08-28 非验收实现) | 08、09 | 对齐 Unreal password command policy，唯一 keyboard edit owner 显式消费 canonical secure classification；secure Ctrl+Left/Right 与 Ctrl+Backspace/Delete 只使用 hard-line boundary，不再把正文交给 Unicode word-boundary 查询。普通字段保留共享 Unicode word navigation，所有删除仍进入 exact edit/document+Surface transaction 和 secure redaction。回归源码已补；managed Runtime/product input 未通过，不生成非渲染策略截图 |
| secure pending model-value zeroization(2026-08-28 非验收实现) | 08、09 | Surface-owned focused bound-refresh store以`Zeroizing<String>`持有pending明文，supersede/detach/policy change/Surface switch/clear/Drop擦除allocation；accepted transfer以`mem::take`移交既有property/document状态，不增加全文clone。该边界不覆盖request rejection、component/document/history/layout/platform/crash明文owner，不能宣称端到端zeroization；managed Runtime仍待通过 |
| NumberField focused Float model refresh(2026-08-29 MVP非验收实现) | 08、09 | 对齐Unreal `SSpinBox`的typed value/editable String authority：独立numeric model UUID与canonical/edit-base revisions进入共享Float属性事务。Bound refresh立即更新canonical且保留活动buffer，Enter在stale base上报告content-free conflict，blur采纳最新canonical；显式SetValue关闭buffer。model UUID按owner insertion incarnation精确失效，无关topology变化保持稳定；无第二pending queue。五项固定profile counter和产品源码回归已加入，managed Runtime/profile/power/WGPU仍开放 |
| clipboard transfer transaction(2026-08-28 MVP基础设施) | 08、09 | request绑定UUID transfer、Copy/Cut/Paste intent与surface-local edit revision，入站typed read/write/failure result；cut仅在匹配write ack后删除，paste经共享constraint owner应用。每Surface manager使用与ABI host-output同一256-row上限，同owner未送出请求可替换；dynamic DTO携带viewport/surface，App event-loop完成后精确回送。Windows使用真实winit HWND与`CF_UNICODETEXT`，32 KiB UTF-8正文保证最坏JSON扩张低于256 KiB envelope；非Windows typed `Unsupported`。managed系统剪贴板、window/seat/principal/deadline、timeout/fault injection与跨平台backend仍开放 |
| editable text property transaction(2026-08-27 MVP基础设施) | 08、09 | keyboard/text/IME/clipboard、a11y SetValue/Replace/Selection与generic外部正文更新共享固定十项prepare/commit；reserved value property或非法grapheme state零写入，value change一次推进text revision，caret-only不推进。generic派生caret/selection/composition单写fail closed；外部正文变化按Unreal语义保留合法caret或按grapheme clamp并清选择/组合，同值no-op。存储值与显示文本分离，NumberField外部Float不字符串化；a11y旧8–9份binding report收敛为一份且清理composition clauses。Material editable descriptor已移除raw `KeyboardText`，component reducer只消费semantic ValueChanged/Commit/Focus；Search/Field/Source editor共享正文属性推断。focused bound-text policy、数值字段内部编辑解析、产品document authority/history/rebase与managed Cargo/profile/WGPU仍开放 |
| versioned document edit receipt(2026-08-28 M1非验收接线) | 08、09 | 公共`TextEditChange`已删除raw action与完整before/after editable snapshot，改为固定大小、无正文的versioned receipt：document UUID、相邻revision、typed kind/source、old/new byte range与带focus affinity的最终selection；schema、nil identity、revision跳变/耗尽、反向range均typed fail closed。每Surface的manager document session现于双preflight后签发生产receipt，keyboard/text/IME/clipboard/a11y共享该gateway。产品阈值、model-refresh/rebase与managed Runtime仍开放，M1未验收关闭 |
| revision-bound document snapshot lease(2026-08-28 M1基础设施) | 08、09 | 内部piece document按revision惰性持有连续`Arc<str>`：初始复用original，同revision只展平一次，旧lease跨edit稳定，no-op保持pointer identity；lease携带document UUID与typed revision，source index删除临时String二次全文复制，document/lease Debug不泄露正文。Surface session已接入显式snapshot/active-lease byte预算并在detach/identity切换时回收；ASCII/no-CRLF incremental grapheme splice 与无分配 piece preflight 已完成静态实现，Unicode/CRLF fallback、产品阈值标定和managed profile仍开放 |
| retained document storage structural optimization(2026-08-28 已实测) | 09 | 17场景×31样本基线证明独立addition chunk/piece增长与整条hard-line重解析是主导结构热点；实现切到单一append-only addition source与separator-neutral local hard-line edit。同矩阵10k尾插p50 1,710.706→4.508 ms、分配8.127 GB→3.643 MB，百万字符100次尾/中插分别711.913→0.061 ms与799.927→0.034 ms；52/52 direct-source测试绿色。Surface manager session现已接线，WPR sampled stack/功耗因Windows policy受阻，matched Unreal runtime、managed产品路径与WGPU/PNG仍开放，不声称功耗/参考引擎对齐 |
| surface-session document admission store(2026-08-28 M1非验收接线) | 08、09 | document replace拆为零mutation prepare与expected-key commit；store只允许`with_limits`，无Default/全局manager，显式限制document/replacement/retained source/addition source/piece/snapshot/active lease并在commit/flatten前typed拒绝，lease Drop释放预算。`UiInputManager`现按Surface持有session并完成document/Surface双preflight、content-free receipt、topology-gated detach回收与O(1) aggregate report；direct document suite 54/54。产品阈值标定、增量grapheme handles、model rebase与managed Runtime仍开放 |
| exact committed edit intent(2026-08-28 M1非验收接线) | 08、09 | edit owner直接发布exact old/new byte range与kind，replacement借用final state slice；word-delete序列最多一次commit，preedit/caret/selection/cancel/identical replacement为state-only。keyboard/text/IME/delete-surrounding/cut/paste/a11y已贯通manager document gateway；focus-loss改为state-only cancel且不伪造receipt。Surface十属性与document store各自拥有exclusive prepared owner并由薄双commit coordinator提交；delta undo/redo复用同一gateway。产品阈值、model rebase与managed Runtime仍开放 |
| cooked font bytes / runtime default composite baseline(2026-08-29 非验收基础设施) | 01、06、09 | project font importer/cache/runtime registration 已贯通 versioned `FontBlobArtifact`，project runtime 不再重开 source；引擎内嵌 2-face default TTC 并建立独立 runtime primary face/CompositeFont/UI-family 基线，项目覆盖清除后恢复内置 Fira Mono + zh-Hans face。SDF default resolver先尊重项目owner，再直接消费内置primary face。explicit > project > runtime 优先级与双索引重建已落源码，无 per-glyph 新工作；clean package、shipping direct-path policy、Cargo/WGPU/PNG/profile/power 仍开放 |
| clean-process default face admission(2026-08-29 MVP静态修正) | 01、02、06 | 空/失效family query现按explicit -> project default -> runtime primary -> runtime family -> platform/asset fallback解析，系统字体Disabled时可先取得内置primary再进入CompositeFont CJK itemization。默认层变化同步失效face-match/fallback cache；fresh packaged DB的`A界` regression要求neutral handle反查两个真实face。仅cache miss增加bounded lookup，无per-glyph新工作；managed Cargo/WGPU/PNG仍开放 |
| FontObject/Typeface owner-scoped pipeline(2026-08-29 MVP静态实现) | 01、02、03、05、06、09 | `style.font`硬切为资产owner、`font_family`为owner内typeface；primary/composite/fallback/line-metric证书与shaped/fallback cache共享同一owner identity。资产CompositeFont与有序face slice在generation发布时成为Arc索引；owner attach/remove推进generation但保留物理face去重。owner fallback不读取其他资产fallback并集；SDF有效shaped handle继续直接消费face/glyph，只有无效handle恢复才复用同一owner resolver且不跨face复用glyph id。显式unknown owner会清空local family后进入默认链，registered owner保持borrowed query；candidate dedupe保留local-only/external-fallback来源，不因owner缺失同名face而自动全局搜索。请求不再从source索引重复物化owner face集合；managed Cargo、真实shape/raster、WGPU/PNG/profile/RSS/power仍开放 |
| Engine-owned last-resort face(2026-08-29 MVP静态实现) | 01、02、03、05、06、09 | fallback terminal从“任意request primary + glyph 0”硬切到generation-owned packaged Fira Mono face；无face synthetic glyph路径在当前canonical service已为0。last-resort identity进入DB等价/cache失效，line-metric envelope覆盖其extents，SDF按resolved face bytes消费。unknown scalar handle与glyph-0 SDF轮廓回归已落源码；专用全码点LastResort字体、typed ResolvedGlyphStatus、Cargo/Native/SDF/WGPU真实像素仍开放 |
| FontFace | 01 | `FontFace`(单 face)/ `CompositeFont`(family+回退)分层 |
| 回退规则 / 文本字体回退 | 06 | 脚本感知 + Unicode 范围 + 链式 + 深度限 + tofu |
| 富文本(HTML/BBCode) | 07 | BBCode 全集 + HTML 受控子集 + 装饰器 + 内联对象 |
| 多平台 IME 输入法接口 | 08 | TSF/IMM32(Win)、NSTextInputClient(mac)、IBus/fcitx(Linux)、Web |
| letter/word-spacing 与 OT features(2026-07-02 评审收口) | 02、03、07 | `TextShapeRequest.features`(tnum/smcp/liga 等)进 features_hash;spacing 在布局层应用 |
| language/locale 与 Han 消歧(2026-07-02 评审收口；2026-08-26 canonical identity/explicit fallback key 基础设施) | 02、06 | `UiResolvedStyle.language` 段落/run 级字段已进入 layout/shaped/SDF/native fallback；Runtime Text 的 ICU4X owner 一次结构化解析同时产出 canonical tag 与显式 language/script/region fallback key，request 重借用、fallback、direct/Cosmic analysis 共用该 identity。CompositeFont culture selector 只在 index cache miss/font generation 发布时编译，按 Unreal 父文化组合进入 culture-priority bucket，再落 generic/default；cache-hit 前不解析 locale。未写入标签的 likely script/region、版本化 locale data 与完整 fallback decision receipt 仍开放，动态测试尚未执行 |
| 混 face 行度量与 baseline(2026-07-02 评审收口) | 03 | 行 ascent/descent 取行内各 run face 度量 max;baseline 统一 alphabetic(D7) |
| 字体失效级联(2026-07-02 评审收口) | 01、09 | face 失效 → 缓存/图集/SDF bake 级联剔除;09 缓存契约表持"失效来源"列 |
| 文本选择/caret affinity/双击选词(2026-07-02 评审收口) | 03、08 | `CaretAffinity` 模型、软换行行尾归属、grapheme/word 边界导航 |

### 7.1 已识别全局缺口与承接(2026-07-02 评审收口)

以下缺口在本轮评审中识别,登记归属,防止落入无主区:

- **gamma/linear 混合策略**:归 05 渲染规则;V1 观测不动(维持现有混合行为),仅记录对拍差异。
- **native/SDF paragraph parity 闸门**:归 05 新增里程碑 SM-M5;验收=同串同布局两路径 bbox/advance 逐项断言。
- **超长文本分段与虚拟化**:归 09 PF-M5(段落级脏跟踪、可视区增量 shape)。
- **document revision底座(2026-08-28 current-source校准)**:crate-private piece storage、owner+revision、old/new byte dirty span与revision-bound source index已存在；replace强制expected key并在stale/revision exhaustion时零mutation失败。每个产品`RuntimeUiSurface`现通过自身`UiInputManager`接入document session，连续编辑保留UUID/相邻revision并发布content-free receipt；source index失效仍全文snapshot重建，增量grapheme index/paragraph reflow、产品阈值与managed Runtime不得标记完成。
- **editable state 原子投影(2026-08-28 current-source校准)**:surface 的 value/caret/selection/composition 已由逐属性 best-effort 写入收敛为固定十项 prepare/commit；reserved value property 和非法 grapheme state 在写前拒绝，文本值变化只登记一次 layout/text revision，caret-only 保持 render-only。manager product path现把exact range intent、document admission、public receipt与bounded delta history接到该边界；model-refresh/rebase、产品阈值和managed Runtime仍开放，不能标记M1/M2验收完成。
- **external editable value 投影(2026-08-27 current-source校准)**:generic property 入口对可编辑正文已复用同一事务，显示文本变化保留合法caret或按grapheme clamp并清空selection/composition；同值保持no-op。派生编辑属性generic单写已拒绝，NumberField外部Float保留原类型。Material component raw `KeyboardText` 编辑旁路也已删除，reducer只消费semantic event做镜像/validation；完整alias与canonical value-property推断将Search/Field/Source editor送入Surface事务。focused binding conflict/rebase、数值内部编辑parse/commit和产品document authority仍未接入，`RTE-P1-007`只能算关闭当前surface/component bypass，不能整体关闭。
- **focused bound-text policy(2026-08-28 MVP非验收接线)**:对齐 Unreal `RefreshImpl`/`OnBoundTextChanged`，公共 model-update 请求现区分 `BoundRefresh`、`ExplicitSetText`、`ExplicitLoadText` 并携带 expected document UUID/revision。聚焦的 bound refresh 只进入 manager-owned 每 owner 最新 pending，不覆盖 edit buffer；失焦以 expected key 做 CAS，未编辑则经同一 document+Surface 双事务应用，已编辑则发布无正文 `StaleDocument` conflict。显式替换聚焦时立即执行并把 caret 放到末尾；IME preedit 先还原 committed base，避免临时组合串污染 document range/revision。secure pending 正文只放 Surface secure store；manager 只持元数据。generic editable property mutation 保留为显式替换兼容入口，不伪装成 bound refresh。managed Runtime/WGPU/profile/power 与自动三方合并仍开放，状态 `focused_bound_model_update_gateway_implemented_unvalidated / versioned_compare_and_swap_receipt_implemented_unvalidated / secure_pending_surface_owned_unvalidated / managed_acceptance_pending`。
- **NumberField typed edit session(2026-08-29 MVP非验收实现)**:canonical `Float value`、活动 `String value_text` 与 edit-active authority 已分离；共享数值事务覆盖字符/IME/clipboard、Enter/Escape/blur、a11y typed SetValue 与 Unreal-style Up/Down canonical step。独立 numeric model UUID、canonical/edit-base revisions 与 versioned Float gateway 现实现 focused bound refresh：canonical 立即更新但活动 buffer 保留，Enter stale conflict 保留输入，blur 采纳最新 canonical；显式 SetValue 关闭 buffer。model key 要求 finite TOML Float 和完整 revision authority；tree insertion incarnation 变化才失效 UUID，阻止 retained node pool 同 `UiNodeId` 复用旧 CAS key，同时无关 topology 变化保持 binding key 稳定。128-byte hard bound、invariant parser、typed receipt/Commit 与坏 policy 零写入已有源码回归；locale/precision formatter、独立外层/内层 focus target、managed Runtime/profile/power/WGPU 仍开放，不能标记 Text08 完成。
- **Autocomplete canonical query projection(2026-08-27 MVP正确性)**:Surface原有query编辑合同与renderer默认value读取已统一到metadata-level borrowed canonical property resolver；raw/V2 Autocomplete的visible text、editable layout、caret/selection现在都消费query，selected value保持选择模型属性。render不新增属性名分配；动态Cargo仍待验证。
- **versioned document edit receipt(2026-08-28 current-source校准)**:旧公共`UiTextEdit`的raw action与完整before/after snapshot已硬删除，`UiWidgetEvent::TextEditChange`只携带无正文、固定大小的versioned receipt，并要求非nil document identity、严格相邻revision及带affinity的最终byte selection。manager document session现从changed receipt的old/new length和dirty ranges进行不读取正文的O(1) public projection并成为产品Surface producer；snapshot-bound增量grapheme消费、model rebase与managed Runtime仍开放。
- **revision-bound snapshot lease(2026-08-28 M1前置)**:内部piece document现对明确请求的连续source按revision最多展平一次并通过`Arc` lease共享；lease绑定document UUID与typed revision，旧lease跨真实edit稳定，no-op不失效。source index不再为同一次grapheme rebuild复制第二份正文，Debug不发布正文。Surface session已消费显式snapshot/active-lease预算并在owner teardown时回收；全文grapheme扫描仍为`O(N)`，M1不关闭。
- **retained document structural optimization(2026-08-28 measured implementation)**:同一17场景×31样本矩阵已先基线、后实现、再复测。单一append-only addition source使连续range可coalesce；separator-neutral local edit只更新一个stable hard-line model，CR/LF结构编辑仍bounded reparse。10k尾插p50提升379.46x且计数分配降低2,231.08x，百万字符尾/中插不再全文复制；49项production用例加3项结构守卫绿色。content-free report仍不是admission limit；WPR sampled stack/power、matched Unreal runtime、产品接线与WGPU/PNG开放，不据wall time声称功耗对齐。
- **surface-session document admission(2026-08-28 M1非验收接线)**:current product每个`RuntimeUiSurface`独立持有`UiInputManager`，document authority不进入可Clone/serde的Surface或进程全局。manager session现用双preflight提交exact edit，持有显式限额store、content-free receipt、delta history、Surface identity和topology-gated teardown；aggregate report为O(1)，direct document suite 54/54。没有生产Default和猜测阈值；产品阈值标定、secure policy、增量grapheme handles、model rebase与managed Runtime仍开放。
- **exact committed edit intent(2026-08-28 M1非验收接线)**:`edit_state`在执行insert/delete/replace/composition commit时直接保留exact old/new byte range与typed kind，replacement借用final state slice，不复制第二份正文，也不在每次按键做全文diff。word delete序列最多一次commit；preedit/caret/selection/cancel/identical replacement保持state-only。keyboard/text/IME/delete-surrounding/cut/paste/a11y已由manager document store成功commit后签发public receipt；focus-loss恢复为state-only cancel。E盘focused harness 12/12、document 54/54、history 3/3；产品阈值、model rebase与managed Runtime/profile/WGPU仍开放。
- **输入约束收据(2026-08-27 current-source校准)**:filter、single-line hard-line admission与max-grapheme截断已有共享typed receipt且不携带raw text；constrained preedit cursor/clause以只保存平台实际引用端点的UTF-8 byte map迁移，完全删除和调整均有typed证据。single-line Enter按命令语义handled Submit而不是伪造newline rejection，repeat不重复commit。当前prefix/suffix计数仍为`O(N)`，平台clause生产继续归08。
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

2026-08-29 非验收基础设施：canonical shaping 已从裸 generation probe 收敛为持有 exact generation +
`Arc<FontDatabase>` 的 `FontCollectionSnapshot`，cosmic locale cache 使用调用方 snapshot，不再在 attempt 内
重读 global DB。registry/snapshot/metrics 已归入 `FontCollectionService`，handle 纳入 collection identity。
同日追加的 owner-ready 切片用 `(collection_id, generation)` revision token 贯通 shaped cache、物理/虚拟 canonical
fragment、layout fence、measure/layout cache、rich/plain glyph artifact 与 retained `UiSurface`；artifact 在投影完成后
同时保留 database 和 handle-resolver snapshot，renderer line view 不再按裸 generation 重抓进程集合。独立集合的
foreign mutation 不会使 surface 失效，owned mutation 才推进重排，且长期 layout session 不重建。源码回归与
rustfmt/diff/static scan 通过。后续 screen-space 切片又让 `TextRenderState`、`ScreenSpaceUiTextSystem` 与
`ScreenSpaceUiRenderer` 接受同一显式 collection；plan/segment cache 与 artifact admission 均使用完整 revision，
不再在 renderer 帧路径读取进程级 generation。进程默认只停留在 renderer 构造适配层。真实 Core manager/
window/PIE owner 接线、backend face slot reclaim、Cargo/WGPU/PNG/profile/power 仍开放，状态为
`collection_bound_layout_surface_renderer_pipeline_static_implemented / managed_validation_pending`。

2026-08-30 Core manager 注入边界复核：`TextModule.Manager.FontServices` 已作为 Graphics manager 的显式
依赖，生产 `create_render_framework_with_render_features(&CoreHandle)` 解析该 Core-owned collection，随后
经 `WgpuRenderFramework` 与 `SceneRenderer` 的显式 `..._and_font_collection` 构造器传入 screen-space
文本子系统。静态回归检查锁定这条链，并确认 create 入口不再读取 process-global collection；仍保留的
renderer-side process-default 构造器只属于 bootstrap compatibility adapters；Editor host 的直接 UI/text
适配器继续由单一进程 collection 持有，不与 Runtime Core/session collection 混用。窗口/PIE 实际运行、受管
Cargo/WGPU/PNG、profile、RSS/power 和 Unreal 同负载对比仍未完成，状态为
`core_manager_font_collection_injection_static_implemented / window_pie_wiring_open / managed_validation_pending`。

2026-08-30 Surface input geometry owner 复核：Runtime retained Surface 的 layout/measure 已绑定
Core collection，但 artifact 缺失时的 caret、selection、IME rect 与 pointer hit-test source-metric recovery
仍创建 process-global direct provider。现在 `UiTextMeasureCache` 暴露 exact snapshot，IME 一次 context
更新与 pointer 一次命中各捕获一次，并通过 collection-bound provider 重塑；generation 不一致时 fail closed
到已发布 artifact/glyph advances，避免旧 layout 与新 snapshot 混合。Editor/standalone 兼容 API 继续显式使用
单一 process-owner snapshot。该结构对齐 Unreal `FSlateFontMeasure` 持有具体 `FSlateFontCache` 的 measure/hit
owner，不改变布局算法或逐 glyph 复杂度。静态 suite 19/19、rustfmt、diff-check 通过；独立 collection glyph
identity Rust 回归已写但未运行。Cargo、真实 IME/pointer、WGPU/PNG、profile/RSS/power 仍开放。状态：
`surface_input_geometry_core_collection_static_implemented / managed_product_validation_pending`。

2026-08-30 兼容入口 snapshot owner 复核：剩余 `DirectTextShapeRunProvider` 已从零状态对象改为
创建时捕获不可变 `FontCollectionSnapshot`，同一次 standalone/editor measure、line-break 或
source-range 查询不再因中途 publication 重新读取 process collection。horizontal/vertical 请求
统一进入 canonical diagnostics backend，并发布固定 revision；Runtime retained 路径继续使用
Surface/Session 显式 collection，不新增 shaper。20/20 静态 suite、Python compile、rustfmt 与
scoped diff-check 通过，跨 generation face-identity Rust 回归已写但未运行。状态：
`one_shot_provider_snapshot_bound_static_implemented / cross_generation_mix_removed /
managed_product_validation_pending`。

动态 Runtime UI 同步复核：`RuntimeDynamicSession::build` 在 module activation 后解析一次 Core-owned
collection，并把同一 `Arc` 同时传给 `RuntimePreparedProject::load_runtime_ui_surfaces` 和 HUD/menu
`RuntimeUiExtractCache`。`RuntimeUiSurfaceSet::load`、`UiV2SurfaceBuilder` 与回退 `UiTextMeasureCache`
均沿显式 collection 参数构造；回退 extract key 读取其自身 layout session 的 font generation，不再读取
process-global generation。旧 `UiTemplateSurfaceBuilder`/`UiSurface::new` 未进入上述动态 Runtime 生产路径；
`UiSurface::new` 仍是 Editor host/standalone 的进程 owner 入口，而不是 Core runtime fallback。静态 ownership
suite 18/18、rustfmt 与 scoped diff-check 通过；独立 collection 发布后相同 world/viewport 必须重建
fallback extract 的 Rust 行为回归已写入但未运行。Cargo/WGPU/PNG、project/no-project
产品帧与多 Core 隔离仍待 managed validation。状态：
`dynamic_runtime_ui_and_fallback_cache_core_collection_injection_static_implemented /
process_global_fallback_key_removed / managed_product_validation_pending`。

2026-08-29 又补齐 FontObject owner 的 session 生命周期：Text Core 维护不可 Clone 的
`RuntimeFontAssetClaimScope` 聚合计数，Runtime UI 在首个 layout 前认领依赖，renderer 在 collection refresh
前做稳定路径 reconciliation；最后一个 scope 释放时一次性退休所有 unclaimed owner，并裁剪 renderer
本地负缓存，避免 project/session fallback 污染。稳定帧不取 claim 锁、不分配、不 clone DB、不发布 generation。
这是源码复杂度与生命周期基础设施，不是产品性能结论；release 与 changed/new asset admission 已在一次
collection mutation/publication 中完成。Cargo/WGPU/PNG、hot-reload/project-switch、profile/RSS/power 和真实
文本截图仍待 managed validation。状态：
`collection_owned_font_claim_scope_static_implemented / stable_path_lock_free_by_source /
release_plus_admission_single_publication_static_implemented / managed_validation_pending`。

2026-08-29 clone boundary follow-up：collection mutation 现在提供 published snapshot lease，runtime
font claim/admission/retire 只消费 receipt 时不再复制发布后的完整数据库；`TextRenderState` 的 legacy
mutable database 仍因 native/SDF lazy mutable cache 保留。外层 mutation clone、owner staging clone、
legacy result clone 已拆成独立 profile 边界，owner staging in-place API 与性能/功耗结论必须等待受管
31 样本矩阵。状态：`published_arc_receipt_path_static_implemented /
renderer_mutable_owner_profile_gated / managed_validation_pending`。

screen-space renderer 的旧 single-asset load/resolve 与 standalone admit/retire 入口已删除；测试用
`ensure_font_asset_record` 只包装正式 `refresh_font_asset_records`，并持有真实 collection claim scope。
生产与回归因此共用一个批量依赖 admission owner，不再维护第二套 per-asset publication 语义。

2026-08-30 shaping source identity guard：`BackendShapeRequest::canonicalized` 现在要求局部 UTF-8
文本长度与绝对 `source_range` 字节跨度严格相等，并以 checked subtraction 拒绝反向范围；合法非零
绝对起点仍保留。测试覆盖合法非零绝对起点、跨度不一致、反向范围，以及多硬行局部 slice 对应的绝对
source range。错误请求在 fallback、cache key 和 backend 之前 fail closed 为 `BidiInvariant`，模型测试
已覆盖。Composite activation 20/20、text pointer 1/1、segment cache 13/13、decoration map 4/4、layout
order 4/4 静态契约通过；该切片不改变换行/测量算法。Cargo、真实 WGPU/PNG、31 样本 profile、RSS/power 与 Unreal
同负载仍待受管验收。状态：
`backend_source_range_invariant_static_implemented / malformed_identity_rejected_before_backend /
managed_product_validation_pending`。

2026-08-30 grapheme index incremental splice review：此前每次 document revision 都会使 grapheme
index 失效，下一次查询再从完整 snapshot 重建。现在 edit receipt 在缓存 revision、UTF-8
grapheme 边界以及一边界 ASCII/no-CRLF 上下文满足时，仅拼接替换区间并按 checked byte delta
移动 suffix；Unicode、combining/emoji/ZWJ/RI、CRLF、非边界或 stale index 一律保留原有完整
重建路径。splice 避免完整 snapshot rebuild；ASCII/no-CRLF admission 直接遍历 retained pieces，
不再物化临时 context `String`，并在 piece coverage 不完整时 fail closed。端到端 edit 复杂度仍需受管 profile 验证。新增四个固定 profile counter，并覆盖
ASCII suffix shift、empty insertion/deletion 与 Unicode/CRLF fallback 的
源码回归；未运行 Cargo，也没有 latency/RSS/power 或 WGPU/PNG 数据。状态：
`ascii_grapheme_index_incremental_splice_static_implemented /
unicode_context_rebuild_fallback_preserved /
ascii_incremental_preflight_allocation_free_static_implemented /
managed_profile_pending`。

2026-08-30 glyph artifact source-slice admission：普通与富文本 artifact builder 在 visual
projection 前统一校验 line/run source range 可由当前 source snapshot 精确切出；越界、反向、
非 UTF-8 边界统一 `LayoutFailed`，不再由 projection 将失败切片默认为整行/整 run source map。
零宽 virtual anchor 仍可用，但必须位于合法 UTF-8 边界。新增 run scalar-split 与 virtual-anchor
boundary 回归，局部 Rustfmt 通过；Cargo、真实 WGPU/PNG、31 样本 profile、RSS/power 与 Unreal
同负载仍待受管验收。状态：
`glyph_artifact_source_slice_admission_static_implemented / virtual_anchor_boundary_preserved /
managed_product_validation_pending`。

2026-08-29 SDF font-face recovery review：生产 `SdfFontAssetFaceCache` 已硬切为 lookup-only，只从
`TextRenderState` 已采用的 collection database 解析 asset owner/runtime default face；raster 阶段不再解析
manifest、注册/删除 face 或修改 renderer 私有数据库。离线 artifact cache 继续读取 `.zsdf` manifest/bitmap，
测试 fixture 的 source registration helper 仅在 `cfg(test)` 存在。静态契约 9/9、rustfmt 与 scoped
diff-check 通过；managed Windows build 在 Cargo 启动前因兼容池 `cargo_reuse_pool_busy` 退出，未执行 Cargo，
不跟踪占用 job。WGPU/PNG/profile/power 仍开放。状态：
`single_renderer_font_admission_pipeline_static_implemented / sdf_runtime_face_lookup_only_static_implemented /
managed_build_and_product_evidence_pending`。

2026-08-29 Cosmic locale cache 隔离修正：线程本地 `FontSystem` cache 改为按首次调用传入的
`FontCollectionSnapshot` 惰性初始化，不再先读取进程共享字体库再刷新到 renderer/session 集合。
这样首个 shaping attempt 的数据库 lineage 与调用方一致，并消除一次无意义的整库快照/重建；
locale eviction 与 generation refresh 仍由同一个 cache owner 管理。并行 paragraph prewarm 的进程默认
wrapper 同时缩为 `cfg(test)`，生产路径只接受 session-owned collection；系统字体 opt-in 使用
published snapshot receipt 避免丢弃的整库 clone；无调用的默认 finish wrapper 已删除。
该切片通过 Rustfmt、静态 ownership guard 和 scoped diff 检查；Cargo、真实 shaping/raster、WGPU/PNG、profile/RSS/power
仍待 managed validation，状态：
`cosmic_snapshot_bound_cache_static_implemented / explicit_parallel_shape_collection_static_implemented /
session_isolation_cold_start_corrected /
managed_product_validation_pending`。

2026-08-30 rich parser representation admission：新增独立 `RichParseBudget` 与 typed
`RichTextParseError`。source 在 compiled cache lookup/copy 前拒绝，visible output 在 builder append
与 emoji expansion 前拒绝；有效 token 总数、单 token bytes、单 token attribute count/bytes 在
dispatch/name/value materialization 前受请求预算约束。HTML/BBCode 共用 ActiveTag 栈受默认 128 层请求预算约束并在增长前返回
`ActiveTagDepthBudgetExceeded`。默认 32 MiB 对齐现有 retained text-document 量级，effective byte limit
同时受 `u32` 可表示范围限制。`CompiledRichText` 的 byte/count/cell projection index 全部走
checked build，旧 `u32::MAX` 饱和 identity 已删除；失败 single-flight 不驻留。UI 投影稳定
`ZR-TEXT-LAYOUT-012` 并走 failure layout。parser root/builder/run-alignment 为 715/162/100 行，
当前可复现静态集合 34/34；本次两个 E 盘 Cargo 检查分别在 90/120 秒无输出、无结论，owned 进程已停止；
WGPU/PNG/profile/RSS/power/Unreal matched evidence 未完成。状态：
`rich_parser_typed_byte_admission_implemented / rich_compiled_index_saturation_removed /
rich_active_tag_depth_admission_implemented /
rich_tokenizer_count_and_materialization_budget_implemented /
managed_product_validation_pending`。

2026-08-30 rich compiled grapheme owner correction：current-source 与 Unreal
`IRichTextMarkupParser`/`FTextRunParseResults` 对照确认，parser/compiler 只应发布 stripped text、run
range 与 metadata；grapheme/cluster 属于有实际消费上下文的 shaping/layout owner。旧
`CompiledRichText::cluster_ranges` 没有生产消费者，却为每个可见 grapheme 常驻一个 `(u32, u32)`。
E 盘 release 隔离基线（每档 31 样本 ASCII）显示：1 MiB 为 8 MiB payload、p50 65,236 us；8 MiB
为 64 MiB payload、p50 736,093 us；32 MiB 为 256 MiB payload、p50 3,074,179 us。该字段、全篇
segmentation pass、equality/byte accounting 与测试 accessor 已硬切，不保留兼容/lazy 副本；该孤立 owner
的 post-cutover payload 精确为 0 且不再有构建阶段。完整静态集合 34/34、rustfmt、source guard 与
scoped diff-check 通过。该数据不是端到端 parser/layout/renderer 或功耗结论；Cargo、WGPU/PNG、RSS/power
及 Unreal matched-load 仍开放。状态：`rich_compiled_duplicate_cluster_owner_removed_static /
isolated_o_g_time_and_memory_stage_removed / managed_product_validation_pending`。详见
[`09/2026-08-30-rich-cluster-range-profile-and-owner-review.md`](09/2026-08-30-rich-cluster-range-profile-and-owner-review.md)。

2026-08-30 rich table projection algorithm correction：旧 compiled owner 对每个 cell 全量重扫
runs、paragraphs、tables 三次，4,096 对象隔离 release 基准每次执行 50,331,648 次比较，31 样本
p50/p95/p99 为 60,544/85,779/123,556 us，而实际仅输出 8,192 个索引。按报告先测量再改结构，现以
request-local `RichRangeIntervalIndex`（平衡树 + subtree `max_end`）按相交范围查询，table 再做
depth/containment 过滤；canonical 输入先做线性顺序检查，仅 defensive 乱序输入排序；UI projection 删除
二次 sort/dedup。该树只在 compiled construction 存活，不成为常驻 cache 或第二 source authority。
同输入 31 样本最终隔离复测在 4,096 对象得到 p50/p95/p99 3,337/4,467/5,611 us，旧/新 p50
为 18.14x，进入 interval node 为 215,046，较旧 50,331,648 次比较低 234.05x。256 到 4,096
对象的 p50 增长由 260.97x 收敛为 22.70x；首样本 working-set delta 从 208,896 增至
360,448 bytes，因此临时内存、allocation/RSS/power 仍未关闭。边界/乱序 interval 单测与完整静态集合
当前可复现静态集合 34/34、rustfmt、source/diff guard 通过；Cargo、真实 table layout、WGPU/PNG 和 Unreal matched-load
仍开放。状态：
`rich_table_projection_interval_owner_static_implemented / quadratic_rescan_removed /
isolated_post_profile_complete / managed_product_validation_pending`。详见
[`09/2026-08-30-rich-table-cell-projection-profile-and-redesign.md`](09/2026-08-30-rich-table-cell-projection-profile-and-redesign.md)。

2026-08-30 rich representation count admission：在区间投影修复后继续补齐 M1 资源边界，
`RichParseBudget` 现在额外限制 runs、paragraphs、tables、table cells 与 retained projection
indices；builder 在 run/paragraph/table `Vec` 增长前 fail closed，BBCode table state 在 cell
关闭前的下一次 cell admission 超限即返回 typed error，compiled projection 查询共享总索引 cap；
BBCode block/table depth 默认 32/8 层，超限不再静默 suppression 或饱和到同一 `u16` depth。
默认 representation 数量分别为 131,072 / 16,384 / 4,096 / 65,536 / 262,144。六类超限回归与
当前可复现静态集合 34/34、rustfmt、source/diff guard 通过；general span/node、decorator panic/time/cancel 与 managed
Cargo、WGPU/PNG、产品 profile/RSS/power 仍开放。parser root/builder/run-alignment 当前
715/162/100 行，均未超过 800 行结构预算。
状态：`rich_representation_count_admission_static_implemented /
projection_index_cap_static_implemented / rich_block_table_depth_admission_static_implemented /
managed_product_validation_pending`。

2026-08-30 rich decorator exact-tag dispatch：Zircon decorator 在注册时已唯一拥有 normalized tag，
旧 registry 却为每个 candidate token 线性扫描所有 decorators。E 盘 release 基线以 4,096 次末项命中、
31 样本测得 16/256/4,096 decorators 的 p50 为 517/7,381/116,314 us；decorator 数增加 256x，
p50 增长 224.98x。Unreal `FRichTextLayoutMarshaller::TryGetDecorator` 的线性扫描服务于任意
`ITextDecorator::Supports` 谓词；Zircon 是 exact tag 合同，不复制无对应语义的扫描实现。

`DecoratorRegistry` 现以单一 `HashMap<String, Box<dyn RichTextDecorator>>` 为 owner，registration
通过同一次 `Entry` 完成 duplicate admission/insertion，borrowed tag lookup 后再调用 callback；没有镜像 Vec、
global registry 或 iteration contract，既有 decorator generation/cache identity 不变。同样 31 样本后测
p50 为 140/142/139 us，4,096 decorators 下 p50/p95 改善 836.79x/1,040.07x。lookup loop 的
working-set delta 不包含 registry 建表，不能作为 retained HashMap 内存结论。当前可复现静态集合 34/34、rustfmt、
唯一 owner/零线性 dispatch guard 通过。后续 provider admission 以 `catch_unwind` 将 callback panic
变为 tagged typed failure，默认限制单次 decorator metadata 64 KiB、请求 retained run metadata
32 MiB；builder 只对非合并物化 run 计费，UI 将 panic 映射 `LayoutFailed` 而非 budget diagnostic。
补充 no-op dyn callback 边界复测把新 `catch_unwind` 纳入计时：16/256/4,096 decorators 的 p50
为 146/149/154 us，4,096 档旧/新为 733.54x，未恢复 provider-count 线性项。
Rust panic/metadata 行为回归已写但未运行；deadline/cancel、provider lease/revoke、registration count、
callback 临时 allocation、Cargo、WGPU/PNG、RSS/power 与 Unreal matched-load 仍开放。状态：
`rich_decorator_exact_tag_hash_dispatch_static_implemented /
isolated_linear_dispatch_bottleneck_removed_profiled /
rich_decorator_panic_and_metadata_admission_static_implemented /
managed_product_validation_pending`。详见
[`09/2026-08-30-rich-decorator-dispatch-profile-and-redesign.md`](09/2026-08-30-rich-decorator-dispatch-profile-and-redesign.md)
及 [`07/2026-08-30-rich-decorator-provider-admission.md`](07/2026-08-30-rich-decorator-provider-admission.md)。

2026-08-30 rich owned parse clone hard cut：生产消费者追踪确认 UI/runtime 已全部持有
`Arc<CompiledRichText>`，公开 `RichTextParser::parse()` 与 crate bridge 仅剩测试消费者，却在每次
canonical cache 命中后深拷贝 runs/paragraphs/tables 及动态 style/link metadata。E 盘 31 样本 clone
基线在 4,096/32,768/131,072 runs 下每次分别分配 12,355/98,819/395,267 次、请求
1,014,784/8,118,272/32,473,088 bytes，p50 为 2,454/22,059/111,366 us；最大档首次
working-set delta 为 40,169,472 bytes。Unreal marshaller 由 parser output 直接创建 layout model/runs，
不在 canonical owner 后暴露一个只复制部分身份的第二 artifact API。

生产 `RichTextParser::parse` 已硬切，唯一 public materialization 为
`compile() -> Arc<CompiledRichText>`；owned parse method/bridge 只在 `cfg(test)` 存在，测试 corpus
不进入生产二进制。没有兼容 alias、第二 cache 或 detached snapshot。被删除生产阶段 post allocation/
bytes 精确为 0 且阶段不存在；这不是端到端 compile/layout/frame 结论。当前可复现静态集合 34/34、rustfmt 与
唯一 owner guard 通过，Cargo、外部消费者迁移、WGPU/PNG、RSS/power 和 Unreal matched-load 仍开放。
状态：`rich_owned_parse_clone_hard_cut_static_implemented /
immutable_compiled_artifact_public_owner_converged / isolated_clone_profile_recorded /
managed_product_validation_pending`。详见
[`09/2026-08-30-rich-owned-parse-clone-profile-and-cutover.md`](09/2026-08-30-rich-owned-parse-clone-profile-and-cutover.md)。

2026-08-30 rich parser generation exhaustion：RRT-P1-017 的 cache identity 回绕路径已静态硬切。
parser identity 由 `Option<NonZeroU64>` 表达有效/耗尽状态，allocator 用
`fetch_update + checked_add`，不会在 atomic wrap 后复用旧 owner；compile 在 source/cache 工作前返回
typed `ParserIdentityExhausted`。decorator/emoji registration 先 checked-admit 下一代，再修改唯一
registry；`u64::MAX` 后返回各自 `GenerationExhausted` 且 owner 不变。此项修复 identity 正确性，不是
性能优化，未声明 latency/power 收益。当前可复现静态集合 35/35、rustfmt/source guard 通过；owner-local
Rust 边界测试已写但未运行。provider lease/revoke、targeted retirement、RuntimeRichTextService、managed
Cargo、WGPU/PNG、RSS/power 仍开放。状态：
`rich_parser_non_reusing_generation_static_implemented / cache_identity_wrap_alias_removed /
managed_product_validation_pending`。详见
[`07/2026-08-30-rich-parser-generation-exhaustion.md`](07/2026-08-30-rich-parser-generation-exhaustion.md)。

2026-08-30 compiled-rich Surface-session owner hard cut：process-global parser/cache/free compile/
lookup/shared report 已从 production 删除；每个 `RichTextParser` 持有 bounded cache，现有
`UiSurface -> UiTextMeasureCache -> SharedTextLayoutSession` 显式持有并贯通 layout/measure/prewarm/
retained document/render preparation。测试 corpus 的 static parser 仅 `cfg(test)`。完整静态集合 36/36；
owner-local reuse/isolation/clear Rust 回归已写未运行，managed Cargo、WGPU/PNG、multi-Surface
allocation/RSS/contention/power 与 Unreal matched-load 仍开放。状态：
`RRT-P1-013_process_global_owner_cut_static_complete / managed_product_validation_pending`。详见
[`07/2026-08-30-runtime-rich-text-service-owner-cutover.md`](07/2026-08-30-runtime-rich-text-service-owner-cutover.md)。

2026-08-30 one-shot layout-session current-source audit：P1-14 的旧“产品频繁构造短命 session”在当前
仓内未复现。`UiSurface` 与 dynamic HUD/menu 已 retained cache/session + Core font collection；HUD/menu
的 default cache 仅测试使用。剩余入口为明确 standalone/compatibility、测试或 native framebuffer
验证。未做无证据优化，也未引入 TLS/global cache；后续只有真实产品 callsite 的 construction、cold/warm、
allocation/RSS/backend/power profile 才能授权 owner 迁移。详见
[`09/2026-08-30-one-shot-layout-session-current-source-audit.md`](09/2026-08-30-one-shot-layout-session-current-source-audit.md)。

2026-08-30 rich provider generation retirement：decorator/emoji 成功注册在 generation 提交后主动清空
parser-owned compiled residency；duplicate/invalid/exhausted 失败不清 cache，外部持有的 compiled `Arc`
继续作为 last-use artifact。静态集合 36/36、定向 Rustfmt 通过，Rust 行为测试未运行。provider-qualified
snapshot/unregister/revoke、single-flight cancel、managed Cargo/WGPU/PNG 仍开放。状态：
`RRT-P1-016_current_registration_retirement_static / provider_revoke_open /
managed_product_validation_pending`。

RRT-P1-010/014/016 provider lifecycle 架构重审已完成：采用上层 qualified immutable catalog
snapshot、Surface frame-boundary publication、Core service call admission/drain/timeout 作为唯一 revoke/
module-unload fence；不建立 text-global registry，也不把 retained provider limit 混入 request-local parse
budget。实现与 1/64/1,024-provider profile 尚未开始。详见
[`07/2026-08-30-rich-provider-snapshot-and-revoke-design.md`](07/2026-08-30-rich-provider-snapshot-and-revoke-design.md)。

执行前 Runtime Interface 定向审计确认 `TextModule` owner 与零 production stub，但 Runtime06 plugin
lifecycle 仍 `in_progress`，namespace 分类/文档同步/app call-site 存在审计漂移，且 `zircon_runtime`
仍因大型 production owner 被分类为 `needs-refactor`。因此本 Text 生命周期不落局部 unload fence、
provider identity/counter 或未测 numeric budget；M1-M4 继续等待 Runtime06/Core owner 收敛与 E 盘
1/64/1,024-provider profile，当前目标绕过该边界继续其它非验收工作。

2026-08-30 rich style shaping projection：italic 与 OpenType features 已由 rich override 贯通到
resolved style、font query、horizontal/vertical backend request、Cosmic fallback 和 shaped-cache identity；
公共 neutral italic 请求走同一映射。feature 同 tag 冲突采用最后声明生效并按 tag 稳定排序，cache 与 backend
消费同一 canonical list。静态 Runtime Text 集合 47/47，Rust 行为回归未运行，状态为
`RRT-P1-023_italic_and_feature_projection_static_complete / managed_validation_pending`。

letter spacing 的 current-source/Unreal/Cosmic/Godot 调研已完成，但实现未开始。禁止直接采用 Cosmic
逐 glyph 且含末尾的 spacing；目标是 shaping 后、measure/artifact 前的唯一 neutral cluster-gap owner，
并要求 `liga=0`、cache identity、RTL/纵排/负间距政策和 31 样本 E 盘基准。详见
[`07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md`](07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md)。

2026-08-30 rich-table geometry static cutover：session 已快照 typed geometry budget，shared
rich/VerticalRl intrinsic 与 table 两阶段布局共用 bounded/unbounded constraint；旧 `f32::MAX/4`、
byte-derived provisional frame 和 non-finite-to-zero 已删除。`2^24` 默认值只表达 `f32` 数值安全
天花板，后续较低产品政策仍需 E 盘 corpus。column/row solver、track prefix、line/frame/advance、box、
translation 和 aggregate 逐层受检；plain size/fixed-height/range-width 也消费 shared admission owner，
typed receipt 返回 `GeometryTooLarge`。基础设施静态合同 31/31，
Rust 行为测试已写未运行；状态：`RRT-P1-033_geometry_budget_and_table_cutover_static_complete /
managed_compile_render_and_profile_pending`，详见
[`07/2026-08-30-rich-table-geometry-budget-review.md`](07/2026-08-30-rich-table-geometry-budget-review.md)。

2026-08-30 rich projection index admission：`CompiledRichText` 的 `u32` capacity gate 之后，
`UiParsedText` 根/子 projection 也已硬切为 fallible checked construction；不再裸 cast 或静默
丢弃无效索引，artifact rebuild 失败时不发布部分 view。静态 Runtime Text 集合 47/47，
managed Rust validation 待办。状态：
`RRT-P1-011_compiled_admission_and_ui_projection_static_complete / managed_validation_pending`。

2026-08-30 rich format version identity：公开/runtime 枚举已硬切为 `Plain`、
`MarkdownInlineV1`、`BbCodeV1`、`HtmlSubsetV1`，wire/style 值只接受 `plain`、
`markdown_inline_v1`、`bbcode_v1`、`html_subset_v1`。cache key 直接持有格式枚举，不再依赖
手写 `u8` 标签。旧完整 Markdown/HTML 承诺与旧 alias 已移除；后续 current-source closure 又在七个
internal/UI/renderer/transport consumer 清理 67 处旧 `RichTextFormat/UiRichTextFormat` 枚举引用，并将残留
`LinkRef::href` fixture 迁到 `UiRichLinkTarget`，完整 Runtime/RuntimeInterface 枚举扫描为零，word-boundary
契约防止回归。RRT-P1-024
consumer closure 静态完成，
structural HTML-subset diagnostics 以四个稳定 code/source range/recovery、256 条独立预算和 truncation
receipt 进入 canonical artifact；attribute/style 单遍 follow-up 将 code 扩展到八个且 cache accounting
同步；malformed-tag/unterminated-quote/malformed-or-unrecognized-entity follow-up 将 code 扩展到十二个，
畸形 source 保留为 visible text，普通 less-than 文本不误报，EOF 仍保持 source-ordered receipt。
parser root/html/diagnostics/active-tags child 为 558/259/108/123 行，完整静态集合 47/47。RRT-P1-025
artifact/current-source follow-up 同时把 11 个不存在的 `[link]` fixture 迁为 `[url]`，并断言 composite
artifact 确实持有 canonical typed target；inline-widget 产品 fixture 的旧 `bbcode` wire 值也已迁到
`bbcode_v1`，Runtime authored-value 扫描为零。当前 authoring diagnostic 类静态完成；consumer closure
定向静态集合 47/47（最终复跑 1.744 s），managed Rust、
bounded corpus profile 与 product evidence 待办。详见
[`07/2026-08-30-rich-format-version-identity-review.md`](07/2026-08-30-rich-format-version-identity-review.md)。

2026-08-30 inline-widget identity follow-up：固定尺寸 direct-child arrange/render/hit 闭环与 renderer
no-placeholder 已保持，compiled `Widget { id: u64 }` 进一步硬切为 owner-local
`RichInlineWidgetSlotId`。text artifact/UI projection 不再提前构造 live `UiNodeId`；Surface 仅在当前
`&mut UiTree` 布局作用域内把 slot 解析为当前 owner 的 direct child，且不保留跨帧 binding，因此
destroy/rebuild/换 surface 只能从当前树重解，不能复用旧 lease。未来 desired-size retained binding 必须携
surface-session identity + node incarnation + revoke 合同。当前为
`RRT-P1-026_fixed_size_direct_child_complete / RRT-P1-027_typed_local_slot_current_tree_binding_static_complete /
managed_validation_pending`；完整静态批次 47/47（1.744 s），managed Cargo/profile/power/WGPU/PNG 仍待办。

2026-08-30 rich-table layout work receipt：解析侧已有 request-local table/cell/token/depth/size
admission，本轮不重复增加猜测阈值。session 新增逐帧 content-free report，记录实际 table/source/cell、
preferred/final cell pass、resolved track 与 admitted line/box 工作量，并在 frame end 投影十二个固定 profile
名称。失败路径保留已发生工作，只有通过整表 geometry admission 的结果计入 published output；布局算法、
缓存和 admission 行为不变。完整 Runtime Text 静态集合 52/52，Rust 行为测试未运行。状态：
`RRT-P1-038_table_layout_work_receipt_static_complete / managed_profile_and_budget_decision_pending`。
详见 [`07/2026-08-30-rich-table-layout-work-receipt-review.md`](07/2026-08-30-rich-table-layout-work-receipt-review.md)。

2026-08-30 rich prepared-run current-source 校准：正常 rich artifact route 已消费 composite artifact 的
generation-bound glyph slice，不再逐 run 重整形。剩余差距是 serializable layout/paint 字符串驻留和
compiled style range projection；现有 layout-cache/renderer-batch/fallback 指标尚缺 paint projection 的
phase-local allocation/time，故不改 DTO、serde/remote contract 或添加 cache。状态：
runtime 投影边界已增加固定 scope 与十二项低基数 work/byte counter，segment cache 只报告新 materialize，
完整 cache hit 发布零；完整静态集合 52/52。状态：
`RRT-P1-034_paint_projection_profile_infrastructure_static_complete /
RRT-P1-036_managed_baseline_and_owner_decision_pending`。详见
[`07/2026-08-30-rich-prepared-run-current-source-review.md`](07/2026-08-30-rich-prepared-run-current-source-review.md)。

2026-08-30 rich accessibility semantic projection：a11y own name 与 relation text 已从 raw
`text/label/value` scalar 硬切到 generation-bound `RichSemanticProjection`。投影复用 Surface render cache
的 node-command index，只接受 source/format/artifact 一致的当前 `CompiledRichText` visible text；不新增 parser、
全 command scan、layout-line 拼接或第二 semantic cache。stale/missing/ambiguous rich artifact 不再回退朗读
markup，plain 与显式 a11y/alt/tooltip 合同保持。复杂度为 O(log nodes + node commands + source validation +
visible materialization)，generation compare 为 O(1)。完整 Runtime Text 静态集合 53/53，Rust 行为回归已写未运行。
隐藏 relation target 的后续 owner 已收口：没有 render command range 时，通过 Surface 已有
`SharedTextLayoutSession`/compiled cache 生成 visibility-independent projection；一旦存在已发布视觉 range，
仍严格验证其 source/format/generation，不以新解析掩盖 stale render。无第二 parser/cache 或 eager hidden-tree
遍历。完整 Runtime Text 静态集合现为 54/54。状态：
`RRT-P1-039_visibility_independent_surface_semantic_owner_static_complete /
RRT-P1-040_typed_children_and_managed_validation_pending`。qualified semantic identity/action route、typed child、
managed screen-reader、WGPU/PNG、RSS/power 仍开放。详见
[`07/2026-08-30-rich-accessibility-semantic-projection-review.md`](07/2026-08-30-rich-accessibility-semantic-projection-review.md)
与 [`07/2026-08-30-rich-visibility-independent-semantic-owner-review.md`](07/2026-08-30-rich-visibility-independent-semantic-owner-review.md)。

2026-08-30 rich list semantic metadata hard cut：BBCode list item 不再把 ordered/unordered、canonical
ordinal、marker style 和 nesting level 丢成一个 marker range。`RichListItemKind`、
`RichOrderedListMarker` 与 `RichListItem` 现在是 compiled semantic authority；layout 只派生 marker
`UiTextRange` 并在私有段落几何 projection 中测量，禁止从 marker string 反推语义或写回 model。
ordered ordinal 使用 checked advance，level 为一基 semantic depth。完整 Runtime Text 静态集合
55/55，Rust 行为测试已写未运行。状态：
`RRT-P1-037_typed_list_item_metadata_static_complete /
RRT-P1-040_qualified_publication_and_managed_validation_pending`。完整 typed block tree、HTML list、
qualified a11y child/action、managed WGPU/PNG 与 profile 仍开放。详见
[`07/2026-08-30-rich-list-semantic-metadata-hard-cut.md`](07/2026-08-30-rich-list-semantic-metadata-hard-cut.md)。

2026-08-30 rich inline image semantic fallback：`InlineObjectRef::Image` 已保留有预算的 alt/tooltip；
HTML `alt/title` 与 BBCode attribute form 进入同一 compiled run。`CompiledRichText` 一次生成独立预算的
semantic text，无 inline 时共享 visible Arc；accessibility 不新增 parser/run walker/cache。显式空 alt 的
decorative 语义、tooltip 次级 fallback、相邻合并 inline 的逐 placeholder replacement 与 malformed range
fail-closed 均有 Rust 合同。完整静态集合 56/56。状态：
`RRT-P1-029_inline_image_semantic_fallback_static_complete /
RRT-P1-040_qualified_inline_children_and_managed_validation_pending`。resource outcome、qualified child/action、
managed WGPU/PNG 与 profile 仍开放。详见
[`07/2026-08-30-rich-inline-image-semantic-fallback-owner-review.md`](07/2026-08-30-rich-inline-image-semantic-fallback-owner-review.md)。

2026-08-30 rich typed image-icon foundation：family-only glyph 已从 production contract 删除；
`RichIconAssetId`、显式 size/baseline/alternative text、`IconAsset` dependency、共享 horizontal/VerticalRl
geometry、UI texture collector 与 renderer image batch 已静态贯通，paint 不再二次 shape。完整静态集合
40/40（最终复跑 0.222 s）。
状态：`RRT-P1-028_typed_image_icon_asset_hard_cut_static_complete /
intrinsic_metric_revision_readiness_font_icon_and_managed_validation_pending`。详见
[`07/2026-08-30-rich-icon-asset-and-font-lease-architecture-review.md`](07/2026-08-30-rich-icon-asset-and-font-lease-architecture-review.md)。

2026-08-30 rich inline resource outcome owner review：authored-size image/icon 不复制 mutable
resource state，layout 在 fallback/ready 切换时保持稳定；load/upload/fallback 归既有 frame resource
prepare。当前 `Option` admission 与被忽略的 upload result 会丢失失败原因，后续必须建立由 requested/
resolved ID、management/readiness generation 和 prepared revision 限定的 typed receipt，再用受管 profile
决定是否优化 registry resolution、upload、binding 或 batch rebuild。状态：
`RRT-P1-029_resource_outcome_architecture_review_complete /
frame_qualified_prepare_receipt_implementation_not_started / managed_profile_and_product_validation_pending`。
详见 [`07/2026-08-30-rich-inline-resource-outcome-owner-review.md`](07/2026-08-30-rich-inline-resource-outcome-owner-review.md)。

2026-08-30 rich link tooltip metadata：typed target 之外，HTML `a[title]` 与 BBCode
`[url href=... title=...]` 现在把共享 `Arc<str>` tooltip 贯通 parser、quota、compiled residency 与
hit projection；不提前接入 surface overlay-ID/timer 状态。residency 算法归位独立
`compiled/memory.rs`；当前 root/memory 为 730/76 行，完整静态集合 58/58（最终复跑 0.236 s）。状态：
`RRT-P1-030_typed_target_and_tooltip_metadata_static_complete /
RRT-P1-040_qualified_link_child_and_managed_validation_pending`。详见
[`07/2026-08-30-rich-link-tooltip-metadata-owner-review.md`](07/2026-08-30-rich-link-tooltip-metadata-owner-review.md)。

2026-08-30 rich typed dependency closure foundation：`CompiledRichText` 已删除只含 image 却命名为
全部资源的 `resource_ids()`，改为排序去重的 `Arc<[RichTextDependency]>`；当前首个合格 variant 为
`ImageTexture(ResourceId)`，GPU texture collector 显式按 kind 消费；后续 image-icon cutover 已加入
`IconAsset(RichIconAssetId)`；widget 已使用独立 owner-local slot kind，仍不把 font family、widget slot 或
decorator generation 伪造成 resource lease。
状态：`RRT-P1-020_typed_image_and_icon_dependency_foundation_static_complete /
generation_font_widget_decorator_lease_and_managed_validation_pending`。详见
[`07/2026-08-30-rich-typed-dependency-closure-foundation.md`](07/2026-08-30-rich-typed-dependency-closure-foundation.md)。

2026-08-30 rich cache owner-qualified reset telemetry：compiled cache 继续由
`SharedTextLayoutSession -> RichTextParser -> CompiledRichTextCacheOwner` 单链持有；旧 UI cumulative-delta
sampler 已删除。cache mutex 内一次 take/reset 六项区间事件，residency gauge 保留，parser/decorator/emoji
generation 与 saturation receipt 随同快照发布；profile 仅有固定低基数名称且不含 markup。状态：
`RRT-P1-022_parser_provider_qualified_reset_snapshot_static_complete /
project_surface_correlation_and_managed_profile_pending`；当前基础设施静态集合 35/35（最终复跑
0.315 s），managed profile/RSS/power 未完成。详见
[`07/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md`](07/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md)。

2026-08-30 rich single-flight contention instrumentation：保留现有 `OnceLock` single-flight 语义，
cache owner 新增 compile in-flight gauge 与完成 waiter 的 count/total/max nanos，使用 initializer-local
`Cell` 和 RAII gauge guard，不复制 parse、不添加 timeout。当前基础设施静态集合 36/36（0.206 s），
root/tests/profile 为 541/340/739 行。状态：
`RRT-P1-014_contention_measurement_static_complete /
bounded_worker_cancellation_and_managed_profile_pending`。详见
[`07/2026-08-30-rich-single-flight-contention-instrumentation.md`](07/2026-08-30-rich-single-flight-contention-instrumentation.md)。

2026-08-31 rich paint-block geometry owner review：既有 `O(lines + runs)` 仅证明 glyph route
directory；接口层 paint-run frame 仍逐 run 重复 line grapheme/advance prefix work，inline renderer 又逐对象
查 line/run 并重算同一 prefix，最坏仍可达 `O(R * G + I * L + I * G)`。已按 Unreal Slate positioned
`ILayoutBlock` 单一 owner 确定目标边界，并在 profiling feature 加入 7 个固定低基数 work/frame-agreement
计数器；普通 build 为零尺寸 aggregate。Interface exact-production-helper 的 Windows release-only ignored
benchmark 已静态实现 1/100/1k/10k runs、3 次 warm-up、31 个原始 timing/RSS 样本与 p50/p95/p99 输出，
renderer harness 同步覆盖 dense LTR/RTL/VerticalRl inline 与多 hard-line 查找 lane，counter capture 不进入
计时。两者均尚未执行。状态：
`rich_paint_block_current_source_review_complete /
inline_measurement_instrumentation_implemented_static /
interface_and_renderer_release_profile_harnesses_implemented_static /
managed_31_sample_baseline_and_single_owner_cutover_pending`。详见
[`09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md`](09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md)。

同一 paint boundary 的 correctness prerequisite 已继续收敛：rich run 的 frame、font size 与 line height
必须是可绘制有限正值；renderer compatibility style lookup 复用 layout 对非法 size/family override 的
admission；present rich layout 的空失败 projection 不再落入 generic plain batch。resolved paint projection
还会单遍验证每个非空 run 的连续 visual range 与精确 UTF-8-safe line slice，损坏输入整批失败；空 metadata
run 与 grapheme 内 scalar-aligned 样式边界保持合法。focused 静态合同 6/6、完整 Runtime Text 静态集合
94/94；新增 Rust 回归尚未通过 managed Cargo，真实 WGPU/PNG 与 31 样本 profile 也未执行。状态：
`rich_paint_correctness_prerequisites_static_complete /
resolved_run_visual_slice_congruence_static_complete /
managed_validation_and_profile_pending`。详见
[`03/2026-08-31-rich-paint-run-cardinality-fail-closed.md`](03/2026-08-31-rich-paint-run-cardinality-fail-closed.md)。

2026-08-31 rich inline UI texture prepare owner 已完成静态基础设施收敛：scene-resource prepare 用
frame epoch、精确 management/readiness generation、requested/resolved ID 与 prepared revision 发布有序
typed receipt，区分 unresolved/not-ready/load-failed/wrong-kind/invalid-descriptor/generation-changed/
upload-failed/ready。descriptor admission 与 GPU publication 复用同一个 `ResourceSnapshot<TextureAsset>`，
image binding 只接受 receipt 中 `Ready` 且与 streamer 当前 prepared revision 精确一致的纹理，其余保持
authored geometry 并使用共享 fallback；renderer 不再为有 receipt 的依赖重复 registry/load。4 个固定 work
counter 已记录 locator scan rows、snapshot loads、prepared reuse 与 upload attempts。直接 ID 路径为常数查找，
但 locator-derived 兼容路径在建立 secondary index 前仍是最坏 `O(D * R + B)`；按计划必须先完成
1/16/128/512 managed profile，当前未做缓存或索引优化。2026-08-31 二次 owner 复核将 collector
输出硬切为模块私有 `UiTextureDependencies`，frame prepare 不再接受可重复的裸 ID slice；复用既有
HashSet+sorted Vec，不新增 dedupe pass 或分配。focused 静态合同 7/7、完整 Runtime Text 静态集合
106/106；Rust/Cargo、WGPU/PNG、RSS/power 与 Unreal matched-load 仍待验证。状态：
`RRT-P1-029_frame_qualified_prepare_receipt_static_implemented /
single_snapshot_and_exact_revision_binding_static_implemented /
distinct_dependency_owner_invariant_static_implemented /
managed_validation_and_profile_pending`。详见
[`07/2026-08-30-rich-inline-resource-outcome-owner-review.md`](07/2026-08-30-rich-inline-resource-outcome-owner-review.md)。

- 迁入记录：[`../../_archive/zircon_runtime/text/09/2026-07-09-index-output-records.md`](../../_archive/zircon_runtime/text/09/2026-07-09-index-output-records.md)
