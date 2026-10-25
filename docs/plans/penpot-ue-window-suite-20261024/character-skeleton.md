# 角色、骨骼与骨骼网格窗口族

当前接续第38节 Physics r05：为现有RagdollProfile补受校验约束的TOML写出接口，新增9项编解码回归定义，0执行；当前Physics源17项。r04骨骼typed key/整批候选、Skeleton r05/32项和Penpot rev76保留。共享缺件不阻止域源码工作，goal active；原Document/CAS/history/save、真实预览、正式导出、normal EXE/实际150%DPI与整族验收仍open。 TreeRow公共源码复核、正式导出身份及UTF-8版本链见第39节；compact尚未发布。

同轮后续installed已由同owner的Prefab限定资格交付推进为 `20b3bf85…` / entry=`4e95a18e…`，当前11 sealed和角色两条test纯parse/18哈希复核通过；glTF条件及source preparation/guard/wrapper不变，原28698和首版指针漂移失败保留，见[第21节后续记录](character-skeleton-design.md#21-gltf限定入口交付与角色owner三路径审阅)。当前capture-ready=false，准备请求仍仅排队，无Rust执行或时段。

本轮最初入口交付（28698历史，当前20b3见顶部）：已核对当时的immutable capsule `28698b9e…`/entry `0b064521…` 的当前指针及11个sealed成员，6份helper native与Home archive字节一致；两条glTF精确 **test** 配方在新pinned client纯parse通过。角色owner已实际审核同身份三ownedPaths review-only输入及18项源码/依赖哈希，全部匹配。旧“route不可用/当前dev”在第17/20节为历史；现在缺正常准入、当前完整writer/受管资源回执和Parent有界稳定时段，capture-ready仍false，没有Cargo。见[设计第21节](character-skeleton-design.md#21-gltf限定入口交付与角色owner三路径审阅)。

前轮托管对账（第20节历史）：原Jenkins owner已实际读取r4，非queue ACK；本域核对其reconciliation SHA及当时14份源码/依赖、guard/format回执全部匹配。当时Parent执行范围记dev，r4显式test提案保留，二者未运行；当时缺glTF route/三路径envelope/完整source准备。该观察保留，当前状态由顶部和第21节更新，不重跑旧query或盲重提。

最新公共合同接续：13:55Z只读API确认Shared仍rev1/4主件，本域rev22/linked library及原60个跨文件实例有效。第四个PropertyEditorRow的 `value` 为必填单控件槽，当前已发布4项中无一被该槽接受；已给出既有Field发布及只读覆盖合同的最小依赖，下一步为真实Head/Parent两行，不因Euler/Axis未交付而阻止这两个只读字段，见[设计第19节](character-skeleton-design.md#19-四个公共主件的source与必填value槽核对)。没有新增空槽板、私有Field或未发布UUID。

最新设计接续：实际文件已到 **rev22/1538对象/102flow**，链接真实公共库 rev1，在两张新板上复用60个跨文件原生实例，包含48个真实Wolf joint和Head过滤的9个匹配/祖先行；旧详情组件增加一个同文件实例。保存、关闭、fresh重开及原生Copy检查完成，原317个非根对象和公共库未改。8张图已逐图复核；官方PNG请求实际404、ZUI source-owner/产品接线仍未完成，见[设计第18节](character-skeleton-design.md#18-共享原生实例的完整源骨骼树与head内容设计)。

CPU源码接续：按Parent明确的两文件交接，原glTF plugin已补meshopt白名单并保留真实未支持Draco反例；3文件最终guard/格式通过，角色两正例原字节未变。第17节当时安装的cdd client仅准材质，glTF被拒绝的原证据保留；新28698路由和角色输入审阅见第21节。两正例和lib反例仍未运行。

最新预览接续：已给Parent准备typed locator/subject generation及Runtime提交pose generation两份未应用diff，精确9个公共目标源保持原样；source/pose/frame/picking/资源生命周期最小合同见[设计第16节](character-skeleton-design.md#16-typed-subject已提交pose与真实frame最小适配)。仅语法/应用检查，尚无生产backend、Jenkins或正常EXE验证，不计通过。

九族归属对账：依据 Parent 最新用户交接及 [overview](overview.md)，Static Mesh、Texture 独立资产工具归资源会话 `01a16d53-60db-7d02-b03e-0042f277783d`；本域仍负责 Skeletal Mesh、Skeleton、Physics Asset，Retarget/Control Rig 边界不变。公共 PreviewSession/GPU image/media 与最后消费者 teardown 由 Parent 统筹，对齐资源域 RT-S06，不复制 backend。跨域材质/资源关联使用真实 registry/Toolkit 与焦点返回；源码和实际 Wolf/Knight 关联核查见[设计第15节](character-skeleton-design.md#15-九族归属与真实资源关联打开合同)。

本记录接续 [套件总览](overview.md) 与 [公共组件约定](component-reuse.md)。本会话为 `01a16d52-6b1e-7543-82f0-4f05ca92ad58`，父会话为 `01a16d4e-0491-7353-84fa-038bd7706546`。任务 ID 为 `ue-character-skeleton-20261024`。用户于 2026-10-24 明确本会话负责 Skeleton、Skeletal Mesh、Persona 角色相关窗口、Physics Asset、重定向及适用 Control Rig/角色工具。

当前已形成源码盘点、接线前核查、[具体设计与实际 Penpot 记录](character-skeleton-design.md) 及 [按 owner 划分的共享消费合同](character-skeleton-shared-contracts.md)。第14节的详情三状态及原 Retarget 基线均保留；第18节增加真实共享实例骨骼树，替代“当前没有公共实例”的旧观察。**完整 UE 设计、共享样式传播、原资产导出、产品实现及窗口族验收均未完成**。第11节Table geometry/PNG与早期TreeRow插件导入失败保持；新树通过真实共享库原生Copy接续，不代表旧导入器已修复。CS-G10最小修复和静态guard/格式保持；新托管限定test入口与同身份三路径审阅已到位，待当前写者/资源和Parent稳定时段后正常准入，实际Rust仍未执行，见第21节。服务和桌面由Parent统筹，仅用隔离API/headless，不启停服务、不占桌面；下表“适用”及设计保存不代表产品实现。

## 参考身份与读取边界

- Zircon 初始 HEAD：`75a76ee963792d5a316f738519e966937f770036`，实际 checkout 为 `E:/Git/ZirconEngine`，初始分支 `main`。存在大量其他会话未提交修改；本会话不接管这些修改，不创建源码副本、工作树或额外主题。
- UE 参考树：`E:/Git/ZirconEngine/dev/UnrealEngine`，只读。其根没有 `.git`，不能编造 UE commit。实际 `Engine/Build/Build.version` 为 `6.0.0`、Changelist/CompatibleChangelist 均为 `0`、BranchName 为 `UE5`；不以该字符串假定版本为 UE 5.x。
- Build.version SHA-256：`3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6`。主要布局文件哈希见本任务产物清单。读取的 `dev`、UE 根、Engine、Source、Editor、Plugins、Plugins/Animation 祖先目录均未发现 `AGENTS.md`；未修改参考树。
- UI 依据为 [UE/Penpot 保真流程](../../../.codex/skills/zircon-project-skills/zr-ui-layout-reference/references/ue-penpot-fidelity.md)。通用区域关系沿用套件已有 ReactBits App Shell / Forms 映射；具体布局、样式和命令以以下 UE 源码为准。

以下源码表中 `Editor/` 相对于 `dev/UnrealEngine/Engine/Source/Editor/`，`Animation/` 相对于 `dev/UnrealEngine/Engine/Plugins/Animation/`，`Experimental/` 相对于 `dev/UnrealEngine/Engine/Plugins/Experimental/`。每个源码入口均须按实际注册继续追踪，不能仅凭 `PersonaTabs.h` 中声明的常量认定窗口存在。

## Toolkit、工厂和布局盘点

| 标识 | 窗口/模式 | UE 实际入口 | Zircon 适用性及原因 | 当前实现结论 |
| --- | --- | --- | --- | --- |
| CS-SK | Skeleton Editor | `Editor/SkeletonEditor/Private/SkeletonEditor.cpp`、`SkeletonEditorMode.cpp` | 适用：真实 Skeleton 层级、引用姿势、Socket 和关联网格的作者入口 | 未找到独立已接入的 Skeleton 产品窗口；待 Penpot 和数据接线 |
| CS-SM | Skeletal Mesh Editor | `Editor/SkeletalMeshEditor/Private/SkeletalMeshEditor.cpp`、`SkeletalMeshEditorMode.cpp` | 适用：蒙皮网格、骨骼、Morph、材质/LOD、关联 Skeleton/Physics 资源 | 当前运行时支持蒙皮不代表有该编辑窗口 |
| CS-PA | Physics Asset Editor | `Editor/PhysicsAssetEditor/Private/PhysicsAssetEditor.cpp`、`PhysicsAssetEditorMode.cpp` | 适用：Zircon 已有 Ragdoll profile、骨骼形状及约束数据 | 既有 `physics/editor/ragdoll_profile.zui` 四个主体仍为 `Space` |
| CS-IK | IK Rig Editor | `Animation/IKRig/Source/IKRigEditor/Private/RigEditor/IKRigToolkit.cpp`、`IKRigMode.cpp` | 适用目标：骨骼、目标与 IK 求解；Zircon 已有 TwoBone IK/LookAt 数学能力 | 未查证 IK Rig 资产、求解器栈与 Editor 的实际接线，保持未实现 |
| CS-RT | IK Retargeter | `Animation/IKRig/Source/IKRigEditor/Private/RetargetEditor/IKRetargetEditor.cpp`、`IKRetargetApplicationMode.cpp` | 适用目标：用户明确要求重定向、源/目标骨架与姿势对照 | 既有 Retarget extension 是固定展示数据和预览动作，不能算真实重定向 |
| CS-CR | Control Rig Editor | `Animation/ControlRig/Source/ControlRigEditor/Private/Editor/ControlRigEditorMode.cpp` | 部分适用、具体能力待定：骨骼/控制层级、局部/全局姿势、IK 操作可映射；完整 RigVM 不能从动画图能力推定 | 既有 Control Rig extension 未证明真实控制/姿势输出 |
| CS-MR | Modular Control Rig | 同上 `FModularRigEditorMode` / legacy 分支及 `ModularRig*TabSummoner.cpp` | 待定：未查证 Zircon 模块 Rig 资产、连接器和执行队列后端；不默认复制空工具 | 无本轮设计或实现 |
| CS-MODEL | Skeletal Mesh Modeling Tools | `Animation/SkeletalMeshModelingTools/Source/SkeletalMeshModelingTools/Private/SkeletalMeshModelingToolsEditorMode.cpp`、`SkeletalMeshModelingToolsEditorModeToolkit.cpp`、`SkeletalMeshModelingToolsCommands.cpp` | 待定：权重/Morph/骨骼编辑与本域相关；几何工具、皮肤权重持久化和撤销需逐项核实 | 不把运行时 Morph 渲染等同 Morph 作者工具 |
| CS-CFX | Character FX 工具框架/示例 | `Experimental/CharacterFXEditor/BaseCharacterFXEditor/Source/BaseCharacterFXEditor/Private/BaseCharacterFXEditorToolkit.cpp` 及 Example 对应 Toolkit | 示例编辑器不适用产品交付；实际角色 FX 工具待定，需要真实功能 owner 和资产类型 | 不将 UE 示例插件照搬成 Zircon 空窗口 |
| CS-DNA | MetaHuman/DNA、RigLogic 专属工具 | `Animation/DNACalib`、`Animation/RigLogic`、`Animation/GeneSplicer` | 当前不适用：尚无已核实的 Zircon DNA 资产/执行与产品需求；未来新增支持需重审 | 排除不表示 UE 没有这些工具 |
| CS-DEFORM | Deformer Graph / MLDeformer / Cloth | `Animation/DeformerGraph`、`Animation/MLDeformer`；Mesh Toolbox 扩展 | 待定并交接相应变形/布料 owner：需实际后端与范围确认；不以本次骨骼复刻顺带新建系统 | 不计本窗口族完成 |

Control Rig 源码同时包含 `WITH_RIGVMLEGACYEDITOR` 和当前模式。当前普通模式布局为 `Standalone_ControlRigEditMode_Layout_v1.10`，legacy 为 `v1.9`；当前 Modular 为 `Standalone_ModularRigEditMode_Layout_v1.3`，legacy 为 `v1.5`。设计不得误将前一个 legacy 分支当成唯一目标。

## 从注册发现的页签和关联窗口

Persona 工厂权威入口为 `Editor/Persona/Private/TabSpawners.cpp`、`PersonaModule.cpp`、`Public/PersonaTabs.h`；SkeletonTree 独立工厂为 `Editor/SkeletonEditor/Private/SkeletonTreeSummoner.cpp`。下列注册页签均需有设计和实际能力状态，关闭态页签也不能从清单漏掉。

| 标识 | 页签/弹层 | 注册或 Slate 依据 | 判定与依赖 |
| --- | --- | --- | --- |
| CS-TREE | Skeleton Tree、骨骼/Socket/虚拟骨/附加资源行 | `SkeletonEditor/Private/SSkeletonTree.cpp`、`SSkeletonTreeRow.cpp`、`SkeletonTree*Item.cpp` | 适用核心；读真实骨骼，选择与视口/Details 双向同步；Socket/虚拟骨需要真实可保存模型 |
| CS-VIEW | Persona Viewport 1–4 | `Persona/Private/TabSpawners.cpp` 的 viewport summoner，`PersonaModule.cpp` 的注册 | 第一视口适用核心；其他三个待定，需多预览 session/相机与 GPU 输出能力；不能复制静态图 |
| CS-DETAIL | Details：Bone、Socket、Body、Constraint | Persona Details 工厂；`Persona/Private/SkeletalMeshSocketDetails.cpp`；Physics editor 选择回调 | 适用核心；共享属性行承载真实对象与事务，不能以字符串伪造数值修改 |
| CS-ASSET | Asset Details / Mesh Asset Properties | `CreateAssetDetailsTabFactory`，Skeletal Mesh/ Skeleton mode | 适用；与骨骼选择 Details 分开，保留资源身份、保存及 dirty 状态 |
| CS-PREVIEW | Preview Scene Settings、Preview Mesh/Animation 选择 | `CreateAdvancedPreviewSceneTabFactory`；`Persona/Private/PersonaToolkit.cpp` | 适用；真实环境、相机、资源就绪、播放状态和错误反馈 |
| CS-LEGACY-RT | Skeleton Retargeting / Retarget Sources | `CreateRetargetSourcesTabFactory`；`Persona/Private/SRetargetSources.cpp`、`SRetargetSourceWindow.cpp` | 适用目标，旧 Skeleton retarget source 与现代 IK Retargeter 分开；引用姿势/来源存储需核实 |
| CS-CURVES | Curves 元数据、Curve Debugger | `CreateCurveMetadataEditorTabFactory`、`CreateCurveViewerTabFactory` | 角色窗口需要展示/选择；共享曲线编辑器与动画会话协调，不重复创建 |
| CS-NOTIFY | Skeleton Animation Notifies | `CreateAnimNotifiesTabFactory` | 适用共享页签；实际通知资产/编辑行为由动画 owner 交接 |
| CS-SLOT | Anim Slot Manager / Slot Names | `CreateSkeletonSlotNamesTabFactory` | 适用共享页签；需真实 slot/group 数据与 Montage owner 协作 |
| CS-BROWSER | 兼容动画资产浏览、关联资产跳转 | `CreateAnimationAssetBrowserTabFactory`、Persona toolkit | 适用；从真实 catalog 筛选骨架兼容资源，跳转须真实打开同一资源身份 |
| CS-FIND | Find/Replace | `CreateAnimAssetFindReplaceTabFactory` | 适用共享目标，需骨骼/通知/曲线引用和批处理事务；无后端时保持不可验收 |
| CS-MORPH | Morph Targets | `CreateMorphTargetTabFactory`、Skeletal Mesh mode | 适用；真实 Morph 权重与 GPU 网格变化；作者删除/改名/镜像另需持久化能力 |
| CS-MAP | Mesh Control Rig Mapping | `SkeletalMeshEditorMode.cpp` 的 `FMeshControllerMappingTabSummoner`；`SkeletalMeshEditor/Private/SControlRigMappingWindow.cpp` | 待定；这是 Mesh 的映射窗口，不等于完整 Control Rig Editor |
| CS-TOOLBOX | Skeletal Mesh Toolbox | `CreatePersonaToolboxTabFactory`、SkeletalMeshModelingTools Toolkit | 待定具体工具；必须验证 Accept/Cancel、事务与真实网格/权重改变 |
| CS-BLEND-PICK | Blend Profile/Mask 选择、创建、改名、删除 | `SkeletonEditor/Private/SBlendProfilePicker.cpp`、`SSkeletonTree.cpp` 的 New/BlendProfile menu；`Persona/Private/BlendProfileCustomization.cpp` | 适用角色共享数据；具体时间/权重 profile、mask 数据和动画混合 owner 待核实 |
| CS-BONE-PICK | Virtual Bone 目标选择、骨骼映射/选择弹层 | `SSkeletonTree.cpp` 的 `CreateVirtualBoneMenu`；`Persona/Private/SBoneMappingBase.cpp` | 适用；仅合法目标可提交，取消/弹层焦点返回；虚拟骨与导入骨骼的删除规则不同 |
| CS-SOCKET-MENU | Socket 类型过滤、Attach Asset 子菜单、重命名 | `SSkeletonTree.cpp`、`SkeletonTreeSocketItem.cpp`、`Persona/Private/SkeletalMeshSocketDetails.cpp` | 适用；真实 Socket/附加资源引用是前置条件，需分别验证菜单和详情提交 |
| CS-MORPH-RENAME | Rename Morph Target 弹窗 | `Persona/Private/SRenameMorphTargetDialog.cpp` | 适用网格作者目标；名称冲突、确认/取消、焦点返回和保存/撤销待接线 |
| CS-SLOT-REF | Slot 名称引用检查/改名关联窗口 | `Persona/Private/SSkeletonSlotNames.cpp` 创建 `SSlotNameReferenceWindow` | 适用共享关联窗口；与 Montage/资产引用 owner 协作，不把本地字符串改名等同引用更新 |
| CS-PH-GRAPH | Physics Asset Graph | `PhysicsAssetEditor/Private/PhysicsAssetGraph/PhysicsAssetGraphSummoner.cpp` | 适用：体/约束拓扑与选择；不能用装饰连线证明约束编辑 |
| CS-PH-PROFILE | Physical Animation / Constraint Profiles | `PhysicsAssetEditorProfilesSummoner.cpp` | 适用目标：Ragdoll/约束 profile；需区分 Zircon 现有单 profile 与 UE profile 切换能力 |
| CS-PH-TOOLS | Physics Tools / body generation | `PhysicsAssetEditorToolsSummoner.cpp`、`PhysicsAssetEditorActions.cpp` | 适用：真实生成胶囊/球/盒与约束，形状参数编辑、重生成、模拟；Convex/TaperedCapsule 按后端逐项待定 |
| CS-IK-SKEL | IK Rig Hierarchy / Solver Stack / Retarget Chains / Asset Browser / Output Log | `IKRigMode.cpp`、对应 `IKRig*TabSummoner.cpp` | 适用目标；需要 goals、chain、solver 执行与诊断后端，不能拿固定四链表验收 |
| CS-RT-TABS | Retarget Hierarchy / Op Stack / Asset Browser / Overrides / Variables / Output Log | `IKRetargetApplicationMode.cpp`、对应 `IKRetarget*TabSummoner.cpp` | 适用目标；本地参考是 Op Stack 版本，不能只按旧版 Chain Mapping 惯例绘制 |
| CS-RT-POSE | Edit Retarget Pose | `IKRigEditor/Private/RetargetEditor/IKRetargetEditPoseMode.cpp`、`IKRetargetCommands.cpp` | 适用核心；源/目标姿势、选择及撤销作用于实际骨架，真实对照预览 |
| CS-CR-TABS | Rig Hierarchy / Curve Container / Validation / Animation Attributes / Dependency Graph | `ControlRigEditorMode.cpp` 及对应 `Rig*TabSummoner.cpp` | 角色相关部分适用目标；当前是否有等效数据待定；不可因注册组件便称可用 |
| CS-RIGVM | RigVM Graph / Explorer / Execution Stack / Compiler Results / Find References / Details | 同上模式及继承的 `FRigVMNewEditorMode` / RigVM TabSummoner | 待定：Zircon 动画图与 RigVM 不同，需引擎 owner 确认等效能力；不复制 UE VM 专属调试空壳 |
| CS-MOD-TABS | Module Hierarchy / Module Assets / Event Queue | `ModularRigModelTabSummoner.cpp`、`RigModuleAssetBrowserTabSummoner.cpp`、`ModularRigEventQueueTabSummoner.cpp` | 待定：依赖模块 Rig 模型、连接器、执行调度 |
| CS-POSE | Pose 编辑器、Pose Watch Manager | `Persona/Private/SPoseEditor.cpp`、`SPoseWatchManager.cpp`；相关动画/Blueprint 工厂 | 真实姿势查看/选择归本域；Pose 资产作者与 Watch 调试协议由动画 owner 协调，避免双重所有权 |
| CS-ANIM | Sequence/Montage/BlendSpace/AnimBlueprint Preview、Asset Override、时间轴 | Persona 工厂及对应动画 Toolkit | 不属于本会话主实现范围，交给动画会话；本域保留关联跳转和兼容骨架过滤验收 |

`PersonaTabs.h` 另有 MirrorSetup、DebugHistory、PreviewManager 等声明；在本轮列出的实际 mode 工厂中尚未证明其注册/可用，不能把声明或历史常量算成已完成窗口。补充搜索 `Engine/Source/Editor` 与 `Engine/Plugins` 的 C++ 定义/使用，相关 `FPersonaTabs::*ID` 只命中 Persona 常量定义；Editor/Animation 范围的对应字符串搜索也只命中常量。保留待定状态，不在默认菜单增加不能证明可打开的窗口。

Control Rig 动态/继承工厂补充核查：

- `Engine/Plugins/Runtime/RigVM/Source/RigVMEditor/Private/Editor/RigVMNewEditorMode.cpp` 实际注册 Execution Stack、Explorer、Details、Compiler Results、Find References。Graph 是 `RigVMNewEditor.cpp` 在 `DocumentManager` 中注册的 `FRigVMGraphEditorTabSummoner` 文档工厂，不能只查 mode 的 `RegisterFactory` 而漏掉。
- 当前 `FControlRigEditorMode` 继承该 base 的同一 TabFactories，再加入 Rig Hierarchy、Curves、Validation、Attributes、Dependency Graph、Persona Viewport/Preview。当前 Modular mode 使用 `Standalone_ModularRigEditMode_Layout_v1.3`，明确禁止 Execution Stack 的 tab spawner；不能推断全部 base 页签在 Modular 模式可用。
- `RigOverrideListTabSummoner.cpp` 实际定义 `OverrideList` / “Active Overrides”，body 要求 `IsModularRig()` 并创建 `SOverrideListWidget`。但对 `Engine/Plugins/Animation` 与 `Engine/Source/Editor` 的 C++/头文件搜索只有类声明、定义和 include，没有构造/注册使用；base 注册列表也无此项。因此仍为“工厂存在、实际可打开入口未证明”，不算普通/Modular 模式已注册窗口。

## 已确认的 UE 区域关系

这些是源码系数，不是截图测量值，也不是最终产品的固定坐标。实际分配应继承共享 Dock、最小尺寸和文本度量。

| 窗口 | 布局身份 | 区域关系 |
| --- | --- | --- |
| Skeleton | `Standalone_SkeletonEditor_Layout_v1.5` | 水平 Skeleton Tree / 中央视口区 / 右侧页签区 `0.2 / 0.6 / 0.2`；右上 Details/Curve Debugger/Preview，右下 Notify/Asset Browser/Curves/Slots；Retarget 与 FindReplace 默认关闭 |
| Skeletal Mesh | `Standalone_SkeletalMeshEditor_Layout_v5` | 水平 Toolbox / 视口区 / 右侧 `0.2 / 0.6 / 0.2`；右上 Skeleton/Morph/Curves，右下 Asset Details/Details/Preview；不能直接复用 Skeleton 的左右顺序 |
| Physics Asset | `Standalone_PhysicsAssetEditor_Layout_v5.3` | 左侧 Skeleton Tree 上、Physics Graph 下；中间真实视口；右上 Details/Preview，下方 Tools/Profiles；左上下 `0.6 / 0.4`，右上下 `0.6 / 0.4` |
| IK Rig | `Standalone_IKRigEditor_Layout_v1.127` | 左层级/求解器栈，中视口/日志，右 Details/Preview 与 Browser/Retarget Chains；源码水平系数 `0.2 / 0.8 / 0.2`，须归一化而非当成百分比 |
| IK Retargeter | `Standalone_IKRetargetEditor_Layout_v1.022` | 左上 OpStack/Hierarchy、左下 Browser/Overrides/Variables；中间预览和底部日志；右侧 Details/Preview；内层左右 `0.2 / 0.8`，右侧与主区 `0.2 / 0.8` |
| Control Rig 当前普通模式 | `Standalone_ControlRigEditMode_Layout_v1.10` | 左上预览、左下层级/执行栈/曲线/Explorer；中上 Graph、下 Compiler/Find；右 Details/Preview/Attributes；水平 `0.2 / 0.6 / 0.2` |

窄窗口的设计应在共享 Dock 和抽屉能力内保留访问所有页签的路径，记录折叠规则并验证焦点返回；目前没有 Penpot 尺寸设计或实际运行证据，不预先宣称 640×520 可用。层级区、详情区、资产区各有独立滚动 owner；普通面板使用流式约束，不以绝对坐标拼图。

## 真实能力与原始资产

| 所有者/入口 | 已证实内容 | 尚缺的产品证据 |
| --- | --- | --- |
| `zircon_runtime/src/core/framework/animation/asset/skeleton.rs` | `AnimationSkeletonAsset` 存名字、parent index、TRS，并验证父索引及环；二进制编解码 | Editor 树从实际资源读取、稳定骨骼选择、事务、保存/重开 |
| `zircon_runtime/src/core/framework/animation/pose_snapshot.rs`、`pose_bone.rs`；`animation/manager/pose.rs` | 真实姿势和 manager 数据入口 | 同一 skeleton 身份/代际的局部与全局姿势投影、GPU 帧关联 |
| `zircon_runtime/src/graphics/scene/renderer/mesh/build/skinning.rs`、skinned WGSL | 运行时真实蒙皮路径 | 从本窗口编辑/播放到可见像素和骨骼 gizmo 的实际验证 |
| `zircon_editor/src/ui/animation_editor/capabilities.rs` | 当前公开表明确 `animation.preview.runtime = false`、`animation.compiler.semantic = false`；现有能力测试也断言它们不可用 | 必须实现真实后端与产品接线才可改变能力声明，不能只把标志改为 true |
| `zircon_editor/src/ui/preview_scene/preview_scene.rs` | 已有泛型 `PreviewSceneBackend` 生命周期/subject/playback/focus 合同 | 在 Editor 与 animation editor 插件源码中只找到 `tests/cases.rs` 的 `FakePreviewBackend` 实现；抽象合同不能算已存在生产预览后端 |
| `zircon_plugins/animation/runtime/tests/animation_ik_contract.rs` | 真实 TwoBone IK、LookAt API 与已有契约用例 | 本轮未运行这些测试；不能推定已有 IK Rig/Control Rig 图作者能力 |
| `zircon_plugins/physics/editor/src/ragdoll_profile_editor.rs` | `generate_initial_ragdoll_profile` 从实际传入骨架生成胶囊体、质量和约束默认值，并调用 profile 验证 | 实际骨架传入、选择体/约束、gizmo、模拟、持久化与撤销 |
| `zircon_plugins/physics/editor/ragdoll_profile.zui` | 现有稳定控制 ID：`RagdollProfileRoot`、`RagdollProfileToolbar`、`RagdollSkeletonTree`、`RagdollProfilePreview`、`RagdollBoneProperties` | 四个主体组件是 `Space`；仅资产存在，不是 Physics Asset 编辑器完成 |
| `zircon_runtime/src/core/framework/physics/skeletal_pose.rs` | 实体级 `SkeletalPoseTargets` / `SimulatedPoseFeed`，真实逐骨骼 transform 与 weight | 可视化编辑与模拟帧的窗口接线 |
| `zircon_editor/assets/ui/editor/animation/extension_retarget_workspace.zui` | 既有节点、组件引用及事件身份 | 固定 `SK_Mannequin/SK_Robot` 和四条链；缺真实 source/target/chain/pose 会话；保留 ID 再迁移，不能另建同功能平行资产 |
| `zircon_editor/assets/ui/editor/animation/extension_control_rig_workspace.zui` | 既有节点、引用与事件身份 | 固定表格/预览动作未证明实际控制、编译或姿势输出 |
| `zircon_editor/assets/ui/editor/workbench/primitives/data/preview_viewport.zui` | `WorkbenchPreviewViewport` 当前是 `Image`，指向 `animation_mannequin_viewport.svg` | **静态角色图，明确不满足本任务实时预览**；共享组件由父会话统筹 |
| `zircon_editor/src/ui/retained/dispatch/workbench/tests/preview_features.rs` | 默认关闭 mock 导航、显式 opt-in 预览标签及回退测试已存在 | 本轮未运行；维持 preview gate，不能通过去掉标签将占位功能包装成产品 |

本轮尚未选择或载入可验收的实际角色资源。测试夹具只能用于契约测试；产品验收须记录导入源、catalog UUID/locator、骨架和网格哈希、骨骼/顶点数、姿势来源，并证明实际运行帧使用同一资源。不得从 UE 参考树拷贝角色资源来替代数据接线。

## 给父会话的公共依赖提案

以下是具体能力提案，尚未发送成功或获父会话确认；不改公共 catalog/tokens/Starship，不新建重复组件。

| 提案 | 复用身份/所有者 | 本域所需合同 | 交付判据 |
| --- | --- | --- | --- |
| CS-D01 | `WorkbenchTreeRow`；现有虚拟列表/选择 owner | 骨架资源身份+代际+bone index/path 构成真实行身份，展开/过滤/多选，选择与焦点分离；socket/virtual bone 类型明确 | 真实骨架首尾滚动、搜索保留祖先、空结果、中文/长名、选中骨骼同步到 Details/viewport |
| CS-D02 | `WorkbenchPreviewViewport` 与现有 Editor PreviewScene/runtime scene media owner | 用真实 GPU 外部图像/预览 session 承载 skinned mesh，保留资源就绪/错误/设备失效及姿势代际；支持骨骼/Socket/碰撞体 picking、相机聚焦 | 不带静态 mannequin 作为运行画面；pose/mesh 修改前后可见像素变化且源资源可追溯 |
| CS-D03 | `WorkbenchPropertyEditorRow`、NumberField、Dropdown；原事务/焦点 owner | TRS 局部/全局模式、Socket 父骨骼、质量/形状/约束轴、提交/取消/重置；drag 合并为一次 undo | 输入校验、Escape 取消、提交/撤销/重做、保存重开与真实资源一致 |
| CS-D04 | `DocumentHost`、`UiHostWindow`、Dock、菜单、catalog、全局 tokens/Starship；父会话 | 独立资源 Toolkit 的页签注册、布局持久化、dirty/关闭、关联资产跳转与快捷键 scope | 复用现有 Dock/command/history；三尺寸访问路径与弹层焦点返回有证据 |
| CS-D05 | Penpot native components/ownership/export plugin；父会话 | 指定共享文件/page/library/component ID、可编辑实例、共享样式 token 和原始 owner export allowlist | 保存 ACK、关闭、fresh 重开、组件实例关联不丢；导出保留原节点 ID、事件、绑定和 imports |

原始 Retarget、Control Rig、Ragdoll 资产均是迁移候选，不把新建 `unreal/windows/*` 同功能文件当成原始资产导出落地。Skeleton/Skeletal Mesh 没有发现可直接承担完整独立 Toolkit 的现有窗口资产，新增 owner 必须先与父会话确认公共注册依赖。

## 本窗口族必须覆盖的操作

骨骼树命令依据 `SkeletonTreeCommands.cpp`、`SSkeletonTree.cpp` 的 command list/context menu/selection/PostUndo，而非猜测快捷键。焦点在字段内时，编辑键与复制/粘贴先由字段消费；菜单关闭与取消回到发起控件，选择和键盘焦点分别取证。

| 操作组 | UE 目标与真实效果 | 本轮状态 |
| --- | --- | --- |
| 骨骼选择 | 单选、范围/多选、树/视口互选、搜索/过滤、展开、Focus Camera `F`；Details 读同一骨骼 | 待实现/验证 |
| 姿势 | Reset selected `Ctrl+I`，Reset all `Ctrl+Shift+I`；引用姿势与预览姿势区分；真实局部/全局 TRS | 待真实 model/transaction/viewport 接线 |
| Socket | Add、rename、delete、骨架/网格 Socket 区别、复制 `Ctrl+C`、粘贴 `Ctrl+V`、向选中骨骼粘贴 `Ctrl+Shift+V`、附加资源、提升/还原 | 当前 SkeletonAsset 未提供 Socket 字段；需查明持久化 owner，不能仅增 UI 字段 |
| Socket/虚拟骨/附加资源删除 | `Platform_Delete`、类型与选择合法性、一个可撤销事务、重开结果；`SSkeletonTree::CanDeleteSelectedRows` 不允许直接删除普通导入骨骼 | 待实现；普通骨骼结构编辑需独立建模工具/事务，不误套 Delete 路由，不让树/pose 索引悬空 |
| 网格预览 | 相机轨道/平移/缩放、F 聚焦、播放/暂停/参考姿势、骨骼/Socket 显示、Morph 权重、材质与 LOD | 待真实 mesh/pose/GPU 接线 |
| Physics | 体/约束/图三方选择、胶囊/球/盒参数和 gizmo、约束轴/限位、生成/重生成、模拟全部 `Alt+I` / 选中 `Alt+Shift+I`、停止恢复编辑姿势 | 待真实 profile 编辑和运行模拟取证 |
| 重定向 | 源/目标资产选择、兼容性、真实 chain/goal/op stack、编辑 retarget pose、播放对照、导出/保存结果、失败日志与撤销 | 固定四链条不能验收；后端待接线 |
| Control Rig | 层级/控制选择、局部/全局变换、IK 目标、真实求解/输出 pose、验证诊断 | 适用子集待确定，完整 RigVM/Modular 能力待定 |
| 关联与历史 | Browse 同一 catalog 资源、打开 Skeleton/Mesh/Physics/Animation、每文档 dirty/save/undo/redo、关闭未保存文档的实际策略 | 复用 DocumentHost/原 history；父会话依赖待交接 |

## 证据与验收安排

使用既有 `tools/penpot/apps/zircon-zui-plugin/src/model.ts` 的 `LayoutReviewCaseMessage`，不创造并行协议。case 的 `sourcePath` 必须是本次实际加载的原始 `.zui`；未知 Penpot/组件/EXE 身份保留为空，不能写成候选值或复用父窗口的通过状态。

| 矩阵 | 要求 | 当前证据 |
| --- | --- | --- |
| 尺寸/DPI | 1280×800、900×620、640×520 @ 96 DPI；1280×800 @ 实际 144 DPI，物理 1920×1200 | 全部待采集；既有主窗口 125% 不计本域通过 |
| 通用状态 | default、empty、selected、focused、open、focus-return、scroll start/end、long-en、long-zh | 全部未执行 |
| 角色状态 | 实际骨骼树首尾、真实选中骨骼 TRS、Socket 操作、pose 改变、Morph 改变、Physics body/constraint 编辑与模拟、retarget pose/source-target | 全部未执行，不使用静态角色图片代替 |
| Penpot 身份 | 文件/page/board、共享 library/component/version、实例关联、保存 ACK、关闭、fresh 重开、官方 PNG/ZUI | 当前rev63/2839对象/1业务主件；125外部linked roots及2159共享source refs已核对；Physics三尺寸空态保存重开见第29节，Shared rev5保持；Field/官方PNG/ZUI与Engine闭环仍缺 |
| 导出证明 | 一次真实 Penpot 样式编辑→导出原始 owner→产品实际加载→同状态截图变化，完整依赖哈希 | 未执行；不能以手改 ZUI 冒充 Penpot 导出 |
| 原生入口 | 普通启动的 Engine EXE、PID/birth、EXE/DLL/资源闭包、`GetDpiForWindow`/Winit/client extent、normal-frame nonce/generation/source、成功 present/readback、ACK/release | 待产品窗口接线与排队；保留现有严格原生协议 |
| 对照 | 同内容/尺寸/DPI 的官方 Penpot PNG与原生全窗口图、并排图、diff、每图复核 | 已有Penpot操作图与逐图记录；尚无正常EXE原图、官方导出和双端比较；关键边界/文本行框误差目标 ≤1 DIP，尚未测量 |

只有资源就绪并且捕获有效才可 `passed`；双端比较及逐图审阅通过才可 `accepted`。Preview image/fixture/offscreen-only、pending receipt、队列 ACK、静态检查均不能提升这些状态。

## 当前差异账本

下列所有行均为 open；图像栏未采集属于明确的证据缺口。每行在实施后按实际窗口/状态/尺寸/DPI拆分，追加真实 Penpot/Editor 原图、diff 与审阅结论，不覆盖历史失败。

| ID | 参考/目标 | 实际差异与证据 | 原因/依赖 | 方案与状态 |
| --- | --- | --- | --- | --- |
| CS-G01 | Skeleton/SkeletalMesh mode + SSkeletonTree | 已保存关闭fresh重开真实48骨骼树与Head过滤9行的共享native Copy，详情旧组件保持；无独立产品窗口或运行预览，见设计第18节 | 缺本域Toolkit、真实collection/选择/Details绑定、作者/事务；CS-D01/03/04 | 官方源owner/export与真实资产浏览/姿势闭环接续，Socket作者仍open |
| CS-G02 | Persona 实时 skinned preview | 既有 preview component 明确指向 mannequin SVG；无真实帧证据 | 已确认静态组件，CS-D02 父 owner | 真实 PreviewScene/GPU 图像与骨骼 picking，保留静态组件仅作已标注历史展示；open |
| CS-G03 | Socket/virtual bone/attached asset | 当前 `AnimationSkeletonAsset` 仅 name/bones/TRS，未有 Socket 作者证据；图像缺失 | 持久化/引用 owner 未核实，需 Runtime/资产 owner | 记录资源关系及事务再设计；不可只加无效按钮；open |
| CS-G04 | PhysicsAsset mode、graph、tools/profiles | Ragdoll ZUI 仍为四个 `Space`，无 body/constraint 编辑或模拟原图 | Toolkit/asset type 已注册，但 create operation 只 OpenView，未调用真实 generator；已有 PIE collider overlay/picking/mirror，未证明 preview session 和 profile 作者事务接线 | 复用原注册/profile/overlay 与稳定 ID，补真实生成/选择/参数/模拟，不能把开视图算生成；open |
| CS-G05 | IKRig/IKRetargetApplicationMode | 固定 Mannequin/Robot/四条链与 UE OpStack 目标不同；图像缺失 | 没有证明真实 retarget 资产/求解/保存 | 先核实后端，保留完整目标设计与缺失项；open |
| CS-G06 | 当前 ControlRig/Modular mode | 固定展示无法证明控制、IK pose、编译；图像缺失 | 部分数学 IK 可用，RigVM/模块 Rig 未证明 | 提交适用子集与后端提案，不能假装支持完整 UE RigVM；open/待后续设计决策 |
| CS-G07 | 全部模式的字体/图标/flow/交互 | 原 Retarget 基线及只读骨骼详情组件/三实例已保存关闭重开；目标 UE 设计、官方 PNG、同状态原生图和逐图差值未完成 | Shared最新rev5/4主件/34颜色/3字体；本域118文字覆盖metadata已修复并fresh重开，下一次原生库同步待证；官方导出/Field/PNG依赖与桌面时段仍open，见设计第26节 | 复用共享catalog/tokens/Starship与动画/材质owner；专用组件不替代公共基础实例/样式；实际1 DIP门槛；open |
| CS-G08 | Jenkins 托管检查与画面 | 保留历史 recovery-pending；新增角色CPU回归，静态检查通过，但原request pre-submit因5项外域AI源码漂移拒绝，原query authorization pending；无Cargo启动 | runtime capability ready、nativeLifecycleVerified=false，不构成本域执行回执；原请求与修订候选哈希分开保留 | 沿原identity对账；不代修服务或本地构建；open |
| CS-G09 | Persona 真实 PreviewScene / 编译 | 当前 capability 明确 runtime preview/semantic compiler 不可用；PreviewScene 仅抽象合同和 Fake backend；无图像 | 已核实为生产接线缺口，不只是更换图片 URL；CS-D02 与动画/运行时 owner | 接真实 secondary preview session、subject、playback 和 GPU 呈现；保持 false 至真实验证，open |
| CS-G10 | 候选真实 GLB 的导入前置策略 | 原遗漏已按Parent交接补meshopt白名单；Draco负例仍精确拒绝；三源静态guard/格式保持 | 原Jenkins owner已交两条限定test入口，原同身份三路径/18输入不变；当前准入、writer/资源和执行回执仍缺，历史capsule观察保留于第21节 | 未重提Cargo、两CPU正例及lib反例未运行；GPU/preview ready继续open |

## 接续与生命周期

产物清单位于 `E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/artifacts.json`。本域已创作四份规范文档、任务 metadata、隔离 authoring consumer、真实 joint 草案、实际 save 回执、浏览器截图与 CPU profile；自有 browser/临时静态 host 均已退出。没有源码树副本、Rust 编译产物、服务重启或共享桌面输入。浏览器临时目录仍登记保留，实际释放 **0 字节**，任务生命周期未关闭；不清理他人产物或修改 Parent 清单。

具体设计已补齐六类主要窗口的 flow 拓扑、三尺寸/真实150%规则、稳定选择与文档/事务边界、原资产身份迁移、命令焦点、真实输入候选及 LayoutReviewCase 设计。Wolf 48-joint 源 JSON 的父链/局部 TRS、四元数有限性与范数及14条动画名称已核对；它与 knight 的压缩 weights/inverse-bind/animation accessor 仍需 meshopt 解码，实际 importer/运行预览未验证。简单单 buffer 数值读取失败、实际 Skeleton 导入失败及其诊断均保留，不能当作已就绪资源。

实际 Penpot file/page 已分配；下一项依赖是公共组件/样式具体身份、动画/材质共享合同交接、桌面时段及本域 Jenkins 托管路径。接续时先核对同一记录和源哈希，再按 Penpot 可编辑设计→保存/关闭/重开→原始资产导出→真实数据接线→聚焦托管检查→原生矩阵/双端逐图审阅推进；本域整体 goal 保持未完成。

## 当前原生同步/覆盖证据（rev54，接续设计第26节）

Shared当前实读rev5、原4 masters/34 colors/3 typographies，Button main已有真实editor.surface.1引用；不再以历史无style作为当前状态。本域控制板`ebad071c-29d4-8010-8008-d734c39ad699`真实验证UPDATE/颜色覆盖/reset/Undo。rev39曾把94原骨骼标签变成Tree item，已原生Undo恢复；根因是本域早期只写content-group而缺text-content-text。6个原生编辑确认该细分标记，112个余项经rev53官方change协议只改touched；当前rev54/118个业务caption均有文字保护，保存关闭fresh重开hash58e0e1d2一致。Shared/公共源未改；修复后的下一次真实库UPDATE尚待验证。

Reset清除了控制板按钮尺寸覆盖（77.333→240）并令文字被后绘制Search背景遮住；保留失败布局，不当成仅颜色reset通过。Undo不还原library syncedAt，81文本positionData与7图标flip归一化等残余单列。当前119 linked roots/2013非root source refs，2132合计；早期计数历史保留。原Field、官方样式/PNG/ZUI导出、640末尾滚动/真实骨骼网格姿势预览/EXE/150%DPI仍open。

详见[完整本轮交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-library-sync-reset-r01/parent-handoff.json)及[逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-library-sync-reset-r01/visual-review.json)。r03九源指纹保持，Editor18与原glTF3项均0执行；不追加原glTF scope。未启停服务/占桌面/提交编译或冻结，全部自有进程退出、runtime临时文件0 B、释放0 B，整族passed/accepted=false。

## 当前骨骼定位投影（r04，24项定义未执行）

在既有只读 SkeletonReference 中新增 checked reveal：按资源 active version、revision 与原始 joint slot 验证目标；返回根到父的祖先键，或明确的 FilteredOut。它与树行共用原有字面子串筛选规则，不按同名骨骼或可见行号替代目标。该结果只表示筛选包含关系；展开、实体化、scrollIntoView、选择和焦点仍由共享 host 完成。

本轮仅改本域5路径（含2个新文件），Parent editing/mod.rs 保持 da2c4c13 原字节。当前10个域文件加 Parent Root 共11路径，回归由18增为24项静态定义，0执行。当前源和独立 Editor 范围以[唯一后继指针](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-model/current-source-revision.json)及[本轮合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-reveal/consumer-contract.json)为准；旧9/12/18项记录均保留历史，不可覆盖当前源。原glTF身份与3路径未追加。实际产品接线、托管执行、骨骼/网格/姿势预览、正常EXE及DPI证据仍open，整族passed/accepted=false。

## 当前控制板复核（rev57）

原Reset重叠失败证据保留；仅在受控板`ebad071c-29d4-8010-8008-d734c39ad699`用原生Column/Fit content改为源尺寸纵排，3控件各约240×32、间隔4、行容器高104。一次真实Reset overrides将该板Filter覆盖恢复为源Button及editor.surface.1；不是UE横向工具栏布局通过。rev57保存、关闭、fresh重开及独立API内容hash783a5f86一致。193个变化均限该板，全部其它对象、1业务component和Shared保持；119外部root/2013后代sourceRef仍有效。

[逐图复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-flow-r02/visual-review.json)记录5原生浏览器图。新probe底部行为说明和原640详情Scale/source path仍裁切，未计尺寸完整性通过。本helper未留到原生HTTP save ACK，不能把0匹配记录当0写入；实际持久化读回与raw ACK覆盖分开，见[观察缺口](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-flow-r02/save-observer-gap.json)。本轮未运行Rust/plugin或正常EXE，r04十一源指纹与24项未执行计划保持，passed/accepted=false。

## 当前 Physics Asset 空态设计（rev63）

新增1280×800、900×620、640×520三张原生可编辑板，对应UE Physics Asset v5.3的左层级/图表、中预览、右详情/预览设置及工具/配置区域。当前文件rev63/2839对象/1业务主件，125外部root/2159共享source refs；Shared rev5保持。保存关闭fresh重开与关闭browser后独立API的dataHash均05f076e1c5d694fb6e2652e64686fe4f16486ab1e0c4eca391f886d1c1bfa6a9。原有对象未改。

仅覆盖“未打开配置”的设计状态；原ragdoll_profile.zui六节点/四Space及原hash未改，真实打开/生成、碰撞体/约束编辑、undo、preview/EXE/DPI与官方导出仍未接入。8图均实际打开，首次画布重叠失败和500移动回执保留，后用原生Y位置检查器整理；窄Search内部240溢出仍open。详见[设计第29节](character-skeleton-design.md#29-physics-asset未选择配置的三尺寸设计rev63)和[逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-physics-empty-design-r01/visual-review.json)。r04十一源/Editor24定义、原glTF三源本轮匹配；Rust仍0执行，没有新请求或freeze。整族passed/accepted=false。


## 当前 Physics 配置编辑候选（r01；尚无产品消费者）

在实际Physics Runtime与Editor原owner中新增配置候选API：按配置内完整bone_path修改Shape、Mass、BodyOffset、Constraint、BlendWeight，保持源配置不变及骨骼原始顺序；整批失败不返回部分候选，无净变化返回None。Runtime的RagdollProfile.validate复用现有manager的约束数值与复合子体变换规则，同时保留原profile更严格的Box正尺寸限制。此前可解析的倒置限位、非法drive/threshold及非单位复合子体缩放现在会明确报错，新增公开InvalidConstraint错误变体；真实消费者的兼容性仍须托管检查。

本轮8个源码路径含3个新增文件，format及同源change-guard均exit0、0错误/警告/抑制；Runtime6和PhysicsEditor12回归仅已定义、执行0，与骨骼r04的24项分开。[精确源交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/source-handoff.json)、[消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/consumer-contract.json)及[独立配方草案](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/regression-plan.json)保留边界。尚无UI/命令、host版本准入、undo/save/reopen、真实profile/secondary preview或正常EXE证明；原.zui、注册/manifest/lock及唯一backend均未改。没有新Jenkins身份/提交、query、capture或freeze，原glTF三ownedPath不扩展。passed/accepted/captureReady=false。


## 当前 Retarget 无资源状态设计（rev66）

新增三尺寸可编辑板，按实际UE IKRetargetApplicationMode v1.022的嵌套splitter安排左侧操作/层级与资源、中央视口/输出、右侧详情/预览。状态为source/target/definition/frame=None、0条映射；不含旧三rig/四链/Ready模板。旧导入板及原.zui/31节点/29control IDs/15events保持历史，原产品默认Ready问题仍未修复。当前file/page rev66/2941对象/1业务主件，新增102节点及6个Shared Button原生linked实例，Shared rev5保持；131外部root/2171个有效sourceRef，新增207处真实样式引用。

三尺寸与fresh重开共4张浏览器原图均已打开复核；新中文提示可读，640的资源溢出caption只是设计目标，没有真实menu行为。保存HTTP200原始请求/响应、关闭/fresh重开及独立API内容hash ff0c39c4533f4c8ef4f4ddd018609185635658c7408d9316d96cf1493ea5ef81 保留在[当前交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-retarget-capability-state-r01/parent-handoff.json)与[逐图复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-retarget-capability-state-r01/visual-review.json)。Zircon现有Preview/Apply只返回Not implemented (preview)，没有实际求解/ready consumer；不将此空状态板算真实预览。原Jenkins owner的r02→r04版本回执03cc2f4e已核对，24项与Physics6＋12均0运行，glTF原身份不扩。正常EXE/真实150%DPI/官方导出/双端1DIP及整族passed/accepted继续open。


## Physics 源后继 r02：已安装骨骼到生成器

承接第30节与Retarget rev66设计，本轮新增Physics Editor资源适配和10项回归定义。generate_ragdoll_profile_from_skeleton通过现有ResourceManager.snapshot_expected读取指定版本AnimationSkeletonAsset，保留原joint slot及显式作者路径关联，再调用原generate_initial_ragdoll_profile。原生成器、既有Runtime校验、Skeleton r04十一源、原glTF三ownedPath及公共root均保持；没有新增store/clock/backend或改原.zui。

当前Physics后继为10源（本轮2处接线修改、2新文件），静态格式/guard均0；待执行Runtime6＋Editor22，0执行。精确后继、版本与消费职责见[源交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-skeleton-source-r02/source-handoff.json)。现create仍仅OpenView；作者路径producer、当前文档事务/保存/undo、同session entity↔profilebone、真实secondary mesh/pose/物理帧、正式导出与正常EXE/DPI仍open。原glTF身份/18输入不扩展，未新建执行身份或发起capture/freeze；passed/accepted/captureReady=false。


## 33. Control Rig真实源层级与未配置状态（rev72）

依据UE新编辑器Standalone_ControlRigEditMode_Layout_v1.10实际源码，新增1280×800、900×620、640×520三张可编辑板，左右中权重0.2/0.6/0.2，左上/下0.5/0.5，中上/下0.8/0.2。真实Knight_Head源为node27/mesh4/skin4，head原joint slot14、parent chest slot3；只有GLB JSON源投影，RigDefinition/compiled Rig/runtime pose/frame均为空，不把源骨骼树当持久Rig或有效预览。

本轮426个新增节点含21只原native linked实例（15 TreeRow、3 SearchInput、3 Button）；保存关闭fresh重开及独立API确认rev72/3367对象，152个外部root/2498个有效源引用，Shared rev5未改。12张原图已逐图打开；自身缩放说明重排并刷新三处派生文字缓存后可读，但900/640选中head名称可见宽度均0，共享TreeRow C6仍阻断窄尺寸验收。[逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-rig-source-state-r01/visual-review.json)与[交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-rig-source-state-r01/parent-handoff.json)保留首版、HTTP500失败及修正证据。

原ControlRig .zui的31节点/29ID/15绑定保持，无新decoder、Rig solver、graph surface、backend或Field。33个保护源及9个参考输入hash保持，沿用对应Physics r02与Skeleton r04同源静态资料；原glTF请求/3 ownedPaths不扩。无新Jenkins身份/query/capture/freeze/Cargo或桌面；正式导出/真实消费者、正常EXE/实际150%DPI/双端1DIP仍open，passed/accepted/captureReady=false。


## 34. SkinN Data与原始骨骼槽位映射（r05）

新增GltfSkinReference领域源码，按caller提供的Skin Data active version调用现有snapshot_expected；关联明确的SkeletonReference及导入器已写出的skeleton_asset、按skin.joints原顺序的节点/骨名和逆绑定引用。key与node index/locator双向查找保留原joint slot和资源ABA身份，显示名只作一致性检查，不取首skin、过滤行或推导registry UUID。

当前15源（14域＋原Parent root），8项新回归使原Editor过滤器共32项定义、0执行；格式与guard同源0错误/警告，原24个测试体完整保留。[源码交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-skin/source-handoff.json)及[消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-skin/consumer-contract.json)列出精确hash/差异。此为glTF源映射，不加载mesh/逆绑定或产生pose/frame；SceneMesh现未保存node.skin()、多资源publication/ready/lease及当前pick准入仍由Parent接续。26个参考/保护输入保持，Physics r02与原glTF三路径/身份不扩，无新query/capture/freeze/Cargo、服务或桌面。Penpot仍仅最近rev72实证，窄TreeRow裁切、正式导出及正常EXE/150%DPI/整族验收均open。


## 35. Physics 世界平移到 body offset（r03）

prepare_ragdoll_body_translations 读取真实 RagdollProfile、每个目标的实际 bone_to_world 与 world_delta，复用 checked affine inverse，将世界增量按向量转换到骨骼局部空间，再一次调用 prepare_ragdoll_profile_edits。actor 平移不参与向量变换；仅修改 body_offset.translation，保留旋转、缩放、shape、mass、constraint、blend 和骨骼顺序。整批失败不改源；空批次或净零变化返回 None。

UE依据为 PhysicsAssetEditorEditMode.cpp 的 StartTracking/InputDelta：停止模拟后编辑，并通过原事务提交。该切片只覆盖 Zircon 现有每骨单 body offset 的平移；UE aggregate primitives、旋转/缩放 gizmo、CoM 和网格拟合仍缺。r03为12源、8项新增定义，相关 Editor 作者回归30项、Runtime作者回归6项均未执行；不是整个包测试总量。格式和源码guard通过只绑定r03当时字节，后继源以第37节为准。

[原r03交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-body-translation-r03/source-handoff.json)与[消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-body-translation-r03/consumer-contract.json)保留。原Jenkins的8项配方交接不等于正式收件或执行；glTF及Skeleton Editor身份保持独立。没有新增store、clock或backend。

## 36. TreeRow 窄板原生布局修正（rev76）

角色 file b4a1e3ce-9e6a-81fe-8008-d69d60adb48e/page b4a1e3ce-9e6a-81fe-8008-d69d60adb48f 的三张 Control Rig 源状态板中，15个既有 native linked TreeRow 已通过原生 flow override 重新布局。保留 component/file/shapeRef、child顺序、业务main及原metadata/guard；不detach、不复制共享main、不改Shared。Shared当次实读rev5/4主件/34颜色/3文字样式，hash259ddb2aec9c0a16f3a36755fa4a0a991b96116ac66401b24f230a2ea8e12132保持。

API的rev73只保存属性，首次fresh读回未证明reflow。后经原生inspector修改gap 6→7→6取得真实布局，再依据children反向z顺序将15行设为row-reverse，保存至rev76。1280×800、900×620、640×520关闭并fresh重开，6张原图逐图打开。图标left12/宽16，caption left34.00000048；选中head字形可见宽29.1875，caption宽依次168.39999952、92.39999952、40.39999952，修正900/640原宽0。3367对象中的105处变化均在15个copy子树内。

[逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-character-tree-overrides-r01/visual-review-r08.json)、[布局合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-character-tree-overrides-r01/current-layout-contract-r08.json)及[640实图前后差异](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-character-tree-overrides-r01/penpot-before-after-640.png)保留。中断于rev漂移的geometry helper未写入手工坐标，失败诊断原样保留。浏览器77/107/145%画布缩放及DPR1不等于Windows实际DPI。

当前本地native flow与官方TreeRow painter/导出合同仍存在差异：[source-contract-audit.json](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-character-tree-overrides-r01/source-contract-audit.json)记录15处caption几何与旧metadata guard不符，以及无expander的content变体仍预留18px的具体源位置；这是源码算术检查，未执行官方导出。已交原public owner；不自行重签guard、不写公共源或.zui。640根行在viewport顶部裁切、真实首尾scroll/focus、骨骼glyph、SearchInput fit、Field、同步/reset、正式导出/产品加载/EXE/DPI仍open。

## 37. Physics typed key 与整批编辑候选（r04）

本轮在原Physics Editor内新增binding.rs：RagdollSkeletonBoneKey由已安装骨骼生成候选提供，保留ResourceActiveVersionKey、原revision、原joint slot和完整作者路径。prepare_bound_ragdoll_profile_edits及prepare_bound_ragdoll_body_translations先用现有snapshot_expected重核真实skeleton载荷，再逐项检查key与source路径/slot，最后复用原批量编辑或世界平移函数。重复属性编辑按手势顺序，空或净零返回None；任何错误不发布局部结果。候选仅持私有before/after和来源版本，validate_current拒绝骨骼替换/ABA及profile任意内容变化。

此处没有伪造profile资源UUID或修订号，也没有新文档store/history/clock/backend。当前profile尚无已发布的通用宿主文档载体；完整值相等不能证明文档ABA安全，snapshot读锁在返回后不保留。Parent仍须在原document/source/revision/edit lease下做资源准入和最后CAS，一次history操作、原save token保存、undo/redo及focus-return。真实作者路径producer、同secondary session的entity↔profilebone/frame/pose generation与共同资源生命周期仍是明确依赖，不由本候选授权当前pick。

11项新增回归定义覆盖重复显示名与原slot、整批顺序及不变字段、no-op、异资源key、同版本不同路径映射、profile缺骨、无效profile、等值载荷替换、同ID同revision重建、任意profile变化及多骨affine平移/重复目标。相关作者Editor定义共41项、Runtime6项均0执行；Skeleton Editor仍32项，三类身份不混用。

本轮scoped rustfmt和源码guard通过，0错误/警告/抑制；静态证明不等于Rust运行。现行53172128… capsule的真实parser函数已在内存检查精确Physics单包配方，返回build_scope_policy，因为当前nested plugin白名单不含Physics Editor；未dispatch、QueryOnly或启动Cargo。精确声明为zircon_plugin_physics_editor/defaults off/features[]/locked/profile test/jobs1/lib/filter ragdoll_bound_edit_tests::/threads1，物理E:/cargo-targets。交原Jenkins owner补受限资格，不借glTF、Editor或旧8项输入身份，不自行改parser或扩大包。

[本轮源码交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/source-handoff.json)、[宿主消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/consumer-contract.json)、[精确测试计划](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/test-plan.json)与[入口检查](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/route-scope-check.json)分别记录当前源及未完成门。r03原12源中仅两处域root接线后继，其余10源保持；manifest/lock/plugin注册/原ZUI/Parent root/Skeleton r05保持。未创建源码副本、执行身份或九root冻结窗口。goal active，passed/accepted/normalExe=false。

文档修复说明：上轮第35/36节和顶部摘要在PowerShell到Python管道中丢失非ASCII字符。此轮以真实r03与Penpot r08证据重写可读内容，旧回执和前像SHA保留，不把历史静态回执改签为本轮结果。后继写入前已验证UTF-8码点，避免再次以问号替换中文。

后续继续推进域内真实适配；本族全部窗口、交互、真实预览及双端验收范围不缩小。

## 38. Physics Profile 持久化编码（r05）

当前Runtime已有RagdollProfile::from_toml与完整作者校验，新增codec.rs提供to_toml：先复用同一validate，再以既有toml1.1.2/serde schema写出字符串。RagdollProfileEncodeError区分Profile准入失败与Toml序列化失败，保留原错误source。不改RagdollProfileError既有变体、字段/schema、原读取器、manifest/lock或capability；不写文件、不发布资源、不建立第二store/clock/history。

宿主可在候选准备阶段对candidate.after().to_toml()做编码预检，但不能据此标记已保存。实际保存必须从原Document取得真实已提交revision与原save token，编码该revision的完整profile，使用原持久化流程写入；仅在保存成功且token仍匹配时标记该revision已保存。编辑或资源变化、编码/写盘失败都须保留dirty和原history。原document/source/revision/lease的一次CAS与undo/redo、同secondary session实体/骨骼/pose/frame关联仍由Parent集成；此API不提供这些权限。

新增9项Runtime编解码回归定义：既有稀疏TOML默认值、中文/转义/长作者路径和父后子前顺序、全部8种既有碰撞形状及嵌套compound与UUID/url配对、稀疏轴与全部drive字段、Some(0)与None及锁轴区别、有限浮点边界/TRS/负零、不合法profile原错误、legacy三轴数组和root/bone未知字段拒绝。测试仅用内存单元数据调用真实codec，不注册生产夹具或宣称真实角色/资源ready。相关Runtime作者定义由6增至15，Editor作者41项、Skeleton Editor32项不变，全部仍0执行。

同源scoped rustfmt、4源码及4文档guard、增量patch反向check和diff检查记录在[本轮交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-codec-r05/source-handoff.json)。当前17源为r04十四源中一个Runtime mod后继、其余13源保持，另显式纳入Runtime lib接线与2新文件；r03/r04旧交接及旧过滤器输入原字节保留。托管最小新范围为zircon_plugin_physics_runtime/defaults off/features[]/locked/profile test/jobs1/lib/filter skeletal::codec_tests::/threads1，物理E:/cargo-targets；不启用backend-jolt，不借glTF、Skeleton或Editor11项身份。现行53172128…真实parser对此Runtime单包配方仍返回build_scope_policy，零dispatch/Cargo，限定资格交原Jenkins owner接续正常prepare/intake。

编码保留当前typed schema能表达的数据；注释、格式和已被宽松nested decoder丢弃的字段无法从DTO恢复。静态源码确认root/bone拒绝未知字段，而共享PhysicsColliderShape、PhysicsJointConstraintMetadata与PhysicsJointDrive没有deny_unknown_fields。原始文档完整保存须选择严格拒绝未知nested字段，或由原Document保存未建模原文；不以writer存在宣称整份原始文本无损。[三处共享小补丁提案](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-codec-r05/strict-nested-fields-proposal.json)及精确diff已通过非修改apply --check；共享源保持，兼容性决策与其独立回归待Parent/共享owner评审。

本轮没有写Penpot、原ZUI/公共主题/控件/Shared、桌面或服务，也没有发出全局冻结时段或执行身份。正常受管准备沿任务授权继续，不要求人类补grant；真实source/resource/pool/cache/native与Rust/产品证据仍分别pending，goal active，passed/accepted=false。

## 39. TreeRow 公共复核后的当前边界

[公共源码审阅](<E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/character-tree-content-lane-r01/handoff.md>)已消费，细节见[具体设计第39节](character-skeleton-design.md)。归档rev76保留15个linked root和90个直接part源引用，但有15处caption guard、75处inactive-part visibility及15处allocation parent差异。native与插件content都保留disclosure槽；content-compact仍是未发布提案，不能用插件单端公式或Reset迁移。

三张业务板缺正式asset envelope/guard，局部可读设计不等于整板可导出。公共审阅所列乱码SHA属于r03历史；r04修复→r05后继→本轮当前字节的四条链已核验，现四文档连续问号/U+FFFD均为0。原失败/源码/图像记录保持。本轮只更新本域解释与静态文档证据，不执行新Jenkins/request/host/desktop动作；goal active，正式导出/产品验收仍open。
