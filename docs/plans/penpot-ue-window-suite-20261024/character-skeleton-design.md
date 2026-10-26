# 角色与骨骼窗口族：具体设计与实际 Penpot 记录

当前接续第38节 Physics r05：为现有RagdollProfile补受校验约束的TOML写出接口，新增9项编解码回归定义，0执行；当前Physics源17项。r04骨骼typed key/整批候选、Skeleton r05/32项和Penpot rev76保留。共享缺件不阻止域源码工作，goal active；原Document/CAS/history/save、真实预览、正式导出、normal EXE/实际150%DPI与整族验收仍open。 TreeRow公共源码复核、正式导出身份及UTF-8版本链见第39节；compact尚未发布。

Penpot file `b4a1e3ce-9e6a-81fe-8008-d69d60adb48e` / page `b4a1e3ce-9e6a-81fe-8008-d69d60adb48f` 已分配并接续。以下为第18节当时的认证回读，保留历史：**rev22/vern0、1538对象/102flow/1本域组件**，新增60个跨文件原生Copy和1个原有详情组件Copy；公共库现为rev1/isShared=true/4组件，但colors/typographies仍0。身份、保存关闭fresh重开及8图复核见第18节。第11～17节保持各自历史观察和失败，rev18及“公共库rev0”不再作为当前状态。原Retarget及其rev5子树哈希、本域原317个非根对象保持。生产资源句柄、EXE指纹、正式ZUI回写和原生截图仍未取得；候选命令/case均为设计标识，所有产品 `passed/accepted=false`。

## 1. 实现入口与依赖边界

| 范围 | 当前实际 owner | 本域下一步 | 共享依赖 |
| --- | --- | --- | --- |
| Skeleton / Skeletal Mesh Toolkit | 尚未核实到完整独立窗口 ZUI；已有骨架资产、animation session、通用 DocumentHost | 先在指定 Penpot 形成独立文档窗口设计，再给 Parent 提交 Toolkit 注册与资产路径；此刻不新建空壳窗口 | CS-D01/02/03/04/05 |
| Physics Asset | `zircon_plugins/physics/editor/ragdoll_profile.zui`；profile generator 与 runtime | 复用原 asset ID `physics.ragdoll_profile` 和六个原节点，接真正的 profile 编辑 | CS-D01/02/03/04/05 |
| IK Rig / Retarget | `zircon_editor/assets/ui/editor/animation/extension_retarget_workspace.zui` | 保留原资产 owner，设计真实层级/OpStack/pose 工作区；后端能力不足的功能保持未完成 | CS-D01/02/03/04/05；真实 retarget 资产/求解器 |
| Control Rig | `zircon_editor/assets/ui/editor/animation/extension_control_rig_workspace.zui` | 实际控制层级、IK goal、输出 pose 的适用子集；完整 RigVM、Modular Rig 继续待定 | CS-D01/02/03/04/05；真实控制图/求解/持久化 |

Runtime world、资产解码、动画求值、GPU skinning 与物理执行由原 Runtime/插件 owner 负责。Editor 持有作者文档、选择/焦点、相机工具、gizmo、菜单、撤销和视图状态。正常 Engine EXE 及其主循环仍由现有 App/host 创建，不为本域增加第二个执行程序或离屏验收通道。边界依据项目架构 baseline 与 Runtime/Editor 收束计划，不把临时树行、像素截图或 UI 字符串当成权威模型。

## 2. Flow 布局与共享样式

所有窗口使用同一套 `UiHostWindow` / `DocumentHost` / Dock / `MenuBar`。普通区域是纵向、横向、可伸缩和滚动 flow；只有原生 popup、gizmo 和合法 overlay 使用叠层。下述树为拟定语义结构，不能作为绝对坐标导出。

```text
共享资源 Toolkit（Vertical flow）
  MenuBar / DocumentHost 文档页签（共享 owner）
  资源工具条（Horizontal flow，溢出进入共享菜单）
  Dock 工作区（Fill，采用该 Toolkit 的 UE 区域关系）
    面板页签与 WorkbenchPanelHeader（Auto）
    面板内容（Fill，树/详情/资产列表分别拥有自己的滚动）
  文档/资源状态反馈（共享 owner，不叠盖工作区）
```

| 语义 | 复用组件/样式 | 本域约束 |
| --- | --- | --- |
| 面板标题和页签 | `WorkbenchPanelHeader`、DocumentHost/Dock | 使用 title/actions slots；标题、dirty、关闭与激活规则由共享 host 处理 |
| 树行和资产行 | `WorkbenchTreeRow`、`WorkbenchListRow`、原虚拟 collection owner | Row 的 selected、focused、ancestor-match、disabled 分开；缩进/展开箭头不能伪装成文本空格 |
| 属性 | `WorkbenchPropertyEditorRow` + `WorkbenchNumberField` / Dropdown / Field | label/value slot；TRS 轴按共享最小字段宽度和 flow 换行，不能压成不可编辑宽度 |
| 面板操作 | `WorkbenchButton`、`WorkbenchIconButton`、Dropdown | 图标使用共享 s/m/l/xl tier；有实际能力且满足 CanExecute 才可用 |
| 真实预览 | CS-D02 对现有 `WorkbenchPreviewViewport` 的生产能力提案 | 当前 SVG Image 不满足要求；不能先写一个新同名/平行组件骗过引用检查 |
| 颜色/字体 | `editor_tokens.zui`、`unreal/starship.zui`、已有状态角色 | 选中与非激活选中、焦点边、错误、禁用均引用同一共享样式；不复制十六进制色表或第二套字号 |
| 密度 | tokens 的 `controls.dense_height`、`chrome.panel_header_height`、`chrome.viewport_toolbar_height`、`density.row_height`、`density.gap_*`、`axis_value_field_*` | 使用已解析的共享 token 表达式；下列源码数值只是盘点，不能为本域再建常量 |

当前 tokens 的密集控件与行高为 28，panel header 为 30、viewport toolbar 为 28，axis field 的 min/preferred/max 为 54/62/72。骨骼树 UE 行还可含 AutoHeight inline editor，不能用固定 28 高裁掉额外内容；应由公共 TreeRow owner 提供内容测量/行度量。UE 的 16px 图标记录交 Parent 映射到共享 tier，不在本域添加数值图标大小。Roboto 主字体与中文 fallback 必须记录实际字体资源，尚无双端字体证据。

### 2.1 骨骼树

依据 `SkeletonEditor/Private/SSkeletonTree.cpp` 和 `SSkeletonTreeRow.cpp`：顶部依次为 New（Auto）、Search（Fill）、Filter（Auto），其下为 pinned commands，再是可伸缩的树。Search 聚焦时全选文本；Filter 自己拥有弹层，关闭后回到发起按钮。树的名称列包含展开箭头、类型图标和名称，额外列按 Show Bone Index / Retargeting / Blend Profile 等能力显示。

树行显示 bone、skeleton socket、mesh socket、virtual bone、attached asset 的实际类型。过滤可保留灰色祖先，或按 UE 选项隐藏祖先/展平；不能让搜索结果默默改变权威骨架关系。树只由一个虚拟化滚动 owner 控制，展开项高度须与索引测量一致。文本可省略显示并提供完整可访问名称/提示，tooltip 不替代可选择目标与键盘导航。

已核实 `components/workbench/primitives/data/tree_row.zui` 的 `WorkbenchTreeRow` 已提供 expanded/tree_depth/selected/focused 等状态和共享样式，但 root 的 min/preferred/max height 均为 dense height。优先在本域 item 的 Vertical flow 中复用该标准行，并把 UE inline editor 放在其后的 AutoHeight 内容，虚拟 collection 测量整个 item；不为每骨创建一个公共“骨骼专用 TreeRow”。若现有 collection 只能固定高度，再向 Parent 提交实际受限调用点和行度量变更，不能直接改全局行高或裁掉 inline editor。

### 2.2 各窗口区域

下表的权重来自实际 UE mode 源码；同一行各组分别在自己的 Splitter 中归一化。Zircon 的窄尺寸折叠为待 Penpot 验证的适配设计，不冒充 UE 截图测得的布局。

| Toolkit / UE layout | 首选工作区 flow | 默认页签与访问路径 |
| --- | --- | --- |
| Skeleton / `Standalone_SkeletonEditor_Layout_v1.5` | 水平 Tree : Viewport : Right = .2 : .6 : .2；Right 纵向上下 stack | 左 Tree；中央 live preview；右上 Details/Curve Debugger/Advanced Preview，右下 Notify/Asset Browser/Curves/Slots；Retarget Source 和 Find/Replace 经 Window 菜单打开 |
| Skeletal Mesh / `Standalone_SkeletalMeshEditor_Layout_v5` | 水平 Toolbox : Viewport : Right = .2 : .6 : .2 | 右上 Skeleton Tree/Morph/Curves；右下 Asset Details/Details/Preview；左 Toolbox 的模式需真实建模后端，不把 Skeleton 版树放到左边冒充该模式 |
| Physics / `Standalone_PhysicsAssetEditor_Layout_v5.3` | 水平 Hierarchy+Graph : Viewport : Right；左上下 .6 : .4，右上下 .6 : .4 | 树与 Physics Graph 互选；右上 Details/Preview，右下 Tools/Profiles。没有图 owner 时 Graph 保持具体缺口，不能用静态连线图验收 |
| IK Rig / `Standalone_IKRigEditor_Layout_v1.127` | 水平 Left : Center : Right = .2 : .8 : .2；左 Hierarchy/Solver Stack，中央 Viewport/Output Log | 右 Details/Preview 与 Asset Browser/Retarget Chains。真实 solver/goal 属性来自 rig 资产，不能固定四条链 |
| Retarget / `Standalone_IKRetargetEditor_Layout_v1.022` | 水平 Main : Right = .8 : .2；Main 内 Left : Center = .2 : .8；Left 上下 .6 : .4，Center preview/log .9 : .1 | 左上 OpStack/Hierarchy，左下 Asset Browser/Overrides/Variables；右 Details/Preview；source/target mesh 在真实 preview 中同步，pose 编辑是明确模式 |
| Control Rig / 当前 `Standalone_ControlRigEditMode_Layout_v1.10` | 水平 Left : Center : Right = .2 : .6 : .2；左 Preview/Hierarchy，各 .5；中 Graph/Compiler+Find .8 : .2 | 层级/执行栈/曲线/Explorer，右 Details/Preview/Attributes；完整 Graph/RigVM 仍需真实能力。legacy 布局不混入当前默认 |

### 2.3 三尺寸与 DPI 的拟定规则

使用共享 `ui_asset_side_min_width` / `ui_asset_center_min_width`、字段内容宽度、splitter hit size 和已有断点策略，按可用内容宽度求解；不新设本域魔法断点。面板的真实内容最小宽度高于基础 token 时，应尊重内容测量。

| 逻辑客户区 | 拟定行为 | 必须实证的边界 |
| --- | --- | --- |
| 1280×800 / dpi=1 | 优先保留 UE 默认 Dock 拓扑、面板相对权重和激活页签 | Tree、viewport、详情及底部日志/资产区均可用；比较真实文本行框，不能仅套比例 |
| 900×620 / dpi=1 | 先保留三列和独立滚动；工具条溢出使用共享菜单，低优先级页签保留在其 stack | TRS 字段不小于共享内容最小宽度，标题不遮按钮；必要时用共享 Dock 的合并策略 |
| 640×520 / dpi=1 | 当总最小宽度不足时，使用共享 Dock 将辅助 stack 合并为可切换页签/现有 drawer；当前编辑视口与当前主树保持可达 | Details、Graph/日志、工具/Profile 等均有 Window 菜单/页签入口；输入提交与弹层关闭后焦点回原编辑对象；不能只隐藏右列 |
| 1280×800 / dpi=1.5 | 与该逻辑尺寸的设计相同；实际物理客户区 1920×1200 | 普通 EXE 上 `GetDpiForWindow=144` 与 Winit scale=1.5、客户区物理尺寸同时成立；不得仅改截图元数据或拉伸图片 |

上述窄尺寸合并若共享 Dock 尚不支持，记录 CS-D04 的具体依赖和 Penpot 差异，交 Parent 决定公共实现；本域不新增第二套抽屉系统。用户保存的布局与默认布局分开，恢复默认布局后才采集默认 reference；调整 splitter 不改变资产 dirty。

## 3. 真实数据、选择与作者事务

### 3.1 权威数据和稳定身份

| 对象 | 当前真实数据 | 设计要求与缺口 |
| --- | --- | --- |
| 骨架 | `AnimationSkeletonAsset` 的 `bones` / `parent_index` / local TRS；合法性与二进制编码已存在 | 原资产 handle/locator + revision/generation + bone index 是运行选择的基础；路径/名称供诊断与重导入重映射验证，不能只按显示名寻址 |
| 姿势 | `AnimationPoseSnapshot`、pose bone、animation pose manager | 引用姿势、当前求值姿势、Editor 暂时预览偏移分开；local/global 读数注明空间，转换由权威 Transform 规则处理，不在 UI 再写一套矩阵算法 |
| Mesh / skin | glTF importer 保存 skin 与 inverse-bind 数据；Runtime 有 GPU skinning | mesh 实例、skin 与 skeleton 必须匹配；共享骨架并不等于可任意混用 joint index。检查 skin 绑定、palette generation 和资源就绪 |
| Socket / virtual bone / attached asset | 当前 SkeletonAsset 没有这些字段的已核实持久化 owner | skeleton 与 mesh Socket 需分别有真正归属和稳定 ID；创建/提升/还原是资源事务。未补持久化前，不能把任意字典留在 UI 当完成 |
| Physics | `RagdollProfile` / `RagdollBoneProfile` 的 bone_path、parent、shape、mass、body_offset、constraint、blend_weight | profile ID + revision + bone_path 在当前 DTO 中可用；body/constraint 选择还需明确具体对象身份。当前每 bone 一 shape，不能暗示已支持 UE 的每骨多碰撞体 |
| Retarget / Control Rig | 已有预览 ZUI 与部分 IK math，但尚未证明完整作者资源和求解链 | 不把当前静态 rows 作为真实资源；chain/goal/op/control ID 必须由实际资产提供，能力表保持真实值 |

当前 `zircon_editor/src/core/editing/animation_document/kind.rs` 仅有 Sequence、Graph、StateMachine。它没有 Skeleton/Mesh/Physics/Rig 作者文档类型；现有 animation history 不能直接宣称覆盖本域。最小变更应复用当前文档/事务 host 机制，为实际资源 owner 增加必要文档能力，不把 Skeleton 强行伪装成 Sequence，也不先建通用万能文档系统。

选择状态为文档范围内的 selected set、active item、keyboard focus 和 range anchor。Tree、viewport、Details、Physics Graph 读同一 selection projection；viewport picking 返回真实资源/对象键，再由 Editor 验证当前 generation。折叠与搜索默认保留真实选择；对象被删除/重导入后，重新验证键并选择合法邻项或清空，不能保留悬空 bone index。过滤、hover、focus、相机变化不进入资产撤销历史。

### 3.2 详情和事务

| 触发 | 暂态 | 提交/失败与历史 |
| --- | --- | --- |
| 数字/文本输入 | Field 自持 draft，IME composition 不提交资源；局部错误在原字段显示 | Enter/明确 Submit 后按实际资源 validator 验证；成功只记一次事务，失败保留 draft 和焦点、权威值不变 |
| 轴向拖动 / gizmo | begin 时记录资源 revision 和起始值；显示真实预览反馈 | release 提交一个可撤销事务；Escape 恢复起始值且不增历史；资源换代取消，不能提交到旧对象 |
| 多选属性 | 相同值显示值，不同值显示共享 mixed 状态 | 修改仅对可编辑同类对象原子提交；一项失败不部分改写多个骨骼/碰撞体 |
| Reset / Socket / 生成碰撞 | CanExecute 检查对象类型、资源写权限、后端就绪 | 模型验证成功后提交，刷新树/详情/预览；撤销/重做恢复原引用和选择。生成真实 profile 调用原 generator，不从屏幕行名拼 DTO |
| 保存 / 关闭 / 重开 | dirty 属于资源文档，布局/选择另存视图状态 | 保存走原 asset manager；记录资源 bytes/hash/依赖，重开重读相同对象。保存失败保持 dirty，关闭未保存文档用共享既有策略 |

Bone preview transform 和修改导入 reference skeleton 是两件不同操作：本轮优先真实姿势预览与明确可保存的 pose 作者。若改变 skeleton bind/rest pose，必须同步 mesh inverse-bind、动画引用及 importer 规则；未建立该事务前不可把只动 gizmo 当成修改骨架资源。

## 4. 真实预览的最小闭环

当前 `ui/preview_scene/preview_scene.rs` 已定义 `PreviewSceneBackend` 与共享 `PreviewScene`：创建/销毁 secondary session、subject、playback、invalidate、focus。`PreviewSubject` 支持 primary/additional/animation assets 与参数。尚未找到生产 backend；能力表明确 runtime preview/semantic compiler 为 false。这项缺口由 CS-D02 与动画/运行时 owner 接续，不改 false 标志来模拟能力。

```text
Catalog/asset manager 的实际资源与依赖
  → Editor 文档读句柄、validated selection、preview subject
  → 现有 preview session backend（生产实现待接）
  → Runtime secondary world：mesh + skeleton + animation/physics
  → 真实 pose 求值 / skin palette / GPU render
  → 既有 UI GPU image/media owner → 正常 Engine 窗口 present
```

打开工具组时创建一个共享预览 session；多个 Persona 关联页签共享 subject/playback，不为每个面板复制 runtime world。subject 切换时旧帧不得作为新资源 ready 帧；记录 subject/pose generation。关闭最后一个消费者或 host 退出时按原生命周期销毁、解绑 image/picking 并等待相关回执。错误和设备失效时保留实际诊断和资源身份，旧成功帧不能重新计通过。

必须打通的首个闭环：从实际项目打开 wolf 模型及其 `Skin0/Skeleton` → Tree 显示 48 个真实关节 → 选中真实关节同时更新 Details 和 viewport 标记 → 播放真实 Gallop 动画 → pause/seek 与 local/global pose 读数改变 → 同一 mesh 的 skinned pixels 改变 → 关闭并重开。资源标签的 URI/GUID 以 importer 返回的 catalog 记录为准，本文不猜最终 URI。

Picking、相机 F、骨骼/Socket/碰撞 overlay 由 Editor 工具生成；Runtime 仍只持权威世界与中性 render 数据。每次 capture 需关联真实 frame/session、asset dependency fingerprints、pose/palette generation 和 present/readback；图片文件内容有效不能单独证明资源正确。

## 5. 窗口特有操作

### 5.1 Skeleton / Mesh / Persona

依据 `SkeletonTreeCommands.cpp` 和 `SSkeletonTree.cpp` 的 CanExecute/选择回调：

| 命令 | 焦点和对象条件 | 真正效果/验收点 |
| --- | --- | --- |
| F / Focus Camera | 非文本编辑且当前有合法选择、真实 preview ready | 相机包围选中 bone/socket/body；tree 与 viewport 选择保持同一对象 |
| Ctrl+I / Ctrl+Shift+I | 选中可重置骨骼 / 所有骨骼，preview pose 可编辑 | reset selected/all 的实际姿势；不顺带重写 imported bind pose |
| Ctrl+C / Ctrl+V / Ctrl+Shift+V | Field 先消费文本复制粘贴；树中仅在合法 Socket 操作时使用骨骼命令 | 复制 Socket、按原骨骼/根回退粘贴、粘贴到选中骨骼，包含实际 parent 与 transform；不能只复制显示名 |
| Delete | selected item 类型合法且有写 owner | UE 普通导入 bone 不属于 `CanDeleteSelectedRows`；仅 Socket/virtual bone/attached asset 的合法删除走一个事务 |
| Add / Mesh Socket / Promote / Remove Mesh Override | 符合当前 Socket filter、选中 bone/Socket 类型和资源权限 | Active/All/Mesh/Skeleton/Hidden 显示策略正确；mesh override 与 skeleton fallback 保存/撤销/重开后仍正确 |
| Tree search/filter/expand | 树或搜索控件拥有焦点 | 搜索保留祖先/展平/隐藏祖先选项、首尾滚动、选中高亮与焦点边分别正确 |
| 关联资产跳转 | 当前实际 catalog 关系有效 | 打开同一 skeleton 的 Mesh/Animation/Physics，复用或激活相应文档；引用不存在时给真实诊断 |

