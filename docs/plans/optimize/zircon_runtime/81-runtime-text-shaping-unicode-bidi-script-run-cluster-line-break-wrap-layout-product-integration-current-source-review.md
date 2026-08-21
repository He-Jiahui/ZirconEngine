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
| RTS-P0-001 | 长行语义M0已实现、managed验证待完成；完整typed defer/cancel仍属M3 | 旧实现把超长logical line伪装成多行；M0已删除该语义并将7个测试文件的24处旧标识符迁移到`TextShapingWorkBudget`，当前Rust源码中旧引用为0。该临时合同尚无production scheduler caller，不能成为source/script/cluster边界 | source line保持唯一；相关test target可编译；随后交付保留上下文和cluster map的backend work unit，并使长Arabic/Indic/emoji/ligature语义与budget/cancel测试通过 |

### 4.2 继承P0

| 既有阻断 | Current-source状态 | 唯一owner |
|---|---|---|
| 真实shaping全失败仍发布synthetic glyph与猜测advance | `cosmic/fallback.rs`仍生成FNV glyph ID，face/instance为None；canonical public路径仍把错误折叠为run或空fallback | Runtime11B P0-2；Runtime81只补typed shaping/budget/cluster关闭条件，不重复计数 |

## 5. P1差距

### 5.1 Unicode、language、script、emoji与BiDi

| ID | 差距 | 工程化要求 |
|---|---|---|
| RTS-P1-001 | `ShapingTextView::v1_disabled`保持identity，canonical normalization与offset map没有产品策略 | 定义NFC/NFD/none policy、original-normalized双向source map、canonical-equivalence golden与安全上限 |
| RTS-P1-002 | language tag只trim、`_ -> -`、ASCII lowercase，不能做BCP-47 canonicalization/likely script/region | 使用版本化locale service，输出canonical tag、script/region与fallback decision receipt |
| RTS-P1-003 | 多个Unicode crate各自带数据版本，artifact/diagnostic/cache key不记录统一版本 | 建立`UnicodeDataSnapshot`，绑定Bidi/Script/LineBreak/Grapheme/Emoji/Vertical版本与hash |
| RTS-P1-004 | script segmentation只用单值`UnicodeScript`，没有Script_Extensions | 使用Script_Extensions和language/neighbor context生成稳定script run |
| RTS-P1-005 | Common/Inherited/Unknown简单粘邻run，paired bracket不继承配对侧script | 引入paired-bracket stack与context resolution，覆盖嵌套括号、标点、combining mark |
| RTS-P1-006 | emoji由手写码位范围判定，不覆盖完整property、ZWJ、tag、keycap、VS15/VS16与版本差异 | 使用版本化emoji property/sequence数据并保持sequence-level itemization |
| RTS-P1-007 | `FontScript::Other`用cluster首个codepoint编码，既不是script identity也会放大cache cardinality | 使用稳定ISO15924/ScriptExtensions set；未知脚本为typed Unknown，不塞codepoint |
| RTS-P1-008 | fallback按grapheme逐段决策，无法证明一个face覆盖完整emoji/Indic/combining sequence | 在shaping cluster/sequence层做coverage与fallback，保留候选和缺失码位原因 |
| RTS-P1-009 | face/span解析大量`Option`，partial coverage、pending font、policy reject与backend unsupported不可区分 | 返回typed itemization/fallback outcome并携range、face、generation、reason |
| RTS-P1-010 | UAX#9依赖存在，但没有BidiTest/isolates/bracket/overflow-level官方corpus receipt | 对固定Unicode版本跑官方conformance和混排产品golden，保存fail range与level trace |
| RTS-P1-011 | shaping已计算visual glyph order，UI `visual_order.rs`又按grapheme做第二次BiDi reorder/mirroring | 只发布一次canonical visual cluster order；UI不得重新分析同一paragraph |
| RTS-P1-012 | UI用`source_subrange`按run字节比例映射fragment，mirroring只处理单字符fragment | 贯通backend cluster/source map，ligature、combining、mirrored pair、inline run不做比例猜测 |

### 5.2 Shaping contract、cluster、budget、cache与vertical

