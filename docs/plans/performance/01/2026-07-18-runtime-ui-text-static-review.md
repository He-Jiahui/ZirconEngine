---
related_code:
  - zircon_runtime/src/ui/text
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/text_pipeline/measure_cache.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/TextLayout.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp
  - dev/slint/internal/core/textlayout.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/textwrapper.rs
tests:
  - three source-level RED to GREEN performance guards passed
  - rustfmt check and scoped git diff check passed
  - current-source Windows runtime lib build running through shared Cargo coordinator
  - cache/hit/paragraph/BiDi/table scale counters pending
  - F4 text editing, IME and pixel trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI text逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`zircon_runtime/src/ui/text/**`tracked source 56/56：顶层12/12，geometry helper/test 2/2，layout engine production/test 40/40，rich-text helper/test 2/2。累计UI tracked source371/783。物理current UI另有外部未跟踪`zircon_runtime/src/ui/tests/v2_asset/performance_guards.rs`，所以current source为784；该新增文件未纳入本批并继续pending。

静态扫描命中39处clone、34处collect、2处sort、34处to_string、13个parse调用、36个measure调用和43个layout调用，未命中lock/thread spawn。clone/collect集中在rich table source slice、rich adapter、overflow/style、measure cache、paragraph、visual order和vertical layout；测试提供Unicode/BiDi/kinsoku/ellipsis/inline/table/vertical语义，但没有长文本、大表、分配或backend-call规模门禁。

## PERF-MVP-298：frame cache hit仍复制正文与完整layout

`resolve_or_shape`原先在任何lookup前调用`resolved_text()`生成String，frame/persistent cache hit再clone包含完整lines/runs/String/advance Vec的resolution；miss还让两层cache各持source/layout。style key逐request clone/normalize font/language，width bucket为常量`"n"`走真实measure。

本轮RED→GREEN把无preedit resolved text改为Cow borrow，只有frame miss才创建一个`Arc<str>`并共享给两层cache。EditorUI03/Text09仍需把resolution改为generation-owned Arc handle，并把style/width metric绑定compiled style generation。UE `FTextLayout`维护LineModels/LineViews、per-line shaped/wrapping cache和dirty flags；Zircon不能在cache命中后再深复制整个布局产品。

## PERF-MVP-299：hit/caret/selection重复分配与prefix shape

hit-test线性找line并原先总是构造advance Vec；source-metrics caret/selection为了弥补stale advances，会对每个boundary重新shape从line start到caret的prefix，IME composition每span计算两端。keyboard grapheme/word/line navigation也反复从字符串头扫描。

本轮RED→GREEN让已有finite non-negative advances直接borrow，保留invalid/zero fallback语义。EditorUI03联动Text03/09需让layout artifact包含line interval、cluster source↔visual map与prefix advances，所有interaction查询不得shape。Fyrox wrapper以line ranges输出，UE per-line cache也说明line/cluster索引应归布局owner而不是输入事件临时重算。

## PERF-MVP-300：paragraph/wrap按line重复全局扫描

newline splitter原先为每segment分配String，本轮RED→GREEN改为borrowed slice。其后block layout仍对每physical line多次`rfind`前缀、扫描全部paragraph overrides、测indent/list prefix；`slice_runs`复制所有相交run text/style/inline/link。glyph wrap对增长current+grapheme反复fit query，风险与Text03既有O(G²)根因一致。

Text03应让single shaped paragraph与排序paragraph/run spans共同产出line range/constraints，使用monotonic cursors，不按line从头扫描或复制source/run DTO。

## PERF-MVP-301：rich/BiDi adapter多层owned projection

`UiParsedText`在完整`RichParseResult`之外再次clone stripped text、paragraphs并为每run创建substring String；link click重新parse全文。visual order先创建per-grapheme owned token与cluster，随后clone cluster、创建fragment，再物化最终visual text/runs，中间多层String/Vec仅供一次布局。

Text07既有compiled artifact handoff需扩展到interaction与UI line adapter，Text02/Text03 shaped cluster order直接提供visual/source index。single command generation parse≤1，link hit parse=0，UI adapter不得创建per-run substring和per-grapheme owned String。

## PERF-MVP-302：rich table每cell双layout与全文切片

每个table cell先用nowrap执行preferred layout，再按resolved track执行actual layout；每次cell layout都扫描完整runs/paragraphs/tables，clone局部text、style/inline/link、paragraph与nested table，local RichParseResult/UiParsedText又双持text/paragraphs。复杂度随cell数乘整个document metadata，而非cell-local content。

Text07应在compiled artifact建立cell→source/run/paragraph/nested-table range索引；Text03 intrinsic sizing与final placement共享一次cell shape/layout artifact。table span/track排序可保留确定性，但不得以每cell复制全文换取方便。

## PERF-MVP-303：fit与ellipsis固定多轮完整shape

ShrinkToFit/Clamp miss执行natural measure加固定8轮binary search；ellipsis另建grapheme ranges、candidate text/advances，rich inline每line重复测ellipsis。稳定cache可以遮蔽后续帧，但首次布局、resize和style generation仍形成明显burst。

Text03/Text09应复用single shaped paragraph的cluster advances与有界scale decision cache，记录backend calls和fit iterations。不得以降低字体质量或固定advance规避真实fallback/hinting/kerning语义。

## 责任计划与验收

EditorUI03收到layout artifact/cache-hit/caret interaction handoff；Text03既有prefix/wrap handoff已补paragraph/fit证据；Text07既有compiled rich artifact handoff已补visual/table证据，并联动Text02/09。以1/10k/100k chars、1/100/10k lines、1/100/1k paragraphs/runs/table cells、stable 300 frames与连续100k hit/caret/link操作记录source/layout/cluster/table ownership bytes、parse/shape/layout/backend calls、line/run/full-document visits、cache hits、fit iterations及CPU p50/p95/p99。current-source Cargo、F4文本编辑/IME产品trace、像素与规模counter完成前继续留`pending.md`。