Morph/LOD/材质、Slot、Notify、Curves、Retarget Source 与 pose 工具的窗口盘点已在主账本逐项列出。每项只有实际可读写后端接通才启用；图标和页签可编辑设计不等于该工具完成。跨窗口跳转需保留原文档 dirty、selection 与 preview ownership，不把每次跳转当成创建一套静态工作区。

### 5.2 Physics Asset

以原 `RagdollProfile` 为作者资源。生成入口调用 `generate_initial_ragdoll_profile`，输入是实际 skeleton 的 parent 和 transform；当前生成器每 bone 生成一个 Capsule、默认质量与约束，profile.validate 后才可提交。UI 应如实显示生成结果，不声称拥有 UE 全部 fitting 工具。

已有注册应保留：`physics/editor/src/plugin.rs::register_ragdoll_profile_editor` 已声明 asset type/Toolkit `physics.ragdoll_profile`、模板 `plugins://physics/editor/ragdoll_profile.zui` 及 creation template `physics.ragdoll_profile.from_skeleton`。但 `asset.physics.ragdoll_profile.create` 当前的事件只是 `WorkbenchMenu(OpenView(...))`，没有实际 skeleton/profile 参数或生成事务。对 Editor 与 physics 插件的调用搜索只见 generator 定义、lib 导出和测试调用；因此“从骨架创建”的注册不算真实生成功能，接线应修正原 creation operation，不能另加相同工具入口。

Body/Constraint/Graph/Tree 选择同步，Details 对接 shape、mass、body_offset、constraint 限位/驱动及 blend_weight。`PhysicsJointConstraintMetadata` 有 linear/angular limits、drives、break thresholds、projection tolerance，UI 只呈现实际关节类型和 validator 支持的项；单位换算应来自同一物理/数学 owner。Box/Sphere/Capsule 等能否创建、凸包/多体/Weld/Profile 的支持状态逐项记录，不能从 enum 存在推断完整操作已接通。

运行时已有 `RagdollProfile::spawn` / `spawn_configured` 与 `RagdollRuntime::{configure_profile,set_mode,remove}`，模式为 Animated/Simulated/Blended。编辑器模拟必须在预览 secondary world 中执行，开始前保存编辑姿势，结束/失败回到编辑状态，停止后不将模拟瞬态写入骨架 reference pose。UE Alt+I / Alt+Shift+I 分别对应模拟全部/选中；当前 API 的“选择子集模拟”能力尚未证明，后者不能靠更改按钮文本冒充。模拟中编辑限制、碰撞拖拽、碰撞体 gizmo、约束轴显示及回到焦点均要实际验证。

已有物理显示链也应复用：`viewport_overlay_provider.rs` 注册 `physics.viewport.overlay.provider`，读取 `PhysicsPieMirror` 的真实 `PhysicsOverlayFrame`；`overlay.rs::build_physics_overlay_extracts` 从 collider shape/transform 生成 `SceneGizmoOverlayExtract` 与 segment pick shapes，并按真实 entity 标记 selected。`runtime_mirror.rs` 已有 play_session_id/sequence/owner generation 的错会话、旧帧拒绝与结束清空。当前 provider 读的是同一 PIE mirror，Context 只取 selected entity；需与 CS-D02 owner 接续 preview secondary session 的资源/会话身份和 collider entity↔profile bone 映射，不能直接把主场景 PIE 帧放进 Physics Asset 视口。该链目前显示 colliders，不能从它推断 Constraint graph/轴编辑已实现。相关测试源存在但本轮未运行，且不替代正常窗口证据。

### 5.3 IK Rig / Retarget / Control Rig

重定向设计跟随当前 UE OpStack 版本。Source/Target 必须由实际资产选择器绑定，检查骨架/chain/goal 兼容性；编辑 retarget pose 与正常播放分为可识别模式，退出模式后恢复对应播放状态。Apply/Export 必须产生可打开、可重放的真实输出资产；无求解/保存 owner 时保持未完成，不输出虚假成功。

Control Rig 的真实可行子集是已有 TwoBoneIk/LookAt 等数学能力对应的实际 goal/pose 控制，仍需 Editor 资源、调度与预览接线。Graph、Compile、RigVM、Modular Rig、Validation/Influence/Module Browser 不因数学 IK 存在而算支持。Parent/用户的适用子集决策尚未返回前保留完整目标布局和对应缺口，不给主菜单挂无效果入口。

所有快捷键先由 popup/active text editor/当前 Toolkit command scope 处理，再按共享 host 规则冒泡；Escape 关闭最上层弹层或取消本次 draft/gizmo，focus-return 指向原稳定对象/控件。Ctrl+Z/Y 与 Save 使用当前文档历史；实际键位和 scope 需要与共享 host 对齐后验证，不另注册全局冲突键。

## 6. 原始 ZUI 导出身份与迁移契约

导出前以原文件为基线记录完整 `asset`、`components`、`nodes`、`control_id`、`imports`、slots、bindings 和每项 events；下面只列迁移锚点，不能用子表取代完整基线。原文件当前未修改。

| 原资产 | 必须保留的结构锚点 | 必须保留并实测的事件锚点 |
| --- | --- | --- |
| `extension_retarget_workspace.zui` | `retarget_workspace` / `WorkbenchExtensionRetargetWorkspace`，main/left/center/right，left_content/chain_table/right_content，source/target/solver property row 的 value slot；保留原 component key `WorkbenchExtensionRetargetWorkspace` | `WorkbenchExtension/RetargetPreview`、`RetargetApply`、Source/Target/Solver 的 Edit+Commit，三项资产 row 与四项 chain row 的原事件 ID |
| `extension_control_rig_workspace.zui` | `control_rig_workspace` / `WorkbenchExtensionControlRigWorkspace`，main/left/center/right，left_content/solve_table/right_content，control/space/weight property row 的 value slot；保留原 component key | `WorkbenchExtension/ControlRigPreview`、`ControlRigValidate`、Control/Space/Weight 的 Edit+Commit，三项层级 row 与四项 solve row 的原事件 ID |
| `ragdoll_profile.zui` | `ragdoll_profile_root` / `RagdollProfileRoot`、body、toolbar、`ragdoll_skeleton_tree`、`ragdoll_preview`、`ragdoll_bone_properties`；`root.node`、`asset.id=physics.ragdoll_profile` | 当前无业务事件；新增真实事件需要原 owner 注册，不能凭空声称原有 binding 已支持 |

已发现的合同核查项：Retarget ZUI Preview route 为 `workbench.extension.retarget.preview`，builtin binding/spec 为 `workbench.extension.retarget.preview.invoke`；示例 row 也有 `.rig.mannequin` 与 `.mannequin_row.select` 的命名差异。已追踪正常 control 路径：`dispatch_componentized_workbench_control` → `dispatch_control_state` → `projection_support::binding_for_control`，按实际投影中 control/event kind 对应的 `route.binding_id` 查 binding 表，再执行已解析 payload；按 ID 分发路径还会检查该 ID 确实属于当前 host route。

继续追踪已收束“投影解析优先级”问题：`runtime/build_session.rs` 安装 builtin bindings；`runtime/projection.rs::project_v2_binding_ids` 对 `UiBindingRef` 调用 `template/adapter.rs::resolve_binding`。adapter 只按稳定 ID 查注册表并校验 event kind，不按源 route 字符串猜动作；host 的 action_id 从该 binding 的 payload 提取。`builtin/bindings/install.rs` 明确以 `WorkbenchExtension/` + spec control ID 注册对应 action。故此处两种命名并非已证明的失效原因，不提交无依据的批量 route/ID 改写。实际加载、preview gate、最终命令及真实后端效果仍未验证，不能把分发合同存在当作 retarget 已实现。

固定示例行转换为真实 collection 时，source node/template identity、运行 item identity、事件路由是不同层次。保留源模板与事件 ID，并让 row instance 绑定真实资源/对象键；点击结果不能按 `mannequin` 等源码名称选择假对象。导出器若不能表达当前 collection/binding 及保留身份，向 Parent 提交 CS-D01/05 的具体缺口后处理，不能先删旧节点、复制一批固定角色、重建另一份 mock 窗口。数据样例在 Penpot 可用于可读设计，产品列表必须来源于实际模型。

Penpot 采用共享组件实例与共享样式，能编辑具体 layout/content/instance properties。保存 ACK 后关闭，fresh 重开核实 file/page/component/instance/styles/flow 与源 metadata；官方 PNG 和导出 `.zui` 同属该已保存 revision。此步骤完成后才能修改本域三个原始 ZUI 并接运行接线。现阶段尚无这类证据。

## 7. 真实验证输入的只读盘点

2026-10-24 只读解析现有 GLB header/JSON chunk，未提取纹理、未创建资产副本、未导入或运行 Engine。已检查的 7 个 WoC 玩家、2 个生物模型均有 skin/weights；以下两项优先作为实际输入候选，**不是已经 ready 的资源**。

| 候选源文件 | 真实数据 | 指纹与使用条件 |
| --- | --- | --- |
| `examples/woc/assets/m8/models/creatures/wolf_basic.glb` | 333040 bytes；1 skin；48 joints；1 weighted mesh primitive；14 animation clips；Gallop 通道作用于全部 48 joints，translation/rotation，采样输入范围约 .0333333–.4666667 秒 | SHA256 `655da121afa8768c13a3b2aa172e6875c716d06e352110bf68eaa4f34c9fec6e`；首选单 skin 闭环，最终 asset URI/GUID 和运行依赖待 importer/catalog 验证 |
| `examples/woc/assets/m8/models/chars/players/knight.glb` | 1212756 bytes；9 skins 的关节列表相同，23 joints；9 weighted mesh primitives；22 clips；Running_A 通道作用于全部 23 joints，translation/rotation/scale，范围 0–约 .8 秒 | SHA256 `ce0d0a5ec1d67594629ecac20a7576fde22a44456e7628e8eb100891bc6dda97`；验证多网格共享骨架和导入关联，不能只显示第一个部件算完整网格 |

两项候选均要求 `EXT_meshopt_compression`、`EXT_texture_webp`、`KHR_mesh_quantization`。wolf 有 9 个压缩 bufferView、knight 有 50 个；实际 BIN 是 buffer 0，buffer 1 带 meshopt fallback 标记，解压后的 inverse-bind/顶点/动画数据放入 buffer 1。对所有这些 view，已只读核对压缩源范围在 BIN 内、解压目标范围在声明内、count × stride 等于解压 byteLength；**未实际执行解码或验证数值权重/姿势**。

一次尝试直接读数值的内存脚本因假定 accessor 的 buffer=0 而失败（AssertionError），原失败保留在产物清单。进一步检查排除了“没有 sparse 支持”假设（两项均无 sparse accessors）；实际需要 meshopt 解码。`gltf_decode/buffers.rs` 会为 fallback 分配目标，`gltf_decode.rs` 调用 `gltf_meshopt::decode_meshopt_views` 后才供 importer 读取。这是源码已存在的正确路径；没有据简单读取失败认定资产损坏，也没有把结构核查降格成数值通过。下一步应由 Jenkins 对实际候选进行聚焦导入验证，再进入产品闭环。

`gltf_importer/runtime/src/lib.rs` 会加入 animation/skin 子资产，保留 MeshSkin inverse-bind；现有测试源断言 `Skin0/Skeleton`、`Skin0/InverseBindMatrices` 与 `Animation0` 关系。`mesh_skin_assets_by_mesh` 当前对同一个 mesh 的多个 node-level skin 保留首项，须验证候选具体绑定，不能把源码中支持路径等同于该输入成功运行。`subassets.rs` 的 scene entity 还将 animation player/skeleton 字段置为 None，故有 subasset 不等于已有可播放 scene wiring。

当前候选的 root joint 外侧 parent 均未声明非默认 TRS；knight 的 9 个 mesh node 各绑定其对应 skin。Importer 只将直接 parent 同在 joint set 时写入骨骼 `parent_index`，并从该 joint 的 local transform 建资产。其他资产若有非 joint 中间父节点/非默认上层变换，应专门核对，不从这两项样例推广为任意 glTF 层级都已正确。现有 `woc_meshopt_required_glb_imports` 测试引用 `dev/world-of-claudecraft/public/models` 的另一模型，不能用其历史结果替代本候选验证或无范围扫描全部资产。

反例：`examples/vampire/assets/models/kenney_graveyard/character-skeleton.glb` 虽有名字 skeleton 和 32 animations，JSON 中无 skin、无 JOINTS_0/WEIGHTS_0。它不能证明本窗口的真实骨骼蒙皮能力。所有解析结果只证明源文件结构，不证明运行时资产 ready、姿势求值或可见帧。

## 8. LayoutReviewCase 与逐图验证设计

沿用 `tools/penpot/apps/zircon-zui-plugin/src/model.ts` 的 `LayoutReviewCaseMessage` 和原生 evidence gate。`viewport` 是逻辑尺寸，`dpi` 是比例值 1 / 1.5；原生设备 DPI 另以 96 / 144 记录。`physical_viewport` 的现有算法为 logical × dpi 四舍五入。`state` 使用现有枚举，不能自造 `long-zh` 等 state；长文本用 locale + 真实 data 组合。

case 的 `sourcePath` 只能是实际导出且实际加载的 owner。Skeleton/Mesh 新 owner 尚未分配，故本轮不生成一个指向不存在文件的“有效” case。Physics 的 host 用实际产品集成后的 `plugin` 或 editor host，不凭资产路径推断；目前 `workbenchState` adapter 只接受 editor host，若 plugin 需新 adapter，提交 Parent 具体差异。不能用 component/fixture host 图冒充完整 Engine 窗口。

| case 设计组 | 现有合法 state / 变量 | 必须由真实产品产生的状态和效果 |
| --- | --- | --- |
| CS-BASE | default；全部四种尺寸/DPI | 正常默认布局、同一实际资源、未被错误 popup 遮住，资源全部 ready |
| CS-EMPTY | empty | 合法空文档/无可用 mesh 或真实搜索无结果；区别缺资源错误，不通过藏控件伪造空数据 |
| CS-SELECT | selected | 实际 bone/Socket/body/constraint/goal 选择，树、详情、viewport 同一对象 |
| CS-FOCUS | focused | Search、TRS 字段与树行分别取证；IME draft、错误提交、focus ring 与 selection 不混淆 |
| CS-POPUP | open + focus-return | Filter/context menu/asset picker 的实际打开、选择或取消、焦点返回、键盘可用性 |
| CS-SCROLL | scroll-before / scroll-after，或 selected + scrollPosition=start/end | 真实 48-bone hierarchy 展开后的首/末行；Details、资产列表的独立 scroll owner 和首尾目标 |
| CS-TEXT | default/selected/focused；locale=en-US/zh-CN | UI 文案走原 i18n；通过真实作者输入或实际资产 metadata 获得长名/中文，保留完整对象身份；不以测试 textOverrides 证明真实绑定 |
| CS-POSE | default/selected/dragging | reference pose、真实 clip playback/pause/seek、local/global pose、reset、undo/redo 的数据与画面变化 |
| CS-SOCKET | selected/open/focus-return | Add/rename/parent/TRS/copy/paste/promote/remove/delete、保存重开；持久化未就绪时该组保持未完成 |
| CS-PHYS | selected/dragging/running/default | Capsule 参数与 gizmo、质量/约束、真实生成、模拟/停止回编辑姿势、撤销/保存重开 |
| CS-RT | selected/open/running/failure | source/target、OpStack/chain/goal、retarget pose、真实输出与失败日志；缺后端时不能只展示 planned case |
| CS-RIG | selected/dragging/running/failure | 真实 goal/IK 输出 pose、合法性诊断、历史/保存；完整 RigVM/Modular 分列待定 |
| CS-LINK | open/focus-return/default | 实际关联资源跳转、回到原文档、dirty/历史与预览生命周期 |

通用状态要覆盖全部要求尺寸；每项特有交互至少具有标准尺寸的完整操作证据，并在窄尺寸验证可达、在 150% 验证关键边界与文本。最终以显式 case 清单记录实际覆盖，不能从一个窗口/一个 case 推出整个族完成。不能使用平台重采样制造 150% case。

逐 case 必须留存：Penpot revision/实例/共享样式和官方 PNG；source ZUI/依赖/theme/locale/data fingerprints；正常 Engine EXE/DLL/PID/birth/client extent/实际 DPI；native capture 入口、frame nonce/generation、资源 ready、present/readback、ACK/release；未裁剪全窗口原图；同状态并排图和 diff；逐图的面板边界/行框测量、文本/裁剪/焦点/滚动/操作复核。允许为测量另取 crop，但不能替代原生原图。

关键边界/文字行框误差目标 ≤1 逻辑像素；150% 时先以真实比例转换测量并记 rounding，不用整图平均分遮盖局部错误。native 有效且资源 ready 才可 passed；两端同状态比较和逐图审阅完成才可 accepted。资源缺失、静态 SVG、只读解析、单元测试、队列 ACK、pending receipt 都不得提升产品状态。

### 8.1 必须完成的一次样式闭环

在已分配真实 Penpot 中，对本域实例做一个共享样式允许的可见修改（优先面板标题/树行选中态的合法 instance/shared-style 修改），记录修改前后身份与属性；由 Parent 协调涉及共享样式的影响。保存/关闭/重开 → 插件导出原 asset → 依赖 hash 改变 → 正常产品实际加载相同导出 → 同一 case 原生截图出现对应变化 → 官方图/并排/diff/逐图复核。若修改了全局 style，必须由 Parent owner 提交；不得以手改 ZUI 或离屏渲染替代证明。

## 9. 分阶段验证与当前未完成项

本节保留首次规划检查点：当时只有手写规范文档和只读数据盘点，执行路径/Markdown/JSON/TOML/源指纹结构核查，未触发 Rust 编译。后续实际Penpot操作和托管提交分别见第11～14节；下表是当时快照，不覆盖最新状态。各实施阶段分别执行相关检查；提交任何编译命令前声明最小模块、package、default-features=false、所需 features、target/filter、profile 和明确命令，走支持的 `dev-module-build.ps1 -Package` 或受管 recipe，不选择过大的预设构建矩阵。

| Gate | 下一步可审阅结果 | 当前状态 |
| --- | --- | --- |
| 真实设计 | 已分配 file/page、实际共享组件/样式身份、组件实例、flow、保存/关闭/重开和官方图 | file/page 回读完成；其余 pending，本文不能替代 |
| 原资产导出 | 三个原 ZUI 的稳定 ID/import/slot/binding/event 保留与导出差异 | 未执行 |
| 真骨骼预览 | wolf 真实导入/Tree/Details/pose/GPU/正常窗口闭环 | 生产 PreviewScene、UI GPU image、picking 及资源接线未完成 |
| 作者事务 | Skeleton/Socket/Pose/Physics/Rig 的实际文档 owner、undo/save/reopen | 已有 animation 文档不含这些类型；具体 owner 接线未完成 |
| Physics | profile editor、body/constraint/gizmo、真实模拟/恢复 | generator/runtime 已有，产品 UI 接线与证据未完成 |
| 重定向与角色工具 | 实际 retarget/rig 资源、求解/输出与应用边界 | 完整后端未证明；RigVM/Modular/DNA 等适用性按盘点逐项保留 |
| 验收 | Jenkins 相关检查+聚焦 Rust、四尺寸/DPI与全部状态的双端图/差异/逐图复核 | 全部未执行；不从源码或计划宣称完成 |

待 Parent 接续的设计决策集中在 CS-D01–05：真实 preview owner、文档/事务与骨架扩展资源 owner、公共 TreeRow/collection/export 的稳定身份能力、窄窗口 Dock 合并，以及真实 Rig 子集。本文无公共源修改、无服务/桌面占用、无新构建/缓存/源副本；释放空间 0 字节。任务仍 active。

## 10. 给公共 owner 的最小合同清单

这些是基于已有调用点的提案，尚未变更公共 API 或登记虚假实现。由 Parent 合并共享工作分配，原职责保持单一。

