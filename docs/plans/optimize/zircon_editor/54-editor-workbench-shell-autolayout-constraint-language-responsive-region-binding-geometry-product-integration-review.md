---
related_code:
  - zircon_editor/assets/ui/editor/layout/shell_regions.toml
  - zircon_editor/src/ui/workbench/autolayout
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/template_bridges.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window/mounted_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window/refresh_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window/resolution_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/layout_frames.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/responsive_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/style.rs
tests:
  - zircon_editor/src/ui/workbench/autolayout/css_like_constraint/tests.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/vertical_bands/tests.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/breakpoints.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/geometry.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/region_contracts.rs
  - zircon_editor/tests/integration_contracts/workbench_autolayout.rs
  - zircon_editor/src/ui/retained_host/ui/tests/workbench_layout_frames.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/52-editor-builtin-view-window-descriptor-catalog-content-provider-capability-template-localization-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
  - docs/ui-and-layout/workbench-skeleton-contract.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SSplitter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SSplitter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingSplitter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingSplitter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SResponsiveGridPanel.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SResponsiveGridPanel.cpp
  - dev/godot/scene/gui/split_container.h
  - dev/godot/scene/gui/split_container.cpp
  - dev/godot/scene/gui/box_container.h
  - dev/godot/scene/gui/box_container.cpp
  - dev/godot/editor/docks/editor_dock_manager.h
  - dev/godot/editor/docks/editor_dock_manager.cpp
  - dev/Fyrox/fyrox-ui/src/dock/config.rs
  - dev/Fyrox/fyrox-ui/src/dock/mod.rs
  - dev/Fyrox/fyrox-ui/src/dock/tile.rs
  - dev/Fyrox/fyrox-ui/src/grid.rs
  - dev/bevy/crates/bevy_ui/src/layout/mod.rs
  - dev/bevy/crates/bevy_ui/src/ui_node.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/UXML/RenderGraphViewer.uxml
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph/RenderGraphViewer.SidePanel.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 54 · Editor Workbench Shell AutoLayout / Constraint Language / Responsive Tier / Region Binding / Geometry Authority 产品集成工程化差距

## 1. 结论

Zircon Editor已经有一组值得保留的壳层布局基础：`WorkbenchSkeleton`、typed region、轴向约束、breakpoint tier、逻辑像素解析、Taffy template bridge、drawer resize和89项聚焦测试都是真实代码；产品渲染、pointer、drag和viewport也确实会消费componentized template layout frames。因此问题不是“没有布局算法”，而是声明、求解和发布没有收敛成一个产品authority。

最严重的第一处断路是，仓库虽然提交了`assets/ui/editor/layout/shell_regions.toml`和446行CSS-like declaration parser，production却没有任何caller加载`WorkbenchShellRegionsAsset`或解析声明；`WorkbenchSkeleton::from_shell_regions_asset`只在测试出现，产品每次仍从`jetbrains_default()`与ZUI template构造，资产中的`panel_asset`也没有consumer。当前“可配置壳层”和“CSS-like constraint”只能算测试DTO，不是Editor能力。

第二处是同一次shell recompute同时维护两套几何权威：custom `WorkbenchShellGeometry`求解Left/Document/Right/Bottom，componentized bridge再用另一套drawer rule和Taffy求解真实frame。渲染、hit-test、drag与viewport优先使用template frames，最小窗口、floating window和template frame复用却依赖legacy geometry；窄屏时两边对Bottom Drawer的折叠规则本身就不同。更危险的是template recompute失败只记日志，随后仍把旧template frames与新model/legacy geometry一起发布，没有generation、原子commit或rollback。

本报告登记 **3项P0、52项P1、12项P2与36个资格门**。Editor54唯一拥有Workbench壳层声明编译、region binding、responsive policy、单一geometry authority及同代发布；Editor01/Runtime11A继续拥有通用retained UI增量布局，Editor13拥有workspace/docking持久化，Editor52拥有view/provider catalog，Interface03拥有跨DLL UI公共合同。

