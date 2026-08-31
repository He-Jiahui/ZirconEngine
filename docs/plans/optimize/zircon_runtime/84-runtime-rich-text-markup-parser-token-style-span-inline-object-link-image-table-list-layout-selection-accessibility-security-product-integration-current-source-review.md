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

Zircon的富文本并非只有临时字符串替换。当前源码已经建立`CompiledRichText`这一份`Arc`持有的解析产物，包含源markup、plain text、style run、paragraph、table、typed inline/link/dependency索引和cell projection；document-sized cluster range已从parser artifact删除，grapheme/cluster所有权留在实际消费它的shaping/layout。`UiResolvedTextLayout`通过type-erased handle携带同一artifact，UI layout、renderer、typed image dependency和link hit-test都能回到该产物。HTML subset、BBCode、三种Markdown marker、emoji shortcode、自定义BBCode decorator、block/list/table、横排/竖排inline layout、image/link和bounded 256-entry/8 MiB cache也是真实实现，不能推倒后重新堆一套parser。

但这仍不是可承载大型项目、插件/DLC、UGC、复杂文档和无障碍产品的工程级Rich Text service。入口没有input/token/node/span/attribute/depth/output/time/deadline/cancellation预算，返回值又是不可失败的`RichParseResult`；未知标签、错误属性、结构截断、受限降级和decorator失败都没有结构化diagnostic、source map或subset version。缓存限制的是结果驻留，不限制单次解析、grapheme对齐、cell projection、layout或第三方decorator占用CPU和内存。

更严重的是样式合同双向断裂：`StyleOverride`公开`italic`、`letter_spacing`和OpenType `features`，但内建markup parser只会生产italic，后两项在production parser没有producer；`TextStyle`与`resolve_rich_run_style()`又只真正传递weight、font size和family。italic只被renderer用于选择`UiTextRunKind::Italic`标签，没有选择italic face；letter spacing和features没有进入shaping。当前测试主要证明italic字段被解析和run被分类，不能证明像素或glyph结果正确。

inline object的MVP生命周期已部分收敛。Image已有alt/tooltip但仍缺单位、region、load/error状态；Widget markup编译为owner-local `RichInlineWidgetSlotId + size`，Surface只在当前树layout期解析direct child，并由普通child arrange/render/input/a11y路径负责生命周期，renderer实心矩形已删除且不保留跨帧binding。Icon已从family glyph硬切为typed image asset并统一layout/paint geometry，但仍缺generation/readiness/intrinsic metric，font-icon lease尚未实现。Link已有typed target/tooltip，仍缺action/visited/disabled/trust/provenance。URI scheme在resource locator和host request边界已有再次校验，这一点应保留，但它不能替代内容信任、bidi spoof、插件执行和富文本语义安全模型。

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

HTML/BBCode tokenizer支持quoted attributes、实体解码、block/list/table与受控resource locator；table已有8层nesting、64列、rowspan 64和padding clamp。当前dirty parser还加入了未闭合delimiter frontier，使HTML/BBCode/Markdown的末端搜索按单调frontier推进；深度超过32时用active-tag index定位同名close。旧Runtime11B关于这些具体搜索仍为O(n²)的描述已经过时。grapheme alignment 已使用单调run cursor、canonical ASCII fast path与仅在新输出run materialize时clone metadata；source/token/tag/block/table/output与metadata预算也已接入。剩余瓶颈必须由managed profile重新证明，不能沿用旧结论。

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

parser registry和cache都是process-global `OnceLock`，没有project/session/plugin/DLC owner、shutdown或retired generation；旧generation只能等待偶然eviction。identity/generation使用递增atomic而没有exhausted outcome。初始快照中的`CompiledRichText::from_projection()`还会构造空source、默认format和默认generation，模糊projection与真实source artifact身份；2026-08-26已确认生产cell path使用`UiParsedText` range/index view强持有parent `Arc<CompiledRichText>`，因此删除这个只剩测试使用的伪构造器。RRT-P1-018进入`implementation_complete_static_checked / managed_validation_pending`。

2026-08-26 current-source修正：`UiRichTextArtifactHandle::PartialEq`不再把`TypeId`当作完整身份。interface只负责“payload类型 + owner identity”的type erase；compiled-rich owner比较完整immutable compiled artifact语义（包含source、format、parser/decorator/emoji generation、parsed runs及projection indexes，但排除`estimated_bytes`驻留统计），resolved-glyph owner比较source/origin/font generation/style/writing mode、glyph/layout line及logical-virtual rebuild input。同一`Arc`有O(1)快速路径，可再生的logical shaped fragment cache不参与身份。回归分别覆盖相同身份、内容/格式/解析器代际变化、驻留估算变化、font generation变化、不同payload类型及`UiResolvedTextLayout` dirty equality。因此原`TypeId`误判已完成实现与静态检查，RRT-P1-021进入`implementation_complete_static_checked / managed_validation_pending`；process-global owner和generation retirement/exhaustion仍然开放。

### 5.5 Style parser和shaper之间丢字段

BBCode/HTML/Markdown会写italic，`StyleOverride`还公开letter spacing和OpenType features；但后两项在内建production parser没有赋值点，基础`TextStyle`也没有对应字段，`resolve_rich_run_style()`仅应用weight/font size/family。renderer通过`ui_run_kind()`把italic标成run kind，不会选择italic face；features只参与内存估算，letter spacing也未进入shaping。这是producer与consumer同时缺失的合同断链，不能用italic parser field assertion验收。

### 5.6 Cell projection与layout成本未受控

为每个table cell建立projection时会重新扫描全部runs、paragraphs和tables，形成`cells * (runs + paragraphs + tables)`成本。layout又先为每个cell执行no-wrap preferred layout，再按最终column width重做一次；provisional block extent使用`f32::MAX / 4`，intrinsic path还能以source length乘line height构造巨大extent。cell总数、token、attribute和row数量未被全局budget约束，现有column/rowspan cap不足以限制文档级工作量。

### 5.7 Inline object只有显示占位，没有对象生命周期

Widget ref只有`id`和`size`，没有owner/surface/generation/child handle，renderer只填充矩形；它不参与child arrange、paint、input route、focus、a11y或unload。Icon现有强类型asset、显式geometry/alt、typed dependency和image batch，但尚未绑定asset generation/readiness/intrinsic metric，font-backed icon也没有font/face lease。Image已有alt/tooltip与resource id，仍缺region/tint/relative unit/load/error outcome。dependency closure现表达ImageTexture与IconAsset，Widget child和decorator-owned dependency仍缺。

### 5.8 Layout DTO反向重建语义

resolved line/run DTO复制text和glyph advances，没有stable span id、style handle、inline identity或semantic node。renderer再用source ranges回查compiled artifact以恢复run kind/link/inline object。某些路径还会为slice重新measure以重建item advances。artifact handle equality又不可靠，使“layout属于哪一代source”缺少强合同。

2026-08-26 current-source补充：renderer的非inline rich paint run仍会逐run调用canonical shaping service；当前只新增`text.render/shape_renderer_fallback`、`rich_render_fallback_shape_request_count`和`rich_render_fallback_shape_source_bytes`基线观测，不提前实施cache或prepared-run arena。结构目标仍是本报告RRT-P1-034/M4定义的generation-owned prepared run，由compiled rich与glyph sidecar共同拥有，renderer只投影、不重新shape；在managed Windows的cold/first-paint/stable-repaint、cache、allocation、GPU与power数据完成前，不宣称热点、收益或Unreal经验值接近。

### 5.9 Rich document被排除在增量、editing与a11y之外