| 依赖 | 实际源点与最小必要工作 | 本域提供/消费 | 阶段验证 |
| --- | --- | --- | --- |
| CS-D01 collection/行度量 | 保留当前 `WorkbenchTreeRow` 的默认 dense 状态；复用 item flow 容纳 inline editor，核实 virtual collection 对可变 item extent 与稳定 key 的支持 | 输入为实际 skeleton revision、typed item keys、展开/过滤集合；输出 visible rows、selection/focus/picking 目标 | 同名骨骼不误选、过滤/展开后 range 与滚动首尾不漂移、inline editor 不裁切、中文长名可达 |
| CS-D02 production preview | 生产实现接入现有 `ui/preview_scene` backend 六项能力；由现有 UI GPU image/media owner 呈现 Runtime 帧，不另造图片组件 | 原 PreviewSubject/Playback；补足真实 ready/error/subject与pose generation/frame关联/picking 的最小需要，具体 DTO 复用原 Runtime 中性合同 | 一次资源切换和一次真实姿势改变，同时验证帧资源/图像变化/旧帧失效；关闭最后消费者无 session 泄漏 |
| CS-D03 document/history | 复用 `core/editing` 与 host 文档历史机制，明确 Skeleton/Socket/Pose/Profile 的真实资产 owner；不借用 Sequence 类型伪造支持 | 实际资源 read handle、revision、typed mutation、validator、dirty/save/undo；本域 Details/命令只发作者操作 | 一次 gizmo 拖动只记一次历史、失败原子性、Escape 取消、重开验证序列化、重导入后键失效处理 |
| CS-D04 Toolkit/Dock/commands | 复用 DocumentHost/UiHostWindow、原资产 browser/context menu/window menu/command scope；新增独立 Toolkit 注册和窄窗口 stack 合并由公共 owner接入 | 本域最终 Penpot 导出的原/新 owner 及合法命令能力；选择/视图状态不写 Runtime world | 三尺寸全页签可达、popup focus-return、键位冲突与文本消费、dirty关闭和关联文档复用 |
| CS-D05 Penpot/export/review | 使用已分配真实 file/page，待实际共享组件/样式身份并复用现有插件；若 collection roundtrip 或稳定节点/事件保存缺能力，提交具体源模型差异后补齐 | 原 Retarget/ControlRig 各31节点、29 control IDs、15 events，Ragdoll6节点/6 control IDs/0原事件的基线，最终真实绑定与依赖 | 保存/关闭/重开、原ID/slots/imports/events闭包、一次样式修改到正常EXE截图的实证、复用原严格capture协议 |

新增 Skeleton/Mesh 文档 owner 的文件名/注册点应在已分配页面形成设计并核对公共 host 后确定；本域不先创建未被设计或加载的“真实”路径。现有三个业务原资产路径已明确，所有适用能力和未完成项继续由主账本逐项保留。

## 11. 实际 Penpot 操作与未完成边界

实际原始资产为 `zircon_editor/assets/ui/editor/animation/extension_retarget_workspace.zui`，SHA256 `4fe1c1d1543f92bd925f809563a07b2f424e843cfd483931cf770210a120670f`。31 原节点、29 唯一 control IDs、15 事件保持不变。隔离 consumer 直接复用 checkout 的 parser、LayoutDependencies、字体 manifest、共享 tokens/Starship 和现有插件，不修改公共入口。组件 root 仅在 review projection 中设为可见的 1280×800 view；原 component 类型与源文件未改。展开后为 34 个语义节点，实际 Penpot board 是 `3aae194a-8d90-80a9-8008-d6a74bfb8568`，main flow frame 是 `3aae194a-8d90-80a9-8008-d6a74c60628d`。

当前插件 native primitive cache 按资产创建本地 master，尚无已核实的跨文件发布库查询。为遵守公共组件唯一 owner，本域使用现有 semantic materialization 做布局研究，实际 native components=0。此选择不满足用户的共享原生实例/样式要求，相关 gate 保持 pending；不将空组件 UUID 或普通 flow 形状冒充实例。

| 操作 | 真实结果 | 限度 |
| --- | --- | --- |
| 首次官方 source-closure helper | 拒绝非 canonical main workbench owner，未启动浏览器或导入 | 不修改公共 helper 的 owner 保护 |
| 原资产实际导入 | 03 回执取得真实 board，后端修订 1、123 对象 | consumer save URL 监听遗漏 query/legacy route，初次 save ACK 失败仍保留 |
| 原生 inspector 编辑 | 05 回执通过真实 Penpot Layout inspector 将 main column gap 8→6；真实 update-file HTTP 200 | Table 重排导致 ZUI 导出被保护拒绝，未回写原资产 |
| 保存、关闭、fresh context 重开 | 06 回执 observed save HTTP 200；关闭原 context，新 context 重开同 file/page/board，修订 5、gap=6、17 个 flex/grid frame | 只证明该可编辑基线持久化，未证明 UE 布局/业务/组件库或产品就绪 |
| 官方 ZUI 导出 | `retarget_root_chain_table_row` 的 `ZUI text cell-0` width 197.12→199.28，x=20/y=4/h=22；`assertExportShapeGuard` 报 changed geometry without a ZUI source mapping | 不删除 guard、不篡改 baseline、不绕过导出回写；由公共插件 owner 修复 flow 派生 Table geometry 映射 |
| 官方 Canvas PNG 导出 | 插件明确报 `Penpot shape export did not return a valid PNG byte stream`，没有下载到 PNG | 浏览器 editor 截图只作 Penpot 操作证据，不能充当官方 PNG 或 Engine 原生捕获 |

完整实际回执：[06-persistence-and-export-diagnostic-receipt.json](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-design/06-persistence-and-export-diagnostic/06-persistence-and-export-diagnostic-receipt.json)。重开截图：[retarget-reopened-editor.png](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-design/06-persistence-and-export-diagnostic/06-persistence-and-export-diagnostic-retarget-reopened-editor.png)。03～05 失败、实际 save request/response 和失败截图按原身份保留。所有自有 headless browser 与临时插件静态 host 已退出；没有重启 Penpot/Jenkins，没有共享桌面/DPI/Engine 输入。

列 gap=6 是样式传播实验值，没有 UE 测量依据，不能作为最终局部样式。原始 ZUI 的共享 token 未变；导出/产品截图链修复后，应以 UE 测量决定恢复 token 或明确的局部覆盖，不能把这次实验静默固化到产品。

当时复用 worker SHA256 `40d1150059b975aa02d616a390021e3b1a5a2592703661efc78f605e6c7bafb6`，UI 来自 `served-dist-current-20261023`；该次 checkout 的 173 项 worker 输入和 513 项 UI 输入逐哈希匹配既有编译回执，未重新编译。后续4项worker输入已变，当前闭包差异见第13节，不能继续声称旧bundle对应最新源码。该复用来源核对不等于本轮 Jenkins 插件检查或 Rust 测试通过。无官方 PNG、原始 ZUI 导出、正常 EXE、真实骨骼/网格/姿势预览及双端比较，全部产品 case 继续 pending，`passed/accepted=false`。

当前画面仍是原始三项虚构 rig/四条链模板，不能作为 UE Retarget 目标交付。下一项独立工作是在专属文件设计真实 `wolf_basic.glb` 的 48-joint 层级与从 JSON 读取的局部 TRS；这不证明压缩 mesh 权重、动画求值或 GPU preview 已运行。Skeleton 新 body 的最终 owner/Toolkit 注册仍交 CS-D04；公共实例、官方导出和 production preview 继续交 CS-D05/02。

## 12. 真实 joint 草案与公共导入阻塞

已读取真实 `examples/woc/assets/m8/models/creatures/wolf_basic.glb` 的 JSON 节点、skin 0 joint 顺序及父子关系，形成 48 项 `gltf:skin:0:joint:<index>:node:<index>` 设计数据键。逐项核对名称、父键、translation/rotation/scale 默认值及有限性，四元数平方范数误差小于 `1e-5`，14 条动画名称一致。选中项为真实 `Head`（joint 7 / node 7）。这只是源 JSON local TRS；Engine 坐标转换、inverse-bind/accessor 解码、蒙皮和 pose 求值仍未执行，不能作为真实预览就绪。

草案实际输入位于任务报告目录，生成模块直接复用原 GLB、现有组件 imports、tokens、Starship 与 LayoutDependencies，没有创建产品 owner 或私有 TreeRow。草案中的 `zircon_editor/assets/ui/editor/character/skeleton_editor_*` 是明确的 proposed owner 字符串，checkout 中没有这些资产，不能登记成已加载窗口。整窗草案包含树、真实骨骼详情、局部 TRS、关联动画列表和明确未就绪的 preview 表面，不使用角色图片；共同 Menu/Dock/宿主壳继续依赖 Parent。

| 尝试 | 真实结果与保留证据 | 当前结论 |
| --- | --- | --- |
| 07 | closure loader 在 child-first 顺序先展开标题后，PanelHeader title slot 看到了 Label 而非 WorkbenchSectionTitle；改用现有 canonical 资产的 parent-first 顺序 | 仅修正本域草案生成顺序，公共 slot 接受约束未改；初始输入保留 |
| 08/09 | consumer 的 Locator.evaluate 首参数误取为 payload，真实插件收到 undefined source，TOML 导入失败 | 本域脚本错误已修正；原文件未变，失败回执保留 |
| 10 | 实际整窗导入期间 status evaluate 30 秒超时 | 未取得 board/保存，不以本地生成代替实际导入 |
| 11 | 145-node 整窗草案在实际 Penpot 的 180 秒观察内仍处 creating design board，没有成功 board 身份；自有 browser/host 正常退出 | 后端仍为修订 5、123 对象，仅 Retarget board；不反复新建未完成板 |
| 12 | 缩至 26-node Skeleton Tree 草案，设计默认可见真实行 0～16，完整 48 项模型保留；真实插件拒绝 TreeRow painter state 的 paint 差异 | 没有已保存 Skeleton board 或 native component；滚动末端、整窗及窄窗仍未完成 |

11 的实际 CPU profile：[actual-import-cpu-profile.json](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-design/11-skeleton-profiled-import-1280/actual-import-cpu-profile.json)。本地公共 projector 对同一输入耗时约 0.363 秒、145 shapes、1 root、无错误诊断；真实浏览器 profile 观察到 Penpot `create_shape`、`commit_fills`、`commit_strokes`、`font_proxy` 和 CLJS 拷贝开销。该 profile 不足以单独证明根因，由公共插件 owner 对真实浏览器路径继续定位。

12 的完整实际失败回执：[12-skeleton-tree-1280-receipt.json](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-design/12-skeleton-tree-1280/12-skeleton-tree-1280-receipt.json)，失败截图：[12-skeleton-tree-1280-authoring-failure.png](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-design/12-skeleton-tree-1280/12-skeleton-tree-1280-authoring-failure.png)。失败发生在 `joint_0` 的 transient TreeRow `53f5a8ad-bb1d-8060-8008-d6ae42648f6a`；它不是已保存 board/component 身份。

具体公共修复提案已交 Parent：`tools/penpot/apps/zircon-zui-plugin/src/penpot-control-content.ts` 的 `treeRowState` / `sameTreeRowState` / `refreshTreeRowContent` 观察到 SDK paint 的 `fillColorGradient`、`fillColorRefFile/Id`、`fillImage`、`strokeColorRefFile/Id`、`strokeCapStart/End`、`strokeColorGradient`、`strokeImage` 从 missing 变成 null。当前 `canonicalTreeRowPaint` 只规范颜色大小写，snapshot 仍做严格键集合比较。应由公共 owner 核实这些已知可选空字段的等价规范化，并继续拒绝非空引用/图片/渐变/端帽改变、真实 paint/geometry/拓扑篡改；不能宽泛忽略 null 或移除保护。相关真实 SDK 格式源为 `third_party/penpot/frontend/src/app/plugins/format.cljs`，既有聚焦测试入口为 `penpot-control-content.spec.ts`；本域没有修改或运行这些公共测试。

Retarget 的 Table 派生 geometry 和 PNG 导出故障也保持 open。失败源、脚本修正、真实拒绝和未取得保存身份分别记录；所有产品 case 继续 pending，`passed/accepted=false`。Parent 队列 ACK `01a16dab-70a9-7fb1-a857-b07a6197d373` 只表示消息已排队。没有共享桌面/DPI/Engine 输入；自有浏览器/静态 host 均退出，临时目录保留于登记的任务根，释放 0 字节。

Parent 最新交接已记录 Widget/UI 同源的 TreeRow missing→SDK null 阻塞，并把其最小 nil 等价提案（SHA256 `712b20a270db5578484bb13bc07e68aba256633eaacda36b1c06bf827175f080`）交原 public pipeline owner 审查/集成/聚焦测试。本域补充真实骨骼行复现及字段清单，消费同一公共修复，不另写同功能 patch。Parent 的交接不是修复已完成；仍需新实际 bundle/source 身份和本域重新导入证据。

补充复用要求：Physics 不新建 Toolkit/creation ID 或 overlay provider；修复原 creation operation 的真实作者操作，在现有 provider/mirror/中性 render 协议上接 preview session、对象映射和生命周期。这项具体范围依赖 CS-D02/03/04，不能在本域单改全局 PIE mirror 使其他运行窗口串会话。Socket/virtual-bone 方面，对 `zircon_runtime/src`、`zircon_editor/src`、`zircon_plugins` 的 Rust/TOML 以骨骼 Socket/attachment/virtual-bone 组合关键词搜索未发现持久化合同；结合当前 SkeletonAsset 字段，继续保留 CS-G03，不能推断普通网络 Socket API 可承担该资源。

## 13. 真实角色 CPU 回归与本轮公共输入状态

新增唯一测试文件 [character_skinning_sources.rs](../../../zircon_plugins/gltf_importer/runtime/tests/character_skinning_sources.rs)。初次静态候选 SHA256 `019e238b9c9aa82f5fec4af36f78308e146b2c60ed16d9e8e1a9f5fede5851a0` 随被拒的 v2 请求保留；后续源审查纠正实际导入名称应为 `Node<node-index>:<source-name>`，并将 header 元数据读取和生产 required-extension 校验分开。最终候选 SHA256 为 `115cfbe9720690a17c115aa5ea708b9d2e93ecbab425730e53db88ef3d4dd9b8`，尚未重新提交。直接读取已入库 Wolf（1 mesh/1 skin/48 joints/14 clips）和 Knight（9 meshes/9 skins/23 joints/22 clips）；调用现有 `import_gltf` 与 `DefaultAnimationManager::sample_clip_pose`，检查实际 meshopt 数据解码、权重/关节/逆绑定矩阵、关节槽位、Skeleton/Clip 二进制重读、有限且实际变化的姿势。不改 importer、动画模型、Cargo manifest 或资产，不复制 fixture；这是待执行测试，不是 GLB ready、GPU 或正常 EXE 预览证明。

最终 `rustfmt --check` 与 `zircon-change-guard` 通过，guard 为 0 错误/0 警告，最终 patch digest `d55136195bdf71abaa6c8dae6f2707d51490aa6b47c6e0ed48be67ef29875937`；初次 digest `3cfd654ffa69c974d173b8e5a6dceed40b5c61ca3a747a13647b98ad477427d8` 保留为历史。托管范围为单 package `zircon_plugin_gltf_importer_runtime`，defaults off，features=`runtime,zircon_runtime/native-host,zircon_runtime/animation`，单 integration target `character_skinning_sources` 的两个测试，单 job/单线程；当前模块入口映射 profile=`dev`、action=`test`。初次入口和完整源/依赖哈希保留在 [source-and-static-checks.json](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/source-and-static-checks.json)，[最终静态候选记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/source-and-static-checks-final.json) 单列当前哈希和未提交状态。静态通过不能计为 Rust 测试通过。

原 request/operation 为 `ue-character-source-20261024-01`，driver=`fc55322804fb53ab3e39f996c848ef5e5f3907497d3f38167b7095244b2a7399`，runtime=`start-1792836412672596900`。初次 identity 的 `ownerThreadId` 被当前 validator 拒绝；保留原请求及日志，按实际 schema 用同一 request 的 v2 移除该字段并绑定 12 项输入哈希。v2 在 pre-submit 两次源码捕获中发现 `zircon_plugins/ai/editor/src/authoring/{asset,edit,error,topology}.rs` 和 `src/tests/authoring.rs` 五项外域改动，因此拒绝提交。没有 Cargo 启动或 server admission；不重复换 ID 或用本地编译替代。保留 [原日志](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/jenkins-character-sources-submit.log) 和 [v2 漂移日志](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/jenkins-character-sources-submit-v2.log)。原 client PID32436（出生 `2026-10-24T11:49:38.497378Z`）已退出。

本轮重新核对原 worker compile receipt，173 个输入中有 4 项现已变化：`penpot-projection.ts`、`penpot-prefab-icons.ts`、`tools/penpot-preview-layout-audit.ts`、`tools/zui-layout-real-session.ts`。此前 173/173 的结果仅代表当时，当前为 169/173；不能把旧 bundle 当作当前源码闭包。公共 TreeRow/Table/PNG 修复及新 bundle 回执仍由原 owner 负责。该次观察时仍是 rev5/123 objects 的 Retarget 基线；此后实际详情组件已推进到第 14 节的 rev18，Skeleton 树本身仍无保存成功身份。

已排队的本轮交接为 `01a16db4-361f-7b13-adf0-d9e55e9daec4` 和测试范围消息 `01a16db9-61e2-7483-81d4-e1b71a37c7f6`；只证明投递。其后的完整 Jenkins 漂移交接在 11:52Z 被官方 `codex queue` 拒绝：`queue cannot contain more than 100 submissions`（-32600），因此**没有发出**；详情保留本节和 Parent 交接文件。不清除他人的队列、不把失败冒充已通知，也不更换非官方消息路径。

11:55Z 沿原 request 的只读 query 返回 `Current session authorization is pending`，仍无 server admission。12:27Z 后续实质进展交接已成功排队为 `01a16de1-ec4d-7951-8b1f-5dcca9d64410`，含下节真实组件和本节 importer 缺口；不能将该 ACK 解释为 Parent 已读或授权/测试通过。

进一步只读源码定位到 **CS-G10：候选 GLB 的实际导入前置策略不一致**。两份源 GLB 的 `extensionsRequired` 均为 `EXT_meshopt_compression, EXT_texture_webp, KHR_mesh_quantization`。`zircon_runtime/src/asset/importer/ingest/gltf_decode.rs` 的 Runtime 白名单已含三项，并在校验、加载受限 buffers 后真实调用 `decode_meshopt_views`；但 `zircon_plugins/gltf_importer/runtime/src/lib.rs` 的 `STABLE_IMPORTER_SUPPORTED_REQUIRED_EXTENSIONS` 遗漏 meshopt。后者调用同一解码函数，按其源码会先拒绝 required extension，尚未实际执行 Rust 来证明具体错误回执。公共修复提案是由真实 importer owner 审查该唯一 policy 差异，沿既有解码/边界检查接通并运行两份真实输入；不是新增 decoder、改全局 capability 或放宽未知 required extension。生产 importer 未由本域修改，角色 CPU 回归和 ready gate 保持 open。

## 14. 本域专用原生骨骼详情组件：实际保存与逐图复核

使用实际认证 `update-file` API，在专属 file/page 创建只读的 `SkeletonBoneReferenceDetails` 组合，保留旧板；未使用失配旧 plugin bundle，也未新建 TreeRow、PropertyEditorRow、NumberField、AxisValueField、按钮或主题 master。共享基础控件/样式仍 pending；本组件仅是本域可编辑叶子文字与 flow 组合，尚无 runtime binding/作者命令。它呈现 GLB JSON 中 Head 的局部参考 TRS，不冒充实际求值 pose。

