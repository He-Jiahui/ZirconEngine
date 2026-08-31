---
title: Runtime Text Shaping、Unicode、BiDi、Script Run、Cluster、Line Break、Wrap、Layout 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime81
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/framework/text
  - zircon_runtime/src/text/shaping
  - zircon_runtime/src/text/hard_line.rs
  - zircon_runtime/src/text/language.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/text/layout
  - zircon_runtime/src/text/cache/index.rs
  - zircon_runtime/src/text/cache/mod.rs
  - zircon_runtime/src/text/cache/hard_line_index.rs
  - zircon_runtime/src/text/cache/measure_cache.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/parallel
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/glyph_artifact.rs
  - zircon_runtime/src/ui/text
  - zircon_runtime/src/ui/surface/render/text_prewarm
  - zircon_runtime/src/ui/surface/text_artifact.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/runtime_lines.rs
tests:
  - zircon_runtime/src/text/shaping/tests.rs
  - zircon_runtime/src/text/layout/line_break/tests.rs
  - zircon_runtime/src/text/layout/line_break/boundary_correction/tests.rs
  - zircon_runtime/src/text/layout/rich_advance_index/tests.rs
  - zircon_runtime/src/ui/text/layout_engine/tests
  - zircon_runtime/src/ui/surface/render/text_prewarm/tests.rs
  - zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/zircon_runtime/text/02/failure-2026-07-18-shaped-run-per-glyph-string-and-text-duplication.md
  - docs/plans/zircon_runtime/text/02/failure-2026-07-18-shaping-quadratic-metadata-and-backend-projection.md
  - docs/plans/zircon_runtime/text/03/failure-2026-07-18-layout-prefix-and-grapheme-remeasurement.md
  - docs/plans/zircon_runtime/text/09/failure-2026-07-18-text-cache-linear-lookup-and-eviction.md
  - docs/plans/zircon_runtime/text/09/failure-2026-07-18-text-layout-roundtrip-and-generation-retry.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheHarfBuzz.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheHarfBuzz.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/TextLayout.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/ShapedTextCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/ICULineBreakIterator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/ICUWordBreakIterator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/BreakIterator.h
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/godot/modules/text_server_adv/text_server_adv.h
  - dev/godot/modules/text_server_adv/script_iterator.cpp
  - dev/godot/modules/text_server_adv/script_iterator.h
  - dev/godot/servers/text/text_server.cpp
  - dev/godot/servers/text/text_server.h
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/parley_context.rs
  - dev/bevy/crates/bevy_text/src/error.rs
  - dev/bevy/crates/bevy_text/src/text.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/run.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/textwrapper.rs
  - dev/Fyrox/fyrox-ui/src/font/mod.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: m0_validation_pending
source_recheck_required: true
implementation_record: docs/plans/optimize/zircon_runtime/81/2026-08-21-source-line-shaping-work-budget.md
---

```zircon-workflow
{"schema":1,"workflow_id":"runtime81-text-shaping-engineering","goal":"Converge Runtime text shaping from semantic correctness repair through bounded asynchronous product execution.","milestones":[{"id":"M1","title":"M0 source-line semantics and typed work-budget evidence","depends_on":[]}]}
```

# 81 · Runtime Text Shaping、Unicode、BiDi、Script Run、Cluster、Line Break、Wrap、Layout 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon 当前文本链不是逐字符宽度相加的临时实现。横排 direct path 已使用 RustyBuzz，竖排支持 TTB/BTT、`vert/vrt2`、vertical metrics 与 `unicode-vo`；paragraph BiDi 基于 `unicode-bidi`，line order执行L1/L2；grapheme来自`unicode-segmentation`，基础软断点来自`unicode-linebreak`。fallback span、script itemization、hard-line streaming、prefix advance、bounded boundary correction、resolved glyph artifact、viewport hard-line window与多类cache都是真实底座，应在重构时保留。

但这些算法尚未组成一个可证明正确、可取消、可增量、可发布状态的工程级文本产品。normalization被明确禁用；language只做trim/lowercase；script run没有Script_Extensions与paired-bracket继承，emoji靠手写码位范围；line breaking和Arabic justification又叠加手写标点、假名与joining表。cluster advance把ligature宽度按grapheme数均分，固定8-grapheme边界窗口被当作通用修正；UI随后再次按grapheme重排、镜像并构造公开shaped DTO，权威backend cluster并未贯穿layout、caret和render contract。

本报告冻结时存在一项新的P0：`TEXT_SHAPING_RUN_MAX_BYTES`定义与`text/mod.rs` re-export已删除，7个测试文件仍有24处引用。M0实现现已将这24处全部迁移到typed `TextShapingWorkBudget`，并保留一条source hard line；该临时合同当前服务于scale fixture与后续scheduler接口，尚未接入production defer，且不允许切割script run、cluster或伪造layout line。静态解析门已恢复，Cargo行为与性能批量验证仍待协调器串行执行，详见implementation record。

Runtime11B P0-2的synthetic shaping fallback仍开放，本轮不重复计数。本报告新增 **1项P0、48项Runtime81独有P1、12项P2与48项资格门**。目标是建立`UnicodeDataSnapshot -> TextAnalysisArtifact -> ShapingOutcome -> GlyphClusterMap -> ParagraphLayoutArtifact -> DocumentLayoutSession -> Product Text Receipt`，而不是继续在UI层补标点表、猜advance或扩大常量。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime81责任 | 不重复登记 |
|---|---|---|---|
| Font asset/database/fallback collection | Runtime11B / Runtime80 | shaping输入的resolved face、language/script与completeness消费 | font blob、system provider、collection generation、color/variation导入 |
| Text shaping/Unicode/BiDi/line layout | Runtime81 | analysis、itemization、cluster、line break、wrap、justification、document layout与typed outcome | editing transaction、IME、secure input由Runtime82承接 |
| Glyph atlas/SDF/GPU submit | Runtime11C / Runtime79 | canonical glyph artifact与layout/render交接 | atlas page、upload、batch、clip、WGPU submit |
| UI/Editor product | Runtime81定义中立layout artifact；Editor后续专题消费 | viewport、caret geometry所需cluster contract与product receipt | Editor authoring、localization工具和font toolkit |

`TextDirection`、`TextRange`、typed shape/layout status和中立cluster DTO应留在`core::framework::text`；Unicode data、analysis、shaper、paragraph/document session归`zircon_runtime::text`。`zircon_runtime_interface`不得根据resolved line重新“猜”glyph；Editor和renderer只能消费generation-qualified artifact或明确的typed fallback。

### 2.2 Zircon物理冻结

指纹算法与Runtime80一致：相对路径排序；逐文件SHA-256；以`path<TAB>lowercase-hash`用LF连接且末尾无LF，再对UTF-8清单做SHA-256。production classifier排除路径段`/tests/`和叶文件`tests.rs`，保留inline tests。

物理冻结集合为`related_code`展开后的Rust文件，加上`zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs`；frontmatter中的`zircon_runtime_interface/src/tests/render_contracts.rs`只作跨合同旁证，不进入本表统计，其余聚焦测试已被上述目录覆盖。

| 范围 | 文件 / 物理行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Shaping core | 41 / 6,398 / 206,598 / 86 | framework DTO/error、normalization、language、BiDi、script/itemize、direct/cosmic/vertical、hard line |
| Layout core | 44 / 11,261 / 372,030 / 168 | advance index、measure、line break、wrap/overflow/tab/align、cache、parallel与session |
| UI text | 64 / 13,175 / 440,935 / 194 | request、viewport、geometry、visual order、plain/rich/vertical layout与tests |
| Product contract/consumer | 13 / 5,987 / 208,111 / 45 | glyph artifact、prewarm、interface shaped/layout DTO、renderer handoff、Editor runtime lines |
| 去重合计 | **162 / 36,821 / 1,227,674 / 493** | 全集fingerprint `2759b66d8b81aa7990a3195f486db466d7d40c4ea0a2f8a9414b9a5de898292c` |
| Production | **114 / 25,861 / 859,391 / 141** | production fingerprint `8e65b085208d7d575d17d50b6beec41205e072d0949188a2bf2da461ec0fabd2` |

冻结时15个范围内路径dirty，包括`hard_line.rs`、line-break module、horizontal/vertical direct shaper、shaping tests、UI wrapping/paragraph/candidate line与Editor runtime lines。本文没有修改任何源码；实施前必须重取两个指纹并先解决P0，故`source_recheck_required: true`。

依赖快照为RustyBuzz 0.20.1、Cosmic Text 0.18.2、`unicode-bidi` 0.3.18、mirroring 0.4.0、normalization 0.1.24、linebreak 0.1.5、script 0.5.8、segmentation lock 1.13.3、`unicode-vo` 0.1.0。当前没有把它们归并成一个可查询的Unicode data/version receipt。

### 2.3 参考物理冻结

| 参考 | 文件 / 物理行 / 非空行 / bytes | 采用的判断基线 |
|---|---:|---|
| Unreal | 12 / 8,751 / 7,350 / 345,393 | Slate shaper、HarfBuzz face/cache、BiDi block、line model/view、lazy generation、ICU line/word iterator |
| Godot | 6 / 13,046 / 11,314 / 511,990 | ICU/HarfBuzz shaped text、script/emoji/bracket run、locale line break、kashida、tab、overrun、caret/hit API |
| Bevy | 4 / 2,487 / 2,218 / 91,683 | Parley Font/Layout context、retained layout、typed error、word-break/overflow-wrap/font feature/variation投影 |
| Fyrox | 4 / 2,693 / 2,463 / 93,451 | 简单formatted text/wrap/font fallback基线，用于识别最低产品闭环，不作上限 |
| 去重合计 | **26 / 26,977 / 23,325 / 1,042,517** | fingerprint `4dfba95cf95fe8ab01136c84f85c15da77f4af72afd53420f19c5bc7a398cd18` |

本地Unity Graphics只含TextMesh Pro shader/sample资源，没有TextCore/TMP或Unity主文本引擎的analysis/shaping/layout源码；本文不据此猜测Unity闭源行为，也不把shader样例计入参考冻结。

### 2.4 证据限制

- 本轮是current-source静态review；按用户要求未修改Rust/Cargo/assets，也未运行Cargo、Editor、真实窗口或GPU。
- 未运行Unicode Bidi/LineBreak/Grapheme/Normalization官方corpus、复杂脚本golden、font fuzz、长文档soak或跨平台输入法。
- 未在相同font、locale、文本、像素质量和warm/cold状态下与Unreal做CPU/RSS/p95/p99基准；因此不宣称性能优于Unreal。
- 当前MVP 00仍在执行，本轮只建立差距与资格合同，不越过baseline实施非MVP源码重构。

## 3. 当前链与可保留底座

### 3.1 当前实际链

```text
TextStyle + source
  -> disabled normalization view
  -> hard-line segmentation
  -> grapheme + fallback-face span + BiDi level + script itemization
  -> RustyBuzz horizontal/vertical direct, else Cosmic, else synthetic fallback
  -> ShapedGlyphRun / ShapedTextLine
  -> grapheme advance projection + UAX14/custom break candidates
  -> CandidateLine + visual-order UI materialization
  -> UiResolvedTextLayout + opaque resolved glyph artifact
  -> interface UiShapedText reconstruction / renderer artifact-or-fallback route
```

### 3.2 应保留成果

- RustyBuzz direct path保存font face/instance、glyph ID、source range、offset/advance、direction、script、cluster flags；不要退回逐字符measure。
- `unicode-bidi` paragraph levels与line L1/L2、mirroring helper、逻辑glyph顺序保留是正确方向。
- 竖排真实使用TTB/BTT、`vert/vrt2`、vertical metrics和`unicode-vo`，应补capability/provenance而非删掉。
- `unicode-linebreak`基础UAX14机会、CR/LF/VT/FF/NEL/LS/PS hard-line支持、streaming/window hard-line API可保留。
- boundary correction将work限制在固定窗口、Arabic probe限制32项/5次、cache有局部capacity/byte cap，说明已有预算意识；问题是正确性证明、owner和统一budget不足。
- resolved glyph artifact能把canonical glyph slice、font generation与layout line绑定；应让所有product contract消费它，而不是另造DTO真值。

## 4. P0与继承阻断

### 4.1 新增P0

| ID | 阻断 | Current-source证据 | 关闭条件 |
|---|---|---|---|
| RTS-P0-001 | 长行语义M0已实现、managed验证待完成；`TextShapingWorkBudget` 现已在 retained session cache miss 与 parallel pending job 发布完整请求的 inline/oversized-synchronous 字节回执，但不执行切分或 defer | 旧实现把超长logical line伪装成多行；M0已删除该语义，当前 64 KiB 以上请求仍保留完整 source/context 同步进入 backend。继续保持开放：交付 typed deferred/cancelled work unit、保留 cluster/context map，并使长Arabic/Indic/emoji/ligature语义与budget/cancel测试通过 |

### 4.2 继承P0

| 既有阻断 | Current-source状态 | 唯一owner |
|---|---|---|
| 真实shaping全失败仍发布synthetic glyph与猜测advance | 2026-08-26 current-source已修正：canonical service返回typed outcome，无font为`FontUnavailable`、backend glyph无实际face为`FallbackExhausted`，无face glyph不会请求rasterization；实现静态完成、managed验证待定 | Runtime11B P0-2为唯一owner；Runtime81保持typed shaping/budget/cluster后续条件，不重复计数 |

## 5. P1差距

### 5.1 Unicode、language、script、emoji与BiDi

| ID | 差距 | 工程化要求 |
|---|---|---|
| RTS-P1-001 | V1 已明确为 `ShapingTextView::source_preserving`：原始 UTF-8 与 byte offset 保持不变，组合/分解等价序列不共享 shaped-cache identity；NFC/NFD 仍未实现 | V1 policy 为 `implementation_complete / managed_validation_pending`。V2 启用 canonical normalization 前必须提供版本化 original-normalized 双向 source map、backend parity golden 与安全上限 |
| RTS-P1-002 | 2026-08-26 已以 Runtime Text 单一 ICU4X owner 完成 syntax validation、canonical casing、请求级一次结构化解析、显式 script projection 与 cache exact identity；likely script/region、版本化数据和 fallback decision receipt 仍缺失 | `canonical_tag_and_explicit_script_projection_implemented / likely_subtag_receipt_open / static_checks_complete / managed_validation_pending`；继续扩展同一 locale owner，禁止 cache/UI/interface/itemizer 另建策略 |
| RTS-P1-003 | 2026-08-26 已建立编译期 `UnicodeDataSnapshot`：十二个 provider role 的独立实现/数据版本形成 request-bound 16-byte identity，并进入 script/emoji/word/WordSmart analysis、Bidi paragraph/line、line-break map、shaped artifact/cache 与 fallback diagnostic；JoiningType 已作为独立 capability 纳入，当前 schema/generation 为 4，dynamic generation、完整 layout/document artifact 仍开放 | `compiled_snapshot_identity_implemented / full_artifact_and_hot_update_open / static_checks_complete / managed_validation_pending`；继续沿同一 owner 完成 lease/invalidation，禁止以单一假版本覆盖 mixed data |
| RTS-P1-004 | 2026-08-26 已硬切到定长 `ScriptExtension` 候选交集；显式 BCP-47 script subtag 由 canonical request typed identity 提供，itemizer 不再拆分字符串，前一 resolved neighbor 只做多候选消歧 | `non_validation_implementation_complete / static_checks_complete / managed_validation_pending`；likely-subtag 仍归 RTS-P1-002，官方 corpus 仍开放 |
| RTS-P1-005 | 2026-08-26 已使用 `unicode-bidi` 完整 paired-bracket 数据和摊还线性 opening stack；leading/nested/opening-context 回归已编码 | `non_validation_implementation_complete / static_checks_complete / managed_validation_pending`；未通过 corpus 前不关闭 RTS-GATE-004 |
| RTS-P1-006 | 2026-08-26 已删除宽码位范围并由 paragraph analysis 使用 Unicode 17 emoji property 区分默认 presentation、VS15/VS16 与 keycap；相邻 emoji range 合并后供 fallback/itemize/backend 二分查询 | `mvp_property_presentation_implemented / full_rgi_open / static_checks_complete / managed_validation_pending`；完整 RGI ZWJ/tag/RI/modifier sequence、版本化 snapshot 与官方 corpus 仍开放 |
| RTS-P1-007 | 2026-08-26 fallback 与 shaping 共用同一 paragraph script analysis；`Other(FontScriptTag)` 只接受 canonical-form packed ISO15924，受检反序列化拒绝任意整数，未分配字符进入 typed `Unknown`，码点伪 script 源扫描为 0 | `non_validation_implementation_complete / static_checks_complete / managed_validation_pending`；historic-script/Indic corpus 与实际 cache cardinality 数据仍开放 |
| RTS-P1-008 | 2026-08-26 resolver 的整 grapheme cmap coverage 与 PartialCoverage/LastResort/DepthLimit receipt 已贯穿 shaping span，同 face 的 complete/missing 决策不再合并 | `grapheme_coverage_receipt_implemented / full_sequence_coverage_open / static_checks_complete / managed_validation_pending`；RGI/Indic shaping sequence 与 backend glyph coverage 仍开放 |
| RTS-P1-009 | 2026-08-27 missing primary 已从 neutral `FontUnavailable` 升级为 request-owned `FontPrimaryUnavailable` receipt；generation retry/stale cache/stale worker 统一发布 `FontGenerationChanged` deferred receipt。同步 resolver 的 request envelope 现聚合 resolution/candidate cache hit-miss、真实 coverage probe、candidate visit/reject 与最终选择分类，generation 重试丢弃的 attempt 也保留 | `primary_and_generation_capability_causes_implemented / bounded_candidate_decision_receipt_implemented / deferred_terminal_split_implemented / full_capability_trace_open / static_checks_complete / managed_validation_pending`；exact candidate ordinal/face trace、pending dependency、policy reject与backend capability组合仍开放 |
| RTS-P1-010 | UAX#9依赖存在，但没有BidiTest/isolates/bracket/overflow-level官方corpus receipt | 对固定Unicode版本跑官方conformance和混排产品golden，保存fail range与level trace |
| RTS-P1-011 | 2026-08-26重审修正旧前提：shaping必须保留logical glyph/level，最终L1/L2归post-wrap line owner；Plain/Horizontal的`CanonicalPhysicalLineFragment`现保存grapheme级`BidiLineOrder`且UI优先消费，不再读取paragraph重分析 | 部分实现、静态确认、managed待验收；virtual display route仍显式独立，rich/vertical/viewport fallback与caret/hit/accessibility尚未共享同一canonical visual-cluster artifact，继续开放 |
| RTS-P1-012 | 2026-08-27 Plain source-congruent physical line由canonical fragment order约束；`source_subrange`只对等长映射精确切分，否则保留完整source range。neutral DTO 的advance cardinality/finite invariant失效时不再等分行宽；runtime hit-test 也已删除默认style临时重塑，严格source route复用同一cluster index，否则只选整行端点 | `plain_exact_projection_and_invalid_dto_endpoint_geometry_implemented / runtime_default_style_reshape_removed / proportional_geometry_guess_removed / rich_virtual_backend_cluster_map_open / static_checks_complete / managed_validation_pending`；继续贯通rich/virtual backend cluster/source map，非同构路径不得猜测内部range或advance |

2026-08-24 BiDi invariant implementation/performance-plan update：当前 UI 仍未删除其 adapter-level visual projection，故 `RTS-P1-011/012` 的“唯一 canonical visual-cluster artifact”目标保持 open；但错误契约已从 `text/shaping/bidi.rs` 贯穿 direct、Cosmic 与 UI layout publication。所有无效 resolved/line/glyph range、paragraph mismatch、缺 level、projection/advance cardinality 均成为 typed `BidiInvariantError`，再映射为 `TextLayoutError::BidiInvariant` 或 overflow-clipped 空安全 layout，不能回退 logical order 或 base level。范围检查是 O(1)，不改变正常 UAX#9 L1/L2 重排的复杂度，也不新增分析集合、shape call 或 renderer pass。性能验证计划只使用现有 profile counters/scopes，在相同字体、fallback、cache、device 与 power policy 下由 coordinator 采集 cold/warm 1/100/1k/10k grapheme 的 p50/p95/p99、RSS、backend calls 和 power trace，并与同配置 Unreal workload 比较；当前没有受管运行数据，不能声明收益、瓶颈消失、功耗或“最优规模”。

### 5.2 Shaping contract、cluster、budget、cache与vertical

