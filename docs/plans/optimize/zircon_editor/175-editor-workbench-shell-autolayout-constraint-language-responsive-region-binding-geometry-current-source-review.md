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
  - zircon_editor/src/ui/workbench/autolayout/css_like_constraint/declaration_parser/aspect_ratio_tests.rs
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
refreshes:
  - docs/plans/optimize/zircon_editor/54-editor-workbench-shell-autolayout-constraint-language-responsive-region-binding-geometry-product-integration-review.md
  - docs/plans/optimize/zircon_editor/127-editor-workbench-shell-autolayout-constraint-language-responsive-region-binding-geometry-current-source-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 175 · Editor Workbench Shell AutoLayout / Constraint Language / Responsive Tier / Region Binding / Geometry 当前源码复核

## 1. 结论

Editor54的主裁决没有改变：Zircon已经有typed region、drawer state、轴向约束、logical/physical scale boundary、Taffy retained tree、template frame、pointer/drag/viewport consumer和大量局部测试，但这些基础仍未形成可配置、可编译、单权威、同代原子发布的Workbench布局产品。当前代码不能据此宣称已达到Unreal/Godot/Fyrox/Bevy或Unity Graphics示例所体现的工程完整度，更没有跨引擎同场景性能或表现证据。

三项P0仍全部为 **Open**：提交的`shell_regions.toml`和CSS-like declaration API依旧没有生产reader/caller；legacy `WorkbenchShellGeometry`与componentized/Taffy frame仍在同一次recompute中并行求解并共同影响产品；template recompute失败仍只写日志，然后读取bridge中的既有frame并提交新的model/legacy geometry。`CommittedShellState`没有source/model/token/viewport/frame generation，不能证明paint、pointer、drag、resize、minimum、floating和a11y读取同一代结果。

本轮确认的局部进展是：active design-token snapshot会刷新默认region extent；CSS-like值解析会拒绝一部分非有限值并显式标出部分unsupported语法；side drawer增加了预算平衡helper；关键布局路径增加了profile counter和bounded Editor error log。这些进展使9项P1成为Partial，但没有关闭任何canonical P0/P1。当前状态为 **3项P0 Open；52项P1中43 Open / 9 Partial；12项P2中10 Open / 2 Partial；36门中24 Fail / 11 Partial / 1 Pass**。

## 2. 审查边界与currentness

### 2.1 当前磁盘冻结

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 证据 |
|---|---:|---|
| Zircon布局源码、资产与聚焦测试 | **72 / 11,521 / 10,563 / 398,957 / 106 / 4** | autolayout、生产recompute/template/render链、Runtime/Interface constraint以及聚焦测试；fingerprint `7accc225c610450d4e65743ccd513f533e4baf4e62c400ee3e087618f951e7ef` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics参考 | **20 / 16,103 / 13,972 / 589,880 / 42 / 0** | splitter、dock、responsive arrange、single-tree computed publication与真实pane实例化；fingerprint `e4db5ee5108ad855ae62c2d9bab1796e9d20ced3b50fdcae090d29da3ac36e0c` |
| 计划与契约 | **8 / 3,531 / 2,549 / 509,274 / 2 / 0** | owner、ABI、workspace persistence、provider和Workbench契约；fingerprint `b43391ae9271aa2b79b7914fe5bd721b6f8a023bdaf63f4f2971f1fcfb8b8efb` |
| 全部选择集 | **100 / 31,155 / 27,084 / 1,498,111 / 150 / 4** | normalized path + NUL + raw bytes + NUL的SHA-256；fingerprint `c6482b5c4f18c1e9420d198c279044dd459e6aebcc67775d0bb68193be6c941f` |

冻结时间为`2026-08-27T17:32:49+08:00`，HEAD为`ea35974cdf64068f6789010451d20bbf69e0a29d`，共享工作树有8,268条status记录。聚焦范围内存在大量非本轮产生的在途修改与未跟踪测试文件；本报告审查当前磁盘内容，不回退共享修改，实施前必须重取fingerprint和caller矩阵。

