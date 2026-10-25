# 粒子与效果编辑器：Niagara 源码映射与实施记录

本窗口族由用户指定给聊天 `01a16d52-a4a5-7271-90a2-336091337737`；Parent 为 `01a16d4e-0491-7353-84fa-038bd7706546`。范围覆盖 Particle System、Emitter 及有关模块、参数、曲线、时间控制、图编辑和实时预览。原 overview 的“材质与效果”初始范围包含 Niagara；此处按用户新的独立分工承接粒子域，不改写其它聊天的记录。

最近Penpot实证（Shared palette接续；本轮未复读）：原file已保存并关闭fresh重开rev41/vern0，仍3页388对象。原Stack的14个业务frame直接引用Shared的surface.0/1；整份data排除这14处fills后摘要与rev40完全相同，18copy、106source refs、组件、文字及布局未变。当时Shared为rev5、原4main/34颜色/3文字样式，Button main有1处真实填色引用、typography非空引用仍0。VFX五类只读投影源码已交付，宿主尚未安装；当前Editor34（21 direct＋13 curve，含本轮5项批量候选）、Runtime10、VFX5分包各执行0。typed输入、真实Graph/clock/preview/save、正式ZUI、正常EXE/150%DPI和双端逐图仍open，passed/accepted=false。

历史状态（空端口消费者接续，rev38）：原file已实际保存并fresh重开rev38/vern0、3页388对象（266＋35＋87）。原18个copy=7业务＋11 Shared、106个source refs及1个私有main保持；640能力说明原叶已修改，11个原linked文本均引用Shared body/strong并保留中文和shapeRef。原生Update后library仍rev2，syncedAt已为2026-10-24T18:39:30.407199Z；Shared4主件/34颜色/3文字样式保持，但同响应21处typographyRefId均null，master非空style refs为0。实例本地样式消费和自动master继承/reset分开记录。新System/Emitter只读source投影复用GraphNodeView、空Vec和域禁用原因，Editor29项与Runtime10项候选各执行0。真实VFX五类typed接线、Graph/input/focus、正式ZUI、生产preview、正常EXE/150%DPI与双端逐图仍open，passed/accepted=false。

当前锁生成前置（本轮原owner交付）：受管resolver已实现，原JM generation server preparation真实建立；实际资源/sole-lock writer未准入、resolver未启动，受信独立锁输出及后继源准入仍pending。详见本文末尾；管理检查不计产品通过。

## 权威输入和修改边界

