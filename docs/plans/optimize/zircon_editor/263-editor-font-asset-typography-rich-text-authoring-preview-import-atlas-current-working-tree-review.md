---
title: Editor Font Asset、Typography、Rich Text Authoring、Preview、Import 与 Atlas 当前工作树复审
category: zircon_editor
report_id: Editor263
review_date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/97-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/145-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-current-source-review.md
canonical_parent_owners:
  - docs/plans/optimize/zircon_editor/145-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-current-source-review.md
  - docs/plans/optimize/zircon_runtime/201-runtime-text-font-document-shaping-layout-raster-atlas-render-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/196-runtime-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-product-integration-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
related_code:
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/retained_host/host_contract/paint_text
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/typography.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui
  - zircon_plugins/ui_asset_authoring
  - zircon_plugins/ui_document_importer
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/FontEditor/Private/SCompositeFontEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/FontEditor/Private/FontFaceEditor.cpp
  - dev/godot/editor/import/dynamic_font_import_settings.cpp
  - dev/godot/editor/import/resource_importer_dynamic_font.cpp
  - dev/bevy/examples/ui/text/font_atlas_debug.rs
  - dev/bevy/examples/ui/text/font_variations.rs
  - dev/bevy/examples/ui/text/system_fonts.rs
  - dev/Fyrox/editor/src/plugins/inspector/editors/font.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/Texture2DAtlas.cs
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor263 · Font Asset / Typography / Rich Text / Preview / Atlas 当前工作树差距

## 1. 结论

当前 Editor 文本能力不能概括为“临时绘制几行字”。Retained Host 已经能从 Runtime shaped artifact 取得精确 face、generation、collection index 和 glyph placement，按整段 preflight 后投影；无法完整消费时整段回退，避免同一行混用两套 glyph identity。Host 字体集合缓存、Runtime artifact face cache和layout cache已有容量上限，Swash路径可输出alpha、subpixel和color bitmap。UI Asset Editor也有真实document/session、compile、preview `UiSurface`、undo/replay和style/theme inspection基础。这些实现应保留。

但这些基础还没有组成项目字体与富文本作者产品。`ui_asset_authoring`只注册UI Layout、UI Widget、UI Style三类资源；`ui_document_importer`只解析`.zui`并包装为View/Style/Component资产。当前选择集中没有字体资产toolkit、字体导入设置、composite/fallback editor、glyph coverage admission、variation/color face inspector、rich text document editor或从source到cooked font artifact的Editor receipt。

现有Font Atlas工作区仍是固定的`Inter_UI`、`4096 glyphs`、`4 pages`和`12 missing`。Bake/Inspect操作只映射到静态“queued”反馈，字段编辑仅经过通用control route，没有字体domain job、artifact、page texture、UV、residency或missing-glyph snapshot。它是产品外观fixture，不是Runtime201 atlas authority的consumer。

Preview与宿主渲染还存在结构性分裂：`UiAssetPreviewHost::new/new_v2`没有显式`TextRuntimeContext`、font/Unicode/locale generation或device profile；Host同步枚举系统字体并维护进程全局cache；非默认variation instance因`fontdue`不支持而整段退回host layout；glyph raster cache是无容量和字节预算的全局`HashMap`。因此当前preview既不能证明与Runtime/packaged build一致，也不能证明长期Editor会话的内存上界。

本报告不重复Editor145的lossless UI document P0，也不重复Runtime201的文本Runtime P1。本轮没有新增唯一P0，登记36项Editor侧P1、10项P2和28个资格门：

| 等级 | Open/Fail | Partial | Closed/Pass | 合计 |
|---|---:|---:|---:|---:|
| P0（由父owner追踪） | 0 | 0 | 0 | 0 |
| P1 | 30 | 6 | 0 | 36 |
| P2 | 9 | 1 | 0 | 10 |
| Gate | 23 | 5 | 0 | 28 |

