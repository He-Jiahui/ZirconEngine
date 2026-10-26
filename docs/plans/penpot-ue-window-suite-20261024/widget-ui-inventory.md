# Widget/UI 源码窗口盘点与能力差异

Hierarchy 当前四路径修订已接真实 document.root 的 Wrap/单子项 Unwrap：完整原文档上先预留 node/control ID，再更新原根及原 v2 serializer/history/selection；原根和后代、绑定与组件引用保留。新增3项回归，原1项 root 可用性测试两条期望同步调整，原测试11326B前缀保持。四路径格式/guard/补丁反查通过，Rust执行仍0；原中断 Wrap identity 不扩写/重投。详见[实际实现与消费者复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/hierarchy-root-wrap-r02/review.md)。仅 document root 源码分支有后继；组件根/多选/named-slot、native enabled投影及正常EXE/几何/焦点/150%DPI/双端仍open。此前“root gap”记录保留为旧时点。

Hierarchy Wrap With 已修一个稳定身份缺陷：原实现先移除子树再分配容器 node/control ID，可能重用原节点或后代的身份；现在在完整原文档上先分配两项 ID，再走原 v2 写回、选中与单次 history。[本次两路径实现与复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/hierarchy-wrap-identity-r01/review.md)保留旧测试5736B前缀，新增两项 v2/绑定/组件引用/重开/undo/redo 回归，当前0执行。格式、guard、补丁反查通过；15依赖/保护输入及6旧身份与终态控制未变。UE root/multi-selection/named-slot、正常EXE/150%DPI/双端仍open；不改变97ID/6events，也不应用Palette clear公共候选。


Palette clear生产链新增精确发现：原v2已声明has_clear_action，但ViewTemplateNodeData缺字段、retained转换固定false，且原生pane只有Press没有clear配对Release。已形成[19路径完整未应用consumer候选r02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-clear-wiring-r01/review.md)，含typed source metadata→DTO/materializer/converter、原pointer/source/frame的Down/Up、原query/焦点链及5拟增回归；14额外公共路径只交Parent。32307B/SHA aaa219dd3178c74a66f8e7602e331d3b21544eaf9ac05932e66615e7eea07efd，19stdin fmt/apply-check0，未覆入源做guard/类型或执行。实际checkout仍为G09/actual-field r02的27项定义；候选32项不扩旧身份。IME/popup/ACK、公共left geometry与EXE/DPI/双端仍open，原Penpot rev29保持。

G09 当前源码能力新增实际字段资格复核：同一 active pane/source 仍须有原 control/edit/commit 身份的 enabled 查询控件，且来自实际 paint/hit 模型、几何有限且为正。原query/caret与selected/page受检链保持；这是两条既有消费者的源码修复，非正常GUI证据，未扩大UI运行时。 [当前字段资格r02静态回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-current-field-r02/review.md)固定73保护输入/6旧identity-control保持、guard 0/0/0及87源依赖指纹。格式/精确补丁检查通过，类型与27项测试未执行；实际EXE/150%DPI/官方导出/双端1DIP及整族passed/accepted仍false。此前25项和各旧revision只代表原时点。

本清单是源码盘点结果，不是窗口验收。已取得局部源样本和锚点设计的真实 Penpot 浏览器图，正常 EXE 图、官方完整 PNG 和双端测量仍缺。父会话与任务记录见 [widget-ui.md](widget-ui.md)。

当前Palette clear/focus/hide原生设计已保存至rev29并两次只读fresh重开；17Shared heads中5个为本次新增的SearchInput/Button真实linked copies，8local mains及原3496非根对象保持。[实际可编辑板/五图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-palette-clear-focus-r01/review.md)补充W02/G09设计输入；不证明产品过滤、clear、焦点或native隐藏链已执行。W02/G09下表已按37路径真实集成和4路径文本目标退休更新，早期“未找到过滤链路/提案未应用”不再作为当前事实。

## 参考身份

