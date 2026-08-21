---
title: Runtime Rich Text、Markup、Parser、Token、Style Span、Inline Object、Link、Image、Table、List、Layout、Selection、Accessibility、Security 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime84
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/text/model/rich.rs
  - zircon_runtime/src/text/rich
  - zircon_runtime/src/text/cache/rich_cache.rs
  - zircon_runtime/src/text/layout/rich.rs
  - zircon_runtime/src/text/layout/rich
  - zircon_runtime/src/text/layout/rich_advance_index.rs
  - zircon_runtime/src/text/layout/rich_vertical.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_inline.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_inline_vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table
  - zircon_runtime/src/ui/surface/input/rich_link.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
  - zircon_runtime/src/graphics/scene/resources/ui_texture.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
tests:
  - zircon_runtime/src/text/rich/tests.rs
  - zircon_runtime/src/text/rich/tests
  - zircon_runtime/src/text/layout/rich/tests.rs
  - zircon_runtime/src/text/layout/rich_advance_index/tests.rs
  - zircon_runtime/src/text/layout/rich_vertical/tests.rs
  - zircon_runtime/src/ui/text/rich_text/tests.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/rich_blocks.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/rich_inline.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/rich_table
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_inline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_table.rs
  - zircon_runtime/tests/runtime_text_rich_blocks.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/IRichTextMarkupParser.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ITextDecorator.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/RichTextLayoutMarshaller.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/RichTextLayoutMarshaller.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/RichTextMarkupProcessing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateWidgetRun.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/SlateWidgetRun.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SRichTextBlock.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/RichTextBlock.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/RichTextBlockDecorator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/RichTextBlockImageDecorator.cpp
  - dev/godot/scene/gui/rich_text_label.h
  - dev/godot/scene/gui/rich_text_label.cpp
  - dev/godot/scene/gui/rich_text_effect.h
  - dev/godot/scene/gui/rich_text_effect.cpp
  - dev/godot/scene/resources/text_paragraph.h
  - dev/godot/scene/resources/text_paragraph.cpp
  - dev/godot/tests/scene/test_rich_text_label.cpp
  - dev/bevy/crates/bevy_text/src/text.rs
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/text_access.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/run.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/textwrapper.rs
  - dev/Graphics/README.md
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Samples~/Common/TextMesh Pro/Resources/TMP Settings.asset
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Samples~/Common/TextMesh Pro/Resources/Fonts & Materials/TMP_Node.hlsl
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Rich Text、Markup、Parser、Token、Style Span、Inline Object、Link、Image、Table、List、Layout、Selection、Accessibility、Security 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon的富文本并非只有临时字符串替换。当前源码已经建立`CompiledRichText`这一份`Arc`持有的解析产物，包含源markup、plain text、style run、paragraph、table、cluster range、inline/link/resource索引和cell projection；`UiResolvedTextLayout`通过type-erased handle携带同一artifact，UI layout、renderer、image dependency和link hit-test都能回到该产物。HTML subset、BBCode、三种Markdown marker、emoji shortcode、自定义BBCode decorator、block/list/table、横排/竖排inline layout、image/link和bounded 256-entry/8 MiB cache也是真实实现，不能推倒后重新堆一套parser。

但这仍不是可承载大型项目、插件/DLC、UGC、复杂文档和无障碍产品的工程级Rich Text service。入口没有input/token/node/span/attribute/depth/output/time/deadline/cancellation预算，返回值又是不可失败的`RichParseResult`；未知标签、错误属性、结构截断、受限降级和decorator失败都没有结构化diagnostic、source map或subset version。缓存限制的是结果驻留，不限制单次解析、grapheme对齐、cell projection、layout或第三方decorator占用CPU和内存。

更严重的是样式合同双向断裂：`StyleOverride`公开`italic`、`letter_spacing`和OpenType `features`，但内建markup parser只会生产italic，后两项在production parser没有producer；`TextStyle`与`resolve_rich_run_style()`又只真正传递weight、font size和family。italic只被renderer用于选择`UiTextRunKind::Italic`标签，没有选择italic face；letter spacing和features没有进入shaping。当前测试主要证明italic字段被解析和run被分类，不能证明像素或glyph结果正确。

inline object也只是结构占位。Image没有alt text、tooltip、单位、region、load/error状态；Widget只有裸`u64 id + size`，renderer实际画实心矩形，没有child ownership、layout/paint/input/a11y或generation；Icon只是一个font glyph。Link仅有href，缺action/target/tooltip/visited/disabled/trust/provenance。URI scheme在resource locator和host request边界已有再次校验，这一点应保留，但它不能替代内容信任、bidi spoof、插件执行和富文本语义安全模型。

