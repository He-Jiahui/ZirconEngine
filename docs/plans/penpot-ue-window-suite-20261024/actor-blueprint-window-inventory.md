# Actor / Blueprint 与遗漏窗口盘点

状态：源码盘点，未实现验收。所有“适用”均指 Zircon 有使用目标，不表示存在完整后端或本轮已完成。归属以用户分工和对应会话的落盘记录为准；没有收件依据的建议不写成已接收任务。

参考前缀 `UE/` = `dev/UnrealEngine/Engine/Source/`，`UEPlugins/` = `dev/UnrealEngine/Engine/Plugins/`。下表列出实际源文件、可检索 symbol/tab identity 和当前能力依据；参考树只读。

当前套件 `overview.md` 只声明窗口族，没有全部独立会话和窗口级收件记录。因此“遗漏”表是待 Parent 对账的覆盖候选；既不推断别的会话没有实现，也不将这些窗口自动纳入本会话的 UI 修改范围。

## 本窗口族：Toolkit、TabFactory、Slate 与操作

| ID | 窗口/状态 | 适用性与理由 | 实际 UE 源码依据 | Zircon 当前能力与缺口 | 建议归属 |
| --- | --- | --- | --- | --- | --- |
| ABW-01 | Actor Blueprint 主资产编辑器 | 适用：组合组件、属性、图与编译工作流 | `UE/Editor/Kismet/Private/BlueprintEditor.cpp:2471` `RegisterTabSpawners`、`:2529` `InitAssetEditor`、`:2636` `RegisterApplicationModes`；`BlueprintEditorModes.cpp:517` unified mode | 有 `editor.prefab` view descriptor 和独立文档框架；未证实有 Actor 类文档及持久化命令 | 本会话；共享文档框架归 Parent |
| ABW-02 | 场景 Actor 实例的组件树 | 适用：已有场景和反射组件能力 | `UE/Editor/SubobjectEditor/Private/SSubobjectInstanceEditor.cpp`：`OnDeleteNodes`、`OnAttachToDropAction:416`、`AddNewSubobject:543`、reset defaults transaction `:700` | `core/editing/command/add_component.rs` 有真实动态组件添加、gateway/selection/world/history 校验。不能推断删除/挂接/类覆盖全部接通 | 本会话；主场景 Details 集成归 Parent |
| ABW-03 | Blueprint Components 树 | 适用：类/Prefab 组成的编辑目标 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:293` `FConstructionScriptEditorSummoner`；`:305` 返回 `GetSubobjectEditor`；`UE/Editor/SubobjectEditor/Private/SSubobjectBlueprintEditor.cpp:333,576,716` | 现有 Prefab 左列为固定示例行；没有 class component template 的实际写入命令 | 本会话 |
| ABW-04 | Class Defaults、data-only defaults、多选 defaults | 适用：用户明确要求类与实例区分 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:147` `FDefaultsEditorSummoner`；`BlueprintEditorModes.cpp:164` `Standalone_BlueprintDefaults_Layout_v7` | Prefab instance override query 可复用作诊断，不能替代 class-default model；继承/reset/persistence 待接 | 本会话 |
| ABW-05 | Class Settings、父类与接口列表 | 适用：用户明确要求接口及类身份 | `UE/Editor/Kismet/Private/SBlueprintEditorToolbar.cpp:278` `EditGlobalOptions`；`BlueprintEditor.cpp:3657` reparent transaction | 反射类型存在；可编辑父类/接口 schema、兼容性校验和迁移未确认。映射待 Parent 决策 | 本会话 |
| ABW-06 | My Blueprint（变量、函数、宏、事件分发器） | 适用：作为真实脚本资产的成员导航 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:395` `FMyBlueprintSummoner`；`SMyBlueprint.cpp`；`BlueprintEditorCommands.cpp:86` 起 new document commands | 有通用树/列表和图 descriptor；没有已确认的 Actor 成员模型与全部成员命令 | 本会话 |
| ABW-07 | Construction Script 图 | 适用：用户明确要求构造逻辑 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:90` `FGraphEditorSummoner`；`BlueprintEditor.cpp` 图文档与跳转；`UE/Editor/GraphEditor/Private/GraphEditorActions.cpp` | 不等同于 ABW-03。需真实构造图模型、运行时重建命令和预览更新，当前未找到本族后端 | 本会话业务；材质牵头图表面，Parent 跨域合同/集成 |
| ABW-08 | Event Graph、函数图及文档历史 | 适用：用户明确要求图编辑与执行链路 | 同上 `FDocumentTabFactoryForObjects<UEdGraph>`；`OnTabActivated:55`、`SaveState:73`；`BlueprintEditor.cpp:4792` `JumpToNode` | Runtime 已有 `script.bindings` 与 onStart/onUpdate/onFixedUpdate 回调，但图到 module 的编译/持久化、typed pins、source map 和 undo 未确认；材质/动画图不能冒充 Actor event graph | 本会话 |
| ABW-09 | Blueprint Interface 编辑模式与函数签名 | 适用：目标是接口契约；具体 VM 映射待定 | `UE/Editor/Kismet/Private/BlueprintEditorModes.cpp:338` `Standalone_BlueprintInterface_Layout_v4`；`BlueprintEditor.cpp:2644` 仅 Interface mode | 不应为 interface 添加 Actor Components/实例 Viewport。当前无已确认的可编辑接口资产 schema | 本会话 |
| ABW-10 | Palette、图右键动作菜单 | 适用：节点发现、类型过滤与键盘操作 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:357` `FPaletteSummoner`；`UE/Editor/GraphEditor/Private/GraphEditorActions.cpp:20` 起 | 共享 `GraphNodePaletteDescriptor` 是扩展描述。需要以本族真实可执行节点供给 action，不用静态假列表 | 本会话数据；共用菜单/图组件 owner 待定 |
| ABW-11 | Compiler Results、错误节点跳转、compile status | 适用：用户要求真实编译反馈 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:433` `FCompilerResultsSummoner`；`BlueprintEditorCommands.cpp:58` F7；`SBlueprintEditorToolbar.cpp:496` 状态 brush；`BlueprintEditor.cpp:4340` 错误跳转 | 有共享 diagnostic row 和任务日志，尚无本族编译器/diagnostic node mapping；Prefab validate 目前只是 preview feedback | 本会话；通用日志/任务模型复用 Parent |
| ABW-12 | Find Results / Find in Blueprints / Replace References | 适用：资产与节点级检索、引用修改 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:414,452`；`BlueprintEditorCommands.cpp:54` Ctrl+F、Ctrl+Shift+F | 资产引用索引可作为候选，不能推断有图节点引用替换；跨资产 rename/replacement 与 undo 待实现 | 本会话；跨资产索引归资源会话协调 |
| ABW-13 | Bookmarks、图导航前进/后退 | 适用：大图定位和文档恢复 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:376` `FBookmarksSummoner`；`BlueprintEditorCommands.cpp:69` Home/PageUp/PageDown/BackSpace | 需要绑定真实图/节点 ID 和文档历史，不是屏幕坐标或静态定位按钮 | 本会话 |
| ABW-14 | Blueprint Debugger、Execution Flow、Call Stack、watch/breakpoints | 待定：先确认 Zircon VM 可提供对应调试数据和事件图定位 | `UE/Editor/Kismet/Private/BlueprintDebugger.cpp:101,147,167,186`；`SBlueprintEditorSelectedDebugObjectWidget.cpp`；`UE/Editor/GraphEditor/Private/GraphEditorActions.cpp:48` F9 | 不直接接入 UE 的 Kismet 调试对象。需要 VM owner 给出暂停、堆栈、watch、断点 source map 和生命周期合同 | 本会话盘点；VM 调试 owner 与 Parent 确认 |
| ABW-15 | Blueprint Timeline 内嵌文档 | 适用，但模板/图/运行语义仍需本域后端 | `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:119` `FTimelineEditorSummoner<UTimelineTemplate>`、`SaveState`/`ConstructTabNameForObject`；`STimelineEditor.cpp` Event/Float/Vector/LinearColor | 既有 timeline strip 不是完整曲线/事件轨。沿用同 owning asset/history、seconds 与稳定 track/key ID，不另建平行编辑器 | 本域业务；动画 chat `01a16d51-741b-73b3-a61a-2189cd38f81e` 负责共享 timeline/curve/transport，Parent 集成 |
| ABW-16 | User Defined Enum 编辑器 | 待定：Zircon 是否需要资产化可编辑 enum schema | `UE/Editor/Kismet/Private/UserDefinedEnumEditor.cpp:33,367` `UserDefinedEnum_EnumeratorEditor`；`BlueprintEditorModule.cpp:464` | 有反射类型不代表可在运行时增删枚举、迁移已序列化值；需 schema 决策 | 本会话盘点；数据资产会话协调 |
| ABW-17 | User Defined Struct 成员与默认值 | 待定：可编辑 schema 与引用迁移策略尚未确认 | `UE/Editor/Kismet/Private/UserDefinedStructureEditor.cpp:333,343,348` 两个 tabs；`BlueprintEditorModule.cpp:471` | 通用 JSON/Data 资产不自动具备 UE user-defined struct 的布局/版本迁移语义 | 数据资产会话；本会话提供图类型依赖 |
| ABW-18 | 类型/父类选择器、Class Viewer | 适用：反射组件及脚本类型发现 | `UE/Editor/ClassViewer/Private/ClassViewerModule.cpp:31,51` `ClassViewerApp` | 复用真实 component type catalog；过滤继承可编辑性与 capability。不能硬编码 UE UClass 列表 | 本会话数据；选择器归共享/资源 owner |
| ABW-19 | Macro Library / Function Library | 待定：用户要求图的关联窗口，先确认脚本资产种类 | `UE/Editor/Kismet/Private/BlueprintEditorModes.cpp:434` macro mode；`BlueprintEditor.cpp:2652,2660` library 分流 | 不给纯函数库显示无效的 Actor Components/实例按钮；编译和调用模型未确认 | 本会话 |
| ABW-20 | Blueprint Diff / merge 冲突 | 待定：需真实资产修订和图结构 diff/merge | `UE/Editor/Kismet/Private/BlueprintEditorCommands.cpp:102` `BeginBlueprintMerge`；`UE/Editor/Kismet/Private/SCSDiff.cpp` | Git 文本 diff 不足以证明节点/组件语义合并。不得自动覆盖冲突 | 版本/资源 owner；本会话提供语义 |
| ABW-21 | Animation Blueprint 专用图 | 适用，移交动画会话：不属于本族独占实现 | `UE/Editor/UnrealEd/Private/AnimBlueprintEditorBridge.cpp`；Kismet `AddNewAnimationLayer` | 现有 AnimationGraph model 与本族 Actor 事件图不能互换 | 动画与角色会话 |
| ABW-22 | Widget Blueprint Designer/Graph | 适用，Widget 会话已明确承接 UI 范围 | `UE/Editor/UMGEditor/Private/TabFactory/NavigationTabSummoner.cpp`；`UE/Editor/UMGEditor` | 现有 UI document authoring 不等于 Actor 类编辑器；该会话未承接 AI | Widget 会话，见下方收件范围对账 |
| ABW-23 | UE C++/UHT/Kismet 专用编译中间产物、UObject 修复 | 不适用：Zircon 不使用 UHT、K2 字节码与 UBlueprint 序列化 | `UE/Editor/Kismet/Private/BlueprintEditorCommands.cpp:64` `RepairCorruptedBlueprint`、`:96` `SaveIntermediateBuildProducts`；`SBlueprintEditorToolbar.cpp` developer compiler menu | 可另定义 Zircon 自身编译诊断，不能复制无效 UE 动作并称功能已实现 | 不创建本族产品入口 |
| ABW-24 | Level Blueprint / 关卡事件图 | 待定：关卡事件编排有使用目标，先确认场景脚本资产及生命周期 | `UE/Editor/LevelEditor/Private/LevelEditorActions.cpp:2951` `OpenLevelBlueprint` 取得 PersistentLevel 的 `GetLevelScriptBlueprint` 并打开独立资产编辑器；`UE/Editor/Kismet/Private/BlueprintEditor.cpp:1907,4089,4364,4848` level-script 分流及 component capability | 当前 scene/level 存在不等于拥有 level-script graph。关卡绑定、Actor 引用失效、保存/加载和执行入口未确认；不能套用 Actor 组件树 | 本会话图合同；主场景菜单/关卡与 Parent 协调 |
| ABW-25 | Create Blueprint From Selection 模态对话框 | 适用：从场景选择创建可编辑组合资产；三种 UE 创建语义的映射待定 | `UE/Editor/KismetWidgets/Private/CreateBlueprintFromActorDialog.cpp:181` `GetValidCreationMethods`；`:436` Subclass / Child Actors / Harvest Components；`:877` `OpenDialog`、`:914` `OnCreateBlueprint`；LevelEditorActions `:3075` 入口 | Prefab create-from-selection 命令未注册。需要真实名称/路径/父类校验、能力过滤、创建/替换事务及失败恢复；不能把三种模式都映射到同一假按钮 | 本会话业务；资源路径选择、模态焦点和主场景事务依赖共享 owner |

