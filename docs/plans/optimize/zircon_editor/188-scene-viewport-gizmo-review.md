---
title: Editor Scene Viewport Transform Manipulation、Gizmo、Pivot、Coordinate Space、Grid、Snapping、Workplane、Numeric、Surface/Vertex Alignment、Preference、Transaction 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor188
review_date: 2026-08-27
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_editor/src/core/editing/interactive_transform
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editing/command/batch_transform.rs
  - zircon_editor/src/scene/viewport/handles
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_interaction_cancel.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_reset_from_scene.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_feedback.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/binding/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/ui/workbench/model/status_bar_model.rs
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/grid_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/grid
tests:
  - zircon_editor/src/core/settings/tests/persistence.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/transaction_engine/journal_scene_commands.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/toolbar_dispatch.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/typed_command.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/status_bar.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/67-editor-scene-viewport-transform-manipulation-gizmo-pivot-coordinate-space-grid-snapping-workplane-numeric-surface-vertex-alignment-preference-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/174-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
  - docs/plans/optimize/zircon_editor/180-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/186-editor-scene-object-creation-placement-palette-factory-asset-drag-drop-template-favorites-preview-transform-transaction-plugin-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/187-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/67-editor-scene-viewport-transform-manipulation-gizmo-pivot-coordinate-space-grid-snapping-workplane-numeric-surface-vertex-alignment-preference-transaction-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/67-editor-scene-viewport-transform-manipulation-gizmo-pivot-coordinate-space-grid-snapping-workplane-numeric-surface-vertex-alignment-preference-transaction-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Scene Viewport Transform Manipulation、Gizmo、Pivot、Coordinate Space、Grid、Snapping、Workplane、Numeric、Surface/Vertex Alignment、Preference、Transaction 与 Product Integration 当前源码复核

## 1. 结论

Editor67之后出现了值得保留的实质进展。新的`InteractiveTransformSession`会冻结Document、world generation、去除ancestor/descendant重复后的selection roots、primary world pivot、每目标before/world matrix/parent inverse，并在预览前完成finite/TRS residual校验；多目标preview具有prefix rollback，cancel具有恢复失败分类，finish生成`BatchTransformCommand`，Move/Rotate/Scale也有准确history label和versioned journal payload。这些改动把旧的“单节点local preview + 固定Move label”提升为真正的多目标authoring transaction底座。

但上层gizmo数学几乎原样保留。`TransformHandleDragSession`仍只携primary、X/Y/Z、Local/Global、裸step和一个snap bool；Move/Rotate/Scale继续共用`projected_axis_delta`。Rotate仍是像素标量乘`0.01`，Scale仍是单轴加法并硬夹`0.05`，center anchor仍不可交互。batch session只是把primary算出的world delta传播到root集合，pivot始终是primary origin；它没有解决group pivot、individual origin、plane/view handles、ray-plane、surface/vertex、numeric entry或typed receipt。

Grid/Snap产品也未工程化。`GridMode`继续同时决定可见性与三类snap；extract只有`visible/snap_enabled`，Runtime固定生成XZ、`-10..10`、1单位间距的21x21网格。Scene toolbar可循环三种GridMode，但Workbench `ToggleSnap`仍固定发送`VisibleAndSnap`，componentized projection对任何`SetGridMode`都把Snap置active。三个project-scope scalar与异步settings authority是真实底座，却不是versioned snap/workplane profile。

本轮没有新增P0。Editor67的27项P1当前为 **20 Open / 7 Partial**，8项P2为 **7 Open / 1 Partial**；48门为 **34 Fail / 14 Partial / 0 Pass**。目标不是重写已经可用的transaction authority，而是让pure solver、qualified session、grid/snap事实、product model和receipt收敛到同一条链。

本轮只做review，没有修改production Rust，也没有运行Cargo、Editor、真实拖拽、save/reopen、multi-selection hierarchy、surface/vertex query、render golden、fault/scale/soak/profile或同硬件跨引擎benchmark。Tooling按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。当前不能声称该域的功能、表现或性能达到或超过Unreal。

