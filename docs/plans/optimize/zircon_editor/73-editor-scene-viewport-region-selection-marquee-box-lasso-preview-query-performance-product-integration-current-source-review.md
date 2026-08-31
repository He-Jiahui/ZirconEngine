---
report_id: Editor73
title: Editor Scene Viewport Region Selection Current-Source Review
category: zircon_editor
review_date: 2026-08-23
baseline_head: a922089697e41e07fa29e3e42a5e4c9afc1ae31b
baseline_epoch: 341
reference_revisions:
  unreal: a922089697e41e07fa29e3e42a5e4c9afc1ae31b
  godot: 8c7e6c5877a78e8e61ea4fd42673219a9091dca7
  fyrox: 8d815db36494f1badb347547dfc7094bf4fbbdf8
  bevy: fb89a8649d9b359e53ffb6e5492ebb7c059ac8af
  unity_graphics: a7e4c051d256a781ab362c64316b125a1e104694
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/73-editor-scene-viewport-region-selection-marquee-box-lasso-preview-query-performance-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Region Selection、Marquee、Box/Lasso、Preview、Query、Mutation、Performance 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon已经存在一条可执行的矩形框选最小链路：primary press会建立`ViewportDragSession::PrimarySelection`，移动超过固定阈值后把session标记为active，release时由`selectable_owners_in_rect`扫描候选，最后把`Replace / Extend / Toggle`应用到`SelectionModel`。点选侧最近还获得了renderer-visible spatial snapshot、generation currentness、ray query统计和stale/preparing fail-close；Cancel与第二按键覆盖旧drag的两条旧P0路径也已在当前工作树中被修正。这个进展应保留，不能按旧报告快照误判为“完全没有框选”。

但当前实现仍只是release-time矩形命中辅助函数，不是工程级Region Selection产品。拖动中没有marquee填充/描边、window/crossing状态、候选preview或query状态；move事件甚至不产生RenderChanged/PresentationChanged，因此即使后续单独添加绘制代码也不会稳定刷新。session只保存`start/current/active/target/mutation`，没有document、world、viewport、view、frame、pointer/capture generation或source product identity；release也不重新验证桥接器、world/view与候选generation。场景或视图在拖动期间替换时，旧候选的数值NodeId可能被提交到新场景。

矩形查询把拖动方向归一化后丢弃，逐个扫描所有renderable proxy和scene gizmo pick shape，只做投影圆与矩形相交。它没有Window/Crossing/AutoDirection、透明frustum/呈现像素、严格包含、lasso、subobject/provider、query receipt、预算、取消或异步preview。它还绕过点选已经获得的renderer-visible snapshot，继续消费retained static candidates。Editor03/59/70已经拥有几何精度、共享空间产品、可见性、hidden/locked、near-plane、owner dedup与selection eligibility父问题；本报告不重复计数这些底层差距，只登记Region Selection如何消费父能力、如何预览和原子提交的产品缺口。

选择提交也不是一个用户手势对应一个原子变更。`Toggle`对候选逐项调用`toggle_active`，会让domain generation与model revision按候选数增长，并逐步改变primary/order；最终顺序又依赖render mesh与gizmo遍历。产品没有`RegionSelectionRequest / QueryReceipt / SelectionMutationBatch / MutationReceipt`，无法证明preview与commit使用同一来源、同一政策和同一generation，也无法在100K/1M候选下给出延迟、分配、降级与结果上限。

目标不应是继续给`selectable_owners_in_rect`增加布尔参数。正确边界是：Editor73拥有qualified gesture/session、shape与policy、preview product、provider dispatch、query/commit parity和atomic selection batch；Editor59提供共享Selectable Spatial Product与input/capture terminal receipt；Editor70提供effective visibility/eligibility；Runtime/Graphics提供renderer picking/ID或CPU spatial query事实。透明选择走generation-qualified BVH/convex-volume product，呈现像素选择走renderer ID/picking product；两者都必须返回typed receipt，不允许静默切换语义。

本报告新增 **0项P0、24项P1、10项P2**，登记 **48个全部Fail的资格门**。Editor03/59/70父账继续唯一计数，不因本报告中的消费依赖而重复增加。