目标不是在Editor中重建第二套font database或atlas，而是建立以下作者链：

```text
FontSource + FontAuthoringDocument
  -> FontImport/Validation Job
  -> versioned FontCookReceipt + CoverageManifest
  -> Runtime FontCollectionAuthority
  -> TextPreviewSession (font + Unicode + locale + device profile pinned)
  -> Runtime glyph/layout/atlas artifacts
  -> Editor diagnostics, inspectors and atlas projections

RichTextAuthoringDocument
  -> schema/trust/decorator validation
  -> Runtime201 compile/layout artifact
  -> source-mapped preview and accessibility diagnostics
```

## 2. 扫描范围与冻结指标

### 2.1 当前工作树选择集

本轮读取当前磁盘内容，覆盖UI Asset authoring/import、Retained Host text/typography、Font Atlas产品surface和最小Runtime typography合同边界；Tooling按用户要求排除。fingerprint按规范化lowercase相对路径排序，以`path + NUL + raw bytes + NUL`计算SHA-256：

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Editor UI asset authoring/import | 158 | 36,611 | 33,789 | 1,288,676 | 214 | 58 | `dc98e08c10a884c626ff2fb7549344326003e3e127ee10d34359bea9a1499b8a` |
| Retained Host text/typography | 41 | 7,415 | 6,695 | 244,253 | 154 | 6 | `c98f197c20905c3c9eb92662a0d802158decb20f26d5dc58e9038a946e66348b` |
| Font Atlas产品surface | 5 | 2,348 | 2,290 | 109,296 | 1 | 0 | `61ad5b0e60190af81c1044416199534db30ff687a93d692b5bf129393a73815b` |
| Typography合同边界 | 12 | 3,970 | 3,690 | 141,212 | 8 | 0 | `c76d51324648cb5edb0b902d7961f64c1a0a991c078bb539dfd9dc0ef8f41ace` |
| 去重总选择集 | **216** | **50,344** | **46,464** | **1,783,437** | **377** | **64** | `3379c5e161a7111f97405ca681d3480caf8ae1e500e4bda28c30748138a3760b` |

冻结时该闭包有149条git status记录，说明大量文件正处于并行修改状态。本文只对上述fingerprint对应内容负责，实施前必须重扫受影响owner。

### 2.2 验证限制

本轮是review-only，没有运行Cargo、Editor、真实字体导入、保存/重开、UI automation、系统字体枚举、locale/RTL/vertical、IME、GPU atlas、device loss、fault、scale、long-soak、视觉golden或benchmark。377个test marker和64个ignored marker只表示源码中的测试意图，不能据此声明动态正确性、性能或表现达到或超过Unreal。

### 2.3 Owner边界

| 主题 | Editor263负责 | 父owner |
|---|---|---|
| 字体资产作者产品 | toolkit、import settings、coverage/许可/variation/fallback、preview与receipt | Runtime201/80负责Runtime font authority与cooked artifact语义 |
| UI typography authoring | typed inspector、resource picker、cascade/source解释、device/locale preview | Runtime201负责resolved style、shape、cache、raster与atlas合同 |
| 富文本作者体验 | source document、schema/decorator/inline object、source-mapped diagnostics与preview | Runtime201/84负责parser、trust、layout和render |
| Font Atlas工作区 | 消费真实Runtime snapshot并提供truthful command/diagnostic | Runtime201/79负责atlas allocation、upload、residency和render |
| UI Asset文档安全 | 只记录与文本字段相关的当前断点 | Editor145继续拥有lossless CST、save/merge/transaction P0 |

## 3. 当前真实基础与断点

### 3.1 应保留的基础