| ID | 差距 | 工程化要求 |
|---|---|---|
| RTS-P1-013 | `TextLayoutError`只有粗枚举，无backend、font、range、feature、retryability或cause chain | 定义`ShapingError/Outcome`与稳定code、phase、range、dependency、retry/budget字段 |
| RTS-P1-014 | `SharedTextLayoutSession`记录global计数后返回空`ShapedGlyphRun`，稳定generation下还会cache该fallback | error/pending/budget状态不得伪装ready run或进入ready cache；调用方显式处理 |
| RTS-P1-015 | 删除64 KiB source-line cap后，任意长unbroken input可一次送入RustyBuzz/Cosmic | source line与backend work unit分离；work unit有上下文重叠、cluster去重与语义证明 |
| RTS-P1-016 | shape request没有输入字节、glyph、CPU、内存、deadline、cancellation或partial result合同 | 引入per-document/per-frame budget token和cooperative cancellation，超限返回typed partial/deferred |
| RTS-P1-017 | direct path只检查cluster offset为char boundary、finite与nonempty，缺完整覆盖/单调性/unsafe-to-break诊断 | 验证backend cluster invariants并保存unsafe-to-break、ligature caret和malformed result |
| RTS-P1-018 | horizontal/vertical direct大量以`Option`回退，具体face parse、buffer、feature与shape失败被抹平 | backend adapter返回typed capability与failure，不以`None`触发无差别fallback |
| RTS-P1-019 | vertical TransformOrRotate为判断substitution provenance再次完整shape一遍 | 从backend glyph info/feature plan取得provenance，或缓存comparison result并受budget约束 |
| RTS-P1-020 | `vert/vrt2`、vertical orientation、font substitution与rotation没有统一per-cluster capability receipt | 发布`VerticalGlyphDecision`，包含chosen feature、orientation、rotation、font与fallback reason |
| RTS-P1-021 | Cosmic是whole-request fallback，与direct itemized result之间没有可组合的partial recovery合同 | 按analysis run组合backend outcome，禁止一处失败迫使整段退到不同cluster语义 |
| RTS-P1-022 | `ShapedGlyphRun`持完整`Arc<str>`，cache/parallel/projection仍可能重复source与per-glyph metadata | 落实既有Text02 failure：document-owned rope/snapshot + range lease + SoA/cluster table |
| RTS-P1-023 | shaped cache和document/source identity多用`DefaultHasher`，只适合进程内临时查找 | 区分ephemeral hash与stable content identity；artifact/replay必须用版本化stable digest |
| RTS-P1-024 | hard line、paragraph、shape run和layout line生命周期混在`ShapedTextLine`/per-call session中 | 建立retained paragraph analysis/layout artifact，dirty range只使依赖run/line失效 |

### 5.3 Line break、wrap、measure、ellipsis、tab与justification

| ID | 差距 | 工程化要求 |
|---|---|---|
| RTS-P1-025 | UAX14基础来自`unicode-linebreak`，但没有Unicode version、locale tailoring与rule trace | line-break artifact携data version、tailoring profile与chosen rule/opportunity |
| RTS-P1-026 | 没有官方LineBreakTest/WordBreakTest/GraphemeBreakTest current receipt | 固定版本跑全corpus，并为引擎tailoring单列expected override与理由 |
| RTS-P1-027 | `kinsoku.rs`手写日文标点、小假名与pair表，不可版本化、配置或按locale选择 | 编译JLREQ/locale tailoring data，Editor可诊断规则来源、优先级和冲突 |
| RTS-P1-028 | `smart.rs`用手写尾标点/分隔符判断“word smart”，不是Unicode word segmentation | 由WordBreak + locale dictionary + style policy生成候选，不维护另一套字符表 |
| RTS-P1-029 | Thai/Lao/Khmer/Myanmar等没有dictionary break，自动hyphenation也不存在 | 接入可选locale dictionary/hyphenator provider，具备版本、license、budget与fallback |
| RTS-P1-030 | wrap只有有限枚举，缺`word-break`、`overflow-wrap`、`hyphens`、line-break strictness等正交策略 | 定义可组合style contract并映射到analysis/layout artifact，不靠新枚举爆炸 |
| RTS-P1-031 | soft hyphen断开时固定渲染ASCII `-`，未按font/language/script/style重新shape | 由hyphenation decision产生virtual source mapping，并用当前run字体/feature shape真实hyphen |
| RTS-P1-032 | EndWord ellipsis靠whitespace退回，不遵循Unicode word boundary或locale | 消费word-boundary artifact；marker也需真实shape、source mapping和accessibility语义 |
| RTS-P1-033 | tab只按space width乘`tab_size`生成均匀stop，没有显式/右/中/小数tab或BiDi/vertical模型 | 定义paragraph tab-stop list、alignment与leader，按writing mode和direction布局 |
| RTS-P1-034 | ligature覆盖多个grapheme时advance按grapheme数均分，caret/hit/wrap位置不来自字体 | 消费OpenType ligature caret/GDEF或backend cluster carets；无数据时输出明确fallback policy |
| RTS-P1-035 | boundary correction固定左右各8个grapheme，只修总宽，不能证明任意contextual substitution正确 | 用unsafe-to-break/cluster boundary或可扩展context plan；固定窗口只作优化并有fallback |
| RTS-P1-036 | Arabic joining机会由手写码位表推断，UI直接插最多32个U+0640并最多重shape 5次 | 使用Unicode Joining_Type、backend safe-to-insert-tatweel与language/font justification API |