本轮是review-only：未修改production Rust/ZUI，未运行Cargo、真实Editor、GUI/GPU、native input、render golden、save/reopen、plugin reload、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon gesture/selection | **19 / 1,552 / 1,362 / 50,498 / 14** | drag session、input route、cancel、selection model/mutation与反馈 | `3c3ec9cce655f1bb1ca5ca77e8343cdfa4153bd9e6075b33d9b59bea6c9bd6af` |
| Zircon query/spatial | **21 / 2,127 / 1,941 / 73,576 / 24** | rect scan、pointer candidates、renderer-visible snapshot、projection与extract cache | `80448704072a486bd62fa10a52535ca292e4dff91506fb1be76e7422bbfe5729` |
| Zircon host/focused tests | **11 / 2,855 / 2,603 / 99,268 / 40** | pointer dispatch、event effect、editing viewport与selection integration | `eb064404eadb265e95ef9aa8786b3ed1047316624b6f00b376fa939f18d6a345` |
| Zircon runtime spatial | **4 / 668 / 599 / 23,508 / 8** | visible spatial storage/query与graphics-side implementation | `d0a9b336a39f9de08cc6b2611264c1861dca8c87e1464673ad3000ecb5fc83a1` |
| Zircon deduplicated set | **55 / 7,202 / 6,505 / 246,850 / 86** | 四组按规范化路径去重 | `5617a137622478ca27584d18f5b8a3b03952fa1fb4387458c4b2af347b2ec379` |
| Unreal selected set | **12 / 2,016 / 1,679 / 67,192 / 0** | legacy Box/Frustum与ITF Marquee/Frustum interaction | `bb1f07f00fea158f96838a5a5e24c6d3a1a0bb7fdedd680e6a8be3c426a9a0fa` |
| Godot selected set | **6 / 14,062 / 11,889 / 553,908 / 0** | region gesture、selection frustum、DynamicBVH与gizmo/subgizmo query | `99fc1144f3eae5f62fa30dc6cfa4fc08c786dacbea180015f55c884e4f77fcb3` |
| Fyrox selected set | **6 / 2,139 / 1,983 / 82,567 / 0** | dedicated select mode、selection frame、world bounds与command commit | `fd031d722407a3079e410d16fb146f3b143f97911686727871b975841ba46d3f` |
| Bevy selected set | **6 / 3,494 / 3,209 / 138,543 / 8** | pointer identity、backend hit merge、ray viewport mapping与cancel lifecycle | `e750bfa03ce032d04ed212cb5a85e131bfbe8ded98f963c550e92db4bba4927a` |
| Unity Graphics selected set | **3 / 4,203 / 3,548 / 192,487 / 0** | Picking/SelectionOutline editor view、filter与GPU culling integration | `8fe31f5f64abb710f95cdf96638935723848f3fce135457f6796bdd03b4ef0f5` |
| Five-engine deduplicated set | **33 / 25,914 / 22,308 / 1,034,697 / 8** | 五类本地参考按路径去重 | `2705de8213233d7abbe94c52957ec86371f7be5b4ab958d9952e2e6df2a67d45` |

指标是本轮工作树的文本物理统计，不是功能覆盖率。fingerprint按规范化小写相对路径排序后，对`path + NUL + lowercase(file SHA-256) + LF`清单取SHA-256。55个Zircon文件中24个已有其他会话修改，本报告审查当前工作树而非仅审查HEAD；实现前必须重取终态并复核fingerprint。

### 2.2 本报告拥有与不拥有的范围

本报告拥有Scene Viewport Region Selection产品：gesture/session、marquee/lasso visual、window/crossing/visibility/target/mutation policy、preview、query request/receipt、provider dispatch、preview/commit parity、atomic batch commit、设置/能力、诊断/预算及产品资格。

本报告不拥有通用pointer/capture、点选、基础selection data model、renderer-visible spatial snapshot、scene visibility/lock/isolate、真实几何bounds、near-plane裁剪或通用transaction/history。它们分别由Editor03、Editor59、Editor70、Editor63及Runtime/Graphics父报告拥有。Editor73只能声明消费合同和集成门，不复制第二套authority。

### 2.3 Current-source更正

| 旧结论/风险 | 当前工作树事实 | 本报告处理 |
|---|---|---|
| Pointer Cancel被直接吞掉 | 已映射为`CancelInteraction`并进入controller cancel path | Editor59旧P0需要终态复核；Editor73不重复登记 |
| 第二按键可覆盖primary drag并误提交 | right/middle press在已有drag时返回，release只清对应Orbit/Pan | Editor59旧P0路径当前已阻断；不重复登记 |
| stale/preparing renderer product仍用于点选 | 当前点选拒绝并请求render rebuild | 保留该基础；Region query尚未消费，登记集成差距而非重报点选问题 |
| 框选完全不存在 | release-time矩形扫描与Replace/Extend/Toggle真实可执行 | 以“最小链路存在、产品合同缺失”为基线 |

当前更正不等于父报告自动关闭；父owner仍需在其冻结语料和动态测试下复核状态。

### 2.4 关键源码锚点

| 证据 | 关键路径 |
|---|---|
| Zircon gesture route | `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs`；`zircon_editor/src/scene/viewport/controller/viewport_drag_session.rs`；`zircon_editor/src/scene/viewport/interaction/viewport_input.rs` |
| Zircon release/query | `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_selection.rs`；`zircon_editor/src/scene/viewport/pointer/overlay_router/selectable_owners_in_rect.rs` |
| Zircon cancel/mutation | `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_interaction_cancel.rs`；`zircon_editor/src/scene/selection/selection_model.rs`；`zircon_editor/src/scene/selection/domain_selection.rs` |
| Unreal legacy | `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorDragTools.h`；`Private/EditorDragTools.cpp`；`Public/DragTool_BoxSelect.h`；`Private/DragTool_BoxSelect.cpp`；`Public/DragTool_FrustumSelect.h`；`Private/DragTool_FrustumSelect.cpp` |
| Unreal ITF | `dev/UnrealEngine/Engine/Source/Editor/Experimental/EditorInteractiveToolsFramework/Public/EditorDragTools/{DragToolInteraction,MarqueeSelectInteraction,BoxSelectInteraction,FrustumSelectInteraction}.h`及对应`Private/EditorDragTools/*.cpp` |
| Godot | `dev/godot/editor/scene/3d/node_3d_editor_plugin.{h,cpp}`；`dev/godot/editor/scene/3d/node_3d_editor_gizmos.{h,cpp}` |
| Fyrox | `dev/Fyrox/editor/src/interaction/select_mode.rs`；`dev/Fyrox/editor/src/world/selection.rs`；`dev/Fyrox/editor/src/scene_viewer/mod.rs` |
| Bevy | `dev/bevy/crates/bevy_picking/src/{backend,pointer,hover,events}.rs`；`dev/bevy/crates/bevy_picking/src/mesh_picking/ray_cast/mod.rs` |
| Unity Graphics | `dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.cs`；`Runtime/GPUDriven/Culling/{InstanceCuller,InstanceOcclusionCuller}.cs` |