产品侧证据最直接：对`zircon_editor/src`、`zircon_app/src`和`assets`共5,949个tracked文件检索`UiRichTextFormat`、`rich_text_format`和`RichText`，命中为0。当前能力主要存在于runtime源码、fixture、test和proof command；没有UI Asset属性编辑、实时预览、语法诊断、style/decorator asset、Editor compile validation或真实产品迁移。`docs/plans/zircon_runtime/text/07`虽已实现compiled artifact主链，但其failure仍为open，managed Cargo、真实WGPU framebuffer和新产品PNG证据尚未完成。

本轮不新增P0。Runtime11B已经唯一拥有富文本budget/diagnostic/decorator isolation、全量rich layout和巨大intrinsic extent等父问题；Runtime78/79/82分别拥有a11y、GPU renderer与editing父边界。Runtime84登记48项P1、12项P2和48项资格门，用于把Rich Text从“能解析并画出若干样例”收束为owner-qualified、budgeted、diagnosable、incremental、semantic和可产品化的服务。

## 2. 审查边界、证据等级与冻结快照

### 2.1 证据等级

| 等级 | 本轮使用方式 | 能证明什么 |
|---|---|---|
| E3 | 逐文件读取parser/model/cache/layout/UI/render/input/a11y/interface owner，并沿artifact handle、resource和link路径追调用 | 当前生产合同、复杂度、生命周期和断链 |
| E2 | `git grep`检索rich contract/style/handle/product authoring与consumer；对Editor/App/assets作全tracked inventory负检索 | owner不存在、字段无consumer、产品接入为0等静态事实 |
| E1 | 读取45个聚焦test/proof文件和Text07 failure/child records，但未运行 | 测试意图、静态覆盖与仍待托管验证的证据 |
| E0 | 未运行Cargo、Editor、真实WGPU、screen reader、fuzz、fault、soak或benchmark | 不得宣称动态通过、像素正确或性能优于Unreal |

### 2.2 Zircon冻结范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 证据 |
|---|---:|---|
| Parser / model / cache production | 19 / 4,580 / 4,207 / 151,387 / 19 | DTO、HTML/BBCode/Markdown、table/list、decorator、compiled artifact、registry与cache |
| Layout / UI / render / a11y / interface production | 41 / 11,529 / 10,824 / 410,343 / 28 | horizontal/vertical/table layout、handle、resource、paint、link、semantic extraction与public DTO |
| Production去重合计 | **60 / 16,109 / 15,031 / 561,730 / 47** | fingerprint `3270460f2e25d334fa62631c93b689be44c3841f4ac0d205bea9420303a9381e` |
| Focused tests / proofs | **45 / 15,401 / 14,260 / 554,770 / 332** | parser/layout/cache/artifact/hit/render/table/product proof静态证据；fingerprint `c3c759a469ffe3c6772d41731b46bd25f4c202c65f708a8d86dfc14915e802e1` |
| 全集去重合计 | **105 / 31,510 / 29,291 / 1,116,500 / 379** | fingerprint `214be21bc8e025fa99af60d9477f05595354a58f126d8e459e92f8eaa5f3addf` |

冻结时范围内5个文件dirty：`text/rich/parser.rs`、`text/rich/tests.rs`、`text/layout/rich_advance_index/tests.rs`、`ui/text/layout_engine/wrapping/tests.rs`和`tests/runtime_text_rich_blocks.rs`。其中parser/test由并行Text五项修复Session持有；本报告记录的是当前工作树，不把未完成的managed validation当成accepted milestone。冻结时`parser.rs` SHA-256为`20b1b7b5a60979cde4da77c4df9479ab7ab813e9b78b212cdec359f09045c6f5`，`tests.rs`为`ecb015bbc61e75107fa6156637666a5b66c72dfa6cd992f42747b91db0573a26`。

### 2.3 参考冻结范围

| 参考 | 文件 / 行 / 非空行 / bytes | 使用边界 |
|---|---:|---|
| Unreal Slate / UMG | 13 / 2,326 / 1,894 / 77,960 | marshaller、decorator owner、style DataTable、design-time lifecycle、真实widget/image run与Editor validation |
| Godot | 7 / 11,294 / 9,687 / 397,330 | typed item tree、incremental paragraph、thread/cancel、selection/search、image/link/list/table和a11y产品面 |
| Bevy | 3 / 2,908 / 2,601 / 104,051 | child entity TextSpan、change detection与typed mutable access；不作为markup parser参考 |
| Fyrox | 3 / 1,886 / 1,723 / 63,206 | serializable RunSet、resource load status/wait/error；不作为parser性能上限 |
| Unity Graphics | 3 / 175 / 133 / 9,001 | 本地corpus只有SRP说明与TMP sample/settings/shader，无TextCore/TMP engine源码，只作负边界 |
| 参考去重合计 | **29 / 18,589 / 16,038 / 651,548** | fingerprint `311d118fe2872285aebbf62c0aedd0ca5496d3c47b950eb0b2a633e6c9b4691c` |