### 2.2 静态一致性阻断

当前`drawer_layout.rs`中的`narrow_width_collapses_a_visible_bottom_drawer_to_its_tab_strip`仍以5个参数调用只有4个参数的`compacted_bottom_region_input`。本轮按用户要求只做review，没有运行Cargo，因此不把静态观察冒充编译结果；但在修正调用并取得Windows Cargo receipt前，当前在途源码不能作为可构建基线或任何性能证据。该漂移不新增canonical finding，归入G01之外的实施前build prerequisite与`source_recheck_required`。

## 3. 当前产品authority证据

| 声明或状态 | 当前生产事实 | 判定 |
|---|---|---|
| `shell_regions.toml` | 全量production反查为0个路径引用 | tracked资产，不是产品source authority |
| `WorkbenchShellRegionsAsset` | production仅有类型、parser与re-export；`from_shell_regions_asset`无外部生产caller | 测试DTO，不是load/compile/reload链 |
| CSS-like constraint | 入口仍是调用者预拆分的`(&str, &str)`；production无reader/caller | typed builder，不是完整约束语言 |
| `WorkbenchSkeleton::jetbrains_default()` | 产品默认region extent由其解析active tokens | hardcoded skeleton仍是实际source |
| legacy `WorkbenchShellGeometry` | 每次full/window-metrics recompute仍执行；决定minimum、floating、reuse并保存在Host | 第二套产品geometry authority |
| componentized/Taffy frames | render、pointer、drag、resize与viewport直接消费 | 当前可见/交互frame authority |
| root template bridge | `recompute_layout_with_workbench_model_at_scale`仍以`_model`、`_metrics`命名并完全忽略 | API名称不能证明model-driven layout |
| workbench template bridge | model确实驱动drawer/toolbar/control state，随后Taffy recompute | 可保留的真实产品底座 |
| layout失败 | 两级bridge捕获`Err`并写bounded log，函数最后仍返回`layout_frames()` | 有诊断但无candidate/rollback/last-good generation |
| missing frame | frame DTO为`Option<UiFrame>`，多个shell pointer/drag consumer仍`unwrap_or_default()` | absence继续伪装为合法零frame |
| responsive | drawer、node visibility、toolbar和legacy geometry分别调用tier/compact helper | 共享helper不等于单一compiled policy |

## 4. 三项P0状态

| ID | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| ED54-P0-01 | **Open** | asset/parser仍0生产reader；产品继续走hardcoded skeleton和ZUI document | 建立`WorkbenchLayoutSource -> compiler -> immutable artifact`唯一入口，link provider/token/region/responsive dependency并发布source receipt |
| ED54-P0-02 | **Open** | builder先算legacy geometry，再按legacy equality决定是否复用Taffy frame；minimum/floating/reuse与可见frame分属两套结果 | Taffy/componentized tree成为唯一authority；迁移期legacy只能是shadow oracle且不得参与产品决策，最终硬删除 |
| ED54-P0-03 | **Open** | bridge失败只记录日志，caller仍读取旧frames、提交新model/geometry并清dirty/pending commit | candidate compile/layout/validate成功后原子换代；失败完整保留last-good并发布typed health，不得部分推进 |

## 5. P1逐项复核

### 5.1 Source、产品装配与authority

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ED54-P1-01 | Open | 无catalog owner、selection precedence、stable revision或load receipt；建立project/default/extension source lifecycle |
| ED54-P1-02 | Open | asset只替换regions，其余字段继承`jetbrains_default()`；root bridge仍忽略model/metrics；完整声明或显式记录继承provenance |
| ED54-P1-03 | Open | `panel_asset: String`不解析catalog/provider、不持有mount lease；改为qualified pane/provider reference |
| ED54-P1-04 | Open | 无immutable compiled layout、dependency set、compiler/schema version与BuildSet identity |
| ED54-P1-05 | Open | parser仍接收tuple而非真实asset/declaration reader；补grammar reader或诚实降名 |
| ED54-P1-06 | Partial | active token `Arc`变化会刷新region defaults，但template/chrome/minimum没有共同token generation |
| ED54-P1-07 | Partial | template失败已有bounded Editor error log，但没有source-aware Unavailable、完整last-good或health state |
| ED54-P1-08 | Open | visible frame不能回溯source region、declaration、token、provider或owner generation |