`BlueprintEditorTabs.cpp` 共列出 15 个 tab 常量；`DebugID` 的存在不单独证明当前 unified layout 启用它。`UserDefinedStructureID` 还要结合实际独立 struct Toolkit 注册；不以常量表替代全部 Toolkit 盘点。

## 其他窗口覆盖候选：世界创作

| ID | 窗口/工具 | 适用性、理由 | 实际 UE 源码/注册依据 | Zircon 当前源码能力 | 建议归属及缺口 |
| --- | --- | --- | --- | --- | --- |
| OVW-01 | Landscape Manage / Sculpt / Paint | 适用：Terrain 与 layer stack 资产已注册 | `UE/Editor/LandscapeEditor/Private/LandscapeEditorModule.cpp:89` 注册 mode；`SLandscapeEditor.cpp:174,192,436` `FLandscapeToolKit` inline/tab UI | `zircon_plugins/terrain/editor/src/plugin.rs` 有 heightfield/weightmap/create/open/sculpt descriptors；`assets/ui/editor/world/extension_terrain_editor_workspace.zui` 存在 | 建议世界创作会话；descriptor 不等于可执行 tool factory，笔刷采样/预览/undo 尚待逐项验证 |
| OVW-02 | Foliage Paint / Select / instance list | 适用：植被/散布工作流用得到 | `UE/Editor/FoliageEdit/Private/FoliageEditModule.cpp:66` mode；`FoliageEdModeToolkit.cpp:27,45` inline toolkit | `extension_foliage_editor_workspace.zui` 与 `extension_scatter_editor_workspace.zui` 已存在；feedback 中相应动作仍有 preview 占位 | 建议世界创作；先确认实际实例数据、seed、笔刷、选择、碰撞与撤销 owner |
| OVW-03 | Navigation 可视化/数据切换 | 适用：已有 Navigation plugin 和 NavMesh 资产 | `UE/Editor/LevelEditor/Private/LevelEditorActions.cpp:4585` `CycleNavigationDataDrawn`，Alt+N | `zircon_plugins/navigation/editor/src/plugin/registration/operations.rs` 注册实际操作 factories；runtime mirror 有 PIE 数据消费者 | 建议导航会话；不要创建无来源的独立 UE “Navigation Tab”来冒充匹配 |
| OVW-04 | NavMesh/NavigationSettings 属性与 bake | 适用：本机资产 type 已注册 NavMesh/NavigationSettings | `UE/Editor/UnrealEd/Private/WorldPartition/SWorldPartitionBuildNavigationDialog.cpp`；LevelEditor Build/Navigation 操作 | `zircon_editor/src/core/asset/type_registry/builtin.rs:35` 起；Navigation editor registration/assets/operations | 建议导航会话；需真实 bake job、dirty/failed 状态和取消/结果加载 |
| OVW-05 | Levels / Level Details | 适用：scene/level 持久化、加载与当前关卡用得到 | `UE/Editor/LevelEditor/Private/LevelEditor.cpp:166,167` IDs；`SLevelEditor.cpp:1191,1202` 实际 body；`WorldBrowser/Private/WorldBrowserModule.cpp` | `zircon_runtime/src/scene/module/default_level_manager.rs` 与 project IO/lifecycle；`extension_level_streaming_workspace.zui` | 建议世界创作；当前关卡、可见/锁定、保存、加载/卸载与事务需核验 |
| OVW-06 | World Composition | 待定：UE legacy tiled-world 模型是否对应 Zircon streaming | `UE/Editor/LevelEditor/Private/SLevelEditor.cpp:1211`；`WorldBrowserComposition` ID | level manager 存在不证明 tiled world、origin rebasing 或 tile LOD UI | 世界创作/场景运行时 owner 与 Parent 决策 |
| OVW-07 | World Partition Editor | 待定：需对应的空间分区与加载 cell 真实后端 | `UE/Editor/WorldPartitionEditor/Private/WorldPartitionEditorModule.cpp:88,1160` `WorldBrowserPartitionEditor` | 未证实可直接替换成现有 Level Streaming 工作区 | 世界创作；缺 cell 编辑、加载状态、runtime streaming 策略与持久化来源 |
| OVW-08 | Content Bundle Browser | 待定：需 content bundle/外部数据层资产生命周期 | 同上 `:89,1169` `ContentBundleBrowser` | 当前 asset/level 注册不足以证明 content bundle semantics | 世界创作与资源会话协调 |
| OVW-09 | Layers | 适用：Actor 分组、可见性与选择 | `UE/Editor/LevelEditor/Private/SLevelEditor.cpp:1161,1616` `LevelEditorLayerBrowser` | 现有 scene selection/visibility 与相机 render layer 是不同概念；未确认 editor layer model | 世界创作；不可把 render mask 当 Actor 编辑层 |
| OVW-10 | Data Layers Outliner | 待定：是否需要运行时数据层激活/加载语义 | 同上 `:1220,1625` `LevelEditorDataLayerBrowser`；`UE/Editor/DataLayerEditor/Private/DataLayerEditorModule.cpp` | 未核实资产 schema 和运行时状态合同 | 世界创作；与普通 Layers 分开决策 |
| OVW-11 | Level Instance 编辑模式 | 适用：嵌套场景/Prefab 关联创作目标 | `UE/Editor/LevelInstanceEditor/Private/LevelInstanceEditorModeToolkit.cpp:94` `FModeToolkit::Init` | `scene/world/project_io/prefab.rs`、Prefab query 和 `extension_prefab_editor_workspace.zui` | 本会话与世界创作协调；不得把只读 prefab inspector 计为 level instance edit/commit/revert |
| OVW-12 | Actor Merge | 待定：Zircon 是否提供合并 mesh、碰撞、材质等真实资产写入 | `UE/Editor/MergeActors/Private/MergeActorsModule.cpp:26,108` `MergeActors` | 存在 mesh/import/render 并不证明 merge tool；本轮未证实生产写入链路 | 模型/资源工具会话；避免只放 Merge 按钮 |

