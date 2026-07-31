---
related_code:
  - zircon_runtime/src/text/font
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/shaping/horizontal/backend.rs
  - zircon_runtime/src/text/shaping/vertical.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheCompositeFont.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp
  - dev/bevy/crates/bevy_text/src/parley_context.rs
tests:
  - paired handle source-level RED to GREEN guard passed
  - Rust 1.94.1 module-wrapper handle registry tests 4/4 passed
  - rustfmt check and scoped diff check passed
  - current-source Windows zircon_runtime font tests pending
  - multilingual fallback and 1/100/10000 glyph counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text font逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/font`当前源32/32个Rust文件已逐文件阅读，覆盖asset/file/system source注册、descriptor/coverage/matching/composite fallback、database与shared generation、face/instance handle、variation、vertical/decoration metrics、manifest及全部测试。`database.rs`、`database/tests.rs`及其新拆子文件属于同时进行的Text01改动，本轮只读并保留其当前内容。

## PERF-MVP-246：每字形全局句柄锁

`project_shape_result`原先对每个glyph分别调用face与instance注册，`detailed_glyph`又分别解析两类handle；每个调用都进入同一进程级`Mutex<FontHandleRegistry>`，解析还各自读取shared generation。因此一次shape→neutral DTO→internal DTO往返最多产生4次全局锁/glyph，和PERF-MVP-232的双DTO路径叠加后会把并行shaping结果重新串行化。

本轮先加入paired handle roundtrip测试，再新增成对注册/解析API，并把service与layout session改为每阶段一次锁，完成静态RED→GREEN、Rust 1.94.1 module-wrapper 4/4、rustfmt与diff检查。该止损只把主路径4次降到2次/glyph；SDF miss仍有分离解析，稳定文本仍有每字形全局锁。Text09需把handle投影提升为每shape/run或generation批次，稳定face/instance使用immutable generation table或局部unique-pair cache，使锁次数按unique face/instance而不是glyph数增长。

## PERF-MVP-247：fallback cluster重复构建候选

主face缺字时，每个cluster都重新clone composite/query/default fallback families；`dedupe_families`对已有结果反复重新trim/lowercase并线性比较，随后每个family clone全部face IDs、排序match score、按首codepoint过滤，最终再用`Vec::contains`二次去重。混合CJK/emoji/combining文本的缺字cluster会重复相同family与face工作，复杂度包含O(F²)字符串规范化、各family O(C log C)排序与O(K²)候选去重；现有64项primary match cache不覆盖fallback candidate chain。

Text01需在font generation下预编译normalized family identity、排序face lists、composite culture/script/range索引与有界fallback resolution cache。cluster必须仍以all-codepoint coverage确认，不能把当前first-codepoint预筛误当最终命中；记录family normalization、candidate sort、coverage probe与cache hit/miss，避免用只测ASCII的夹具验收。

## 已有根因回链

`vertical_glyph_advance_px`每glyph重新`ttf_parser::Face::parse`；horizontal RustyBuzz与SDF路径调用`effective_instance_variations`时也会重新parse face、collect axes并规范化variation。该证据补强PERF-MVP-240/235，交接Text01建立generation-owned parsed face/axis/metrics，不另建重复性能编号。shared database snapshot的字体bytes为`Arc`浅拷贝，但会clone faces/index/instance/fontdb metadata；当前主要发生在renderer/thread-local cache构建或generation变化，不认定为稳定帧热点。每个新UI font asset仍同步重建完整`FontSystem`，与首次系统字体发现一起回链PERF-MVP-158。

## 参考引擎结论

UE Slate的composite font cache预先构建并排序character ranges，用binary search选择typeface，并长期保存character-list、font face与shaped glyph face data；generation/culture变化显式flush。Bevy把Parley `FontContext`/`LayoutContext`作为长期resource，而不是每cluster重建字体上下文。Zircon应采用同样的generation-owned索引与face metadata owner，同时保持自己的font reload与ABI handle语义。

## 责任计划与验收

Text09收到`failure-2026-07-18-font-handle-per-glyph-global-lock.md`；Text01收到`failure-2026-07-18-font-fallback-candidate-rebuild.md`和`failure-2026-07-18-font-face-metadata-reparse.md`。需用1/100/10k glyph、1/8/64 fallback families、Latin+CJK+emoji+combining+RTL/vertical记录global lock、normalization、sort、coverage、Face parse、axis scan、owned bytes及p50/p95；font generation变化精确失效。current-source Cargo与产品workbench/Console trace完成前，font 32/32继续留在`pending.md`。