| 实际身份 | UUID / 状态 |
| --- | --- |
| 本域专用 component / 草稿版本 1 | `c8bdd097-2862-4d7e-926d-887a559bd5ff` / `Zircon / Character / SkeletonBoneReferenceDetails` |
| main instance | `c888ed9b-7d6f-40f9-a87e-f52437abc603` |
| 设计状态 sheet board | `57d782e2-1b4a-4f57-89e1-61a822458338`；1280×800，**不是产品窗口或尺寸验收** |
| 选中 Head 实例 / 320 宽 | `3c8b98b5-31c0-40c1-a037-f3a575d15319` |
| 空选择实例 / 280 宽 | `3b983047-deac-4567-aaf9-a18a5bf2bce7` |
| 中文与实际长路径实例 / 420 宽 | `59fff02c-f27b-4892-9f27-919f06b9ea79` |
| 最终独立 API 读回 | 12:32:23Z，rev18/vern0、318 objects、79 flow frames、1 component、3 copy heads / 144 个 shapeRef 对象；HTTP200；公共库仍 rev0、0组件/样式、isShared=false |

源 `wolf_basic.glb` SHA256 `655da121afa8768c13a3b2aa172e6875c716d06e352110bf68eaa4f34c9fec6e`，skin0/joint7/node7/Head，父显示名 Spine_4。`gltf:skin:0:joint:7:node:7` 是绑定资源 revision 的设计键；生产名称预期 `Node7:Head` 来自原 importer 源码，尚非运行回读。不能把显示名、joint ordinal 或路径 hash 当成跨重导入不变的 asset UUID。TRS 尚无 Engine 坐标转换或 pose generation。

真实失败和修正均保留：rev6 初图 flow 倒序、中文覆盖在 API 文档中正确但画布仍显示英文；首次 plain-JSON reorder 的 rev7 请求未改变层顺序，fresh 重开断言失败。按 Penpot `layout_data.cljc:419` 的普通 flow 反向画布栈及官方 Transit UUID/关键字类型修正，在 rev8 保存并重开确认顺序；中文仍不显示，不能仅凭 API JSON 判通过。随后进入实际原生文字编辑器，读到 `Bone` 而后端为“骨骼”；实际键盘编辑并提交九项中文/路径覆盖，逐次修订10～18，观察到 HTTP200 保存。关闭原 context、fresh context 重开后再次核对原 componentId/shapeRef 和文本，最终画面已显示中文与完整真实路径。

最终 [重开画面](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-native-bone-inspector/native-text-overrides-fresh-reopen.png)、[原生编辑及重开回执](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-native-bone-inspector/native-text-overrides-receipt.json)、[最终身份读回](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-native-bone-inspector/final-readback.json) 与 [12张逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-native-bone-inspector/visual-review.json) 均保留。实际回执完整记录九项原生编辑和九次 HTTP200 保存；失败/部分修正/最终重开图分别登记。原 Retarget board hash 仍为 `bb9d616aa2ad6c1b6c60f95233e20ff3caa75f6a7b409cd9d67a46a6b7a7a9ef`，与 rev5 回执一致。没有覆盖旧失败图，所有 browser/helper 已退出。

UE 细部差异继续 open：`SkeletonEditor/Private/BoneProxyDetailsCustomization.cpp` 使用 Bone / Reference / Mesh Relative 三种变换显示，`SAdvancedTransformInputBox<FEulerTransform>` 明确 `UseQuaternionForRotation(false)`，有 local/world、scale lock、copy/paste/reset 和单 slider 事务。本组件仅显示 local reference Quaternion 源数值，缺这些模式、Euler 转换、共享 axis input、focus、undo 和运行资源。因此它不代表 UE Details 或 Skeleton 工具完成；组件颜色/字号仅投影既有 tokens，尚无公共 native style 引用。官方导出、正常 EXE、真实预览、900×620/640×520/实际150%DPI、弹层/焦点/首尾滚动和≤1逻辑像素双端误差仍 pending，`passed/accepted=false`。


## 15. 九族归属与真实资源关联打开合同

Parent 已修正 overview 的旧分组：Static Mesh/Texture 独立工具属于资源会话 `01a16d53-60db-7d02-b03e-0042f277783d`，本域保留 Skeletal Mesh/Skeleton/Physics Asset。Retarget/Control Rig 及动画、材质专用表面归属不变。CS-D02 与资源域 RT-S06 共同消费 Parent 单一 PreviewSession/GPU image/media/input/frame 合同；资源与角色仅提供各自实际 subject 数据，不复制 backend，不以 SVG 图替代真实预览。

本轮沿已有资源打开链核查，未修改公共源或运行后台：

| 实际源点 | 已核实行为 | 本域消费及缺口 |
| --- | --- | --- |
| `asset_workspace_state.rs::asset_type_id_for_locator` | catalog item 的 kind 直接映射 AssetTypeId；Mesh 统一为 `mesh` | Static/Skeletal Mesh 的真实分类/Toolkit 选择须在现有 catalog/registry 形成单一规则；不分别抢占通用 mesh 注册 |
| `builtin.rs::builtin_toolkit` | 内建 match 提供 UI 与 AnimationSequence/Graph/StateMachine；未含 Mesh 或 AnimationSkeleton | 此结论限内建路径；Physics 已有扩展 Toolkit 继续复用。缺少本域独立 Toolkit 时不以其它资产工具代替 |
| `asset_event.rs::OpenAsset` | 查实际 indexed type、registry descriptor、command/enabled，再用规范 AssetToolkitOpenRoute 开视图并聚焦 | 材质槽/关联资源用目标真实 locator；ActivateReference 只导航资源。开视图/状态文字不等于完成域加载或 resource ready |
| `gltf_animation_subassets.rs` | SkinN Data 已有 joints、skeleton_asset 与逆绑定子资产引用，另有 typed Skeleton | 使用既有引用及 catalog 身份；不借用 Sequence 或临时 props 保存骨架/Socket |
| `mesh/metadata.rs`、插件 `mesh_skin_assets_by_mesh` | MeshSkinAsset 当前仅含逆绑定矩阵；按 mesh index 保留首个 node skin | 不能从矩阵数/名称猜 skeleton，需 node/mesh/skin/primitive 完整关联；多义绑定交公共资源/Runtime owner，不静默错误绑定 |
| `gltf_labeled_subassets.rs` | primitive 绑定含实际 mesh/material reference | Material 属材质 owner；Texture 属资源 owner。真实 registry/Toolkit 和 focus-return 仍未运行验证 |

独立读取相同真实 GLB JSON：Wolf Node48（Greyboy）→Mesh0→Skin0→Skin0/Skeleton、Material0；Knight Node23～31 对应九个部件，分别是 Mesh0～8、Skin0～8，全部 Material0。原 joint ordinal/key 与父子数据保留；这些是绑定 source revision 的数据身份，不是已解析的运行 URI 或跨重导入稳定 UUID。本轮没有解码 compressed accessors、导入 Engine 或创建新 fixture。

具体 [源关联/导航合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/cross-domain-navigation/source-and-navigation-contract.json) 保存13项实际源哈希、1+9项 node/mesh/skin/primitive/material 关系，运行 locator 明确为空。9项待执行场景为：static/skinned 分类、同mesh多skin不误绑、Knight九部件、材质关联打开、纹理关联打开、缺失/禁用目标、源代际变化后的焦点返回、共享preview最后消费者释放、正常EXE证据。各项均未运行，passed/accepted=false；不能将此只读结构核查计为产品测试。

本域提供源文档/资源revision、typed选择/焦点键、实际primitive/material/skin关系和失败诊断；Parent负责registry/Toolkit host、路由完成/焦点恢复与共享preview生命周期。材质或资源关联编辑返回时核对源文档代际和对象身份，禁止按旧行号恢复；源已关闭或重导入时重新解析或报告过期，不使旧pose/frame计为新资源ready。动画时钟仍消费原owner，开关联资源不创建角色播放时钟。


## 16. Typed subject、已提交pose与真实frame最小适配

Parent 最新明确本域提供 mesh/skeleton/pose、PreviewSubject、picking 和资源输入，动画 owner 提供实际时间/参数/evaluator，Parent 提供单一生产 secondary backend、Renderer image/input、frame-ready/旧帧拒绝及 teardown。本节交付具体未应用补丁，**未修改公共源码/入口、未增加 backend/clock，也未宣称运行预览已接通**。动画专责3个main身份沿原合同；早期rev5/本域rev8与Parent后续rev12观察分别保留，公共库未发布不能计复用。

| 改动/接线 | 精确最小范围 | 责任与当前结果 |
| --- | --- | --- |
| Typed subject 与源替换 | `ui/preview_scene/preview_subject.rs` 三项locator字段改为现有 `ResourceLocator`，由共享owner传入非零 `subject_generation`；同locator重导入/源revision替换必须新代际，不藏入字符串parameter override | Parent持有公共整合；[Editor候选补丁](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/typed-preview-adaptation/preview-subject-and-closed-setters-lf.patch) 尚未应用，SHA256 `1d0cce505e6e30e9cf81b3076d5abc39e748188285ad961f2896382096f9c836` |
| 关闭后命令拒绝 | `preview_scene.rs` 两setter先检查active_session，再判断相等/参数；现状成功close后 `set_subject(None)` 和同值playback会提前返回 `Ok(false)` | 同一Editor候选同步原test helper，并新增关闭后拒绝/同locator新subject代际重新转发2项回归；不修改Fake为生产backend |
| 已提交pose来源 | `RenderSkeletalPoseExtract` 加实际 `animation_generation`；`LevelSystem::build_render_frame_extract` 在既有同World代判断内复制 `frame_state.animation_generation()`，复用原 `AnimationPoseHandle`/entity/skeleton | [Runtime候选补丁](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/typed-preview-adaptation/runtime-committed-pose-generation.patch) 尚未应用，SHA256 `0ffa191542312014b9cbd750169624e2910ece9e87ecb19e2531332e090b3170`；不新增counter/evaluator/ABI |
| Runtime最小调用方 | 1个真实producer、4个其余literal调用方同步；既有LevelSystem提取测试追加第二次真实pose提交，核对递增代际及旧extract/pose保留 | 6文件候选均已语法解析，未类型检查或运行；renderer cache/morph/velocity测试里的合成资源句柄不证明资源加载 |
| 帧关联与呈现 | 复用 `GatewaySessionIdentity`、`RenderWorldSnapshotHandle`、`EditorRuntimeFrame` 和原image/capture协议；当前capture帧只有尺寸/generation/pixels，缺文档/subject/pose/resource关联 | Parent必须把本次提交的真实World/pose/资源闭包与实际Renderer/present帧关联；两份候选不提供或伪造frame-ready producer |
| Picking | 复用 `EditorRuntimeViewportPickRoute::capture_at_identity` 的原端点/票据、request frame/input sequence 和实际result；消费时再比当前doc/subject/frame | Route钉住原端点不等于自动拒绝同session换subject后的旧结果；`entity/instance/subobject`不是默认BoneSlot，需明确帧绑定目标map |
| Secondary session与资源释放 | 复用App的 `RuntimeSession` 权威、gateway和可重试retire lease语义；现有 `AppPlaySessionFactory` 仍是Play场景创建，不是交付的通用角色preview backend | Parent接通真实隔离session；不经主PIE或强造PlayInstanceId。最后消费者先停止新work、取消原route picks、失效旧帧并解绑/释放image，再retire；失败保留Retiring owner待原入口重试 |

最小域输入为实际catalog根locator、source revision、完整node/mesh/primitive/skin/skeleton/material引用和实际secondary-session entity/ResourceId映射；沿第15节Wolf 1项/Knight 9项真实源关联，不猜运行URI。BoneKey采用真实Skeleton身份/revision与已验证joint slot，源node仅作诊断，不能用显示名/当前可见行号。选择/hover与source subject分开；选骨骼不创建world、播放时钟或作者历史。

Pose读取原提交快照，必须核对entity、skeleton ResourceId、实际加载Skeleton的槽位/名称顺序。动画owner负责time/参数与Clip/Graph/StateMachine evaluator；Editor不再采样一套pose。当前 `AnimationPoseSource` 仅Clip/Graph/StateMachine，无Reference变体；bind/reference pose须有真实生产路径和明确证明，不能把Penpot JSON TRS伪标成Clip或以空pose宣布ready。

Parent的最小帧关联合同需要9项：完整gateway身份、实际doc/source revision、viewport/物理尺寸/DPI、subject代际、World身份/世界代、实际提交animation代、实际Renderer frame代、资源ready/error闭包、image/release身份。它们不是互换的计数器：UI Surface、World mutation、pose commit、render frame分别核对。该表是共享owner待整合字段，不是已发布的新私有协议。

[精确源/域输入/8阶段生命周期合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/typed-preview-adaptation/typed-subject-pose-frame-contract.json) 保存30项实际源码哈希及6项open缺口。[Editor候选v2记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/typed-preview-adaptation/proposal-v2.json) 与 [Runtime候选记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/typed-preview-adaptation/runtime-pose-proposal.json) 包含before/after-normalized哈希和检查。首次Editor diff因Windows文本写入转换CRLF而应用检查失败；首版patch/receipt保留，修正版以LF字节保存，未松开空白保护。修正版与Runtime候选 `git apply --check` 均exit0，三份Editor候选经rustfmt解析/格式化，六份Runtime候选仅语法解析并保留其它格式。9个公共目标源字节未变；这些检查不等于类型检查、Jenkins或Rust测试。

候选由Parent/Runtime owner确认并集成后，再按实际package/defaults off/features/target/filter/profile声明托管测试。Editor最小范围为 `zircon_editor` lib、features=[]、`ui::preview_scene::tests::`；Runtime提取回归为 `zircon_runtime` lib、features=native-host,animation、`level_system_render_extract_uses_world_direct_path_and_merges_animation_poses`，renderer受影响范围再由原owner按真实feature接续。当前未执行这些命令，不复用旧GLB请求冒充新候选验证；原CPU/插件/导出/EXE/四尺寸及实际150%DPI双端验收仍pending，passed/accepted=false。

## 17. CS-G10最小导入策略修复与托管入口边界

按Parent“角色第二次接续”明确交接，本域已直接修改两个原owner文件；编辑前实际diff为空且两个SHA均与Parent记录相同，未碰其它域改动。`zircon_plugins/gltf_importer/runtime/src/lib.rs` 的原stable required-extension白名单仅增加 `EXT_meshopt_compression`，沿原 `decode_gltf_source_with_required_extensions` 和已有有界 `decode_meshopt_views`；没有新增decoder、放行未知required扩展、改Runtime或翻preview capability。`src/tests/source_snapshot.rs` 的原负例改为真实未支持的 `KHR_draco_mesh_compression`，仍调用生产 `import_gltf` 并断言精确unsupported-required错误；原测试名/源快照语义保留。

| 最终候选 | SHA256 / 结果 |
| --- | --- |
| 原plugin `src/lib.rs` | `babdda2e6774bf3d0d5bd5e05b2e9e6e00065059993d8ad95acbac14445602ea` |
| 原 `src/tests/source_snapshot.rs` | `a076648ce83a20af5216a0b64fe27a443308c40a5e57813128f70bf8830cfadf` |
| 原角色integration两正例 | `115cfbe9720690a17c115aa5ea708b9d2e93ecbab425730e53db88ef3d4dd9b8`，本轮原字节未变 |
| [两文件最小policy diff](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/policy-fix-r4.patch) | `05796ac768260d73b7c0973c3fc014c72d3ea79293851b673d0064c65d42848a`；不包含先前新增的integration文件 |
| [三文件最终guard](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/change-guard-r4-final.json) | passed，0 errors/0 warnings/0 suppressed，patchDigest=`b53dbca4d60b86fad4c8743abc88913dcaa6b650e52d252717010836cf631b8d` |
| [最终格式检查](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/format-check-r4-final.json) | `rustfmt --edition 2021 --config skip_children=true --check` 两个修改文件exit0；不递归改其它module |

首次格式检查发现新expect_err说明须换行；[原exit1记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/format-check-r4.json)及初次guard保留。只修该换行后执行最终检查，未全局格式化。第13节及此前static-final中的“生产plugin未修改/遗漏meshopt”保留为修复前观察，不能当作当前候选状态；CS-G10现在是**源码最小修复完成、Rust执行未通过**，没有资源ready或GPU证明。

新增[候选r4](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/source-and-static-checks-r4.json)继续原request/operation `ue-character-source-20261024-01`，透明追加3个ownedPaths、before/after/依赖哈希和当前guard；旧v2/source-capture拒绝记录不覆盖，不用新ID重试。两项待执行范围均为独立 `zircon_plugins/Cargo.toml` 下 `zircon_plugin_gltf_importer_runtime`，defaults off，features=`runtime,zircon_runtime/native-host,zircon_runtime/animation`，locked/jobs1/test-threads1：第一项仅integration `character_skinning_sources` 两正例，第二项仅lib filter `source_snapshot_tests::importer_rejects_required_extensions_outside_its_stable_policy`。本次规划显式profile=`test`；早期模块route记为dev的历史命令不改写为新执行证据。全部准确argv已保存在r4中，运行方在实际提交前仍须声明完整命令。

本轮只读校验实际安装指针与capsule、selected client摘要并调用该client parser，未启动管理请求/源码捕获/Cargo。实际capsule=`cdd14500fa39088ae2c9870f62825acc2e526ba647d24d77cd6287722b5d0f31`，client SHA=`5e10bc46daaa2e891d373d2dceb66e6497d4a477d5a9ff9a1f5bda96b45fcc9a`；两个精确argv**都被拒绝**：`The plugin manifest permits only the material_graph_ single-package lib test recipe`。这不同于Parent早期baac胶囊的材质/AI观察，均保留其时间身份；新拒绝不是Rust编译/测试失败或服务不可用结论。

[原始parser证据](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/cargo-scope-parser-r4.json)交原Jenkins实现owner `01a12af6-7511-70d1-a452-61b80678eb56` 接续受限glTF单包的integration与lib配方及原request source-capture-ready窗口；本域不修改管理入口。现 `dev-module-build.ps1` 无manifest selector且调用旧canonical entry，不能把root workspace `-Package`当此插件独立workspace的等价命令。待owner回执后沿其正式manifest-aware托管入口执行，不借material_graph_ filter、不新identity/源码快照、不绕锁或本地Cargo、不冻结所有写者。

本轮没有Penpot/Engine/桌面输入或服务操作，原3份ZUI及9个preview公共目标源未改。所有插件/Rust/normal EXE、真实骨骼姿势/拾取/teardown、官方导出及双端四尺寸/实际150%DPI验收继续pending，`passed/accepted=false`。

本轮r4正式交接结果：原Jenkins owner投递exit0，ACK `01a16e11-473d-7883-a96f-51868967cabb`，仅证明排队，未证明已读/source-capture-ready。Parent投递仍exit1 / `-32600` / 队列满100条，无新message ID、未通知；[两条实际消息及原始结果](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/candidate-r4-queue-results.json)保留，不清他人队列或重放旧请求。

## 18. 共享原生实例的完整源骨骼树与Head内容设计

在原文件/page继续设计，没有另建文件、主件、主题或preview backend。[13:25:20Z认证回读](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/current-readback.json)确认公共库已经由原owner实际推进到rev1、`isShared=true`、41对象、4组件，colors/typographies仍0；本域开始时rev18。此次依据 `SSkeletonTree.cpp` 的New/Search/Filter横向关系、`TreeRow` 现有28逻辑像素行高与12像素深度步长，制作具体骨骼内容和筛选状态。Search独占剩余空间；New/Filter为原生Button实例。本次Head为parents-shown设计变体，不假定它是UE默认过滤模式。

| 实际可编辑身份 | UUID / 尺寸 / 用途 |
| --- | --- |
| 完整源骨骼树板 | `e1f65e1b-4fee-463f-bd67-ae5a5b989524`；448×1568，48个Wolf skin0 joint，实际源顺序/父关系/稳定key |
| Head内容review板 | `2f389980-f6e7-4f28-854d-2308a17dc31d`；1280×800，树400/既有只读详情320/来源与缺口说明列；不是完整Toolkit或假viewport |
| 公共TreeRow | component=`6ddc248b-bd35-80c5-8008-d6c8c7a79944`，main=`6ddc248b-bd35-80c5-8008-d6c8c452bf0c`；57个跨文件Copy，48完整树+9匹配/祖先行 |
| 公共Button | component=`6ddc248b-bd35-80c5-8008-d6c8c01096f3`，main=`6ddc248b-bd35-80c5-8008-d6c8bd9b057f`；2个Copy |
| 公共SearchInput | component=`6ddc248b-bd35-80c5-8008-d6c8c3ce892e`，main=`6ddc248b-bd35-80c5-8008-d6c8c0f9c196`；1个Copy |
| 原有本域详情 | component=`c8bdd097-2862-4d7e-926d-887a559bd5ff`，main=`c888ed9b-7d6f-40f9-a87e-f52437abc603`保持；新添一个同文件Copy，旧3个状态实例不动 |