### 5.2 Region schema、binding与扩展性

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ED54-P1-09 | Open | asset仍要求exact六region bitset；required builtin role应成为graph policy |
| ED54-P1-10 | Open | geometry固定Left/Document/Right/Bottom枚举，不能表达任意slot/tree |
| ED54-P1-11 | Open | region-role结构硬编码，缺cardinality/capability/parent-child compile validation |
| ED54-P1-12 | Open | 无recursive split、tab stack、nested drawer、multi-document host或floating subtree graph |
| ED54-P1-13 | Open | extension contribution未按owner generation编译进region graph，也无原子撤销 |
| ED54-P1-14 | Open | panel引用仍是无类型字符串，无schema/capability/fallback policy |
| ED54-P1-15 | Open | `WorkbenchConstraintTokenName::new`与transparent serde继续接受空值和非法字符 |
| ED54-P1-16 | Open | exact v2，无migration window、unknown-field policy或roundtrip/downgrade合同 |
| ED54-P1-17 | Open | 同region多binding仍以`max`静默聚合extent，未要求显式stack/max/sum/priority rule |

### 5.3 Constraint language与style parity

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ED54-P1-18 | Open | declaration入口不是文本grammar/token stream/AST，只是tuple iterator |
| ED54-P1-19 | Open | 无asset、line、column、span、include chain或secondary token span |
| ED54-P1-20 | Partial | 已区分部分known-unsupported property/unit/syntax，但无versioned capability profile、selector/cascade/inheritance边界 |
| ED54-P1-21 | Partial | flex/grid/gap/inset/overflow/aspect ratio surface扩大且部分语法确定性拒绝；viewport unit、repeat、fit-content、auto-flow等仍缺完整parser-to-Taffy parity |
| ED54-P1-22 | Open | property enum、parser、`UiLayoutStyle`与Taffy mapping仍是多层手工表，缺单一schema生成源 |
| ED54-P1-23 | Open | alias/canonical token映射硬编码，不能版本化、deprecate或由extension negotiation |
| ED54-P1-24 | Open | CSS resolver能返回`UnknownToken`，但skeleton产品默认路径仍用`.ok()?`吞错误并`continue` |
| ED54-P1-25 | Open | error无source asset/owner/diagnostic code/fix-it/dependency trace |
| ED54-P1-26 | Open | 106个静态test marker不能替代真实asset -> compiled artifact -> Taffy -> pixel/hit端到端receipt |

### 5.4 Constraint求解、geometry与缺失状态

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ED54-P1-27 | Open | `solve_axis_constraints`仍返回裸`Vec<ResolvedAxisConstraint>`，不报告feasible/overflow/residual/active constraint |
| ED54-P1-28 | Open | hard minimum总和超过available时测试仍明确返回`[8,8]`给available 5，结果可越界 |
| ED54-P1-29 | Partial | 新`balanced_side_widths_for_budget`改善一般side budget，但solver已超预算时release仍全部加回Document，不能证明收敛到available |
| ED54-P1-30 | Open | vertical fixed bands在tiny height下仍可能由minimum推过host，缺typed overflow policy |
| ED54-P1-31 | Open | CSS数值有局部finite检查，但axis override、chrome metrics与多处fixed-axis仍接受NaN/infinity/negative/reversed min/max |
| ED54-P1-32 | Open | legacy region/splitter/floating getter继续`unwrap_or_default()`，missing变零rect |
| ED54-P1-33 | Open | duplicate floating `MainPageId`仍经`collect::<BTreeMap>` last-write-wins |
| ED54-P1-34 | Open | descriptor HashMap仍覆盖duplicate，missing descriptor仍由下游默认策略吸收 |
| ED54-P1-35 | Open | invalid active tab仍可在geometry/model projection层静默选first page |
| ED54-P1-36 | Open | `_chrome`仍被多个geometry public path接受后忽略，固定枚举/match持续漂移 |

