---
related_code:
  - zircon_runtime/src/text/model/rich.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/inline_widget.rs
  - zircon_runtime/src/ui/layout/pass/inline_widgets.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/RichTextLayoutMarshaller.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/RichTextMarkupProcessing.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/IRichTextMarkupParser.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ITextDecorator.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextDecorators.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateImageRun.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateWidgetRun.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateHyperlinkRun.h
  - dev/godot/scene/gui/rich_text_label.h
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
status: in_progress
---

# 07 富文本(BBCode + HTML 子集)

> 本计划把"带标记的字符串"解析为"样式 run 序列 + 内联对象",喂 `02/03`。承接 `editor_ui/03` 文本主链。三种 inline marker 明确命名为 `MarkdownInlineV1`，不再冒充完整 Markdown；其余版本化语法为 `BbCodeV1` 与 `HtmlSubsetV1`。

## 1. 目标

1. **BBCode**(对齐 godot `RichTextLabel`):`[b][i][u][s][color][bgcolor][size][font][url][img][center][right][code][table]...`;可扩展自定义标签。(2026-07-02 评审收口)`[table]`(含 `[cell]`)明确**降级 V2**:表格布局需要块级嵌套布局能力,超出 V1 内联 run 模型;V1 解析器遇到 `[table]` 按未知标签处理(标签丢弃、内文保留)。
2. **HTML 受控子集**:`<b><i><u><s><span style><font><br><a href><img>`;**白名单标签 + 属性**(安全,不引入完整 HTML/CSS 解析器)。
3. **装饰器架构**(对齐 UE `ITextDecorator`/`FRichTextLayoutMarshaller`):标签 → 装饰器 → run 样式覆盖 / 内联 widget;可注册自定义装饰器。
4. **样式 run 合并**:嵌套标签解析为扁平 `StyledRun` 序列(每 run 一组样式),交 `02` 按 run 整形;run 边界尊重 cluster。
5. **内联对象**:图标/表情/超链接/内联 widget 作为占位 metric 参与 `03` 布局(对齐 UE `SlateImageRun`/`SlateWidgetRun`)。

## 2. 现状与差距

- `ui/text/rich_text.rs`(89 行):仅 markdown `**`/`*`/`` ` `` 三标记 → 简单 run;无颜色/字号/字体/链接/图片/对齐、无 BBCode、无 HTML、无装饰器、无内联对象。
- 渲染 DTO `UiTextPaintRun`/`UiTextRunPaintStyle`/`UiTextPaintDecoration` 在,但富文本上游不产这些。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Framework/Text/RichTextLayoutMarshaller.h` | `FRichTextLayoutMarshaller`:marshaller 把标记串 → `FTextLayout` 的 run;`AppendRunsForText`、装饰器注册表。**本计划富文本→run 主架构** |
| `dev/UnrealEngine/.../Framework/Text/RichTextMarkupProcessing.h` + `IRichTextMarkupParser.h` | `FDefaultRichTextMarkupParser`:标记 tokenize(`<Tag attr="v">text</>`)→ `FTextLineParseResults`/`FTextRunParseResults`——**解析器样板**(UE 用尖括号语法,本计划 BBCode 用方括号、HTML 用尖括号,tokenizer 同构) |
| `dev/UnrealEngine/.../Framework/Text/ITextDecorator.h` + `TextDecorators.h` | `ITextDecorator::Supports`/`Create*Run`;`FHyperlinkDecorator`/`FImageDecorator`/`FWidgetDecorator`——**装饰器接口与内置装饰器** |
| `dev/UnrealEngine/.../Framework/Text/{SlateImageRun,SlateHyperlinkRun,SlateWidgetRun}.h` | 内联图片/超链接/widget run 的 metric 参与布局——**内联对象样板** |
| `dev/godot/scene/gui/rich_text_label.h` | `RichTextLabel` BBCode 全集:`push_*`/`pop` 栈式 item 树、`[table]`/`[cell]`/`[img]`/`[url]`/`[font]`——**BBCode 标签集与语义权威** |

**Rust/wgpu 落地**:无现成富文本 crate 直接对口(html5ever 过重);自研 tokenizer + 装饰器表(轻量、受控)。`02` 已能按 run 整形,本计划只需产 `StyledRun` 序列。

## 4. 目标架构

```
markup string (BBCode | HTML | Plain) →
  parser(tokenize tags + text)→ tag 栈 → DecoratorRegistry 解析 →
    Vec<StyledRun{ byte_range, style_overrides, inline: Option<InlineObject> }>
      → 02 per-run shape(style 覆盖 base style) → 03 layout(内联对象占位 metric)
        → ShapedGlyphRun + inline placement
```

格式由 `RichTextFormat::{Plain, MarkdownInlineV1, BbCodeV1, HtmlSubsetV1}` 选择；版本是 artifact/cache 身份的一部分。解析器产**中立** `StyledRun`,后续链路与纯文本统一。

## 5. 里程碑

### RT-M1 解析器框架 + BBCode 核心标签

实施切片:
1. `text/rich/parser.rs`:tokenizer(BBCode `[tag=val]...[/tag]` + HTML `<tag attr>`)+ 标签栈 → `StyledRun`。
2. `text/rich/decorator.rs`:`DecoratorRegistry` + 内置文本样式装饰器(b/i/u/s/color/bgcolor/size/font);样式 run 合并(嵌套→扁平)。
3. `ui/text/rich_text.rs` 三 marker 解析迁入统一框架并命名为 `RichTextFormat::MarkdownInlineV1`。

测试:`text_rich_bbcode_nested_styles_flatten_to_runs`、`text_rich_color_size_font_overrides`、`text_rich_run_boundaries_respect_clusters`。

### RT-M2 HTML 受控子集 + 安全白名单

实施切片:
1. HTML tokenizer + 白名单标签/属性(`b/i/u/s/span[style]/font/br/a[href]/img[src]`);`style` 仅解析受控属性(color/font-size/font-weight/font-style/text-decoration);未知标签丢弃且发布 bounded non-fatal authoring diagnostic，不执行。属性/值细分诊断仍按后续切片完成。
2. 实体解码(`&amp;`/`&#xNN;`);`<br>` → 强制换行(`02` mandatory break)。

测试:`text_rich_html_whitelist_drops_unknown_tags`、`text_rich_html_entities_decode`、`text_rich_html_br_forces_break`。

### RT-M3 内联对象(图标 / 超链接 / 表情)

实施切片:
1. `InlineObject`(图标/图片/widget 占位):metric(尺寸 + baseline 对齐)参与 `03` 布局;`[img]`/`<img>`/emoji shortcode。
2. 超链接 run(`[url]`/`<a>`):携 href + 命中区间(供交互层),样式默认下划线 + 链接色。

测试:`text_rich_inline_image_reserves_metric_in_layout`、`text_rich_hyperlink_carries_href_and_hit_range`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/text/rich/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | `parse_rich_text(markup, format, base_style) -> RichParseResult`(薄) |
| `parser.rs` | tokenizer(BBCode/HTML/Markdown 三 lexer)+ 标签栈 → `StyledRun` |
| `decorator.rs` | `DecoratorRegistry`、`TextDecorator` trait、内置样式装饰器 |
| `html_subset.rs` | HTML 白名单标签/属性表 + 实体解码 + `style` 受控属性解析 |
| `bbcode.rs` | BBCode 标签集(对照 godot)+ 自定义标签注册 |
| `inline_decorators.rs` | 内置 icon/widget decorator 与显式 metric/baseline 解析 |
| `resource_admission.rs` | image/icon 共享的受控 engine resource locator 准入 |