## 其他窗口覆盖候选：日志、分析、资源与生产工具

| ID | 窗口/工具 | 适用性、理由 | 实际 UE 源码/注册依据 | Zircon 当前能力 | 建议归属及缺口 |
| --- | --- | --- | --- | --- | --- |
| OVW-13 | Output Log / console | 适用：运行/编译输出、命令与错误定位 | `UE/Developer/OutputLog/Private/OutputLogModule.cpp:40,173` `OutputLog` | `ui/host/editor_activity_log.rs`、`workbench/composites/feedback/validation_log.zui` 和共享 diagnostic row | 建议诊断会话/Parent；验证真实日志流、严重级别、搜索、尾随、清空、console 命令和焦点 |
| OVW-14 | Device Output Log | 待定：需远程/设备日志后端 | 同上 `:41,179` `DeviceOutputLog` | Desktop export 不等于存在设备日志传输 | 构建/设备/诊断 owner 决策 |
| OVW-15 | Message Log | 适用：结构化校验消息与定位 token | `UE/Developer/MessageLog/Private/MessageLogModule.cpp:158` `MessageLog` | 既有 diagnostics、通知和资产校验是候选 provider | 诊断会话；要核验消息源、分组/过滤、链接跳转及生命周期 |
| OVW-16 | Statistics / scene resource statistics | 适用：场景资产成本和性能反馈 | `UE/Editor/LevelEditor/Private/LevelEditor.cpp:158` `LevelEditorStatsViewer`；`SLevelEditor.cpp:1269` body；`UE/Editor/StatsViewer/Private/StatsViewerModule.cpp` | `host/runtime_diagnostics_body.zui`、runtime diagnostics host | 诊断会话；需逐列真实采样和排序，不用固定计数 |
| OVW-17 | Session Frontend | 待定：运行 session 发现/远程控制是否对应 Zircon | `UE/Developer/SessionFrontend/Private/SessionFrontendModule.cpp:20,63` `SessionFrontend` | runtime/PIE 数据存在，但不能推断 UE SessionServices 协议等价 | 诊断/运行宿主 owner 决策 |
| OVW-18 | Automation 测试窗口 | 适用：展示/选择受管测试结果；执行依然走 Jenkins | `UE/Developer/AutomationWindow/Private/AutomationWindowModule.cpp:23` `CreateAutomationWindow`；`TraceInsights/Private/Insights/InsightsManager.cpp:57` tab | Jenkins 有真实请求与回执；未核实 Editor UI provider 和取消/重试命令 | 测试工具会话；不能由 UI 发起全仓 Cargo 或复制源码 |
| OVW-19 | Insights Session Info / Trace Control | 适用：真实 trace 采集与身份/控制入口 | `UE/Developer/TraceInsights/Private/Insights/InsightsManager.cpp:51,59,192,1190` | runtime diagnostics 与 `modules/diagnostics/extension_perf_workspace.zui`；未验证 trace ingestion | 诊断会话；真实采样/存储格式/采集权限待对齐 |
| OVW-20 | Timing Insights | 适用：CPU/GPU/任务时序分析 | `UE/Developer/TraceInsights/Private/Insights/TimingProfiler/TimingProfilerManager.cpp:192` 注册 major tab | 当前通用性能工作区不证明有时间事件、线程轨、call tree | 诊断会话；不用虚拟火焰图充当引擎性能记录 |
| OVW-21 | Memory Insights | 待定：精确 allocation trace 与 symbol/source map provider | `UE/Developer/TraceInsights/Private/Insights/MemoryProfiler/MemoryProfilerManager.cpp:174` | 保留内存/采样数据相关代码不等于分配生命周期追踪 UI | 诊断/运行时 owner 决策 |
| OVW-22 | Asset Loading Insights | 适用：资产导入/加载失败和耗时诊断 | `UE/Developer/TraceInsights/Private/Insights/LoadingProfiler/LoadingProfilerManager.cpp:151` | 已有资产 pipeline、任务/错误模型；需要可关联 trace/event identity | 资源与诊断会话协调 |
| OVW-23 | Networking Insights | 待定：网络插件是否可提供包/连接/带宽时序 | `UE/Developer/TraceInsights/Private/Insights/NetworkingProfiler/NetworkingProfilerManager.cpp:149` | `zircon_plugins/net` 的存在不能证明分析 provider 就绪 | 网络/诊断 owner 决策 |
| OVW-24 | Reference Viewer | 适用：资产依赖定位与引用修复 | `UEPlugins/Editor/AssetManagerEditor/Source/AssetManagerEditor/Private/AssetManagerEditorModule.cpp:500,623` `ReferenceViewer` | 资产引用索引与 dependency extractor 可复用；图交互、循环/硬软引用语义待验 | 建议资源会话；本会话需要节点/类引用跳转接口 |
| OVW-25 | Size Map / Asset Audit / disk size | 适用：项目资产大小与打包分析 | 同上 `:497–501,597–623` `AssetAudit`、`AssetDiskSize`、`AssetDiskSize2`、`SizeMap` | 资产 artifact/导出 manifest 可作数据源；未验证 UI 与磁盘/共享依赖记账 | 资源/构建会话；逻辑大小不能冒充唯一物理占用 |
| OVW-26 | Derived Data Resource Usage / Cache Statistics | 适用：Zircon 自身 artifact/cache 诊断 | `UE/Editor/DerivedDataEditor/Private/DerivedDataEditor.cpp:19,20,26,34` 两个 tab | 实际 artifact cache 和 Jenkins 生命周期存在；UE DDC 协议不适用 | 资源/构建/诊断 owner；只显示真实数据，清理由 Jenkins 管理 |
| OVW-27 | Localization Dashboard | 待定：翻译表、收集、编译、发布的 authoring scope | `UE/Editor/LocalizationDashboard/Private/LocalizationDashboard.cpp:137,289,302` 主窗和 target details tab | 既有 i18n content contract 不等于可编辑 localization project backend | 设置/生产工具会话与 Parent 决策 |
| OVW-28 | Device Manager | 待定：设备发现、配对、部署后端 | `UE/Developer/DeviceManager/Private/DeviceManagerModule.cpp:19,39` `DeviceManager` | 目前 Desktop export/launch 能力不能外推 Android/console device manager | 构建/设备 owner 决策 |
| OVW-29 | UE 设备专用部署、Zen/DDC 服务管理、Kismet 内部对象工具 | 不适用：这些 UE 后端/协议不属于 Zircon；通用 UX 另作映射 | 上述 DeviceManager、DerivedDataEditor、Kismet Toolkit 的实际 provider 依赖 | 不复制 UE backend 名称与无效果操作；不因此排除 Zircon 自有构建/缓存诊断 | 不新增无效产品入口 |