### 5.5 Responsive、density、DPI与monitor

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ED54-P1-37 | Open | `WorkbenchChromeMetrics::default()`仍固定`workbench_dense()`，不是当前density generation |
| ED54-P1-38 | Partial | 多路径复用部分tier/compact helper，但drawer、visibility、toolbar与legacy solver仍独立执行，未编译成一次`ResponsivePolicy`选择 |
| ED54-P1-39 | Open | Narrow/Ultra的Bottom Drawer template规则与legacy vertical compact仍不等价 |
| ED54-P1-40 | Open | unknown `responsive_min_tier`继续由`filter_map`静默丢弃，测试还把`None`当预期 |
| ED54-P1-41 | Open | breakpoint无hysteresis/debounce/transaction replay，边界resize可抖动 |
| ED54-P1-42 | Open | tier只看宽度，无content minimum、locale text或container-query policy |
| ED54-P1-43 | Open | invalid scale/extent继续静默归一到1或0，不能区分坏输入与合法零尺寸 |
| ED54-P1-44 | Open | 无monitor identity、logical/physical typed frame、pixel snap或跨monitor rounding contract |

### 5.6 Publication、invalidation、性能与观测

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ED54-P1-45 | Partial | `CommittedShellState`聚合model/geometry/frames是基础，但没有source/token/model/viewport/monitor/frame generation |
| ED54-P1-46 | Open | legacy frame equality仍决定Taffy frame reuse；bridge error后仍可把旧frame当本次结果 |
| ED54-P1-47 | Open | required frame仍为Option并在shell pointer/drag/projection路径转为default frame |
| ED54-P1-48 | Open | full recompute继续同时运行custom solver与Taffy，shadow成本还进入产品热路径 |
| ED54-P1-49 | Open | drawer apply仍先遍历并标记全部surface roots，之后又对变化node单独dirty |
| ED54-P1-50 | Open | responsive pass每次扫描全部nodes并运行时解析字符串tier，非法值静默跳过 |
| ED54-P1-51 | Partial | 已有profile counter和两类bounded layout error log，但无dual-authority divergence、stale generation或missing-frame telemetry |
| ED54-P1-52 | Partial | 有局部perf counter、单pass aggregation与ignored microbenchmark；没有大pane树、连续resize、DPI/locale、pixel/hit parity或soak资格 |

## 6. P2逐项复核

| ID | 状态 | 当前差距与收敛方向 |
|---|---|---|
| ED54-P2-01 | Open | 多个`*Px`实际保存logical units；统一`LogicalPx/PhysicalPx`newtype |
| ED54-P2-02 | Open | EPSILON、minimum、collapse尺寸分散；按policy/token owner集中并标量纲 |
| ED54-P2-03 | Open | `ShellRegionId::ALL`和手工match随枚举扩展漂移；从compiled graph生成遍历 |
| ED54-P2-04 | Open | `_chrome`等忽略参数长期留在求解签名；删除或真实消费 |
| ED54-P2-05 | Open | 多个map依赖隐式唯一性；使用validated typed collection constructor |
| ED54-P2-06 | Open | region/token/property ID缺稳定Display与provenance formatting |
| ED54-P2-07 | Partial | declaration parser和aspect-ratio tests已拆文件，但property registry/lowering/diagnostic仍无单一生成源 |
| ED54-P2-08 | Partial | 新增table/microbenchmark helper与局部fixture复用，但breakpoint/constraint corpus仍大量重复且未形成model/property test kit |
| ED54-P2-09 | Open | collapsed/hidden/missing/zero-size仍混用数值、Option与visibility |
| ED54-P2-10 | Open | floating/region/splitter frame API没有window/local/logical/physical坐标空间标记 |
| ED54-P2-11 | Open | error文案仍直接嵌字段字符串，无稳定diagnostic code与localizable presentation分层 |
| ED54-P2-12 | Open | 产品文档仍需绑定source/compile/publish receipt，不能以测试可达性宣称产品可配置 |