大括号路径是同目录文件集合的简写；冻结fingerprint仍以实际逐文件路径计算。Unity证据只覆盖本地Graphics仓库，不声称审查了完整Unity Editor源码。

## 3. 当前产品链与可保留基础

### 3.1 当前链路

```text
Primary press
  -> PrimarySelection { start, current, active=false, target, mutation }
  -> move: distance >= 4px => active=true; update current
  -> release(active)
       -> selectable_owners_in_rect(start, current)
          -> normalize rect
          -> scan renderable candidate proxy circles
          -> scan gizmo pick shapes
          -> IndexSet<NodeId> -> Vec<NodeId>
       -> select_nodes(scene, ids, mutation)
          -> Replace | Extend | per-id Toggle
```

该链路没有独立的gesture id、query generation、preview generation或commit receipt。`ViewportFeedback`在active region move期间仍为空，Host event effect不会要求presentation重绘。

### 3.2 可保留基础

| 基础 | 保留理由 | 必须收束的边界 |
|---|---|---|
| `ViewportDragSession`枚举 | 已把navigation、handle与primary selection放入显式interaction状态 | PrimarySelection需替换为qualified region session，不继续堆字段 |
| `SelectionMutation`与`SelectionModel` | Replace/Extend/Toggle语义和active domain已有测试 | 扩展为policy + atomic batch，不能逐实体改变revision |
| overlay router与projection | 已能投影候选并合并owner | 只作provider/adapter；空间和可见性truth来自父产品 |
| renderer-visible spatial snapshot | 点选已有generation-qualified source、stats与stale fail-close | Region必须消费共享产品，不能再走无receipt旁路 |
| runtime visible spatial query接口 | 为CPU broad phase和共享索引提供真实底座 | 需支持region volume/policy/result receipt，不在Editor复制索引 |
| Host event effect/invalidation | 已有RenderChanged/PresentationChanged投影机制 | gesture preview必须产生明确effect与coalesced redraw |
| SceneMode input/effect底座 | mode可参与viewport input和side effect | 增加RegionSelectionProvider，而不是由mode重新实现整条手势 |

## 4. 新增P0：0项

本轮没有发现需要新建且未被父报告拥有的不可逆数据损坏、崩溃、权限突破或启动阻断。拖动期间world/view替换可能提交wrong-target selection，但选择本身可逆且当前未发现持久化数据被直接破坏，因此按P1登记。不能为了强调工程目标而人为抬高severity。

## 5. 新增P1：24项

### P1-01：Active Region Drag没有Marquee视觉，也不会请求Presentation刷新

移动只更新`current/active`，`ViewportFeedback`保持为空。产品没有填充、边框、方向/模式样式、DPI缩放或退化矩形提示；Host也没有因move产生RenderChanged/PresentationChanged。用户无法确认起点、范围和当前工具状态。Unreal ITF按Window/Crossing画实线/虚线，Godot与Fyrox都在拖动时显示selection frame。

### P1-02：拖动期间没有候选Preview与增量Highlight产品

查询只在release执行，拖动中不发布candidate set、added/removed diff、query pending/degraded或count。用户无法在commit前判断结果，后续即使添加临时矩形也仍缺“将选中谁”的核心反馈。Preview必须是generation-qualified产品，并由Editor59 highlight owner消费，不能另建第二套高亮authority。

### P1-03：PrimarySelection Session缺少完整身份与显式Phase

session仅保存屏幕点、布尔active、press-time target与mutation，没有`GestureId`、document/world/viewport/view/frame、pointer/capture generation、source product或eligibility revision；也没有Armed/Previewing/Querying/Committing/Accepted/Cancelled/OwnerLost等phase。相同数值NodeId或重建后的surface可被错误解释为同一交互上下文。

### P1-04：Release没有对World/View/Source Currentness做Two-phase Revalidation

bridge与候选在press前同步，但region move/release不重新校验scene、world、view、camera、frame或spatial generation。外部open/reload/play/layout/view变化可让旧代理命中新scene中的同值ID。提交前必须验证gesture snapshot、query receipt和current selection authority；失败应终态cancel/stale，而不是静默筛选“ID是否存在”。

### P1-05：固定4px阈值没有DPI、设备、滞回与退化手势政策

