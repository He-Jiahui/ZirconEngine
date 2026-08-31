---
title: Editor Scene Viewport Transform Manipulation、Gizmo、Pivot、Coordinate Space、Grid、Snapping、Workplane、Numeric、Surface/Vertex Alignment、Preference、Transaction 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor67
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_editor/src/scene/viewport/handles
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/binding/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/workbench/model/status_bar_model.rs
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/grid_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/grid
tests:
  - zircon_editor/src/core/settings/tests/persistence.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/toolbar_dispatch.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/typed_command.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/status_bar.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/65-editor-scene-object-creation-placement-palette-factory-asset-drag-drop-template-favorites-preview-transform-transaction-plugin-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/mvp/05-f4-basic-authoring.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/SnappingUtils.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/SnappingUtils.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorViewportSettings.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorModeManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeManager.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/UnrealWidget.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/UnrealWidget.h
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.h
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.h
  - dev/Fyrox/editor/src/interaction/move_mode.rs
  - dev/Fyrox/editor/src/interaction/rotate_mode.rs
  - dev/Fyrox/editor/src/interaction/scale_mode.rs
  - dev/Fyrox/editor/src/interaction/gizmo
  - dev/Fyrox/editor/src/settings/move_mode.rs
  - dev/Fyrox/editor/src/settings/rotate_mode.rs
  - dev/bevy/crates/bevy_gizmos/src/transform_gizmo.rs
  - dev/bevy/crates/bevy_gizmos_render/src/transform_gizmo_render.rs
  - dev/bevy/examples/gizmos/transform_gizmo.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Gizmo/HierarchicalBox.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/LightAnchorHandles.cs
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/67-editor-scene-viewport-transform-manipulation-gizmo-pivot-coordinate-space-grid-snapping-workplane-numeric-surface-vertex-alignment-preference-transaction-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Transform Manipulation、Gizmo、Pivot、Coordinate Space、Grid、Snapping、Workplane、Numeric、Surface/Vertex Alignment、Preference、Transaction 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前不是空壳。Scene Viewport已经具有Move、Rotate、Scale三种`HandleTool`、overlay extract、renderer-visible picking、preview/finish/cancel生命周期、authoring world预览、撤销命令，以及project-scope平移/旋转/缩放步长设置和真实toolbar route。设置写入还经过共享authority与异步持久化；这些基础应保留并收敛，不能用另一套临时gizmo重写。

但三种变换求解器仍是原型级实现。Move把鼠标位移投影到“世界轴在屏幕上的线”后乘`world_units_per_pixel`；Rotate同样投影轴线，再把像素标量乘固定`0.01`当弧度；Scale仍使用该标量做单轴加法并硬夹到`0.05`。当轴接近视线时Move/Scale退化，Rotate更是没有在旋转环平面内求夹角。当前只支持单节点、X/Y/Z单轴、Local/Global两种空间；center anchor只是绘制元素，没有自由平移、统一缩放或view rotation交互，也没有平面handle、parent/view/normal/workplane空间、pivot mode、numeric entry或surface/vertex/bounds snapping。

网格与吸附也没有形成工程合同。`GridMode`把Hidden、VisibleNoSnap、VisibleAndSnap揉成一个状态，导致“显示网格”和“三类变换吸附总开关”不可独立控制。renderer不读取translation step，只画固定XZ平面、`-10..10`、1米间距、固定major interval的21x21网格；toolbar preset、settings schema、status text又各自维护不同表现，状态栏只显示translation step并硬编码`m`。顶层Workbench Snap route只会设置`VisibleAndSnap`，不能关闭；另有数个Snap/World/Target图标无事件。

Unreal把translation/rotation/scale、surface、actor、vertex、layer snapping分开建模，并从pre-drag transform求绝对/增量结果；Godot已有轴/平面移动、轴/view/trackball旋转、组pivot、vertex snap状态机、单位化数值设置和undo；Fyrox至少对selection root、parent inverse world transform、ray-plane求交、smart move与CommandGroup闭环；Bevy的较小gizmo也已采用ray-plane交点、`atan2`旋转与基于起始值的乘法缩放；Unity Graphics镜像虽不包含完整Scene View源码，仍展示了`Tools.pivotRotation`、native snap接入、modifier与负scale policy。Zircon不能把“有三个图标和三段浮点运算”视为达到这些系统的证据。