新板使用flow，保留实际componentFile/componentId及每个后代shapeRef；按Penpot实际容器克隆合同重映射shape/parent/frame关系。语义child顺序按真实flex实现保存，fresh重开后逐项核对，无顺序不符。原生右侧检查器明确显示 `Copy / Head / WorkbenchTreeRow` 及末行 `Copy / Back_Leg_Tip_R / WorkbenchTreeRow`；不只凭自己写入的metadata认定实例成功。实例覆盖限于文本、宽度、深度、Head选择色`#0070E0`、叶chevron及本域未支持eye/lock action-slot隐藏。公共主件未改，也未复制到本域成为第二套main。共享PropertyEditorRow已存在，但当前slot/typed值合同不满足Euler/Axis，未强行以临时props替代；只读Quaternion组合保留第14节局限。

源来自真实 `examples/woc/assets/m8/models/creatures/wolf_basic.glb`，SHA256=`655da121afa8768c13a3b2aa172e6875c716d06e352110bf68eaa4f34c9fec6e`。记录48个实际skin joint slot、source node、父node、名称和depth；Head=skin0/joint7/node7，parent=`Spine_4`，完整树末行=`Back_Leg_Tip_R`/joint47/node39。`Spine_2001`、`Headtip`等源名称按原字节保留。[源/实例合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/hierarchy-source-contract.json)包含精确key和映射。这里只解析GLB JSON，未执行meshopt accessor decode、import或真实pose/mesh加载；生产BoneKey仍须实际Skeleton ResourceId+revision+验证过的joint slot，不以源显示名/行号或猜测UUID代替。

[真实authoring收据](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/authoring-receipt.json)记录link-file-to-library/update-file/reorder均HTTP200、保存关闭原context、fresh context重开。同轮API在UI退出前读到rev20；之后原生文字布局保存到rev21，review板的1×PNG export设置到rev22。[最终只读audit](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/final-readback-audit.json)于13:46:11Z实际读回rev22/vern0、1538对象/102flow/1组件，核对全部61个新Copy head及其后代引用、flow顺序、48个源joint、真实linked library关系；原317个非根对象全部hash保持，公共库仍rev1且完整响应SHA与编辑前相同，theme源哈希不变。该audit只证明设计数据/引用保持，不是插件/Rust/产品passed。

[8张逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/visual-review.json)包含保存、fresh重开、两次原生实例检查与两张导出失败图。Head过滤、选择、只读数值和来源可读；完整树图在64%画布缩放下顶部被边界/悬浮工具栏遮挡，明确记partial，不能作为首尾滚动证明。77%/64%均为Penpot缩放，不是系统DPI；未做≤1逻辑像素、900×620/640×520、实际150%DPI或产品对照。原中文/空选择/长路径三态继续保留第14节证据，没有把当前英文内容板当作所有状态覆盖。

官方PNG导出实际失败：原生 `Add export` 为本域review板保存PNG/scale1，点击 `Export 1 element` 后进入Internal Error、45秒无PNG；诊断收据确认 `/api/export` HTTP404。原生错误报告点击后 `download.saveAs: canceled`，未取得report.txt，不假报错误报告内容。[诊断收据](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/export-debug-receipt.json)和失败图保留。随后读到并核对Parent“Actor第二次接续”既有诊断：当前 `serve-penpot.mjs` 将全部`/api/`送16060，缺独立exporter路由；官方nginx参考将`/api/export`送`PENPOT_EXPORTER_URI`。复用该公共owner故障，不再重复点击或修改/恢复服务。PNG exporter、插件canonical ZUI导出、生产EXE加载是三项独立未完成链路；截图不是官方导出PNG。

真实设计后的[独立Skeleton源owner具体提案](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/source-owner-proposal.json)给Parent/原公共pipeline owner：拟新增v2 `zircon_editor/assets/ui/editor/character/skeleton_workspace.zui#WorkbenchSkeletonWorkspace`，身份为`res://ui/editor/character/skeleton_workspace.zui`。该路径当前**不存在且未注册**，未创建空壳，也未把content review板冒充完整Skeleton窗口；待canonical源owner映射/host注册，复用上述实际shared UUID。Tree查询/展开/选择是transient状态；Socket/虚拟骨作者事务、UE Euler/space/Axis、Undo/redo/save、骨骼/碰撞体picking及资源关联打开须具体生产模型与命令。父级承接CS-D01 collection、CS-D02唯一backend、CS-D03公共文档、CS-D04 Toolkit/registry、CS-D05源映射/样式/导出；本域承接typed item keys、作者数据和域事务。时间/evaluator仍动画owner，无复制clock或backend。

本阶段没有正式插件roundtrip/受管Rust/正常EXE运行或原生capture，不将API或Copy检查升级为通过。第17节候选3源、guard和原Jenkins身份保持；原Retarget/ControlRig/Ragdoll资产、9个preview公共目标源和catalog/tokens/Starship未改。无就绪EXE/资源闭包指纹，尚不提交虚假的桌面捕获申请。所有窗口族 `passed/accepted=false`，Skeleton作者模型、生产preview、Retarget/Rig求解、Physics编辑及全部双端状态/尺寸/DPI验收仍open。

阶段结构复核：四份文档UTF-8及本地链接有效，产物JSON和3原ZUI/catalog/tokens/Starship的TOML解析通过；r4三个Rust源哈希保持，无新增编译。preview合同30项输入中仅Parent交接与animation-shared-controls两个外域文档更新，其余包括9个拟改公共Rust源均匹配，不覆盖他人记录。[本次复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/structural-review.json)保留时间和精确hash。所有自有Node/Chromium退出，Win32_Process按本阶段命令路径核对无活动进程；exec session编号不作PID。新设计里程碑官方投递一次仍因Parent队列100条失败，无message ID；[本次消息原始回执](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/hierarchy-queue-result.json)保留，不清队列或循环重投。释放空间0 B，保留本阶段实际设计/失败/可复现helper；任务生命周期仍active。

## 19. 四个公共主件的source与必填value槽核对

已读取Parent顶部“公共库最新观测”和 `parent-01a16d4e/ai-followup-03.json`，接续实际4个主件，不再引用rev0作为现态。13:55:09Z认证只读API再次读回Shared rev1/isShared=true/41对象/4主件，colors/typographies=0；本域仍rev22/1538对象、实际linked library保持，原60个跨文件Copy身份无不符。[本轮source/slot回读](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/property-contract-readback.json)保留source-file/component/main/page/library revision及shape层次、sourcePath/sourceNodeId/controlId；未改任一Penpot形状、Shared或原始源映射，无须重复捕获此前未改变的8图。

| 主件 | 本域实际最小内容 | 结论 |
| --- | --- | --- |
| WorkbenchTreeRow | 完整48行源joint、Head过滤9匹配/祖先行 | 已接57个linked Copy，前轮保存关闭fresh重开有效；真实collection/scroll/选择绑定仍open |
| WorkbenchButton | New/Filter | 已接2个Copy，源引用保持；生产命令/菜单/焦点未接，不能按装饰按钮算完成 |
| WorkbenchSearchInput | Head查询设计状态 | 已接1个Copy；原query/expanded/selection设计覆盖记录不变，未伪称真实过滤逻辑 |
| WorkbenchPropertyEditorRow | 下一最小内容为只读 `Bone Name=Head`、`Parent=Spine_4` 两行 | 尚未实例化空required槽；当前没有已发布的兼容value控件，具体依赖如下 |

PropertyEditorRow component=`6ddc248b-bd35-80c5-8008-d6c8cab1e7d0`，main=`6ddc248b-bd35-80c5-8008-d6c8c84d848f`；实际4形状中，`value_slot` 原生shape=`6ddc248b-bd35-80c5-8008-d6c8c88eaacb`，无子项，metadata确指向 `workbench/inputs/property_editor_row.zui` 的原 `value_slot`。源SHA256=`bf29d5740c317b486ec5e6f2ef3faf06f7e800f6ad1dfcd773f0b403e87ef6ce`，root control=`WorkbenchPropertyEditorRowRoot`，typed `value`槽 required=true/multiple=false/kind=linear，仅接受Checkbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle的既有Workbench组件。已用TOML原文件解析复核该列表及catalog一致；当前4个已发布主件与该列表交集为空，SearchInput并不被接受。不能把Text/Container或SearchInput塞入必填槽来声称接通。

最小公共依赖是发布**已有** `zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/field.zui#WorkbenchField`，URI=`res://ui/editor/components/workbench/primitives/inputs/field.zui`，source SHA256=`d57c660ff150361bf86cd0aa77a8fb4a1650de46b699ae17eb12db40b333f98a`，root control=`WorkbenchFieldRoot`，binding namespace=`WorkbenchField`，catalog已登记。不是新增组件/主题或扩展slot accepts。其Penpot component/main尚无发布回执，保留null，不从其它旧文件同名main推导UUID；公共owner已获Parent9项基础组件发布范围，本域不编辑公共库/目录。

两个真实只读字符串行不依赖Euler/Axis，可在Field的canonical identity/slot/覆盖合同交付后单独推进。catalog当前仅公开 `value`、`placeholder`；不能假定只写 `editable_text=false` 就能禁止Runtime输入。现有 `data_sync.rs` 同步typed editable/read_only及focusable/clickable/input flags，Parent/原公共owner需交可支持的只读覆盖/绑定合同；本域保留现有可读的详情组合，不伪造可编辑作者模型。后续在受控板组合原PropertyEditorRow Copy及唯一 `value` Field Copy，记录实际子实例、slot位置、完整shapeRef、覆盖、默认分配/flow与导出guard，再保存关闭fresh重开。不会先做两个空白属性行计作真实内容。

[最小Field/两行消费提案](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/property-slot-proposal.json)给出已存在源和catalog映射、真实Head key/source revision、只读命令边界及明确缺失UUID。源host `zui-layout-component-hosts.ts` 的现有PropertyEditorRow review recipe同样显式嵌入NumberField，支持“必需真实value子组件”的判断；其fixture和新canonical-library单元测试源只读查阅，未执行或冒称部署通过。CS-D01的三基础控件试接已有实际证据，CS-D03最小属性内容依赖Field/只读合同，CS-D05 canonical sourceHash/slot-preserving export、override/reset和两族A/B样式传播各自仍pending。

本轮仅只读API/source及本域合同更新，未新增设计截图、运行插件/托管Rust/EXE、启动host/服务或占桌面。沿原request/失败/guard和owner，不重新投旧消息或改验收门；所有产品 `passed/accepted=false`。

此次slot新事实已按官方CLI投递一次，Parent队列仍100条满/-32600，exit1，无message ID或已读；[实际消息与回执](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/property-followup-queue-result.json)保留，不循环重试。前三项已有设计证据复用；第四项等待的是明确的兼容Field identity/只读合同，不能把当前不可填的required槽判passed或accepted。

本轮结构检查、原3业务ZUI/catalog/theme与r4三Rust源哈希保持，未重复未改变的guard/Rust/截图。[初版结构记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/property-followup-structural-review.json)中两项“完整响应匹配”错误按并发完成数组下标比较，误记false；[更正r02](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-hierarchy-design/property-followup-structural-review-r02.json)按既有file-ID标签、command和完整SHA唯一关联，本域/公共库/连接响应均与第18节完全一致。原检查保留并说明原因，不把此检查错误解释成数据变化；无需重新API或抓图。后续helper增加fileId日志，原观察不覆盖。自有进程退出/临时0 B，保留证据，释放0 B；任务active，产品仍pending。

## 20. 原Jenkins owner实读对账与当前dev范围

收到原实现/监控owner的同链只读对账后，本域直接读取 `E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/ue-character-source-20261024-01/reconciliation.json` 和summary.md，核对完整SHA256=`cfd2769147be075fe2bc8820d6bec32737f637f80a406f12fe51e07fafefbc83`。其recordedAt为13:33:29Z；这是对原13:17 r4的实际读取证明，不是用queue ACK替代已读。本域另实际重算14份当前源码/依赖及r4 candidate、guard、format和原query文件哈希，全部与该receipt吻合；只复用对应源的静态证据，没有重跑检查或Rust。[本域实读收据](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/owner-reconciliation-readback.json)保留观察时间、14项哈希与引用，没有复制外域源/Records快照或认证内容。

| 层次 | 当前事实与边界 |
| --- | --- |
| 源候选 | 三ownedPaths仍为plugin lib=`babdda2e…`、source_snapshot=`a076648c…`、角色integration=`115cfbe9…`；原guard=`b53dbca4…`/format0按源复用；3项Rust未执行 |
| 原身份/输入 | request=operation=`ue-character-source-20261024-01`，session=`01a16d52-6b1e-7543-82f0-4f05ca92ad58`；r4是handoff，不是public Cargo envelope；原v2仅一ownedPath/integration=`019e238b…`已旧，保留5份AI漂移失败 |
| 收件 | 原query SHA=`7c0a874850891c72f76940f347ce75ad3cf22512ee87952de61ca13307b788e3`仍pending/ordinary_entry_unproven；原ownerro snapshot无匹配授权/request/operation及delivery marker，不手写grant；不等于证明从未投递 |
| Records限制 | 全文引用检索10秒到界中断，七域精确候选键后查0不关闭该未完成范围；本域不另扫服务端或重跑query；server/source-preparation/native receipt仍null |
| 路由 | 原owner核对cdd14500…的11 sealed成员匹配；client472–482/server4039–4045仍仅material_graph_材质lib，glTF不可用；本域不改旧sealed、parser或管理入口 |
| 修复owner | 沿既有 `ue-plugin-workspace-entry-20261024` owner资格/发布新的受限immutable实现，涵盖canonical manifest/package/argv、nested lock/.cargo/path deps、source闭包、pool和receipt；不以root -p、只松parser或本地Cargo替代 |

当前profile遵循最新Parent **dev**；r4显式 **test** 提案及原因、对应两组拒绝回执原样保存。原owner另对Parent两组dev精确argv做纯parse，同样build_scope_policy拒绝；两种profile均无执行证据。本域未重新parse/query。最终同身份recipe/profile由执行owner在实际运行前统一，并完整声明packages/features/target/filter/profile/命令；本节是待执行范围记录，不是编译声明或已提交命令。canonical manifest=`E:/Git/ZirconEngine/zircon_plugins/Cargo.toml`，cwd=`E:/Git/ZirconEngine`，仅单包 `zircon_plugin_gltf_importer_runtime`、defaults off、features=`runtime,zircon_runtime/native-host,zircon_runtime/animation`、locked/jobs1/test-threads1、物理E:/cargo-targets；分两次有界运行integration `character_skinning_sources` 两正例及lib精确filter `source_snapshot_tests::importer_rejects_required_extensions_outside_its_stable_policy`。

本域实读收据内给出**未提交的同身份三路径/范围delta**及原owner的dev argv，明确不是public envelope、没有source/cargo admission，也不伪造新资格或请求ID。正式route/recipe ready后由执行owner将三路径输入透明追加同身份，保留旧v2/r4/漂移记录；本域不在未就绪入口盲重提。14项候选依赖核对也不等于完整capture或writer闭包。

capture-ready继续false。原18段pin累计72.309秒，其中主仓两轮30.945/32.168秒；不是全capture wall time，当前完整耗时未知。原capture涉及9roots/full-file-index-v2，主仓52129、zr_vm8011及7嵌套依赖/Git配置；五份AI authoring asset/edit/error/topology及tests/authoring漂移照留，不能归因到某作者或当完整writer集合。当前完整member/writer/资源准备和正式接收仍缺，暂定最长10分钟仅为**正式ready后**Parent协调的低置信度准备预算，须明确起止/超时释放；当前不启动freeze或全局停写。持续native guard可能到EOF，capture完成不能承诺全体立即任意写。

复用原owner13:26健康观察，host lifecycle/兼容与各域EXE门分别open。本轮仅读源/receipt和更新本域文档/清单：0 query/parse/Cargo/capture/恢复/改锁/grant/源码副本/共享cache清理/例行ACK，释放0 B。旧设计rev22及四主件试接合同保持；CPU候选和健康ready均不等于GPU/生产preview/normal EXE通过，窗口族 `passed/accepted=false`。

## 21. glTF限定入口交付与角色owner三路径审阅

原能力owner沿 `ue-plugin-workspace-entry-20261024` 完成交付后，本域实际读取并核对其 `gltf-character-extension/result.json` SHA256=`bfbc7d9e7fbe711383b9e1ba92f66264df60ff1db25ffb45333627ed9e112760`、summary和 #402 outer native SHA=`87f2498d703bca9cc7b43b85d3f87a945e0fefe749f15d51e19236381e2526f8`。当前installed指针实际选中capsule=`28698b9e8dcd4972e372ccc17969d3b3f28c2389327037636f805c5b58efbe4f`，manifest自哈希及5个实现/6个client共11 sealed成员匹配；entry SHA=`0b0645217413afbdad32cdb763011a0e660b49970d12419ce372fa3667244a82`，driver/runtime仍原 `fc553228…`/`start-1792836412672596900`。仅import该pinned entry并调用纯 `parse_cargo` 两次，没有wrapper/query/runner/Cargo调用、source capture或服务动作。[本域交付消费收据](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/gltf-route-delivery-readback.json)包含精确ID、argv、recipe和11项哈希。

| 当前限定配方 | 实际纯parse与待执行范围 |
| --- | --- |
| Integration | `zircon_plugins/Cargo.toml` / `zircon_plugin_gltf_importer_runtime`，defaults off，features=`runtime,zircon_runtime/native-host,zircon_runtime/animation`，**test** profile、locked/jobs1/test-threads1，仅 `--test character_skinning_sources` 两真实CPU正例 |
| Lib负例 | 相同manifest/package/features/profile/资源约束，仅 `--lib source_snapshot_tests::importer_rejects_required_extensions_outside_its_stable_policy` |

两者在当前pinned client均成功，原owner的selector→parser→runner及manifest环境资格回执按实际hash复用；第17节cdd版本test拒绝和第20节dev范围/拒绝原样保留，不再作为当前route状态。最新user交付已明确profile=test，本域审核也选择此精确已资格配方；不继续以dev静默改argv或借root -p绕路。只有正式运行前的完整模块/features/target/filter/profile/覆盖理由声明才是执行声明；本轮纯parse不等于编译或3项Rust通过。

6份helper native原文件与Home archive均由本域重算SHA/字节核对。#401回归68 tests/199 subtests/1旧fixture排除的资格结果、漏ExpectedCapsuleSha256导致安装前拒绝的apply/outer root1，#402补齐后apply/outer root0全部保留；不把旧失败重标成功。complete/EOF/Job0依原资格验证回执，helper native不替代常驻host生命周期证明。11 jobs API/Home pin与仅management绑定变动、tests route恢复、健康ready沿同源交付回执复用，本域未再次调用服务。静态manifest闭包为15本地包/26路径边/24文件或缺失项/2 owning workspace，尚无Cargo解析或完整source capture证明。

本域作为原角色owner完成[同身份三路径输入审阅](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/gltf-route-owner-input-review.json)：原owner生成的 `character-r4-current-input.review-only.json` SHA=`350f253b9112ea00e6772a604835f11cd3d1a52141f643c31918e90eeab66ad8`匹配，request=operation/session/driver/runtime、checkout/full-file-index-v2、externalRoots与物理E:/cargo-targets保持。三ownedPaths/sourceHashes及15 dependencyHashes合计18项当前字节全匹配，包含主/插件manifest与locks；没有修改或复制该文件，没有重写v2/r4失败。最小CPU源和recipe审核已完成，**不等于正常准入或source capture-ready**。