`PRIMARY_NAV_THRESHOLD`是固定逻辑像素/屏幕距离，没有DPI scale、mouse/pen/touch profile、press slop、hysteresis、minimum region extent或1px退化处理。HiDPI和不同输入设备下会产生不同物理手感；极窄region也没有明确转点选、扩张或取消政策。

### P1-06：Mutation只支持Replace/Extend/Toggle，缺少完整Policy与Subtract

当前Ctrl=>Toggle、Shift=>Extend、否则Replace，缺Subtract、平台键位政策、mode override、target level、visibility semantics与稳定disabled reason。Region request应携带typed mutation policy；UI、shortcut、automation与provider都消费同一effective policy。

### P1-07：Mutation在Press时冻结，拖动中修饰键变化无反馈也无确定合同

用户开始拖动后改变Shift/Ctrl不会更新行为或视觉样式。产品既没有声明“press snapshot”还是“live modifiers”，也没有显示effective mutation。必须选定单一政策并测试modifier transition、键盘失焦、sticky key与平台差异。

### P1-08：没有Window/Crossing/AutoDirection选择模式及其视觉合同

矩形先取min/max，拖动方向随即丢失；所有候选都使用intersection。没有fully-contained Window、intersecting Crossing、left/right AutoDirection或solid/dashed cue。底层真实bounds/near-plane精度由Editor03/59拥有，本项只登记Region policy、direction preservation和用户可见语义。

### P1-09：没有PresentedPixels、TransparentVolume与Occlusion-qualified模式

当前region始终扫描静态代理，不能表达“只选屏幕上看见的像素”与“穿透选择frustum内对象”的差异，也没有明确fallback。Unreal区分opaque hit-proxy rectangle与transparent frustum；Unity Graphics把Picking/SelectionOutline作为显式editor view并进入culling/filter。Region policy必须选择source，能力不足时fail/degrade with receipt，不能静默换语义。

### P1-10：Region Shape被硬编码为矩形端点，无法演进Lasso/Polygon

API从`start/end`直接进入rect helper，没有`RegionSelectionShape`、坐标空间、点数/周长上限、简化误差、自交规则或finite validation。后续添加lasso若继续复制扫描循环，会形成第二套query/preview/commit路径。

### P1-11：没有Typed Query Request/Receipt、失败状态与统计

`selectable_owners_in_rect`只返回`Vec<NodeId>`，丢失request identity、source generation、policy、candidate/hit counts、latency、allocation、truncation、stale/degraded reason和provider contribution。点选已有query stats方向，Region却不能审计“为何选中/漏选”或证明currentness。

### P1-12：Region查询没有接入共享Selectable Spatial Product

点选已能使用renderer-visible generation-qualified snapshot，框选仍扫描retained renderable candidates与gizmo shapes。Editor59拥有共享spatial product本体；Editor73必须定义Region consumer adapter、source selection和receipt parity，禁止长期保留旁路。

### P1-13：没有SceneMode、Component Visualizer、Subobject/Subgizmo Provider协议

通用扫描只把mesh/gizmo owner折叠为NodeId。SceneMode可以消费raw input，却不能向同一Region request提供自定义候选、subobject、component handle、admission或commit target。Godot提供gizmo/subgizmo frustum query，Unreal允许editor mode与component visualizer override；Zircon需要owner-qualified provider registry与fault boundary。

### P1-14：Preview与Commit没有同源、同Policy、同Generation保证

当前没有preview，未来若每move临时扫描、release再次扫描，场景变化、provider顺序或不同source会导致“看到A、提交B”。必须冻结或显式重验证query snapshot，并让commit引用accepted `QueryReceiptId`；任何重新查询都要产生可见的changed/stale disposition。

### P1-15：Toggle多选按实体逐项改变Generation/Revision，不是原子手势

`SelectionModel::apply_active`对Toggle逐个调用`toggle_active`。N个候选会产生N次domain generation与model revision，observer可能看到中间集合，失败/取消也没有单一回滚边界。一个Region gesture必须生成一个`SelectionMutationBatch`、一次authority commit和一个receipt。

### P1-16：结果Primary与Order依赖Mesh/Gizmo遍历顺序

候选通过`IndexSet`按发现顺序去重，发现顺序来自renderable与gizmo遍历；不同extract/provider顺序会改变primary和selection order。产品没有stable sort key、primary preservation或nearest/previous-primary policy。相同world、shape与policy必须给出可复现结果。

### P1-17：选择提交是隐式Side Effect，没有Operation/Journal/Observer Receipt

pointer release直接调用`select_nodes`。没有region operation identity、precondition、before/after set、mutation source、history grouping、selection-changed event payload或automation correlation。Selection是否进入undo history可由产品政策决定，但观测与原子receipt不能缺席；Editor63拥有通用transaction，不在此复制。

### P1-18：大场景查询没有Budget、Deadline、Result Cap或异步调度

release事件路径同步O(N)扫描、投影并分配结果，候选数没有上限，也没有time slice、deadline、cancel token、priority或100K/1M admission。长查询会直接阻塞UI，且无法在pointer release后给出pending/accepted terminal状态。

### P1-19：Preview Query没有Coalescing、Backpressure与取消政策

