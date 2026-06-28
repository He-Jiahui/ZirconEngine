---
related_code:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/ui/surface/render/text_measure.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
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
- `measure_cache.rs`:宽度桶缓存在,但喂启发式宽度;无子范围度量。
- `hit_test.rs`:按 fragment 矩形 + 均匀 advance 推 offset → 需改 cluster 反查(接 `02` `source_range`)。
- 无 justify、无 tab stop、无 shrink-to-fit、无竖排布局。

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

**Rust/wgpu 落地**:`unicode-linebreak`(UAX#14 break opportunities,cosmic-text 内置)、cosmic-text `Buffer::set_size` + `layout_runs`(已做断行+对齐,可直接消费其行结果)。CJK 禁则与 justify 在 cosmic-text 结果之上后处理。

## 4. 目标架构

```
ShapedGlyphRun(02, 无宽度约束 + 断点机会) + LayoutConstraints { wrap_width, wrap_mode, align, justify, overflow, tab, orientation }
  └─ line_break(UAX#14 机会 + CJK 禁则 + 连字符) → 贪心/逐字断行 →
     measure(真实 advance/kerning,子范围可查) → align/justify(行内分布) →
     overflow(ellipsis/shrink/clamp) → LaidOutText { lines[LaidOutLine], size, baseline 表 }
```

度量与布局分两层:**measure-only**(taffy 测量闭包用,只算尺寸,走 shaped+measure 缓存,不产顶点)与 **full layout**(产 `LaidOutText`,含每字形定位)。两者共享 shaping 与断行,measure 短路在 size 计算后(接 `editor_layout/17` 两阶段 + `render/14` "measure 必须走 shaped cache")。

## 5. 里程碑

### LB-M1 真实度量(度量=绘制)

实施切片:
1. `graphics/text/layout/measure.rs`:基于 `ShapedGlyphRun` 的 `measure_text_size`、子范围度量 `measured_width(run, byte_start, byte_end, include_kerning)`(UE `GetMeasuredWidth` 对齐);ascent/descent/line_height 取真实 face metrics。
2. 替换 `ui/text` 启发式 measure:`text_measure.rs`、`measure_cache.rs` 改喂真实度量(`render/14` 硬切换 #7);baseline 取真实 ascent。

测试:`text_measure_width_matches_shaped_advance_sum`、`text_measure_subrange_matches_ue_semantics`、`text_measure_cjk_fullwidth_advance`。

### LB-M2 换行与禁则

实施切片:
1. `layout/line_break.rs`:UAX#14 机会(cosmic-text/unicode-linebreak)+ 贪心断行;`wrap_mode = None|Word|Glyph|WordSmart`;长词逐字回退。
2. CJK 行首尾禁则(避头尾):行首禁止标点(`、。」』）` 等)、行尾禁止开括号(`「『（` 等);kinsoku 表 + 挤压/移出策略(对照 godot)。
3. 连字符:soft hyphen(U+00AD)断点 + 末尾连字符字形;字典连字符为可选 feature。

测试:`text_wrap_word_breaks_at_uax14_opportunities`、`text_wrap_cjk_kinsoku_no_leading_punctuation`、`text_wrap_long_word_falls_back_to_glyph`、`text_wrap_soft_hyphen_inserts_hyphen`。

### LB-M3 对齐 / 两端对齐 / 溢出

实施切片:
1. align(left/center/right/start/end 随 BIDI);justify(词间均分 + CJK 字间 + kashida 可选)。
2. overflow:ellipsis(首/中/尾,对齐 godot `OverrunBehavior`)、shrink-to-fit、clamp 字号(`editor_layout/17` 规则4);tab stop。

测试:`text_align_end_follows_rtl_base_direction`、`text_justify_distributes_word_and_cjk_gaps`、`text_overflow_ellipsis_middle_keeps_head_tail`、`text_shrink_to_fit_scales_within_bounds`。

### LB-M4 竖排布局

实施切片:
1. 竖排列切分(主轴 y、列宽=字号+列距)、列对齐、竖排禁则;`LaidOutLine` 在竖排语义下为"列"。

测试:`text_vertical_columns_wrap_on_height`、`text_vertical_kinsoku_applies`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/layout/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | 布局入口装配(薄):`layout(run, constraints) -> LaidOutText`、`measure(...) -> Vec2` |
| `measure.rs` | 真实度量:总宽/子范围宽/行高/ascent/descent;UE `GetMeasuredWidth` 对齐 |
| `line_break.rs` | UAX#14 机会 + 贪心断行 + 长词逐字 + 连字符 |
| `kinsoku.rs` | CJK 避头尾禁则表 + 挤压/移出 |
| `align.rs` | align + justify(词间/字间/kashida) |
| `overflow.rs` | ellipsis(首/中/尾)、shrink-to-fit、clamp、tab stop |
| `vertical_layout.rs` | 竖排列切分与对齐 |

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
}
pub struct LaidOutText { pub lines: Vec<LaidOutLine>, pub size: Vec2,
    pub run: Arc<ShapedGlyphRun> }   // 复用 02 字形,行只持区间
pub struct LaidOutLine { pub glyph_range: (u32, u32), pub origin: Vec2,
    pub baseline: f32, pub width: f32, pub ascent: f32, pub descent: f32,
    pub trailing_whitespace: f32 }

// 度量(UE FShapedGlyphSequence::GetMeasuredWidth 对齐)
pub fn measured_width(run: &ShapedGlyphRun, byte_start: u32, byte_end: u32, include_kerning: bool) -> f32;
pub fn measure_text_size(run: &ShapedGlyphRun, c: &LayoutConstraints) -> Vec2;
```

### 度量算法(对齐 UE)

- 总宽 = Σ glyph.advance(行内,trailing whitespace 不计入 content width,但 layout width 含)。
- **子范围度量**:给字节区间 `[s, e)`,累加 `source_range` 落入 `[s,e)` 的 glyph advance;`include_kerning=false` 时减去簇间 GPOS 调整(UE `bIncludeKerning` 语义)——供光标/选区精确定位。
- 行高:`Normal` = ascent+descent+line_gap(face hhea/OS2);`Scale(k)` = font_size×k;baseline = line_top + ascent。
- BIDI:度量按逻辑序累加(顺序无关于视觉序);命中测试用视觉序 + `source_range` 反查(`hit_test.rs` 改造,`render/14` 硬切换 #3)。
- 竖排:advance 在 y,width→height 语义对调。

### CJK 禁则(`kinsoku.rs`,对照 godot)

- 行首禁则集(不能出现在行首):`、。，．・：；！？）｝】〕〉》」』’”` …
- 行尾禁则集(不能出现在行尾):`（｛【〔〈《「『‘“`。
- 策略:断点落在禁则字符时,优先**前移**断点(把行首禁则字符挤到上一行末——"追い込み"),次选**移出**(把行尾禁则字符移到下一行——"追い出し");可配 squeeze 标点半角。

### 与既有路径硬切换(`render/14` 清单 #2/#3/#7)

| 现有 | 切换 |
|------|------|
| `layout_engine.rs` 全体(等宽 wrap/align/ellipsize/baseline) | 删除;语义迁 `graphics/text/layout/*`(真实度量重写) |
| `layout_engine/tests.rs` 期望值 | 按真实字形度量重标定 |
| `hit_test.rs::hit_test_text_layout` | 改基于 `ShapedGlyph.source_range` 反查;签名/返回类型不变 |
| `text_measure.rs::measure_text_size` | 改调 `graphics/text/layout::measure`(taffy measure 闭包,走 shaped cache) |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_measure_width_matches_shaped_advance_sum` | 度量宽 = Σadvance(含 kerning),与绘制端一致(度量=绘制) |
| `text_measure_subrange_matches_ue_semantics` | 子范围宽对 UE `GetMeasuredWidth` 期望表;include_kerning 两路径 |
| `text_measure_cjk_fullwidth_advance` | CJK 全角 advance = 字号(对照 face) |
| `text_wrap_word_breaks_at_uax14_opportunities` | 断点集 = UAX#14 机会(对照标准用例) |
| `text_wrap_cjk_kinsoku_no_leading_punctuation` | 行首无禁则标点;追い込み/追い出し正确 |
| `text_wrap_long_word_falls_back_to_glyph` | 超宽单词 Word 模式逐字断 |
| `text_wrap_soft_hyphen_inserts_hyphen` | U+00AD 断点末尾出连字符字形 |
| `text_align_end_follows_rtl_base_direction` | End 对齐在 RTL 段落靠左 |
| `text_justify_distributes_word_and_cjk_gaps` | 两端对齐词间 + CJK 字间均分,末行不拉伸 |
| `text_overflow_ellipsis_middle_keeps_head_tail` | 中部省略保头尾,`…` 宽度计入 |
| `text_shrink_to_fit_scales_within_bounds` | 缩放后宽≤bounds,字号≥min clamp |
| `text_hit_test_maps_pixel_to_source_offset` | 像素点→源字节 offset,affinity 正确(对照 cluster) |
| `text_vertical_columns_wrap_on_height` | 竖排按高度断列,列序正确 |

里程碑命令:`cargo test -p zircon_runtime text_measure --locked`、`text_wrap --locked`、`text_align --locked`、`text_overflow --locked`。

## 7. 风险与回退

- cosmic-text `layout_runs` 已做断行/对齐,优先消费其结果再后处理 CJK 禁则/justify;若其断行不可控,改用 `unicode-linebreak` 机会 + 自研贪心。
- Knuth-Plass 最优断行(段落级最小化破碎度)列为 V2,V1 用贪心(对齐多数引擎)。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-06-28 | LB-M3 Auto/Mixed first-strong base direction slice | runtime_text_lb_m3_first_strong_base_direction_check_passed | `ui/text/layout_engine.rs` 现在按 UAX#9 P2/P3 首强字符规则把 `Auto` 与暂存的 `Mixed` request 解析成具体 `LeftToRight` / `RightToLeft` paragraph base direction，再执行 logical Start/End 对齐；surface parser 与 interface enum 仍保留 `Mixed` 作为外部请求值，避免新增兼容 shim；`ui/tests/text_layout.rs` 的 mixed-direction extraction 断言同步为 resolved concrete base direction，同时保留 run-level visual order/source range 覆盖 | `rustfmt` 覆盖 touched layout/extraction files 通过；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-first-strong-check --message-format short --color never` 通过(仅既有 warnings)；Cargo test harness 编译先超过工具窗口，但产出 test binary 后直接运行 `first_strong` 通过 4/4，`start_end` 通过 6/6，`mixed_direction`、`neutral_separator`、`rich_directional_ellipsis` 各通过 1/1；截图 `docs/tests/runtime/text/runtime_text_first_strong_direction_preview_20260628.png` 已视觉检查，repo `target` 下无对应文本验证图 | 这只关闭 paragraph base direction 的 first-strong 首段；完整 UAX#9 level run/isolate/mirror、script segmentation、justify、tab stop、shrink/clamp、ellipsis variants、vertical layout 与完整 LB-M3/LB-M4 仍 pending |
| 2026-06-28 | LB-M3 logical Start/End alignment first slice | runtime_text_lb_m3_rtl_start_end_align_check_passed | `UiTextAlign` 保留 `Start` / `End` 逻辑值，`ui/surface/render/resolve.rs` 不再把 `"start"` / `"end"` 提前折叠成 left/right；`ui/text/layout_engine.rs` 按 resolved line direction 解析 Start/End；`ScreenSpaceUiTextBatch` 携带 `text_direction`，native glyphon 与 SDF draw-plan 对齐 helper 都按方向映射 Start/End，避免 render backend 丢失 RTL 语义 | `rustfmt --check` 覆盖 interface typography、surface resolve、layout engine、render batch、native/SDF text files 与相关 tests 通过；`cargo check -p zircon_runtime_interface --lib --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align-interface` 通过；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align` 通过(仅既有 warnings)；`cargo test -p zircon_runtime start_end --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align` 通过 3/3，覆盖 layout/native/SDF；`render_extract_preserves_logical_start_text_align` 通过 1/1；验证中先遇到结构守卫 `mesh_pipeline_variant_cache_owner.rs` 对 `gbuffer_record` moved-value 后续借用的编译阻塞，已改为借用拼接并用 `runtime_15_non_base_mesh_variant_cache_owner_is_wired` 通过 1/1 证明支撑层恢复；截图 `docs/tests/runtime/text/runtime_text_rtl_start_end_alignment_preview_20260628.png`，确认 repo `target` 下无对应文本验证图 | 这只关闭 Start/End 随显式 RTL base direction 的首段；`Mixed` 的真实 UAX#9 段落方向解析、justify、tab stop、shrink/clamp、ellipsis variants、vertical layout 与完整 LB-M3/LB-M4 仍 pending |
| 2026-06-28 | LB-M2 CJK open punctuation line-end first slice | runtime_text_lb_m2_cjk_open_punctuation_line_end_check_passed | `graphics/text/layout/kinsoku.rs` 增加行尾禁止开标点首段：当 UAX#14 chunk 以 `（` 等开标点结尾或开头时，把开标点与后续文本组成不可 glyph fallback 的 protected chunk；`line_break_chunks(...)` 继续只产 shared metadata，`ui/text/layout_engine.rs` 不持有开标点表 | RED 先证明 `"中（文"` 在单 glyph 宽 Word wrap 下会拆成 3 行；`rustfmt --check` 通过；`line_break_chunks_keep_cjk_open_punctuation_with_following_text` 通过，锁定 shared chunks 为 `"中"` / `"（文"` 且第二 chunk 不允许 glyph fallback；`text_wrap_cjk_kinsoku_no_trailing_open_punctuation` 通过，锁定 visual lines 为 `"中"` / `"（文"` 且无行以 `（` 结尾；既有 `text_wrap_cjk_kinsoku_no_leading_punctuation` 与 `word_wrap_uses_uax14_cjk_break_opportunities` 回归各通过 1/1；runtime lib check 通过(仅既有 warnings)；截图 `docs/tests/runtime/text/runtime_text_cjk_open_punctuation_preview_20260628.png`，确认 repo `target` 下无对应文本验证图 | 完整 LB-M2 仍需完整 greedy line breaker、完整 JIS/UAX line-head/line-tail 禁则表、squeeze/overhang policy generalization、tab/justify/shrink/ellipsis variants、竖排 layout 与更完整 glue 策略 |
| 2026-06-28 | LB-M2 long-word glyph fallback + NBSP glue first slice | runtime_text_lb_m2_long_word_nbsp_check_passed | `graphics/text/layout/line_break.rs` 继续作为 LB-M2 chunk owner，普通过宽 chunk 保持 `allow_glyph_fallback = true`，Word wrap 可退回 grapheme wrapping；含 U+00A0 的 chunk 标记 `allow_glyph_fallback = false`，让 NBSP glue group 在窄 frame 下保持同一不可断 run 并允许 overhang；UI 仍只消费 chunk metadata，不复制 NBSP 规则；验证还修复了下层 review guard `plugin_importer_dx/d13_importer_sdk.rs` 的 stale plan-status include 路径，避免无关支撑文件阻断 runtime lib-test 编译 | `rustfmt --check` 通过；`text_wrap_long_word_falls_back_to_glyph` 通过，锁定 `"abcd"` 在单 glyph 宽 Word wrap 下逐字分成 4 行；`word_wrap_keeps_non_breaking_space_group_together` 通过，锁定 `"a\u{00a0}b"` 在同宽度下保持 1 行且 measured width 超出 frame；runtime lib check 通过(仅既有 warnings)；截图 `docs/tests/runtime/text/runtime_text_long_word_nbsp_preview_20260628.png`，确认 repo `target` 下无对应文本验证图 | 完整 LB-M2 仍需完整 greedy line breaker、行尾禁则/open punctuation push-out、squeeze/overhang generalization、tab/justify/shrink/ellipsis variants、竖排 layout 与更完整 NBSP/ZWJ/emoji glue 策略 |
| 2026-06-28 | LB-M2 soft hyphen break suffix first slice | runtime_text_lb_m2_soft_hyphen_break_suffix_check_passed | `graphics/text/layout/line_break.rs` 为 `LineBreakChunk` 增加显式 `source_range` 与 `break_suffix` metadata，遇到 U+00AD 时把 soft hyphen 从 visual chunk text 移除，并把断行后显示的普通 `-` 绑定到源 soft-hyphen range；`ui/text/layout_engine.rs` 只在真实宽度换行时消费 pending break suffix，不在自然段落结束时显示 `-`；`layout_engine::source_subrange(...)` 与 `ui/text/hit_test.rs` 对 source/visual byte length 不一致的 run 采用保守映射，避免 `pre-` 的可见 hyphen 把 caret source offset 错算成普通 ASCII byte | `rustfmt --check` 通过；`text_wrap_soft_hyphen_inserts_hyphen` 通过，锁定 `"pre\u{00ad}fix"` 在窄 Word wrap 下输出 `"pre-"` / `"fix"` 且 visual text 不含 U+00AD；`text_hit_test_soft_hyphen_break_suffix_maps_to_source_hyphen` 通过，锁定行末命中映射到源 U+00AD 后方；runtime lib check 通过(仅既有 warnings)；soft-hyphen 改动后 `text_wrap_cjk_kinsoku_no_leading_punctuation` 与 `word_wrap_uses_uax14_cjk_break_opportunities` 回归各通过 1/1；截图 `docs/tests/runtime/text/runtime_text_soft_hyphen_preview_20260628.png`，确认 repo `target` 下无对应文本验证图 | 完整 LB-M2 仍需完整贪心断行 owner、行尾禁则/open punctuation push-out、不可断空白、long-word fallback 合同、squeeze/overhang generalization，以及后续 align/overflow/vertical owner |
| 2026-06-28 | LB-M2 CJK kinsoku line-start punctuation first slice | runtime_text_lb_m2_cjk_kinsoku_line_start_check_passed | 新增 `graphics/text/layout/kinsoku.rs`，在共享 layout 层处理行首禁则首段；`LineBreakChunk.allow_glyph_fallback` 让 UI Word wrap 遵循 runtime chunk metadata，而不是在 UI 层复制 CJK 标点表；`line_break_chunks(...)` 对关闭标点 suffix/leading chunk 关闭 glyph fallback，允许 `文。` 作为过宽 kinsoku chunk 保持在同一行；新增 focused test 锁定 `"中文。"` 窄宽度下分成 `"中"` / `"文。"`，无 line 以 `。` 开头 | RED 先失败为 3 行；`rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs` 通过；`cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_cjk_kinsoku_no_leading_punctuation` 通过 1/1；`word_wrap_uses_uax14_cjk_break_opportunities` 通过 1/1；`text_shape_` 通过 6/6；runtime lib check 通过(仅既有 warnings)；截图 `docs/tests/runtime/text/runtime_text_cjk_kinsoku_preview_20260628.png` | 完整 `kinsoku.rs` 仍需行尾禁则、open punctuation push-out、squeeze/overhang policy generalization、软连字符、不可断空白、long-word fallback 合同和后续 align/overflow/vertical owner |
| 2026-06-28 | LB-M2 消费首段: UAX#14 chunks drive Word wrap for CJK | runtime_text_lb_m2_uax14_word_wrap_cjk_check_passed | 新增 `graphics/text/layout/line_break.rs`，以 `ShapedGlyphRun` cluster soft-break flags 为唯一断点来源生成 `line_break_chunks(...)`；`ui/text/layout_engine.rs` 的 Word wrap 消费该共享 chunk 流而不是只按 ASCII space 分块；新增 CJK no-space regression，证明 `"中文"` 在窄 frame 下按 UAX#14 机会断成两行 | `rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs` 通过；`cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never word_wrap_uses_uax14_cjk_break_opportunities` 通过 1/1；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never` 通过(仅既有 warnings)；截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`，target 路径检查无文本验证图 | 完整 LB-M2 仍需把 chunk 流升级为贪心断行 owner、CJK 行首尾禁则、soft hyphen 插入、long-word glyph fallback 与不可断空白策略；LB-M3/LB-M4 的 justify/overflow/vertical layout 仍 pending |
| 2026-06-28 | LB-M2 数据面首段: UAX#14 break opportunity flags from shaped run | runtime_text_sh_m1_uax14_break_flags_focused_tests_passed | `graphics/text/shaping/line_break.rs` 先把 UAX#14 断点机会投影到 `ShapedGlyphClusterFlags`，为后续 `graphics/text/layout/line_break.rs` 的贪心断行、WordSmart 与 CJK kinsoku 提供真实 cluster 数据；当前 UI 仍未切到完整 LB-M2 line breaker | `rustfmt --check` 通过；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never` 通过(仅既有 warnings)；`cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_shape_` 通过 6/6，其中 word-space 与 CJK soft-break focused tests 覆盖 LB-M2 输入数据 | 完整 LB-M2 仍需 `layout/line_break.rs` 贪心断行、CJK 行首尾禁则、long-word glyph fallback 与 soft hyphen 插入；LB-M3/LB-M4 justify/overflow/vertical layout 仍 pending |
| 2026-06-28 | SH-M1 owner slice: cosmic-backed `ShapedGlyphRun` contract and isolated shaping owner | runtime_text_sh_m1_shaping_owner_core_check_passed_focused_libtest_blocked | Added neutral `core/framework/render/text/{shaped_run.rs,shaping_service.rs}` contracts and `graphics/text/shaping/{mod.rs,cosmic.rs}` owner; `cosmic.rs` is now the isolated glyphon/cosmic-text Buffer/LayoutGlyph projection point and emits glyph id, source_range, visual_range, advance, baseline, direction, cluster flags, and rotation contract data; `graphics/text/layout/measure.rs` now derives line width/line metrics/per-grapheme advances from `ShapedGlyphRun` instead of importing backend text types directly | scoped rustfmt passed; `cargo check -q -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` passed with existing warnings only; focused `cargo test -q -p zircon_runtime text_shape_ --lib --no-default-features --locked` compiles past old importer include guards but is blocked by unrelated camera-loop lib-test closure signature errors; screenshot evidence remains `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png` and target image checks are false | Actual fallback-selected `FontFaceId` remains pending; full UAX#9 isolate/mirroring, script segmentation, UAX#14 break data, CJK kinsoku, vertical metrics/rotation policy beyond contract fields, and UI hard cutover from `visual_order.rs` remain future SH/LB slices |
| 2026-06-28 | LB-M1 DTO 首段: measured grapheme advances exported to render layout | runtime_text_sh_lb_m1_shaped_glyph_advances_interface_check_passed_runtime_check_timeout | `ui/text/layout_engine.rs` 将 shared backend `measured_grapheme_widths(...)` 写入 `UiResolvedTextLine.glyph_advances`；`UiShapedText::from_resolved_layout(...)`、rich text paint runs 与 editable decoration geometry 消费该数组生成非等宽 glyph frames/paint frames；新增 focused tests 覆盖 `"Wi"` 非均匀 advance 与 shaped glyph frame projection | scoped rustfmt 通过；`zircon_runtime_interface --lib` 与 `zircon_runtime_interface --tests` Cargo check 通过；runtime no-default check 244s 编译超时无 Rust diagnostics，匹配验证进程已停止；截图路径检查沿用 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png` | 完整 LB-M1 仍需 `ShapedGlyphRun` 上的 UE-style subrange measure/kerning semantics；LB-M2/LB-M3/LB-M4 的 UAX#14、CJK 禁则、justify、shrink、tab、竖排列布局仍 pending |
| 2026-06-27 | 计划建立 | planned | 真实度量(UE 对齐)+ UAX#14 换行 + CJK 禁则 + justify + 竖排布局路线 | 文档 | LB-M1 度量=绘制(editor_layout/17 G1);喂 render/14 顶点生成 |
| 2026-06-28 | LB-M1 首段:真实 glyph metrics 驱动 UI 度量/换行/缓存/命中 | runtime_text_shared_metrics_owner_core_check_passed_focused_test_timeout_visual_evidence | 新增 `graphics/text/layout/measure.rs` 的 line width、line metrics、visual prefix width、grapheme widths、width bucket;`ui/text/layout_engine.rs` 用共享宽度做 wrap/align/ellipsis/baseline,`measure_cache.rs` 用共享 width bucket,`hit_test.rs` 用 measured grapheme midpoint;触达路径移除 fixed half-em 等宽公式 | `rustfmt` 通过;静态扫描确认触达文本路径无旧等宽公式;focused tests 更新 `WWW > iii`、combining-mark ellipsis、measured cache bucket、measured hit-test points;`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1` 通过(仅既有 warning);截图 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`;focused Cargo test 超时无 diagnostics | 完整 LB-M1 仍需基于 `ShapedGlyphRun` 的子范围度量/kerning 语义;LB-M2/LB-M3/LB-M4 的 UAX#14、CJK 禁则、justify、shrink、竖排布局仍 pending |