## 2. 审查边界与冻结语料

### 2.1 Current working tree边界

主仓HEAD为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。transform/controller/settings选择集包含大量dirty与untracked源码，尤其`interactive_transform`、`batch_transform.rs`和多目标EditorState链尚未进入HEAD；本报告以读取时当前磁盘为事实源，不用旧Editor67 fingerprint或HEAD内容覆盖它们，也不回退或格式化其他会话修改。

MVP baseline recovery仍为`in_progress`，F4不能绕过F0-F3。本报告是后续RED、架构切分和hard cutover输入，不是实现完成receipt。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Transform authority/session/command | **8 / 2,185 / 2,010 / 77,284 / 4** | multi-root session、world/local writeback、rollback、batch command/journal、EditorState transaction | `1162c07527140ff91bb50f0220f4f501588d3eb2eaa29e599658e5f538f144dd` |
| Handle solver/controller/render contract | **38 / 2,955 / 2,652 / 97,362 / 21** | Move/Rotate/Scale、basis、overlay、pointer route、projection与feedback | `4f559e2b9af8d547311ca1e4faea4c01046d9315771311561662d391acd52344` |
| Grid/settings/product route | **22 / 5,527 / 5,046 / 224,080 / 11** | scalar authority、binding/route/status/ZUI、grid extract/pass/vertices | `dcc997cb3a943bac8d80172cb0d29587a4b5eba8a4c5ae5868c18fb80f03b05d` |
| Focused tests | **11 / 3,534 / 3,229 / 121,998 / 75** | settings、viewport lifecycle、batch journal、binding/toolbar/status | `36962a0040e0afd8782790a5f046f9cd5dfb65f2b808f2d202afcb052eb871e8` |
| Unreal selected set | **9 / 20,038 / 16,918 / 727,308 / 0** | Widget/mode/pivot、absolute movement、独立grid/rotate/scale/surface/actor/vertex snap | `afc25b7c21219f2247dbf60c7e6fee6c2957ebbe40e7e20e125433d1989fac7e` |
| Godot selected set | **4 / 12,590 / 10,703 / 500,558 / 0** | axis/plane/view/trackball、group pivot、vertex snap、numeric与undo | `e177810108f3af34c65346b0f00c11833d6bf48dd98e916ab33388a9c818c58a` |
| Fyrox selected set | **10 / 2,263 / 2,076 / 82,083 / 0** | selection roots、parent inverse、ray-plane、smart move与CommandGroup | `5d168842d99a905cdbdde37fa86d5e3392c2483831254e3df4b604178a7731db` |
| Bevy selected set | **3 / 1,389 / 1,280 / 49,458 / 0** | opt-in transform gizmo、ray-plane/atan2、view handle、snap与render split | `946b341ee7b29d7f610e2febe2825e6754d9a0a728ab8985fa3fb253e28190ca` |
| Unity Graphics selected set | **2 / 545 / 489 / 26,369 / 0** | pivotRotation、native snap、modifier与negative-size policy | `cf1b91f7526978f3bc23516ba1ceab4e68c480bbe9ea7165187b71ee2f1ccaf6` |

fingerprint方法为规范化相对路径、逐文件SHA-256、排序后的`path::hash`以当前环境换行连接，再对整体做SHA-256。它只证明选择集内容，不代表ABI、artifact、动态行为或性能。Godot、Fyrox、Bevy、Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal跟随主workspace。

### 2.3 Owner边界

Editor188只拥有transform solver、transform-specific session payload、pivot/space/workplane、grid/snap policy、numeric entry、semantic receipt和产品投影。Selection identity归Editor03/181，tool scheduler归Editor174，per-view session归Editor179，pointer/capture/cancel归Editor180，transaction/history归Editor184，placement归Editor186，camera navigation归Editor187。这里定义消费合同，不重复父finding。

## 3. 当前实现拓扑