工程级preview需要随pointer move更新，但当前没有用于合并旧move、取消过期generation、限制in-flight query或丢弃迟到结果的协议。直接在每个move同步全扫会把输入频率放大成CPU/GPU压力。应按gesture generation只保留有限in-flight工作，并明确latest-wins与commit barrier。

### P1-20：临时集合与投影扫描没有Reusable Scratch和规模内存合同

每次release创建`IndexSet`并收集`Vec`，逐候选执行投影/圆相交。底层索引建设由Editor59/Runtime owner负责；Region产品仍需声明scratch arena、tile/readback buffer、result memory cap和allocation telemetry，避免lasso/preview接入后高频堆分配。

### P1-21：没有可持久化Region Selection Profile与Capability Negotiation

阈值、Window/Crossing、透明/遮挡、target level、preview、lasso简化、颜色/线型、最大结果与性能质量都没有user/project/view scope配置。产品也不能表达renderer ID buffer、CPU BVH、subobject provider或lasso capability是否可用。

### P1-22：没有Canonical Command与无障碍/自动化入口

Region selection只能由低级pointer stream触发。没有“开始/更新/接受/取消region selection”的qualified command/request，没有键盘可达的模式切换、screen reader状态、候选计数announcement或测试驱动的canonical invocation。合成鼠标事件不能替代稳定产品API。

### P1-23：没有Region-specific Telemetry、诊断与可复现Receipt

系统不记录gesture-to-first-preview、preview query、release-to-commit、candidate/provider counts、source generation、stale reject、allocation、truncation和degradation。性能或漏选投诉只能靠日志猜测，无法比较CPU frustum与GPU picking路径，也无法建立跨引擎同语义benchmark。

### P1-24：Provider Reload/Fault与Owner Loss没有隔离和确定终态

未来mode/component/subobject provider接入后，当前session没有owner lease、provider generation、panic/timeout/oversize隔离或revocation disposition。document/world/view/surface关闭、plugin reload、capture loss与query source retirement也没有Region-specific terminal receipt。所有路径必须恰好一次Accepted/Cancelled/Stale/OwnerLost/Faulted并清除preview。

## 6. 新增P2：10项

### P2-01：现有专项测试只覆盖最小Happy Path

当前测试主要证明cube rect hit、Extend和Toggle，没有覆盖完整gesture/session/product合同。

### P2-02：没有Marquee/Preview视觉Golden与Screenshot资格

缺少DPI、主题、Window/Crossing线型、active/degraded/stale状态和长时间拖动的真实Host截图证据。

### P2-03：没有World/View/Frame变化期间的Currentness测试

缺open/reload/play、camera move、resize、surface rebuild、world generation rollover后release的fail-close用例。

### P2-04：没有Window/Crossing/透明/遮挡/严格包含Golden

缺部分相交、完全包含、behind-camera、near-plane、occluded、hidden/locked与multi-mesh场景的同政策对照。

### P2-05：没有Subobject/Gizmo/Provider组合测试

缺scene mode override、component visualizer、subgizmo、provider revoke/fault、target-level混合结果与owner aggregation用例。

### P2-06：没有Atomic Batch与Stable Order测试

缺1000对象Toggle只增加一次revision、observer只见before/after、primary/order确定和失败不发布中间态的测试。

### P2-07：没有Cancel/Modifier/Boundary输入矩阵

缺escape、pointer cancel、capture loss、release outside、resize、focus loss、modifier transition、pen/touch与degenerate gesture。

### P2-08：没有Rect/Frustum/Lasso数值性质与Fuzz

缺NaN/Inf、巨大坐标、反向拖动、零面积、多边形自交、简化误差、orthographic/perspective投影的property tests。

### P2-09：没有100K/1M Candidate Benchmark与Soak

缺preview频率、CPU/GPU/readback、allocation、result cap、cancel latency、长时间drag和多viewport并发预算证据。

### P2-10：没有Query/Mutation Receipt的诊断与回放测试

缺typed receipt schema、journal correlation、stale/degraded/fault解释、自动化重放和跨版本兼容测试。

## 7. 五引擎参考结论

| 参考 | 可采用的工程事实 | 不应照搬/证据限制 |
|---|---|---|
| Unreal | click-drag capture lifecycle；Window/Crossing/CrossLeft/CrossRight；solid/dashed marquee；DPI；四角反投影frustum；opaque hit-proxy与transparent frustum；typed element、mode与visualizer routing；transactional include/exclude | legacy Box/Frustum重复实现、全局Editor/UObject结构不适合直接移植；本轮只审查所列本地文件 |
| Godot | region begin/end、8*EDSCALE threshold、可见selection rectangle、六平面frustum、DynamicBVH convex query、gizmo/subgizmo精确query、lock/ownership/group policy | monolithic viewport singleton与节点特化控制流不应成为Zircon边界 |
| Fyrox | 独立Select interaction mode、visible Border、坐标归一化、world bounding box投影、`ChangeSelectionCommand`、Scene/UI selection分离 | 仍是O(N)和简单intersection/replace，只能证明最低产品闭环，不能作为性能终态 |
| Bevy | PointerId/Location/render target identity；多backend `PointerHits` merge/order/depth；typed HitData；block-lower semantics；cancel清理与viewport/DPI ray mapping | 没有Editor marquee产品，不能用作Region parity证明，只采用底层identity/backend组合思想 |
| Unity Graphics | Picking/SelectionOutline/Filtering editor view type；picking material；include/exclude filter；hidden/prefab/pickability进入culling；jobified shared render data与view id | 本地`dev/Graphics`不是完整Unity Editor Scene View marquee源码，只能证明renderer/culling integration方向 |

