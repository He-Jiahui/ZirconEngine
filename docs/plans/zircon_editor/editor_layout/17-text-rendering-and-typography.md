---
related_code:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs
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
  - docs/plans/engine-code-structure-convention.md
status: in_progress
---
# 17 文本渲染与排版规范(度量一致性 / DPI 重栅格 / 换行 / 自适应)

> 本文是文本这条线的权威规范。与 `16`(布局相对/DPI 根缩放)分工:`16` 管"区域/控件几何在不同分辨率下怎么摆",`17` 管"文本字形如何被准确测量与清晰绘制"。**文本字形随 `scale_factor` 重栅格**是 `16` 三层模型第①层(根 DPI 缩放)在文本上的落地。取 dev 引擎(UE Slate / godot / bevy / slint / material-ui)的文本思想,落到既有 **glyphon + fontsdf 双后端 + `ui/text` 布局引擎**,不重写字体库(与 `14` 同调:取思想不取运行时)。

## 1. 目标

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
- hinting 策略:高 DPI 用 light/none(像素网格对齐收益小),低 DPI 可用 auto。
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

   决策树(节点声明优先级):`wrap(默认) → ellipsis → shrink/clamp`。UE `LineHeightPercentage`(`STextBlock.h:121`)用乘数缩行距、不改字号,Zircon 同(行距走 `line_height_ratio`)。

### 3.6 baseline / 行高 / 对齐一致

- baseline = 真实 ascent(measure 与 SDF 同公式),替换 `font_size*0.8`。
- 多行行距统一走 `DEFAULT_LINE_HEIGHT_SCALE`(=1.2)/ `METRICS.line_height_ratio`(`15b`)。
- baseline/ascent/descent 纳入度量来源单源(provider 提供;字号/行距 token 归 `01`/`15b`,字形度量随字体)。
- 垂直对齐(top/center/baseline)单源;`MinDesiredWidth`(UE `STextBlock.h:130`)对应"文本节点最小期望宽"防塌缩。

## 4. 接口与数据结构草案(Rust,规范形态非实现)

```rust
/// 测量与绘制共用的字形度量来源(桥接 fontsdf/glyphon,替换 font_size*0.5 近似)
pub trait GlyphMetricsProvider {
    fn advance(&self, ch: char, font_px: f32, scale: f32) -> f32; // 真实 advance(物理感知)
    fn kerning(&self, a: char, b: char, font_px: f32) -> f32;
    fn ascent(&self, font_px: f32) -> f32;
    fn descent(&self, font_px: f32) -> f32;
}
/// 文本布局输入:换行/溢出/自适应/缩放
pub struct TextLayoutInput<'a> {
    pub metrics: &'a dyn GlyphMetricsProvider,
    pub wrap: UiTextWrap,                 // 默认 Word
    pub overflow: UiTextOverflow,         // clip / ellipsis
    pub auto_size: Option<FitPolicy>,     // None=仅换行
    pub scale_factor: f32,                // 接 16 §3.4
}
pub enum FitPolicy { ShrinkToFit { min_px: f32 }, ClampFontSize { min_px: f32, max_px: f32 } }
/// SDF atlas key 加入 scale(治 G2)
pub struct SdfAtlasGlyphKey { /* ...; */ pub scale_milli: u32 }
```

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
| `word_wrap_uses_real_advance` | 换行点由真实字形宽决定,非 `max_chars` |
| `long_word_falls_back_to_char` | 超行宽单词逐字回退(对标 UE AllowPerCharacterWrapping) |
| `ellipsis_width_is_real_glyph` | `…` 占位用真实字形宽 |
| `shrink_to_fit_clamps_within_min_max` | clamp 字号落 `[min,max]` |
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
| 2026-06-26 | 17.S0a retained-host 预览文本 2x 光栅/下采样首段 | implemented-focused-passed-build-screenshot-passed | 为回应截图中文本锯齿问题,先在 retained-host 预览 painter 做低风险局部修正:`paint_text/draw/glyphs.rs` 以 2x raster font size 生成 glyph bitmap,再按逻辑 extent 下采样;`glyphs/row.rs` 对 supersampled coverage 做平均采样并钳制边界。验证:`cargo test -p zircon_editor --lib paint_text --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel --message-format short --color never -- --test-threads=1 --nocapture` 10/10;`cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626-panel` 通过;`capture_full_workbench_run_mode_visual_artifact --ignored` 1/1 刷新 `docs/tests/editor/editor-window-m3-workbench-run-mode-1672x941.png`,未写 repo `target`。 | 本片只改善 retained-host fallback preview 文本,不关闭 17.S2-S6:运行时 glyph metrics provider、真实测量=绘制、DPI scale atlas key、默认换行和文本自适应仍需后续实现。 |
| 2026-06-26 | 17.S1 文本渲染与排版规范立项 | planned | 代码核实两根因:G1 测量启发式(`layout_engine.rs:21-31,82,486-492` 用 `font_size*0.5` 等宽 + `font_size*0.8` baseline,与绘制端 glyphon/fontsdf 真实度量不一致)、G2 固定字号光栅化无 DPI(`sdf_font_bake.rs:112,122,243`,`SdfAtlasGlyphKey` 无 scale)。给出测量↔绘制单源(provider)、DPI 重栅格(atlas key 加 scale,接 16)、默认多行换行(word+逐字回退,接真实宽)、省略分离、文本自适应(两阶段 + 可选 clamp)、baseline 单源;6 切片 + 9 测试矩阵;带 UE `FShapedGlyphEntry`/`ETextWrappingPolicy`/`STextBlock`/`EFontHinting`、godot `AutowrapMode`/`OverrunBehavior`、bevy `FontSmoothing` 源码证据。 | 按 §6 推进 S2–S6;`13`/`16`/`15a` 已同步对齐本文。 |