参考树：`E:/Git/ZirconEngine/dev/UnrealEngine`，只读。`Engine/Build/Build.version` 实际声明 `6.0.0`、`BranchName=UE5`、`Changelist=0`。`git -C dev/UnrealEngine rev-parse --show-toplevel` 返回 ZirconEngine 根，不能将其 HEAD 当作 UE 的 upstream revision。上游 revision 未确认；以本次具体源码 SHA-256 固定参考字节。

以下路径缩写只在本表使用：

- `UMG` = `dev/UnrealEngine/Engine/Source/Editor/UMGEditor`。
- `WBP` = `UMG/Private/BlueprintModes`。
- `Tabs` = `UMG/Private/TabFactory`。
- `WPreview` = `dev/UnrealEngine/Engine/Plugins/Editor/UMGWidgetPreview/Source/UMGWidgetPreview`。
- `ZUI` = `zircon_editor/assets/ui/editor/ui_asset_editor.zui`。

## 窗口与关联面板

“适用”说明该交互目标属于本窗口族，不代表已有后端或已完成复刻。“待定”保留具体产品决策；不得用简化后的同名占位面板算完成。

Parent [OVW 覆盖对账](parent-window-coverage.md) 的关联消费边界独立记录，不增加本域资产编辑器：OVW-37 Font/FontFace 文档和作者入口由资源会话 `01a16d53-60db-7d02-b03e-0042f277783d` 负责，本域消费字体/atlas/fallback/readiness。OVW-45 Editor Utility Blueprint/Utility Widget 由 Actor `01a16d53-8965-7472-931c-73cba748fe8e` 负责脚本/图/宿主操作合同，本域仅适配真实 Utility UI 内容；Parent 对接 host/VM/文档，完整后端未确认。W11 runtime Widget Graph 不替代该 editor utility 合同；Actor Timeline 子文档继续复用动画 owner 表面。上述字体、共享组件/导出和 Capture 均无新增交付或验收，公共写权/桌面时段未变化。