viewport fast path明确只接受Plain、horizontal、nowrap、clip、non-editable；rich、wrapped、vertical和editable全部全量layout。rich resolved layout把`editable`设为`None`，Runtime82的document/selection/IME/clipboard authority没有接入。accessibility name/extract读取template scalar `text/label/value`，不消费compiled rich artifact；markup可能被原样朗读，link/image/list/table也没有稳定semantic child、alt/action或structure projection。

### 5.10 语法名与产品能力高于真实实现

Markdown只识别bold、italic和backtick marker，没有escape/nesting/link/block语义；HTML是自定义whitelist和recovery，不是HTML parser。格式名已切为versioned subset；bidi control 也已有typed per-compile trust、balanced stack、exact-range diagnostic与cache identity。仍开放的是完整语法能力协商、malicious corpus和产品authoring UX，不能把最小subset宣传为完整Markdown/HTML。

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
| RRT-P1-006 | 2026-08-25 canonical ASCII直接复用run；Unicode用单调cursor并仅在新输出run clone metadata，250k ASCII/50k combining profile记录P50约99.9%/82.1%下降；2026-08-31已修复拆分后的静态契约与typed-link benchmark fixture | managed release benchmark确认无回归；只有新profile证明仍受span materialization支配时才引入arena |
| RRT-P1-007 | ActiveTag累计clone且depth无上限 | bounded stack、delta style与明确TooDeep outcome |
| RRT-P1-008 | decorator registry按token线性遍历 | compiled dispatch table、namespace和deterministic priority |
| RRT-P1-009 | 2026-08-30 已有 catch boundary、typed panic failure、per-call metadata 与 retained-run quota；非协作 provider 仍无 deadline/cancel | provider work unit、catch boundary、quota和typed failure |
| RRT-P1-010 | decorator/parser无owner、unregister、lease与thread contract | project/session/plugin-qualified provider registry与revoke fence |
| RRT-P1-011 | 2026-08-30 parser/compiled owner 已拒绝超 `u32` byte/index；UI projection 根索引裸 cast 与子索引静默 drop 已改为 fallible checked construction | admission前拒绝超限，compiled/UI projection 全链不做 saturating、truncating 或 silent-drop identity |
| RRT-P1-012 | 任意finite positive font/image size和bidi control可进入布局 | trusted-content policy、geometry clamp与spoof diagnostics |

### 6.2 Artifact、cache与lifecycle

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RRT-P1-013 | process-global parser/cache跨project与session共享 | `RuntimeRichTextService`由runtime context持有并显式shutdown |
| RRT-P1-014 | 2026-08-30 `OnceLock` single-flight 仍无 deadline/cancel；现已可观测 in-flight gauge 与完成 waiter count/total/max nanos，算法尚未盲改 | cancellable parse job与所有waiter一致的terminal receipt |
| RRT-P1-015 | cache residency低估allocator/hash/Arc/index开销 | measured resident bytes、tenant quota和admission/eviction reason |
| RRT-P1-016 | 旧decorator/emoji generation只靠偶然eviction | generation retirement、targeted invalidation和last-use lease |
| RRT-P1-017 | parser identity/generation atomic wrap无exhaustion | non-reusing qualified generation或显式Exhausted状态 |
| RRT-P1-018 | 2026-08-26 已删除仅测试使用的伪`CompiledRichText::from_projection()`；production `UiParsedText` range/index view保留parent artifact | managed tests确认nested table/cell projection继续只引用parent identity |
| RRT-P1-019 | 每cell projection反复扫描整个artifact | 一次构建interval/index，cell view按range常数或对数查询 |
| RRT-P1-020 | 2026-08-30 raw `resource_ids()` 已硬切为 typed closure；当前含 `ImageTexture(ResourceId)` 与 `IconAsset(RichIconAssetId)`，compiled residency/collector 按 kind 消费；generation/font/widget/decorator lease 尚未合格 | 后续 dependency 只有取得 qualified generation/lease 后才能扩展；不得塞 family string/裸 id/generation |
| RRT-P1-021 | 2026-08-26 已硬切为payload类型 + runtime-owner semantic identity；同一artifact为O(1)快速比较 | managed test确认layout dirty detection；保持identity覆盖source/format/generation及glyph rebuild语义 |
| RRT-P1-022 | 2026-08-30 已删除 UI 外部累计差分 sampler；cache mutex 内原子 take/reset 六项事件，保留 residency gauge，并投影 parser/decorator/emoji generation 与 saturation receipt；project/surface 显式关联仍缺 | project/parser/provider维度的bounded telemetry与reset snapshot |

### 6.3 Style、layout、inline object与interaction

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RRT-P1-023 | 2026-08-30 italic与features已静态贯通font query/backend request/shaped key；letter spacing仍未实现 | rich style完整映射到font request、shaping key和glyph artifact；tracking统一cluster-gap语义 |
| RRT-P1-024 | 2026-08-30 已硬切为 `MarkdownInlineV1` + `markdown_inline_v1`，cache/artifact identity 直接含 typed format；managed validation 待办 | versioned minimal subset；不得恢复无版本 `Markdown` alias |
| RRT-P1-025 | 2026-08-30 `HtmlSubsetV1` 已有十二个 tag/attribute/value/style/malformed/entity code、source range、256-entry budget与truncation receipt；当前诊断类静态完成，managed/profile/product evidence 尚缺 | deterministic recovery、完整 source-ranged authoring error 与 managed evidence |
| RRT-P1-026 | 2026-08-26 固定尺寸分支已由当前 owner 的真实 direct child 完成 arrange/ordinary paint/input/a11y，renderer placeholder 已删除；desired-size/run-local invalidation仍缺 | real child widget lifecycle；固定尺寸 MVP 已闭环，desired-size 留待实测后实施 |
| RRT-P1-027 | 2026-08-30 artifact 已硬切 `RichInlineWidgetSlotId` owner-local slot，text projection 不再构造 `UiNodeId`；Surface 在当前 `UiTree` 独占布局期解析 direct child且不保留跨帧 lease，destroy/rebind自然重解；retained session/incarnation lease仍缺 | 当前帧 qualified child binding；未来 retained binding 必须带 surface session + node incarnation + revoke合同 |
| RRT-P1-028 | 2026-08-30 family-only icon 已硬切为 `RichIconAssetId` + size/baseline/alt；horizontal/VerticalRl layout与paint共享geometry，renderer直接发image batch且不再shape，`IconAsset`进入dependency/texture collector；共享image prepare已按`ResourceManagementGeneration`失效解析并使用real/fallback GPU texture；40/40静态通过 | authored-size icon不复制render generation；intrinsic metric须绑定qualified texture revision并驱动layout invalidation；font-backed icon必须显式font asset/face lease并走canonical shaping；managed/profile/WGPU待办 |
| RRT-P1-029 | 2026-08-30 image alt/tooltip 已进入有预算的compiled semantic fallback；units/region/tint/load/error/resource outcome仍缺 | typed image item与resource outcome、semantic fallback |
| RRT-P1-030 | 2026-08-30 target已硬切typed owner，HTML/BBCode tooltip以`Arc<str>`进入quota/residency/hit；action/state/trust仍缺 | typed link action、principal、navigation policy、qualified tooltip/state和semantic action |
| RRT-P1-031 | rich/wrapped/vertical没有viewport/incremental layout | paragraph/span dirty index、visible range和retained layout document |
| RRT-P1-032 | table每cell preferred+final两次完整layout | cached intrinsic metrics、dirty cell/track和bounded relayout |
| RRT-P1-033 | 2026-08-30 session geometry budget、typed bounded/unbounded constraint、checked table tracks/frames/boxes/aggregate 与 `GeometryTooLarge` receipt 已静态实现；fake maximum、byte-derived frame、non-finite-to-zero 已删除，managed compile/render/profile 待办 | session-owned geometry budget、typed unbounded constraint、checked extent和GeometryTooLarge outcome |
| RRT-P1-034 | 2026-08-30 composite artifact 已让正常 rich route 直接消费 glyph slice；paint projection fixed scope/12项work-byte counter已静态接线，managed baseline待办 | profile证明后建立stable runtime prepared block/run owner；serializable DTO只在明确跨边界时物化 |
| RRT-P1-035 | rich layout永远`editable: None` | 与Runtime82 revision/selection/IME/clipboard共享document authority |
| RRT-P1-036 | renderer glyph已走directory，inline/style仍按checked source range查询compiled run；projection allocation/RSS未采样 | prepared draw item携带stable span/object/semantic id和generation |
| RRT-P1-037 | 2026-08-30 BBCode list item 已保留kind、checked ordinal、marker enum、一基level与exact range；完整block tree、HTML list及table header/caption仍缺 | typed block tree同时驱动layout、paint、copy和a11y |
| RRT-P1-038 | 2026-08-30 parser已有request-local table/cell/token/depth/size预算；session现报告两阶段实际layout work，managed profile/阈值决策待办 | parser admission与layout work receipt分离；profile后再决定execution policy |

