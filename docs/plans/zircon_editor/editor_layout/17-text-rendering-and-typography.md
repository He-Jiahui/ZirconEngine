---
related_code:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/CompositeFont.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Text/STextBlock.h
  - dev/godot/servers/text/text_server.h
  - dev/godot/scene/resources/font.h
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/slint/internal/core/textlayout.rs
  - dev/material-ui/packages/mui-material/src/styles/createTypography.js
plan_sources:
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_layout/10-real-rendering-pipeline-and-contract.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15a-page-tab-strip-overflow.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - docs/plans/zircon_runtime/text/index.md  # 及 text/01-09 子计划目录(文本实现权威,2026-07-02 评审收口)
  - docs/plans/engine-code-structure-convention.md
status: in_progress
---
# 17 文本渲染与排版规范(度量一致性 / DPI 重栅格 / 换行 / 自适应)

> 本文是文本这条线的**编辑器侧排版验收规范**(实现权威 = `docs/plans/zircon_runtime/text/`,见下节,2026-07-02 评审收口)。与 `16`(布局相对/DPI 根缩放)分工:`16` 管"区域/控件几何在不同分辨率下怎么摆",`17` 管"文本字形如何被准确测量与清晰绘制"。**文本字形随 `scale_factor` 重栅格**是 `16` 三层模型第①层(根 DPI 缩放)在文本上的落地。取 dev 引擎(UE Slate / godot / bevy / slint / material-ui)的文本思想,落到既有 **glyphon + fontsdf 双后端 + `ui/text` 布局引擎**,不重写字体库(与 `14` 同调:取思想不取运行时)。

## 与 zircon_runtime/text 的分工(2026-07-02 评审收口)

文本这条线的**实现权威 = `docs/plans/zircon_runtime/text/`(index + 01-09 九个子计划)**;本文降级为**编辑器侧排版验收规范 / 消费方**,不再持有 runtime 文本文件的实施指令。分工如下:

- **本文保留**:度量=绘制四规则(§1)、DPI 重栅格**验收契约**(§3.2 的断言,不含实现)、编辑器默认排版参数(默认 Word 换行、省略特例、字号/行高 token 消费)、换行自适应决策树(§3.5)、shrink-to-fit 收敛协议。
- **让渡给 runtime text**:`layout_engine.rs` / `sdf_font_bake.rs` 等 runtime 文件的实施指令——整形/度量归 `text/02`/`text/03`,栅格/atlas key 归 `text/04`,SDF 管线归 `text/05`。本文 §5 的落点表自此仅作历史定位参考,以 text 子计划为准。

把"字符错位/溢出、字体像素化、换行与文本大小自适应"从缺陷收敛为规范。四条根规则:

1. **测量 = 绘制**:文本布局几何来自**真实字形度量**(advance/kerning/ascent/descent),与绘制端同一来源——根治错位/溢出。
2. **字形随 DPI 重栅格**:字形按**物理像素**光栅化(`physical_px = logical_px × scale_factor`),scale 变即重栅格——根治像素化(接 `16` §3.4)。
3. **默认多行换行**:默认 word 边界换行 + 超长词逐字回退;装不下才省略;单行+省略是 chrome 固定高条目的**显式特例**。
4. **文本可自适应**:auto-wrap 两阶段布局;可选 shrink-to-fit / clamp 字号,按节点声明。

## 2. 现状(按代码核实)

### 2.1 已经成立的(不重做)

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 双后端文本绘制 | `runtime .../scene_renderer/ui/text.rs`(glyphon bitmap atlas)、`.../ui/sdf_render.rs`(fontsdf SDF) | 绘制端用**真实字形度量**(glyphon `Shaping::Advanced`;SDF `glyph.metrics.advance`/`ascent`) |
| 换行已有三模式 | `runtime .../ui/text/layout_engine.rs::wrap_source_runs` | `UiTextWrap::{None,Word,Glyph}` |
| 省略已有 | `layout_engine.rs::ellipsize_line` | `UiTextOverflow::Ellipsis` 末尾追加 `…` |
| 字号/行高 token | `iface .../surface/render/resolved_style.rs` | `DEFAULT_FONT_SIZE=16.0`、`DEFAULT_LINE_HEIGHT_SCALE=1.2` |
| 纹理线性过滤 | `runtime .../ui/sdf_render.rs` | sampler `mag/min_filter = Linear` |

### 2.2 两个根因(本文要修正的目标)

**G1 — 测量与绘制系统性不一致(错位 / 溢出 / "Sce"类截断的测量侧根因)**

布局引擎 `layout_engine.rs` **完全不接触真实字体度量**,用等宽近似:

```rust
// layout_engine.rs:490-492  每字符一律 0.5×字号,忽略 'i' vs 'w'、忽略 kerning
pub(super) fn text_advance(font_size: f32) -> f32 { (font_size * 0.5).max(1.0) }
// layout_engine.rs:486-488  宽度 = grapheme 数 × 等宽
fn measure_width(text: &str, char_advance: f32) -> f32 { grapheme_count(text) as f32 * char_advance }
// layout_engine.rs:21-31  measure_text_size 用上面的近似
let char_advance = text_advance(font_size);
// layout_engine.rs:82  baseline 硬编码,与 SDF 真实 ascent 不一致 → 垂直错位
baseline: font_size * 0.8,
```

`layout_text`(`:33-90`)的 `max_width`/`line_capacity`/换行/`ellipsize_line`/`aligned_x` **全部**建立在 `char_advance` 上。绘制端却用真实 advance/ascent → 测量≠绘制 → 文本溢出容器、过早换行、截断成 `Sce`、垂直错位。`15a` 已从**几何单源**侧修了页签截断,但**测量侧的根因在本文**。

**勾稽(2026-07-02 评审收口)**:上段"完全不接触真实度量"的表述已**部分过时**——retained-host 已消费 runtime `layout_text` 的 `glyph_advances` 与 `measure_runtime_text_width_with_style(...)`,runtime 侧 Stage A 已切 shared measurement(`graphics/text/layout/measure.rs` 共享宽度驱动 wrap/align/ellipsis/baseline,见 §12 与 `text/03` §12 记录)。G1 剩余缺口收敛为:subrange kerning 语义(`include_kerning` 的 GPOS delta)、baseline/ascent/descent 单源、native/SDF parity;由 `text/03`/`text/04` 关闭,本文以 §7 验收断言守护。

**G2 — 字形按固定字号光栅化、不随 DPI 重栅格(像素化)**

```rust
// sdf_font_bake.rs:243-245  按 font_size 光栅化,无 scale
fn font_size_milli(font_size: f32) -> u32 { (font_size.max(1.0) * 1000.0).round() as u32 }
// sdf_font_bake.rs:112  atlas key 只含 font_size_milli,不含 scale_factor
font_size_milli: font_size_milli(font_size),
// sdf_font_bake.rs:122,127  以该字号取 SDF 度量/光栅
let px = key.font_size_milli as f32 / 1000.0;
```

`SdfAtlasGlyphKey` 不含 `scale_factor`,高 DPI / 窗口缩放时字形被纹理**拉伸**而非重栅格 → 糊。linear 过滤救不了光栅分辨率不足。这是 `16` R4("`scale_factor` 捕获却未参与布局")在文本上的延伸。

### 2.3 仍缺的

- **文本大小自适应**:无 fit-to-width / clamp 字号(搜不到 `auto_size`/`fit_text`/`scale_to_fit` 用于字号)。
- **baseline/ascent/descent 未 token、未单源**:启发式侧 `font_size*0.8`,SDF 侧真实 ascent,两套。
- **测量↔绘制无一致性 guard**:没有"测量宽 = 绘制宽"的测试拦截回归。
- **换行点用 `max_chars` 近似**(`= max_width / char_advance`),非真实字形宽。

## 3. 设计

### 3.1 测量↔绘制单源(核心,治 G1)

引入**统一字形度量提供者**,测量与绘制消费同一来源,替换 `font_size*0.5`/`font_size*0.8` 近似:

- 布局测量改用真实 `advance`(逐字形)+ `kerning`(相邻对)+ `ascent`/`descent`;宽度 = Σadvance + Σkerning,**不在测量阶段取整**(取整只在上屏/scissor,如现 `text.rs`/`render.rs` 的 floor/ceil)。
- baseline = 真实 ascent + 行内垂直居中余量,measure 侧与 SDF 侧同公式。
- 度量来源 = 绘制端已用的 fontsdf/glyphon 字体,经 provider 暴露给 `ui/text` 布局,避免 double-load。

对标 UE `FShapedGlyphEntry`(`dev/UnrealEngine/.../SlateCore/Public/Fonts/FontCache.h:153-165`)——每字形精确 int16 度量、不在测量阶段取整:

```cpp
int16 XAdvance = 0;   // 下一字形的水平步进
int16 YAdvance = 0;   // 纵向步进(竖排)
int16 XOffset  = 0;   // 绘制水平偏移(相对 origin)
int16 YOffset  = 0;   // 绘制纵向偏移(相对 baseline)
int8  Kerning  = 0;   // 与下一字形的 kerning
```

UE 测量(`FShapedGlyphSequence::GetMeasuredWidth`)精确累加 advance,绘制再加 XOffset/YOffset——测量与绘制对同一 shaped 序列,天然一致。Zircon 落点:provider 暴露同样的 advance/offset/kerning 给 measure 与 draw。

### 3.2 字体 DPI 重栅格(核心,治 G2)

- `SdfAtlasGlyphKey` **加入 `scale_factor`**(或量化的 scale 桶);光栅尺寸 = `physical_px = logical_font_size × scale_factor`。
- scale 变(拖到高 DPI 屏 / 改缩放)→ 以新物理像素**重栅格**新 atlas 条目;旧条目 LRU 淘汰。
- 逻辑字号(token,`16` §3.2 逻辑单位)→ 物理光栅链路:`token(逻辑) ──×scale──▶ 物理 px ──光栅──▶ atlas ──linear 采样──▶ 上屏`。
- hinting 策略:引用 `text/04` 的 `HintingMode::{None,Vertical,Full}`(默认 `Vertical`,对小字号清晰);本文不自带 hinting 策略。(2026-07-02 评审收口)
- linear 过滤保留;SDF 边缘由距离场 smooth,bitmap 由物理分辨率足够保证清晰。

对标:
- UE hinting(`dev/UnrealEngine/.../SlateCore/Public/Fonts/CompositeFont.h:24-35`):`EFontHinting{ Default, Auto, Monochrome, None }`。
- UE SDF 生成(`.../SlateCore/Private/Fonts/SlateSdfGenerator.h`):距离场 spread 编码边界,缩放无损。
- Godot MSDF(`dev/godot/scene/resources/font.h:200-202`):`msdf` / `msdf_pixel_range`(=16)/ `msdf_size`(48–96)。
- Bevy 采样按 smoothing 切换(`dev/bevy/crates/bevy_text/src/font_atlas.rs:55-57`):`FontSmoothing::None => ImageSampler::nearest()`,否则 linear;subpixel 偏移分桶减少重复光栅。

### 3.3 换行规范(默认多行,治"换行问题")

默认 **word 边界换行 + 超长词逐字回退**,grapheme-aware,换行点用**真实字形宽**(接 3.1),非 `max_chars` 近似:

| `UiTextWrap` | 行为 | 对标 |
| --- | --- | --- |
| `Word`(**默认**) | word 边界换行;单词超行宽→逐字回退 | UE `ETextWrappingPolicy::AllowPerCharacterWrapping` |
| `Glyph` | 逐字形换行 | Godot `AUTOWRAP_ARBITRARY` |
| `None` | 不换行(配省略,chrome 固定高条目显式用) | Godot `AUTOWRAP_OFF` |

对标 UE(`dev/UnrealEngine/.../Slate/Public/Framework/Text/TextLayout.h:66-73`):

```cpp
enum class ETextWrappingPolicy : uint8 {
    DefaultWrapping = 0,          // 仅按 line-break iterator
    AllowPerCharacterWrapping,    // 单词太长时回退逐字
};
```

对标 Godot(`dev/godot/servers/text/text_server.h:98-103`):`AUTOWRAP_{OFF,ARBITRARY,WORD,WORD_SMART}`。换行点候选用 unicode line-break / word boundary(grapheme cluster 不被拆),而非空格简单切分(参考 `dev/slint/internal/core/textlayout.rs` 的 line-break + cluster 组装)。

**CJK 条款(2026-07-02 评审收口)**:CJK 文本按 UAX#14 逐字可断,并遵守行首尾禁则(行首禁则:句读/闭括号等不落行首;行尾禁则:开括号等不落行尾;实现归 `text/03` kinsoku)。默认 `Word` 策略在 CJK 文本(无空格分词)自动退化为逐字可断,作者无需显式切 `Glyph`;`word_wrap_uses_real_advance` 测试补 CJK 用例(无空格中文在窄容器按 UAX#14 机会断行)。

### 3.4 省略与截断(trim 与 ellipsis 分离)

溢出行为枚举(对标 Godot `OverrunBehavior`,`text_server.h:123-131`):

```cpp
enum OverrunBehavior {
    OVERRUN_NO_TRIMMING, OVERRUN_TRIM_CHAR, /* ... */
    OVERRUN_TRIM_ELLIPSIS, OVERRUN_TRIM_WORD_ELLIPSIS, /* + _FORCE 变体 */
};
```

Zircon:`clip / ellipsis-char / ellipsis-word`;省略号 `…` 宽度取**真实字形宽**(修当前 `ellipsize_line` 内联估算);trim(裁到边界)与 ellipsis(追加省略号)逻辑分离,避免重复应用。多行时只对末行省略(现 `layout_text` 已有 `truncate(line_capacity)` + 末行 `ellipsize`,改其宽度来源即可)。

### 3.5 文本大小自适应(治"文本大小自适应")

两类策略,**默认换行**,自适应按节点显式声明:

1. **auto-wrap 两阶段布局**:① 无宽约束测期望尺寸 → ② arrange 已知宽重排。对标 UE `STextBlock`(`dev/UnrealEngine/.../Slate/Public/Widgets/Text/STextBlock.h:104-110`):

   ```cpp
   SLATE_ATTRIBUTE( float, WrapTextAt )   // 固定宽换行(无延迟,首选)
   SLATE_ATTRIBUTE( bool,  AutoWrapText ) // 按算得宽换行;注释明言"wrapped size computed at least one frame late"
   ```
   `WrapTextAt`(已知宽)无延迟、首选;`AutoWrapText`(依赖布局算得宽)两阶段、晚一帧。Zircon 接 Taffy measure callback:文本节点先报期望尺寸,容器定宽后重排。

2. **shrink-to-fit / clamp 字号**(可选,显式):容器装不下且节点声明可缩时,clamp 字号到 `[min,max]`。对标 Godot `shaped_text_fit_to_width`、Material-UI `clamp(min,pref,max)` + `pxToRem`(`dev/material-ui/.../styles/createTypography.js`)。**不做默认**(默认换行,保密度一致)。

   **收敛协议(2026-07-02 评审收口)**:clamp 在 **arrange 之后的后处理阶段单次执行**,不回喂 measure——避免"缩字号→期望尺寸变→容器重排→再缩字号"的振荡环。同一帧内 shrink 结果不参与下一轮 taffy 求解;测试矩阵补 `shrink_does_not_oscillate`。

   决策树(节点声明优先级):`wrap(默认) → ellipsis → shrink/clamp`。UE `LineHeightPercentage`(`STextBlock.h:121`)用乘数缩行距、不改字号,Zircon 同(行距走 `line_height_ratio`)。

### 3.6 baseline / 行高 / 对齐一致

- baseline = 真实 ascent(measure 与 SDF 同公式),替换 `font_size*0.8`。
- 多行行距统一走 `DEFAULT_LINE_HEIGHT_SCALE`(=1.2)/ `METRICS.line_height_ratio`(`15b`)。
- **行高双源裁决(2026-07-02 评审收口)**:`DEFAULT_LINE_HEIGHT_SCALE`(iface `resolved_style.rs`)与 `METRICS.line_height_ratio`(`15b`)当前是两处常量。裁决:**唯一权威 = `01` typography token**,两处均改为该 token 的投影(编译期投影亦可,但值必须同源)。收束条件:`01` 的 typography token 交付并被 `20` 级联消费后,两处常量删除、只留投影;在此之前任何一处改值必须同步另一处并在 §12 登记。
- baseline/ascent/descent 纳入度量来源单源(provider 提供;字号/行距 token 归 `01`/`15b`,字形度量随字体)。
- 垂直对齐(top/center/baseline)单源;`MinDesiredWidth`(UE `STextBlock.h:130`)对应"文本节点最小期望宽"防塌缩。

## 4. 接口与数据结构草案(Rust,规范形态非实现)

> **`GlyphMetricsProvider` 草案废弃(2026-07-02 评审收口)**:原逐字符 `advance(ch)/kerning(a,b)` API 无法表达连字/GSUB/GPOS/cluster,与 §3.1 UE 样板"测量与绘制对同一 shaped 序列"矛盾——逐字符查询绕开整形,必然与绘制端 shaped 结果分叉。度量契约改为**引用** runtime text 的契约类型:`ShapedGlyphRun` / `TextShapingService`(`text/02`)与子范围度量 `measured_width(run, byte_start, byte_end)`(`text/03`)。测量与绘制消费同一 shaped run,天然一致;本文不再自定义度量接口。