Editor03、53、58、59、63与65已分别拥有selection/pivot父语义、interactive tool、viewport session、input/capture/picking、transaction/history和placement父合同。本报告不重复抬高其开放P0。本轮新增 **0项P0、27项P1、8项P2**，登记 **48个全部Fail的资格门**。目标不是照搬Unreal类层级，而是建立纯数学`TransformManipulationKernel`、generation-qualified `ViewportTransformSession`、`SelectionTransformPlan`、`SnapPolicy/SnapCandidate`、`GridWorkplaneState`、typed preview/commit receipt与单一产品投影。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、save/reopen、multi-selection、parented/nonuniform hierarchy、surface/vertex query、render golden、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。当前不能声称Scene Viewport变换功能、表现或性能达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon transform kernel | **41 / 3,047 / 2,785 / 102,258 / 14** | handles、controller、projection、preview/transaction、grid extract与renderer | `24bc98f5be81522e70715550afcd936b81c4d8cdb80ca5810ff1fc686783ab1e` |
| Zircon settings and product route | **22 / 4,108 / 3,754 / 165,217 / 6** | settings authority/schema、typed command、toolbar/top bar/status bar与snap route | `b76d26a70808e391eef8c59fef114f71a678afd4978aa3b2f4223908fe26df23` |
| Zircon focused tests | **10 / 3,072 / 2,809 / 107,668 / 66** | settings round-trip、viewport state、binding、dispatch、projection与status bar | `cf8836d3f0ae3fa5c3cbd22f6ef28f7436b4b81007befeb8a5a6909e9f653c92` |
| Unreal selected set | **9 / 20,029 / 16,918 / 727,308 / 0** | widget delta、pivot/multi-selection、grid/rotation/scale/surface/actor/vertex snap与settings | `3e8ba2c83eb370ddaf5beaedbdec524cbf8fa5bcf9a86aceca8ce7d3a92b2146` |
| Godot selected set | **4 / 12,586 / 10,703 / 500,558 / 0** | axis/plane/view/trackball、group pivot、vertex snap、numeric settings与undo | `35def937d58afcd8c3054efa834ca415421387ca66338886012b80452d4e9db1` |
| Fyrox selected set | **10 / 2,253 / 2,076 / 82,083 / 0** | selection roots、parent/world conversion、ray-plane math、smart move与CommandGroup | `624efbc943f587a7ca9060ce8f0d4751489ce522186893fd859e680859e704e7` |
| Bevy selected set | **3 / 1,386 / 1,280 / 49,458 / 0** | opt-in transform gizmo、ray-plane solver、view axis、snap settings与render separation | `ad53f2f65ad23624043a1a40dce9caa7adb2949f51975e14bf911d004fa07e3c` |
| Unity Graphics selected set | **2 / 543 / 489 / 26,369 / 0** | pivot rotation、native snap consumer、modifier与negative-size policy | `b645f1284f3ba832ab870cadfedfc8dd030a26f9eac98a8b06aee77fa3e38bf0` |

fingerprint按规范化相对路径排序，并将每个`path + newline + file SHA-256 + newline`聚合后再做SHA-256；它只证明本轮读取的working-tree语料，不是ABI、artifact、动态结果或性能receipt。主仓与Unreal镜像基线为`bee4c707b714738346b49bba15c59468b8bd9b39`；Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。

### 2.2 在途修改隔离

冻结时`scene_viewport_controller_handle_input.rs`和`tests/editing/state/viewport.rs`含非本轮修改：新增secondary navigation button不得替换active gizmo capture的guard和测试。它关闭了Editor59的一条捕获交错路径，但没有改变本报告所审查的单节点session、三类求解器、pivot/space/snap/grid或产品模型；两文件已按working-tree内容计入fingerprint。相邻pointer overlay、precision picking adapter与dispatch也有在途修改，但不计入本轮变换数学语料。

共享索引含前序review修改，本轮只追加Editor67并更新汇总数字，不覆盖或回退其他Session内容。coordinator Session为`optimize-editor67-transform-gizmo-review-r1-20260822`，baseline epoch为339；新报告和三个共享索引取得精确lease。MVP `00-current-source-baseline-recovery`仍为`in_progress`，F4不能绕过F0-F3，因此本轮没有以Cargo结果包装静态审查结论。

### 2.3 范围与非范围

本报告拥有Scene Viewport内translate/rotate/scale求解、gizmo frame、pivot/coordinate/workplane、grid/snap policy、surface/vertex/bounds alignment、numeric transform entry、专项preferences、typed transform receipt以及这些能力的产品控制与资格。

通用selection identity和多选集合归Editor03；generic tool scheduling归Editor53；per-view/surface generation归Editor58；input/capture/picking/cancel归Editor59；document transaction/history归Editor63；placement preview归Editor65；camera navigation归Editor66。Editor67只定义它们必须承载的transform-specific payload和不变量，不新建平行authority，也不重复计数父问题。

## 3. 当前实现拓扑与可保留基础

### 3.1 HandleTool分层是真实基础，但registry仍是闭集

`HandleTool`已经分离overlay、begin/update/end drag，`HandleToolRegistry`集中分派Move/Rotate/Scale，extract与renderer picking不直接写Scene。这个边界可以演进为纯solver和provider registry。当前registry仍以三个字段和`match TransformHandleKind`硬编码闭集，trait没有descriptor、capability、lifecycle、settings schema或custom handle element扩展合同。

### 3.2 预览、取消和历史路径已经存在

pointer move生成`ViewportTransformPreview`，上层更新authoring world并记录latest transform；release后构造`EditorCommand::applied_transform`，Escape/模式切换/World replacement可回滚初始值。该生命周期比直接在pointer handler写历史更可靠，应保留。当前capture只含单个`NodeId + initial + latest`，提交标签固定`Move scene node`，无法表达Rotate/Scale、多选root、pivot、space、snap receipt或每目标失败。

### 3.3 设置authority与project-scope持久化可保留

translation、rotation degree和scale step均通过共享Settings authority解析、校验和异步持久化，focused test验证project scope round-trip。这是正确owner方向。问题在schema只有三个scalar，三类值共用`0.0001..1,000,000`范围，没有单位、preset catalog、独立enabled、grid origin、preserve-offset、surface/vertex policy或per-view override。

### 3.4 Local basis实际是节点local transform，不是world frame

`selected_basis`取得`node.transform`并传给`build_handle_basis`；Local分支直接读取该local transform的right/up/forward，origin也使用local translation。只要节点存在旋转/缩放父级，屏幕中的gizmo frame、world overlay位置和提交delta就可能与用户看到的world transform不一致。Global分支固定X/Y/-Z，也没有parent、view、normal或custom workplane。

### 3.5 三种求解器共享错误的屏幕轴标量

`projected_axis_delta`把`origin`和`origin + normalized axis`投影到屏幕，取鼠标delta在该屏幕方向上的点积。Move随后乘origin处`world_units_per_pixel`；Rotate直接乘`0.01`；Scale直接乘`0.01`。这不是同一个可验证的几何模型：轴接近视线时返回None，透视下Move不是最近点/约束平面解，Rotate没有环平面角，Scale也不是起始尺度比率。