不能错误推导：Unreal默认markup parser本身也较简单，Godot parser同样没有完整hostile-input budget；两者主要证明产品生命周期、typed content tree、authoring和incremental document边界。Zircon若目标是性能和可靠性优于Unreal，必须在这些产品能力之上补预算、隔离、可观测性和同负载证据，而不是照抄语法。

## 3. 当前可保留底座

### 3.1 Canonical compiled artifact

`CompiledRichText`以`Arc<str>`和`Arc<[T]>`持有markup、plain text、runs、paragraphs、tables、cluster ranges及inline/link/resource indexes。`UiResolvedTextLayout.rich_text_artifact`让layout、renderer、resource streamer和link hit-test共享同一解析结果，修复了旧Text07中frame consumer反复parse的问题。这个“先编译、后多consumer读取”的方向正确。

### 3.2 Parser与局部复杂度修复

HTML/BBCode tokenizer支持quoted attributes、实体解码、block/list/table与受控resource locator；table已有8层nesting、64列、rowspan 64和padding clamp。当前dirty parser还加入了未闭合delimiter frontier，使HTML/BBCode/Markdown的末端搜索按单调frontier推进；深度超过32时用active-tag index定位同名close。旧Runtime11B关于这些具体搜索仍为O(n²)的描述已经过时，但输入规模、tag depth、metadata clone、decorator work和layout预算仍未解决。

### 3.3 Cache、generation与single-flight雏形

process-global rich cache有256项/8 MiB驻留上限，key包含markup hash/len、format、parser identity、decorator generation和emoji generation，并用原markup作collision check；同key用`OnceLock`提供single-flight，暴露hit/miss/parse/eviction统计。这些机制应迁入session/project-owned service，而不是删除后退回每帧parse。

### 3.4 Layout、resource与interaction链

横排和竖排路径都能按rich run选择font family/weight/size并处理inline metrics；table有preferred/final两阶段cell layout；Image dependency会进入`ui_texture`资源收集；link pointer hit使用resolved caret geometry，最终host request再次校验scheme。当前实现不是“完全没有富文本布局、图片或链接”。

### 3.5 参考中应吸收的边界

- Unreal把parser/writer、marshaller、decorator instance和`URichTextBlock` owner分开；style set和image row是资产，design-time rebuild、resource release、compile validation与explicit refresh都有产品生命周期。
- Unreal `FSlateWidgetRun`拥有真实child widget、arrange/paint/baseline，不用占位矩形冒充widget。
- Godot用typed Item tree表达paragraph/list/table/meta/image/effect，并保存alt、tooltip、units、region、owner等信息；以paragraph invalid index、worker task、stop flag和progress处理大文档。
- Godot把selection、search、copy、context、visible characters、scroll/follow与rich document放在同一产品模型，不把rich text永久排除在editing/viewport之外。
- Bevy的`TextSpan`是带entity identity和change detection的真实child；Fyrox RunSet是可序列化、可反射并能等待resource状态的document数据，而非只在parser内部存在的临时run。

## 4. P0归属与不得重复计数

本报告新增P0为0。以下父阻断继续由原报告唯一计数：

| 父owner | 继续开放的边界 | Runtime84处理方式 |
|---|---|---|
| Runtime11B / Text07 | RichParseBudget、diagnostic、decorator isolation、rich/vertical全量layout、巨大intrinsic extent及managed验证 | 细化为P1和资格门，不重复计P0；当前frontier/index修复按source drift更新事实 |
| Runtime78 | semantic tree、relation、action、platform adapter与产品screen reader | 只登记rich source/inline/list/table语义断链 |
| Runtime79 / Runtime11C | UI painter order、icon/image GPU、WGPU framebuffer与batch资格 | 只登记rich inline object和artifact handoff差距 |
| Runtime82 | document revision、selection/caret/IME/clipboard/secure editing | 只登记rich document没有接入editing authority |
| Runtime73/74/75/76/77 | style、template、component、layout、input共同服务 | Runtime84只定义rich-specific contract和integration gate |

## 5. 关键断链详证

### 5.1 Parser API无法表达失败、预算与版本

`RichTextParser::parse()`直接返回`RichParseResult`。HTML、BBCode和Markdown都没有`RichParseBudget`或context参数，custom decorator也没有deadline/cancellation。未知tag会按literal/忽略路径继续，mismatched HTML close会弹出matching tag及其上层stack，错误没有code/path/range。public `UiRichTextFormat::{Html, Markdown, BbCode}`也没有subset/schema version，因此内容cook、runtime和Editor无法协商能力或稳定重放。