当前阻塞由“入口不支持”转为明确的执行准备：按原source-readiness条件，原执行owner先确认当前完整writer/member和受管pool/资源，由Parent协调9个参与root的有界源码稳定时段，明确各条命令的开始/超时/取消和EOF后释放。9roots的原精确路径及条件存于本域review，不把三ownedPaths或五AI漂移路径当完整writer集合。600秒只作尚未激活的建议上限，不是Cargo总耗时或capture wall-time；历史72.309秒仅18段pin合计。持续输入guard保留到native EOF，不能capture完就允许全体任意写或无限冻结。时段/准备回执未到，本域不提交wrapper或开始capture；就绪后沿原身份正常准入，再分两次有界运行。

原owner安装后实际query仍exit2/pending/ordinary_entry_unproven，没有Cargo/server receipt；本域读取并校验该新query文件SHA，未重复query、不新grants、不换ID或恢复服务。PR44/45合并与本地main awaiting-main-sync按交付记录保留，不拉取/清理dirty或原工作树。Jenkins删除两个自有已终止helper实际释放16384 B属于**该owner**，本域释放仍0 B，未碰共享cache/未知锁/Penpot或桌面。CPU、normal EXE/产品/性能、host native与其它旧恢复门分别open，所有窗口族 `passed/accepted=false`。

本域已将实际审核结果和具体准备请求交原执行owner/Parent：原执行ownerqueue exit0、ACK=`01a16e47-5dc3-79d1-99ab-7ae217cf369d`；Parent仍满100条/-32600/exit1，无新message ID。两条[行动请求和原始回执](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/gltf-route-coordination-result.json)保留，ACK不等于已读/写者资源就绪/时段。请求覆盖两次命令各自准备、开始/超时/取消/native EOF释放；不是代提Cargo或新恢复，当前未激活freeze。本域停止重复投递和query，等待实际准备/时段回执后再正式声明并运行。

同轮后续复核捕获到installed推进，首版[结构检查](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/gltf-route-delivery-structural-review.json)实际报告两项同一指针漂移，原文件不覆盖。当前capsule=`20b3bf855a26a9516845cf7611b216a2808aef944022cd99554525a71a319c75` / entry=`4e95a18efca2dad76964a8ef1a5ac34c7f943825d5fcda4a471b5e27000ecf1f` / pointer SHA=`2615bb7863b20127dea8c8b7566c7e9793936a474e76765a96208fdf0ce2b9bd`，driver/runtime不变。实际定位到同一能力owner的 `prefab-editor-extension/result.json`（SHA=`2879b421a101f8f01bdafa4b3368388a2107a989f748c4fbf55585ba515f0855`）及资格/安装回执；该新版本增加限定prefab lib路由并明确保留原材质和两glTF路由，不是旧交付损坏或角色源码变化。

[Successor实际核对](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/gltf-route-installed-drift-r02.json)：capsule原文件哈希等于目录ID，11 sealed全匹配，仅runner和entry两个成员改变；client/runner的glTF条件AST相同，source_preparation/checkout_guard及checkout wrapper字节保持。两条角色精确test配方在当前pinned entry再次纯parse通过，18源码输入仍匹配，检查前后installed指针不变。本轮共4次纯parse，无wrapper/query/managed Rust调用。新资格native/health按有哈希的owner回执复用，不声称本域重跑或host生命周期通过；[结构复核r02](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/character-source-validation/gltf-route-delivery-structural-review-r02.json)明确保留首版失败并解释当前版本。原三路径identity/准备请求无需换ID或重投，capture-ready仍false。

## 22. Shared rev2 的本域消费核查（2026-10-24）

2026-10-24T18:07:07Z–18:07:33Z，本域复用原认证，在已分配 file `b4a1e3ce-9e6a-81fe-8008-d69d60adb48e` / page `b4a1e3ce-9e6a-81fe-8008-d69d60adb48f` 的隔离浏览器打开真实库管理器。Shared 实际为 rev2/isShared=true、原4主件、34 colors、3 typographies；不再把历史 rev0/rev1/style0 当作当前未发布状态。catalog 当前 SHA256=`4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5`，NumberField 为10 props，PropertyEditorRow 为原 required value 加 description/source/reset 共4 slots。它们是当前公开源合同；Field/NumberField 仍无已发布 native master，本域没有用 Text/Search 替代必需 value 槽。

原生 UPDATES 页明确显示 “There are no Shared Libraries that need update”，没有可点击的 Shared 更新按钮。本域未强写 syncedAt、detach、reset override 或改 Shared。关闭上下文、fresh 重开后本域仍 rev28/1963 objects/1业务主件，83外部copy及1559后代source refs有效，全部对象指纹未变。打开前、管理器关闭后、fresh 重开三图 PNG 字节完全一致。它证明旧设计与覆盖值保留，未产生新保存写入或样式同步成功回执。

与 Parent 发布回执的“21处真实style ref”须分开核对：本轮 master content/positionData 的21处 typographyRefId 及21处 typographyRefFile 均为 null，native fill/stroke/content 未发现非空样式引用；本域原实例同样未有有效引用。34+3 样式已发布是事实，原 master 到实例的样式引用传播仍待公共owner接续。body/caption/strong 使用 Parent 实际 UUID；无本地主题副本或猜测UUID。具体 API哈希、42空字段、原生无更新面板、source/override检查及逐图记录见[本轮回读复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-shared-styles-rev2/readback-review.json)。

随后准备在原 Spine_4 linked row 上通过 native Assets 消费 `editor.text.primary`，首个 get-file 即 ECONNREFUSED，尚未启动该浏览器或执行颜色点击/写入。18:09:32Z只读检查未见19001/16060/19002监听；原因和生命周期归属未推断，未自行恢复服务。失败回执保留且请求头已脱敏。跨文件同步/reset、CJK和引擎字体、官方导出、原资产加载、正常EXE/DPI及双端≤1逻辑像素比较均未通过。

当前域内只读源码候选为 SkeletonReference 的稳定资源版本+原joint slot身份及 `details()` 的实际父骨骼/参考local TRS/Quaternion借用投影；[详情适配交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-details/source-handoff.json) SHA256=`666e6903a4df30b45b2c2dc45da35ebbb8fa474a144147670f47fc4010866d2a`。原9项回归保留，加3项详情回归共12项，全部未执行。Parent已持有的editing/mod.rs保持其da2c4c13来源，不重放patch。Editor候选与原glTF三路径/350f输入/两配方操作分开；本轮无Rust、freeze、capture、grant、锁或公共产品源修改，窗口族passed/accepted继续false。

## 23. Shared rev4 的真实颜色与字体消费（本域 rev30）

2026-10-24T20:20Z–20:33Z，本域在原 file `b4a1e3ce-9e6a-81fe-8008-d69d60adb48e` / page `b4a1e3ce-9e6a-81fe-8008-d69d60adb48f` 的隔离浏览器接续设计。原内容板 `2f389980-f6e7-4f28-854d-2308a17dc31d` 保持真实1280×800逻辑尺寸；目标是已有 Shared TreeRow `170b04ef-1b6b-4add-aee5-d6eb01afb5a2` 内的 Spine_4 text `7b305f2b-1dcf-47aa-9b78-f178aaad91da`，原component=`6ddc248b-bd35-80c5-8008-d6c8c7a79944` / shapeRef=`6ddc248b-bd35-80c5-8008-d6c8c4c69ae5`，未detach或复制master。首个按标签筛选的预写核查发现两处Spine_4（树行与详情Parent），实际未点击颜色；该失败保留，后续按父级Shared file/component身份唯一匹配。

通过真实 Assets，先应用 Shared `editor.text.primary` / `e022fcac-39df-51eb-8515-4723f9932060` 保存为rev29，再在同一叶节点应用 `editor.typography.body` / `99eb86f9-12ff-5935-96ad-3d1a985e61f0` 保存为rev30。两次均关闭原上下文、fresh重开并认证API读回：paragraph与leaf的fillColorRefId/File及typographyRefId/File均非空且指向原Shared文件，原颜色引用在字体消费后保留。仅该一个text对象改变；1963 objects/1业务组件、83外部linked roots及1559后代source refs、所有标签和x/y/width/height保持，Shared在两次操作中指纹未变。具体前后内容、保存重开和9张逐图记录见[真实消费复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-style-consumption-r03/visual-review.json)。

本次实际Shared为rev4/isShared=true，仍为原4 masters、34 colors+3 typographies，master非空style refs仍0，尚无WorkbenchField/NumberField native身份。原21处空引用与历史rev2/18:09拒绝连接记录保留。当前真实界面已出现“updates in shared libraries”横幅；本域没有点击UPDATE、强写syncedAt/modifiedAt或reset覆盖。这次证明的是原linked实例内的本地共享样式覆盖值持久化，尚不证明master默认值传播、跨文件sync/reset或两族全局A/B。颜色原字面值与共享值均为#c0c0c0，不能当作有意可见颜色变化；尚未完成样式修改→官方导出→原ZUI加载→产品截图变化闭环。

字体消费实际将fontId从`custom-495ff285-7c71-49bc-bc4e-df20017e2d66`改为`gfont-roboto`，variant由normal-400改regular；family Roboto/13.333333/lineHeight1.2保持。后继fresh只读浏览器在rev30无写入，字体CSS和woff2实际HTTP200，CDP对该目标SVG text报告Roboto/isCustomFont=true/7 glyphs。document.fonts.check的拉丁和中文探针均false，不能用该粗探针推断所选文本未加载或完整覆盖；实际目标没有中文字符。SVG含textLength=46.578125和spacingAndGlyphs，固定文本框尺寸未变不证明自然字形宽度、CJK ready、引擎字体ready或≤1逻辑像素。当前PNG含Penpot chrome/77%画布，只是可编辑设计证据。

公共导出依赖已精确定位但未改公共代码：`penpot-text-capture.ts:67`把真实candidate.fills交给`singleSolidFill`；`penpot-capture-validation.ts:270`–271仍拒绝非空fillColorRefFile/Id；TreeRow painter的null/undefined归一化保留真实引用。请公共owner提供canonical样式/token解析与源映射合同，不删除真实ref或detach来绕过。此为当前源码静态追踪，尚未执行新插件导出或测试；源hash与行号在复核中。官方PNG的当前serve脚本仍把所有/api转16060，未分流/api/export，6061监听实测拒绝；因与已知404原因一致，本轮未点击重复导出、未产出official PNG，也未启停服务。

两行Bone Name=Head / Parent=Spine_4继续等待真实canonical WorkbenchField及typed只读slot；Euler/空间/Axis不借旧组件UUID关闭。当前Editor6域文件+Parent Root的7路径静态资料及12项未运行回归保持独立，原glTF三ownedPaths/350f输入/两配方不变。本轮0 Cargo/query/source capture/freeze/桌面/公共源修改，未获得EXE或150%DPI时段；真实mesh/pose/Physics预览、Socket作者/命令/undo/save、全部窗口族的导出和正常EXE矩阵仍open，passed/accepted=false。

## 24. 900×620与640×520原生流式内容候选（rev35）

接续第23节后，本域用真实Penpot原生Duplicate与布局检查器，在同一file/page保存两个独立可编辑内容候选，均保留原Shared/业务组件实例和真实Head参考数据。900×620 board=`fe1bcf3f-8861-80c8-8008-d72d399d764e`（x2280/y2080）；640×520 board=`fe1bcf3f-8861-80c8-8008-d72d551883a5`（x3340/y2080）。没有复制master、detach或新建公共组件。原1280×800内容板的254个对象指纹全部保持；两次复制各254对象，当前rev35/2471 objects/1业务master。原83/1559外部引用保持，加两板各12 linked roots/179后代source refs，当前107/1917。递归核对componentFile/componentId/shapeRef无差异，原Spine_4共享颜色/字体引用也随原生实例复制保留。原上下文关闭后fresh重开，两板和文件指纹均保持，Shared未改。

候选使用原column根flow和row内容flow，原24内边距、12外层间隔、16列间隔、24行高及4属性行间隔保持；说明列仅在候选内隐藏，全部保留于原复核板。900内容rect=852×519，树280×519、详情556×519、Bone data viewport540×467。当前Head参考字段自然高度457，全部字段及源路径可见。640内容rect=592×419，树240×419、详情336×419、Bone data viewport320×367。相同内容仍高457，Scale.X尾部裁切，Scale.Y/Z及source.path在底部之外，溢出90逻辑像素；截图已真实复核，**640窄尺寸完整性未通过**。

请Parent消费具体滚动合同：节点`fe1bcf3f-8861-80c8-8008-d72d551883b1`为640的Bone data viewport；offset0→本例max90时，最后source.path应完整抵达viewport底367。运行时须复用既有ScrollView/collection，按真实内容与Toolkit剩余rect重测`max(0,contentExtent-viewportExtent)`，90只属于本例内容板，不硬编码到产品。保留原typed资源版本/revision/joint slot、行高、间隔和参考local数值，不能压行或删除Scale来隐藏问题。Wheel/键盘/scrollIntoView、首尾夹取、focus-return/选中骨骼与scroll anchor、资源版本变化后的旧key拒绝均需真实实现和EXE验证；本域没有画静态滚动条或宣称已有可工作滚动。

原生动作、254→254的source映射、布局和裁切字段范围、5张保存/fresh逐图记录见[本轮窄尺寸复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-responsive-content-r01/visual-review.json)。900图canvas zoom107%、640图145%是Penpot适配画布缩放，均不证明Windows实际150%DPI；本板只覆盖Head筛选的内容设计，没有完整UE Toolkit/Dock/viewport、默认48行滚动或运行时按钮绑定。官方PNG/ZUI导出、Field、产品加载和真实预览依赖维持第23节状态。未改原业务ZUI/Rust/公共源、未申请capture/freeze/新Cargo身份，Editor12及原glTF3仍未执行。900内容可读不构成passed，整个角色窗口族passed/accepted均false。

## 25. 本域参考空间只读适配与当前18项回归

本轮在原 `skeleton_reference` 模块新增 `space.rs`、`tests/space.rs`，并接入原mod/reference/cases三文件；旧6域文件成为8域文件。Parent持有的editing/mod.rs仍为da2c4c13，未重放接线patch，hierarchy/details/tests/details未改。[当前源码指针](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-model/current-source-revision.json)更新为r03，保留5ff090e4旧指针的逐字节元数据归档、666e6903旧详情handoff与32ce基线；旧9/12项记录仅代表各自当时源码。本轮[源码交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-space/source-handoff.json)为d9a7b5e1，包含5个实际编辑路径、8域文件和只读Parent Root的精确SHA、前后变化、补丁及检查身份。

新增 `reference_matrix(key, Local/SkeletonRoot)` 沿用同一ResourceActiveVersionKey/revision/原joint slot。Local从原始参考TRS生成矩阵；SkeletonRoot按已校验父链迭代累计，复用现有Transform/Mat4/UnitQuaternion/NumericPolicy，不引入第二数学库、求值器、store或clock。派生矩阵归一化四元数副本，原始参考Quaternion/尺度仍由details借用且不改写；有限零/负尺度有效，继承剪切保留在矩阵，不求逆或拆回TRS。非法本地数值、祖先数值或累计溢出返回具体source slot错误，不用identity填补；跨资源、替换payload、同ID/revision重建后的旧key仍拒绝。

空间名称以实际UE源码核对：`BoneProxy.cpp:101`的Reference来自原RefBonePose，`:111`的Mesh Relative来自网格组件GetRelativeTransform，`:123`之后无网格时才以参考父链提供Bone的非局部值。`BoneProxyDetailsCustomization.cpp:207/226/482`的三项显示值初始全部可见；`:264`关闭Quaternion显示，`:319`只允许Bone编辑并提供relative/world控制。本域SkeletonRoot矩阵不能标作UE Mesh Relative或World，不能代替求值姿势/组件位置或GPU frame-ready；UE FTransform在非均匀/负尺度下的TRS及Euler读数一致性仍未证明。当前Penpot raw Quaternion设计差异与生产host依赖保留，详见[只读消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-space/consumer-contract.json)。

9路径当前rustfmt只读检查exit0；同源change guard为0错误/0警告/0抑制，patch digest=`4316e86bb7fda58cf458af0c518a2d07c0270681420b3da635f4359c16f436a0`。这仅为静态检查。原9+详情3+空间6共有18个静态测试定义，全部未执行；新6项覆盖父在子后、同名骨骼与原始数值保留、继承剪切、非法祖先/四元数与溢出、ABA/跨资源键拒绝、两万层迭代。原过滤器 `core::editing::skeleton_reference::tests::` 现在选择18项，不能继续称只运行9或12项。[独立Editor计划](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-space/editor-regression-plan.json)仍无request/operation/attempt，单zircon_editor/defaults-off/features[]/lib/test profile/locked/jobs1/threads1，保持原Runtime target-editor-host及test-support依赖；完整传递闭包待真实准入和执行证明。

未向原glTF `ue-character-source-20261024-01`追加Editor范围或替换350f输入/两配方；本轮无QueryOnly、source capture、九root冻结、Cargo、服务、桌面、公共UI/主题/backend或原业务ZUI写入。当前只读适配尚未连接实际窗口；640的90逻辑像素内容溢出、Field与官方样式/PNG/ZUI导出、生产mesh/pose/Physics预览、Socket作者事务/undo/save、正常EXE三尺寸与实际150%DPI继续open，整族passed/accepted=false，释放0 B。

## 26. 原生库同步失败、撤销与本域文字覆盖修复（当前 rev54）

2026-10-24T21:09Z–21:46Z，在本域原 file `b4a1e3ce-9e6a-81fe-8008-d69d60adb48e` / page `b4a1e3ce-9e6a-81fe-8008-d69d60adb48f` 完成实际原生 UPDATE、覆盖、Reset overrides、Undo及保存关闭fresh重开。Shared真实为rev5，仍原4主件、34 colors+3 typographies；Button main `6ddc248b-bd35-80c5-8008-d6c8bd9b057f` 新增一个 `editor.surface.1` / `599586e3-022c-500b-ac80-5d9c490139af` / #242424的实际fill绑定（ID/File两个非空字段，不是两个样式）。本域没有编辑Shared或公共源码。

新控制板 `ebad071c-29d4-8010-8008-d734c39ad699` 为640×520（x4030/y2080），原生Duplicate新增254对象，命名 Shared Button sync and reset / controlled probe。已有New实例 `ebad071c-29d4-8010-8008-d734c39b7788` 先通过native Assets应用 `editor.accent` / `b3f68044-6a40-566a-9d34-fdb415e552e8` / #0070e0；rev38颜色覆盖保存。rev39原生UPDATE保留该蓝色覆盖并把默认fill绑定传到其它Button，但833个原对象改变，其中原94个树标签全部变为Tree item，按钮/Search文字也回默认。失败图与API字节身份原样保留，不能算跨文件同步通过。

该失败由本域早期authoring metadata缺失具体解释：94树文本中90仅content-group、4另有text-content-attribute，均缺text-content-text。只读本地Penpot `libraries.cljc:1735/1756/1867` 的文字分支表明需要该细分标记；本域author-hierarchy/author-multiskin/native-bone-inspector旧脚本只设置content-group。这是本域作者数据缺陷，不把根因归咎Shared主件；本轮未证明服务实际frontend bundle与参考源码逐字节对应。原源码/历史脚本不覆写以抹去失败。

随后在同一自有隔离浏览器发送一次真实Ctrl+Z，rev40恢复94原树名/文字、原位置及共享样式引用。helper初版漏列Undo操作，因此仅对核过PID/出生/脚本SHA的自有Node临时连接debugger，调用其已有Playwright Page的原生键盘；未操作共享桌面、Penpot/Jenkins进程或update-file伪装撤销。前后全属性差异仍有89原对象：Root Frame新增板的shapes 1项、81文本positionData、7图标flipX/flipY；原content/style/geometry/source身份已恢复。library syncedAt仍21:18:45.918631Z，未随Undo恢复旧值，不能宣称原子撤销或重新同步成功。

rev41在控制板New上真实Reset overrides，背景恢复Shared默认#242424/同一ref，touched清空，但宽度77.3333→240.0000，文字恢复Button。文字并非丢失：实际DOM仍有Button，逻辑字形范围x4154.078125..4193.90625；后绘制Search的#1a1a1a背景x4135.3333..4403.3333覆盖该范围。故本板按钮看似空白，Reset还清除了尺寸覆盖；不能把全Reset当成仅颜色reset或布局通过。保存关闭fresh重开rev41数据hash一致，局部失效布局与640详情底部90逻辑像素溢出均保留。