```rust
/// 度量来源 = runtime text 契约(text/02 ShapedGlyphRun / TextShapingService;
/// text/03 measured_width(run, byte_start, byte_end)),本文只消费不定义。
/// 文本布局输入:换行/溢出/自适应/缩放
pub struct TextLayoutInput<'a> {
    pub run: &'a ShapedGlyphRun,          // text/02 整形产物(替换已废弃的逐字符 provider)
    pub wrap: UiTextWrap,                 // 默认 Word
    pub overflow: UiTextOverflow,         // clip / ellipsis
    pub auto_size: Option<FitPolicy>,     // None=仅换行
    pub scale_factor: f32,                // 接 16 §3.4
}
pub enum FitPolicy { ShrinkToFit { min_px: f32 }, ClampFontSize { min_px: f32, max_px: f32 } }
```

> **`SdfAtlasGlyphKey { scale_milli }` 草案废弃(2026-07-02 评审收口)**:atlas key 的实现权威 = `text/04` 的 `GlyphRasterKey { px_size_bucket, subpixel_bin, format, hinting, synthetic }`(`px_size_bucket` 已按 `logical_px × scale_factor` 量化,天然覆盖 scale)。本文只保留验收断言:**scale 变即重栅格**——`scale_factor` 变化必须导致 key miss 与新物理像素栅格条目(见 §7 `glyph_rerasterized_on_scale_change`)。

## 5. 模块与文件落点(后续切片指引,本文不写代码)

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `runtime .../ui/text/` 下 glyph metrics provider owner | 桥接 fontsdf/glyphon 度量给布局 |
| 改 | `layout_engine.rs` | `measure_text_size`/`measure_width`/`text_advance`/`baseline` 改读 provider;换行/省略宽度来源换真实度量 |
| 改 | `sdf_font_bake.rs` | `SdfAtlasGlyphKey` 加 scale;按物理像素重栅格 |
| 改 | `metrics.rs`(`15b`) | baseline/行距度量单源接入 |

遵 `engine-code-structure-convention.md`:owner 叶子承载、根 wiring 薄、隐藏内容(省略/截断)`log` 标注。

## 6. 里程碑切片化

| # | 切片 | 验证命令 |
| -- | --- | --- |
| S1 | 文档规范成文 + 13/16/15a 对齐 | 交叉引用闭合;无新增代码 |
| S2 | glyph metrics provider + 测量改真实度量(治 G1) | `cargo test -p zircon_runtime text_measure --locked` |
| S3 | atlas key 加 scale + DPI 重栅格(治 G2,接 16) | `cargo test -p zircon_runtime sdf_rasterize_scale --locked` |
| S4 | 默认多行换行接真实宽度 + 省略宽度真实化 | `cargo test -p zircon_runtime text_wrap --locked` |
| S5 | 文本自适应(auto-wrap 两阶段 + 可选 clamp) | `cargo test -p zircon_editor --lib text_autosize --locked` |
| S6 | baseline token 化 + 三档截图验收 | `capture_m3_gui_acceptance_visual_artifacts --ignored` |

## 7. 测试矩阵

| 测试 | 断言 |
| --- | --- |
| `measure_matches_render_width` | 布局测量宽 = 绘制宽(容差内),变长文本不累积偏差 |
| `baseline_consistent_measure_vs_sdf` | measure baseline = SDF baseline |
| `glyph_rerasterized_on_scale_change` | scale 1.0→2.0 触发新物理像素 atlas 条目 |
| `no_pixelation_at_2x` | 2x 下字形物理光栅分辨率约翻倍 |
| `word_wrap_uses_real_advance` | 换行点由真实字形宽决定,非 `max_chars`;CJK 用例:无空格中文在窄容器按 UAX#14 逐字可断 + 禁则(2026-07-02 评审收口) |
| `long_word_falls_back_to_char` | 超行宽单词逐字回退(对标 UE AllowPerCharacterWrapping) |
| `ellipsis_width_is_real_glyph` | `…` 占位用真实字形宽 |
| `shrink_to_fit_clamps_within_min_max` | clamp 字号落 `[min,max]` |
| `shrink_does_not_oscillate` | clamp 为 arrange 后单次后处理、不回喂 measure;连续帧字号稳定不振荡(2026-07-02 评审收口) |
| `auto_wrap_two_pass_reflows` | 容器定宽后重排,行数随宽变化 |

## 8. 风险与对策

- 风险:测量改真实度量引发全编辑器文本几何回退。对策:provider 先在单 slot 验证;字体未加载时保留启发式 fallback;`measure_matches_render_width` 守回归。
- 风险:DPI 重栅格增 atlas 内存。对策:scale 量化分桶 + LRU 淘汰;只对在用 scale 重栅格。
- 风险:两阶段 auto-wrap 晚一帧抖动(UE 已注明)。对策:已知宽优先用 `WrapTextAt` 式直接定宽,`AutoWrapText` 式仅必要时用。

## 9. 完成定义

测量↔绘制单源(provider)成文并点名 G1;DPI 重栅格规范点名 G2、接 16;默认多行换行 + 省略分离 + 真实宽度;文本自适应(两阶段 + 可选 clamp)决策树;baseline/度量单源;`13`/`16`/`15a` 与本文对齐交叉引用;dev 源码证据(UE/godot/bevy/material-ui)落表。

## 10. 边界约束

不重写 glyphon/fontsdf(双后端保留);不做国际化复杂整形(CJK/RTL/BiDi/ligature)——本轮只在 provider 预留 shaped 接口、标注后续;DPI 根缩放归 `16`、约束几何与 measure callback 归 `13`、色/字号 token 归 `01`/`15`;运行时能力(provider 接管线)若属运行时构建回流 `editor_ui/`。本文只立规范 + 指引落点,不产出代码。

## 11. 参考实现对照(dev/ 源码锚点)