### 3.1 新Session是真实底座，但不是完整Transform产品

`InteractiveTransformSession::begin`验证非空selection、primary membership和Mobility，冻结root-filtered targets、Document、world generation、world matrix与parent inverse。`preview`先对全部目标计算pending local TRS，再逐项写入；计算失败零写入，写入失败回滚已写prefix。`finish`生成多目标batch command，commit前还复验world generation与after state。这些使ED67-P1-01/02/10/11/26获得Partial。

边界仍不完整：没有Viewport/ViewInstance/Input/Capture/settings/provider generation；`pivot_world`固定primary且除暴露getter外不参与policy；request仍只有`primary + target_world`；command journal只存before/after targets，不存tool/axis/space/pivot/snap/terminal disposition。session error虽typed，上层`GizmoTransactionError`又把部分interactive error压成message。

### 3.2 World basis已修正，几何solver未修

controller会把primary selection归并到selected root，并用`scene.world_transform`构建overlay与drag起始transform；session再用parent inverse写回local，并以recomposition residual拒绝shear/non-TRS。这修正了旧的直接读取local basis路径。

但Move仍以screen-projected axis dot mouse delta再乘origin处world-units-per-pixel；Rotate仍用同一标量乘`0.01`；Scale仍用同一标量单轴加法并clamp到`0.05`。轴近平行视线时统一返回None，Rotate恰在最可见的view-facing ring方向退化。没有ray/line/plane closest point、signed `atan2`、drag-start ratio或continuous angle accumulator。

### 3.3 多目标采用primary world delta，不是Pivot系统

primary solver输出一个world transform，session计算`target * inverse(primary_frozen)`并左乘所有root frozen world matrix。Move多目标可获得共同delta；Rotate/Scale围绕primary origin传播，而不是Active/Median/Bounds/Individual/Custom策略。没有每目标effective space、per-target admission receipt或pivot placement/undo。

### 3.4 Overlay词汇仍超过可达交互

`GizmoAxis`和`InteractiveTransformAxis`只有X/Y/Z，registry仍硬编码Move/Rotate/Scale三字段。center anchor会绘制，但没有hit identity或solver；没有XY/YZ/ZX plane、view-plane、uniform scale、view ring或trackball。handle extent按distance/ortho size乘magic factor后clamp world size，pick radius也是world常量，未形成DPI/device-pixel不变量。

### 3.5 Grid/Snap仍是耦合布尔链

`SceneViewportSettings::grid_mode`只有Hidden、VisibleNoSnap、VisibleAndSnap。handle begin时把最后一种压成一个`snap_enabled`，Move/Rotate/Scale共同启用；`maybe_snap`只对相对scalar做round-to-step。没有absolute/preserve-offset、modifier override、candidate、hysteresis、target hint或receipt。

render extract只携`visible/snap_enabled`；grid vertex builder固定XZ平面、half extent 10、1单位步长、5线major interval和固定颜色。新增capacity test/ignored micro-benchmark只证明预分配，不证明workplane、dynamic step、camera fade或GPU budget，因此ED67-P2-06只到Partial。

### 3.6 Product route仍存在事实漂移

Scene toolbar的Grid按钮能在三态循环，三个snap按钮只循环隐藏preset且不展示值；status bar只显示translation step并拼接`m`。Workbench `ToggleSnap`固定发送`VisibleAndSnap`，componentized window对任意`SetGridMode`都执行`WorkbenchToolSnap = active`，所以Hidden/VisibleNoSnap也会显示Snap已启用。产品没有mixed/unavailable/effective policy或typed reason。

### 3.7 Tests证明生命周期，不证明solver

当前测试覆盖单次Move折叠为一个undo command、cancel恢复、secondary navigation guard、target deletion/stale world error、transaction fault rollback、world replacement/mode切换清理，以及batch journal含两个target。`batch_transform.rs`也测试commit前world generation变化拒绝。没有直接测试`InteractiveTransformSession`多root传播/parent inverse/TRS residual，也没有Rotate/Scale、degenerate axis、pivot、plane/view/uniform、snap/grid render、numeric entry或性能矩阵。