修复先在控制板Filter原生增删末尾字符，rev41→43，确认真实生成text-content-text。首次严格rich-text hash核查中止：Penpot额外加入paragraph.key=88c80；文字/样式/尺寸/源引用均未改。保留失败receipt，后续只在本域诊断比较中单独识别该native段落key，没有改公共guard。再逐项原生验证5标签，rev43→53，保存关闭重开一致。最后经官方update-file/typed Transit，以rev53及完整dataHash比较前置，仅向其余112个明确text节点的touched集合并入text-content-text。全文件逐对象与data比对证明仅112个touched改变，content/style/geometry/IDs/组件源引用无改；未detach/复制main/写syncedAt。

当前rev54/2725对象/1业务主件，118个不同于主件默认文字的业务caption全部有text-content-text。保存、关闭原上下文、fresh认证重开后的dataHash=`58e0e1d250e9be9d8876b54c2968258763fec12c44b714661db0e9a2c5925def`；Shared完整指纹不变。当前119外部linked roots与2013个非root后代source refs互不重叠，合计2132个sourceRef对象，身份缺失0；原rev35对应107/1834，新控制板12/179。第24节107/1917是早期不一致计数，保留历史，本节按root/非root定义更正，不能将两种口径相加或套旧回执。

[本轮逐图复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-library-sync-reset-r01/visual-review.json)、[精确结构与原生修复](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-library-sync-reset-r01/structural-review.json)以及[同步前后诊断图](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-library-sync-reset-r01/native-sync-before-after.png)保留全部失败与恢复。当前只能证明文字保护metadata与保存重开；修复后的下一次真实Shared发布→原生UPDATE尚未验证，因为Undo后syncedAt已前移，不强改时间或Shared来制造更新。官方PNG/ZUI样式解析、Field、局部reset语义/flow、两族全局A/B及正常EXE继续依原owner接续。

当前r03的8域Rust文件+Parent Root共9路径指纹仍匹配，旧同源format/guard按精确字节复用，Editor18项及原glTF3项仍0执行。本轮0 Cargo/QueryOnly/新request/九root capture/freeze/服务生命周期/桌面/DPI/公共源写入。自有浏览器/API/Node均实际退出，临时runtime文件0 B；保留必要诊断，实际释放0 B。Penpot截图为DPR1、145%画布，不是实际Windows150%DPI；没有新EXE/nonce/frame-ready/GPU证据，整族passed/accepted=false。

## 27. 骨骼定位、隐藏目标与原始祖先键（r04）

只读参考 UE `SSkeletonTree.cpp:1743–1753` 的父链展开及 `1946–1999` 的非隐藏项定位；`1553–1636` 的输入方向/递归保护仍属 host。接续已保存的树/详情设计，在现有 SkeletonReference 新增 `reveal_for_name_query`，复用现有资源 owner 的 active version/revision，继续用原始 joint slot。未增选择 store、事务、时钟、资源加载器或 preview backend。

返回 `Included { key, ancestors }` 时，ancestors 是根到直接父的源键，不含目标和其它分支；Included 仅表示筛选纳入，目标可能仍因折叠而未实体化。Parent 应按现有 expand-on-selection/force 语义应用祖先，再由 collection 在同代树上完成 scrollIntoView。`FilteredOut { key }` 保留隐藏目标及原选择，不清空筛选、不换成邻近可见行。错误版本、同ID重装、删除重建及外域键返回 None。旧引用在 reload Error 后仍可形成只读计划；它不授权当前 picking/edit/preview，也不证明资源 Ready。

行投影与定位共用字面大小写无关子串规则，匹配后代的祖先保留；未扩展 UE expression/flatten/hide-parent/Socket/virtual bone 模式。过滤不会写用户展开集。手工代码审阅检查了 later-parent、重复名称、森林、中文与字面空格、ABA、失败重载和20,000层迭代父链；新增6项回归定义。当前10域源+Parent Root共11路径、相同独立Editor filter静态24项、0执行。详见[消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-reveal/consumer-contract.json)和[当前测试计划](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-reveal/editor-regression-plan.json)。

Parent Root未重放patch，公共 TreeIndex/collection/selection/Toolkit/renderer均未写。本轮未提交或运行Cargo、源捕获、freeze、服务或桌面；Editor request/operation/attempt仍null，原glTF请求不变。静态检查只对本轮绑定哈希有效，不是编译/运行证明。实际展开/滚动首尾/焦点返回、双向骨骼选择、显示帧代际核对及原生图像仍需接入和逐项验证，passed/accepted=false。

## 28. 原生Reset控制板的流式修复与证据边界（rev57）

继续第26节的真实失败，不覆盖rev39更新/原生Undo/Reset240重叠图。本轮API实读本域rev54和Shared rev5/4main/34colors/3typos，公共Field仍未发布。仅选择原受控板`ebad071c-29d4-8010-8008-d734c39ad699`内行容器`ebad071c-29d4-8010-8008-d734c39b1557`，通过原生检查器Column与Fit content(Vertical)形成约240×104纵排，三个原linked roots各约240×32、间隔4；没有改Shared、主件或基础组件内的约束。该板是尺寸/覆盖诊断，UE产品工具栏目标仍保留在原设计，不能用纵排诊断替代横向工具栏。

原生Reset overrides作用于控制板Filter root `ebad071c-29d4-8010-8008-d734c39b7786`。其局部文字Filter恢复为源Button，内部文字220宽恢复且居中，默认fill恢复真实Shared style599586e3-022c-500b-ac80-5d9c490139af/editor.surface.1。另一个default Button与Search现在各居独立行，未再遮挡。此操作本来会清除局部文字、尺寸与样式，不能当成仅颜色reset；该单处文字变化是控制板预期结果，其它业务caption与骨骼/源路径未改。

原生变更rev55→56→57已由API观测，保存后关闭原context、fresh认证重开，再关闭browser后独立API读回。三者dataHash均783a5f866b9e9567d5f255f7f3ee6aceed960b57bebedc39ac2dd9a8f6f7ed3e。2725对象无增删、1业务component记录保持；193个变化只在原254对象控制板内。Shared完整hash259ddb2a保持，linked library时间记录未改；119外部root+2013非root引用合计2132，原source映射全部有效。r04十一Rust源指纹仍相同，复用其同源静态回执，24项回归均0执行。

[5图逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-flow-r02/visual-review.json)均已实际打开。640×520控制板额外纵向高度使底部行为说明裁切；原640详情Scale/source.path裁切和源业务板滚动缺口继续open。浏览器1680×1050、deviceScaleFactor1、canvas145%不是系统150%DPI。没有官方PNG/ZUI或正常EXE图、1DIP差值；不能宣称双端验收。

本轮native save HTTP记录器未匹配到事件，raw ACK覆盖缺失；不把空记录当0写入，也不凭已持久化状态补造file/session/change级ACK。[保存观察缺口](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-flow-r02/save-observer-gap.json)与初次根sentinel检查失败、启动前stdin EOF原件保留。实际持久化读回只按其范围成立。没有重放公共发布、改syncedAt、detach或新组件。依Parent最新通信规则，本次普通域内修复写原manifest供读取，不另排例行ACK；服务/桌面/编译/捕获授权未变，passed/accepted=false。

## 29. Physics Asset未选择配置的三尺寸设计（rev63）

实际只读重新核对PhysicsAssetEditorMode.cpp:241–295：Standalone_PhysicsAssetEditor_Layout_v5.3，横向0.2/0.6/0.2；左SkeletonTreeView上0.6、PhysicsAssetGraphView下0.4；右上DetailsTab/AdvancedPreviewTab（Details前景）0.6，右下PhysicsAssetTools/PhysicsAssetProfilesView（Tools前景）0.4，Viewport隐藏TabWell。本轮在本域原file/page新增可编辑的未打开配置设计，不将空场景、静态图片或原Space当生产预览。主窗口菜单/Toolkit/Dock、激活页签及事件仍等待共享host合同。

三张板实际ID：1280×800为8ae2217e-98a4-4069-91b5-e7c05bb5104b，900×620为a21d2788-0445-413b-9440-441271309b97，640×520为4da0983a-3e83-4da9-9236-1523734347d4；均在原page b4a1e3ce-9e6a-81fe-8008-d69d60adb48f。通过现有linked copy各复制一只Button和SearchInput，保留原componentFile/componentId/shapeRef，文字覆盖含text-content-text；新增114对象、6外部root，没有新公共main。原有全部非根对象/业务component/Shared均原字节。新增209处颜色/字体引用指向实际Shared rev5/34colors/3typos；Roboto/CJK字体测量及官方codec仍未验证。

初次rev59 API保存，浏览器文字布局正常化到rev60。首次板位置y3200与旧Wolf/Knight板重叠，1280/900旧板名穿入；原4张图保留。整体移动的官方mod-obj请求因selrect普通map不是原生Rectangle记录被HTTP500拒绝，随后独立读回确认无持久化变化；初次原生定位器在token输入模式不匹配也仅留检查失败。依据实际17个可见输入及唯一Y=3200，改用原生Y检查器把三板移到y4500，rev61–63，114个新对象随板平移，旧板与内部布局未改。当前无画布重叠。

最终3尺寸及1280 fresh图均实际打开复核；保存、关闭原context、fresh认证重开、关闭browser后独立API dataHash均05f076e1c5d694fb6e2652e64686fe4f16486ab1e0c4eca391f886d1c1bfa6a9，rev63/2839对象/1业务component。125外部root+2034后代合计2159共享source refs有效。新增分栏/上下区域几何与声明系数仅有浮点误差，这不是UE截图或1DIP逐图比较。8张[逐图复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-physics-empty-design-r01/visual-review.json)区分失败与最终图；[精确设计/源映射](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-physics-empty-design-r01/design-contract-placement-r02.json)及[结构核查](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-physics-empty-design-r01/structural-review.json)保留所有新板/节点/实例身份。

空状态明确profile/skeleton/body/constraint/frame为None、0行、未模拟、未Ready；不是把不合法空RagdollProfile验成就绪。当前profile一骨一个shape，与UE多shape/独立constraint选择不同；本轮没有生成或导入真实Physics profile。只读搜索覆盖设计和“打开配置…”caption没有运行事件，不能称可用按钮。已有asset.physics.ragdoll_profile.create仅OpenView，不算调用generate_initial_ragdoll_profile。源ragdoll_profile.zui六节点/六control ID/四Space/0events及SHA4a29da2bef097a6253688e7c21d3e602972b222f2504e6aa16aa2a2e3adbfbb7保持；Graph/Tools/Profiles新增区域要经Parent共享Dock合同和官方导出再接入原owner，未先建新资产/壳或伪造source metadata。

未完成差异：Search根在三尺寸分别238.4/162.4/110.4，内部surface固定240，几何溢出1.6/77.6/129.6；640实际占位文字裁切，900边框仍裁切。该C6公共问题没有在本域修改子节点约束掩盖。原.zui左右min200/240也与窄尺寸比例目标冲突，仍是待共享布局/设计决定；右侧可读文本折行不证明真实Details/Tools滚动。生成/作者与持久化、刚体/约束选择、碰撞体gizmo、模拟、撤销、弹层焦点返回、滚动首尾、长资源名、真实secondary mesh/pose/frame和registry跳转均open。没有用主PIE帧/假backend，也没有新增clock或Renderer实现。

本轮仅设计和文档，8个UE/Physics参考源、r04十一Rust源及原glTF三owned路径均匹配，复用同源r04格式/guard，不拿旧文档guard套新文档。Editor24定义和原glTF3项均0执行，未创建新request/operation/attempt、query/capture/freeze或服务/桌面动作。浏览器deviceScaleFactor1、画布77/107/145%不等于实际150%DPI；无正常EXE、官方PNG/ZUI和双端差异。阶段进展不代表Physics或整族完成，passed/accepted=false。


## 30. Physics 配置校验与原子编辑候选（r01）

本轮承接第29节Physics设计，只改Physics插件原Runtime/Editor owner的8个源码路径，未改Penpot、原.zui、公共host或backend。UE参考职责仍为Physics Asset的骨骼/碰撞体/约束详情；当前Zircon schema一骨一个shape，compound children没有独立稳定shape ID，不能称已达到UE多shape与独立constraint选择。

| 实际源码 | 本轮职责 |
| --- | --- |
| `zircon_plugins/physics/runtime/src/manager/validation.rs` | 既有两个validator仅扩大到crate内部可见，校验主体完全保留 |
| `zircon_plugins/physics/runtime/src/skeletal/profile.rs` | validate复用constraint/compound数值规则，新增InvalidConstraint错误，原严格Box规则保留 |
| `zircon_plugins/physics/runtime/src/skeletal/mod.rs` | 接入新的Runtime测试子模块 |
| `zircon_plugins/physics/runtime/src/skeletal/tests/authoring.rs` | 新增6项约束/drive/threshold/复合形状/解析回归定义 |
| `zircon_plugins/physics/editor/src/lib.rs` | 导出typed API，接入Editor测试子模块 |
| `zircon_plugins/physics/editor/src/ragdoll_profile_editor.rs` | 仅接入mutation子模块，原generate_initial_ragdoll_profile主体保持 |
| `zircon_plugins/physics/editor/src/ragdoll_profile_editor/mutation.rs` | 新增prepare_ragdoll_profile_edits及3个typed输入/错误类型 |
| `zircon_plugins/physics/editor/src/tests/ragdoll_profile_edit_tests.rs` | 新增12项批次、原路径/原slot、中文长路径及错误回归定义 |

`prepare_ragdoll_profile_edits(&RagdollProfile, &[RagdollBoneEdit])`先校验源，按精确作者bone_path映射原slot，只克隆一份候选并依手势顺序赋值Shape/Mass/BodyOffset/Constraint/BlendWeight，最后校验完整候选。MissingBone或最终非法时整批拒绝；重复赋值允许，未暴露的中间值不单独提交；无净变化返回None。无叶名、大小写或trim兜底，不重排/改名/改父，不建store、版本clock、history、World或I/O。纯API不携带document/resource版本权限，旧路径本身不授权当前编辑。

现有profile校验此前没有覆盖constraint，并未应用manager的compound子体变换规则。新逻辑拒绝倒置/非有限limits、非法drive数值、负或非有限可选threshold，以及非有限/非单位缩放的compound子体；原profile更严格的Box正尺寸限制保持。新增公开`RagdollProfileError::InvalidConstraint`会影响下游穷尽匹配；仅已做定向源码查找，尚无编译兼容证明。既有serde字段/schema不变，但此前被接受的非法文件现在from_toml/validate会失败，须保留诊断，不能静默clamp或修复。资产引用解析、backend形状支持和实际resource ready不由数值检查证明，单位仅沿用现有Physics类型，未假定UE显示角度转换。

8源同源format与change-guard最终均exit0，0错误/警告/抑制，guard patchDigest为cd179d916e40303290d56d9841766d09b41926e723f74671d7f95f48909013db；[源码补丁](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/source.patch)含5个原文件和3个新增文件，SHA256=4c3e916493e37a7cd3849b30f21a6fce13490665828f5d12b69233c0c4ba1ca1，反向只读apply检查exit0。首次格式exit1、末尾空行以及一次无部分写入的apply_patch失败均保留；后续修正没有改ACL、锁或服务。[结构复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/structural-review.json)绑定8源与9个受保护输入；后者包括manifest/lock、plugin注册、原ragdoll_profile.zui及Runtime公共schema，均保持。

独立[回归计划](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/regression-plan.json)为两个单包命令：nested manifest `zircon_plugins/Cargo.toml`，分别`zircon_plugin_physics_runtime`的`skeletal::authoring_tests::`（6项）与`zircon_plugin_physics_editor`的`ragdoll_profile_edit_tests::`（12项），均defaults-off、features=[]、locked/lib/profile=test/jobs1/test-threads1、物理E:/cargo-targets。只是未来argv草案，未确认当前入口资格、未创建identity/request/operation/attempt，也未解析完整依赖或运行Cargo；filter只限执行，不限编译闭包。测试使用明确单位数据及真实现有generator，不是实际导入profile或角色预览。骨骼r04十一源/24项及原glTF三源本轮匹配，旧glTF身份与18输入不追加本scope。

[host消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/consumer-contract.json)要求Parent在prepare与commit均核对现有document/resource/profile版本，用完整before/after候选承接一次现有history事务和真实保存；current skeleton joint slot、mesh/node/skin、preview generation及entity↔profilebone必须同session绑定。当前仅交付候选，无消费者、undo/redo、focus/gizmo/picking/持久化或性能通过。原`asset.physics.ragdoll_profile.create`仍只OpenView，未接generator；原RagdollBoneProperties未写事件或临时props，Field身份/typed slots仍等正式交付。生产secondary preview、真实mesh/pose/physics frame、正常EXE/150%DPI、官方导出及双端逐图继续open。sourceCaptureReady/资源准入、passed/accepted均false。


## 31. Retarget 未配置状态的UE嵌套布局（rev66）

实际重读`IKRetargetApplicationMode.cpp`的`Standalone_IKRetargetEditor_Layout_v1.022`：外层水平splitter的左内层未显式设置coefficient，右Details为0.2；`TabManager.h:475`确认默认coefficient=1.0，因此保持外层1:0.2嵌套，不能直接套Physics的20/60/20。左内层再按0.2:0.8划分操作/层级列与视口列；左列0.6:0.4上下分成OpStack/Hierarchy及AssetBrowser/Overrides/Variables，视口列0.9:0.1分成隐藏tab well的PersonaViewport与OutputLog，右为Details/AdvancedPreview。每层gap之后分别归一化，三尺寸的实际几何与这些系数仅有浮点误差，不是UE或文本像素比较。源/hash及就绪条件见[源码与命令合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-retarget-capability-state-r01/source-and-commands.json)。

UE的RunRetarget绑定IsReadyToRetarget，后者检查实际processor初始化；EditPose检查当前mesh存在。Zircon原stable binding/spec将Preview与Apply路由到原command，而feedback明确只有Not implemented (preview)。原.zui仍是三rig、四链和静态Ready，原raw SHA4fe1c1d1543f92bd925f809563a07b2f424e843cfd483931cf770210a120670f保持；本轮没有宣称该源或真实命令已经修复。旧导入板3aae194a-8d90-80a9-8008-d6a74bfb8568的全部122节点及导出guard/失败基线原样保留，不重签baseline。

新增板在同file/page、y5500，1280×800=c773c951-6e28-4701-9944-a667fda0c445，900×620=28c05145-f16c-4430-bf74-2171c48e0822，640×520=99daf9a7-7f8a-419c-809c-986205ad503b。state仅为未选择source/target/definition、0条映射、0日志消息、无frame/mesh/pose；明确显示尚无重定向预览，不放静态角色图片、假链行或可用solver。每板2只原生linked Button作源/目标选择的设计caption，真实registry picker/事件消费者尚未接线；不能把原Source TextChanged/Target Commit的稳定事件改解释为新Button Click。原asset主控角色与新区域关系只放在[外部设计合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-retarget-capability-state-r01/design-contract.json)，没有给新节点伪造ZUI source metadata。

新增102个节点、6个原native linked实例，组件file/component/shapeRef及文字覆盖保留，未新建公共main/主题或复制Field。新文本直接引用实际Shared body和真实颜色，207处引用核对有效；Shared rev5/4主件/34颜色/3字体完整hash保持。当前own rev66/2941对象/1业务main、131外部root、2171个Shared源引用；其余2838个原非根对象及业务component均原字节。首个只读inspector把zero-root自引用当cycle失败，原helper/错误保留；第二次只修sentinel后只读检查成功，无Penpot写入。后续add/reorder官方API各HTTP200保存到rev65，浏览器原生文字布局正常化至rev66，只动39个新节点。