### 6.4 Accessibility、security、Editor与qualification

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RRT-P1-039 | 2026-08-30 own name/relation 已切到 generation-bound visible text；隐藏 target 无 command 时复用 Surface session/cache，已有视觉 range 仍 fail closed | `RichSemanticProjection`与视觉布局共享source generation且不依赖paint visibility |
| RRT-P1-040 | image与icon fallback text已compiled；image/icon/link/widget/list/table仍无qualified semantic child/action identity，widget alt仍缺 | typed semantic tree、qualified identity、relations、actions与fallback text |
| RRT-P1-041 | 2026-08-30 四格式已有 raw/entity/tag 的 bounded source-ranged diagnostic；typed trust进入cache/compiled identity，默认仅允许mark/balanced isolate，trusted legacy controls仍须平衡 | managed copy/a11y/paint logical identity、malicious corpus与产品/profile资格 |
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

## 13. 2026-08-26 dynamic inline widget current-source 重审

2026-08-26 首次重审时，`[widget=id|widthxheight]` 的
`InlineObjectRef::Widget { id: u64, size }` 只形成 external layout metric，graphics rich renderer 仍把它画成
实心矩形。该历史缺陷没有真实 child、surface/owner/generation、生命周期、input/focus/a11y 或 run-local
invalidation，违反本报告 G27/G28 和“禁止实心矩形称作 inline widget”的约束。

本地 Unreal `FSlateWidgetRun` 保存 `TSharedRef<SWidget>`，以显式 Size 或 desired size 度量，通过
`ArrangeChildren` 放置真实 child，并调用 child `Paint`；desired size 变化只 dirty 对应 run layout。Zircon 的
当前 MVP 先落地显式-size 对应分支：markup `id` 必须绑定富文本 owner 的直接 `UiTree` child，树负责强生命期、
事件、焦点、a11y 和正常 render extract，text artifact 只发布 canonical absolute frame。重复 ID、跨 parent、
missing child 和 overflow-omitted run fail closed；renderer 不得建立 registry、重解析 markup、复制 draw list 或
继续画 placeholder。

实现复杂度门为 `O(tree nodes + rich runs + direct children)`，run/frame directory 必须单调构建并按 child ID
有界查找；未经 profile 不增加常驻全局索引。测试先固定 exact binding、invalid/duplicate/omitted binding、真实
child frame/hit/render 和 no-placeholder。

固定尺寸 direct-child 分支已完成静态实现。compiled source range 经 canonical resolved line/run 单调映射到
absolute frame；full layout 以全部 roots 为边界，incremental layout 只扫描本次 arrangement roots 的受影响子树。
绑定 child 继续走普通 subtree arrange/render extract/hit-test；duplicate、missing、cross-parent 与 omitted binding
清空几何，renderer widget placeholder 已删除。静态规模为
`O(affected tree nodes + rich runs + graphemes + direct children)`；没有 global registry 或 per-child run scan。
源码回归已覆盖 exact child frame/render/hit、duplicate、missing、omitted 与 no-placeholder，rustfmt、定向
whitespace 和 `git diff --check` 通过。状态为
`dynamic_inline_widget_architecture_review_complete / fixed_size_direct_child_inline_widget_implemented /
renderer_widget_placeholder_removed / incremental_arrangement_root_bounded / static_checks_complete /
managed_validation_pending`。

2026-08-30 identity follow-up 将 compiled `id: u64` 硬切为 `RichInlineWidgetSlotId`：它只表示当前富文本
owner 的 authoring-local slot，不是跨树 `UiNodeId`，UI text projection 也不再依赖 event/tree identity。
Surface layout 在同一个 `&mut UiTree` 作用域内才执行 `slot -> UiNodeId`，随后以 current direct-child set
fail closed；该 binding 不跨帧驻留，所以 child destroy、同值重建、换树与换 surface 都必须在下一次 layout
重新解析，compiled cache 不能持有旧 node generation。若 desired-size/run-local invalidation 后续需要 retained
binding，必须同时携 `UiSurfaceSessionIdentity` 与 `UiTree::node_incarnation` 并显式 revoke。typed slot 合同进入
完整 Runtime Text infrastructure 静态批次，47/47 在 1.744 s 通过。当前状态追加为
`typed_owner_local_widget_slot_implemented / current_tree_binding_nonretained / static_checks_complete /
managed_validation_pending`。desired-size、retained session/incarnation lease、完整 G27/G28、managed Cargo、
profile/power/WGPU/PNG 仍开放，因此不改变 M5、G46-G48 的开放状态。

## 14. 2026-08-30 M1 representation-budget implementation slice

M1 的最小 representation admission 已实现，但完整 G02-G04 仍未关闭。新增覆盖
source/output、token count/bytes、per-token attribute count/bytes、active-tag depth 的
`RichParseBudget` 和 typed `RichTextParseError`；source 在
global cache lookup/copy 前拒绝，visible output 在 builder append 与 emoji expansion materialization
前拒绝。默认 32 MiB 与现有 retained text-document 量级一致，effective limit 同时受 `u32` 可表示范围
约束；8 MiB compiled cache 继续只是 residency policy，不能授权解析超限。

`CompiledRichText` 对 visible length、run/paragraph/table count 和 cell projection
index 全部 checked build，旧 `u32::MAX` 饱和 identity 已删除。single-flight cell 保存 terminal
`Result`，失败对当前 waiter 一致并在完成后移出 residency。UI 将详细错误投影为稳定低基数
`ZR-TEXT-LAYOUT-012` 并走 failure layout。有效 token 默认总量 65,536、单 token 64 KiB、单 token
64 attributes/16 KiB attribute bytes，HTML/BBCode tokenizer 在 tag/attribute 字符串 materialization 前
返回 typed failure。HTML/BBCode 共享的 ActiveTag 栈以默认 128 层请求预算
约束，并在第 `max + 1` 层、实际 `Vec` 增长前返回 `ActiveTagDepthBudgetExceeded`；5,000 层 release
索引基准必须显式扩大预算，不再把 hostile default success 当兼容合同。parser builder 状态拆入
162 行 child owner，grapheme normalization 拆入 100 行 `run_alignment.rs`，根 parser 为 715 行，
符合结构预算。

本切片关闭 RRT-P1-011 的 production saturation 路径，并部分推进 RRT-P1-001/002/007；ActiveTag
bounded stack 已完成，但 delta-style clone/allocation 优化必须等待 release profiler，不能由静态推断。
general node/span、time work receipt、source-map diagnostic、decorator quota/cancel/panic boundary
仍开放，所以 M1、G02-G04、G46-G48 均不得标 complete。本次两个 E 盘 Cargo 检查分别在 90/120 秒
无输出、无结论后停止，均不是 clean gate。静态合同当前为 38/38；WGPU/PNG、产品
profile/RSS/power/Unreal matched evidence 未执行。