### 3.6 Center anchor与完整overlay存在，但交互只识别X/Y/Z轴

Move和Scale绘制center anchor，Rotate也绘制center anchor；然而`GizmoAxis`只有X/Y/Z，begin drag永远需要轴。产品因此展示了不可操作的中心元素，也没有XY/YZ/ZX平面、view-plane move、uniform scale、view ring或trackball。overlay视觉词汇领先于interaction contract。

### 3.7 Grid extract只有两个布尔值

Editor向Runtime提交`GridOverlayExtract { visible, snap_enabled }`，grid pass只根据`visible`决定是否画预建vertex buffer。builder固定在XZ平面从-10到10逐米画线，major interval和颜色固定；translation snap step、camera distance、projection、unit、origin、workplane、axis、subdivision和fade完全不进入render contract。`snap_enabled`甚至不影响grid geometry。

### 3.8 产品route真实但状态语义分裂

Scene viewport toolbar能切transform space、grid mode和三类step，preset cycle也会落到settings authority。顶层Workbench Snap却无条件发送`SetGridMode(VisibleAndSnap)`，不能关闭；componentized presentation又在任何`SetGridMode`后把Snap标active，即Hidden也可能亮。status bar以translation step拼`m`，不展示rotate/scale或独立吸附状态；另一些Snap/World/Target图标无event。这里需要单一typed product model，不是再加一组图标。

## 4. 五引擎参考证据与适用边界

### 4.1 Unreal：主参考是Typed Element、Widget、Pivot和多层Snap组合

`LevelEditorViewportSettings`分别保存translation、rotation、scale、surface、actor、vertex、layer snap及当前grid index，并支持decimal/power-of-two grid、percentage scaling、arcball和screen rotation。`SnappingUtils`按工具和输入chord解析有效snap，位置可相对grid base做绝对snap，vertex失败后可回退grid，并绘制snap helper。

`LevelEditorViewport`在drag开始前保存被操纵元素的pre-drag transform，排除selection自身后按surface/actor/vertex/grid层级求结果，再通过Typed Element list和pivot把delta应用到多选。`UnrealWidget`负责axis/plane/screen/arcball数学和absolute movement；transaction、pivot更新与模式管理在更高层完成。可借鉴的是“冻结起始事实 -> pure constraint/snap -> validated multi-target plan -> transaction”的分层，不是复制Unreal全局单例和宏体系。

### 4.2 Godot：轴/平面/view/trackball、组pivot和vertex snap已进入同一产品状态机

Godot的3D viewport gizmo hit test覆盖axis与plane translate、axis/view/trackball rotate、axis与plane scale。`_compute_transform`区分local/global、group pivot和parent basis，应用前检查零determinant，并可按政策保留child global transform。vertex snap具有source/target/cancel状态、mesh/collision选择、origin/vertex模式、视觉提示和用户消息。

其Snap设置dialog对translation/rotation/scale使用单位化suffix、不同min/step/max；grid显示是独立View选项，modifier可以临时调整snap。Godot证明这些不是高级装饰，而是日常编辑器状态机的基本组成。

### 4.3 Fyrox：规模较小，但几何与层级基线高于Zircon

Move mode冻结`selection.root_nodes`，记录每个节点初始local position和parent inverse global transform，通过camera ray与constraint plane求交；smart move会精确查询geometry并排除selected descendants。Rotate mode使用ray-ring plane角度并支持per-axis angle snap；三种mode结束时都构造`CommandGroup`。

Scale mode已有乘法更新和uniform center handle，但仍把scale夹为正数且留有uniform行为TODO，所以Fyrox不是最终金标准。它的价值是说明即使较小编辑器也必须先处理selection roots、parent/world转换、ray-plane solver和group command。

### 4.4 Bevy：最小可交互gizmo也采用ray-plane与起始值数学

Bevy `TransformGizmoSettings`提供mode、space、view axis、独立`Option` snap、cursor confinement和screen scale；state冻结start transform、entity、world drag point和origin。Translate/Scale通过viewport ray与约束平面求交，Rotate在环平面上用`atan2(det,dot)`，Scale以cursor/start projected ratio乘初始scale。render又在独立crate中。

其实现仍是单focus entity，没有Editor transaction、多选pivot、workplane或surface/vertex snapping，并且正scale clamp同样不是Zircon的最终目标。但它足以否定“屏幕轴点积乘常数已经可接受”的假设。

### 4.5 Unity Graphics：只能作为consumer证据，不能冒充完整Scene View参考

本地Graphics镜像不含Unity Editor的完整`Handles`与Scene View源码，因此不能据此声称已复核Unity全部变换系统。选取的`HierarchicalBox`明确接入native scale snap、Shift/Alt/Cmd modifier并声明negative-size policy；`LightAnchorHandles`遵循`Tools.pivotRotation`调用位置handle。它们证明工具consumer必须尊重全局pivot/snap/modifier合同，也提醒Zircon必须给negative scale/size一个显式政策。

## 5. 差异矩阵