### 5.2 线性搜索修复不等于线性总成本

frontier修复消除了重复寻找未闭合delimiter的特定二次扫描，active-tag index降低了深层同名close搜索；但`ActiveTag`仍保存累计clone的`StyleOverride`和link，嵌套深度无上限。`align_runs_to_graphemes()`先把累计style、family/features vector、link和inline metadata复制到每个grapheme，再尝试合并。输入虽可线性扫描，内存和clone成本仍可随grapheme数量与metadata体积相乘。

### 5.3 Decorator是frame-path任意代码

`DecoratorRegistry`是`Vec`，每个token逐项调用`supports()`；registry没有namespace、owner lease、unregister或provider generation。decorator可以panic、hang、递归、分配大量output或阻塞UI layout，single-flight waiter也会无限等待；当前没有panic隔离、work unit、deadline、cancellation、output budget、diagnostic和negative cache。

### 5.4 Artifact identity与缓存生命周期不可靠

parser registry和cache都是process-global `OnceLock`，没有project/session/plugin/DLC owner、shutdown或retired generation；旧generation只能等待偶然eviction。identity/generation使用递增atomic而没有exhausted outcome。`CompiledRichText::from_projection()`又构造空source、默认format和默认generation，模糊了projection与真实source artifact的身份。

`UiRichTextArtifactHandle::PartialEq`只比较保存的`TypeId`。两个内容完全不同但运行时类型同为`CompiledRichText`的handle会相等，现有unit test还明确固化这一行为；这不能代表artifact identity，可能让layout equality、dirty detection或cache复用看不到实际内容变化。

### 5.5 Style parser和shaper之间丢字段

BBCode/HTML/Markdown会写italic，`StyleOverride`还公开letter spacing和OpenType features；但后两项在内建production parser没有赋值点，基础`TextStyle`也没有对应字段，`resolve_rich_run_style()`仅应用weight/font size/family。renderer通过`ui_run_kind()`把italic标成run kind，不会选择italic face；features只参与内存估算，letter spacing也未进入shaping。这是producer与consumer同时缺失的合同断链，不能用italic parser field assertion验收。

### 5.6 Cell projection与layout成本未受控

为每个table cell建立projection时会重新扫描全部runs、paragraphs和tables，形成`cells * (runs + paragraphs + tables)`成本。layout又先为每个cell执行no-wrap preferred layout，再按最终column width重做一次；provisional block extent使用`f32::MAX / 4`，intrinsic path还能以source length乘line height构造巨大extent。cell总数、token、attribute和row数量未被全局budget约束，现有column/rowspan cap不足以限制文档级工作量。

### 5.7 Inline object只有显示占位，没有对象生命周期

Widget ref只有`id`和`size`，没有owner/surface/generation/child handle，renderer只填充矩形；它不参与child arrange、paint、input route、focus、a11y或unload。Icon只是glyph+font，未绑定icon asset generation。Image虽能收集resource id，却没有alt/tooltip/region/tint/relative unit/load/error或fallback contract。resource index也只收Image，不表达Icon font、Widget child或decorator-owned dependency。

### 5.8 Layout DTO反向重建语义

resolved line/run DTO复制text和glyph advances，没有stable span id、style handle、inline identity或semantic node。renderer再用source ranges回查compiled artifact以恢复run kind/link/inline object。某些路径还会为slice重新measure以重建item advances。artifact handle equality又不可靠，使“layout属于哪一代source”缺少强合同。

### 5.9 Rich document被排除在增量、editing与a11y之外

viewport fast path明确只接受Plain、horizontal、nowrap、clip、non-editable；rich、wrapped、vertical和editable全部全量layout。rich resolved layout把`editable`设为`None`，Runtime82的document/selection/IME/clipboard authority没有接入。accessibility name/extract读取template scalar `text/label/value`，不消费compiled rich artifact；markup可能被原样朗读，link/image/list/table也没有稳定semantic child、alt/action或structure projection。

### 5.10 语法名与产品能力高于真实实现

Markdown只识别bold、italic和backtick marker，没有escape/nesting/link/block语义；HTML是自定义whitelist和recovery，不是HTML parser。BBCode虽更广，但bidi override可直接注入控制字符，没有trusted content/spoof policy。把三者作为无版本`UiRichTextFormat`公开，会让author误以为兼容完整标准。

### 5.11 Editor与产品链为空

Runtime component catalog含showcase和大量tests/proofs，但Editor/App/assets的5,949个tracked文件中没有富文本format/authoring命中。没有syntax diagnostic panel、source-range highlight、style set/data table、decorator asset、inline object picker、live preview、localization escaping、a11y alt校验、package dependency或hot reload migration。功能存在于代码层，不等于工程产品已可用。

## 6. P1工程化差距