### 5.4 UI/product、viewport、cache、publication与qualification

| ID | 差距 | 工程化要求 |
|---|---|---|
| RTS-P1-037 | `UiShapedText::from_resolved_layout`按grapheme重造FNV glyph ID，缺advance时平均分整行宽 | 删除伪shaped构造；公开contract只接受runtime canonical glyph/cluster artifact或typed unavailable |
| RTS-P1-038 | command随后给synthetic glyph补style-derived font/atlas key，伪装成有资源身份的glyph | font/atlas只能由resolved face/glyph route发布，禁止presentation层补全authority |
| RTS-P1-039 | nonvirtualized layout先按clip丢line，再从`resolved_lines`计算measured width/height，clip改变布局度量 | layout size从完整layout artifact计算；clip只影响paint visibility，不改变intrinsic/desired size |
| RTS-P1-040 | viewport fast path仅plain、horizontal、nowrap、Clip、无preedit；wrapped/rich/vertical/editor仍全量 | paragraph/line virtualizer覆盖wrapped/rich/vertical/preedit，并保持总extent与anchor稳定 |
| RTS-P1-041 | preedit在完整String上replace并直接禁用viewport，长文档一次composition触发全量复制/布局 | document overlay/rope保存preedit span，仅重算受影响paragraph并保持IME source map |
| RTS-P1-042 | rich/vertical intrinsic measure先用`text.len()*em`构造近似方形extent，再执行全布局 | 共享真实incremental measure artifact，禁止虚构constraint驱动不同break结果 |
| RTS-P1-043 | 没有document edit delta、paragraph dirty set、stable line ID与incremental reflow receipt | `DocumentLayoutSession`消费revisioned delta，记录reused/reanalyzed/reshaped/reflowed范围 |
| RTS-P1-044 | `TextDocumentKey(owner, revision)`完全信任调用者递增revision，命中时不校验source | owner API原子提交snapshot+revision；debug/qualification校验content digest并拒绝stale alias |
| RTS-P1-045 | text measure cache只限4096 entries，entry持完整`Arc<str>`和值却没有byte/resident cap | 同时限制entry、source/value bytes、admission与eviction cost，按document owner回收 |
| RTS-P1-046 | 无provider入口每次新建`SharedTextLayoutSession`，调用方可绕过retained cache与document owner | product API必须解析长期session；one-shot仅限显式tool/test并携budget |
| RTS-P1-047 | renderer在artifact缺失/stale时可回到source-isomorphic或visual fallback，layout/render无统一completeness | publication携generation和completeness；fallback重shape必须产生新artifact/receipt而非隐式分叉 |
| RTS-P1-048 | 测试以结构断言、source-string guard和局部unit为主，无官方corpus、真实font golden、soak与同负载Unreal基准 | 建立hermetic correctness/performance qualification matrix并保存current-source fingerprint |

## 6. P2差距