| 能力 | Zircon current source | Unreal / 其他参考 | 结论 |
|---|---|---|---|
| Drag math | 屏幕轴点积；Rotate/Scale乘`0.01` | ray-plane、closest point、ring angle、absolute movement | 原型，不满足几何正确性 |
| Handle family | X/Y/Z轴；center只绘制 | axis、plane、screen/view、arcball、uniform | 产品能力缺口 |
| Selection | active primary单节点 | root-filtered multi-selection、typed element group | 依赖Editor03重构 |
| Coordinate frame | Local/Global；Local取local transform | world/local/parent/view/normal/custom frame | 层级下错误且空间不足 |
| Pivot | primary node origin | active/median/bounds/individual/custom/placed pivot | 无authority |
| Scale | 单轴加法、最小`0.05` | 起始值乘法、uniform/plane、negative policy | 语义不完整 |
| Snap enable | grid mode作为三类总开关 | translation/rotation/scale/surface/vertex独立 | 状态模型错误 |
| Snap math | 相对delta round-to-step | absolute/base-relative/preserve offset/candidate hierarchy | 无策略层 |
| Grid | 固定XZ、1m、21x21 | camera/workplane/unit/major-minor adaptive | 与snap事实脱节 |
| Surface/vertex | 无 | candidate query、filter、target hint、fallback | 产品域缺失 |
| Numeric entry | 无drag numeric session | typed delta/absolute entry、unit-aware fields | 产品域缺失 |
| Transaction | 单节点preview/rollback/history | group plan、semantic label、typed receipt | 局部基础可保留 |
| Settings | 三个project scalar | versioned profile、independent switches、presets、units | 需要扩展authority |
| Product UI | 多处图标/循环，状态不一致 | 单一command/state projection与可见值 | 需要硬切单一模型 |
| Tests | route/lifecycle为主，无solver math | geometry、hierarchy、snap、undo、device/scale矩阵 | 资格证据不足 |

## 6. 新增发现

### 6.1 P1：架构、正确性与产品闭环

#### ED67-P1-01：变换session只冻结active primary的local transform

`TransformHandleDragSession`只有单`node_id`、单local transform、axis、cursor、basis和三个step；EditorState capture同样只有单节点。它没有document/view/world generation、root-filtered targets、parent/world frame、pivot、modifier、snap target、settings generation或transaction group。必须由Editor03/58/63的identity与selection父合同生成immutable `SelectionTransformPlan`，drag期间不能重新从浮动selection猜目标。

#### ED67-P1-02：Local gizmo basis读取local transform，parented对象的world frame错误

overlay origin与right/up/forward直接来自`node.transform`。父节点有旋转、缩放或非均匀scale时，gizmo可画在错误位置/方向并提交错误delta。必须从qualified world transform和parent inverse构造显示frame与每目标local写回计划，并明确shear/不可分解情形。

#### ED67-P1-03：Coordinate Space只有Local/Global，且没有有效空间receipt

缺Parent、View/Screen、Surface Normal、Custom/Workplane等常用空间，也没有“某操作强制Local”或“当前selection无法表示该空间”的rejection。需要stable space enum、frame source identity/generation和effective-space receipt；UI显示请求值与实际值必须一致。

#### ED67-P1-04：没有Pivot Mode、Pivot Placement和多选origin政策

gizmo永远位于primary node origin。缺Active、Median、Bounds Center、Individual Origins、Last Selected、Cursor/Custom Pivot及temporary pivot placement，rotation/scale也没有围绕group pivot重算translation。Editor03拥有selection/pivot父语义；Editor67必须消费其typed pivot frame并定义每种tool的应用规则。

#### ED67-P1-05：Move使用屏幕投影近似，不是world constraint solver

将鼠标delta投影到屏幕轴再乘origin处world-per-pixel，在透视、近裁剪、极端FOV和轴接近视线时不保持几何约束。应从camera ray、drag-start hit和稳定constraint plane/closest-line求world delta，并返回degenerate/repaired reason。

#### ED67-P1-06：缺少平面移动和自由移动，center anchor形成假交互暗示

只支持X/Y/Z axis；XY/YZ/ZX plane、view-plane/free move和axis lock都不存在。center anchor进入overlay但不能被`GizmoAxis`表达。必须让render element、hit id、constraint descriptor和solver一一对应，禁止绘制无可达交互的handle。

#### ED67-P1-07：Rotate把轴线屏幕投影当角度，数学模型根本错误

当前Rotate没有求鼠标射线与旋转环平面的交点，而是复用axis line投影并乘`0.01`。当旋转轴指向相机时轴线投影退化，恰好是该旋转环最可见、最应可用的角度。必须用drag-start/current平面向量和`atan2`有符号夹角，并处理跨`pi`连续性。

#### ED67-P1-08：缺少View Ring、Screen Rotation和Trackball/Arcball

产品只能绕X/Y/Z轴，无法绕视线旋转或自由trackball。需要将view axis与trackball作为明确handle/constraint类型，分别定义local/global composition、snap eligibility、visual feedback和reduced-motion/precision policy。

#### ED67-P1-09：Scale是单轴加法并硬夹正值，不具备工程语义

`scale = initial + scalar * 0.01`依赖像素而非drag-start ratio，缺uniform/plane scale，并把所有分量夹到`0.05`，令负scale、镜像和接近零的明确政策都不可表达。应从起始投影比率做乘法scale，支持uniform/plane/individual origins，并由asset/component policy决定negative/zero admission，而非工具内硬编码。

#### ED67-P1-10：Global/Parent scale在非均匀层级中的可表示性未定义

对有旋转且parent nonuniform scale的对象，world-axis scale可能引入shear，不能无损写回简单TRS。当前实现完全绕过该问题。solver必须返回`ExactTrs`、`ApproximationRejected`、`ShearRequired`等typed outcome，并由scene transform合同决定是否支持matrix/shear，不能静默分解污染数据。

#### ED67-P1-11：变换数学缺finite、determinant和normalization验证

`maybe_snap`、axis normalize、quaternion composition、scale update和preview apply均没有统一NaN/Inf、零轴、零determinant、non-unit quaternion或overflow校验。需要validated input/frame、finite result gate和per-target rejection；任何非法预览都不得写authoring world或进入history。