### 6.1 Parser、token与contract

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RRT-P1-001 | 无input/token/node/span/attribute/depth/output/time预算 | `RichParseBudget`贯穿tokenizer/parser/decorator/layout admission |
| RRT-P1-002 | parse不可失败且无structured diagnostic | `RichParseOutcome { artifact, diagnostics, status, consumed_budget }` |
| RRT-P1-003 | 无source map、error code、provenance与recovery记录 | token/node/span均保存source range和typed recovery reason |
| RRT-P1-004 | Html/Markdown/BbCode无subset/schema version | versioned parser descriptor与cook/runtime capability negotiation |
| RRT-P1-005 | `parse()`从compiled artifact clone完整结果 | consumer持有immutable snapshot/view，不复制全部vectors |
| RRT-P1-006 | grapheme对齐按grapheme复制累计style/link/inline metadata | interned style/span arena与range-based alignment |
| RRT-P1-007 | ActiveTag累计clone且depth无上限 | bounded stack、delta style与明确TooDeep outcome |
| RRT-P1-008 | decorator registry按token线性遍历 | compiled dispatch table、namespace和deterministic priority |
| RRT-P1-009 | decorator无panic/deadline/cancel/output budget隔离 | provider work unit、catch boundary、quota和typed failure |
| RRT-P1-010 | decorator/parser无owner、unregister、lease与thread contract | project/session/plugin-qualified provider registry与revoke fence |
| RRT-P1-011 | `u32` range超大输入饱和且Deserialize无validate | admission前拒绝超限，validated range DTO不做saturating identity |
| RRT-P1-012 | 任意finite positive font/image size和bidi control可进入布局 | trusted-content policy、geometry clamp与spoof diagnostics |

### 6.2 Artifact、cache与lifecycle

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RRT-P1-013 | process-global parser/cache跨project与session共享 | `RuntimeRichTextService`由runtime context持有并显式shutdown |
| RRT-P1-014 | single-flight遇到hang无deadline/cancel/typed wakeup | cancellable parse job与所有waiter一致的terminal receipt |
| RRT-P1-015 | cache residency低估allocator/hash/Arc/index开销 | measured resident bytes、tenant quota和admission/eviction reason |
| RRT-P1-016 | 旧decorator/emoji generation只靠偶然eviction | generation retirement、targeted invalidation和last-use lease |
| RRT-P1-017 | parser identity/generation atomic wrap无exhaustion | non-reusing qualified generation或显式Exhausted状态 |
| RRT-P1-018 | `from_projection()`伪造空source/default format/generation | projection使用独立类型并保留parent artifact identity |
| RRT-P1-019 | 每cell projection反复扫描整个artifact | 一次构建interval/index，cell view按range常数或对数查询 |
| RRT-P1-020 | dependency index只覆盖Image | icon/font/widget/decorator resource统一typed dependency closure |
| RRT-P1-021 | artifact handle equality只比较`TypeId` | 比较stable artifact id + generation，或明确禁止value equality |
| RRT-P1-022 | cache stats为global saturating counters且无租户维度 | project/parser/provider维度的bounded telemetry与reset snapshot |

### 6.3 Style、layout、inline object与interaction

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RRT-P1-023 | italic未选italic face，letter spacing/features未进shaping | rich style完整映射到font request、shaping key和glyph artifact |
| RRT-P1-024 | Markdown能力只有三种marker却以Markdown公开 | 名称改为versioned minimal subset或完成明确支持矩阵 |
| RRT-P1-025 | HTML whitelist/recovery不等于HTML且无diagnostic | versioned HTML subset、deterministic recovery和authoring error |
| RRT-P1-026 | Widget inline object只画实心矩形 | real child widget lease、measure/arrange/paint/input/a11y lifecycle |
| RRT-P1-027 | Widget裸`u64`无owner/surface/generation | qualified child identity与destroy/rebind/revoke合同 |
| RRT-P1-028 | Icon是无generation的font glyph | icon asset/font face lease、fallback与render readiness |
| RRT-P1-029 | Image无alt/tooltip/units/region/load/error/fallback | typed image item与resource outcome、semantic fallback |
| RRT-P1-030 | Link只有href，缺action/target/tooltip/state/trust | typed link action、principal、navigation policy和semantic state |
| RRT-P1-031 | rich/wrapped/vertical没有viewport/incremental layout | paragraph/span dirty index、visible range和retained layout document |
| RRT-P1-032 | table每cell preferred+final两次完整layout | cached intrinsic metrics、dirty cell/track和bounded relayout |
| RRT-P1-033 | provisional/intrinsic extent可接近极大f32 | geometry budget、checked extent和TooLarge outcome |
| RRT-P1-034 | resolved DTO复制字符串并可能重新measure advances | stable span/layout arena与renderer直接消费prepared run |
| RRT-P1-035 | rich layout永远`editable: None` | 与Runtime82 revision/selection/IME/clipboard共享document authority |
| RRT-P1-036 | renderer按source range反向拼回run/inline语义 | prepared draw item携带stable span/object/semantic id和generation |
| RRT-P1-037 | table/list没有header、caption、ordered marker与semantic structure | typed block tree同时驱动layout、paint、copy和a11y |
| RRT-P1-038 | table总cell/row/token数量无document级上限 | parse/layout统一work budget和partial/failed policy |