| ID | 差距 | 工程化要求 |
|---|---|---|
| RTS-P1-013 | 2026-08-27 request receipt catalog 为14个稳定code：12个direct/backend cause加missing primary与generation changed；统一内部 `TextShapingFailure { error, receipt, request_diagnostics }` 与 transient completion 贯穿 stable generation、session、parallel与layout/UI 转发，公开 `TextLayoutService` 仍只投影neutral error | `request_outcome_receipt_implemented / bounded_font_resolution_work_retained / primary_and_generation_causes_retained / public_neutral_error_projection_retained / static_checks_complete / managed_validation_pending`；managed fault injection/并发验证前不关闭 gate |
| RTS-P1-014 | 2026-08-26 current-source已硬切为`TextShapingOutcome`；只有同一稳定font generation的`Ready`可进入cache，Failed/Deferred及跨generation Ready均不物化空run、不入cache；non-Ready envelope 现保留请求 receipt | managed fault/concurrency tests确认typed disposition、receipt retention 与零非Ready admission；global低基数report保持 diagnostics-only |
| RTS-P1-015 | 删除64 KiB source-line cap后，任意长unbroken input保持单一 semantic request；生产 owner 现记录实际 cache-miss/pending backend work 的完整输入字节与阈值分类 | 保持开放：source line与backend work unit分离；work unit有上下文重叠、cluster去重、语义证明和 typed defer/cancel，而不是在 64 KiB 截断 |
| RTS-P1-016 | 2026-08-26 已补 actual backend work 的 inline/oversized-synchronous request count、总输入字节与最大请求字节；cache hit、batch duplicate、invalid request不冒充backend work。仍没有glyph、CPU、内存、deadline、cancellation或partial result合同 | `production_input_byte_work_receipt_implemented / algorithm_unchanged / typed_defer_cancel_open / static_checks_complete / managed_profile_pending`；后续引入per-document/per-frame budget token和cooperative cancellation，超限返回typed partial/deferred |
| RTS-P1-017 | 2026-08-26 已在 horizontal/vertical direct cluster 头发布 RustyBuzz `unsafe_to_break` 的 `Safe/RequiresReshape`，Cosmic/旧 artifact 为 `Unknown`；receipt 现继续保留到 measured cluster 与 `GraphemeAdvanceIndex`，final-line 候选边界可按 `Safe/RequiresReshape/Unknown` 聚合观测 | `direct_break_safety_receipt_and_measurement_retention_implemented / monotonic_candidate_boundary_profile_implemented / final_line_reshape_and_full_coverage_open / static_checks_complete / managed_validation_pending`；不得直接清除 soft break，ligature caret/malformed/full coverage 仍开放 |
| RTS-P1-018 | 2026-08-26 current-source确认 horizontal/vertical direct 与 backend adapter 均返回 typed `Result`；face access/index/parse、empty glyph、source range、cluster offset/ordering与non-finite metric进入12-code receipt。只有 horizontal backend/font/glyph capability 可 alternate backend，vertical 与 source/BiDi/budget invariant fail closed | `typed_direct_backend_failure_receipt_implemented / policy_scoped_horizontal_alternate_backend / vertical_and_invariant_fail_closed / static_checks_complete / managed_fault_injection_pending`；RustyBuzz language optional projection只可能在空串失败，而canonical request已过滤空串并拒绝非法tag，不构成可达的静默fallback |
| RTS-P1-019 | vertical `TransformOrRotate` 仍以同 context 的 `vert/vrt2` enabled/disabled 输出差分判定 substitution provenance；2026-08-26 已补 comparison call/input-byte/output-glyph/changed-cluster 的 request-local 聚合回执，算法未改变 | `vertical_substitution_comparison_receipt_implemented / algorithm_unchanged / static_checks_complete / managed_profile_pending`；先以 Tr 1/100/1k/10k、31-sample 数据确认成本和命中率，再决定 backend trace、feature plan 或有预算的 cache，禁止无 provenance 删掉第二次 shape |
| RTS-P1-020 | 2026-08-26 cluster head 已发布统一 `VerticalGlyphDecision`：effective `vert/vrt2` set、Unicode orientation、substitution proof、rotation、selected face/instance 与 typed fallback reason；neutral `TextGlyph` 保留同一 basis | `vertical_cluster_decision_basis_implemented / direct_feature_set_and_substitution_provenance_retained / neutral_projection_preserved / compatibility_unknown_explicit / static_checks_complete / managed_validation_pending`；RustyBuzz 不公开具体 lookup tag，双 feature 同开时不得猜成单一 feature |
| RTS-P1-021 | 2026-08-26 M0-M2 非验收实现已落地：horizontal direct 完成可达 segments 并发布 source-sorted holes；一次 whole-request Cosmic candidate 经 identity/topology/cluster/range 资格后只填 holes，任何不相容保留完整 candidate；hybrid run 重建 selected-face metrics 并保留 alternate ranges/首因 receipt | `direct_partial_attempt_implemented / source_ordered_hybrid_composition_implemented / selected_face_metric_rebuild_implemented / hybrid_artifact_receipt_and_profile_implemented / fail_closed_whole_candidate_retained / static_checks_complete / managed_validation_pending`；当前仍支付一次完整 Cosmic candidate，managed fault/profile/power/WGPU/PNG 前不声明性能改善或关闭瓶颈 |
| RTS-P1-022 | `ShapedGlyphRun`仍持 exact `Arc<str>`；2026-08-26 已确认 parallel hard-line split 会为每段物化 owner，普通同步 run 无 owner 时也会分配；Unreal shaped sequence 只持 range/index map，由外部文本 owner 保寿命 | `source_lifetime_architecture_research_complete / unreal_external_text_owner_confirmed / source_materialization_and_batch_owner_instrumentation_implemented / algorithm_unchanged / static_checks_complete / managed_profile_pending`；先量化 materialization allocation 与 lease/unique-owner 比，再决定 document snapshot + range lease；SoA/cluster table 分开立项，不与寿命硬切捆绑 |
| RTS-P1-023 | 2026-08-26 current-source确认 shaped/rich/measure/physical-line cache与document revision hash均只用于进程内桶定位，并有完整key及exact source复核；离线`.zsdf`已经用BLAKE3+v1格式版本。真实缺陷是裸`u64 content/source_hash`混淆用途，不是临时哈希算法本身 | `ephemeral_cache_hash_type_implemented / stable_artifact_digest_type_implemented / default_hasher_isolated / sdf_v1_bytes_unchanged / static_checks_complete / managed_validation_pending`；cache统一不可序列化`EphemeralCacheHash`，artifact摘要统一`StableContentDigest`，格式/domain owner负责版本；不得把临时哈希写入artifact/replay，也不得为viewport反复全文稳定hash |
| RTS-P1-024 | 2026-08-27 shape request 已有profiling-only TLS聚合，分别记录Bidi、script/emoji与line-break构造次数、累计输入字节和纳秒；current-source可见horizontal direct转whole-alternate时line-break map构造两次，但尚未证明成本主导。rich advance/physical-line/document级analysis归因仍开放 | `paragraph_lifetime_architecture_review_complete / shape_request_analysis_profile_implemented / duplicate_construction_observable / retained_paragraph_artifact_open / algorithm_unchanged / static_checks_complete / managed_profile_pending`；先跑cold/warm、direct-success/alternate/hybrid/vertical、1/100/1k/10k grapheme与31样本，再决定hoist或retained artifact；dirty-range、source lease与SoA不得捆绑先行 |

2026-08-24 M3 typed-outcome architecture checkpoint：以 `zircon_runtime::text::shaping` 的 internal `TextShapingOutcome::{Ready, Deferred, Failed}` 收敛现有 `Result`/empty-run 混合语义；`Ready` 才能进入 `ShapedRunCache`，并且 artifact ownership继续为现有 `Arc<ShapedGlyphRun>`，故正常路径不需要额外 clone 或 allocation。该 MVP 不在 UI/renderer 新建错误策略，也不把 backend strings 或 raw source 放进 public DTO。Unreal Slate `FShapedGlyphSequence` 的 source-range/sequence ownership与Slint `ShapeBuffer -> glyph cluster -> layout` 的一次分析模型支持这一边界；Zircon 对 generation-deferred 采用显式 outcome 是针对本仓库 cache/retry 约束的有意差异。

2026-08-24 M3-M1 implementation record：`SharedTextLayoutSession` 与 `text::parallel::shape_pool` 已接入 outcome；cache admission 硬限定为 `Ready`，`Deferred`/`Failed` 返回当前调用 empty fallback 而不缓存，且 `text.shape_batch.failed` 将 stable failure 与 generation retry 分开计数。session/parallel non-Ready regression 已加入，限定 Rust 2024 formatting、tracked diff whitespace 和 cache admission source scan 已通过。direct provider、layout/UI publication 和 framework receipt 尚未迁移，因此 M3 整体仍为 `partially_implemented / managed_validation_pending`；Cargo、fault injection、stress profile、Bidi corpus和真实 WGPU 仍未执行。

2026-08-24 RTS-P1-017 backend-cluster audit：horizontal/vertical direct owner 已在 `restore_backend_cluster_logical_order` 前验证 backend order 单调性，随后恢复 logical cluster order；现有 `valid_backend_*` 继续拒绝空 output、非 UTF-8 boundary offset 与非有限 metric。其余“完整覆盖”不能简化为首 cluster 必为零、末 cluster 必为 `text.len()`：leading/trailing format controls、combining marks、missing glyph/tofu 与 future unsafe-to-break/ligature caret 都可能合法地不遵从这个简化。故状态为 `architecture_review_complete / implementation_deferred`；必须由一次性 paragraph analysis artifact 显式发布 source coverage、cluster boundary、unsafe-to-break 与 malformed-result receipt 后再收敛，不在 direct adapter 加会拒绝有效 Unicode shaping 的猜测断言。未运行 Cargo、fault injection 或 corpus。

2026-08-26 RTS-P1-017 unsafe-to-break 复核补充：本地 RustyBuzz 0.20.1 的 `GlyphInfo::unsafe_to_break()`
已可直接读取，语义是“在该 cluster 起点断开时两侧必须重新 shape 才能保持结果”，不是禁止该 UAX#14
break。当前 `ShapedGlyphClusterFlags` 没有 known/unknown provenance，Cosmic fallback 也不发布等价 flag；
Text03 又仍以固定 8-grapheme boundary correction 修总宽。因此禁止把 unsafe flag 简化为清除 `soft_break`，
那会拒绝合法换行。正确 hard cut 需先发布 direct-known/Cosmic-unknown 的 cluster safety receipt，再让 final-line
owner 对 unsafe break 精确 reshape 两侧或使用可证明的 context plan，并保存 source/cluster map。该项保持
`architecture_review_complete / implementation_deferred`，没有性能或正确性完成声明。

### 5.3 Line break、wrap、measure、ellipsis、tab与justification

| ID | 差距 | 工程化要求 |
|---|---|---|
| RTS-P1-025 | 2026-08-26 run-level `UnicodeDataSnapshotId` 已固定 `unicode-linebreak 0.1.5 / Unicode 15.0.0`；cluster-head 2-byte receipt 区分 UnicodeDefault 的 allowed/mandatory、mandatory control 与 unknown | `line_break_data_version_profile_and_opportunity_receipt_implemented / rule_number_and_locale_tailoring_open / static_checks_complete / managed_validation_pending`；当前 provider API 不公开UAX#14 rule number，不得伪造trace |
| RTS-P1-026 | 没有官方LineBreakTest/WordBreakTest/GraphemeBreakTest current receipt | 固定版本跑全corpus，并为引擎tailoring单列expected override与理由 |
| RTS-P1-027 | `kinsoku.rs`手写日文标点、小假名与pair表，不可版本化、配置或按locale选择 | 编译JLREQ/locale tailoring data，Editor可诊断规则来源、优先级和冲突 |
| RTS-P1-028 | 2026-08-26 `smart.rs` 已删除手写字符表，候选要求共享 UAX #29 word end，并由 snapshot-bound GeneralCategory style policy 选择/延伸尾标点；locale dictionary、tailoring/style matrix 与 corpus 仍缺失 | `word_smart_uax29_context_and_general_category_policy_implemented / locale_dictionary_and_tailoring_open / static_checks_complete / managed_validation_pending`；继续复用同一 word owner，不在 UI/layout 旁路重建字符表 |
| RTS-P1-029 | Thai/Lao/Khmer/Myanmar等没有dictionary break，自动hyphenation也不存在 | 接入可选locale dictionary/hyphenator provider，具备版本、license、budget与fallback |
| RTS-P1-030 | wrap只有有限枚举，缺`word-break`、`overflow-wrap`、`hyphens`、line-break strictness等正交策略 | 定义可组合style contract并映射到analysis/layout artifact，不靠新枚举爆炸 |
| RTS-P1-031 | 2026-08-26 typed decision、Plain、horizontal rich 与 VerticalRl rich zero-width virtual artifact 已实现；rich 复用 display-owned UAX#9 sidecar，并按 cluster source mapping 恢复 style span/virtual glyph run slice。typed `DiscretionaryHyphen` role 进入 rebuild identity，统一 virtual-source receipt 保留被消费 U+00AD，accessibility 保持原始 source value | `plain_horizontal_vertical_rich_canonical_virtual_artifact_implemented / typed_virtual_fragment_role_implemented / source_receipt_and_accessibility_preservation_confirmed / virtual_receipt_linear_capture_implemented / static_checks_complete / managed_validation_pending`；managed corpus/profile/power/WGPU/PNG 前不关闭 gate |
| RTS-P1-032 | 2026-08-26 EndWord 已删除 whitespace rollback，并消费 snapshot-bound UAX#29 owner；horizontal text-only rich marker 已在 logical order 生成，以非空 current-run style receipt 统一度量、artifact shaping、glyph slice 与 renderer presentation。single-gap replaced-source receipt 已覆盖 caret/hit-test/selection，歧义 fail closed；accessibility 保持原始 source value。horizontal compiled inline image/widget 已切为 external layout block，普通/ellipsis/inline-only 行不再生成 U+FFFC 字体 glyph，并与 marker 共享 visual geometry owner。ordinary styled VerticalRl、inline external block、U+2026 ellipsis 与 typed discretionary hyphen 已接入 canonical vertical provider；固定尺寸 direct-child widget 已完成静态实现，artifact 已使用 typed owner-local slot且 Surface current-tree binding 不驻留；desired-size retained session/incarnation lease 与 locale dictionary 仍缺失 | `boundary_selection_horizontal_and_vertical_ellipsis_omitted_inline_geometry_implemented / vertical_rich_soft_hyphen_virtual_artifact_implemented / fixed_size_direct_child_inline_widget_implemented / typed_owner_local_widget_slot_implemented / accessibility_source_preservation_confirmed / dynamic_locale_and_managed_validation_open / static_checks_complete`；managed profile/power/WGPU/PNG 前不关闭 gate |
| RTS-P1-033 | tab只按space width乘`tab_size`生成均匀stop，没有显式/右/中/小数tab或BiDi/vertical模型 | 定义paragraph tab-stop list、alignment与leader，按writing mode和direction布局 |
| RTS-P1-034 | 2026-08-27 shaping/renderer 共用 cluster iterator、typed `AtomicCluster` measurement receipt 已贯穿 wrap；canonical glyph artifact 已拥有 caret/hit/selection。缺失或 stale artifact 时，严格 source-congruent 的单一 LTR horizontal line 现以一次完整行 shape 建同一 advance index，caret/hit 只返回 cluster 两端，selection 任意相交扩为完整 cluster；editable pointer 使用 command text/style 进入该资格路径 | `shared_cluster_geometry_and_atomic_wrap_implemented / ltr_source_fallback_caret_hit_selection_implemented / canonical_artifact_fast_path_preserved / rich_bidi_vertical_cross_run_and_gdef_open / static_checks_complete / managed_validation_pending`；无 font caret 时只发布 atomic fallback，不伪装内部精确位置 |
| RTS-P1-035 | boundary correction 仍固定左右各 8 个 grapheme 且只修总宽；2026-08-26 已可对实际候选边界聚合 known-safe、requires-reshape 与 unknown 分布，但尚无 managed corpus/profile 数据 | 用 receipt 驱动 exact two-sided reshape 或可证明的 context plan；固定窗口只作优化且必须有 fallback，数据前不改算法 |
| RTS-P1-036 | 2026-08-26 Joining_Type、backend safety receipt 与行级低基数 probe instrumentation 已实现；UI仍最多插32个U+0640并最多重shape 5次，language/font API 与 31-sample profile 数据未完成 | `unicode_joining_type_and_backend_tatweel_safety_receipt_implemented / arabic_tatweel_probe_instrumentation_implemented / algorithm_unchanged / managed_profile_pending`；数据前不改探测算法 |

### 5.4 UI/product、viewport、cache、publication与qualification

| ID | 差距 | 工程化要求 |
|---|---|---|
| RTS-P1-037 | 2026-08-26 current-source中`UiShapedText::from_resolved_layout`与FNV glyph重造均为0命中；resolved layout只产paint runs/geometry并将public shape artifact标为`Unavailable` | managed contract/source gates继续锁定runtime canonical glyph artifact或typed unavailable |
| RTS-P1-038 | production runtime不构造`UiShapedGlyph`，也不从presentation style补font/atlas；renderer消费runtime-owned `TextGlyph` + resolved face artifact | managed raster regressions确认font/atlas identity只来自resolved face/glyph route |
| RTS-P1-039 | 2026-08-26 current-source已由完整physical-line集合累计`unclipped_measured_width`与总高度，clip仅筛paint lines；Plain/rich/vertical回归均覆盖intrinsic size不变 | 实现完成、静态确认、managed待验收；virtualized total extent另由RTS-P1-040跟踪 |
| RTS-P1-040 | viewport fast path仅plain、horizontal、nowrap、Clip、无preedit；wrapped/rich/vertical/editor仍全量 | paragraph/line virtualizer覆盖wrapped/rich/vertical/preedit，并保持总extent与anchor稳定 |
| RTS-P1-041 | preedit在完整String上replace并直接禁用viewport，长文档一次composition触发全量复制/布局 | document overlay/rope保存preedit span，仅重算受影响paragraph并保持IME source map |
| RTS-P1-042 | 2026-08-26 current-source已删除`text.len()*em`近似方形extent：rich horizontal使用无界轴，vertical仅以hard-line数量给cross axis有限上界且main axis无界 | 实现完成、静态确认、managed待验收；共享incremental measure artifact仍并入RTS-P1-043长期owner工作 |
| RTS-P1-043 | 2026-08-27 current-source校准：crate-private `text/document`已有piece storage、owner+revision、old/new byte dirty span与length delta；replace强制expected key并在stale/revision exhaustion时mutation前失败。现已新增separator-aware stable hard-line model：edit只物化带前后context的局部line envelope，保留未变prefix/suffix ID，split新增revision-qualified ID，merge保留左line ID，并发布old/new reanalyzed ordinal span；grapheme index仍通过完整snapshot全量重建 | `stable_hard_line_owner_and_edit_local_reanalysis_implemented / full_document_hard_line_rebuild_removed_from_edit / grapheme_index_full_rebuild_open / product_authority_unwired / paragraph_reflow_open / static_checks_complete / managed_validation_pending`；下一步让`DocumentLayoutSession`消费line ID/delta并记录reused/reanalyzed/reshaped/reflowed范围。当前`Vec` splice仍可能移动suffix metadata，1/100/1k/10k edit profile前不宣称最终复杂度或调整sequence storage |
| RTS-P1-044 | 2026-08-27 key仍为crate-private且production只由surface node id + qualified `text_layout_revision`创建；revision advance已删除wrap，`u64::MAX`为不可发布exhausted sentinel，两个extract点只通过`retained_text_layout_revision()`构造key。耗尽后pending owner仍完整布局并可走unretained viewport，只关闭retained document复用。frame/layout cache及两个retained owner仍拒绝同key异source，Plain parsed owner计数exact qualification/stale alias | `surface_revision_exhaustion_fail_closed / uncacheable_layout_fallback_preserved / static_checks_complete / managed_validation_pending`；仍保持开放：surface extraction物化新`String`，parsed owner资格仍需全文exact compare，尚无pointer-stable snapshot+revision receipt，也未证明长文档dirty/scroll为`O(visible)`。禁止每viewport全文稳定hash，应由surface持久immutable snapshot贯穿request/parsed/index |
| RTS-P1-045 | text measure/layout cache仍分别以4096/2048 entries作为行为上限；现已补齐source/DTO-owned heap的current/peak常驻下界回执，并覆盖update、LRU eviction与clear，但共享glyph artifact residency尚不能无重复归因 | 保持开放：先用managed规模profile标定source/DTO下界与artifact owner占用，再同时限制entry、source/value/artifact bytes、admission与eviction cost，按document owner回收；禁止在数据前拍定byte cap |
| RTS-P1-046 | Runtime UI full/incremental layout、递归 measure、surface extraction、post-overlap render resolve、component text-field/dialog measurement、artifact preparation现均沿同一`UiTextMeasureCache`/`SharedTextLayoutSession` owner；standalone layout/extract各自只建一个operation-local owner；font-generation变化由`UiSurface`在clean-frame return前观测并以retained owner全量重建，graphics只拒绝post-layout stale artifact且不再创建line/session overlay；Editor projection/materialization消费已发布`render_extract` | 非验收实现与静态确认完成、managed待验收；Editor retained paint的`layout_text`/`shape_text_line`位于既有2048项代际感知cache miss路径，`measure_text_size`另存于终端Host fallback；相对坐标cache/global mutex owner必须等待M0 trace后再改，显式one-shot budget、31样本profile/power、真实WGPU/PNG仍开放；禁止新增process-global cache和renderer-local artifact rebuild |
| RTS-P1-047 | 2026-08-26 current-source已要求prepare精确匹配source/effective style/font generation/writing mode/line snapshot并重建stale artifact，renderer最终消费前再次逐行校验；source/layout owner不相容、line/run越界或UTF-8切片非法返回`Failed(LayoutFailed)`。Plain command 与 rich run 均已发布 typed route：rich 以一次 `O(lines+runs)` 单调投影和 O(1) exact directory index 区分 artifact/intentional visual-only/missing/stale/incomplete，替代旧 `O(R^2)` 双重 `.find`；正常 rich profile 合同要求零 renderer reshape。Rejected 仅在 exact source-isomorphic 时可 fallback，非同构 rejected run 不发布猜测 batch | `plain_and_compiled_rich_typed_route_receipts_implemented / rich_linear_directory_publication_implemented / rich_nonisomorphic_rejection_fail_closed_implemented / static_checks_complete / managed_validation_pending`；真实 WGPU/profile/power/PNG 前不关闭本 ID 与 RTS-GATE-044 |
| RTS-P1-048 | 测试以结构断言、source-string guard和局部unit为主，无官方corpus、真实font golden、soak与同负载Unreal基准 | 建立hermetic correctness/performance qualification matrix并保存current-source fingerprint |

## 6. P2差距

| ID | 差距 | 收敛方向 |
|---|---|---|
| RTS-P2-001 | 2026-08-26 current-source 已无 `pending_common_start/end`；paragraph owner 使用 `ScriptExtension` 定长位集交集，Common/Inherited 通配语义直接表达 leading/intermediate/trailing 策略，并保留 paired-bracket 上下文 | current-source 校准与聚焦回归完成、生产算法无需改动、静态确认完成；managed text test 待验收 |
| RTS-P2-002 | shaping层hard-line容器已从`ShapedTextLine`硬切为`ShapedHardLine`；类型doc明确它位于wrap/ellipsis/placement之前，最终visual line仍归`CandidateLine`/`UiResolvedTextLine` | `shaped_hard_line_term_hard_cut_complete / serde_fields_unchanged / old_rust_symbol_zero / static_checks_complete / managed_validation_pending`；不保留alias/shim |
| RTS-P2-003 | provider/session入口已从`shape_*_line(_with_kerning)`硬切为`shape_*_range(_with_kerning)`，与实际`text + absolute source_range`及可能多hard-line输出一致；低层继续消费`BackendShapeRequest`，预算仍由session cache-miss/prewarm batch owner执行 | `shape_range_api_hard_cut_complete / request_and_budget_owners_preserved / old_rust_symbol_zero / algorithm_unchanged / static_checks_complete / managed_validation_pending`；typed defer/cancel仍归RTS-P0-001/P1-016，不以改名伪装完成 |
| RTS-P2-004 | 2026-08-26 已建立零额外存储的`EphemeralCacheHash`与`StableContentDigest`；document/cache hash不可序列化，SDF generation/offline identity使用稳定摘要且v1字节不变 | `typed_identity_boundary_implemented / static_checks_complete / managed_validation_pending`；后续新增artifact/replay必须由格式/domain owner显式版本化，禁止复用cache hash |
| RTS-P2-005 | 2026-08-26 已删除只含`UiSharedTextShaper`的`UiTextShaperStack`；public/provider/viewport/measure入口直接使用唯一shared adapter，真实direct/Cosmic能力组合仍归Runtime Text shaping owner | `empty_ui_shaper_stack_removed / sole_shared_adapter_preserved / source_guard_updated / static_checks_complete / managed_validation_pending`；未来只有真实capability-ordered backend set+receipt才能重新引入stack语义 |
| RTS-P2-006 | 2026-08-26 interface type erase已同时比较payload type与runtime owner semantic identity；rich identity覆盖source/format/parser generation/parsed projection，glyph identity覆盖font/style/line/rebuild state并排除可再生cache | 实现完成、静态确认、managed待验收；同一`Arc`保持O(1) fast path，`Eq`已删除 |
| RTS-P2-007 | 2026-08-26 layout cache已统计serializable line/run/advance owned bytes；prepare report新增Auto路由后最终native/SDF batch count、text bytes和advance bytes下界。`UiTextPaint`中间复制及跨边界序列化仍待量化 | `layout_dto_and_renderer_batch_residency_receipts_implemented / intermediate_paint_copy_open / algorithm_unchanged / static_checks_complete / managed_profile_pending`；profile证明主导前不硬切String/Arc/range/lease |
| RTS-P2-008 | 2026-08-27 current-source确认这些数值分属correctness context、单行算法工作量、cache residency与async completion，不能并入一个全局可变配置；现已由各实际owner发布budget snapshot，并在`text.runtime_budget.*`投影预算，page-shadow补resident/max/rejection收据 | `owner_local_budget_snapshots_implemented / runtime_budget_profile_projection_implemented / page_shadow_residency_receipt_implemented / algorithm_defaults_unchanged / static_checks_complete / managed_profile_pending`；managed profile前不调8/32/5/16/32 MiB默认值 |
| RTS-P2-009 | 多个测试通过`include_str!`检查源码未恢复旧实现，能防回退却不能证明行为 | source guard只作辅助；以black-box conformance、fault和scale测试为主 |
| RTS-P2-010 | perf测试含ignored/环境敏感路径，test attribute数量容易被误当覆盖率 | required qualification单独登记non-ignored run、机器/字体/阈值与结果artifact |
| RTS-P2-011 | 两个process-global fallback-report Mutex已删除；layout/shaping/backend-route诊断现由`SharedTextLayoutSession`逐帧持有，parallel prewarm完成项合并回同一owner，固定profile维度不含raw text/document id/dynamic label | `session_owned_diagnostics_implemented / process_global_report_mutexes_removed / fixed_backend_route_projection_implemented / parallel_prewarm_merge_implemented / static_checks_complete / managed_validation_pending`；具体document drill-down等待有界owner，详见`81/2026-08-27-session-owned-text-diagnostics.md` |
| RTS-P2-012 | 2026-08-26 `TextLayoutError`保留人类可读`Display`，同时新增稳定`diagnostic_code()`与`message_key()`；core仍不依赖Runtime Text实现类型 | `diagnostic_code_catalog_implemented / backend_neutral_boundary_preserved / focused_behavior_tests_complete / managed_validation_pending`；Editor/telemetry消费code/key，禁止解析Display |