## 2. 审查边界、currentness与证据等级

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 测试 | 证据等级 | 本轮检查重点 |
|---|---:|---:|---|---|
| 聚焦Zircon源码与资产 | 69 / 10,694 / 372,587 | 89个`#[test]`、0 ignore | E3 | 44个autolayout文件、产品recompute/template/render链、共享constraint合同与聚焦测试 |
| `autolayout/**` | 44 / 4,466 / 当前工作树 | 含parser、vertical band与module内测试 | E3 | skeleton、asset、binding、constraint、tier、geometry和fallback逐文件检查 |
| 产品可达性反查 | 全量`zircon_editor/src`与资产引用 | 0个asset loader caller、0个CSS-like product caller | E3 | declaration到compile、layout、publish、render、pointer的真实链路 |
| 参考源码 | 20 / 16,083 / 589,880 | Unreal/Godot/Fyrox/Bevy/Unity Graphics | E2/E3 | splitter、dock persistence、responsive arrangement、single-tree publication与真实产品pane |

69份聚焦文件按normalized relative path排序，写入`path + NUL + raw bytes + NUL`后取SHA-256，working-tree fingerprint为`68b9521019c7a2e8a82baaf69ae9051bc365386227821a572b945c96afffdae9`。20份参考源码fingerprint为`e4db5ee5108ad855ae62c2d9bab1796e9d20ced3b50fdcae090d29da3ac36e0c`。冻结Git基线为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

聚焦范围内14个文件存在非本轮产生的在途修改，包括constraint defaults/parser/tests、region/floating/vertical/window geometry、autolayout root、两份Editor layout contract tests以及host recompute/presentation。本轮按working tree内容审查并设置`source_recheck_required: true`；实施前必须重算69文件fingerprint并重取product caller矩阵，不能回退共享工作树。

### 2.2 产品authority矩阵

| 声明/状态 | 当前producer | 当前consumer | 工程结论 |
|---|---|---|---|
| `shell_regions.toml` | tracked Editor asset | 仅parse/validation测试 | 没有进入产品构造、依赖追踪或reload |
| `WorkbenchShellRegionsAsset` | parser + exact v2 validator | `from_shell_regions_asset`仅测试 | 不是产品配置authority |
| `CssLikeConstraint` declaration parser | tuple API与单元测试 | 0个production caller | 不是样式/布局语言产品能力 |
| legacy `WorkbenchShellGeometry` | shell builder custom solver | minimum、floating、reuse判断 | 非渲染权威仍影响产品决策 |
| `BuiltinWorkbenchWindowLayoutFrames` | template bridge + Taffy | render、pointer、drag、viewport | 当前可见/可交互frame authority |
| template recompute error | diagnostic log | 继续读取bridge旧frames | 失败仍发布新旧混合snapshot |
| responsive tier | width threshold helper | autolayout、drawer、visibility、toolbar分别消费 | 没有单一compiled responsive policy |
| missing template frame | `Option<UiFrame>` | host projection转为`UiFrame::default()` | 缺失被伪装成合法零frame |

### 2.3 两套算法不是等价实现

legacy path固定求解Left/Document/Right/Bottom四区，先调用axis constraint solver，再做side width compact、vertical band、splitter与floating frame计算。即使hard minimum总和超过available，它也没有typed infeasible结果；side compact释放的宽度又全部加回Document，使总宽度继续越界。

template path则直接把drawer尺寸写到真实ZUI/Taffy节点，另行执行breakpoint、side balance、responsive visibility和toolbar priority。它在Ultra/Narrow width下明确把Bottom Drawer压到panel header，legacy path只按可用高度compact。现有产品测试还明确断言template frames优先于legacy geometry，甚至在legacy frame全为零时仍使用template frame。两者不能被解释为“同一算法的缓存层”。

## 3. 必须保留的工程基础

1. 保留typed `ShellRegionId`、`EditorRegionRole`、`LayoutTier`、`ResolutionContext`及axis constraint值对象，但补齐验证、扩展和provenance。
2. 保留Taffy/componentized template作为真实布局树的方向，并让paint、hit-test、drag、resize、a11y与window minimum从同一published generation读取。
3. 保留现有drawer resize、side balance、responsive visibility和toolbar priority行为测试，迁移到统一responsive policy而非删除。
4. 保留`WorkbenchShellRegionsAsset`的kind/id/version/required-region校验作为source schema起点，但升级为可迁移、可依赖追踪的compiled asset。
5. 保留逻辑像素与scale-factor分离，新增有限值验证、pixel snapping、monitor identity和跨monitor重算。
6. 保留constraint parser的typed错误与属性映射测试，但不能把tuple parser命名成完整CSS兼容层。
7. 保留missing/invalid配置fail-closed的方向；错误必须形成可定位diagnostic和last-good/Unavailable状态，不静默回到默认壳层。
8. 保留Editor13的workspace/dock state authority，Editor54只负责编译当前layout graph与发布几何，不复制文档/窗口持久化。