1. `UiAssetEditorSession`不是静态页面：已有document、compile、preview surface、selection、palette、style/theme、undo/replay和runtime report状态。
2. `UiAssetAuthoringEditorPlugin`通过真实asset contribution注册UI Layout/Widget/Style toolkit和creation template。
3. `.zui` importer通过`UiZuiAssetLoader`解析V2文档并输出typed imported asset，不再维持多种历史suffix。
4. `EditorTypographyTokens`已有UI/strong/code family、body/strong/code weight、caption/body/title/overlay size、line-height、smoothing和utility text role。
5. Retained Host可把Runtime artifact的source identity、font generation、face/instance和collection index带入face cache。
6. artifact projection采用整段preflight；face转换、rotation或variation不支持时不会把半套Runtime glyph与半套Host glyph混画。
7. Host字体集缓存容量2、artifact face cache容量64、layout cache容量2,048，且layout key包含host font key和Runtime font generation。
8. Swash raster支持grayscale/subpixel/color内容，fallback到fontdue时也保留显式source与format。

### 3.2 当前分裂的数据流

```text
UI Asset authoring
  text field -> TOML Value::String
  arbitrary typography prop -> generic literal/TOML field
  compile -> UiSurface compatibility constructor
  preview summary -> command count + fixed pixel size

Editor Host paint
  EditorDesignTokens -> 3 logical host faces
  -> synchronous system font discovery
  -> Runtime shaping artifact when completely convertible
  -> otherwise host layout/raster fallback
  -> process-global layout and glyph caches

Font Atlas workspace
  fixed ZUI rows -> string route -> fixed queued/success feedback
  [no FontAsset/FontCookReceipt/AtlasSnapshot consumer]
```

## 4. P1：工程级阻断与重构要求

### 4.1 Font asset、import、cook与catalog

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| ED-TXT-P1-001 | Open | `ui_asset_authoring`只贡献UiLayout/UiWidget/UiStyle；没有Font/FontFace/CompositeFont asset toolkit。 | 建立FontSource、FontFace和CompositeFont的唯一asset type/toolkit，接入Asset Browser、open/create/save/reimport和usage index。 |
| ED-TXT-P1-002 | Open | 当前Editor插件集合没有字体专用importer/settings surface；字体文件只能依赖Runtime通用ingest。 | 建立异步FontImportJob，设置face index、hinting/raster policy、SDF/MSDF、bitmap/color、mipmap、target/profile和determinism。 |
| ED-TXT-P1-003 | Open | 未见face/subface、family/style/weight/stretch、table/format能力与损坏字体的Editor admission report。 | 导入时生成versioned face inventory、supported tables、axis/color/palette/bitmap能力、sanitizer结果和source span diagnostic。 |
| ED-TXT-P1-004 | Open | 没有default/fallback/sub-family、Unicode range、locale/script优先级的composite authoring。 | 建立有stable entry ID的ordered fallback graph，支持range/language/script条件、cycle/overlap/gap验证和单事务编辑。 |
| ED-TXT-P1-005 | Open | 没有由项目文本、localization、manual range和运行采样共同生成的coverage/subset manifest。 | 建立可追溯CoverageManifest，显示required/present/missing glyph、source corpus、normalization/provider版本和target footprint。 |
| ED-TXT-P1-006 | Open | 未见license、embedding/subsetting permission、attribution和shipping target policy的Editor gate。 | Font cook必须验证license metadata与项目策略；不合规时fail-close并提供可定位修复动作。 |
| ED-TXT-P1-007 | Open | variation axis、named instance、optical sizing、color palette在Editor authoring中没有typed inspector。 | 以axis tag/range/default/name建typed control；预览和cook receipt固定有效instance/palette identity。 |
| ED-TXT-P1-008 | Open | UI importer只parse/wrap `.zui`，没有解析字体依赖闭包或生成source-to-font-to-glyph artifact receipt。 | UI/rich import编译必须输出font dependency、fallback closure、coverage result、artifact IDs和generation chain，禁止只凭字符串family通过。 |