## 7. 与参考引擎的差异归纳

| 维度 | Unreal/Godot/Bevy参考 | Zircon当前差异 |
|---|---|---|
| Analysis owner | Unreal line model/view与ShapedTextCache、Godot shaped RID、Bevy retained Parley Layout都有长期owner | Zircon大量one-shot session，paragraph analysis、shape、UI visual projection与renderer fallback可分叉 |
| Unicode/script | Unreal/Godot接ICU/HarfBuzz；Godot有paired-bracket stack与Unicode emoji property | Zircon 已有 request-bound 12-role compiled snapshot、paired-bracket stack、ScriptExtensions 与 emoji/GeneralCategory/JoiningType property；dynamic generation 与 official conformance receipt 仍缺失 |
| Line break/tailoring | Unreal ICU line/word iterator；Godot按locale创建ICU line iterator并支持strictness | Zircon UAX14/UAX29 base 与 WordSmart 已共享 versioned owner；硬编码 kinsoku、locale strictness、dictionary/hyphenator provider 仍缺失 |
| Cluster/caret | Godot shaped text直接提供carets、selection、grapheme bounds与hit API | Zircon 内部 measurement/wrap/artifact 已共享 backend cluster owner 和 atomic fallback；public UI/interface caret、跨-run continuation 与 font ligature caret 仍分叉 |
| Justification/tab/overflow | Godot暴露kashida/word-bound、tab stops、overrun flags与virtual glyph | Zircon Arabic 已有 Joining_Type 与 backend candidate safety receipt，但 font/language justification、probe profile、均匀tab与有限ellipsis/wrap仍未收敛 |
| Failure/product status | Bevy至少以`TextError`公开失败；Unreal/Godot owner能表达loading/shape state | Zircon canonical shaping已返回typed outcome，font unavailable/fallback exhausted不再伪造成glyph success；managed fault/concurrency qualification仍待补齐 |
| Performance | Unreal lazy line view与cache、Bevy retained layout可支撑增量owner | Zircon viewport只覆盖窄plain路径，wrapped/rich/vertical/preedit仍全量，长run又无统一budget |

Fyrox本地实现仍以简单letter/word wrap和fontdue逐字符glyph为主，能力远低于本报告目标；Zircon已有的RustyBuzz/BiDi/vertical底座实际更强，因此不得为了“对齐参考”退回Fyrox式模型。

## 8. 目标架构

### 8.1 `UnicodeDataSnapshot`

统一记录Unicode/CLDR/emoji/line-break/hyphenation provider版本和hash；提供BCP-47、ScriptExtensions、Bidi、Grapheme、Word、Line、Vertical与Joining属性。analysis artifact与cache key必须携该snapshot identity。

### 8.2 `TextAnalysisArtifact`

按immutable document snapshot和paragraph生成normalization/source map、grapheme/word/line candidates、BiDi levels、script/language runs、fallback requirements与unsafe-to-break边界。analysis只做一次，layout/UI/renderer不得重新推断。

### 8.3 `ShapingOutcome`与`GlyphClusterMap`

shape request返回Ready、PendingDependency、MissingGlyph、Unsupported、InvalidInput、BudgetExceeded、Cancelled或BackendFault。Ready保存真实face/instance/glyph、logical/visual cluster、source/normalized ranges、ligature caret、feature/vertical provenance和generation；非Ready不得伪装glyph run。

### 8.4 `ParagraphLayoutArtifact`

消费analysis与shape artifact，按style profile执行UAX14/locale/dictionary/hyphenation、tab、justification、ellipsis和wrap。visual line有stable ID、完整measured extent、paint clip与virtual glyph source map；clip不反向修改layout size。

### 8.5 `DocumentLayoutSession`

project/window/document owner持有snapshot revision、dirty paragraphs、analysis/shape/layout caches、viewport anchor、budget和diagnostics。edit delta只重算依赖范围；font/Unicode/style generation按依赖定点失效，old artifact由lease安全退休。

### 8.6 Product publication

Runtime Interface、renderer、Editor、accessibility和input geometry共享同一generation-qualified artifact。跨边界payload只发布stable schema与typed completeness；不可序列化的runtime lease保持opaque，但不能再用type-only Eq或synthetic DTO替代内容身份。

## 9. 分层重构里程碑

### M0：关闭当前P0并冻结语义

迁移24处旧常量引用；保持source hard line唯一；先定义临时可验证的shape budget接口与长run corpus，使所有受影响test target恢复可编译。不得恢复“超限即新行”。

### M1：Unicode/locale data snapshot

统一版本、BCP-47、normalization/source map、ScriptExtensions、emoji与joining data；引入official corpus harness和data receipt。

### M2：Canonical paragraph analysis

实现paragraph-owned grapheme/word/line/BiDi/script/language/fallback analysis，paired bracket与emoji sequence通过golden；删除UI二次分析。

### M3：Typed shaping与budget

将direct/Cosmic/vertical adapter改为typed outcome；接入deadline/cancel/memory/glyph budget、partial recovery和真实tofu；关闭Runtime11B P0-2的shaping侧条件。

### M4：Cluster/caret权威

贯通backend cluster、unsafe-to-break、ligature caret、logical/visual/source map；删除grapheme平均advance和public synthetic glyph reconstruction。

### M5：Line break与tailoring provider

以UAX14/WordBreak为基础接locale strictness、JLREQ、dictionary和hyphenation provider；soft hyphen、ellipsis、tab stop均产出typed virtual glyph mapping。

### M6：Justification与vertical收敛

用Unicode joining和backend safe insertion替换手写Arabic表，输出font/language justification receipt；vertical feature/provenance不再二次shape。

### M7：Retained document与viewport

实现dirty paragraph增量analysis/shape/reflow，覆盖wrapped/rich/vertical/preedit；cache同时受entry/byte/time budget并按owner teardown。

### M8：Product contract hard cut

Runtime Interface、renderer、Editor、accessibility、hit/caret全部消费唯一artifact；删除synthetic DTO、implicit renderer reshape和one-shot production入口。

### M9：Correctness与性能资格

完成官方Unicode corpus、复杂脚本/字体golden、malformed font/text fault、百万字文档scroll/edit soak、跨平台产品视觉和同负载Unreal p50/p95/p99/RSS比较。

## 10. 资格门

### 10.1 Unicode、locale、script与BiDi门

| Gate | 必须满足 |
|---|---|
| RTS-GATE-001 | NormalizationTest固定版本全量通过，none/NFC策略与双向offset map有golden |
| RTS-GATE-002 | BCP-47 canonicalization、likely script/region和invalid tag返回稳定typed结果 |
| RTS-GATE-003 | artifact/cache/diagnostic记录统一UnicodeDataSnapshot版本与hash |
| RTS-GATE-004 | ScriptExtensions、Common/Inherited与paired brackets在多脚本嵌套golden正确 |
| RTS-GATE-005 | emoji ZWJ/tag/keycap/RI/modifier/VS15/VS16 sequence保持单一analysis/fallback决策 |
| RTS-GATE-006 | Indic/SE Asian/Arabic/Hebrew/CJK/historic脚本itemization不使用codepoint伪script |
| RTS-GATE-007 | BidiTest current版本全量通过并记录失败level/order trace |
| RTS-GATE-008 | isolate、paired bracket、embedding overflow、L1/L2与mirroring产品golden通过 |
| RTS-GATE-009 | 同paragraph只做一次BiDi/script analysis，UI/renderer二次analysis计数为零 |
| RTS-GATE-010 | original/normalized/logical/visual/source mapping对ligature/combining/RTL可逆 |
| RTS-GATE-011 | fallback覆盖完整cluster/sequence，partial/pending/policy reject均有typed receipt |
| RTS-GATE-012 | Unicode/locale data hot update按generation失效且old artifact不被新data解释 |

### 10.2 Shaping、cluster、vertical与budget门

| Gate | 必须满足 |
|---|---|
| RTS-GATE-013 | 受影响7个test文件全部解析新budget合同，旧常量未定义引用为零 |
| RTS-GATE-014 | 任意非Ready shaping状态不进入ready cache、不发布synthetic或空success run |
| RTS-GATE-015 | 长unbroken Arabic/Indic/emoji/ligature保持一条source line与完整cluster语义 |
| RTS-GATE-016 | bytes/glyph/CPU/RSS/deadline/cancel budget超限返回typed partial/deferred且可恢复 |
| RTS-GATE-017 | RustyBuzz/Cosmic cluster coverage、monotonicity与unsafe-to-break invariants有fault tests |
| RTS-GATE-018 | backend parse/shape/feature/font failure保留phase、range、font、retryability与cause |
| RTS-GATE-019 | partial direct/Cosmic recovery不改变无关analysis run的cluster/source mapping |
| RTS-GATE-020 | vertical TTB/BTT、vert/vrt2、orientation/rotation/provenance golden通过且不双shape |
| RTS-GATE-021 | ligature caret来自GDEF/backend；无数据fallback明确且不伪装精确 |
| RTS-GATE-022 | 百万grapheme shape/cache内存与source copy受document budget约束 |
| RTS-GATE-023 | shape worker/cache按session teardown，无process-global orphan work或raw text telemetry |
| RTS-GATE-024 | shaped artifact携font/Unicode/style/document generation，stale组合被拒绝 |

### 10.3 Line break、wrap、tab、ellipsis与justification门

| Gate | 必须满足 |
|---|---|
| RTS-GATE-025 | LineBreakTest/WordBreakTest/GraphemeBreakTest固定版本全量通过 |
| RTS-GATE-026 | zh/ja/ko strict/normal/loose tailoring与JLREQ golden有rule trace |
| RTS-GATE-027 | Thai/Lao/Khmer/Myanmar dictionary break有provider版本、budget与fallback |
| RTS-GATE-028 | auto/manual/none hyphenation按locale/font/style生成真实shaped marker |
| RTS-GATE-029 | word-break/overflow-wrap/hyphens/nowrap组合矩阵与Bevy/Parley语义对照通过 |
| RTS-GATE-030 | soft hyphen不可见/断开/selection/copy/accessibility source map一致 |
| RTS-GATE-031 | start/end/middle/word ellipsis基于真实boundary与marker shape，RTL/vertical正确 |
| RTS-GATE-032 | explicit/left/right/center/decimal tab stops、leader、BiDi与vertical golden通过 |
| RTS-GATE-033 | contextual font rule超过8 grapheme时boundary correction仍正确或走安全fallback |
| RTS-GATE-034 | Arabic justification只在backend/Unicode允许位置插入，orthography golden通过 |
| RTS-GATE-035 | justification virtual glyph有零源范围、caret/selection/accessibility policy与receipt |
| RTS-GATE-036 | 所有tailoring/dictionary/hyphen/justification路径受time/output budget与cancel控制 |

### 10.4 Document、product、cache与性能门

| Gate | 必须满足 |
|---|---|
| RTS-GATE-037 | clip改变paint visibility但不改变相同layout的measured width/height |
| RTS-GATE-038 | wrapped/rich/vertical/preedit百万字viewport只materialize可见窗口+overscan |
| RTS-GATE-039 | 单paragraph edit只reanalyze/reshape/reflow依赖范围并保持stable line identity |
| RTS-GATE-040 | stale owner/revision/source组合无法命中hard-line/measure/shape/layout cache |
| RTS-GATE-041 | cache entry/source/value/resident bytes与eviction work均有上限和低基数report |
| RTS-GATE-042 | production one-shot session创建计数为零，document/window teardown回收全部artifact |
| RTS-GATE-043 | Runtime Interface synthetic glyph数为零，font/atlas identity只来自resolved artifact |
| RTS-GATE-044 | Plain resolved-glyph stale/missing/incomplete已产生typed fallback/rejection receipt且不隐式使用另一套shape真值；compiled-rich parity与managed证据完成后才关闭 |
| RTS-GATE-045 | UI/Editor caret、selection、hit、accessibility与renderer共享相同cluster positions |
| RTS-GATE-046 | App与Editor跨平台Latin/CJK/RTL/Indic/emoji/vertical/justification视觉golden通过 |
| RTS-GATE-047 | 同font/locale/text/quality/warm-cold负载对比Unreal，记录CPU/RSS/p50/p95/p99 |
| RTS-GATE-048 | qualification artifact记录source/reference fingerprint、依赖/data版本、阈值和non-ignored结果 |

## 11. 禁止的临时修补

- 不得只把`TEXT_SHAPING_RUN_MAX_BYTES`常量加回来让测试编译；旧cap会改变logical line、BiDi与复杂脚本语义。
- 不得在固定分块边缘多取几个字符就宣称任意OpenType context正确；必须依赖unsafe-to-break/cluster contract或typed fallback。
- 不得继续增加emoji、kinsoku、word-smart、Arabic joining手写字符表；统一使用版本化data/provider。
- 不得把synthetic glyph映射到某个atlas方框后宣称typed shaping完成；missing必须是真实tofu或明确失败。
- 不得让UI、Runtime Interface或renderer重新做BiDi、镜像、cluster切分和advance猜测。
- 不得用更大的cache entry常量替代byte/time/admission/owner policy。
- 不得用ignored benchmark、source-string test、test attribute数量或单台机器系统字体关闭资格门。
- 不得以不同font/fallback/quality/cache状态的数字宣称性能优于Unreal。

## 12. 本轮产出边界

初始review阶段只完成current-source静态审查、物理冻结、差距登记、目标架构与资格合同。后续M0现已修改production/test源码：source hard line不再受64 KiB执行阈值切割，旧测试引用迁移为typed budget，并记录确定性shape/prewarm请求数；该M0尚未运行Cargo、Editor、真实窗口、GPU、Unicode corpus、font/text fuzz、soak或benchmark。后续顺序为先完成RTS-P0-001 managed验证与typed defer/cancel，再建立Unicode/analysis/cluster authority，随后收敛line break/justification/document virtualization，最后硬切product contract并做资格。Runtime82继续审查editing、IME与secure text；font collection和GPU atlas分别由Runtime80与Runtime79/11C唯一拥有。

### 12.1 2026-08-26 Script itemization 实施前重审

本轮在不轮询 coordinator 的前提下继续推进 `RTS-P1-004/005/007` 的非验收实现。参考边界以
Unreal `SlateTextShaper.cpp` 的 `font-face -> script -> HarfBuzz buffer` 分层为主；Unreal 当前
Common/Inherited 粘前项本身不作为正确性标准。算法语义对照 Godot ICU `ScriptIterator`：opening
bracket 保存外层 script，matching close 继承 opening script，起始括号在首个确定 script 出现后
回填。实际数据不得复制参考引擎字符表，而是复用现有 `unicode-script` 的定长
`ScriptExtension` 位集和 `unicode-bidi::BidiDataSource::bidi_matched_opening_bracket` 的完整
`BidiBrackets.txt` 数据。

冻结的数据流为一次 paragraph-owned `script_segments(text, language)`，其结果同时交给 fallback
face 决策、horizontal/vertical direct shaping 与 Cosmic projection；不得再让 fallback 按 cluster
另算一套 script。run 候选通过 `ScriptExtension::intersection` 收敛，Common/Inherited 保持通配，
明确 BCP-47 script subtag 与前一已解析邻居只用于多候选消歧；未分配字符保持 typed Unknown
语义。paired-bracket stack 只保存 normalized opening identity 与已解析 script，mismatch close
fail closed 为普通 Common/Inherited 处理。复杂度门为 `O(codepoints + emitted_runs + bracket_depth)`，
每码点不得分配 ScriptExtensions 集合，scratch 只允许复用连续 `Vec`。

本切片不关闭 `RTS-P1-002/003/006`：likely-subtag、统一 `UnicodeDataSnapshot`、完整 emoji
property/sequence provider 仍开放；没有官方 corpus、managed Cargo、性能或功耗数据时，状态最多只能写
`non_validation_implementation_complete / static_checks_complete / managed_validation_pending`，不得宣称
`RTS-GATE-004/006` 或 M2 已验收。

### 12.2 Script itemization 非验收实现状态

`script_segment.rs` 现以 `ScriptExtension::intersection` 维护 run 候选，并用
`unicode_bidi::HardcodedBidiData` 的 paired-bracket identity 维护 opening stack。未解析 opening 只形成一个
连续后缀并在 script 确定时各回填一次；matching close 的逆向搜索同时丢弃其上不匹配 opening，因此栈工作量
按 push/pop 摊还，不在每个字符扫描完整深度。随后 emoji presentation 已从 script 语义中拆出，旧
`0x1F000..=0x1FAFF | 0x2600..=0x27BF` 宽范围被删除；MVP property 子集见 12.3，完整
RGI sequence 仍保持 `RTS-P1-006` open。

`cosmic.rs` 在同一 paragraph 请求内只构造一份 `ScriptSegment` 集合，并将其传给 fallback、horizontal
direct、vertical direct 和 Cosmic glyph projection。fallback 不再从 cluster 首码点重算 script；常用脚本仍
使用 typed variant，其它脚本使用 packed ISO15924，未分配字符使用 `FontScript::Unknown`。聚焦回归覆盖
Hira/Kana Script_Extensions、显式 language script、private-use 拒绝、outer/leading/nested brackets、historic
script stable identity 与 typed Unknown。触及生产/测试文件均低于 800 行；scoped `rustfmt --check`、
`git diff --check`、调用点和 retired-codepoint-script 扫描通过。

资产契约随后完成 hard cut：`FontScript::Other(u32)` 改为 `Other(FontScriptTag)`，字段私有且仅可由
canonical-form 四字母 ASCII tag 构造；自定义 Serde 在读入 packed 数值时再次校验，因此不能通过资产数据
绕过类型不变量。序列化仍保持原有 `{"other": <u32>}` 外形，fallback cache hash 也继续使用同一 packed
数值；契约测试覆盖 round-trip 和 malformed packed value 拒绝。旧裸 `Other(u32)` 构造扫描为 0。

当前状态为 `non_validation_implementation_complete / static_checks_complete /
managed_validation_pending`。协调器未恢复前不启动 Cargo，也未获得 Unicode corpus、cache cardinality、
p50/p95/p99、RSS、功耗、WGPU 或截图数据；因此只登记结构/正确性实现，不宣称性能收益、瓶颈消失、
RTS-GATE-004/006 通过或 M2 验收。

### 12.3 Emoji presentation MVP 非验收实现状态

实施前沿 fallback、itemize、horizontal/vertical direct 与 Cosmic projection 复核调用链，确认若在各
consumer 就地替换宽范围，会在同一 paragraph 重复 grapheme/property 扫描。冻结实现因此扩展既有
`ParagraphTextAnalysis`：构造时用 `unicode-properties 0.1.4` 的 Unicode 17 Emoji/Presentation 状态和
extended grapheme boundaries 生成 presentation ranges，并合并相邻 ranges；plain text-default 符号、VS15、
VS16、默认 emoji、keycap 与宽范围中的未分配码点均有聚焦契约。script analysis 保持纯
Script_Extensions 语义，不再让 emoji presentation 污染 Unicode script。

fallback、itemize 与 backend projection 只按 source range 二分读取同一 immutable analysis；生产源码中
Unicode emoji property 查询只存在于 `emoji_presentation.rs`，旧范围、`is_emoji_script`、逐 cluster
`shaped_script_for_cluster`/`font_script_for_cluster` 调用均扫描为 0。目标规模为
`O(codepoints + graphemes + emitted_ranges)` 时间、`O(script_runs + emoji_runs)` 额外空间，glyph loop 不做
property lookup。当前只有结构和静态证据，没有 profiler 样本，故不宣称性能收益或瓶颈消失。

状态为 `mvp_property_presentation_implemented / full_rgi_open / static_checks_complete /
managed_validation_pending`。完整 RGI ZWJ/tag/RI/modifier sequence provider、`UnicodeDataSnapshot`、官方 corpus、
managed Cargo、p50/p95/p99、RSS/功耗、WGPU/PNG、commit 与 WeCom 均保持开放，因此不关闭
`RTS-P1-003/006`、`RTS-GATE-005` 或 M2。

### 12.4 Fallback itemization typed-receipt 非验收实现状态

实施前复核确认 `FallbackResolver` 已对完整 grapheme codepoint 集执行 `face_covers_all`，并产出
`Primary/Fallback/PartialCoverage/LastResort/DepthLimitExceeded` 与 missing；缺陷是 database adapter 只返回
face，span 合并抹掉 receipt，而无 primary 又成为空 `Vec`。参考 Unreal 的 grapheme face sequence -> script
subsequence -> HarfBuzz 分层，但不复制其 first-codepoint coverage；Zircon 保持整 cluster cmap coverage。
Godot 的候选/system fallback -> raw/hex-box 分支仅用于确认缺失必须显式，不引入 synthetic hex box。

`FallbackResolution` 现在由 font owner 构造并以只读 receipt 贯穿 shaping。span 删除重复 optional face，成功
itemization 的 primary face 必填，且仅在 resolution、instance 和范围连续性全等时合并；PartialCoverage 继续
选择真实 face 的 `.notdef` 并保留缺字诊断。无 primary 返回 typed itemization error，Cosmic/service 映射
`FontUnavailable`，不再落入隐式 plain backend fallback。该切片不增加 coverage probe、candidate search 或
glyph pass，规模仍为 `O(graphemes * bounded_candidates)` 时间、`O(fallback_runs)` span 空间。

静态 formatter/whitespace/调用点检查通过，raw span face 与 implicit empty-itemization 扫描为 0。状态为
`typed_receipt_implemented / full_capability_trace_open / static_checks_complete /
managed_validation_pending`；完整 sequence/backend coverage、candidate/pending/policy/capability cause、managed
Cargo、真实 tofu raster/WGPU/PNG、性能/功耗、commit 与 WeCom 仍开放，不关闭
`RTS-P1-008/009/013/018` 或对应 gates。

### 12.5 Direct shaping typed-failure 非验收实现状态