## 4. 参考源码给出的结构约束

| 参考 | 本轮源码事实 | 对Zircon的约束 | 不照搬的部分 |
|---|---|---|---|
| Unreal `SSplitter` | slot持有SizeRule/Value/MinSize；`ArrangeChildren`生成paint使用的同一geometry；resize回写同一slot coefficient | layout、paint与resize必须共享唯一slot/tree authority，不能一套frame交互、另一套frame决定复用 | 不复制Slate attribute宏和C++ ownership |
| Unreal `SDockingSplitter` | 只把持久docking node的coefficient/min/rule绑定到`SSplitter`，resize回写node并请求保存 | Dock persistence是layout node的上层状态，不应另造并行geometry solver | 不复制其TabManager对象图 |
| Unreal `SResponsiveGridPanel` | slot携带breakpoint column rules，由当前available size在同一arrange pipeline选择 | responsive rule必须在唯一布局计算中解析，结果带当前breakpoint/policy generation | 不要求复制Bootstrap式12列模型 |
| Godot `SplitContainer` / `BoxContainer` | container从真实children/minimum size求布局；drag、keyboard、a11y与visual grabber都修改并重排同一container | Zircon drawer resize、keyboard/a11y、minimum和frame publication必须落在同一节点树 | 不复制Godot notification/property系统 |
| Godot `EditorDockManager` | 管理真实dock controls，保存/恢复slot和floating window work-area状态 | shell graph必须能承载真实pane和floating placement，而不是只保存字符串panel路径 | 持久化schema仍归Editor13 |
| Fyrox Dock/Grid | recursive `Tile`含split fraction/window name并创建真实UI；manager从实际tree/floating windows snapshot/restore | region schema需要递归split/tab/floating graph和稳定pane identity，不能固定六行映射 | 不复制其message API或WidgetBuilder |
| Bevy UI/Taffy | 一棵entity UI tree把style输入映射到Taffy，compute后发布`ComputedNode`；Node公开完整flex/grid语义 | Zircon应只有一棵compiled tree和一次同代computed publication；语言能力要与实际style surface显式对齐 | 不要求ECS成为Editor document模型 |
| Unity Graphics RenderGraphViewer | UXML实例化真实`TwoPaneSplitView`；代码查询真实元素，在GeometryChanged后保存resolved pane size并回写初始dimension | 声明式asset必须由产品实例化，持久尺寸来自真实布局事件，不来自平行估算 | 仅为package级E2证据，不推断完整Unity Editor内部架构 |

## 5. P0：必须先封闭的产品真实性与几何一致性缺陷

### ED54-P0-01 · 已提交的shell asset与constraint language没有产品consumer

`shell_regions.toml`、`WorkbenchShellRegionsAsset::parse`、`from_shell_regions_asset`和CSS-like declaration parser形成了看似完整的声明层，但全量production反查没有加载caller；产品固定走`jetbrains_default()`、token default和ZUI template。资产内`panel_asset`不会实例化任何pane。继续扩展parser或schema只会扩大不可达的第二套设计。

**必须重构：** 建立唯一`WorkbenchLayoutSource -> WorkbenchLayoutCompiler -> CompiledWorkbenchLayout`产品链。启动/项目/extension generation选择source，编译器解析panel/provider、token、constraint、responsive rule和region graph；无provider、无token、无reader或编译失败必须显式Unavailable或last-good，不能静默回默认并宣称加载成功。

### ED54-P0-02 · 同一shell snapshot存在legacy与template两套几何authority

同一次recompute先生成custom `WorkbenchShellGeometry`，再生成Taffy template frames。产品render/input/drag使用后者，window minimum/floating/reuse却使用前者；两套responsive/compact算法不同，且legacy比较会决定是否复用template frames。不存在revision/equivalence proof，任何resize、tier切换或token变化都可能让可见frame、hit target与minimum/floating位置分叉。

**必须重构：** 选择componentized/Taffy tree作为唯一geometry authority，custom solver只能在迁移期作为shadow oracle且不得影响产品；所有region/splitter/floating/minimum/pointer/a11y数据由一次layout result导出。迁移期每帧比较必须产出typed divergence receipt，达到门限后硬删除legacy product path。

### ED54-P0-03 · Template layout失败后发布旧frame与新model/legacy geometry的混合代

