# 动画窗口：flow 与真实能力接入合同


## 2026-10-25：Curve TreeContent 接续与 palette 准入次序校正

UE [SAnimSequenceCurveEditor.cpp](E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SAnimSequenceCurveEditor.cpp)435～484行的 TreeContent 顺序为搜索、可滚动树、过滤状态、居中 Transport；同一 external time slider，不新建播放器。本域原三张 Curve 打开态板已按此补齐可编辑组合，实际保存关闭 fresh 重开 rev560/4959对象，原三 main 不变；[当前范围与引用](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-tree-18/board-application-r01.json)。在原570对象作用域内接续，新增210对象、12个 Shared native instances，合计54copies/522refs全解析；4179个作用域外对象（含Root）及Shared数据保持。

原六按钮加Fill的同一Transport root移入左树底部，事件/route/shapeRef/pluginData保持、按钮仍28×28；没有再造clock/播放器/主件。新增TreeContent是原Curve实例外的wrapper组合，不向component copy内部加结构，也不是正式Curve源；SearchInput/TreeRow保留原Shared source/main/leaf引用，隐藏本状态未接语义的box/chevron/eye/lock/action slot，仅为本地override。三行来自原Vec3端点fixture，未打开项目资产；没有生成Hermite线或稳定key身份。640实际树视口仅28高，900/640需要滚动；设计裁切和源码计数不证明输入、滚动首尾、搜索/过滤/选择/焦点已经运行。Curve toolbar、长名称访问、原640 Details/Search/次级tabs、正式源/codec/导出、真实clock/frame/GPU/EXE/150%DPI仍open。

[六张原PNG逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-tree-18/image-review.json)与[有限检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-tree-18/validation.json)记录三个同阶段save/reopen板内RGB差均0；浏览器fit缩放不是nativeDPI或1DIP对照。r01预期建框导出符号缺失（0写入）、r02投影闭包漏传PAGE（仅首板作用域内部分更新）保留；r03先用原570对象和已知新增对象重建原data hash，证明作用域外不变，再复用原新实例接续，不重建首板。

r03设计数据保存/重开已同SHA，但640保存图多出紫色Transport编辑器轮廓，实际1332 RGB像素差保留在原六PNG和validation-r03-overlay.json。r04固定指针至画布外，仅对同一已保存版本和fresh重开重新捕获，未重做布局或修改PNG；最终六图及原六图均实际打开。该修正不证明产品hover/focus或全局style传播。

Jenkins原owner新[接口结果](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/animation-palette-leaf-route-20261025/product-preparation-interface-r01/result.json)SHA`6c1c60fc…`、[gate顺序](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/animation-palette-leaf-route-20261025/product-preparation-interface-r01/source-policy-and-gate-order.json)SHA`50cbfbf0…`已实际核对，原supported-interface字节保留。actual-checkout Cargo没有独立prepare-only/准备后暂停事务；正确顺序是源owner审当时源并声明唯一命令→透明登记独立identity→normal automatic intake→服务端完整source/external/member/compiler闭包→readguard/pool与E inventory/reservation/cache writer→全部准入后才Cargo→真实native及最终源证明后释放。资源静态空闲和11/12pin观察不替代准入，不要求先取得不存在的prepare-only回执。

本域此前已实际提交的`ue-animation-palette-20261025-01`/`validate-graph-palette-01`/queue15804保持同一身份；[次序对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-tree-18/preparation-order-reconciliation.json)关联它的一次QueryOnly pending，不重POST/query、不改runtime/grant/DB/policy或借旧Editor/Runtime请求。旧准备消息本身没有产品intake，本域后继队列也不补作server source/resource/native receipt；验证过的产品Rust仍0，实际执行未知。无新Cargo/服务/桌面/freeze/公共源码或业务ZUI修改；本族passed/accepted=false，保留必要证据，释放0B。

本轮只落实Curve打开态的TreeContent位置及原实例组合；原默认Closed板、640可达性及完整Toolkit运行验收不因此关闭。

## 2026-10-25：Curve 打开态与独立 palette 正式提交

在原动画 file/page 接续三张 **Curve Editor 已打开**的可编辑 Toolkit 板：1280×800 `eacc278d-0a68-8098-8008-d7d12e5c7a05`；900×620 `eacc278d-0a68-8098-8008-d7d18b28d577`；640×520 `eacc278d-0a68-8098-8008-d7d1e61bbbc5`。按[UE AnimationEditorMode](../../../dev/UnrealEngine/Engine/Source/Editor/AnimationEditor/Private/AnimationEditorMode.cpp)保留 Document/Curve 共用 stack，默认 Closed 的旧板不改；[SAnimSequenceCurveEditor](../../../dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SAnimSequenceCurveEditor.cpp)439～467 的 external time slider、树搜索/状态/左侧 Transport 作为未完成目标。本轮复用原 Curve/Timeline/Transport 与 Shared 实例，不建新 main；新增570对象、42 native copies、321 shapeRefs 全解析，4178原非Root对象及原三main保持。

新板首次打开页签时挤掉 Transport，现按同一 stack 的实际可见子项和文本行高重算内容区，保留失败图与r01/r03/r04辅助脚本失败。修后 rev525 保存、关闭、fresh重开，完整data SHA `3f680a2e8ce6b48b3078aa2d8f46110e225c29eb6944b21245c0fe9af6778266`相同；r05三对板内RGB差0，逐张原图复核见[图像台账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-curve-toolkit-17/image-review.json)。Curve 激活文字使用真实 Shared rev5 的 primary/secondary color refs，本地覆盖不冒称 master 继承或全局A/B。

曲线仅投影原 `write_sequence_asset` 六个Vec3/Hermite端点。范围0～2秒、值域0～6，横向gutter由实际Roboto测量得出；上下边距依据实际分配与行框，640稀疏Y刻度保持端点值。未生成Hermite求值线、持久keyID或运行clock。三板中 Timeline 隐藏，独立Curve inset不能作为双view共同时间列通过。当前UE树搜索/滚动状态/工具栏、左侧Transport位置、640 Details/Search/长次级页签、真实焦点/命令ACK、正式源导出和EXE/144DPI仍open；[有限检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-curve-toolkit-17/validation.json)不是窗口族passed/accepted。

独立palette已沿正式wrapper实际提交 `ue-animation-palette-20261025-01` / `validate-graph-palette-01`，队列15804，客户端exit0；一次同身份QueryOnly为`ordinary_entry_unproven / Current session authorization is pending`，无产品执行回执，已交原Jenkins owner。两源仍fd826da9/5a6ec5a2；不扩张旧capabilities/Triggered/Runtime身份，详[原始交付](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/palette-product-validation-r01/summary.md)。

## AN-P04-ID02 source lease r02：未应用续案

沿 Parent 对 r01 的准备→提交 ABA/lease 审阅，补交 [lease-r02逐文件提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-typed-target-proposal-r01/lease-r02.md) 与 [组合8文件diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-typed-target-proposal-r01/typed-target-lease-r02.proposal.diff)（SHA `3531b070c16488a331084cc1b8a8d9a64cb9c97f9a3948d7629400177fb4c229`）。以当前实际前像替代原r01 diff，原ed47…保留，不叠加应用；没有Rust实施。5个域候选沿r01不变，另给通用store专用SourceReplaced错误，公共host候选由Parent持有。

候选以原 exact-instance edit lease 和原 TransactionScope，在checked prepare之前激活事务，经scope context核源Arc/revision，再用唯一原AnimationEditCommand/CAS/history、显式None/域错cancel和同源dirty guard。read handle只短期借用，不进history，不增Weak/epoch/GraphDeltaCommand。原typed role/pin/legacy歧义拒绝继续；生产输入前的exact view/route/schema/transform校验、菜单不fall through Scene仍按r01由Parent接通。

新实核旁路：restore rollback先remove Session再尝试context detach/unregister并忽略错误。active scope可挡detach、dirty guard可触发原source回滚，但不会恢复被删Session；完整生命周期门继续open，Parent须在首次清退前用原admission核源/初始revision/已接受编辑并处理失败。clone-before-command-permit预算、开始scope对no-op的route/临时reservation/cancel事件影响亦未关闭。

8份stdin rustfmt/只读apply-check通过，canonical源0写/类型检查0/新测试定义及执行0。原47观察当前43匹配；editor_event/types、两Sequence源、shared graph合同后继另记，不回滚。当前生产RemoveTrack/RebindTrack与Undo/Redo已有selection reconciliation调用；实际UI/输入/渲染ACK仍缺。StateMachine沿原editor.animation_graph descriptor，无漏分支伪修复。详见 [当前合同与证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-typed-target-proposal-r01/lease-r02-contract.json)；无新Jenkins身份/query/submit/capture/服务/桌面，整族passed/accepted=false。

## Sequence Toolkit 空轨道设计 r04：AN-SQ-V01/V03/V07（2026-10-25）

本次在原 file `b4a1e3ce-9e6a-81fe-8008-d69d60a13f99` / page `b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a` 增加三个可编辑空轨道板；当前保存、关闭、fresh 重开均为 **rev498 / 4179 objects**。原 3 main 及 3728 个既有非 Root 对象逐值不变；Shared 保持 rev5 / 4 main / 34 colors / 3 typographies。本轮未 sync、reset、detach 或发布共享库。

| 逻辑尺寸 | 新板 UUID | 原板复制来源 |
| --- | --- | --- |
| 1280×800 | `dddddf8a-c730-80a7-8008-d7c0be624a47` | `20e44a50-0bca-5601-a15b-0d0f0a414e4a` |
| 900×620 | `dddddf8a-c730-80a7-8008-d7c0dcabf570` | `6e49f04c-e6af-5f85-a290-200039752d64` |
| 640×520 | `dddddf8a-c730-80a7-8008-d7c0f710352a` | `870c4eef-bdb3-5d46-912b-efcc806d15ae` |

复用 [AnimationEditorMode.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/AnimationEditor/Private/AnimationEditorMode.cpp) 的 20/60/20 水平比例、中央与右列 60/40 分割、Document/Curve 同栈且 Curve 默认关闭。每板保留 flow、150 对象和 13 个 native copy；总计 450 新对象、39 copies（33 Shared + 6 本域 Timeline/Transport）、246 个实际 sourceRefs 全解析。没有新 main 或 Curve 副本。

空态取自既有 `write_sequence_asset` 源 fixture，再以现有 RemoveTrack 语义删除唯一轨道：Hero Sequence、2 秒、30 fps、Root/Hero、target=None，保留 **1 个空 binding、0 轨道、0 keys**。编译器接受空 tracks；没有新增 schema，也没有实际运行该 fixture。选轨清理仍要求实际 host 调用 source reconciliation；删除、undo、切源后的 selection/focus 回执继续 open。0.500 s / Frame 15 仅是保留的编辑器本地游标，不是播放、seek 或 GPU 帧回读；viewport 仍明确 Preview unavailable / mesh None。

本轮实际发现并修正的实例差异：r02 的 640 属性值沿 240 宽模板跑出分配区、空态消息过宽及 0 s/2 s 两处旧关键帧注解；r03 按每行真实 allocation、源 nameColumn min60/preferred/max105、token gap4 和实际文字测量重排，只读值不再越界，空态消息使用真实 strip 宽度并在 ruler 下方垂直居中、保留源左对齐。未烘焙固定 gutter、点或新插值线。r03 行框检查另发现继承 Timeline Noto 文字上沿越界最多3逻辑 px；r04 只改新实例共15处本地 line-height=1.5，并按 ceil(fontSize×1.5) 补足不足框高，保留字体、文本、shapeRef、主件及 flow。当前27条可见 Noto positionData 对名义框越界为0；不推导全局中文字体或 EXE≤1DIP 通过。

r01 旧 native symbol 在 mutation 前失败、r02 截断图及 r03 行框失败均保留。r02/r03/r04 共18张原工作区 PNG 已逐张打开；9对同阶段保存／重开图按实际选中画板边缘比较，内部 RGB 差均0。原 SVG group getBoundingClientRect 包含被裁剪的外溢子对象，原字段保留在 actor receipt，比较框另见 bounds review；不把该 group union 当画板宽高。71%/98%/134% workspace zoom 与 deviceScale1 不属于原生150%DPI。

证据目录：[penpot-sequence-empty-16](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-empty-16/validation.json)，逐图记录 [image-review.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-empty-16/image-review.json)，原失败 [validation-r03-linebox-findings.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-empty-16/validation-r03-linebox-findings.json)。16项源 pin 当前匹配；未改 Rust/ZUI/catalog/tokens/公共 host，六条原 transport events/routes 保持。

640 的 Search 提示、长次级页签和完整可达性仍 open。只读属性文字仍是 wrapper，尚无正式 typed value slot/binding；Timeline 外组合不等于原单 Canvas，正式 Canvas consumer/Curve source/canonical export、真实单时钟和 requested/actual/backend ACK、selection/undo/focus/input caller、normal producer/GPU、托管插件与 Rust、正常 EXE 三宽和实际150%DPI/双端1DIP/样式 roundtrip 均未关闭。此批设计不提供产品通过：`passed=false, accepted=false, captureReady=false`。

接续 [源码盘点](animation.md)、[Parent 交接](animation-parent-handoff.md) 和项目 UE/Penpot fidelity 约定。真实file `b4a1e3ce-9e6a-81fe-8008-d69d60a13f99` / page `b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a` 已有三种专责共享控件及三尺寸Sequence Toolkit结构设计和fresh重开回执；完整业务能力、官方导出、真实引擎预览及验收仍缺。以下各业务行为是接入合同，不能从共享设计进度推断已实现。共享timeline/curve/transport由本聊天负责，实际身份及公共集成边界见 [共享控件合同](animation-shared-controls.md)；不创建平行控件/主题。

所有下列控件行为均是接入目标。当前已有的 UI 节点、模型或命令不证明该目标已在产品连通。Parent 统一协调服务启动、公共组件/主题及桌面截图；本域在已分配文件内进行独立设计，不启动服务或占用桌面。节点图只向材质 owner 交接动画语义，不另建 graph surface。

## Montage侧栏抽屉候选rev405

在原rev383三个默认Toolkit板之外，沿真实Slate的Dock to Sidebar路径，新增900×620和640×520的Details、Sections打开状态。实际保存、关闭、fresh认证重开为rev405/3351对象/原3main；原2658个非Root对象和main记录全部保持，Shared仍rev5且完整data hash不变。本轮692个新增对象包含60个native copies（52 Shared、8 owner），324条叶shapeRef全部解析；没有新main、detach、reset、sync或公共文件写入。

| 尺寸与状态 | 实际board UUID | 最终fresh重开图 |
| --- | --- | --- |
| 900×620 Details | `8644f8cd-8258-80dc-8008-d7a1a00481d5` | [原PNG](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-drawers-14/drawer-900-details-reopened-r04.png) |
| 900×620 Sections | `5c64f781-a29b-8085-8008-d7a1c8463eac` | [原PNG](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-drawers-14/drawer-900-sections-reopened-r04.png) |
| 640×520 Details | `a3465818-80a9-80f4-8008-d7a1f4cbc653` | [原PNG](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-drawers-14/drawer-640-details-reopened-r04.png) |
| 640×520 Sections | `ce760ec0-5a7e-8002-8008-d7a21e351e33` | [原PNG](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-drawers-14/drawer-640-sections-reopened-r04.png) |

参考真实`SDockingTabStack.cpp:751`的Dock to Sidebar及`STabSidebar.cpp:786–843`的覆盖层/宽度限制。候选rail取现有28逻辑像素控件层级，overlap=4来自UE源码；availableWidth=windowWidth−(28−4)，显式示例系数0.5得到900宽抽屉438、640宽抽屉308。该系数代表用户已打开并调整宽度的状态，不是UE默认断点或自动响应式策略；28的精确视觉和UE实际rail度量仍待核对。原20/60/20默认板与裁切事实保留。drawer和rail为新板的普通flow/外wrapper，非已交付公共Dock组件。

Details中Section=Loop、Start=1.000s、Next=Loop↻、Slot=DefaultSlot四个readonly夹具值完整可见；值仍在PropertyEditorRow实例外，不冒称typed slot/真实绑定。Sections保留UE横向AutoWidth命令顺序Default、Clear、Preview All；Default的真实回调是MakeDefaultSequence。完整加载Roboto 700后，三标题实测宽43.2704/31.4635/67.0535，小于原44/32/72文字框，按钮仍为68/56/96；不通过扩展按钮掩盖字体未就绪。禁用外观不能代替真实eligibility/ACK，结构夹具不能代替Montage作者模型或预览。

历史失败完整保留：r01缺原生事件符号且0创建；r02在rev403保存重开成功但drawer/rail被body遮住；r03只改四个新增根的子项层叠顺序至前景，rev404且0新增对象。初次比较用SVG包围框误含画板外文字/编辑器区域，纠正为原PNG真实画板边框ROI；不重写原图。初次字体测量未显式加载700变体，所报六处超宽无效；r04加载实际字体、按原按钮padding核文字框并重开至rev405。24张原PNG已逐张打开，12对画板内RGB差0；这些仅证明可见候选及保存持久性，不证明UE/EXE的≤1DIP。

| 差异 | 当前事实与待交付边界 |
| --- | --- |
| MT-D01 | 打开状态能显示四值和三个命令；默认板侧栏裁切仍在，未实现真实drawer命令/切换 |
| MT-D02 | Parent拥有公共Dock；失焦关闭、pinned保留、ESC/IME/popup/capture/focus-return仍需真实host，普通dismiss不统一Stop/reset |
| MT-D03 | rail精确尺寸/样式/活动标题旋转、pin/dismiss/独立窗口入口仍未逐图对齐UE |
| MT-D04 | drawer覆盖底层timeline；背景完整重排、closed状态及短区间完整名称/滚动/键盘访问仍待实现 |
| MT-D05 | Montage持久模型/codec/稳定ID/事务、真实clock/角色/GPU、正式源wrapper/导出、正常EXE及150%DPI仍缺 |

参见[实例与源检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-drawers-14/source-validation.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-drawers-14/visual-review.json)和[最终实际身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-drawers-14/board-application-r04.json)。五个自有隔离浏览器actor已结束，27个已观察PID/创建时间成员无存活匹配；未触碰其他进程。源资产、catalog及五份UE参考共7pin本轮匹配；未写Rust/ZUI/公共源，未运行插件功能/Rust/EXE、未启动服务或操作桌面。整个窗口族passed/accepted仍false，释放0B。

## Montage Toolkit三尺寸接续rev383

2026-10-25T04:15:15Z先实读本域原file/page为rev274/2158对象/原3main，Shared为rev5/41对象/四main/34颜色/3typographies。接续在同一页新增三个完整Toolkit组合板，最终rev383/2659对象；原2157个非Root对象、原main记录及Shared完整data hash保持。Root只增加三板成员；本轮新增501对象、45个native copies（39 Shared、6 owner），243个叶shapeRef全部解析。没有新main、detach、公共库写入或自动sync。历史rev14空作者区、rev140 Sections和rev274 Sequence板全部保留。

| 逻辑尺寸 | 新board UUID | 内容及实际证据 |
| --- | --- | --- |
| 1280×800 | `081ff34a-fe69-596e-9260-53610383da90` | [最终重开PNG](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-toolkit-13/toolkit-1280-reopened-r04.png)；Default/Clear/Preview All及四个readonly值完整可见 |
| 900×620 | `8091305f-f44d-5eb3-a22b-70e5e3029b43` | [最终重开PNG](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-toolkit-13/toolkit-900-reopened-r04.png)；Preview All、DefaultSlot及部分侧栏可达性仍受限 |
| 640×520 | `39a46fef-93f9-5ef1-8398-d251eee24935` | [最终重开PNG](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-toolkit-13/toolkit-640-reopened-r04.png)；侧栏按钮/值/页签裁切，短区间完整名称访问仍缺 |

Toolkit chrome及区域组合采用原生row/column flow，保留UE `AnimationEditorMode.cpp`的20/60/20、中心及右列60/40关系。时间数据面和18个文字裁剪viewport是Canvas数据坐标例外。左侧Skeleton Tree、右上Details和已打开的Sections前景为本板选择状态；不把它记成实际用户打开资产、切tab或默认Asset Browser前景的证据。Curve Editor默认closed，页签well隐藏。27个Button、12个PropertyEditorRow、3 Timeline和3 Transport均为原组件实例；Readonly值在实例外wrapper内，不是已交付的typed slot绑定。

本板明确是UE源码语义布局夹具：视图0–2秒、派生游标0.5秒；Default从0秒指向Loop，Loop从1秒指向自身；一条DefaultSlot内两个未解析Clip段，另有1.2–1.6秒NotifyState和1.75秒点Notify。时间顺序与Next关系、源Clip局部时间与Montage位置、点与状态区间分别表达。designKey不是持久元素ID，未创建Montage资产/schema、未运行采样/通知/预览，禁用外观不代表真实capability ACK。Preview明确显示未接入，没有角色图或假GPU帧。`Default`的实际UE回调是`SAnimMontageSectionsPanel::MakeDefaultSequence`（:142）；原r01辅助plan中的`DefaultSortSequence`注记保留为已纠正历史，不能作为生产route。

时间投影复用原Timeline Canvas边界：range=[0,2]秒，x0=Canvas.x+52，x2=Canvas.right−20。标尺按现有`build_timeline_ruler_ticks`的80逻辑像素间隔政策核对，三尺寸显示0/0.5/1/1.5/2、0/1/2、0/2；本轮只是源码对照设计，未执行Rust函数。九个gutter标签与相应时间行文字框的y/height读回差为0，18个普通裁剪框按真实投影区间限制文字，不平移事件或扩大时长。该0值仅是Penpot设计几何检查，不能计作UE↔EXE的1DIP结果。中文显式复用原main已有Noto字体身份，Roboto/Noto均实际加载；共享typography引用与局部font覆盖分开，full sync/reset及引擎字体ready仍未验证。

首轮r01背景遮住覆盖层、标签多行堆叠及640标尺重叠，r02修正后仍有行标签位移和区间文字相互覆盖，原12张图保留。r03辅助检查错误地要求六个原main衍生普通标签都有共享typography引用，在18个普通裁剪框已保存后停止；错误与现场保留，fresh只读实见rev381后r04复用全部框接续，没有重套rev379。最终[源/引用/重开检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-toolkit-13/source-validation.json)、[18图逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-toolkit-13/visual-review.json)及[最终对象与身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-toolkit-13/board-application-r04.json)保留全部来源。r01/r02/r04均实际保存、关闭、fresh认证重开；九对板内RGB差0。工作区缩放71%/98%/134%、deviceScaleFactor=1，均非正常EXE或实际150%DPI。

AN-M01～14仍按原Montage文档/codec/稳定ID、Slot与角色资产、事务/history、单一真实clock/preview/GPU的合同接续；此设计没有补齐任何后端。Timeline原源仍只有一个Canvas，新增Toolkit/覆盖层无正式source-equivalent wrapper；Curve仍无正式ZUI，六条Transport原event/route及Fill不变。官方owner迁移/codec/导出、typed slot、900/640 overflow与完整名称访问、输入/焦点/ESC/撤销/滚动首尾、插件功能/托管Rust、真实EXE/同状态并排diff/样式往返及实际DPI继续open；本轮执行0Rust/0插件功能测试，产品passed/accepted=false。

## Graph typed目标未应用提案AN-P04-ID02