实施前沿 `horizontal/vertical backend -> direct -> cosmic` 完整复核失败链。底层 variation、font bytes、
face index、RustyBuzz face parse 与 empty output 原先被 `Option` 合并，直接层又以 `.ok()?` 合并 source、
cluster boundary、finite metrics 与 logical-order invariant，最终只剩 whole-request fallback 或错误的
`BidiInvariant`。本切片先修复正确性与可观测边界，不改变 fallback 候选、shape pass 或产品策略。

`BackendShapeError` 现在保留 font operation、face、原始 `FontDatabaseError`、parse 与 empty cause；
`DirectShapeError` 再保留 itemization、Bidi、backend、invalid source range 和 typed backend-glyph invariant。
horizontal/vertical direct 返回 required run 或 receipt。Cosmic 只在一个 match 中决定，12.8 已将 policy
收敛为稳定 failure receipt：Bidi/source-range/itemization/budget fail closed，只有 horizontal backend capability
failure 可进入 whole-request Cosmic；vertical 返回 `ShapingFailed`。renderer/UI 不获得第二套解释规则。

静态扫描为 backend `Option<Run>` 0、direct `Option<ShapedGlyphRun>` 0、direct `.ok()?`/`Ok(None)` 0；
scoped `rustfmt --check`、`git diff --check`、调用点和文件规模检查通过，相关 owner 最大 727 行。错误分类、
invalid UTF-8 cluster offset 与 invalid itemization range 回归已写入源码但尚未执行。12.8 已补低基数 cause
report 与 capability/invariant policy；managed Cargo、p50/p95/p99、RSS/功耗、WGPU/PNG、commit 与 WeCom
仍开放，不关闭 `RTS-P1-013/018/019/021` 或对应 gates。

相邻 lowest-owner 复核补齐两处同类边界：非空 hard-line separator 的非法 range 现在保留为 typed
itemization failure，只有真实空 separator 才返回 `Ok(None)`；两个 RustyBuzz backend 改为直接接收
validated `Iso15924Tag`，不再把 script identity 降为 `&str` 后 `.ok()?` 重解析。Common/Inherited/Unknown
仍按既定语义使用 backend inference。生产 backend `Script::from_str`/script `.ok()?` 扫描为 0；该结果
仍是静态结构证据，不增加性能或验收结论。

对照 Unreal `FShapedGlyphSequence` 的 source range/index 与 Godot failed-subrun start/end 后，补齐
`RTS-P1-013/021` 的最小前置：`DirectShapeError::Backend` 现在同时拥有 itemized segment `TextRange`
和原始 `BackendShapeError`，horizontal/vertical 三个 backend 调用点均显式附加该 range。该 typed cause
仍在 Cosmic whole-request boundary 消费；12.8 已补 code/report，但 run-local composition 与上层 outcome receipt 保持 open。

### 12.6 Locale canonical identity 基础设施重审与非验收实现状态

实施前确认 locale 的结构瓶颈不是某个 lowercase 循环本身，而是 owner 分裂：Runtime Interface、
`text/language.rs` 与 shaped cache 各自解释 trim/case/separator，服务入口又只检查空值；Cosmic FontSystem cache
每请求重新生成 `String`。这会让非法标签进入 backend、让 canonical-equivalent 输入依赖手写 byte identity，
并在 `layout_session -> service -> shape_text` 多层边界重复工作。没有 profiler 运行数据时，本轮只把这些作为
源码证明的结构缺陷，不声称它们是已量化主瓶颈。

Unreal 主参考显示 `FCulture` 集中缓存 canonical name、script 与 region，`GetCanonicalName` 委托同一 culture
implementation，而 `GetPrioritizedParentCultureNames` 再从 language/script/region 生成独立父文化顺序。由此冻结
Zircon 边界：Runtime Interface 只传中立字符串，`text/language.rs` 是唯一 policy owner；canonical identity 与
future fallback receipt 不得混为一个 helper，也不得由 cache 猜测 likely script。

当前实现引入 text-feature scoped `icu_locale_core 2.2.0`。`Locale::try_from_str` 一次结构化解析负责语法、canonical
casing 与标签中显式 script 提取，underscore 只在应用边界转换；canonical 输入返回 borrowed `Cow`。
`BackendShapeRequest::canonicalized` 对非法非空标签返回 `InvalidLanguage`，改变后的 canonical tag 由 scope 持有，
定长 `TextLanguageScriptSubtag` 与 tag 一起进入 request，私有 canonical 标记使下游重复 canonicalize 只重借用。
shaped cache 删除四个手写 normalization/hash/match helper，改为 exact canonical
hash/equality；Cosmic 四项 LRU 删除每请求 normalization/owned string，仅在 miss 插入新 locale 时分配。
Runtime Interface 的旧公开 helper 已硬删除，UI/graphics 调用点统一依赖 Runtime Text owner。

静态证据：retired locale helper 0，cache 手写 language byte/case normalization 0，language/itemizer 手写 tag split 0，
所有 `canonicalized()` 调用点均显式处理 `Result`。本切片生产 owner 为 language 192 行、shaped model 605 行、
script analysis 624 行、Cosmic 783 行、service 636 行；194 行 shaped-model 测试已从内联 owner 硬切到
folder-backed `model/shaped_run/tests.rs`，均低于 800 行。scoped formatter 与 whitespace 检查通过。managed Cargo 未运行，locale corpus、
cache-cardinality、p50/p95/p99、RSS/功耗、WGPU/PNG、commit 与 WeCom 均无新证据。

状态为 `canonical_tag_and_explicit_script_projection_implemented / likely_subtag_receipt_open /
static_checks_complete / managed_validation_pending`。likely script/region、版本化 locale snapshot、父文化/fallback
decision receipt 和对应 generation invalidation 仍归 `RTS-P1-002`，因此不关闭 `RTS-GATE-006/012/047/048` 或 M2。

### 12.7 Unicode provider snapshot 基础设施重审与非验收实现状态

源码与本地依赖调研确认当前 provider 数据并不同步：`unicode-linebreak 0.1.5` 为 Unicode 15，
`unicode-bidi 0.3.18` 与 `unicode-bidi-mirroring 0.4.0` 为 Unicode 16，Normalization、Script、Grapheme、
Emoji properties 与 VerticalOrientation revision 均为 17；`icu_locale_core 2.2.0` 当前只提供 locale syntax/casing，
没有可声明的 CLDR data version。由此拒绝单一 `UNICODE_VERSION=17` 的错误设计，改由 `text/unicode_data.rs`
记录九个 provider 的 implementation revision 与可用 data version。

所有 provider 在 `zircon_runtime/Cargo.toml` 精确锁到当前 `Cargo.lock` 解析版本，版本升级必须与 snapshot schema/
generation 同步。稳定 FNV schema fingerprint 与 generation 组成 16-byte `UnicodeDataSnapshotId`；完整 descriptor 只保留
一份，不复制进热路径。`BackendShapeRequest` 在 analysis 前冻结 identity，canonical reborrow 保留它；
`ParagraphTextAnalysis`、`BidiParagraph`、`BidiLineSignature`/`BidiLineOrder`、
`LineBreakOpportunityMap`、`ShapedGlyphRun`、shaped-cache exact/direction-alias fingerprint 与
`TextLayoutFallbackReport` 均传播同一 identity。direct/Cosmic consumer 对 request/artifact、cache admission 对
key/artifact 做 debug invariant。

序列化采用硬切：缺失 snapshot identity 的旧 `ShapedGlyphRun` 必须失败，不能以 current data 回填后继续解释旧
cluster/range。回归同时覆盖依赖导出的 Unicode data version、mixed version 可见性、generation identity、canonical
reborrow、script/Bidi/line-break analysis retention、cache generation isolation、diagnostic identity 与 legacy wire
rejection。热路径新增固定 16 bytes/request/artifact/key；按 1024-entry shaped-cache 上限，key 增量约 16 KiB，
不增加 analysis、shape call 或 renderer pass。新增绑定涉及的 Bidi owner 697 行、line-break owner 177 行、Cosmic
owner 762 行，均低于 800 行。

scoped formatter/parser 与 whitespace 检查通过；没有 managed compiler/test、corpus、p50/p95/p99、RSS/功耗或
WGPU/PNG 新证据。状态为 `compiled_snapshot_identity_implemented / static_checks_complete /
managed_validation_pending`。动态 provider replacement、old-generation descriptor/lease retirement、完整
analysis/layout/document artifact 贯通仍开放，故不关闭 `RTS-P1-003`、`RTS-GATE-003/012` 或 M1/M2。

### 12.8 Stable direct failure receipt 与 fallback policy 非验收实现状态

继续重审 P1-021 后确认不能直接实现 run-local recovery：direct horizontal/vertical 在首个 segment error 时返回整 run
失败，Cosmic 又只接受完整 request；当前没有 subrun glyph/source map、line metric merge 或 Bidi order composition
合同。此时局部拼接会让 cluster、baseline 和 source range 语义漂移，因此本轮先完成 P1-013 的 policy 前置。

新增 `shaping/failure_receipt.rs` 单一分类 owner。`TextShapingFailureReceipt` 是无分配 Copy schema，包含 12 个显式
discriminant 与稳定 `text.*` code、phase、optional source range、optional face、dependency、disposition 和 optional
budget。`TextShapingFailureReport` 按 code 使用定长 `[u64; 12]` 计数并保留最后 receipt，同时携 Unicode snapshot
identity；全局 mutex 只在 direct error 冷路径获取，不进入 glyph/cluster 成功热循环。旧
`DirectShapeError::is_bidi_invariant` 已删除，生产 failure policy owner 扫描为 1。

Cosmic 唯一策略边界现调用该 owner：Bidi、非法 source range、itemization/missing fallback span 和
`FontDatabaseError::SourceBudget` 均为 terminal；只有 horizontal backend font-database、face parse、empty output 与
backend-glyph invariant 可标记 `AlternateBackend`。vertical direct failure 全部 terminal。由此 font source admission
budget 不再可能因整请求 Cosmic fallback 被绕过，结构 invariant 也不会被备用结果掩盖；但 horizontal fallback 仍是
whole-request，不宣称 cluster 语义已统一。

`shaping/outcome.rs` 随后建立统一 non-Ready envelope：`TextShapingFailure` 同时持 neutral
`TextLayoutError` 与 optional request receipt，`Deferred`/`Failed` variant 只表达 disposition。direct terminal failure
及 alternate-backend 后最终失败均保留该 receipt；stable generation helper、session/cache、layout transform 与内部 UI
forwarding 不再把它压成 coarse error。`map`/`and_then` 保持 envelope，session regression 固定 face/range receipt retention
与零 cache admission。只有跨公开 `TextLayoutService` 时调用 `into_error()` 投影 neutral error，因此不形成
`core/framework/text -> text` 反向依赖。全局 report 仍只写不读，不参与 request policy。

本地 Unreal 对照确认 Slate 的 `FShapedGlyphSequence` 由 font-cache/shaper owner 持 glyph 与 source-range/subsequence
语义，renderer 只消费序列；其 `ShapeBidirectionalText`/`ShapeUnidirectionalText` 返回 sequence ref，并未提供 Zircon
所需 typed failure receipt。因此本实现沿用其 sequence ownership，同时在 Runtime Text 内补强请求失败合同，而不是把
分类策略下放 UI/renderer。

回归已编码 stable code label 唯一性、face/range/phase retention、horizontal/vertical disposition、Bidi/budget terminal、
低基数计数、outcome transform retention 和 session/cache retention。静态 parser/formatter、whitespace、唯一分类 owner、
两个记录点、旧 raw non-Ready constructor 与 receipt-losing shape conversion 扫描通过；当前文件为 outcome 208 行、
receipt 457 行、direct error 112 行、Cosmic 783 行、service 633 行、session 749 行，均低于 800。managed
compiler/test、fault injection、实际 error cardinality、p50/p95/p99、RSS/功耗与产品图均无新证据。状态为
`request_outcome_receipt_implemented / public_neutral_error_projection_retained /
run_local_composition_open / static_checks_complete / managed_validation_pending`，不关闭
`RTS-P1-013/021`、`RTS-GATE-016/020/021/047/048` 或 M2。

### 12.9 Explicit locale-script owner 收敛与非验收实现状态

后续重审发现 locale canonicalization 虽已集中，但 `script_segment.rs` 仍对 canonical 字符串执行第二套
`split(['-', '_'])` 并把第二段当作 script；这既重复解析，也让 private-use/extension subtag 有机会被误认。
本地 Unreal `FCulture` 对照再次确认 canonical name、script、region 应来自同一 culture implementation，而不是由
shaper/itemizer 重拆名称。

`CanonicalTextLanguage` 现在用一次 ICU4X `Locale::try_from_str` 同时形成 borrowed/owned canonical tag 和结构化
`Locale.id.script` 投影。`CanonicalBackendShapeRequest` 持有二者，重借用保持定长四字节 script identity；service 的
fallback span 与 Cosmic paragraph analysis 都从该 backend request 构造 `ParagraphTextAnalysis`。无显式 script 的
`ja-x-Kana`、Unicode extension 与非法标签不会伪造 script，`zh-Hans` 的 `Hans` 则跨 canonical reborrow 保留。

生产手写 tag split 扫描为 0；formatter/parser、whitespace、请求接线和文件预算检查通过。该切片没有添加 likely-subtag
provider，没有把 language 推断硬编码成 script，也没有运行 managed Cargo、corpus、性能/功耗或 WGPU/PNG 验收。
状态为 `canonical_tag_and_explicit_script_projection_implemented / likely_subtag_receipt_open /
static_checks_complete / managed_validation_pending`，只更新 `RTS-P1-002/004` 的非验收完成项。

### 12.10 CompositeFont culture owner / query identity 重审与非验收实现状态

继续审查 locale 主链时发现，资产 DTO 的 `FontCultureTag::matches` 用不区分 subtag 边界的大小写 prefix 匹配，
`CompositeFontIndex` 又在每次候选查询中调用它；当 generic Han sub-font 声明在前时，它还能先于 culture-specific
sub-font 命中。这不是微观比较函数问题，而是资源 schema、运行时 policy、请求 identity 和优先级 owner 混在一起。

本地 Unreal 主参考给出明确结构：`FCulture::GetPrioritizedParentCultureNames` 形成
language-script-region、language-region、language-script、language；`RefreshFontRanges` 只判 selector 是否属于该集合，
把命中范围放进独立 priority bucket，`GetTypefaceForCodepoint` 先查 priority、再查 generic、最后 default，bucket 内
保持资产声明序。Zircon 据此没有引入额外的“最具体 selector 排序”，也没有硬编码 language-to-script。

当前 `text/language.rs` 一次 ICU4X 解析同时形成 canonical tag 与 Copy
`TextLanguageFallbackKey { language, script, region }`。shaping request 保留该 key，fallback span 直接传入 resolver；
fallback candidate/line-metric cache identity 精确哈希这三个候选相关分量。`FontCultureTag` 回到 opaque asset value，
`CompositeFontIndex` 仅在 cache miss/generation 发布时编译 selector；无效受限值为 `Some(empty)`，不会伪装成无 cultures
的 unrestricted sub-font。显式 descriptor identity 的 cache-hit 前置只哈希作者字节，不调用 locale parser，避免把
ICU 工作重新放回查询热路径。

复杂度保持 index build `O(subfonts + scripts + ranges + culture selectors)`；查询新增的 culture 工作为命中 sub-font
上的 `O(compiled selectors)` 定长 subtag 比较，不增加 Unicode/codepoint pass、font coverage probe 或 shape pass。
当前只有源码结构量化，没有运行时 profile，不能据此声称耗时或功耗改善。聚焦回归与静态扫描已经编码/通过静态检查，
managed Cargo、locale corpus、cache cardinality、p50/p95/p99、RSS/功耗、WGPU/PNG、commit 与 WeCom 仍开放。状态为
`canonical_locale_fallback_key_and_culture_priority_implemented / likely_subtag_receipt_open /
static_checks_complete / managed_validation_pending`，不关闭 `RTS-P1-002/008/009/018`、M2 或对应 gates。

### 12.11 RustyBuzz cluster break-safety receipt 重审与非验收实现状态

前一轮已经拒绝把 `unsafe_to_break` 误写成 no-break：RustyBuzz 0.20.1 的公开合同是“若在该 cluster 起点断开，
两侧必须重新 shaping 才能得到正确结果”。Text03 当前的 soft opportunity 在前一 cluster 的尾部，而 safety receipt
属于后一 cluster 的起点；直接修改 `soft_break` 不但 owner 错位，还会删除合法 UAX#14 机会。正确顺序必须是 shaping
发布 provenance，final-line owner 在选定边界后决定复用、局部重塑或保守完整重塑。

本地 Unreal `SlateTextShaper.cpp` 使用 `HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES`，由 shaper 构造带 source index、
character/grapheme-cluster count 的 shaped sequence；其当前 `FShapedGlyphEntry` 没有 unsafe flag。Zircon 沿用该
shaper-owned sequence 边界，但利用 RustyBuzz 已公开的 `GlyphInfo::unsafe_to_break()` 补强内部 receipt，不把该差异
扩散到 Runtime Interface 或 renderer。

新增 `#[repr(u8)] ShapedGlyphBreakSafety { Unknown, Safe, RequiresReshape }`。horizontal/vertical backend 各在唯一
glyph-info 投影点采集 flag；direct 恢复 logical cluster order 后用 `any` 在该 cluster 的既有 slice 上聚合，并由一个
model-owned helper 保证只有 cluster 头可发布 known receipt。Cosmic 的完整 flags constructor 显式写 `Unknown`，virtual
glyph 与旧 serde 数据也由 default 得到 `Unknown`。receipt round-trip 进入 shaped artifact，neutral framework glyph
没有获得 line-break policy。相邻 lower-owner 审查修复 vertical `vertical_buffer` 的 `&str`/`Iso15924Tag` 类型不一致，
两个 RustyBuzz backend 的 raw script 参数扫描为 0。

新增工作是每 backend glyph 读取一个已有 mask 位、每 cluster 在已有 slice 上聚合；没有新增 Unicode pass、Vec、range
副本、shape call 或 renderer pass。enum 大小契约为 1 byte，但未运行动态内存/延迟测试，不能声明 `ShapedGlyph` resident
size 或功耗收益。scoped formatter/whitespace、构造点、raw-script、文件预算扫描通过，最大触及 production owner 为
Cosmic 784 行。source-present 回归未执行。状态为 `direct_break_safety_receipt_implemented /
final_line_reshape_open / static_checks_complete / managed_validation_pending`；`RTS-P1-017/035`、corpus、Cargo、
shape-call/p50/p95/p99/RSS/功耗、WGPU/PNG、commit 与 WeCom 仍开放。

### 12.12 Soft-hyphen virtual artifact 重审状态

`line_break/soft_hyphen.rs` 已用 `DiscretionaryHyphenDecision` 保存 consumed U+00AD range、explicit marker mode、
zero-width virtual anchor 与 checked rebase。Plain wrapping 把 marker 投影到 anchor，line range 仍消费 U+00AD，因此
`LogicalVirtualLineSequence` 可用当前 Plain style 一次 shape，并让 final metrics 与 glyph artifact 共用 retained fragment。
profiling-feature contract 要求 logical virtual shape=1、retained projection=1、renderer projection/fallback reshape=0；
managed 执行仍待完成。

该检查点之后，rich horizontal 与 VerticalRl 已取得 display-owned UAX#9 sidecar、resolved style identity、canonical
virtual glyph slice 与 consumed U+00AD replacement receipt；accessibility 继续使用原始 source value。VerticalRl
按 typed role 校验 ASCII marker 与 replaced range，Cargo/corpus/profile/power/WGPU 仍开放。当前状态为
`plain_horizontal_vertical_rich_canonical_virtual_artifact_implemented /
typed_virtual_fragment_role_implemented / source_receipt_and_accessibility_preservation_confirmed /
virtual_receipt_linear_capture_implemented /
static_checks_complete / managed_validation_pending`；`RTS-P1-031` 保持开放。

### 12.13 Line-break profile/opportunity receipt 非验收实现状态

前一 Unicode snapshot 切片已经把 `unicode-linebreak 0.1.5 / Unicode 15.0.0` 纳入 request-bound 16-byte identity，并
随 `ShapedGlyphRun` 发布；因此 `RTS-P1-025` 的数据版本部分已有 owner。剩余直接缺口是 cluster 只有两个 bool，无法说明
机会来自 provider allowed、provider mandatory、显式 mandatory control，还是旧数据 unknown。

新增 1-byte `LineBreakTailoringProfile::{Unknown, UnicodeDefault}`、1-byte
`ShapedGlyphLineBreakOpportunity::{None, ProviderAllowed, ProviderMandatory, MandatoryControl}` 与 2-byte receipt。
`LineBreakOpportunityMap` 一次构建机会 Vec；horizontal/vertical direct 和 Cosmic 的 cluster 头从同一 map 投影，hard-line
separator 由 itemizer 显式发布 control，non-head/legacy 为 unknown。framework/renderer 命中为 0，layout 没有新增基于
receipt 的策略分支。

复杂度仍是一次 provider scan/build，加每 cluster 两次 `partition_point` 和窗口内机会检查；没有新增 Unicode pass、
shape call、Vec 或 renderer pass。完整 flags 大小断言为 11 bytes。聚合 shaping tests 将 line-break/script 回归拆到 child
owner 后为 773 行，production shaped model/line-break/Cosmic 为 691/248/797 行。formatter、whitespace、owner/consumer 和
neutral-boundary 静态扫描通过；source-present tests 未执行。

`unicode-linebreak` API 不公开具体 UAX#14 rule number，locale tailoring profile/data 与 Editor trace 仍未实现。状态为
`line_break_profile_opportunity_receipt_implemented / rule_number_and_locale_tailoring_open /
static_checks_complete / managed_validation_pending`；不关闭 `RTS-P1-025/026`、Cargo/corpus、性能/功耗、WGPU/PNG、
commit 或 WeCom。

### 12.14 Ligature caret/advance owner 重审状态

`text/layout/measure.rs` 当前把一个 backend cluster 的 advance 按其覆盖 grapheme 数均分；artifact hit path 对同一
multi-grapheme cluster 却按 leading/trailing whole-cluster edge 处理。由此 artifact 缺失/过期 fallback 与 retained
artifact 形成两套 caret 语义。Unreal 的 `FShapedGlyphEntry` 保留 character/grapheme-cluster count，
`HasFoundGlyphAtOffset` 对 `NumGraphemeClustersInGlyph > 1` 采用 atomic ligature，overflow 也不部分裁切该 glyph。

本地 `ttf-parser 0.21.1/0.25.1` 明确跳过 GDEF LigCaretList，RustyBuzz 0.20.1 无 ligature caret API，因此不能在现有栈
上宣称 font-derived interior caret。先引入 layout/public geometry 共用的 cluster/source contract，并发布 typed
`FontCaret/AtomicCluster` policy，再替换均分兼容层；否则只改 measure 会让 wrap/hit/accessibility 继续分叉。状态为
`architecture_review_complete / atomic_cluster_artifact_present / cluster_aware_layout_contract_open /
gdef_caret_provider_open / managed_validation_pending`，没有代码行为或性能完成声明。

### 12.15 EndWord UAX #29 boundary owner 非验收实现状态

旧 `word_ellipsis_prefix_end` 只按 whitespace 回退，已完成内容若由 `-` 等标点分隔或处于 CJK 无空格文本会被错误清空。
新增 `text/word_boundary.rs` 零拷贝 view：借用 source、携 `UnicodeDataSnapshotId`，由 `unicode_word_indices()` 统一提供
previous/next/range/completed-prefix 查询。EndWord layout 与 UI navigation 均已删除本地 policy 并消费该 owner；查询
最多扫描到目标 prefix，不分配 paragraph-sized boundary Vec。