`recompute_shell_template_bridge_layout_frames`失败只记录日志，caller继续从bridge取既有frames并构造新presentation；没有prepare/commit、frame generation、rollback或last-good source identity。这样一次普通layout error就能把新的pane/model/tier与旧的frame组合，render、pointer、resize和drag看到的状态也无法证明同代。

**必须重构：** 使用immutable `CompiledWorkbenchLayout`与`PublishedWorkbenchLayout{source_revision, token_generation, model_revision, viewport_metrics, frame_generation}`。编译/求解在candidate中完成，校验必需frame、有限值、bounds与consumer projection后一次原子替换；失败保留完整last-good generation并发布typed health/diagnostic，不能部分推进model或frame。

## 6. P1：工程化能力差距

### 6.1 Source、产品装配与authority（ED54-P1-01～08）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED54-P1-01 | shell asset没有catalog owner、selection rule或load lifecycle | 建立project/default/extension source precedence、stable asset ID、revision和load receipt |
| ED54-P1-02 | `from_shell_regions_asset`只替换regions，chrome assets、mode和其他字段仍来自hardcoded skeleton | source schema必须完整声明或显式继承每一层，并把resolved provenance写入compiled snapshot |
| ED54-P1-03 | `panel_asset: String`没有catalog/provider解析或真实mount | 编译期解析typed pane definition/provider，运行期持有mount lease并处理Unavailable/reload |
| ED54-P1-04 | 没有immutable compiled layout、dependency set或generation | 编译结果绑定source hash、schema/compiler version、token/provider dependencies和BuildSet |
| ED54-P1-05 | CSS-like API只接收测试构造的`(property, value)`列表 | 定义真实asset/declaration reader或诚实降名为typed constraint builder |
| ED54-P1-06 | active design token只影响部分default extents，template与其他defaults没有同一token snapshot | 每次compile/layout消费单一token generation，依赖变化精确invalidates |
| ED54-P1-07 | 失败路径没有产品级Unavailable/last-good状态与用户可定位诊断 | 把parse/link/constraint/layout阶段错误纳入Editor health与source location |
| ED54-P1-08 | 无法从visible pane/frame回溯source region、constraint、token和provider | 发布debug/provenance graph，支持frame到authoring declaration的双向定位 |

### 6.2 Region schema、binding与扩展性（ED54-P1-09～17）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED54-P1-09 | asset固定要求六个region row | schema改为稳定node graph，required builtin role作为policy而非行数断言 |
| ED54-P1-10 | geometry固定枚举Left/Document/Right/Bottom四区 | 用typed node/slot identity表达任意split、stack、overlay和floating subtree |
| ED54-P1-11 | 每个region role映射硬编码，asset不能改变结构职责 | 编译期验证role cardinality、capability与合法parent/child关系 |
| ED54-P1-12 | 无法表达递归split、tab stack、nested drawer或multi-document host | 引入versioned `RegionGraph`和deterministic traversal/order |
| ED54-P1-13 | extension不能贡献受约束的region/pane/placement rule | 通过Editor50 owner-generation contribution lease编译进graph并支持原子撤销 |
| ED54-P1-14 | panel引用是无类型字符串 | 使用qualified view/provider ID、schema version、capability和fallback policy |
| ED54-P1-15 | `WorkbenchConstraintTokenName(String)`允许空值和非法字符 | 构造/serde统一验证、canonicalize并给出source-aware error |
| ED54-P1-16 | schema只接受exact v2，没有migration/unknown-field/roundtrip政策 | 建立version window、migration chain、unknown-field策略与golden roundtrip |
| ED54-P1-17 | 多binding落到同一shell region时静默取max extent | 编译期给出冲突/aggregation rule，要求source显式选择stack/max/sum/priority |