### 7.1 综合判断

成熟实现的共同点不是“能框出多个对象”，而是可见、可解释、可取消的gesture；明确的Window/Crossing/occlusion policy；真实空间或renderer product；extension/subobject参与；统一selection commit；以及规模化query lifecycle。Zircon当前仅覆盖最后release时的最小相交扫描，尚未形成这些产品合同。

## 8. 目标架构

### 8.1 Qualified Gesture与Policy

```rust
struct RegionSelectionGestureId(u64);

struct RegionSelectionSessionKey {
    document: DocumentSessionId,
    world: WorldGeneration,
    viewport: ViewportSessionId,
    view: ViewGeneration,
    pointer: PointerCaptureGeneration,
    gesture: RegionSelectionGestureId,
}

enum RegionSelectionShape {
    Rect { start: ScreenPoint, end: ScreenPoint },
    Polygon { points: BoundedScreenPolyline, simplification: PolygonTolerance },
}

struct RegionSelectionPolicy {
    coverage: CoveragePolicy,       // Window | Crossing | AutoDirection
    visibility: VisibilityPolicy,   // PresentedPixels | TransparentVolume
    target: SelectionTargetLevel,   // Entity | Component | Subobject
    mutation: SelectionMutation,    // Replace | Extend | Subtract | Toggle
    eligibility_revision: EligibilityRevision,
}
```

Session只持有identity、phase、shape、effective policy和receipt引用，不复制world、selection或renderer authority。Modifier政策必须明确是live还是press snapshot；默认建议live effective policy配合视觉样式更新，但commit仍引用最后一次accepted preview/query receipt。

### 8.2 Query Planner与Shared Spatial Product

`RegionSelectionQueryPlanner`根据policy与capability选择source：

1. `PresentedPixels`使用renderer ID/picking buffer的有界tile readback或等价GPU结果。
2. `TransparentVolume`把rect/polygon构造成world convex volume，走generation-qualified CPU BVH/spatial product。
3. gizmo、mode、component visualizer和subobject通过同一provider registry贡献typed hit record。
4. source不支持请求语义时返回Unsupported/Degraded receipt；不得静默从occluded切到transparent。
5. 每个结果携带stable target address、owner/provider、coverage/visibility facts、sort key与source generation。

Editor59/Runtime owner实现共享spatial product；Editor73只拥有request planning、provider composition和Region receipt。

### 8.3 Preview Product

`RegionSelectionPreviewProduct`至少包含session key、shape/style、effective policy、query receipt id、candidate set/diff、count、pending/degraded/stale状态和presentation generation。Host按move coalesce更新marquee和highlight；旧generation迟到结果直接丢弃。Cancel、owner loss或commit terminal必须同generation清除preview。

### 8.4 Atomic Commit

Release先验证session、query receipt、eligibility和selection authority，再生成`SelectionMutationBatch`。Batch一次计算before/after、primary/order与stable diff，一次增加authority generation/revision，一次发布selection event和`SelectionMutationReceipt`。Toggle/Subtract不得逐实体向observer暴露中间集合。若产品选择不把selection进入undo history，也仍需journal/diagnostic receipt；若进入history则复用Editor63 transaction。

### 8.5 Performance与Failure政策

Preview latest-wins并有固定in-flight上限；move只更新shape/presentation，query按帧预算或时间窗coalesce。CPU路径使用共享BVH、convex query与reusable scratch；GPU路径使用有界tile/readback buffer和async fence。Request携带deadline、result/memory cap和cancel token；超限返回Truncated/Deferred/Rejected，不冻结UI。Provider有owner lease、generation、timeout/panic/result cap；fault只隔离该provider并进入receipt。

## 9. 分层里程碑

### ED73-M0：Currentness与RED Guards

- 固化drag期间world/view/source替换后release必须fail-close。
- 固化active move当前不刷新、无marquee的产品RED证据。
- 固化mass Toggle多次revision与遍历顺序不稳定。

### ED73-M1：Identity、Shape与Policy Schema

- 建立qualified session key、phase machine、Rect/Polygon shape和finite/bounded validation。
- 建立coverage/visibility/target/mutation policy与modifier resolver。
- 明确DPI/device threshold、degenerate gesture和terminal disposition。

### ED73-M2：Marquee与Preview Presentation

- 建立DPI/theme/a11y-aware marquee style和Window/Crossing cue。
- active move产生coalesced PresentationChanged并发布preview lifecycle。
- 接Editor59 highlight product，不复制highlight authority。

### ED73-M3：Typed Query/Receipt与Shared Product接入

- 建立request/hit/receipt/stats/error schema。
- Region消费Editor59/Runtime共享spatial product，删除retained O(N)产品旁路。
- 完成source generation、stale/degraded/currentness门。

### ED73-M4：Window/Crossing与Presented/Transparent语义