compiled Unicode snapshot 现有独立 Word provider role，当前 schema/generation 为 3。它与 Grapheme 当前共享
`unicode-segmentation 1.13.3 / Unicode 17`，但作为不同能力分别进入 fingerprint。source-present 回归覆盖连字符、
apostrophe、CJK 与 UI navigation；未执行 managed tests。

状态为 `endword_unicode_boundary_selection_implemented /
horizontal_text_only_rich_marker_artifact_implemented /
private_omitted_source_geometry_receipt_implemented /
accessibility_source_preservation_confirmed / locale_dictionary_open / static_checks_complete /
managed_validation_pending`。`RTS-P1-032` 的 horizontal inline external block、ordinary styled VerticalRl、
vertical external block、U+2026 ellipsis 与 typed soft-hyphen 已完成静态实现；动态 widget，
`RTS-P1-026/028/029` 的 corpus/dictionary/tailoring，以及 Cargo、性能/功耗、WGPU/PNG、commit、WeCom 仍开放。

### 12.16 Shared cluster geometry 与 atomic wrap 非验收实现状态

架构复审后的第一步不是调整均分公式，而是删除并行 owner。新增 `text/cluster_geometry.rs`，以一个零分配 iterator
同时聚合 `ShapedGlyph` 与 renderer `TextGlyph`；`glyph_artifact.rs` 删除本地 cluster 搜索，caret/hit/selection 与
measurement 共享 backend cluster/source coverage。混合方向 cluster 保留 advance 但对 caret fail closed。

`measure.rs` 一次 shape 同时产出 compatibility grapheme advances 与 `MeasuredGlyphCluster` receipt；多 grapheme cluster
在没有 font caret 时为 `AtomicCluster`。该 receipt 进入 plain fragment、`GraphemeAdvanceIndex` 与 `RichAdvanceIndex`。
plain/rich corrected glyph ranges 使用单调 cluster 游标将 tentative end 推到原子 cluster 末端；UI 当前行已有内容时也
先按剩余宽度生成 corrected ranges 并整 range 提交，首个完整单元放不下才换行后按整行宽重算。跨-run leading
grapheme continuation 分支未伪造独立 segment 之外的 cluster truth，仍保持开放。

静态复杂度为 cluster aggregation `O(glyphs)`、boundary coalescing `O(candidate_ranges + clusters)`；普通 width-only
投影不保留 cluster Vec，retained index/fragment 各保留紧凑 receipt，并且 ordinary path 不新增 shape call。尚未通过
managed Cargo、corpus 或 profiler 验证，因此没有 p50/p95/p99、RSS、功耗或其它引擎耗时对标结论。

状态为 `shared_cluster_geometry_receipt_and_plain_rich_atomic_boundary_coalescing_implemented /
cross_run_continuation_and_public_geometry_open / gdef_caret_provider_open / static_checks_complete /
managed_validation_pending`。`RTS-P1-034/017/035`、真实 WGPU/PNG、commit 与 WeCom 仍开放。

### 12.17 WordSmart UAX #29 context 与 GeneralCategory policy 非验收实现状态

`line_break/smart.rs` 已删除 ASCII、CJK、Arabic 与其它 Unicode 收尾标点/闭合符号手写表。protected candidate 先要求
紧邻 `WordBoundaryMap` 发布的 UAX #29 word end，再由 Unicode `General_Category` style policy 以 `OtherPunctuation`
触发，并仅以 `ClosePunctuation/FinalPunctuation` 延伸同一 run；open punctuation、dash、symbol/emoji 与 separator 不触发。

`WordEndCursor` 对 UAX #29 ranges 和有序 chunks 单调推进，静态规模为 `O(text + chunks + word ranges)`，无第二张
paragraph-sized boundary Vec。split/merge 只接受 text/visual/source 长度同构的 chunk，merge 还要求两条 range 都连续；
非同构或断裂映射 fail closed。GeneralCategory 作为独立 snapshot capability 加入后，后续 JoiningType capability 又使
compiled identity 升为 12 roles、schema 4、generation 4；共享 crate/data 不能合并语义角色。

状态为 `word_smart_uax29_context_and_general_category_policy_implemented / locale_dictionary_and_tailoring_open /
static_checks_complete / managed_validation_pending`。这不是 `RTS-P1-028` 关闭：locale dictionary、完整 style contract、
WordBreakTest/多脚本 corpus、managed Cargo、profile/RSS/功耗、真实 WGPU/PNG、commit 与 WeCom 仍开放。

### 12.18 Rich horizontal phase attribution 非验收实现状态

public caret/artifact 重审暴露了一个可测量 hypothesis：Glyph rich range index、rich layout materialization 与 UI item
projection 可能对同一 styled source 重复请求 shaping。当前没有用猜测改 composite artifact、cache 或 paragraph model；
先在 `layout_engine/rich_layout/profile.rs` 以 provider adapter 聚合三个 phase 的 request count 与 input bytes。

每个 phase 只有一个 scope 和两个结束时 counter，数量不随 run/glyph 增长；普通非 profiling build 不保留计数字段，
也没有 TLS/global registry、结果 clone 或策略分支。profiling contract 使用真实 BBCode Glyph-wrap 并要求每个 phase 的
span/counter sample 恰好一次。主 rich owner/profile child 为 404/140 行，formatter、whitespace、counter owner 与文件预算
静态检查通过。

状态为 `rich_shape_phase_instrumentation_implemented / repeated_shaping_hypothesis_unconfirmed /
static_checks_complete / managed_profile_pending`。managed compile、31-sample 1/100/1k/10k runs、cache/backend attribution、
allocation/RSS/p50/p95/p99、功耗、Unreal 对标、WGPU/PNG、commit 与 WeCom 均未完成，不授权结构优化或关闭
`RTS-P1-022/024/034`。

### 12.19 Unicode Joining_Type 与 Arabic Kashida candidate 非验收实现状态

`text/joining_type.rs` 现以 `icu_properties 2.2.0 / Unicode 17` 的编译期 trie 单一拥有 Joining_Type；raw ICU 类型不
穿透到 layout/UI/renderer。`align.rs` 已删除 Arabic letter/non-left 手写范围，逻辑相邻字符按 Left/Right/Dual/
JoinCausing/Transparent 连接语义判定，并以 Arabic script gate 排除其它 joining script。透明 mark 与 ZWJ 保留 grapheme
连接，ZWNJ 继续阻断；属性表只初始化一次，扫描为 `O(graphemes)` 且无逐字符分配。

`layout/arabic_justification.rs` 进一步消费一次完整 candidate shape。成功收据要求 source/line identity 完整、Tatweel
独占 backend cluster、glyph id 非零、advance 为正，并与左右 RTL cluster 使用同一 face/instance；mixed virtual/source
cluster、tofu、fallback-face 断裂、缺邻接和非增长 candidate 都 fail closed。共享 cluster iterator 增加 glyph span 后，
校验为 `O(glyph clusters + inserted tatweels)`，范围 scratch 受 32 上限约束；UI 只消费收据 width/count。

这仍只部分推进 `RTS-P1-036`。font/language justification capability、最多 32 个候选/5 次 reshape 的预算与算法仍开放。
本地 Unreal 源未找到可直接对照的 Tatweel/Kashida 算法，因此只采用其 shaper-owned artifact/validation 边界，不宣称实现
等价。状态为 `unicode_joining_type_and_backend_tatweel_safety_receipt_implemented /
language_font_justification_and_probe_strategy_open / static_checks_complete / managed_validation_pending`。managed Cargo、
Arabic corpus、1/100/1k/10k profile/RSS/功耗、真实 WGPU/PNG、commit 与 WeCom 均未完成。

候选搜索现在具备不改变算法的行级观测：`line_box/profile.rs` 仅在 profiling build 保留六个聚合字段，并为每条实际进入
fit 的物理行发布一个 scope；probe loop 内没有 profiler event。拒绝码 0/1..13/14 分别表示无安全拒绝、显式 backend
safety rejection 与 receipt-count mismatch。真实 Arabic fixture 的 profiling contract 已写入，但 managed 执行和 31-sample
规模数据尚未取得。增量状态为 `arabic_tatweel_probe_instrumentation_implemented / algorithm_unchanged /
static_checks_complete / managed_profile_pending`，因此 `RTS-P1-036` 保持开放。

### 12.20 Rich immutable artifact 结构性修正状态

本轮先重审完整调用链和本地 Unreal `FSlateTextRun`、`FShapedGlyphSequence`、`FShapedTextCache`，未先改缓存
阈值或局部循环。确认 Zircon 的结构性瓶颈是一个 rich handle 只能 exact-downcast 为 compiled metadata 或 glyph
artifact，导致保留链接/inline 元数据时 renderer 对每个 visual paint run 独立 reshape；RTL visual projection 还会
把一个逻辑 joining run 拆成多个 grapheme-sized 请求。该问题不是 hash、锁或小对象分配微调可以消除的。

已实现 private `ResolvedRichTextArtifact`：同一 handle 强持有 compiled metadata、一个完整 immutable glyph artifact、
精确 layout-line snapshot 与 run-to-glyph-slice directory。物理行 glyph 只存一次，renderer 借用 slice；跨样式
ligature 由首个 run 唯一拥有，continuation 发布空 slice receipt。ellipsis、inline-only、VerticalRl 等暂不产 glyph
的行发布 negative receipt，使 extract 不会逐帧重建。每行的 font pair 一次批量注册，
静态规模为 `O(line/style intersections + glyphs + clusters + paint runs)`。

新增 profiling-only 聚合 owner 可分别记录 rich artifact shape request/input bytes、mapped run、shaped-cache
hit/miss、renderer artifact-run 与 fallback request/input bytes；没有 per-glyph/per-grapheme event，也没有普通 build
全局诊断状态。源码回归覆盖 composite 双能力解析、identity、run slice、跨-run ligature 空回执、cache eviction、
font-size override 与显式 fallback line reuse，但这些测试尚未经过 managed Cargo 执行。

horizontal rich U+00AD 已进一步复用 `LogicalVirtualLineSequence`：source hint 由 break decision 消费，显示 run
发布 end offset 的 zero-width anchor；logical cluster 以单调游标解析回 rich source style，相邻同 style 合并后
shape，再由 sidecar 恢复 visual order/source ownership。virtual glyph 只归属零宽 run；字体代际 rebuild 只有在
source/style/writing mode/完整 layout-line snapshot 相等时才复用 sidecar。新增的
`rich_artifact_virtual_line_count` 是每次 artifact build 的低基数聚合值，不在 glyph loop 发事件。VerticalRl
现以同一 typed discretionary-hyphen receipt 保留 logical sidecar。

状态为 `rich_composite_artifact_and_run_slice_route_implemented /
rich_horizontal_soft_hyphen_virtual_artifact_implemented /
rich_vertical_soft_hyphen_virtual_artifact_implemented / virtual_receipt_linear_capture_implemented /
static_checks_complete /
managed_profile_pending`。31-sample cold/warm
1/100/1k/10k、backend call、p50/p95/p99、allocation/RSS、功耗、Unreal 经验值对标、真实 WGPU/PNG、commit 与
WeCom 均未完成；不能声称瓶颈已消失、功耗接近其它引擎或算法已达到最优规模。

### 12.21 Rich ellipsis current-run style owner 非验收实现状态

继续对照本地 Unreal `FSlateTextRun` 与 `FindOrAddOverflowEllipsisText` 后，确认 overflow marker 属于当前
text run 的 shaped/cache product，而不是 renderer 用 zero-width anchor 猜测的临时文本。旧 Zircon horizontal
rich 路径先做 UAX#9、再追加 neutral-style ellipsis，并清空 logical sidecar；即使 glyph artifact 可生成，renderer
也因 `start == end` 无法解析 rich presentation。该结构问题同时破坏 contextual ordering 与 style identity。

现由 `CandidateLine` 为 generated cluster 发布非空 style-source owner：soft hyphen 使用被消费 U+00AD 范围，
End/EndWord/Middle ellipsis 使用逻辑插入点前的 current run，Start 使用后继 run。horizontal text-only rich
ellipsis 先用该样式度量并写入 logical sequence，再统一执行 UAX#9、style-span shape、glyph projection 和 run-slice
publication；renderer 只消费目录中的同一 owner 解析 font/color/decoration。公共 DTO 未增加 renderer 字段。

owner preview、capture、style coalescing、glyph projection 与 directory build 都是单调线性遍历，静态规模为
`O(graphemes + style intersections + glyphs + paint runs)`，没有 `styles * graphemes` 嵌套扫描。状态为
`rich_text_only_ellipsis_virtual_artifact_implemented / virtual_source_and_style_receipt_implemented /
private_omitted_source_geometry_receipt_implemented / accessibility_source_preservation_confirmed /
static_checks_complete / managed_profile_pending`。horizontal inline external block 已完成静态实现；VerticalRl
external/marker、动态 widget、managed Cargo、31-sample p50/p95/p99、
allocation/RSS、功耗、Unreal 经验值对标、真实 WGPU/PNG、commit 与 WeCom 均开放；
`RTS-P1-032` 不关闭，不能声称瓶颈或功耗 gate 已通过。

### 12.22 Ellipsis omitted-source geometry receipt 非验收实现状态

结构复审确认 public zero-width anchor 只能表达 insertion point，不能表达 ellipsis 替换的 source interval；由
renderer、selection 或 accessibility 各自猜测都会形成并行 owner。现统一由 `CandidateLine` 发布 generated cluster
receipt：非空 `style_source_range` 决定 shaping/presentation owner，可选 `replaced_source_range` 表达被替换源码。
final-line ellipsis 只在 retained source ranges 的补集恰好为一个连续非空区间时发布 replacement；多段缺口、
越界或重叠均 fail closed。

`LogicalVirtualLineSequence` 将 replacement 纳入 immutable identity，rich glyph run directory 保存同一回执；
private glyph geometry owner 处理省略区间内 caret upstream/downstream、marker 中点 hit-test 与 selection span 合并。
公共 `UiTextLineSourceMap`、renderer DTO 和 accessibility DTO 不增加第二套映射。accessibility name/value 继续读取
template metadata、component state 或 widget value 的原始语义文本，不消费 `UiResolvedTextLine.text`。

replacement 生成沿 retained ranges 单调扫描，geometry 查询复用现有 cluster/glyph walk，静态规模为
`O(clusters + glyphs)`，没有持久 paragraph-sized map。源码回归覆盖 End/Middle/Start、歧义 fail-closed、caret
affinity、marker 边界 hit 与 selection 合并；本轮未执行 managed Cargo、profile、功耗或真实 framebuffer，因此
状态为 `private_omitted_source_geometry_receipt_implemented /
accessibility_source_preservation_confirmed / static_checks_complete / managed_validation_pending`。

### 12.23 Horizontal rich inline external-block artifact 非验收实现状态

对照本地 Unreal `SlateImageRun`、`SlateWidgetRun` 与 `SlateTextRun` 后，结构复审确认 compiled inline
image/widget 应是独立 layout block：它拥有精确 source character range、参与 measure/arrange/hit-test 并由对象
run paint，而不是交给字体后端整形 U+FFFC。当前 horizontal rich layout 从 compiled run 元数据发布 explicit
external-cluster receipt；literal U+FFFC 因没有该 receipt 继续按普通文本处理。

logical display sidecar 现在可由 virtual cluster 或 external cluster 建立。UAX#9 与 final grapheme advances 保留
external cluster，rich style spans 在它两侧拆分，glyph projection 排除它，run directory 为 inline paint run 发布
显式空 glyph slice。该路径覆盖普通 text+inline、inline+ellipsis 与 inline-only；纯 inline 行只有在全部 cluster 均
被证明 external 时才允许 accepted zero-glyph text artifact，其他空 shaping 继续 fail closed。caret、hit-test 与
selection 复用同一个 visual-cluster/final-advance geometry owner；BiDi 投影已拒绝的 sidecar 不得参与几何。

capture、style coalescing、glyph projection、directory publication 与 geometry 均为单调游标遍历，静态规模是
`O(clusters + style intersections + glyphs + paint runs)`，不新增 per-inline font shape、renderer-local placeholder
推断或 paragraph-sized map。847 行 glyph root 已拆为 538 行 owner 与 361 行 geometry leaf；logical sidecar
把 fragment cache/glyph projection/receipt validation 下沉到 50/152/59 行 leaf 后为 762 行，rich builder 523 行，builder tests
327 行，均低于
800 行 review warning。本轮仅完成 rustfmt
parser/diff/static contract 检查，
未执行 managed Cargo、profile、功耗或 framebuffer。状态为
`horizontal_inline_external_cluster_artifact_implemented /
inline_empty_glyph_slice_and_geometry_receipt_implemented /
logical_virtual_glyph_projection_owner_split_implemented /
logical_virtual_fragment_validation_owner_split_implemented /
typed_virtual_fragment_role_implemented / virtual_receipt_linear_capture_implemented /
vertical_rich_and_external_block_canonical_artifact_implemented / static_checks_complete /
vertical_rich_ellipsis_virtual_artifact_implemented /
vertical_rich_soft_hyphen_virtual_artifact_implemented / managed_validation_pending`。ordinary styled VerticalRl 与
无 generated marker 的 inline external block 已复用
canonical vertical provider；zero-width generated anchor 按 typed role 校验 U+2026 ellipsis 或 ASCII discretionary
hyphen，后者还必须保留 replaced U+00AD range。动态 widget、locale dictionary、31-sample
p50/p95/p99、allocation/RSS、功耗、真实 WGPU/PNG、commit 与 WeCom 仍开放；`RTS-P1-032` 不关闭。

### 12.24 Compiled-rich typed linear renderer publication 非验收实现状态

current-source 复审发现 renderer 对每个 rich paint run 先 `.find` layout line，再 `.find` composite glyph run，
最坏复杂度为 `O(R^2)`；这与“run directory”名称和 Unreal shaped-run/block publication 模型均不一致。
`text_paint_runs_from_resolved_layout` 已保证按 `layout.lines -> line.runs` 展平，因此 renderer 现一次单调扫描布局，
runtime composite 用 exact directory index O(1) 返回 slice。每个 cursor 同时校验 text/source/visual identity，
cardinality/order 漂移整批 incomplete，slice start/end 与 artifact line snapshot 也 fail closed。

route 类型区分 canonical `Artifact`、generated-marker compatibility `VisualOnly` 与
`Rejected(Missing|Stale|Incomplete)`，并进入现有低基数 report/profile aggregate。正常 rich profiling contract
从“每 paint run 一次 fallback shape”改为“artifact run 数等于 materialized text run，fallback span 为 0”。
Rejected text run 仅在 resolved line 证明 exact source-isomorphic 时允许 renderer reshape；non-isomorphic
missing/stale/incomplete 不发布 batch，避免猜测 BiDi/virtual-marker/source mapping。生产
renderer root 从 816 行降到 754，rich leaf 485，typed route owner 727，runtime composite 339；测试根从旧
855 行降到 651，route child 384。静态复杂度为 `O(lines + runs)`，本轮未执行 managed Cargo 或性能采样，因此状态为
`rich_renderer_typed_linear_run_directory_implemented / compiled_rich_route_receipt_implemented /
rich_nonisomorphic_rejection_fail_closed_implemented /
static_checks_complete / managed_validation_pending`。31-sample p50/p95/p99、allocation/RSS、功耗、Unreal 同负载
经验值、真实 WGPU/PNG、commit 与 WeCom 仍开放；不能据此声称瓶颈已经通过动态数据消失。

2026-08-31 current-source 范围校正：12.24 的 `O(lines + runs)` 证明只属于 glyph route directory。
接口层 paint-run frame 仍对每个 run 重复 line grapheme validation/count/prefix sum；inline renderer 随后再次从头
查 line、查 run、统计 prefix grapheme 并求和 advance。该路径最坏仍可达 `O(R * G + I * L + I * G)`，因此不能把
route directory 的收敛外推到完整 rich paint。profiling feature 现发布 7 个固定低基数 inline work/frame-agreement
回执，普通 build 为零尺寸 aggregate；Interface exact production helper 的 Windows release-only ignored
benchmark 已静态覆盖 1/100/1k/10k runs、3 次 warm-up 与 31 个原始 timing/RSS 样本输出，但尚未运行。
renderer 独立 harness 也已静态覆盖 dense LTR/RTL/VerticalRl inline 与 1/100/1k hard lines，并将 counter
capture 放在计时外；该 harness 同样尚未运行。
在 E 盘完整 31-sample baseline 前不实施结构优化。Unreal Slate 的
`ILayoutBlock` location/size 单一 owner 是目标边界，详细计划见
[`../../zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md`](../../zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md)。

### 12.25 VerticalRl typed marker admission 非验收实现状态

vertical rich sidecar 不再用“存在 generated anchor”作为整行拒绝条件。candidate/logical owner 显式发布
`Ellipsis | DiscretionaryHyphen | Justification` role；builder 仅放行 exact U+2026 `Ellipsis` 与 exact ASCII `-`
`DiscretionaryHyphen`，且后者必须携非空 replaced U+00AD range。ordinary styled text、external block、ellipsis 与
soft hyphen 因而可在同一 canonical vertical artifact 中发布；untyped/justification/mismatched marker fail closed。

测试分别固定 vertical ellipsis/soft-hyphen canonical virtual glyph、role identity、unsupported justification
negative artifact 与 renderer canonical route。生产 rich builder 为 523 行，私有 builder tests 为 327 行；logical
sidecar 的 50/152/59 行 fragment/projection/validation leaf 把主 owner 收敛到 762 行。本轮仅完成 rustfmt parser/diff/static
contract 检查，未执行 managed Cargo、profile、功耗或 WGPU/PNG。
状态为 `vertical_rich_ellipsis_virtual_artifact_implemented /
vertical_rich_soft_hyphen_virtual_artifact_implemented / typed_virtual_fragment_role_implemented /
logical_virtual_glyph_projection_owner_split_implemented / virtual_receipt_linear_capture_implemented /
logical_virtual_fragment_validation_owner_split_implemented /
static_checks_complete /
managed_validation_pending`。

### 12.26 Vertical generated-fragment owner 重审与非验收实现状态

实现前 current-source 逐层追踪发现，vertical rich 先调用 `apply_visual_order_with_advances`，随后才插入 ellipsis，且
`vertical.rs` 把结果包装为 `LayoutWithoutArtifact::without_retained_fragments`。soft hyphen 同时通过
`append_source_owned_discretionary_hyphen` 把显示 `-` 绑定到非空 U+00AD source range。这三个行为使 vertical
marker 无法拥有 horizontal 已有的 logical display sequence、typed replacement receipt 和 display-owned UAX#9
顺序；仅在 artifact builder 放宽 U+2026 字符过滤不能形成真实闭环。

本地 Unreal 参考采用 `FLineModel -> FRunModel -> ILayoutBlock` owner 链：run model 保存测量/文本范围，flow
阶段通过 `FRunModel::CreateBlock` 物化 block，hit-test 与 paint 继续消费同一 block。这里借鉴的是“generated
display fragment 必须在 layout owner 被类型化并保留到 paint”的边界，而不是声称 Unreal 已提供同名 vertical
soft-hyphen 策略。

实施顺序固定为：

1. 为 candidate receipt 与 logical sidecar 增加低基数 typed virtual role（ellipsis、discretionary hyphen、
   justification），禁止 artifact builder 按显示字符猜 marker 语义。