#### ED67-P1-12：Gizmo屏幕尺寸和pick tolerance不是稳定不变量

Perspective extent以camera distance乘`0.22`再clamp `0.75..3.5`，Orthographic以size乘`0.35`再同样clamp；没有FOV、viewport、DPI或实际像素目标。pick radius又是world常数。必须以目标device-pixel尺寸生成projection-aware world extent，并让visual/hit tolerance在DPI和多viewport下可验证一致。

#### ED67-P1-13：Grid visibility与全部transform snap被错误耦合

`GridMode`同时控制显示和snap；`VisibleAndSnap`开启平移、旋转、缩放三类snap，隐藏网格也必然关闭。必须拆为独立`GridVisibility`、Translate/Rotate/Scale Snap Policy和临时modifier override，迁移旧设置时给出确定映射。

#### ED67-P1-14：Grid renderer固定XZ 1米21x21，和snap step没有事实关系

render packet只传`visible/snap_enabled`，builder固定`-10..10`、每米一线、固定major interval和颜色。translation step即使改为`0.1`或`2.0`，网格仍完全相同。需要typed `GridRenderDescriptor`携workplane、origin、unit、minor/major step、camera scale、fade、axis和generation，renderer只消费该事实。

#### ED67-P1-15：没有Grid/Workplane session与持久化边界

当前网格永远是世界XZ平面。缺XY/YZ、selection/local/normal、自定义平面、origin offset、rotation、lock/follow selection、per-view/per-document persistence和reset。应由Editor58 viewport session承载`GridWorkplaneState`，项目默认与用户/view override经共享settings authority解析。

#### ED67-P1-16：Snap profile只有三个scalar，schema与preset catalog分裂

三类step共用宽泛的`0.0001..1,000,000`范围；toolbar另写三组硬编码preset，status只理解translation。需要versioned `TransformSnapProfile`，为每类值定义单位、range、preset id、enabled、absolute/preserve-offset、rounding和override层；schema、menu、status与solver都消费同一resolved snapshot。

#### ED67-P1-17：当前只snap相对drag delta，没有绝对网格与preserve-offset政策

`maybe_snap`只对从初始值算出的标量round-to-step。它不能把对象对齐到grid origin，也不能区分“保持初始余量”和“绝对落格”，rotation/scale同样缺base/reference。需要`SnapAnchor + SnapRounding + OffsetPolicy`，结果包含原值、candidate、chosen value和误差。

#### ED67-P1-18：三类snap没有独立enable、temporary override或modifier语义

用户不能只开rotation snap、暂时反转snap、降低精度、锁轴或切换absolute/relative。输入协议也没有modifier进入drag session。应与Editor59 qualified input和Editor08 keymap/chord authority连接，在begin/update receipt中冻结或显式更新effective modifier policy。

#### ED67-P1-19：Surface、Vertex、Bounds、Object与Actor Snap完全缺失

没有candidate provider、scene query、target filter、selected-descendant exclusion、normal alignment、offset、collision/render geometry source、fallback hierarchy或target deletion处理。需要provider-neutral `SnapCandidateQuery/Result`，Runtime提供geometry/bounds事实，Editor负责session/admission/selection policy。

#### ED67-P1-20：没有Snap Receipt、Target Hint和受预算约束的candidate选择

当前用户只看到transform变化，无法知道是否吸附、吸到谁、采用何种规则或为什么失败；renderer也没有helper extract。必须产生chosen/rejected candidate、distance、normal、source generation和fallback reason，并以bounded query、hysteresis和visual hint避免每move全场景扫描与目标抖动。

#### ED67-P1-21：Drag过程中没有单位化Numeric Entry session

不能键入绝对/增量position、angle、scale，也不能切换local/global、轴、单位、表达式、确认/取消或回到pointer drag。需要与当前interaction group绑定的`TransformNumericEntrySession`，解析结果进入同一个solver与transaction，不得旁路为Inspector字符串写入。

#### ED67-P1-22：缺少Precision、Axis Lock和输入连续性政策

没有slow/precision modifier、动态轴锁、cursor confinement/warp、pointer离窗、DPI变化、camera move during drag或设置变化时的连续性定义。session应冻结必要frame，也可通过显式rebase receipt接收合法变化；任何terminal event必须可恢复初值或提交最后有效值。

#### ED67-P1-23：Handle registry不能承载插件或专业工具扩展

尽管已有trait，registry仍硬编码三个零状态tool字段和闭集enum，没有descriptor id、provider generation、capability、custom overlay/hit schema、settings contribution、lifecycle lease或unload terminalization。应在Editor50/53 extension与tool scheduler父合同上注册transform constraint/provider，不能让插件直接改核心match。

#### ED67-P1-24：Workbench Snap是单向开启，active projection还会谎报状态

Workbench route无条件设置`VisibleAndSnap`，不能toggle off；componentized window对任何`SetGridMode`都把Snap标active，包括Hidden/VisibleNoSnap。必须从resolved `TransformSnapProductModel`投影checked/mixed/unavailable，command只表达intent，不能根据“曾发送某命令”猜状态。

#### ED67-P1-25：重复产品面缺可见值与typed menu，部分图标完全无事件

Scene toolbar通过图标轮换preset却不显示当前数值或列表；status只显示translation step，另有Snap/World/Target icon无event。应收敛为icon + value/menu、独立translate/rotate/scale controls、workplane/pivot/space菜单与数值编辑，所有surface消费同一snapshot并具有明确Unavailable状态。

#### ED67-P1-26：Transaction payload没有变换语义，提交标签固定为Move

