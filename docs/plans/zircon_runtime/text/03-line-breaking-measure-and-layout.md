---
related_code:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/candidate_line.rs
  - zircon_runtime/src/ui/text/layout_engine/direction.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/range_mapping.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/text_geometry.rs
  - zircon_runtime/src/ui/surface/text_shape.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break/tests.rs
  - zircon_runtime/src/graphics/text/layout/line_break/glue.rs
  - zircon_runtime/src/graphics/text/layout/line_break/glyph_fallback.rs
  - zircon_runtime/src/graphics/text/layout/line_break/smart.rs
  - zircon_runtime/src/graphics/text/layout/line_break/soft_hyphen.rs
  - zircon_runtime/src/graphics/text/layout/line_break/wrap_space.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku.rs
  - zircon_runtime/src/graphics/text/layout/kinsoku/tests.rs
  - zircon_runtime/src/graphics/text/layout/align.rs
  - zircon_runtime/src/graphics/text/layout/overflow.rs
  - zircon_runtime/src/graphics/text/layout/tab.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text_pixel_snap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_advances.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_char_run.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
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
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/text_measure.rs
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime/src/ui/tests/text_layout/mod.rs
  - zircon_runtime/src/ui/tests/text_layout/alignment.rs
  - zircon_runtime/src/ui/tests/text_layout/wrapping.rs
  - zircon_runtime/src/ui/tests/text_layout/overflow.rs
  - zircon_runtime/src/ui/tests/text_layout/direction.rs
  - zircon_runtime/src/ui/tests/text_layout/edit_state.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontMeasure.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontMeasure.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/godot/servers/text/text_server.h
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/slint/internal/core/textlayout/linebreaker.rs
  - dev/slint/internal/core/textlayout/linebreak_unicode.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/textwrapper.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
status: in_progress
---

# 03 换行规则 / 文本长度计算 / 布局 / 对齐

> 本计划在 `02` 的 `ShapedGlyphRun` 之上做**行切分、度量、对齐、竖排布局**。它是 `editor_layout/17 G1`(测量=绘制)与 G3(默认多行换行)的运行时实现,根治"等宽近似 → 错位/溢出/`Sce` 截断"。

## 1. 目标

1. **文本长度计算(UE 对齐)**:基于真实字形 advance/kerning 的度量,支持**子范围度量**(对齐 `FShapedGlyphSequence::GetMeasuredWidth(StartIndex, EndIndex)`)、行高/上下行距、tab stop、首行缩进;BIDI 与竖排下度量正确。
2. **换行规则**:UAX#14 行断机会 + word/glyph 模式 + **CJK 行首尾禁则**(避头尾)+ 连字符(soft hyphen + 字典可选)+ 长词逐字回退 + 不可断空白处理。
3. **对齐与两端对齐**:left/center/right/start/end(随 BIDI 基方向)+ justify(词间 + CJK 字间 + 阿拉伯 kashida 可选,对齐 godot `JustificationFlag`)。
4. **溢出**:clip / ellipsis(首/中/尾省略)/ shrink-to-fit / clamp 字号(接 `editor_layout/17 §规则4`)。
5. **竖排布局**:列切分(主轴 y)、列间距、竖排禁则、行(列)对齐。

## 2. 现状与差距