### 4.2 Typography与rich text authoring

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| ED-TXT-P1-009 | Partial | `EditorTypographyTokens`已有family/weight/size/line-height/smoothing，Host能投影UI/strong/code三类face。 | 保留token基础，但将项目字体引用改为typed asset handle，并记录default/inherited/effective value与owner generation。 |
| ED-TXT-P1-010 | Open | Runtime201已确认`UiResolvedStyle`缺italic/slant、stretch、spacing、features、variation和family stack；Editor也无对应authoring。 | 先补单一`TextTypographyDescriptor`合同，再生成Editor controls，禁止Editor先造私有字段。 |
| ED-TXT-P1-011 | Open | Inspector只专门编辑`control_id`和`text`；其余prop以扁平path和TOML literal呈现。 | 为typography建立schema-driven group、typed numeric/unit/enum/tag/resource controls、validation和reset/inherit命令。 |
| ED-TXT-P1-012 | Open | 未见字体资源picker、face/instance picker、fallback preview、OpenType feature picker或palette picker。 | picker必须消费FontCatalogSnapshot，显示availability、coverage、license、target与stale generation，提交stable ID而非display string。 |
| ED-TXT-P1-013 | Partial | Style/theme inspector与cascade inspection真实存在，但typography仍是generic declaration，Runtime contract也不完整。 | 为每个effective typography值显示source rule/token、specificity/layer、fallback reason、why-won和Runtime accepted/rejected receipt。 |
| ED-TXT-P1-014 | Open | `asset_editor`选择集中没有RichText/TextDocument专用authoring document或toolkit。 | 建立富文本source/structured双视图、selection/source map、undo group、large-document增量模型和lossless roundtrip。 |
| ED-TXT-P1-015 | Open | 没有markup schema、decorator、link/image/inline widget/table/list的typed palette与property editor。 | 从Runtime parser extension snapshot生成palette；每个extension绑定capability/generation/schema/trust和reload状态。 |
| ED-TXT-P1-016 | Open | 未见parse/token/run/depth/table/security diagnostic的source-range Editor projection。 | 编译结果按document revision返回typed diagnostics、source span、preview node/run和修复动作；旧revision结果不得覆盖新文档。 |
| ED-TXT-P1-017 | Open | Preview没有long text、pseudo locale、RTL/vertical、complex script、emoji/color和missing glyph corpus矩阵。 | 建立可保存TextPreviewScenario，组合locale/direction/writing mode/device/font scale/corpus并输出可比较artifact。 |
| ED-TXT-P1-018 | Open | 未见glyph cluster、advance/bearing/baseline、line break、caret/hit/source map、fallback face的调试overlay。 | 增加只读Runtime artifact inspector；不得通过Host重新推算并冒充Runtime truth。 |