- CPU convex volume实现transparent query，GPU picking实现presented pixels。
- 完成Window/Crossing/AutoDirection和strict containment。
- 依赖Editor70统一visibility/eligibility，不复制hidden/lock policy。

### ED73-M5：Provider、Subobject与Lasso

- 建立owner-qualified RegionSelectionProvider registry和target address。
- 接SceneMode、component visualizer、gizmo/subgizmo与plugin revoke/fault。
- 在同一query/preview/commit路径加入bounded/simplified lasso。

### ED73-M6：Atomic Selection Mutation

- 建立stable result order、primary policy、before/after diff与batch commit。
- Replace/Extend/Subtract/Toggle均一次revision、一次event、一次receipt。
- 接Editor63可选history/journal，不在pointer handler隐式修改。

### ED73-M7：Budget、Async与Backpressure

- 建立deadline、result/memory cap、cancel、latest-wins和commit barrier。
- CPU/GPU query使用reusable scratch/readback并记录分配与延迟。
- 完成100K/1M、multi-view和长drag benchmark。

### ED73-M8：Profile、Capability与Product

- 持久化user/project/view Region profile并支持schema migration/LKG。
- 建立真实toolbar/menu/shortcut/command、disabled reason、tooltip和screen reader状态。
- 自动化通过canonical request而非仅合成pointer stream。

### ED73-M9：Fault、Soak与跨引擎资格

- 完成world/view/surface loss、plugin reload、provider fault、GPU readback failure矩阵。
- 完成GUI/GPU golden、native input、save/reopen、locale/HiDPI/a11y和soak/profile。
- 与Unreal/Godot建立同语义Window/Crossing/transparent/presented benchmark；48门全Pass后才提升实现状态。

## 10. 资格门

| Gate | 要求 | 当前 |
|---|---|---|
| ED73-G01 | Region Selection拥有唯一Editor产品authority | Fail |
| ED73-G02 | 每次gesture有qualified stable identity | Fail |
| ED73-G03 | session绑定document/world/viewport/view/pointer generation | Fail |
| ED73-G04 | gesture phase与terminal disposition完整且恰好一次 | Fail |
| ED73-G05 | Rect/Polygon shape有finite、bounded、坐标空间合同 | Fail |
| ED73-G06 | threshold按DPI/device profile解析 | Fail |
| ED73-G07 | degenerate region与point selection转换政策明确 | Fail |
| ED73-G08 | modifier press/live语义确定且可见 | Fail |
| ED73-G09 | active drag有DPI/theme-aware marquee | Fail |
| ED73-G10 | Window/Crossing/AutoDirection有不同视觉cue | Fail |
| ED73-G11 | active move触发coalesced presentation invalidation | Fail |
| ED73-G12 | preview发布candidate set/diff/count/status | Fail |
| ED73-G13 | preview与highlight共享父owner产品 | Fail |
| ED73-G14 | cancel/owner loss/commit原子清除preview | Fail |
| ED73-G15 | query request携带shape/policy/source/currentness | Fail |
| ED73-G16 | query receipt携带generation/stats/error/degradation | Fail |
| ED73-G17 | Region消费共享Selectable Spatial Product | Fail |
| ED73-G18 | stale/preparing/retired source一律fail-close | Fail |
| ED73-G19 | Window使用完整包含语义 | Fail |
| ED73-G20 | Crossing使用相交语义 | Fail |
| ED73-G21 | AutoDirection保留drag direction并稳定解析 | Fail |
| ED73-G22 | PresentedPixels由renderer picking/ID product回答 | Fail |
| ED73-G23 | TransparentVolume由generation-qualified spatial query回答 | Fail |
| ED73-G24 | 不支持的source/policy不静默换语义 | Fail |
| ED73-G25 | visibility/eligibility只消费Editor70 effective snapshot | Fail |
| ED73-G26 | provider registry有stable owner/type/generation | Fail |
| ED73-G27 | SceneMode/visualizer/gizmo/subobject走同一request | Fail |
| ED73-G28 | target address支持Entity/Component/Subobject | Fail |
| ED73-G29 | provider panic/timeout/oversize/revoke被隔离 | Fail |
| ED73-G30 | lasso与rect共享query/preview/commit产品 | Fail |
| ED73-G31 | preview与commit引用同一accepted query receipt | Fail |
| ED73-G32 | requery导致变化时有明确stale/changed disposition | Fail |
| ED73-G33 | Replace/Extend/Subtract/Toggle全部typed且确定 | Fail |
| ED73-G34 | 一个gesture只提交一次selection generation/revision | Fail |
| ED73-G35 | observer只看见atomic before/after集合 | Fail |
| ED73-G36 | result order与primary政策跨遍历稳定 | Fail |
| ED73-G37 | mutation产生typed receipt与journal correlation | Fail |
| ED73-G38 | query有deadline/cancel/result/memory cap | Fail |
| ED73-G39 | preview latest-wins且in-flight有界 | Fail |
| ED73-G40 | CPU/GPU路径有reusable scratch/readback预算 | Fail |
| ED73-G41 | 超限返回typed defer/truncate/reject状态 | Fail |
| ED73-G42 | Region profile有scope/schema/migration/LKG | Fail |
| ED73-G43 | capability snapshot决定可用模式并给disabled reason | Fail |
| ED73-G44 | command/keyboard/automation可canonical驱动与取消 | Fail |
| ED73-G45 | screen reader获得mode/count/status/terminal反馈 | Fail |
| ED73-G46 | latency/count/allocation/stale/degraded telemetry完整 | Fail |
| ED73-G47 | 100K/1M、fault/soak/GUI/GPU/native input矩阵通过 | Fail |
| ED73-G48 | 同语义跨引擎benchmark有可复现receipt | Fail |