2. vertical rich 在 ellipsis/soft-hyphen materialization 后、UAX#9 前 capture sequence，并让
   `LayoutWithoutArtifact` 保留与最终可见 column 一一对应的 sidecar。
3. 删除 source-owned `-` compatibility 分支；U+00AD 始终保留在 replaced-source receipt，display run 使用
   zero-width anchor。vertical provider 只放行明确支持的 ellipsis/discretionary-hyphen role，justification
   继续 fail closed。
4. 单元测试固定 role identity、垂直 soft-hyphen canonical virtual glyph、U+2026 省略号、external block、BiDi
   与 renderer typed artifact route；后续 managed lane 再执行 Cargo、31-sample p50/p95/p99、allocation/RSS、
   功耗和真实 WGPU/PNG。

实现前复杂度审计同时发现 `capture_with_external_source_ranges` 为每个 grapheme 调用
`virtual_source_receipts.iter().find(...)`，最坏为 `O(clusters * virtual receipts)`。本切片必须把它改为与
external-range cursor 相同的有序 receipt cursor，并对乱序/重叠输入 fail closed。

上述顺序已落地：source-owned hyphen append 分支删除；vertical rich 先按 current run style 度量并物化
ellipsis/soft-hyphen，再 capture typed sidecar、执行 UAX#9，并通过 `LayoutWithoutArtifact` 只保留最终可见 columns
对应的 sequence。artifact gate 消费 role 而非猜测字符语义，renderer 回归要求 soft-hyphen/ellipsis 均走
canonical `Artifact` route。receipt capture 已改为有序游标，并对乱序、重叠或越界 receipt fail closed。

capture、role publication、UAX#9、glyph projection 与 run-directory 各为单调遍历，静态复杂度为
`O(clusters + virtual receipts + external ranges + glyphs + runs)`；每个 cluster 只增加一个紧凑枚举字段，不新增
paragraph map、renderer reshape 或第二次 shaping。本轮仅完成 rustfmt/diff/source contract 检查；managed Cargo、
profile/RSS/power 与 WGPU/PNG 未执行。本节状态为
`vertical_generated_fragment_architecture_review_complete / typed_virtual_fragment_role_implemented /
vertical_rich_generated_fragment_retention_implemented /
vertical_rich_soft_hyphen_virtual_artifact_implemented / virtual_receipt_linear_capture_implemented /
static_checks_complete / managed_profile_pending`。

### 12.27 Dynamic inline widget owner 架构重审与实施顺序

current-source 复审确认当前 `[widget=id|widthxheight]` 仍不是 Unreal `FSlateWidgetRun` 对等能力：compiled
run 只保存裸 `u64 + size`，文本布局把 U+FFFC 当 external block，但 graphics rich renderer 最终只在该 frame
画一个实心矩形。该路径没有真实 UI child、surface/owner 约束、生命周期、input/focus/accessibility，也不能在
widget 变化时使对应 run layout 失效。继续在 renderer 增加 batch 或全局 `id -> widget` registry 会把 paint
层错误提升为第二套 UI 树，禁止采用。

本地 Unreal `SlateWidgetRun.h/.cpp` 的权威边界是：run 强持有 `SWidget`，measure 使用显式 Size 或
`GetDesiredSize()`，layout block 通过 `ArrangeChildren` 安排真实 child，paint 调用 child 的正常 `Paint`，并在
desired size 变化时只 dirty 该 run。Zircon MVP 按现有标记的显式 `widthxheight` 采用对应的固定-size 分支：

1. `id` 只允许绑定富文本 owner 的直接 UI child；UI tree 是生命周期、事件、焦点、无障碍和绘制 owner，文本
   artifact 不保存跨 surface 裸指针，也不建立进程全局 registry。
2. rich layout 继续以显式 `widthxheight` 度量 external block，并从 canonical layout/compiled artifact 发布
   `child UiNodeId -> absolute UiFrame` 放置回执。无效、重复、非直接 child 或被 overflow 省略的绑定 fail closed，
   对应 child 不参加 paint/hit-test。
3. UI measure/arrange 在同一帧消费该回执；绑定 child 走正常 subtree arrange/extract/render/input/a11y，graphics
   rich renderer 删除 widget 实心矩形。不得在 renderer 重新 parse markup、反查 UI tree 或复制 child draw list。
4. 绑定枚举、layout line/run 匹配和 child arrangement 必须是单调遍历，静态上限
   `O(tree nodes + rich runs + direct children)`；不得为每个 child 扫描全部 runs，且不得新增 paragraph-sized map。
5. 先补 exact binding、visible/ellipsized、duplicate/missing child、真实 child render/hit frame 与 renderer
   no-placeholder 回归；managed Cargo、31-sample p50/p95/p99、allocation/RSS、功耗与真实 WGPU/PNG 后续执行。

固定尺寸 direct-child 分支现已实现：compiled widget source range 通过 canonical resolved lines/runs
单调投影为绝对 frame，布局阶段只遍历本次 arrangement roots 的受影响子树，并以哈希 membership 将绑定映射到
直接 child；真实 child 继续走普通 subtree arrange、render extract、hit-test、focus 与 accessibility。重复、缺失、
跨 parent 或 overflow-omitted 绑定清空 child subtree geometry；graphics rich renderer 的实心矩形分支已经删除。
静态复杂度为 `O(affected tree nodes + rich runs + graphemes + direct children)`，没有进程全局 registry、
per-child 全 run 扫描或 paragraph-sized 持久映射。源码回归已覆盖 exact frame/render/hit、duplicate、missing、
omitted 与 renderer no-placeholder；rustfmt、定向 whitespace 和 `git diff --check` 通过。

当前状态为 `dynamic_inline_widget_architecture_review_complete /
fixed_size_direct_child_inline_widget_implemented / renderer_widget_placeholder_removed /
typed_owner_local_widget_slot_implemented / current_tree_binding_nonretained /
incremental_arrangement_root_bounded / static_checks_complete / managed_validation_pending`；desired-size 分支、
retained surface-session/node-incarnation lease、G27/G28 完整资格、managed Cargo/profile/power/WGPU/PNG、commit 与 WeCom
均未关闭，`RTS-P1-032` 继续保持 open。

### 12.28 UI text cache byte-residency pre-cap instrumentation

current-source 重审确认 `TextMeasureCache` 的 4096-entry LRU 与 `TextLayoutCache` 的 2048-entry LRU 只能约束
对象数量，无法说明长 source、rich run、line advance 与 editable/composition DTO 的常驻规模。直接按平均字符串长度
设 byte cap 会把未测假设固化为淘汰策略，并且若每个 layout entry 都重复归因共享 glyph artifact，还会把并不随该
entry 逐出而释放的内存重复计费。因此本切片先建立无行为变化的测量基线，暂不调整 admission、LRU 顺序或上限。

两个通用 cache owner 现维护 `estimated_bytes` 与 frame 内 `peak_estimated_bytes`，insert、同 key update、LRU
eviction、finish trim 与 clear 都在同一最低 owner 原子更新。UI measure/layout admission 使用 weighted insert：measure
计入 key 的 source/style-owned 字符串，layout 还计入 serializable resolved-layout DTO 自有的 line/run text、advance、
editable text 与 composition clause。profile 仅发布四个低基数聚合计数：measure/layout 的当前与峰值
`cache_dto_source_*_bytes`。

该数字明确是 source/DTO-owned heap 的保守下界，不是 allocator RSS，也不包含 opaque/shared glyph artifact、Arc
共享分摊、哈希桶 capacity 或 allocator overhead。共享 artifact 必须在唯一 residency owner 能提供不重复的引用/释放
回执后再归因，不能把 `Arc::strong_count` 当稳定预算依据。源码回归覆盖 insert/update/evict/clear/peak，rustfmt、
定向 whitespace、owner/source guard 与 scoped `git diff --check` 已通过；managed Cargo、规模 profile、RSS/power、
artifact owner 归因、byte cap/admission/eviction 调优与真实 WGPU/PNG 均未执行。

本节状态为 `cache_dto_source_residency_receipt_implemented /
cache_update_evict_clear_accounting_implemented / static_checks_complete /
artifact_residency_and_byte_cap_open / managed_profile_pending`；`RTS-P1-045` 继续保持 open。

### 12.29 Oversized shaping work production receipt before scheduler cutover

current-source 调用链确认 64 KiB `TextShapingWorkBudget` 原先只有测试和计划引用；retained
`SharedTextLayoutSession` 在 shaped-cache miss 后无条件同步进入 canonical backend，parallel prewarm 则把去重后的
pending job 交给 `parallel_for` 并在调用方 join。直接以 64 KiB 分割 source 会破坏 Arabic/Indic/emoji/ligature 的
上下文和 cluster identity；直接返回 Deferred 又会改变现有同步 UI layout 合同。因此本切片先发布实际 backend
work 的规模回执，不改变算法或调度。

本地 Unreal `FTextLayout` 参考显示稳定 `FLineModel` 持有 dirty state 和 estimated geometry，按需物化
`FLineView`；`FSlateFontCache` 的 range shaping API 明确保留范围外文本作为 shaping context。采用的是“稳定 source
owner + 按需 materialization + 上下文保留”边界，不把任意字节阈值当 line/run/cluster 边界，也不声称 Unreal 有
同名 64 KiB 策略。

`TextShapingWorkReport` 现在由 budget owner 记录四个低基数聚合值：阈值内请求数、超阈值但仍同步完成的请求数、
同步输入总字节与最大单请求字节。同步 session 只在 canonical cache miss 后记录；parallel prewarm 只对去重后真正
进入 pending work set 的请求记录，cache hit、batch duplicate 和 invalid request 均不计费。session 每帧重置并合并
parallel receipt；batch 与 UI profile 分别发布 aggregate counter，禁止携带 source 文本。

回归覆盖 budget 分类/合并、session miss 后一次计费且 hit 不重复、parallel duplicate 只计唯一 work，以及 4-byte
测试阈值下完整字符串仍被 shape 并标记为 oversized synchronous。`layout_session.rs` 的新增回归已拆到测试叶子，
owner 根保持低于 800 行。rustfmt、scoped diff-check、call-site/report/private-boundary 与文件预算静态检查通过；
managed Cargo、1/100/1k/10k 的 31-sample profile、CPU/allocation/RSS/power、typed defer/cancel、真实 WGPU/PNG 均未执行。

本节状态为 `shaping_work_budget_production_receipt_implemented /
cache_miss_and_unique_pending_attribution_implemented / source_semantics_preserved /
algorithm_unchanged / static_checks_complete / typed_defer_cancel_and_managed_profile_pending`；
`RTS-P0-001`、`RTS-P1-015` 与 `RTS-P1-016` 继续保持 open。

### 12.30 Direct backend typed failure current-source correction

对 `RTS-P1-018` 的 current-source 复审确认原“大量 `Option` 触发无差别 fallback”描述已经陈旧。
`shape_horizontal_request`、`shape_vertical_request`、`shape_horizontal_run` 与 `shape_vertical_run` 均返回
typed `Result`；font variation/bytes/index、face parse、non-empty glyph output、source slice、cluster offset、metric finite
和 logical-order invariant 会被映射为 `DirectShapeError`，再发布包含 code/phase/range/face/dependency/disposition/
budget 的稳定 receipt。

fallback policy 由 receipt 决定，而不是由 `None` 决定：只有 horizontal orientation 的 backend/font-face/glyph
capability failure 允许 alternate Cosmic backend；vertical failure 以及 source range、itemization、BiDi 和 font-source
budget failure均 terminal。Cosmic fallback 后仍会验证 raster face，不能用无 face glyph 伪装成功。

剩余两个 `Language::from_str(...).ok()` 不构成可达失败擦除。锁定依赖 RustyBuzz 0.20.1 的实现只拒绝空字符串，
而 `BackendShapeRequest::canonicalized` 已在 backend 前过滤空语言、拒绝非法 BCP-47，并把 request 标记为 canonical；
因此该 projection 只表达 optional language，不是 fallback trigger。新增一个永远不可达的 backend language error 会复制
上游 `InvalidLanguage` owner，禁止这样处理。

现有回归覆盖 face/range/policy receipt、horizontal/vertical alternate policy、BiDi 与 font-source budget terminal、
direct/backend 12-code 唯一低基数 label，以及 non-Ready 不进入 cache。本次只做 current-source/依赖源码/调用链静态校准，没有新增
运行时代码；managed fault injection、Cargo、corpus、profile/power 与 WGPU/PNG 仍待后续。状态为
`typed_direct_backend_failure_receipt_implemented / policy_scoped_horizontal_alternate_backend /
vertical_and_invariant_fail_closed / canonical_language_projection_proven /
static_checks_complete / managed_fault_injection_pending`。

### 12.31 Break-safety retention into final-line advance index

current-source 重审发现 `ShapedGlyphBreakSafety` 虽已由 direct backend 发布在 cluster 头，但
`MeasuredGlyphCluster` 与 `GraphemeAdvanceIndex` 之前丢弃该字段，final-line owner 因而无法区分 known-safe、
requires-reshape 与 unknown，`RTS-P1-017` 的消费前置并未真正闭环。本切片把 receipt 保留到 crate-private
measurement artifact；缺失或非 cluster-head provenance 显式降为 `Unknown`。由原始 metric 合成的 compatibility
cluster 同样为 `Unknown`，hard line separator 与 inline external block 则作为引擎拥有的语义边界发布 `Safe`。

`GraphemeAdvanceIndex` 现在可查询精确 source boundary：文档端点为 `Safe`，落在 backend cluster 内部必为
`RequiresReshape`，同一起点存在多条 receipt 时按 `RequiresReshape > Unknown > Safe` 保守合并。profiling build
在现有 boundary correction 选出候选 ranges 后，用一个单调 cluster cursor 聚合四个低基数计数：候选 range 数、
safe、requires-reshape 与 unknown boundary 数。该观察路径为 `O(candidate boundaries + measured clusters)`，不分配
boundary Vec、不发布 source 文本，也不产生 per-boundary event；普通非 profiling build 不执行额外查询。

本切片没有改变 UAX#14 opportunity、atomic cluster coalescing、固定 8-grapheme correction、layout 输出或 shape-call
数量。源码回归覆盖 multi-grapheme cluster receipt 保留，以及文档端点、atomic 内部、known-safe/unknown 起点和
单调聚合计数；Rust 2024 rustfmt、scoped diff-check、全部构造点与四个 profile 名静态检查通过。managed Cargo、
official corpus、候选分布、exact two-sided reshape、shape-call/p50/p95/p99、RSS/功耗、WGPU/PNG 均未执行。

状态为 `break_safety_measurement_retention_implemented /
monotonic_candidate_boundary_profile_implemented / algorithm_unchanged / static_checks_complete /
exact_final_line_reshape_and_managed_profile_pending`；`RTS-P1-017` 与 `RTS-P1-035` 继续保持 open。

### 12.32 TransformOrRotate comparison-shape research and pre-optimization receipt

`RTS-P1-019` 的 current-source 调用链已从 Unicode VO owner 重审到 backend。itemizer 对每个 grapheme 解析
`Upright/Sideways/TransformOrRotate`，再按连续相同 face、instance、BiDi direction/level、script 与 orientation
合并为 `LogicalSegment`。vertical direct 对 Upright/Tr segment 执行一次 TTB/BTT shape；只有 Tr segment 再以
相同 text/direction/script/language/非竖排 feature、显式关闭 `vert/vrt2` 执行 comparison。因此 comparison 次数是
`O(Tr logical segments)`，最坏 `O(graphemes)`，不是每 glyph 固定执行两次。

参考引擎没有给出可直接照抄的 provenance 捷径。本地 Unreal `SlateTextShaper.cpp` 以一次 `hb_shape` 构造 retained
`FShapedGlyphSequence`，但该路径只设置 LTR/RTL 和 `kern/liga`，没有 TTB/BTT、`vert/vrt2` 或 Tr substitution
receipt；它支持的是 sequence owner/一次 shaping 的总体方向，不能证明 Zircon 可删除 correctness comparison。
本地 Godot Advanced TextServer 会为 vertical BiDi run 选择 TTB/BTT，并以 font/span features 单次 `hb_shape`，
同时公开 `vert/vrt2` feature tag；其 glyph DTO 也没有发布“该 Tr cluster 实际发生 substitution”的等价 receipt。
在 RustyBuzz 当前 API 不提供 lookup execution trace 的条件下，enabled/disabled output 差分仍是 Zircon 现有唯一
可证明的 per-cluster decision source。

为先测量再优化，原有 direct backend-call TLS 已从临界 797 行的 `cosmic.rs` 下沉到
`cosmic/direct_profile.rs`。`profiling` capture active 或 Tracy build 时，每个 direct request 只在 TLS 累加整数，成功发布 run 后一次性发送
8 个固定低基数 counter；失败并进入 terminal/alternate policy 时丢弃局部报告。新增四项分别记录 Tr comparison
call、完整 segment input bytes、disabled-output glyphs 与 changed clusters。没有 per-segment event、source label、
`Instant` 或全局 profiler lock，普通非 profiling build 以及 CPU capture inactive 时不创建报告。

managed scale harness 已从 Latin/CJK/RTL/ligature/vertical-CJK 扩为额外 `vertical_tr` workload，并要求非 Tr 四项
为零、Tr comparison 覆盖完整输入且调用上界保持线性。规模/计数测试从 774 行测试根拆到 293 行专责叶子，测试根
降至 523 行；`cosmic.rs` 降至 722 行。Rust 2024 rustfmt、scoped diff-check、cfg 对称、8 个唯一 counter 名和
文件预算静态检查通过。managed Cargo、Tr 1/100/1k/10k 的 31-sample counter/p50/p95、allocation/RSS、功耗、
真实 WGPU/PNG 尚未执行，因此没有瓶颈消失、Unreal 耗时接近或算法最优声明。

状态为 `vertical_substitution_comparison_receipt_implemented / request_local_capture_only_aggregation_implemented /
algorithm_unchanged / static_checks_complete / managed_profile_pending`；`RTS-P1-019` 与 `RTS-P1-020` 保持 open。

### 12.33 Common/Inherited script-run current-source correction

对 `RTS-P2-001` 的 current-source 与锁定依赖源码复审确认，原 `pending_common_start/end` 问题描述已经失效。
当前 `script_segment.rs` 没有 pending Common 状态；paragraph analysis 以 `unicode-script 0.5.8` 的定长
`ScriptExtension` 位集做单次前向交集。该依赖把 Common/Inherited 表示为覆盖全部已知 script bits 的集合，因而
前导 Common/Inherited 会在首个 specific script 到来时收敛到该脚本；行内和尾随 Common 会继续属于前一个兼容
script run；全 Common 文本保持 `Zyyy`。不相容的两个 specific scripts 才产生边界。

本地 Godot `modules/text_server_adv/script_iterator.cpp` 同样将 Common/Inherited 视为与任意脚本兼容，并在首个
specific script 到来时回填 opening-bracket context。Zircon 在此基础上还消费 Script_Extensions 多候选交集与完整
BiDi paired-bracket 数据；因此新增 `CommonResolutionState` 会复制现有集合状态、增加分支，并可能让 fallback 与
HarfBuzz script owner 再次分叉，不能作为 current-source 修复。

新增聚焦回归固定四类策略：前导 Common+Inherited 归首个 specific script、跨脚本中间标点归前一个 run、尾随
标点归前一个 run，以及纯 Common 文本保持单一 `Zyyy` segment。生产算法、分配次数和 `O(codepoints + runs +
bracket_depth)` 复杂度均未改变；Rust 2024 rustfmt 与源码语义检查完成，managed Cargo/text test 尚未执行。
状态为 `stale_pending_common_finding_corrected / script_extension_policy_regressions_added /
production_algorithm_unchanged / static_checks_complete / managed_text_test_pending`；`RTS-P2-001` 的实现项完成，
其动态验收保持开放。

### 12.34 Unified vertical cluster decision receipt

`RTS-P1-020` 的 current-source 复审确认，竖排决策此前分散在四处：itemizer 暂存 Unicode VO orientation，
backend 的 `vertical_substituted` 在投影后丢失，direct 只留下 rotation，selected face/instance 则是另一组 glyph
字段。布局、renderer 与诊断因此无法区分 Unicode sideways、强制 sideways、Tr substitution 成功、无 substitution
旋转 fallback，以及缺 backend provenance 的 compatibility 路径。

本切片在 cluster-head `ShapedGlyphClusterFlags` 中保留紧凑 `TextVerticalGlyphDecisionBasis`，包含 orientation、
实际提交给 backend 的有效 `vert/vrt2` feature set、`NotChecked/NotObserved/Observed` substitution 状态和 typed
fallback reason。`ShapedGlyph::vertical_glyph_decision()` 以该 basis 和同一 glyph 已有 rotation、selected
`FontFaceId/InstancedFaceId` 组成完整 `VerticalGlyphDecision`，不复制字体身份。neutral projection 把 basis 原样
传入 `TextGlyphFlags`，`TextGlyph::vertical_glyph_decision()` 再与 generation-qualified font handles/rotation 合成最终
renderer-neutral receipt；renderer 不重新计算 Unicode VO 或查询字体库。

direct vertical backend 从同一 `projected_vertical_features` 得到 effective set。Tr comparison 的 cluster output
差分直接成为 `Observed/NotObserved`；no-substitution 旋转使用 `NoVerticalSubstitution`，Unicode/forced sideways 分别
使用独立原因。compatibility/Cosmic 路径仍可发布 Unicode orientation 与有效请求 set，但 Tr 必须标为
`BackendProvenanceUnavailable + NotChecked`，不能伪装 direct proof。hard separator 显式为
`NonRenderingControl`。RustyBuzz 当前不公开具体 GSUB lookup trace；当 `vert` 与 `vrt2` 同时有效时 receipt 只陈述
可证明的 feature set，不猜测单一 chosen tag。

该路径不新增 shape call、paragraph pass、字体查询或 heap allocation；backend call 数仍为 2 个源码点，完整
decision accessor 为常量时间。聚焦回归覆盖 feature override set、compat projection、Tr observed/not-observed 与
neutral font-handle projection。Rust 2024 rustfmt、全仓构造点、serde default、scoped diff 与文件预算静态检查通过；
managed Cargo、真实字体组合、serialization round-trip、per-glyph size/RSS/profile/power/WGPU/PNG 尚未执行。
状态为 `vertical_cluster_decision_basis_implemented /
direct_feature_set_and_substitution_provenance_retained / neutral_projection_preserved /
compatibility_unknown_explicit / static_checks_complete / managed_validation_pending`；`RTS-P1-020` 的非验收实现完成。

### 12.35 Horizontal run-local alternate-backend composition plan

`RTS-P1-021` 的 current-source 重审确认，当前 direct horizontal 在首个 logical segment backend error 时丢弃此前
成功结果，随后 Cosmic 对完整 request 再 shape。Cosmic 输出虽最终按 source range 归一化，但它只发布完整 hard-line
结果；直接截取 glyph 会遗漏跨边界 cluster 资格、selected-face line envelope、raw metric span 和 failure ownership。
因此 12.8 的“不得直接局部拼接”结论继续成立，先建立 composition artifact 再切策略。

