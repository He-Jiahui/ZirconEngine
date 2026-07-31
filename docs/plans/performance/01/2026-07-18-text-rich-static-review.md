---
related_code:
  - zircon_runtime/src/text/rich
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/graphics/scene/resources/ui_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/RichTextLayoutMarshaller.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/RichTextMarkupProcessing.h
  - dev/godot/scene/gui/rich_text_label.h
tests:
  - shared builtin parser and borrowed no-replacement guards authored
  - monotonic grapheme-to-run source guard authored
  - current-source Windows zircon_runtime text::rich tests queued
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text rich逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/rich/**`当前源14/14个Rust文件、3,150行已逐文件阅读，覆盖BBCode token/style/block/table attributes/placement、decorator registry、emoji shortcode、HTML subset、inline decorators、plain/Markdown/HTML/BBCode parser及全部block/table/parser测试。调用图继续追到UI prewarm/measure/layout/link hit、scene resource streamer和screen-space UI renderer，确认解析是MVP编辑器rich label的跨阶段生产路径。

## PERF-MVP-238：局部parser setup、临时String与run对齐

内部`parse_rich_text`原先每次构造`RichTextParser::default()`，即重新分配11个boxed decorators及内置emoji `BTreeMap<String,String>`。BBCode `expand`和HTML `decode_entities`对没有任何替换的普通片段也总是分配String，随后立即复制进`RichParseResult.text`。最终`align_runs_to_graphemes`为每个grapheme从runs开头`find`所属run，标记密集文本最坏O(G×R)。

本轮先写共享实例、borrowed no-replacement和monotonic cursor门禁并记录RED，再用`OnceLock<RichTextParser>`复用只读builtins、自定义parser仍保持实例隔离；两个helper返回`Cow<str>`，只有真实替换才拥有字符串；run alignment改为排序runs单调cursor，复杂度近O(G+R)。rustfmt与diff检查通过，Cargo动态仍在协调器队列。

## PERF-MVP-239：跨帧阶段重复解析与多份文本所有权

同一rich command先在shape prewarm的`layout_prewarm_text`解析，measure与full layout入口各自调用`parse_source_text`；scene resource streamer每帧为inline texture再次parse全部rich commands；renderer在paint前再次parse，`inline_layout_frame`对每个inline fallback又parse一次；link hit按输入事件再parse。稳定command即使layout cache命中，graphics resource/render阶段仍没有compiled artifact handle。

`UiParsedText`在完整`RichParseResult`之外再clone stripped text、paragraph Vec，并为每run创建substring String、clone style/inline/link。该DTO随后只靠byte ranges关联paint/layout，迫使下游再次查找/parse；rich layout中的per-grapheme run扫描另由PERF-MVP-237覆盖。Text07需要发布generation-owned `Arc<CompiledRichText>`，Text09负责有界cache和generation失效，UI/graphics只消费共享ranges、inline/link/resource索引。

## 参考引擎结论

UE `FRichTextLayoutMarshaller`长期持有shared parser与decorators，默认markup parser还直接提供`GetStaticInstance()`；`SetText`把一次parse结果追加为目标`FTextLayout` runs，而不是每个绘制消费者重parse。Godot `RichTextLabel`保存item tree、line cache与first-invalid-line/font-line，只在内容/字体失效后更新。Zircon已用共享built-in parser关闭setup重复，但仍缺command-generation级compiled parse artifact与局部失效。

## 责任计划与验收

Text07收到`failure-2026-07-18-rich-text-reparsed-across-frame-consumers.md`并联动Text09。验收覆盖1/100/1k commands、1/100/1k runs、stable 300 frames：每command generation parse≤1、stable parse=0、per-run substring ownership=0、resource/render不扫描markup；记录parse calls/bytes、artifact/cache bytes、command/run visits和p50/p95。current-source Cargo、workbench rich labels、inline image/link/table horizontal/VerticalRl与RenderDoc/像素证据完成前，本目录保持pending。