## 4. Canonical finding状态账本

### 4.1 P1架构、正确性与产品闭环

| Finding | 当前 | 当前证据 | 必须收敛到 |
|---|---|---|---|
| ED67-P1-01 session只冻结active primary local transform | Partial | 新session冻结Document/world generation、多选roots/world/parent inverse；仍缺view/input/settings generation | qualified session identity + immutable SelectionTransformPlan |
| ED67-P1-02 Local basis与parented world frame错误 | Partial | primary改用world transform，写回用parent inverse；缺层级golden | world-resolved frame与per-target exact writeback资格 |
| ED67-P1-03 只有Local/Global且无effective receipt | Open | spec仍只有两值 | Global/Local/Parent/View/Normal/Workplane + reason |
| ED67-P1-04 无Pivot Mode/placement/multi origin policy | Open | `pivot_world`固定primary且无policy | Active/Median/Bounds/Individual/Custom typed pivot |
| ED67-P1-05 Move不是world constraint solver | Open | 仍是screen axis dot + world/pixel | ray/line/plane solver与degenerate disposition |
| ED67-P1-06 无plane/free move且center是假暗示 | Open | axis enum只有XYZ，anchor不可pick | element/hit/constraint/solver一一对应 |
| ED67-P1-07 Rotate数学错误 | Open | 仍为`projected_axis_delta * 0.01` | plane intersection + signed atan2 + continuity |
| ED67-P1-08 无View ring/screen/trackball | Open | 类型、overlay、solver均无 | view ring与arcball contracts |
| ED67-P1-09 Scale为单轴加法且硬夹正值 | Open | 仍加法与`max(0.05)` | drag-start multiplicative ratio + target scale policy |
| ED67-P1-10 nonuniform parent/shear政策缺失 | Partial | residual检查会拒绝non-TRS | typed Exact/Rejected outcome + hierarchy tests |
| ED67-P1-11 缺finite/determinant/normalization gate | Partial | session使用ValidatedTransform/inverse/finite residual；handle math仍裸 | solver input/result统一numeric gate |
| ED67-P1-12 gizmo尺寸/pick tolerance不稳定 | Open | world factor/clamp与world pick radius | device-pixel/DPI/FOV invariant |
| ED67-P1-13 grid visibility与全部snap耦合 | Open | GridMode仍是三态总开关 | 独立visibility与三类snap policy |
| ED67-P1-14 固定XZ 1单位21x21 grid | Open | renderer内容未变 | typed GridRenderDescriptor |
| ED67-P1-15 无Grid/Workplane session/persistence | Open | 始终world XZ | per-view GridWorkplaneState与override |
| ED67-P1-16 snap profile只有三个scalar | Partial | project-scope schema/authority/persistence真实；无profile/preset owner | versioned resolved TransformSnapProfile |
| ED67-P1-17 只有relative delta snap | Open | `maybe_snap`只round scalar | anchor/rounding/offset policy与receipt |
| ED67-P1-18 无独立enable/temporary override/modifier | Open | 只有冻结bool | qualified modifier/effective policy |
| ED67-P1-19 surface/vertex/bounds/object snap缺失 | Open | provider/query/target filter为0 | bounded generation-qualified candidate providers |
| ED67-P1-20 无Snap receipt/hint/budget | Open | feedback只有node/world transform | chosen/rejected candidate + hint + hysteresis |
| ED67-P1-21 无Numeric Entry session | Open | transform-specific numeric类型/route为0 | 与当前constraint/transaction共用solver |
| ED67-P1-22 无precision/axis lock/input连续性 | Open | handle input没有modifier/rebase | freeze/rebase与唯一terminal disposition |
| ED67-P1-23 registry不能扩展专业constraint/provider | Open | 三字段+closed enum match | extension lease/generation/capability/unload |
| ED67-P1-24 Workbench Snap单向开启且active失真 | Open | 固定VisibleAndSnap；任意SetGridMode都active | resolved product snapshot投影 |
| ED67-P1-25 重复产品面缺可见值/typed menu | Open | toolbar循环隐藏值，status仅translation | 单一model的菜单/值/mixed/unavailable |
| ED67-P1-26 transaction payload无语义且label固定 | Partial | batch多target，三tool transaction label已区分；payload无tool/space/pivot/snap | semantic TransformCommandPlan/Receipt |
| ED67-P1-27 tests不证明solver/hierarchy/snap | Partial | 新增stale/rollback/batch journal测试；核心矩阵仍空 | math property + hierarchy + product/render golden |

