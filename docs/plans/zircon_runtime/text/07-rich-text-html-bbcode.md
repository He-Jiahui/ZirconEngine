---
related_code:
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/RichTextLayoutMarshaller.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/RichTextMarkupProcessing.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/IRichTextMarkupParser.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ITextDecorator.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextDecorators.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateImageRun.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateHyperlinkRun.h
  - dev/godot/scene/gui/rich_text_label.h
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
status: in_progress
---

# 07 富文本(BBCode + HTML 子集)

> 本计划把"带标记的字符串"解析为"样式 run 序列 + 内联对象",喂 `02/03`。承接 `editor_ui/03` 文本主链。当前 `rich_text.rs` 仅 markdown 三标记(`**bold**`/`*italic*`/`` `code` ``),本计划补 BBCode 全集 + 受控 HTML 子集 + 装饰器 + 内联对象(图标/超链接)。

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

格式由 `RichTextFormat::{Plain, BBCode, Html, Markdown(既有)}` 选择;解析器产**中立** `StyledRun`,后续链路与纯文本统一。

## 5. 里程碑

### RT-M1 解析器框架 + BBCode 核心标签

实施切片:
1. `graphics/text/rich/parser.rs`:tokenizer(BBCode `[tag=val]...[/tag]` + HTML `<tag attr>`)+ 标签栈 → `StyledRun`。
2. `graphics/text/rich/decorator.rs`:`DecoratorRegistry` + 内置文本样式装饰器(b/i/u/s/color/bgcolor/size/font);样式 run 合并(嵌套→扁平)。
3. `ui/text/rich_text.rs` markdown 解析迁入统一框架(保留为 `RichTextFormat::Markdown`)。

测试:`text_rich_bbcode_nested_styles_flatten_to_runs`、`text_rich_color_size_font_overrides`、`text_rich_run_boundaries_respect_clusters`。

### RT-M2 HTML 受控子集 + 安全白名单

实施切片:
1. HTML tokenizer + 白名单标签/属性(`b/i/u/s/span[style]/font/br/a[href]/img[src]`);`style` 仅解析受控属性(color/font-size/font-weight/font-style/text-decoration);未知标签/属性丢弃(不报错、不执行)。
2. 实体解码(`&amp;`/`&#xNN;`);`<br>` → 强制换行(`02` mandatory break)。

测试:`text_rich_html_whitelist_drops_unknown_tags`、`text_rich_html_entities_decode`、`text_rich_html_br_forces_break`。

### RT-M3 内联对象(图标 / 超链接 / 表情)

实施切片:
1. `InlineObject`(图标/图片/widget 占位):metric(尺寸 + baseline 对齐)参与 `03` 布局;`[img]`/`<img>`/emoji shortcode。
2. 超链接 run(`[url]`/`<a>`):携 href + 命中区间(供交互层),样式默认下划线 + 链接色。

测试:`text_rich_inline_image_reserves_metric_in_layout`、`text_rich_hyperlink_carries_href_and_hit_range`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/rich/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | `parse_rich_text(markup, format, base_style) -> RichParseResult`(薄) |
| `parser.rs` | tokenizer(BBCode/HTML/Markdown 三 lexer)+ 标签栈 → `StyledRun` |
| `decorator.rs` | `DecoratorRegistry`、`TextDecorator` trait、内置样式装饰器 |
| `html_subset.rs` | HTML 白名单标签/属性表 + 实体解码 + `style` 受控属性解析 |
| `bbcode.rs` | BBCode 标签集(对照 godot)+ 自定义标签注册 |
| `inline.rs` | `InlineObject`、内联 metric 与 baseline 对齐 |

契约层 `core/framework/render/text/rich.rs`:`RichTextFormat`、`StyledRun`、`StyleOverride`、`InlineObjectRef`(serde)。

### 核心类型

```rust
pub enum RichTextFormat { Plain, BBCode, Html, Markdown }
pub struct StyledRun {
    pub byte_range: (u32, u32),       // 源(已剥标记)文本字节区间
    pub style: StyleOverride,         // 覆盖 base TextStyle 的增量
    pub inline: Option<InlineObjectRef>,
    pub link: Option<LinkRef>,        // href + 命中
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
    Image { texture: ResourceId, size: Vec2, baseline: InlineBaseline },
    Icon  { glyph: char, font: FontFamilyName },
    Widget { id: u64, size: Vec2 },   // 内联 widget 占位(UE SlateWidgetRun)
}
// (2026-07-02 评审收口)内联对象基线对齐模式:
pub enum InlineBaseline {
    Baseline, // 对象底边坐 alphabetic baseline(默认,图标/表情)
    Center,   // 对象垂直中心对齐行 x-height 中心
    Top,      // 对象顶边对齐行 ascent
    Bottom,   // 对象底边对齐行 descent
}
pub trait TextDecorator {
    fn tag(&self) -> &str;
    fn apply(&self, attrs: &TagAttrs, ctx: &mut DecorateCtx); // 改 StyleOverride 或产 InlineObject
}
pub struct RichParseResult {
    pub text: String,                 // 剥标记纯文本
    pub runs: Vec<StyledRun>,
    // (2026-07-02 评审收口)段落级覆盖:byte_range 为剥标记文本内的段落区间
    pub paragraphs: Vec<((u32, u32), ParagraphOverride)>,
}
```

解析 → run 合并:嵌套标签维护样式栈,每遇文本片段产一个 `StyledRun`(当前栈样式合并);相邻同样式 run 合并;run 边界后续由 `02` 对齐 cluster(标记不可切簇)。

(2026-07-02 评审收口)**簇内样式边界裁决**:标记边界落在组合簇(grapheme cluster)内部时,样式边界**向簇起点对齐**——整簇取**簇首字符所在 run 的样式**,后续字符的样式覆盖被吸收丢弃(不拆簇、不产生半簇 run);`text_rich_run_boundaries_respect_clusters` 的期望按此标定(如 `a[b]\u{0301}[/b]` → `á` 整簇非 bold)。

(2026-07-02 评审收口)**内联对象行度量规则**:`InlineObject` 按其 `InlineBaseline` 模式换算出等效 ascent/descent(如 `Baseline` 模式下 ascent=对象高、descent=0),该 ascent **参与 03 行 ascent 的 max 计算**(与混 face 行度量同一 max 规则,见 03 §6"混 face 行度量"/D7);对应布局槽位为 03 `LayoutItem::Inline`(03 已预留,本计划 RT-M3 落地时回填其解析来源)。

### 安全(HTML 子集)

- **白名单**:仅列表内标签/属性进解析,其余**静默丢弃**(标签丢、文本保留)。
- `style` 属性只取 color/font-size/font-weight/font-style/text-decoration;不解析任意 CSS、不支持 `url()`/脚本/事件属性。
- `img src`/`a href` 仅接受 `res://`/相对资源路径或受控 scheme;不发起网络请求(资源经资产系统)。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| `ui/text/rich_text.rs` markdown 三标记 | 迁 `graphics/text/rich/`,保留为 `RichTextFormat::Markdown`;调用方改 `parse_rich_text` |
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
| `text_rich_hyperlink_carries_href_and_hit_range` | 链接 run 携 href + 命中字节区间,默认下划线+链接色 |
| `text_rich_markdown_compat_unchanged` | 既有 markdown 三标记行为不回退 |

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