## 17. 2026-08-30 representation count admission

M1 now also bounds materialized run, paragraph, table, table-cell, and retained projection-index
counts. The run/paragraph/table builder owners reject before their vectors grow; BBCode table state
admits a cell before closing/pushing it; compiled interval queries share a total projection-index
cap. Defaults are 131,072 / 16,384 / 4,096 / 65,536 / 262,144. BBCode block/table nesting is also
bounded at 32/8 by default and returns typed failure before owner growth; silent suppression and
`u16::MAX` depth aliasing are removed. Six typed over-limit regressions cover the public parser path,
and Runtime Text static tests pass 38/38. General node/span depth, decorator isolation, time/cancel
receipts, managed Cargo, WGPU/PNG and the prescribed product profile/RSS/power matrix remain open.
Status: `representation_count_admission_static_implemented / projection_index_cap_static_implemented /
block_table_depth_admission_static_implemented / managed_product_validation_pending`。

## 15. 2026-08-30 compiled grapheme owner structural cutover

在继续增加 node/span/table budget 前，重新沿 production consumer 追踪
`CompiledRichText::cluster_ranges`：除构建、identity/容量核算与一个测试断言外没有调用方，真实 cluster
消费已经由 shaping/layout artifact 持有。Unreal `IRichTextMarkupParser::Process`、
`FTextRunParseResults` 与 default rich markup parser 同样只把 stripped output、line/run range 和 metadata
交给 marshaller/layout，不在 compiled markup artifact 常驻全篇 grapheme vector。

按优化纪律先完成 E 盘 release 隔离测量。ASCII 1/8/32 MiB、每档 31 样本的 vector payload 为
8/64/256 MiB，p50 为 65,236/736,093/3,074,179 us；32 MiB 首次 working-set delta 为
269,508,608 bytes。该 `O(G)` owner 没有生产价值，因此不以更小 quota 掩盖架构问题，而是硬切字段、
全篇 `grapheme_indices(true)` pass、equality/byte accounting 和 accessor，不保留 compatibility/lazy
副本。post-cutover 该 owner 的 payload 精确为 0 且阶段不存在；这不是用空循环伪造的 timing 改善。

回归门禁禁止 `cluster_ranges` 字段/accessor、compiled 层 segmentation import 与全篇 materialization
回归；Runtime Text 静态集合通过 34/34，rustfmt/source/diff guard 通过。Cargo 仍在 Runtime Text 前被
unrelated interface session export 阻断，故 G46-G48、端到端 parser/layout/frame、RSS/power、WGPU/PNG
与 matched Unreal load 均保持开放。状态：
`compiled_grapheme_owner_review_complete / duplicate_cluster_index_hard_cut_static /
isolated_baseline_profile_recorded / managed_product_validation_pending`。

## 16. 2026-08-30 table projection quadratic-rescan correction

The next structural review found `CompiledRichText` rescanning all runs, paragraphs, and tables for
every cell. An E-drive release isolation profile (31 samples, one matching range per object) measured
50,331,648 interval comparisons and p50/p95/p99 60,544/85,779/123,556 us at 4,096 objects, while
only 8,192 indices were emitted. This is an algorithmic `O(C * (R + P + T))` bottleneck, not merely a
vector-allocation issue.

The owner now builds request-local balanced `RichRangeIntervalIndex` trees with subtree `max_end`,
queries candidates per cell, applies table depth/containment, and drops the trees after compiled
construction. Canonical source order is admitted by a linear check; only defensive out-of-order
constructor input is sorted. UI projection no longer sorts/deduplicates the checked output.

The same E-drive 31-sample final-path lane at 4,096 objects entered 215,046 interval nodes and measured
p50/p95/p99 3,337/4,467/5,611 us, improving old p50/p95 by 18.14x/19.20x. From 256 to 4,096 objects,
p50 growth reduced from 260.97x to 22.70x. First-sample working-set delta increased from 208,896 to
360,448 bytes, so allocation/RSS/power acceptance remains open. Existing nested-table semantics plus
out-of-order/boundary-touching interval regression pass; complete static Runtime Text tests are 38/38
and rustfmt/source/diff guards pass. Managed Cargo, real table layout, WGPU/PNG and Unreal matched-load
remain open. Status: `table_projection_interval_owner_static_implemented / quadratic_rescan_removed /
isolated_post_profile_complete / managed_product_validation_pending`。

## 18. 2026-08-30 exact-tag decorator dispatch correction

RRT-P1-008 current-source review found a semantic mismatch: every Zircon decorator uniquely owns one
normalized exact tag, but `DecoratorRegistry::apply` scanned the complete provider vector for each
candidate token. With 4,096 final-tag dispatches per sample, the 31-sample E-drive release baseline
measured p50 517/7,381/116,314 us for 16/256/4,096 decorators. The 256x provider-count increase caused
224.98x p50 growth with no output allocation in the timed loop.

Unreal `FRichTextLayoutMarshaller::TryGetDecorator` scans because each `ITextDecorator::Supports` is an
arbitrary predicate and ordering is part of that contract. Zircon has no predicate-based `Supports`, so
the aligned architectural decision is to preserve parser/widget-owned decorator lifetime and explicit
marshaller dispatch while indexing Zircon's stronger exact-tag identity. A single parser-local
`HashMap<String, Box<dyn RichTextDecorator>>` now owns registration and lookup. Registration performs
duplicate admission and insertion through one `Entry`; dispatch borrows the token tag and invokes the
callback after immutable resolution. Decorator generation and compiled cache keys are unchanged.

The same indexed lanes measured p50 140/142/139 us. At 4,096 decorators p50/p95 improve
836.79x/1,040.07x, and dispatch no longer scales with unrelated provider count. Static Runtime Text
tests pass 38/38; one owner/zero linear dispatch and Rust 2024 format guards pass. This advances
RRT-P1-008 and the algorithmic portion of G09.

A follow-up RRT-P1-009 infrastructure slice catches callback unwind as
`DecoratorPanicked { tag }`, caps accepted per-call dynamic metadata at 64 KiB by default, and caps
cumulative retained run metadata at 32 MiB. The builder charges only non-merged materialized runs
before publication; UI maps panic to `LayoutFailed` instead of the budget diagnostic. Typed Rust
regressions are written but unrun. A no-op dynamic-callback boundary profile including
`catch_unwind` measured p50 146/149/154 us at 16/256/4,096 decorators versus the old
541/7,869/112,965 us; the largest lane improves 733.54x and no provider-count slope returns.
Deadline/cancel, callback-private temporary allocator quota,
RRT-P1-010 provider lease/revoke, registration-count admission, retained registry allocation/RSS,
Cargo, WGPU/PNG, package power and matched Unreal load remain open. Status:
`exact_tag_decorator_hash_dispatch_static_implemented /
isolated_linear_dispatch_bottleneck_removed_profiled /
decorator_panic_and_metadata_admission_static_implemented /
managed_product_validation_pending`。

## 19. 2026-08-30 immutable compiled artifact owner cutover

RRT-P1-005 consumer tracing confirmed that production UI/runtime callers already retain
`Arc<CompiledRichText>` and that public `RichTextParser::parse()` had no production consumer. It still
deep-cloned every run, paragraph, table, and dynamic metadata field after canonical compile/cache
lookup, producing a detached payload without the complete source/parser-generation identity of its
parent. Local Unreal rich-text flow passes parser output straight into marshaller-created layout runs;
it does not add a second partial owned artifact after the canonical parse owner.