已实际保存、关闭原context、fresh认证重开并在browser退出后由新独立API核对，dataHash均ff0c39c4533f4c8ef4f4ddd018609185635658c7408d9316d96cf1493ea5ef81。当前helper在导航前安装response observer，捕获1笔`/api/main/methods/update-file`真实HTTP200及完整request/response字节；顶层ACK仍写rev65并含lagged后继，未把该字段冒充最终rev66。源身份、实际后继与保存读取分列在[保存观察范围](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-retarget-capability-state-r01/save-observer-gap.json)，早期Physics/control保存观察缺口不据此改写。新native截图4张全部实际打开，详见[逐图复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-retarget-capability-state-r01/visual-review.json)。

1280、900、640及1280 fresh图未见板内重叠或明显文案裁切；900/640侧栏中文自然换行。640将资源页签组显示为“资源 / …”，这只是待Parent Dock overflow合同的设计，未实现菜单/页签/focus，也没有证明真实长资源名或滚动。原asset左右min270/320与窄尺寸嵌套比例仍冲突，原生Button样式也未证明UE工具栏视觉；CJK字体ready、UE逐行测量和1DIP目标均未验。浏览器deviceScaleFactor1与画布77/107/145%不等于实际150%DPI。

本轮无Rust源码/原资产/公共owner写入，无Jenkins新身份/QueryOnly/提交/capture/freeze，无服务启停、共享桌面或EXE。9个参考/原owner源、骨骼r04十一源/24定义、Physics r01八源/18定义和原glTF三ownedPath均匹配；原owner22:50:51Z的03cc2f4e回执所读18/18另作历史观察，不冒充本轮完整准入。正式plugin/export、真实definition/chain/processor、undo/保存、关联registry/Toolkit/focus-return、唯一secondary preview/正常EXE/DPI/双端状态比较仍open。新空态设计不关闭Retarget产品缺口，passed/accepted/captureReady=false。


## 32. Physics 从已安装 Skeleton 生成候选（r02）

第29节Physics三尺寸设计和第30节typed编辑继续保留。本轮只接领域输入：新增zircon_plugins/physics/editor/src/ragdoll_profile_editor/source.rs，并在原ragdoll_profile_editor.rs及lib.rs接线；新增tests/ragdoll_profile_source_tests.rs。API generate_ragdoll_profile_from_skeleton接现有ResourceManager与ResourceActiveVersionKey，使用snapshot_expected的单authority锁配对校验，不另建loader、skeleton key或版本时钟。

source_paths必须按真实AnimationSkeletonAsset.bones原始slot提供完整作者路径，父路径由原parent_index确定。显示名、过滤树行及重复leaf不作身份，不trim/改大小写/猜UUID。原始TRS逐骨检查finite后原样传给既有生成器，原generate_initial_ragdoll_profile函数体与base完全相同；capsule仍是local translation长度启发式，body_offset仍identity，不能称UE基于真实网格几何的自动拟合或有效动画pose。重复/空路径、cycle及self-parent复用当前RagdollProfile策略，self-parent仍返回MissingParent而非新造错误语义。

返回RagdollProfileFromSkeleton的profile与source version/revision均只读；original_joint_slot按精确profile path返回原payload slot。它是生成候选，不是新的作者store、持久key、entity绑定或当前pick权限。真实作者full-path producer仍缺，AnimationSkeletonAsset仅有name/parent_index；既有runtime exact-name/unique-leaf查找也尚未接成typed resource slot。不能据此关闭重复名runtime歧义或同session body关联。

10个新回归覆盖真实ResourceManager API安装载荷、child-before-parent/森林/源TRS不变、重复名与中文长路径、path count、非法父索引/cycle/self-parent、路径policy、root非finite三类TRS、错误kind/payload、资源替换及旧candidate保留、同id/revision删除重建ABA、空Skeleton拒绝。它们是显式unit payload，不是已导入Knight/Wolf、真实mesh/pose/GPU或EXE证明。Physics现10源，Runtime6＋Editor22定义；0项执行。当前源码格式与guard同源通过、无错误/警告/抑制，精确hash/patch见[后继交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-skeleton-source-r02/source-handoff.json)与[独立配方计划](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-skeleton-source-r02/regression-plan.json)。r01八源原件和初次格式失败保留，未拿旧检查覆盖新两文件。

Parent须在prepare与commit核对资源/文档当前版本及readiness，持有一次before/after历史事务、save和focus-return；错误/过期不写入或造历史。现asset.physics.ragdoll_profile.create仍OpenView，没有假称已生成。secondary session、mesh/node/skin、实际pose及body entity↔profilebone、frame-ready/teardown仍归唯一host；本轮没有并行backend或借主PIE帧。one-shape-per-bone/compound子shape ID、单位转换、真实作者与undo/选择/模拟、official export、正常EXE/150%DPI/双端1DIP保持open。原glTF三路径/身份、Skeleton r04/24和原资产未改，无新请求/身份/Cargo/query/capture/freeze，passed/accepted/captureReady=false。


## 33. Control Rig真实源层级与未配置状态（rev72）

实际参考为dev/UnrealEngine/Engine/Plugins/Animation/ControlRig/Source/ControlRigEditor/Private/Editor/ControlRigEditorMode.cpp:263–374的Standalone_ControlRigEditMode_Layout_v1.10，而非条件分支中的legacy v1.9。水平0.2/0.6/0.2；左上PersonaViewport隐藏tab well、下方Hierarchy/Execution/Curve/GraphExplorer按0.5/0.5分；中间GraphEditor初始ClosedTab、CompilerResults/FindReferences为OpenedTab，按0.8/0.2分；右侧Details/AdvancedPreview/RigAnimAttribute，foreground为Details。首次参考投影在写入Penpot前已纠正，原pre-correction记录保留，没有把legacy布局写入后当新版通过。

源数据来自实际knight.glb（ce0d0a5e…）的Knight_Head node27/mesh4/skin4、23 joints，未取Mesh0或首skin。过滤head保留root slot0/node22→hips slot1/node21→spine slot2/node12→chest slot3/node11→head slot14/node10；选中head的原parent slot3。源local translation为[0,0.26879656314849854,0]、Quaternion[0,0,0,1]、scale[1,1,1]。这是只读GLB JSON投影，未decode meshopt/accessor/inverse-bind、未导入Registry或生成有效pose；源树不是持久RigHierarchy。RigDefinition=None、controls/graph documents/nodes=0、compiledRig/production frame=None、previewReady=false。

三板实际ID分别为1280×800=2b313fba-3b7e-4093-adf2-e0c5ac87a05c、900×620=ec82e74c-393d-498a-a802-d66cbe3178c0、640×520=a5ee9c22-c321-48c5-a73a-388a56543655，均在本域已分配file/page的y6600。新增426节点、21只原native linked copy（15 TreeRow、3 SearchInput、3 Button），保持原主件UUID/shapeRef与child顺序；没有detach、新建main或公共theme。261处新增Shared style引用有效。Shared仍rev5/4主件/34颜色/3字体，无canonical Field，因此右侧仅真实源说明，不宣称完成PropertyEditorRow/Field编辑。

官方add/reorder保存到rev68，原生文字测量到69；详情缩放三分量的第一次修改因Transit类型不完整HTTP500，失败请求及响应保留，独立读取证明当时文件未变。精确typed Transit请求保存content到70后，fresh画面仍使用旧positionData，r02四图保留为失败证据。只读核对dev/penpot现有process-operation/set-shape-attr语义后，仅删除三个新说明文本原有派生position-data；HTTP200到71，原生重新测量到72。没有修改Penpot源码、shared parser/严格ACK规则或其它节点缓存。

真实native保存两笔完整request/response已捕获：顶层ACK分别68和71，lagged后继69和72；最终72由fresh认证重开及所有browser退出后的独立API证明，不能用顶层ACK冒充当前版本。[ACK复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-rig-source-state-r01/native-ack-review.json)与[结构复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-rig-source-state-r01/structural-review.json)确认dataHash=7f9ba5fb7731af193bb6b313d88e1f1953ef1c49a3dedef527d9e31754473337、3367对象/1业务main、152外部root/2498有效源引用；原2940个非zero-root对象、业务component及Shared完整字节不变。

初版、content-only r02、缓存重测r03共12张全幅原图均实际打开，[逐图复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-rig-source-state-r01/visual-review.json)分列每张发现。最终三尺寸及1280 fresh右侧XYZ缩放可读；900与640选中head名称仍完全不可见：glyph宽29.1875、可见宽0，1280可见29.1875。TreeRow仍保留240px主件内部caption位置，消费root随UE比例缩小；本域未私改共享child几何、detach或扭曲比例掩盖C6。行高28.000000119、每级10px缩进仅当前组成，不是UE逐行度量通过。640列表28px偏移为设计剪裁，不是实际scroll/reveal/focus证据；“层级 / …”也不是已实现溢出菜单。

空graph和viewport明确未配置，不用静态角色图、假节点或不可用solver冒充功能。Graph公共表面仍由材质域、timeline/evaluator由动画域、registry/Toolkit/history/唯一secondary host由Parent持有；“打开Rig…”仅native Button设计，未接真实command/focus-return。原ControlRig资产及15绑定、typed Edit/Commit事件种类均保持，不把原反馈preview/validate说成求解或校验完成。

[源连续性](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-rig-source-state-r01/source-continuity.json)实核33保护源及9参考输入保持，Physics r02十源和Skeleton r04十一源（含Parent root）静态回执只在同字节范围复用；原glTF三ownedPath保持，未声称本轮重核完整18输入或writer census。本轮无Rust/原ZUI修改、正式plugin/export、Jenkins新身份/query/POST/capture/freeze/Cargo、服务启停或共享桌面。浏览器deviceScaleFactor1与画布77/107/145%不等于Windows实际150%DPI。样式→官方导出→产品加载A/B、真正Rig/preview/undo/保存、正常EXE、LayoutReviewCase/双端1DIP及全族passed/accepted继续open。


## 34. SkinN Data与原始骨骼槽位映射（r05）

本轮承接第33节实际Knight_Head node27/mesh4/skin4和原r04只读模型，推进生产preview所需的领域输入。UE SkeletalMeshLODRenderData.h:70–71明确RenderSection::BoneMap是USkeletalMesh::RefSkeleton数组索引，树行顺序不是渲染骨槽位；本实现只处理Zircon现有glTF源映射，不宣称已实现UE section压缩/BoneMap或GPU palette。现gltf_animation_subassets.rs:197–233按skin.joints顺序生成Skeleton，:670–710的gltf_skin Data写同序joints、明确skeleton_asset及可选inverse-bind引用/count，是本轮唯一生产schema依据。

新增skin/mod.rs、skin/binding.rs、skin/metadata.rs和tests/skin.rs；原mod/reference/cases仅接线及joint_count只读访问。GltfSkinReference::read(resources, expected_skin, &SkeletonReference)从现有snapshot_expected<DataMarker,DataAsset>取得同一资源锁内配对的Data snapshot。校验JSON format/record locator、kind/index、明确Skeleton locator、完整joint count、原slot骨名一致性、唯一node index与同源Node引用、可选SkinN/InverseBindMatrices引用及count。没有创建source DTO、临时props、加载器、资源clock/store或backend，DataAsset原文和canonical JSON仍由原importer拥有。

返回有序GltfSkinJoint的SkeletonBoneKey、node index/locator、key_for_node/joint_for_key及skin version/record；骨名可以重复且不作身份。registry提供的sidecar ID原样使用，不从locator造UUID。没有逆绑定时保持None/0，不补假identity数组；有引用/count也没有加载矩阵或宣称finite/palette正确。canonical SkinN/NodeN label检查只验证当前producer已明确输出的引用，不替代registry查找或从mesh猜skin。

该对象保留Skin Data snapshot并借用SkeletonReference，两者不是一次多资源publication事务，也不证明当前ready/lease/pose/session/frame/pick/edit权限。即使名字和count碰巧相同也不能证明source generation对应，Parent须验证当前资源组及真实node/mesh/primitive/skin关联。实际gltf_labeled_subassets.rs:220–239的SceneMesh转换尚未保存node.skin()；本适配不伪造关联或把所有导入scene强行放player。跨域最小后续是Parent registry/唯一preview消费既有真实绑定，再把本原slot映射交给动画pose/实际picking，而非在域内改共享PreviewSubject或另造backend/clock。

8项回归定义覆盖child-before-parent与重复中文骨名、非首skin和独立sidecar ID、错误Skeleton/declared/actual count、schema/format/payload locator、重复node/骨名不一致/跨源引用、逆绑定有无和count、wrong kind/实际payload type、Skin替换与删除重建ABA、Skeleton foreign/replace/ABA和旧snapshot只读保留。全部是显式unit payload，经真实ResourceManager API安装，不是导入Knight、mesh/pose/GPU或EXE证据。当前32项＝原9 cases＋3 details＋6 space＋6 reveal＋8 skin，0执行；完整未来命令仍单zircon_editor/defaults off/features=[]/Windows lib/显式profile test/locked/jobs1/threads1/E物理根，见[独立计划](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-skin/regression-plan.json)。无新identity、parser/query/POST/Cargo，不混原glTF两recipe。

本轮15源同源格式/guard通过，0错误/警告/抑制；首轮仅新re-export换行格式失败及对应旧guard保留，后继按当前15源重新检查。三个已有文件移除本轮新增后精确还原r04 hash，另8个旧源（含Parent root da2c4c13）不变，原24测试体未改。增量[patch](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-skin/source.patch)reverse-check通过，不重放Parent root；[membership](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-skin/source-membership.json)仅说明14域＋1依赖，不是全checkout/writer census。26个reference/Physics/glTF保护输入匹配；原glTF3 ownedPaths及18输入边界不扩，本轮未声称重核完整18。Penpot最近实证72、四张最终图及TreeRow900/640裁切保持历史观察，本轮无API/浏览器/UI改动；official export、生产consumer、真实preview/author/undo/save/focus、正常EXE/实际150%DPI/双端1DIP和passed/accepted/captureReady仍false。


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

设计当前实证仍是rev76；本轮未写Penpot、Shared、公共pipeline或桌面。

## 38. Physics Profile 持久化编码（r05）

当前Runtime已有RagdollProfile::from_toml与完整作者校验，新增codec.rs提供to_toml：先复用同一validate，再以既有toml1.1.2/serde schema写出字符串。RagdollProfileEncodeError区分Profile准入失败与Toml序列化失败，保留原错误source。不改RagdollProfileError既有变体、字段/schema、原读取器、manifest/lock或capability；不写文件、不发布资源、不建立第二store/clock/history。

宿主可在候选准备阶段对candidate.after().to_toml()做编码预检，但不能据此标记已保存。实际保存必须从原Document取得真实已提交revision与原save token，编码该revision的完整profile，使用原持久化流程写入；仅在保存成功且token仍匹配时标记该revision已保存。编辑或资源变化、编码/写盘失败都须保留dirty和原history。原document/source/revision/lease的一次CAS与undo/redo、同secondary session实体/骨骼/pose/frame关联仍由Parent集成；此API不提供这些权限。

新增9项Runtime编解码回归定义：既有稀疏TOML默认值、中文/转义/长作者路径和父后子前顺序、全部8种既有碰撞形状及嵌套compound与UUID/url配对、稀疏轴与全部drive字段、Some(0)与None及锁轴区别、有限浮点边界/TRS/负零、不合法profile原错误、legacy三轴数组和root/bone未知字段拒绝。测试仅用内存单元数据调用真实codec，不注册生产夹具或宣称真实角色/资源ready。相关Runtime作者定义由6增至15，Editor作者41项、Skeleton Editor32项不变，全部仍0执行。

同源scoped rustfmt、4源码及4文档guard、增量patch反向check和diff检查记录在[本轮交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-codec-r05/source-handoff.json)。当前17源为r04十四源中一个Runtime mod后继、其余13源保持，另显式纳入Runtime lib接线与2新文件；r03/r04旧交接及旧过滤器输入原字节保留。托管最小新范围为zircon_plugin_physics_runtime/defaults off/features[]/locked/profile test/jobs1/lib/filter skeletal::codec_tests::/threads1，物理E:/cargo-targets；不启用backend-jolt，不借glTF、Skeleton或Editor11项身份。现行53172128…真实parser对此Runtime单包配方仍返回build_scope_policy，零dispatch/Cargo，限定资格交原Jenkins owner接续正常prepare/intake。

编码保留当前typed schema能表达的数据；注释、格式和已被宽松nested decoder丢弃的字段无法从DTO恢复。静态源码确认root/bone拒绝未知字段，而共享PhysicsColliderShape、PhysicsJointConstraintMetadata与PhysicsJointDrive没有deny_unknown_fields。原始文档完整保存须选择严格拒绝未知nested字段，或由原Document保存未建模原文；不以writer存在宣称整份原始文本无损。[三处共享小补丁提案](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-codec-r05/strict-nested-fields-proposal.json)及精确diff已通过非修改apply --check；共享源保持，兼容性决策与其独立回归待Parent/共享owner评审。

本轮没有写Penpot、原ZUI/公共主题/控件/Shared、桌面或服务，也没有发出全局冻结时段或执行身份。正常受管准备沿任务授权继续，不要求人类补grant；真实source/resource/pool/cache/native与Rust/产品证据仍分别pending，goal active，passed/accepted=false。

## 39. TreeRow 公共源码审阅与文档编码闭环

消费公共owner于2026-10-25T16:22:02Z完成的[审阅](<E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/character-tree-content-lane-r01/handoff.md>)（SHA256 b57afa8c…）及[详细记录](<E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/character-tree-content-lane-r01/review.json>)（7b5bbd87…）。本轮重算其归档rev76的15行，确认15处caption guard不符、90个直接painter part源引用保留、75处hidden=true与原hidden=false guard冲突，以及15个allocation parent仍指向main板父级。原part-ID列表与copy ID不同，单凭该差异不能判为sourceRef丢失；原有source-admitted remapping必须继续严格校验。

当前native与插件的content语义均保留disclosure槽，只隐藏展开符/尾部动作；不能把第36节原审计中的ix=dx建议当作插件单端修复。native rows.rs:39和commands.rs:47–67仍沿disclosure计算对象图标。默认行及现有对齐content保持；content-compact仅为待Parent统一native/catalog后的候选opt-in名，尚未公开或实施。候选应由真实源明确启用，外层已处理缩进时源depth/indent为0；12/16/6/12度量下caption X34、宽max(0,W-46)。另有plugin图标Y8、native及归档flow Y6的静态公式差异，需与横向变体分开核对，不能据此宣称实际主题/1 DIP通过。

迁移必须保持原component/main/child/shapeRef及15个linked root。现NativePainterPanel/Icon没有源visibility字段，refresh拒绝拓扑变化；五个inactive parts应获得同源、身份稳定的可见性表示，再按新鲜精确前像逐实例迁移。直接layoutTreeRowText、全量native Reset、删part、detach、复制新main或从当前读回重签baseline均不能接纳这些改动；未知布局/paint/owner/node/instancePath变化继续拒绝。路径、前像和责任分配以[proposed-scope.json](<E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/character-tree-content-lane-r01/proposed-scope.json>)为准，本域未执行该提案。

三张Control Rig业务板（1280×800、900×620、640×520）的归档root均无devZirconZui role=asset、source envelope或export-shapes-guard。TreeRow子实例的canonical metadata不授予整板正式导出身份，不能补签。先在原Shared单TreeRow正式源面验证，再由本域/Parent提供来自原业务资产的真实invocation owner与instancePath；当前业务板、640 root裁切/首尾scroll/focus、SDK/export、normal EXE/实际150%DPI和双端误差继续pending。

编码版本链已逐路径核验：公共review.domainDocEncoding引用的四个SHA正是r03损坏版本；它们匹配[原修复记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/documents-repair.json)的before。17:40:19Z修复后的after又匹配r05文档变更的before，r05 after匹配本轮编辑前原字节。当前四文档严格UTF-8解码，连续ASCII问号及U+FFFD均为0；本轮没有从问号推测语义，而是依据原audit/handoff、源代码和本次公共审阅补明边界。原乱码版本SHA、失败、图像、审计和source handoff均保留。

[本轮文档证据](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/tree-content-public-review-r02/handoff.json)只覆盖这四份说明文档及对应静态检查。无新SDK/export/Node/Cargo/Jenkins请求、host/desktop/Penpot/Shared/源码操作；Physics r03/r04/r05及其测试输入未改，不用新文档guard覆盖其运行门。goal active，passed/accepted=false。