| 编号 | UE 窗口 / Tab | Toolkit / Factory / Slate 依据 | Zircon 适用性、理由和当前事实 |
| --- | --- | --- | --- |
| W01 | Widget Blueprint Designer / `SlatePreview` | `WBP/WidgetDesignerApplicationMode.cpp`；`Tabs/DesignerTabSummoner.cpp`；`UMG/Public/Designer/SDesignerView.h`、`UMG/Private/Designer/SDesignerView.cpp` | 适用。原 `DesignerPanel`、`DesignerCanvasPanel`、Select/ResizeSlot/PreviewInteract 与 `UiAssetEditorSession` 接续；画布渲染、选择手柄、撤销和原资产消费仍需 GUI 验证。 |
| W02 | Palette / `WidgetTemplates` | `Tabs/PaletteTabSummoner.cpp`；`UMG/Private/Palette/SPaletteView.cpp`、`SPaletteViewModel.cpp` | 适用。真实目录、完整 query、同源 snapshot/当前页、borrowed hit/frame 和注册窗口/active pane/source 已经由原37路径G09签认稿接到原Document guard/Session受检选择；另4条既有native路径退休hide/exit/blur/确认关闭的Palette文本目标。当前25项Palette、原4项builtin及相关34项Canvas定义均未执行。原资产97ID/6events保持；真实过滤、清空、IME/caret和正常EXE验收仍open，不能再以早期“未找到过滤链路”代替当前源码事实。 |
| W03 | Library / `WidgetLibrary` | `Tabs/LibraryTabSummoner.cpp` 创建 `SLibraryView`；实际实现位于 `UMG/Private/Library/SLibraryView.cpp` | 适用。复用 `.zui` 外部 widget/component 发现与引用链；需核实资产浏览、过滤、实例/引用插入能否以同一目录提供。不能复制 Palette 数据伪装为 Library。 |
| W04 | Hierarchy / `SlateHierarchy` | `Tabs/HierarchyTabSummoner.cpp`；`UMG/Private/Hierarchy/SHierarchyView.cpp`、`SHierarchyViewItem.cpp` | 适用。Zircon 已有树选择、插入、移动、改父、wrap/unwrap、undo stack；保留稳定 node/control ID。独立滚动、重命名、锁/可见性、拖放和上下文菜单仍需对齐。 |
| W05 | Bind Widgets / `BindWidget` | `Tabs/BindWidgetTabSummoner.cpp`；`UMG/Private/Widgets/SBindWidgetView.cpp`，包含原生 `FObjectProperty`、optional/bound/class 校验及 Goto Code Definition | 待定。UE 的原生类成员绑定与 `.zui` 的事件/属性绑定不是同一合同。候选是展示 Zircon 组件所需 props/slots/bindings 的满足情况；需要已有类型/源码导航 owner 确定模型。该待定不取消 W06 的绑定编辑要求。 |
| W06 | Details / `WidgetDetails`，Slot、Layout、Events、Navigation、Bindings | `Tabs/DetailsTabSummoner.cpp`；`UMG/Private/Details/SWidgetDetailsView.cpp`；`UMG/Private/Customizations/CanvasSlotCustomization.cpp`、`HorizontalAlignmentCustomization.cpp`、`VerticalAlignmentCustomization.cpp` | 适用。接续现有 widget prop/state、slot padding/size、layout semantic、binding event/route/payload 的真实 EditorManager 命令。UMG Canvas Min/Max anchors、offsets、alignment、stretch/rebase 与 Zircon overlay/flow slots 的语义差异必须明确，不把 overlay anchor 数字直接当作完整 UMG anchor 模型。 |
| W07 | Designer Tool Palette / `TopLeftTabID` | `Tabs/DesignerTabSummoner.cpp`；`UMG/Private/ToolPalette/WidgetEditorModeUILayer.cpp`；`UMG/Private/Designer/SDesignerToolBar.cpp`、`DesignerCommands.cpp` | 适用。Select/ResizeSlot/PreviewInteract 是现有实际工具。UE W/E 对应 Layout/Render Transform，G outlines、L respect locks；Zircon render transform、grid/rotation snap 等尚未核实，不用无效果按钮补齐。 |
| W08 | Navigation Simulation / `NavigationSimulation` | `Tabs/NavigationTabSummoner.cpp`；`UMG/Private/Navigation/SWidgetDesignerNavigation.cpp` | 适用。Zircon preview host 已使用 `UiNavigationDispatcher`；这不证明已有可视化导航模拟器。需提供真实 focus path、方向导航结果及从弹层返回的状态。 |
| W09 | Animations list / `Animations` | `Tabs/AnimationTabSummoner.cpp` 创建 `SUMGAnimationList`，含筛选、创建、删除、重命名和选中动画 | 适用，未确认 Zircon widget animation 文档模型/命令。共享 `WorkbenchTimelineStrip` 只能作为布局组件候选，不能宣称已具备动画编辑。 |
| W10 | Animation Sequencer drawer / `WidgetAnimSequencer`，独立曲线窗 `SequencerGraphEditor` | `WBP/WidgetDesignerApplicationMode.cpp`；`UMG/Private/WidgetBlueprintEditor.cpp` 的 `OnGetWidgetAnimSequencer`、drawer opened/dismissed、Sequencer；`UMG/Private/Animation/Sequencer2DTransformTrackEditor.cpp`、`MarginTrackEditor.cpp`、`UMGDetailKeyframeHandler.cpp` | 适用。目标包括轨道绑定、关键帧、scrub、播放/停止、Details keyframe、drawer 焦点及关闭返回。[本域消费者合同](widget-ui-animation-consumer.md) 已核实动画 owner 三个 main 身份，复用中性表面；Widget 保有属性/event 轨道、采样适配与源历史。外 Timeline wrapper/Curve ZUI、公共发布与 Widget 后端仍 open；不建立另一个 UI runtime。 |
| W11 | Widget Blueprint Graph / `Document` | `WBP/WidgetGraphApplicationMode.cpp`；`UMG/Private/WidgetBlueprintEditor.cpp`；继承 Kismet graph tab factories | 适用，当前 `UiAssetEditorMode` 仅 Design/Split/Source/Preview。必须复用已有图编辑/事件执行能力并确认 Widget 事件的存储与运行接口，不能把 Source 文本模式改名为 Graph。 |
| W12 | My Blueprint / Details / Palette in Graph | `WBP/WidgetGraphApplicationMode.cpp`；`dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp`、`BlueprintEditorModes.cpp` | 适用的事件/变量导航目标。Graph 激活会选择 Designer 选中的 widget 变量；需要保留跨模式选择并打开真实事件/函数。Blueprint 专用类型、K2 编译等见下文不适用项。 |
| W13 | Compiler Results / Find Results | Designer 与 Graph layouts；Kismet shared factories | 适用。编译/解析/资源错误可复用现有 structured diagnostics，并定位到真实源节点和行。Graph find 依赖实际 Graph 文档；不得用固定成功消息替代检查结果。 |
| W14 | Blueprint Preview / `WidgetPreview` | `WBP/WidgetPreviewApplicationMode.cpp`；`Tabs/PreviewTabSummoner.cpp`；`UMG/Private/Preview/SWidgetPreview.cpp` | 适用。UE `UMG.EnablePreviewMode` 初值 false；需按启用状态记录，不能称默认可见。Zircon 复用 `UiAssetPreviewHost::new_v2` / `UiV2SurfaceBuilder` 和真实资源，不使用截图作预览。 |
| W15 | Preview Detail View / `DebugDetails` | `Tabs/PreviewDetailsTabSummoner.cpp`；`UMG/Private/Preview/SPreviewDetails.cpp` | 适用的运行实例检查目标。需区分 source props 与 preview 实例状态，属性提交与重建策略要可验证。现有 preview mock 是可标注的设计测试输入，不是生产运行能力证明。 |
| W16 | 独立 Widget Preview toolkit 的 Viewport / Details | `WPreview/Private/WidgetPreviewToolkit.cpp`；`Widgets/SWidgetPreview.cpp`、`SWidgetPreviewViewport.cpp`、`SWidgetPreviewDetails.cpp` | 待定的独立资产/窗口形态。插件 `.uplugin` 声明 experimental、默认启用。Zircon 当前已有 Preview 模式；需决定是否复用该模式并保留目标，或在已有文档/资源系统增加独立 preview 配置。 |
| W17 | Preview Scene Settings / `WidgetPreviewToolkit_PreviewScene` | `WidgetPreviewToolkit.cpp` 注册；`SpawnTab_PreviewSceneSettings` | 待定。是独立 preview 的真实 scene 参数，不是简单背景色面板。需要确认世界空间 UI/预览场景是否为本轮产品能力；没有 backend 不得放假 3D 控件。 |
| W18 | Message Log / `WidgetPreviewToolkit_MessageLog`、preview status | `WidgetPreviewToolkit.cpp`；`Widgets/SWidgetPreviewStatus.cpp`；`WidgetPreviewCommands.cpp` | 适用的诊断与 Reset 操作目标，复用现有诊断/通知 owner；Reset 快捷键为 Ctrl+R，需要实际销毁/重建实例并验证状态变化。独立 toolkit 是否交付随 W16 决策。 |
| W19 | Widget Reflector | `UMG/Private/WidgetBlueprintEditorToolbar.cpp`；独立 preview toolbar | 关联共享工具。Zircon 已有 `ui/workbench/reflection/widget_reflector.rs`，需父工作台统一入口和身份，Widget 窗口只贡献自己的真实 runtime tree；不另建重复 Reflector。 |
| W20 | Graph Bookmarks | `dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/BlueprintEditorModes.cpp:101` 注册 `FBookmarksSummoner`；`BlueprintEditorTabFactories.cpp` 创建书签视图 | 适用的图导航目标，依赖 W11 的真实图文档、稳定节点定位和持久化。不能漏掉继承 factory 中默认布局未打开的面板。 |
| W21 | Replace Node References | 同一 base mode 注册 `FReplaceNodeReferencesSummoner`；`BlueprintEditorTabFactories.cpp` | 待定。需要 W11 的类型兼容、引用模型和可撤销替换事务；是否复用既有通用图替换界面由图编辑 owner 确认。Blueprint 反射类型本身不移植。 |