### 6.4 Accessibility、security、Editor与qualification

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RRT-P1-039 | a11y读取raw scalar而不读compiled artifact | `RichSemanticProjection`与视觉布局共享source generation |
| RRT-P1-040 | image/link/widget/list/table没有semantic child/action/alt | typed semantic tree、relations、actions与fallback text |
| RRT-P1-041 | bidi override/control无content trust与spoof policy | trusted authoring gate、isolation default和visual/logical diagnostic |
| RRT-P1-042 | scheme校验之外无link principal/domain/provenance | parse、preview、runtime navigation共享security policy snapshot |
| RRT-P1-043 | Editor无rich property、source editor、preview与diagnostics | UI Asset Editor接入同一parser/artifact/diagnostic service |
| RRT-P1-044 | 5,949个Editor/App/assets文件产品命中为0 | 迁移至少一个真实Editor flow和一个runtime/WOC flow |
| RRT-P1-045 | 无style set、decorator/image row资产及cook依赖 | versioned RichStyleAsset/DecoratorDescriptor/InlineResourceArtifact |
| RRT-P1-046 | source/style/provider hot reload无prepare/atomic publish/last-good | qualified generation transaction与targeted surface invalidation |
| RRT-P1-047 | Text07 managed Cargo、WGPU framebuffer和新产品PNG仍未闭合 | 保留open failure，完成托管验证后再发布accepted evidence |
| RRT-P1-048 | 无fuzz/fault/scale/同负载性能资格 | corpus+property fuzz、malicious decorator、large doc和跨引擎benchmark |

## 7. P2扩展差距

| ID | 扩展能力 | 工程边界 |
|---|---|---|
| RRT-P2-001 | 完整Markdown/GFM profile | versioned extension set、escaping、AST和conformance corpus |
| RRT-P2-002 | 更广HTML/CSS text subset | 明确白名单、sanitization、layout mapping和unsupported diagnostic |
| RRT-P2-003 | ruby/annotation与复杂东亚排版 | typed ruby item、shaping/layout/a11y和fallback |
| RRT-P2-004 | drop cap、small caps与initial letter | paragraph metrics、font feature和fragmentation |
| RRT-P2-005 | animated/custom text effects | time source、material/effect budget、visibility pause和sandbox |
| RRT-P2-006 | sprite/vector/animated inline media | resource lease、baseline、frame sync、a11y和fallback |
| RRT-P2-007 | rich document search/highlight | source/plain/visual range mapping与incremental index |
| RRT-P2-008 | context menu、copy-as-plain/copy-as-markup | permission、sanitization、selection provenance和clipboard formats |
| RRT-P2-009 | table caption/header/sort/freeze/fragmentation | semantic table model与large-table virtualization |
| RRT-P2-010 | nested list marker/style/counter system | typed counter scope、layout、copy和a11y semantics |
| RRT-P2-011 | localization-safe rich message composition | argument/translation/markup分域转义与style/action allowlist |
| RRT-P2-012 | remote collaborative rich document | revision/operation identity、conflict policy、untrusted content gate |

## 8. 目标架构

| 层 | 目标对象 | 责任 |
|---|---|---|
| Source | `RichMarkupSource`、`RichParserDescriptor` | source identity、format/subset version、trust、locale、owner |
| Admission | `RichParseBudget`、`RichParseRequest` | input/token/node/depth/output/time/provider quota与cancellation |
| Compile | `RichParseOutcome`、`RichDiagnosticSet`、`CompiledRichTextArtifact` | typed status、source map、immutable styled/block tree、dependency closure |
| Registry | `RichParserProviderRegistry`、`RichDecoratorLease` | project/session/plugin owner、priority、generation、revoke和isolation |
| Runtime | `RuntimeRichTextService`、`RichTextSnapshot` | cache、single-flight、last-good、reload、telemetry和shutdown |
| Layout | `RichLayoutDocument`、`RichLayoutGeneration` | paragraph/span dirty、viewport、selection、stable prepared runs |
| Inline | `RichInlineObjectLease`、`InlineResourceOutcome` | image/icon/widget真实resource与child lifecycle |
| Semantics | `RichSemanticProjection` | visual/plain/source range、link/list/table/image/widget a11y语义 |
| Product | `RichStyleAsset`、`RichAuthoringSession` | style/decorator asset、preview、diagnostic、cook和migration |
| Evidence | `RichGenerationReceipt`、`RichQualificationReport` | source/provider/layout/render/a11y generation与fault/perf证据 |