### 6.3 Constraint language与style parity（ED54-P1-18～26）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED54-P1-18 | 所谓declaration parser不是文本grammar，只解析调用者已拆分的tuple | 定义真实grammar/tokenizer/AST，或缩小能力名与公开承诺 |
| ED54-P1-19 | declaration没有file/line/column/span | AST与diagnostic保留source span、property/value片段和include chain |
| ED54-P1-20 | 没有selector、cascade、inheritance、specificity或明确“不支持”边界 | 发布versioned capability profile，禁止用“CSS-like”掩盖不确定语义 |
| ED54-P1-21 | viewport units、`repeat`、`fit-content`、grid auto flow/implicit tracks、direction、box sizing、overflow等缺失 | 按真实产品需求补齐并为每项提供parser→style→Taffy parity；不需要的能力显式拒绝 |
| ED54-P1-22 | parser重复表达`UiLayoutStyle`/Taffy语义，存在三层漂移 | 由单一schema/metadata生成property registry、type checking与lowering |
| ED54-P1-23 | aliases与property family硬编码，无法版本化或由extension贡献 | 建立canonical property ID、alias deprecation和schema capability negotiation |
| ED54-P1-24 | token解析失败通过`.ok()?`变成`None`，caller直接`continue` | missing/invalid token必须是typed compile error或显式optional declaration |
| ED54-P1-25 | parse error缺source asset、owner、修复建议和相关token定义 | 统一diagnostic code、primary/secondary span、fix-it与dependency trace |
| ED54-P1-26 | 测试只验证局部DTO映射，没有产品asset到pixel/hit frame parity | 增加golden source、compiled snapshot、Taffy frame、pointer和pixel端到端测试 |

### 6.4 Constraint求解、geometry与缺失状态（ED54-P1-27～36）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED54-P1-27 | `solve_axis_constraints`只返回vector，不报告feasible、overflow或residual | 返回typed solve result、active constraints、residual与诊断 |
| ED54-P1-28 | hard minimum总和超过available时仍返回超出host的尺寸 | 明确overflow/scroll/compression/failure policy，任何frame必须有限且边界可证明 |
| ED54-P1-29 | side compact释放宽度后全部加回Document，超预算时总宽仍不收敛 | 删除平行solver；shadow期至少以conservation invariant阻断发布 |
| ED54-P1-30 | top/toolbar/status/content vertical band在极小高度下同样可越界 | 统一用真实tree minimum/overflow policy，并覆盖zero/tiny host |
| ED54-P1-31 | axis override和chrome metrics接受NaN、infinity、负数及反转min/max | 构造边界验证有限值、domain、ordering和hard ceiling |
| ED54-P1-32 | region/splitter/floating getter在missing key时返回零rect | 返回`Option/Result`与missing reason，consumer不得把absence当合法geometry |
| ED54-P1-33 | 重复floating `MainPageId`在map collect时last-write-wins | 编译/求解前拒绝重复stable identity并保留owner/source |
| ED54-P1-34 | descriptor重复会被map覆盖，missing descriptor又回退default | Editor52 catalog提供唯一、generation-qualified lookup；缺失fail-close |
| ED54-P1-35 | active tab失效时静默选择first page | 由Editor13在model transaction中修复/迁移并发布diagnostic，不在geometry层猜测 |
| ED54-P1-36 | 多个geometry函数接受却忽略`chrome`，固定四区枚举又散布多处 | 收敛依赖签名、generated traversal和typed node graph，禁止假dirty dependency |

### 6.5 Responsive、density、DPI与monitor（ED54-P1-37～44）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED54-P1-37 | `workbench_layout_defaults()`固定使用`workbench_dense()` | 默认值来自当前design-token/density snapshot并带generation |
| ED54-P1-38 | tier、drawer collapse、visibility和toolbar priority分散在四条路径 | 编译为单一`ResponsivePolicy`，一次选择后传给唯一layout tree |
| ED54-P1-39 | Narrow/Ultra时Bottom Drawer在template与legacy path行为不同 | 用一组source rule和同一frame result消除分叉 |
| ED54-P1-40 | 未知`responsive_min_tier`字符串被`filter_map`静默忽略 | template compile阶段拒绝非法tier并定位node/attribute |
| ED54-P1-41 | breakpoint无hysteresis/debounce，边界resize可反复切换 | 定义enter/exit阈值、resize transaction和稳定测试 |
| ED54-P1-42 | 只有viewport宽度阈值，没有content minimum/container query策略 | tier selection结合真实minimum、locale/text、available container与policy |
| ED54-P1-43 | `ResolutionContext`对非法scale/extent静默fallback/zero | 非有限、非正scale与非法extent返回typed error，保留last-good metrics |
| ED54-P1-44 | 没有monitor identity、pixel snap和跨monitor rounding contract | 发布logical/physical frame、rounding policy、monitor generation与迁移测试 |

