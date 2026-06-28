---
related_code:
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime/Cargo.toml
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheHarfBuzz.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/godot/modules/text_server_adv/script_iterator.cpp
  - dev/godot/servers/text/text_server.h
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/slint/internal/core/textlayout/shaping.rs
  - dev/slint/internal/core/textlayout/glyphclusters.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
status: in_progress
---

# 02 文本整形 / Unicode / BIDI / 竖排

> 本计划把"码点串 + 样式 + face"整形为"定位字形序列 `ShapedGlyphRun`"——文本主链核心。承接 `editor_ui/03 §2.2` 缺口 1(shaping 权威未定)。它产出的真实字形度量是 `03`(换行/度量)与 `editor_layout/17 G1`(度量=绘制)的唯一数据源。

## 1. 目标

1. **整形权威定稿**:cosmic-text(内置 rustybuzz)做 GSUB/GPOS 整形——连字、kerning、上下文替换、标记定位;glyphon 退为 bitmap 绘制后端,不再"挂名未接"。
2. **Unicode 完整支持**:grapheme cluster、script 检测分段、规范化(NFC)、组合字符、变体选择符(VS15/16)、emoji ZWJ 序列、控制字符处理。
3. **BIDI(UAX#9)**:段落级方向解析、level run 切分、视觉序重排、镜像字符(括号/箭头)、`base_direction = Auto|Ltr|Rtl`。
4. **竖排**:朝向枚举(对齐 godot `Orientation::{Horizontal,Vertical}`);竖排主轴 advance、baseline 居中、横排正字(`upright`)/旋转(`mixed`/`sideways`)模式;CJK 标点竖排形。
5. **cluster→source 映射**:每字形携 `source_range`(源文本字节区间),供命中测试/光标/选区/IME 精确反查。

## 2. 现状与差距

- `ui/text/shaper.rs`:`UiTextShaper` trait + `UiTextShaperStack`,但 `active_layout_backend_for_intent` **永远回落** `Heuristic`;`fallback_reason_for_backend` 注释 "backend not connected"。glyphon 仅用于绘制,布局走 `layout_engine.rs` 等宽近似。
- `layout_engine/visual_order.rs`:低保真 BiDi——仅检测 LTR/RTL 字符范围(ASCII + 阿拉伯/希伯来块)并重排 run,**无** UAX#9 level 解析、无镜像字符、无中性字符 resolution。
- `grapheme.rs`:`unicode-segmentation` 提供 grapheme/word 边界,但未进入整形(整形端无 cluster 概念)。
- **无脚本分段**:混排文本未按 script 切段,复杂文种(阿拉伯连写、天城文重排)不可能正确。
- **无竖排**:渲染 DTO `UiShapedGlyph` 无朝向字段;布局只有水平。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Fonts/SlateTextShaper.cpp` | `ShapeBidirectionalText`(ICU bidi 拆方向)→ `ShapeUnidirectionalText`(按 script/font 分段)→ HarfBuzz `hb_shape`;`FinalizeTextShaping` 建源索引→字形映射;`PerformKerningOnlyTextShaping`(ASCII 快路径)。本计划整形流水主样板 |
| `dev/UnrealEngine/.../Fonts/FontCacheHarfBuzz.cpp` | `hb_font_t` 创建、`hb_buffer` 方向/script/language 设置、cluster 读取——cosmic-text 内部等价,理解 cluster 语义用 |
| `dev/UnrealEngine/.../Fonts/FontCache.h` | `FShapedGlyphEntry { GlyphIndex, SourceIndex, NumCharactersInGlyph, NumGraphemeClustersInGlyph, TextDirection, bIsVisible }`——本计划 `ShapedGlyph.source_range` 即 `SourceIndex + NumCharactersInGlyph` 区间化 |
| `dev/godot/modules/text_server_adv/text_server_adv.cpp` | `shaped_text_shape`:HarfBuzz + ICU bidi(`ubidi_*`)+ script run;竖排 `_shape_run` 的 `Orientation` 分支与 `vertical` advance;`gr_features`(OpenType features)。竖排与 BIDI 权威 |
| `dev/godot/modules/text_server_adv/script_iterator.cpp` | `ScriptIterator`:Unicode script 分段(common/inherited 归并),本计划脚本分段算法直接对照 |
| `dev/godot/servers/text/text_server.h` | `Glyph { flags: GRAPHEME_IS_RTL/VIRTUAL/SPACE/... }`、`Orientation`、`Direction`——字形标志位与朝向枚举样板 |
| `dev/bevy/crates/bevy_text/src/pipeline.rs` | parley `break_and_shape` → `PositionedLayoutItem::GlyphRun` 的 Rust 落地;`source_range`/`text_byte_offset` 取法 |
| `dev/slint/.../textlayout/{shaping,glyphclusters}.rs` | `TextShaper` trait、`Glyph { text_byte_offset }`、glyph cluster 分组——轻量 Rust 形态 |

**Rust/wgpu 落地**:cosmic-text `BufferLine::shape`/`ShapeLine`/`ShapeRun`/`ShapeGlyph`(已内置 bidi + script run + rustybuzz);`unicode-bidi`(若需独立 BIDI)、`unicode-script`、`icu_normalizer`(NFC,可选)。

## 4. 目标架构

归属:契约层 `core/framework/render/text/{shaped_run.rs,shaping_service.rs}`(已由 `render/14` 定稿,本计划扩展朝向/标志位);实现层 `graphics/text/shaping/`(`cosmic_text` 隔离于 `cosmic.rs`)。

```
TextShapeRequest { text, style, base_direction, orientation, wrap..(交 03) }
  └─ normalize(NFC,可选) → bidi(UAX#9, level runs) → script segment →
     per-run shape(cosmic-text/rustybuzz, font fallback 交 06) →
     visual reorder(RTL) → ShapedGlyphRun { glyphs[ShapedGlyph], lines, base_direction, orientation }
```

整形与换行的关系:cosmic-text 一次 `shape` 同时给 cluster 与断点机会;`03` 在 `ShapedGlyphRun` 之上做行切分与对齐(本计划交付**无宽度约束的**整形 + 断点机会标注,`03` 消费)。

## 5. 里程碑

### SH-M1 整形后端接入(替换启发式)

实施切片:
1. `graphics/text/shaping/cosmic.rs` 隔离层:`TextShapeRequest → cosmic-text Buffer → ShapedGlyphRun`;cluster→`source_range`;断点机会位标注(交 03)。
2. `ShapedGlyph` 扩展:`source_range`、`cluster_flags`(RTL/space/mandatory-break/whitespace,对齐 godot `GraphemeFlag`)、命中 `font_id`(回退后实际 face)。
3. `ui/text/shaper.rs` 适配:`UiTextShaperStack` 改持 `&dyn TextShapingService`,`shape_text` 投影 `ShapedGlyphRun → UiResolvedTextLayout`;删除 `UiTextBackendIntent`/`active_layout_backend_for_intent`/`fallback_reason_for_backend`(`render/14` 硬切换清单 #1)。

测试:`text_shape_latin_kerning_matches_face_metrics`、`text_shape_ligature_fi_single_glyph`、`text_shape_clusters_map_source_ranges_monotonic`。

### SH-M2 Unicode 与 BIDI

实施切片:
1. script 分段(`ScriptIterator` 对照,common/inherited 归并到邻接 script);per-script run 整形。
2. BIDI:`base_direction = Auto` 走首强字符规则;level run 切分 + 视觉重排 + 镜像字符表(括号/箭头);RTL run 内字形逆序。
3. 规范化(NFC,可选 feature)、变体选择符、emoji ZWJ 序列保簇、控制字符不可见标志。

测试:`text_bidi_mixed_ltr_rtl_visual_order_matches_uax9`、`text_bidi_mirrors_paren_in_rtl`、`text_bidi_mirrors_arrow_in_rtl`、`text_shape_emoji_zwj_sequence_single_cluster`、`text_script_segmentation_arabic_latin_runs`。

### SH-M3 竖排

实施切片:
1. `Orientation::Vertical` + `VerticalMode::{Upright,Mixed,Sideways}`;竖排主轴 advance(`vmtx`/合成)、baseline 居中。
2. CJK upright 正字、拉丁 sideways 旋转 90°、标点竖排形(`vert`/`vrt2` GSUB 若 face 提供,否则合成);`ShapedGlyph.rotation` 字段。

测试:`text_vertical_cjk_upright_advances_on_y`、`text_vertical_latin_sideways_rotated`、`text_vertical_punctuation_centered`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/shaping/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | `SharedTextShapingService::shape` 装配(薄) |
| `cosmic.rs` | **`cosmic_text` 唯一隔离层** —— Buffer/ShapeLine/ShapeRun/ShapeGlyph 只在此;出口 `ShapedGlyphRun` |
| `script_segment.rs` | Unicode script 分段(`unicode-script`,common/inherited 归并;对照 godot `ScriptIterator`) |
| `bidi.rs` | UAX#9 包装(cosmic-text 内置 or `unicode-bidi`):level run、视觉重排、镜像字符表 |
| `vertical.rs` | 竖排:朝向解析、主轴 advance、字形旋转决策、标点居中 |
| `normalize.rs` | NFC 规范化(可选 feature `text-normalize`) |
| `cache.rs` | shaped run 缓存(`ShapedTextCacheKey → Arc<ShapedGlyphRun>`,见 09) |

### 契约扩展(回填 `render/14` 的 `shaped_run.rs`/`shaping_service.rs`)

```rust
// ShapedGlyph 扩展(在 render/14 既定字段上加)
pub struct ShapedGlyph {
    pub glyph_id: u16,
    pub font_id: ResourceId,        // 06 回退后实际命中 face
    pub source_range: (u32, u32),   // cluster→源字节区间(单调,覆盖完整)
    pub offset: Vec2,
    pub advance: f32,               // 横排=x 进格;竖排=y 进格
    pub line_index: u32,
    pub atlas: GlyphAtlasRef,
    pub direction: TextDirection,
    pub cluster_flags: ClusterFlags,// 新增
    pub rotation: GlyphRotation,    // 新增:None | Cw90(竖排 sideways)
}
bitflags! { pub struct ClusterFlags: u16 {  // 对齐 godot GraphemeFlag
    const RTL = 1; const SPACE = 2; const WHITESPACE = 4;
    const MANDATORY_BREAK = 8; const SOFT_BREAK = 16; const TAB = 32;
    const VIRTUAL = 64;  // 控制字符/不可见;不进图集
    const CLUSTER_START = 128;
}}
pub enum GlyphRotation { None, Cw90 }

// 请求扩展
pub enum TextOrientation { Horizontal, Vertical }
pub enum VerticalMode { Upright, Mixed, Sideways } // 竖排时 latin 处理
// TextShapeRequest 增 orientation: TextOrientation, vertical_mode: VerticalMode
```

### BIDI 算法落点(`bidi.rs`)

优先用 cosmic-text 内置 bidi(`BufferLine` 已按段落解析方向、产出 level)。`base_direction`:
- `Auto`:首个强方向字符(L/R/AL)定段落基方向(UAX#9 P2/P3)。
- 视觉重排:cosmic-text 的 `ShapeLine::layout` 已按 level 重排;本层只在投影时保 `source_range` 不乱。
- 镜像字符:RTL level 内对 `Bidi_Mirrored` 码点(`( ) [ ] { } < >` 等)换镜像字形(查 cosmic-text/face 的 `rtlm` 或镜像表)。

若 cosmic-text bidi 细节不足(嵌套隔离符 LRI/RLI/PDI),回退独立 `unicode-bidi` crate 跑 `BidiInfo`,再喂 per-level run 给整形(UE `ShapeBidirectionalText` 同结构)。

### 竖排落点(`vertical.rs`)

V1 范围(对齐 godot vertical 基线):
- 主轴 = y;`advance` 取 `vmtx` 竖直进格,缺失则 `ascent+descent`。
- `VerticalMode::Mixed`(默认):CJK/全角 upright(`rotation=None`,水平居中到竖列),拉丁/数字 `Cw90`(`rotation=Cw90`)。
- `Upright`:全部正字。`Sideways`:整行旋转(含 CJK)。
- 标点:若 face 有 `vert`/`vrt2` GSUB feature 则启用得竖排形(句读居中);否则合成偏移居中。
- baseline:竖排 baseline 居列中线;`ShapedLine` 在竖排下 `baseline_y` 语义转为"列中线 x"。

V2(本计划不实现,留接口):双向竖排混排、`text-combine-upright`(纵中横)、避头尾竖排禁则交 `03`。

### 与既有路径的硬切换(`render/14` 清单 #1,本计划执行整形侧)

| 现有 | 切换 |
|------|------|
| `shaper.rs::UiTextBackendIntent` 三态 + 回退理由 | 删除;`UiTextShaperStack` 只持 service 适配器 |
| `layout_engine.rs` 等宽 `text_advance`/`measure_width` | 删除(度量迁 03 真实字形);`visual_order.rs` 低保真 BiDi 删除(BIDI 迁本计划 `bidi.rs`) |
| `graphics/.../ui/text.rs` glyphon 既做布局又绘制 | glyphon 仅绘制;布局数据来自 `ShapedGlyphRun` |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_shape_latin_kerning_matches_face_metrics` | "AV"/"To" 的 advance 含 kerning,与 face `kern`/GPOS 一致 |
| `text_shape_ligature_fi_single_glyph` | "fi" 合字为单字形,`source_range`=两字符区间 |
| `text_shape_clusters_map_source_ranges_monotonic` | 所有 glyph `source_range` 单调、无空洞、并集=全文本 |
| `text_script_segmentation_arabic_latin_runs` | "abcمرحبا" 切 2 run,script 标注正确,common 归邻 |
| `text_bidi_mixed_ltr_rtl_visual_order_matches_uax9` | 取 UAX#9 标准用例若干,视觉序与参考一致 |
| `text_bidi_mirrors_paren_in_rtl` | RTL run 内 `(` / `)` 按视觉方向互为镜像且保留 source_range |
| `text_bidi_mirrors_arrow_in_rtl` | RTL run 内 `→` 渲染为 `←` 镜像字形且保留 source_range |
| `text_shape_emoji_zwj_sequence_single_cluster` | 👨‍👩‍👧(ZWJ)为单 cluster,source_range 完整 |
| `text_shape_variation_selector_keeps_cluster` | 基字 + VS16 同簇,emoji 呈现 |
| `text_vertical_cjk_upright_advances_on_y` | 竖排 CJK `advance` 在 y、`rotation=None` |
| `text_vertical_latin_sideways_rotated` | 竖排拉丁 `rotation=Cw90` |
| `text_vertical_punctuation_centered` | 句读竖排居中(有/无 `vert` 两路径) |

里程碑命令:`cargo test -p zircon_runtime text_shape --locked`、`text_bidi --locked`、`text_vertical --locked`。

## 7. 风险与回退

- cosmic-text bidi 隔离符支持不足 → 切 `unicode-bidi` 独立跑,接口不变。
- 竖排是长尾:V1 只保证 CJK 正字 + 拉丁旋转 + 标点居中,旋转字形依赖 face GSUB,缺失则合成,对拍 godot。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-06-28 | SH-M2 interim RTL mirrored punctuation | runtime_text_sh_m2_rtl_mirrored_punctuation_check_passed | `ui/text/layout_engine/visual_order.rs` 在当前低保真 visual-order scaffold 中对 RTL visual span 的单码点镜像标点执行表驱动替换，先覆盖括号/箭头/常见成对符号，并保持 `VisualTextFragment.source_range` 不随 visual glyph text 改写；新增 `text_bidi_mirrors_paren_in_rtl` 与 `text_bidi_mirrors_arrow_in_rtl` 锁定原 `)` 的 14..15 source range 成为 visual `(`、原 `(` 的 9..10 source range 成为 visual `)`，以及原 `→` 的 5..8 source range 成为 visual `←` | `rustfmt --check zircon_runtime/src/ui/text/layout_engine/visual_order.rs zircon_runtime/src/ui/text/layout_engine/tests.rs` 通过；`cargo test -p zircon_runtime text_bidi_mirrors --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-bidi-mirror --message-format short --color never -- --nocapture` 通过 2/2；视觉证据 `docs/tests/runtime/text/runtime_text_rtl_mirrored_punctuation_preview_20260628.png` 已检查 | 这只是 SH-M2 mirror-table 首段；完整 UAX#9 level run、isolate/LRI/RLI/PDI、script segmentation、cosmic/unicode-bidi hard cutover 与竖排仍 pending |
| 2026-06-28 | SH-M1 support fix + UAX#14 break opportunity flags | runtime_text_sh_m1_uax14_break_flags_focused_tests_passed | 新增 `graphics/text/shaping/line_break.rs`，使用 `unicode-linebreak` 生成 UAX#14 break opportunity map；`cosmic.rs` 在 cluster-start glyph 上写入 `soft_break` / `mandatory_break`，并过滤 synthetic end-of-text mandatory break；同时修复 stale importer include guards 与 camera-loop test callback signature，使 `text_shape_` focused lib-test 能完整运行 | `rustfmt --check` 通过；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never` 通过(仅既有 warnings)；`cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_shape_` 通过 6/6 | 这不是完整 LB-M2 断行算法；script segmentation、UAX#9 mirror/isolate、真实 fallback-selected face id 与竖排 `VerticalMode::Mixed` 仍 pending |
| 2026-06-28 | SH-M1 owner slice: cosmic-backed `ShapedGlyphRun` contract and isolated shaping owner | runtime_text_sh_m1_shaping_owner_core_check_passed_focused_libtest_blocked | Added neutral `core/framework/render/text/{shaped_run.rs,shaping_service.rs}` contracts and `graphics/text/shaping/{mod.rs,cosmic.rs}` owner; `cosmic.rs` is now the isolated glyphon/cosmic-text Buffer/LayoutGlyph projection point and emits glyph id, source_range, visual_range, advance, baseline, direction, cluster flags, and rotation contract data; `graphics/text/layout/measure.rs` now derives line width/line metrics/per-grapheme advances from `ShapedGlyphRun` instead of importing backend text types directly | scoped rustfmt passed; `cargo check -q -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` passed with existing warnings only; focused `cargo test -q -p zircon_runtime text_shape_ --lib --no-default-features --locked` compiles past old importer include guards but is blocked by unrelated camera-loop lib-test closure signature errors; screenshot evidence remains `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png` and target image checks are false | Actual fallback-selected `FontFaceId` remains pending; full UAX#9 isolate/mirroring, script segmentation, UAX#14 break data, CJK kinsoku, vertical metrics/rotation policy beyond contract fields, and UI hard cutover from `visual_order.rs` remain future SH/LB slices |
| 2026-06-28 | SH-M1 DTO 首段: render-facing glyph contract fields | runtime_text_sh_lb_m1_shaped_glyph_advances_interface_check_passed_runtime_check_timeout | `UiShapedGlyph` 补齐 `font_id`、`cluster_flags`、`rotation`，并在 neutral projection 中按 grapheme 标出 cluster_start/RTL/space/tab/whitespace/break/virtual 占位；`UiRenderCommand::text_paint(...)` 在已有 layout projection 中用 style font key 填充 missing glyph font_id，先建立 render-facing contract 形状 | scoped rustfmt 通过；`zircon_runtime_interface --lib` 与 `zircon_runtime_interface --tests` Cargo check 通过；runtime no-default check 244s 编译超时无 Rust diagnostics，匹配验证进程已停止 | 仍未实现完整 cosmic-text `ShapedGlyphRun`、真实 fallback-selected face id、source_range、脚本分段、UAX#9、镜像字符或竖排 Cw90；下一步应由 `graphics/text/shaping` owner 输出真实 run 后替换当前 neutral projection |
| 2026-06-27 | 计划建立 | planned | 整形后端接入 + UAX#9 BIDI + 竖排路线;ShapedGlyph 扩展朝向/标志位 | 文档 | SH-M1 替换启发式(render/14 硬切换 #1);喂 03 度量 |
| 2026-06-28 | SH-M1 首段:共享文本服务接入 UI shaper 状态 | runtime_text_shared_metrics_owner_core_check_passed_focused_test_timeout_visual_evidence | `ui/text/shaper.rs` 将 active layout backend 收敛为 `SharedTextService`,Native/SDF render-mode intent 不再记录 "backend not connected" fallback;`graphics/text/layout/measure.rs` 隔离 `glyphon`/`cosmic_text` 度量类型,UI 只消费 neutral `UiResolvedTextLayout`/`UiSize` | `rustfmt` 通过;旧 fallback 文案静态扫描为空;`text_shaper` focused tests 更新为 shared service 语义;`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1` 通过(仅既有 warning);截图 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`;focused Cargo test 超时无 diagnostics | 完整 SH-M1 仍需 `ShapedGlyphRun`/cluster source_range/脚本分段/UAX#9 替换 `layout_engine/visual_order.rs`;本切片不实现竖排或镜像字符 |