硬约束：parser不做renderer IO；renderer不重新parse；decorator不持有无owner全局状态；projection不伪造source identity；rich与plain共享font/shaping基础但不能丢rich style；visual与a11y必须来自同一artifact generation；Editor preview与shipping runtime使用同一compiler/service合同。

## 9. 分层重构路线

### M0：Truthfulness与断路止血

新增失败测试证明即使`StyleOverride`携带italic/features/letter spacing也没有完整进入shaping、handle equality把不同artifact判等、a11y读取raw markup、Widget只是矩形。将Markdown/HTML命名改为显式subset capability；保留Text07 failure open，直到托管动态证据完成。

### M1：Versioned contract与diagnostic

落地`RichParserDescriptor`、source identity/trust、`RichParseBudget`、typed outcome、source map和diagnostic。先用现有parser填充新合同，再删除不可失败旧入口，禁止双轨长期共存。

### M2：Artifact与style完整性

建立interned style/span/block arena，修正italic/letter spacing/OpenType feature到font/shaping key的完整映射；拆分真实source artifact和cell projection，修复handle identity/equality并建立validated serialization。

### M3：Owned service与provider isolation

把global parser/cache迁入runtime context；provider/decorator具有owner、namespace、priority、generation、quota、panic boundary、deadline、cancellation、revoke fence和shutdown。single-flight只等待有界job并传播同一terminal outcome。

### M4：Incremental document与table

引入paragraph/span/cell dirty index、visible range、retained layout和prepared run arena；cell index一次构建，intrinsic/final metrics可复用。把rich接入Runtime82 document revision/selection/IME/clipboard，不再永久全量layout。

### M5：真实inline object、resource与security

Image/Icon/Widget/Link全部使用typed owner-qualified object；真实child widget参加measure/arrange/paint/input/a11y，resource有loading/error/fallback。统一link principal、scheme/domain/target policy和bidi/content trust。

### M6：Semantic projection与accessibility

由compiled block/span tree生成plain text、semantic children、list/table structure、link action和image/widget alt；与visual layout共享artifact/layout generation，按Runtime78接入平台adapter。

### M7：Editor、cook与产品迁移

建立RichStyleAsset、decorator/image descriptor、source editor、range diagnostic、live preview、package dependency和atomic hot reload。先迁移一个Editor flow与一个runtime/WOC flow，禁止继续只增加fixture。

### M8：Fault、scale与性能资格

覆盖malformed/deep/huge markup、decorator panic/hang/output amplification、resource failure、reload/unload、large table、rich selection、screen reader和shutdown。以同字体、同markup、同inline resource、同viewport和同机器对照Unreal/Godot；没有原始数据不得声称性能优于Unreal。

## 10. 48项资格门