## 7. 参考引擎约束

| 参考 | 源码结构事实 | Zircon必须满足的约束 |
|---|---|---|
| Unreal `SSplitter` / `SDockingSplitter` | slot rule、minimum、arranged geometry与resize coefficient属于同一树；dock node持久化回写同一slot | layout、paint、resize与persistence不得由平行solver估算；所有consumer读取同一generation |
| Unreal `SResponsiveGridPanel` | breakpoint rule在同一次arrange中按available size选择 | responsive rule必须成为compiled tree输入，而不是四条后处理路径 |
| Godot `SplitContainer` / `BoxContainer` | 从真实child/minimum求布局；drag、keyboard、a11y和visual grabber修改同一container | drawer resize、minimum、hit target与a11y必须绑定同一node/lease |
| Godot `EditorDockManager` | 管理真实dock controls、floating placement和workspace restore | region graph必须承载真实pane identity，不是六行字符串映射 |
| Fyrox Dock/Grid | recursive tile表达split/window并创建真实UI，manager从实际tree snapshot/restore | 需要versioned recursive graph、deterministic traversal与真实mount |
| Bevy UI/Taffy | entity UI tree把style映射到Taffy并发布`ComputedNode` | 一棵compiled tree、一次同代computed publication；language surface与实际style显式对齐 |
| Unity Graphics RenderGraphViewer | UXML实例化真实`TwoPaneSplitView`，GeometryChanged后保存resolved size | 声明资产必须由产品实例化，持久尺寸来自真实layout event |

## 8. 目标架构与重构顺序

```text
WorkbenchLayoutSource(project/default/extension)
  -> parse RegionGraph / PaneRef / ConstraintAst / ResponsivePolicy
  -> link provider + token + locale + density + owner generation
  -> validate identity / source spans / capability / migration
  -> immutable CompiledWorkbenchLayout
  -> one Componentized/Taffy tree prepare
  -> validate required frames / finite bounds / overflow / projection
  -> atomic PublishedWorkbenchLayout(generations + disposition + provenance)
  -> paint | pointer | drag/resize | window minimum | floating | a11y
```

1. **M0 capability truth**：把未接产品的asset/CSS-like层标为experimental/internal，修复当前静态签名漂移，建立三个P0 negative regression。
2. **M1 source compiler**：建立versioned RegionGraph、PaneRef、ConstraintAst、ResponsivePolicy、source span、dependency set和diagnostic；接Editor52 provider与Editor50 owner generation。
3. **M2单一geometry authority**：Taffy树导出region/splitter/floating/minimum/consumer projection；legacy仅shadow比较，随后硬删除其产品调用与reuse判断。
4. **M3原子发布**：candidate求解、校验和projection全部成功后一次commit；失败保留完整last-good并发布health/diagnostic。
5. **M4 responsive/DPI/workspace**：统一hysteresis、content minimum、locale/density、monitor/snapping；与Editor13真实dock/tab/floating state回写协同。
6. **M5性能资格**：dependency-indexed dirty subtree；1K node、多window、连续resize与DPI/locale churn下报告CPU、allocation、dirty count、p50/p95/p99、pixel/hit/a11y parity和soak。

## 9. 资格门状态