本地 Unreal `SlateTextShaper.cpp` 的可采用标准是 retained sequence ownership：先按 BiDi direction，再按 font face
和 script 构造 `FHarfBuzzTextSequenceEntry::FSubSequenceEntry`，每个 subsequence 独立 `hb_shape`，最后由同一
`FShapedGlyphSequence` owner 收集 face、source index、glyph count 与 direction。Unreal 没有 Zircon 的 typed alternate
backend failure，也不提供可照抄的局部 backend recovery；因此 Zircon 复用 subsequence/sequence owner 思路，但必须
额外保留 failure range、qualification 和 fail-closed fallback。

实施顺序冻结如下：

1. **M0 direct attempt artifact**：horizontal direct 完成全部可达 logical segments；成功 segment 保持原 glyph/source/
   face/metric span，失败 segment 形成排序、互不重叠的 hole，并保留原 `DirectShapeError`。itemization、BiDi、source
   与 budget terminal error 仍由唯一 receipt owner 判定，不能转成 hole。
2. **M1 candidate qualification/composition**：沿现有 Cosmic 完整 request 只生成一次 candidate。只有 candidate glyph
   source range 完全落入某个 hole、没有 glyph 跨越 hole 边界、hard-line topology/Unicode snapshot/request identity
   全等且每个非空 hole 有 candidate glyph 时，才用 candidate 填 hole。glyph 最终按 canonical source order 合并并
   重新定位；direction/bidi level/script/font/cluster receipt 均来自各自 backend，不由 compositor 猜测。
3. **M1 metric rebuild**：最终 line baseline/height、raw line metrics 与 contiguous selected-face metric spans 必须从
   合成后的实际 face IDs 重新聚合，不能沿用 direct hole 前的 envelope，也不能把 Cosmic line box 整体覆盖 direct
   line。无法形成完整 span coverage 时 sidecar 显式不可用。
4. **M2 receipt/profile**：hybrid artifact 保留 alternate ranges 与首因 receipt；profiling 只发布 request/hole/direct-
   glyph/alternate-glyph/rejected-composition 低基数聚合。cache 只接收完整 qualified run，partial attempt 永不 admission。
5. **安全回退**：任何 identity、topology、range、cluster 或 metric 资格失败均使用已经生成的 whole-request Cosmic
   candidate；若 candidate 自身失败则保留原 direct receipt。vertical 继续 terminal，renderer/UI 不获得组合策略。

目标复杂度为 direct `O(segments + glyphs)`、candidate qualification/merge `O(lines + holes + glyphs)`；hole lookup 使用
单调 cursor，不允许 per-glyph full-hole scan、重复 shape candidate 或 renderer-local reshape。M0-M2 实现前不声明
run-local recovery、性能改善或 cluster 语义统一。

2026-08-26 M0-M2 非验收实现记录：`horizontal/direct.rs` 现在继续处理失败 segment 之后的可达工作并形成有序 hole；
`horizontal/composition.rs` 以跨行单调 cursor 校验 direct/Cosmic source order、run/line identity、hole containment 与
coverage，跨 hole cluster、空 hole、direct overlap、非单调字形或拓扑不一致均保留 whole candidate。成功路径仅合并
hole 内 glyph，随后从最终 face IDs 重建 line envelope、raw metrics 与 contiguous metric spans，复杂度为
`O(lines + holes + glyphs)`，未增加 renderer/UI 策略。

纯数据 failure receipt 已下沉到 model，hybrid `ShapedGlyphRun` 只在成功组合时按需分配
`TextHorizontalCompositionReceipt`，保存 owning-source 绝对 alternate ranges 与首因 receipt；其 Box/Vec 常驻量进入
shaped-cache byte estimate。request-local profile 在进入 Cosmic 前 detach，最终一次性发布 candidate request/input
bytes/hole/retained-direct-glyph/selected-alternate-glyph/rejected-composition/direct-backend-call 聚合，没有 source label、
逐 glyph event 或全局热路径锁。`cosmic.rs` 已按所有权拆到 751 行，100 行恢复叶与 633 行组合叶均低于 800 行门禁。

静态证据为 Rust 2024 rustfmt、scoped diff-check、全 `zircon_runtime/src` 的 `ShapedGlyphRun` 构造字段扫描、唯一 failure
classification owner、单调 hole cursor 与文件规模检查。没有执行 Cargo、fault injection、1/100/1k/10k 31-sample、
allocation/RSS、功耗、真实 WGPU 或 PNG；完整 Cosmic candidate 仍会生成一次，因此不能宣称后端工作减少、瓶颈消失、
达到最优规模或接近 Unreal 耗时。当前状态为
`direct_partial_attempt_implemented / source_ordered_hybrid_composition_implemented /
selected_face_metric_rebuild_implemented / hybrid_artifact_receipt_and_profile_implemented /
fail_closed_whole_candidate_retained / static_checks_complete / managed_validation_pending`。

### 12.36 Source lifetime/range lease pre-optimization research

`RTS-P1-022` 的 current-source 调研确认，`BackendShapeRequest` 只在 `with_source_owner` 获得与 request text 完全相等的
`Arc<str>` 时复用；普通同步 shaping 会在 `ShapedGlyphRun` 最终化时分配 exact source。parallel
`horizontal_paragraphs` 先从 `&str` 切 hard line，再由 `Into<Arc<str>>` 为每段物化 owner；prewarm request 与最终 run
虽可共享该段 Arc，但没有 document snapshot + subrange lease。cache exact collision guard、measure grapheme projection、
line text 和 glyph artifact qualification 又直接读取 run 的 exact source，因此不能只替换字段而不重建坐标与寿命合同。

本地 Unreal `FSlateTextShaper` 接受外部 `TCHAR* + start + len`，`FShapedGlyphSequence` 只保存
`FSourceTextRange`、glyph `SourceIndex` 与 `SourceIndicesToGlyphData`，`GetAllocatedSize` 不包含源字符串；文本寿命属于外部
layout/string owner。这支持 Zircon 的目标方向是 document-revision snapshot + range lease，而不是让每个 shaped run 拥有
一份字符串。但 Zircon cache 还必须按唯一 owner 归因内存，禁止把同一完整 document Arc 对每个 lease 重复计费，也禁止
以 `Arc::strong_count` 决定 admission。

预优化 instrumentation 已实现但算法未变：shaped artifact 边界发布 source materialization、exact-owner reuse、allocation
count/bytes；parallel batch 发布 source lease count、unique Arc owner count、leased bytes 与 unique owner bytes。所有计数
均为请求级低基数聚合，不含 source label，也不进入 glyph/cluster 循环。managed profile 必须覆盖同步/parallel、cold/warm、
1/100/1k/10k hard lines、稳定文档/单行编辑和可控 hybrid failure，每组 31 samples，并同时记录 shaped-cache hit/miss、
current/peak bytes、allocation/RSS、p50/p95/p99 与有效传感器下的等负载功耗。

只有数据证明 source materialization 或重复常驻是主导项，才进入两阶段硬切：M1 建立 immutable document snapshot、validated
owner range、absolute source origin 与 wire-compatible slice serialization；M2 让 parallel/cache/shaped artifact 使用 lease，
cache 以唯一 owner registry 一次归因并在 update/evict/clear 精确减账。per-glyph SoA/cluster table 属于独立数据布局决策，
必须另测 glyph count、结构 size、cache density 与 renderer access，不与 source lifetime 一次迁移。当前状态为
`source_lifetime_architecture_research_complete / unreal_external_text_owner_confirmed /
source_materialization_and_batch_owner_instrumentation_implemented / algorithm_unchanged /
static_checks_complete / managed_profile_pending`；没有 source allocation/RSS/timing/power 数据，不授权 lease/SoA 优化。

### 12.37 Ephemeral cache hash and stable artifact digest boundary

`RTS-P1-023` 的 current-source 重审把原问题拆成两个边界。shaped cache、parallel pending 去重、rich compile
cache、UI measure/layout cache、physical-line fragment retention 和 `TextDocumentKey(owner, revision)` 的哈希都只在
当前进程内选桶；命中后仍比较完整 style/range/generation/Unicode key，并对 source-backed entry 做 exact source
比较。它们没有 serde、磁盘 codec 或 replay 输出。每 viewport 改算全文 BLAKE3 反而会把当前 `O(1)` document
revision identity 退化成 `O(document bytes)`，不能作为“稳定化”修复。

持久边界只有独立 owner 才需要 deterministic digest。本地 `.zsdf` 已在 v1 codec 中写入 32-byte variation/source
BLAKE3 和 32-byte whole-artifact checksum；格式版本负责解释这些字节。此次把缓存身份硬切为不可序列化、无字节导出的
`EphemeralCacheHash`，并把 SDF generation/offline identity 硬切为 `StableContentDigest`。`DefaultHasher` 只封装在
`EphemeralCacheHasher` 一个模块内，`StableContentDigest` 只允许固定 32-byte 往返；public build-tool inspection 继续
投影原 `[u8; 32]`，v1 header长度、字段顺序、摘要算法和 artifact path字节均未改变。

本地 Unreal 的 `FCachedShapedTextKey::GetTypeHash` 同样只服务 `TMap`，完整 key equality 决定命中；其 shaped sequence
保留 source range/index 而非持久 content hash。Unreal 持久数据另用 Blake/SHA/IO hash owner。这支持“临时查找与持久
身份分层”，不支持把运行时 hash 伪装成跨进程 ID。新增源码契约覆盖同进程相等输入、BLAKE3 byte-exact round trip
及新类型与 `u64/[u8;32]` 等尺寸；Rust 2024 rustfmt、scoped diff-check、DefaultHasher owner和裸生产 hash字段扫描通过。

该切片不改变 hashing复杂度、cache equality/admission、shape次数、SDF字节或renderer行为，也没有新增每帧全文 digest。
managed Cargo、SDF encode/decode golden、cache collision回归、profile/RSS/功耗和WGPU/PNG尚未执行，因此不声明性能改善、
瓶颈消失或动态验收完成。状态为 `ephemeral_cache_hash_type_implemented /
stable_artifact_digest_type_implemented / default_hasher_isolated / sdf_v1_bytes_unchanged /
algorithm_unchanged / static_checks_complete / managed_validation_pending`。

### 12.38 Paragraph, hard-line, shaped-run and layout-line lifetime audit

`RTS-P1-024` 的 current-source 复核先纠正原描述：`SharedTextLayoutSession` 由 `UiTextMeasureCache` 长期持有，内部已经分离
`ShapedRunCache`、`HardLineIndexCache` 与 UI layout/measure cache；相同 shaped key 命中时不会重建 `BidiParagraph` 或
`ParagraphTextAnalysis`。`HardLineIndexCache` 对带 `TextDocumentKey` 的 plain viewport 也已保留共享 source owner，未
keyed 路径才按调用扫描并只返回窗口。

仍需量化的重复分析点有两个：direct horizontal/vertical 与 Cosmic fallback 各自构造 line-break/hard-line 视图，rich
layout 的 `RichAdvanceIndex`、physical-line fragments 与 viewport projection 也有独立 hard-line/cluster 投影 owner。
这些复杂度目前均为输入或可见行线性，但没有证据表明重复分析占据 p95、分配或功耗主导。

本切片只冻结研究矩阵，不引入 retained artifact：plain/rich × direct-success/partial-fallback/terminal × cold/warm ×
1/100/1k/10k hard lines，另测单行编辑、viewport scroll、同一 document revision 多宽度布局。每组 31 samples 记录
analysis construction count、hard-line materialization bytes、line-break opportunity bytes、shaped-cache hit/miss、layout
DTO current/peak、allocation/RSS、p50/p95/p99；fallback 区分 direct partial、terminal Cosmic 和完整 direct。若重复分析不
达主导阈值，保持当前 owner；若确认主导，M1 才设计 document-revision-owned paragraph artifact，包含 source snapshot、
Bidi/script/line-break analysis、hard-line index 与 dirty-range dependency。glyph SoA、source lease、renderer artifact 不
得一并硬切。

本审查没有改变 shape-call、cache admission、line-break policy、dirty invalidation 或 renderer 行为；没有伪造性能数据。
Rust 2024 静态扫描确认 session/cache owner 分层、唯一 hard-line builder 与现有 source identity 约束；managed Cargo、
analysis counters、31-sample profile、RSS/功耗及 WGPU/PNG 仍待执行。状态为
`paragraph_lifetime_architecture_review_complete / duplicate_analysis_instrumentation_deferred /
algorithm_unchanged / static_checks_complete / managed_profile_pending`。

### 12.39 TextLayoutError stable diagnostic catalog

`core/framework/text::TextLayoutError` now publishes two constant projections: `diagnostic_code()`
under the `ZR-TEXT-LAYOUT-*` namespace and `message_key()` under `text.layout.*`. This keeps the
core contract backend-neutral while allowing Editor, telemetry, and localization owners to consume
machine-readable identity without parsing the English `Display` string or retaining implementation
receipts. The mapping is exhaustive and covered by a focused uniqueness/prefix test; it adds no
request allocation or cache state. Status:
`diagnostic_code_catalog_implemented / backend_neutral_boundary_preserved /
focused_behavior_tests_complete / managed_validation_pending`。

### 12.40 Remove the one-member UiTextShaperStack abstraction

Current-source call-graph review found that every production entry constructed `UiTextShaperStack`
only to forward to its sole `UiSharedTextShaper` field. It had no backend collection, capability
ordering, fallback receipt, lifetime state, or cache ownership. The wrapper and its delegation-only
test are removed; public, provider, viewport, measurement, and source-range entrypoints now call the
single shared adapter directly while retaining the `UiTextShaper` contract.

The local Unreal reference has `FSlateFontCache` own one concrete `FSlateTextShaper`; BiDi, script,
face, and HarfBuzz/kerning method decisions live inside that real shaper rather than in a one-member
UI stack facade. Zircon likewise keeps direct/Cosmic composition in Runtime Text shaping owners. The
source guard now rejects reintroduction of `UiTextShaperStack`. No shaping, layout, cache, or render
algorithm changed. Status: `empty_ui_shaper_stack_removed / sole_shared_adapter_preserved /
source_guard_updated / static_checks_complete / managed_validation_pending`。

### 12.41 Serializable DTO and renderer-batch residency receipt

The lifetime audit found two different contracts that must not be changed together. Public
`UiResolvedTextLayout`/`UiShapedText` DTOs are serde-capable boundary values, while
`ScreenSpaceUiTextBatch` is an internal renderer-owned materialization. The layout cache already
accounts owned line/run strings and advance payloads. `ScreenSpaceUiTextPrepareReport` now adds a
separate lower-bound receipt over final native/SDF batches after Auto routing: materialized batch
count, UTF-8 text bytes, and glyph-advance bytes. Each final batch is counted once; no raw text,
hash label, capacity scan, or routing branch is added.

The profile publication remains folder-backed: DTO/input/resolved-batch counters live in
`profile/dto_residency.rs`, resident SDF font failures live in `profile/sdf_residency.rs`, and the
root profile owner is back below the 800-line budget. Counter names and report fields are unchanged
by that extraction.

This does not authorize an `Arc<str>` or range/lease migration. The intermediate `UiTextPaint`
source/run clones and actual serialization boundary still need attribution across plain/rich,
artifact/visual/fallback, cold/warm, 1/100/1k/10k lines, and 31-sample allocation/RSS/timing runs.
Only if duplicate text storage is dominant may an internal non-serializable lease be added with
explicit materialization at the versioned interface boundary. Status:
`layout_dto_and_renderer_batch_residency_receipts_implemented / intermediate_paint_copy_open /
algorithm_unchanged / static_checks_complete / managed_profile_pending`。

### 12.42 Owner-local Runtime Text budget snapshots

The current-source audit rejected a single mutable `TextRuntimeProfile` object. The listed values do
not share semantics: the 8-grapheme boundary context bounds correctness repair, 32 tatweels and five
measurements bound one justified line, 16 entries/32 MiB bound retained hard-line indexing, and the
SDF/page-shadow byte limits bound different render-memory lifetimes. Coupling them would let a cache
tuning change silently alter text correctness or shaping work.

Each effective owner now publishes its own immutable snapshot and the existing profile path projects
them under `text.runtime_budget.*`. Boundary correction exposes per-edge context, maximum reshaped
window, and correction steps. Arabic justification exposes maximum materialized tatweels and fit
measurements alongside its existing requested/probe/candidate-byte/safety/acceptance receipt. The
hard-line report includes effective entry/byte limits plus resident/eviction/oversized state. SDF
scheduler diagnostics include the full configured in-flight, glyph, source-byte, completion-depth,
and completion-byte limits rather than only backlog usage.

Bitmap page-shadow now reports resident pages/bytes, the effective 32 MiB ceiling, and cumulative
budget rejections through the native prepare report. A known rejected page is not retried again by a
patch in the same commit. Profile projection lives in the 57-line
`prepare_report/profile/runtime_budget.rs`; the root profile owner is 771 lines and remains a stage
orchestrator.

This follows the useful part of local Unreal `FontCache.cpp`: `FSlateFontCache` owns its atlas policy,
publishes atlas/shaped-sequence memory statistics, and exposes cache controls at the cache owner. It
does not justify one cross-domain knob object or a blind value copy. No default, line break, shaping,
tatweel selection, cache admission limit, SDF scheduling decision, or raster route changed. Focused
snapshot/receipt tests are authored and scoped rustfmt/diff/conflict checks pass; Cargo, 31-sample
timing/RSS/power, WGPU, and PNG remain pending. Status:
`owner_local_budget_snapshots_implemented / runtime_budget_profile_projection_implemented /
page_shadow_residency_receipt_implemented / algorithm_defaults_unchanged / static_checks_complete /
managed_profile_pending`。

### 12.43 Session-owned fallback and backend-route diagnostics

`RTS-P2-011` 的 current-source 调研确认 layout fallback 与 shaping failure 分别由一个
`OnceLock<Mutex<...>>` 进程全局 report 聚合，但公开 getter 在 workspace 没有消费方，状态也不参与 retry、fallback
或 cache policy。本地 Unreal `FontCache.cpp` 由 `FSlateFontCache` 在自身构造/析构边界拥有 shaper/cache/resources，并
从该 owner 发布 cache/stat/flush 信息；这支持 session/cache owner diagnostics，不支持全进程共享 last failure。

Zircon 已删除两个全局 getter、mutex 与双写路径。`SharedTextLayoutSession` 逐帧持有 layout code、typed shaping
failure 与 direct/whole-alternate/hybrid/deferred/terminal route；`begin_frame` 清零。parallel prewarm 在批次完成边界聚合相同
fixed value并合并回session，worker不持session，也不形成`parallel -> layout_session`反向依赖。cache hit不冒充
backend work；whole Cosmic recovery用空`alternate_ranges` receipt与hybrid ranges区分。

UI只投影35个固定session名称，不保存raw text、pointer、document ID、动态后端名或source label。具体document drill-down仍
开放：`TextDocumentKey`尚未贯穿rich/measure/hit-test/standalone，未来必须由有界document owner承接，禁止直接用作
metric label。scoped Rust 2024 rustfmt/diff/conflict与旧全局符号扫描通过；当前session root为762行，
profile总发射66项，focused capacity为128，完整集成capture为160。Cargo、fault、31-sample timing/RSS/power、WGPU与PNG尚未执行，
不声明动态性能改善或accepted。
详细报告见`81/2026-08-27-session-owned-text-diagnostics.md`。状态：
`session_owned_diagnostics_implemented / process_global_report_mutexes_removed /
fixed_backend_route_projection_implemented / parallel_prewarm_merge_implemented /
document_drilldown_owner_open / static_checks_complete / managed_validation_pending`。

### 12.44 Hard-line and shape-range terminology hard cut

`RTS-P2-002/003` 的 current-source 调用图确认：旧 `ShapedTextLine` 是 shaping 输出中的 hard-line 容器，direct
路径遍历 `hard_lines(request.text)`，Cosmic 也通过 `normalize_cosmic_hard_lines` 对齐 backend raw lines；真正的 wrap、
ellipsis 与 placement 后 visual line 由 `CandidateLine`/`UiResolvedTextLine` 发布。旧 provider/session
`shape_horizontal_line`/`shape_vertical_line` 又接受 `text + absolute source_range`，可在一个请求中形成多个 hard lines，
名称并不等于单个 layout line。

本地 Unreal `FSlateTextShaper::ShapeBidirectionalText/ShapeUnidirectionalText` 接受外部文本 start/len，并输出
`FShapedGlyphSequence` + `FSourceTextRange`；layout line不由 shaper入口命名。这支持按 source range描述shaping请求、把
hard-line与visual-line分开，不要求Zircon复制Unreal类型层次。

实现已硬切为 `ShapedHardLine`、`ShapedGlyphRun::hard_line_text`、
`shape_horizontal_range(_with_kerning)` 与 `shape_vertical_range(_with_kerning)`。41个Rust文件机械迁移，无旧alias、
re-export或wrapper；serde字段与结构顺序不变，`BackendShapeRequest`、session work-budget、cache admission及
direct/Cosmic算法均未改变。旧五个Rust符号精确0命中；新类型50处，range入口40/38/2/10处；scoped Rust 2024
rustfmt/diff/conflict通过，关键文件743/791/215行。Cargo、serde golden、产品WGPU/PNG与性能/功耗未执行。状态：
`shaped_hard_line_term_hard_cut_complete / shape_range_api_hard_cut_complete /
request_and_budget_owners_preserved / serde_fields_unchanged / old_rust_symbol_zero /
algorithm_unchanged / static_checks_complete / managed_validation_pending`。

### 12.45 Missing-primary and generation-deferred capability causes

`RTS-P1-009/013/014` 的 current-source 调用图确认两处 capability cause 在 request outcome 前丢失：
`fallback_text_spans` 的 `PrimaryFaceUnavailable` 在 Cosmic 被压成无receipt的 `FontUnavailable`；稳定generation重试耗尽、
session stale cache/ready与parallel stale worker只返回 `FontGenerationChanged`，session diagnostics又用
`record_terminal_failure`记录显式deferred。这样 public neutral error虽然正确，内部owner却无法区分缺primary与普通字体失败，
也会把可重试generation变化污染terminal route。

本地 Unreal `FCompositeFontCache::GetFontDataForCodepoint` 在sub/default/fallback选择后继续保留font-data/face状态，
`FSlateTextShaper` 还显式汇总loading faces，并在face无效或loading时进入替代字符路径；Zircon不复制其逐字符实现，但采用相同的
owner原则：font resolution/load state必须在shaping request完成前成为typed cause，不能由最终中性错误或空glyph反推。

实现向稳定catalog尾部追加 `FontPrimaryUnavailable` 与 `FontGenerationChanged`，总数14；generation receipt使用
`Deferred` disposition。所有稳定generation、session与parallel入口复用同一构造，ready/deferred/failed cache admission未改。
session-owned report新增deferred failure/run两个固定计数；后续12.46再加入request-resolution固定聚合，当前UI profile为35个
session维度、函数总发射66项、focused容量128、完整集成capture容量160，不含raw text、document ID或动态label。候选顺序、coverage结果、fallback span、
backend call、cache key和shape算法均未改变；时间/空间规模不变。

相关Rust文件经2024 rustfmt通过，trailing whitespace为0，failure code显式variant与`ALL`均为14，
neutral generation `.into()`扫描为0；当前关键owner仍低于900行。Cargo、fault injection、并发stress、
31-sample timing/RSS/power、WGPU与PNG均未执行，不声明动态性能收益。完整candidate ordinal/coverage reject、pending dependency、
policy reject与backend capability组合仍开放。状态：
`primary_and_generation_capability_causes_implemented / deferred_terminal_split_implemented /
fixed_profile_projection_implemented / algorithm_unchanged / static_checks_complete /
managed_validation_pending / full_capability_trace_open`。详细报告见
`81/2026-08-27-text-capability-cause-receipts.md`。