The required pre-change E-drive release profile used 31 samples. At 4,096/32,768/131,072 runs the
clone performed 12,355/98,819/395,267 allocations, requested
1,014,784/8,118,272/32,473,088 bytes, and measured p50 2,454/22,059/111,366 us. The largest lane's
p95/p99 were 232,754/331,802 us and its first-sample working-set delta was 40,169,472 bytes.

Production `parse()` and its bridge are hard-cut. `compile() -> Arc<CompiledRichText>` is the sole
public materialization entry; downstream parsed views borrow from that retained parent. The owned
helper remains only under `cfg(test)` for corpus assertions, so it cannot enter a production binary.
No alias, facade, second cache, or lazy detached snapshot remains. The removed production stage now
has exact post allocation/bytes zero because the stage does not exist. The current reproducible static
suite passes 34/34;
rustfmt and source/diff guards pass. This closes production RRT-P1-005 but does not close M1 or
G46-G48: managed Cargo, external downstream migration, real WGPU/PNG, product latency/allocation/RSS/
power, and matched Unreal-load validation remain open. Status:
`RRT-P1-005_production_owner_cutover_static_complete /
isolated_clone_baseline_recorded / managed_product_validation_pending`。

## 20. 2026-08-30 non-reusing parser generation boundary

RRT-P1-017 current-source review confirmed that parser identity used `fetch_add().max(1)` and
decorator/emoji generations used `wrapping_add`, eventually reusing an old compiled-cache identity.
Both registration paths changed their registry before advancing generation, which is the wrong commit
order for any explicit exhaustion policy.

Local Unreal uses widget-owned decorator instances and strong references retained by one
`FRichTextLayoutMarshaller`; `SetDecorators` replaces that owner's array. It does not identify
unrelated provider owners by a wrapping process-global integer. Until RRT-P1-013 replaces Zircon's
global cache with RuntimeRichTextService ownership, its numeric key must therefore be strictly
non-reusing.

Parser identity now has an explicit optional nonzero state and an atomic
`fetch_update + checked_add` allocator. Compile fails typed before source/cache work when exhausted.
Decorator and emoji generation use checked next-value admission before registry mutation; exhaustion
leaves owner state unchanged. This is a correctness repair rather than an optimization, so no profile
or power claim is made. The current reproducible static suite passes 35/35, rustfmt passes, and source
guards report zero `fetch_add`/`wrapping_add` in the owner. Rust boundary tests are written but unrun.
This statically closes RRT-P1-017; RRT-P1-010/013/016, managed Cargo, WGPU/PNG, product
RSS/power, and matched Unreal-load evidence remain open. Status:
`RRT-P1-017_non_reusing_identity_static_complete / managed_product_validation_pending`。

## 21. 2026-08-30 Surface-session rich parser/cache owner cutover

RRT-P1-013 consumer mapping found all production compilation below the existing Surface-owned
`SharedTextLayoutSession`. The old static built-in parser and independent `shared_cache()` therefore
created a second, process-wide lifecycle that mixed unrelated Surface residency, counters, LRU
pressure, and shutdown. Unreal's `URichTextBlock`/`FRichTextLayoutMarshaller` keeps parser/decorator
state with its retained widget/marshaller owner; the aligned Zircon MVP boundary is the retained
Surface session, not a new application singleton.

Production free compile/lookup/shared-report APIs are removed. `RichTextParser` owns one bounded
`CompiledRichTextCacheOwner`; `SharedTextLayoutSession` owns the parser and every layout, measure,
prewarm, retained-document, render-preparation, and profiling call uses that explicit owner. Only
cfg-gated corpus helpers retain a static default parser. The static suite passes 36/36, and a Rust
test records same-owner reuse plus cross-owner clear isolation, but managed Cargo has not run.

This statically closes the process-global ownership portion of RRT-P1-013, not the full rich service
milestone. RRT-P1-010/014/016, multi-Surface quota and cancellation/retirement, WGPU/PNG, allocation/
RSS/contention/power, and matched Unreal-load validation remain open. Status:
`RRT-P1-013_process_global_owner_cut_static_complete / managed_product_validation_pending`。

## 22. 2026-08-30 current registration generation retirement

RRT-P1-016 follow-up found that checked decorator/emoji generation publication changed cache identity
but left every old compiled entry resident until unrelated LRU pressure removed it. The parser now
clears only its owned compiled cache after a successful registry mutation and generation commit.
Failed duplicate/invalid/exhausted registration leaves registry, generation, and healthy residency
unchanged. Existing consumer `Arc<CompiledRichText>` values remain valid last-use artifacts; only the
cache owner's old-generation residency is retired.

The focused Rust test covers successful decorator and emoji retirement, failed-registration
preservation, and old-artifact readability, but managed Cargo has not run. The reproducible static
suite remains 36/36 and targeted Rustfmt passes. This is the safe current mutable-registration slice,
not the full lifecycle contract: RRT-P1-010 project/session/plugin-qualified provider snapshots,
unregister/revoke fences and registration-count admission, RRT-P1-014 cancellable single flight, and
RRT-P1-016 concurrent snapshot publication/targeted retirement remain open. Status:
`RRT-P1-016_current_registration_retirement_static / provider_snapshot_revoke_open /
managed_product_validation_pending`。

## 23. 2026-08-30 rich style shaping projection and tracking review

RRT-P1-023 current-source review confirmed that `StyleOverride` retained italic and OpenType
features while the resolved `TextStyle`, font query, backend request, Cosmic attributes, and shaped
cache silently used normal/default shaping. `TextStyle` now owns immutable feature data and italic;
rich style resolution, horizontal/vertical request construction, font selection, public neutral
service projection, Cosmic fallback, and shaped-cache identity all consume that same style.

The cross-layer contracts failed before implementation and the complete reproducible Runtime Text
static suite now passes 47/47. Canonical OpenType feature identity retains one value per tag with
last-declaration precedence and stable tag order, so conflicting inputs cannot diverge between cache
identity and backend execution. Focused Rust tests cover rich override projection, canonical feature
inheritance/cache separation/conflicts, italic font query, public request projection, and Cosmic
italic attrs, but managed Cargo has not run. Status:
`RRT-P1-023_italic_and_feature_projection_static_complete / managed_validation_pending`.

Letter spacing remains deliberately unimplemented after reference review. Unreal scales tracking by
font size/1000, disables `liga`, adds only inter-glyph gaps to the previous advance, and bypasses RTL/
unsupported cases. Cosmic 0.18.2 instead adds its convenience spacing value to every glyph including
the last, while Zircon's direct RustyBuzz path has no equivalent. The accepted direction is one
backend-neutral cluster-gap owner before measurement/artifact publication, with tracking in cache
identity and explicit RTL/vertical/negative-spacing policy. An E-drive 31-sample matrix is required
before implementation and optimization claims. Detailed record:
[`../../zircon_runtime/text/07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md`](../../zircon_runtime/text/07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md).

## 24. 2026-08-30 rich-table geometry budget review

RRT-P1-033 current-source review separated the already repaired general rich intrinsic path from the
remaining table-only defect. The non-acceptance implementation now removes the horizontal
`f32::MAX / 4` square and final/VerticalRl source-byte-derived frame. Existing parse/shaping budgets
remain separate because their units cannot safely supply a logical-pixel limit.

Local Unreal Slate confirms the target boundary: no-wrap is a text-layout mode, measured block widths
form desired size, and final view/allotted geometry is applied later. The required Zircon owner is a
runtime/session geometry budget plus typed bounded/unbounded constraints, checked table prefix sums,
and a `GeometryTooLarge` outcome. The implemented `2^24` default is only the `f32` exact-integer
safety ceiling; the documented E-drive viewport/DPI/font/table corpus must still select any lower
product policy.
`GeometryTooLarge` now owns `ZR-TEXT-LAYOUT-013` and `text.layout.geometry_too_large`; the targeted
static contracts now pass 31/31, and production table paths return it with owner/source/work context
when admission fails. Focused Rust tests are written but unrun. Status:
`RRT-P1-033_geometry_budget_and_table_cutover_static_complete /
managed_compile_render_and_profile_pending`。详见
[`../../zircon_runtime/text/07/2026-08-30-rich-table-geometry-budget-review.md`](../../zircon_runtime/text/07/2026-08-30-rich-table-geometry-budget-review.md)。