### 6.6 Publication、invalidation、性能与观测（ED54-P1-45～52）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED54-P1-45 | model、legacy geometry、template frames没有共同revision | 单一published layout generation绑定全部consumer snapshot |
| ED54-P1-46 | 非权威legacy frame equality决定是否复用权威template frame | reuse只由compiled dependency key和template tree dirty set决定 |
| ED54-P1-47 | host projection把missing/zero-size frame转成`UiFrame::default()` | 传播typed missing/hidden/collapsed状态并阻断错误hit target |
| ED54-P1-48 | 每次shell recompute同时执行custom solver和Taffy tree | shadow迁移结束后删除双算；迁移期独立计时且不进入产品决策 |
| ED54-P1-49 | drawer dimension变化前对所有root调用`mark_roots_layout_dirty` | 建立dependency-indexed dirty roots/subtrees和增量layout receipt |
| ED54-P1-50 | responsive pass每次扫描全部nodes并反复解析字符串attribute | template compile时建立typed responsive index，按tier delta更新受影响节点 |
| ED54-P1-51 | 没有dual-authority divergence、stale-frame fallback或missing-frame telemetry | 增加有界计数、sampled trace、source/generation和health status |
| ED54-P1-52 | 没有大pane树、连续resize、locale/density/DPI切换、视觉与输入性能资格 | 建立同workload CPU/allocation/layout latency、pixel/hit parity和soak receipt |

## 7. P2：质量与维护性收敛

| ID | 当前差距 | 收敛方向 |
|---|---|---|
| ED54-P2-01 | 多个`*Px`类型实际保存logical px，命名容易与physical pixel混淆 | 统一`LogicalPx/PhysicalPx`newtype和转换点 |
| ED54-P2-02 | EPSILON/最小尺寸/折叠尺寸分散 | 按policy/token归属集中并记录量纲 |
| ED54-P2-03 | `ShellRegionId::ALL`与多处手工match会随枚举扩展漂移 | 从schema/node graph生成稳定遍历 |
| ED54-P2-04 | `_chrome`参数以忽略前缀长期留在公开求解签名 | 删除无效依赖或真实消费并测试 |
| ED54-P2-05 | geometry内部直接暴露/依赖多个map的隐式唯一性 | 用typed collection constructor封装唯一性与排序 |
| ED54-P2-06 | region/token/property ID缺一致的Display/debug provenance | 提供稳定diagnostic formatting，避免Debug字符串成为协议 |
| ED54-P2-07 | parser property表、family分类和lowering散在大文件 | 按grammar/schema/lowering/diagnostic拆分，保持生成源单一 |
| ED54-P2-08 | breakpoint与constraint tests大量手工重复fixture | 建立table/property/model test helper，但保留可读golden case |
| ED54-P2-09 | collapsed、hidden、missing和zero-size在多处用数值/Option混合表达 | 引入typed layout visibility/disposition |
| ED54-P2-10 | floating/region/splitter frame API没有统一坐标空间标记 | frame类型携带window/local/logical space |
| ED54-P2-11 | error文案直接包含字段字符串，缺稳定diagnostic code | code与localizable presentation分离 |
| ED54-P2-12 | 文档把测试可达性描述成“authored path feeds geometry”，与产品事实不一致 | 文档绑定product receipt并随source fingerprint currentness更新 |

## 8. 目标架构

```text
WorkbenchLayoutSource(s)
  + project/default/extension precedence
  + RegionGraph / PaneRef / ConstraintAst / ResponsivePolicy
  + source spans / schema version / dependency identities
                         |
                         v
WorkbenchLayoutCompiler
  parse -> link providers/tokens -> validate graph -> lower UiLayoutStyle
  -> CompiledWorkbenchLayout(source revision, compiler version, dependency set)
                         |
                         v
Single Componentized/Taffy Layout Tree
  model revision + viewport/monitor metrics + density/locale/token generation
  -> prepare layout -> validate required nodes/finite bounds/consumer projections
                         |
                         v
Atomic PublishedWorkbenchLayout
  frame generation + region/splitter/floating/minimum frames
  + hidden/collapsed/missing disposition + provenance + diagnostics
                         |
        +----------------+----------------+----------------+
        v                v                v                v
      paint           pointer         drag/resize        a11y
```

Editor13的workspace/dock document只保存stable node/pane identity、split coefficient、tab/floating placement和migration state；Editor52解析pane/provider；Editor54把两者与layout source编译成当前树。任何consumer都不得重新估算frame，任何错误都不得把新model与旧frame拼成一个snapshot。

## 9. 分阶段重构

### M0 · Capability truth与negative regression

- 把未接入产品的shell asset/CSS-like能力标记为experimental/internal，冻结0 production caller事实；
- 为三个P0补negative tests：asset未消费、dual geometry divergence、template failure旧新混代；
- 建立legacy/template frame divergence trace，legacy仍不得影响新的产品决定。