## 11. 测试与动态证据矩阵

| 层级 | 必须新增的证据 |
|---|---|
| Pure model | session phase、identity、DPI threshold、modifier resolver、shape validation、coverage/mutation policy |
| Geometry | perspective/orthographic frustum、contain/intersect、near-plane、polygon/lasso finite/property/fuzz |
| Currentness | world/view/frame/source/eligibility rollover、surface rebuild、late query、owner loss |
| Presentation | marquee fill/stroke、solid/dashed、preview diff、pending/degraded/stale、DPI/theme/a11y golden |
| Provider | mode/visualizer/gizmo/subobject、mixed target、revoke/panic/timeout/oversize、stable ordering |
| Mutation | all four mutations、atomic revision/event、primary/order、observer、journal/history integration |
| Input | mouse/pen/touch、modifier transition、escape/cancel/capture loss、release outside、resize/focus loss |
| Renderer/spatial | presented pixel vs transparent volume、occlusion、visibility/eligibility、GPU failure/fallback receipt |
| Performance | 100K/1M candidates、high-frequency move、CPU/GPU/readback、allocation、cancel latency、multi-view |
| Product | 真实Editor/native input、command/keyboard/automation、save/reopen、locale/HiDPI/screen reader |
| Comparative | Unreal/Godot同场景Window/Crossing、opaque/transparent、subobject和大场景query receipt |

当前没有执行上述动态矩阵。静态cube rect hit、Extend/Toggle unit test或源文本存在不能把任何Gate改成Pass。

## 12. Owner路由与禁止重复实现

| 责任 | Canonical owner | Editor73只能做什么 |
|---|---|---|
| selection model、mode/gizmo/picking总账 | Editor03 | 提交Region atomic mutation需求，不重复其几何/eligibility finding |
| input/capture/point picking/shared spatial/highlight | Editor59 | 消费qualified terminal、spatial与highlight receipt，不复制产品 |
| object visibility/hidden/lock/isolate/eligibility | Editor70 | 消费effective snapshot/revision，不另写filter |
| document/world lifecycle | Editor61 | 绑定session generation并在替换时终态化 |
| transaction/history/journal | Editor63 | 提交可选batch command/receipt，不建私有history |
| multi-view session/currentness | Editor58/72 | 绑定具体viewport/view/slot，不使用全局current viewport |
| runtime spatial/render picking | Runtime/Graphics相关owner | 只请求neutral query/product，不把Editor policy下沉为第二authority |

禁止用以下临时方案关闭本报告：只画一个矩形但不发布preview；每move同步全扫候选；继续以`Vec<NodeId>`作为无receipt结果；复制点选与框选两套空间索引；用proxy transform/scale冒充所有真实bounds；把Window/Crossing写成调用点布尔值；lasso复制第三套query；provider直接修改SelectionModel；Toggle逐实体递增revision；release重新扫描却不检查preview parity；source不可用时静默退回另一语义；用合成鼠标和小cube test代替真实GUI/GPU/规模资格。

## 13. 状态与产出记录

- 审查状态：`complete`，仅表示本轮current-source差距建账完成。
- 实现状态：`not_started`。
- 新增finding：`0 P0 / 24 P1 / 10 P2`。
- 资格门：`0 Pass / 48 Fail`。
- 建议首个实施点：ED73-M0，先用RED tests固定currentness、无presentation invalidation与mass Toggle非原子，再进入M1身份/shape/policy；不得先在overlay里孤立画框。
- 实施前置：重取55个Zircon文件、Editor03/58/59/61/63/70/72及Runtime spatial/Graphics父owner终态，重新冻结working-tree fingerprint。
- 验证声明：本轮未运行Cargo与动态产品验证，不能宣称功能、性能、表现、无障碍、插件安全或跨平台已达到目标。

## 14. 最终判断

当前Zircon Region Selection不是“缺一个Lasso按钮”，而是release-time helper与完整交互产品之间存在结构断层。已有drag enum、selection model、projection/overlay、renderer-visible snapshot和runtime spatial query是可保留积木；无marquee/preview、无qualified session、无policy/request/receipt、旁路O(N)扫描、逐实体Toggle和无预算/终态则是必须替换的临时实现。

正确路线是先建立currentness与gesture state machine，再完成visible preview和typed query，随后让Window/Crossing、presented/transparent、provider/subobject与lasso共享同一spatial/renderer产品，最后以atomic batch、profile/capability、规模/fault与跨引擎资格收口。只有48个资格门全部通过，Region Selection才可从“release时能得到一组NodeId”提升为接近Unreal/Godot成熟度、并可继续追求超越目标的工程级产品。