Rotate和Scale结束也以`Move scene node`执行，command只包含单节点before/after，没有tool、space、pivot、targets、snap receipt、partial failure或merge identity。Editor63拥有transaction authority；Editor67必须提供semantic `TransformCommandPlan/Receipt`，由Editor63原子提交并按interaction group合并。

#### ED67-P1-27：现有测试没有证明任何核心solver、层级或snap正确性

66个focused test declaration主要覆盖settings round-trip、command codec、toolbar projection、pointer route与单节点Move transaction lifecycle；没有Rotate/Scale solver、axis degeneracy、plane/free/view/uniform、parented/nonuniform hierarchy、multi-selection/pivot、absolute/preserve-offset、surface/vertex、grid render、numeric input、product toggle truthfulness或性能预算。必须先补RED math/property/golden矩阵。

### 6.2 P2：质量、可维护性与资格证据

#### ED67-P2-01：Magic constants分散且没有单位/依据

`0.22`、`0.35`、`0.75..3.5`、`0.01`、`0.05`、pick radius、grid extent/颜色/major interval和preset序列分散在handles、renderer与UI route。应进入validated profile、render style或pure solver私有常量，并注明device pixel/world unit/radian/ratio语义。

#### ED67-P2-02：状态栏把世界单位硬编码为米

translation step总是拼接`m`，没有项目unit scale、角度/比例格式、locale、scientific notation或large/small world显示政策。数值与网格产品需要统一`EditorUnitFormatter`，显示单位不能改变solver canonical unit。

#### ED67-P2-03：Gizmo与Snap控制缺accessibility和混合状态表达

icon-only控件需要稳定accessible name/tooltip/focus/keyboard route；多选不一致、不可表示space、snap临时反转和provider unavailable需要mixed/disabled reason。颜色不能成为唯一轴/hover/snap反馈。

#### ED67-P2-04：没有结构化Transform/Snap diagnostics

应记录session/view/document generation、constraint、input ray、effective frame、candidate count/chosen target、solver repair/reject、targets applied、transaction id和terminal reason，并有采样、容量和隐私边界。diagnostics用于解释，不替代typed receipt。

#### ED67-P2-05：没有确定性input trace、math golden和render golden

需要记录initial scene/frame/input sequence/resolved profile和expected receipts，支持headless replay；render golden覆盖DPI、FOV、orthographic、hover/active/snap hint、grid fade和轴接近视线。手工拖动截图不能作为唯一回归证据。

#### ED67-P2-06：没有hot-path、query、allocation与soak预算

需定义60/120/240Hz pointer-to-preview p50/p95/p99、每move allocation、1/1K/100K target transform成本、surface/vertex candidate query预算、grid vertex/update预算和8小时drift/NaN/transaction soak。未取得profile前不能声称性能优于Unreal。

#### ED67-P2-07：Snap/Workplane preference缺schema迁移和corruption recovery

未来profile需要stable version/id、project/user/view scope、preset rename、unit migration、invalid custom plane、provider missing和corrupt value repair。失败持久化不能改变effective authority，旧`GridMode`迁移必须可逆验证。

#### ED67-P2-08：缺少同语义跨引擎功能与性能基线

最终比较必须固定scene hierarchy、selection、pivot、viewport/FOV/DPI、input trace、snap targets、硬件、warm-up和统计口径，分别比较move/rotate/scale/grid/surface/vertex/numeric。先证明功能同构和结果误差，再比较latency/CPU/GPU；只比较平均帧率无效。

## 7. 目标架构与职责边界

### 7.1 Runtime：提供纯数学与Scene事实，不拥有Editor交互事务

建议建立：

- `TransformManipulationKernel`：输入validated ray/camera、constraint、drag start、frame/pivot和snap decision，输出finite `TransformDelta/TransformSolution`。
- `TransformFrame`与`PivotFrame`：显式world origin/basis、source identity/generation和space kind，禁止用裸local TRS冒充world frame。
- `TransformConstraint`：Axis、Plane、ViewPlane、Ring、ViewRing、Trackball、AxisScale、PlaneScale、UniformScale与custom provider。
- `TransformApplyPlanner`：在Runtime Scene transform合同上把world delta规划为per-target local result，检测shear、零determinant、nonfinite和不可表示结果。
- `SnapCandidateProvider`：向Editor返回generation-qualified vertex/surface/bounds/object candidate；provider不得拥有selection、UI或transaction。
- `GridRenderDescriptor`：承载workplane、origin、minor/major step、unit scale、camera-relative extent/fade、style和generation。

数学模块必须无UI、无全局settings读取、无authoring world写入，才能做property/fuzz/benchmark并供placement、level design和未来插件复用。

### 7.2 Editor：拥有qualified session、policy、产品编排和命令计划

建议建立：

- `ViewportTransformSession`：绑定viewport/document/world generation、tool/provider generation、input capture和terminal lifecycle。
- `SelectionTransformPlan`：由Editor03 selection生成root-filtered targets、initial world/local/parent transform、active item、pivot与每目标admission。
- `TransformInteractionCoordinator`：冻结/重基camera、frame、constraint、profile和modifier，调用Runtime kernel，发布preview/receipt。
- `TransformSnapProfile`与`SnapQueryCoordinator`：从共享settings authority解析独立snap政策，预算化查询候选并维护hysteresis/visual hint。
- `GridWorkplaneState`：作为Editor58 per-view payload，支持project default和view/document override。
- `TransformNumericEntrySession`：与pointer drag共享constraint、solver和Editor63 interaction group。
- `TransformCommandPlan/Receipt`：描述tool/space/pivot/targets/snap和per-target结果，由Editor63 transaction原子提交。
- `TransformManipulationProductModel`：单一投影toolbar、top bar、status bar、menu、shortcut与Unavailable/mixed状态。