## 25. 2026-08-30 rich UI projection index admission

RRT-P1-011 review confirmed that `RichParseBudget` caps indexed bytes at `u32::MAX` and
`CompiledRichText::new` checks visible bytes plus run/paragraph/table counts before constructing its
artifact. The remaining UI adapter still built root indices with `as u32` and used `filter_map` for
child run/paragraph indices, which could turn an invariant failure into a partial semantic view.

`UiParsedText::from_compiled` and `from_projection` are now fallible. Root and child indices use one
checked conversion, invalid run/paragraph/table indices return `TextLayoutError::LayoutFailed`, and
the artifact rebuild path records the failure without publishing a partial replacement. The
failing-first projection contract and complete Runtime Text static suite pass 47/47; focused Rust
behavior coverage is written but managed Cargo has not run. Status:
`RRT-P1-011_compiled_admission_and_ui_projection_static_complete / managed_validation_pending`。

## 26. 2026-08-30 rich format version identity hard cut

RRT-P1-024/RRT-P1-025 current-source review confirmed that the public `Markdown` variant only
implemented strong/emphasis/inline-code markers and that `Html` selected a deliberately bounded V1
whitelist/recovery parser. Local Unreal exposes an injectable `FDefaultRichTextMarkupParser` rather
than claiming that its angle-bracket syntax is HTML; syntax capability and layout capability remain
separate owners.

Runtime and interface contracts are now hard-cut to `MarkdownInlineV1`, `BbCodeV1`, and
`HtmlSubsetV1`, with exact `markdown_inline_v1`, `bbcode_v1`, and `html_subset_v1` wire/style values.
Old variants and style aliases are absent. `RichTextFormat` is hashable and the compiled cache key
owns it directly, removing the parallel hand-maintained `u8` identity. Parser dispatch, UI conversion,
fixture input, and framebuffer proof commands use the versioned identity. Failing-first static
coverage and the complete Runtime Text suite pass 47/47; focused wire round-trip and legacy-rejection
Rust tests are written but managed Cargo has not run.

The structural follow-up adds four stable warning codes for unsupported, unmatched, implicitly
closed, and EOF-unclosed tags. `RichParseResult` retains typed code/severity, source-markup range,
recovery and a truncation receipt; the independent default cap is 256 and compiled-cache byte
accounting includes diagnostic capacity. The complete static suite passes 47/47; behavior tests are
written but unrun.

The attribute/style follow-up accumulates compact flags during the existing tokenizer and value-
projection passes, adding unsupported/malformed attribute, invalid value, and unsupported style
property codes without rescanning attributes. The final authoring-diagnostic follow-up adds malformed
tag, unterminated quoted attribute, malformed entity, and unrecognized entity classifications during
the existing tokenizer/entity-decoder path. Malformed source is preserved literally, ordinary less-
than text does not produce a warning, and the EOF path publishes earlier entity diagnostics before a
later malformed tag. Diagnostic construction and active-tag ownership moved to semantic 108/123-line
children; the HTML parser is a 259-line format owner and the shared parser root is 558 lines. The
47/47 static suite, Rustfmt, source-size gate, and
scoped diff-check pass.

Status: `RRT-P1-024_versioned_format_identity_static_complete /
RRT-P1-025_authoring_diagnostic_static_complete /
managed_profile_and_product_validation_pending`. Managed Rust behavior, bounded-corpus performance,
and product evidence remain open before RRT-P1-025 can close. Detailed record:
[`../../zircon_runtime/text/07/2026-08-30-rich-format-version-identity-review.md`](../../zircon_runtime/text/07/2026-08-30-rich-format-version-identity-review.md).

## 27. 2026-08-30 rich-table layout work receipt

RRT-P1-038 current-source review corrected the old parser premise: `RichParseBudget` already admits
request-local token, nesting, table, cell, run, paragraph, projection, source-byte, and visible-output
counts. The remaining structural gap was the absence of frame-owned evidence for the preferred and
final cell layout passes, so selecting a new execution limit or intrinsic cache would have been an
unmeasured behavior change.

Local Unreal `FTextLayout` retains line models/views and exposes a layout lifecycle around regeneration.
Following that boundary, `SharedTextLayoutSession` now owns a saturating, content-free
`TextTableLayoutWorkReport`. The table path records actual table/source/cell topology, preferred/final
cell calls and input bytes, resolved tracks, and only geometry-admitted published lines/boxes. Twelve
fixed-name counters publish at frame end; no dynamic source/table label is emitted. The instrumentation
does not change layout order, failure policy, admission, or cache behavior.

The failing-first contract and complete Runtime Text static suite pass 52/52; focused owner/reset and
saturation Rust tests are written but managed Cargo has not run. Status:
`RRT-P1-038_table_layout_work_receipt_static_complete /
managed_profile_and_budget_decision_pending`. The E-drive 31-sample matrix, allocation/RSS/power,
threshold and retained-cache decisions, real WGPU rendering, and PNG evidence remain open. Detailed
record: [`../../zircon_runtime/text/07/2026-08-30-rich-table-layout-work-receipt-review.md`](../../zircon_runtime/text/07/2026-08-30-rich-table-layout-work-receipt-review.md).

## 28. 2026-08-30 rich prepared-run current-source correction

RRT-P1-034's earlier statement that every non-inline rich paint run reshapes is no longer true. The
current `ResolvedRichTextArtifact` owns compiled metadata, generation-bound glyphs, exact layout lines,
and a run-to-glyph directory. Valid rich artifact routes consume glyph slices directly; fallback is
restricted to intentional visual-only or source-isomorphic missing/stale/incomplete routes.

The remaining RRT-P1-034/036 work is duplicated serializable layout/paint string residency and checked
compiled-style range projection. Unreal retains shared line text, run style/block identity, and shaped
subsequences under one layout owner. Zircon must profile `UiRenderCommand::text_paint` allocation/time
and stable repaint residency before changing its serde/remote DTO or adding another cache. Status:
The runtime renderer now brackets real transient text-paint materialization with one fixed scope and
publishes twelve content-free command/run/text/style-byte counters. Segment-cache rebuilds contribute
actual work; an exact cache hit publishes zero rather than replaying cached counts. The complete static
suite passes 52/52. Payload bytes remain a lower bound, so allocator/RSS evidence is still required.
Status: `RRT-P1-034_paint_projection_profile_infrastructure_static_complete /
RRT-P1-036_managed_baseline_and_owner_decision_pending`. Detailed record:
[`../../zircon_runtime/text/07/2026-08-30-rich-prepared-run-current-source-review.md`](../../zircon_runtime/text/07/2026-08-30-rich-prepared-run-current-source-review.md).

## 29. 2026-08-30 rich accessibility semantic projection

RRT-P1-039 was reproduced in both node-name fallback and referenced label/description text: the
accessibility extractor cloned template `text`/`label`/`value` scalars without consulting the current
compiled artifact, so versioned markup could be exposed as spoken text. The visual path already owns
the correct generation through `UiRenderCommand -> UiResolvedTextLayout -> CompiledRichText`.

`RichSemanticProjection` now retains that compiled `Arc` and exposes its visible text only after exact
source-markup and versioned-format validation. Accessibility obtains candidates from the Surface
render cache's per-node command range, rejects missing/stale/ambiguous generations, and never reparses
markup or concatenates clipped layout lines. Plain text and explicit a11y/alt/tooltip priority remain
unchanged. Lookup is `O(log nodes + node commands + source validation + visible materialization)`;
generation disambiguation is `O(1)` and no second cache was added.