接Parent保留legacy named ID/codec/compiler的明确决定，已沿既有[共享图动画消费](shared-node-graph-contract.md#动画消费适配)形成[具体提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-typed-target-proposal-r01/proposal.md)和[七作者文件未应用diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-typed-target-proposal-r01/typed-target.proposal.diff)。候选在Editor原域使用`Named(String)/OutputSink`，直接适配已有泛型NodeId；typed pin操作不降成旧output字符串，旧目的/删除歧义及Additive缺pin在首次写前拒绝。真实source handle/revision准备检查仍属原store，原AnimationEditCommand/CAS/history/retention唯一；不外套GraphDeltaCommand，不改Runtime/codec/compiler/公共Graph。

Session label→typed projection/canvas、明确instance/source的host入口、菜单Delete不泄漏Scene、原slot生产命中及焦点/生命周期逐文件owner与完整preimage见[源合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-typed-target-proposal-r01/source-contract.json)。这些都是未实施接线，七文件diff不能单独算typed UI可用；原公开command/codec/normalize及ZUI/六Transport事件保持。原r01两源477ca8cf/3c39e75a及四项定义不改；12/14历史保护pin与两Sequence后继a254f962/2385484c分时点保留，不重签旧guard。仅列8组必要后续用例，新Rust定义0/执行0，无新identity/request/attempt/query/submit/capture/freeze/Cargo/服务/桌面，passed/accepted=false。

## 缺源重绑保持完整资产AN-SQ07-REVIEW-NOOP-01-r03

接续Parent对原r01及time/binding后继的源码审阅，确认跨实体`rebind_track`原先先执行`retain_mut`：合法的空source binding会被删除，随后`moved_track=None`却返回false。`store.prepare_mutation`在false时丢弃clone，所以此发现不是已发生持久文档/GUI丢失的证据。已仅在原跨实体分支首次修改前扫描所有匹配entity/property的真实源track，缺源立即false；同实体原位重绑、正常跨实体destination target authority、RemoveTrack保留空binding及帧秒helper正文均不改。不能复用只找首binding的`find_track_mut`来拒绝后续同entity的合法track；未新增entity唯一性/schema约束，也未改Runtime/compiler/host。

原mutation从`01bb971d…`后继为`a254f962ffcccdbd7346ea4e2a19ce451973c3ca44a51fdd02d5dc1db2efac63`，只增加10行只读预检，其外全部字节保持。原测试叶从`7a692ffc…`后继为`2385484cf70754e7484124322af9541ab811dc8e7771001d53b16577f575d3cd`；原九项15024字节（含原五项9354字节）完整保留，只在末尾新增fixture/helper与三项`sequence_rebind_`：12组source Some/None、空/有其它轨及新/既有Some/None目标，断言false、完整asset/order/target/codec不变；同矩阵的prepare=None、revision/source/last-good/dirty/history/undo/redo不变；首个匹配binding为空、后续有真实track时仍可跨实体移动并保留目的target和channel。第三项不锁定成功路径上空binding的清理策略。codec是内存往返，未称磁盘保存/资产实际加载。

[本轮增量补丁](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-rebind-noop-r03/actual.patch)、[源合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-rebind-noop-r03/source-contract.json)与[检查记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-rebind-noop-r03/validation.json)保留Parent原发现、26个保护/参考观察pin及首次edition假设在0写入前被实际2021声明拒绝的静态记录；没有改旧packet或重套旧patch。仅按实际edition进行stdin rustfmt与增量反向检查，测试全叶静态12项、`sequence_rebind_`静态8项，三项新增执行0，不能拿旧filter/guard或codec断言定义当运行结果。

未来候选仍是根`zircon_editor`单包/defaults off/features=[]/Windows host lib/locked/test语义/jobs1/threads1/filter `core::editing::animation_document::mutation::sequence::tests::sequence_rebind_`，实际传递target-editor-host/test-support依赖保留；未建立request/attempt/intake或运行Cargo/QueryOnly/DryRun/capture/freeze。Parent Runtime四源、AN-G13/AN-P04/Graph身份guard、公共assets/catalog/六event和Penpot rev274保持。本修复未连接生产sample/clock/GPU/EXE/DPI，AN-SQ07运行回归及全族passed/accepted仍open。

## Sequence Toolkit三尺寸源布局AN-SQ-V01/V02/V04-rev274

沿原AN-SQ-V01～07，在本域原file/page新增三张完整区域组合的可编辑设计板。依据[UE AnimationEditorMode](E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/AnimationEditor/Private/AnimationEditorMode.cpp)51～117行，左/中/右分配为20/60/20，中央Viewport/Document及右侧Details/Asset Browser为60/40；Document与Curve Editor在同一stack，Curve Editor默认Closed，两处中央well隐藏。左右各4 DIP splitter仅为本次投影，尚无UE原生尺寸对照。工具栏文字来自同目录AnimationEditor.cpp及AnimationEditorCommands.cpp；Save/Browse和顶层菜单留给Parent公共框架，当前设计文字不证明命令已接通。

| 逻辑尺寸 | 新board UUID | Dock左/中/右宽度 | 中央Viewport/Document高度 |
| --- | --- | --- | --- |
| 1280×800 | `20e44a50-0bca-5601-a15b-0d0f0a414e4a` | 254.4 / 763.2 / 254.4 | 408 / 272 |
| 900×620 | `6e49f04c-e6af-5f85-a290-200039752d64` | 178.4 / 535.2 / 178.4 | 300 / 200 |
| 640×520 | `870c4eef-bdb3-5d46-912b-efcc806d15ae` | 126.4 / 379.2 / 126.4 | 240 / 160 |

实际保存/关闭/fresh重开为rev274/2158 objects，data SHA `6188614e9d14e1c20812ea41ad53888a556ef615fbc82756e32a676dd23658cf`。新增450对象中39个native实例为18 Button、12 PropertyEditorRow、3 SearchInput、3 Timeline、3 Transport，合计246个逐叶shapeRef全部解析；其中33个Shared外链copy、6个owner copy。原1707个非Root对象及3个main逐对象/记录hash均不变，Root仅新增三板成员。没有新main、detach、发布本域3main或写Shared；Shared本轮前后rev5及hash相同，不自动sync。外层使用native flow和既有样式；数据点、游标与实例内caption/glyph仅作绘制投影。

源内容复用未执行的`write_sequence_asset`：实体`Root/Hero`的Transform.translation、2s/30FPS、两个Vec3 Hermite key、frame15→0.5s，当前helper正文与r06相同。它不是骨骼clip、真实已打开资产或求值后的曲线；本板Curve Editor默认关闭，没有制造Hermite线。Preview明确为unavailable/mesh None，Skeleton为空、Asset Browser为无数据设计状态，均未接真实catalog/backend回执。属性值为原实例外wrapper中的本地文字，尚非typed slot/binding；原`animation_sequence_body.zui`仍只有6个header/slot control，TimelineStrip仍是单Canvas，不能把本次完整组合算成已正式导出的源组件。

首轮真实图发现覆盖层在背景后、Button内部保留原宽度及多余默认well，r02仅修本轮新copy层序/几何、隐藏well、按源80 DIP密度保留5/3/2个时间标签。r03实际Roboto载入前校验曾失败，0写入/0截图并保留[失败回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-toolkit-12/native-caption-r03-font-failure.json)；随后载入同一已声明字体，按真实advance+左右12 DIP token将Reimport Animation从140改145、Apply Compression从136改141，18个标题均单行，标尺文字中心与源tick列误差0。均保留component/main/shapeRef及原6 transport events，不把本地caption覆盖说成公共Button自动伸缩已修。

[18张原工作区PNG逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-toolkit-12/visual-review.json)与[最终几何/源引用](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-toolkit-12/caption-geometry-r03.json)保留r01/r02失败和三轮9对保存/重开板内RGB差0。SVG group的包围框包含overflow，未误作板边界；比较按实际选中外框内缩取像素，完整原图未改写。71%/98%/134% fit缩放及内部数值误差不是native DPI或UE/EXE 1DIP证明。640的四行值、两个侧栏tab和Search仍裁切；900的Search末端及类型注释也受裁切，交Parent公共Dock/字段布局/overflow，不缩改UE默认比例来掩盖。字体/图标/菜单精确度、全部真实输入/焦点/弹层/scroll/undo、生产clock/sample/frame ACK、官方codec及正常EXE/150%DPI仍open。

[源对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-toolkit-12/source-validation.json)记录15个预核源及7个补充观察源均未变，另列Button/ruler指纹，不称完整source闭包；本轮仅写本域两份设计/交接文档，未改Rust/ZUI/catalog/公共Runtime/host。静态检查与生命周期见[本轮记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-toolkit-12/validation.json)。原AN-P04-ID01/AN-G13/Sequence修复及各未运行回归保持；无Cargo/Jenkins/query/DryRun/新identity/source capture/服务/桌面动作，passed/accepted=false。

## Graph输出节点身份创建保护AN-P04-ID01-r01

继续真实Graph作者命令核查，确认现有DTO的`Output { source }`无命名ID，但原Editor remove/connect/disconnect用字符串`output`寻址该唯一sink；`add_node`却允许Blend同名，后续连接可同时更新Blend与Output，删除可同时移除两者。反向先有命名`output`再创建sink也会引入同一歧义。这是源码数据流确认，尚无Rust/运行时复现。UE `AnimationGraphSchema.cpp:241`以FGraphNodeCreator建立默认Root，`EdGraph.h:300`创建独立NodeGuid，`Output Pose`标题不承担身份；UE不以字面字符串`output`作为这一限制。当前guard只是保护Zircon已有命令身份，不能当作完整UE节点实现。

已仅在原`mutation/graph.rs::add_node`拒绝新Blend ID=`output`，以及已有命名`output`时创建sink；正常命名/原重复检查和其它mutation正文逐字保持，原测试叶独立挂载。当前mutation SHA `477ca8cfee0d4160867c3a0caf92fec79433bed05e4af5a1f4aa3fb295e92b86`，原fe9ee98f…保留历史。Runtime的`collect_nodes`确实排除Output后再索引命名节点，因此既有Clip ID=`output`配合Output source=`output`是合法源；未改asset/compiler/codec、自动rename、capability或公共Graph/event DTO。既有此类资产的Editor寻址歧义仍缺typed node/sink context或经明确决策的迁移，交Parent/公共Graph owner；当前boolean拒绝也未补生产菜单eligibility/解释。UE默认Root与Zircon当前可创建/删除Output的行为差异继续open。

新增四项`graph_output_identity_`只定义、执行0：有/无sink时的新别名拒绝且bytes不变；四种原命名节点后加sink拒绝且payload不变；合法legacy codec/编译与普通ID连接/删除；原CAS/文档history中的拒绝no-op、合法创建/连线、invalid→last-good/codec及undo/redo/dirty。格式和[实际增量patch](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-output-identity-r01/actual.patch)反向检查通过，[源合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-output-identity-r01/source-contract.json)/[检查记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-output-identity-r01/validation.json)保留原参考和边界。未来scope候选是根`zircon_editor`单包/defaults off/features=[]/Windows host lib/locked/test语义/jobs1/threads1/filter `core::editing::animation_document::mutation::graph::identity_tests::graph_output_identity_`，不是下节nested palette配方；未新建request/attempt/operation或调用QueryOnly/Cargo。

AN-P04原两插件源、AN-G13、AN-SQ07/08、Parent四Runtime源、ZUI/六events/catalog保持。r03 palette消费当时29/30和原r02记录均不回写；本次私有graph mutation是后继依赖变化，当前对原r02为28/30（另catalog后继），不借旧source guard背书新源。Penpot复用rev183，正式导出/production graph/input/focus/cancel/真实preview/EXE/DPI及全族passed/accepted仍open。

## Sequence选轨曲线完整路径解析r01

继续原Sequence作者链，发现Session曲线投影与已修复的mutation完整路径查找不一致：合法源可将同entity的不同property分放多个binding，`select_timeline_span`/Timeline枚举可找到后面的轨道，但 `session/curve_foundation.rs::project_selected_track_curves` 先find第一个entity再找property，前置空binding或其它property会使其报missing。当前source compiler只在单binding内检查重复property，接受这种源结构；当前结论来自具体控制流，未声称Rust或GUI运行复现。

已限域修改原 `curve_foundation.rs`：按entity过滤全部bindings、展开tracks再匹配property；唯一匹配保留既有曲线分量/时间/值/切线投影，缺源仍诊断missing；多个目标共用同一路径时明确报ambiguous，不以Vec位置选目标。生产文件SHA由 `5d9ff349…`到 `c4d3add5303e91e691be8b2fb4bf175d0aabf60707b9ed26e18fd24f603bbd26`。没有调整Runtime/codec/compiler、public graph/Canvas/host、共享CurveModel或curve编辑事件；持久key identity与typed target选择仍缺。

原 `session/tests/cases.rs` 的14109字节/7测试前缀完整保留，仅追加1个内存source helper和3回归，当前SHA `10d9bdcc88775c2db92c6cbe5b0067840e96596089906881ee0c2a3f3abc8bfe`。新filter `ui::animation_editor::session::tests::sequence_curve_binding_lookup_` 精确静态选择3项，当前该cases模块共10项，执行均0：

1. leading空/其它property、另一entity同property、目标Some/None四组真实source经二进制codec重读与源编译后选择，检查Vec3/Hermite时间/值/两切线和Timeline一致，文档bytes不变。
2. 同一AnimationAuthoringDocument读句柄经prepare_mutation/AddKey/AnimationEditCommand/CAS/唯一document history提交，Curve读取新keys；undo/redo还原视图和bytes、dirty状态，保留两个无关binding及大整数target_id字符串。
3. source compiler接受的跨binding多target同路径保持原source与transient选择，并返回明确ambiguous，避免按顺序显示错误目标。

[精确两源增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-curve-lookup-r01/actual.patch)、[source/前缀/依赖合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-curve-lookup-r01/source-contract.json)和[当前静态检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-curve-lookup-r01/source-validation.json)已形成。首次新增断言的换行格式失败原样保留，纠正后rustfmt check、scoped diff/reverse check和source guard全通过，guard0错误/0警告/0抑制，14参考源保持；这不是三项Rust通过。对Editor与动画图两个源码树排除test/定义模块后未查到curve_foundation生产调用，记录搜索范围，不扩大成全仓结论；正式outer composition/host仍需Parent消费该现有投影。

最小未提交验证候选为根zircon_editor单包/defaults-off/features=[]/Windows host lib/locked/jobs1/threads1/上述唯一prefix/test语义，保留既有target-editor-host/test-support传递依赖；实际wrapper profile/env、完整source/external/writer、物理E资源/pool/cache和native EOF须原authority准备，不借旧capabilities/Triggered/Runtime Bool/palette身份。没有QueryOnly/DryRun/Cargo/intake/request/capture/freeze、Penpot/ZUI/公共handler或服务/桌面动作。source-08及rev180/182/483旧包不改签，后继源另列。真实GUI选择/gesture/undo、实际采样/pose/GPU、正常EXE/144DPI/官方导出及全族passed/accepted仍open，释放0B。

## Sequence跨binding完整路径解析AN-SQ07-r04

继续原两私有作者路径，发现并修复了r03只解决缺源no-op之后的真实命令问题：source compiler逐binding编译，property重复检查只在单个binding内，因此同一entity可有多个binding。原Add/RemoveKey及同实体Rebind只查首个entity binding；若首项空或仅含其它property、真实track在后项，合法track会被误报缺失/忽略。旧CreateTrack还可能在前项重复创建已有完整path；RemoveTrack和跨实体Rebind对重复path可能批量删轨/只保留最后一个结果。源码推导依据在[本轮合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-binding-lookup-r04/source-contract.json)，未将推导称为已运行的Rust/GUI复现。

已将原find_track_mut改为在全部binding中完整匹配entity+property：唯一track可编辑；多个实际命中使用既有InvalidTrackPath返回明确ambiguous诊断，首次源写入前退出。same-entity rebind与cross-entity preflight复用该解析，CreateTrack现先检查完整path，唯一已有track为no-op，RemoveTrack也先确认唯一真实track。既有同path rebind/目的冲突no-op保持；未新增schema/compiler拒绝、自动合并/重命名或删除合法旧源。ambiguous是作者命令缺binding身份而拒绝，资产仍可读取和source compile，不证明World绑定可执行。

唯一匹配修改保留实际owning binding的target_id、顺序及无关轨道；新增缺失path仍沿原首matching binding创建权威，不猜测被删旧轨属于哪个目标。跨实体新/既有目标Some/None沿原目标binding权威，删除仍保留空binding。持久BindingId/KeyId及typed target caller未因此交付，不以临时迭代引用或Vec位置冒充跨revision身份。原Store.prepare_mutation→AnimationEditCommand→document CAS/history仍唯一，公共host/Runtime/compiler/schema/capability和六Transport事件不改。

当前mutation SHA `a882466d5e9097ac0ecdfa8deb001fd793b95fa06fc14f4a3aa400c42d668ee7`、测试SHA `6cb5006de91002bd733c12113dcc90805e806f4463f6737ab25307738a456580`；原12测试20751字节完整保持，在末尾新增6项sequence_binding_lookup_，全叶静态18项/发现未知/执行0。四种leading空/其它property与owning target Some/None组合保护重绑、加删key、Create no-op/保留创建权威、删轨保留target；四种歧义target组合×七个命令情形共28组保护五种命令类型的直接源与prepare错误、codec/revision/last-good/dirty/history不变；三笔真实原事务及三次undo/redo保护每编辑一份历史。codec仅内存往返，未声称磁盘资产已加载。

[最终增量patch](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-binding-lookup-r04/actual.patch) SHA `5143c1a7399d7e43782d95609c718a21f1590613f9171fdfe67e2d3f6343fe3b`，19份Runtime/host/codec/compiler/旧修复/catalog保护源一致；edition2021 stdin rustfmt、差异/反向check及[最终guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-binding-lookup-r04/guard-r02.json)0错误/警告/抑制。先前guard.json只属于同轮初版测试，后继r02补充cross-new/cross-existing Some/None回归，原指纹保留。所有静态证据见[validation](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-binding-lookup-r04/validation.json)，不能当Rust/产品passed。

后续拟验证范围是zircon_editor/defaults off/features=[]/Windows host lib/locked/jobs1/threads1，filter core::editing::animation_document::mutation::sequence::tests::（当前静态18项）。该root Editor scope含target-editor-host传递依赖，不是nested palette leaf；尚未运行wrapper/DryRun/parser/query或创建identity，正式profile/argv和完整source/external/writer/工具/E资源/pool/cache准入由原authority核实后再声明。旧capabilities/trigger-condition/Runtime Bool身份不扩张。源身份、生产Canvas/gesture/稳定key、真实clock/GPU、正式导出/正常EXE/DPI及整族passed/accepted继续open，captureReady=false，释放0B。

## AN-P04叶包固定路由实际交付r04

原管理owner沿`ue-plugin-workspace-entry-20261024`完成`AN-P04-PALETTE-LEAF-ROUTE-01`；后者是协调标签，不是产品request/op。已实际读回[原交付](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/animation-palette-leaf-route-20261025/result.json)、[摘要](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/animation-palette-leaf-route-20261025/summary.md)、[限定接口](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/animation-palette-leaf-route-20261025/supported-interface.json)和增量patch，四份SHA均匹配通知。当前两palette Rust仍`fd826da9756d45329be6154fe4c10073f8364790a331ebc8ab3454e4fe0a16ee`／`5a6ec5a227c0f32b0104a2f435a606896b39967fbc1376a05b0c293dc8c7775f`，当前canonical wrapper/entry/runner/新增focused test四源与原交付匹配；c2806705胶囊6server+6client共12不可变成员也匹配。本轮未fresh查询在线API或断言新的服务selection，原owner当时API/Home/public transport读回按原证据复用。

原Jenkins #476 queue15592 SUCCESS的293项是管理回归（87 palette+206既有），#477 CheckOnly queue15596、#478 apply queue15600是部署回执。#476～479的native终止与local/Home archive及#479自有清理按原owner报告复用，不计作本域Rust/画面通过；原438039是其逻辑清理字节，物理回收未知，本域释放0B。旧receiver拒绝及r01/r02/r03回执全部保留，不能继续把旧“不支持nested manifest/profile”当当前阻塞。

限定语义仅为`zircon_plugins/Cargo.toml`中`animation_graph/editor`／`zircon_plugin_animation_graph_editor`，defaults off、features=[]、lib、显式profile=test、唯一`tests::graph_palette_matches_creatable_editor_capabilities`、locked/jobs1/threads1、Windows host且无`--target`、物理E。wrapper现按这个唯一Package/filter映射nested manifest和显式test profile，canonical cwd仍根checkout，不以root-p或任意cwd替代。Runtime native-host及Editor target-editor-host仍是传递声明；管理合成闭包检查不等于实际Cargo解析。原material/Prefab/AI/PFX/Sound/glTF/Node路线不由本域修改。

下列只是实际source/resource/pool准备就绪后，由源owner透明登记独立palette identity并再次声明时采用的模板；本轮未调用，包括DryRun/QueryOnly。它不是现在的执行请求：

```powershell
& 'E:/Git/ZirconEngine/tools/dev/dev-module-build.ps1' -Package zircon_plugin_animation_graph_editor -Action test -Features @() -LibTests -TestFilter 'tests::graph_palette_matches_creatable_editor_capabilities' -Jobs 1 -TestThreads 1 -JenkinsIdentityFile '<独立且匹配当前源的 palette identity JSON>'
```

本轮[消费记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/palette-leaf-route-delivery-r02.json)明确区分routeQualified=true与runnableNow=false：完整当时source/external membership、writer/工具、physical E reservation、兼容pool/cache/native writer准入尚未建立，captureReady=false，产品request/operation/attempt/server receipt全null。原唯一Rust测试静态1项，发现未知/执行0；不重套旧patch、不扩张capabilities/trigger-condition或私有Graph/Runtime Bool身份。五项advisory policy沿原交付保留，不追认未知历史写者；真实palette点击/Graph typed caller、EXE/DPI及整族passed/accepted仍open。

## AN-P04受管入口核对与依赖后继r03

已逐字节核对原Jenkins owner的[palette-r02-reconciliation](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/animation-original-queries-r05-20261024/palette-r02-reconciliation.json)，46805B/SHA `eaea4a71a937bd9e25e585c163c3c13b359976854fbb9cc6b13fd2589529958f`；摘要6077B/SHA `037cb9e23a2680624268e7134e4c8deb75521d00f0171b59df899d3901ed0e2c`。原AN-P04两源、actual.patch和原静态回执均保持；r03消费时30项依赖对原r02为29匹配，唯一catalog从`357e63b2…`后继为`9556e66d…`。后续AN-P04-ID01私有graph mutation变化见上节，当前28/30与该次29/30分开。原四域文档相对r02均有后继，r03又有三份相对owner报告的后继，分别保存观察时SHA，不推断写者、不回滚或改旧packet。此处只对源身份和现成回执复核，不重跑同字节fmt/Rust。

原owner对canonical `zircon_plugins/Cargo.toml` / `zircon_plugin_animation_graph_editor` / defaults off/features=[] / locked/lib/test / `tests::graph_palette_matches_creatable_editor_capabilities` / jobs1/threads1的纯client parse及runner `validation_recipe`均获`build_scope_policy`拒绝，actual source-member catalog也缺animation_graph/editor。`runnableNow=false`，没有此候选receiver schema/server/native回执。原module wrapper不选nested manifest或显式test profile，静态argv仅root recipe/元数据dev且不带`--profile`；root语法可接受不证明该包在根workspace，禁止现在执行原候选wrapper或改cwd/root-p/材质filter替代。下节已改为不可执行的历史提议，不删除原提出/失败证据。

最小接续仍归原管理owner：为显式Package+唯一filter联动canonical nested manifest/test profile映射、client+runner有限策略、actual source catalog和native/editor传递feature合同，完整manifest/lock/.cargo/跨workspace/source/pool身份保留，正式聚焦管理验证后发布新immutable capsule。此为未实施提案，不在本域改公共wrapper/sealed/策略或创建repair identity。leaf features=[]仍传递Runtime native-host及Editor target-editor-host，不能称轻量leaf；真实source/writer/resource准入及执行前声明仍缺。[本域消费记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/palette-receiver-consumption-r01.json)单列当前pins及历史证据；1项Rust定义执行0，旧capabilities/Triggered身份/r05 pending不重query、不借作palette结果，无新提交/服务/桌面/freeze，passed/accepted=false。

## Montage字体行框复核MT-S05-rev183

沿rev181记录的`MT-S05-current-CJK-linebox`，先独立读取当前rev182/1708对象和原Sections板53对象，实际加载的Noto Sans SC在10.666667px下fontBoundingBoxAscent=12、Descent=3，14个原文字均fontReady。原共享caption的1.2倍行高约12.8px，保存的名义行框为y1=-2/y2=13。仅将这些已有本地Noto字体覆盖文字的lineHeight改为1.5，即16px（ceil(15px字体框)+1px取整余量），全部原分配高度16～40px均能容纳。没有挪动框、增大字号、改字串、造Label或修改公共typography；共享caption UUID、color refs、token-source及Noto本地字体覆盖保持，导出须明确保留此局部行高覆盖，不把公共caption的默认1.2改签为1.5。

原生text update实际保存、关闭、fresh认证重开到rev183，data SHA `b415594e0bda3fbcb2fafc5ec363beedca588c9336941ec87c6a76766bb66d0a`。14个原leaf只变content.lineHeight及重测positionData，读回y1=0/y2=15、最大行框越界0；其余1694对象、原3个main、53外框/flow和9个Shared Button head/text shapeRef完整保持，仍为原Preview All及Preview/Default/Next。Shared实际rev5前后同hash，未sync或写入。[字体度量与当前原框](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-linebox-11/preflight.json)、[14项实际行框](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-linebox-11/text-frame-review.json)和[源保护检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-linebox-11/source-validation.json)保留可复核身份；56份UE/原资产/域源指纹全同，未改上轮binding修复。

[三张原工作区PNG及诊断diff逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-linebox-11/visual-review.json)均已实际打开，保存/重开板内RGB差0；整张PNG有工作区状态差异，原图不改写。diff是板内crop、RGB差×4及最近邻×2诊断，不是官方导出或产品比较。旧rev181的2px差异保留历史，只关闭本板14处名义行框子差异；完整MT-S05的实际引擎字体/中文/1DIP/native150%DPI、公共样式A/B/reset、正式codec与双端对照仍open。未运行Plugin功能/Rust/Jenkins/query/新请求/capture，无服务/桌面操作；两次隔离browser均真实退出，passed/accepted=false。

### Parent第六次Sections复核与未同步样式

原public owner于2026-10-25T05:45:09Z的[实际只读审查](E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/animation-montage-sections-intake-r01/parent-r06-sync-review/handoff.md)读到原file/page rev405/3351对象/3main；本域本轮仅消费回执，未新读API、保存重开或截图。原Sections板仍53对象、9原Shared Button head/text来源及全部外几何/拓扑/根pluginData保持，14处Noto Sans SC、lineHeight=1.5及positionData/rect/文字与rev183后像逐项相同，Preview All修正保持。91为非空style引用出现次数（含content/positionData重复，5个UUID），18空另计；不将其当91个独立绑定或全局字体继承。

Shared返回rev5/41/4main/34色3typo，但linkedLibraries.syncedAt仍2026-10-24T18:55:49.510728Z；九Button根仍#242424/alpha1、0 color ref，尚未消费main新surface.1引用。当前对rev140仍23对象变化：content15/fills14/positionData23/touched9，文案/行高/覆盖是有语义的变化。MT-S01样式继承、MT-S05全局字体及两族inherit/override/reset保持open；本轮不Update/reset/full sync、detach或改Shared，不以发布版本/颜色外观替代同步证据。未来update前须取真实before，after须保存关闭fresh重开，并核九标题/十四Noto/行框/source refs与全部目标覆盖；rev183或rev405的保存一致性不代替该实验。

原Montage owner仍v2/31nodes/29control IDs/15events/7imports、root collapsed；该53对象板官方asset/document role为0，原Intro/Combo表行不能重命名冒充本板Factory Default/自环模型。保留原ID/events/绑定，Sections稳定业务身份、动作payload/事务与source node+invocation需沿既有Montage合同闭合，再由原pipeline执行exact-owner导出。prepareRealOwnerInput/ComponentReviewHost仅是源闭包候选，reviewCase undefined/managedPresentationPrepared=false；当前normal producer仍LevelEditor/Scene，不能借其nonce/frame给Montage。[本轮源码指纹与边界记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-05/parent-r06-consumption.json)不改旧51/52历史、catalog9556后继或1338+3main/Root+19未覆盖限制；MT-S01～05整体、官方ZUI、模型/clock/GPU、EXE/DPI/双端1DIP及passed/accepted继续open。

## Sequence删轨保留binding目标AN-SQ07-r02

接续AN-SQ07的目标身份修复，另发现`RemoveTrack`用`retain_mut`清掉最后一轨所在的整个binding；随后同实体`CreateTrack`新建`target_id=None`的binding，丢失原稳定目标及binding顺序。Runtime实际按target_id优先、entity path后备解析，目标可因此改变。这是本轮源码数据流核查结果，尚无Rust/运行时复现。UE的`FMovieSceneBinding::RemoveTrack`只移除轨道，`FTrackModel::Delete`调用该方法；Zircon原`compile_binding`也保留并接受空轨道集合，不需要改编译约束或schema。

已只在原`mutation/sequence.rs::remove_track`遍历binding并删除匹配轨道，保留空binding、Some/None目标和顺序。不存在的轨道返回false且不再顺带删除已有空binding。原同实体重绑分支、跨实体目标归属及AN-SQ08帧秒helper保持；没有新增binding删除命令。当前源SHA `01bb971da3e81b60b09d6ba7006ceaef65799b54806cabe4032d54fb44b93b99`，是r07记录的`2e0d8676…`之后本域改动，旧23源packet不改写。

原`tests/sequence_rebind.rs`前9354字节及五项回归逐字保持，追加四项`sequence_binding_`：最后轨道删除的Some/None目标与codec/编译、重建同/异属性轨道的目标与无关binding、空binding上的无效果删除，以及原CAS/文档历史两次操作的undo/redo、dirty、codec/current/last-good。四项仅定义、执行0，codec检查是内存二进制往返，未声称完成磁盘保存或实际资产加载。两个原Rust文件格式及增量补丁反向检查通过，[补丁](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-binding-r02/actual.patch)、[源合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-binding-r02/source-contract.json)与[检查记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-binding-r02/validation.json)保留最初补丁CRLF校验失败及LF修正后的结果。此修复仍待实际受管回归，不启用新capability。

未来最小候选为`zircon_editor`单包/defaults off/features=[]/host lib/locked/Cargo test默认test语义/jobs1/threads1/filter `core::editing::animation_document::mutation::sequence::tests::sequence_binding_`，不是本轮执行或新请求。原两个Editor身份和独立Runtime准备合同保持，未query/重投。Penpot仍复用rev182，不改3main/9copy/216refs、原六event或公共Runtime/compiler/host/ZUI/catalog。真实时钟、采样ACK、正式导出、正常EXE/DPI与AN-SQ-V01～07继续open。

## Sequence source-08原owner审阅接续r01

已实读原public owner的[实际审阅](E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/animation-sequence-source-intake-r01/handoff.md)，SHA `357f11544c62bbfe4eeba5f4be939e3c6358ec77adbf4a83f5f5fc32b6fa2f1e`及其17项只读核查；6份r06证据和3份native终态SHA均匹配。rev180、后继rev182及本域最终rev483分别保留；owner的rev405/3351/Shared5是其API观察，不称本轮fresh状态。本轮无API/浏览器/截图或SDK refresh。其9copies/216refs、21 absolute glyph在28×28内中心差0、21路径/metadata、18隐藏caption和3main/board拓扑结论沿原时点复用；297 scope的78项后继geometry/positionData/touched不回滚，不重签旧closure。

本轮只读当前23域源全部匹配owner观察，其中20份仍等于原r06，三份mutation/sequence、session/support、session/tests后继SHA单列；15实现源有14份匹配。`plugin.ts`由owner记录的 `1db1b92f…`后继为 `7ce103d7d8eff6a6883d467846259f29d13bc5133da43008711459ac3e945c35`，不推断写者或沿用整组15/15结论。当前exportSelectedZui仍检查原asset board/bridge/document并走capture/reconcile；后继字节不授这9个缺官方身份的草稿导出权。详见[逐源/原回执对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/source-owner-review-r01-consumption.json)。

| 沿既有AN-SQ-V02/V04的责任 | 本次核实边界及最小接续 |
| --- | --- |
| 图标与实例身份 | `createPrefabIcon`已设置absolute，layout/semantic icon刷新按实际board算glyph；保留原21处设计覆盖与SVG，不以rawRPC无效果认定正式Plugin失败、不反转SDK顺序。 |
| Timeline再生成 | `nativePainterSourceParts`虽有timeline-strip属性映射，`nativePainterContent`没有Canvas/timeline-strip生成分支；`refreshNativePainterContent`先删除识别的parts再创建。缺支持/身份/拓扑须在任何破坏性操作前拒绝；不调用该入口冒充Timeline已实现。此结论为静态分支，未运行重现。 |
| 正式source-owner | generic `prepareRealOwnerInput`和`validateRealOwnerExportSources`已存在，旧“源码只固定LevelEditor”仅历史。9root缺官方asset/document/bridge/immutable guard，不能对mutable草稿补签；原单Canvas不吸收外header/outliner/fixture兄弟图元。保留6/1/8 controls、animation_timeline_slot及6原events/routes。 |
| 共用投影与生命周期 | Parent/原public owner在asset-renderer/control-content/native-painter-content/bridge projection路径接实际allocation、字体测量和既有TimelineModel/CurveModel/domain；共同plot rect含实测value-axis inset，局部坐标加origin一次、inverse先减。52只属旧fixture，范围/游标/源revision/选择/分配/字体变化应刷新同一实例，不烘焙Hero端点或另建Curve/main/player/schema/global handler。 |

[原owner逐按钮映射与invalidation合同](E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/animation-sequence-source-intake-r01/painter-invalidation-review.json)已给责任路径，本轮不复制提案或改公共源码。正式outer composition、Canvas消费、Curve源、持久key身份/gesture/history、真实clock/pose/GPU及normal family producer仍缺；原r06三图只是历史fixture，frame15/0.5s无seek/present ACK。AN-SQ-V01～07、官方导出/正常EXE/144DPI/双端≤1DIP仍open，passed/accepted/captureReady=false。仅现3文档接续和必要静态证据，无Rust/插件功能执行、query/intake/服务/桌面或新消息，释放0B。

## Sequence同步时间列复核AN-SQ-V02/V04-rev182

接续Parent对rev180/r06的实测，原两view都标0–2s，但Timeline以Canvas左4/右20作绘图区，Curve以左52/右20作绘图区，导致0s差48逻辑px、2s同列。这是既有源审阅投影的范围/inset不一致，r06保存重开RGB一致仍保留原结论。当前先读rev181、核对297个r06对象全同，再在原三块板/九个实例修改78处既有painter几何，保存、关闭、fresh重开为rev182；3个main、9copy/216refs、其余1630对象及target的非几何字段保持。曲线所有源点完全未动，没有新增Hermite采样线。

本组同步Sequence视图明确共用`range=[0,2]s`，`x=plot.left+(t-range.start)/range.duration*plot.width`。当前`plot.left=Canvas.x+52`、`plot.right=Canvas.right-20`；52由4边距、44值轴标签栏和4间隙组成，Timeline给该值轴空间保留同一时间列。重算Timeline标尺、0s点/标签和0.5s游标，以及Curve网格；末端点保持。Curve值域仍[0,6]、top12/bottom28，网格/值标签按原源点相同y投影更新。文本盒的末端约束与数据坐标分开。此合同仅适用于这组同步组合；独立Curve可有自己的range/inset，生产适配须按实际字体测量值轴栏，并把同一plot矩形提供给两view，不向通用模型硬塞52常量。

实际`AnimationTimelineFoundationView`已有range/playhead；`AnimationCurveFoundationView`只有selected_track_id/curves，没有range/inset。此处[0,6]是审阅view的瞬时值域，不是source字段或已接通的host状态。可复用调用合同为现有`CurveBounds::new(time_range.start,time_range.end,0.0,6.0)`和`CurveCanvasTransform::new(bounds,CurvePoint::new(plot.width,plot.height))`；仅在该同步组合由同一owner把Timeline秒range传入CurveBounds，渲染加plot原点一次、hit test先减原点。生产owner传入view bounds/测量plot和事件逆变换的调用仍未交付，不据本板启用编辑或seek。

| 逻辑尺寸 | 两view 0s中心的文档x | 两view 2s中心的文档x | 0.5s视图游标x |
| --- | --- | --- | --- |
| 1280×800 | 284 | 1248 | 525 |
| 900×620 | 1804 | 2388 | 1950 |
| 640×520 | 3004 | 3328 | 3085 |

[原实例投影读回](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-axis-10/board-application-r07.json)及[六张原图/三张r06→r07诊断差异逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-axis-10/visual-review.json)记录三个board内save/reopen RGB差均0。API几何最大误差0只证明本次设计投影；不是EXE/1DIP/native150%DPI。既有`CurveCanvasTransform`仍接收局部plot宽高，调用方加一次origin，inverse先减origin；Timeline模型本身不持屏幕原点。正式owner导出/生产Canvas/手势和实际求值/clock/frame ACK仍是原AN-SQ-V02/V04的open部分。

[当前源对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-axis-10/source-reconciliation.json)保留r06当时23/23；后继20/23差异正是AN-SQ07重绑与AN-SQ08时间修复的三源2e0d8676/bacd4f59/cb89de35。`write_sequence_asset`正文逐字保持，0–2s/30FPS/Vec3/Hermite及frame15→0.5s假设仍成立，仅为源码推导，夹具执行/实际资产加载仍0。没有改旧packet、回滚源码或新建/重投Jenkins身份。Shared的r06保存rev2、退出后rev3保留历史，本轮另观测rev5且未sync；不据新revision推断main有效样式引用/传播通过。原public handoff ACK复用，AN-SQ-V01～07整体及EXE/DPI验收继续open。

## Sequence帧秒一致性与原测试输出修正AN-SQ06/08-r01

继续真实scene Sequence接线，发现源编译器允许0.25/0.5等大于Real::EPSILON的有限FPS，但Session和Add/RemoveKey都将除数`.max(1.0)`，0.5FPS的frame1因此落在1秒而非2秒；duration_frames按0.5算帧数，游标/范围与原key秒不一致。已仅修改两个既有`frame_to_seconds`及各自sanitizer：去掉1FPS下限，合法范围与编译器一致为finite且大于f32::EPSILON；0/负数/NaN/inf和非法极小正数回退30，raw source FPS及其编译错误不被修正。没有改Runtime/compiler、事件DTO或添加公共时钟。

原Session [tests/cases.rs](../../../zircon_editor/src/ui/animation_editor/session/tests/cases.rs)的before SHA `cd2dbb0d…`匹配r01一行候选，已把未来磁盘夹具根改为已登记`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-session-tests`；原3项文件测试正文保持，本轮未创建夹具或触碰旧E:/ZirconBuilds。新4项`sequence_frame_time_`回归全在内存中使用真实AnimationAuthoringDocument、Session、mutation、二进制codec和history，覆盖0.25/0.5/1/30FPS的范围/游标/key/Playback秒、Add/RemoveKey往返、事务undo/redo后的共享读句柄，以及非法/epsilon/subnormal输入保留fallback与编译诊断；静态定义4、执行0。[当前源码与检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-time-r01/validation.json)不作为求值/GPU或EXE证据。

AN-SQ07原位重绑分支与五测试保持；其原mutation SHA `45ccb7fc…`是上轮历史，本轮只追加时间helper两行，当前指纹另记，不重写旧回执。新聚焦候选仍为zircon_editor单包/defaults-off/features=[]/Windows host lib/locked/jobs1/threads1/test语义，仅filter `ui::animation_editor::session::tests::sequence_frame_time_`；实际wrapper profile/env/源资源准入仍交原owner，未调用编译或DryRun，不并入旧Editor或Runtime identity。AN-P05三catalog条目在SHA `9556e66d…`已实际可读，后续依赖是canonical main/库发布及原导出，不能继续称尚未登记，也不能把这三条目等同本域Timeline/Curve/Transport三设计主件。原Shared/公共资产/六事件及AN-G13、AN-P04和Parent sampler未改；正式导出、真实clock/预览/EXE/DPI及全族验收继续open。

## Sequence属性重绑保留实体目标AN-SQ07-r01

当前真实`AnimationDocumentMutation::RebindTrack`有持久身份缺陷：同实体只剩一条轨道时，原`mutation/sequence.rs::rebind_track`先移除轨道及空binding，再以`target_id=None`重建binding，丢掉已有stable target。Runtime `animation/sequence/compiled.rs:60～63`先按target_id解析、再按entity_path回退，因此属性改绑可能改变实际目标。UE `FMovieSceneBinding`也将ObjectGuid与Tracks分开保存；本修复只沿Zircon既有可选target_id语义，不新增GUID、key字段或schema。当前结论来自原源码数据流，尚无Rust执行复现。

已在原[sequence mutation](../../../zircon_editor/src/core/editing/animation_document/mutation/sequence.rs)加同实体分支，先沿原目标冲突检查，再原位修改property_path；目标ID、channel/key/tangent、track顺序及其余binding保持。跨实体仍采用既有目标binding自己的Some/None，创建新目标binding继续None，不把原实体的target_id搬到另一个实体。调用仍是原EditorAnimationEvent→document mutation→AnimationEditCommand/CAS→同一document history，不改公共host/event、capability、Runtime或ZUI。AN-SQ01的持久key身份/selection随rebind变化没有因此解决。

[五项叶模块回归](../../../zircon_editor/src/core/editing/animation_document/tests/sequence_rebind.rs)覆盖单轨Some/None与真实binary codec往返、同实体轨道顺序/无关binding、no-op/缺源/冲突、跨实体目标权威，以及真实文档事务/源编译/序列化/一次undo/redo/dirty。只由原mutation叶文件挂载，AN-G13的cases.rs与三条条件回归字节不变。两源stdin rustfmt逐字一致；[当前源与验证回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-rebind-r01/validation.json)保留静态5项、执行0。最小后续scope为zircon_editor单包、defaults off/features=[]、Windows host lib、test语义、locked/jobs1/threads1，filter `core::editing::animation_document::mutation::sequence::tests::sequence_rebind_`；实际profile/env及准入仍由原受管入口核对，不创建/扩充旧Editor或Runtime请求。本轮无新Penpot/桌面/服务/编译/清理；AN-SQ07运行回归及正常EXE仍pending，整个窗口族passed/accepted=false。

## Montage Preview All原实例文字修正rev181

按Parent的MT-S01具体复核，先读当前rev180，再仅在原Sections板`14920d87-e194-5b50-99ce-6379f2233b26`的leaf `0495933a-aca6-806e-8008-d7089e0334bd`将`Preview All Sections`改为UE `SAnimMontageSectionsPanel.cpp:226`的`Preview All`。没有重放rev140或复制master。原Shared Button component `6ddc248b-bd35-80c5-8008-d6c8c01096f3`、main `6ddc248b-bd35-80c5-8008-d6c8bd9b057f`、text shapeRef `6ddc248b-bd35-80c5-8008-d6c8bdb3e2f8`及9个head/flow保持；53个外框、其余1707对象和原3个component记录均未变。目标仅content与positionData变化，除文字外的样式/metadata/touched覆盖保持，当前Shared实际rev4且本轮未同步或写入。

[原生保存关闭重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-caption-09/native-caption-r01.json)到rev181，保存与fresh重开data SHA均`bad696d100f9a7fb281dfa42232ca4e42709b75993f7c67ce1ae7ba33df68f44`；[三张当前原图逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-caption-09/visual-review.json)及[重开原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-caption-09/sections-caption-reopened.png)均已实际打开。板内修改像素只在目标标题范围，保存/重开板内像素完全相同；整个workspace有状态栏差异，不能称整张PNG一致。原9组顺序、None/自环菜单保持，71% fit/DPR1仍非EXE或实际150%DPI。

[文字行框与覆盖复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-caption-09/text-frame-review.json)中目标166×28分配框内测宽122.609375→67.0625 DIP，修后行框高16、越界0；九个Button文字框均无越界。但14个原Noto Sans SC文字框仍有y1=-2的2 DIP名义上沿越界，编辑前已存在且本轮未改，登记`MT-S05-current-CJK-linebox`；这不是实际引擎字形裁切证据，也不通过整板1DIP。[源检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-caption-09/source-validation.json)保留rev140到本轮起点的23文字差异与91非空/18空样式引用，非空引用全部解析；52源中51匹配历史、52匹配Parent当前观察，唯一历史变化为catalog `4c2f849c…`→`9556e66d…`。旧1338+3main保护及20未覆盖仍按原范围保留。本轮只关闭MT-S01标题子差异，其余MT-S01及MT-S02～05、官方导出、真实Montage/clock/GPU/EXE/DPI继续open；无Cargo/Jenkins/query/服务/桌面或公共源码修改，passed/accepted=false。

## Sequence源夹具三尺寸设计rev180

原file/page已新增三张可编辑Sequence数据审阅板，当前rev180/1708对象/3原main，isShared=false。每板原生实例化Timeline、Curve、Transport各一份，共9个owner copies、216个逐叶shapeRef，未复制main。原rev148的1410个非Root旧对象及三component记录完整比较未变，Root仅增加三板；新对象共297个。[最终实例读回](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/board-application-r06.json)记录全部身份、覆盖及几何。这不是正式源导出或Widget消费；旧14/6/3计数保留历史，当前本域Transport/Timeline/Curve为17/9/6。

| 逻辑尺寸 | 新board UUID | 最终重开原图 |
| --- | --- | --- |
| 1280×800 | b1d6a1f0-dc13-5d7f-b005-c42a7921706d | [1280原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/sequence-1280-reopened-r06.png) |
| 900×620 | 7fa9a6a5-de18-5039-8eeb-8bb49259ca5f | [900原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/sequence-900-reopened-r06.png) |
| 640×520 | eb296a7d-15e4-5417-93ba-61134650f4dd | [640原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/sequence-640-reopened-r06.png) |

数据只读自原Session测试`write_sequence_asset`：Hero Sequence、Root/Hero、Transform.translation、2秒/30FPS、两个Vec3 Hermite key及原切线。0.5秒/frame15和选中是派生审阅状态，[源夹具与指纹](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/source-fixture.json)明确未运行测试、未生成/打开资产、未连接World或骨骼pose。Curve仅画六个原端点和导数说明，没有另写插值器或伪造Hermite求值线；设计点UUID不是持久source key。原AN-SQ01～06的身份、AddKey与runtime差异仍open。

实际`SSequencer::ConstructOutlinerAndTrackArea`将Transport置于outliner底部，本轮已按此调整；Curve是可选面板打开后的审阅组合，不宣称完整默认Toolkit。外层保持flex与原Shared样式引用，数据/图标painter使用坐标例外。key/游标是wrapper内的审阅兄弟节点，不把Timeline Surface外组合等同原单Canvas源边界；标尺与lane painter随实际Canvas宽度更新。六按钮保持28 DIP与原2 DIP间隔，不借缩小按钮关闭旧StateMachine窄栏差异。

首轮隐藏key图层、1280旧宽、640说明裁切与Transport错位均留有失败图。r04 guard误用path原始x/y并计入隐藏caption；r05仅position被按钮flow回排，零保存写入、rev178哈希未变，不能算修复。r06只将三个新copy的21个可见glyph设为绝对定位并原生居中，保留路径、18个隐藏caption和六事件；两次真实保存后rev180，落库selrect对目标最大误差0。该数值仅覆盖这些图标，不代表整窗/文字1DIP；主件未改，正式codec的实例painter刷新交原pipeline owner。

[原生r06保存关闭重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/native-transport-glyph-r06.json)的data SHA为`098cba1488c0a10eb2cbb46fa030930027ea239aa059f51c0a74fab741ee19be`。六张最终图及失败/无效果图共30张均实际打开；[逐图记录与AN-SQ-V01～07](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/visual-review.json)保留完整Toolkit/分隔与同步滚动、生产Canvas/gesture/focus/undo、持久key/正式导出、clock/求值/帧、EXE/DPI/并排diff、公共发布与聚焦测试缺口。71%/98%/134% workspace缩放、DPR1不是实际150%DPI，也没有运行时按钮效果。

Shared在r06保存重开期间为rev2且完整data保持；浏览器退出后独立只读已见rev3，41对象/4原main/34色/3typo，34/3集合逐项未变，实际非空shape样式引用仍0。本域未写Shared或同步新版本，不继续称当前库rev2。[源验证](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-source-08/source-validation.json)保留首次“退出后库仍同hash”的断言失败及更正。23份域源码指纹仍匹配；原ZUI/catalog/Runtime/AN-G13/AN-P04未改，4213入口本轮拒绝连接，未跑插件/Rust、旧QueryOnly或新capture，passed/accepted=false。

## Sequence源投影的可用性边界r01

继续设计前，对原Sequence而非骨骼Clip检查了真实接线：[共享消费六项合同](animation-shared-controls.md#sequence源适配与生产接线r01)与[23源指纹/调用范围](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-consumer-r01/sequence-source-contract.json)保留确切文件与symbol。原`AnimationSequencePaneBody`只有六个control和`animation_timeline_slot`，retained生成的HybridSlotAnchor没有source node/instance；当前foundation只在已查定义/helper/测试内调用。现有轨道和curve数据投影存在，生产canvas/gesture/原owner codec消费尚未证明，不能把anchor或TimelineStrip input标志作为真实编辑器接入。

设计需继续保留三个本域真实差异。第一，Timeline/Curve投影key ID当前是track path加time bits，移动时间或rebind会换ID；没有持久key字段，不能让选中高亮/焦点/undo跟着旧ID漂移，也不能借临时UUID掩盖。第二，AddKey当前复制末key值并清新key切线，空轨为Scalar(0)，不是当前求值/当前属性捕获；Record继续无有效动作，不预先展示保持曲线形状的插键。第三，Session scrub/playback只更新frame/flag，未给runtime求值/实际时间/帧回执；播放样式和playhead文本不能成为预览成功证据。

源测试里的Hero Sequence是2秒/30 FPS、Root/Hero的Transform.translation、两个Vec3 Hermite key及切线；它是未运行的源夹具，不是已打开的用户资产或骨骼动画。Scalar/Vec分量可由现foundation投影，Bool/Integer/Quaternion返回无标量curve；空集合的原因展示仍须设计/接线，未新增类型转换、clock/player/World、schema、组件UUID或Penpot图板。本轮只读API见rev148/1411对象/原3main，沿用此前关闭重开/13图记录，不追加保存实验或新图身份。正式源导出、真实三尺寸/150%DPI、同状态并排diff及全族验收继续pending。

## 状态机TreeRow实例几何接续rev148

原file/page在rev146基础上完成本域四个StateMachine板的八个TreeRow设计实例修正，当前rev148/1411对象/原3个main。八个外框已是318/223/158 DIP宽，内部painter却仍缓存240 DIP导入几何，导致窄板右侧操作槽越界。此次仅以真实`tree_action_button_rect`、`tree_action_icon_rect`与`tree_label_rect_for_variant(false)`及原tokens重算copy图元：操作槽20、图标16、右内距12、槽中心距32，标题x=52、右保留60，文字宽依次206/111/46。native update-position移动原SVG子树，保留源图形与笔宽；实例内部约束为right/leftright，外层flow及UE 25/55/20目标未改。[源依据与尺寸](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-overflow-07/source-contract.json)、[精确操作计划](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-overflow-07/layout-plan.json)均保留。所有生成painter图元属于组件内部绘制几何，不把普通面板布局改为手工XY。

当前[原生r02保存关闭重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-overflow-07/native-layout-r02.json)记录相同data SHA `393678dc8eeccb9a77e07cc3e5d0c84adf977905869c8b8448703a38c4ef7ef6`。对象与组件UUID、componentFile/componentId/shapeRef、原文字/样式覆盖及全部184项受控记录的plugin metadata保持；Shared rev2未改。服务端几何与source公式最大差约0.000000477 DIP，仅为这些图元的数值比较，不能据此声称整窗或文字达到1DIP。Eye/lock仍是通用painter绘制，没有状态机业务事件，不把可见图标算成有效操作。

| Triggered夹具尺寸 | 本轮最终重开原图 | 当前限制 |
| --- | --- | --- |
| 1280×800 | [1280原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-overflow-07/state-machine-1280-reopened-final.png) | 71% workspace缩放，非正常EXE |
| 900×620 | [900原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-overflow-07/state-machine-900-reopened-final.png) | 98% workspace缩放，未覆盖真实输入/焦点 |
| 640×520 | [640原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-overflow-07/state-machine-640-reopened-final.png) | 134% workspace缩放；Transport末项裁切、GraphRef行框越界仍open |

13张本轮PNG均逐张实际打开；三个最终board内部关闭前/重开像素相同，workspace外部状态标记/检查器另有差异。1280比较条件板只完成本轮八实例范围内的几何读回，没有新增该板截图，不冒称完整四状态画面验收。DPR1/browser fit不是实际150%DPI。原始[640修正前图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-overflow-07/state-machine-640-before.png)与rev147失败实验图保留。

长`condition.graph.zranim`原生auto-height实验并未换行，仍测得116.625宽、分配110.4，超过文字行框6.225 DIP；该夹具末端侵入右padding，不能仅因字形接近可见就通过行框/长引用可达性。rev148已仅恢复四个引用原fixed高度、sizing和内容，未插入换行或缩字号。六个Transport按钮固定28 DIP，加原六个2 DIP间隔及Fill需180 DIP，而640左栏只有158 DIP。两项均交Parent现Dock/overflow/完整引用focus入口，保留UE默认目标，不以更小按钮或填常量关闭差异。

正式导出仍有明确依赖：八个linked copy继承的`tree-row-authoring-painter.parts`是主件图元ID，与本地copy ID不符；实际shapeRef能完整映射六部件，metadata当前仍缓存240宽，未篡改任何immutable original/generated guard。原public owner须沿[现有刷新与来源映射交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-overflow-07/painter-reflow-handoff.json)核对canonical来源、copy映射及受控设计增量，再复用`refreshTreeRowContent`/`layoutTreeRowText`/`refreshAssetTextLayout`。本轮未执行这些正式plugin入口，不声称当前copy可直接导出，也不重写guard来取得通过。共享源/catalog/ZUI、三专责main、AN-G13/AN-P04和Runtime均未改；未重查两笔Jenkins原请求、未编译/新建请求/source capture或占桌面。原SM-R01～08及全族passed/accepted=false保持。

## Montage Sections源状态复核rev140

Parent本轮rev80/16:19:31Z与Bool sampler/AN-P04交接按历史时刻保留，未重应用旧patch。原file/page已接续真实保存、关闭、fresh认证重开到rev140/1411对象/原3main。新增一个1280×640源状态复核board 14920d87-e194-5b50-99ce-6379f2233b26，位于x1600/y5200；共53新对象，九个按钮均由Shared库原生instantiate创建。当前本域copy 23、Shared外链copy 25；不新增main、发布库、替换原资产或修改Runtime/handler。这是Sections局部复核板，完整Toolkit/三尺寸/产品验收继续缺失。

两张状态来自只读UE源：AnimMontageFactory::EnsureStartingSection生成Default @ 0秒/Next=None；第二张自环由SAnimMontageSectionsPanel::SetNextSectionIndex(0,0)允许行为推导，未运行UE或加载Zircon Montage。OnGetSectionMenuContent仅在已有Next时显示Remove Link，因此左菜单只有Next Section/Default，右菜单才有Remove Link。工具栏顺序为Create Default/Clear/Preview All Sections；每条链为Preview/Default/Next。Child Montage禁用编辑项而保留Preview的合同明确标在板上，未伪造已执行的禁用、菜单焦点或历史动作。

| 当前真实依赖/身份 | 本轮实际消费与限制 |
| --- | --- |
| Shared file b4a1e3ce-9e6a-81fe-8008-d69d607aac12/page b4a1e3ce-9e6a-81fe-8008-d69d607aac13 | 只读发现已rev2/isShared=true/41对象/四原main/34 colors/3 typographies；17:37:45.684Z为样式记录修改时间。旧rev1/0样式断言先失败，写入前已对账，不能继续称公共样式为空。 |
| Button component 6ddc248b-bd35-80c5-8008-d6c8c01096f3/main 6ddc248b-bd35-80c5-8008-d6c8bd9b057f | 九个native linked copy保留source file/component/shapeRef及子文字来源，未复制main；每个copy ID、文字覆盖和边界见[实例读回](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-05/board-application.json)。Shared默认灰色按钮与UE FlatButton.Success/Danger及FontAwesome图标仍有差异。 |
| Caption c0224e78-f918-526f-8026-a76cd8f347af及已发布颜色 | 新文字存原typographyRefFile/Id，保留10.666667/1.2；本地Noto Sans SC font-id/family/variant覆盖明确记录。新frame/文字颜色引用原editor.surface.0/.1/recessed与editor.text.primary，91条非空嵌套样式引用读回（另18个Button原空引用不计），非新主题。 |
| Tokens与库保护 | editor_tokens.zui SHA dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743匹配样式来源；Shared完整data hash 8c92ed01fdd17984271d5a2e0fce383b6dd9fb36cd8b0a5b02c76b9c22d331bb前后一致。未点击共享库更新横幅或批量同步旧实例。 |

[r04原生保存/重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-05/browser-review-r04.json)记录HTTP保存、同data hash、page errors0、browser PID38220已退出、native exit0。[关闭前原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-05/sections-review-r04-before-close.png)与[重开原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-05/sections-review-r04-reopened.png)已逐张实际打开，二者PNG SHA均493f3930676c5e672283454013f962788428831e2ca776373f41e047112ee571；1600×1100隔离工作区/71%缩放，不是EXE或实际DPI。r03两张逆序负例亦实际打开，保持原文件；r04修正新板paint order并原生reflow，读回九组多子项flow的逻辑顺序与位置顺序相符。

原三main记录及已登记1338个旧对象比较未变；源保护覆盖不等于全页1358个旧对象逐一通过。Root的新增board列表预期变化，另19个遗留Transport对象没有独立rev105内容比较，保留其身份/现状并列入[覆盖限制](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-05/validation.json)，不清理未知对象或声称全对象保持。52份UE/原资产/源码/theme/catalog/Button指纹无漂移；原六Transport事件、15个Montage事件、AN-P04两文件、AN-G13三文件/compiler及Parent四Runtime文件均未写入。

同轮补读的真实行为见[source-contract](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-05/source-contract.json)：Montage scrub走JumpToPosition→SetPosition(time,false)，不转发bFireNotifies；PostRedoUndo重建后用SetPosition(PrevTime,false)恢复原位置；segment拖动在首次更新触发pre/事务并CollapseAnimSegments、drop触发post，和section时间在!bIsDragging提交的路径不同。Absolute/Relative/Proportional关联、Notify State起止端的link mode/slot同步、parent/child segment菜单及替换/批量拖放策略也不能用泛型section矩形代替。均未据此实现新模型、播放器或开启capability。

[逐图复核/差异MT-S01～05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-05/visual-review.json)保留UE按钮风格/图标、通用Menu/overlay与焦点输入、真实资产/采样/pose、完整三尺寸及DPI/EXE/diff、样式全局A/B/reset到产品等缺口。4213本轮仍拒绝连接；没有插件功能导出、Rust执行、新Jenkins请求/query、桌面或服务操作。原两请求pending及SM-R01～08保持，passed/accepted=false；原r01/r02编辑符号失败与预检失败保留，当前只复用既有原生content/geometry接口，不归咎正式Plugin。

## Transport源边界与流布局rev105

同一真实file/page当前rev105/1358对象、原3个component/main UUID、isShared=false。Transport draft2已按原`.zui v2`修成六按钮+Fill，clock/status原UUID移到外wrapper历史观察区；14native Transport copies及原六按钮/descendant ID保持。Timeline draft2、Curve draft1各原main只修flow/sizing，不改变合同；6+3copy保留原sourceRef和全部文字/样式覆盖。Shared rev1完整data hash保持、53个无关对象未变，公共源/原ZUI/全局handler均未改。原rev80“不改三个main”只描述该历史阶段，不适用于本轮本域布局修复。

源边界review board为`a0218fa5-6d64-5f7b-9dc4-2728371fb09f`（1280×240），native copy`079c29bb-22cd-807b-8008-d6f4da981985`；Fill`eef53cdd-5e92-53d0-ab13-41e71e2d7556`；archive host`f91f0f56-887b-5987-9fad-f0ef36ce01a6`。组件/file/page/源SHA/原六事件及14实例映射见[source-mapping](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/source-mapping.json)，完整[原生操作与保存关闭重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/browser-migration-r13.json)。新review保留旧clock/status作为明确标注的设计历史读数，没有真实clock或播放效果。

本轮校正21个外层、63个内部flow及8个原Timeline/Curve main顺序，另8处源sizing；9copy的150个数据图形按真实Canvas视口恢复矩形/Point/Rect，服务端读回矩形误差0。12处段落对齐使用原生set-touched/text-content-attribute，使重复native sync保留原文字及样式覆盖。数据Canvas是坐标例外，普通布局仍为flow。raw-RPC顺序修复不代表正式Plugin API缺陷，不把该矩形读回结果当整窗/文本1DIP。

| 三尺寸/状态 | 最终实际原图，均已打开 | 仍未通过 |
| --- | --- | --- |
| Transport源边界1280×240 | [关闭前](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-source-r13-before-close.png) / [重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-source-r13-reopened.png) | 未发布，原9旧copy逐叶event metadata原本即缺，正式codec须按canonical shapeRef无损解析 |
| Shared空态1280/900/640 | [1280](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-22192a1e-86c1-43c9-8ecf-994d44c4ee5c-r13-reopened.png)、[900](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-42f2a33a-c0f9-4d00-9ae2-95e6ef72e2c7-r13-reopened.png)、[640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-dd27f2e0-1c55-45a2-aa5b-ebe28371473f-r13-reopened.png) | 标尺末端/曲线纵轴/中文空态可见，未加载真实动画、GUI gesture或时钟 |
| Montage空态1280/900/640 | [1280](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-c0af7cda-882d-59b1-8d0e-df0bca32c639-r13-reopened.png)、[900](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-83826cfb-3a1d-5056-ace5-9497f37d4eb4-r13-reopened.png)、[640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-f49cf095-55f3-53d1-8e42-310c80075626-r13-reopened.png) | 时间标尺/空轨道/分段区可见，分段跳转及真实资产/预览仍缺 |
| BlendSpace夹具1280/900/640 | [1280](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-d727a5e1-2ffa-5283-adf2-478d394e887c-r13-reopened.png)、[900](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-167949d2-6b88-5729-8113-e8d2ac75f97a-r13-reopened.png)、[640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-7da83ecb-a904-5fc1-bd45-b092ca4975ac-r13-reopened.png) | 三点与1e-20范围来自原source fixture，未求真实权重或GPU pose |
| 状态机Greater1280、Triggered1280/900/640 | [Greater](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-1a5971e6-93f3-5887-b2f1-a2bfd7a862d1-r13-reopened.png)、[1280](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-4c1c84d4-32fe-5f4e-8f2b-113e0fa8ff0b-r13-reopened.png)、[900](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-36a967ac-8978-550f-8cd7-2c0577be6ca3-r13-reopened.png)、[640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/transport-consumer-6bc1b99a-9c15-5e0f-80e9-689b787f47cf-r13-reopened.png) | 640左栏Loop、900/640树actions裁切；GraphRef窄窗贴边，合法value providers与真实编译/活动态未接 |

15张原图的逐图意见、SHA、尺寸/71%98%134%画布缩放、历史负例和差异责任见[visual-review](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/visual-review.json)。r01～r12的失败保留：调用符号/脚本错误、文本段落覆盖、仅modifiedAt变化的错误guard、旧copy原缺metadata、旧points导致渲染边界重算、r12普通map破坏原生几何类型的HTTP500均不计通过；r13使用原生Point和原Rect写入获HTTP200，rev105保存/关闭/fresh重开同hash，browser PID33256已退出、native exit0、page errors0。只有r03的16张、r09和r11各4张旧图实际打开，其余旧图不冒称复核。

[共享消费最新合同](animation-shared-controls.md#源边界与widget消费接续rev105)接续Parent已落地Bool/Step sampler及Widget sample/finish候选；17项Widget/3项公共sampler回归未运行，无clock/Stop/present。插件4213于17:06:56Z实际拒绝连接，未启动服务。全部本轮PNG是1600×1100隔离headless工作区，不是正常EXE/原生exporter或实际DPI；源导出、样式A/B→产品、输入/焦点/撤销、双端并排diff及≤1逻辑像素门槛仍open，passed/accepted=false。

## 状态机作者区rev80

本轮在原file/page新增四张可编辑业务板，最终服务端保存和fresh认证重开均为rev80/1342对象/原3个main；新增16个原生copy，原862对象除根board列表外均未改，三个main完整子树哈希不变，Shared rev1数据和正式库连接保持。实际[保存重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-03/browser-review-r05.json)、[新对象/引用读回](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-03/reopened-r05.readback.json)及[八张原图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-03/visual-review.json)均已形成。rev31公共消费板保留历史，不再称为当前页面revision。

| 源夹具状态 / 尺寸 | 实际board UUID | 实际重开PNG |
| --- | --- | --- |
| Greater / 1280×800 | `1a5971e6-93f3-5887-b2f1-a2bfd7a862d1` | [比较条件](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-03/state-machine-compare-1280x800-r05-reopened.png) |
| Triggered / 1280×800 | `4c1c84d4-32fe-5f4e-8f2b-113e0fa8ff0b` | [Triggered](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-03/state-machine-triggered-1280x800-r05-reopened.png) |
| Triggered / 900×620 | `36a967ac-8978-550f-8cd7-2c0577be6ca3` | [900尺寸](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-03/state-machine-triggered-900x620-r05-reopened.png) |
| Triggered / 640×520 | `6bc1b99a-9c15-5e0f-80e9-689b787f47cf` | [640尺寸](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-03/state-machine-triggered-640x520-r05-reopened.png) |

三列依据UE `AnimationBlueprintEditorMode.cpp` 的`Stanalone_AnimationBlueprintEditMode_Layout_v1.6`：25/55/20，左侧预览/导航各半，右侧Details/预览参数各半；本轮展示中心编译结果已打开的80/20变体，不能把UE默认关闭的编译页签说成默认打开。外壳菜单/Dock/完整Toolkit仍沿Parent公共框架接入。

业务数据严格来自[已有条件回归source夹具](../../../zircon_editor/src/core/editing/animation_document/tests/state_machine_conditions.rs)：Condition editing、entry Idle、两个GraphRef状态Idle/Run、引用`res://animation/condition.graph.zranim`、Idle→Run混合0.2s、signal Greater Scalar(0.5)，以及已修复mutation产生的Triggered/value=None。该GraphRef未解析、夹具未作为产品资产加载、三条Rust回归仍执行0次；设计中的Entry不是实际运行活动态，蓝色条件块也不是运行时权重。UE状态/转换brush和活动色依据、源hash及完整差异见[source-contract](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-state-machine-03/source-contract.json)。

每板实际使用Shared Button×1、TreeRow×2和原Transport×1，均为native linked copy，记录source file/component/library revision/copy/shapeRef及覆盖值。TreeRow的Idle/Run只是文字child覆盖，catalog未公开text参数，原eye/lock绘制也没有状态机业务语义。Details保持只读源摘要；四个已发布基础main不能提供PropertyEditorRow要求的Field/Dropdown/NumberField等value槽，不造重复Field或虚构UUID。状态节点/转换是本域Penpot数据面形状，不新建通用graph main，也没有接上真实选择/拖动/上下文菜单/焦点/撤销。

首轮发现raw-RPC构造的新Flex列表与Penpot原生paint order相反；r02仅反转64个新容器的内部列表，并给16个copy根恢复fill尺寸，四个新Transport copy暂以row-reverse覆盖表达源按钮顺序。Shared Button固定220宽文字child仍被96宽host裁切，r05经实际公开`workspace/update-dimensions`缩至76×26后可见，UUID/shapeRef/文本覆盖不变。r03/r04内部编译符号不可调用均在写入前失败，HTTP写0且浏览器退出；失败回执和r01/r02原图保留。正式插件的`appendChild/children`已有natural-child-ordering适配，不能将本域raw-RPC错误直接宣称为公共importer缺陷。

本轮另外核实[Transport源边界](animation-shared-controls.md#状态机消费中的源边界与实例尺寸)：原ZUI是六按钮+Fill，草稿main内的clock/status文字不属于该源。该rev80时点原main未改、13copy迁移待办；后续rev105已完成本域Penpot边界修复及14copy传播，正式source导出/发布仍由原owner接续。三种左栏宽318/223/158分别裁切status、clock/status、Loop/clock/status；640的长GraphRef文本也裁切。不能以root尺寸适配等同紧凑布局或响应式通过。

最终8张1600×1100全工作区原PNG均已实际打开；board fit为71/98/134%，deviceScaleFactor=1，当前八项差异仍open。没有官方native PNG exporter、EXE帧、真实DPI144、同状态双端并排/diff或≤1DIP测量；真实菜单/页签/停靠、弹层和焦点返回、滚动首尾及域内操作回归仍待接通。未执行新编译、未提交新Jenkins身份、未动服务或共享桌面；`passed=false/accepted=false`。

## 公共基本组件受控消费 r03

接续Shared rev1真实四main，同一已分配file/page新增受控board `9bc129dd-b697-4a5c-9714-d6ab3a46ed2d`，当前rev31/862对象；Button/SearchInput/TreeRow/PropertyEditorRow都是正式跨库linked copy，保存关闭fresh重开后sourceFile/component/shapeRef和覆盖保持，Button native reset有效。原BlendSpace/Montage/shared空态数据保持，不重建基础控件或复制原三个专责main。具体身份见 [公共消费节](animation-shared-controls.md#公共库四主件消费-r03)。

内容为“编译动画图”文字、动画参数搜索文字及现有Graph测试字段名grounded的静态设计引用，未加载运行资产或绑定命令，不当作可执行面板。Property只核查label/身份；required value槽接受的七类控件不在当前四个发布main中，不能以raw Text或自造Field替代。真实参数区后续复用这批公共件和owner发布的值编辑器；动画adapter提供typed参数/history/真实preview回执，640紧凑候选不改变UE默认布局。

rev28的Search/Tree长文字换行越出原单行框，两图保留。rev30短值已保存，但Search旧positionData仍画长文本，仅校验API content不足。rev31沿既有字体流程失效同一文字的derived position-data，由真实渲染器测量生成，fresh重开后短中文/Tree单行显示；未手写坐标或修改公共Penpot源码。六张1600×1100工作区图已实际打开，逐图意见及两次编辑器探查失败保留。71%缩放不算nativeDPI/1DIP；省略、完整slot/codec/跨族样式、真实效果、正式导出及EXE/GPU证据仍pending。

## 区域与组件实例

外层继续使用现有 Editor shell、Dock、DocumentHost、菜单和工具栏；本域只提供作者内容。正常标题/工具/属性/列表用 HorizontalBox/VerticalBox 及 stretch/min/max。每个长列表只有一个滚动 owner；轨道 outliner 与 lane 共享纵向滚动位置，标尺/lane/曲线共享时间变换。只在时间、混合平面和节点图内使用数据坐标。

| 文档/区域 | flow 树与可编辑内容 | 现有源身份与数据依赖 |
| --- | --- | --- |
| 骨骼 Animation Sequence | Dock 三列；中列 VerticalBox：真实 viewport → 文档页签 → timeline toolbar → 同步 outliner/lane → transport；曲线作为关联文档页签 | 保留 `AnimationSequencePaneBodyRoot`、`AnimationEditorHeaderPanel`、`AnimationTimelineSlotPanel` 和 `animation_timeline_slot`。现有 body 处理的是属性 Sequence，Clip 作者路径须先分配真实 ResourceKind/Toolkit 身份，不能把两种字节格式混用。角色域提供 skeleton/mesh subject |
| Montage | 与 Sequence 共用 Toolkit/时间轴；lane 分 section 边界、Slot/segment、point notify/state/marker；右下 Sections 页签显示实际 next/loop 关系 | 迁移原 `WorkbenchExtensionMontageEditorWorkspace`，保留原始 root/control/event 映射。`WorkbenchExtensionMontageEditorSlotField` 的 Change/Submit 分别是编辑草稿/提交事务；真实 Montage 资产及 Slot 身份合同尚缺，不能把 Intro/Combo 示例保存成用户资产 |
| BlendSpace 1D/2D | 中列真实 viewport 与采样面；参数/坐标输入跟随 flow 工具行；采样面旁显示实际 evaluator 权重；浏览器、Details 走共享 Dock | 复用 `WorkbenchExtensionBlendSpaceWorkspace`、`WorkbenchSampleGrid`、`WorkbenchSampleWeights`、`WorkbenchWeightHeatmap`；`WorkbenchExtensionBlendSpaceSearch`/Preview/Apply 的既有事件保留并接真实过滤/预览/提交。sample 选择身份来自实际资产，不从固定 Run_Fwd profile 构造 |
| AnimGraph / StateMachine | 左侧 Preview 与导航/观测页签；中心 VerticalBox：面包屑 → 数据画布 → 编译诊断；右侧 Details/Preview Parameters 与 Browser | 保留 `AnimationGraphPaneBodyRoot`、`AnimationGraphCanvasSlotPanel` 和 `animation_graph_canvas_slot`；复用已有 Graph/StateMachine 文档命令/历史及 current/last-good 编译产品。状态权重/活跃色由真实实例返回，不按选择状态伪造 |
| Sequencer 适用子集 | toolbar → 时间标尺 → 绑定 outliner 与属性 lanes → playback/view-range 和 transport；曲线与同一资产/时间上下文相连 | 迁移 `WorkbenchExtensionSequencerWorkspace`，保留 `WorkbenchExtensionSequencerFrameRateDropdown`、`WorkbenchExtensionSequencerWorkRangeField` 等控制身份；真实源为 entity/property Sequence。CameraCut/Audio/Event Cues 是否可执行逐轨道证明，不由固定行标签推断 |
| Curve | 关联页签中使用 HorizontalBox：曲线树/过滤 → 曲线数据面；上下工具行及数值输入流式排列 | 复用 `ui/curve` 与 `curve_foundation()`；scalar/vector 组件有实际投影，Bool/Integer/Quaternion 保持各自非 scalar 语义。切换/关闭动画文档时关联 curve owner 更新/关闭，不留下旧源的编辑入口 |

`WorkbenchPanelHeader` 使用既有 title/actions slots；`WorkbenchPropertyEditorRow` 使用 value slot；图标按钮用既有 WorkbenchIconButton。`WorkbenchTransportControls` 六条现有事件分别是 record.toggle、play.invoke、pause.invoke、previous.invoke、next.invoke、loop.toggle。现阶段 previous/next 跳硬编码端点，不能重命名即声称单帧支持。共享 owner 应区分 start/end 与 step forward/backward，并明确现有事件映射；录制需要实际记录目标/数据/停止提交行为，不能仅让按钮变色。

样式只引用既有 tokens/Starship。UE 实际依据含 `StarshipStyle.cpp:5878` 的 AnimTimeline splitter/label/item/header/section marker、`:4136` 的 StateNode 和 `:4217` 的 TransitionNode；其中 section marker 为 11×12，状态图标 16×16、transition 图标 25×25。具体色值是 Slate linear color，映射到 Penpot/ZUI 时必须记录颜色空间与透明度，不能把浮点数直接当 sRGB。现有共享 token 不足时提交具体语义/测量提案，由公共 owner 集成。

## 源字段、控件与命令的具体映射

| 对象/属性 | 已有真实源与运行语义 | 控件和缺失通路 |
| --- | --- | --- |
| State 类型与打开目标 | `AnimationStateKindAsset` 为 Clip/GraphRef/SubMachine/BlendSpace1D/2D；当前 CreateState 只创建 GraphRef | 共享 dropdown 用实际类型及能力门槛；GraphRef 双击打开 Graph，Clip 打开 Clip 作者入口，SubMachine 打开子状态机；不能把全部目标解作 graph_locator。持久资源引用及缺失诊断留在源 owner |
| BlendSpace1D 参数 | `AnimationBlendSpace1DAsset.parameter` 对应 Scalar；samples为 position+Graph引用 | 参数 selector 与数值输入来自真实编译参数；采样面用同一 scalar 横轴；移动/增删/替换 sample 需新增本域文档 mutation及历史映射，不能调用模板 profile |
| BlendSpace2D 参数 | 一个 Vec2 参数，samples为 Vec2 position+Graph引用；编译要求至少3点、不重复/非有限/共线 | X/Y 输入编辑同一个 Vec2；轴名、范围、grid/snap 的 UE metadata 当前未在该模型确认，必须记录差异/由 owner 决定持久位置；不硬编码两套 Speed/Direction 参数。拖动预览与拖动采样点分清身份 |
| BlendSpace 样本跳转/权重 | 样本引用 Graph，Runtime 输出最多三组实际 graph/weight；外凸包按实际边界采样 | 双击先导航该真实 Graph，进入 Clip 的路径应明确展示；不能默认把 graph 字节当 clip。权重表/heatmap取同一求值 revision/参数；无合法求值显示诊断，不填固定百分比 |
| Transition 持续时间 | 源为 duration_seconds；当前 CreateTransition.duration_frames 固定除30 | Details以秒为权威；若展示帧，显式显示选定display FPS并由统一换算层转换。不得把任意Sequence FPS直接套入状态机，也不得换UI标签后仍用30计算 |
| Exit time / interruption | 源有可选exit_time及 None/CurrentToNext/NextToNext/Both；Runtime将exit归一值夹到[0,1]，source compiler未拒绝>1 | 可选归一值字段和枚举使用共享属性行；需本域 mutation/codec/host映射及回读；明确源>1与runtime夹取的取舍，未定时不得显示与执行不一致的值 |
| Transition 顺序/身份 | 源Vec顺序经编译保留，Runtime选第一个满足条件者；当前命令按from/to对找到一条；UE还有PriorityOrder与共享规则GUID | 两条同时可执行转换必须有可解释顺序；重排和多条同端点转换需要明确身份及事务。不能仅把UE PriorityOrder输入接到未存在字段，也不能按行号恢复已重排对象的焦点 |
| Condition | 源Vec支持比较或无值Triggered；当前set按parameter名覆盖；运行时额外表达式API支持All/Any/Not，但该源Vec路径只构成All | Trigger不显示比较值输入；比较操作使用实际类型；同参数上下界等多条件需要独立选择/删除身份和源命令。规则图中的Any/Not/自定义蓝图逻辑尚无本源持久合同，不画出可执行假图 |
| Layers | 已有状态机引用、weight、Override/Additive、mask_weights字段及runtime lowering | 复用公共列表/属性行；本域需新增对应事务/引用验证，角色域提供mask目标。现有保存路径应保留未编辑的既有层，不能重建只含当前可见字段的资产 |
| Key/curve 修改 | 源channel有时间/值、插值及tangent；当前命令只有add/remove key等，AddKey复制最后一个key值，不读取目标时间求值或World值 | 真正MoveKey/SetValue/SetTangents/SetInterpolation与录制/SetKey须定义采样来源、取消和inverse delta；不得把当前AddKey当“记录当前姿态/属性”。目前Curve foundation只提供投影视图 |

UE Transition 的更完整字段依据 `dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Public/AnimStateTransitionNode.h`：PriorityOrder、CrossfadeDuration（秒）、BlendMode/CustomBlendCurve/BlendProfile、自动规则时间/重入门槛/SyncGroup、开始/结束/中断事件、双向/禁用/仅活跃评估及共享规则身份。`AnimStateTransitionNode.cpp:875` 双击明确返回 BoundGraph。Zircon 当前简单条件/转换不等同于全部规则和blend逻辑；上表保留可支持字段，其余留为明确差异，不先建无效果属性控件。

## 共享组件的具体提案（未修改公共源）

| 编号/候选公共路径 | 所需输入和反馈 | 本域提供及责任边界 |
| --- | --- | --- |
| AN-P01 `ui/timeline`、`WorkbenchTimelineStrip`、`WorkbenchTransportControls` | 一份时间轴context：source/document revision、seconds权威时钟、display FPS、view/play range、track/key/section选择、真实playback反馈；step与start/end分开 | 本域提供实际文档投影与可逆命令；共享owner提供命中/输入/滚动/缩放与组件instance。已有ruler/snap/overlap/selection及TimelineModel.apply(delta)复用，不另造协议 |
| AN-P02 `ui/curve`、共享属性输入 | 实际curve/key/component identity、时间变换、值/切线/插值编辑和取消/完成反馈 | 使用CurveModel/CurveCanvasTransform及本域源通道；domain mutation持有数据，renderer不复制资产。修订后重新投影选择，Quaternion不当四条独立scalar旋转验收 |
| AN-P03 `ui/preview_scene`、`WorkbenchPreviewViewport`、原生GPU viewport承载 | secondary session、资源subject、未保存文档执行revision、真实时间/参数、求值诊断与rendered-image身份、focus/teardown；[CS-D02/D03细化](#an-p03角色共享消费者-cs-d02d03) | 动画域控制文档与播放，角色域提供mesh/skeleton，runtime拥有World/加载/求值；生产adapter与公共image承载由Parent分配owner，不能用Fake或静态Image替代 |
| AN-P04 `animation_graph/editor/src/plugin.rs`、公共palette/capability投影 | 逐节点可创建性、typed拒绝诊断、实际源schema与打开目标 | 本域明确当前Graph只可创建Output/Blend；Clip/Additive/Mask拒绝、blend_space_1d/2d无Graph schema。[两文件精确候选](#an-p04-精确-palette-候选-r01)交Parent/插件owner集成，未改公共源 |
| AN-P05 catalog/Penpot library/导出映射 | 保留现有component/node/control/slot/event/binding ID、shared-style version和实际依赖闭包 | Transport/BlendSpaceDetails/SampleWeights的直接定义已经存在；公共owner登记既有身份并分配组件instance，本域不重复注册或更换主题 |

事件映射至少记录四项：原始event ID、authored route、安装后的binding payload、实际handler。当前 `runtime/builtin/bindings/install.rs` 以 `WorkbenchExtension/<suffix>` 安装binding，Montage/Sequencer若干authored route与内建action文字不同。不能仅比字符串便断言事件丢失，也不能只保留字符串便声称操作已接通；后续产品回读/调用证据必须确认实际binding和真实文档命令。例：`WorkbenchExtension/MontageEditorAttackRow` 的authored route是 `.montage.attack`，内建payload action是 `.attack_row.select`，后者目前仍是模板交互。

## AN-P04 两文件修复r02

Parent原AN-P04授权的两文件候选现已落实到实际checkout：[plugin.rs](../../../zircon_plugins/animation_graph/editor/src/plugin.rs) SHA `fd826da9756d45329be6154fe4c10073f8364790a331ebc8ab3454e4fe0a16ee`；[tests/cases.rs](../../../zircon_plugins/animation_graph/editor/src/tests/cases.rs) SHA `5a6ec5a227c0f32b0104a2f435a606896b39967fbc1376a05b0c293dc8c7775f`。r02写前两目标与原r01 before SHA精确匹配、另30份依赖当时均匹配；当前两源保持，r03消费时依赖为29/30，本轮私有graph后继后为28/30，另catalog后继`9556e66d…`按[入口/依赖接续](#an-p04受管入口核对与依赖后继r03)单列，保留他域改动。原proposal/失败未改，不增加并行实现。[实际增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/animation-palette-apply-r02/actual.patch)与[源码/格式回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/animation-palette-apply-r02/applied-source.json)记录当轮两路径，r01“未应用”只描述原历史阶段。

Graph创建项现为Blend/Output；Clip仍由原capability拒绝，BlendSpace1D/2D是StateMachine内嵌state kind，不在Graph schema内。StateMachine保留state ID，名称明确Graph Reference State；transition/condition继续为原边及Details命令。已有Clip/Additive/Mask资产读取、schema/compiler和001/002诊断、capability表、原六transport events均保持。没有把移除不支持创建项当作BlendSpace/UE完整工具完成；production palette点击consumer、必填引用、真实命令/焦点/ESC/undo仍未接。

原错误BlendSpace登记测试已替换为`tests::graph_palette_matches_creatable_editor_capabilities`，双向比较注册palette与公开available graph-node能力，非空保护仍在。两源stdin rustfmt输出逐字节一致、git diff --check exit0仅静态，Rust执行0。所需最小recipe是canonical nested `zircon_plugins/Cargo.toml`的zircon_plugin_animation_graph_editor单包lib/defaults off/features=[]/Windows host/test profile/该唯一filter/jobs1/threads1/locked；Runtime native-host和Editor target-editor-host及支持库均是传递依赖，完整准入尚未建立。原提出的module-wrapper调用未执行，现已核对其缺nested manifest/test profile映射，client和runner有限策略也拒绝所需recipe，故当前不可执行；不以root-p/任意cwd或其它filter替代。待原owner正式交付receiver/source catalog/传递features与源资源准入后再精确声明执行，不复用两笔旧Editor identity。原capabilities/Triggered请求和r05 pending保持独立，产品passed/accepted=false。

## AN-P04 精确 palette 候选 r01

接续已有可达性结论，不重做盘点。本节保留r01时未应用候选；当前已按上述r02在两文件应用。原候选为 [animation-palette-r01.proposal.diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/animation-palette-r01.proposal.diff)，SHA `20e807622ad02cde62d1294bc45cdec68d44c3faedc4effb0d0d9e5716e98632`；只涉及 `zircon_plugins/animation_graph/editor/src/plugin.rs` 和原 `src/tests/cases.rs`。输入指纹、symbol锚点及未决项在同目录 `animation-palette-r01.proposal.json`。未修改公共descriptor、schema、capability、源资产或事件；这只是当前能力的创建菜单收敛，UE完整节点/状态类型目标仍未实现。

| 创建表面 | 候选差异 | 保留的实际语义与缺口 |
| --- | --- | --- |
| `animation_graph.palette.graph` | 删除当前不可创建的 `clip`，以及无Graph schema的 `blend_space_1d/2d`；保留原 `blend`、`output` ID及pins | Clip/Additive/Mask的已有资产仍由 `AnimationGraphNodeAsset` 和 `session/graph.rs::graph_node_label` 读取、投影；resolver原001/002诊断保留。没有禁用既有资产或借palette提升能力 |
| `animation_graph.palette.state_machine` | 保留 `state` ID，display name明确为 `Graph Reference State`；移除 `transition`、`condition` 伪节点描述 | 创建state仍须真实 `state_machine_locator/state_name/graph_locator`，仅GraphRef；transition是from/to边操作，condition是该边的Details mutation，原Create/RemoveTransition、SetTransitionCondition不删不改。其他state kind/sample作者命令仍缺 |
| 原错误登记回归 | 将 `blend_space_graph_nodes_are_registered` 替换为 `graph_palette_matches_creatable_editor_capabilities` | 双向比较palette IDs与公开 `animation_editor_capability_table()` 中available的 `animation.graph.node.*`，防止再宣传不可创建节点，也防止隐藏真实可创建节点；测试候选未编译执行 |

现有 `GraphNodeDescriptor` 只有ID/名称/category/pins，`GraphNodePaletteDescriptor` 的capability只限制整组；本候选不加第二套逐节点开关。所查 `zircon_editor`、动画插件、Runtime、RuntimeInterface和App引用仅覆盖登记/存储/materialization/测试，未找到把该palette点击绑定到真实创建命令的生产consumer；不能把登记修正说成菜单已接通。公共图表面owner需用现有typed event，提交前重读当前document/source revision、验证必填引用、同一次document mutation/history；焦点返回、ESC/cancel、重复Output与过期selection拒绝仍须实际host验证。Blend的descriptor固定a/b而源为inputs列表，端口与连线的完整对应继续交公共图表面合同，不在此候选擅改。

候选已用stdin格式化并通过 `git apply --check --whitespace=error-all`，两份目标源未写入；没有Jenkins提交或Rust结果。公共owner集成后，应以 `zircon_plugin_animation_graph_editor` 的lib、defaults off、features=[]、test profile，仅过滤 `tests::graph_palette_matches_creatable_editor_capabilities` 验证跨模块合同，并沿原登记测试确认menus/commands保留；`lib.rs` 以 `#[path="tests/cases.rs"] mod tests` 挂载，filter不能误加cases层级，零测试不是通过。实际编译前须重新声明精确范围。AN-P04、正式palette/UI链与UE覆盖仍open。

## 参数与真实权重接续 r05

资产默认值和预览参数各有权威owner。`mutation/graph.rs::set_parameter` 修改 `AnimationGraphParameterAsset.default_value`；现有host将 `SetGraphParameter` 经 `prepare_mutation → AnimationEditCommand → HistoryContextId::Document(document) → commit_after_apply/project` 提交。command的CAS swap在apply/revert都单调推进revision并重编译；focused动画document的undo/redo仍由原history路由处理。现有拖动/数值输入若要共用该作者命令，应在commit时提交一次；pointer move仅保留本次gesture候选，ESC/失焦/换源取消候选，不把每次move循环当原子batch，也不在stale revision下写旧inverse。该gesture接入尚未实施。

`AnimationManager::set_parameter` 则仅修改调用方持有的 `AnimationParameterMap`，过滤非有限值但不按编译参数类型校验、没有accepted/rejected回执；现有 `AnimationEditorSession` 只有Sequence瞬时状态。Graph presentation展示source defaults，StateMachine的parameter_items为空，都不是实际preview参数。可复用framework `AnimationCompiledStateMachine::parameters()` 的name/kind/default_value做typed投影，不能从字符串标签或缺省零值推断参数已安装。1D只接受Scalar，2D是单一Vec2；新Graph默认值literal `0`会推断Integer，`0.0`才是Scalar，预览adapter须按编译类型构造value，不复用字符串猜型。StateMachine未确认持久defaults字段时，不把预览参数写入不存在的字段。

最小真实接入仍由Parent生产secondary owner分配：同一preview session/subject/instance与clock，消费当前source/installed generation，应用经typed校验的Scalar或Vec2覆盖，随后在同一参数revision求值并回读actual参数。预览覆盖不dirty、不进document undo；显式“修改默认值”才走原source mutation。source edit/undo/redo、compile失败/last-good、换subject、关闭最后consumer时须退休旧请求；保留last-good须同时标明其source revision，不能把旧结果标成当前source。现有公开manager API不提供这些生命周期回执，不在本域另建World、时钟或直接翻preview capability。

权重出口已精确定位到插件 `CompiledStateMachineEvaluation::graph_samples()`；它来自当前active state的compiled采样，最多三个正权重 `(&AssetReference, Real)`，零权重被过滤。同步 `AnimationManager::evaluate_state_machine` 的 `AnimationStateMachineEvaluation` 只有parameters/state/graph/transition，不能提供BlendSpace权重；pure compiled `evaluate`也不等于真实instance的exit-time/transition进度、pose发布或GPU frame。生产管线已在 `evaluation/pipeline/state_machine_cache.rs` 保持instance参数投影与sampling hint，公开editor回读尚缺，Parent宜从同一求值结果投影，避免UI再算一套权重。

source与compiled sample都只有position+graph，没有stable sample ID。`BlendSpace1D::from_compiled` 内部排序保留的u32 sample ordinal仅对该compiled revision有效；公开graph_samples又丢失ordinal，同一Graph可被不同位置引用，不能靠Graph locator匹配逐样本高亮。允许在同一revision内展示带Graph引用的真实权重结果，不能伪造逐sample身份；新增持久sample ID及codec/save迁移、revision限定的compiled对应出口须另交owner决定，不能用位置、行号、time bits或临时UUID冒充已持久身份。正式sample mutation/history仍open。

AN-P05仅向原pipeline owner `01a0f04d-9d54-7fe2-9adf-120186804cfb` 交既有三项登记：`WorkbenchTransportControls` → `workbench/animation/transport_controls.zui`，`WorkbenchBlendSpaceDetails` → `workbench/animation/blend_space_details.zui`，`WorkbenchSampleWeights` → `workbench/composites/animation/sample_weights.zui`。本轮三项仍不在 `components/catalog.toml`，所有sourceAsset/control/binding/event/slot保留；未扩张为公共源、主题、handler或业务ZUI写权。原三个Penpot main与Widget WG-C01～08/Bool sampler候选继续复用，公共发布、实际参数权重/pose/frame、正式导出、真实EXE与逐图验收保持pending。

### AN-P05 三项既有 catalog 登记候选 r01

已形成单文件、未应用的[18行精确diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/animation-catalog-r01.proposal.diff)，SHA `3a269273bfa4fe04491ddd2a2f3f5dbba3304a7eaae72528f76ae604a8dec076`。目标仅为原[components/catalog.toml](../../../zircon_editor/assets/ui/editor/components/catalog.toml)末尾追加三个descriptor；当前实源仍68项，内存候选71项，原68项解析值和顺序不变。三个entry仅有component_id、document_id、同名binding_namespace和tier=composite，不新增props/slots、源文件、组件实现、事件或能力。

| 原component/source | 原root / control数 / 事件数 | 现有组合与登记边界 |
| --- | --- | --- |
| WorkbenchTransportControls：[transport_controls.zui](../../../zircon_editor/assets/ui/editor/workbench/animation/transport_controls.zui) | root / 8 / 6 Click | HorizontalGroup+原IconButton实例；原六ID/route不变。catalog namespace不替换WorkbenchExtension事件namespace；无Stop/Seek/context或clock能力 |
| WorkbenchBlendSpaceDetails：[blend_space_details.zui](../../../zircon_editor/assets/ui/editor/workbench/animation/blend_space_details.zui) | root / 20 / 4 Click+3 Change+3 Submit | 原资产/x-axis/interpolation三处PropertyEditorRow.value消费分别为Dropdown/Field/Dropdown，符合现有slot accepts；没有把内部value暴露为该复合组件公开slot/prop |
| WorkbenchSampleWeights：[sample_weights.zui](../../../zircon_editor/assets/ui/editor/workbench/composites/animation/sample_weights.zui) | sample_weights_root / 27 / 0 | 原PanelHeader.title消费Caption，符合现有slot accepts；Normalize仍不可交互，静态权重/default不变，不因登记就成为真实evaluator反馈 |

逐源SHA、55个control、16条event与既有`GAMEPLAY_ANIMATION_BINDINGS`的kind/action对应、4处内部slot及15份ZUI widget/style依赖见[机器合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/animation-catalog-r01.proposal.json)。这是ZUI源码依赖检查，不包括icons/fonts、Renderer/EXE或实际编译的完整资源闭包。初次生成因末尾额外空行被whitespace guard拒绝exit128，以及辅助模块路径误写均保留在initial_generation_failures；修正生成器后`git apply --check --whitespace=error-all`实际exit0，未放宽检查、未写公共catalog或源资产。

真实注册链是`builtin_component_descriptors`的include_str→原`parse_editor_component_catalog_manifest`，serde对props/slots缺省为空。Widget的`UiAssetPaletteCatalog::with_builtin_primitives`只接Primitive，三个Composite不会自行进入该原生palette；本候选不添加palette分支或替代Widget schema。原事件仍由`insert_workbench_extension_bindings`用WorkbenchExtension/<suffix>安装，catalog namespace不会重写绑定；跨域实例路由仍按已有显式context提案，不能拿登记替代实际backend ACK。

这三个catalog ID与原Timeline/Curve/Transport三Penpot main不是同一组：仅Transport对应现有component `982ef940-2239-4167-a24a-56c80ec35fbf` / main `44eb5ccd-8ac2-44f2-a4ad-d0d618daa096`；Details/Weights尚无正式分配的source main，不编造UUID、不复制现有三main。原pipeline的`prepareRealWorkbenchInput`/source-export validator当前仍固定LevelEditor owner，三条Editor metadata不修复generic export。Public owner需沿原源做canonical master/正式库身份、sourceHash/slot/override-reset、源导出和产品加载；当前没有进行这些操作。

候选只做TOML结构、精确source/slot/event/依赖和apply-check。集成后的最小已有Rust检查可用`zircon_editor` lib/defaults off/features=[]/test profile/jobs1/threads1，唯一filter `builtin_catalog_uses_the_typed_editor_metadata_path`，预期1项；覆盖真实Rust parser和打包文件存在，不能证明组件编译/交互。现有`builtin_component_asset_entries_resolve_to_declared_component_definitions`只筛选components/路径，**不覆盖这三份workbench/源**，本轮静态逐源核对不能冒称其Rust通过。本轮没有发新Jenkins请求或编译；原capabilities/trigger-condition身份和证据不作为本候选验收，后续实际执行须重新声明精确scope。

## Trigger 条件修复的具体回归要求

静态链路已定位到 `animation_document/mutation/state_machine.rs:99`。最小候选修复仅在本域mutation按operator创建value：Triggered应为None；比较operator需要合法、有限且匹配类型的Some值。公共compiler的 `ZR-ANIM-COMP-STATE-008` 和拒绝规则保留；不通过放宽编译器或 capability 标志掩盖问题。公共binding形状、插件palette或UI输入规则若需变化，先向Parent交接具体diff。

应通过实际binding/host/document链验证：从合法数值条件切为trigger、切回比较、重复相同trigger无额外历史、保存字节fresh解码仍为无值trigger、current编译与last-good revision一致、undo/redo恢复两种源及dirty/选择；另保留非法比较literal的拒绝/无部分修改。现有文档历史CAS和编译结果owner复用。

本轮已实施 [私有mutation修复](../../../zircon_editor/src/core/editing/animation_document/mutation/state_machine.rs)：Triggered直接生成None，不解析旧比较literal；比较继续严格解析合法Some，失败不写源。新增 [实际源码/编译/序列化/历史回归](../../../zircon_editor/src/core/editing/animation_document/tests/state_machine_conditions.rs)，由既有tests/cases.rs结构声明接入，覆盖空/旧trigger/残留literal、重复no-op、非有限/空比较拒绝、切回比较、binary fresh解码及CAS历史undo/redo后的current/last-good。`rustfmt --check`和七路径静态guard通过；托管请求 `ue-animation-20261024-trigger-condition-01` 经字段更正后，在提交前两次source capture间因其它域计划文档变化而退出1/pending，未启动Cargo。保留首个拒绝及同ID更正输入，不将写好测试算通过；真实UI binding/focus/dirty/选择尚未验收。

收到Parent接续后，原mutation哈希仍与其只读确认一致，本轮未重写生产修复。私有leaf现补为3条回归：新增旧非法Triggered+Some源进入真实CAS文档命令后仍报`STATE-008`并保留revision1的last-good；经实际mutation修复后revision3有效；undo回非法源时current失败但保留revision3 last-good，redo到revision5再次有效。每个状态均经文档bytes编码/新解码核对，并保留compiler拒绝；这是保存格式的序列化/重读回归，不是Editor磁盘保存、正常窗口或播放证据。新leaf的`rustfmt --check`已通过，Rust执行仍须原`trigger-condition-01`身份对账和受管过滤；不重投旧capability基线或拿其过滤验收修复。

## 选择、焦点与事务

| 操作 | 持久源/瞬时状态与撤销 | 完成时必须观察到的事实 |
| --- | --- | --- |
| Scrub / play / loop / speed / preview 参数 | 属于编辑会话/预览实例；与资产内容修改分开。按当前文档 duration/FPS 和播放范围更新同一时钟 | timeline、曲线 playhead、真实 pose/属性和播放控制一致；暂停后停止推进；切文档不串用旧 session；资源加载失败反馈实际诊断 |
| Key/curve 拖动或数值提交 | 使用同一文档命令，连续拖动明确一次提交/取消边界；撤销后重投影选择和曲线并使预览失效 | 撤销/重做恢复源值、dirty/revision、选取和预览；Escape 取消不产生半次保存；非法时间/值不写入源 |
| Montage section/segment/Slot/next | 持久资产事务；选择 section、preview jump 与修改 next 关系分别处理 | 修改能保存重开，下一段/循环实际影响播放；删除引用有确定诊断/修复；正反/loop/跳段的事件行为可验证 |
| BlendSpace 点拖动/增删/替换 | 点坐标与引用走文档事务；预览游标和输入参数不伪装成持久采样点 | 资产和 evaluator 重采样一致；权重/pose 随参数改变；无效/不兼容样本保持可解释拒绝；双击打开同一个真实 Clip |
| Graph 节点/连线/参数、state/transition/condition | 使用现有文档/历史 owner；编译失败保留合法 last-good，并明确当前草稿与执行 revision | 编译诊断能导航真实节点；双击规则/内嵌 BlendSpace 跳转保留上下文；删除状态后 entry/transition 不留悬空引用；undo 恢复源而非仅恢复画面 |
| 保存/关闭/切资产 | 复用实际 job ticket、文档 dirty 和 close 决策；关联 curve/Sections/Preview 的 owner 跟随文档生命周期 | 保存落到原资产，fresh 打开读取相同源；关闭失败保留可恢复 session；完成/取消弹层后返回发起控件或其有效祖先 |

键盘作用域跟随聚焦的文档和控件。文本框先处理输入、IME、剪贴板及编辑撤销，timeline/graph 不抢占文本。Sequencer 使用其实际 Space/K/箭头/逗号句号/方括号命令表；不能把同一全局键表硬套所有窗口。BlendSpace 按 `SAnimationBlendSpaceGridWidget.cpp:1435` 的实际 Ctrl 预览、Alt 高级 tooltip、PageUp/PageDown 导航和 Escape 清选；剪贴板/删除复用其选择与可执行条件。只读/无选择/加载失败的操作必须由真实 capability 决定可用性并提供诊断。

右键菜单由命中对象身份决定：空 lane、key、curve、section、segment、sample、graph node/edge 分别使用实际命令集合。打开时保存发起 control/object identity，关闭后恢复焦点；对象在期间已被删除时返回有效的文档编辑面。tab 拖动、停靠、窄屏侧栏收起和返回均使用公共 owner 的生命周期，不在本域另造 Dock 系统。

## Montage、Notify 与跳段的具体边界

以下UE路径相对 `dev/UnrealEngine/Engine/Source/`；引用树只读。所有新增持久字段/命令均是待实施合同，当前Zircon没有通过这些功能的运行证明。

| UE源码与行为 | 设计/命令分界 | Zircon缺口及实施约束 |
| --- | --- | --- |
| `Runtime/Engine/Classes/Animation/AnimMontage.h:37` 的 `FCompositeSection`：SectionName、链接时间、NextSectionName、metadata；`:83` 的SlotName+AnimTrack | 选section、从section预览、修改next关系是三个不同操作；next可None/下一段/自身或其它合法section，显示实际引用并可导航 | 不能用固定Intro/Combo行或跳到数字时间代替持久next graph。需要真实Montage资源/作者文档与稳定section/slot身份；时间/名称编辑不能丢引用 |
| `Editor/Persona/Private/SAnimMontageSectionsPanel.cpp:511/531/548/573`：按时间建立默认链、清链、设next、移除next，均有各自FScopedTransaction；预览全部/选中另走RestartPreview | 默认链/清链/改链是可撤销资产事务；PreviewAll/PreviewSection只改播放session | 不把按下预览写成资产历史；undo/redo后链图、Details、选中和真实播放读取同一revision；循环链需要有界调度/停止，不能无限执行 |
| `Editor/Persona/Private/SAnimMontagePanel.cpp:418/434`：AddNewSection事务；删除section先清所有指向该名字的NextSectionName，再EnsureStartingSection/刷新/重启预览 | 删除需一次原子修改section集合与入边，并有明确起始段/选择回退 | 当前资产/命令链缺失。undo应恢复段与原入边，不能只恢复一行UI；若保留不兼容UE的约束必须登记差异与设计决定 |
| `Runtime/Engine/Classes/Animation/AnimCompositeBase.h:67` 的FAnimSegment：资源引用、StartPos、AnimStartTime/EndTime、AnimPlayRate、LoopingCount；GetLength按trim/loop/有效rate计算 | lane显示Montage坐标，trim与源Clip局部时间分别可查；拖动段起点、修剪两端、改速率/循环均为域delta | 不把TimelineSection的range单独当完整segment模型；需要资源/skeleton兼容、有限时间/范围/循环合法性、实际sample映射与undo。跨域skeleton合法性向角色owner消费 |
| `Runtime/Engine/Public/Animation/AnimTypes.h:282-351` 的Notify：触发/结束偏移、权重阈值、Notify或NotifyStateClass、Duration/EndLink、MontageTickType、过滤/概率/服务器/同步跟随标志、editor Guid | 点notify与范围notify独立命中/编辑；开始/结束handle共享时间变换。queued/branching语义和filter不是图标属性 | 当前Clip点事件仅name/time/payload/target_id，不足以表示NotifyState begin/end/tick或branching point。逐项确认可支持字段及runtime生命周期；不伪造duration、概率、LOD或同步开关 |
| `Runtime/Engine/Private/Animation/AnimInstance.cpp:3127/3153/3179/3514`：JumpToSection、JumpToSectionEnd、SetNextSection、SetPosition分别存在 | 持久默认next与session临时next override分开；jump与seek不混成普通连续tick | 需要显式preview transport intent及后台确认。跳段是否发中间notify/结束活跃state、反向/loop边界由域实现并验证；不能直接跨时间差全量重放事件 |

现有Zircon点事件基础应复用而非删除重写：[sampling contract](../../../zircon_runtime/src/core/framework/animation/clip_event_sampling.rs) 已有固定from/to/looping范围、direction由区间确定、cursor只属于该固定范围、事件数/文本字节/跨度预算及延期；[实际采样器](../../../zircon_runtime/src/animation/clip_event.rs) 通过 `ProjectAnimationClipEventSampler` 对真实Clip资源求值。非loop范围夹到有效duration；同时间按event名称和track index稳定续读；loop接缝正向归end事件、反向归start事件。`from==to`没有区间事件。已有 [逆向/loop测试源码](../../../zircon_runtime/src/animation/tests/clip_event.rs) 证明这些边界有测试定义，本轮没有执行这些Rust测试，不能写通过。

采样游标不能跨跳段、换向或换asset继续使用。现有 [事件入队](../../../zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs) 已按replacement epoch拒绝旧结果、Deferred保留实体、Oversized给诊断，并用entity cursor轮转；新预览/通知链需要沿用有界与拒旧原则。当前没有把共享scrub或Montage跳转接入这些生产范围的证据；editor seek默认只预览，事件触发模式须是明确可见的域策略，不能因为共用playhead就产生游戏副作用。

最小真实验证应覆盖：真实两个section与一个有效clip segment的保存fresh重开；next链/自循环/清链和删除入边的一次undo；从选段预览与全段预览；trim/rate/loop映射；正向、逆向、重复loop接缝、同时间多个事件、预算延期、asset未就绪和旧epoch。NotifyState/SyncMarker/branching语义尚无实际模型时各自留open，不用点事件测试替代。

## 真实预览与失败边界

数据流沿既有职责推进：原始 Clip/Sequence/Graph/StateMachine → Editor 文档 revision 与会话时间/参数 → runtime-backed secondary session → 已加载资源和实际 evaluator → GPU image → Editor viewport/原生帧。`PreviewSubject` 中的 asset locator 只作为身份；解析资源、加载 mesh/skeleton、World 权限与求值留在 runtime-backed owner，不把 Editor 选择写入主 World 序列化。

当前接口尚未证明覆盖未保存草稿 revision、求值结果反馈和 rendered image 身份。生产 owner 接入前须明确这些已有合同的实际承载位置；不新增无消费者的 trait，也不由本域隐藏构造第二个 runtime。跨域必须共享 session 与 subject 生命周期，切换资产/关闭/取消加载时防止旧结果覆盖新文档。

空数据用真实空文档/空资源查询；无 skeleton、缺 Clip 引用、非法 graph、加载中/失败分别显示真实状态，不回退到静态 mannequin、固定权重或“Ready”。错误消息和中文/长文本走既有 i18n 合同。模型未就绪和资源未就绪都不能成为 passed。

### AN-P03：角色共享消费者 CS-D02/D03

2026-10-24接收Parent转交的 [角色域第10节](character-skeleton-design.md#10-给公共-owner-的最小合同清单) 与 [Parent统一边界](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)。角色owner为 `01a16d52-6b1e-7543-82f0-4f05ca92ad58`。本节接续AN-P03，不新增平行preview系统、trait或角色文档。本域继续牵头timeline/curve/transport，提供真实动画时间、参数及evaluator适配；角色提供mesh/skeleton/pose、Rig/Physics作者语义、资源与稳定选择键。Parent持有单一生产backend、Runtime/Renderer/GPU image及输入呈现集成，公共实现不由各域并发改写。

#### 现有接口能复用到哪里

| 现有源码 | 已核实的能力 | 当前不能证明的部分 |
| --- | --- | --- |
| [PreviewSceneBackend / PreviewScene](../../../zircon_editor/src/ui/preview_scene/preview_scene.rs) | `create_secondary_session/destroy_secondary_session/set_subject/set_playback/invalidate_views/focus_views`六项；`SharedPreviewScene`为`Rc<RefCell<...>>`，可在Editor内共享同一会话；setter仅在backend成功后记录本地值 | 所查Editor/Runtime/Host/App/动画插件仅找到测试Fake实现；返回成功和本地值相等均不证明资源、pose或GPU帧就绪。Rc不提供跨线程Runtime资源同步 |
| [PreviewSubject](../../../zircon_editor/src/ui/preview_scene/preview_subject.rs) | `primary_asset/additional_assets/animation_asset/parameter_overrides`；资产locator保持opaque | 不含document/source revision、subject generation、typed pose或资源ready反馈；同locator新revision可能与旧subject相等，不能靠`set_subject`的no-op检测重载草稿 |
| `PreviewPlayback` | `playing/looping/rate/time_seconds`有限值校验、backend接受后更新本地快照 | 没有真实clock/seek完成/pose生成/Stop恢复/帧关联；需读Runtime实际应用时间，不用本地模板时钟宣称播放 |
| `PreviewScene::close/Drop` | close幂等；销毁失败保留session句柄可重试，成功后清subject；Drop尝试close | Drop忽略销毁错误；没有GPU fence、异步加载/帧/拾取取消、最后消费者引用或资源释放回执，不能凭Drop推断无泄漏 |
| [WorkbenchPreviewViewport](../../../zircon_editor/assets/ui/editor/workbench/primitives/data/preview_viewport.zui)、[capabilities](../../../zircon_editor/src/ui/animation_editor/capabilities.rs) | 原始viewport仍是非交互Image，源为mannequin SVG；`animation.preview.runtime`与`animation.compiler.semantic`仍false | 不是生产GPU viewport或picking；本次不改资源、不翻flag，不借主窗口已能捕获就宣称preview接通 |

#### Editor 与 Runtime 的输入和确认

| 权威/边界 | 最小输入、反馈与行为要求 | 实际集成位置/责任 |
| --- | --- | --- |
| Editor文档/作者 | document ID、资源locator、实际read handle、source revision、编译可执行revision及诊断。未保存草稿须明确提交哪一版本；编译失败保留最后可执行版本时也必须标出版本差异，不能静默播放旧结果 | 动画复用 [animation document](../../../zircon_editor/src/core/editing/animation_document/document.rs) 的作者/编译边界；CS-D03的Skeleton/Socket/Pose/Profile实际文档owner、typed mutation、validator和持久化由角色/Parent确认。不得扩用Sequence enum冒充角色文档 |
| Runtime subject/session | 单一secondary session及其generation、subject身份和generation、mesh/skin/skeleton资源与revision、动画执行revision、参数与请求顺序；解析、资源加载、World权限与求值由Runtime拥有 | Parent沿现有preview_scene接生产adapter。Editor不持有Runtime World可变指针，不写主World/PIE；subject字符串不承载私造的pose JSON或revision编码 |
| 时间/pose | 同一clock的requested/applied time、play/loop/rate/seek结果与参数、求值诊断；真实pose及其发布generation/等效不可变身份、骨架/资源revision、必要的local/global变换含义 | 动画时间/参数与角色subject组合后交Runtime求值。复用 [AnimationPoseOutput](../../../zircon_runtime/src/core/framework/animation/pose_output.rs)/[AnimationPoseSnapshot](../../../zircon_runtime/src/core/framework/animation/pose_snapshot.rs) 提交版本；当前snapshot是Arc map，不凭空称其已有跨预览的整数generation字段 |
| ready/error | 区分请求已接收、资源解析/加载、类型与mesh/skin/skeleton兼容、pose提交、skin palette准备、GPU资源/渲染、Editor实际呈现。失败需含阶段、关联身份、诊断及可恢复策略 | [GPU skinning readiness](../../../zircon_runtime/src/core/framework/animation/gpu_skinning.rs) 只有准备摘要，不能等同GPU已调度或已present；Parent提供实际状态传递，角色提供资源兼容诊断 |
| GPU帧与呈现 | 可核对session/document/source与执行revision、subject/pose generation、实际采样时间、viewport与尺寸、render/publication身份、有效图像/纹理资源及其所有权/寿命。UI实际显示的帧与状态须对应 | 复用 [现有UI GPU image路径](../../../zircon_runtime/src/graphics/scene/renderer/ui/image.rs) 及既有media owner；具体外部图像承载/回执待Parent确认，不新建图片组件。资源ready、纹理解析成功、画面呈现是不同回执 |
| picking/focus | 拾取绑定当前已展示的viewport/frame/输入序号及source/subject/pose；回读再核对身份，角色将结果映射真实骨骼/目标；焦点在当前有效窗口/控件解析 | 复用 [Runtime viewport pick ABI](../../../zircon_runtime_interface/src/runtime_api/frame/viewport_pick.rs) 的票据与StaleFrame等结果，secondary session接线待核实；不得用别帧补命中。`focus_views`不替代Parent popup/drawer焦点返回 |

上述是现有边界的待接语义，不声称已有统一DTO或新增字段。Parent应先复用现有Runtime中性帧/资源/拾取合同，确定实际承载位置后再给具体diff。不能为填接口另造相同信息的私有GPU handle或第二份pose资产。

每次资源切换、草稿revision/骨架拓扑变化、seek重置、viewport退休或关闭都需要定义失效边界。旧加载、pose、图像和pick结果必须在写入/发布前核对当前session/source/subject及请求顺序，在UI消费时再次核对；generation已失效的图像不得留作“当前Ready”。若保留最后好帧，只能显式标记为过期/失败画面，不能把它关联到新subject或新pose。Runtime pose提交与GPU frame不可仅用墙钟时间近似配对。

最后一个消费者关闭须先停止接收新命令并退休generation，取消或排空加载/求值/帧与pick，撤销UI image绑定，按Renderer拥有的同步规则释放纹理/帧引用，再销毁secondary session并确认。实际顺序和异步完成由Parent/Runtime/Renderer owner确定；销毁失败要保留可重试的owner和诊断，不能在最后Rc销毁时吞错后宣称释放。关闭一个关联tab不能误销毁仍被其它消费者使用的session；会话共享范围/最后消费者计数目前没有生产实现回执。

实体属性`AnimationSequenceAsset`不能代表骨骼`AnimationClipAsset`或Montage。现有作者enum仅Sequence/Graph/StateMachine；clip加载/骨架通道求值与Montage的section/segment/Slot/notify语义分别按本域缺口处理。角色消费时间和真实pose，不把Skeleton/Rig/Physics作者操作挤进属性Sequence。时间轴view/selection及播放均是瞬时状态；角色gizmo/Details编辑由CS-D03实际文档产生一次typed事务/inverse，Escape取消、失败原子、undo/redo重采样当前时间、保存重开及重导入稳定键失效分别验证，不建第二个undo栈。

#### Physics preview 的会话和骨骼映射

已只读核实 [PhysicsPieMirror](../../../zircon_plugins/physics/editor/src/runtime_mirror.rs)：单个mirror持有一个`play_session_id`；`begin_session`清旧frame，`end_session`只结束匹配会话。`apply_frame`拒绝WrongSession、sequence不递增的Stale，以及低于当前`owner_generation`的StaleOwnerGeneration。[现有回归源码](../../../zircon_plugins/physics/editor/src/tests/overlay_provider_tests.rs)覆盖这些分支；本轮没有执行该测试，也没有证明它已消费secondary preview。

[PhysicsOverlayFrame](../../../zircon_plugins/physics/runtime/src/overlay_frame.rs)只有`owner_generation/colliders`，不是骨骼profile映射；[overlay](../../../zircon_plugins/physics/editor/src/overlay.rs)以collider entity生成形状/拾取对象，[provider](../../../zircon_plugins/physics/editor/src/viewport_overlay_provider.rs)从所持mirror取当前frame。不能把collider entity直接当骨骼index，也不能仅凭selection参数认定provider已经按preview session隔离。

具体待接合同是：Parent/Physics owner沿现有provider/mirror/event consumer接线确定secondary session的镜像归属或隔离实例，明确preview session到既有play_session_id的关联、sequence权威、owner_generation退休和最后消费者结束；不得对主PIE共享mirror调用begin_session来顶替preview，不挪主场景PIE帧，不重复注册overlay provider/Toolkit/creation ID。角色/Physics提供profile资源及revision、稳定body/constraint键、skeleton revision与稳定bone键到当前collider entity的双向映射，以及root/local/world变换规则。资源重导入、profile编辑或切session使映射失效；映射与pose/overlay/frame代际需可核对，过期pick不得选择新profile的骨骼。现有三项mirror检查须保留，不能只增frame sequence便绕开session或owner generation。

#### AN-P03 未决项与验证交接

| 未决项 | 当前缺口和最小交付 | owner边界 |
| --- | --- | --- |
| AN-P03-U01 | 单一生产backend与secondary session共享/最后消费者管理未取得实现或运行回执 | Parent统筹；动画/角色仅提供域输入，当前不修改公共Rust |
| AN-P03-U02 | source/执行revision、同locator新草稿、typed参数与subject/pose失效的实际承载未定 | 动画/角色给真实文档read handle，Parent/Runtime确定中性接口 |
| AN-P03-U03 | ready/error、pose→palette→GPU资源→实际frame/present关联及旧帧拒绝未接 | Parent/Runtime/Renderer负责生产路径；角色核对subject，动画核对应用时间/pose |
| AN-P03-U04 | secondary picking、selection映射、focus、退休与GPU引用释放没有真实验证 | Parent集成帧/输入/生命周期；角色提供稳定对象键，域不并改 |
| AN-P03-U05 | CS-D03角色文档/Socket/Pose/Profile事务持久化、重导入键失效未取得完整合同 | 角色/Parent确认真实作者owner；动画不借Sequence伪造支持 |
| AN-P03-U06 | Physics secondary mirror隔离、session/sequence/ownergeneration关联及entity↔profilebone映射未接 | Parent协调Physics/角色，保留既有provider/mirror与主PIE隔离 |

候选源码以上述已存在的preview_scene、animation document/Runtime pose、UI image、Runtime pick ABI及Physics mirror/provider为准，均只是后续交接位置，不代表本轮修改或接口已批准。最小联合验证为：真实mesh/skeleton与合法Clip加载→一次资源切换→一次实际pose改变/seek→核对资源/pose/GPU帧和正常窗口图像变化→拒绝旧subject/pose/frame/pick→关闭一个消费者保留其它消费者→关闭最后消费者无session/资源泄漏；Physics另证同场运行的主PIE与preview不串帧、映射选中正确且旧generation拒绝。作者操作再验证一段drag一次历史、失败/Escape及保存重开。未接通前六项全部open；正式画面仍复用LayoutReviewCase/原生协议、三尺寸与实际DPI144、双端逐图对照，不用Fake/静态SVG或主PIE帧验收。

## Penpot、导出与逐图验收

共享空态于revn8保存并fresh重开：302对象/3个native components/9linked instances/66flow容器，三尺寸各有字体就绪并逐图复核的工作区PNG。真实UUID、各图结果和源边界见 [共享控件逐图记录](animation-shared-controls.md#真实-penpot-组件与逐图记录)。后续蒙太奇作者区已接续到revn14，见下节。这些是本域空态设计进度；完整业务窗口、公共样式和基本组件实例复用、官方导出、引擎接入及双端验收仍未完成。

在已分配文件中，每个真实 board 记录 file/page/board/component/instance/shared-style/version，以及源 asset/control/node/slot/event。普通层为 flow，数据面是可解释例外。先保存并收到服务端 ACK，关闭旧 context，再 fresh 认证重开同一文件核对；随后官方导出回原始 `.zui v2`。导出前后用实际身份索引核对 import/component 引用和所有绑定/事件，不以物理路径别名替换稳定逻辑 ID。当前Shared rev1已发布四个基础main，本域rev31四个linked实例及保存fresh重开见[公共消费r03](animation-shared-controls.md#公共库四主件消费-r03)；早期空库仅为历史。动画三专责main仍在本域isShared=false文件，交Parent审核实际UUID/版本/slot/event并集成发布，与四基础main及上述三catalog候选不混同，不伪造跨文件消费。

原有事件包含固定示例对象名。进入真实数据时须逐项记录保留的语义、对象参数和命令映射；不能把既有 `section.intro` 事件悄悄改作任意 section，或删除原 ID 后声称全部保留。涉及公共事件/注册变化先向 owner 交接具体迁移。

验收复用 LayoutReviewCase 和正常 EXE evidence，尺寸为 1280×800、900×620、640×520 的100%，另加1280×800实际150%/DPI144。每组覆盖默认/真实空数据/选中/焦点/菜单与右键/弹层返回/滚动首尾/中文与长文本及上述特有交互。每图记录双端同 case、官方 PNG、正常窗口原图、资源 closure/EXE fingerprint、原生捕获入口及有效 nonce/publication、并排/difference、≤1 DIP 的关键边界/文本行框测量和人工逐图结论。尚无资源、PNG、原生 EXE 或双端比较的 case 保持 pending。

样式传播证明使用真实 Penpot 中受导出器支持的一个局部或共享属性；先记录原值/共享 owner，再完成保存关闭重开→导出→原资产部署→产品加载→原生截图变化。不能只改 `.zui`、网页 CSS 或捕获脚本制造变化。Jenkins 恢复后按实际受影响插件/Editor 模块执行聚焦检查；原 pending 基线身份保留，未执行不能写通过。

## 蒙太奇作者区：rev14设计与复核

真实file/page保持本文件首段身份。revn14/vern0读回533对象、3个原native components、15个组件copies、150个flow容器；未新增或复制main。本次在专属页面增加三个作者区空态board，各复用Timeline Surface与Transport实例；Curve main及原九个共享控件copies保留。本次无公共文件、tokens、Starship、共享事件/host或ZUI修改。

| 作者区逻辑尺寸 | board UUID | Timeline instance | Transport instance |
| --- | --- | --- | --- |
| 1280×800 | `c0af7cda-882d-59b1-8d0e-df0bca32c639` | `2cee2534-e38b-52aa-a4a1-faeef841cc23` | `eb1040be-5f82-58a6-9ac7-9152fc4e490f` |
| 900×620 | `83826cfb-3a1d-5056-ace5-9497f37d4eb4` | `9c128f5a-99ca-5f44-8f80-296f12dcf071` | `f40dd0f9-4936-5dc5-ad36-d099b88df825` |
| 640×520 | `f49cf095-55f3-53d1-8e42-310c80075626` | `0b8904d0-4ec9-5ad6-831d-33f21484897d` | `4fd6bff0-fd4c-58a4-8f55-144985f344f1` |

依据`AnimationEditorMode.cpp`的0.2/0.6/0.2三列及中/右列0.6/0.4分割，组合资产详情、预览、Timeline/Transport、选择详情和Montage Sections。Sections/Asset Details为本设计选用的前景页签；不宣称复刻UE默认Asset Browser/Skeleton Tree前景状态。`AnimModel_AnimMontage.cpp`与`SAnimMontageSectionsPanel.cpp`提供分段/排序/跳转业务依据；设计为空资产/空subject，没有虚构分段、Slot、通知、pose或资产时长。尺上的0–1s仅是视图范围。外框菜单、完整页签、Dock及其输入仍待Parent公共组件，board尺寸是作者区尺寸，不是正常EXE客户区验证。

证据位于`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-animation-domain-02`：`montage-empty.summary.json`保存UE/主题/原资产SHA及新增身份；`montage-empty-reopen.json`记录旧context关闭、fresh认证重开、revn14保持、字体就绪与browser35568实际退出；`visual-review-r14.json`逐图记录实际打开的三张完整工作区PNG。当前实测renderer为SVG，Noto Sans SC400及Roboto400已加载；不拿未启用的Wasm通道作为当前字体判据。71%/98%/134%是Penpot工作区缩放，不是系统DPI。

本次修复仅限这些实例：18个原canonical path按实例逻辑位移平移且保留glyph ID/shapeRef；窄lane159.2显示0/0.5/1三档，较宽lane315.2/543.2保留五档；640的时间字段64与状态字段103.2能容纳本空态文字。保留400/500请求、浏览器启动失败、错误Wasm门、r10重叠与缺图标、r12清空SVG文字缓存、r13继承过时master行框坐标等失败证据；r14借用实际SVG读回的行度量并按行框/对齐规则定位17项，逐图已见刻度、六个播放图标和完整状态。17项行宽算术检查不等同双端1DIP像素测量。

| 未完成项/提案 | 具体依赖与状态 |
| --- | --- |
| 原资产映射与官方导出 | 候选源`res://ui/editor/animation/extension_montage_editor_workspace.zui#WorkbenchExtensionMontageEditorWorkspace`，原control root同名；现有Overlay/示例分段不是本设计已接入。29个control IDs/15个事件保留，正式源owner导出尚未运行；Timeline仍只有内Canvas映射。 |
| 窄窗刻度/时间文本度量 | 现有`ui/retained/ui/data/projection/timeline_strip.rs`接受duration/current_time/tick_interval；共享Canvas generation及Transport需统一按实际行宽决定标尺密度/时间字段，不能把本空态64宽套给长时间。先向Parent交接设计差异，不改公共源或强迫消费者FPS。 |
| 文本/向量导出检查 | 本手写API设计遇到绝对path与文字缓存坐标不一致；需要正式pipeline核对实例变换、字体tuple及SVG/Wasm两条呈现的行框。此现象不证明正式导出器已有相同缺陷；本域不替换其codec。 |
| 公共样式及基本组件实例 | 当前仍是已有主题的本文件投影；通用Dock/Tab/属性行库和共享style链接等待Parent。独立关掉重开不能证明公共发布/跨文件传播。 |
| 真动画、所有交互及验收 | Montage作者模型/事务/实际播放未取得；生产GPU backend未交付。插件聚焦检查、受管Rust执行、EXE资源闭包/指纹、原生入口、实际150%DPI、非空/选中/焦点/弹层/滚动/长文本、Penpot↔EXE并排/差异及样式往返全部pending；本三board均`passed=false/accepted=false`。 |

只有资源和原生case就绪后才向Parent请求桌面时段；本次没有桌面输入、DPI修改、原生捕获或服务启动。

## 蒙太奇源与事务边界r01

按Parent本轮Montage接续范围只读核对了47份实际文件；[源指纹与15事件的安装结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/montage-source-transactions-r01/montage-source-r01.json)及[14项操作矩阵](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/montage-source-transactions-r01/montage-boundary-matrix.json)逐项包含路径、SHA、symbol/行号、当前支持、owner和未决合同。未增加Montage DTO、resource kind、player、clock、World、UI v2字段或公共handler；这不是已交付的资产/运行时接口。

原ZUI仍为WorkbenchExtensionMontageEditorWorkspace/root montage_editor_workspace，31节点/29 control IDs/15 events，SHA 422445449969ae037746028e3a61d75825b366a2f856889f95831a31ecc35977。七个import仍引用既有基础组件，没有Timeline/Transport import。事件按WorkbenchExtension/<suffix>安装：全部15个ID及event kind匹配；九条authored route和canonical payload action不同，由同一binding ID关联，不能仅据字符串差异判定断线。menu.item.ability.montage_editor已映射原workspace URI；静态存在不等于本轮实际点击过菜单。

实际模板字段处理只改Surface的value/value_text；Preview/Apply最终落到feedback.rs的“Not implemented (preview)”状态/输出。AM_DashAttack、Intro/Combo、HitPause、UpperBody、Blend、2.8m均为示例字串，没有文档身份或runtime回执。现有AnimationAuthoringDocumentKind、asset和host document_kind_for_route只有Sequence/Graph/StateMachine；限定animation数据/runtime/文档/host路径的两次rg查询也未找到Montage资产或播放器。此结论限定这些已检查路径，不推断仓库所有动态插件都绝无实现。

| 操作 | UE实源依据及必须保持的行为 | Zircon适配边界与当前缺口 |
| --- | --- | --- |
| AN-M01 创建/加载/保存 | AnimMontageFactory::FactoryCreateNew检查骨骼/源动画；EnsureStartingSection保证Default或首段在0秒 | 先由Parent确认规范资产kind、resource/codec/version和稳定元素ID，再接原作者store；当前三板是未打开资产，不是合法新Montage的空段落状态。 |
| AN-M02 Slot | SAnimMontagePanel::RenameSlotNode/AddNewMontageSlot同时登记Skeleton slot；CanRemoveMontageSlot要求多于一个slot，移除后ConditionalRelink | 骨骼Slot注册属角色域，Montage轨属本域；跨资产history/撤销和引用解析需Parent统筹。现Slot输入框只有字符串，无真实事务。 |
| AN-M03 Segment | FAnimSegment区分StartPos和源AnimStartTime/AnimEndTime；GetLength包含LoopingCount、源rate scale与有效rate绝对值，近零rate有既有fallback | 时间轴横轴为秒；源区间、放置、有效rate/方向及重复次数分开。TimelineSection矩形不等于segment资产，不复制采样器或新建clock。 |
| AN-M04 Section身份 | FCompositeSection存链接起点、SectionName、NextSectionName；时间排序和执行顺序分离 | domain提供不随重命名/排序/拖动变化的持久ID；可视range由边界/总长投影，不能当源start/end双字段或由row/time生成身份。 |
| AN-M05 拖动/数字 | AnimModel_AnimMontage::OnSetEditableTime在!bIsDragging时一次scoped transaction，Link/排序/刷新notify offset并dirty | 同一次revision受检domain mutation、inverse和history；拖动临时态与numeric最终提交共用命令，一手势一次undo。当前无Montage gesture/命中/事务实现。 |
| AN-M06 Next/循环 | SetNextSectionIndex允许所有有效目标含自身；IsLoop仅在路径回到起始段时为真 | 允许自环/合法循环；A→B→C→B时A不属于该循环，B/C属于。不得套用Graph无环限制；target必须属于当前文档/版本。 |
| AN-M07 Default/Clear/Remove Link | Default按时间重排后顺连，末段None；Clear清所有next；Remove只清一个，各有事务 | 每项一次可逆domain mutation；undo还原原ID/链接/顺序，不能重新计算default冒充逆操作。相同起点的tie策略尚未确定。 |
| AN-M08 删除及引用修复 | RemoveSection清所有incoming next并补起始段；OnAnimSegmentRemoved含Notify State EndLink，清删除目标并修后续索引 | 一次事务处理selection、section、notify起止关联及inverse；不能只删矩形。稳定link映射由规范资产提供，当前尚缺。 |
| AN-M09 Normal/Selected/All预览 | SectionsPanel转发原IsPlaying；AnimPreviewInstance真实重建实例/跳转并保留play/pause；All改实例链接，ResetSectionsOrder恢复资产next | 单一Parent secondary session/clock和角色subject/pose；预览覆盖不dirty资产。UE预览强制weight1/移除blend-out是编辑器策略，不能据此称Runtime blend已完成。 |
| AN-M10 Seek/Stop/Step | AnimPreviewInstance把StepForward/Backward、JumpStart/End及Normal/All分别实现；runtime另有事件/branching point处理 | 沿原显式context迁移提案；requested/actual/status/source/session/frame分开，seek事件/方向/loop端点及Stop恢复由真实owner决定。原六events不改，Previous/Next不偷换step。 |
| AN-M11 Notify/State/RootMotion | Notify State有EndLink和active branching points；section移动刷新trigger offsets | Clip点event_tracks不等于Notify State生命周期，原2.8m字串不等于root-motion delta。缺typed对象、真实回调和pose/frame关联。 |
| AN-M12 源事件/菜单/字段 | UE Toolkit实际构建Montage model和preview；Zircon现有模板链与其不等价 | 原15events及29 controls保留；新domain参数/host路由先给具体diff，不覆盖公共动画handler，不按焦点猜域。 |
| AN-M13 History/Save/Close | UE资产事务与临时预览状态分离 | 规范Montage资产批准后复用原document CAS、HistoryContextId::Document及save/last-good链；当前没有Montage variant。旧revision/session/frame须在写前与发布时拒绝；dismiss/tab close/document close分开。 |
| AN-M14 共享表面/验收 | UE源码只作为依据，不提供Zircon后端 | Timeline仅内Canvas是当前源，外wrapper不算已导出；Transport draft2为六按钮+Fill，clock/status属consumer wrapper；Curve正式ZUI表面仍缺。资源/EXE有效才passed，双端逐图通过才accepted。 |

共享TimelineRange::new会把非有限值归零并排序反向区间，contains含两端；这些是投影辅助行为，不是Montage资产的校验或事件边界政策。domain须先验证源值并决定segment/通知区间包含关系，再投影到共享类型。公共Bool/Step sampler的既有精确key边界也不能外推为Montage播放边界。正式DTO/稳定ID、相同时间排序、scrub触发通知及Stop/preanimated恢复、跨Skeleton历史等仍是具体待Parent/域owner决定项，不用示例数据填补。

17:37:34Z只读API再次取得原file/page rev105/1358对象/原3main/isShared=false；三个Montage子树各76对象，与实际r13重开PNG对应的源对象逐项相同。[复用证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/montage-source-transactions-r01/penpot-existing-evidence.json)保留原board/尺寸/PNG SHA，没有新设计写入或截图，不把rev14/80图重命名为新图。上节rev14阶段失败及复核原样保留；当前仍无打开的Montage资产/subject，0–1秒只是视图范围，未覆盖非空/选中/焦点/弹层/滚动及真实150%DPI。

[插件readiness](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/montage-source-transactions-r01/plugin-readiness.json)于17:37:36Z对4213/manifest.json实际拒绝连接；未启动/恢复服务或执行功能导出。原两Jenkins请求及pending不变，本轮纯文档/源核查无编译或新请求；AN-P04两源码、AN-G13三源码/compiler及Parent四Runtime路径只读保护。EXE、GPU、原生捕获、尺寸/DPI、同状态并排/diff/逐图1DIP及样式到产品闭环仍open，passed/accepted=false。

## BlendSpace 1D作者区接续rev483

2026-10-25在原动画file `b4a1e3ce-9e6a-81fe-8008-d69d60a13f99` / page `b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a`，以原rev17三张2D板为原生复制来源，形成三张可编辑1D作者区。最终保存、关闭、fresh重开到rev483/3729对象，data hash `0b6a3f02d83776f270c137765f3808a520e081bda08b2c3efa3fb83959b48cf4`。本阶段开始rev405的3350个非Root对象、原3main及定义逐项保持；Sections53对象、原2D、Montage和Sequence板未回写。新增378对象、24native copies（21 Shared PropertyEditorRow、3 owner Transport），147处shapeRef均解析到各自当前设计main，未新建或发布main。

| 逻辑尺寸 | board UUID | Grid UUID | Transport copy UUID |
| --- | --- | --- | --- |
| 1280×800 | `1ad455f7-9b66-8058-8008-d7b160267045` | `1ad455f7-9b66-8058-8008-d7b160267075` | `1ad455f7-9b66-8058-8008-d7b16026705e` |
| 900×620 | `9a516842-ccaa-808d-8008-d7b2f1d3c95d` | `9a516842-ccaa-808d-8008-d7b2f1d40aa3` | `9a516842-ccaa-808d-8008-d7b2f1d40a8c` |
| 640×520 | `307b32f7-8ceb-80ee-8008-d7b3b7b2703b` | `307b32f7-8ceb-80ee-8008-d7b3b7b28f3e` | `307b32f7-8ceb-80ee-8008-d7b3b7b28f27` |

布局继续使用flow三列与预览/文档分区；属性行在tabs后流式排列。数据canvas内以两个11×11菱形显示样本，移除1D不适用的Y轴标签与第三样本显示；未删除复制树中保留的隐藏旧字段。UE依据为只读 `SAnimationBlendSpaceGridWidget.cpp` 的SingleAxis、仅X subdivisions及 `SampleValueToNormalizedPosition` 的归一Y=0.5。矩形grid外框保留；左列仍是Asset Details候选，不等于UE默认Skeleton Tree前景或完整Toolkit/chrome复刻。

源fixture为 `zircon_plugins/animation/runtime/tests/blend_space_state.rs::blend_space_1d_state_blends_production_graph_poses` 的Idle状态、`speed: Scalar`、position 0/1与请求输入0.25；真实helper `state_machine_interruption.rs::uri` 生成 `res://animation/state-blend-space-idle.graph.zranim` / `res://animation/state-blend-space-run.graph.zranim`。板上 `idle.graph` / `run.graph` 仅短显示名。未运行该测试、加载项目资产或获得实际权重/pose/GPU帧，不能将0.25当真实clock或将预期0.75/0.25当运行回执；未画绿色实际预览标记。

本次横向range `[0,1]` 仅从这两处源position导出，是view范围，不是新增持久轴字段、单位、FPS或资产clamp。Grid左右inset均24，顶部16、底部38；令 `u=(position-min)/(max-min)`，中心X为 `grid.x+24+u*(grid.width-48)`，中心Y为 `grid.y+16+(grid.height-54)*0.5`。三板样本实际selrect中心与此投影差0；path原始x/y保留复制前坐标，因此校验读取呈现selrect。未生成Hermite线、输入命中或权重插值器；当前 `ui/sample_grid/generation.rs` 仍是通用2D点/刻度模型。

每板7个PropertyEditorRow引用Shared component `6ddc248b-bd35-80c5-8008-d6c8cab1e7d0` / main `6ddc248b-bd35-80c5-8008-d6c8c84d848f`，每实例4个已发布设计源parts全部解析；Transport引用原component `982ef940-2239-4167-a24a-56c80ec35fbf` / main `44eb5ccd-8ac2-44f2-a4ad-d0d618daa096`，每copy覆盖当前21设计parts，仍不是正式canonical owner/事件codec发布。原6个control/event/Click/route逐字保留；无subject时按钮仅设计opacity0.45，真实eligibility/ACK尚未接通。

Shared仍为rev5/4main/34色3typo，前后data相同，无sync/Update/reset/detach。42处本地属性标签/值显式引用 `editor.text.primary` / `e022fcac-39df-51eb-8515-4723f9932060` / `#c0c0c0` / opacity1，`sourceMasterInherited=false`；这不补原Sections9root无fill引用、字体全局A/B或继承/reset验收。84处新板Noto文字框沿已知本地lineHeight1.5修正，42个属性标签/值框高24，640值槽可用宽62.4，`run.graph`行宽61.265625，保留60名称列minimum和原字号；检查包含保留的隐藏字段，不能作为84项实际交互已通过。

| 延续原BlendSpace差异 | 本次实际支持与source evidence | 具体依赖及仍未完成 |
| --- | --- | --- |
| 1D/2D类型和范围 | 真实StateKind的1D为Scalar参数与position/graph，画面只有单轴、两源样本；原2D板保持 | 源无持久axis range/grid/interpolation字段，不使用模板Speed/0–620或NumberField默认42/0–100；正式view策略由域/Parent接既有文档。 |
| 稳定选择、拖动与undo | 选中run只表示本fixture当前revision的序号1 | `samples: Vec`无稳定sample ID；不得以index/position/time bits/显示名/Graph引用替代持久身份。接原history/CAS及gesture/cancel之前不启用样本修改。 |
| 当前published main与typed value槽 | Shared设计main4parts解析成功；当前 `property_editor_row.zui` 已7nodes，额外value_column/description/source/reset不在这份4part设计闭包 | readonly普通文字是设计注释，**不符合正式value槽accepts**。原public owner交当前WorkbenchField/NumberField的完整source、typed props及codec；不新造master、改catalog或给普通Label补签。 |
| 参数求值与真实preview | 原Runtime存在1D邻接线性blend与端点clamp；本轮只读15源pin匹配 | 当前compiled输出Graph/weight不能证明到稳定sample的UI回执；动画适配接真实参数/事务，角色交subject/mesh/pose，Parent交单一secondary backend/clock/frame/ready/input生命周期。actual weights/pose仍null。 |
| UE布局/交互及产品闭环 | 三尺寸可编辑body与短Graph文字可见，非完整Toolkit或输入合同通过 | Ctrl预览/PageUpDown/Escape、命中/拖动/捕获/焦点/弹层/滚动/空数据/完整长引用、正式ZUI/export/normal BlendSpace producer/EXE/实际150%DPI/双端逐图≤1DIP及样式到产品闭环全部open。 |

[源身份与边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/source-mapping.json)、[最终应用](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/board-application-r05.json)、[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/image-review.json)、[最终检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/validation.json)保留全部实际证据。r03/r04/r05共18张原PNG已逐张打开，九对保存/重开画板内RGB差0，只证明各自Penpot保存一致；工作区71%/98%/134%与DPR1不是原生DPI。最终原图：[1280](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/blend-space-1d-1280-reopened-r05.png)、[900](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/blend-space-1d-900-reopened-r05.png)、[640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/blend-space-1d-640-reopened-r05.png)。

r01原生复制符号失败、r02部分保存后API500、r03布局未重排、r04行框和640文字clip均保留；r03沿原部分board续作，无重复1280板；r04触发真实layout/update，r05只修新板文字。初次检查误用path原始x/y和包含caption溢出的SVG subtree bbox，错误报告原样保留，后续仅校正校验投影/ROI，未新增浏览器或Penpot写入。五个自有actor均退出；PID39348已按创建时间确认为外部postgres复用，未操作。无Rust/ZUI/能力表/公共handler/6events/服务/桌面/Jenkins身份变化；Rust和插件功能执行0，captureReady=false、passed/accepted=false，释放0B，必要失败/原PNG/过程证据保留。

## BlendSpace作者区rev17

在原分配file/page保存三个BlendSpace 2D作者区设计。rev17/vern0有809对象、3个原native components、18copies及255个flow容器；未增加main。三个新Transport实例仍引用原component `982ef940-2239-4167-a24a-56c80ec35fbf`/main `44eb5ccd-8ac2-44f2-a4ad-d0d618daa096`。完整main子树与rev14读回相等；公共文件、主题、原ZUI、共享事件和host未修改。

| 作者区逻辑尺寸 | board UUID | SampleGrid设计节点 | Transport instance |
| --- | --- | --- | --- |
| 1280×800 | `d727a5e1-2ffa-5283-adf2-478d394e887c` | `9826a815-3f64-5d8c-86a3-d9c6f9ee7481` | `0c57f2b9-2071-5a0e-965f-3663eeea23d6` |
| 900×620 | `167949d2-6b88-5729-8113-e8d2ac75f97a` | `6d400155-39a1-5a55-a84d-67afba5f3932` | `ae5e2803-56a2-50ee-ae6c-ae6b96f7b67f` |
| 640×520 | `7da83ecb-a904-5fc1-bd45-b092ca4975ac` | `69e98a98-27a3-5cc3-bed7-733117288504` | `106cd476-8ac7-55e3-a1be-29232af0b368` |

布局依据仍为`AnimationEditorMode.cpp`的20/60/20三列及60/40分割，左列选择Asset Details而非UE默认Skeleton Tree前景；中列预览与BlendSpace文档/transport，右列Details与Asset Browser。`SAnimationBlendSpace.cpp:55-105`提供独立Grid文档和scrub bar关系；`SAnimationBlendSpaceGridWidget.cpp:272-289,655,750-835`与`StarshipStyle.cpp:898-900`提供11×11 key、字体、底部X端点值与左侧Y值依据。已目视只读`Content/Editor/Slate/Common/Key.png`，修正首版圆点为九个11×11可编辑菱形path，保留marker ID、中心和数据。UE栅格alpha/色调、axis title/arrows及精确行框仍未对齐；当前painter的X刻度位于上方，本设计位于下方，差异保持open。普通区域使用flow，仅采样坐标与轴刻度使用数据canvas；通用Dock/Tab/属性行库仍依赖Parent。功能结构继续映射已有App Shell/Forms参考，不引入新主题。

画面数据明确标作设计fixture：来自`zircon_runtime/src/core/animation/compiler/tests/state_machine_cases.rs:235`声明的三点`Blend`状态，参数`direction: Vec2`，坐标`[0,0]`、`[1e-20,0]`、`[0,1e-20]`，Graph引用`res://animation/sample-{0,1,2}.graph`。只读提取源值，不声称本轮运行了该测试或加载了这些资源。选中index1；0/5e−21/1e−20是从fixture导出的编辑视图范围和刻度，不是持久轴设置。权重显示未求值、预览无subject、时间显示破折号且无播放会话；未虚构duration、clip、pose、权重或GPU帧。

证据仍在`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-animation-domain-02`。首写rev15后浏览器原生保存102个新文字节点的`positionData`而成为rev16，首轮捕获因此明确失败并保留；`blendspace-fixture-font-cache-r15-r16.json`证明仅新文字缓存变化、组件定义及main子树未变。Python与浏览器JSON对1.0/1的序列化差异使用解码值比较，未把JSON字节差异误报为组件修改。rev16重开稳定后修正九个标记为rev17；`blendspace-fixture-r17-reopen.json`证明关闭旧context、fresh认证重开保持17、SVG实际Noto Sans SC400/Roboto400就绪、无page errors、browser22892实际退出。三张`blendspace-fixture-{1280x800,900x620,640x520}-r17.png`已逐张实际打开，科学计数、三点名称、蓝色选中菱形、六个transport图标及完整会话状态可见；102个可见新文本行的缓存横向边界没有超出文本框1逻辑像素，属于算术检查。71%/98%/134%是工作区缩放，未证明实际DPI或双端1DIP；逐图意见见`blendspace-fixture-visual-review.json`，全部`passed=false/accepted=false`。

| 参考/实际差异 | 已核实原因与候选路径 | 依赖与未决操作 |
| --- | --- | --- |
| 稳定采样选择、拖动与撤销缺失 | `core/framework/animation/asset/state_kind.rs`的samples为Vec(position,graph)，未含持久sample ID；`ui/sample_grid/generation.rs::SampleGridPoint`只有x/y/label/selected，原Canvas输入flags全false | 复用StateMachine原文档/history；Parent协调共享命中/输入投影。fixture index不能当正式稳定身份；确定文档/state/revision与sample选择映射，禁止label充当key。 |
| 模板轴/插值与真实源字段不同 | 原`blend_space_details.zui`含Speed/Direction/0–620、Triangulated/Grid/Weighted；真实2D资产只含Vec2参数名、位置及Graph引用 | 先绑定真实参数类型和样本；轴范围作为编辑视图还是新增资产字段必须由Parent跨域合同明确，不默认批准独立BlendSpace格式。 |
| 小数刻度显示0，无法区分有效坐标 | `ui/sample_grid/generation.rs:206-212`的EPSILON/.0/.1格式会将5e-21与1e-20标成0；Canvas projection允许有限有效范围，geometry按范围正常归一 | 未应用提案`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/blendspace-sample-grid-formatting.proposal.diff`及同名json交Parent：仅有限非零绝对值<0.1使用科学计数，附既有测试模块中的候选回归；保留现有常用标签。提案未编译/提交，不解决所有相邻刻度、密度、非有限值或本地化策略。 |
| 样本编辑/求值/预览仍未接通 | 原`extension_blend_space_workspace.zui#WorkbenchExtensionBlendSpaceWorkspace`有57个control IDs/7个本文件事件；导入Details另有事件，现有selection仍静态profile，创建状态仅GraphRef | 保留源身份及六条原Transport路由；动画接真实事务和时间/参数，角色提供subject，Parent交生产GPU backend。未授权增加资源kind或翻capability。 |
| 正式导出、产品与交互验收缺失 | 原始ZUI未动；没有generic owner导出、资源闭包/EXE指纹、原生捕获或真实播放 | Parent/原pipeline owner运行插件检查与正式导出；受管Rust、UI选择/焦点/弹层返回/滚动/长文本、实际150%DPI、并排/差异与样式往返全部pending。Penpot标记修改不等于产品截图变化。 |

本轮只修改本域计划/设计/交接；Triggered修复与三条测试不重写，compiler/capability不改。两个原请求r04 QueryOnly均exit2/pending/ordinary_entry_unproven，未起Cargo；完整精确测试名/发现链、原身份和命令范围见`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/managed-resume-r04.json`。仍待原owner/monitor的capture-ready、ETA和参与路径，由Parent安排有界源码写稳定窗。AN-G13、真实UI/EXE和公共导出保持open；没有新服务、桌面或DPI操作。


### Curve-only UnsupportedSurface 设计接续（2026-10-25，rev589）

本域在原文件/page `b4a1e3ce-9e6a-81fe-8008-d69d60a13f99` / `b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a`，只用原 Curve component `78492c86-3bb4-47dd-97c6-752d6ff37e3d` / main `599963dc-b91b-4740-926b-d38154aa8f3a` 形成六个可编辑原生实例：显式 default 与 mixed key interpolation 各 1280×800、900×620、640×520。它们是共享表面 owner 的拒绝状态验证板，不是资源实际资产或完整 Toolkit。源输入来自资源真实测试定义的 source fixture，Rust 执行未被证明；expected kind 仍须真实 semantic catalog/Toolkit admission，不能用此设计代替。

rev570→588 新增150对象，原5738非root对象与三main records逐项不变；最终rev589仅修正六个说明文字叶的 line-height1.0→1.5。r01真实行框暴露两行重叠5.671875逻辑px，保留原图/失败；r02实际行框分离0.984375逻辑px，并非与UE/EXE的1DIP误差验收。12 caption保留NotoSC且有Shared rev5真实secondary color ref，不是main继承/全局样式AB；150 shapeRef全部解析，6实例未新增Timeline/Transport/main/clock。

r01/r02各12张原工作区PNG均逐张打开；最终r02保存关闭fresh context重开仍rev589，data SHA `61940856fe82a321c12fdd68903bdbe2fefea729cb9e8dd15b9d31d1ade06dbb`，六对未缩放画板RGB差均0。71/98/134%是Penpot适应缩放，不是系统DPI。正式Curve ZUI/source-owner codec、消费者host/输入/撤销/保存/actual clock或采样ACK、正常EXE及144DPI、双端逐图仍open，LayoutReviewCase未就绪，passed/accepted=false。实际身份、引用、逐图、差异和进程退出见 [phase20交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-rejection-20/handoff.json)。

同轮独立源审阅发现 Sequence AddKey/RemoveKey 会将非法FPS换成30；两私有路径最小候选改为按compiler同门限明确拒绝并在首次源查改前返回错误，3回归候选包含完整bytes/last-good/CAS/history不变。候选未应用，新增leaf未写入checkout，未并入原Bool/palette/Curve或Runtime请求；[候选与边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-frame-admission-r01/summary.md)。现有Bool实际修复仍保留原源字节。


### Sequence 帧命令 FPS 准入已落实（2026-10-25，r02）

原r01未应用提案保持历史；本轮在fresh前像daa0b9c4…匹配后，已将同一e87613a3…最小补丁应用到原 `mutation/sequence.rs` 及新增私有 `tests/sequence_frame_admission.rs`。当前源码分别f6c8adb0…/cbf0bc0e…，AddKey/RemoveKey在查改track前拒绝nonfinite或<=f32::EPSILON的FPS，不再选择30来增删错误时间的键；有效FPS的原换算、Bool sampler、精确键与稳定binding修复保留。Session非法FPS的显示fallback仍未改，它不授权作者源时间或预览clock。

三项必要回归已接原literal cfg(test)模块：非法FPS/符号零/NaN/Infinity下完整asset bytes不变，有效极小正FPS/0.5/30/120的原换算和no-op，原Document/CAS已有last-good后prepare错误保留revision/current compile/last-good/dirty/history/undo-redo。两文件rustfmt、diff、反向apply只读检查及canonical scoped change-guard通过，guard0errors/0warnings/0suppressed；三测试执行0、未类型检查。 [实际应用/源hash](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-frame-admission-r01/applied-r02.json) 与 [静态回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-frame-admission-r01/static-validation-r02.json)。

原Bool397/Palette385身份、输入、投递与失败不重签或重投；旧Bool mutation源daa0…现在是有明确作者修复后继的历史字节，不能拿旧closure/guard验收f6c8…新源。本轮无新Jenkins身份/QueryOnly/POST/Cargo、服务/桌面/Penpot/ZUI操作；原rev589图仍是其时点设计证据。公共host/Runtime/compiler/schema/clock未改。受管Rust、完整作者UI/真实采样、正式导出及EXE/150%DPI/全族验收保持open，goal active，passed/accepted=false。


### Sequence FPS 拒绝与预览投影接续（r03，2026-10-25）

本轮在原owner路径实际落实公开timeline_foundation及内部project_sequence_timeline的非法FPS拒绝；不再将显示fallback的30FPS换算成可交给PreviewPlayback的源秒值。有效FPS、源key秒值、UI帧选择及原Document/CAS/history保留；这是foundation投影准入，不是生产clock/播放器或GPU接通。原mutation f6c8adb0…及三项frame_admission回归cbf0bc0e…未再修改。

此前r02遗漏了一项既有回归：sequence_frame_time_invalid_fps_keeps_fallback_and_source_diagnostic仍expect非法FPS AddKey成功。此次明确修正为sequence_frame_time_invalid_fps_rejects_mutation_and_preview_projection，检查Add/Remove错误、完整codec bytes/NaN bits不变、只读帧显示与compiler诊断保留；不是已执行Rust失败。新增同一Session经原AnimationEditCommand/CAS/唯一history提交非法FPS、undo恢复有效投影、redo再次拒绝的回归。旧优化测试的空源/顺序断言保留，仅为内部Result显式unwrap。三路径增量[补丁](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-frame-admission-r01/candidate-r03.patch)及[applied-r03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-frame-admission-r01/applied-r03.json)记录前后像；保护的Runtime/compiler/catalog和r02两源未写。

三文件rustfmt/diff/reverse-check及change-guard均0错误；8个sequence_frame_回归定义、实际Rust执行0。原wrapper DryRun exit0只给出zircon_editor/defaults off/features[]/lib/locked/jobs1/threads1/filter sequence_frame_的argv，无--profile（Cargo test默认test），未验证当前server recipe/源资源准入、未创建identity或提交。现host还没有消费此foundation形成真实clock/frame，正式导出/EXE/150%DPI和全族验收仍open；不把Err准入算禁用按钮/实际采样或产品通过。


### Sequence侧栏drawer候选（phase21，2026-10-25）

原文件/page保存关闭fresh重开rev616/6513objects，新增900/640各Details与Asset Browser四张native候选，board分别6242403f…/fa587fbd…/4264635e…/a65eb5bc…。原5888非Root、3main及Sharedrev5不变；624新增对象、52copies（44Shared/8owner）、328refs有效。原20/60/20默认板保留，1280默认板实际捕获。STabSidebar半宽上限给438/308，rail28/overlap4仍候选；属性名列105、gap4、值列stretch和搜索surface32来自现有源。

28文本实测无声明横向溢出；十张原工作区PNG逐张打开，五对严格内部crop无resize RGB差0，原图保留；不是normal EXE或nativeDPI。640左Asset Details裁切、drawer遮挡中心timeline/焦点返回/正式typed value slot仍open；不把设计fixture或未接通预览算完成。原Timeline外组合与单Canvas、Curve source/Transport6events边界保留，官方export/Plugin/Rust/EXE/144DPI仍pending，passed/accepted=false。完整板ID、失败接续、引用、逐图/差异与实际进程终态见 [phase21交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-drawers-21/handoff.json)。


### 私有倍率变更回执与资源结构候选接续（2026-10-25）

现set_playback以实际存储rate位值判断changed，修正1.0到相邻f32恰差EPSILON仍写值却报false的情况。重复位值no-op、非有限请求整次拒绝保持；只改原session/sequence.rs及新增私有3回归，不改clock、Public Host/六事件、source history或capability。最终fmt/diff/reverse/guard0/0/0，18保护源保持，Rust0。资源RT-S25-KEYS01原codec已有跨通道整批typed insert/remove候选，仍不构成语义准入/真实菜单/保存能力；沿共享合同r05消费，不造第二adapter或Delta。[实际变更/失败与边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-playback-ack-r01/handoff.json)。


### 状态转换 duration 精确事务接续（2026-10-25）

已在原 mutation/state_machine.rs 修正既存转换 duration 的变更回执：原 f32 EPSILON 比较会将合法极小秒值改0、符号零或相邻 finite 值写入 clone 后报false，store.prepare_mutation据此丢弃候选。现只比较实际存储位值，旧frame→30FPS秒换算及compiler有限非负约束保留；UE AnimStateTransitionNode.h:38-39 的 CrossfadeDuration 是 seconds浮点量，保留旧frame合同不代表该时间差异已对齐。前像2b67e8d9…为原条件修复后像，现e0afe5aa…是本批后继；set_transition_condition全文和原state_machine_conditions.rs逐字保持，不重套Triggered或把旧条件请求当本批验收。

新增私有 state_machine_duration.rs 三项回归覆盖完整source/codec、Document prepare不写源、单History/CAS事务的dirty/undo/redo/当前编译与last-good、真实no-op与缺目标清洁性。补丁07a88aac…，fmt/diff/reverse/guard全0且guard0/0/0，26保护源保持；Rust执行0。精确[后像/保护检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/state-machine-duration-r01/applied.json)、[静态检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/state-machine-duration-r01/static-validation.json)、[必要聚焦范围](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/state-machine-duration-r01/test-scope.json)保留。未创建或提交新的验证身份；正在等待的播放回执queue18979/#408仍只有其原3测试filter及immutable输入，本批是后继checkout源，不借其结果。

另已确认非Bool AddKey取末键值：现有Linear(0s,1)/(1s,9)在0.5s的末值9与真实采样点值5不同，UE SequencerChannelTraits.h:198-206 EvaluateAndAddKey会先求值。已有raw/compiled sampler仍private，公开纯查询仅Bool。编译产物只读出口可复用sample_compiled，但必须关联当前document revision/唯一target，不能拿last_good当当前源，也不能以单点采样声称Hermite切分或极短interval形状保留；无效中间文档仍须可编辑。具体[原调用点/公开出口边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/state-machine-duration-r01/source-followup.json)交Parent，Runtime/公共host/原6events/共享3main及RT-S25 codec未改。

本批没有Penpot/服务/桌面或新编译动作；正式导出、真实clock/GPU/EXE/实际150%DPI与完整窗口族验收继续open，passed/accepted=false，goal active，释放0B。

### Curve 原请求终态与 compiled-track 只读出口（2026-10-25）

原 curve-lossless queue16509 已精确对上 management #393 FAILURE，109B archive 的拒绝为 `.cargo/config.toml=null` 只读依赖在自动 context 前触发 control-path 检查。两源+十依赖12/12匹配，原 filter 静态8项（7普通+1ignored），Rust发现/执行未知、verified0。旧 pending/输入/marker/失败全部保留，不删依赖或重投；palette窄例外不覆盖本recipe。[本域终态消费](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/curve-lossless-admission-r01/original-terminal-consumption-r01.json)。

已形成公共Runtime四路径**未应用**的 `sample_compiled_animation_track` 精确diff，复用原sample_compiled，原Bool出口/Step边界/private traits不变；rustfmt解析与git apply --check通过，两回归定义未执行，源码未写。当前非Bool AddKey仍取末键，当前revision/唯一真实target、一次Document/CAS/history与invalid中间态策略须接入；last-good不能充当前源，单点值不保证Hermite切分形状。[补丁及范围](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/compiled-track-query-r01/summary.md)、[consumer准入](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/compiled-track-query-r01/consumer-boundary.json)。RT-S25不引入Timeline/FPS/clock；公共整合交Parent。无新身份/编译/query/capture/服务/桌面动作，goal active，真实EXE/DPI/完整族验收open，passed/accepted=false，释放0B。

### Sequence 跨实体重绑的非拥有空 binding 隔离（2026-10-25）

已在原作者 `mutation/sequence.rs::rebind_track` 修复新发现的数据隔离错误：合法移动唯一源track时，原retain_mut会顺带删除同entity下未拥有该track的空binding及其target身份。现仅实际移走track的binding依原规则判断是否清退，其他空binding/target_id/原顺序完整保留。源compiler允许空tracks和同entity多binding；UE MovieScene操作也按明确binding对象/GUID定位。原同实体重绑、目标选择、缺源/冲突/歧义、旧时间与Bool修复及源owner最后一轨清退规则保持，未宣称解决typed目标选择或骨骼Montage。

新私有两回归定义覆盖12组直接重绑（owner target Some/None、有无余轨、目标新建/既有Some/None）和6组原Document/store prepare→一次command/history→codec重读→undo/redo，连同完整target/顺序/compile/dirty/no-op。23项保护与原sequence_rebind.rs逐字保留；rustfmt/diff/reverse及guard均0，guard 0/0/0；Rust类型检查/执行0。真实wrapper DryRun已0退出，仅证明root zircon_editor/lib/defaults-off/features[]/locked/filter sequence_rebind_isolation_/jobs1/threads1配方文字；metadata dev/无--profile/Cargo默认test分列。该slice向既有受管milestone lane交独立scope，未创建新request/attempt/identity，不扩旧请求。[精确补丁与验证边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-rebind-isolation-r01/summary.md)、[有限测试scope](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-rebind-isolation-r01/test-scope.json)。

另原播放回执 queue18979/#408于14:44:16.363Z成为Pipeline FAILURE，console终句为Resource admission remains pending；最后archive仍waiting/checkout_source_busy，不能当Rust失败/通过或native EOF/释放证明。[本次真实终态GET](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-playback-ack-r01/validation-server-408-terminal-r01.json)。原identity/input/marker与等待历史均保留，不重投/扩大filter。本次新私有源是该旧完整checkout观察的后继，不冒充旧输入已覆盖。公共sampler四源/未应用proposal、6events/3main/主题/原ZUI均未改；无新Cargo/capture/Penpot/桌面/服务，真实预览/导出/EXE/150%DPI/全族验收open，goal active，passed/accepted=false，释放0B。


### Sequence path-only 选择准入 r01（2026-10-25）

实际 Session `select_timeline_span` 先核当前完整 entity/property 路径的唯一真实轨道；缺轨 false、多 binding 同路径明确拒绝且保留旧选择及 transport。`reconcile_source_change` 在原 focused Document undo/redo 后清理缺失/歧义选择，源恢复不复活已退休的 transient selection。原 host `editing.rs/history.rs` 与 `pane_presentation.selection_summary` 链路保留；未造 target ID、绑定位置 identity、第二历史或 clock，正式 typed binding/revision 接线仍 pending。

新增3项回归定义，原 Curve 歧义回归强化入口拒绝且保留旧选择的投影拒绝；Rust执行0。最终 fmt/diff/reverse-check/guard通过（0错误/0警告/0抑制），10保护依赖不变。原 playback setter 与测试字节保持，但 Session 文件为后继，旧 filter 不覆盖本切片；原 palette/Curve/playback 身份及失败不改、不重投，无新 Cargo/Query/POST/capture/freeze/桌面/服务。Penpot rev616/phase21 原图为历史，非本修复 EXE 证据；正式导出/typed target/实际窗口/144DPI及全族验收继续open，goal active，passed/accepted=false，释放0B。[源差异与检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-selection-admission-r01/summary.md)。


### Montage Sections 三尺寸双状态原生流式板（2026-10-25，phase22）

原动画file/page现rev642/6831对象，新增1280×800、900×620、640×520的Next=None/自环六板，均从原53对象Sections板原生复制并保留引用。原6512个非根对象、原3main、Shared rev5未改；新增54 native heads/108 refs，36 heads可见，隐藏比较状态不算新消费。命令/链使用原生流式布局和换行；Preview All修正保留。

14张原工作区PNG已逐图打开。r02六对画板内RGB差0仍发现1280 None缓存箭头：r03仅重建该叶positionData，content/盒子/shapeRef不改，保存关闭fresh重开后为Next方块；其余五板子树不变，明确复用rev641图。r03最终对有194像素差≤2 RGB色阶，零差断言失败保留。最终87个可见文本记录行框在自身/父盒内，7/7限定源pin匹配；不是UE/EXE 1DIP或native DPI验证。r01辅助脚本失败及同身份续作过程保留。

这六板是Sections源夹具；真实Montage模型/clock/authoring/history、精确FontAwesome/Combo/menu输入与焦点、滚动长名、官方ZUI/style/export、完整Toolkit/normal EXE/GPU/150%DPI仍open（MT-S01..07）。未改原6events/业务ZUI/public源，未新建main/clock/store/producer。manifest readiness不算插件测试，无新Cargo/Jenkins身份/query/replay、桌面或服务动作。3个自有browser已退出，释放0B，passed/accepted=false，goal active。[实际产物与差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-responsive-22/summary.md)。


### Sequence 新建轨道 binding 准入 r01（2026-10-25）

已修复原 CreateTrack 在缺失属性时按同entity第一个binding插入的问题。现写源前只接受零或唯一候选，多binding明确InvalidTrackPath；现有唯一完整轨道仍false/no-op，重复轨道沿原规则拒绝。原target_id完整十进制字符串、空binding和源顺序均保留，零候选新建None/唯一候选复用原target。没有增加typed DTO、迁移旧源或修改compiler/capability/Runtime/公共host/history/clock。UE UMovieScene::AddTrack:1173以明确ObjectGuid查FindBinding后才Modify；本修复不等同typed选择已交付。

改动只在原mutation/sequence.rs、原tests/sequence_rebind.rs及新增tests/sequence_create_admission.rs。旧lookup回归的已有路径no-op/Document/history断言保留，错误的首binding插入期待改为拒绝并保完整source/codec。新3定义覆盖16直接拒绝、16原store.prepare拒绝及5个合法prepare→一次原command/history→codec→undo/redo循环，校验revision/compile last-good/dirty/undo及完整target/顺序；未来同私有module过滤器静态37定义，Rust发现/执行未证明（verified0）。最终fmt/diff/reverse/guard通过，guard 0错误/0警告/0抑制，29保护路径未变；首次新文件格式失败及修正保留。

现production codec→dispatch→host→store链保持，prepare错误在开启transaction前传回，不把它报Created成功；这是实源审阅而非新GUI/host-lifecycle验证。实际typed binding/document/view/revision交接、官方源导出/真实GPU/normal EXE/144DPI/双端验收继续open。根zircon_editor/lib/defaults-off/features[]/locked/单jobs单threads的37定义候选交原Jenkins milestone lane；未调用wrapper/DryRun/Cargo、未创建identity/query/重投或扩旧请求。原palette#385、curve#393、playback#408及其旧输入/markers/失败完整保留。goal active，passed/accepted=false，释放0B。[增量补丁与精确验证边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-create-admission-r01/summary.md)。


### Graph 默认参数作者位事务 r01（2026-10-25）

原 `SetGraphParameter` 对 Scalar/Vec2/3/4 使用数值 PartialEq，符号零编辑虽改写 clone 却返回 false，store.prepare_mutation 因而丢弃候选。现仅原私有 mutation/graph.rs 按各浮点分量 to_bits 判断作者值变化，原 Bool/Integer/Trigger 比较、类型/finite 解析、既有节点 legacy 保护及唯一 Document/CAS/history 保持。没有更改 Runtime DTO/codec/compiler、公共 host、clock、capability 或原6events。

新增3项私有回归定义：20组每分量双向 ±0 的完整源/codec；20组 prepare→一次原command/history→codec重读→undo/redo；120组精确no-op/非法literal的完整源、revision/last-good/dirty/history不变。最终 fmt/diff/reverse-check 与 guard 均通过，guard 0错误/0警告/0抑制；首次新文件标准换行格式失败保留。20保护pin不变，原 graph_output_identity 9定义原文保留，当前该私有 Graph module 12定义；Rust发现/类型检查/执行尚未证明，verified0。

同属本域的新代码不扩原 r02/#416 的5项 sequence_key_exact_；该笔的签认输入保持，实际运行结果按原句柄消费。新的 Graph scope 仅交既有受管milestone准备，未创建identity/Query/POST/Cargo或改旧失败。typed target、正式ZUI导出/GUI、真实GPU/normal EXE/150%DPI与整族passed/accepted继续open，goal active，释放0B。[精确增量与验证边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-parameter-bits-r01/summary.md)。


### Montage 时间轴独立页三尺寸与行框实测（2026-10-25，phase23）

实际原文件rev642/6831→rev661/7067，原3main、6830非根对象与Shared5不变；新增3板/236对象，Timeline与Transport共6 native copies/141refs全部解析。沿原流式root分配和Canvas实际allocation，Section/Slot/Notify行高24/48/24，标尺、分段、通知、cursor共用0..2s plot。f32迭代修正重复末端；r02 RGB0仍查出36自身/15父盒超限，r03仅按语义行高居中36文本+6自有clip frames，57最终行框最大溢出0.00634765625逻辑px，字体/字号/文字/段落/6events与Fill不变。

最终6张PNG及3差异图逐张打开，三对画板内RGB0，保存关闭fresh重开同数据。仅独立dock-tab候选，DefaultSlot未解析真实Skeleton组，Clip A/B、Window/Hit和0.5s仍是原夹具；UE默认Toolkit及640裁切、正式源导出/真实clock/GPU/normal EXE/实际150%DPI/全部交互和双端<=1DIP均open。没有把三main注册/ref解析或该独立页当作产品通过，passed/accepted=false，goal active。[身份、逐图、差异与未完成项](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-timeline-focus-23/summary.md)。

原queue20063/#416已真实FAILURE，550B原receipt waiting/checkout_source_busy与blockingReadOwner及控制台Resource admission remains pending保留；Cargo/native EOF/资源释放仍未知、verified Rust0。原身份/input/marker不重投，不借其5项filter验收本片。[服务器终态](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-key-exact-r01/server-build-416-terminal-r03.json)。r01读取ECONNRESET与认证头异常已在自有日志脱敏，r03写前ABI失败保留；4自有客户端退出，释放0B。