### 4.2 P2质量与资格证据

| Finding | 当前 | 当前证据 | 必须收敛到 |
|---|---|---|---|
| ED67-P2-01 magic constants无单位/依据 | Open | solver/extent/pick/grid/preset仍分散 | profile/style/private solver constants |
| ED67-P2-02 状态栏硬编码米 | Open | translation step仍拼`m` | project unit + locale formatter |
| ED67-P2-03 accessibility/mixed状态不足 | Open | icon-only且无mixed/reason | keyboard/focus/name/非颜色反馈 |
| ED67-P2-04 无结构化Transform/Snap diagnostics | Open | 只有error/status/feedback | bounded trace + typed receipt |
| ED67-P2-05 无deterministic trace/math/render golden | Open | lifecycle tests不是solver replay | initial state + input + expected receipt/golden |
| ED67-P2-06 无hot-path/query/allocation/soak预算 | Partial | grid capacity有ignored micro-bench；产品预算仍无 | 60/120/240Hz、1/1K/100K、query/grid/soak profile |
| ED67-P2-07 preference无migration/corruption recovery | Open | 三scalar只有普通settings persistence | versioned profile migration/repair |
| ED67-P2-08 无同语义跨引擎基线 | Open | 未运行固定scene/input/hardware比较 | 功能同构后再比较p50/p95/p99/CPU/GPU |

## 5. 五套参考实现差异

### 5.1 Unreal：完整Widget、Mode、Pivot与Snapping policy

`FWidget`区分axis/plane/screen/arcball和absolute movement，使用custom coordinate system、drag-start cache与pixel-aware widget scale；ModeTools拥有pivot和selection应用。`FSnappingUtils`独立translation、rotation、scale、surface、actor、vertex policy与helper drawing。Zircon不应复制类层级，但必须达到同等事实分离、interaction vocabulary和result可解释性。

### 5.2 Godot：同一状态机内具axis/plane/view/trackball和vertex snap

Godot的3D editor在同一transform flow中处理多种handle、group pivot、snap modifiers、vertex candidate、numeric state和undo。它说明plane/view/trackball、group pivot与numeric entry不是可延期的装饰能力，而是变换工具的基本产品闭环。

### 5.3 Fyrox：较小Rust Editor也使用selection roots和ray-plane

Fyrox move/rotate/scale modes会处理selection roots、parent/world conversion、ray-plane intersection、smart move和CommandGroup。Zircon的新multi-root transaction已接近其事务边界，但几何仍低于这一Rust基线。

### 5.4 Bevy：最小opt-in gizmo也高于Zircon solver

Bevy transform gizmo把interaction与render插件拆开，状态冻结initial transform/drag-start world/gizmo origin，包含View handle，并用ray-plane、signed angle和起始值语义。它不是Unreal产品上限，却直接反证screen-axis标量可作为长期Rotate/Scale实现。

### 5.5 Unity Graphics：只作consumer与policy证据

Graphics镜像不是完整Unity Scene View源码；这里仅使用`Tools.pivotRotation`、`Handles` native snap、modifier和negative-size admission作为consumer证据，不以其文件规模为完整引擎比较。

## 6. 目标架构

### 6.1 Runtime pure math与scene facts