### 4.3 Preview context、Host字体与缓存

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| ED-TXT-P1-019 | Partial | Preview会真实compile UI document、构造`UiSurface`并计算layout，不是截图占位。 | 保留真实surface，但让preview session持有显式Runtime/Text context、document revision和artifact receipt。 |
| ED-TXT-P1-020 | Open | `UiAssetPreviewHost::new/new_v2`只接size/document/compiled，没有TextRuntimeContext、font/Unicode/locale snapshot。 | 定义`TextPreviewSession`，在构造时pin project runtime profile、font collection、Unicode、locale、parser extensions和budgets。 |
| ED-TXT-P1-021 | Open | preset固定1280x720、1100x780、1920x1080、640x480；没有DPI、font scale、safe area、platform或locale。 | 使用项目DeviceProfile authority，并支持多profile矩阵、user font scale、orientation、safe zone和platform font policy。 |
| ED-TXT-P1-022 | Open | preview generation没有font import/cook、collection、Unicode、locale、parser/catalog、surface/frame统一lineage。 | 形成source revision到presented frame的generation chain；任一依赖变化只发布新generation，旧job必须cancel/stale。 |
| ED-TXT-P1-023 | Open | Runtime artifact包含非默认variation时，Host因fontdue不能应用坐标而返回`None`并整段fallback。 | Preview必须显示typed fidelity degradation；实现能消费同一variation instance的raster path，或明确该profile不可预览。 |
| ED-TXT-P1-024 | Partial | 整段artifact preflight避免部分glyph混画，是正确的fail-closed基础；但fallback只体现在profile counter。 | 将fallback原因、受影响run/face/instance、generation与恢复动作投影到preview，不得静默改变glyph/advance。 |
| ED-TXT-P1-025 | Open | `HostTextFontSet::resolve`同步`Database::load_system_fonts()`，选择结果依赖平台与本机。 | 系统字体发现进入异步project/profile policy，结果稳定排序并缓存为snapshot；packaged-only预览禁止本机字体污染。 |
| ED-TXT-P1-026 | Open | glyph raster cache是进程全局`OnceLock<Mutex<HashMap<...>>>`，没有entry/byte上限、eviction、trim或shutdown。 | 接Runtime196 residency profile和memory pressure；限制entries/bytes/single-item，提供LRU/clock eviction、clear和health snapshot。 |
| ED-TXT-P1-027 | Partial | layout cache限制2,048项且key含font generation，但只按entry计数，进程全局，命中不更新顺序。 | 改为preview/context-owned cache，限制bytes，记录hit/miss/evict/peak；以真实recency和frame pin控制退役。 |
| ED-TXT-P1-028 | Partial | Host font set容量2、artifact face cache容量64且后者按generation/instance key缓存。 | 两者仍是进程全局authority；加入context owner、bytes、lease、trim/revoke、shutdown与generation exhaust策略。 |

### 4.4 Font Atlas产品面、diagnostics与qualification

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| ED-TXT-P1-029 | Open | Font Atlas ZUI硬编码Inter UI、Latin/Cyrillic/CJK/Icons、glyph/page/missing计数与三项option。 | 页面只能由FontCatalog/AtlasHealth snapshot生成；无provider时显示Unavailable，不保留看似真实的固定数据。 |
| ED-TXT-P1-030 | Open | Bake/Inspect route只返回固定`queued`字符串，没有job id、admission、progress、cancel或terminal receipt。 | 命令提交到FontCook/Atlas inspection job authority；UI只在accepted receipt后显示queued，并消费progress/terminal outcome。 |
| ED-TXT-P1-031 | Open | Font/Range/Size edit与commit仅属于通用field action列表，没有FontAuthoringDocument transaction。 | 所有编辑进入typed command、validation、undo/redo、dirty/save和cross-asset dependency transaction。 |
| ED-TXT-P1-032 | Open | 页面没有真实page texture、UV、glyph id/codepoint/cluster、bearing/advance、face/instance、format或palette inspect。 | 建立Runtime snapshot投影和GPU readback/debug view；明确CPU artifact、GPU allocation、validity和present generation。 |
| ED-TXT-P1-033 | Open | 未见Font Atlas workspace消费Runtime201 atlas/page/residency/missing snapshot的生产路径。 | 定义versioned只读`TextAtlasHealthSnapshot`与cursor，Editor不得访问或复制Runtime内部cache。 |
| ED-TXT-P1-034 | Open | 页面没有upload pending、eviction/churn、fragmentation、memory bytes、device loss、recovery或backpressure。 | health面板必须显示预算、occupancy、queue、high-water、evict reason、device epoch和recovery receipt。 |
| ED-TXT-P1-035 | Open | missing glyph固定为12，无法定位源文本、asset、locale、fallback decision或package target。 | 缺字diagnostic关联asset/document/source range/style/family/locale/target和fallback trace，并提供加入coverage或替换字体动作。 |
| ED-TXT-P1-036 | Open | 未见font import/preview/atlas的roundtrip、fault、platform、visual corpus、scale、soak和同内容benchmark资格。 | 建立font corpus与自动qualification，记录correctness、p50/p95/p99、allocation、cache/atlas churn、fallback率和long-session RSS。 |