### M1 · Source schema与compiler

- 建立versioned RegionGraph、typed PaneRef、ConstraintAst、ResponsivePolicy与source span；
- 接入catalog/provider、token/density/locale依赖与extension contribution generation；
- 编译出immutable artifact和diagnostic，不允许invalid token/unknown tier静默消失。

### M2 · 单一geometry authority硬切

- 让componentized/Taffy tree导出region、splitter、floating、minimum和consumer projection；
- 迁移paint/pointer/drag/resize/a11y/window minimum到同一generation；
- parity gates通过后删除custom geometry的产品调用与reuse判断。

### M3 · 原子发布、last-good与恢复

- candidate compile/layout/validate成功后一次commit；
- 失败保留完整last-good generation，发布typed health与source diagnostic；
- 关闭missing→zero、duplicate last-win和model/frame跨代组合。

### M4 · Responsive、DPI与持久布局协同

- 单一responsive policy支持hysteresis、content minimum、locale/density与container；
- 建立logical/physical pixel、monitor generation、snapping和跨monitor迁移；
- 与Editor13完成split/tab/floating state schema及真实geometry event回写。

### M5 · 增量性能与产品资格

- 编译typed node/property/responsive indexes，按dependency dirty subtree；
- 连续resize、多pane、多window、多DPI/locale/density下做pixel/hit/a11y parity；
- 同workload报告layout CPU、allocation、dirty node count、p50/p95/p99与resident bytes。

## 10. 资格门

| Gate | 验收内容 |
|---|---|
| G01 | 69份聚焦文件与20份参考文件路径存在，实施时fingerprint已重取并处理14个在途文件 |
| G02 | 产品启动实际加载选定`WorkbenchLayoutSource`，receipt绑定source revision、BuildSet与compiler version |
| G03 | `shell_regions.toml`中的pane引用经Editor52 provider解析并mount真实内容 |
| G04 | 无reader/provider/token/schema时入口fail-close或完整保留last-good，不静默回hardcoded success |
| G05 | CSS-like能力名称与versioned capability profile一致，unsupported property确定性报错 |
| G06 | parser diagnostic包含asset、line、column、span、code和dependency trace |
| G07 | schema migration、unknown field、roundtrip和downgrade policy通过golden测试 |
| G08 | RegionGraph支持recursive split、tab、drawer、document host和floating node |
| G09 | plugin region/pane contribution按owner generation原子安装与撤销 |
| G10 | duplicate node/pane/floating identity在compile阶段拒绝，不last-write-wins |
| G11 | invalid/empty token name、NaN/infinity/negative/reversed constraint全部拒绝 |
| G12 | constraint property到`UiLayoutStyle`/Taffy lowering有全量capability parity测试 |
| G13 | token、density、locale、provider变化只使依赖的compiled/layout subtree失效 |
| G14 | 产品只有一个geometry authority；legacy solver不参与minimum、reuse、floating或input决策 |
| G15 | paint、pointer、drag、resize、viewport和a11y读取同一frame generation |
| G16 | template/layout失败时model、frames和consumer projections都不部分推进 |
| G17 | published snapshot绑定source/model/token/viewport/monitor/frame generation |
| G18 | required frame缺失返回typed failure，不转成`UiFrame::default()` |
| G19 | hidden、collapsed、missing与valid zero-size具有不同typed disposition |
| G20 | axis hard minimum不可满足时返回typed infeasible/overflow结果 |
| G21 | 所有成功frame有限、非负，并满足父子bounds/overflow policy与宽高conservation invariant |
| G22 | tiny/zero host、极端drawer尺寸和极长locale文本不会生成越界hit target |
| G23 | responsive tier、drawer、visibility与toolbar只由一个compiled policy决定 |
| G24 | breakpoint hysteresis在边界连续resize中不抖动，transaction结果可重放 |
| G25 | Bottom Drawer等所有region在同一tier只有一套collapse规则 |
| G26 | active density/design token snapshot贯穿default、template与minimum计算 |
| G27 | 非法scale/extent保留last-good并报告错误，不归零后继续发布 |
| G28 | logical/physical pixel、snapping和monitor迁移在100%/125%/150%/200%通过golden |
| G29 | Editor13 dock/tab/floating state由真实layout event回写且save/reopen等价 |
| G30 | extension reload、pane unavailable、locale/density/DPI切换均原子换代 |
| G31 | dirty propagation不再每次扫描全部node或标记全部root |
| G32 | 1K pane/node、连续resize与多window workload报告CPU、allocation和dirty-node规模 |
| G33 | fault injection覆盖parse/link/layout/publish/mount失败且无新旧混代 |
| G34 | pixel、hit-test、drag target、splitter、keyboard/a11y action在同一golden frame一致 |
| G35 | soak结束后无stale layout generation、orphan pane mount或extension owner引用 |
| G36 | 同场景同窗口同DPI同硬件且G01-G35全过前，不宣称布局性能或表现优于Unreal |