### 7.3 Owner规则

`zircon_runtime`不得依赖Editor command、selection或settings UI；`zircon_editor`不得复制ray-plane/ring/scale math；renderer不得从布尔grid mode猜step/workplane；UI不得直接拼preset或从last command猜checked状态；插件必须经Editor50/53注册provider并持lifecycle lease。

## 8. 分阶段重构计划

### ED67-M0：真实性止血与RED基线

修正Workbench Snap单向开启和错误active投影；不可操作center/status icons先标Unavailable或移除。补Rotate/Scale、axis degeneracy、parented transform、grid/snap状态和transaction label RED tests，冻结当前行为差异。

### ED67-M1：Validated math、frame与result合同

建立camera ray、world frame、constraint、finite input/result、ring angle、closest-line/ray-plane、multiplicative scale和typed rejection。property/fuzz覆盖NaN/Inf、近平行、极端FOV/DPI和round-trip。

### ED67-M2：Selection plan、coordinate space与pivot

接入Editor03/58 qualified identity，生成root-filtered multi-target plan；实现world/local/parent/view与Active/Median/Bounds/Individual pivot，解决parent inverse、nonuniform scale和shear admission。

### ED67-M3：完整Move/Rotate/Scale hard cutover

实现axis/plane/view move，axis/view/trackball rotate，axis/plane/uniform scale；render element、hit id、constraint和solver严格一一对应。删除屏幕轴点积乘常数旧路径，不保留compat shim。

### ED67-M4：Grid、Workplane与Snap profile

拆分grid visibility与三类snap enable，建立versioned profile、absolute/preserve-offset和modifier override；实现per-view workplane和camera-adaptive grid descriptor，迁移旧`GridMode`与三个scalar。

### ED67-M5：Surface/Vertex/Bounds/Object Snap与Numeric Entry

建立provider registry、candidate budget/filter/hysteresis、target hint和typed receipt；加入unit-aware numeric delta/absolute entry，复用同一solver/session/transaction，不从Inspector旁路。

### ED67-M6：Transaction、Undo/Redo和单一产品模型

通过Editor63提交atomic multi-target transform plan，补semantic label、merge/group、cancel/rollback、target deletion和partial failure policy。toolbar/top bar/status bar/menu统一投影同一product snapshot。

### ED67-M7：Extension、diagnostics、persistence与规模

接入Editor50/53 provider lifecycle，完成custom constraints/candidate providers、profile migration/corruption recovery、structured diagnostics、100K selection与candidate query budgets。

### ED67-M8：产品资格与跨引擎基线

完成Windows设备/DPI矩阵、render/input golden、save/reopen、fault/soak/profile；以同scene、同输入、同snap语义与Unreal/Godot/Fyrox/Bevy可达功能比较正确性和延迟。全部资格门通过后才允许声明工程级或性能领先。

## 9. 资格门

以下48项门在本轮全部为Fail；存在类型名、图标、局部函数或单一happy-path test不计通过。

| ID | 资格门 | 当前 |
|---|---|---|
| TRF-GATE-01 | Session绑定viewport/document/world/tool/input generation | Fail |
| TRF-GATE-02 | Selection roots在drag开始冻结且不含ancestor/descendant重复应用 | Fail |
| TRF-GATE-03 | World/local/parent transform与parent inverse可复验 | Fail |
| TRF-GATE-04 | Active/median/bounds/individual/custom pivot行为明确 | Fail |
| TRF-GATE-05 | World replacement、target deletion、provider unload有terminal receipt | Fail |
| TRF-GATE-06 | Multi-target preview/commit/cancel保持原子或明确全量回滚 | Fail |
| TRF-GATE-07 | 每目标admission/rejection不依赖裸NodeId | Fail |
| TRF-GATE-08 | Drag期间selection/settings/camera变化具有freeze或rebase政策 | Fail |
| TRF-GATE-09 | Axis move使用ray/line/plane几何并覆盖近平行退化 | Fail |
| TRF-GATE-10 | Plane与view-plane move结果通过golden | Fail |
| TRF-GATE-11 | Axis ring旋转使用有符号角并连续跨越pi | Fail |
| TRF-GATE-12 | View ring与trackball rotation通过golden | Fail |
| TRF-GATE-13 | Axis/plane/uniform scale基于drag-start乘法结果 | Fail |
| TRF-GATE-14 | Negative/zero scale和mirror政策由目标合同决定 | Fail |
| TRF-GATE-15 | NaN/Inf/zero determinant/non-unit quaternion在preview前拒绝 | Fail |
| TRF-GATE-16 | Nonuniform parent/shear不可表示结果不静默污染TRS | Fail |
| TRF-GATE-17 | Local gizmo使用world-resolved basis | Fail |
| TRF-GATE-18 | Global/Local/Parent/View/Normal/Workplane有效空间可追踪 | Fail |
| TRF-GATE-19 | 请求space与effective space不一致时UI有typed reason | Fail |
| TRF-GATE-20 | Group rotation围绕pivot正确更新每目标translation | Fail |
| TRF-GATE-21 | Individual Origins scale/rotate结果可复验 | Fail |
| TRF-GATE-22 | Pivot placement、reset和undo/redo闭环 | Fail |
| TRF-GATE-23 | Gizmo目标device-pixel尺寸跨FOV/DPI/投影稳定 | Fail |
| TRF-GATE-24 | Visual element、hit identity、constraint与solver一一对应 | Fail |
| TRF-GATE-25 | Grid visibility与translate/rotate/scale snap独立 | Fail |
| TRF-GATE-26 | Grid descriptor消费真实workplane/origin/unit/step | Fail |
| TRF-GATE-27 | Grid minor/major/fade跨距离与投影render golden通过 | Fail |
| TRF-GATE-28 | Absolute、relative与preserve-offset snap分别验证 | Fail |
| TRF-GATE-29 | Temporary snap override/precision/axis lock有输入资格 | Fail |
| TRF-GATE-30 | Surface/vertex/bounds/object candidate provider可替换且带generation | Fail |
| TRF-GATE-31 | Candidate排除selection自身并有bounded query/hysteresis | Fail |
| TRF-GATE-32 | Snap hint与receipt准确展示target、rule和fallback | Fail |
| TRF-GATE-33 | Numeric absolute/delta entry复用当前constraint与solver | Fail |
| TRF-GATE-34 | Numeric unit/expression/confirm/cancel不旁路transaction | Fail |
| TRF-GATE-35 | Multi-target command含tool/space/pivot/snap语义 | Fail |
| TRF-GATE-36 | Move/Rotate/Scale历史label与merge identity准确 | Fail |
| TRF-GATE-37 | Preview failure、cancel、undo、redo恢复同一事实 | Fail |
| TRF-GATE-38 | Toolbar/top bar/status bar/menu投影单一product model | Fail |
| TRF-GATE-39 | Checked/mixed/unavailable状态来自snapshot而非last command | Fail |
| TRF-GATE-40 | Snap/grid/space/pivot当前值可见且可直接选择/输入 | Fail |
| TRF-GATE-41 | Settings schema/version/scope/migration/corruption recovery通过 | Fail |
| TRF-GATE-42 | Plugin constraint/candidate provider unload不会遗留active session | Fail |
| TRF-GATE-43 | Deterministic input trace与headless math replay通过 | Fail |
| TRF-GATE-44 | Perspective/orthographic、多FOV/DPI visual golden通过 | Fail |
| TRF-GATE-45 | 1/1K/100K target transform latency/allocation达预算 | Fail |
| TRF-GATE-46 | Surface/vertex query与grid update达CPU/GPU预算 | Fail |
| TRF-GATE-47 | 8小时drag/snap/undo soak无漂移、NaN或history泄漏 | Fail |
| TRF-GATE-48 | 同语义跨引擎正确性与性能benchmark有可复验receipt | Fail |