## 5. P2：精度、可维护性与性能债务

| ID | 状态 | 当前证据 | 重构要求 |
|---|---|---|---|
| ED-TXT-P2-001 | Open | Asset Editor多个文件为700至956行，混合state、mutation、projection、cache和tests。 | 按document/command/service/projection/diagnostic拆owner，禁止新字体功能继续塞入generic inspector。 |
| ED-TXT-P2-002 | Open | Font Atlas control/action/route与字体名字以裸字符串分散在ZUI、binding、navigation和feedback。 | 由descriptor生成typed IDs，启动时验证重复、悬空route、缺失resource和provider capability。 |
| ED-TXT-P2-003 | Open | Host font cache key用`DefaultHasher`对request、runtime family和完整font bytes生成`u64`。 | 区分安全source identity与进程内hash；持久/artifact lineage使用内容摘要，cache比较保留完整identity防碰撞。 |
| ED-TXT-P2-004 | Open | 字体装载会复制source bytes并同步对整份bytes hashing，然后fontdue/Swash各建解析视图。 | cook阶段生成digest和共享blob；Editor以lease共享bytes，量化parse/hash/copy latency和peak memory。 |
| ED-TXT-P2-005 | Open | layout/raster/font cache没有统一entries/bytes/capacity/hit/miss/evict/peak snapshot。 | 使用同一TextResidencySnapshot和采样epoch，产品面不得拼接互不一致的counter。 |
| ED-TXT-P2-006 | Open | 高频style/font family仍是`String`，preview和Host重复trim/clone/query。 | authoring边界保留字符串，compile后使用interned typed handles并固定catalog generation。 |
| ED-TXT-P2-007 | Partial | 选择集中已有大量单元与source/perf tests，但64个ignored marker未形成受管证据。 | 将性能证据放入固定环境runner，记录硬件/build/corpus/raw samples；ignored测试不能作为关闭finding的依据。 |
| ED-TXT-P2-008 | Open | 固定“opened/queued/selected”文本可能在无domain provider时呈现成功。 | success文案只能由accepted/terminal domain receipt生成；no provider、rejected、stale、cancelled必须分别呈现。 |
| ED-TXT-P2-009 | Open | 文本、字体路径、locale、用户输入和rich payload可能进入diagnostic/profile，未见统一privacy/content policy。 | 定义redaction、hashing、opt-in corpus和export policy，默认不采集正文、secret、用户path或未发布字体。 |
| ED-TXT-P2-010 | Open | 选择集、route/static fixture和关键断言未进入自动staleness gate。 | 为报告owner建立source fingerprint、forbidden fixture、contract字段和provider consumer检查，变更后自动要求复审。 |

## 6. 参考引擎对比与采用路由

| 参考 | 可验证的工程形态 | Zircon差异 | 应采用的原则 |
|---|---|---|---|
| Unreal FontEditor | 独立Font/FontFace editor；Composite Font可编辑default/fallback/sub-typeface/character range；修改使用transaction并flush font cache刷新preview。 | Zircon没有Font asset toolkit/composite authoring，Font Atlas是固定fixture。 | 建立独立asset/editor与事务化fallback graph；preview由authoritative cache invalidation驱动。 |
| Godot dynamic font import | import settings直接预览AA、mipmap、embedded bitmap、MSDF、variation；按Unicode range/glyph选择preload configuration并检查supported glyph/contour。 | Zircon没有字体import settings、coverage/variation/MSDF preview。 | 导入设置、真实能力查询、glyph/range选择和目标artifact必须在同一receipt链。 |
| Bevy text examples/pipeline | system font、generic family、weight、variation使用typed资源；font atlas debug读取实际atlas texture/set，而不是固定数字。 | Zircon Runtime底层能力更广，但Editor没有typed consumer。 | 让Editor消费Runtime资源snapshot；示例可以简单，产品诊断不能伪造数据。 |
| Fyrox font inspector/formatted text | Inspector通过FontResource编辑器选择资源，FormattedText保留wrap/alignment/brush/size和资源语义。 | Zircon Inspector主要把字体/排版当literal字符串。 | 先建立typed resource property editor，再扩展高级排版，不以通用TOML输入代替领域工具。 |
| Unity Graphics Texture2DAtlas | allocator、texture identity、cached/update/release/clear与GPU validity是明确Runtime状态。 | Zircon Runtime atlas已有更强局部基础，但Editor页面没有allocation/validity/residency投影。 | Editor只读消费CPU/GPU validity与generation snapshot；Bake authoring和Runtime residency必须分权。 |