## 11. 缺失测试矩阵

1. 真实启动从`WorkbenchLayoutSource`加载、link provider、mount pane并生成首帧receipt。
2. 修改asset region/token/responsive declaration后精确invalidate并原子换代。
3. invalid token、unknown tier/property、duplicate identity、missing provider与schema migration。
4. custom/template shadow在Narrow/Ultra、bottom drawer、tiny host与极端minimum下的divergence corpus。
5. template recompute在prepare/link/layout/publish各阶段失败，断言完整last-good不变。
6. paint、pointer、drag、resize、viewport与a11y携带相同frame generation。
7. hard minima总和大于available、NaN/infinity、negative scale和zero host的property/model tests。
8. locale、density、theme token、125%/150% DPI和跨monitor连续迁移pixel golden。
9. recursive split/tab/floating graph的save/reopen/migration和provider unavailable恢复。
10. extension contribution install/reload/revoke与正在drag/resize并发。
11. 1K pane/node连续resize下dirty subtree、allocation、p50/p95/p99 layout latency。
12. 长时间resize/tier/DPI/provider churn后0 stale generation、0 orphan mount和stable memory。

## 12. Owner、依赖与非目标

| 域 | Canonical owner | Editor54依赖/交接 |
|---|---|---|
| Workbench source/compiler/region graph/responsive/geometry publication | Editor54 | 本报告唯一owner |
| Generic retained tree、Taffy mapping、incremental layout、paint/hit/a11y | Editor01 + Runtime11A | Editor54消费并提出shell workload，不复制通用引擎 |
| Workspace profile、dock/tab/floating persistence与migration | Editor13 | 提供stable state，Editor54编译/发布当前geometry |
| Builtin/extension pane descriptor、provider与capability | Editor52 + Editor50 | 编译期link，按owner generation mount/revoke |
| UI公共constraint/style/ABI | Interface03 | Editor54不私自扩展跨DLL wire合同 |
| Design token、density、locale | Editor12及现有UI token owner | 提供generation-qualified dependency snapshot |

本报告不优化PowerShell/Python tooling，不重写Taffy，不把完整Web CSS当作目标，也不复制Unreal Slate/Godot object system。性能目标是在保持source可追踪、错误恢复、DPI/a11y和一致性后，以单树、编译索引、增量dirty和可测数据布局超越参考实现，而不是删掉工程语义换取局部微基准。

## 13. 禁止的临时修补

1. 禁止只在startup手写读取一次`shell_regions.toml`，然后继续让template走hardcoded结构。
2. 禁止再加第三套geometry或用更多frame copy桥接两套authority。
3. 禁止用“多数窗口尺寸下看起来相同”代替single-generation invariant。
4. 禁止template layout失败后继续发布新model配旧frame。
5. 禁止把missing frame、invalid metrics或unknown tier转换为零值/`None`后静默继续。
6. 禁止把更多CSS property塞进大match而不建立grammar、capability和lowering parity。
7. 禁止用字符串拼接pane、region、token、monitor或owner identity。
8. 禁止为通过测试删除narrow/bottom drawer、template-priority或resize现有行为。
9. 禁止让legacy shadow solver继续决定reuse、minimum或floating产品结果。
10. 禁止在无pixel/hit/fault/soak和同workload receipt时宣称达到或超过Unreal。

## 14. 状态与产出记录

本轮完成69份Zircon聚焦源码/资产、89项聚焦测试、全量产品caller反查及20份参考源码对照，新增本专项报告并同步Editor索引、顶层索引、coverage与跨报告P0 owner总账。未修改production/tests，未运行Cargo、GUI、真实DPI/monitor、fault injection、pixel、a11y、soak或性能基准。

静态review完成不表示重构完成。先以M0冻结三个P0和产品truth，再按M1-M4硬切到compiled source、单一Taffy geometry与原子published generation；只有G01-G36全部通过，Workbench Shell才可声明具备工程级声明布局与响应式产品闭环。