Runtime提供validated camera ray、line/plane closest point、signed angle/arcball、multiplicative scale、TRS representability、aggregate bounds和bounded spatial candidate query。它不拥有Editor selection、history、keymap或UI。

```text
ValidatedTransformInput + CameraRay + ConstraintFrame + DragStart
    -> TransformManipulationKernel
    -> TransformSolution { world_delta, per_target_result, repaired/rejected }

SceneSpatialSnapshot + SnapCandidateQuery
    -> bounded providers
    -> ranked candidates + completeness + generation
```

### 6.2 Editor qualified session与产品 authority

Editor179/180的per-view/capture identity与Editor184 transaction组合为唯一`ViewportTransformSession`。它冻结SelectionTransformPlan、pivot/effective space、resolved snap profile、settings/provider generations与input owner；preview和terminal receipt都带这些资格。现有`InteractiveTransformSession`应被吸收为其authoring-world participant，而不是另起第二套写回。

```text
qualified input + per-view session + immutable selection/pivot/profile
    -> pure solver + candidate providers
    -> typed preview receipt + render hint
    -> atomic authoring participant
    -> semantic TransformCommandPlan
    -> document transaction/journal
    -> single product snapshot/status/diagnostics
```

### 6.3 Grid/Snap单一事实模型

`GridVisibility`、`GridWorkplaneState`、Translate/Rotate/Scale policies、absolute/preserve-offset、candidate providers和temporary modifiers必须独立建模，再由一个resolved snapshot投影给solver、renderer、toolbar、Workbench和status bar。renderer不得从裸bool猜grid geometry，UI不得从last command猜active。

## 7. 分阶段重构计划

### Editor188-M0：真实性止血与RED基线

修复Workbench ToggleSnap与active谎报；不可交互center anchor移除或标Unavailable。先写Rotate view-facing axis、Scale ratio/negative policy、parented/nonuniform、多root pivot、grid descriptor、toggle false-state和commit receipt RED tests。

### Editor188-M1：Pure solver与numeric hardening

实现ray/line/plane Move、signed-angle Rotate、multiplicative Scale、finite/result gate和typed degeneracy；旧`projected_axis_delta`只可留作明确的screen-space mode，不得继续作为三solver共同authority。

### Editor188-M2：Selection plan、space与pivot

把现有root filtering/world/parent inverse提炼为immutable SelectionTransformPlan；加入Parent/View/Normal/Workplane、Active/Median/Bounds/Individual/Custom pivot与不可表示reason。

### Editor188-M3：完整handle vocabulary hard cutover

建立axis/plane/view/free/uniform/trackball descriptor与stable hit id，保证visual/hit/constraint/solver一一对应；以device pixel约束尺寸和tolerance。删除closed enum的平行旧path。

### Editor188-M4：Grid/Workplane与Snap profile

拆分visibility和三类snap policy，引入workplane/origin/unit/minor/major/fade descriptor及user/project/view resolution、schema migration和corruption recovery；renderer只消费descriptor。

### Editor188-M5：Candidate与Numeric Entry

接入bounded surface/vertex/bounds/object providers、self exclusion、generation、hysteresis和hint；Numeric Entry复用当前constraint/solver/transaction，支持absolute/delta/unit/expression/confirm/cancel。

### Editor188-M6：Qualified session与semantic transaction

补齐viewport/document/world/input/capture/settings/provider generation；preview/commit/cancel只产生一个typed terminal receipt。batch journal携tool/space/pivot/snap语义、targets和merge identity，保留现有原子rollback能力。

### Editor188-M7：单一产品模型与extension lifecycle

toolbar、Workbench、status/menu只消费resolved snapshot；支持checked/mixed/unavailable与reason。constraint/candidate provider接Editor174/171 extension lease，unload先quiesce并terminalize active session。

### Editor188-M8：设备、规模、故障和性能资格