| Gate | 验收条件 |
|---|---|
| G01 | 所有format由versioned descriptor声明subset/capability，旧无版本入口无production caller |
| G02 | input/token/node/span/attribute/depth/output/time均有budget并产出consumption receipt |
| G03 | invalid、unsupported、too-deep、too-large、cancelled和provider-failed为不同typed status |
| G04 | 每个diagnostic有code、severity、source range、parser/provider generation和recovery |
| G05 | malformed corpus不会panic、hang、越界或静默伪装完整成功 |
| G06 | source/plain/cluster/span/object range映射通过UTF-8、grapheme、BiDi和entity property tests |
| G07 | unterminated delimiter与deep mismatched close有linear work counter，不只做wall-clock断言 |
| G08 | nested style不按grapheme复制可变metadata，allocation/clone budget有基准证据 |
| G09 | decorator dispatch不随无关provider数逐token线性增长 |
| G10 | decorator panic/hang/递归/output amplification被quota、deadline和cancel隔离 |
| G11 | provider注册要求owner/namespace/version/priority/lease，duplicate policy确定 |
| G12 | plugin/DLC revoke后新parse拒绝，旧snapshot在lease结束前安全读取 |
| G13 | cache按project/session/provider隔离并有resident/admission/eviction预算 |
| G14 | same-key single-flight对success/failure/cancel传播一致terminal outcome |
| G15 | old generation可定向retire，不依赖偶然LRU eviction |
| G16 | parser/decorator/emoji generation不复用，exhaustion有typed outcome |
| G17 | artifact serialization验证range、finite number、schema、checksum和size cap |
| G18 | projection保留parent artifact identity，不伪造空source/default format |
| G19 | 不同compiled artifact handle不再因相同`TypeId`而相等 |
| G20 | cell projection通过一次性index构建，复杂度不再按cell重复扫描全artifact |
| G21 | resource dependency闭包包含image/icon/font/widget/decorator owner和generation |
| G22 | cache/parse telemetry按租户有界、可快照且不会泄漏markup内容 |
| G23 | italic选择真实italic/oblique face并有glyph/framebuffer证据 |
| G24 | letter spacing和OpenType features进入shaping/cache key且有golden |
| G25 | underline/strike/color/background与source/visual ranges在BiDi/vertical下正确 |
| G26 | Markdown/HTML subset conformance corpus与unsupported diagnostic匹配公开能力 |
| G27 | Widget inline object拥有真实child并参加measure/arrange/paint/input/focus/a11y |
| G28 | Widget identity包含surface/owner/generation，destroy/rebind无悬挂引用 |
| G29 | Icon/Image resource有generation、readiness、failure和fallback outcome |
| G30 | Image alt/tooltip/units/region/tint进入layout、render和semantic projection |
| G31 | Link action/target/tooltip/state/principal由typed policy校验且host再次验证 |
| G32 | untrusted bidi control和markup/action注入有隔离、diagnostic与security tests |
| G33 | rich/wrapped/vertical viewport只layout可见/overscan段并保持稳定geometry |
| G34 | paragraph/span/cell mutation只失效受影响layout，不做全document重建 |
| G35 | table intrinsic/final pass复用metrics且有row/cell/work budget |
| G36 | extreme finite size/extent返回TooLarge，不把极大f32传给layout/render/spatial index |
| G37 | prepared render run携带stable span/object/style/generation，不按range反向猜测 |
| G38 | rich document接入revision、selection、caret、IME、clipboard和undo/redo authority |
| G39 | source/plain/visual selection与copy在inline object、BiDi、vertical、table中一致 |
| G40 | visual与a11y使用同一artifact/layout generation且无raw markup朗读 |
| G41 | image/link/widget/list/table生成正确semantic child、role、name、action与structure |
| G42 | Editor property/source editor显示range diagnostic并使用shipping parser/artifact |
| G43 | RichStyleAsset/decorator/image descriptor有cook dependency、schema与version |
| G44 | source/style/provider reload通过prepare/validate/atomic publish并保留last-good |
| G45 | 至少一个Editor flow和一个runtime/WOC flow有真实rich authoring与产品迁移证据 |
| G46 | managed Cargo与Text07 open failure全部闭合，不用静态测试代替验证receipt |
| G47 | 真实WGPU、a11y adapter、resource failure、plugin unload和shutdown矩阵通过 |
| G48 | 同负载p50/p95/p99、alloc、RSS、layout/paint/frame数据附原始证据后才可声称优于Unreal |

## 11. 禁止的临时实现

1. 禁止在renderer、a11y或hit-test路径重新parse markup。
2. 禁止用`TypeId`、指针地址或markup hash单独冒充artifact identity。
3. 禁止继续公开italic/features/letter spacing却缺producer/consumer或只设置run标签。
4. 禁止用实心矩形称作inline widget，或用裸`u64`跨surface引用child。
5. 禁止让custom decorator在UI frame路径执行无budget、无deadline、无panic隔离的任意代码。
6. 禁止把256项/8 MiB cache称作完整DoS防护；单次parse/layout必须另有work budget。
7. 禁止把三种marker称作完整Markdown，或把whitelist parser称作HTML兼容。
8. 禁止让accessibility朗读raw markup，或用链接颜色代替link semantic/action。
9. 禁止用测试fixture、proof command、showcase catalog冒充Editor/App产品接入。
10. 禁止另建Editor-only parser/preview；authoring和shipping必须共享versioned compiler合同。
11. 禁止在未关闭Text07 managed validation与WGPU证据前把状态改为accepted/complete。
12. 禁止在没有相同markup/font/resource/viewport/机器的原始benchmark时声称性能优于Unreal。

## 12. 本轮完成边界

本轮完成105个Zircon入选文件、29个参考文件的静态current-source审查，沿source -> parser/decorator -> compiled artifact/cache -> layout/table -> render/resource/link -> a11y -> Editor/App/assets追踪完整链路；登记0项新增P0、48项P1、12项P2、M0-M8路线和48项资格门。当前frontier与active-tag index修复已按工作树事实记录，但其managed validation仍待完成；Text07 failure继续open。

本轮只新增review文档并更新索引，没有修改production/test/assets，没有运行Cargo、Editor、WGPU、screen reader、fuzz、fault、soak或benchmark；tooling按用户要求暂不纳入。下一实施入口应是M0 truthfulness失败测试和M1 versioned budget/diagnostic contract，不能从新增tag或renderer fallback开始。