| ID | 差距 | 收敛方向 |
|---|---|---|
| RTS-P2-001 | `pending_common_start`只用于状态存在性，真正split只读`pending_common_end`，意图不清且易误修 | 用显式CommonResolutionState和测试表达leading/intermediate/trailing punctuation策略 |
| RTS-P2-002 | `ShapedTextLine`在shaping层承载hard-line结果，名称让source line、shape run、layout line混淆 | 固定`ParagraphAnalysis/ShapeRun/VisualLine`术语和类型边界 |
| RTS-P2-003 | `shape_horizontal_line`实际可接完整paragraph/超长hard line，API名称掩盖输入/预算语义 | 改为shape analysis run/paragraph request并显式传scope/budget |
| RTS-P2-004 | `TextDocumentKey::fingerprint`与多个`source_hash`各自用DefaultHasher，stable/ephemeral用途未在类型区分 | 建立`EphemeralCacheHash`与`StableContentDigest`新类型 |
| RTS-P2-005 | `UiTextShaperStack`目前只有一个wrapper backend，名字暗示并不存在的多backend policy | 删除空抽象或真正承载capability-ordered backend set与receipt |
| RTS-P2-006 | `UiRichTextArtifactHandle::PartialEq`只比较type ID，同类型不同内容被视为相等 | comparison使用stable artifact identity/generation；若不参与值语义则移除误导性Eq |
| RTS-P2-007 | interface shaped/layout DTO复制line/run/cluster `String`，跨extract/debug序列化放大内存 | runtime内用range/Arc/lease，只有真正跨进程的versioned payload才materialize |
| RTS-P2-008 | 8-grapheme、32 tatweel、5 probe、16 document、32 MiB等常量分散，缺profile/receipt | 归入TextRuntimeProfile和budget snapshot，保留合理默认但可审计 |
| RTS-P2-009 | 多个测试通过`include_str!`检查源码未恢复旧实现，能防回退却不能证明行为 | source guard只作辅助；以black-box conformance、fault和scale测试为主 |
| RTS-P2-010 | perf测试含ignored/环境敏感路径，test attribute数量容易被误当覆盖率 | required qualification单独登记non-ignored run、机器/字体/阈值与结果artifact |
| RTS-P2-011 | fallback report是process-global Mutex聚合计数，没有session/document/backend低基数维度 | 并入session diagnostics receipt，限制cardinality且不记录raw text |
| RTS-P2-012 | `TextLayoutError`显示字符串硬编码英语且没有stable diagnostic code/catalog | code与本地化message分离，由Editor/telemetry消费结构化字段 |

## 7. 与参考引擎的差异归纳

| 维度 | Unreal/Godot/Bevy参考 | Zircon当前差异 |
|---|---|---|
| Analysis owner | Unreal line model/view与ShapedTextCache、Godot shaped RID、Bevy retained Parley Layout都有长期owner | Zircon大量one-shot session，paragraph analysis、shape、UI visual projection与renderer fallback可分叉 |
| Unicode/script | Unreal/Godot接ICU/HarfBuzz；Godot有paired-bracket stack与Unicode emoji property | Zircon用多个独立crate加手写emoji/标点/joining表，无统一data snapshot或conformance receipt |
| Line break/tailoring | Unreal ICU line/word iterator；Godot按locale创建ICU line iterator并支持strictness | Zircon UAX14 base之上用硬编码kinsoku/smart字符表，无dictionary/hyphenator provider |
| Cluster/caret | Godot shaped text直接提供carets、selection、grapheme bounds与hit API | Zircon layout把ligature advance均分，再由UI/接口按grapheme重建，cluster authority不唯一 |
| Justification/tab/overflow | Godot暴露kashida/word-bound、tab stops、overrun flags与virtual glyph | Zircon仅均匀tab、手写Arabic插U+0640、有限ellipsis/wrap枚举 |
| Failure/product status | Bevy至少以`TextError`公开失败；Unreal/Godot owner能表达loading/shape state | Zircon内部虽有`TextLayoutError`，canonical调用仍常转为空或synthetic success |
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
| RTS-GATE-044 | renderer artifact stale/missing产生typed fallback receipt，不隐式使用另一套shape真值 |
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