完成mouse/trackpad/pen/keyboard、DPI/FOV/projection、1/1K/100K targets、candidate query、grid CPU/GPU、8小时soak与deterministic replay。功能同构后，再固定scene/input/hardware对Unreal/Godot/Fyrox/Bevy报告p50/p95/p99、CPU/GPU、allocation与memory。

## 8. 资格门

| Gate | 验收条件 | 当前 | 当前证据 / 缺口 |
|---|---|---|---|
| TRF-GATE-01 | Session绑定viewport/document/world/tool/input generation | Partial | Document/world/tool已冻结；缺viewport/input |
| TRF-GATE-02 | Selection roots冻结且去除ancestor重复 | Partial | 代码已root-filter；无专项多层级测试 |
| TRF-GATE-03 | World/local/parent inverse可复验 | Partial | world basis和parent inverse已接；缺golden |
| TRF-GATE-04 | 多种pivot行为明确 | Fail | 只有primary origin |
| TRF-GATE-05 | replacement/deletion/unload有terminal receipt | Partial | stale/delete/replacement错误清理存在；无统一receipt/provider unload |
| TRF-GATE-06 | multi-target preview/commit/cancel原子 | Partial | pending-first与rollback代码存在；无multi fault matrix |
| TRF-GATE-07 | per-target admission不依赖裸NodeId | Fail | 仍用NodeId，缺qualified target generation |
| TRF-GATE-08 | selection/settings/camera变化freeze或rebase | Partial | selection/spec/steps部分冻结；camera取current且无policy |
| TRF-GATE-09 | axis Move使用ray几何且覆盖退化 | Fail | screen axis近似 |
| TRF-GATE-10 | plane/view-plane Move golden | Fail | 不存在 |
| TRF-GATE-11 | ring Rotate signed angle连续跨pi | Fail | 像素标量 |
| TRF-GATE-12 | view ring/trackball golden | Fail | 不存在 |
| TRF-GATE-13 | axis/plane/uniform Scale基于起始乘法 | Fail | 单轴加法 |
| TRF-GATE-14 | negative/zero/mirror由目标合同决定 | Fail | 工具硬夹0.05 |
| TRF-GATE-15 | invalid numeric preview前拒绝 | Partial | session validator存在；handle math未统一 |
| TRF-GATE-16 | nonuniform/shear不静默污染TRS | Partial | residual会拒绝；缺测试和typed product receipt |
| TRF-GATE-17 | Local gizmo使用world-resolved basis | Partial | 当前路径已改world transform；缺层级golden |
| TRF-GATE-18 | 全部effective spaces可追踪 | Partial | Global/Local进入spec；其余不存在 |
| TRF-GATE-19 | requested/effective space差异有reason | Fail | 无admission receipt |
| TRF-GATE-20 | group Rotate围绕pivot更新translation | Fail | 只围绕primary |
| TRF-GATE-21 | Individual Origins可复验 | Fail | 不存在 |
| TRF-GATE-22 | pivot placement/reset/undo闭环 | Fail | 不存在 |
| TRF-GATE-23 | device-pixel尺寸跨FOV/DPI稳定 | Fail | world factor/clamp |
| TRF-GATE-24 | visual/hit/constraint/solver一一对应 | Fail | center anchor无interaction |
| TRF-GATE-25 | grid visibility与三类snap独立 | Fail | GridMode耦合 |
| TRF-GATE-26 | grid descriptor消费workplane/origin/unit/step | Fail | extract仅两个bool |
| TRF-GATE-27 | grid minor/major/fade render golden | Fail | 固定geometry且无golden |
| TRF-GATE-28 | absolute/relative/preserve-offset分别验证 | Fail | 只有relative scalar round |
| TRF-GATE-29 | temporary override/precision/axis lock合格 | Fail | 无modifier协议 |
| TRF-GATE-30 | replaceable candidate provider带generation | Fail | provider为0 |
| TRF-GATE-31 | candidate self exclusion/budget/hysteresis | Fail | query为0 |
| TRF-GATE-32 | hint/receipt准确展示target/rule/fallback | Fail | hint/receipt为0 |
| TRF-GATE-33 | Numeric Entry复用constraint/solver | Fail | session为0 |
| TRF-GATE-34 | Numeric unit/expression/terminal不旁路transaction | Fail | route为0 |
| TRF-GATE-35 | multi-target command含tool/space/pivot/snap | Partial | batch有targets，语义字段缺失 |
| TRF-GATE-36 | 三tool label与merge identity准确 | Partial | label已区分；merge identity缺失 |
| TRF-GATE-37 | preview failure/cancel/undo/redo恢复同一事实 | Partial | 单Move与batch基础有测试；无multi全矩阵 |
| TRF-GATE-38 | 全产品面消费单一model | Fail | toolbar/Workbench/status分裂 |
| TRF-GATE-39 | checked/mixed/unavailable来自snapshot | Fail | componentized按command猜active |
| TRF-GATE-40 | snap/grid/space/pivot值可见可直接编辑 | Fail | 隐藏循环与缺控件 |
| TRF-GATE-41 | settings schema/scope/migration/corruption通过 | Partial | scalar schema/scope/persistence存在；无profile migration/repair |
| TRF-GATE-42 | plugin unload不遗留active session | Fail | provider lifecycle不存在 |
| TRF-GATE-43 | deterministic input trace/headless replay | Fail | 无transform trace |
| TRF-GATE-44 | projection/FOV/DPI visual golden | Fail | 无golden |
| TRF-GATE-45 | 1/1K/100K transform预算 | Fail | 无profile |
| TRF-GATE-46 | candidate query/grid CPU/GPU预算 | Fail | 仅synthetic grid capacity bench |
| TRF-GATE-47 | 8小时soak无漂移/NaN/history泄漏 | Fail | 无soak |
| TRF-GATE-48 | 同语义跨引擎benchmark可复验 | Fail | 未运行benchmark |