## 10. 测试与验证矩阵

### 10.1 Runtime unit / property / fuzz

覆盖ray-line/ray-plane近平行、ring signed angle、trackball、axis/plane/uniform scale、world/local/parent conversion、pivot application、nonuniform parent/shear、NaN/Inf/overflow、absolute/preserve-offset rounding和deterministic replay。property必须验证零输入无副作用、结果finite、cancel恢复初值、相同输入相同receipt。

### 10.2 Editor integration

覆盖single/multi/root-filtered selection、parent/child混选、Active/Median/Bounds/Individual pivot、space切换、camera/workplane变化、numeric接管pointer drag、surface target deletion、provider unload、World replacement、transaction failure、undo/redo/save/reopen和settings migration。

### 10.3 Product与render golden

覆盖toolbar/top bar/status bar/menu同状态，snap独立开关、mixed/unavailable、keyboard/focus/accessibility、DPI 100/125/150/200%、perspective/orthographic、axis near-view、grid minor/major/fade、hover/active/snap target hint和long localized values。

### 10.4 Performance与soak

记录input-to-preview p50/p95/p99、solver CPU、candidate query、grid update、allocation、history size和render cost；固定1/1K/100K target、dense snap candidates、60/120/240Hz输入和8小时交替move/rotate/scale/snap/undo场景。

### 10.5 跨引擎比较

固定scene hierarchy、nonuniform parent、多选pivot、viewport/FOV/DPI、输入轨迹、snap step/target、硬件与warm-up；先比较最终transform误差和功能覆盖，再比较latency/CPU/GPU/alloc。Unity Graphics只作为consumer证据，不纳入无法取得同语义Scene View内部数据的结论。

## 11. Owner路由与非重复计数

| 父问题 | canonical owner | Editor67处理方式 |
|---|---|---|
| Selection identity、multi-selection、pivot父语义 | Editor03 | 消费qualified selection/pivot；不重复其P0 |
| Tool scheduler、resource lease、extension lifecycle | Editor53 | 注册constraint/provider adapter；不另建scheduler |
| Per-view session、surface/currentness | Editor58 | 将transform/workplane作为viewport payload；不另建view id |
| Input/capture/picking/cancel generation | Editor59 | 定义transform-specific input/terminal payload；不重复捕获P0 |
| Document transaction/history/merge/savepoint | Editor63 | 提供TransformCommandPlan；提交与journal仍由父owner |
| Placement preview transform | Editor65 | 复用Runtime kernel和snap provider；placement session仍归Editor65 |
| Camera navigation、projection与frame | Editor66 | 只消费camera ray/snapshot；不拥有navigation state |
| Scene transform/component/identity事实 | Runtime Scene父报告 | Runtime提供world frame/apply planner；Editor不复制scene authority |

## 12. 最终判定

当前实现可判定为“存在单节点X/Y/Z Move/Rotate/Scale原型、真实overlay/picking/preview/cancel/history链和project-scope步长设置”，不能判定为“工程级Scene Viewport transform manipulation”。最危险的不是缺少更多图标，而是显示frame、约束几何、层级写回、pivot、snap/grid和transaction没有共享一份可验证事实；继续在三个`update_drag`中加条件只会扩大错误面。

实施必须先做ED67-M0/M1的RED和纯数学合同，再接selection/pivot与完整solver，之后才进入grid/surface/numeric和产品硬切。48项资格门全部通过且取得同语义动态证据前，不得宣称该领域达到或超过Unreal。
