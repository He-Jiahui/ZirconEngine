---
related_code:
  - zircon_runtime/src/text/shaping
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/font/database.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/slint/internal/core/textlayout/shaping.rs
  - dev/bevy/crates/bevy_text/src/pipeline.rs
tests:
  - line-break/script/line-start source and behavior guards authored
  - current-source Windows zircon_runtime shaping tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text shaping逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/shaping/**`当前源18/18个Rust文件、3,660行已逐文件阅读，覆盖BIDI、cosmic/font-system cache、fallback spans、horizontal/vertical RustyBuzz backend与projection、UAX14、normalization、script segmentation、provider root及测试。`cosmic/font_system_cache.rs`与`fallback_spans.rs`含Text01活跃owner改动，本轮只读；直接修改限定在独占的`cosmic.rs`、`line_break.rs`与`script_segment.rs`。

## PERF-MVP-234：有序元数据与projection二次扫描

原`LineBreakOpportunityMap::flags_for_cluster`为每个glyph fold全部break opportunities；`script_for_range`为每个glyph从头find script segment；`line_visual_start`为每个layout line从文本开头split扫描。长Console/CJK文本分别形成O(G×B)、O(G×S)与O(T×L)。本轮先以源码门禁确认三处RED，再把前两项改为`partition_point`窗口/索引、line starts改为每段shape一次预计算；行为测试覆盖soft/mandatory break、Latin/Arabic边界与trailing newline，源码门禁防回归，rustfmt/diff通过。

horizontal/vertical `project_backend_run`仍为每个backend glyph线性扫描boundaries找cluster end，再filter全部source glyph并collect临时Vec以算overlap；segment内G glyph最坏O(G²)并有G个临时分配。BIDI `line_order`还分别为visual indices与logical levels计算两次reordered levels；fallback spans在primary非全覆盖时按grapheme收集codepoint Vec并先分配family String再尝试合并。这些归Text02继续线性化。

## PERF-MVP-235：重复shape与backend状态重建

neutral service先用`BidiParagraph::new`解析base direction，随后cosmic `shape_text`再次构建完整BidiInfo。cosmic Buffer执行`Shaping::Advanced`后，horizontal language/variable segments通过RustyBuzz再次shape；vertical upright segments也再次shape。每段重新获取face bytes、构造RustyBuzz Face、variations/features Vec与UnicodeBuffer，再把glyph投影回第一次结果。

thread-local locale FontSystem cache让每个shaping worker各持database与最多4个FontSystem；generation变化在下一次caller shape中同步clone backend DB并重建全部locale entries。该模式符合“worker-owned FontSystem”的线程安全方向，但缺少single-shape、共享face data与总内存/refresh预算。

## 参考引擎结论

Slint `ShapeBuffer`先itemize text runs，再对每run调用一次`font.shape_text`并直接扩展同一个glyph Vec；Bevy/Parley把build/break/layout保存在computed layout并复用capacity。UE `FShapedGlyphEntry`/sequence同样围绕一次run shaping与最终mapping组织。Zircon需要保留locl/variation/vertical精度，但应把参数送进唯一backend，不应先Advanced shape再整段二次shape。

## 责任计划与验收

Text02收到`failure-2026-07-18-shaping-quadratic-metadata-and-backend-projection.md`，并需联动Text01 shared face bytes与Text09 worker/cache预算。门禁覆盖1/100/1k/10k Latin/CJK/RTL/vertical glyph：metadata/projection visits近O(G)、temp overlap Vec=0、backend shape/Bidi analysis各1、face bytes copy=0、generation refresh不阻塞每个worker。current-source Cargo与产品workbench/Console trace完成前，本目录保持pending。