契约层 `text/model/rich.rs`:`RichTextFormat`、`StyledRun`、`StyleOverride`、`InlineObjectRef`(serde)。

### 核心类型

```rust
pub enum RichTextFormat { Plain, MarkdownInlineV1, BbCodeV1, HtmlSubsetV1 }
pub struct StyledRun {
    pub byte_range: (u32, u32),       // 源(已剥标记)文本字节区间
    pub style: StyleOverride,         // 覆盖 base TextStyle 的增量
    pub inline: Option<InlineObjectRef>,
    pub link: Option<LinkRef>,        // typed target + shared tooltip
}
pub struct StyleOverride {            // None 字段表"继承"
    // (2026-07-02 评审收口)bold: Option<bool> 改 weight: Option<u16>——对齐已落地的 font_weight 1..1000 契约;
    // [b]/<b> 标签映射 weight=Some(700)
    pub weight: Option<u16>, pub italic: Option<bool>, pub underline: Option<bool>,
    pub strike: Option<bool>, pub color: Option<Vec4>, pub bg_color: Option<Vec4>,
    pub font_size: Option<f32>, pub family: Option<FontFamilyName>,
    // (2026-07-02 评审收口)新增可选字段:
    pub letter_spacing: Option<f32>,          // 字距(逻辑像素)
    pub features: Option<Vec<FontFeature>>,   // OpenType feature 覆盖(进 02 整形键)
}
// (2026-07-02 评审收口)块级(段落级)标签落地模型:[center]/[right] 等不是 run 样式,
// 是段落属性覆盖,单独出通道交 03 的 LayoutConstraints:
pub struct ParagraphOverride {
    pub align: Option<TextAlign>,     // [center]→Center,[right]→Right
    pub indent: Option<f32>,          // 首行缩进覆盖
    // 后续块级属性(line_height 覆盖等)在此扩展
}
pub enum InlineObjectRef {
    Image {
        texture: ResourceId,
        size: Vec2,
        baseline: InlineBaseline,
        alternative_text: Option<String>,
        tooltip: Option<String>,
    },
    Icon  {
        asset: RichIconAssetId,
        size: Vec2,
        baseline: InlineBaseline,
        alternative_text: Option<String>,
    },
    Widget { id: u64, size: Vec2 },   // 内联 widget 占位(UE SlateWidgetRun)
}
// (2026-07-02 评审收口)内联对象基线对齐模式:
pub enum InlineBaseline {
    Baseline, // 对象底边坐 alphabetic baseline(默认,图标/表情)
    Center,   // 对象垂直中心对齐行 x-height 中心
    Top,      // 对象顶边对齐行 ascent
    Bottom,   // 对象底边对齐行 descent
}
pub trait RichTextDecorator {
    fn tag(&self) -> &str;
    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool;
}
pub struct RichParseResult {
    pub text: Arc<str>,               // 剥标记、可共享的可见文本
    pub runs: Vec<StyledRun>,
    // (2026-07-02 评审收口)段落级覆盖:byte_range 为剥标记文本内的段落区间
    pub paragraphs: Vec<((u32, u32), ParagraphOverride)>,
    pub tables: Vec<RichTable>,
    pub authoring_diagnostics: Vec<RichTextAuthoringDiagnostic>,
}
```

解析 → run 合并:嵌套标签维护样式栈,每遇文本片段产一个 `StyledRun`(当前栈样式合并);相邻同样式 run 合并;run 边界后续由 `02` 对齐 cluster(标记不可切簇)。

(2026-07-02 评审收口)**簇内样式边界裁决**:标记边界落在组合簇(grapheme cluster)内部时,样式边界**向簇起点对齐**——整簇取**簇首字符所在 run 的样式**,后续字符的样式覆盖被吸收丢弃(不拆簇、不产生半簇 run);`text_rich_run_boundaries_respect_clusters` 的期望按此标定(如 `a[b]\u{0301}[/b]` → `á` 整簇非 bold)。

(2026-07-02 评审收口)**内联对象行度量规则**:`InlineObject` 按其 `InlineBaseline` 模式换算出等效 ascent/descent(如 `Baseline` 模式下 ascent=对象高、descent=0),该 ascent **参与 03 行 ascent 的 max 计算**(与混 face 行度量同一 max 规则,见 03 §6"混 face 行度量"/D7);对应布局槽位为 03 `LayoutItem::Inline`(03 已预留,本计划 RT-M3 落地时回填其解析来源)。

### 安全(HTML 子集)