- `layout_engine.rs`:`layout_text`/`wrap_source_runs`/`append_word_wrapped_segment`/`ellipsize_line`/`aligned_x` 全部建立在 `text_advance(font_size)=font_size*0.5` 等宽近似上 → `editor_layout/17 G1` 根因;`baseline: font_size*0.8` 硬编码 → 垂直错位。
- 换行:有 `UiTextWrap::{None,Word,Glyph}`,但 word 仅按空格(不识 CJK 无空格断点、不走 UAX#14)、无禁则、无连字符。
- `measure_cache.rs`:宽度桶缓存在,但喂启发式宽度;`graphics/text/layout/measure.rs` 已有 owner-local source byte 子范围度量首段并喂回 `measured_grapheme_widths(...)`;2026-07-03 已通过 `ui::surface::measure_text_source_range_width(...)` 暴露 include-kerning=true 的 public source-range measure 入口;2026-07-04 `ui/text/geometry.rs` 的 IME/caret 简单 LTR source-isomorphic 路径已直接消费 source-range shaped width,并通过 `ui/surface/text_geometry.rs` 暴露 public cursor/range geometry surface;同日 `TextShapeRequest.include_kerning` 已接到 cosmic-text `kern=0` feature,`measure_text_source_range_width_with_kerning(...)` 在请求 unkerned 时重新走同一后端 shaping；同日晚间 `UiSurface` 渲染提取已成为 `UiTextMeasureCache` 的真实生产 consumer,owner text 与 TextField 统一走 `resolve_text_layout_with_cache(...)`;随后 IME update 路径在请求 cursor/composition rect 前刷新当前树的 render extract,并直接消费 `UiRenderCommand.text_layout`,不再单独重跑 direct layout resolve。2026-07-06 `UiTextMeasureCache` 已接入 generic measure/layout cache 与同帧 frame dedup,重复 natural-size/full-layout 请求同帧先命中 dedup；随后 shared shaped-run provider routing 已接通,measure miss 与 full layout miss 共享 `ShapedRunCache`,`layout_engine/line_box.rs` 对不含 tab 的小字号 label 直接采用 provider grapheme advances,只有真实包含 `\t` 时才额外测量空格宽度做 tab stop。`render_perf_text_measure_then_layout_shapes_once` 已关闭单 label shape-count evidence:稳定 `"Hg"` line metrics 预热后,`editor base.zui` 的 measure+layout 只对真实 source label miss/insert 一次 shaped run；2026-07-07 `render_perf_text_scroll_list_reuses_cache` 已关闭 scroll-list shape/layout 首段:滚动 3 行后只为新进入视口 row 增加 shaped miss/insert,重叠 row 命中 shaped cache。2026-07-08 `UiTextShapePrewarmRequest` / `UiTextMeasureCache::prewarm_horizontal_paragraphs(...)` 已把 PF-M2 parallel shape pool 接到 UI cache 显式预热入口,可见 editor row 先批量进入同一个 `ShapedRunCache`,后续 layout 只增加 shaped hits;随后 `ui/surface/render/text_prewarm.rs` 在 render extract 前自动收集可见 owner text 并预热同一 cache,组件 painter 生成的 `Text` command 与 rich/vertical prewarm 也已由阶段 09 PF-M2 follow-up 接到同一 cache。本轮没有保留“用真实字符串行高替换 metrics sample”的实验方案,避免改变既有换行容量。复杂 BIDI/竖排 source-range 几何、完整 backend cluster reverse lookup、scroll raster/upload counters 与窗口级 QA 仍等待。
- `hit_test.rs`:已优先消费 `UiResolvedTextLine.glyph_advances` 做视觉 grapheme midpoint 反查,tab/justify/kashida 等 layout-stage advance 结果不再被默认样式重测量覆盖；post-wrap visual-order adapter 已硬切到 02 `bidi.rs` 的 UAX#9 level/isolate/L1/L2 owner，旧 ASCII/RTL-block 猜测已删除。2026-07-10 `hit_test/visual_source.rs` 已把 visual cluster source range + direction 贯穿 midpoint hit，RTL cluster leading/trailing edge 分别回到 logical end/start 并返回 Downstream/Upstream affinity；caret/range geometry 与完整 backend `ShapedGlyph.source_range` map 仍需同样硬切。
- Justify、shrink-to-fit、clamp font size、tab stop 已有首段；kashida 已有 advance-based 首段但尚未插入真实 tatweel glyph；2026-07-03 已补 SDF char-run 对 resolved grapheme advances 的投影与 mixed overlay whole-grapheme fallback 首段,并补 `sdf_char_run.rs` 对 ZWJ/zero-width/Bidi format/variation selector 等 invisible format controls 的零槽位/零 advance 规则,避免 `glyph_advances` 契约在组合字符/emoji cluster 或 format-control scalar 上被字符数校验/错误 fallback advance 打断；同日 `text_pixel_snap.rs` 把 native glyphon `TextArea` 与 SDF draw planning 的文本 frame 原点贴像素规则合并到单一 owner,避免布局 frame 小数原点在 native/SDF 上屏阶段重新制造左右落点漂移；后续又把 horizontal/vertical SDF glyph bitmap frame 的 x/y 原点经 `text_glyph_device_frame(...)` 同步吸附,保留 layout advances 与 bitmap extent 但消除小字号 glyph bitmap 左右小数落点；editor retained-host 已把 runtime/host advance 与 shaped-origin 接受路径改成 fail-closed,并在 2026-07-04 将每 grapheme/shaped-origin advance 可接受窗固定为 `0.0625px`,让 `editor base.zui` 这类小字号 label 的 0.125px 局部借位直接回退 host natural spacing；同日又新增 retained 1/8px phase-bin 接受门,即使 `folder-open.svg` 这类 label 只有 +0.05px shaped-origin delta,只要会把 glyph 推到另一个最终 raster phase 也会回退 host natural spacing；仍缺复杂 overflow/shrink/clamp/tab 交互、完整 native/SDF paragraph parity、真实 shaping cluster backend 数据与竖排布局。

- `zircon_editor` retained-host fallback raster:2026-07-04 继续处理同一用户截图中“字体已是等线但字符左右间距/落点仍怪”的观感问题。布局桥已保证 host/runtime advances 与 shaped positions fail-closed 后,本轮确认更低层 fontdue fallback alpha mask 仍未消费 pen-origin subpixel phase,会让小字号 tab/file labels 在整数像素采样上显得忽左忽右；`draw/glyphs.rs`/`raster.rs`/`draw/glyphs/row.rs` 现在把 retained placement phase 以 8x/8-bin 传到 fallback downsampling,把最大 retained-host origin 量化误差收窄到 0.0625px。该修复不改变 `layout_text` 算法、ZUI 控件字体族、root painter 或 runtime atlas owner,仍需 live editor window typography QA 与 focused Cargo 绿跑。

- `zircon_editor` retained-host system-ui fallback stack:2026-07-05 继续收窄同一 crop 的字体 identity 风险。默认 `system-ui` / `ui-sans-serif` 不再只交给 fontdb generic `Family::SansSerif`,而是在 `paint_text/font.rs` 的字体 owner 中按 `DengXian`、本地化 `等线`、`Microsoft YaHei UI`、`Segoe UI`、`Family::SansSerif` 查询,保证 retained-host raster 与 runtime measure 更稳定地落到同一 editor UI face。显式 `sans-serif`/`monospace` 等 generic 仍保持原义,不新增控件局部字体、ZUI token 分支、root painter shortcut 或字距常量。验证图/日志:`docs/tests/runtime/text/runtime_text_editor_retained_system_ui_fallback_stack_preview_20260705.png` SHA256 `285C542709C091DBC29D7DB6C82BBEEBD627B4792B0DF8E9CE33E8F5AE9C25F0` / `runtime_text_editor_retained_system_ui_fallback_stack_validation_20260705.log` SHA256 `453143764C1F4C7237AC821E6DA789926CD853A4F39BC255A9FDB7FF6334D1FF`;Cargo 编译日志 `runtime_text_editor_retained_system_ui_fallback_stack_cargo_test_20260705.log` SHA256 `CF682BDA40453E91ED4AFF7AE3434A9297B7404DB4DC33F34F60A3BF66375531`。focused Cargo 在 904s Windows 编译窗口超时且无 Rust diagnostics/test summary,不计 green。

- `zircon_editor` retained-host framebuffer ink spacing guard:2026-07-04 在同一截图问题上新增 framebuffer 级回归,不只看命令录制或示意图。`paint_text_tests.rs::retained_text_editor_crop_labels_keep_stable_ink_spacing` 直接绘制 `editor base.zui` 与窄 `folder-open.svg`,扫描真实 `HostRgbaFrame` 的 ink left edge、ink center、painted pixel count 与 internal empty columns,并比较 8.875px/8.925px 近起点,防止布局桥和 retained raster phase 修复后仍出现可见左右跳。该守卫不替代 live editor window typography QA,focused Cargo 仍需空闲 lane 绿跑。

- `zircon_editor` retained-host shaped-origin latest-crop follow-up:2026-07-04 根据当时 editor crop 复查后,临时移除 `shaped_positions_preserve_retained_raster_bins(...)` shaped-position 专用否决,让 glyph id/source range/advance/monotonic 检查都匹配的 `ShapedGlyph` pen origin 继续被接受。该记录现在是历史状态:2026-07-05 的 shaped-origin phase fallback 已重新收束为 same-phase shaped origin 继续接受、cross-phase shaped origin 回退 host natural spacing。runtime-advance projection 的 `runtime_advances_preserve_retained_raster_bins(...)` 累计守卫始终保留。历史验证图/日志: `docs/tests/runtime/text/runtime_text_editor_retained_shaped_origin_spacing_latest_crop_preview_20260704.png` / `runtime_text_editor_retained_shaped_origin_spacing_latest_crop_validation_20260704.log`。

- `zircon_editor` retained-host shaped-origin phase fallback:2026-07-05 针对用户最新局部截图中“等线已生效但字符左右间距/渲染位置仍偏左或偏右”的剩余问题,确认 matched `ShapedGlyph` origin 仍可能跨越 retained 1/8px raster phase。`paint_text/draw/layout.rs` 的 shaped-position 接收路径现在通过 `shaped_positions_preserve_retained_raster_bins(...)` 比较 host natural origin 与 runtime shaped origin；same-phase shaped origin 继续作为 layout authority,跨 phase 的 origin fail-closed 到 host natural spacing。`draw/layout/tests.rs` 将旧跨相位接受用例改为拒绝,并保留 same-phase shaped-position acceptance。验证图/日志: `docs/tests/runtime/text/editor_text_retained_phase_guard_preview_20260705.png` / `editor_text_retained_phase_guard_validation_20260705.log`。

- `zircon_editor` retained-host same-phase origin drift guard:2026-07-07 根据用户最新 editor crop 继续复核后,确认仅用 1/8px same-phase 判断仍会放行 `0.04px~0.05px` 的局部 shaped/runtime origin 借位；在 DengXian/等线小字号 tab/file label 中,这类偏移虽未跨 raster bin,仍可能表现为单字左右不适。`paint_text/draw/layout/metrics.rs` 新增 `glyph_origin_matches_without_visible_drift(...)`,并让 shaped-position 接收路径与 runtime-advance projection guard 同时要求 same-phase 与 <= `0.03125px` 可见漂移阈值；`draw/layout/tests.rs` 保留 subvisible same-phase acceptance,新增 same-phase visible drift reject。该切片不新增 letter-spacing、控件局部字体、ZUI token、root painter、runtime FontDatabase 或 atlas routing。验证日志: `docs/tests/runtime/text/runtime_text_editor_retained_same_phase_origin_drift_guard_validation_20260707.log` SHA256 `7193440149FA3C4FB4CC384E0905769C93D398050968D1B0349899946943FA36`;随后 direct editor test binary 运行 same-phase proof 通过 1/1,真实 retained framebuffer/full-label/narrow-label PNG 已写入 `docs/tests/runtime/text`,SHA256 `8C81D6D27699ED503196F146636A3CF7EB51D202FF4E933AC96E6D1F17BD4E83` / `1C33579842EE9D0A912695219CDDA508BF247703151729C60A8EC93AD5365128` / `83B6CFDE5EAC92A9D2E349C605630484BC1FB5C3DA059F99D508D20B0E443339`;direct log SHA256 `542818B5FF18C3ABADE5A1A8713F11062C16CCFBE3B019728CF1886EDAD06ED5`,acceptance log SHA256 `B9BB847E9E8EB857F24619B2BDC40F71D7782D47E738BC05FDA141A2D230AF4D`;target/cargo-target 同名截图扫描为 0。Cargo wrapper proof 三次仍未产生 `test result`,不声明 Cargo green。
- `zircon_editor` retained-host proof stem hook:2026-07-07 为上面的 same-phase guard 准备独立 fresh proof PNG 归档,`paint_text_tests.rs::export_editor_crop_framebuffer_if_requested()` 新增 `ZR_TEXT_EDITOR_CROP_PROOF_STEM`。`ZR_TEXT_EDITOR_CROP_PROOF_DIR` 仍控制是否写文件,默认 stem 继续兼容旧 20260705 crop evidence；设置 stem 后可在 `docs/tests/runtime/text` 写出本切片专属 framebuffer、full-label crop、narrow-label crop 与 log,避免覆盖旧图。该切片只改测试证据命名,不改变 runtime layout、字体、ZUI、root painter、FontDatabase、atlas 或局部 letter-spacing。验证日志: `docs/tests/runtime/text/runtime_text_editor_retained_same_phase_origin_drift_guard_proof_stem_hook_validation_20260707.log` SHA256 `A7936DC9ADD771338B9852F6FEC819821DD1334630E65247C6609E039A6D4F48`;focused Cargo proof 三次尝试均未产生 `test result`,最新 rerun2 日志 SHA256 `6471B3A85089FD72C3B8EC752306C4860A60E428E469FBC97DEF0731EBFE2F3A`;direct binary proof 已使用 same stem 写出 PNG/log,目标目录为 `docs/tests/runtime/text`,target/cargo-target 同名截图扫描为 0(scan SHA256 `39736FD9FC4F493629257F0F73F6CFD8CD129D120A955ABA3E0E0D6F3D0E9B95`)。

- `zircon_editor` retained-host font collection index propagation:2026-07-07 继续处理用户指出的“等线已生效但字符左右间距/渲染位置仍偏左或偏右”。根因复核发现 `fontdb::FaceInfo.index` 只用于选中系统字体,但 `fontdue::FontSettings` 与 `swash::FontRef::from_index(...)` 仍默认 face 0；对 `.ttc` 字体集合会造成 runtime family/name 与实际布局/栅格化 face 不一致,表现为 glyph advance、bearing 与字符重心不舒服。`paint_text/font.rs` 现在把 fontdb face index 存入 `HostTextFont.collection_index`,传给 fontdue `collection_index`,并写入 font cache key；`paint_text/raster.rs` 用同一 index 创建 swash `FontRef`；`font/tests.rs` 锁定 collection index settings 与 cache-key 分离。该切片不新增 letter-spacing、控件局部字体、ZUI token、root painter、runtime FontDatabase 或 atlas route。direct editor test binary 运行 `retained_text_editor_crop_labels_keep_stable_ink_spacing` 通过 1/1,新的 framebuffer/full-label/narrow-label PNG 写入 `docs/tests/runtime/text`,SHA256 `8C81D6D27699ED503196F146636A3CF7EB51D202FF4E933AC96E6D1F17BD4E83` / `1C33579842EE9D0A912695219CDDA508BF247703151729C60A8EC93AD5365128` / `83B6CFDE5EAC92A9D2E349C605630484BC1FB5C3DA059F99D508D20B0E443339`;metrics log SHA256 `6DA9022798A3DC8C0E24FA4DA8F3465F945F875DF556A53CB59FE5053C62487E`,direct log SHA256 `ED7F8435FF0344BA915C1B71B571C559048463E7DFC64581DDE3CD90366DCCEE`;target/cargo-target 同名截图扫描为 0(scan SHA256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`)。当前机器导出的 proof 像素与 same-phase guard 一致,说明本机选中的 face 很可能已是 index 0；本切片关闭的是非 0 collection face 的错面风险。focused Cargo wrapper 两次超时停在编译/警告阶段,不声明 Cargo green。

- `zircon_editor` retained-host nearest-phase quantization:2026-07-07 继续复核用户最新 crop 中“字形左右间距怪、位置像偏左/偏右”的剩余观感。7 月 6 日 high-phase in-pixel clamp 已作为历史策略保留,但当前有效修复改为完整 screen x 的 1/8px 最近点量化:`paint_text/draw/placement/metrics.rs` 先对 `screen_x * 8` 四舍五入,再用 Euclidean div/rem 拆分 pixel 与 bin,因此 `20.95px`/`44.95px` 进入下一 pixel 的 `0/8` phase,而不是被拉回当前 pixel 的 `7/8`。该修复不新增 letter-spacing、控件局部字体、ZUI token、root painter、runtime FontDatabase 或 atlas route。验证证据写入 `docs/tests/runtime/text/runtime_text_editor_retained_phase_quantization_*`;本轮 recheck wrapper harness 通过 6/6,proof PNG 已人工复核,同名 target/cargo-target PNG 扫描为 0；更宽 Cargo screenshot rerun 在 Windows 编译 904s 后超时,不声明 package Cargo green。

- `zircon_editor` retained-host grayscale device glyph placement:2026-07-08 按用户最新局部截图继续收敛默认灰度编辑器标签的“等线已生效但 glyph 左右相位仍忽左忽右”观感。默认 `HostTextSmoothing::Grayscale` 现在只保留 Alpha coverage,但 glyph placement 改为 nearest device pixel: `RetainedGlyphPlacement::from_device_pixel_x(...)` 输出整数 pixel 与 `0.0` sample offset；显式 `HostTextSmoothing::Subpixel` 继续使用完整 screen x 的 1/8px 最近相位。`runtime_advances_preserve_retained_raster_bins(...)` 因此在默认灰度下按 device-pixel bin fail-closed,避免 `20.49px` 与 `20.51px` 这类跨像素边界的 runtime/shaped projection 继续进入 retained draw。该切片不新增 letter-spacing、控件局部字体、ZUI token、root painter、runtime FontDatabase 或 atlas route；它只收窄 retained-host 默认灰度小字号的最终落点策略。scoped rustfmt/diff check 已通过,直接运行已编译 editor test binary 通过 1/1 并把 retained framebuffer proof 写入 `docs/tests/runtime/text/runtime_text_editor_retained_grayscale_device_glyph_snap_20260708*.png`;focused Cargo wrapper 仍未单独声明 green。

- `zircon_editor` retained-host grayscale line-snap subpixel glyph phase:2026-07-08 根据用户最新 editor crop 复查后,上面的 default grayscale per-glyph device-pixel placement 成为历史尝试。根因是逐字整数像素取整会把自然 fractional advances 改成忽宽忽窄的整数步进,使 DengXian/等线小字号 label 看起来左右间距不舒服。当前有效规则恢复为:默认 `HostTextSmoothing::Grayscale` 只把 line origin 吸附到 nearest device pixel,每个 glyph 仍走完整 screen x 的 retained 1/8px phase；显式 `HostTextSmoothing::Subpixel` 继续保留 fractional line origin + 1/8px glyph phase。`draw/placement.rs` 不再保留 `from_device_pixel_x(...)`,`draw/layout/tests.rs` 新增 `retained_text_run_keeps_subpixel_glyph_phases_for_grayscale_smoothing` 锁定 `editor base.zui` 至少保留 3 个非零 1/8 phase。该切片不改字体族、letter-spacing、ZUI token、root painter、runtime FontDatabase、atlas route 或组件局部样式；proof PNG 写入根目录 `docs/tests/runtime/text/runtime_text_editor_grayscale_line_snap_subpixel_glyph_phase_20260708*`,误写到 `zircon_editor/docs/tests/runtime/text` 的同名文件已清理。

- `zircon_editor` retained-host pen-origin raster-bearing correction:2026-07-05 继续处理最新 editor crop 中“等线已生效但字符左右间距/渲染位置仍偏左或偏右”的低层落点问题,并收窄 2026-07-04 的 bitmap-left handoff。`paint_text/draw/glyphs.rs` 的正常绘制路径重新以 `RuntimeTextGlyph.origin_x` 对应的 layout pen origin 为 placement authority,再叠加当前 raster backend 的 `metrics.x_offset` bearing；`RuntimeTextGlyph.x` 只保留为 invalid-origin fallback。这样 matched shaped origin、line-origin snap、resolved family 与 swash/fontdue 当前 raster bearing 不再互相覆盖。验证图/日志: `docs/tests/runtime/text/runtime_text_editor_retained_pen_origin_raster_bearing_preview_20260705.png` / `runtime_text_editor_retained_pen_origin_raster_bearing_validation_20260705.log`。

- `zircon_editor` retained-host layout bitmap-left authority reopen:2026-07-05 用户继续用最新 editor crop 指出等线已生效后字符左右间距仍怪,因此重新复查上一条 pen-origin+raster-bearing 纠偏。该结论现在作为历史状态保留,已被同日 raster-bearing alignment reclose supersede。当时规则曾调整为:正常路径分离两种 authority,`origin_x` 只用于 retained raster phase/subpixel sample,有限 `RuntimeTextGlyph.x` 继续作为最终 bitmap-left draw pixel；只有 layout `glyph.x` 非 finite 时才回退 `origin_x + raster.metrics.x_offset`。`paint_text/draw/glyphs.rs`/`draw/glyphs/tests.rs` 曾按该规则更新,不改变 ZUI 字体 token、root painter、runtime FontDatabase 或 atlas routing。验证图/日志: `docs/tests/runtime/text/runtime_text_editor_retained_layout_bitmap_left_authority_preview_20260705.png` SHA256 `1256315A761B5815D1E816607C8ACA16102C354FC3B461650BDEDF80497AF46D` / `runtime_text_editor_retained_layout_bitmap_left_authority_validation_20260705.log` SHA256 `23AFB4FC698A2C5E305867AD1308D057E20AAF94999521F62FEC33E413A7AB90`;target/cargo-target 同名扫描为 0。focused Cargo 因外部 cargo/rustc lanes 活跃 deferred,live editor window typography QA 仍 open。

- `zircon_editor` retained-host raster-bearing alignment reclose:2026-07-05 用户最新局部截图继续显示等线小字号标签的字符左右落点不舒服,说明有限 `RuntimeTextGlyph.x` 作为最终 bitmap-left authority 仍会把 layout bearing 与实际 raster bearing 的差异投到屏幕上。当前生效规则恢复为 `RuntimeTextGlyph.origin_x` 正常持有 pen-origin placement authority,最终 bitmap-left draw pixel 使用 `origin_pixel_x + raster.metrics.x_offset`;`RuntimeTextGlyph.x` 仅保留 invalid-origin fallback。新增 `retained_glyph_bitmap_pixel_x_ignores_stale_layout_left_when_origin_is_valid` 锁定 stale layout-left 不能覆盖有效 origin。验证图/日志: `docs/tests/runtime/text/runtime_text_editor_retained_raster_bearing_alignment_preview_20260705.png` SHA256 `7B66F057AA95412E65BDE04CF9908C42D47E3CD3598EE041622E23F5AB4A663B` / `runtime_text_editor_retained_raster_bearing_alignment_validation_20260705.log` SHA256 `255DCCF5EA9B559595EB0A50BADF2A36C04BB1515894DF247977D38857A763C2`;focused Cargo `retained_text_editor_crop_labels_keep_stable_ink_spacing` 第三轮通过 1/1,并导出真实 framebuffer/crop PNG 到 `docs/tests/runtime/text`;target/cargo-target 同名扫描为 0。

- `zircon_editor` retained-host document tab clip-guard centering:2026-07-05 最新截图继续显示 `editor base.zui` / `folder-op...line.svg` 这类页签文本虽然已走等线,但图标+标题整体视觉中心仍像被拉偏。复核后确认 Button 内容层把 `text_clip_guard` 加进 label measured width 后参与 icon+title centering,导致真实文字墨迹被半个 guard 推离 tab center。`template_buttons/content/metrics.rs` 现在拆分 `measured_label_ink_width(...)` 与 `label_text_slot_width(...)`;`content/entry.rs` 只用 icon+title ink width 参与居中,把 clip guard 留在 text command slot width；`template_buttons_tests/paint.rs::document_tab_button_centers_icon_and_title_ink_without_clip_guard_bias` 锁定 `DockTab0` + `folder-open-outline` + `editor base.zui` 路径。该切片不改字体 token、root painter、runtime FontDatabase、glyph atlas 或 text raster owner。验证图/日志: `docs/tests/runtime/text/runtime_text_editor_tab_clip_guard_centering_preview_20260705.png` SHA256 `33B58720DB6662D1F6455EB5C40676239C5A8505AC5D8C8B13AA49E13CC6E476` / `runtime_text_editor_tab_clip_guard_centering_validation_20260705.log` SHA256 `D0E62EAF9A757D0C6680FE3CB3F3E6799E0ADFBE396535478278EC4BE28A771D`;target/cargo-target 同名扫描为 0。focused Cargo 因外部 cargo/rustc lanes 活跃 deferred,live editor window typography QA 仍 open。

- `zircon_editor` retained-host Fontdue bearing-fraction sampling:2026-07-05 继续收窄同一最新 editor crop 中“字符左右间距/渲染位置仍偏左或偏右”的 fallback raster 残留问题。前一切片已让正常绘制回到 `origin_x + raster metrics.x_offset`,但 FontdueFallback 自身仍只把 pen-origin phase 传给 downsampling,高分辨率位图的 fractional bitmap-left bearing 被整数 draw origin 吃掉。`paint_text/raster.rs::fontdue_fallback_sample_offset_x(...)` 现在把 pen-origin phase 与 bitmap-left fraction 合并为 `sample_offset_x`;`draw/glyphs.rs` 与 `draw/glyphs/row.rs` 允许组合 sample offset 超过 1px,避免下采样又夹回 0.999 重新产生 floor bias。验证图/日志: `docs/tests/runtime/text/runtime_text_editor_retained_fontdue_bearing_fraction_preview_20260705.png` / `runtime_text_editor_retained_fontdue_bearing_fraction_validation_20260705.log`。

- `zircon_editor` retained-host subpixel line-origin preservation:2026-07-05 继续补齐显式 Subpixel/LCD background composite 的 layout 前置条件。`draw/placement.rs::retained_text_origin_for_smoothing(...)` 最初让默认 grayscale 仍把 line origin 吸附到 nearest device pixel,但显式 subpixel 保留 finite fractional line origin;`draw/layout.rs` 的生产入口只读取当前 `HostTextSmoothing` 并传给该 owner。随后同日 grayscale fractional-origin follow-up 曾短暂让默认 grayscale 也切到 finite fractional line origin,但 2026-07-06 已恢复默认 Grayscale device-origin snap；本条现在只是 smoothing-aware line-origin owner 的历史前置证据。验证图/日志: `docs/tests/runtime/text/runtime_text_editor_retained_subpixel_line_origin_preview_20260705.png` / `runtime_text_editor_retained_subpixel_line_origin_validation_20260705.log`。

- `zircon_editor` retained-host grayscale fractional line-origin:2026-07-05 这条记录现为历史尝试。后续用户 crop 继续显示 DengXian/等线小字号 tab/file label 的整段文字有左右漂移感,说明默认 grayscale 保留 fractional line origin 会把整行固定在小数 device origin 上,与当前 alpha coverage + 1/8px glyph phase 叠加后仍不舒服。2026-07-06 的 grayscale device-origin snap 已 supersede 该默认灰度策略；显式 Subpixel 仍保留 finite fractional line origin,非 finite 值仍归零。历史验证图/日志: `docs/tests/runtime/text/runtime_text_editor_retained_grayscale_fractional_origin_preview_20260705.png` / `runtime_text_editor_retained_grayscale_fractional_origin_validation_20260705.log`。

- `zircon_editor` retained-host grayscale device line-origin snap:2026-07-06 针对用户最新截图中“字体已是等线但字符左右间距/渲染位置仍偏左或偏右”的整段起点问题,默认 `HostTextSmoothing::Grayscale` 重新通过 `retained_text_origin_device_px(...)` 把 line origin 吸附到 nearest device pixel,而 `HostTextSmoothing::Subpixel` 继续保留 explicit fractional origin。每个 glyph 的 retained 1/8px alpha phase 仍保留；当前 2026-07-07 规则使用完整 screen x 的 nearest-phase quantization,所以这不是组件局部 font、letter-spacing、ZUI token、root painter、runtime FontDatabase 或 atlas routing 改动。`draw/placement/tests.rs` 锁定 Grayscale `8.875px -> 9.0px`、Subpixel `8.875px` 原样保留；`draw/layout/tests.rs` 锁定 8.875px 与 9.0px 灰度起点产生相同 glyph origins；`paint_text_tests/latest_crop.rs` 新增真实 `HostRgbaFrame` byte equality,证明 `editor base.zui` 在 44.875px 与 45.0px 默认灰度起点下输出同一像素。

- `zircon_editor` retained-host crop framebuffer export hook:2026-07-05 在同一 editor crop 问题上补齐验证输出链路。`paint_text_tests.rs::retained_text_editor_crop_labels_keep_stable_ink_spacing` 仍负责 `editor base.zui` 与 `folder-open.svg` 的真实 `HostRgbaFrame` ink profile 断言；本轮新增 `ZR_TEXT_EDITOR_CROP_PROOF_DIR` 环境变量钩子,让该 test 在需要时把 retained-host framebuffer PNG/log 导出到调用方指定目录。默认测试不写文件；2026-07-05 第三轮 focused Cargo 已通过 1/1,真实 test export 和汇总日志位于 `docs/tests/runtime/text/runtime_text_editor_retained_crop_export_cargo_passed_20260705.log`。

- `zircon_editor` retained-host crop framebuffer region guard:2026-07-05 继续收紧同一 editor crop 验证闭环。导出钩子证明可以保留完整 proof framebuffer,但断言仍需要覆盖 proof framebuffer 中实际 tab/file label 区域。本轮让 `retained_text_editor_crop_labels_keep_stable_ink_spacing` 扫描 `editor_crop_proof_framebuffer()` 的 full/narrow label regions,锁定 painted pixels 与最大内部空列,并在 opt-in export log 中写入 `proof_full_label` / `proof_narrow_label` profile。验证日志: `docs/tests/runtime/text/runtime_text_editor_retained_crop_framebuffer_region_guard_validation_20260705.log`；复用 preview PNG: `runtime_text_editor_retained_crop_framebuffer_export_preview_20260705.png`。

- `zircon_editor` retained-host crop region export PNGs:2026-07-05 继续把验证链路从“能导出整图”推进到“能导出可直接查看的文字局部”。`export_editor_crop_framebuffer_if_requested()` 现在在 `ZR_TEXT_EDITOR_CROP_PROOF_DIR` 设置时,从同一 `editor_crop_proof_framebuffer()` 写出完整 framebuffer、full-label crop 与 narrow-label crop 三张 PNG,并在 log 记录三份路径。默认测试仍无文件副作用；第三轮 focused Cargo 导出 `runtime_text_editor_retained_crop_framebuffer_20260705.png` SHA256 `69B95BB5E3AAFDA2B586C785F5AAD9361452605D6594653DD2CA663AAF750073`,`runtime_text_editor_retained_crop_full_label_20260705.png` SHA256 `B4F1C117C182F89AB59C449E1C6A4B0EEBC338128190A63A52CE56D8423B4628`,`runtime_text_editor_retained_crop_narrow_label_20260705.png` SHA256 `8AEE9DCCC0ECE7AAD53D5FC9C42C7494694801430CEDBF399558064609E0A324`;target/cargo-target 同名扫描为 0。

- `zircon_editor` retained-host latest crop framebuffer guard:2026-07-05 针对用户补充的最新截图中 `editor base.zui` 与 `folder-op...line.svg` 局部文字仍显左右间距/落点不舒服的问题,继续把同一目标链路落到可复查回归。`paint_text_tests/latest_crop.rs` 新增 `retained_text_editor_latest_crop_labels_keep_stable_ink_spacing`,复现最新双页签 crop 几何,扫描文字区域的 painted pixels、ink center 与 internal empty columns,并通过同一 opt-in proof 目录导出 `runtime_text_editor_latest_crop_framebuffer_20260705.png`、full-label crop 与 narrow-label crop。2026-07-06 复跑时按 `paint_text.rs` 的 `#[path = "paint_text_tests.rs"] mod tests;` 结构给 child owner 补显式 `#[path = "paint_text_tests/latest_crop.rs"]`,避免 Rust 子模块解析回落到错误目录；父 `paint_text_tests.rs` 现为 727 行,latest-crop child owner 为 197 行。该切片不改字体 token、ZUI 控件局部字距、root painter、runtime FontDatabase、glyph atlas 或 draw placement 生产规则；focused Cargo 已在 high-phase proof rerun 中通过并刷新 proof PNG,target/cargo-target 同名扫描为 0。

- `zircon_editor` retained-host high-phase no-rollover placement:2026-07-06 用户继续指出最新 crop 中 DengXian/等线小字号字符左右间距仍不舒服。该切片当时把上沿 phase clamp 在当前 pixel cell 的 `0.875` bin,不再执行 pixel rollover；`draw/placement/tests.rs` 锁定 `20.95px` 与 `44.95px` 不滚格,`paint_text_tests/latest_crop.rs` 也用 `44.95px` 覆盖旧滚格阈值附近的整段 label 稳定性。2026-07-07 复核发现该 clamp 会留下 high-phase 左/右偏置,当前已由 nearest-phase quantization supersede。该历史切片不改字体 token、root painter、runtime FontDatabase、atlas routing、layout advance 或容器居中策略；focused Cargo `retained_text_editor_latest_crop_labels_keep_stable_ink_spacing` 通过 1/1 并刷新 test-emitted proof PNG,静态验证日志位于 `docs/tests/runtime/text/runtime_text_editor_retained_high_phase_no_rollover_validation_20260706.log`,Cargo 复跑日志位于 `docs/tests/runtime/text/runtime_text_editor_latest_crop_high_phase_no_rollover_cargo_rerun3_20260706.log`。

- `zircon_editor` retained-host nearest phase rollover:2026-07-06 该记录最初被 no-rollover clamp supersede,但 2026-07-07 用户 crop 复核后,其核心策略已恢复为当前有效实现。当前 `draw/placement/metrics.rs` 通过 full screen x 的 1/8px 最近点量化让 rounded bin 到 8 时进入下一 pixel cell 的 `0.0` phase,并把最大量化误差限制在 `0.0625px`。历史验收日志 `runtime_text_editor_retained_nearest_phase_rollover_acceptance_20260706.log` SHA256 `619ECCF3EBD438CA8B70C16B1F99E88AA69F4357A037DBFC91A93390CDDB0FB7`；当前 2026-07-07 证据见 phase quantization 条目。

- `zircon_editor` retained-host SubpixelMask sample phase:2026-07-05 在同一小字号 label 左右落点问题上继续补低层行采样一致性。`draw/glyphs/row.rs::sampled_subpixel_coverage(...)` 现在对 supersampled RGB/SubpixelMask 下采样也应用 `sample_offset_x`,与 AlphaMask 的 1/8px phase/downsampling 语义一致；`sampled_subpixel_coverage_applies_fallback_phase` 锁定 offset `0.0` 与 `0.5` 的 RGB coverage 差异。该切片不改变默认 grayscale Alpha coverage、ZUI 字体族、root painter、runtime FontDatabase 或 atlas routing；验证图/日志为 `docs/tests/runtime/text/runtime_text_editor_retained_subpixel_sample_phase_preview_20260705.png` / `runtime_text_editor_retained_subpixel_sample_phase_validation_20260705.log`。

- `zircon_editor` retained-host same-style shaped cluster line guard:2026-07-05 继续收束最新 editor crop 中 `folder-op...line.svg` 类文件名标签的左右间距问题。此前 runless shaped text 为了保留 cluster paint style,会把同一行的 `folder-op`、省略号和 `line.svg` 拆成多条 retained-host text command；同样式片段因此各自重新 layout/raster,在小字号下可能制造片段边界的左右落点不连续。本轮在 `render_command_conversion/text/commands/shaped.rs::push_shaped_text_commands(...)` 前置 `uniform_cluster_text_style(...)`,同样式多 cluster 直接输出整行命令,混合 style cluster 仍按 cluster split。验证图/日志: `docs/tests/runtime/text/runtime_text_editor_retained_same_style_cluster_line_preview_20260705.png` / `runtime_text_editor_retained_same_style_cluster_line_validation_20260705.log`。

- `zircon_runtime` graphics text shaping glyph-offset px projection:2026-07-04 继续处理同一 editor crop 中“等线已生效但字符左右间距/渲染位置仍偏左或偏右”的底层 shaping 单位问题。`graphics/text/shaping/cosmic.rs` 现在把 glyphon/cosmic `LayoutGlyph.x_offset/y_offset` 通过 `glyph.font_size` 投影到像素空间后再写入 `ShapedGlyph.offset_x/y`,避免 retained-host 后续按像素使用一个仍是 em-relative 的偏移值,导致小字号 glyph pen origin 欠投影。该切片不改变 retained-host painter、ZUI 字体、runtime FontDatabase、glyph atlas 或 layout line-break 策略。

- `zircon_runtime` source-range unkerned measure backend request:2026-07-04 继续收束同一截图问题背后的字距语义分叉。`TextShapeRequest` 与 `ShapedGlyphRun` 现在携带 `include_kerning`,默认序列化保持 true；`graphics/text/shaping/mod.rs` 暴露 owner-local `shape_horizontal_line_with_kerning(...)`,`cosmic.rs` 在 false 时向 glyphon/cosmic attrs 写入 OpenType `kern=0`；`graphics/text/layout/measure.rs::measure_text_source_range_width_with_kerning(...)` 不再在既有 kerned run 上假装去 kerning,而是按请求重新 shape 后再测量 source range。该切片不改变默认绘制路径、ZUI 字体、retained-host painter、runtime FontDatabase 或 glyph atlas。

- `zircon_runtime` source-prefix absolute range geometry:2026-07-04 继续处理同一 editor crop 中字符左右定位不稳的 source geometry 分叉。`ui/text/geometry.rs::measured_source_prefix_width(...)` 现在从 `line.source_range` 派生绝对 `UiTextRange`,并把原始 `measure_context.text` 交给 backend source-range measure,不再用 `line.text.as_str()` 当作 0-based 临时字符串重测。该路径仍只覆盖 simple LTR、source-isomorphic、无 tab/ellipsis/justify 的保守可测行；复杂 BIDI/竖排、完整 cluster reverse lookup 与 live editor window QA 仍等待。

- `zircon_runtime` source metrics vertical fail-closed:2026-07-04 在同一 source geometry 路径上补上 writing-mode 边界。`ui/text/geometry.rs::line_accepts_source_measure(...)` 现在只允许 `UiTextWritingMode::HorizontalTb` 消费 source-range shaped width,`VerticalRl` 继续使用已解析 `glyph_advances` 推进 caret/selection y,避免在竖排 backend source-range 几何完成前把水平测量宽度误喂给竖排主轴。该切片不实现完整竖排 source-range 几何,只关闭错误横向 fallback 泄漏。

- `zircon_runtime` surface render measure-cache consumer:2026-07-04 关闭 `measure_cache.rs` “存在但生产渲染提取未消费”的缺口。`UiSurface` 现在持有 serde-skip 的 `UiTextMeasureCache`,`rebuild_render_extract(...)` 每帧 `begin_frame()` 后把缓存传入 render extract；owner text 与 TextField 的 layout request 都走 `resolve_text_layout_with_cache(...)`。同时因 `UiResolvedTextLine.frame` 保存绝对坐标,`UiTextMeasureKey` 纳入 exact `frame`/`clip_frame` bit key,只允许同一布局输入跨 rebuild 命中,禁止不同节点位置复用绝对线框。该切片不声明完整 09 性能缓存体系、layout-pass measure cache 或 live editor-window QA 完成。

- `zircon_runtime` frame-dedup production routing:2026-07-06 继续补上 09/PF-M1 cache data-plane 到生产 UI adapter 的接线。`UiTextMeasureCache` 现在持有 natural-size 与 full-layout 两张 `TextFrameDedup` 表,在 persistent measure/layout cache 前按 exact key + exact text 复用同帧重复结果；`resolve_or_shape(...)` 返回 owned `UiTextLayoutResolution`,render extract 只消费 adapter 输出。该切片关闭重复请求同帧去重接线,不声明 measure 闭包与 full layout 共用 shaped-run、计数式 perf test、复杂 source-range 几何或 live editor-window QA 完成。

- `zircon_runtime` IME render-extract geometry consumer:2026-07-04 继续处理编辑器截图中“小字号文本左右落点/字距不舒服”的同源几何问题。`ui/surface/input/editable_text/ime_context.rs` 不再为输入法光标/合成框单独构造 `UiTextLayoutRequest` 和 direct `resolve_text_layout(...)`;当 IME preedit/commit/text/keyboard 事件需要刷新上下文时,先调用 `UiSurface::refresh_render_extract_for_current_tree()`,再从同一 target 的 `UiRenderCommand.text_layout` 读取 paint 已用的 TextField layout。layout 不存在时保留 fixed-column fallback,但正常路径的 paint/caret/composition 现在同帧同源。

- `zircon_runtime` Dialog action runtime measure:2026-07-04 继续清理 runtime surface render 侧仍会制造同字符数不同字形左右留白不一致的路径。`ui/surface/render/dialog.rs::action_width(...)` 删除 `text.chars().count() * DIALOG_ACTION_CHAR_WIDTH + 20` 估算,改为用 runtime `measure_text_size(...)` 加显式 `DIALOG_ACTION_TEXT_PADDING_X` 计算 action slot 宽度,保留 `DIALOG_ACTION_MIN_WIDTH`。`ui/tests/render_dialog.rs` 新增同字符数 `iiiiiiiiiiii` / `WWWWWWWWWWWW` action frame 差异回归。

- `zircon_editor` retained-host diagnostics/text marker runtime measure:2026-07-04 继续处理编辑器截图中同字符数不同 glyph advance 导致左右留白不均的问题。`paint_diagnostics/marker.rs` 删除 `APPROX_GLYPH_WIDTH` 估宽,debug-refresh marker frame 改为 `measure_runtime_text_width(label, MARKER_FONT_SIZE)` + 既有 padding/clip；`paint_primitives/text_markers.rs` 删除 `text.chars().count() * 8.0` text-bar frame 估算,改用同一 retained runtime text width helper。新增同字符数 `iiiiiiiiiiii` / `WWWWWWWWWWWW` 诊断 marker 与 text-bar frame 差异回归。该切片不改 ZUI 字体、root painter、runtime FontDatabase、glyph atlas 或 live editor-window QA。

- `zircon_editor` retained-host line-origin snap:2026-07-04 针对同一 editor crop 中整段小字号 label 在 `8.875px` 等小数起点上看起来偏左/偏右的问题,retained-host 曾与 SDF `text_frame_device_origin(...)` 策略对齐,在 `layout_text_run(...)` 入口把整行起点贴到设备像素。2026-07-05 曾尝试让默认 grayscale 也保留 finite fractional line origin,但 2026-07-06 用户最新 crop 已把默认 Grayscale 重新收束为 device-origin snap；显式 Subpixel 仍保留 fractional origin。本条只保留为历史取舍记录与当前策略来源。

- `zircon_editor` retained-host resolved font-family projection:2026-07-04 继续收束同一 editor crop 中“等线已生效但字距/落点仍不舒服”的根因。`paint_text/font.rs::runtime_text_style_for_face(...)` 现在先通过 retained-host font cache 解析实际 host face,再把 `HostTextFont.runtime_family` 写入 runtime `UiResolvedStyle.font_family`,避免 layout/shape 仍拿 `system-ui` 这类请求 family 而 raster 已使用 DengXian/等线实际 face。该切片不改变 ZUI 字体偏好、root painter、runtime FontDatabase 或 atlas routing；它只保证 retained-host runtime measurement/shaping 与 retained software raster 选择同一个已解析 family。

- `zircon_editor` retained-host render-command alignment resolved-family measurement:2026-07-04 继续处理同一 crop 中右对齐/居中 compact label 看起来偏左偏右的问题。`paint_template_nodes/render_command_conversion/style/text.rs::aligned_text_x(...)` 现在先把 command `UiResolvedStyle` 通过 `text_paint_style_from_resolved_style(...)` 和 `runtime_text_style_for_face(...)` 投影到 retained-host 已解析 font family,再调用 runtime `measure_text_size(...)` 计算对齐起点,避免 command style 的泛型/requested family 与实际 glyph paint family 分叉。该切片不改变组件 ZUI、root painter、runtime FontDatabase、atlas routing 或 glyph raster；它只让 render-command 对齐测量与 retained glyph paint 使用同一字体身份。

- `zircon_editor` retained-host divider label measured gap:2026-07-04 继续扫同类“字符左右留白/位置看起来偏”的生产路径,确认 Material Divider 横向 label gap 仍用 `label.chars().count() * font_size * 0.56` 估宽,并且 bounds 字号来源与实际 label text frame 不一致。`paint_template_nodes/material_primitives/divider/geometry/label_bounds/horizontal.rs` 现在用 `measure_runtime_text_width(label, divider_font_size(node, rect.height))` 计算 gap 宽度,`divider/horizontal.rs` 把绘制 rect 传入 bounds,让断线 gap、文本 frame 和 retained runtime text measurement 使用同一个 painted font size。该切片不改变 ZUI 资产、root painter、runtime FontDatabase、glyph atlas 或组件局部字体策略。

- `zircon_editor` retained-host chip label runtime measure:2026-07-04 继续处理用户 crop 中等线已生效但 compact label 左右间距仍不舒服的问题,确认 Material Chip label frame 仍用 `label.chars().count() * font_size * CHIP_LABEL_WIDTH_RATIO` 估宽。`paint_template_nodes/material_primitives/chip/geometry/label.rs` 现在用 `measure_runtime_text_width(label, chip_font_size(node))` 计算 label frame 宽度,保留原有 left/right padding、avatar/icon/delete slot 与 available-width clamp；`metrics.rs` 删除旧 `CHIP_LABEL_WIDTH_RATIO` 常量,避免后续生产路径复用字符数比例。该切片不改变 ZUI 资产、root painter、runtime FontDatabase、glyph atlas 或组件局部字体策略；剩余 Material Alert/Avatar/Badge char-count 估宽随后由下一切片收束。

- `zircon_editor` retained-host remaining material primitive runtime measure:2026-07-04 同一问题继续收束到剩余 Material primitive owner。Alert message、Avatar text、Badge root text 与 Badge overlay frame/text 现在全部调用 `measure_runtime_text_width(...)` 计算宽度,并删除 `ALERT_TEXT_WIDTH_RATIO`、`AVATAR_TEXT_WIDTH_RATIO`、`BADGE_ROOT_TEXT_WIDTH_RATIO`、`BADGE_TEXT_WIDTH_RATIO`。该切片保留各 owner 原有 action/icon padding、avatar center alignment、badge padding/min-width、overlay anchor 与 clamp 规则；生产 Material primitive `TEXT_WIDTH_RATIO`/`LABEL_WIDTH_RATIO` 与 `chars().count()*font_size` 估宽扫描已归零,仅测试夹具保留旧公式作反例。

- `zircon_editor` retained-host dialog action runtime measure:2026-07-04 继续扫非 Material template node 的同类生产路径,确认 `template_dialogs/actions/labels.rs::action_width(...)` 仍用 `text.chars().count() * DIALOG_ACTION_CHAR_WIDTH + 20` 估算 action 文本槽宽。该 owner 现在使用 `measure_runtime_text_width(text, DIALOG_ACTION_FONT_SIZE)` 加显式 `DIALOG_ACTION_TEXT_PADDING_X` 和 host `text_clip_guard` 计算宽度,并删除旧 `DIALOG_ACTION_CHAR_WIDTH`。按钮、列表、树行与 Asset Browser toolbar/tab button 入口已确认不走字符数估宽；本切片不改 ZUI 资产、root painter、runtime FontDatabase、glyph atlas、组件局部字体策略或对话布局调用方。

- `zircon_editor` retained-host Document Tab runtime measure:2026-07-04 最新截图中 `editor base.zui` 与 `folder-open-line.svg` 指向 Workbench document tabs,其可视 frame 与 drag hitbox 仍分别使用字符数/ASCII 宽度估算。`workbench/document_tabs/metrics.rs` 现在改为 `document_tab_preferred_width_from_title_width(...)`,由 caller 传入 retained-host runtime text 已测量的 title width；`chrome_template_projection/dock_header.rs` 使用 `measure_runtime_text_width(..., DOCUMENT_TAB_TITLE_FONT_SIZE)` 投影可视 tab；`retained_host/tab_drag/tab_width.rs` 删除 `estimate_text_width`/`ascii_char_width` 并复用同一 runtime measurement,使 paint frame、close button 与 drag midpoint 在 DengXian/等线宽度下同源。

- `zircon_editor` retained-host Host Page Tab runtime measure:2026-07-04 继续扫顶部主页面页签,确认 `workbench/page_tabs/metrics.rs` 仍以 `title.chars().count() * TITLE_WIDTH_PER_CHAR` 估算 page tab 宽,且 retained host page pointer 以可用空间均分 hitbox。`page_tabs/metrics.rs` 现在改为 `main_page_tab_preferred_width_from_title_width(...)`;`chrome_template_projection.rs` 用 `measure_runtime_text_width(..., MAIN_PAGE_TAB_TITLE_FONT_SIZE)` 计算可视页签宽；`host_page_pointer/tab_strip_geometry.rs` 通过 `HostPagePointerItem.title` 使用同一 runtime measurement 分配 shared hitbox,使 paint frame、overflow 判定与 pointer route 同源。

- `zircon_editor` retained-host Workbench Menu Bar runtime measure:2026-07-04 继续扫顶部菜单栏,确认 `chrome_template_projection/menu_chrome.rs` 与 `retained_host/menu_pointer/build_host_menu_pointer_layout.rs` 仍以 `label.chars().count() * 7.0 + 24.0` 估算 menu slot width。新增 `workbench/menu_bar/metrics.rs` 作为共享 owner,由 visual menu chrome 与 retained pointer hitbox 同时消费 `measure_runtime_text_width(..., WORKBENCH_MENU_SLOT_FONT_SIZE)` 后的宽度,使 `iiiiiiii` / `WWWWWWWW` 这类同字符数不同 glyph advance 的标签不再被分配同一槽宽。

- `zircon_editor` retained-host Asset Browser file-name runtime compaction:2026-07-04 继续处理截图中 `editor base.zui`、`folder-open-line.svg` 这类文件名标签的左右间距观感。`asset_browser/name_compaction.rs` 新增为共享 file-like display-name compaction owner,由 `summary_nodes.rs`、`thumbnail_nodes.rs` 与 `table_nodes.rs` 消费 `measure_runtime_text_width(...)` 后决定是否保留完整名称或使用 `prefix...tail.ext`;旧 visible char limit/name char budget 不再决定紧凑文件名,同字符数的 `iiii...` 与 `WWWW...` 会按真实 glyph advance 分流。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Fonts/FontCache.h` | `FShapedGlyphSequence::GetMeasuredWidth`/`GetMeasuredWidth(Start,End,bIncludeKerning)`/`GetGlyphAtOffset`(像素偏移→字形,带边界):**本计划度量与命中测试主样板**;`FShapedGlyphSequence` 的 `TextBaseline`/`MaxTextHeight`/`SourceTextRange` |
| `dev/UnrealEngine/.../Fonts/FontMeasure.cpp/.h` | `FSlateFontMeasure::MeasureStringInternal`/`Measure`/`FindLastWholeCharacterIndexBeforeOffset`:子串度量与 offset 反查的实现细节 |
| `dev/UnrealEngine/.../Framework/Text/TextLayout.h` | `FTextLayout`:`ETextJustify`、`ETextWrappingPolicy`、`FTextLayout::FlowDirection`、line view/run/block 的布局模型、`CreateWrappingCache`、`MarginAndJustification` |
| `dev/godot/servers/text/text_server.h` | `AutowrapMode::{OFF,ARBITRARY,WORD,WORD_SMART}`、`LineBreakFlag`、`OverrunBehavior::{TRIM_CHAR,TRIM_WORD,TRIM_ELLIPSIS,...}`、`JustificationFlag::{KASHIDA,WORD_BOUND,TRIM_EDGE_SPACES,...}`——换行/溢出/两端对齐枚举权威 |
| `dev/godot/modules/text_server_adv/text_server_adv.cpp` | `shaped_text_get_line_breaks`(ICU `ubrk_*` 行断)、`shaped_text_fit_to_width`(kashida + 字间 justify)、CJK 禁则——本计划换行与 justify 算法对照 |
| `dev/slint/.../textlayout/{linebreaker,linebreak_unicode}.rs` | Rust UAX#14 行断器 + 简单回退:断点机会编码、贪心断行循环——落地首选参照 |
| `dev/Fyrox/fyrox-ui/.../formatted_text/textwrapper.rs` | 极简 `TextWrapMode::{NoWrap,AtWidth,ByWords}` + 空白修剪 Rust 实现 |

**Rust/wgpu 落地**:`unicode-linebreak`(UAX#14 break opportunities,cosmic-text 内置)、cosmic-text `Buffer::set_size` + `layout_runs`(已做断行+对齐,可直接消费其行结果)。CJK 禁则与 justify 在 cosmic-text 结果之上后处理。(2026-07-02 评审收口:按 D1 改判——`layout_runs` 仅作对拍参考,不进入生产链路,见 §4/§7。)

## 4. 目标架构

```
ShapedGlyphRun(02, 无宽度约束 + 断点机会) + LayoutConstraints { wrap_width, wrap_mode, align, justify, overflow, tab, orientation }
  └─ line_break(UAX#14 机会 + CJK 禁则 + 连字符) → 贪心/逐字断行 →
     measure(真实 advance/kerning,子范围可查) → align/justify(行内分布) →
     overflow(ellipsis/shrink/clamp) → LaidOutText { lines[LaidOutLine], size, baseline 表 }
```

度量与布局分两层:**measure-only**(taffy 测量闭包用,只算尺寸,走 shaped+measure 缓存,不产顶点)与 **full layout**(产 `LaidOutText`,含每字形定位)。两者共享 shaping 与断行,measure 短路在 size 计算后(接 `editor_layout/17` 两阶段 + `render/14` "measure 必须走 shaped cache")。

**换行归属裁决(2026-07-02 评审收口,D1)**:02 只交付"无宽度约束整形+断点机会标注",不产出行;行切分/重排归 03 自研贪心断行;cosmic-text `Buffer::set_size`+`layout_runs` 仅作对拍参考。shaped cache 键不含 wrap。

**BIDI 行级视觉重排裁决(2026-07-02 评审收口,D2)**:02 输出逻辑序 glyph + 每 glyph `bidi_level: u8`;UAX#9 L1/L2 行级视觉重排由 03 在断行后调用 02 `bidi.rs` 的 per-line reorder API 完成。

## 5. 里程碑

### LB-M1 真实度量(度量=绘制)

实施切片:
1. `graphics/text/layout/measure.rs`:基于 `ShapedGlyphRun` 的 `measure_text_size`、子范围度量 `measured_width(run, byte_start, byte_end, include_kerning)`(UE `GetMeasuredWidth` 对齐);ascent/descent/line_height 取真实 face metrics。
2. 替换 `ui/text` 启发式 measure:`text_measure.rs`、`measure_cache.rs` 改喂真实度量(`render/14` 硬切换 #7);baseline 取真实 ascent。

测试:`text_measure_width_matches_shaped_advance_sum`、`text_measure_subrange_matches_ue_semantics`、`text_measure_cjk_fullwidth_advance`。

### LB-M2 换行与禁则

实施切片:
1. `layout/line_break/mod.rs`:UAX#14 机会(cosmic-text/unicode-linebreak)+ 贪心断行;`wrap_mode = None|Word|Glyph|WordSmart`;长词逐字回退。`layout/line_break/glue.rs` 只承接不可断 glue 与 variation selector 分类,`layout/line_break/glyph_fallback.rs` 只承接普通过宽 chunk 是否退回 glyph wrapping 的宽度/字素数 predicate,`layout/line_break/soft_hyphen.rs` 只承接 U+00AD visual chunk 拆分与 break suffix metadata,`layout/line_break/wrap_space.rs` 只承接普通 ASCII wrap-space edge trimming,`layout/line_break/smart.rs` 承接 WordSmart 首段收尾标点 glue、Unicode ellipsis/leader、standalone interrobang 与 double/interrobang 尾标点和连续收尾标点 cluster re-split,避免主断行 owner 继续混入 Unicode glue 表、glyph fallback 宽度策略、soft-hyphen source/visual 特例、smart-wrap 标点特例或 UI-local 空格策略。
2. CJK 行首尾禁则(避头尾):行首禁止标点(`、。」』）` 等)、行尾禁止开括号(`「『（` 等);kinsoku 表 + 挤压/移出策略(对照 godot)。
3. 连字符:soft hyphen(U+00AD)断点 + 末尾连字符字形;字典连字符为可选 feature。
4. (2026-07-02 评审收口)泰/老/高棉字典断行 **V1 豁免**:这三类无空格 SEA 脚本的词典断行(需要 ICU 词典数据)V1 不做,豁免期按 glyph 断;换行黄金集中的泰文用例期望值按该豁免标定并显式标注 `sea_dictionary_break_exempt`。后续以 `icu_segmenter` 可选 feature 单独立项补齐。

测试:`text_wrap_word_breaks_at_uax14_opportunities`、`text_wrap_cjk_kinsoku_no_leading_punctuation`、`text_wrap_long_word_falls_back_to_glyph`、`text_wrap_soft_hyphen_inserts_hyphen`。

### LB-M3 对齐 / 两端对齐 / 溢出

实施切片:
1. align(left/center/right/start/end 随 BIDI);justify(词间均分 + CJK 字间 + kashida 可选)。
2. overflow:ellipsis(首/中/尾,对齐 godot `OverrunBehavior`)、shrink-to-fit、clamp 字号(`editor_layout/17` 规则4);tab stop。
3. (2026-07-02 评审收口)`max_lines`(超行截断+末可见行尾省略)与 `first_line_indent`(首行缩进)在本里程碑落地,类型定义见 §6 `LayoutConstraints`。

测试:`text_align_end_follows_rtl_base_direction`、`text_justify_distributes_word_and_cjk_gaps`、`text_overflow_ellipsis_middle_keeps_head_tail`、`text_shrink_to_fit_scales_within_bounds`。

### LB-M4 竖排布局

实施切片:
1. 竖排列切分(主轴 y、列宽=字号+列距)、列对齐、竖排禁则;`LaidOutLine` 在竖排语义下为"列"。
2. 2026-07-01 首个实现切片已把 `UiTextWritingMode::{HorizontalTb,VerticalRl}` 加入 public contract,并贯穿 `UiResolvedStyle`、`UiResolvedTextLayout`、`UiShapedText` 与 `UiTextPaint`;surface parser 接受 `writing_mode = "vertical-rl"` / `font.writing_mode = "vertical-rl"`。UI resolved-layout 首段由 `zircon_runtime/src/ui/text/layout_engine/vertical.rs` 承接:以 frame height 为主轴断列,列从右向左排布,复用现有 CJK kinsoku chunk metadata,shaped DTO 的竖排 glyph frame 沿 y 轴推进并给 ASCII glyph 标记 `Cw90`。
3. 2026-07-01 vertical_rl 几何消费首段已把 `ui/text/hit_test.rs` 与 `ui/text/geometry.rs` 接到竖排语义:命中测试按 x 选择右到左列、按 y/glyph advance midpoint 反查 source byte offset;caret/range/IME cursor rect 在竖排下投影为横向 1px bar,避免继续使用横排竖线几何。
4. 2026-07-01 render-contract 编辑装饰消费首段已把 `zircon_runtime_interface/src/ui/surface/render/text_geometry.rs` 接到 `UiResolvedTextLayout.writing_mode`:selection 在竖排下使用整列宽度与 y 轴 range span,composition underline 改为列右侧竖向 side rule,caret 改为横向 bar,避免 neutral paint DTO 在平台绘制前仍输出横排装饰几何。
5. 2026-07-02 SDF render 消费首段已把 `ScreenSpaceUiTextBatch.writing_mode` 从 render extract 传到 `sdf_render.rs`;`VerticalRl` 下 SDF glyph quad 沿 y 轴推进并按列中心投影,避免布局 DTO 已是竖排但 SDF 上屏仍按横排 cursor_x 排列。该切片只处理 vertex projection,字体竖排替换与 Latin sideways rotation 仍属于 shaping/font-orientation 后续。

测试:`vertical_rl_wraps_columns_on_frame_height`、`text_vertical_kinsoku_applies_to_column_break`、`render_extract_parses_vertical_rl_writing_mode_layout`、`ui_text_writing_mode_vertical_rl_serializes_as_contract_value`、`ui_shaped_text_contract_derives_vertical_rl_glyph_bounds`、`text_hit_test_vertical_rl_uses_column_x_and_vertical_advances`、`source_geometry_uses_vertical_writing_mode_advances`、`text_input_ime_cursor_rect_uses_vertical_rl_geometry`、`ui_text_decorations_use_vertical_rl_geometry`、`sdf_draw_plan_vertical_rl_advances_glyphs_on_y_axis`。

(2026-07-02 评审收口)LB-M4 收束条件补充:`ui/text/layout_engine/vertical.rs` 首段实现需在本里程碑收尾时语义迁移到 `graphics/text/layout/vertical_layout.rs`,UI 层仅保留投影消费;迁移条目已列入 §6 硬切换表。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/layout/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | 布局入口装配(薄):`layout(run, constraints) -> LaidOutText`、`measure(...) -> Vec2` |
| `measure.rs` | 真实度量:总宽/子范围宽/行高/ascent/descent;UE `GetMeasuredWidth` 对齐 |
| `line_break/mod.rs` | UAX#14 机会 + 贪心断行 + 长词逐字 + 连字符 |
| `line_break/glue.rs` | NBSP/ZWJ/NBHY/NNBSP/WJ/ZWNBSP/variation selector 等不可断 glue 分类,输出给 `LineBreakChunk.allow_glyph_fallback` |
| `line_break/glyph_fallback.rs` | 普通过宽 chunk 是否退回 glyph wrapping 的 shared measured-width + grapheme-count predicate |
| `line_break/soft_hyphen.rs` | U+00AD soft hyphen visual chunk 剥离、断点 `-` suffix metadata 与 source range 归属 |
| `line_break/wrap_space.rs` | ASCII wrap-space 行首/行尾 trimming 策略,输出 source offset 与 byte count 给 UI run/range mutation |
| `line_break/smart.rs` | WordSmart 首段 smart-wrap 策略:ASCII 收尾标点 `,.:;!?`、Unicode ellipsis/leader `…`/`‥`、standalone interrobang `‽`、Unicode double/interrobang punctuation `‼`/`⁇`/`⁈`/`⁉` 与 Arabic/RTL 常见收尾标点 `،`/`؛`/`؟` 绑定前一词,支持标点后 ASCII/Unicode right closing quote 链 `"`/`'`/`’`/`”`,并覆盖全角/CJK 收尾标点 `、。，．・：；！？`、其后的 CJK/fullwidth 闭合符号链 `）］｝｠】〕〉》」』〗〙〛〟〞＂＇`、以及 `go?!` / `go！？` / `go，」！` / `go‽!` / `go⁉!` / `go؟!` 这类连续收尾标点 cluster,必要时在 chunk 内循环切分并关闭 protected 段 glyph fallback,但不吸收后续词 |
| `ui/text/layout_engine/wrapping.rs` | UI Word/Glyph wrapping orchestration、newline segmentation、leading grapheme continuation 与 line width fit helper |
| `ui/text/layout_engine/candidate_line.rs` | UI candidate line text/run/source/visual range mutation、pending break suffix 追加与 trailing wrap-space run 修剪 |
| `ui/text/layout_engine/direction.rs` | UI paragraph/base direction first-strong 解析、strong char helpers 与 RTL direction predicate |
| `ui/text/layout_engine/ellipsis.rs` | UI ellipsis projection: clipped-line merge、shared overflow segment 到 rich run/source/visual range remap、ellipsis marker run insertion |
| `ui/text/layout_engine/line_box.rs` | UI line-box measured/tab-aligned advances、Justify gating、line width clamp、logical Start/End x alignment 与 fallback advance minimum |
| `ui/text/layout_engine/range_mapping.rs` | UI layout 内部 source/visual byte subrange mapping,供 ellipsis projection 与 visual-order 重排共同消费 |
| `kinsoku.rs` / `kinsoku/tests.rs` | CJK 避头尾禁则表 + 挤压/移出; module-local owner regressions 独立承接 halfwidth kana、JLREQ pair、white bracket 等边界 |
| `align.rs` | align + justify(词间/字间/kashida) |
| `overflow.rs` | ellipsis(首/中/尾/word-trim) |
| `tab.rs` | tab stop / `tab_size` advance expansion |
| `vertical_layout.rs` | 竖排 shared owner：列容量、右到左 column frame placement、cross/main axis extents；UI adapter 消费其结果 |
| `ui/text/layout_engine/vertical.rs` | `VerticalRl` UI adapter：CandidateLine/rich-run/ellipsis DTO 投影；断行继续复用 shared line-break/kinsoku，列 frame/extent 语义已迁 `vertical_layout.rs` |

### 核心类型与接口

```rust
pub struct LayoutConstraints {
    pub wrap_width: Option<f32>,     // 竖排时为 wrap_height
    pub wrap_mode: TextWrapMode,     // None | Word | Glyph | WordSmart
    pub align: TextAlign,            // Left|Center|Right|Start|End|Justify
    pub justify: JustifyFlags,       // WordBound | CjkInter | Kashida | TrimEdgeSpaces
    pub overflow: TextOverflow,      // Clip | Ellipsis(EllipsisPos) | ShrinkToFit | Clamp
    pub line_height: LineHeight,     // Normal | Scale(f32) | Absolute(f32)
    pub tab_stops: TabStops,
    pub orientation: TextOrientation,// 接 02
    // (2026-07-02 评审收口)新增:
    pub max_lines: Option<u32>,      // 超出行数截断;末可见行行尾按 overflow 的 ellipsis 语义收尾
    pub first_line_indent: f32,      // 首行缩进(逻辑像素);实现归 LB-M3
}

// (2026-07-02 评审收口)TabStops 完整定义:
pub struct TabStops {
    pub default_tab_size: u32,           // 默认制表宽 = tab_size × space_advance(主 face 空格 advance)
    pub stops: Vec<TabStop>,             // 显式 stop,按 x 升序;越过全部显式 stop 后回落默认宽
}
pub struct TabStop { pub x: f32, pub align: TabAlign }
pub enum TabAlign { Left, Center, Right, Decimal } // Decimal 按小数点字符对齐
// (2026-07-02 评审收口)EllipsisPos 完整定义(对齐 godot OverrunBehavior 的 TRIM_ELLIPSIS 族):
pub enum EllipsisPos { Start, Middle, End }

// (2026-07-02 评审收口,D1/07 内联对象)LaidOutText 改多 run 模型:
pub struct LaidOutText {
    pub runs: Vec<Arc<ShapedGlyphRun>>, // 复用 02 字形;富文本/回退分段时 >1
    pub items: Vec<LayoutItem>,         // 布局项序列(逻辑序)
    pub lines: Vec<LaidOutLine>,
    pub size: Vec2,
}
pub enum LayoutItem {
    Text { run_index: u32, glyph_range: (u32, u32) },
    Inline { object_id: InlineObjectId, size: Vec2, baseline: InlineBaseline }, // 预留:07 §4 内联对象(image/widget)回填
}
pub struct LaidOutLine {
    pub spans: Vec<(u32 /* run_index */, (u32, u32) /* glyph_range */)>, // 行内按逻辑序引用各 run 片段
    pub visual_order: Vec<u32>,      // (2026-07-02 评审收口,D2)行级视觉序:spans 索引的视觉排列,由 02 bidi.rs per-line reorder API 产出
    pub origin: Vec2, pub baseline: f32, pub width: f32,
    pub ascent: f32, pub descent: f32, pub trailing_whitespace: f32,
}

// (2026-07-02 评审收口)caret 亲和性:软换行行尾/BIDI 边界同一 offset 有两个视觉位置
pub enum CaretAffinity { Upstream, Downstream }
// - hit_test 返回 (source_offset, CaretAffinity);
// - caret 几何计算入参含 affinity:软换行点 offset 处,Upstream → 上一行行尾 caret,Downstream → 下一行行首 caret;
// - edit_state 光标携带 affinity(点击/End 键置 Upstream,Home/常规输入置 Downstream);
// - 08 IME 光标矩形(SetCursorArea 入参)引用同一 (offset, affinity) 模型,不另建几何。

// 度量(UE FShapedGlyphSequence::GetMeasuredWidth 对齐)
pub fn measured_width(run: &ShapedGlyphRun, byte_start: u32, byte_end: u32, include_kerning: bool) -> f32;
pub fn measure_text_size(run: &ShapedGlyphRun, c: &LayoutConstraints) -> Vec2;
```

(2026-07-02 评审收口)`LaidOutText` 多 run 化后,单 run 场景 `runs.len()==1` 且 `items` 只有一个 `Text` 项,现有单 run 消费方按 `spans[0]` 语义平移;`LayoutItem::Inline` 的解析来源与 `InlineBaseline` 定义见 07 §4(本处仅预留槽位,07 落地时回填)。

### 度量算法(对齐 UE)

- 总宽 = Σ glyph.advance(行内,trailing whitespace 不计入 content width,但 layout width 含)。
- **子范围度量**:给字节区间 `[s, e)`,累加 `source_range` 落入 `[s,e)` 的 glyph advance;`include_kerning=false` 时按同一 backend 请求 unkerned shaping(当前 cosmic-text 路径写入 OpenType `kern=0`),再测量该 source range,避免在已有 kerned run 上反推簇间 GPOS delta——供光标/选区精确定位。
- 行高:`Normal` = ascent+descent+line_gap(face hhea/OS2);`Scale(k)` = font_size×k;baseline = line_top + ascent。
- BIDI:度量按逻辑序累加(顺序无关于视觉序);命中测试用视觉序 + `source_range` 反查(`hit_test.rs` 改造,`render/14` 硬切换 #3)。
- **visual_order 按行应用(2026-07-02 评审收口,D2)**:视觉重排是**行级**操作——断行完成后,对每一 `LaidOutLine` 的 spans 调用 02 `bidi.rs` 的 per-line reorder API(UAX#9 L1:行尾空白/分隔符复位为段落 level;L2:按 level 由高到低逐段反转),结果写入 `LaidOutLine.visual_order`;glyph 存储保持逻辑序不动,渲染/命中测试按 `visual_order` 遍历。禁止在整形阶段或段落级做一次性重排。
- 竖排:advance 在 y,width→height 语义对调。

#### 混 face 行度量(2026-07-02 评审收口,D7)

- 一行内含多个 face(fallback/富文本混排)时:行 `ascent`/`descent` = 行内各 run face 度量的 **max**;`line_gap` 取**主 face**(段落解析出的首选 face),不参与 max。
- 单 face 度量优先级:OS/2 `fsSelection` 的 `USE_TYPO_METRICS` 位置位时取 `sTypoAscender/sTypoDescender/sTypoLineGap`,否则取 `hhea` 的 ascent/descent/line_gap;`usWinAscent/usWinDescent` 仅作裁剪参考,不进行高计算。
- baseline 统一 alphabetic:所有 run 对齐到同一 alphabetic baseline,行 baseline = line_top + max_ascent。
- 测试:`text_line_height_mixed_face_uses_max_metrics`(拉丁主 face + CJK fallback face 混行,断言行高 = max(ascent)+max(descent)+主 face line_gap,baseline 取 max_ascent)。

### CJK 禁则(`kinsoku.rs`,对照 godot)

- 行首禁则集(不能出现在行首):`、。，．・：；！？）］｝｠】〕〉》」』〗〙〛’”〟〞`、小书写假名 `ぁぃぅぇぉっゃゅょゎゕゖァィゥェォッャュョヮヵヶㇰㇱㇲㇳㇴㇵㇶㇷㇸㇹㇺㇻㇼㇽㇾㇿ`、日文非行首字符 `ー々〻ゝゞヽヾ`、JLREQ hyphens `‐〜゠–`、全角 spacing voicing marks `゛゜` 与半角 `｡｣､･ｧｨｩｪｫｯｬｭｮｰﾞﾟ` …
- JLREQ cl-08 分离禁止成对字符:`——`、`……`、`‥‥`、`〳〵`、`〴〵` 之间不可断;跨 chunk 合并并对单 chunk 内 pair 关闭 glyph fallback,避免过窄 frame 把第二个符号拆到下一行。
- 行尾禁则集(不能出现在行尾):`（｛｟［【〔〈《「『〖〘〚‘“〝` 与半角 `｢`。
- 策略:断点落在禁则字符时,优先**前移**断点(把行首禁则字符挤到上一行末——"追い込み"),次选**移出**(把行尾禁则字符移到下一行——"追い出し");可配 squeeze 标点半角。

### 与既有路径硬切换(`render/14` 清单 #2/#3/#7)

| 现有 | 切换 |
|------|------|
| `layout_engine.rs` 全体(等宽 wrap/align/ellipsize/baseline) | 删除;语义迁 `graphics/text/layout/*`(真实度量重写) |
| `layout_engine/tests.rs` 期望值 | 按真实字形度量重标定 |
| `hit_test.rs::hit_test_text_layout` | 改基于 `ShapedGlyph.source_range` 反查;签名/返回类型不变 |
| `text_measure.rs::measure_text_size` | 改调 `graphics/text/layout::measure`(taffy measure 闭包,走 shaped cache) |
| `ui/text/layout_engine/vertical.rs`(竖排首段) | 列容量、右到左 frame、axis extents 已硬切 `graphics/text/layout/vertical_layout.rs`;UI 层保留 CandidateLine/rich/ellipsis DTO 投影，完整 `LaidOutText` hard cut 仍后续 |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_measure_width_matches_shaped_advance_sum` | 度量宽 = Σadvance(含 kerning),与绘制端一致(度量=绘制) |
| `text_measure_subrange_matches_ue_semantics` | 子范围宽对 UE `GetMeasuredWidth` 期望表;include_kerning 两路径 |
| `text_measure_cjk_fullwidth_advance` | CJK 全角 advance = 字号(对照 face) |
| `text_wrap_word_breaks_at_uax14_opportunities` | 断点集 = UAX#14 机会(对照标准用例) |
| `text_wrap_cjk_kinsoku_no_leading_punctuation` | 行首无禁则标点;追い込み/追い出し正确 |
| `text_wrap_cjk_kinsoku_no_leading_halfwidth_small_kana` | 半角小假名不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_small_katakana_ka` | 小写 ka/ke 假名 `ゕゖヵヶ` 不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_katakana_phonetic_extension_small_kana` | 片假名音标扩展小假名 `ㇰ..ㇿ` 不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_cjk_white_close_punctuation` | CJK 白闭括号/引号 `〗〙〛〟` 不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_cjk_double_prime_closing_quote` | CJK 双素引号闭合变体 `〞` 不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_fullwidth_white_close_parenthesis` | 全角白右圆括号 `｠` 不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_spacing_voicing_mark` | 全角 spacing dakuten/handakuten `゛゜` 不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_jlreq_hyphen` | JLREQ hyphens `‐〜゠–` 不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_keeps_jlreq_inseparable_ellipsis_pair_together` | JLREQ cl-08 `……` 在过窄 frame 下不可在两个 `…` 之间断行;允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_prolonged_sound_mark` | 日文全角长音符不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_iteration_mark` | 日文迭代符号不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_leading_vertical_iteration_mark` | U+303B vertical ideographic iteration mark `〻` 不可成为行首;过窄 frame 下允许 protected overhang |
| `text_wrap_cjk_kinsoku_no_trailing_halfwidth_open_punctuation` | 半角开引号不可留在行尾;与后续 glyph 同 chunk |
| `text_wrap_cjk_kinsoku_no_trailing_cjk_white_open_punctuation` | CJK 白开括号/引号 `〖〘〚〝` 不可留在行尾;与后续 glyph 同 chunk |
| `text_wrap_cjk_kinsoku_no_trailing_fullwidth_white_open_parenthesis` | 全角白左圆括号 `｟` 不可留在行尾;与后续 glyph 同 chunk |
| `text_wrap_long_word_falls_back_to_glyph` | 超宽单词 Word 模式逐字断 |
| `text_wrap_soft_hyphen_inserts_hyphen` | U+00AD 断点末尾出连字符字形 |
| `word_wrap_keeps_variation_selector_sequence_together` | variation selector 与 base glyph 保持 glue;过窄 frame 下允许 overhang |
| `word_wrap_keeps_additional_glue_sequences_together` | NBHY/NNBSP/WJ/ZWNBSP 不被 glyph fallback 拆开 |
| `word_smart_keeps_ascii_trailing_punctuation_with_previous_word` | WordSmart 下 ASCII 收尾标点不可成为 wrapped line leader;`go,` 可 protected overhang |
| `word_smart_keeps_ascii_closing_quote_after_trailing_punctuation_with_previous_word` | WordSmart 下 ASCII 收尾标点后的闭合引号与前一词保持同一 protected chunk;`go,"` 可 protected overhang |
| `word_smart_keeps_unicode_closing_quote_after_trailing_punctuation_with_previous_word` | WordSmart 下 ASCII 收尾标点后的 Unicode right closing quote `’`/`”` 与前一词保持同一 protected chunk;`go,”` 可 protected overhang |
| `word_smart_keeps_fullwidth_trailing_punctuation_with_previous_word` | WordSmart 下全角/CJK 收尾标点 `，`/`。`/`？` 与前一词保持同一 protected chunk;`go，` 可 protected overhang |
| `word_smart_keeps_cjk_closing_delimiter_after_fullwidth_punctuation_with_previous_word` | WordSmart 下全角/CJK 收尾标点后的 CJK/fullwidth 闭合引号/括号与前一词保持同一 protected chunk;`go，」` 可 protected overhang |
| `word_smart_keeps_trailing_punctuation_cluster_without_absorbing_next_word` | WordSmart 下连续收尾标点 cluster `go?!` 保持前词 protected chunk,后续 `a` 仍独立换行候选 |
| `word_smart_keeps_ellipsis_trailing_punctuation_with_previous_word` | WordSmart 下 Unicode ellipsis/leader `…`/`‥` 与前一词保持 protected chunk,后续词仍独立换行候选 |
| `word_smart_keeps_unicode_interrobang_punctuation_with_previous_word` | WordSmart 下 Unicode standalone interrobang `‽` 与前一词保持 protected chunk,`go‽` 可 protected overhang 且后续词仍独立换行候选 |
| `word_smart_keeps_unicode_double_punctuation_with_previous_word` | WordSmart 下 Unicode double/interrobang punctuation `‼`/`⁇`/`⁈`/`⁉` 与前一词保持 protected chunk,`go⁉` 可 protected overhang 且后续词仍独立换行候选 |
| `word_smart_keeps_arabic_trailing_punctuation_with_previous_word` | WordSmart 下 Arabic/RTL 常见收尾标点 `،`/`؛`/`؟` 与前一词保持 protected chunk,`go؟` 可 protected overhang 且后续词仍独立换行候选 |
| `text_align_end_follows_rtl_base_direction` | End 对齐在 RTL 段落靠左 |
| `text_justify_distributes_word_and_cjk_gaps` | 两端对齐词间 + CJK 字间均分,末行不拉伸 |
| `text_overflow_ellipsis_middle_keeps_head_tail` | 中部省略保头尾,`…` 宽度计入 |
| `text_shrink_to_fit_scales_within_bounds` | 缩放后宽≤bounds,字号≥min clamp |
| `text_hit_test_maps_pixel_to_source_offset` | 像素点→源字节 offset,affinity 正确(对照 cluster) |
| `text_line_height_mixed_face_uses_max_metrics` | (2026-07-02 评审收口,D7)混 face 行:行高 = max(ascent)+max(descent)+主 face line_gap,baseline = max_ascent |
| `text_caret_affinity_soft_wrap_boundary` | (2026-07-02 评审收口)软换行点同一 offset:Upstream 返回上一行行尾 caret 矩形,Downstream 返回下一行行首 caret 矩形 |
| `vertical_rl_wraps_columns_on_frame_height` | 竖排按高度断列,列序右到左,`UiResolvedTextLayout.writing_mode` 保持 `VerticalRl` |
| `text_vertical_kinsoku_applies_to_column_break` | 竖排列断点复用 CJK 行尾/行首禁则,开标点不留在上一列末 |
| `render_extract_parses_vertical_rl_writing_mode_layout` | surface render extract 可解析 `writing_mode = "vertical-rl"` 并产出竖排 resolved layout |
| `ui_text_writing_mode_vertical_rl_serializes_as_contract_value` | public interface writing-mode contract 序列化为 `vertical_rl` |
| `ui_shaped_text_contract_derives_vertical_rl_glyph_bounds` | shaped DTO 继承竖排 writing mode,ASCII glyph 竖排 frame/rotation contract 有界 |
| `text_hit_test_vertical_rl_uses_column_x_and_vertical_advances` | 竖排命中测试先按 x 选择右到左列,再按 y midpoint/advance 返回 source byte offset |
| `source_geometry_uses_vertical_writing_mode_advances` | `caret_frame_for_text_layout`/`text_range_frames_for_text_layout` 在 `VerticalRl` 下把 advance 投影到 y,输出横向 caret/range bar |
| `text_input_ime_cursor_rect_uses_vertical_rl_geometry` | TextInput 的 IME cursor rect 复用竖排 resolved-layout caret frame,拒绝横排竖线 fallback |
| `ui_text_decorations_use_vertical_rl_geometry` | render-facing `UiTextPaintDecoration` 在 `VerticalRl` 下输出列宽 selection、右侧 composition side rule 和横向 caret bar |

里程碑命令:`cargo test -p zircon_runtime text_measure --locked`、`text_wrap --locked`、`text_align --locked`、`text_overflow --locked`。

## 7. 风险与回退

- cosmic-text `layout_runs` 已做断行/对齐,优先消费其结果再后处理 CJK 禁则/justify;若其断行不可控,改用 `unicode-linebreak` 机会 + 自研贪心。(2026-07-02 评审收口:本条已按 D1 改判——自研贪心断行是主链,cosmic-text `layout_runs` 定位为**对拍参考**(黄金集比对/回归定位工具),不作为生产回退路径。)
- Knuth-Plass 最优断行(段落级最小化破碎度)列为 V2,V1 用贪心(对齐多数引擎)。

## 8. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前概述（2026-07-11）：LB-M4 的共享 `layout_text` 已在产品 proof 中按 240px 竖排主轴把含开闭引号、句读和逗号的 CJK 文本断成两列，并由 resolved layout 产出右到左 column frames；不是测试手工拼 DTO。`vertical_rl_` 7/7 覆盖列容量/右到左 frame、命中测试、render extract 与 TextInput IME cursor rect，`text_vertical_` 17/17 含竖排禁则；`text_caret_affinity_soft_wrap_boundary` 1/1 关闭同一 source offset 的 Upstream=上一行末、Downstream=下一行首。WGPU 产品帧两列整体 changed=4114、bbox 68×240，右/左列分别 2548/1566 像素。复杂 mixed-BiDi caret/range 的产品级 source-affinity 对拍、全量精确换行/advance/bbox corpus 与平台候选窗实机仍保持 open。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`03/2026-07-09-line-breaking-measure-and-layout-output-records.md`](03/2026-07-09-line-breaking-measure-and-layout-output-records.md)