## Editor 目录交叉扫描补充候选

对 `Engine/Source/Editor` 的全部 `.cpp` 做只读入口扫描，模式为 `Register(?:Nomad)?TabSpawner|FWorkflowTabFactory|FDocumentTabFactory|FModeToolkit`，实际命中 60 个第一层目录、125 个文件。这个目录分组包含框架和 `Experimental`，不是“60 个编辑器模块或窗口”的完整计数；不覆盖无这些字面模式的宏/封装、其他扩展、Developer 或 Plugins 全树。对新增实用候选继续核查了实际注册位置：

| ID | 窗口/工具 | 适用性、理由 | 实际 UE 源码/注册依据 | Zircon 当前能力 | 建议归属及缺口 |
| --- | --- | --- | --- | --- | --- |
| OVW-30 | Undo History | 适用：可见的文档事务与撤销定位，直接关联本族风险 | `UE/Editor/UndoHistoryEditor/Private/UndoHistoryEditorModule.cpp:16,24` `UndoHistory` | `zircon_editor/src/core/editing/engine/history.rs` 与 document retention/journal 存在 | Parent/诊断会话；本族提供当前文档 history context。不可把 engine 的全局 undo list 与当前资产历史混淆 |
| OVW-31 | Render Resource Viewer | 适用：纹理/缓冲/资源成本与泄漏诊断 | `UE/Editor/RenderResourceViewer/Private/RenderResourceViewerModule.cpp:16,20` `RenderResourceViewer` | RHI/graphics resource 实现及 diagnostics 为数据候选；当前未证明有逐资源采样和 source map | 渲染/诊断会话；真实资源 ID、显存大小/共享记账和生命周期待接 |
| OVW-32 | Pixel Inspector | 待定：需要实际 GBuffer/像素读取和材质归属 | `UE/Editor/PixelInspector/Private/PixelInspectorModule.cpp:154,174` `LevelEditorPixelInspector` | 正常画面 readback 不自动具备 GBuffer/材质语义 | 渲染/材质会话；实际帧、像素、色彩空间与深度/法线 provider 待定 |
| OVW-33 | Environment Light Mixer | 适用：场景环境光/天空/雾等集中创作 | `UE/Editor/EnvironmentLightingViewer/Private/EnvironmentLightingModule.cpp:45` 注册主窗 | 有场景组件、真实 IBL/环境渲染代码与 `extension_lighting_bake_workspace.zui`，但集中编辑/预览/撤销未验 | 世界创作/灯光会话；不能把静态 lighting bake demo 视为实时 mixer |
| OVW-34 | World Bookmark Browser | 适用：场景位置/状态导航；与图 bookmarks 分开 | `UE/Editor/WorldBookmark/Private/WorldBookmark/WorldBookmarkModule.cpp:37,174` `WorldBookmarkBrowser` | 有 scene/camera/selection；未确认项目化 bookmark 持久化和可加载状态合同 | Parent 主场景/世界创作；本族 graph bookmarks 保持独立 |
| OVW-35 | Struct Viewer | 适用：反射数据类型的只读发现，不等于 User Defined Struct 作者工具 | `UE/Editor/StructViewer/Private/StructViewerModule.cpp:48` `StructViewerApp` | 反射 schema/provider 现有；可编辑动态 struct schema 未确认 | 类型/数据资产/设置会话；本族使用真实类型过滤与选择 |
| OVW-36 | Sound Cue / Sound Class / Sound Submix | 待定：Audio 总体在资源工具会话分工内，但其 RT-20 把这组图形工具的能力和具体 owner 留给 Parent 确认 | `UE/Editor/AudioEditor/Private/SoundCueEditor.cpp:52,59,64,69`；`SoundClassEditor.cpp:75`；`SoundSubmixEditor.cpp:94` | `zircon_plugins/sound/editor/mixer_console.zui`、`acoustic_debug.zui`、`src/live_output/model.rs` 已存在；不证明对应 UE authoring graph/试听全通 | 资源域已盘点，图形 authoring owner 待 Parent 确认；不得把声音播放服务直接算作 Cue/Class 图作者工具 |
| OVW-37 | Font / Font Face | 适用：字体资源、fallback 与中文字形预览直接影响 UI 验收 | `UE/Editor/FontEditor/Private/FontEditor.cpp:136` 五类 tabs；`FontFaceEditor.cpp:39,46,51` preview/properties | Font 资产类型、文字渲染和 `modules/ui/extension_font_atlas_workspace.zui` 已有；字体作者工具尚未验 | UI/资源会话；真实字体资源、atlas/glyph/fallback/readiness 与修改传播要分开验 |
| OVW-38 | String Table Editor | 适用：面向本地化数据，不等于全局 Localization Dashboard | `UE/Editor/StringTableEditor/Private/StringTableEditor.cpp:30,138,144` `StringTableEditor_StringTable` | 既有 i18n/content 合同；本轮未确认可编辑 string-table 资产 schema 与引用更新 | 数据资产/设置会话；中文、长文本、key 重命名与引用迁移待验证 |
| OVW-39 | Source Control Changelists / 文件变更 | 适用：项目版本状态、提交与冲突的 UX 目标 | `UE/Editor/SourceControlWindows/Private/SourceControlWindowsModule.cpp:70,80` `SourceControlChangelists` | 项目版本操作由 Jenkins 管理；产品内 provider/授权和回执展示尚未核查 | Parent/版本工具 owner；UI 不得绕过 Jenkins 直接写 Git，也不因查看窗口推断推送授权 |
| OVW-40 | Config Editor / Device Profiles / Input Bindings | 适用：设置会话需明确是否覆盖这些关联窗口 | `UE/Editor/ConfigEditor/Private/ConfigEditorModule.cpp:38`；`UE/Editor/DeviceProfileEditor/Private/SDeviceProfileEditor.cpp`；`UE/Editor/InputBindingSettingsViewer/Private/InputBindingSettingsViewerModule.cpp` | 有配置、commands/keybinding owner，未确认 profile override 编辑器；不直接采用 UE ini/CVar provider | 设置与插件会话；保留项目/用户 override、reset、持久化和快捷键冲突合同 |
| OVW-41 | User Asset Tags 管理 | 待定：区分资源元数据标签、gameplay tags 和 UI filtering | `UE/Editor/UserAssetTagsEditor/Private/UserAssetTagsEditorModule.cpp:25,52` `ManageTagsTabId` | 现有资产索引/标签相关能力需确认 schema；不同域标签不可共用错误写命令 | 资源会话；先确定真实元数据 provider、引用/批量操作与 undo |
| OVW-42 | Gameplay Debugger 工具模式 | 适用：Actor/AI/nav 运行数据联查目标 | `UE/Editor/GameplayDebugger/Private/GameplayDebuggerToolkit.cpp:78` mode toolkit | runtime diagnostics、Navigation PIE mirror 可供数据；不等于 UE GameplayDebugger category/backchannel | AI 与诊断会话协调，具体收件待 Parent 确认；本族仅提供真实 Actor 关联定位 |
| OVW-43 | Mass Entity Debugger | 待定：Zircon ECS 调试类似需求；UE Mass provider 本身不适用 | `UE/Editor/MassEntityDebugger/Private/SMassDebugger.cpp:102–130` processors/graph/elements/archetypes/breakpoints/entities/query tabs | Zircon scene/world 的存在不能证明 Mass 数据模型或全部 ECS introspection provider | 运行时/诊断 owner 决策；不复制无来源的 entity/archetype 统计 |
| OVW-44 | NNE Model Data Editor | 待定：Zircon neural 模型检查目标可用，但 UE NNE 后端/模型格式不直接适用 | `UE/Editor/NNEEditor/Private/NNEEditorModelDataEditorToolkit.cpp:117,123` `NNEModelDataTab` | `zircon_plugins/neural/editor/src/plugin.rs` 与 runtime model_asset tests 存在；作者工具/执行图/模型数据支持仍需核实 | neural/资源 owner 与 Parent 决策 |
| OVW-45 | Editor Utility Blueprint / Utility Widget | 待定：需要真实编辑器脚本执行范围、命令和资产模型 | `UE/Editor/Blutility/Private/BlutilityModule.cpp:143,148` 按资产 path 动态注册 utility tabs | 有脚本和 UI 作者能力不代表可以安全执行任意编辑器操作；当前未证实 utility document provider | 本会话/Widget/脚本 owner 协调；不得把 runtime Widget Graph 冒充 editor utility |