| 维度 | 锚点 | 取什么 |
| --- | --- | --- |
| 精确字形度量 | `dev/UnrealEngine/.../SlateCore/Public/Fonts/FontCache.h:153-165` | `FShapedGlyphEntry{XAdvance,YAdvance,XOffset,YOffset,Kerning}`,测量不取整 |
| SDF 字体 | `dev/UnrealEngine/.../SlateCore/Private/Fonts/SlateSdfGenerator.h` | 距离场 spread,缩放无损 |
| hinting | `dev/UnrealEngine/.../SlateCore/Public/Fonts/CompositeFont.h:24-35` | `EFontHinting{Default,Auto,Monochrome,None}` |
| 换行策略 | `dev/UnrealEngine/.../Slate/Public/Framework/Text/TextLayout.h:66-73` | `DefaultWrapping` + `AllowPerCharacterWrapping` |
| 两阶段自适应 | `dev/UnrealEngine/.../Slate/Public/Widgets/Text/STextBlock.h:104-130` | `WrapTextAt`/`AutoWrapText`(晚一帧)/`LineHeightPercentage`/`MinDesiredWidth` |
| autowrap / 省略 | `dev/godot/servers/text/text_server.h:98-103,123-131` | `AutowrapMode` / `OverrunBehavior` |
| MSDF | `dev/godot/scene/resources/font.h:200-202` | `msdf` / `msdf_pixel_range` / `msdf_size` |
| 采样 / subpixel | `dev/bevy/crates/bevy_text/src/font_atlas.rs:55-57` | `FontSmoothing::None=>nearest` 否则 linear;subpixel 分桶 |
| line-break / cluster | `dev/slint/internal/core/textlayout.rs` | unicode line-break + grapheme cluster 组装 |
| clamp / rem | `dev/material-ui/.../styles/createTypography.js` | `clamp(min,pref,max)` + `pxToRem` 响应式字号 |

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-07-03 | 17.S2 retained-host pen-origin subpixel phase | implemented-rustfmt-visual-recorded-cargo-deferred | 针对用户最新 editor crop 中“等线已生效但字符左右间距/字形左右落点仍怪”的问题,继续收敛 retained-host 文本定位链路:`paint_text/draw/layout.rs` 的 `RuntimeTextGlyph` 新增 `origin_x`,在 host/runtime advance projection 后同时保留字形 bitmap-left draw x 与 pen-origin x;`paint_text/draw/glyphs.rs` 改用 `origin_x` 选择 1/3 subpixel bin,只在非 finite 时 fallback 到 `glyph.x`;`paint_text/raster.rs` 将 `CachedGlyphMetrics.x_offset` 语义统一为相对 pen origin,swash 与 fontdue fallback 不再把 fontdue left bearing 混入 subpixel phase。新增回归 `runtime_positioned_glyphs_keep_pen_origin_for_subpixel_phase`、`swash_metrics_x_offset_is_relative_to_pen_origin`、`fontdue_fallback_metrics_x_offset_is_relative_to_pen_origin`。验证:`rustfmt --edition 2021 --check` 覆盖 touched retained-host text files 通过;`git diff --check --` 覆盖 touched files 通过(仅 LF-to-CRLF working-tree warnings);验证图 `docs/tests/runtime/text/runtime_text_editor_pen_origin_phase_preview_20260703.png`,SHA256 `03CA205AC0C1BD890C37955AEBEB8BE57C7242C9C3542D6DCB9D21F4F8C24032`;验证日志 `docs/tests/runtime/text/runtime_text_editor_pen_origin_phase_validation_20260703.log`,SHA256 `44D9B5BFBF12F7FE473E955012FD7A7D1E039433AD92B66DD89C9A26F7253DE7`;repo `target`、`E:\cargo-targets`、`D:\cargo-targets` 同名扫描 0。Cargo 因外部 cargo/rustc lanes 活跃延后,不计通过。 | 关闭 bitmap-left-bearing 参与 subpixel phase 导致的局部左右抖动首段。仍 open:真实 Workbench/Asset Browser/Component Atlas 窗口截图 QA、focused Cargo 绿跑、GPU draw-list/glyphon atlas migration、native/SDF paragraph parity 与 DPI/subpixel/hinting。 |
| 2026-07-03 | 17.S2 retained-host runtime style projection from global preferences | implemented-focused-passed-screenshot-metadata-recorded | 根据用户要求修正“不写死使用字体,允许偏好设置全局切换字体/颜色主题/样式”的文本链路:`paint_text/font.rs` 新增 `runtime_text_style_for_face(...)`,把 `HostTextPreferences`/resolved face/font request 投影为 runtime `UiResolvedStyle`,并让 `measure_runtime_text_width_with_style(...)`、`draw/layout.rs::runtime_single_line_text(...)` 和 paint_text tests 复用同一个 helper。控件仍只声明 UI/code/strong 等语义,不在按钮、tab、列表或容器内绑定具体字体；embedded fallback 仍只是空白兜底。验证:`rustfmt --edition 2021 --check --config skip_children=true` 覆盖本片 touched editor/runtime 文件通过;`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-runtime-style-0703` 通过(既有 warnings);focused `runtime_text_style_for_face_projects_the_same_font_request` 1/1 通过;direct retained text measure 3/3 通过;component atlas 与 M3 screenshot harness 均通过。截图证据位于 `docs/tests/editor`:component atlas 修改时间 `2026-07-03 02:58:37 +08:00`,82837 bytes,SHA256 `C8B263CED8D786FA3C98CE98EF1F8DE971E20740ED133BBFDE1CBE0F34D9D7D2`;Workbench 修改时间 `2026-07-03 02:58:58 +08:00`,72663 bytes,SHA256 `A4A46502C35184DEDD16D3BB93BFDD86E4832843FF04CAEE8B5D7DE942D3D543`;target/cargo-targets 同名截图扫描 0。 | 关闭 retained-host measurement/layout 使用分散 style 构造的缺口,并守住全局字体偏好入口。仍 open:偏好设置 UI/持久化、窗口级视觉质量评审、GPU draw-list/glyphon atlas migration、native/SDF paragraph parity、DPI/subpixel/hinting 与最终 Unreal-like 组件视觉统一。 |
| 2026-07-02 | 17.S2 retained-host render-command text style preservation | implemented-rustfmt-visual-recorded-focused-test-timeout | 针对用户截图中的编辑器字体混乱继续排查,确认 retained-host template render-command conversion 在 fallback text 和 runless shaped text 两条路径仍会把 style 重置为 default。`fallback.rs` 现在从 resolved `font_weight` 生成 strong paint style;`shaped.rs` 对 shaped clusters 分段发出 `HostPaintCommand` 并保留 `Plain/Code/...` run kind style,无 cluster fallback 才按 text font weight 生成 style。验证图 `docs/tests/runtime/text/runtime_text_editor_render_command_style_preservation_preview_20260702.png`,SHA256 `00322BDCD03C4B3F5BBFA361BC9095CCEA84C9C57FA96181F8D05687DC9589F1`;repo `target`、`E:\cargo-targets`、`D:\cargo-targets` 同名扫描均为 0。scoped rustfmt check 通过;focused Cargo `preserve` filter 约 184s 超时且无 Rust diagnostics,不计通过。 | 关闭 retained-host 命令转换层 style reset 的首段。仍 open:真实 Workbench/Asset Browser 窗口级字体一致性 QA、偏好 UI/持久化、runtime FontDatabase/FontFace DTO 全接入、native/SDF paragraph parity、DPI/subpixel/hinting 与 face-id reconciliation。 |
| 2026-07-02 | 17.S2 retained-host GPU draw-list font family+weight projection | implemented-focused-passed-visual-recorded-wrapper-blocked-by-unrelated-shader-test-cfg | 针对用户截图中的编辑器字体混乱继续补 retained-host → runtime GPU 链路:Chrome command stream 的 `runtime_draw_list` 现在从 `UiTextRunPaintStyle` 经 `paint_text` face helper 投影 runtime family 与 normalized weight,并写入 `UiSurfaceCommandKind::Text.font_family/font_weight`;runtime WGPU glyphon attrs 消费该 family/weight,Strong 只提升到至少 bold,code/mono 不再在 GPU draw-list 阶段丢回默认 UI 字体。验证:`rustfmt --check` 与 scoped diff-check 通过;`cargo check -p zircon_runtime --lib --no-default-features` 和 `cargo check -p zircon_editor --lib --no-default-features` 均通过;direct focused runtime/editor binaries 各 1/1 通过。Wrapper focused Cargo 复跑被无关 dirty shader/zshader test-cfg import drift 阻断,不计本切片失败。验证图 `docs/tests/runtime/text/runtime_text_editor_gpu_draw_list_font_projection_preview_20260702.png`,SHA256 `AE5D6B6847FD676E4876620D15073D320A1BF79561C0CD239B9C2745C0EADFB2`;repo `target`、`E:\cargo-targets`、`D:\cargo-targets` 同名扫描均为 0。 | 关闭 GPU draw-list family/style/weight 丢失的首段。仍 open:真实 Workbench/Asset Browser 窗口级字体一致性 QA、偏好 UI/持久化、runtime FontDatabase/FontFace DTO 全接入、baseline/DPI/native-SDF paragraph parity、subpixel/hinting 与 face-id reconciliation。 |
| 2026-07-02 | 17.S2 retained-host swash subpixel collapsed to grayscale alpha default | implemented-focused-passed-screenshot-metadata-recorded | 针对用户新截图中 `Preview / References / Metadata / Plugins` 小字号仍有彩色边缘/不像稳定等线观感的问题,本片复核 `docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md` 中 "subpixel AA 需要特殊 blend,V1 可用 grayscale AA" 的约束,并对照 Unreal Slate 默认 bitmap 路径:FreeType normal render 输出 alpha/grayscale mask,不是直接把 RGB subpixel mask 混进软件 framebuffer。`paint_text/raster.rs` 仍用 swash 读取当前全局偏好解析出的字体 face,但把 `Content::SubpixelMask` 折叠为 `CachedGlyphRasterFormat::AlphaMask` 灰度 coverage,避免 retained-host 软件混色产生蓝/橙边。未新增 concrete font family、平台字体路径、局部主题色或控件级字体策略;字体/颜色/样式仍由 `EditorTypographyTokens -> HostTextPreferences -> fontdb` 与后续 editor preference UI 切换。验证:Cargo wrapper 首次 904s 超时只完成编译,不计通过;随后直接运行已生成测试程序 `zircon_editor-2ac1fe4cc1fe2ea7.exe swash_subpixel_mask_is_collapsed_to_alpha_for_retained_host` 1/1 passed,`retained_text_raster_uses_swash_alpha_for_ui_face` 1/1 passed,ignored M3 screenshot harness 1/1 passed。截图证据:`docs/tests/editor/editor-window-m3-asset-browser-900x620.png` 修改时间 `2026-07-02 09:25:56 +08:00`,73127 bytes,SHA256 `4816E6D59A87642F980DFD20506D16D941B9B06DD1C2AD468B126F2FFB0ABDCE`;utility-tabs crop `editor-window-m3-asset-browser-utility-tabs-grayscale-alpha-crop-20260702.png` 修改时间 `2026-07-02 09:27:13 +08:00`,2836 bytes,SHA256 `D85A5CDFB4EBDA2A2869295B094E449554AC7AF40229F3409824A05115ADCC88`;3x nearest-neighbor crop `editor-window-m3-asset-browser-utility-tabs-grayscale-alpha-crop-3x-20260702.png` 修改时间 `2026-07-02 09:27:29 +08:00`,4748 bytes,SHA256 `50FF9BCDC7952C6BC935ADDB0EE3A6505F738AF9AE376B347126F547DA2F093D`;repo `target` 与 `D:\cargo-targets\zircon-editor-text-grayscale-aa-0702` 同名截图扫描均为 0。 | 关闭 retained-host 默认 RGB 子像素直混导致的彩边问题。仍 open:真正 ClearType/subpixel AA 需按 text 04 增加 subpixel bins 与正确 blend/背景假设;偏好设置 UI/持久化、runtime FontDatabase/FontFace DTO、GPU draw-list family/style/weight、baseline/DPI/native-SDF parity 与窗口级文本 QA 继续推进。 |
| 2026-07-02 | 17.S2 retained-host runtime font weight propagation | implemented-check-visual-passed-focused-tests-timeout | 针对用户截图中的编辑器字体混乱,本片补齐 retained-host 与 runtime 文本契约之间的字重缺口:`UiResolvedStyle` 新增 `font_weight`,runtime style 解析 `font.weight`/`font_weight`/`text_font_weight`,layout cache key 与 glyphon `Attrs::weight(...)` 消费该字段,`UiTextPaint`/run 与 font resource key 输出 `family:wNNN`;retained-host `measure_runtime_text_width_with_style(...)` 和 `runtime_single_line_text(...)` 从 `HostTextFontRequest.weight` 传入 runtime style。验证:`cargo check -p zircon_runtime_interface --lib`、`cargo check -p zircon_runtime --lib --no-default-features`、`cargo check -p zircon_editor --lib --no-default-features` 均通过;focused interface text-paint contract 1/1 通过;runtime/editor lib-test filters 分别 904s 编译超时无 Rust diagnostics,不计通过。验证图 `docs/tests/runtime/text/runtime_text_editor_font_weight_propagation_preview_20260702.png`,SHA256 `964E5942FD788A88D6BA8CFC756CD937A6D24E2E15C9C1A48AF0CB56FC86F832`,target/cargo-targets 同名扫描 0。 | 关闭 retained-host family 已传但 weight 丢失的首段问题。仍 open:偏好 UI/持久化、完整 runtime FontDatabase/FontFace、GPU draw-list family/style/weight、baseline/DPI/native-SDF parity 与真实窗口级文本 QA。 |
| 2026-07-02 | 17.S2 retained-host global typography preferences + utility-tab UI style | implemented-focused-passed-build-screenshot-metadata-recorded | 根据最新要求 supersede 2026-07-01 的 DengXian/mono/platform-path 固定字体路线:控件不再写死等线、Segoe、Cascadia 或 utility-tab mono 策略。`EditorDesignTokens` 新增 `EditorTypographyTokens`,默认只声明 `system-ui`/`monospace` 逻辑族与字号/字重/行高;`editor_tokens.zui` 同步主题默认值。`paint_theme/typography.rs` 新增 `HostTextPreferences`,retained host 启动时从 design tokens 投射并可由后续全局偏好 UI 覆盖。`paint_text/font.rs` 改为通过 `fontdb` 解析当前偏好 family/weight,仅在系统/用户 family 不可解析时使用 embedded fallback 防止空白;runtime measurement 继续经 `zircon_runtime::ui::surface::measure_text_size(...)` 并携带同一 resolved family。Asset Browser utility tabs 回到普通 UI text style,不再强制 code/mono。验证:`cargo fmt -p zircon_editor -p zircon_runtime_interface --check`;`cargo check -p zircon_runtime_interface`;`cargo check -p zircon_editor --no-default-features --jobs 1`;`cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host`;`cargo test -p zircon_runtime_interface editor_design_tokens` 5/5;focused `retained_text_font_request_uses_global_preferences` 1/1;focused `asset_browser_utility_tab_label_uses_ui_text_style` 1/1;focused `retained_ui_runtime_family_resolves_from_preferences_without_platform_paths` 1/1;M3 screenshot harness 1/1,均使用 `D:\cargo-targets\zircon-editor-text-preferences-0702`。截图证据只记录文件刷新与人工检查输入,不直接判定文本质量:Asset Browser `2026-07-02 03:22:27 +08:00`,73712 bytes,SHA256 `CB35E99D3D049F2455FBB57A9CB104C52563C6EB054A726BAF9E64E5108CDFDF`;utility-tabs UI-preference crop `2026-07-02 03:25:22 +08:00`,3256 bytes,SHA256 `C97A03DAF57EDBA2AF6B71E59FCC6E9862BFF6ACD27A0E41484669BD3355E1ED`;3x crop `2026-07-02 03:25:33 +08:00`,5345 bytes,SHA256 `D4D1B49CDCBA69AF80F5CD3102C48353B2B154AE46D5CE37D45B10CEDCE47ADE`;target scan `NO_MATCHING_SCREENSHOTS_IN_TARGETS`。 | 关闭“控件硬编码字体/utility tab 强制 mono”问题。仍 open:偏好设置 UI 持久化、运行时 FontDatabase/FontFace DTO 全接入、GPU draw-list family/style、baseline/DPI/native-SDF parity 与窗口级文本评审。 |
| 2026-07-01 | 17.S2 retained-host utility-tab mono route + compact slot | implemented-focused-passed-build-screenshot-metadata-recorded | 用户最新截图要求 Asset Browser utility tabs 本身是等宽观感,因此本片把 utility tabs 作为 code/mono text consumer 处理,不改变普通 UI DengXian 比例字体策略。`style_selector/workbench_button/tab_like.rs` 增加 Asset Browser/Assets Activity utility-tab classifier;`template_buttons/content/metrics.rs` 对这类 label 设置 `UiTextRunPaintStyle::code`,并把 label slot 改为 utility tab 4px padding;普通 strong button 测试改为非 utility control,防止强文本被误当 code。验证:`cargo fmt -p zircon_editor --check`;focused `asset_browser_utility_tab_label_uses_mono_text_style` 1/1;direct `button_label` 4/4;direct `asset_browser_utility_tabs_use_compact_slate_tab_strip_geometry` 1/1;direct `dock_tab_button_measures_label_with_declared_font_size` 1/1;M3 screenshot harness 1/1;component atlas screenshot harness 1/1;editor-host build passed in `D:\cargo-targets\zircon-editor-text-mono-tabs-0701`。截图证据只记录文件实际输出与修改日期,不直接判定最终文本质量:Asset Browser `2026-07-01 23:23:01 +08:00`,63538 bytes,SHA256 `1FE3DE25520212D2A63100E70DF9B760289FDEC713C628617146259334C0708A`;utility-tabs mono crop `2026-07-01 23:24:25 +08:00`,3081 bytes,SHA256 `E49FAEF998E7A10F5C4FB673713C2DBA21D816AD074D71C924525ECB3E4D9D9F`;3x crop `2026-07-01 23:24:25 +08:00`,5025 bytes,SHA256 `EDCBFEE05E31F092BD0721BBF7DF67DAF7ADDBEB47AA23216C5CE3126CE4EF8B`;component atlas `2026-07-01 23:22:35 +08:00`,74980 bytes,SHA256 `AA77420265441DFBA75E65F82D4BAE682160F1DC652E18DCE8368FF398CDEC89`;target scan `NO_MATCHING_SCREENSHOTS_IN_TARGETS`。 | 关闭上一行记录的 "若 utility tabs 要等宽,需新增 tab mono token" 首段。普通 UI 仍是 DengXian/等线比例字体,utility tabs/code 为等宽;完整 runtime FontDatabase/FontFace DTO、GPU draw-list family/style、baseline/DPI/native-SDF parity 与窗口级文本评审仍 open。 |
| 2026-07-01 | 17.S2 retained-host DengXian priority evidence refresh 22:09 | implemented-focused-passed-build-screenshot-metadata-refreshed | 重新生成并复核最新证据:普通 UI 字体候选仍为 `Deng.ttf -> Dengl.ttf -> segoeui.ttf`,strong UI 为 `Dengb.ttf -> Deng.ttf -> segoeuib.ttf`,code text 仍保留 Cascadia/Consolas mono。验证:`cargo fmt -p zircon_editor --check`;direct `retained_ui_runtime_family_prefers_dengxian_when_available_on_windows` 1/1;direct `paint_text` 20/20;direct `button_label` 4/4;direct dock-tab 度量 1/1;direct integer slot 1/1;component atlas screenshot 1/1;M3 screenshot harness 1/1;editor-host build passed in `D:\cargo-targets\zircon-editor-text-dengxian-0701` with existing warnings。截图证据只记录文件实际输出与修改日期,不直接判定最终文本质量:Asset Browser `2026-07-01 22:08:15 +08:00`,62627 bytes,SHA256 `DFA19780B09D6C2F323EC4B80407BF992B0202EAC033849892E942D1EAC84FDE`;utility-tabs DengXian crop `2026-07-01 22:09:39 +08:00`,2812 bytes,SHA256 `71EFF8ADD1DA1DB908389A9776B3C48DB04DC98971271C4347FA90E613435B26`;3x crop `2026-07-01 22:09:39 +08:00`,4596 bytes,SHA256 `D7C7D86ABCB0D7B65425AC31137D4F5163F8DD9A1FE23AC039D2465CB82CDC82`;component atlas `2026-07-01 22:07:54 +08:00`,74980 bytes,SHA256 `AA77420265441DFBA75E65F82D4BAE682160F1DC652E18DCE8368FF398CDEC89`;target scan `NO_MATCHING_SCREENSHOTS_IN_TARGETS`。 | 当前普通 UI 是 DengXian 比例字体,不是等宽;若 utility tabs 要等宽,需新增 tab mono token。完整 runtime text/GPU/native parity 与窗口级文本评审仍 open。 |
| 2026-07-01 | 17.S2 retained-host DengXian priority verification refresh | implemented-focused-passed-build-screenshot-metadata-refreshed | 当前生效代码复核:普通 UI 字体候选为 `Deng.ttf -> Dengl.ttf -> segoeui.ttf`,strong UI 为 `Dengb.ttf -> Deng.ttf -> segoeuib.ttf`,code text 仍保留 Cascadia/Consolas mono。回归测试 `retained_ui_runtime_family_prefers_dengxian_when_available_on_windows` 在 Windows 有 Deng 字体时锁定 `DengXian`,按钮 label line-height 继续 snap 到整数像素。验证:`cargo fmt -p zircon_editor --check`;direct `retained_ui_runtime_family_prefers_dengxian_when_available_on_windows` 1/1;direct `paint_text` 20/20;direct `button_label` 4/4;direct dock-tab 度量 1/1;direct integer slot 1/1;component atlas screenshot 1/1;M3 screenshot harness 1/1;editor-host build passed in `D:\cargo-targets\zircon-editor-text-dengxian-0701` with existing warnings。截图证据只记录文件实际输出与修改日期,不直接判定最终文本质量:Asset Browser `2026-07-01 21:55:51 +08:00`,62627 bytes,SHA256 `DFA19780B09D6C2F323EC4B80407BF992B0202EAC033849892E942D1EAC84FDE`;utility-tabs DengXian crop `2026-07-01 21:57:07 +08:00`,2824 bytes,SHA256 `869168A7A04768D1D9596B0C39E228FA984461CE19A03FB4432F6B4F6415F052`;3x crop `2026-07-01 21:57:07 +08:00`,4653 bytes,SHA256 `B9F323C0407EA9452385C0A94AECCD49A924BCBAD7BBC40B02B3EDF579B1AD29`;component atlas `2026-07-01 21:55:26 +08:00`,74980 bytes,SHA256 `AA77420265441DFBA75E65F82D4BAE682160F1DC652E18DCE8368FF398CDEC89`;target scan `NO_MATCHING_SCREENSHOTS_IN_TARGETS`。 | 本刷新确认 Segoe-priority experiment 已被 DengXian-priority 方案覆盖。普通 UI 当前是 DengXian 比例字体,不是等宽;若 utility tabs 要等宽,需新增 tab mono token。完整 runtime FontDatabase/FontFace DTO、GPU draw-list family/style、baseline/DPI/native-SDF parity 与窗口级文本评审仍 open。 |
| 2026-07-01 | 17.S2 retained-host DengXian priority restoration + integer button text slot | implemented-focused-passed-build-screenshot-metadata-recorded | 针对用户最新 crop 指出 tab labels 仍不是等线观感,复核后确认根因不是 raster,而是上一片拉丁 UI 一致性切片把普通 UI/strong UI 候选顺序改成 Segoe UI 优先,测试也断言 Segoe。按当前要求恢复等线优先:`paint_text/font.rs` 的普通 UI 顺序为 `Deng.ttf -> Dengl.ttf -> segoeui.ttf`,strong UI 顺序为 `Dengb.ttf -> Deng.ttf -> segoeuib.ttf`,code style 仍优先 Cascadia/Consolas 等宽。`paint_text_tests.rs` 改为 Windows 有 Deng 字体时断言 `runtime_font_family_for_face(Ui/UiStrong) == DengXian`。同时 `template_buttons/content/metrics.rs` 将按钮/utility tab label line-height snap 到整数像素,12px tab label 使用 14px 文本槽而不是 14.4px。验证:`cargo fmt -p zircon_editor --check`;direct `retained_ui_runtime_family_prefers_dengxian_when_available_on_windows` 1/1;direct `paint_text` 20/20;direct `button_label` 4/4;direct dock tab 度量 1/1;component atlas screenshot 1/1;M3 screenshot harness 1/1;editor-host build passed in `D:\cargo-targets\zircon-editor-text-tabs-0701`。截图证据只记录文件实际输出与修改日期,不直接判定最终文本质量:Asset Browser `2026-07-01 20:47:01 +08:00`,62627 bytes,SHA256 `DFA19780B09D6C2F323EC4B80407BF992B0202EAC033849892E942D1EAC84FDE`;utility-tabs DengXian crop `2026-07-01 20:48:35 +08:00`,2823 bytes,SHA256 `C0EA3A82F121551601FCB71D3AF538C55A9C3A5A22937AF5BB637ED53745D261`;3x crop SHA256 `279D6E3BBF66581CDD2C7417DD375E724137932BD7C4A9AAF605E7027C38266F`;component atlas `2026-07-01 20:46:24 +08:00`,74980 bytes,SHA256 `AA77420265441DFBA75E65F82D4BAE682160F1DC652E18DCE8368FF398CDEC89`;target scan `NO_MATCHING_SCREENSHOTS_IN_TARGETS`。 | 本切片 supersede 之前的 Segoe-priority experiment,当前计划语义恢复为普通 UI=DengXian/等线比例字体、code/数值=Cascadia/Consolas 等宽。若产品要求 utility tabs 本身也等宽,需新增独立 tab mono token;完整 runtime FontDatabase/FontFace DTO、GPU draw-list family/style、baseline/DPI/native-SDF parity 与窗口级文本评审仍 open。 |
| 2026-07-01 | 17.S2 retained-host native-scale swash text and real-strong cleanup | implemented-focused-passed-build-screenshot-metadata-recorded | 针对用户再次指出 utility-tab 文本仍不像等线/等宽观感的问题,按根因链复核 retained-host CPU text path:普通 UI 文本仍保持 DengXian/等线比例字体,code style 才保持 Cascadia/Consolas 等宽。问题集中在 swash `Format::Subpixel` glyph 被 3x 光栅后再压回 1x,以及 strong text 在已经选 `Dengb.ttf` 后又横向重复绘制一遍。`paint_text/raster.rs` 现在让 swash 子像素 glyph 以 native 1x 逻辑字号缓存/绘制,只有 swash 放置失败或 native 覆盖率过低的细线 glyph 才回退到 3x fontdue alpha path;cache key 改为 font face + glyph + logical px + fallback scale。`draw/glyphs.rs` 改读每个 raster 自带 scale;`draw/glyphs/row.rs` 取消 strong 伪加粗 extra pass,让 selected tab/button 只依赖真实 strong UI face。验证:`cargo fmt -p zircon_editor --check`;`retained_text_preserves_small_underscore_stroke_contrast` 修复后 1/1;`paint_text` 19/19;`asset_browser_utility_tabs_use_compact_slate_tab_strip_geometry` 1/1;ignored M3 screenshot harness 1/1;`cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never` 通过(既有 warnings)。截图证据只记录实际文件与修改日期,不直接判定最终文本质量:Asset Browser `2026-07-01 19:18:01 +08:00`,SHA256 `3906027A2EBED13D1257BC528DED673994A772200E4D8BB9623AF62F69305AE8`;native-subpixel crop `2026-07-01 19:23:03 +08:00`,SHA256 `49CCF1310FED9878BA32C15C63774116EB1BC717CC3506C5C8D0FFF5711DE9C7`;4x zoom `2026-07-01 19:23:19 +08:00`,SHA256 `48242ECE6D04E1671B835572E6505AA4044FFA92682EEF28173A59E1DC4D09E7`;target scan `NO_MATCHING_SCREENSHOTS_IN_TARGETS`。 | 本切片关闭 retained-host swash 子像素路径被二次下采样和 real-strong 叠加伪加粗的问题。仍 pending:如果产品要求 utility tabs 使用等宽字形,需单独定义 UI tab mono token,因为当前计划语义是 UI=DengXian 比例字体、code=Cascadia/Consolas 等宽;完整 runtime FontDatabase/FontFace DTO、GPU draw-list family/style propagation、DPI scale/subpixel bins、native/SDF paragraph parity、baseline/ascent/descent single source、vertical layout 与窗口级文本视觉评审仍 open。 |
| 2026-07-01 | 17.S2 retained-host button font-weight strong UI face consumer | implemented-focused-passed-build-screenshot-metadata-recorded | 用户最新 crop 指出 utility-tab 文本仍未达到等线/等宽预期观感后,复核链路确认 button label path 仍把所有 `HostPaintCommand::text(...)` 强制写成默认 `UiTextRunPaintStyle`。本切片把 `TemplatePaneNodeData.font_weight>=600` 映射为 strong text run,并在 retained-host 字体路由中新增 UI strong face:Windows 优先 `Dengb.ttf`,普通 UI 仍 `Deng.ttf`/`Dengl.ttf`,code style 仍优先 mono。`template_buttons/content/metrics.rs` 同时用该 style 做 runtime text measurement,避免 strong label 使用普通宽度。验证:`cargo fmt -p zircon_editor --check`;focused `button_label_uses_strong_text_style_when_node_font_weight_is_strong` 1/1;focused `retained_text_font_face_tracks_ui_and_code_styles` 1/1;focused `asset_browser_utility_tabs_use_compact_slate_tab_strip_geometry` 1/1;ignored M3 screenshot harness direct binary 1/1;editor-host build passed in `D:\cargo-targets\zircon-editor-text-tabs-0701` with existing warnings。截图证据只记录文件实际输出与修改日期,不直接判定最终文本质量:Asset Browser `2026-07-01 18:38:01 +08:00`,SHA256 `C8875BE085BD5AE359C0FA2DB701CA8D0355521AC8425138CD9895E07F4EF6CC`;strong-font crop `2026-07-01 18:39:19 +08:00`,SHA256 `C666E61590C39CF0000D682A1F94F69EA331DB5633D290BB5A0278A13870FA55`;4x zoom `2026-07-01 18:39:19 +08:00`,SHA256 `2FD7828F6CA41997E49D8D782004F9398EB1F5321A9E8371C02BFB0D54314ACF`;target scan 0。 | 本切片关闭按钮/Tab label 字重未进入 text paint style 的缺口。仍 pending:完整 runtime FontDatabase/FontFace DTO、GPU draw-list family/style propagation、baseline/ascent/descent single source、DPI scale/subpixel bins、native/SDF paragraph parity、vertical layout 与窗口级文本视觉评审。 |
| 2026-07-01 | 17.S2 retained-host swash/subpixel utility-tab raster bridge | implemented-focused-passed-build-screenshot-metadata-recorded | 针对用户新 crop 中 `Preview / References / Metadata / Plugins` 小字号仍像 fallback bitmap 的问题,继续从文本原子而不是 tab 容器入手。`paint_text/font.rs` 现在保留 fontdue 字体和原始 font bytes,供 swash shaping/raster 复用;`paint_text/raster.rs` 改为 swash-first `Format::Subpixel` 栅格化,cache key 保持 font face/glyph/physical px,失败时才 fontdue fallback;`draw/glyphs/row.rs` 增加 RGB 子像素 coverage 采样,`blend.rs` 增加 channel coverage blending。focused tests 锁定 UI face 保持比例宽度而 code face 保持等宽,并锁定 UI glyph 使用 swash/subpixel path。验证:`cargo fmt -p zircon_editor -p zircon_runtime --check` 通过;`cargo test -p zircon_editor --lib paint_text --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never -- --nocapture --test-threads=1` 18/18 通过;`cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never -- --ignored --nocapture --test-threads=1` 1/1 通过;`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never` 通过(既有 warnings);`cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never` 通过(既有 warnings)。首次 editor build 曾被 `dynamic_api/shader_prewarm.rs` 缺失 `shader_prewarm_source_hash(...)` 阻断,已补为同文件内 blake3 source hash helper 后重跑通过。截图证据只记录实际文件与修改日期,不直接判定最终文本质量:Asset Browser `docs/tests/editor/editor-window-m3-asset-browser-900x620.png` 修改时间 `2026-07-01 16:00:02 +08:00`,61509 bytes,SHA256 `AE5C48546AD2485DBCBE3B3C8FBD61AD27781E44EBC7946DAA5A8AC2EDBF9E06`;utility tabs crop `docs/tests/editor/editor-window-m3-asset-browser-utility-tabs-font-crop-20260701.png` 修改时间 `2026-07-01 16:01:01 +08:00`,2369 bytes,SHA256 `C0F8E072E56B721064D977985784274641D52D3CB8EA9631683071985A5CEE7D`;4x zoom crop `docs/tests/editor/editor-window-m3-asset-browser-utility-tabs-font-zoom-20260701.png` 修改时间 `2026-07-01 16:01:01 +08:00`,5053 bytes,SHA256 `3875F37273020AD260B1B40710878F2272173AC76AFAB60FBF01AAAB64BFD866`;repo `target` 与 `D:\cargo-targets\zircon-editor-text-tabs-0701` 同名截图扫描均无匹配。 | 本切片关闭 retained-host CPU text raster 仍停留在 fontdue grayscale bitmap 的局部问题,但不是完整 runtime text stack 完成。仍 pending:`GlyphAtlasSet`/GPU atlas 真实接入、DPI scale key/subpixel bins、runtime FontDatabase/FontFace DTO、GPU `UiSurfaceDrawList` font-family/style propagation、native/SDF paragraph parity、baseline/ascent/descent single source、vertical layout 与窗口级文本视觉评审。focused `asset_browser_utility_tabs_use_compact_slate_tab_strip_geometry` 曾在 604s 超时,不计通过。 |
| 2026-07-01 | 17.S2 retained-host utility-tab coverage downsampling review | implemented-focused-passed-screenshot-metadata-recorded | 用户最新 crop 指出 Asset Browser utility tabs 文本仍不是期望的等线观感,复核后确认这些 label 已走普通 UI text/DengXian runtime family,问题集中在 retained-host CPU glyph row 覆盖率合成。`paint_text/draw/glyphs/row.rs` 从 RMS downsampling 改成普通像素使用 arithmetic average,只在 high-coverage isolated stroke 时保留 128 coverage floor,避免小字号边缘被抬成粗块。验证:`cargo fmt -p zircon_editor --check` 通过;`cargo test -p zircon_editor --lib sampled_coverage --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never -- --nocapture --test-threads=1` 3/3 通过;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 通过。截图证据只记录文件实际输出与修改日期,不直接判定文本质量有效:Asset Browser `2026-07-01 14:04:47 +08:00`,SHA256 `7B93021A6A8F201E184C495267160B65FCCB2FE2F650A63A34D094E3B7480FAD`;utility tabs crop `2026-07-01 14:06:32 +08:00`,SHA256 `D642C0E5CD58FC2259C0CB333A42542604ABF909CCC42548DAEC2FF2B842989A`;3x zoom crop `2026-07-01 14:06:32 +08:00`,SHA256 `04E7CBBA4720D2AAC038B3D8AF8CA8B585617A22CA623476EA4B4B60AFC30A63`;repo `target` 与本次外部 cargo target scan 均无同名截图。 | 本切片只收敛 retained-host CPU grayscale raster 的边缘厚重问题,遵守 leaf owner 结构而未新增 root painter 分支。仍 pending:ClearType/subpixel native quality、runtime FontDatabase/FontFace DTO、GPU `UiSurfaceDrawList` font-family/style propagation、shaped glyph paint DTO、DPI reraster/subpixel bins、native/SDF paragraph parity、baseline/ascent/descent single source、vertical layout 与窗口级文本视觉评审。 |
| 2026-07-01 | 17.S2 retained-host runtime glyph-advance draw projection | implemented-focused-passed-build-screenshot-metadata-recorded | 用户截图复核暴露 retained-host 已用 runtime `layout_text(...)` 做 display text/family projection 后,实际字形绘制仍可能使用 fontdue 原始 x advance。`paint_text/draw/layout.rs` 现在把 runtime `glyph_advances` 按 grapheme 投影到 host `RuntimeTextGlyph` 绘制坐标,保留 fontdue glyph index/bounds 仅用于 raster;`draw/glyphs.rs` 直接消费这些 runtime-projected glyph positions。验证:`cargo fmt -p zircon_editor` 与 `cargo fmt --check -p zircon_editor` 通过;`cargo check -p zircon_editor --no-default-features --locked --target-dir D:\cargo-targets\zircon-editor-text-spacing-0701-check --message-format short --color never` 通过(仅既有 warnings);`runtime_positioned_glyphs_use_runtime_grapheme_advances` 1/1 通过;`retained_text_run_carries_runtime_projected_spacing` 1/1 通过;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 通过并刷新 `docs/tests/editor`。截图证据只记录文件实际输出与修改日期,不直接判定文本质量有效:Asset Browser `2026-07-01 12:34:01 +08:00`,SHA256 `40B3C1C5BD7D9E1CEE5DE282CBEF649D6AA21A1CCA5CC091D5BF6B71C1EE28F3`;utility tabs crop `2026-07-01 12:35:19 +08:00`,SHA256 `AE63ADADCB27E34B4E9BDFCAC88FF0B22AFB2768BA924170A38744C34876A5F9`;3x zoom crop `2026-07-01 12:35:19 +08:00`,SHA256 `8367763422CA9F8D4C4F4978F3C59ADC2C5B12B37781810942EC547001111C7F`;target scan `NO_MATCHING_SCREENSHOTS_IN_TARGETS`。 | 本切片关闭 retained-host draw x-position 仍走旧 fontdue advance 的问题。仍 pending:runtime FontDatabase/FontFace DTO、GPU `UiSurfaceDrawList` font-family/style propagation、shaped glyph paint DTO、DPI reraster/subpixel bins、native/SDF paragraph parity、baseline/ascent/descent single source、ClearType-like quality、vertical layout 与窗口级文本视觉评审。 |
| 2026-07-01 | 17.S2 retained-host runtime font-family measurement/projection | implemented-focused-passed-build-screenshot-metadata-recorded | 用户截图暴露 retained-host 已切到 DengXian/Mono raster face 后,measurement/projection 仍可能使用 runtime 默认字体族。`paint_text/font.rs` 现在暴露 `runtime_font_family_for_face(...)` 与 `measure_runtime_text_width_with_style(...)`;普通 UI measurement 显式设置 DengXian/等线 runtime family,code style 显式设置 Cascadia/Consolas 等宽 family。`paint_text/draw/layout.rs` 在调用 runtime `layout_text(...)` 计算单行 ellipsis/display_text 时也传同一 family,避免 displayed text 与 host raster face 用不同字体度量。focused tests 覆盖 UI 与 code family 分流,并断言 helper 与 `zircon_runtime::ui::surface::measure_text_size(...)` 一致。截图证据:`editor-window-m3-asset-browser-900x620.png` 修改时间 `2026-07-01 11:37:11 +08:00`,SHA256 `9A47921836E870BD4414AD40C02534D2E7086355BA78A3129A1238A1A8AFF7F3`;utility tabs crop `2026-07-01 11:43:39 +08:00`,SHA256 `72B23815D657450782CA45EF052D6CE974193FDDDBCB9BDB544394ABFCF4E253`;3x zoom crop SHA256 `F8F0344D354864E8A84E74E490B1E2C1BC32C191ECAA1F0290A0D134FB517F3`;component atlas `2026-07-01 11:43:12 +08:00`,SHA256 `12D98FCCF5718E2913670AEA3F07A6CE70C540F2C4A9DDB45325D353F507900F`;target scan `NO_MATCHING_SCREENSHOTS_IN_TARGETS`。 | 本切片关闭 retained-host runtime family mismatch for measurement/projection only。GPU `UiSurfaceDrawList` font-family/style propagation、runtime FontDatabase/FontFace DTO、DPI reraster、native/SDF paragraph parity、ClearType-like quality、baseline/ascent/descent single source、vertical layout 与窗口级文本视觉评审仍 pending。 |
| 2026-07-01 | 17.S2 retained-host UI/monospace font-face raster routing | implemented-focused-passed-build-screenshot-metadata-recorded | `zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs` 新增 `HostTextFontFace` 与 style-to-face 路由:普通 UI text 优先 Windows DengXian/等线(`Deng.ttf`/`Dengl.ttf`),code text 优先 Cascadia Mono/Code/Consolas;`draw/layout.rs` 把 font face 带入 fontdue layout,`draw/glyphs.rs` 将 face 传给 raster,`raster.rs` cache key 从 glyph index + pixel size 扩展为 font face + glyph index + pixel size。focused tests 锁定 UI 与 code style 的 font face 分流,以及 UI/Mono cache entry 不混用。截图证据只记录 `docs/tests/editor` 文件已刷新和修改时间:component atlas `2026-07-01 10:08:56`,M3 workbench `2026-07-01 10:09:36`,M3 asset-browser list `2026-07-01 10:10:00`;这些截图供人工检查,不直接判定文本渲染质量有效。 | 本切片只关闭 retained-host fallback font 固定路径与 glyph cache font identity 漏项。完整 `UiTextPaint`/shaped glyph DTO、runtime FontDatabase/FontFace contract、DPI 重栅格、baseline/ascent/descent 单源、native/SDF paragraph parity、vertical layout 与窗口级文本视觉评审仍 pending。 |
| 2026-07-01 | 17.S2 retained-host render-command alignment surface measurement consumer | implemented-focused-passed-build-screenshot-timestamp-recorded | `render_command_conversion/style/text.rs` 的 text origin 对齐宽度现在消费 `zircon_runtime::ui::surface::measure_text_size(...)`,覆盖 Center/Right 与逻辑 Start/End;旧 `chars().count() * font_size * 0.5` editor-local half-em 估算已从该 command bridge 移除。`Justify` 在 retained render-command conversion 中保持 line-start,等待 runtime paragraph layout/paint DTO 承担真正的段落级 justification。focused tests 覆盖 combining-mark 文本宽度与 logical alignment。截图证据只记录 `docs/tests/editor` 文件已刷新和修改时间:component atlas `2026-07-01 04:28:35`,M3 workbench `2026-07-01 04:31:01`,M3 asset-browser list `2026-07-01 04:31:18`;这些截图供人工检查,不直接判定文本渲染质量有效。 | 仍需继续 17.S2 完整 `UiTextPaint`/shaped glyph DTO、SDF/native paragraph parity、subrange kerning semantics、DPI scale atlas/baseline、默认多行换行/省略分离、auto-wrap 两阶段、clamp 和文本视觉人工评审。 |
| 2026-06-26 | 17.S0a retained-host 预览文本 2x 光栅/下采样首段 | implemented-focused-passed-build-screenshot-passed | 为回应截图中文本锯齿问题,先在 retained-host 预览 painter 做低风险局部修正:`paint_text/draw/glyphs.rs` 以 2x raster font size 生成 glyph bitmap,再按逻辑 extent 下采样;`glyphs/row.rs` 对 supersampled coverage 做平均采样并钳制边界。验证:`cargo test -p zircon_editor --lib paint_text --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never -- --test-threads=1 --nocapture` 10/10;`cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel` 通过;`capture_full_workbench_run_mode_visual_artifact --ignored` 1/1 刷新 `docs/tests/editor/editor-window-m3-workbench-run-mode-1672x941.png`,未写 repo `target`。 | 本片只改善 retained-host fallback preview 文本,不关闭 17.S2-S6:运行时 glyph metrics provider、真实测量=绘制、DPI scale atlas key、默认换行和文本自适应仍需后续实现。 |
| 2026-06-26 | 17.S1 文本渲染与排版规范立项 | planned | 代码核实两根因:G1 测量启发式(`layout_engine.rs:21-31,82,486-492` 用 `font_size*0.5` 等宽 + `font_size*0.8` baseline,与绘制端 glyphon/fontsdf 真实度量不一致)、G2 固定字号光栅化无 DPI(`sdf_font_bake.rs:112,122,243`,`SdfAtlasGlyphKey` 无 scale)。给出测量↔绘制单源(provider)、DPI 重栅格(atlas key 加 scale,接 16)、默认多行换行(word+逐字回退,接真实宽)、省略分离、文本自适应(两阶段 + 可选 clamp)、baseline 单源;6 切片 + 9 测试矩阵;带 UE `FShapedGlyphEntry`/`ETextWrappingPolicy`/`STextBlock`/`EFontHinting`、godot `AutowrapMode`/`OverrunBehavior`、bevy `FontSmoothing` 源码证据。 | 按 §6 推进 S2–S6;`13`/`16`/`15a` 已同步对齐本文。 |