### 12.46 Request-owned fallback candidate decision receipt

继续重审 `RTS-P1-009/016` 与 Runtime Font 80 `RFF-P1-032/M3/M5` 后，确认完整
`FontResolveOutcome` 不能由当前同步、process-shared `FontDatabase` 伪造：Pending asset dependency、collection generation、
policy reject与backend capability必须由后续session-owned collection/capability compiler发布。当前可验证的真实信息只包括
resolution/candidate generation cache命中、现有候选循环访问与coverage拒绝、partial ranking和最终选择来源。

实现新增固定152-byte `TextShapingRequestDiagnostics`，由 transient `TextShapingCompletion`/`GenerationTaggedShapedRun`
与glyph run并行传递。它不进入公开serde `ShapedGlyphRun`、shaped cache key或resident-byte预算；session与parallel只在实际
cache-miss backend work完成时合并，cache hit不重放历史候选成本。generation-stability owner累计所有被丢弃attempt的
resolution work和restart数，最终成功、terminal或deferred结果均保留同一请求收据。

resolver在原有循环内计数，不新增probe：`decision_coverage_call_count`与实际
`face_covers_codepoint`调用同口径，涵盖candidate compiler family-face过滤、完整覆盖短路、partial ranking及missing diagnostic；
resolution cache hit只记录hit与逻辑选择，不冒充产生缓存值时的candidate visit/probe。固定聚合包含35个session profile名，
layout-resolve总发射66项，focused capacity为128、完整集成capture容量160；raw text、face/family、candidate ordinal、document ID与动态label均为0。

相关Rust文件经2024 rustfmt/diff-check静态通过，artifact/cache-budget源码无新诊断类型引用；固定大小、cache hit零历史probe、
generation attempt累计、session merge与profile name回归已写入但尚未由Cargo执行。未运行managed Cargo、fault/concurrency、
31-sample p50/p95/p99、RSS/功耗、WGPU/PNG，因此不声明动态性能改善或验收完成。状态：
`bounded_candidate_decision_receipt_implemented / transient_completion_envelope_implemented /
generation_attempt_cost_retained / shaped_artifact_pollution_zero / static_checks_complete /
managed_validation_pending / full_font_resolve_outcome_open`。详细报告见
`81/2026-08-27-font-resolution-request-receipts.md`。

### 12.47 Fallback cache structural profiling gate

在实现 resolver 优化前，对当前 `FallbackCaches` 与本地 Unreal
`FontCacheCompositeFont.cpp`/`SlateTextShaper.cpp` 进行了重新调用图审查。Zircon 的 family、composite、candidate、
resolution 与 line-metric 五类可写 LRU 共用一个 `Mutex<FallbackCacheState>`；命中也会更新 `HashMap` entry 与
`BTreeMap` LRU，因此 warm resolution hit 仍是独占临界区。cold composite 编译当前也在该锁内。另一个独立假设是
whole-text primary 预扫描若在长文本尾部失败，会让已验证前缀在 cluster itemization 中再次参与coverage决策。

本地 Unreal 在 composite cache 建立时预编译typeface与有序priority/ordinary range，codepoint查询使用边界检查和
binary search；shaper对grapheme做一次顺序face/loading-state扫描并合并相邻同face区间。这支持后续评估immutable
collection snapshot、线性request itemizer与更窄缓存写边界，但不授权复制Unreal的首码点cluster策略或降低Zircon完整
cluster coverage正确性。

实现只增加测量基础设施：全部fallback-cache状态访问汇聚到`with_state`；仅test/profiling构建测量锁获取、等待与持有
纳秒，普通构建无`Instant`成本。每个itemization通过cache-owner TLS聚合三个固定profile值，完成时一次发布；没有用
并发不安全的全局前后快照差值，也没有per-lock profiler事件或动态label。全局cache report保留累计值供隔离benchmark。
算法、cache容量、LRU、probe、candidate顺序与span结果未改。

必须先以31-sample cold/warm矩阵证伪/确认四个方向：warm hit共享写锁串行、cold composite compile convoy、late primary
reject重复probe、resolution/candidate两层key/lock重复。未取得数据前禁止分片锁、lock-free snapshot、request memo、
移除primary预扫描或重写decision graph。scoped rustfmt/diff-check通过；managed Cargo、CPU trace、p50/p95/p99、RSS/功耗、
WGPU/PNG尚未执行。状态：`request_local_cache_lock_profile_implemented /
structural_bottleneck_hypotheses_documented / resolver_algorithm_unchanged /
structural_optimization_profile_gated / static_checks_complete / managed_profile_pending`。

### 12.48 Shape-request paragraph-analysis construction profile

`RTS-P1-024` 的重审确认当前普通非空shape请求构造一次`BidiParagraph`、一次
`ParagraphTextAnalysis`（script + emoji presentation）与一次direct `LineBreakOpportunityMap`；horizontal direct
未完整完成并进入whole Cosmic candidate时，Cosmic为alternate glyph projection再构造一份line-break map。该重复在源码上
成立，但其命中率和成本尚未用动态数据证明，不能据此提前合并direct/Cosmic owner或建立大一统retained paragraph artifact。

实现新增folder-backed profiling leaf，由`shape_text_with_diagnostics`在canonical request边界启动/结束TLS；三个实际构造器
分别记录build count、累计input bytes与elapsed nanos，另有request count/bytes，共11个固定`text_analysis_*`名称。模块与
构造器调用只存在于test/profiling构建，且未激活capture时不调用`Instant::now`；scalar/grapheme循环不触碰profiler锁，
request完成后一次发布并清空TLS。normal build、analysis算法、cache identity、glyph/layout结果与owner寿命均未改变。

managed矩阵必须覆盖horizontal direct/whole-alternate/hybrid、vertical、generation retry和error路径，规模为1/100/1k/10k
grapheme，每lane预热后31样本；同时记录wall p50/p95/p99、11项收据、route/resolution/cache-lock计数、CPU stack、allocation、
RSS/功耗与glyph/layout hash。只有alternate line-break或跨owner重复构造成为主导项后，才允许hoist或设计retained artifact。
scoped rustfmt/diff-check通过，Cargo与动态profile未运行。状态：`shape_request_analysis_profile_implemented /
duplicate_construction_observable / retained_paragraph_artifact_open / algorithm_unchanged /
static_checks_complete / managed_profile_pending`。详细报告见
`81/2026-08-27-paragraph-analysis-construction-profile.md`。

### 12.49 Atomic-cluster source fallback geometry

`RTS-P1-034` 的 current-source 重审确认 canonical `ResolvedTextGlyphArtifact` 已经统一 backend cluster 的
caret、hit-test 与 selection；真实分叉位于 artifact 缺失/失效后的 source-measure 路径。旧 caret 只保留 prefix width，
selection 可能按 visual span 重复 shape 两个 prefix，editable pointer 则退回 DTO grapheme midpoint，均丢失已有
`MeasuredGlyphCluster::AtomicCluster` receipt。

本切片让 `GraphemeAdvanceIndex` 同时拥有 LTR atomic caret span、physical hit 与 selection range coalescing。
source fallback 仅接受 exact source/visual congruence、单一 LTR run、HorizontalTb、非 Justify、非 ellipsis、无 tab；
合格行完整 shape 一次，caret/hit 查询为 `O(log graphemes + log clusters)`，selection 以单调 cluster 扩张替代每 span
重复 prefix shape。canonical artifact 命中不增加 shape，rich/secure/virtual/BiDi/vertical 不满足资格时继续 fail closed
到原 artifact/DTO，不从 source order 猜 physical geometry。editable pointer 复用 render command 已有 text/style 接入该路径。

没有新增 public ABI、serde、layout DTO、cache key 或 renderer contract。源码行为回归已编写；Rust 2024 rustfmt、scoped
diff-check、调用点和文件预算静态检查通过，最大触及 owner 684 行。managed Cargo、官方 corpus、1/100/1k/10k × 31
样本 shape-call/CPU/allocation/RSS/功耗、WGPU/PNG 均未执行；cross-rich-run、rich/BiDi/vertical missing-artifact geometry、
arbitrary source-range 与 GDEF caret provider 仍开放。状态：`atomic_cluster_ltr_source_fallback_geometry_implemented /
source_aware_editable_pointer_route_implemented / canonical_artifact_fast_path_preserved / static_checks_complete /
rich_bidi_vertical_gdef_and_managed_validation_open`。详细报告见
`81/2026-08-27-atomic-cluster-source-fallback-geometry.md`。

### 12.50 Invalid resolved-advance geometry hard cut

`RTS-P1-012` 的 current-source 复核找到最后一条显式比例几何：neutral
`UiTextLineSourceMap::advance_to_visual_offset` 在 `glyph_advances` 与 visual grapheme 数量不一致时，仍把
`measured_width` 等分到每个 grapheme。该输入已经违反 `UiResolvedTextLine` 的“一 visual grapheme 一 advance”合同，
而 interface owner 没有 font database、shape session、resolved face 或 backend cluster artifact，不能合法重建内部位置。

本切片硬切该兼容策略：只有 cardinality 精确、全部 finite 且非负的 advance 才进入原 exact-prefix cache；无效 DTO
只保留 leading `0` 与 trailing sanitized `measured_width` 两个已知物理端点，所有内部查询收敛到 leading edge。
Runtime hit-test 在持有 command text/style 时仍可由 shaping owner 恢复，不把恢复职责下沉到 neutral interface。
有效布局的复杂度和缓存算法不变，无效输入由 scan/division 收敛为 `O(1)`；这不是性能收益声明。

后续调用图复核又删除 runtime 的默认-style 重塑旁路：canonical artifact 仍最优先；严格 source-congruent LTR
行由同一 `GraphemeAdvanceIndex` 同时返回 ordinary grapheme midpoint 与 atomic cluster endpoint；没有 source owner 时
仅接受 cardinality/finite/non-negative 完整 DTO，否则按 aggregate midpoint 选择两个行端点。旧临时
`SharedTextLayoutSession`、layout-level default style 和 fallback advance allocation 均已移除；有效 tab/BiDi/vertical
DTO 快路未改变。

数量不匹配与 `NaN`/负值回归已编码；Rust 2024 rustfmt、scoped diff-check、source scan 与 255-line owner budget
静态通过。Cargo、corpus、性能/功耗、WGPU/PNG 未执行。状态：
`invalid_resolved_advance_endpoint_geometry_implemented / runtime_default_style_reshape_removed /
proportional_fallback_removed / valid_exact_prefix_fast_path_preserved / static_checks_complete /
managed_validation_pending`。详细报告见
`81/2026-08-27-invalid-resolved-advance-fail-closed.md`。

### 12.51 Document revision foundation current-source correction

`RTS-P1-043` 原先把 document edit delta 记为完全缺失，但 crate-private `text/document` 已有 immutable
original/addition chunk、piece list、owner+revision、old/new byte dirty span、length delta 与 revision-bound
hard-line/grapheme index。它尚无产品消费者，index失效后也仍先物化完整`String`再全量构建，因此只属于内部底座，不能关闭
document authority、stable line 或 incremental reflow。

本切片先修复identity正确性：`replace`强制携带expected `TextDocumentKey`，stale key在任何mutation前返回typed error；
revision由`saturating_add`改为`checked_add`，耗尽时保持source、piece、index与key全部不变；document authority移除
`Clone/PartialEq`，阻止同owner+revision分叉出不同source。另修正多行replacement测试的
UTF-8 byte delta：13-byte replacement替换6 bytes应增长7，new dirty end为19。没有为了计算paragraph dirty范围在一次编辑中
构建old/new两个全文snapshot；该伪增量方向被明确拒绝。

本地Unreal Slate参考显示`FLineModel`独立持有shaped cache、break candidates、estimated geometry与分项dirty flags，
`ModelChangeCounter`记录model变化，visual `LineView`可按model延迟生成。Zircon下一步应先建立separator-aware stable hard-line
owner，再测量1/100/1k/10k-line edit/scroll数据后选择index/reflow结构。Rust 2024 rustfmt、scoped diff与source guard通过；
managed Cargo、性能/功耗、WGPU/PNG未执行。详细报告见
`81/2026-08-27-document-revision-foundation-review.md`。

### 12.52 Non-empty whitespace layout admission hard cut

render-command layout resolution 与 owner-overlap prewarm 原先都用 `!text.trim().is_empty()` 判断是否存在文本。
这会让只含space、tab或hard separator的command保留source却没有`UiResolvedTextLayout`：space/tab的advance与tab stop、
separator的line box/baseline，以及caret/selection/IME geometry均失去owner。`trim`只适用于显式wrap/justify/form/query/
presentation策略，不能定义空文档。

两处资格门现在只拒绝真正的empty source。spaces进入正常prewarm；spaces与hard separator进入布局解析；普通空display
source继续跳过。editable空source仍由owner路径发布editable state，whitespace-only行不做Justify的既有策略也未改变。
三态行为回归已编码；Rust 2024 rustfmt、scoped diff/source guard与246/483行owner budget通过。managed Cargo、tab/
Unicode separator corpus、editing geometry、WGPU/PNG仍开放。状态：
`nonempty_whitespace_layout_admission_implemented / owner_prewarm_whitespace_admission_implemented /
empty_display_source_fast_path_preserved / static_checks_complete / managed_validation_pending`。详细报告见
`81/2026-08-27-whitespace-layout-admission.md`。

### 12.53 Stable separator-aware hard-line model

`RTS-P1-043` 继续完成其首个line lifetime前置。`TextDocumentHardLineModel`现在保留稳定ID、content byte length与
separator byte length，不缓存必须随前缀编辑整体平移的absolute suffix range。`replace`在piece mutation前只物化受影响line及
前后各一条context组成的局部envelope，先完整准备新model splice；CRLF、VT/FF/NEL/LS/PS与canonical hard-line scanner一致。
未变prefix/suffix保留ID，line内容修改与merge保留左affected ID，split的额外line获得revision-qualified creation ID；edit receipt
发布old/new reanalyzed ordinal span。原`TextDocumentSourceIndex`不再保存第二份hard-line vector，只保留grapheme index。

这移除了edit path的整文档hard-line snapshot/rebuild，但不冒充完整增量布局：grapheme index仍会全文snapshot重建，`Vec` line
sequence插删仍可能移动suffix metadata，UI/service/`DocumentLayoutSession`也尚未消费line ID。Rust 2024格式、source scan、
281/385行模块预算与七类hard-line边界回归静态完成；managed Cargo、1/100/1k/10k edit profile、RSS/功耗、WGPU/PNG未执行。状态：
`separator_aware_stable_line_owner_implemented / edit_local_reanalysis_implemented /
full_document_hard_line_rebuild_removed_from_edit / product_session_unwired /
static_checks_complete / managed_validation_pending`。详细报告见
`81/2026-08-27-stable-hard-line-model.md`。

### 12.54 Physical-line content/placement geometry hard cut

审查`resolve_line_widths_with_provider -> aligned_x -> UiResolvedTextLine -> render/hit/geometry`完整调用链后，最初的
“超宽nowrap Center/Right可能被clamped width错误对齐”假设未成立。本地Unreal
`GetLineViewHorizontalDisplayOffset`以`max(DrawWidth, ViewSize)`计算justify空间；line自然宽度超过viewport时extra为0，display
origin同样留在viewport start。Zircon以clamped placement extent计算出的origin与其一致，同时natural measured width与advances
继续溢出并由clip裁剪，因此本轮不改算法。

后续完整调用图复核确认这不是局部alignment修补，而是公开line-view合同问题。现已硬切：required
`placement_frame`承载paragraph/rich-cell slot，`line.frame`只承载absolute natural content geometry；Plain/rich horizontal与
`VerticalRl`四条producer统一发布两者，table projection原子平移两者。line selection/clip admission消费slot，renderer、caret、
selection与IME继续消费content frame，rich activation通过content `hit_frame()`排除aligned empty slot。旧缺字段serde shape
显式拒绝，不保留ambiguous compatibility default。

该正确性迁移每published line增加一个16-byte `UiFrame`，没有新增shape、wrap、allocation或search loop，不据此声明性能改善。
受管基准必须覆盖1/100/1k/10k physical lines，horizontal left/center/right nowrap short/overwide、wrapped、`VerticalRl`与rich table，
分别记录cold layout、warm retained layout、clipped scroll和pointer hit的31-sample CPU p50/p95/p99、allocation/RSS、shaping/cache
计数；产品Native/SDF场景另记录GPU timestamp、upload bytes和同scene power trace。先比较父基线与候选的相同source/font/backend/
viewport，再决定是否处理line selection或publication结构；无matched Unreal workload不得声称功耗或经验值已对齐。

状态：`current_overflow_alignment_matches_unreal / content_and_placement_geometry_separated /
four_layout_routes_and_interaction_consumers_migrated / static_checks_complete /
managed_profile_power_wgpu_png_pending`。详细报告见
`81/2026-08-27-nowrap-clip-width-semantics-review.md`。

### 12.55 Surface text-layout revision exhaustion hard cut

产品surface的`UiLayoutCache::advance_text_layout_revision`仍使用`wrapping_add(1)`，而extract直接把
`(node_id, raw revision)`发布成`TextDocumentKey`。这会在耗尽后让变化后的source重新使用旧revision 0；当前parsed owner虽有exact
source二次资格，也不能把未来所有cache的identity正确性建立在这条防线上。

revision现在checked advance，`u64::MAX`固定为不可发布sentinel；两个extract key点只消费
`retained_text_layout_revision()`。pending owner的key硬切为显式`Option`，无key仍进入普通layout、editable projection、shape
prewarm与unretained viewport window，仅关闭跨帧retained document复用，不丢文本。接口与render回归已编码；89/248/519行
owner预算、Rust 2024格式、source scan和scoped diff静态通过。Cargo、serde sentinel round-trip、node-pool exhaustion fault、
WGPU/PNG未执行。状态：`surface_text_revision_wrap_removed / exhausted_identity_fail_closed /
uncacheable_layout_fallback_preserved / static_checks_complete / managed_validation_pending`。详细报告见
`81/2026-08-27-text-layout-revision-exhaustion.md`。

### 12.56 Collection-qualified layout and artifact publication

2026-08-29 current-source continuation found that canonical shaping had gained an explicit collection while
UI measurement, physical/logical fragment fences, rich/plain artifact freshness, and retained surface
invalidation still sampled the process-global generation. Two independent collections at generation 1 could
therefore share a false freshness decision even though shaped-cache handles were already collection-qualified.

The pipeline now uses one `FontCollectionRevision(collection_id, generation)` supplied by
`TextShapeRunProvider`. The same collection snapshot certifies coverage/line metrics and projects font handles;
`UiTextMeasureCache` and `UiSurface` retain that provider. Resolved artifacts retain both the immutable database
snapshot and the post-registration resolver snapshot, so renderer acquisition cannot bind glyph IDs to another
collection or a newer registry publication. Default constructors remain process adapters; real runtime manager,
window/PIE injection and screen-space render-state binding are still open. Rustfmt, scoped diff, old-symbol,
global-probe and conflict scans pass. Cargo, corpus, WGPU/PNG, p50/p95/p99, allocation/RSS and power were not run.
Status: `collection_qualified_layout_publication_static_implemented /
renderer_artifact_lease_static_implemented / managed_validation_pending`.

### 12.57 Screen-space renderer revision boundary

The follow-up renderer audit found four remaining process-generation reads below the product rendering owner:
plan cache publication, retained segment products, post-font-load artifact admission, and the default render-state
constructor. This was a structural correctness problem rather than a local cache optimization: two independent
collections at the same generation could reuse a plan/segment or accept glyph IDs from the wrong collection.

`ScreenSpaceUiRenderer`, `ScreenSpaceUiTextSystem`, and `TextRenderState` now accept one explicit
`FontCollectionService`. Process worker-budget selection is independent from that owner. Plan and segment cache
keys carry the complete collection revision, and artifact admission verifies the artifact's retained database/
resolver lease against it. The plan key observes the service's published revision while raster publication uses
the revision already adopted by render state, preserving publication ordering without a database clone.

Same-generation foreign-collection regressions cover all three cache/admission boundaries. Each key gains one
`u64` collection identity and remains O(1); admission remains one O(text batches) pass with no new shape,
coverage, source copy, registry lock, or per-glyph loop. Static formatting, diff, global-probe, conflict, and file-
budget checks pass. Core manager/window/PIE wiring, managed Cargo/corpus, 31-sample CPU/allocation/RSS, power,
matched Unreal workload, and WGPU/PNG remain open. Status:
`screen_space_renderer_collection_revision_static_implemented /
process_default_restricted_to_adapter /
core_manager_injection_and_managed_validation_pending`.

### 12.58 Rich source admission and text-module compile cleanup

The current-source review found that rich layout still had a structural fail-open boundary: missing
or malformed runs could be skipped by `RichAdvanceIndex::source_spans`, while forced-line and
VerticalRl range producers recovered invalid offsets with sentinels. The source contract is now
owned by `rich_source.rs`; one validated run pass checks source identity, monotonic non-overlapping
UTF-8 ranges, and legal gaps, while checked `usize`/`u32` helpers are reused by horizontal,
VerticalRl, table, materialization, artifact, and prewarm owners. Generation changes remain
`Deferred`, and malformed geometry returns `LayoutFailed`.

The same implementation pass removed remaining text-module compiler diagnostics: glyph projection
has one crate-local owner, fallback hash branches are unit typed, family identity maps are explicitly
typed, SDF face-cache callers use the database-only lookup contract, runtime default-face access is
scoped to the text tree, and Cosmic caches clone the supplied immutable snapshot. Static text
regressions pass 29/29, including the dedicated infrastructure compile-contract suite 11/11; targeted
Rustfmt and scoped diff checks pass. Workspace Cargo is still red only in untouched
graphics/core/plugin/dynamic-api modules, so WGPU/PNG, 31-sample profile, allocation/RSS/power, and
matched Unreal product validation remain open. Status:
`rich_source_fail_closed_static_implemented / text_module_compile_diagnostics_closed /
static_checks_complete / managed_validation_pending`.

### 12.59 Rich parser byte admission and indexed-artifact hard cut

P2-4 was a representation defect, not a micro-optimization: parser and compiled-artifact paths
converted byte offsets and projection indices with `unwrap_or(u32::MAX)`, so distinct over-limit
nodes could alias. The parser now owns configurable source/output byte budgets; default capacity
matches the existing 32 MiB retained-text scale so large Plain viewport documents are not regressed,
while every effective limit remains capped by the `u32` representation. Source is rejected before
cache lookup/copy, visible output is checked before append and during emoji expansion, and public
parse/compile entrypoints return typed errors.

Compiled rich construction now checks visible length, collection counts, grapheme ranges and every
table-cell projection index. Cache single-flight stores one terminal `Result`; failed construction is
removed from residency after current waiters receive it. UI maps the detailed parser error to the
stable low-cardinality `ZR-TEXT-LAYOUT-012` diagnostic and publishes the normal failure layout rather
than partial geometry. This is a correctness/admission cut, not measured speed or power evidence.
Static Runtime Text contracts pass 29/29; the latest E-drive Cargo fingerprint reports zero primary
errors in the owned text/UI-text/framework-text trees but the workspace remains red in unrelated
graphics/core modules. WGPU/PNG, 31-sample latency/allocation/RSS/power, and matched Unreal product
validation remain open.