| Gate | 状态 | 当前缺口 |
|---|---|---|
| G01 | Pass | 72份Zircon、20份参考与8份计划/契约路径存在并已冻结；实施仍须重取current-disk指纹 |
| G02 | Fail | 产品不加载versioned `WorkbenchLayoutSource`，无compiler/BuildSet receipt |
| G03 | Fail | `panel_asset`未经Editor52 provider解析或mount |
| G04 | Fail | 无reader/provider/token时仍由hardcoded default形成表面成功 |
| G05 | Partial | 部分unsupported property/unit/syntax确定性报错，但无versioned capability profile |
| G06 | Fail | diagnostic无asset/line/column/span/code/dependency trace |
| G07 | Fail | 无migration、unknown-field、roundtrip/downgrade policy |
| G08 | Fail | 无recursive split/tab/drawer/document/floating RegionGraph |
| G09 | Fail | extension region/pane未按owner generation原子安装撤销 |
| G10 | Fail | duplicate floating/descriptor identity仍可last-write-wins |
| G11 | Partial | CSS局部拒绝invalid numeric；token name、axis/chrome/reversed constraint仍未统一拒绝 |
| G12 | Partial | property到`UiLayoutStyle`有局部测试，未覆盖完整capability到Taffy parity |
| G13 | Partial | active token pointer变化会刷新default extents，其他dependency/subtree无精确失效 |
| G14 | Fail | legacy solver仍参与minimum、floating、reuse与Host state |
| G15 | Partial | paint/pointer/drag/viewport主要共享template frames，但无共同generation且a11y未证明 |
| G16 | Fail | template失败仍可提交新model与旧frame |
| G17 | Fail | published snapshot不绑定source/model/token/viewport/monitor/frame generation |
| G18 | Fail | required frame missing仍可变成default frame |
| G19 | Fail | hidden/collapsed/missing/valid-zero没有统一typed disposition |
| G20 | Fail | axis hard minimum不可满足时仍返回裸vector |
| G21 | Partial | 多处做non-negative normalization，但无finite/bounds/conservation/overflow publish proof |
| G22 | Fail | tiny/zero host仍可能产生越界band或hit target |
| G23 | Partial | 共用tier helper是基础，四条responsive policy仍未合一 |
| G24 | Fail | 无breakpoint hysteresis与可重放resize transaction |
| G25 | Fail | Bottom Drawer同tier仍有template/legacy两套collapse规则 |
| G26 | Partial | active density token只贯穿部分region default，不覆盖template/chrome/minimum |
| G27 | Fail | invalid scale/extent归零或回退后继续发布 |
| G28 | Fail | 无100/125/150/200%与跨monitor pixel golden |
| G29 | Fail | Editor13 state未由单一真实layout event同代回写 |
| G30 | Fail | extension/pane/locale/density/DPI切换无atomic generation |
| G31 | Partial | changed node会局部dirty，但drawer仍标全部root、responsive仍扫全部node |
| G32 | Partial | 有ignored局部microbenchmark和perf counter，无1K pane连续resize同workload报告 |
| G33 | Fail | 无parse/link/layout/publish/mount fault-injection原子性矩阵 |
| G34 | Partial | 有局部template/pointer/drag frame测试，无pixel/hit/splitter/keyboard/a11y同代golden |
| G35 | Fail | 无layout generation、pane mount或extension owner长时leak census |
| G36 | Fail | 未取得G01-G35全过及同硬件同场景跨引擎证据，禁止宣称优于Unreal |

## 10. 缺失测试与本轮非目标

后续实施至少需要：真实启动asset-to-pane首帧receipt；invalid token/property/tier/identity与schema migration；dual-authority divergence corpus；template各阶段fault与完整last-good；paint/pointer/drag/resize/a11y同generation；hard-minimum infeasible、NaN/infinity/zero host property tests；locale/density/多DPI/跨monitor pixel golden；recursive split/tab/floating save/reopen；extension install/reload/revoke并发；1K node连续resize与长时间churn。

本轮只写review、索引与coverage，没有修改Rust实现，也没有运行Cargo、Editor、ZUI compiler、GUI、DPI/monitor、fault、scale、soak或跨引擎benchmark。Tooling实现按用户要求排除，未来迁移Rust；本轮没有查询、轮询、等待或实时跟踪协调器状态。