扫描中 `EditorFramework`、`EditorWidgets`、MainFrame 与 UnrealEd 的通用 Toolkit/模式是共享基础，归 Parent；不能每个命中都新增一个窗口。PListEditor、ZenEditor、StorageServerWidgets、VirtualizationEditor 的 UE 平台/服务后端不直接适用（见 OVW-29）；CSVtoSVG 是离线工具候选，当前没有 Zircon 产品窗口需求，不直接纳入本族实现。ClothPainter、SequenceRecorder、Font 等具体收件仍待对账；Audio 已有资源域记录，不把 overview 缺行当作无人负责的证明。

## 已有会话收件范围对账

下表来自实际落盘记录和本次用户分工，不是跨会话已读回执，也不表示对应窗口已实现或验收。

| 会话 | 已声明范围及依据 | 与本次候选的关系 |
| --- | --- | --- |
| 资源 `01a16d53-60db-7d02-b03e-0042f277783d` | [资源工具记录](08-resource-tools.md)、[交接](08-resource-tools-parent-handoff.md)：Browser/Drawer、选择/导入、Static Mesh、Texture、Audio、DataTable、独立 Curve | OVW-36 已被 RT-20 盘点为待定，图形能力和 owner 待 Parent 确认；OVW-24/25/35/37/38/41 是否含完整工具仍需逐窗确认。早期 overview 把 Mesh/Texture 归材质不能覆盖用户本次分工。 |
| Widget `01a16d52-d54d-7ac0-b0d5-21864f383a1e` | [Widget 记录](widget-ui.md)：明确仅 Widget/UI，不接管 AI | ABW-22 归此域；OVW-45 的 Editor Utility Widget 需要单独确认执行/资产模型。 |
| AI `01a16d53-0693-72e2-bd58-d34a65b938fa` | [AI 交接](ai-parent-handoff.md)：Behavior Tree/Blackboard、感知、EQS/StateTree 待定项 | 不与 Widget 合并记账；OVW-03/04/42 的导航和调试跨域入口仍需 Parent 指定。 |
| 动画 `01a16d51-741b-73b3-a61a-2189cd38f81e` | [动画记录](animation.md)：Sequence/Montage/BlendSpace、AnimGraph/StateMachine、时间轴及相关曲线 | ABW-15 复用其时间轴能力，ABW-21 由其与角色域协调；不复制成 Actor 图后端。 |
| 角色 `01a16d52-6b1e-7543-82f0-4f05ca92ad58` | [角色交接](character-skeleton-parent-handoff.md)：Skeleton/Skeletal Mesh/Physics Asset/Persona、IK、Control Rig | 与 ABW-21 有接口交叉；ClothPainter 等单项仍需收件确认。 |
| 材质 `01a16d50-6c7f-7370-853d-4581184b66a1` | [材质记录](material-family.md)：材质域，不承接 Texture/Static Mesh/Niagara/动画 | OVW-32 Pixel Inspector 属候选依赖，不能据材质范围直接认定已收件。 |
| 设置 `01a16d53-33db-7fb2-8f59-11118c3ec933` | [设置交接](settings-plugins-parent-handoff.md)：设置与插件 | OVW-27/38/40 等关联工具需窗口级确认，不把通用设置页面当完整 Config/Device Profile 作者工具。 |
| Parent `01a16d4e-0491-7353-84fa-038bd7706546` | [主工作台交接](main-workbench-handoff.md)：主窗口、共享库/主题、服务和桌面协调 | 世界创作、日志、分析、版本与构建诊断候选尚无完整窗口级收件表；由 Parent 统筹归属，不自动纳入本族。 |