三个 Designer/Graph/Preview 模式以 `UMG/Private/WidgetBlueprintEditor.cpp:596` 的 `RegisterApplicationModes` 为入口。UMG 的 10 个 `*TabSummoner.cpp` 已逐一核对，包含 10 个具名 tab 和动画 drawer 身份；Graph 还包含继承的 core/blueprint factories，不能仅盘点 UMG 自己的 TabFactory。

## 明确不适用与交接

| 项 | 原因 / 处理 |
| --- | --- |
| UHT、UObject `BindWidget` / `BindWidgetAnim` 元数据和 K2/Blueprint 字节码编译器本体 | 不属于 Zircon `.zui v2` 与运行时接口；保留其视觉/操作目标，用既有 Zircon类型和命令映射。UI 绑定检查与事件图仍是 W05/W06/W11 的开放要求。 |
| Behavior Tree、Blackboard、AI 调试 | 用户将本会话限定 Widget/UI，属于其它独立会话；不沿用旧 overview 的合并分工。 |
| 全局菜单、Dock/tab 框架、主题设置、公共控件重新实现 | 父工作台 owner；提交具体依赖和差异，不创建窗口专用平行主题或 duplicate components。 |
| SCS Viewport / Class Defaults 的 Blueprint 专用 factory | `WidgetBlueprintApplicationMode` 向 base mode 传入 `bRegisterViewport=false`、`bRegisterDefaultsTab=false`，不属于本 UMG 模式的标准注册集合。基础 widget 属性仍由 W06 处理。 |
| 未启用插件的任意 extender 注入面板 | UMG 广播 `OnRegisterTabsForEditor` / `OnRegisterLayoutExtensions`；实际加载模块和扩展注册清单未获得，属于待核实扩展范围，不能宣称当前表已覆盖全部第三方动态注入窗口。 |