HTML own-name, BBCode relation, stale-source, source/format, and generation contracts are written. A
hidden relation target with no render command now resolves through the same Surface
`SharedTextLayoutSession` and compiled cache; existing visual ranges remain authoritative. This avoids
an accessibility parser, second cache, and eager hidden-tree parse. The complete Runtime Text static
suite passes 54/54.

RRT-P1-040 still owns typed link/inline/list/table semantic children. Review found that the current
accessibility DTO/action route accepts only real `UiNodeId`, while rich runs have no qualified dispatch
identity; synthetic byte-offset ids are therefore rejected until a compiled-run or real UI-child owner
is designed. Managed Rust, AccessKit/screen-reader, product a11y, allocation/RSS/power, WGPU, and PNG
evidence remain open. Status:
`RRT-P1-039_visibility_independent_surface_semantic_owner_static_complete /
RRT-P1-040_typed_children_and_managed_validation_pending`. Detailed records:
[`../../zircon_runtime/text/07/2026-08-30-rich-accessibility-semantic-projection-review.md`](../../zircon_runtime/text/07/2026-08-30-rich-accessibility-semantic-projection-review.md) and
[`../../zircon_runtime/text/07/2026-08-30-rich-visibility-independent-semantic-owner-review.md`](../../zircon_runtime/text/07/2026-08-30-rich-visibility-independent-semantic-owner-review.md).

## 30. 2026-08-30 rich list semantic metadata hard cut

RRT-P1-037 current-source review found a structural loss rather than a marker formatting bug. The BBCode
block parser already owned list kind, ordered marker algorithm, ordinal, and list-stack depth, but emitted
only marker text plus `ParagraphOverride.list_prefix`. Any later copy or accessibility implementation
would therefore have had to infer authoring semantics from rendered text.

The compiled model now owns `RichListItemKind`, `RichOrderedListMarker`, and `RichListItem`. Ordered kind
contains a checked canonical ordinal and marker enum; every item contains one-based semantic level and its
exact compiled-visible marker range. UI layout derives only `UiTextRange` for hanging-indent geometry.
Physical-paragraph overlap resolution uses a private layout projection, so it cannot overwrite the
semantic model or create a second list authority. The parser remains single-pass O(n) with O(1) metadata
per admitted list paragraph and no additional text clone.

The complete Runtime Text static suite passes 55/55 in 0.239 s; focused Rust behavior tests are written
but managed Cargo has not run. No timing/allocation/RSS/power gain is claimed, and no cache or execution
threshold was introduced. Status:
`RRT-P1-037_typed_list_item_metadata_static_complete /
RRT-P1-040_qualified_publication_and_managed_validation_pending`. Complete typed block tree, HTML list,
table header/caption semantics, qualified a11y children/actions, managed WGPU/PNG, and profile evidence
remain open. Detailed record:
[`../../zircon_runtime/text/07/2026-08-30-rich-list-semantic-metadata-hard-cut.md`](../../zircon_runtime/text/07/2026-08-30-rich-list-semantic-metadata-hard-cut.md).

## 31. 2026-08-30 rich inline image semantic fallback owner

RRT-P1-029 current-source review found that HTML attribute admission discarded `alt/title`, BBCode image
had no attribute form, and `InlineObjectRef::Image` retained only texture/size/baseline. The later
generation-bound accessibility correction therefore read U+FFFC from compiled visible text. Implementing
replacement inside accessibility would create a second per-snapshot run walker outside cache budgets.

Image runs now retain `alternative_text` and `tooltip`; HTML and BBCode attribute forms enter the same
compiled artifact. These strings count toward existing run metadata quota and residency. A separate
`max_semantic_text_bytes` gate builds one semantic `Arc<str>` from the ordered inline index before cache
publication; no-inline artifacts share the visible Arc. Explicit empty alt is decorative, tooltip is used
only when alt is absent, merged adjacent images repeat fallback per placeholder, and malformed ranges fail
closed. `RichSemanticProjection` performs an O(1) retained owner read.

The complete Runtime Text static suite passes 56/56 in 0.141 s; focused Rust behavior/admission/residency
tests are written but managed Cargo has not run. No timing/allocation/RSS/power gain is claimed. Status:
`RRT-P1-029_inline_image_semantic_fallback_static_complete /
RRT-P1-040_qualified_inline_children_and_managed_validation_pending`. Resource units/region/tint/outcome,
icon/widget alternatives, qualified a11y child/action identity, managed WGPU/PNG, and profile evidence
remain open. Detailed record:
[`../../zircon_runtime/text/07/2026-08-30-rich-inline-image-semantic-fallback-owner-review.md`](../../zircon_runtime/text/07/2026-08-30-rich-inline-image-semantic-fallback-owner-review.md).

## 32. 2026-08-30 rich link target owner hard cut

RRT-P1-030 current-source review found a security-owner split, not a local validation-cost problem. Parser
admission built and allowlisted a `ResourceLocator`, discarded it into `String`, and input application
maintained another path/scheme algorithm. The prior allocation-free validator benchmark therefore measured
an implementation that should not exist.

RuntimeInterface now owns `UiRichLinkTarget`: a private shared canonical locator admitted by constructors
and serde. `LinkRef`, hit testing, dispatch effect, transaction, and host request carry it without reparsing;
the application boundary validates only the real node owner. The canonical locator is shared through
`Arc`, while serde retains the existing `href` scalar wire. Construction is `O(B)` once; downstream clones
are `O(1)` and application performs no path scan.

The complete Runtime Text static suite passes 57/57 in 0.215 s; Rust behavior tests are written but managed
Cargo has not run. This structural cut has no timing/allocation/RSS/power claim. Status:
`RRT-P1-030_typed_link_target_foundation_static_complete /
RRT-P1-040_qualified_link_child_and_managed_validation_pending`. Action/principal/navigation semantics,
qualified accessibility identity, managed host/WGPU/PNG, and matched E-drive profiling remain open.
Detailed record:
[`../../zircon_runtime/text/07/2026-08-30-rich-link-target-owner-hard-cut.md`](../../zircon_runtime/text/07/2026-08-30-rich-link-target-owner-hard-cut.md).

## 33. 2026-08-30 rich typed dependency closure foundation

The image-only `resource_ids()` API was unsafe to extend because its sole production consumer treated every
id as a texture. `CompiledRichText` now retains sorted/deduplicated `Arc<[RichTextDependency]>`; the first
admitted variant is `ImageTexture(ResourceId)`, which texture discovery explicitly matches. Residency
accounts the enum slice and the ambiguous API is removed. Construction remains `O(R + D log D)` with
`O(D)` temporary memory and borrowed artifact reads.

The complete Runtime Text static suite passes 59/59 in the final 0.363 s rerun; focused Rust behavior tests are written but
managed Cargo has not run. Status:
`RRT-P1-020_typed_image_dependency_foundation_static_complete /
icon_font_widget_decorator_lease_and_managed_validation_pending`. Detailed records:
[`84/2026-08-30-rich-typed-dependency-closure-foundation.md`](84/2026-08-30-rich-typed-dependency-closure-foundation.md) and
[`../../zircon_runtime/text/07/2026-08-30-rich-typed-dependency-closure-foundation.md`](../../zircon_runtime/text/07/2026-08-30-rich-typed-dependency-closure-foundation.md).

## 34. 2026-08-30 rich cache owner-qualified reset telemetry

Current source already gives each `SharedTextLayoutSession` one `RichTextParser` and one private
`CompiledRichTextCacheOwner`; the structural defect was the reporting algorithm. UI retained an external
sampler that subtracted cumulative saturating counters. After any counter reached `u64::MAX`, later
intervals could no longer distinguish no activity from exhausted telemetry, and the report did not carry
the parser/provider generation that produced it.