- 实际 checkout：`E:/Git/ZirconEngine`，开始时 HEAD `75a76ee963792d5a316f738519e966937f770036`，分支 `main`。共享工作树有其它聊天的改动，保持原状。用户明确要求此 checkout；不创建源码副本、snapshot 或另一个 worktree。
- UE：只读 `dev/UnrealEngine`。`Engine/Build/Build.version` 自报 6.0.0、BranchName `UE5`、Changelist 0，不能把它当作已验证的发布版本或提交号。后续以选定参考文件的 SHA-256 绑定源依据。
- 原始生产资产：`zircon_plugins/particles/editor/{authoring,preview,particle_system.drawer}.zui`，保留现有 asset/root/control ID。尚未写入生产资产；先完成用户要求的真实 Penpot 阶段。
- 公共 catalog、tokens、Starship、菜单/窗口宿主和跨窗口协议归 Parent 统筹。本聊天只提交明确依赖，不建立平行主题或复制通用组件。
- 产物清单：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/artifacts.json`。无本任务编译产物、源码副本或自有长期进程；保留实际Penpot结构预览、失败图和保存/重开记录，实际释放0字节。隔离浏览器使用自有E盘临时目录，不操纵共享桌面。
- 当前没有 `read_thread` / `send_message_to_thread` 工具；本文是共享文件交接。Parent已明确回报读取先前版本，该事实不补记失败投递为成功，也不证明已读本轮增量。桌面输入/捕获的排队确认仍待协调，未占用桌面。
- Parent 后续已明确统一协调服务、公共库/文件页面和桌面。已按其指定的 `codex queue --thread ... --message` 汇报，命令 exit0，返回消息 ID `01a16d5d-d303-7d60-b220-642b49f48ab1`；只证明入队，未证明已读。本聊天不再启动/恢复 Jenkins/Penpot，不占桌面，不重复问用户服务信息。
- 已读取随后新增的 [主工作台交接](main-workbench-handoff.md)：原 Runtime11/P21 持有者和四个服务进程均已缺席，15432/16379/16060/19001 关闭，原终态缺失。旧数据库、账户和 Penpot 已保存文件应保留；本聊天没有另启 Penpot。用户明确要求等待 Parent 告知共享入口。

### Parent最新分工与真实身份

最新权威为 [Parent会话交接](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)，上面的Runtime11/P21和Jenkins失败是保留的历史观察。Parent已恢复独立服务并记录真实身份；本聊天没有启动/恢复服务、修改其身份或接管进程。

| 项目 | 已核实身份及边界 |
| --- | --- |
| 粒子设计文件/初始页 | file `b4a1e3ce-9e6a-81fe-8008-d69d60c876ee` / page `b4a1e3ce-9e6a-81fe-8008-d69d60c876ef`；[04 Particles and Effects](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60c876ee&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60c876ef) |
| 项目/团队 | project `b4a1e3ce-9e6a-81fe-8008-d69d606f6160`；team `822661f1-7ba3-81f7-8008-c6dcb4ae6a8f`；分配回执为Parent报告根的 `penpot-assignments.json` |
| 初始独立API读取 | 复用P21认证，只在内存使用cookie；两个独立上下文get-file/get-file-libraries均HTTP200，当时四文件均revn0/vern0、仅根节点、无组件/样式/外链库。此为设计前历史；随后本域文件已发生下述实际保存 |
| 时间轴/曲线/播放 | 动画owner `01a16d51-741b-73b3-a61a-2189cd38f81e`；复用现有TimelineModel/CurveModel/Transport，不把粒子专有单位/模拟语义写进公共控件 |
| 节点图 | 材质owner `01a16d50-6c7f-7370-853d-4581184b66a1`牵头；复用GraphModel/Canvas/command保留域schema，粒子不另建通用图表面 |
| 其它公共合同 | Parent负责host/Dock/menu/属性行/主题/catalog、跨域协议和集成；重叠路径先交接，不覆盖其它owner改动 |
| 桌面 | 尚未获得时段；可独立API或隔离无头浏览器设计。实际EXE/资源、board和状态/DPI就绪后，按记录发送具体请求，Parent明确授予后才能操作桌面 |

本聊天读取证据为报告根的 `penpot-assignment-readback.json`；它只证明文件身份与库存，不是设计save/close/reopen收据。最初检查把JSON camelCase `pagesIndex`误按Transit字段读取的失败与修正一并保留。向Parent实际汇报消息 `01a16d76-23d2-7d21-95a1-631e98607572` 已排队，尚无已读证明。具体消费合同见 [共享组件适配](particles-shared-contracts.md)。

### 真实Penpot结构草案（revn7阶段历史）

局部作者body、独立Preview body和场景Component drawer各有1280×800、900×620、640×520画板，另有三种作者状态各三尺寸，共18板；这些是body尺寸提案，尚未与公共host的完整客户区分配匹配，不算三个客户区或150% DPI验收。实际UUID和职责见 [设计身份表](particles-design.md#真实penpot局部草案身份) 和新增状态身份表。当前revn7/vern0为266个原生shape、131个flex frame、134个可编辑text。原业务组件与3实例保留，新增4实例复用同一main并用native shapeRef/touched覆盖模板文本；没有复制公共按钮、输入框、图或时间轴。runtime默认128/16与创建模板256/32分别标注为设计fixture，没有冒充已加载产品资源。

记录在报告根 `penpot-layout-draft.json`。原 `penpot-layout-draft.png`保留顺序反转失败，修正版另存，不能覆盖失败图。JSON的`set`操作不解码`:any`中的layout enum/geometry而触发真实500，随后改用Penpot支持的`assign`并通过服务器校验；没有使用skip-validate。另已修正原control引用、JSON namespace投影读取及默认seed为0xC0FFEE（12648430）。这些失败/修正均留在同一记录。公共共享样式/组件实例、交互状态设计、官方ZUI导出和产品加载仍待完成，不将占位surface或动作文字当已接通控件。

新增状态的实际保存及重开见 `penpot-state-design.json`、`penpot-state-reopen.json`。初次点击因同名画布标题在视口外而超时，改用真实图层UUID后完成640错误态与900空态捕获，失败保留。revn6→7期间三个旧实例图层名同步为main名称；仅在内存还原这三名，稳定身份/层级/组件引用/业务元数据摘要即可完全匹配revn5，不把全字段摘要变化误称为几何变动或无文档保存。随后两个fresh context保持revn7和相同内容摘要。实际中文SVG字体为系统Noto Sans SC回退，尚无公共字体依赖闭包及Zircon端等价证明。

## UE Toolkit、TabFactory、Slate 与样式依据

以下 `N/` 均为 `dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/NiagaraEditor/`。分类的“适用”表示 Zircon 产品应覆盖该职责，**不表示当前已经实现或通过**。条件性功能保持待定，不通过隐藏功能缩减复刻目标。

默认布局依据 `N/Private/Toolkits/SystemToolkitModes/NiagaraSystemToolkitMode_Default.cpp:25` 的 `Standalone_Niagara_System_Layout_v32`：左侧上方 Viewport/Blueprint Preview，下方 Parameters/User Parameters/Scratch Pad；中部上方 System Overview/Document/Baker，下方 Timeline/Curves/Log/Stats；右侧 Details 与关联页签。源码使用 splitter 与约 0.33 的初始系数，不能把全部系数当作对每个尺寸不变的像素宽度。

完整 System/Emitter Tab 注册入口是 `N/Private/Toolkits/SystemToolkitModes/NiagaraSystemToolkitModeBase.cpp:237`。Emitter 通过 `NiagaraSystemToolkit` 的 emitter 模式编辑，并非凭名字另造第二套 Toolkit。此入口共 21 个启用的本地 spawner；SystemParameterDefinitions 的 spawner 在该处被注释，另外的外部扩展仍可注册 tab。

补充核对 `N/Private/Toolkits/*Toolkit.h` 的实际继承：共有System、Script、ParameterCollection、ParameterDefinitions、SimCache五类Toolkit，均已在下表或关联表覆盖；没有独立的EmitterToolkit类。外部factory由ModeBase:357调用 `UNiagaraEditorSettings::GetExternalTabSpawnersForAsset`，再逐个 `FWorkflowTabFactory::RegisterTabSpawner`；Settings.cpp:742仅 `ExecuteIfBound`。在当前只读 `Engine/Plugins` 的C++源码搜索中，`SetOnGetExternalAssetTabSpawners`只发现Settings.cpp:707定义，未发现具体安装调用；因此没有可据此追加的具名外部tab。该静态搜索不是UE运行态插件启用清单，不把“未发现安装调用”写成“外部插件永远没有窗口”。

| UE 窗口/Tab | 分类 | Zircon 对应能力与理由 |
| --- | --- | --- |
| System/Emitter Toolkit，System Overview | 适用 | runtime 有 `ParticleSystemAsset.emitters`；需同一 source document 的系统/发射器选择，不采用固定 Sparks 列表 |
| Viewport | 适用 | runtime 有 CPU simulation、sprite snapshots 与 render extract；Editor 尚无 PreviewWorld/真实 viewport consumer |
| Curve Editor | 适用 | runtime 有 scalar/color lifetime keys；需 key 身份、单位、选择和可撤销编辑 |
| Sequencer / Timeline | 适用 | runtime 有 play/pause/stop/tick/rewind；需真实 clock、seek/reset 和组件寿命管理 |
| Details / Selection | 适用 | 有 emitter 参数和 component 结构；需 typed mutation、焦点返回和 source/runtime 状态区别 |
| System Parameters | 适用 | 映射 backend、seed、loop、emitter 属性；不能声称支持 Niagara 参数命名空间、绑定和 type system |
| User Parameters | 待定 | 尚无 Niagara 风格用户参数 store 与覆盖层；保留目标、依赖 source schema 决策 |
| User Parameters Hierarchy Editor | 待定 | 依赖用户参数 store 与 hierarchy 身份；当前无真实数据可编辑 |
| Selected Emitter Graph | 适用 | 用户明确要求节点编辑；当前粒子 emitter 为固定字段，无 Niagara graph source/编译合同 |
| System Script | 待定 | VFX runtime 已有固定五类节点 lowering/executor，但没有 Niagara 通用 script document/VM；不能用装饰连线代替 |
| Scratch Pad Scripts | 待定 | 缺少可保存/复用的 module script、签名、局部/全局依赖和编译器 |
| Scratch Pad Hierarchy Editor | 待定 | 依赖真实 scratch-pad module 模型；当前没有可编辑层级 |
| Emitter Summary View Hierarchy Editor | 待定 | 可先做真实参数摘要，但 Niagara summary hierarchy 的持久化/版本层未实现 |
| Message Log | 适用 | runtime 已有 bounded diagnostic page、sequence 与 dropped/stale 信息；尚未被粒子 Editor 消费 |
| Script Stats | 待定 | 当前有 particle counts 和 GPU feedback DTO；没有真实 script instruction/performance 数据 |
| Generated Code | 待定 | 需实际编译 IR/code/source map；VFX pass 名称字符串不等于生成代码 |
| Preview Settings | 适用 | 预览相机、背景、网格与能力反馈；需绑定实际 preview scene，不改变主场景 |
| Debug Cache Spreadsheet | 待定 | runtime snapshot 可提供当前值；没有 Niagara simulation cache、逐帧 attribute schema |
| Baker | 待定 | 当前无粒子缓存/flipbook 烘焙和 artifact publish；禁用入口须说明真实缺口 |
| Versioning | 待定 | 缺少 emitter/module source version、迁移与依赖更新合同 |
| Blueprint Preview | 不适用本粒子实现 | UE Actor/Blueprint 驱动预览无对应 Zircon Blueprint runtime；真实组件/场景预览仍属于适用 Preview Settings/Viewport |

Scalability 模式追加核对 `N/Private/Toolkits/SystemToolkitModes/NiagaraSystemToolkitMode_Scalability.cpp:33`：实际注册 `ScalabilityContextTabID`，SystemAsset 模式另注册 `ResolvedScalabilityTabID`；`ScalabilityViewModeTabID`、`EffectTypeTabID` 只有常量定义，不能计为此函数的活动 tab。两种 System/Emitter scalability layout 均存在。

| Scalability 关联窗口 | 分类 | 原因 |
| --- | --- | --- |
| Scalability Context / Active Scalability | 待定 | runtime 有容量/backend 等配置和诊断，但没有 Niagara device profile、distance/significance/effect type 规则及 resolved view；不能用固定性能数据填充 |
| Effect Type 规则编辑 | 待定 | `AssetDefinitions/AssetDefinition_NiagaraEffectType.cpp` 的 `OpenAssets` 实际通过 `FSimpleAssetEditor` 打开，diff 用 `SDetailsDiff`；不是 Scalability 模式常量对应的已注册独立 tab。Zircon 暂无对应可保存规则模型 |

关联 Toolkit 入口仍需分别保留：

| UE 关联编辑器 | 源入口 | 分类及理由 |
| --- | --- | --- |
| Script / Module / Dynamic Input / Function | `N/Private/Toolkits/NiagaraScriptToolkit.cpp`；NodeGraph、ScriptDetails、SelectedDetails、Parameters、InputPreview、Parameters hierarchy、Stats、MessageLog、Versioning；ParameterDefinitions 注册被注释 | 节点编辑职责适用；可运行 script/model 依赖待补，不宣称整个 Toolkit 可用 |
| Parameter Collection / Instance | `N/Private/Toolkits/NiagaraParameterCollectionToolkit.cpp:162`；`NiagaraParameterCollectionEditor_Main`，`Standalone_Niagara_ParameterCollection_Layout_V1` | 待定；当前粒子 schema 无全局 collection/instance override 模型 |
| Parameter Definitions | `N/Private/Toolkits/NiagaraParameterDefinitionsToolkit.cpp:46`；Options、Parameters、SelectedDetails 三个 spawner | 待定；缺少跨资产共享 parameter definition 与订阅关系 |
| Sim Cache Editor | `N/Private/Toolkits/NiagaraSimCacheToolkit.cpp`；Spreadsheet、Viewport、PreviewSettings、Overview，`Standalone_Niagara_SimCache_Layout_V1` | 待定；runtime snapshot 不是可读写 SimCache 资产 |
| Niagara Debugger / Outliner / HUD | `N/Private/NiagaraDebugger.cpp` 与 `NiagaraSystemToolkitMode_Default.cpp` 调试菜单 | 当前 runtime diagnostics 适用；远程会话、capture/playback 与全局 outliner 需能力决策，不用假性能值填充 |

Toolkit 之外的关联窗口/弹层也列入范围：

| UE 窗口/弹层 | 源入口 | 分类及理由 |
| --- | --- | --- |
| Add Emitter / System、Emitter 创建 | `N/Private/Widgets/AssetBrowser/SNiagaraAddEmitterToSystemWindow.cpp`；`NiagaraSystemToolkit.cpp:613`；`NiagaraSystemFactoryNew.cpp`、`NiagaraEmitterFactoryNew.cpp` | 适用；已有 create CPU Sprite/add emitter descriptor，需接真实 asset/template 选择、确认、取消和焦点返回 |
| Module Wizard | `N/Private/Widgets/Wizard/SNiagaraModuleWizard.cpp` | 待定；当前无可保存/编译的通用 module script/signature，不能以固定参数行代替模块创建 |
| Pin type / parameter selector | `N/Private/Widgets/SNiagaraPinTypeSelector.cpp`、`NiagaraEditorWidgets/Private/DetailCustomizations/SNiagaraNamePropertySelector.cpp` | 实际粒子参数/资源选择适用；Niagara 完整类型及 namespace/script pin 依赖待定 |
| Curve Key Selector | `NiagaraEditorWidgets/Private/SNiagaraCurveKeySelector.cpp` | 适用；用于真实 lifetime curve key 编辑/选择，稳定 key ID 和撤销仍待补 |
| Data Channel Wizard / Asset | `N/Private/Widgets/DataChannel/NiagaraDataChannelWizard.cpp`、`N/Private/AssetDefinitions/AssetDefinition_NiagaraDataChannel.cpp` | 待定；无对应粒子数据通道资产/读写模型，不新增假字段；资产定义中现有动作是标记依赖可编译资产 dirty，不能据此声称专用 Toolkit 已存在 |
| Validation Rule Set / Details Diff | `N/Private/AssetDefinitions/AssetDefinition_NiagaraValidationRuleSet.cpp` | 当前真实 compile diagnostics 适用；可编辑 rule-set 资产与跨修订 diff 待定；该文件只定义 diff，不凭文件名发明专用编辑器 |

UE `Blueprint Preview` 的 UE Blueprint 对象模型、Niagara VM 专属调试数据和 generated HLSL 不能直接套用到 Zircon 粒子后端；保留对应通用预览/诊断职责，UE 专属语义不作为已实现功能。外部插件通过 `FWorkflowTabFactory` 添加的窗口需按实际启用插件继续扩展盘点，本表不宣称覆盖未启用的全部 UE FX 插件。

主要 Slate 核对入口：`N/Private/Widgets/SNiagaraSystemViewport.cpp`、`SNiagaraParameterPanel.cpp`、`SNiagaraScriptGraph.cpp`、`SNiagaraDistributionCurveEditor.cpp`；Overview/Stack 的实际 owner 则在邻接模块 `Source/NiagaraEditorWidgets/Private/SNiagaraOverviewGraph.cpp`、`SNiagaraStack.cpp`、`SNiagaraStackEntryWidget.cpp`，不是同名 Editor/Widgets 路径。外部 `FWorkflowTabFactory` 由 editor settings 按资产返回；本地没有一个能代表整个窗口族的单独“NiagaraTabFactory”文件。

已核对的行为约束：

- `SNiagaraOverviewGraph.cpp:174/194` 双向同步 ViewModel/graph 选择，并使用更新 guard 避免递归；重命名进入 `FScopedTransaction`。多处只改 UI selected flag 不能等价替代此合同。
- `SNiagaraStack.cpp:334` 将 tree scroll、selection 回调连到 ViewModel，采用 multi selection，重建时恢复 `GetLastScrollPosition()`；行拖拽由 `CanDrag()` gate；上下文菜单接入 Cut/Copy/Delete/Rename；`OnKeyDown` 经命令列表消费。
- `SNiagaraScriptGraph.cpp:318` 绑定双击跳转、快捷创建节点、位置上下文菜单；新增/重命名进入事务。
- `NiagaraEditorCommands.cpp:23/25/54`：无修饰 `E` 打开 Add Emitter、`Space` 是 Reset Simulation、`Ctrl+O` 为 Collapse to Headers。不能凭其它编辑器习惯把 Space 无条件做成 play/pause；输入框、弹层和 dock 焦点内的命令路由须单独验收。Save/Undo/Redo 等通用命令沿 Toolkit/Generic command binding 核对，不自造 UE 默认键位。

样式以 `N/Private/NiagaraEditorStyle.cpp` 和其 `FAppStyle` 依赖为依据，保留 stack 类型/状态与 selection 的语义；Zircon 映射到现有 tokens/Starship。UE 中不同上下文的字号不同，不能把某一条 `DEFAULT_FONT(...,9)` 当成整个窗口唯一字号。继承的布局参考是 [ReactBits App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell)、[Forms](https://pro.reactbits.dev/docs/app-ui/forms) 的区域/字段关系；具体外观和行为服从用户指定的本地 UE 源码。

## 当前引擎能力与真实缺口

| 边界 | 当前源码事实 | 实施要求 |
| --- | --- | --- |
| system / emitter 数据 | `runtime/src/asset.rs` 有 system id/backend/seed/looped/emitters；emitter 有 rate/capacity/lifetime/burst/shape/velocity/gravity/drag/material/texture/color/size keys | 建 source document revision、稳定子项身份与 typed mutation，映射实际字段，不宣称 Niagara VM/任意 module stack 等价 |
| 实际模拟 | `runtime/src/service.rs` 有 `ParticlesManager::{instantiate,remove,play,pause,stop,tick,rewind_preview,snapshot,build_extract}` | Editor 必须持有独立 preview manager/instance、clock 和真实 renderer consumer；关闭时移除实例 |
| GPU | manager instantiate 明确将 GPU 请求退回 CPU 并发出诊断；renderer-owned GPU 路径另有实现 | UI 显示实际 backend/admission/fallback，不能把请求 GPU 显示成已运行 GPU |
| authoring / preview ZUI | 现有三个 ZUI 有稳定 asset/root/control ID，toolbar/emitter/module/curve/diagnostics/viewport/transport/stats 核心节点是 `Space` | 保留 ID，在真实 Penpot 中设计，再 export/apply 原始资产；不新建一套平行窗口资产掩盖原占位 |
| command | `editor/src/authoring.rs` 注册 12 条 operation descriptor，`plugin.rs` 只注册 extension | 缺少 handler/factory、source mutation、undo/redo、durable save 及 runtime receipt；全部需真实接入 |
| core Workbench VFX | `zircon_editor/src/ui/retained/dispatch/workbench/vfx_workspace_state.rs` 有固定 Sparks、280/sec、0.65s、M_Bolt_01 | 该页面只可作为既有展示线索；不能作为实时粒子或参数修改证据 |
| VFX graph runtime | 当前 `runtime/src/plugin.rs` 已有 `compile_vfx_graph_program`，五类节点 lowering 到 `ParticleSystemAsset` / `ParticleGpuProgram`；`VfxRuntime` 有 configure/tick、prepare collector、真实 GPU simulation/transparent executor、bounded probe；configured dispatch 按容量推导 | 复用真实后端；旧评审“全部 no-op”已过期。旧 `compile_vfx_graph` 仍只做 metadata/report 校验，不能和真实 program compiler 混用 |
| VFX graph Editor | `editor/src/plugin.rs` 仍仅 descriptor，未安装 editor extensions | 缺少 graph source session、稳定 node/edge 身份、UI/commands/undo/save 与上述 runtime 的连接；不宣称 Niagara 通用 graph 完成 |
| 文档和保存 | 现有 CPU Sprite TOML template；没有与粒子 operation 连通的 open/save 文档生命周期 | 修改成功需要 revision/dirty/undo；保存需要真实路径写入与重开值一致，失败保持 dirty |

真实 VFX compiler 的边界需进入 UI：节点限 SpawnRate、Lifetime、Velocity、ColorOverLife、ShaderGraphMaterial；同类重复节点拒绝；SpawnRate 和 material 必需；rate/lifetime/velocity/color 校验有限值，capacity 有 backend 上限；material 必须是可解析、binding-free color Material shader graph，texture binding/roughness 等不被接受。当前 `VfxGraphAsset.nodes` 没有 edge/pin 或稳定 node ID，支持“节点列表 lowering”并不等于支持 Niagara 参数映射、脚本、任意连线和模块图。`VfxRuntime::configure` 仅允许配置一次，真实参数修改还需受控 preview 替换/重建和旧实例回收，不能仅再次调用 configure。

## 向 Parent 的公共能力交接

本表是提案和依赖，尚未修改公共 owner，也未表示 Parent 已接收。

逐区域 flow、三尺寸 Dock 提案、实际 slots/props、原 control binding/event、参数/曲线/transport/保存前后条件及UE菜单/键位eligibility详见 [具体设计](particles-design.md)。Preview/Timeline 静态默认、NumberField 约束、中文诊断高度、实际命令/保存复用入口、u64保真和全局键位冲突分别记为 PFX-SHARED-07～11；looped、validator、reset/RNG、内嵌场景asset与独立override、emitter级启用/隔离差异记为 PFX-RUNTIME-01～05；粒子source codec缺口记为 PFX-DOCUMENT-01。它们仍待实现/验证。

| ID | 复用/需求 | 粒子域约束 |
| --- | --- | --- |
| PFX-SHARED-01 | catalog 中 `UiHostWindow` / `DocumentHost` / `MenuBar` / `StatusBar`、Dock/tab 生命周期 | 同一粒子 source document 关联 Overview/Details/Timeline/Curves；焦点与关闭返回需真实命令合同 |
| PFX-SHARED-02 | `WorkbenchPanelHeader`、`WorkbenchTreeRow`、`WorkbenchPropertyEditorRow`、`WorkbenchNumberField`、`WorkbenchDropdown` | 接口保留稳定 item key、source path、选中、禁用原因、commit/reset/undo；不是只提供显示字符串 |
| PFX-SHARED-03 | `WorkbenchPreviewViewport` / `WorkbenchTimelineStrip` / 原 `WorkbenchTransportControls` | 动画owner负责timeline/curve/transport，Parent统筹真实viewport与跨域集成；粒子提供实际clock、lifetime单位与simulation命令，已注册外壳不等于live preview |
| PFX-SHARED-04 | 共用graph/curve编辑surface | 材质owner牵头graph、动画owner负责curve；需要selection、pan/zoom、稳定身份、command和undo；粒子提供真实领域适配，不另建通用画布 |
| PFX-SHARED-05 | tokens / Starship / Penpot library instances | Parent 提供当前 library/component/version/token 身份；粒子域只组合和声明必要 variant；样式修改传播需真实设计和产品两端证据 |
| PFX-SHARED-06 | 桌面输入和捕获顺序 | 请求排队；当前无 thread 工具、未占桌面；实际 100%/150% DPI 以 GetDpiForWindow 与物理 client extent 为准 |

## 先后顺序与验收边界

1. 完成源依据、身份和能力盘点；排除/待定项保留原因。
2. 核实真实 Penpot 服务及持久化工作文件、library 权限和官方 export 入口；不能以 mock RPC 的 official frontend browser contract 代替真实保存。
3. 在真实 Penpot 中形成可编辑 system/emitter 设计，使用现有组件实例/共享样式/flow；保存、关闭原上下文、fresh reopen 并核验身份。
4. 将导出 delta 应用于上列原始 ZUI；检查稳定 ID、组件引用、binding 和 event，记录依赖闭包哈希。
5. 对真实 CPU 粒子子集接入 source session、参数 commit/reset/undo、preview play/pause/stop/rewind、真实 save/open；Graph 等尚缺后端的面板继续保持明确未完成。
6. Jenkins 托管最小插件检查和 Rust 聚焦测试；每个 compilation command 前声明 package/features/target/filter/profile/command，禁止全仓构建。
7. 正常 EXE 原生捕获，沿用 LayoutReviewCase 的 nonce/generation/source/process/媒体证据合同；形成官方 Penpot 图与产品同状态并排图、差异图和逐图复核。

必要矩阵：1280×800、900×620、640×520 均实际 100%；1280×800 实际 150%（1920×1200、144 DPI）。每种覆盖默认、空数据、选中、焦点、弹层及焦点返回、滚动首尾、中文长文本；另覆盖参数修改→实际效果变化→undo/redo→reset、play/pause/stop/rewind、保存关闭重开与错误反馈。布局和文本行框目标 ≤1 逻辑像素；没有有效资源/捕获不得 `passed`，没有双端对比及逐图复核不得 `accepted`。

Penpot file/page/board/component已按实际API读回记录在[具体设计](particles-design.md)；选定源码及主题基线指纹已记录。官方导出、实际产品依赖闭包/EXE fingerprint、捕获入口/nonce/generation仍待验证，不预填。一次真实样式变更→export→产品加载→新截图的证明也未完成。

机器可读的 [验收要求](particles-cases.json) 复用现有 `LayoutReviewCaseMessage` 字段及 native protocol 路径，明确标记为 pending requirements；它不是引擎发布的 case 或验收收据。矩阵展开为 120 项共通状态捕获要求和 104 项粒子交互捕获要求，实际捕获 0。`source-inspection.json` 保存本轮选定源码的 hashes 及 36 项只读结构检查结果；检查只涉及 schema/原始资产身份/协议路径/DPI 算术/review state/tab 清单，不代表插件运行、Rust 测试或产品通过。

同一报告根的 `design-inspection.json` 保留具体设计阶段54项结构检查、32份选定源码hash及一次检查器误要求未声明input flag的失败/修正；后续 `interaction-inspection.json` 记录加入UE菜单/快捷键和既有keymap冲突后的当前文档及源码检查。历史阶段metadata不充当最新产品状态；两者均没有运行Penpot、插件/Rust测试、服务恢复、桌面输入或产品窗口。

后续聚焦验证按实际修改边界选择，不能在共享服务未恢复时本地运行代替：

| 受影响边界 | 最小候选命令（尚未执行） | 覆盖理由 |
| --- | --- | --- |
| Particles Editor source codec、runtime转换、曲线及参数领域适配 | `cargo test --manifest-path zircon_plugins/Cargo.toml --package zircon_plugin_particles_editor --no-default-features --lib --profile test --jobs 1 --locked source_tests:: -- --test-threads 1`（待原owner资格化粒子配方后由正式wrapper执行，本行不是运行记录） | 单包、显式空features；静态定义15项直接测试和8项curves子测试，共23项，非Cargo发现数。沿原request/r2对账完整当前输入/授权/锁；不换身份、不自行重试。旧r2仅有9项源hash，不能背书当前源码，Cargo未运行 |
| 实际改动 runtime 模拟/资源生命周期时 | `tools/dev/dev-module-build.ps1 -Package zircon_plugin_particles_runtime -Action test -LibTests -TestFilter service -Jobs 1 -TestThreads 1` | 仅在代码受影响时覆盖 manager 的真实 play/pause/reset/removal；graphics 为现有必需依赖，不能假称无渲染依赖 |
| 接入/修改真实 VFX lowering 时 | `tools/dev/dev-module-build.ps1 -Package zircon_plugin_rendering_vfx_graph_runtime -Action test -LibTests -TestFilter vfx_graph_lowering -Jobs 1 -TestThreads 1` | 现有 lower/reject/emitter isolation 测试；真实 product-frame GPU 测试另按环境和改动单独声明，不把本 filter 当 GPU 完整验收 |

source codec范围已向用户声明并实际执行Jenkins入口预检，未提交或启动编译。其余两项仍只是候选范围，不创建构建目录。真正运行前仍需用户可见的模块/包/features/target/filter/profile/完整命令声明；Jenkins决定兼容构建池，物理位置必须在项目指定的 `E:/cargo-targets`。

## 本域实际源码转换实现

`zircon_plugins/particles/editor/src/source`新增`ParticleSystemSource`及严格serde/TOML codec，`src/lib.rs`仅作模块接入与公开导出。它读取既有`templates/cpu_sprite_system.toml`，保留runtime字段、源资源URI、独立显示名称及emitter/burst/color/size项的持久UUID键；`format_version=1`，旧模板缺少版本和键时兼容读取，未知字段/版本明确报错。seed写为完整u64十进制字符串，兼容旧整数，不经过浮点。

`validate`覆盖空作者资产、非有限数、反向范围、重复ID/键、重复或无序曲线时间及资源URI；capacity=0也不会跳过检查。`to_runtime_asset`构造现有`ParticleSystemAsset`，由既有资源owner回调提供ready material/texture handles；未ready则返回诊断，资源引用不会被悄悄丢弃。CPU便捷转换对Gpu请求返回错误。该层不建立新文档、history、clock或资源服务，`to_toml`成功只表示编码，尚无durable-save回执。

新增10项`source_tests::`测试源覆盖原模板往返、u64边界、中文重命名/重排后身份、重复键、非法值、空数据/未知格式、资源未就绪，以及通过既有`ParticlesManager`的参数→真实CPU模拟/暂停/停止；另验证旧模板首次key adoption需要持久化、删除后新增不复用历史键。`decode_toml`返回`ParticleSourceRead { source, assigned_keys }`，宿主据此处理迁移dirty/save；不能仅使用值读取接口就把新键当作磁盘已有数据。

测试尚未运行，不当作预览或引擎执行证据。实际`rustfmt --check`、限定`git diff --check`和9文件`zircon-change-guard patch`通过（0错误/0警告）；首次格式检查失败及修正已保留。最新静态补丁digest为`6617624067636c528672e5e8aebcab114dc24b21a98d4ce4eea5dd21ad705630`；初版8测试/旧digest和其入口拒绝仍作为独立阶段保留，不覆盖旧身份。

初版预检身份为`ue-particles-source-codec-20261024 / validate-source-codec-r1`。实际预检exit2、`cargo_option_unsupported: --manifest-path`，没有启动Cargo，未用根workspace或本地编译替代。接续Parent既有修复请求`ue-plugin-workspace-entry-20261024`，保留`source-codec-inspection.json`、`source-codec-jenkins-identity.json`和`source-codec-jenkins-preflight.json`。该轮随后由r2接续，旧identity不覆盖最新源码；Parent第二次接续明确要求沿原request/r2由原owner对账新hash、授权和锁，不自行另建operation。Parent进度、宿主合同及锁图消息ACK为`01a16d9e-1cfb-7030-9d2f-4a250f6d1f2e`、`01a16da1-47dd-7671-8e5f-1eb9506f4230`、`01a16da4-048a-75d0-869a-eb8fc1fdef59`，原ACK不等于已读。

只读核对还确认`zircon_plugins/Cargo.lock`中本包dependency entry尚未反映新增serde/toml/uuid，`--locked`运行前需受管对账。三个包均已存在：serde 1.0.228、toml 1.1.3+spec-1.1.0、uuid 1.23.4（本包manifest沿现有兼容版本要求1.23.1）。早期粒子单包提案只补三条现有依赖；Parent随后明确合并AI的toml边，当前以operator的combined-lock-resolution-plan-r2为唯一候选范围：同一锁文件、两个既有entry、共四条依赖边，保留其它字节与解析版本。实际锁文件、diff及生成回执仍未产生变化；不能据“没有新registry包”假称锁文件已就绪。

历史r2预检沿Parent的 `ue-plugin-workspace-entry-20261024.route-02.json` 核实，正式 `local-cargo.ps1` 当时选用capsule `baac70c9…` 内的pinned client `ede21bdd…`，解析层支持canonical插件manifest；初次只看checkout旧解析器而误判实际路由的记录已更正。已预先声明最小范围，由wrapper对同一request下的显式 `validate-source-codec-r2`（当时9源文件与6依赖hash、实际checkout、E盘输出）运行一次`-DryRun`。实际exit2、`Current session authorization is pending`，没有Cargo或Rust执行。此为历史路由/预检事实；当前`cdd14500…`只资格化材质配方和source closure，粒子尚不具备执行资格。r2身份/预检另存，r1历史不覆盖。向Parent的首条进度已入队，随后入口更正投递因官方队列已达100条上限失败；具体失败与待读取合同见particles-shared-contracts.md及r2报告。

### 同一源的曲线领域适配增量

新增 `src/source/curve.rs` 和 `src/tests/source_curve.rs`，原lib.rs与source/mod.rs仅接线；没有改Cargo依赖、公共curve/model、主题或运行时。`ParticleSourceCurveAdapter<'a>`短期借用宿主的可变ParticleSystemSource，按稳定emitter key定位，向现有 `zircon_editor::ui::curve::CurveModel` 提供size multiplier与RGBA五个Linear视图。CurveId由emitter key/channel组成，CurveKey使用持久source key，名称、time和排序索引不参与身份。

`ParticleSourceCurveEdit`包含curve/key身份、expected旧点和replacement。写入前检查选择、key存在、旧值、finite、t∈[0,1]、size非负及重复time；失败不写source。颜色四视图共享源key/t，改一个通道只改该分量值；跨key移动用原地排序保留身份，返回字段级inverse。adapter没有独立source副本、文档store/history/dirty/save/clock；宿主仍须在既有事务与retention准入后调用，并把逆操作交现有history。GPU多key的运行时首尾近似仍为开放差异，Linear源视图不证明GPU等价。

新增8项回归源覆盖共享RGBA时间与inverse、跨key后身份、错误/过期输入原子拒绝、真实模板/中文名投影、codec重开与runtime值、空曲线，以及真实CPU manager输出变化和恢复。CPU测试只读取真实sprite snapshot，尚未执行，不是GPU或正常EXE证据。逆操作在首次写入前取得所有权，后续排序不分配临时向量。该曲线阶段11文件source guard digest为 `fdbf620c0c8d798c61b04904288fe19819e7c50523f19e483462163043dd3b01`（0错误/0警告），格式检查通过；初次格式失败保留于 `source-curve-inspection.json`。当时提出的r3/source_计划未创建身份或提交；现依Parent第二次接续撤下该计划，保留原request/r2及source_tests入口对账。前轮新增代码/更正投递因100条队列上限失败，未称送达。

### Parent第二次接续后的参数合同与原验证入口

已实际读取Parent「粒子第二次接续」与 `particles-followup-02.json`。Parent确认读过前版设计、共享合同和r2预检；本域不等待分配、不重投满队列、不改原3ZUI/公共source/锁文件。640自动合并tab仍只是PFX-LAYOUT-01差异提案，没有获得默认重排批准。当前源变化在原request `ue-particles-source-codec-20261024` / operation `validate-source-codec-r2` 留待Jenkins owner对账，不将早期冻结identity改写成覆盖当前23项。

新增 `src/source/parameter.rs::ParticleSourceParameterEdit`，提供Seed、SpawnRate、Capacity和Lifetime四类字段修改。`apply_parameter_edit`先校验整个source及稳定emitter key/expected旧值，使用codec同一套范围校验；range两端一次提交，inverse身份在首次写入前取得所有权。容量保留完整u32范围，0值不绕过其它source校验，4096不被示例100截断。`parse_particle_seed(&str)`与serde共用严格ASCII十进制解析，完整保留2^53+1和u64最大值，非法/溢出文本在边界返回错误，没有浮点桥接。`is_noop`供宿主判断，不代替文档revision/retention/eligibility。

新增5项测试源覆盖严格seed、真实CPU发射率/容量输出变化和逆操作恢复、生命周期范围原子修改、过期/缺失/非法输入无写入及0容量/no-op。旧key-adoption测试补明：重复decode旧模板会产生另一组身份；to_toml后assigned_keys欠账仍在，编码重读只证明codec往返。宿主必须在同一live document保留首次source/assigned_keys，实际保存重开才记持久。已有8项曲线测试经 `tests/source.rs`子模块接入，曲线测试文件本轮未改；原 `source_tests::`现静态列出23项，执行0。

这些API没有注册全局命令、创建文档/history/preview store或返回保存/播放成功。Reset的目标值与来源、一次gesture准入和历史消费由现有宿主合同接续，不能把inverse本身当Reset按钮已接通。参数阶段源hash、初次格式失败/修正、静态检查及原身份保留见产物根 `source-parameter-inspection.json`；Jenkins、插件导出及正常EXE视觉门继续pending。

### 真实seek接续的新增数值证据

本轮只读核对动画owner的AN-PFX-G01～06后，复现了当前runtime rewind剩余时间不前进的binary32算术条件：1_000_000秒配1/60步长、1024秒配0.00001步长均通过现输入校验，却在扣减后保留相同remaining；另一个half-ULP样本先前进一步再停住。只执行最多3步的数值轨迹，没有调用可能挂住的真实manager/EXE。证据及真实源hash见 `shared-contract-inspection.json.runtimeSeekNumericProgressAudit`，PFX-RUNTIME-06保持open。

已沿 [共享合同的精确提案](particles-shared-contracts.md#pfx-runtime-06数值不前进与生产时间接口的精确提案) 交清runtime service/preview、clock DTO、单handle step、错误/测试及域preview job的具体候选路径。UE DesiredAge/GetAge和MultiTick_GameThread的逐步时间预算是实际参考。runtime/public与新preview目录尚无写权，本轮不实施；已存在source/parameter/curve Rust未再改动，23项回归仍未运行。原身份、原3ZUI、共享库和桌面边界保持，未重投满队列。

## 开放差异

| ID | 参考/状态 | 实际差异与证据 | 已知原因/依赖 | 状态 |
| --- | --- | --- | --- | --- |
| PFX-001 | 默认 System/Emitter，所有尺寸/DPI | 原始authoring/preview核心仍为Space；已有局部Penpot结构图，完整控件/UE样式和Editor图缺失 | 公共组件/样式、正式导出及业务宿主接入 | open |
| PFX-002 | Overview/stack/参数、选中和空数据 | runtime 固定字段结构，尚无 source editor document、module/script schema、revision/undo | 粒子业务 owner；Graph/curve 公共 surface 依赖 Parent | open |
| PFX-003 | live viewport、参数/transport 改动 | runtime API 存在，Editor 没有实例/clock/render consumer；无截图和运行结果 | PreviewWorld、renderer、document adapter | open |
| PFX-004 | open/save/reset/undo/快捷键/上下文跳转 | operation descriptor 没有执行器；不可验证真实保存/撤销 | source document、typed handlers、durable asset store | open |
| PFX-005 | Niagara graph、scratch pad、generated code、sim cache/baker/scalability | 五类节点 VFX runtime 已具备真实 lowering/executor，但 Editor 未接入；任意脚本、edge/pin、模块图和缓存职责仍缺 | 待分项能力/设计决策，见上表；不预先删去用户节点编辑目标 | open |
| PFX-006 | editable Penpot 与样式传播 | 本域revn30；原18板/业务主件及7实例保留，旧验证页4外链及新增Stack页7外链已保存/fresh重开；45 refs/主件闭包已核，styles仍0、4个value槽空、Tree选中/焦点与长文本仍open。原生PNG /api/export实际404，官方ZUI与完整设计未通过 | 公共owner的typed props/slots/events/export、共享样式及专用组件；局部消费不等于完整UE设计 | open-partial-native-design |
| PFX-007 | Jenkins 聚焦测试与原生产品验证 | 恢复r2已由Parent核实；本域14:39Z原身份QueryOnly exit2/authorization pending、无新提交或执行job。当前d2873胶囊11文件匹配，client/server及粒子metadata route已交付；25项静态定义、执行0 | 原owner自动收件对账、精确粒子配方/完整闭包及资源准入、合并四边受管锁生成；不以material/AI recipe或root -p替代 | open |
| PFX-008 | 同尺寸/DPI 双端逐图复核 | 本轮已有Penpot草案图及中文/长文本设计观察；无有效正常EXE截图、实际100%/150%或双端比较，未测≤1逻辑像素 | 桌面排队、公共字体/设计、产品实现及原生证据链 | open |
| PFX-DOCUMENT-01 | TOML作者源及真实参数转换 | DTO/codec/稳定UUID及runtime lowering已有实现和静态检查；未有编译执行、DocumentHost/factory/history/dirty-save/preview消费者 | 域内adapter接既有公共宿主合同与受管测试；原3 ZUI仍未导出修改 | implemented-codec-consumers-pending |
| PFX-SHARED-12 | TreeRow源text_overflow=ellipsis；中文/长标题 | linked copy的128×16文字框把“发射器／sprite_emitter”渲染成两行；60%整板图、100%诊断原图及SVG两条text run均保留 | 原因未证实；Parent/public owner核对既有painter、typed text覆盖/reset与正式导出；未改共享源、detach或缩短fixture | open-proposal |

目前交付为源码映射/交互合同、真实Penpot局部可编辑结构和待受管测试的源codec实现；窗口族完成度不得据此升级。实际清理释放0字节，产物清单保持active。

## Shared rev1四主件的真实消费验证

接续Parent的13:25:59Z新观测，先只读核对四主件及其源ZUI v2路径、control/sourceNodeId与PropertyEditorRow的value槽。真实Penpot UI连接`00 Shared Components`的link-file-to-library于13:41:24Z返回HTTP200，直接库记录revn1/vern0、isIndirect=false。随后只在粒子原file新增page `92a91f8e-23b8-8096-8008-d6cd96ee165a`、board `92a91f8e-23b8-8096-8008-d6cdb4e2219e`并拖入4个原生copy。没有复制main、detach或修改Shared/其它族文件。完整身份、源hash/slot、真实操作、失败及两图复核见 [设计记录](particles-design.md#shared-rev1原生消费者身份与逐图复核) 和报告根 `penpot-shared-components-consumption.json`。

最终rev19的四根/文字引用与覆盖在13:58:10Z及13:58:15Z两个fresh context中保持摘要`41b4f85e004d62d8d286aa943c9f6e21475456613efd267139fddba0c72f9c1a`；原页266对象逐项摘要及私有业务组件摘要未变。原生Reset overrides只验证Penpot实例复位，粒子source reset/undo/模拟未执行。自有headless runner因有界总时限退出，context/browser已关闭，进程轮询exit0；共享桌面未使用。

PFX-SHARED-12长文本差异和PropertyEditorRow空value必需槽继续open。尚无已发布Field/NumberField可嵌入；没有把空槽、搜索外观或按钮文字当成功能。仍需公共owner正式source/slots/export、共享样式与全局A/B，以及原3 ZUI导出和文档/生产preview接线。没有ready EXE/资源闭包，未请求桌面时段。本轮Cargo、托管Rust、插件导出和原生产品捕获均0。

## Jenkins恢复r2后的原身份接续

> 以下保留该阶段历史观察；resolver实现和JM server准备已由本轮原owner交付。当前生成/资源状态见本文末尾新增记录，旧空值不作为当前未实现的结论。

已读Parent「Jenkins最新恢复结果」及`parent-01a16d4e/jenkins-recovery-handoff-02.json`：原恢复owner `01a12af6-7511-70d1-a452-61b80678eb56`沿原request/attempt/runtime完成root0/active0、双EOF/readersJoined/childrenGone/jobClosedAfterZero，Parent核对197不可变文件及固定工具。此为恢复范围，不回填粒子产品Rust、原P21/Runtime11或画面验收。

本域实际安装读取已晚于Parent的20b3快照，当前为`9c684aef96b8b1c94319aa8619207f8b07aefffa71d571d4f58708d2c3408233`；manifest和5 server/6 client文件均独立匹配，installed SHA `c41a585f6cea0746fd9cc147a9579079e9f8b65a8139d0f81b1225f67cffdb8d`，同driver/runtime。14:39:34～37Z沿原session/request `ue-particles-source-codec-20261024` / operation `validate-source-codec-r2` / 原identity文件，通过正式`submit-jenkins-management.ps1 -Action validate ... -QueryOnly`做一次只读查询。原wrapper exit2、pending/ordinary_entry_unproven、message为`Current session authorization is pending`，3.594秒仅客户端查询时间；外层PowerShell管道exit1另记，不能混成wrapper exit。没有新提交/server receipt/执行job或Cargo，也未换ID、改身份/授权或写数据库。installed跨查询保持同一字节。

9c684新增AI精确lib筛选，但粒子仍不在client `entry.py:472`、server `runner.py:4054`和metadata成员表`runner.py:720`内。`generate`仍仅source-index/render-template，没有受支持的lock-resolution配方。当前插件锁SHA仍`04de41d28ef79436b8bc4c264d8fb122d303ae2a7005085522783afdeae02bbf`，AI缺toml、粒子缺serde/toml/uuid；合并计划未有actual diff/after SHA/生成receipt。本域不会借AI/material配方、root -p、手拼锁或去掉--locked。

冻结r2九项source hash仍有5项不同，六项dependency hash保持；当前16个本地Rust模块及4个唯一include输入候选SHA `5b524bffaf7d29f2be59be655154be0f1493f8ba93db0c7fb67589e252e8c831`未改，不冒充完整传递闭包或冻结执行身份。继续由原owner核对自动收件、当前完整source/features/resource、writer及物理E根准入，再沿原身份交真实粒子单包/lib/test/defaults-off/features=[]/source_tests::/jobs1/thread1/locked验证。没有captureReady或已测捕获时长，不申请无限稳定写入窗口。具体源码落点、原始查询输出及交接见报告根`jenkins-r2-recovery-continuation.json`和共享合同末节。本轮产品源码、锁、Penpot、服务和桌面操作均0；passed/accepted=false。

### 原合并锁计划的四项实际缺口

已独立读取并核对原owner的implementation-r3交付：summary SHA `f2c5fb3fede306a5d1cbf7905455096fcb42dd9afa8edffeb0c6b8667d98ae78`、result SHA `560402c62522790d61bcb42f7f59b529e86bcd5df537449249af66f21245e82f`、generation-gate-observation SHA `fae88bfcea2050dbdbcf7981f1bdda21d10613a2dc4725ff2f98796c1bb74884`及#406 outer native SHA `85d6af0d84532d513562ef9ab8c90f161bf968daccd1ce672c013bcf4e5fb757`全部匹配。outer原件为root0、afterTerminal active0、双EOF/readersFinished/nativeWaiterJoined；该helper终态不代替host生命周期或粒子Rust。#406的81项管理检查/267子检查、1历史fixture排除，以及#405原失败、六成功/两失败archive一致和管理分支待定时集成，均保留其范围。

当前9c684的AI四个精确filter及26项静态workspace输入合同已由原owner交付，粒子recipe/完整metadata闭包仍未交付。原combined-lock-resolution-plan-r2 SHA `1f8593fa…`、插件锁SHA `04de41d…`和root锁/两域manifest/插件workspace manifest五项实际当前hash均保持；四边仍缺，生成diff/after SHA/生成及恢复receipt仍null。完整细项见 [四门交接](particles-shared-contracts.md#原combined-lock-resolution-plan-r2的四门实据)。这不是旧r2计划成功，也不改变未知create收件或原粒子QueryOnly的状态。

四项缺口分别为：Management.generate缺真实定向resolver＋完整候选diff＋before CAS/恢复事务；原计划没有server-issued generation phase/source-preparation回执；当前能力修复writer的9路径不含插件锁，单锁writer未准入；resource policy v11观察reserve35 GiB、E余量18320347136 B（17.06 GiB），且无resolver专用可信预算/实际reservation。该余量为14:34Z观察，不能视为当前可用准入或resolver实际耗用；不借其它预算、扩大writer或清共享缓存。

本域将上述事实合并至既有`jenkins-r2-recovery-continuation.json::combinedPlanOperatorFollowups`，未新建或改写原计划/身份，也未重发QueryOnly、test或generation。继续等待原authority实现并审查真实事务和server/source/writer/resource phase；届时由源owner提供真实ready窗口的候选，不无限停写或把当前静态清单当freeze。原owner本轮实际清理24576 B单独归其receipt；本域释放0 B，产品源码/锁/Penpot/桌面均无改动，Rust与窗口族验收仍pending。


## Parent第三次接续：域输入预检与真实Stack组合

已读Parent「粒子第三次接续：PFX-SHARED-11 的输入与保存边界」及`particles-followup-03.json`。System kind统一`particles.system`，未来有独立后端才注册`particles.emitter`；当前authoring未安装此kind，不凭标题/活动tab声称已获取文档焦点。真实focused ViewInstance、DocumentId、session和revision由Parent公共host接续，辅助pane保持原文档。域参数预检没有伪造这些身份或第二套history。

在既有`src/source/parameter.rs`内新增`validate_parameter_edit(&ParticleSourceParameterEdit)`，只读检查整个当前源、稳定emitter key、expected旧值及codec同一范围规则。`apply_parameter_edit`复用该检查并在写入前重查live source，inverse仍在首次写入前取得。`ParticleSourceParameterEdit::seed_from_decimal(expected, replacement)`把两端字符串分别交既有严格u64解析器，不经过JS Number、浮点或serde旧整数兼容。预检成功不等于host的revision/input/retention许可，不是12 operations的执行器安装。

新增两项有界回归源：只读预检不改源、间隔编辑后旧delta拒绝且不写；两端seed严格解析与2^53+1/u64::MAX经真实source修改和inverse保持精度。`source_tests::`现为17直接定义＋8曲线定义＝25，Cargo发现数未知、执行0。对本轮两文件运行直接rustfmt（edition2021），初次格式差异及修正后check exit0保留于`source-parameter-inspection.json::parameterPreflightStage`；没有Cargo、托管测试或新请求。旧`source-closure-candidate.json`及原r2 identity保留历史字节，本轮parameter/tests新hash列为delta，不能继续以旧候选或9项hash背书当前完整闭包。

同一Penpot文件新增page`f2894894-da30-8095-8008-d6e405e95ace` / board`f2894894-da30-8095-8008-d6e431a06285`，1280×800。它是有边界的Stack组合设计，包含系统/发射器结构、按实际CPU固定职责分组的字段、真实模板只读值和曲线单位；不是默认Toolkit完整窗口或可运行预览。普通frame/text为原生流式布局，值来自现有`templates/cpu_sprite_system.toml`；7个控件均通过真实Shared库卡片原生拖入，中文覆盖使用原生图层文字编辑。控件source/master/child refs、保存HTTP200、关闭和fresh重开及逐图记录见[最新设计身份](particles-design.md#system-stack组合页的真实flow与外链实例)。

四个PropertyEditorRow的必需value槽仍空，右侧只读模板值不充当输入控件；Tree仍是源default的selected=false/focused=false，不能将caption或系统状态当选中/焦点通过。Reset Simulation仅设计意图，产品clock/reset/preview消费者尚未安装。普通frame/text仅投影当前editor_tokens数值，colors/typographies仍未发布，未另造主题或改Shared。官方1x PNG导出一次实际POST`http://127.0.0.1:19001/api/export`返回404，30秒未收到download；无PNG输出、无重试或服务操作。100%工作区图保留，不能替代官方PNG、ZUI导出或EXE截图。具体公共接续见[输入、保存与导出边界](particles-shared-contracts.md#parent第三次接续的具体消费边界)。


## 粒子托管入口d2873已交付与原lineage修订登记

> 以下保留该阶段历史观察；resolver实现和JM server准备已由本轮原owner交付。当前生成/资源状态见本文末尾新增记录，旧空值不作为当前未实现的结论。

原Jenkins owner于本轮交付canonical `zircon_plugins/Cargo.toml` 的粒子单包入口。独立核对指定`particles-editor-extension/result.json` SHA `37e311e36ef55e7500af48ed3e9ae23f1b8bd411a0536b3719522729566779a1`与`summary.md` SHA `40413f4b040ead2c37d88a4d35c8f43de38979a8d420967922021fad9057a44a`，当前installed确为`d2873ff353ea93d6b76f7d7caa12878761146e326070fd4c4526b73f3dc23112`，5server＋6client文件实际hash均匹配。entry `82d965a2…`、runner `93067f14…`，driver/runtime仍为原fc553228…/start-1792836412672596900；9c684/cdd的“无粒子配方”是历史，不能继续作当前结论。本轮只读本地部署/回执，未重启、重新安装或运行QueryOnly。

当前bounded范围为`zircon_plugin_particles_editor`单包/lib/test profile/defaults off/features=[]/`source_tests::`/jobs1/threads1/locked。client entry:472起、runner:4054起及metadata catalog:720起已接入particles/editor；静态metadata回执为2工作区、17包manifest、31条path边，保留Runtime native-host和经Editor引入的target-editor-host。它不等于Cargo实际解析或完整9-root输入capture。#410的94管理tests＋305subtests/JUnit399通过，排除1个已记录baseline；#411 queue14583复用同源检查后安装成功。8份native local/Home实际hash与owner记录的认证HTTP hash一致、terminal/active0/双EOF/readers完成；只证明管理helper生命周期，产品Rust执行仍0。#408/#409及#410原失败保留。管理PR51已合并的远端事实按owner已核实回执引用，本地main仍awaiting-main-sync，本域不另pull/switch或覆盖共享改动。

已审阅owner指出的候选六项变化：四份本域文档记录实际Penpot shared/Stack、输入保存合同和未完成验收；`source/parameter.rs`与`tests/source.rs`为上一阶段登记的只读预检/严格seed工厂与2项回归。16路径在本轮更新文档前均匹配本域15:43阶段指纹，也匹配owner16:36观察；两Rust hash仍为`eb5df5ef…`、`eb520514…`。没有新Rust/manifest/原3ZUI/公共source变化。现已在既有`jenkins-r2-recovery-continuation.json::sourceLineageRevisions`追加本地审阅修订，透明列旧候选→owner观察→本轮当前hash、具体理由、16个本包模块/6个include边/4个唯一输入和原六dependency指纹。旧`source-closure-candidate.json` SHA5b524bff…及r2冻结输入SHA1e5aa440…原字节保留；这不是替换原identity、新attempt/operation或服务端已受理的修订。

原lineage仍为本会话session / `ue-particles-source-codec-20261024` / `validate-source-codec-r2`。server revision/source preparation/resource/cache/pool回执尚空，原未知送达不改称“从未收件”，不反复QueryOnly、不重投旧输入或手动grant。captureReady=false、预计耗时未知、未要求freeze。combined-lock-r2仍缺AI→toml与PFX→serde/toml/uuid四边；installed generate只有source-index/render-template，缺受管定向resolver/CAS/完整diff/失败恢复与sole canonical writer及资源准入。owner16:36的E空闲16145526784B/15.04GiB低于35GiB reserve是OS观察，不能当资源拒绝回执或自动放宽。无Cargo/锁生成/缓存清理，当前25项定义执行0，passed/accepted=false。

## Parent第四次接续：实例实际时钟与单步的五路径增量

Parent已正式确认原Penpot rev30、3页逐页388对象（跨页唯一ID386）、18 copy=7业务+11 Shared及106个有效source refs；4+7 Shared消费、Stack、Editor 25项候选源码及d2873管理配方已收件。四个typed value槽仍空、Tree长caption两行、生产viewport/timeline/input缺失；动画和材质专用main仍非已发布通用库。当前文件不回退、不另造共享控件或主题，此次不操作Penpot/plugin host/桌面。

本轮Parent明确授权的Runtime源码仅为 `zircon_plugins/particles/runtime/src/service.rs`、`service/preview.rs`、`lib.rs`、`tests/service.rs`、`tests/service/preview.rs`。已先在同任务artifacts登记三个现有文件before hash、两个新文件缺席、原50个Runtime src输入及依赖/原r2影响，再实现如下最小公共边界：

- `ParticlesManager::preview_clock(handle)`：同一manager锁内只读当前实例的handle、age_seconds、playing、time_scale、backend、fallback_to_cpu；不创建文档/全局clock或frame-ready状态，不使缓存失效。
- `step_preview(handle, dt)`：先校验handle和严格GPU边界，再校验finite、dt > Real::EPSILON、乘一次time_scale得到的scaled_dt有限且正、actual age + scaled_dt有限且严格前进；全部通过后才临时play所选实例、调用一次该实例tick、恢复原playing、同锁读回actual age并invalidate snapshot。其它handle的粒子/age/RNG/playing不变。
- 新 `ParticlePreviewError` 在preview子模块内；不扩展 `ParticleSimulationError`。GPU且fallback=false明确拒绝step；backend=Gpu/fallback=true按实际CPU模拟读回，不能据此宣称GPU执行或媒体帧就绪。

10个 `particle_preview_` 候选回归从 `service.rs → tests/service.rs → tests/service/preview.rs` 接线，覆盖真实两handle隔离、暂停/playing恢复、倍率2与0.5、raw dt的0/负值/微小/NaN/无穷拒绝、0倍率/乘法下溢及溢出、age不能前进/溢出、未知和已删handle、GPU fallback显式标识及非fallback拒绝。小于EPSILON的scaled_dt只要实际age可前进仍允许，未加无依据阈值。拒绝回归比较完整实例（含粒子/RNG）和缓存/诊断状态；暂停成功回归与既有单次manager.tick产生的独立实例逐项比较。

[Runtime source packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/runtime-preview-clock-step.json)包含精确5文件、全部52个Runtime rs源/51条literal模块边/13条include输入、before/after及依赖指纹、fmt与静态guard实际回执。测试仅有10个静态定义，Cargo发现未知、执行0；与Editor原25项 `source_tests::` 完全分开。未来最小范围为 `zircon_plugin_particles_runtime` 单包/lib/test/defaults off/features=[]/jobs1/threads1/locked/`particle_preview_`，仍需原Jenkins owner单独支持Runtime配方及真实source/resource/cache/lock/pool准入；Runtime graphics+native-host、SDK runtime/declaration、wgpu29/naga29 WGSL依赖不得省略。当前d2873仅证实Editor入口，未拿root -p或其配方替代。

原r2/request/operation、旧candidate原字节保留；新Runtime源使旧候选的传递Runtime闭包不再代表当前checkout，已在原lineage另记本地增量供owner审阅，server revision/测试/生成回执仍空，未重投或编造attempt。旧 `rewind_preview` 的f32 remaining无界循环保持原字节，继续禁作生产seek；此次没有UI接线、新scheduler/preview store或全局步数阈值。后续seek仍须既有jobs的total/slice预算、整数步计划、取消/新请求/关闭gate及actual进度。家族passed/accepted=false。

## 受管resolver已实现：真实准备完成与资源拒绝

本轮复用原Jenkins owner的 `combined-ai-particles-lock-resolution-r2` 交付，已核实summary SHA `6a091cc1…`、result SHA `7d7d9738…`及原server preparation `fdc33cc1…`。原 `jm24 / prepare-lock-resolution-r2` 同输入经#360 SUCCESS建立真实generationRequestId `lockgen-3c23f3791938bf820ae9a2bb8a8734c9718e461fa67311cbf27e00fe`。prepare/admit/resolve/publish/recover受管事务已实现；#416的110 tests+317 subtests/427 checks仅为管理代码验证。此前“generate只有source-index/render-template、resolver未实现、没有server preparation”的记录保留为历史，不再代表当前交付状态。

解析准备列出157份manifest/1075项解析输入，inputMetadataReady=true、captureReady=false；这不是产品完整源码捕获。实际native cargo --version只证明受信Cargo 1.94.1身份及该命令原生终止，不证明resolver、cargo update或Rust执行。原owner要求的generation专用受信stable>=1.97独立锁输出能力仍待就绪；不升级现工程工具链、不启用nightly/bootstrap或借其它树绕行。

原 `inspect-lock-resource-r2` 的#361 SUCCESS是完成了真实检查，其payload为admitted=false、reservation=null、singleLockWriter=null。policy v11保留35GiB reserve；该回执的E可用15137222656B（约14.10GiB），不是本域当前磁盘测量。单独resolver预算1CPU/2GiB RAM/256MiB disk是估计，不是已测峰值或编译预算。#417仅撤销本次检查等待行，保留blocked generation准备，不代表已预留资源或撤销整个generation。

实际resolver/update/生成候选diff/after SHA/发布/真实生成恢复仍未发生，原04de41d插件锁及报告所列保护输入均未变，AI→toml与PFX→serde/toml/uuid四边仍缺。后续由原owner取得受信工具能力、当前解析输入修订、实际resource/pool/sole-lock writer，再做完整候选审阅/CAS/恢复；不重放旧AI/粒子业务payload、不冻结写者、不降低阈值或清共享缓存。server generation准备属于JM原请求，不是粒子原r2的source revision、Rust执行或Runtime测试准入。

原owner另在e0635d5d…选择下修复receiver单pointer并保留旧sealed改写及API/Home差异证据；本域只核对其固定capsule的11项pin，未重新查询selector、发布入口或改变full-source policy。外部verify_declared_inputs策略owner/发布验收、运行host lifecycle及local main sync继续独立open。15份管理helper本地native回执已复读hash；三方归档一致复用原owner证据，不冒称本域重新查询HTTP/Home或产品截图。

本轮只更新四份本域文档及原lineage，Rust源码未改。Editor 25项 `source_tests::` 与Runtime 10项 `particle_preview_` 各执行0；[原续接记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/jenkins-r2-recovery-continuation.json)保留新旧门槛与所有失败。[Runtime source packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/runtime-preview-clock-step.json)原SHA `0b47e9c6…`保持。Penpot/typed输入/正式ZUI/正常EXE及150%DPI双端验收均未由此推进，passed/accepted=false；本轮释放0B。

## Shared rev2发布后的实际接续

已读Parent `settings-plugins-followup-05.json::publishedSharedStyles`：Shared file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` 当前发布rev2，四个component/main UUID保持，34 colors/3 typographies及21处master引用由Parent实际认证读回。本域读取原报告并核对SHA，没有在本轮重新获得get-file。body/caption/strong为原公共style身份，后续通过原native linked instances更新；不detach、不造主题或复制main。历史本域rev1同步与styles0不能描述当前已发布库。

[本轮接续回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-shared-rev2-sync.json)登记原11个Shared copy/root/component/main与本地中文覆盖、两页board身份和源证据。实际对19001/readyz、16060/readyz、19002/manifest.json各一次检查均为WinError10061连接拒绝；无HTTP响应、API读取、浏览器、库更新或保存。本域不取得服务生命周期/桌面权，未把连接失败推断成数据库或设计丢失，也未覆盖旧成功读回或重投满队列。

公共catalog当前SHA `4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5`；10个NumberField prop默认与源逐项一致，4个PropertyEditorRow slot的kind/required/accepts与源一致，连同68组件和发布身份共24项静态检查通过。Seed严格十进制u64；容量完整u32同样不可经f32丢精度；发射率和寿命按源schema传值，模板42/0..100/precision0不成为真实域限制。四个typed value槽仍未填，新的Field/Number Penpot主件尚未发布，产品handler/输入及资源准入仍需原owner接续。

下一步在原owner核对服务身份并恢复readiness后，打开同一原文件执行native library update，逐项保留原copy/shapeRef/中文/touched，确认真实fill/stroke/typography style refs及library syncedAt，保存关闭后fresh重开复读。特别检查现有content-group/text-content-text覆盖是否阻止文字样式继承；没有实测前不称继承或override-reset通过。随后单独验证CJK行框、正式export、真实产品样式A/B及实际150%DPI。本轮仅四份文档和既有证据lineage变化，Editor25与Runtime10各执行0，释放0B。

## 空端口消费者接续：640、源投影与真实样式消费

已实读材质 `pfx-empty-ports-contract-08.json`（SHA `2a950ff2d42eaab7173eefe54a375aee2ddb500727ed5d0e1426e833453ce53b`）及共享合同新增段；UE只读 `EdGraphSchema_NiagaraSystemOverview.h:19–20`明确DISALLOW/false。接受真实节点且inputs/outputs/edges=[]为ready无连接模型，不代表compile/preview成功；原空FText TODO不复制成Zircon的空白原因。

本域在既有Editor `source/**`、root/module和`tests/source.rs`范围增加 `source/overview.rs`，复用实际 `GraphNodeView/GraphEdgeView/GraphNodeBounds`。System身份在原DocumentId内为System枚举；Emitter使用既有assigned key，重命名/重排不改身份。布局只接受宿主实际测量的graph-local bounds；缺key/重复key、缺布局及非有限/非正/溢出/不可前进边界均返回错误，不生成假位置或成功空图。空emitter源只显示真实System，不造Emitter。只读投影不修改source、不实施GraphModel mutation/permit、编译、preview或另建layout store；实际retained/input/源代次绑定仍归宿主接线。

新增4个 `particle_overview_` 回归源覆盖真实模板→共享presenter/0诊断/伪造pin MissingPin、空System、稳定身份及测量布局、非法源与布局拒绝；本包当前29项静态定义（21 direct＋8 curve），执行0。Runtime原10项 `particle_preview_` 完全分包分filter且源码未改。[source packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-overview-projection.json)记录精确4写路径、17个Editor源、before/after、既有共享依赖及fmt；原r2/input/candidate/lock原字节保留，新scope只追加本地lineage，不当server revision或Rust准入。

640 selected板 `24e30f02-3e16-4926-bd5a-a8f0eacf1da7` 的原叶 `1848b40d-63ca-45f5-ba39-ae1500755dca` 已通过原生文字编辑器改为“设计夹具；连线不可用：此资源未定义端口或连线。实际Graph/焦点待接入。”，保留原flow/ID，没有批准或安装640自动合并tab。原Stack note同步纠正过期“样式未发布”。11原linked文字通过原生Assets应用现有body/strong，中文和源引用保持；随后原生Update推进真实library syncedAt，并再次保存关闭fresh重开核对。没有写Shared/Material、复制main、detach或新增主题；仍无Graph公共main及Field/Number主件。

本轮独立JSON API读Shared raw SHA仍为 `a446a0bf8e1081f964037146e0086470daaa9835487999c0c27166ee55e26006`，与Parent17:49–17:59原响应相同：34 colors/3 typographies确已发布；21是nullable typographyRefId字段数，其值全null、非空master引用为0。实例现有真实body/strong引用是本地覆盖，不倒推master已绑定。最初Updates面板无待更新；本域先使用真实样式后才出现update banner，Update后的syncedAt有实际API回执。11实例/106 source refs、cap文字、文件/库rev和保存fresh相等记录在 [原生消费者实证](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/graph-consumer-native-trial.json)。初次style读取过早、严格selector匹配失败及其修正均保留，未把点击或入队当保存。

五类VFX源是另一插件的真实 `VfxGraphAsset/VfxGraphNode`：SpawnRate、Lifetime、Velocity、ColorOverLife、ShaderGraphMaterial；没有持久node key/pin/edge/layout，也没有Serde codec。当前PFX Editor未直接依赖该runtime，不复制enum、捏造TOML源或借Material模型。具体交接建议是在已直接依赖runtime的 `zircon_plugins/rendering/features/vfx_graph/editor` 做typed只读投影，Parent明确新source/test路径和独立包入口后接续；本轮未跨写。另一方案向PFX Editor加runtime边会扩大既有combined4边lock合同，未擅自加入。当前node_index只能是精确source/revision内的临时定位，重排即失效，不作持久作者key。

实际 [Stack原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-stack-shared-rev2-synced.png) 与 [640 fresh原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-graph-consumer-640-synced.png)为100%画布、deviceScale1的完整Penpot workspace，仅证明此轮设计可见；不替代正式PNG/ZUI导出、引擎窗口、CJK/≤1逻辑像素或实际150%DPI。空typed槽、Tree长字、独立命中/选中、键盘/菜单/程序入口禁用原因、旧gesture取消/无delta undo及真实焦点返回都继续pending。一次读取脚本漏根自环导致自有headless helper挂起，已核对PID/出生身份后终止其9进程树，原exit1/0保存均保留；重试采用visited-set，未操纵共享服务/桌面。

## 动画 r03 消费边界与源指纹对账

已读取动画 [r03 合同](animation-shared-controls.md#r03粒子第四次接续与实际adapter)、[原交接](animation-parent-handoff.md#粒子第四次共享消费接续r03)及两份机器提案。该 owner 记录的较晚保存重开为 rev140、源映射回执为 rev105，三个专责 main 仍 isShared=false；Parent 17:03:59Z 的 rev100/1358 是历史观察。本域最新已验证为原 file rev38，11 个 Shared 基础实例没有引用三动画 main；本轮不调用 Penpot，不从它们推导专责控件发布或消费。

现有 `ParticleSourceCurveAdapter` 借用同一源；System playhead 只能读同 handle 的实际 simulation 秒，normalized lifetime t∈[0,1] 独立，多 birth/lifetime 不存在唯一同步 cursor。RGBA 横移按 emitter/source key 归并一次域 mutation 后重投影四通道；分量 value 只写该分量，Linear、无 tangent，有限 HDR 不夹 0..1。单 key delta/inverse 不等于 multi-key atomic batch；gesture/history/retention 仍需真实宿主包装。

Runtime 五路径已存在，指纹仍匹配 Parent。`preview_clock`/`step_preview` 的实际六字段和 raw/scaled/age 前进预检保持；数值/backend 预检拒绝发生在修改前，但 `Simulation` 错误可能已有 tick 副作用，不能从 Err 推导 age/state 回滚。外层只在原 context 仍有效时重新读 clock；请求、source/install/cancel/frame 不塞入 Clock，也不把一次 tick 视为总量/slice 预算或 GPU/frame-ready。旧 f32 remaining rewind 仍禁作生产 seek。

动画 r03 的 20 份源指纹当前为 18 匹配：`tests/source.rs` 已在本域 stage5 完成借用范围修正（d315646e…→07a552c0…）；公共 catalog 现为 e4bba09c…，不同于 owner 当时的 4c2f849c…。后者只读保留、不归责写者或回退。已对当前 68 组件、NumberField 10 props、PropertyEditorRow 4 slots、PanelHeader.actions 等受影响合同完成 19 项 catalog 结构核对，另2项核对 Runtime 与既有 source packet；旧整份 catalog 的 24 项回执不覆盖新哈希。源差异及原输入均保留于 [r03 消费对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/animation-consumer-r03-reconciliation.json)。

原三 main、六条 animation route、public source/codec/slot、三份粒子 ZUI 及 Rust 均未修改。Stop 仍只有 descriptor，ResetSimulation 无已交付 handler；真实当前文档的标准 Button 组合、eligibility/ACK/focus 和 canonical 发布归 Parent/原 pipeline 协调。Editor 当前 21＋8＝29 定义与 Runtime10 分包、执行0；原 r2/input/candidate/锁、资源与 EXE/DPI 门保持，本轮只追加四文档和本地 lineage，不产生 server revision 或新测试提交。

## VFX 五类只读投影：已交付源码与未完成产品门

本节是 Parent 第五次接续之后的当前状态，前文“待 VFX leaf 授权/交接”保留为当时记录。已在原 `zircon_plugin_rendering_vfx_graph_editor` 完成获授三文件：[crate root](../../../zircon_plugins/rendering/features/vfx_graph/editor/src/lib.rs)、[只读投影](../../../zircon_plugins/rendering/features/vfx_graph/editor/src/overview.rs)及[回归候选](../../../zircon_plugins/rendering/features/vfx_graph/editor/src/tests/overview.rs)。既有 Cargo 已直接依赖真实 VFX runtime 和 Editor；没有新增依赖、改锁或向 PFX Editor 复制五类 enum、Material 模型、codec、schema、camera 或 clock。

投影直接借用实际 `VfxGraphAsset/VfxGraphNode`，五类 payload、空源、重复类型与顺序均保留。布局只取当前 host 测得的 graph-local bounds；source/revision 不符、缺失/多余索引、非法或不可前进边界明确拒绝。`node_index` 只在同一源/revision 内临时定位；`locate/resolve` 检查真实 source 和 revision，重排/换源后不能沿用旧 locator，更不能将其保存为作者 key。该接口没有安装 host、GraphModel、命令、生产 preview 或 frame。

[原三文件 source packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/vfx-source-overview-projection.json) SHA `1390a50d3bba719b33c1cb794c0a17716f96bd03d4379c0f7b34532e3a49fde7` 保持不变；5 个 `overview_tests::` 只是静态定义，Cargo 发现及执行均为 0，与 PFX Editor29、Runtime10 分包分过滤器。原 fmt/精确三路径 guard `5fac260d…` 和 74 项结构检查只证明其静态范围，不能提升为产品通过；原 r2 identity/candidate/journal 及 Runtime/Editor source packets 仍保留原字节，本次四文档更新不作为 server revision 或新测试提交。

已消费材质两文档勘误及 [消费回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/material-erratum-vfx-consumer-reconciliation.json)：原 Shared JSON 当时即 rev2/41 对象/4 main/34 colors/3 typographies；consumer own styles0 不代表库0。master 有效 style refs 仍0，实例本地覆盖、发布、继承/同步及 A/B 分开。真实源无 ports/edges 时没有连线交互；单独 `edges=[]` 不禁用其它具有合法端口的图，真实 required-input 诊断保留。GraphCanvasState 不拥有 source-bound link gesture，取消旧拖线且无 delta/undo 仍待真实宿主接线。

当前验收清单保留三项开放行为：五类源在真实宿主中的只读选择/Details/布局与源切换；所有连线入口共享同一不可用原因及实际焦点返回；原 `graph-edit` 的真实 typed edit/compile/undo/preview。只读投影没有替代第三项。正式 VFX leaf 配方、source/resource/pool/cache/lock 准入与实际 Rust 交原 Jenkins owner；官方 Penpot/ZUI 导出、正常 EXE、三尺寸/实际150%DPI及逐图双端验收继续 open，passed/accepted=false。材质 r4 的 Jenkins/原生状态不借作本域结果，不重开原身份或清 writer。

## Shared rev5 的真实引用继承与 rev40 后继

20:32–20:48Z 认证 API 已读到 Shared rev5/isShared=true/41对象/原4main/34 colors/3 typographies；Button main 现在有1处真实 fill style binding，原rev2“有效引用0”只保留为历史，typography引用仍0。本域未改Shared。通过原生“更新”使原两个Button copy继承 editor.surface.1（599586e3-022c-500b-ac80-5d9c490139af）引用，保存为rev39并关闭fresh重开；11个caption覆盖及106个source refs不变，业务页266对象和私有业务组件的整页hash保持。

原画布外说明801230ec-35bd-474b-adf8-bf9ede60e2b8已通过真实图层列表/viewport textbox改正“master引用仍空”，保存rev40、关闭后fresh确认；原0,832/1280×48几何保持。当前三页388对象/18copy（11 Shared）/106refs均有效。两次未保存的selector失败及后续DOM正文等待超时保留；该等待超时不否定已读回的rev40或实际截图中的绘制内容。

[原100%工作区图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-stack-shared-rev5.png)左侧裁切，原图保留；[原生fit-all后的完整工作区图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-stack-shared-rev5-fit.png)实际80%缩放，未裁剪/缩放图片或改产品几何。两图已逐图复核；四个typed value仍空，长字/生产viewport/timeline/input/handler均未完成，不作1逻辑像素或EXE/DPI证据。[完整本轮回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-shared-rev5-consumer.json)保留身份/引用/失败/六个自有浏览器终态。未启停服务或用共享桌面，原r2及Rust/锁均未改；passed/accepted=false。

## 一次曲线手势的批量源修改：源码候选

现有单点 adapter 可以修改一个 RGBA 分量，但四条显示曲线共用一个源 key/t。源码分析表明：同一拖动若逐条提交原始 expected time，第一条改完时间后，后续条目会被判 stale；此前单点 inverse 不证明整批操作原子。现已在原 [curve adapter](../../../zircon_plugins/particles/editor/src/source/curve.rs) 增加 `apply_batch(&[ParticleSourceCurveEdit])`，没有修改共享 CurveModel/controls、Runtime、VFX、宿主或三个 ZUI。

全部 expected 值先对同一借用源校验，RGBA 按源 key 合并且同意一个目标 t；重复分量、矛盾时间、过期/已删/其它 emitter 或非法数值整批拒绝，拒绝前无源写入。检查整批最终时间而非中间逐点状态，因此可交换两 key 的时间，同时保留未选 key 的冲突检查。全部校验和 inverse 分配完成后才写源并原地排序；返回整批 inverse，由宿主作为原 history 中的一次操作保留。Linear、normalized lifetime 0..1、有限 HDR、不夹颜色和非负 size 约束保持。

[原回归文件](../../../zircon_plugins/particles/editor/src/tests/source_curve.rs) 新增5个 `particle_curve_batch_` 定义，覆盖 RGBA/undo-redo/codec lowering、两 key 时间交换、末项失败时整源及排序不变、顺序无关与另一 emitter 隔离、空批次和 stale 重放。Editor 当前是21 direct＋13 curve＝34个静态定义，执行0；Runtime10、VFX5仍独立且各执行0。格式检查及精确两源文件 guard 通过不等于 Rust 或真实手势通过。

[增量 source packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-curve-batch-mutation.json) 记录 before/after、原17源比较及7份历史 packet；只有上述两源改变。原 r2 ordinal6/input/candidate 和全部历史 packet 原字节不变，不新建 request/operation/attempt、QueryOnly 或扩大测试。此增量需要原 owner 登记 source/preparation 修订；1075项解析准备与产品全源准入、受管锁/资源/writer、真实宿主一次 permit/retention/dirty/preview/focus 仍待接续。此前 Penpot rev41/Shared rev5 为上轮实证，本轮未访问设计、服务或桌面；正常 EXE、150%DPI及双端门继续 open，passed/accepted=false，释放0 B。

## 监督恢复：burst 时间与数量的真实源修改

2026-10-25 01:02 PDT 的监督恢复明确保持 goal active、共享工作并行。本域继续实现和设计，不再以等待完整公共交付为停止条件。现已在原 Editor 源增加 [burst pair mutation](../../../zircon_plugins/particles/editor/src/source/burst.rs)：使用真实 emitter/burst stable key，以原时间和数量作为前置条件，在同一独占源借用下校验后同时写入两值并返回 inverse。源顺序、相同发生时间和零数量保留；非法、stale、已删和其它 emitter 的目标在写入前拒绝。发生时间为 simulation 秒，与 lifetime 曲线 0..1 分开。

`parse_particle_burst_count` 将严格无符号十进制文本直接解析为 u32，覆盖 16777217 和 4294967295，不经 f32；源与 TOML/lowering 保留完整整数范围。原 [真实模板](../../../zircon_plugins/particles/templates/cpu_sprite_system.toml) 没有 burst；新增双 burst 是 [回归文件](../../../zircon_plugins/particles/editor/src/tests/source_burst.rs) 显式创建的作者数据，不能当模板或产品捕获。5 个新定义覆盖真实 CPU 模拟调用、inverse、codec、原顺序、另一 emitter 隔离和拒绝无写入；它们尚未编译或执行。Editor 当前 21 direct＋13 curve＋5 burst＝39 个静态定义，Runtime10/VFX5 仍分包且各执行0。

[本轮源码 packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-burst-edit-r01.json) 记录原五路径 before/after、fmt 与精确 guard。原 r2/input/candidate 和 Runtime/VFX/曲线历史 packet 均保留；新变更是本地 lineage，不冒充 server revision 或 Rust 准入。宿主 document/session/revision、一次 permit/history/dirty、实际 preview install、durable save 及焦点仍需真实接线；domain inverse 不等于已验证撤销。

原 Penpot file/page 中已保存三张新 burst 源状态板，并复用原业务组件实例、Shared 颜色及字体。首次逐图发现默认子项插入顺序、Unicode 传输及旧 positionData 绘制问题，已保留失败原图；后续原生图层选择、流式布局和文字重测以 [设计回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-burst-states-r01.json) 的实际保存/fresh/逐图结果为准。它们是可编辑源状态设计，不充当实际输入、效果帧或窗口族验收。原三个 ZUI、公共源、锁、服务和共享桌面均未改；产品 passed/accepted 继续 false。

最终已实际保存、关闭和 fresh 重开至 rev49：三页479对象、21copy（其中11 Shared）、118 source refs全有效；原388对象及业务main保持，Shared rev5的原响应hash不变。三张完整PNG逐图复核通过本次“源状态文字可读/无裁切”检查，10项API/身份/保存/图像结构检查通过；这不包含UE对齐、Rust、正式导出、真实操作或EXE/DPI。新三个copy继承同一原业务main，未造Field/Number/Graph/Timeline或新主题。后续继续当前源revision及真实宿主接线，goal保持active，不等待Parent完整共享包才推进。

## 曲线 gesture 调用边界：共享 trait 现在接收整批 delta

2026-10-25 的本次接续把原 [ParticleSourceCurveAdapter](../../../zircon_plugins/particles/editor/src/source/curve.rs) 的 `CurveModel::Delta` 从单个 `ParticleSourceCurveEdit` 改为 `Vec<ParticleSourceCurveEdit>`；`CurveModel::apply` 只调用一次已有 `apply_batch`，并返回完整 inverse vector。此前“trait 仍单点、消费者须另调批量方法”是历史状态。没有增加另一 adapter、Curve DTO、store、clock、backend 或公共源改动。

原13项曲线回归已迁移到这个真实 trait 入口，RGBA 整批、时间交换、末项拒绝、顺序无关及空批次/重放均不再绕过 trait 直接测试 helper。单点编辑同样传一个元素的 vector；undo/redo 保留整批 vector，不弹出一个 inverse 或逐条 apply。Editor 仍为21 direct＋13 curve＋5 burst＝39个静态定义，Runtime10/VFX5另包另过滤器，三者执行均为0。

实际源码核对还确定：`ui/curve/model.rs` 只有中立 CurveModel 协议，没有 Curve 事务命令；`GraphDeltaCommand` 要求真实 GraphModel/GraphEditContext，不能把粒子曲线强行包装成 GraphModel。现有 AnimationEditCommand 绑定 CoreEditContext 的 animation_documents 与真实 revision，不能用动画 document、首个 DocId 或外造 workspace 承接粒子修改。Parent 的真实粒子 document authority、retention/一次 permit、history/dirty 和 preview/save 投影仍须集成；本域提供的只是已经能被共享 trait 调用的原子域入口。

[本次 source packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-curve-gesture-boundary-r01.json) 与 [精确宿主边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-curve-gesture-host-integration-r01.md) 登记两源 before/after、当前19源闭包观察、原 r2 未变和共享只读依赖。首次 fmt 错用2024 edition的失败保留，按实际 workspace edition2021检查通过；静态guard通过不计Rust或产品通过。原Penpot rev49、三个ZUI、Runtime/VFX、Cargo/锁及服务/桌面保持，未重投原operation或伪造server revision。正常EXE、实际150%DPI、正式导出及双端逐图仍open，passed/accepted=false。

本次向Parent官方queue投递上述新源码产物时exit1：`queue cannot contain more than100submissions`（-32600，tool36aa0e）；没有送达回执，不重复投递或改thread/数据库。文件及差异台账已落盘，继续独立工作。

随后对原Penpot四个空PropertyEditorRow值槽进行真实原生尝试，未成功生成新Text；没有通过API强改组件copy。关闭/fresh确认PFX仍rev49、Shared仍rev5且两份原响应hash完全一致，四槽仍空。实际Penpot源码的父容器选择会跳过copy内部结构，普通Text绘制不提供原canonical Slot的内容挂载合同；已排除slot锁定推测，原生等待超时及该源码限制分别保留。[精确操作、身份与原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-source-values-r01.json)是新增缺口证据，不是新设计保存或产品通过。具体交接已追加本域design/shared-contracts；仍继续域实现，不以缺共享slot而标记整族blocked。

## Burst typed caller：捕获原 pair，不在提交时重取 expected

Parent 对 supervised-resume-r01 的09:45Z独立消费与39定义仍是该时点证据。本次在原 [burst 模块](../../../zircon_plugins/particles/editor/src/source/burst.rs) 增加 `ParticleSystemSource::capture_burst_edit` 和现有 DTO 的 `set_replacement_from_text`。前者从实际 stable emitter/burst key 校验并一次捕获 time/count，初始 replacement等于expected；后者只更新 replacement，保留同一 captured keys/expected，不重新读取当前源。没有新增 DTO、document/store、clock、World、Toolkit或preview权限，也不分配/重命名作者key。

两个文本全部解析通过才替换 proposed pair；任一错误保留原delta，UI原input owner继续持有原始草稿并处理错误焦点，不能忽略Err后提交旧proposal。count复用严格十进制u32 parser，不经f32；time解析为实际Real并校验有限非负，拒绝将非零输入下溢成0，保留真实零及可表示的正subnormal，不套用播放step的EPSILON或新时间阈值。

新增5个 `particle_burst_typed_` 回归候选覆盖：相同时间下按stable key捕获、16777217/u32::MAX精度；非法pair与下溢不改delta；源并发修改后不把旧expected重基到新值；缺失identity/no-op仍须当前preflight；原pair apply、codec、inverse/redo与真实CPU调用路径。当前 `source_tests::` 静态为21 direct＋13 curve＋10 burst＝44，原 `cases.rs` 的1项不在filter；Runtime10/VFX5仍分包，全部Cargo发现/执行0，不能回写Parent此前39或旧r2输入。

[本轮packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-burst-typed-caller-r01.json)登记两源before/after、当前19源观察、前轮曲线入口与旧packet保护。宿主必须在原same-document/view/source revision的admitted read中捕获，并覆盖校验/keys payload的retention峰值；提交时先复核原authority/revision和完整pair，再判断no-op并按一次原命令prepare/commit/history处理。域field-level前置条件不表达document/CAS，encode/reopen不表达durable save。原三个ZUI、Penpot rev49、Shared5、Runtime/VFX、Cargo/锁、服务及桌面没有本次写入；真实host、Rust/native、官方导出、EXE/DPI和双端验收继续open。