- **白名单**:仅列表内标签/属性进解析,其余标签丢弃、文本保留，并在独立 request budget 内发布 source-ranged warning；不得执行未知内容。
- `style` 属性只取 color/font-size/font-weight/font-style/text-decoration;不解析任意 CSS、不支持 `url()`/脚本/事件属性。
- `img src`/`a href` 仅接受 `res://`/相对资源路径或受控 scheme;不发起网络请求(资源经资产系统)。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| `ui/text/rich_text.rs` markdown 三标记 | 迁 `text/rich/` 并硬切为 `RichTextFormat::MarkdownInlineV1`;调用方改 `parse_rich_text` |
| 调用方直接传 raw 文本 | 经 `parse_rich_text` 产 `StyledRun`(Plain 时单 run) |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_rich_bbcode_nested_styles_flatten_to_runs` | `[b]a[i]b[/i][/b]` → run a(bold)+ run b(bold+italic) |
| `text_rich_color_size_font_overrides` | `[color=#f00][size=24]` 覆盖正确,pop 恢复 |
| `text_rich_run_boundaries_respect_clusters` | 标记不切组合簇;run 边界落 cluster 边界。(2026-07-02 评审收口)簇内边界向簇起点对齐,整簇取簇首字符 run 样式,期望据此标定 |
| `text_rich_html_whitelist_drops_unknown_tags` | `<script>`/`<div onclick>` 丢弃,内文保留 |
| `text_rich_html_entities_decode` | `&amp;`/`&#x4E2D;` 解码正确 |
| `text_rich_html_br_forces_break` | `<br>` 产 mandatory break |
| `text_rich_inline_image_reserves_metric_in_layout` | 内联图占位宽高进 03 行度量,baseline 对齐正确 |
| `text_rich_bbcode_builtin_icon_emits_inline_metric_contract` | typed icon asset 的 size/baseline/alt 进入 compiled run |
| `screen_space_ui_plan_renders_bbcode_icon_as_asset_batch` | icon paint 只产 image batch，不产生 renderer-local glyph shaping |
| `text_rich_hyperlink_carries_href_and_hit_range` | 链接 run 携 href + 命中字节区间,默认下划线+链接色 |
| `text_rich_markdown_inline_v1_contract` | versioned inline 三标记行为不回退且不扩大 Markdown 承诺 |

里程碑命令:`cargo test -p zircon_runtime text_rich --locked`。

## 7. 风险与回退

- HTML 安全面:严格白名单,不引入 html5ever;未知一律丢弃。复杂排版(table/嵌套 block)BBCode 优先,HTML 子集只覆盖内联样式 + 简单块。
- 内联 widget 与 UI 系统耦合:V1 内联对象只支持 image/icon 占位 metric;内联交互 widget(`SlateWidgetRun` 全功能)随编辑器富文本控件计划。

## 8. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态（2026-07-13）：RT-M1–RT-M7 已完成解析、块布局、span、cell box 与真实 VerticalRl WGPU 表格闭环；验收图仍为 1080×1450、257,649 bytes、SHA256 `82EC5035EDB80AC4F6D894C9A1A000279F23B75B95D4FE1881B0AC70655813DE`，target 同名计数 0。RT-M8 关闭表格链接交互：共享 hit test 先按完整 physical line frame 选候选，Horizontal/Vertical table link 2/2、padding 负向 1/1、surface host activation 1/1、既有普通链接回归 3/3。随后 `text_rich` 全量 74/75 唯一暴露 RT-M7 短 CJK + 大 padding 的 vertical preferred-frame 精度问题；已用 source-length bounded frame 与 font-size minimum row extent 修正，但最终 Cargo retry 被 concurrent Environment PMREM/ProceduralSky API drift 的 100 个外部错误阻断，故不冒充 75/75。Text03 完整 VerticalRl 首行缩进/列对齐及 Text05 Native/SDF 竖排 parity 仍 open，总状态保持 `in_progress`。

- 产出记录：[`07/2026-07-11-rich-text-html-bbcode-output-records.md`](07/2026-07-11-rich-text-html-bbcode-output-records.md)
- fixed 已修复：[rich-inline-provider-export-name](../shader/06/fixed-2026-07-11-rich-inline-provider-export-name.md)
- fixed 已修复：[rich-table-layout-provider-visibility](../../zircon_editor/editor/15/fixed-2026-07-12-rich-table-layout-provider-visibility.md)

当前状态（2026-08-01）：Text07 的编译 rich artifact 已接入 `UiResolvedTextLayout`，并移除 renderer/resource/link 对 markup cache 的 lookup-only 依赖；table cell 改为 parent compiled artifact 的 range/index projection，不复制 text、metadata 或 `CompiledRichText`。extract 现在先验证并保留可解析的 layout artifact，只在缺失或类型不匹配时编译；handle 直接强持有 type-erased `Arc`，全局 weak-token registry 已删除，最后一个 layout/command clone 释放时自动回收。cache eviction 后的 extract pointer-identity + link-hit 回归已覆盖该链路，后修复复核 P0=0/P1=0。计划仍为 `in_progress`，因为 managed Cargo、真实 WGPU framebuffer 和新的 `docs/tests/runtime/text` PNG 尚未执行或生成。

当前状态（2026-08-26）：修正上述artifact handle的value equality遗漏。公共type-erased handle现在同时校验payload类型和runtime-owner identity；compiled rich比较完整immutable compiled artifact语义但排除`estimated_bytes`驻留统计，resolved glyph artifact同时覆盖font generation、样式、writing mode、glyph/layout line及logical-virtual rebuild输入。缓存片段不进入identity，同一`Arc`走O(1)快速路径；不同链接markup、格式、解析器代际、字体代际、payload类型与layout dirty equality均有回归。生产cell projection继续由`UiParsedText` range/index view强持有parent artifact；伪造空source/default generation的测试专用`CompiledRichText::from_projection()`已删除，测试改为显式构建真实compiled owner。identity与snapshot职责分别下沉到53/66行的`glyph_artifact/identity.rs`和`glyph_artifact/snapshot.rs`，根owner保持797行。此切片为`implementation_complete_static_checked / managed_validation_pending`，不改变计划整体`in_progress`：managed Cargo、dirty-detection与stale-snapshot动态回归、真实WGPU framebuffer及`docs/tests/runtime/text`新PNG仍开放。

当前状态（2026-08-26，rich glyph artifact）：原先同一
`UiResolvedTextLayout.rich_text_artifact` 的 compiled metadata 与 glyph payload 互斥，导致布局可以保留
link/inline 元数据时 renderer 仍逐 paint run reshape。现由 private composite payload 同时持有 compiled
artifact、style-aware immutable glyph artifact、精确 layout-line snapshot 与 run-to-glyph-slice directory；
公开 DTO 和 runtime interface 未扩散 renderer 类型。整行 glyph 只存一份，paint run 借用切片；跨 run
ligature 的 continuation 获得空切片回执，不重复 shape/绘制。普通 rich layout、render prepare、cache
eviction 与 font-size override 的源码回归已覆盖该 owner 路径，显式 fallback 行也保留 negative receipt。

状态为
`rich_compiled_glyph_composite_and_run_slice_implemented /
rich_horizontal_soft_hyphen_virtual_artifact_implemented /
rich_text_only_ellipsis_virtual_artifact_implemented /
private_omitted_source_geometry_receipt_implemented /
horizontal_inline_external_cluster_artifact_implemented /
inline_empty_glyph_slice_and_geometry_receipt_implemented /
vertical_rich_and_external_block_canonical_artifact_implemented /
vertical_rich_ellipsis_virtual_artifact_implemented /
vertical_rich_soft_hyphen_virtual_artifact_implemented /
typed_virtual_fragment_role_implemented /
vertical_rich_generated_fragment_retention_implemented /
virtual_receipt_linear_capture_implemented /
rich_renderer_typed_linear_run_directory_implemented /
accessibility_source_preservation_confirmed /
static_checks_complete / managed_validation_pending`。horizontal rich U+00AD 现在消费 source hint 后发布
zero-width display anchor，由 retained logical sidecar 保留 bidi/style mapping 并生成 virtual glyph slice；
text-only horizontal ellipsis 也在 logical order 中生成，并由非空 style-owner receipt 同步驱动 current-run
度量、artifact shaping 与 renderer presentation。generated cluster 的可选 replaced-source receipt 现进入
logical/glyph identity；单一连续省略区间由 glyph owner 用于 caret、hit-test 与 selection geometry，歧义补集
fail closed。accessibility 继续读取原始 template/component/widget value，不消费视觉省略文本。horizontal
compiled inline image/widget 现按 Unreal
`SlateImageRun`/`SlateWidgetRun` 的 external layout block 语义处理：精确 compiled source range 是唯一准入，
UAX#9 与 final advance 保留 U+FFFC cluster，style shaping/glyph projection 跳过它，renderer inline run 获得显式
空 glyph slice；普通 literal U+FFFC 仍为文本。inline-only 横排行也可发布已接受的零 glyph 文本制品，而不是
negative fallback。ordinary styled VerticalRl、inline external block、U+2026 ellipsis 与 discretionary hyphen
已调用 canonical vertical provider；vertical rich 在 marker 物化后、UAX#9 前保留 logical sidecar，gate 按 typed
role 校验精确 marker grapheme，soft hyphen 还必须携 non-empty replaced U+00AD range。动态交互
widget、managed Cargo、性能/功耗与真实 WGPU PNG 仍开放，
所以计划整体继续 `in_progress`。

renderer publication 后续结构复审删除了 per-paint-run 的 layout-line `.find` + artifact-run `.find` 双重
扫描。`text_paint_runs_from_resolved_layout` 已保证 layout/run 展平顺序，因此 renderer 现在一次单调遍历
`layout.lines -> line.runs`，并通过 runtime composite 的 exact directory index O(1) 解析 glyph slice；静态规模
由最坏 `O(R^2)` 收敛为 `O(lines + runs)`。每个 run 在 paint 前获得 `Artifact`、intentional `VisualOnly` 或
`Rejected(Missing|Stale|Incomplete)` receipt；范围、顺序、snapshot 或 slice bound 不一致均 fail closed，不再
静默混入普通 fallback 统计。Rejected text run 只有在 resolved line 证明 exact source-isomorphic 时才允许
renderer reshape；generated/non-isomorphic rejected run 不发布猜测 batch。此项为
`rich_renderer_typed_linear_run_directory_implemented / static_checks_complete /
rich_nonisomorphic_rejection_fail_closed_implemented / managed_validation_pending`，未声明实测性能收益。

2026-08-31 范围校正：上述 `O(lines + runs)` 只覆盖 typed glyph-artifact route publication，不能作为
rich paint geometry 的端到端复杂度结论。当前 `text_paint_runs_from_resolved_layout` 仍逐 run 重扫行级
grapheme/advance，inline renderer 又逐对象查 line/run 并重算同一 prefix。profiling feature 已加入 7 个固定低基数
工作量/paint-frame 一致性计数器，普通 build 不保留计数字段；Interface exact-production-helper 的 Windows
release-only ignored benchmark 已覆盖 1/100/1k/10k runs、3 次 warm-up、31 个原始 timing/RSS 样本与
p50/p95/p99 输出；renderer 独立 harness 也已静态覆盖 dense LTR/RTL/VerticalRl 1/100/1k/10k inline 与
1/100/1k hard lines，并将 counter capture 放在计时外。两者均尚未运行。完整 baseline、单一 block geometry
owner 硬切、managed Cargo/profile/power/WGPU/PNG 均保持 pending。详见
[`09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md`](09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md)。

动态 inline widget 重审已经否决 renderer 实心矩形与全局裸 ID registry。MVP 采用 Unreal
`FSlateWidgetRun` 的 child-arrangement 边界：`[widget=id|widthxheight]` 的显式 size 继续参与 canonical rich
layout，markup 首字段只编译为 owner-local `RichInlineWidgetSlotId`，不再在 text artifact 中伪装成
`UiNodeId`。Surface layout 在持有当前 `UiTree` 独占访问时才把 slot 解析为当前富文本 owner 的直接 UI child；
UI tree 负责 child 生命周期、input/focus/a11y 与正常
paint，text layout 只发布绝对 frame。无效/重复/被省略的绑定 fail closed 并隐藏 child，graphics renderer 不再
伪造 widget 外观。固定尺寸 direct-child 分支现已实现：绑定目录按 compiled source range 与 resolved
line/run 单调投影 frame，布局只扫描本次 arrangement roots 的受影响子树；真实 child 走普通
arrange/render extract/hit-test，duplicate/missing/cross-parent/omitted binding 清空几何，renderer 实心矩形已删除。
该 MVP 不保留跨帧 child binding：删除、重建、换树或换 surface 都在下一次 layout 从当前 direct children
重新解析，因此 compiled cache 不会钉住旧 surface/node generation。未来 desired-size/run-local invalidation
若引入 retained binding，必须同时携 `UiSurfaceSessionIdentity` 与 `UiTree::node_incarnation` 并显式 revoke；
不得把这两个运行期身份复制进 compiled artifact。
源码回归已覆盖 exact frame/render/hit、duplicate、missing、omitted 与 no-placeholder；静态复杂度为
`O(affected tree nodes + rich runs + graphemes + direct children)`，rustfmt、定向 whitespace 与
`git diff --check` 通过；typed slot 静态合同进入完整 47/47 infrastructure 批次并在 1.744 s 通过。当前为
`dynamic_inline_widget_architecture_review_complete /
fixed_size_direct_child_inline_widget_implemented / renderer_widget_placeholder_removed /
typed_owner_local_widget_slot_implemented / current_tree_binding_nonretained /
incremental_arrangement_root_bounded / static_checks_complete / managed_validation_pending`；desired-size、
retained session/incarnation lease、managed Cargo、真实性能/功耗、真实 WGPU PNG 与完整 G27/G28 仍开放。

2026-08-28 Text07 结构规范收口：845 行的 `text/rich/parser.rs` 已把完整 Markdown dialect walker
迁入 77 行的 `text/rich/parser/markdown.rs`，共享 root 以 771 行继续唯一持有 builder、HTML/BBCode
dispatch、grapheme 对齐和 run merge。移动函数名称唯一，未增加第二份 parser state、facade、缓存或格式
分支，RichTextFormat 路径与结果合同不变。Rust 2024 rustfmt、scoped diff-check、冲突扫描和文件预算静态
通过；Cargo、parser corpus、性能/功耗与 WGPU/PNG 未执行。状态为
`markdown_dialect_owner_split_static_passed / parser_algorithm_unchanged /
managed_validation_pending`，Text07 仍保持 `in_progress`。

2026-08-30 Text07 representation admission：`RichTextParser::compile` 已硬切为 typed
`Result`，`RichParseBudget` 在 cache copy 前限制 source，在 builder append/emoji expansion 前限制
visible output；parser/tokenizer 还限制有效 token 总数、单 token bytes、单 token attribute count/bytes，
并以默认 128 层请求预算约束 HTML/BBCode 共用 ActiveTag 栈。默认值分别为 65,536 token、64 KiB/token、
64 attributes/16 KiB attribute bytes/token；token/attribute 在字符串构造前拒绝，第 `max + 1` 层在
`Vec` 增长前返回 `ActiveTagDepthBudgetExceeded`。默认 32 MiB 对齐 retained Plain document 量级，
8 MiB compiled cache 继续只管 residency。
`CompiledRichText` 的所有 byte/count/projection index 改为 checked construction，删除 `u32::MAX`
饱和别名。失败 single-flight 对当前 waiter 返回同一 terminal error 后移出 residency；UI 映射
`ZR-TEXT-LAYOUT-012` 并走 failure layout。段落 list-prefix 的最后一条饱和转换也改为 typed failure。
admission/builder 状态下沉到 child owner；后续 grapheme normalization 独立迁入
`parser/run_alignment.rs`，parser root/builder/alignment 当前为 720/183/100 行。

M1 representation admission 继续限制 runs、paragraphs、tables、table cells 和 retained cell
projection indices，默认 131,072/16,384/4,096/65,536/262,144；BBCode block/table nesting 默认
32/8 层，并在 owner vector 增长前返回 typed failure，不再静默 suppression 或把超大 table depth
饱和成相同 `u16` identity。compiled cell projection 的 `O(C * (R + P + T))` 全扫描已由 request-local
interval owner 替代；4,096 对象、31 样本隔离复测 p50/p95/p99 为
3,337/4,467/5,611 us，对旧 p50/p95 改善 18.14x/19.20x。首样本工作集增量为 360,448 bytes，
所以临时内存、allocation/RSS/power 不据此关闭。

custom decorator dispatch 也按 RRT-P1-008 完成先 profile 后硬切。旧 exact-tag Vec 对 4,096 次末项
命中在 16/256/4,096 decorators 的 31 样本 p50 为 517/7,381/116,314 us；现由 parser-local
HashMap 唯一持有 normalized tag，registration 通过 `Entry` 判重/插入，cache decorator generation
合同不变。同样后测 p50 为 140/142/139 us，4,096 decorators 下为 836.79x；不把 lookup-loop
working set 当成 registry retained memory。callback panic 现由 `catch_unwind` 投影为 typed failure；
默认单次 decorator metadata 64 KiB、全请求 retained run metadata 32 MiB，非合并 run 在发布前累计
计费。Rust panic/metadata 回归已写但未运行；deadline/cancel、callback 临时 allocator quota 仍开放。

RRT-P1-005 的 detached parse clone 也已硬切。生产 `RichTextParser::parse()` 没有消费者，却在每次
compiled cache 命中后复制全部 runs/paragraphs/tables 和动态 metadata；131,072-run 隔离基线每次
分配 395,267 次、请求 32,473,088 bytes，31 样本 p50/p95/p99 为
111,366/232,754/331,802 us。生产现在只暴露 `compile() -> Arc<CompiledRichText>`，neutral parsed
view 必须从 retained parent artifact 借用；owned parse 仅为 `cfg(test)` corpus helper。被删除生产阶段
post allocation/bytes 为 0 且不再存在，不保留 alias 或第二 cache。

这只关闭超限 identity、source/output、token/attribute materialization、ActiveTag 与 BBCode
block/table depth，以及孤立 cell projection/dispatch/owned-clone；general node/span、time/decorator deadline、
delta-style clone 优化或完整 diagnostic 均未完成。Text07 仍为 `in_progress`。当前可复现静态集合 34/34；
本次两个 E 盘 Cargo 检查分别在 90/120 秒无输出、无结论后停止，Cargo clean、真实 WGPU framebuffer、
`docs/tests/runtime/text` 新 PNG、产品 profile/RSS/power 仍待 managed validation。状态：
`rich_parser_typed_byte_admission_implemented / rich_compiled_index_saturation_removed /
rich_active_tag_depth_admission_implemented /
rich_tokenizer_count_and_materialization_budget_implemented /
rich_representation_and_block_table_depth_admission_implemented /
rich_table_projection_quadratic_rescan_removed_isolated_profiled /
rich_decorator_exact_tag_hash_dispatch_isolated_profiled /
rich_decorator_panic_and_metadata_admission_static /
rich_owned_parse_clone_hard_cut_static /
managed_validation_pending`。

2026-08-30 RRT-P1-017 identity exhaustion：compiled-rich cache identity 不再通过
`fetch_add().max(1)` 或 `wrapping_add` 回绕复用。parser identity 现在是
`Option<NonZeroU64>`，atomic allocator 以 `fetch_update + checked_add` 进入显式 exhausted
状态；耗尽 parser 在 source/cache 工作前返回 typed `ParserIdentityExhausted`。decorator/emoji
registration 在修改唯一 registry 前先用 `checked_add` admit 下一代，耗尽返回各自
`GenerationExhausted`，owner 与当前 generation 保持不变。UI 将 identity exhaustion 映射为
`LayoutFailed`，不冒充 representation budget failure。Unreal 对照仍是 widget/marshaller 强持有具体
parser/decorator owner；Zircon 在 RuntimeRichTextService 切换前保留数值 cache key 时必须保证其永不别名。
owner-local Rust 边界回归已写但未运行；当前可复现静态集合 35/35、rustfmt 与 source guard 通过。
provider lease/revoke、targeted retirement、process-global service、managed Cargo、WGPU/PNG、RSS/power
仍开放。状态：`rich_parser_identity_generation_non_reusing_static /
mutation_before_generation_admission_removed / managed_validation_pending`。详见
[`07/2026-08-30-rich-parser-generation-exhaustion.md`](07/2026-08-30-rich-parser-generation-exhaustion.md)。

2026-08-30 RRT-P1-013 process-global owner hard cut：production `shared_cache`、free rich compile/
lookup 与 shared report API 已删除。bounded compiled cache 由每个 `RichTextParser` 唯一持有；
`SharedTextLayoutSession` 通过 `Arc` 保留该 parser，layout、resolved layout、measure、prewarm、
retained document 与 render preparation 都显式使用同一 Surface session。独立 session 不共享
artifact/counter/clear lifecycle；static built-in parser 只在 `cfg(test)` corpus 中保留。

先失败后实现的 owner contract 与完整 Runtime Text 静态集合现为 36/36；same-session pointer reuse、
cross-session isolation 与 owner-local clear 的 Rust 回归已写但未运行。主 `layout_session.rs` 已按职责从
976 行拆为 476 行生产编排与 479 行 child tests。此切片未进入忙碌的 managed Cargo 队列，也不声明
latency/RSS/contention/power 收益；RRT-P1-010/014/016、Cargo、真实 WGPU/PNG 与 matched Unreal
multi-Surface profile 仍开放。状态：`rich_process_global_parser_cache_removed_static /
surface_session_rich_owner_injected_static / managed_product_validation_pending`。详见
[`07/2026-08-30-runtime-rich-text-service-owner-cutover.md`](07/2026-08-30-runtime-rich-text-service-owner-cutover.md)。

2026-08-30 RRT-P1-016 current-registration retirement：decorator/emoji 成功注册在 checked generation
提交后立即清理其 parser-owned compiled cache，不再等待 8 MiB LRU 偶然驱逐旧 generation。注册失败不改变
registry/generation，也不清理当前健康 residency；已经交给 layout/render consumer 的
`Arc<CompiledRichText>` 保留 last-use 生命周期，但不再由 cache owner 驻留。Rust 行为回归覆盖 success
retirement、failed-registration preservation 与旧 artifact 可读性，当前未运行；静态集合 36/36 与定向
Rustfmt 通过。project/session/plugin-qualified provider snapshot、unregister/revoke fence、并发 publication、
registration-count admission、managed Cargo/WGPU/PNG 仍开放，因此只记为
`rich_current_registration_generation_retirement_static / provider_revoke_open /
managed_product_validation_pending`。

RRT-P1-010/014/016 的下一阶段架构已完成 current-source/Unreal/Core lifecycle 重审：provider
catalog 必须由上层 project/runtime-plugin owner 限定并以不可变 snapshot 注入 Surface；compile 只克隆
snapshot `Arc`，不持 registry 写锁调用第三方代码。撤销必须复用 Core 的 service admission close、
`ServiceCallGuard` in-flight drain 与 timeout，再允许 module/native library unload；cache clear 或 artifact
`Arc` drop 不能冒充 revoke fence。provider retained budget 与 request-local `RichParseBudget` 分离，阈值须先做
1/64/1,024-provider E 盘 release profile。当前仅完成设计，尚未改公共 API。详见
[`07/2026-08-30-rich-provider-snapshot-and-revoke-design.md`](07/2026-08-30-rich-provider-snapshot-and-revoke-design.md)。

2026-08-30 RRT-P1-023 style shaping projection：rich run 的 explicit italic true/false 与
OpenType features 现进入 `TextStyle`、horizontal/vertical backend request、font query、Cosmic attrs 和
shaped-cache identity；公共 `TextFontRequest.italic` 也映射到同一 backend style。feature 由 immutable
`Arc` 随 resolved style 保留并在 canonical request 统一规范化，不增加 per-glyph payload。先失败后实现的
冲突合同规定同一 feature tag 最后声明生效，canonical list 按 tag 稳定排序，使 cache 与 backend 共用唯一语义。
跨层合同与完整静态集合通过 47/47，Rust 行为测试已写未运行。当前状态为
`rich_italic_and_feature_projection_static_complete / managed_validation_pending`。

letter spacing 没有随字段一起草率投影。Unreal 的 tracking 是 cluster/glyph advance 语义，非零时关闭
`liga`、只增加相邻 glyph 间隙并对 RTL/不支持输入 bypass；Cosmic 0.18.2 的快捷属性则给末 glyph 也加
advance，且 direct RustyBuzz 路径无对应实现。后续必须先跑 31 样本 E 盘 direct/fallback、LTR/RTL、
horizontal/vertical、rich-span 规模矩阵，再实现唯一 backend-neutral cluster-gap owner。详见
[`07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md`](07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md)。
RRT-P1-023、managed Cargo、真实 WGPU/PNG、RSS/power 仍开放，Text07 继续 `in_progress`。

2026-08-30 RRT-P1-033 table geometry static cutover：`TextLayoutGeometryBudget` 已成为
`SharedTextLayoutSession` 不可变快照，默认 `2^24` logical-pixel 仅作为 `f32` 数值安全天花板，
不是未经样本验证的产品 clamp。typed bounded/unbounded constraint 已由 shared rich/VerticalRl
intrinsic 与 table preferred/final pass 共用；`f32::MAX/4`、source-byte × line-advance 假 frame 和
non-finite-to-zero 均已删除。column/row solver、track prefix、line/placement frame、glyph advance、
cell box、translation 与整表累计现在逐层受检，失败记录 owner/source/work receipt 并返回
`GeometryTooLarge`；plain size、fixed-height 与 range-width 快捷度量也接入同一 shared admission owner。
基础设施静态合同 31/31；Rust 行为测试已写未运行。状态为
`RRT-P1-033_geometry_budget_and_table_cutover_static_complete /
managed_compile_render_and_profile_pending`；E 盘 31 样本、RSS/power、真实 WGPU/PNG 仍开放。详见
[`07/2026-08-30-rich-table-geometry-budget-review.md`](07/2026-08-30-rich-table-geometry-budget-review.md)。

2026-08-30 RRT-P1-011 index admission：parser budget 与 `CompiledRichText` 构造已负责 `u32`
byte/index 上限；UI adapter 剩余的根 `as u32` 与子 projection `filter_map` 静默丢弃已删除。
`UiParsedText` 根/子构造现为 fallible checked path，无效 run/paragraph/table index 返回
`LayoutFailed`，artifact rebuild 不发布部分语义。静态集合 47/47；Rust 行为测试已写未运行，
状态为 `compiled_and_ui_projection_index_static_complete / managed_validation_pending`。

2026-08-30 RRT-P1-024/RRT-P1-025 format identity hard cut：当前三 marker 语法已从会误导调用方的
`Markdown` 改为 `MarkdownInlineV1`，BBCode/HTML owner 同步版本化为 `BbCodeV1` 与
`HtmlSubsetV1`；runtime/interface serde 与 style property 只接受 `markdown_inline_v1`、
`bbcode_v1`、`html_subset_v1`，旧值不保留 alias。compiled cache key 直接持有可 hash 的
`RichTextFormat`，删除手写 `u8` 映射，使新增 grammar version 自动获得不同 artifact/cache identity。
静态迁移契约已通过；wire round-trip/rejection Rust tests 已写未运行。RRT-P1-024 的命名/identity 切片
已纳入 47/47 静态集合，RRT-P1-025 的 bounded authoring diagnostic、source range 与 recovery receipt
已有 structural slice：unsupported/unmatched/misnested/unclosed tag 以四个稳定 code、source-markup
range 与 recovery 写入 canonical `RichParseResult`；默认上限 256，超限设置独立 truncation receipt，cache
驻留估算包含诊断容量。后续同一单遍已增加 unsupported/malformed attribute、invalid value 与 unsupported
style property；本轮继续补齐 malformed tag candidate、unterminated quoted attribute、malformed /
unrecognized entity，稳定 code 扩为十二个。畸形 markup/entity 按原 source text 恢复，普通 `one < two`
不误报；诊断在 tokenizer/entity decode 的既有 O(n) 路径中生成，EOF source order 已有行为契约。
HTML 状态机与活动标签栈拆到独立 owner 后 parser root/html/diagnostics/active-tags child 为
558/259/108/123 行。静态集合
47/47；Rust 行为测试已写未运行。RRT-P1-025 当前诊断类已静态完成，managed Rust、bounded corpus
profile 与 product evidence 仍开放。详见
[`07/2026-08-30-rich-format-version-identity-review.md`](07/2026-08-30-rich-format-version-identity-review.md)。

2026-08-30 测试 owner 收敛：`rich/tests.rs` 中 parser 规模语料、两项 ignored release evidence 与
legacy 对照 helper 原样迁入 `rich/tests/parser_performance.rs`，根/子 owner 为 758/238 行；测试样本、
阈值和断言未改变。结构合同与完整 Runtime Text infrastructure 静态批次 47/47（1.744 s）通过，
状态为 `rich_parser_performance_test_owner_split_complete / behavior_unchanged /
managed_validation_pending`。

2026-08-30 RRT-P1-038 table layout work receipt：current-source 重审确认 parser 已有 token、table、
cell、depth、run、paragraph、projection、source/output request-local 预算，旧 finding 的解析侧结论已过时；
实际缺口是 preferred/final 两阶段布局工作不可见。`SharedTextLayoutSession` 现在逐帧持有
`TextTableLayoutWorkReport`，累计 table/source bytes/total-max cells、两阶段 cell 次数与输入 bytes、
resolved tracks 以及几何验收后发布的 line/box 数。计数只用 saturating arithmetic，profile 仅发布十二个
固定名称，不含 source text 或动态标签，也不改变布局顺序、失败政策或缓存。完整 Runtime Text 静态集合
52/52 与定向 Rustfmt 通过；Rust 行为测试已写未运行。状态为
`RRT-P1-038_table_layout_work_receipt_static_complete / managed_profile_and_budget_decision_pending`；
31 样本 E 盘 profile、阈值/retained intrinsic cache 决策、RSS/power、真实 WGPU/PNG 仍开放。详见
[`07/2026-08-30-rich-table-layout-work-receipt-review.md`](07/2026-08-30-rich-table-layout-work-receipt-review.md)。

2026-08-30 RRT-P1-034/036 current-source 校准：`ResolvedRichTextArtifact` 已合并 compiled metadata、
generation-bound glyph sidecar、exact layout lines 与 run-to-glyph directory；正常 artifact route 直接消费
glyph slice，不再逐 rich paint run 调 renderer fallback shaping。剩余结构成本是 serializable layout line/run
字符串、`UiTextPaintRun` 再物化与 compiled style 的 checked range lookup。对齐 Unreal 的目标仍是共享全文、
run/style/block 与 shaped cache 的同一 retained owner，但在 E 盘 cold/first-paint/stable-repaint allocation/
timing/RSS/power 之前不改 DTO/serde/remote contract，也不添加第二 cache。状态为
runtime renderer 现新增固定 `materialize_transient_text_paint` scope 与十二项 command/run/text/style-byte
计数，segment cache 只记本帧重建，完整命中发布零新 work。52/52 静态集合通过；字节只是 payload 长度
下界，仍需 allocator/RSS 动态证据。状态为
`RRT-P1-034_paint_projection_profile_infrastructure_static_complete /
RRT-P1-036_managed_baseline_and_owner_decision_pending`。详见
[`07/2026-08-30-rich-prepared-run-current-source-review.md`](07/2026-08-30-rich-prepared-run-current-source-review.md)。

2026-08-30 RRT-P1-039 accessibility semantic projection：own name 与 `labelled_by`/description
relation text 过去直接读取 template scalar，富文本标签可能作为可访问名称暴露。当前新增 runtime-private
`RichSemanticProjection`，强持有同一 `CompiledRichText` generation，并只在 current render command 的
source、versioned format 与 artifact 全部一致时投影 compiled visible text。accessibility 通过 render cache
现有 per-node range 查找，不扫描整表、不重 parse、不从 clip/ellipsis layout line 拼接；多候选 generation
比较为 O(1)，stale/missing/ambiguous artifact fail closed，plain 与 explicit a11y/alt/tooltip 优先级不变。
HTML own-name、BBCode relation、stale source 与 artifact source/format/generation 回归已写；完整 Runtime Text
静态集合 53/53。后续隐藏 relation target 已通过同一 Surface `SharedTextLayoutSession` 的 compiled cache
获得 visibility-independent projection；存在 render command range 时仍以视觉 artifact 为权威并 fail closed，
没有 a11y parser、第二 cache 或全 hidden-tree eager parse。完整静态集合现为 54/54。状态：
`RRT-P1-039_visibility_independent_surface_semantic_owner_static_complete /
RRT-P1-040_typed_children_and_managed_validation_pending`。RRT-P1-040 的 qualified semantic id/action route、
link/inline/list/table typed child、managed Rust/AccessKit/screen-reader、WGPU/PNG 与 RSS/power 仍开放。详见
[`07/2026-08-30-rich-accessibility-semantic-projection-review.md`](07/2026-08-30-rich-accessibility-semantic-projection-review.md)
与 [`07/2026-08-30-rich-visibility-independent-semantic-owner-review.md`](07/2026-08-30-rich-visibility-independent-semantic-owner-review.md)。

2026-08-30 RRT-P1-037 list semantic metadata hard cut：BBCode parser 过去已知 ordered/unordered、
marker style、ordinal 与 nesting depth，却只保留可见 marker range，后续若要 copy/a11y 只能从字符串反推。
当前 `ParagraphOverride` 已硬切为 typed `RichListItem`：ordered kind 同时持有 checked ordinal 与五种
`RichOrderedListMarker`，item 持有一基 semantic level 与 exact compiled-visible marker range。UI layout
只派生 geometry range，并通过私有 `ResolvedParagraphLayoutOverride` 做物理段落 sweep，不再把 range 写回
semantic model。该改动不声明完整 typed block tree、HTML list 或 a11y child 已完成；完整 Runtime Text 静态
集合 55/55，Rust 行为测试已写未运行。状态：
`RRT-P1-037_typed_list_item_metadata_static_complete /
RRT-P1-040_qualified_publication_and_managed_validation_pending`。详见
[`07/2026-08-30-rich-list-semantic-metadata-hard-cut.md`](07/2026-08-30-rich-list-semantic-metadata-hard-cut.md)。

2026-08-30 RRT-P1-029 inline image semantic fallback：HTML/BBCode image compiled run 现在保留
`alternative_text` 与 `tooltip`，HTML whitelist 接受 `alt/title`，BBCode 在原 positional form 外支持
`[img src=... alt=... title=...]`。两项字符串进入既有 run metadata quota/cache residency；compiled owner
在 inline index 建成后一次物化受独立 `max_semantic_text_bytes` 约束的 semantic `Arc<str>`，无 inline
时与 visible text 共用 Arc。a11y 只做 O(1) owner read，不逐帧扫 run 或重 parse。显式空 alt 表示 decorative，
缺失 alt 才使用 tooltip；无 fallback 的 image 及尚未 qualified 的 icon/widget 不暴露 U+FFFC。完整 Runtime
Text 静态集合 56/56，Rust 行为测试已写未运行。状态：
`RRT-P1-029_inline_image_semantic_fallback_static_complete /
RRT-P1-040_qualified_inline_children_and_managed_validation_pending`；resource readiness/region/units/load-error、
qualified child/action、managed WGPU/PNG 与 profile 仍开放。详见
[`07/2026-08-30-rich-inline-image-semantic-fallback-owner-review.md`](07/2026-08-30-rich-inline-image-semantic-fallback-owner-review.md)。

2026-08-30 RRT-P1-030 link target owner hard cut：parser 过去先校验 `ResourceLocator`，随后把它降回
`String`；hit/effect/host request 多次 clone，effect application 又维护一套 `split_once`/`Path`/fallback
parse 规则。当前 RuntimeInterface 私有字段的 `UiRichLinkTarget` 成为唯一许可 owner：构造与 serde 只接受
`res/lib/package/builtin`，保留原无 scheme 的 `res://` shorthand，并以 `Arc<ResourceLocator>` 在 compiled run、
hit、effect 与 host request 间 O(1) 共享。应用边界只校验真实 `UiNodeId` owner，不再解释链接字符串；serde
仍输出既有 `href` scalar wire。完整 Runtime Text 静态集合 57/57，Rust 行为测试已写未运行。状态：
`RRT-P1-030_typed_link_target_foundation_static_complete /
RRT-P1-040_qualified_link_child_and_managed_validation_pending`；action kind、tooltip/state、navigation policy、
trust/principal、qualified semantic child/action、managed Cargo/host/WGPU/PNG/profile/RSS/power 仍开放。详见
[`07/2026-08-30-rich-link-target-owner-hard-cut.md`](07/2026-08-30-rich-link-target-owner-hard-cut.md)。

2026-08-30 RRT-P1-028 typed image-icon foundation：匿名 `glyph + family` 路径已硬切为
`RichIconAssetId(ResourceId)`，run 显式持有 size、baseline 与 alternative text；内建 `[icon]` 只接受
受控 engine resource locator，不保留旧 family fallback。horizontal/VerticalRl layout 与 renderer 共享
同一几何，paint 直接产生 image batch，不再为 icon 二次 shape；`IconAsset` 已进入 typed dependency
closure 和 UI texture collector，alternative/decorative text 同步进入 quota、residency 与 compiled semantic
projection。完整静态集合 40/40（最终复跑 0.222 s），Rust 行为测试已写未运行。状态：
`RRT-P1-028_typed_image_icon_asset_hard_cut_static_complete /
intrinsic_metric_revision_readiness_font_icon_and_managed_validation_pending`；intrinsic metric/revision
invalidation、qualified readiness receipt、font-icon lease、真实 WGPU/PNG 与 profile/RSS/power 仍开放，
不声明性能或像素完成。详见
[`07/2026-08-30-rich-icon-asset-and-font-lease-architecture-review.md`](07/2026-08-30-rich-icon-asset-and-font-lease-architecture-review.md)。

2026-08-30 RRT-P1-029 inline resource outcome owner review：authored size/semantic fallback 继续由
compiled run 持有，mutable ready/fallback/error 明确归 frame render-resource prepare。当前
`ui_texture_id_for_upload -> Option` 与被丢弃的 upload 结果会把 unresolved、load failure、dimension
mismatch、upload failure 合并；后续必须先建立 management/readiness generation-qualified typed receipt，
再谈缓存或 registry 算法优化。状态：`RRT-P1-029_resource_outcome_architecture_review_complete /
frame_qualified_prepare_receipt_implementation_not_started / managed_profile_and_product_validation_pending`。
详见 [`07/2026-08-30-rich-inline-resource-outcome-owner-review.md`](07/2026-08-30-rich-inline-resource-outcome-owner-review.md)。

2026-08-30 RRT-P1-030 link tooltip metadata：`LinkRef` 新增共享 `Arc<str>` tooltip；HTML
`a[title]` 与 BBCode `[url href=... title=...]` 进入同一 compiled run，decorator quota、compiled
residency 与 `UiTextLinkHit` 保留同一份 metadata。该字段不被误接到以 overlay ID/timer 为 owner
的 surface tooltip 状态机；qualified hover/a11y action 后续再消费。完整 Runtime Text 静态集合
58/58（最终复跑 0.236 s），Rust parser/cache-eviction hit 行为测试已写未运行。residency
估算已从 803 行 root 拆到独立 `compiled/memory.rs`；当前 root/memory 为 730/76 行。状态：
`RRT-P1-030_typed_target_and_tooltip_metadata_static_complete /
RRT-P1-040_qualified_link_child_and_managed_validation_pending`；visited/disabled/action/policy/principal、
managed Cargo/host、WGPU/PNG/profile/RSS/power 仍开放。详见
[`07/2026-08-30-rich-link-tooltip-metadata-owner-review.md`](07/2026-08-30-rich-link-tooltip-metadata-owner-review.md)。

2026-08-30 RRT-P1-020 typed dependency closure foundation：旧 `resource_ids()` 只收集 image texture，
却用无类型名称向 GPU collector 暗示“全部资源”；若直接加入 icon/font/widget/decorator 会把非纹理 identity
错误送进 texture streamer。当前新增 `RichTextDependency::ImageTexture(ResourceId)`，compiled artifact
持有排序去重的 typed slice，纹理收集端显式 match kind，residency 按 enum 元素计费，旧 API 已删除。
Icon asset、widget owner-local slot 与 decorator generation 均保持各自 typed kind，不伪装成 texture 资源。完整静态集合
59/59（最终复跑 0.363 s），Rust 行为测试已写未运行。状态：
`RRT-P1-020_typed_image_dependency_foundation_static_complete /
icon_font_widget_decorator_lease_and_managed_validation_pending`；无性能或像素声明。详见
[`07/2026-08-30-rich-typed-dependency-closure-foundation.md`](07/2026-08-30-rich-typed-dependency-closure-foundation.md)。

2026-08-30 RRT-P1-022 cache telemetry current-source correction：compiled rich cache 已由
`SharedTextLayoutSession -> RichTextParser -> CompiledRichTextCacheOwner` 单链持有，旧 UI sampler
却只能对累计 `u64` 做外部差分；计数饱和后会永久失真，也无法把样本绑定到 parser/provider 代际。
当前由 cache mutex 内的 `take_report()` 原子复制并重置六项区间事件，驻留 entries/bytes 与上限作为
gauge 保留；parser owner 同步投影 parser identity、decorator generation、emoji generation 与 saturation
receipt。Surface profile 当前发布 16 个固定名称（含后续 RRT-P1-014 的 4 项 contention 字段），不含
markup、pointer 或动态 tenant label。状态：
`RRT-P1-022_parser_provider_qualified_reset_snapshot_static_complete /
project_surface_correlation_and_managed_profile_pending`；当前基础设施静态集合 36/36（最终复跑
0.206 s），managed Cargo/profile/RSS/power 仍开放，不提前宣称性能收益。详见
[`07/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md`](07/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md)。

2026-08-30 RRT-P1-009/014 single-flight contention measurement：current source 已有 decorator
`catch_unwind`、typed panic failure、per-call metadata 与 retained-run budget，因此计划中的“完全无隔离”已过时；
真正未闭合的是非协作 provider 的 deadline/cancel，以及 `OnceLock::get_or_init` 对同 key 调用者的阻塞。
本轮不改变 single-flight 算法，只在 cache owner 内增加 point-in-time in-flight gauge 与已完成 waiter 的
count/total/max nanos；`Cell` 标记真正执行 initializer 的调用者，RAII guard 在 unwind 时回收 gauge，profile
固定字段从 12 增至 16。完整基础设施静态集合 36/36（最终复跑 0.206 s），production/tests/profile owner
为 541/340/739 行。状态：`RRT-P1-014_contention_measurement_static_complete /
bounded_worker_cancellation_and_managed_profile_pending`；禁止在 contention 数据前直接移除 single-flight、
复制 parse 或加入任意 timeout。详见
[`07/2026-08-30-rich-single-flight-contention-instrumentation.md`](07/2026-08-30-rich-single-flight-contention-instrumentation.md)。

2026-08-30 RRT-P1-041 bidi-control authoring/trust gate：current-source 重审确认 UAX#9 shaping、
logical range、visual reorder 与 a11y logical offset 已有明确边界；缺口在 rich parser 无法说明不可见
direction control 的来源。当前 Plain/Markdown-inline/HTML-subset/BBCode 的可见 source slice 统一进入
bounded 诊断 owner，方向 mark、embedding/pop、override、isolate 分别使用稳定 code 013..016；HTML
numeric entity 在原 decode loop 内回报精确实体 range，BBCode literal 使用 token range。文本不 strip、
replace 或自动插入 FSI/PDI，全部复用现有 256/自定义 authoring quota 与 truncation receipt。完整基础设施
后续已增加 typed `RichTextContentTrust`：默认 `Untrusted` 允许 mark 与 balanced isolate，拒绝 legacy
embedding/pop/override；显式 `TrustedAuthoring` 仍要求控制栈平衡。专用深度上限 125 在栈增长前失败，
trust 同时进入 cache key 与 compiled identity，raw/entity/tag 共享相同策略与精确 source range。完整基础设施
静态集合 38/38（最终复跑 0.090 s），当前 parser root/html/bidi leaf/root tests/parser-performance
tests/bidi tests 为 558/259/195/758/238/215 行。
状态：`RRT-P1-041_trust_gate_and_balanced_isolation_static_complete /
managed_copy_a11y_render_and_profile_pending`；managed Rust、copy/a11y/WGPU/PNG/profile 仍开放。详见
[`07/2026-08-30-rich-bidi-control-authoring-diagnostics.md`](07/2026-08-30-rich-bidi-control-authoring-diagnostics.md)。