这些参考不要求复制Unreal UObject、Godot RID或Bevy ECS API。Zircon应保留Rust ownership、immutable snapshot和typed failure，补齐的是Editor document、job、artifact、preview context和truthful projection。

## 7. 目标架构与实施顺序

### Phase A：Truth hard cut

1. 将Font Atlas固定计数和静态成功反馈改为Unavailable/provider snapshot。
2. 建立Font/FontFace/CompositeFont的asset type、toolkit、operation和provider capability。
3. 定义Editor/Runtime共享的FontAssetId、FontFaceId、FontInstanceId、FontArtifactId、FontGeneration和TextPreviewSessionId。

### Phase B：Font import与composite authoring

1. 实现FontImportJob、ImportSettingsDocument、face inventory、sanitizer、license和target capability。
2. 实现transactional composite fallback graph、range/language/script规则、coverage分析和cycle/overlap/gap diagnostics。
3. 生成immutable FontCookReceipt、CoverageManifest和package footprint，不让Editor读取Runtime可变database内部结构。

### Phase C：Typography与rich text document

1. 依赖Runtime201补齐`TextTypographyDescriptor`，再生成Editor schema controls和resource pickers。
2. 建立RichTextAuthoringDocument、lossless source/structured projection、extension palette、trust和source-mapped diagnostics。
3. 接入localization corpus、pseudo/RTL/vertical/complex-script/emoji/color矩阵。

### Phase D：Preview fidelity

1. `TextPreviewSession`显式pinRuntimeProfile、font/Unicode/locale/parser generation、device/DPI/font scale和budget。
2. Preview只消费Runtime layout/glyph artifact；fallback必须给出typed degradation receipt。
3. 补variation/color/vertical raster fidelity或明确fail-close，不允许无提示改变排版。

### Phase E：Residency与Atlas health

1. 将Host font/layout/raster cache收敛到context owner，补entry/byte/single-item admission、memory pressure和shutdown。
2. Runtime发布versioned AtlasHealthSnapshot：page/allocation/UV/format/residency/upload/device epoch/missing/backpressure。
3. Font Atlas workspace消费snapshot与job receipt，支持inspect、filter、source定位、cancel和恢复。

### Phase F：Qualification

1. 运行import/save/reopen/reimport/package、多project/multi-preview隔离与plugin reload测试。
2. 建立Latin/Arabic/Indic/SEA/CJK/Emoji/color/variation/vertical/bidi/combining/invalid字体corpus。
3. 建立DPI/backend/device-loss视觉golden、fault/scale/long-soak和same-content benchmark；数据不足前不得宣称优于Unreal。

## 8. 工程资格门