最新 [Parent 分工记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md) 已确认九个实际 chat ID；材质牵头图表面，动画负责 timeline/curve/transport，Parent 负责跨域合同/公共集成。本表历史域记录若尚写“等待分配”，以最新 Parent 分配为准；不据此推定那些窗口已完成。Actor 本域需求见 [共享合同](actor-blueprint-shared-contracts.md)。

## 覆盖对账与后续盘点边界

本轮从 Kismet 的 application modes、15 个 tab 常量、图/时间轴 document factories、SubobjectEditor 类/实例实现、独立 enum/struct Toolkit、Class Viewer、Level Blueprint 与创建类对话框，以及上列 world/tooling registrations 进行交叉核查，已形成 25 个本族条目和 45 个覆盖候选。条目可以组合多个同一 Toolkit 的状态，不是“实现了 70 个窗口”。

材质/纹理/模型、Niagara、动画/Persona、Widget/AI、设置/插件、Content Browser、Data/Curve 工具已被套件 overview 提及。本会话记录接口交叉处，不重复接管。UE Plugins 下的全部实验/行业/平台工具、每个版本化 layout 和所有 dockable clone 尚未穷尽，不能据此声称完成 UE 全窗口枚举。

Parent 随后已提供 [45 项窗口级覆盖对账](parent-window-coverage.md)，接续以上早期建议，不覆盖其源码/适用性和未完成记录。世界创作、导航、日志/分析、设备/版本等没有独立实现聊天的条目由 Parent 保留待接续；资源收件 Audio 全族、Font/Font Face、String Table/类型与资源分析，设置收件配置/本地化，AI 收件 Gameplay Debugger。OVW-45 Editor Utility 的脚本/图合同由 Actor 牵头，Widget 协作 UI 内容，Parent 对接 VM/宿主。上述是收件分工，不表示真实后端、设计或窗口验收已完成。

下一轮按实际注册/Toolkit 继续核查，保留既有稳定条目 ID；不要把盘点或归属行移入已完成统计。本域当前五板和业务数据实例仅覆盖设计切片，不能据此减少 ABW-01…25 或 OVW 候选的实际能力缺口。