The cache owner now copies and resets six interval event counters while holding the cache mutex. Residency
entries/bytes and configured limits remain gauges across snapshots. The parser owner stamps parser identity,
decorator generation, and emoji generation onto the same snapshot; checked overflow publishes one
`telemetry_saturated` receipt and the following take starts a fresh interval. Surface profiling currently
publishes 16 fixed low-cardinality names after the RRT-P1-014 measurement extension, and no markup, pointer,
resource id, project string, or dynamic tenant label.

This is a correctness foundation, not a measured speedup. Explicit project/surface correlation still belongs
to the outer profiling session. The current infrastructure static suite passes 35/35 in the final 0.315 s
rerun; rustfmt and scoped diff-check pass, and old sampler symbols scan to zero. Managed Cargo, matched
profile, RSS, power, WGPU/PNG, commit, and WeCom remain open. Status:
`RRT-P1-022_parser_provider_qualified_reset_snapshot_static_complete /
project_surface_correlation_and_managed_profile_pending`. Detailed records:
[`84/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md`](84/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md) and
[`../../zircon_runtime/text/07/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md`](../../zircon_runtime/text/07/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md).

## 35. 2026-08-30 rich single-flight contention instrumentation

Current-source review corrected two stale premises. Decorator execution already has `catch_unwind`, a typed
panic failure, a per-call metadata quota, and an aggregate retained-run quota. The remaining RRT-P1-009 gap
is deadline/cancellation for a non-cooperative provider. RRT-P1-014 remains real: the first
`OnceLock::get_or_init` caller can block every same-key waiter indefinitely.

The local Unreal `FShapedTextCache` uses synchronous instance-local `Find -> Add`, but that relies on Slate's
calling model and is not evidence that a shareable Zircon parser should duplicate concurrent parse work.
Replacing `OnceLock` with one parse per caller would trade blocking for unbounded CPU amplification and still
would not terminate a hung decorator. Adding an arbitrary wait timeout without a bounded worker/cancellation
owner would only abandon callers while leaking execution capacity. Both changes are rejected before data.

The cache owner now exposes a point-in-time `compile_requests_in_flight` gauge plus completed waiter count,
total wait nanoseconds, and maximum wait nanoseconds. An initializer-local `Cell` distinguishes the caller
that actually runs the `OnceLock` closure; an RAII guard decrements the gauge during ordinary return or unwind.
Already-complete artifacts return before timing, so hot hits do not pay an `Instant` read. The four additions
bring the existing fixed cache profile from 12 to 16 names without source text or dynamic labels.

The managed profile decision matrix is fixed before algorithm work: uncontended warm hit, unique miss, and
same-key 2/4/8-caller contention; 1/4/16 KiB admitted markup; built-in-only and custom-provider cases. Evidence
must report wait count/total/max, in-flight gauge, parse count, cache hit/miss, wall time, CPU, allocation/RSS,
and power from E/D/F artifacts. A blocked-provider fault case must separately prove bounded worker capacity and
terminal receipts before any timeout is accepted.

Current infrastructure static contracts pass 36/36 in the final 0.206 s rerun; rustfmt and scoped diff-check
pass. `rich_cache.rs` was split at the domain boundary into 541 production lines and 340 test lines; profile is
739 lines. Managed Cargo/profile/fault/RSS/power remain open. Status:
`RRT-P1-014_contention_measurement_static_complete /
bounded_worker_cancellation_and_managed_profile_pending`. Detailed records:
[`84/2026-08-30-rich-single-flight-contention-instrumentation.md`](84/2026-08-30-rich-single-flight-contention-instrumentation.md) and
[`../../zircon_runtime/text/07/2026-08-30-rich-single-flight-contention-instrumentation.md`](../../zircon_runtime/text/07/2026-08-30-rich-single-flight-contention-instrumentation.md).

## 36. 2026-08-30 rich bidi-control trust and source diagnostic foundation

Current source already resolves UAX#9 in shaping, retains logical source ranges, and applies visual ordering
as a projection. The structural security defect was earlier: raw Unicode controls were accepted by every
format, HTML entities could synthesize them, and BBCode exposed marks, embeddings, overrides, and isolates
as undifferentiated literal tags. Moving sanitization into shaping would have split visual output from copy,
hit testing, and accessibility identity.

The parser classifies bidirectional mark, embedding/pop, override, and isolate controls into stable
authoring codes 013..016. Plain, Markdown-inline, HTML-subset, and BBCode use one source-range owner; HTML
observes decoded entities inside its existing loop and BBCode uses the literal token range. Diagnostics share
the existing bounded vector and truncation receipt. Logical text is preserved: no strip, replacement, or
automatic FSI/PDI insertion is performed.

`RichTextContentTrust` is now a per-compile input and part of both `RichTextArtifactKey` and
`CompiledRichText`. The existing entry point defaults to `Untrusted`: directional marks and balanced isolates
remain valid, while legacy embedding/pop/override controls fail with exact source ranges. Only explicit
`TrustedAuthoring` permits legacy controls, and it still rejects unmatched terminators or unterminated openers.
A dedicated 125-level explicit-stack budget fails before growth. Raw scalars, HTML entities, and BBCode literal
tags therefore share one policy and cannot cross-hit cache entries compiled under another trust level.

The additional source scan is `O(B)` over non-overlapping emitted slices, entity observation adds no second
entity scan, and no source-map/per-frame owner is added. The stack allocates only when explicit controls occur
and is hard bounded. Static contracts pass 38/38 in the final 0.090 s rerun; Rust behavior tests are written but
unrun. Status: `RRT-P1-041_trust_gate_and_balanced_isolation_static_complete /
managed_copy_a11y_render_and_profile_pending`. Managed Cargo, malicious corpus execution, copy/a11y/paint
projection, WGPU/PNG, allocation/RSS/power, commit, and WeCom remain open. Detailed record:
[`../../zircon_runtime/text/07/2026-08-30-rich-bidi-control-authoring-diagnostics.md`](../../zircon_runtime/text/07/2026-08-30-rich-bidi-control-authoring-diagnostics.md).

## 37. 2026-08-31 rich paint-block geometry owner review

The prior `O(lines + runs)` statement is now explicitly scoped to typed glyph-artifact route
publication. Paint geometry is not yet linear end to end: interface paint-run construction repeats
line-wide grapheme validation/count and prefix sums for every run, then inline rendering repeats line,
run, grapheme-prefix, and advance-prefix searches. This is both a worst-case quadratic term and a
duplicate geometry owner.

Unreal Slate remains the primary boundary: `FTextLayout` creates positioned `ILayoutBlock` values once;
`FSlateImageRun` and `FSlateWidgetRun` paint, hit-test, and arrange children from the same block
location/size. Zircon profiling builds now expose seven fixed inline probe/work/frame-agreement counters,
while ordinary builds retain a zero-sized aggregate. A focused profiling regression and a Windows
release-only exact-helper benchmark for 1/100/1k/10k runs, three warm-ups, 31 raw timing/RSS samples,
and p50/p95/p99 are written. A separate renderer harness covers dense LTR/RTL/VerticalRl inline objects
and 1/100/1k hard lines with counter capture outside timed planning. Neither benchmark has run under
managed Cargo. No algorithm cutover or performance result is claimed before the E-drive 31-sample
matrix. Status:
`RRT_paint_block_geometry_current_source_review_complete /
inline_measurement_instrumentation_implemented_static /
interface_and_renderer_release_profile_harnesses_implemented_static /
managed_baseline_and_single_owner_cutover_pending`. Detailed plan:
[`../../zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md`](../../zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md).