| Gate | 当前 | 通过条件 |
|---|---|---|
| EDTXT-G01 | Partial | UI Asset document/session/preview真实存在，进一步由Font/RichText toolkit复用同一transaction authority |
| EDTXT-G02 | Fail | Font/FontFace/CompositeFont asset type、factory、toolkit、save/reopen闭环 |
| EDTXT-G03 | Fail | Font importer有typed settings、async job、cancel、terminal receipt和target profile |
| EDTXT-G04 | Fail | face/table/axis/color/bitmap/sanitizer能力由真实source生成并可定位diagnostic |
| EDTXT-G05 | Fail | license/embedding/subset/attribution是cook和package硬门 |
| EDTXT-G06 | Fail | composite fallback/range/language/script authoring可undo且验证cycle/overlap/gap |
| EDTXT-G07 | Partial | family/weight/size/line-height/smoothing token基础存在，完整typography descriptor端到端闭合 |
| EDTXT-G08 | Fail | Inspector使用typed font/resource/axis/feature/palette controls而非literal TOML |
| EDTXT-G09 | Fail | RichText document、schema/extension palette、lossless roundtrip和source map闭合 |
| EDTXT-G10 | Fail | rich trust/security/budget diagnostics关联revision和source range |
| EDTXT-G11 | Fail | localization/pseudo/RTL/vertical/complex-script/emoji/color preview矩阵可保存可重放 |
| EDTXT-G12 | Partial | Preview构造真实UiSurface，且显式pinTextRuntimeContext与全部provider generations |
| EDTXT-G13 | Fail | device profile覆盖DPI/font scale/platform/system-font policy/safe area/orientation |
| EDTXT-G14 | Fail | source/import/cook/font/Unicode/layout/atlas/frame拥有统一lineage和stale fence |
| EDTXT-G15 | Fail | non-default variation/color/vertical artifact可保真预览或typed fail-close |
| EDTXT-G16 | Partial | artifact采用整段preflight，所有fallback进一步发布可定位degradation receipt |
| EDTXT-G17 | Fail | system font discovery异步、policy化、稳定排序，packaged preview可重放 |
| EDTXT-G18 | Partial | face/layout cache有局部容量，全部cache还需entries+bytes+pressure+shutdown |
| EDTXT-G19 | Fail | glyph raster cache有硬上限、eviction、trim和long-session RSS证据 |
| EDTXT-G20 | Fail | Font Atlas没有静态业务数据，只投影versioned Runtime snapshot |
| EDTXT-G21 | Fail | Bake/Inspect命令有job admission/progress/cancel/terminal receipt |
| EDTXT-G22 | Fail | page texture/UV/glyph metrics/face-instance/format/palette inspect来自真实artifact |
| EDTXT-G23 | Fail | atlas occupancy/bytes/churn/upload/device-loss/backpressure/recovery可观察 |
| EDTXT-G24 | Fail | missing glyph可定位asset/source/locale/target/fallback并提供修复动作 |
| EDTXT-G25 | Fail | import/save/reopen/reimport/package和multi-preview隔离集成测试通过 |
| EDTXT-G26 | Fail | Unicode/locale/DPI/backend视觉golden与differential corpus通过 |
| EDTXT-G27 | Fail | fault/scale/long-soak、cache/atlas residency和memory pressure资格通过 |
| EDTXT-G28 | Fail | same-content benchmark报告p50/p95/p99、allocation、fallback率、atlas churn且可与参考比较 |

## 9. 完成判据与禁止事项

Editor263不能以“能选择一个font family字符串”“能显示富文本”“有一个Font Atlas页面”“Bake按钮显示queued”或“Host能画Runtime glyph”作为完成。完成要求是：字体source从导入、验证、composite/coverage authoring、cook、Runtime collection admission到preview/atlas投影始终携带stable identity、generation、target、budget、diagnostic和terminal receipt；RichText从source range到Runtime artifact与preview diagnostics可双向定位；Editor与packaged Runtime在同一context/corpus上产生可比较结果。

禁止为关闭本报告而在Editor中复制Runtime font database、shaper、glyph cache或atlas。禁止继续增加固定glyph/page/missing数字。禁止在没有domain receipt时显示queued/saved/baked/validated。禁止把本机系统字体解析结果当作项目可打包字体真值。