## 几何与操作目标

- Designer：横向主区左栏 0.15；其 Palette/Library 和 Hierarchy/Bind Widgets 两组上下各 0.5。其余 0.85 的上区 0.7 内含隐藏 tab well 的 Designer、默认关闭 Tool Palette、右侧 Details；下区 0.3 是默认关闭 Animations/Compiler Results。Navigation 可注册但不在默认 opened layout。
- Graph：左栏 0.15 的 My Blueprint/Details 上下各 0.5，中栏 0.70 的 Document/结果区为 0.8/0.2，右栏 Palette 0.15 默认关闭。切换模式保留真实 widget 选择并导航对应变量。
- Preview mode：Viewport 0.8 与 Details 0.15；不要把未归一化 coefficient 当作绝对屏幕像素。独立 Preview toolkit 则为左/右 0.85/0.15，左侧 Viewport/Message Log 0.85/0.15，右侧 Details/Scene Settings 共用 tab stack。
- Hierarchy：真实多选、展开、搜索、Rename、上下文菜单和剪切/复制/粘贴/删除按 UE command list 与事务核对。Designer/Hierarchy/Details 共享选择；输入中的快捷键不得穿透到画布。撤销要恢复源、选择、预览及外部效果。
- Designer：画布允许专用显式放置；普通 Palette/Details/工具栏继续 flow 布局。窄尺寸的具体 Dock/折叠方案待真实 Penpot 设计核对，不能靠隐藏关键操作过验收。三栏共用滚动与 UE 面板独立滚动的现状差异见 G03。

## 公共组件提案与开放差异

全部行当前都是 `open`；Penpot 局部设计图已取得，Editor 图和双端差异图仍缺，没有像素误差结论。