## 9. 验证与当前源码守卫

静态守卫确认：`TransformManipulationKernel`、`ViewportTransformSession`、`SelectionTransformPlan`、`TransformSnapProfile`、`GridWorkplaneState`、`SnapCandidateQuery`、`TransformNumericEntrySession`、`TransformCommandReceipt`、`GridRenderDescriptor`在含untracked内容的当前Runtime/Editor树中均为0。当前solver仍命中`projected_axis_delta`三处consumer；Rotate仍命中`scalar * 0.01`，Scale仍命中`max(0.05)`；Grid extract仍只有`visible/snap_enabled`，vertex builder仍是half extent 10；Workbench ToggleSnap仍固定`VisibleAndSnap`，componentized projection仍对任意`SetGridMode`置active。

最低验证矩阵包括：pure geometry property/fuzz；parented/nonuniform/shear hierarchy golden；multi-root preview/fault/cancel/undo/redo；pivot/space/plane/view/uniform/trackball；absolute/preserve-offset和candidate target lifecycle；numeric entry；settings migration/corruption；product truthfulness；DPI/FOV/projection render golden；device/scale/soak/profile与同硬件跨引擎对照。

本轮没有运行Cargo或Editor，因此没有build/test/runtime/performance green声明。落盘只执行frontmatter path、finding/gate计数、索引唯一性、source guard、fingerprint currentness和diff whitespace静态检查。

## 10. 最终判定

Zircon已经有一条值得继续深化的多目标authoring transaction链，不能丢弃或绕开；但它目前只是把旧primary solver的world delta安全地传播到selection roots。工程级Transform Manipulation必须同时解决几何正确性、qualified identity、pivot/space、grid/snap事实、numeric/candidate、产品真实性和资格证据。

正确顺序是：**保留multi-root/rollback/batch authority -> pure Move/Rotate/Scale solver -> SelectionPlan/space/pivot -> complete handles -> Grid/Workplane/Snap profile -> candidate/numeric -> qualified receipt/semantic journal -> single product model -> device/scale/fault/performance资格**。在48门全部Pass前，该域保持“工程化重构待实施”。
