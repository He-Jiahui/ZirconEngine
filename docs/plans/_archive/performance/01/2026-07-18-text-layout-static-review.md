---
related_code:
  - zircon_runtime/src/text/layout
  - zircon_runtime/src/ui/text/layout_engine
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/slint/internal/core/textlayout/shaping.rs
  - dev/slint/internal/core/textlayout/linebreaker.rs
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/textwrapper.rs
tests:
  - current-source Windows zircon_runtime layout tests pending
  - 1/100/1k/10k grapheme and run scaling counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text layout逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/layout/**`当前源20/20个Rust文件、4,215行已逐文件阅读，覆盖align、kinsoku及测试、line-break glue/glyph-fallback/greedy/smart/soft-hyphen/wrap-space及测试、measure、overflow、rich horizontal/vertical及测试、tab与vertical layout。调用图确认这些函数由`ui/text/layout_engine/**`的普通、rich、vertical、ellipsis与wrapping生产路径消费，不是孤立helper。

## PERF-MVP-236：plain layout重复前缀shape/measure

`measured_grapheme_widths_with_provider`先获得一次整行shaped run，却为每个grapheme调用`measured_width`，后者重新遍历全部lines/glyphs判断range overlap，长行最坏O(G²)。greedy `appended_text_fits_with_provider`为每个候选chunk重新分配`current_text + next_text`并shape增长中的完整前缀；UI wrapping对连续chunks反复调用该入口。`ellipsize_text`的start/end/middle策略逐字素构造候选String并回调完整measurement，middle每轮最多测左右两个候选，因此同样产生O(G²)字节复制与shape。

修复应在一次shaped paragraph上建立cluster边界与prefix advance索引，子范围度量、行宽累计、glyph fallback与ellipsis只查询该索引；ligature/kerning边界若不能直接相减，应使用有界相邻cluster修正或backend提供的range measure，不能回退为重shape全部前缀。soft hyphen、BIDI与vertical range语义要求Text03统一实现，本轮没有用不正确的简单二分或逐字符advance替换。

## PERF-MVP-237：rich layout逐字素run扫描与整源复制

rich glyph/word/vertical wrap为每个grapheme从`parsed.runs`开头线性`find`所属run，随后clone/resolve完整`TextStyle`并对单grapheme shape。source-range measurement又逐grapheme重复该流程；word wrapping在overflow判断前后对同一chunk调用两次，glyph fallback尾段还再测一次。最终`layout_rich_ranges_with_provider`为每个视觉行扫描所有runs、clone相交runs，并把整份`parsed.text` clone进临时`RichParseResult`后再次layout，形成O(G×R + L×(R+T))访问/复制。

Text03应与Text07共享编译后的排序run spans和style identity，按连续same-style text span一次shape并生成cluster prefix advances；断行使用单调run cursor/two-pointer，视觉行只保存source/run ranges并借用原始文本。这样也恢复当前per-grapheme shaping丢失的跨字素kerning/ligature语义，而不是仅加缓存掩盖重复调用。

## 参考引擎结论

Slint先把itemized runs shape进单一`ShapeBuffer`，line breaker消费fragments与已有glyph advances；Bevy/Parley把`Layout`保存在`ComputedTextBlock`并在同一layout上`break_all_lines`，输出阶段保留Vec容量。Fyrox wrapper逐glyph接收advance并以累计width断行。UE在`FShapedGlyphSequence`上提供whole/sub-range `GetMeasuredWidth`，`FTextLayout`另持wrapping cache和line views。共同点是shape结果是布局输入，不在每个grapheme、chunk或候选前缀上重建shape。

## 责任计划与验收

Text03收到`failure-2026-07-18-layout-prefix-and-grapheme-remeasurement.md`，rich run ownership联动Text07。验收需覆盖1/100/1k/10k grapheme与1/100/1k runs：backend shape calls、glyph/run visits、style clones、candidate/source cloned bytes、layout p50/p95；目标为每连续style span shape不超过一次、run visits近O(G+R)、plain range/ellipsis近O(G)或O(G log G)、per-line source clone为0。current-source Cargo、workbench/Console trace与horizontal/vertical/RTL/inline/ellipsis产品像素完成前，本目录保持pending。