| ID | 参考 / 实际差异 / 已知原因 | 依赖与具体接续；待决策 |
| --- | --- | --- |
| G01 | Parent 已确认既有 Starship 用途映射；原资产仍只直接导入 editor_tokens。最小审查 diff 已具备；实际公开函数确认 source-owner visual-delta codec 当前拒绝 classes、imports、stylesheet 结构，不能把 paint 导出当共享继承。 | [候选增量与精确 API 边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/starship-class-adoption-r01.md)；仅 shell/真实 Designer 画布/toolbar，保留语义钩子并收窄局部规则。公共 owner 接续明确的结构迁移/正式导出支持；候选未应用，不改公共主题/catalog，不把现有三列套成 Dock。 |
| G02 | UiAssetEditorMode仍无Graph、v2仍无持久animation集合。原visible窄Host已在源码接入Session sample/finish和canonical Bool/Step sampler；17+3回归均未执行，同一真实clock、Stop/Cancel及retained/present仍无生产闭环。 | [原消费合同](widget-ui-animation-consumer.md)与[当前窄Host](widget-ui-preview-visibility.md)保留lease/source/frame/incarnation/sequence及恢复边界；不造第二clock/schema/player、不将sample查询或Published视为实际呈现，图/动画/Parent沿原owner接线。 |
| G03 | 当前 ZUI 的 LeftScrollRegion 将 Palette 与 Hierarchy 共滚；CenterScrollRegion 同时包含 Designer/ActionBar/Source；右侧 Inspector/Stylesheet 共滚。UE 使用独立 dock tabs 和面板。原页 revn9 已有三个尺寸的 opened Dock 目标，独立内容区仅为设计结构；900/640 的 tab/search 实际浏览器图仍裁剪或换行。 | Parent Dock/tab owner 提供独立 host slots、最小宽度/overflow/focus 合同；域内保持 UE opened 拓扑。640 左栈约95.7、900约134.7逻辑像素，见[逐图差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/widget-toolkit-design-review.md)。侧栏合栈仅另列候选，不自动折叠，不以源三个 ScrollableBox 或设计裁剪框宣称实际滚动一致。 |
| G04 | Canvas真实语义字段已接现有列表/编辑/删除/undo；16预设、popup ticket及r03缺省字段保全已有源码，Canvas叶22项测试0编译/0执行。原页rev22补pointer-leave保留焦点状态，旧8main保持；Overlay九点仍不等价Canvas。 | [当前设计/重基与原source限制](widget-ui-design.md#canvas-锚点的缺省源字段保全-r03)。单选/原生Canvas/正尺寸/auto-size关闭；Free/组件实例、拖动/吸附、共享popup/焦点及GUI仍open。现preferred-size API不替代Canvas offset resize；真实handle必须按UE StartingOffsets/方向/pivot及原输入事务接续。 |
| G05 | 现有预览 host 使用真实 UiV2SurfaceBuilder，但普通 EXE 中本窗口的 current-frame resource/paint/cursor 关联尚未实测。 | 复用 `capture-editor-normal-frame.py`、`record-editor-normal-window-evidence.py`、`zircon_editor_normal_frame_evidence.py` 和 `LayoutReviewCaseMessage`；先确认真实入口加载本源资产及依赖闭包。 |
| G06 | `workbench.extension.ui_asset_editor.open` 指向 `extension_ui_asset_editor_workspace.zui`，而完整 session 的 `editor.ui_asset` 指向原 ZUI。二者不能互相替代。 | 核对 `EditorManager::open_ui_asset_editor` 到正常 native window 的真实活动路由/资源选择；原 extension 的示例控件不计完整编辑能力。 |
| G07 | 局部源样本有真实ACK/关闭重开/正式ZUI导出；原页revn9另有七张完整边界目标。重试前确认普通Replace会remove旧局部板的6个owned主件，其中Button/Label已被七板直接引用325次；追加导入的cache仍按asset独立，不能称复用。 | 公共owner提供保留原main和source-owner的完整更新/安全追加合同，含qualified source/structure/closure检查、引用和rollback；[具体源码步骤与实际依赖](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/source-import-reuse-preflight.md)。本轮未执行Replace/重造main；canonical composite、编辑写回和正式全量导出继续pending，不detach补图。 |
| G08 | EXE/运行资源指纹和多尺寸/DPI/状态画面尚缺；历史 125% 的主窗口证据不能移用。 | 等待桌面排队时段，复用父会话真实构建而非并行重建产品；请求本窗口自己的 case/nonce/generation、资源 ready、GetDpiForWindow、成功 present 和完整捕获。 |
| G09 | 原r02已实际落实37路径（35既有+2新模块），finder不再重复改；三条Manager转发位于原Canvas完整后缀前。输入→同snapshot/当前页→当前frame→原guard/session checked链和Ready-empty shell已接入源码。后继4条native路径仅退休Palette文本目标，不重置query或发布结构，也不调用preview Stop/Cancel/finish。 | [当前实际集成](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/view-retained-integration-r03/review.md)与[文本目标退休](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-input-retirement-r01/review.md)的静态guard/fmt/diff通过；Palette25、builtin4、Canvas34均0执行，正式新request/operation尚无回执。真实无匹配→清空→切文档、旧row/page重显有效期、IME/caret/焦点/present继续open；旧Layout/Canvas/#383/#389身份和失败不扩写、不重投。 |
| G10 | 已接 Actor AB-WG-01～07。真实 PreviewHost 输入结果经 bool 入口到 presentation，尚未接通用 Route/Action consumer；pointer 汇总报告未保留原 binding ID/compiled handle，通用 consumer 又只返回 effects/字符串错误。Source outline 不等于精确 binding/callable/Graph 定位。 | [Widget 消费合同](widget-ui-graph-consumer.md) 列出来源传递、同次输入、去重、释放锁后执行、真实 record/error 和失效边界；Parent/Runtime/VM 按原 scope 接续。保留原 ID/target/payload；binding target telemetry、delivered、suggestions、Preview updated 均不关闭外部完成/导航或 GUI 验收。 |
| G11 | Widget最新保存/重开版本rev22/3314objects/8local mains，原12 Shared linked heads/99实例sourceRefs保持。Shared最新有版本回执rev5/4main/34colors+3typographies；Button已有1组真实surface.1 ref，r03只读完整data保持；旧rev0/style0是历史。 | [Shared消费/覆盖/reset](widget-ui-shared-consumer.md)与147个Toolkit背景显式色引用均已有设计证据，尚不证明七板全部语义组件、typed value槽、canonical sourceHash/正式导出或全局A/B通过。不复制共享main或平行theme，实际字体另见G14。 |
| G12 | 原完整导入曾因TreeRow paint missing→null回滚，0 ACK；13:20Z后实读公共源码已含nil归一及其它修正，worker产物已换为442bed70…，不能把旧失败说成新worker仍失败。 | 原提案/失败保留；没有本域新完整导入或公共聚焦测试终态。重试前另发现G07主件复用边界，已交精确步骤；不运行会删除已引用主件或重复建main的路径，不因源码变化关闭G12/G13。 |
| G13 | 源样本 Export Canvas PNG 等待 90 秒无 download；新锚点 board 的官方 SDK export 另返回 4 字节 `null`，signature `6e756c6c`，不是有效 PNG。 | 两项失败各自保留，原因尚未定位；Parent/原插件 owner 核查 SDK/worker/message/download 链。三张已复核浏览器图只证明实际设计，不替代官方导出或正常 EXE 证据。 |
| G14 | 原生Roboto400标题的CDP face为Roboto，与原注册engine WOFF Roboto-Regular不同；自然advance与既有Google Latin对照相同，差engine face 1.70526123046875。font-id、资源200及文档fonts loaded不足以证明字形就绪。 | [实际双字体对照](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/font-family-selection-r03/hypotheses.json)与[源/加载bundle](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/font-family-selection-r03/source-trace.json)定位custom/Google同family CSS竞争边界。Parent/public frontend owner接稳定内部family与HTML/SVG/position-data/codec；本域不改字号或字体系统，字体/EXE/1DIP门均open。 |

## 本轮实际检查

Python `tomllib` 确认原 ZUI 仍为 v2、97 nodes、97 control IDs 和原 6 条事件。局部 Penpot Inspector 将 Designer gap 从 4 改为 6；保存关闭重开后官方导出只产生该字段 delta。原资产字节与现有 apply 契约在内存产生的候选一致（SHA `97992ab7cbab62665f17b550ef6ae115ea513302c48db47c44ba2c6030cec383`）。这个传播用局部覆盖尚未在普通 EXE 加载验证，不代表已对齐 UE 间距。实际证据见 [记录](widget-ui-penpot-evidence.md)；共享桌面输入和原生捕获未执行。
