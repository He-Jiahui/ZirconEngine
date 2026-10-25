# AI 窗口族：UE 源码盘点与交付边界


当前执行接续：候选21已接BT/黑板菜单资格、真实目录/键搜索、schema新键和semantic改名业务候选，26份作者pin/61静态定义（57authoring/4gameplay），仍无Rust发现或执行回执；最新设计保存证据是52的原file/page rev47/863（不是本轮live读回）。旧下文rev37/36、候选18/38均按原观察保留。见[菜单53](ai-design-and-acceptance.md#菜单作者语义53真实选择目录搜索与单候选)与[监督恢复52](ai-penpot-design-r01.md#监督恢复52原640板空状态与真实保存重开)；原正常EXE/DPI/官方写回和实际事务/保存/执行仍open。

Owner：`01a16d53-0693-72e2-bd58-d34a65b938fa`。Parent：`01a16d4e-0491-7353-84fa-038bd7706546`。

本轮范围为 Blackboard、Behavior Tree、Zircon 适用的 EQS、StateTree 与关联 AI 调试表面。最近独立设计观察为**原 Penpot file/page rev37/862对象，保留十三业务案例、四业务主件、58业务副本和七个Shared副本；类型菜单的共享搜索、受控Button覆盖/重置、原Search文字的Shared颜色引用及新进程重开已有证据，原资产导出、生产宿主接线及视觉验收未完成**。库原生更新提示经点击后消失且新进程未再显示，但syncedAt未推进，未捕获同步HTTP ACK，持久化同步仍open。源候选18为17份本域文件、34 authoring+4 gameplay共38项静态定义，0 Rust发现/编译/执行；旧rev14/19/28/36与12源24/28项按观察时点保留，不代表当前候选。板身份、原尺寸渲染图和逐图未完成项见 [实际设计记录](ai-penpot-design-r01.md)。用户原答复“等待parent告知”已由真实分配续接，不另建文件或平行组件库。

最新管理交付 `combined-ai-particles-lock-resolution-r2` 的五阶段resolver事务已有管理测试和真实准备；#361资源检查成功执行，但reservation/writer拒绝，resolver未启动。#416管理427 checks不是AI产品测试；专用工具、实际资源/单锁writer、完整候选和原子发布仍待准入，captureReady=false。原owner回执中的24项属于历史观察，不能替代候选18的38项。具体收件边界见 [接续24](ai-parent-handoff.md#联合锁资源阶段实际回执24)。

## 输入身份与工作边界

- 实际 checkout：`E:/Git/ZirconEngine`，初始 HEAD `75a76ee963792d5a316f738519e966937f770036`，分支 `main`。已有大量其他会话的主题、插件、原生捕获和 UI 绘制改动；本轮没有修改这些文件。
- 参考树 `dev/UnrealEngine` 只读。其 `Engine/Build/Build.version` 写明 MajorVersion 6、MinorVersion 0、PatchVersion 0、Changelist 0、BranchName UE5，不能凭目录或 BranchName 断言 UE5 的某个正式版本。当前证据使用实际文件 SHA256；参考 checkout 的不可变上游 revision 仍待补充。
- 已读根 `AGENTS.md`、本套件 overview/component-reuse、UI 技能的 `references/ue-penpot-fidelity.md`、工程与产物管理规则。没有启动退役协调器，没有创建源码副本。
- 产物清单：`E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/artifacts.json`。证据指纹见同目录 `source-identities.json`。
- 最新统筹依据：[Parent 会话记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md) 和 `parent-01a16d4e/penpot-assignments.json`。AI file `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5a`、初始 page `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5b`，Parent 核实初始 revn/vern 为 0/0、page object count=1；不是设计通过。材质会话牵头共享图表面，动画会话拥有共享时间轴/曲线/播放，Parent 统筹跨域合同/公共框架/主题与集成。
- 用户选择的 UE 源码是视觉和行为目标。区域、输入、属性关系沿用项目批准的 [App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell)、[Forms](https://pro.reactbits.dev/docs/app-ui/forms)、[Command Menu](https://pro.reactbits.dev/docs/app-ui/command-menu) 作为关系映射；本轮未据此另行推导像素或编写布局。

下文的“适用”是产品目标分类，不是“已实现”。“待定”是当前没有对应后端/路线证据，不能用另一个能力更名替代。“不适用”保留具体理由和邻接依赖。

## 源码索引

相对 `dev/UnrealEngine/Engine` 的索引：

| 代号 | 源码入口 | 负责的事实 |
| --- | --- | --- |
| BT-T | `Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeEditor.cpp`、`BehaviorTreeEditorModes.cpp`、`BehaviorTreeEditorTabs.cpp`、`BehaviorTreeEditorTabFactories.cpp` | Toolkit、两种模式、全部基本 tab、布局、选中、双击、Undo/Redo、调试命令映射 |
| BT-S | 同目录 `SGraphNode_BehaviorTree.cpp`、`SGraphNode_Decorator.cpp`、`BehaviorTreeConnectionDrawingPolicy.cpp`、`EdGraphSchema_BehaviorTree.cpp`、`EdGraphSchema_BehaviorTreeDecorator.cpp` | 真节点/端口/连线、装饰器与服务、创建菜单、拓扑限制与复合装饰器图 |
| BT-C | 同目录 `BehaviorTreeEditorCommands.cpp`、`BehaviorTreeEditorToolbar.cpp`、`BehaviorTreeDebugger.cpp`、`FindInBT.cpp`、`SBehaviorTreeDiff.cpp` | 命令、模式/调试工具栏、搜索、历史和差异 |
| BB | 同目录 `SBehaviorTreeBlackboardEditor.cpp`、`SBehaviorTreeBlackboardView.cpp`、`DetailCustomizations/BlackboardDataDetails.cpp`、`BlackboardKeysDetails.cpp`、`BlackboardSelectorDetails.cpp` | 键编辑、运行值、详情、类型和选择器 |
| BT-STYLE | `Source/Editor/BehaviorTreeEditor/Public/BehaviorTreeColors.h`、`Source/Editor/EditorStyle/Private/StarshipStyle.cpp` 的 `BTEditor.*` | 节点类别、执行/选择/断点状态、字体、刷子、图标；LinearColor 不能直接当 sRGB 十六进制复制 |
| EQS | `Plugins/AI/EnvironmentQueryEditor/Source/EnvironmentQueryEditor/Private/EnvironmentQueryEditor.cpp`、`SEnvQueryProfiler.cpp`、`SEnvQueryLoadGraph.cpp`、图节点/Schema 文件 | EQS graph、properties、profiler、generator/test 与统计 |
| ST | `Plugins/Runtime/StateTree/Source/StateTreeEditorModule/Private/StateTreeEditor.cpp`、`StateTreeEditorModeToolkit.cpp`、`StateTreeEditorLayoutHost.cpp`，Public 的 `StateTreeEditorWorkspaceTabHost.h` | 基本四 tab、Workspace 扩展的九类 tab、布局、嵌入模式 |
| ST-S | 同模块 `SStateTreeView.cpp`、`SStateTreeViewRow.cpp`、`HierarchyGraphView/*`、`StateCentricView/*`、`FindTools/*`、`Debugger/*`、`Simulation/*`、`SStateTreeEventsView.cpp`、`StateTreeEditorStyle.cpp` | 状态层级、条件/任务/转换、搜索、绑定、事件、统计、调试/模拟和样式 |
| DBG | `Source/Runtime/AIModule/Private/GameplayDebugger/GameplayDebuggerCategory_AI.cpp`、`GameplayDebuggerCategory_BehaviorTree.cpp`、`GameplayDebuggerCategory_EQS.cpp` | 选定真实 Actor 的 AI/黑板/EQS 调试覆盖层 |

## 窗口与邻接表面盘点

| ID | 窗口/表面与 UE 身份 | 分类 | Zircon 对应与理由 |
| --- | --- | --- | --- |
| AI-W01 | Behavior Tree graph document，`Document` | 适用 | 有 `AiBehaviorTreeDescriptor`、节点目录和实际编译/执行器；现有插件只注册工具与空画布，编辑闭环未实现 |
| AI-W02 | 节点 Details，`BehaviorTreeEditor_Properties` | 适用 | 节点 implementation、parameters、children、abort_policy 有真实 DTO；需要持久编辑、类型校验与撤销 |
| AI-W03 | Find in Behavior Tree，`BehaviorTreeEditor_Search` | 适用 | 对当前文档节点和属性搜索并跳转；不是另一个资产浏览器 |
| AI-W04 | BT 内 Blackboard 运行值，`BehaviorTreeEditor_Blackboard` | 适用 | `AiPieMirror` 保存真实 frame.blackboard；仍需 UI 绑定、会话/世界/实体选择和陈旧状态清理 |
| AI-W05 | Blackboard 编辑模式，`BehaviorTreeEditor_BlackboardEditor` | 适用 | 有 `AiBlackboardSchemaDescriptor`，没有独立黑板资产 Toolkit；需真实键创建/重命名/删除、引用检查与保存 |
| AI-W06 | Blackboard Details，`BehaviorTreeEditor_BlackboardProperties` | 适用 | 当前 schema 只有 id/name/keys(type/required)，UE parent/inherited/base-class/instance-sync 等语义存在缺口 |
| AI-W07 | 图创建菜单、节点上下文菜单、端口拖出菜单 | 适用 | 从真实节点目录生成；需按父节点类别、单父、非循环、执行状态过滤，不能使用硬编码假菜单 |
| AI-W08 | 节点的 Decorator/Service 子区域及其上下文菜单 | 适用；后端有缺口 | Zircon Decorator 恰有一个子节点，Service 必须零子节点；UE 两者均可附着宿主。Decorator 视图映射待定，Service 还缺实际附着执行及更新逻辑 |
| AI-W09 | Composite Decorator 子图 | 待定 | Zircon 条件节点不等于 UE 复合逻辑子图；没有可证明的同等序列化合同 |
| AI-W10 | BT 调试实例选择、暂停/恢复/停止、当前/保存值 | 适用 | 有真实镜像与 node-result 事件；暂停命令、UI 选择和真实 PIE 传递仍未验证 |
| AI-W11 | BT Back/Forward Into/Over、Step Out、父树/子树跳转 | 适用；后端缺口 | runtime 有 subtree；当前镜像只保留最新帧，不包含 UE 的逐步回溯历史或断点控制合同 |
| AI-W12 | Behavior Tree Diff | 待定 | 有版本操作基础，但本轮没找到该资产的语义 diff 工具；不以普通文本 diff 冒充图差异 |
| AI-W13 | New Blackboard/资产选择及资产跳转 | 适用 | 需接入现有资产系统与命令总线；BT 注册的 `.btree.toml` importer 是 descriptor，尚未发现具体 open/import/validate/compile handler |
| AI-W14 | New Task/Decorator/Service 的 UE Blueprint 类向导 | UE 专属实现不适用 | Zircon 没有 UE Blueprint/UClass 体系；Zircon 节点目录/脚本扩展为邻接依赖，不能伪称 UE 向导已还原 |
| AI-W15 | EQS graph，`EnvironmentQueryEditor_UpdateGraph` | 待定 | 当前 AI 插件、runtime AI DTO 未找到 query/generator/test 后端；perception 或 navigation 不是 EQS 的替代 |
| AI-W16 | EQS Properties，`EnvironmentQueryEditor_Properties` | 待定 | 依赖 W15 的真实数据、参数和 undo 合同 |
| AI-W17 | EQS Profiler，`EnvironmentQueryEditor_Profiler` | 待定 | UE 统计 Max/Avg/Load/Count 来自实际 query；没有计时/结果后端，禁止假统计 |
| AI-W18 | EQS testing pawn 与 GameplayDebugger EQS | 待定 | 需要真实世界 query 执行、采样候选和评分渲染；不能把静态点云当作执行反馈 |
| AI-W19 | StateTree View，`StateTreeEditor_StateTreeView` | 待定 | 没找到 Zircon StateTree 资产/运行合同；Behavior Tree 及动画状态机不是 StateTree |
| AI-W20 | StateTree Asset/Selection Details | 待定 | `StateTreeEditor_AssetDetails`、`StateTreeEditor_SelectionDetails`；依赖 W19 与真实 schema/bindings |
| AI-W21 | StateTree Compiler Results | 待定 | `StateTreeEditor_CompilerResults`；需真实编译器及可点击诊断跳转 |
| AI-W22 | StateTree Bindings、Outliner、Search、Statistics | 待定 | WorkspaceTabHost 的四个独立身份；不能遗漏为“一个状态树窗口” |
| AI-W23 | StateTree GraphView、StateCentricView | 待定 | 同一资产的两种投影视图，均依赖真实状态/转换/条件数据 |
| AI-W24 | StateTree Debugger、Events、Simulation | 待定 | WorkspaceTabHost 的三个独立身份；需要 trace、实例、事件、模拟执行，禁止装饰性时间线 |
| AI-W25 | GameplayDebugger AI/BT、感知视口与实体表；对应 OVW-42 | 适用；AI 牵头数据投影 | Parent 已明确共享 PIE/diagnostic/trace 宿主归属，Actor 仅供真实关联定位；AI editor 新增每 agent 真实投递来源，四项回归未执行。现有 Table 仍无真实 UI 绑定，源 revision/trace/stack/断点缺口 open；见 [专项合同](ai-gameplay-debugger.md) |
| AI-W26 | UE Visual Logger、Rewind/Insights 公共宿主 | 邻接公共工具，本窗口族不单独重建 | 作为 AI 调试的跳转依赖登记；公共宿主和 trace 基础设施由其 owner 统筹，不另建同名平行系统 |
| AI-W27 | MassAI/MassEntity 专用调试窗口 | 当前不适用 | 没找到 Zircon MassEntity 执行模型；不能由一般 ECS 的存在推定适用。若 Parent 有路线证据再重新分类 |

EQS tab 名称已核对 `EnvironmentQueryEditor.cpp` 的实际字符串常量。StateTree 扩展身份已从 `StateTreeEditorLayoutHost.cpp:16` 起核对：

| 表面 | 实际 tab ID | 注册条件 |
| --- | --- | --- |
| Bindings | `StateTreeEditor_Binding` | 默认注册 |
| Outliner | `StateTreeEditor_StateTreeOutliner` | 默认注册 |
| Search | `StateTreeEditor_StateTreeSearch` | 默认注册 |
| Statistics | `StateTreeEditor_StateTreeStatistics` | 默认注册 |
| Graph View | `StateTreeEditor_StateGraphView` | 默认注册 |
| Events | `StateTreeEditor_Events` | 默认注册 |
| Simulation | `StateTreeEditor_Simulation` | 默认注册 |
| Debugger | `StateTreeEditor_Debugger` | `WITH_STATETREE_TRACE_DEBUGGER` |
| State Centric View | `StateTreeEditor_StateCentricView` | `IsStateCentricViewEnabled()` |

## Zircon 的真实能力与缺口

已确认源码能力：

1. `zircon_runtime/src/core/framework/ai/behavior_tree.rs` 提供稳定节点 ID、拓扑、implementation、参数、abort policy 的可序列化 DTO；`blackboard.rs` 提供 bool/integer/scalar/string/vec3/entity 六类值和 schema。
2. `zircon_plugins/ai/runtime/src/behavior_tree/compile.rs` 有真实 `.btree.toml` 解析、拓扑/节点目录编译；manager、executor、perception 与 effects 均为真实运行路径。源码存在不代表本轮测试或窗口执行通过。
3. `zircon_plugins/ai/editor/src/plugin.rs` 已注册 BT view/template/palette、资产 Toolkit 与 import/open/validate/compile 命令描述；未找到本插件的命令实现和文档编辑会话。
4. `runtime_mirror.rs` 按 play_session/world/entity 接受真实 debug snapshot 并拒绝跨会话/旧序列；独立 node-result mirror 处理真实节点事件。现有 overlay 可消费镜像生成实际感知几何。
5. 依据材质 owner 的交接，已继续核对 `zircon_editor/src/ui/graph/{model,canvas,routing,commands,node_widget}.rs`：存在实际 `GraphModel`、节点/端口/边投影、Tree/Dag 验证、视点/多选/拖动、两种路由、revision-bound/budgeted delta 命令与 clipboard 协议。AI 应实现域适配器；没有理由重建这些基础。它们还不是可复用的已发布 Penpot 组件或已接 AI 文档的 retained 窗口。

运行时能力不能按目录名称一概计为完整：`manager/validation.rs:143` 只允许 Decorator 恰有一个子节点，Service 零子节点；`behavior_tree/executor.rs:869` 的 `evaluate_service` 在无子节点时只读取 `service_result` 并返回状态。当前 `update_blackboard_distance` 注册到这条路径，没有计算距离或写回黑板。此 Service backend 保持明确未完成，不能用其状态返回作为真实服务执行反馈。

确认不能作为交付的现有表面：

- `zircon_plugins/ai/editor/behavior_tree.zui` 的 `AiBehaviorTreeGraphCanvas`、`AiBehaviorTreeInspector` 是 `Space`，Table 没有 bindings/events；这不是可工作的 graph/blackboard editor。
- `zircon_editor/assets/ui/editor/workbench/modules/core/ai/behavior_workspace.zui` 的 `BT_Enemy`、`Combat Root`、`Attack Target`、`Running`、`Runtime Trace: branch changed to Attack` 是静态示例文本。当前 `module_command_feedback.rs` 对保存/编译返回 `Not implemented (preview)`。不得用选择该页签的截图声称 AI 窗口完成。
- `zircon_plugins/ai/editor/perception_debug.zui` 是未绑定实体表；实际 overlay controller 的存在不等于表格已显示真实调试数据。
- `zircon_plugins/ai/dist/src/lib.rs` 的动态入口 `invoke_command: None`，命令清单为空；这不是 import/open/validate/compile 的执行入口。当前搜索也没有在 Editor core/App 找到这些 AI 业务操作的注册实现，仍须接真实文档命令链。

## 设计与实现阶段

| 阶段 | 本轮结果 | 下一步与完成条件 |
| --- | --- | --- |
| UE/能力盘点 | 已形成源码有据的范围和差异；无运行验收 | Parent 确认 EQS/StateTree 路线及共享 graph owner |
| 真实 Penpot | 最近独立观察rev36/862对象，四业务main/58业务copy+7 Shared copy/194 shapeRefs保持。原类型菜单200×240内加入原生SearchInput、flow重排和保存重开；受控Button真实accent覆盖后原生重置，新进程读回data/objects/components一致。十三业务案例与旧鼠标原型证据保留 | 左右键规则、可编辑搜索、Esc/方向键/焦点返回仍未通过；Property.value未绑定，Shared rev2虽有34 colors/3 typographies，其Button源仍缺默认style RefId。官方PNG/ZUI导出、全业务组件复用、640/900裁切与debug overflow仍open |
| 原始 ZUI v2 | 未改动 | 在 Penpot 设计验证后，将导出 delta 写回原 owner，保留稳定 ID/事件/绑定；不写一套替代演示资产 |
| Editor 功能 | 候选18保留不可变BT编辑/拓扑与共享Down/Up路由，新增精确tree-schema语义引用扫描和带旧值、hash/revision及业务inverse的改名计划；不修改标题/事件/任意同名字符串。17源按字节复用格式/guard，34 authoring+4 gameplay=38静态定义，全部未Rust执行 | 真实关联/index provider、operation factory、跨文档history/save准入、GraphModel预算/一次permit/prepare/apply/undo、同代际pin测量/paint/hit/focus及执行反馈仍缺。未知引用阻塞，运行值只读；纯值计划不能取得生产权限或热替换schema |
| 插件/Rust 验证 | 原owner最新e063指针交付恢复receiver，11 job API/Home一致是其回执范围；四精确AI filters保留。#416管理427 checks、#360真实准备、#361实际资源拒绝和#417关闭本次等待行分别记账；旧d287/9c及667漂移保留 | 38项候选0执行，原bt01/create身份不重放。嵌套锁仍04de41d2…，四边尚缺；工具独立锁输出、实际source/resource/pool/cache/sole writer、完整候选diff/发布恢复未准入。外部full-source policy归属/验收独立open；不以当前pins或管理成功替代产品结果 |
| 正常 EXE 与视觉 | 无本轮捕获 | 经 Parent 排队后在正常窗口捕获，完成所有尺寸/DPI/状态的双端比较与逐图复核 |

具体交互与证据合同见 [ai-design-and-acceptance.md](ai-design-and-acceptance.md)。发给 Parent 的依赖、共享提案和执行阻碍见 [ai-parent-handoff.md](ai-parent-handoff.md)。这两份记录没有聊天送达/已读回执含义。

Parent要求的传递依赖预检保留于`ai-authoring-readiness-03.json`：AI leaf features=[]仍继承runtime native-host与Editor target-editor-host，只是静态manifest观察。原18.94GiB与recovery07的15.55GiB各按观察时间保留，不推断增长归属；实际#361资源拒绝另按接续24记账。旧十源及原十一项输入不覆盖后续代码；接续12的十二源由`bt-edge-routing-12-*`记录，当时24 authoring+4 gameplay均未运行，当前候选见上表。以下接续13/15保持原时点记录，不代表新的源码、服务或资源准入。

## 生产输入包13接续

Parent第四次实际复核已读回rev28/855对象/4业务main/64copies，并核对当前十二源与八依赖；较早分配空文件、仅读rev14和20+4均是历史观察。当前24 authoring+4 gameplay共28定义仍0执行，原bt-edge-routing-12候选未改。本轮按 [输入合同13](ai-design-and-acceptance.md#生产适配最小输入包13) 提交资产身份/源baseline、四operation、同文档事务、同代际几何/焦点与save-token所需的最小输入。

新增实际核查：资源OpenAsset分支只打开表面/写route metadata，没有AI factory或文档注册；现存enemy_behavior_tree.toml是普通Data旧格式，不能当当前可编辑BT源。source/revision/geometry/保存authority尚无真实运行输入，本包不新增空factory、演示资产或执行高亮。静态源码能力和输入要求不等于正式导出、产品Rust或正常EXE通过；完整窗口族继续pending，passed/accepted=false。

## 当前Penpot/插件接续15

已实际核实AI原file/page仍rev28/855、Shared rev1；19002插件manifest/bundle HTTP200且bundle同本地发行物442bed70…，旧4213失败仅作历史。正式codec所需source-owner导入消费已交原owner，业务设计元数据没有冒充原ZUI可逆guard。900/640图仍分别左右裁10.5/101.5，640缺debug selector/overflow，已交共享graph/Parent的真实pan/zoom与host输入合同，未坐标修图或删操作。见 [接续15](ai-penpot-design-r01.md#当前发行物与窄窗实际读回15)。无Penpot形状/ZUI/Rust改动或新测试/生成提交，28定义仍0执行；正常EXE/150%DPI/双端验收继续open。


黑板作者语义最新接续：[黑板54](ai-design-and-acceptance.md#黑板菜单54创建候选语义改名与引用未知边界)。未知引用不当零，Type/Delete与运行值写入明确未支持；宿主lease/CAS/history、正式source/export和产品验收未完成。
