---
related_code:
  - zircon_editor/src/core/document
  - zircon_editor/src/core/editing
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/selection
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/host/editor_scene_document_submission.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_plugins/prefab_tools
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/FileHelpers.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/HitProxies.h
  - dev/godot/editor/editor_data.cpp
  - dev/godot/editor/editor_node.cpp
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/Fyrox/editor/src/scene
  - dev/bevy/crates/bevy_scene
  - dev/Graphics
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 03 · Scene、Prefab、Selection、Mode、Gizmo 与 Picking 工程化差距

## 1. 结论

当前 Scene authoring 已经具有一批应当保留的工程基础：scene open/create 有 project-session ticket 与路径校验，`EditorAuthoringWorld` 把 workbench 与 runtime world 隔在稳定 gateway 后，核心 scene command 可逆且可写 journal，SelectionModel 支持 Edit/Play 双域和多选，Scene Mode registry/stack 有插件 panic 隔离，Hierarchy rename/reparent 已进入 transaction，pointer route 也能采用 renderer-visible spatial broad phase。这些实现不能再被旧计划笼统描述为“只读 hierarchy”或“完全没有模式系统”。

但产品闭环仍有三条确定的数据完整性 P0：打开非默认 scene 后 Save Project 会把当前 world 写到 manifest 的 default scene；打开另一 scene 会先替换 world、清空 history/selection，完全没有 dirty/close 决策；scene DTO 中存在的 `prefab_instance` 在 World 转换时被忽略，保存又无条件写成 `None`。这三条都不是功能丰富度问题，而是会覆盖错误文件或静默丢失用户数据。

其余结构仍停留在“单 world + primary selection + proxy picking + descriptor-only prefab”的早期产品形态。多选只在模型和输入层成立，下游 highlight、handle、frame selection 与 overlay context仍只看 primary；子节点 gizmo 混用 local/world transform；拖动期间直接修改 world，释放时才补录 transaction；Prefab 菜单虽可见，五个 operation 均没有 command factory，Prefab Editor 本身还明确显示 placeholder，runtime plugin importer则只返回“backend is not installed”。因此本轮结论是 `review_complete / implementation_pending`，不是 scene authoring 已达到工程引擎标准。

本报告记录 3 个 P0、30 个 P1、8 个 P2。没有运行 Cargo、Editor、真实 GPU picking、超大 scene、父子变换、process-kill 或磁盘故障测试；所有性能结论只写复杂度和缺失预算，不宣称当前实现慢于或快于 Unreal。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 干净 production 文件/行数 | 本轮状态 |
|---|---:|---|
| Scene Mode | 16 / 915 | E3：registry、factory、activation、stack、isolation、context 与 overlay builder；fingerprint `48eb401c...f784f4e` |
| Selection | 5 / 282 | E3：Edit/Play domain、primary、replace/extend/toggle、generation；fingerprint `6015f9a2...5de0ebc` |
| Viewport | 111 / 5,999 | E3：controller、camera、handles、interaction、pointer route、precision source、render extract；排除2个在途文件；fingerprint `de2952cd...ec49a35` |
| Scene document/product clean set | 9 / 2,144 | E3：lifecycle、route、installer、world replacement、state/gateway；fingerprint `24fcc12f...05257` |
| Prefab vertical clean set | 17 / 4,176 | E3：asset DTO/document/cache、World I/O、Prefab Tools runtime/editor/dist与placeholder pane；fingerprint `00c8603e...91b1b` |
| focused combined evidence | 167 / 17,004 | 101个test attributes、0 ignored；未运行，包含production内联测试与专用测试 |
| dedicated clean test files | 8 / 2,551 | 60个test attributes、0 ignored；排除在途interaction extract测试 |

fingerprint 使用按相对路径排序后的 `path + NUL + per-file SHA-256` 清单再次计算 SHA-256。它只用于说明本轮实际阅读的干净文件集合，不是版本兼容 ID，也不能替代构建和行为测试。

### 2.2 在途文件隔离

成文时以下相关路径有其他 Session 或用户修改，本报告不把它们纳入 clean fingerprint，也不对其局部新行为给出完成结论：

- `zircon_editor/src/scene/viewport/interaction_extract/cache.rs`
- `zircon_editor/src/scene/viewport/interaction_extract/tests.rs`
- `zircon_editor/src/ui/host/scene_inspection_publication.rs`
- `zircon_editor/src/ui/layouts/views/hierarchy.rs`
- `zircon_editor/src/ui/retained_host/hierarchy_pointer/content_height.rs`
- `zircon_editor/src/ui/retained_host/hierarchy_pointer/row_metrics.rs`
- `zircon_editor/src/ui/workbench/project/editor_project_document.rs`
- `zircon_editor/src/ui/workbench/project/editor_project_document_save.rs`
- `zircon_editor/src/ui/workbench/project/project_root_path.rs`

`editor_project_document_save.rs` 当前差异只有 import 排序；`HEAD` 与 working tree 均调用 `world.save_scene_to_project(project, &project.manifest().default_scene)`。P0-01 因而不是根据在途改动推断，但实施前仍须重新读取 active owner。Hierarchy 的 rename/reparent 产品路径使用其余干净 callback、intent和transaction owner确认；本报告不评价当前在途 layout/virtualization细节。

### 2.3 本轮追踪的产品链

1. Scene picker ticket -> `SceneDocumentRoute::open/create` -> `ProjectSceneDocument` -> `EditorStateSceneInstaller` -> `replace_world` -> lifecycle activation。
2. File Save Project -> `EditorProjectDocument::save_to_project` -> runtime scene writer -> post-persist default-scene reimport。
3. Hierarchy/viewport input -> SelectionModel -> Scene Mode stack -> handle/pointer route -> `ViewportFeedback` -> world preview mutation -> core transaction history。
4. Render packet -> renderer-visible spatial snapshot -> precision candidate -> runtime pointer dispatcher -> selection/highlight/overlay publication。
5. Scene/Prefab TOML DTO -> artifact/project document -> `Scene::from_scene_asset` / `to_scene_asset` -> Prefab Tools command/asset toolkit/importer。

## 3. 已有工程基础，重构时必须保留

### 3.1 Scene route 与 authoring boundary

- Open/Create request绑定project session ticket、project root和scene URI；stale ticket、跨project路径和无效URI会在install前拒绝。
- Create使用staged source与catalog rollback；install失败不会把新scene伪装成成功打开。
- `AuthoringWorldSeed` 与 `EditorAuthoringWorld` 隔离runtime `LevelSystem`，workbench不直接长期持有可变runtime level。
- `DocumentLifecycleAuthority` 已能按 `(project_root, scene_uri)` 建立不同 document id，并在active scene切换时发布 Closed/Opened message。

### 3.2 Command、Hierarchy 与 Selection

- `EditorCommand` 的 create/delete/update/reflected-field命令实现 apply/revert/journal；delete捕获record，rename/reparent/transform捕获before/after。
- Hierarchy drag source会保留完整 authoritative selection；rename/reparent最终映射到 `EditorIntent` 和共享transaction，而不是在UI回调中直接改Scene。
- `SelectionModel` 使用稳定插入顺序的 `IndexSet`，具有primary、replace/extend/toggle、Edit/Play domain generation与cross-domain revision。
- Scene替换会清除selection，command执行可携带selection snapshot；这些是未来per-document selection authority可复用的primitive。

### 3.3 Scene Mode 与插件隔离

- `SceneModeRegistry` 同时拥有descriptor和factory，校验mode id与重复注册；builtin Select/Transform也走同一registry。
- `SceneModeStack` 支持base + overlays，input从顶层向下传递，PassThrough时恢复selection/effect checkpoint。
- `IsolatedSceneMode` 通过plugin boundary捕获panic，恢复SceneModeCtx并使overlay失效；enter失败会返回结构化 `EnterFailure`。
- Viewport overlay provider有capability gate、panic quarantine和prepare/install两阶段注册，不应退回无隔离的直接插件回调。

### 3.4 Viewport picking 与交互抽取

- Handle controller只计算 `ViewportTransformPreview`；最终world写入由workbench state接管，具备继续收敛到interactive transaction的边界。
- renderer-visible spatial snapshot在pointer事件时执行ray broad-phase，只投影返回owner，不再为每个事件重新扫描全部render mesh。
- Handle、scene gizmo与renderable统一进入precision candidate/runtime pointer dispatch，并用priority使UI handle优先于world object。
- Interaction extract有generation key/cache与显式invalidate，未来可升级而不重建第二套pointer真值。

这些基础说明目标不是推倒重写，而是把当前局部正确的owner提升为真正的Scene Document、Interactive Edit、Picking和Prefab authority。

## 4. P0：先封闭数据覆盖与静默丢失

### P0-01 · 打开非默认 Scene 后 Save Project 覆盖 default scene

`ProjectSceneDocument` 在open时保留所选 `scene_uri/source_path`，`DocumentLifecycleAuthority` 也知道active scene key；但 `EditorStateSceneInstaller::install_scene` 只把world和project root交给 `EditorState::replace_world`。`EditorState` 只有 `project_path: String`，没有active scene identity。保存端随后固定调用 `project.manifest().default_scene`，保存后的reimport也固定重导default scene。

结果是：用户打开 `project://scenes/level_b.scene.toml`、编辑并点击 Save Project，当前world会写入 `level_a` default scene；`level_b`仍未保存。必须让 `SceneDocumentSession` 唯一拥有source identity，Save从active document取得目标并执行generation/digest CAS。Save Project若语义是批量保存，必须枚举dirty documents；不得再把“当前world”与“default scene path”隐式配对。

### P0-02 · Open Scene 先替换 world 并清空 history，完全绕过 dirty 决策

`SceneDocumentRoute::open` 验证ticket后立即调用installer。`replace_world`进入exclusive transition，清空Global history/context，替换authoring world并重置selection；之后才激活新document。这里没有查询DirtyRegistry、当前scene saved generation、在途gizmo或close coordinator，也没有Save/Discard/Cancel。

任何未保存scene edit都会在打开另一scene时从内存消失，Undo也被同步清空。Open Scene、New Scene、Open Project、Close Project和process shutdown必须共享document transition coordinator：先冻结active document与dirty generation，完成Save/Discard/Cancel，再commit world/history/selection切换。不能只在tab close补prompt而保留scene route旁路。

### P0-03 · Prefab instance 元数据在 World load/save round-trip中被无条件删除

`SceneEntityAsset` 明确定义 `prefab_instance: Option<PrefabInstanceAsset>`，project document、artifact cache和asset management也会保存、统计、枚举其中的source引用。但 `Scene::from_scene_asset` 从未读取该字段，`to_scene_asset` 则对每个entity固定写 `prefab_instance: None`。生产代码中没有另一个World owner接收这份关系。

因此只要包含Prefab instance的scene被Editor加载并保存，instance source、local transform和所有overrides都会静默消失。M0必须先做到lossless preservation：在完整实例化系统落地前，Editor对未知/未展开的Prefab数据至少要使用sidecar/opaque retained component无损round-trip；若做不到必须拒绝保存并显示blocking diagnostic，绝不能继续写 `None`。

## 5. P1：产品化前必须闭合的架构与工作流

### P1-01 · 产品只有一个全局 authoring world，没有多 Scene/Level document模型

`EditorState` 只拥有一个 `EditorAuthoringWorld`、一个viewport controller、一个selection和Global history；打开scene就是整体replace。不存在并行scene tabs、per-scene history/dirty/viewport state、additive sublevel、streaming level、scene dependency或跨scene引用编辑。应让每个 `SceneDocumentSession` 独立拥有world/history/selection/viewports，workspace只保存active id和tab布局；运行时组合另由Level/World composition owner负责。

### P1-02 · Lifecycle 已有 scene identity，产品 state/save authority却把它丢弃

`DocumentLifecycleAuthority` 维护 `SceneDocumentKey { project_root, scene_uri }`，route document也保留source path，但安装边界只传 `&Scene`。这是两套owner真值：消息层知道文档，world/save层不知道。应把install输入升级为包含document id、source identity、base digest/schema、world和load diagnostics的不可变bundle，并由session registry原子接管。

### P1-03 · install 成功、lifecycle activation失败时没有回滚

Open路径先 `installer.install_scene(document.world())`，再 `activate_document(document)`；Create路径也在install后才finish/activate。后一步若因document id耗尽、lifecycle invariant或未来message admission失败，旧world/history已被删除，新world已安装，却向caller返回失败。Route必须使用prepare/commit协议：新session离线构建并验证，lifecycle reservation成功后一次commit；commit后发布失败只能进入可诊断degraded state，不能回到“操作失败但状态已变”。

### P1-04 · Authoring gateway 把所有错误压成 `None`

`try_with_world` / `try_with_world_mut` 对gateway error直接 `.ok()?`；许多caller随后把None显示为“No project open”或空feedback。DLL/session/backend故障、重复callback和真正未加载因而不可区分。应返回typed `Result<Option<T>, GatewayError>`，产品把Disconnected/Faulted/StaleGeneration/NotLoaded分别投影，并在写操作失败时保留document incident。

### P1-05 · Scene保存、渲染和产品查询依赖整棵Scene clone

`try_snapshot()`克隆完整Scene；Save Project先取得whole-scene snapshot，route/install也在多个边界clone world。大型world会产生与节点、组件、资源引用总量同阶的峰值内存和UI停顿。目标应是generation-bound immutable snapshot、copy-on-write chunk或streaming serializer；渲染、save和inspection分别订阅所需域，不能用完整Scene clone作为通用线程安全协议。

### P1-06 · Scene object命令缺少Duplicate、Copy/Paste与批量结构编辑

核心命令只有CreateNode、DeleteNode、UpdateNode和SetReflectedSceneField。工程工作流缺duplicate subtree、copy/paste、paste-as-child、replace、group/ungroup、batch rename/reparent、asset drop instancing等可逆命令。所有结构操作必须保留稳定ID remap、internal/external references、selection restoration、resource dependencies和journal codec，不能由UI临时拼接多次单节点修改。

### P1-07 · Component topology没有命令所有权

Reflected field只能修改已存在、已注册且editable的字段；没有Add/Remove/Replace/Reorder component、reset-to-default、copy/paste component或multi-object component edit。Prefab override也无法表达component topology差异。需要component schema registry + typed topology commands，并明确unique/multiple component policy、dependency validation、constructor failure rollback和serialization migration。

### P1-08 · Hierarchy只表达parent关系，缺工程Outliner语义

Scene node没有显式sibling order/folder/layer/visibility/lock/isolation/editor-only状态，Hierarchy drop只能改parent，不能before/after排序。大型工程还需要filter-aware range selection、rename validation、multi-drag cycle preview、hidden/locked selection policy和level ownership。Editor-only组织信息应进入workspace/authoring metadata，不要污染runtime transform hierarchy；影响runtime顺序的字段则必须由Scene schema拥有。

### P1-09 · Selection只按Edit/Play分域，不按document/viewport/tool分域

一个全局SelectionModel被所有scene route和viewport共享，entity id只是 `u64`。未来多document、multiworld或多个viewport会发生id碰撞和焦点串扰；节点删除后的stale id也没有中央prune/reconcile owner。Selection key应至少包含document/world generation + typed object id，selection session按document持有，并提供viewport/tool临时selection overlay与失效事件。

### P1-10 · Shift 只等价于Extend，没有Hierarchy range anchor

SelectionMutation支持Replace/Extend/Toggle，但没有anchor、visible order snapshot或Shift range语义。Hierarchy长列表中Shift-click不能选择连续区间，filter/expand变化也没有定义。需要由Hierarchy projection发布generation-bound visible ordering，selection command携带anchor和range policy；Ctrl/Command平台映射也应由input policy统一拥有。

### P1-11 · Multi-selection只在模型层成立，下游仍是primary-only

Render packet只接收 `selected: Option<u64>`，selection highlight和anchor最多生成一个；handle build/pick context同样只有一个selected；Frame Selection只看active primary；overlay provider context也只暴露primary。框选得到多个对象后，用户只能看到和变换一个。应让selection snapshot携带ordered items/primary/bounds，渲染全部highlight，gizmo按pivot policy作用于全部root selections，并保证父子同时选中时只应用一次world delta。

### P1-12 · 子节点Gizmo把local transform当world transform

`selected_basis` 和 `begin_transform_session` 直接读取 `node.transform`，而scene渲染、camera和selection定位会使用 `Scene::world_transform`。有平移、旋转或缩放父节点时，handle会画在错误世界位置；Global move又把world axis delta直接加到local translation，Local rotation/scale也没有通过parent inverse转换。必须在drag开始冻结world/local/parent matrices，工具在world中求delta，再以可逆且处理non-uniform/negative scale的规则写回local transform。

### P1-13 · Gizmo preview不属于core interactive transaction

开始拖动只调用 `ensure_mutation_available()` 并保存initial/latest；每个pointer move直接 `scene.update_transform`，释放时才构造 `already_applied` command补进history。拖动期间autosave、inspection、plugin observer和crash snapshot可看到未提交的中间状态，其他mutation也没有被真正transaction lease封闭。应由transaction engine拥有long-lived interactive edit：begin/reserve、preview publication、coalesce、commit/rollback；外部observer只看到明确标记的preview generation或最后commit。

### P1-14 · Move/Rotate/Scale全部写成“Move scene node”历史记录

`finish_gizmo_transaction` 无论active handle kind都调用 `execute_gizmo_scene_command("Move scene node", ...)`。Undo菜单、journal diagnostics和审计无法区分旋转/缩放，未来工具参数也无来源。Interactive edit session必须冻结tool id、axis/plane、space、pivot、snap policy和affected roots，并生成正确的typed label/journal metadata。

### P1-15 · 没有group pivot、individual origins与多选transform规则

Handle session只保存单node id和单initial transform。缺median/bounds center/active element/custom pivot、individual origins、world/local/parent space、多选父子去重和锁定轴。需要先定义selection transform graph与pivot contract，再实现一条batch command；不能循环调用单node update，因为中途失败会产生部分提交。

### P1-16 · Frame Selection只对primary点使用固定常量

实现取primary world position，perspective最小距离固定6，orthographic size固定2.5/距离比例；不读取mesh/scene gizmo/component bounds，也不框住multi-selection。应由authoritative spatial bounds service计算选择根集合的合并AABB/OBB/sphere，处理无bounds节点、极端尺度、近远裁剪和当前视向，并允许focus/orbit target与camera framing分离。

### P1-17 · ViewportInput不是完整编辑器输入协议

枚举只有pointer move、三键press/release、scroll和resize；没有key down/up、modifier snapshot、text/IME、double click、drag/drop、touch、pen pressure/tilt、focus/capture loss或pointer id。CancelInteraction另由命令路由注入，工具无法统一处理Esc/capture loss。应复用runtime typed input carrier，按window/viewport/pointer/device标识路由，并让Scene Mode声明capture与shortcut conflict policy。

### P1-18 · Viewport settings可Serialize却没有持久化owner

`SceneViewportSettings`包含handle、space、projection、orientation、gizmos、display、grid、lighting和skybox，但controller构造时始终Default，更新只写内存。只有snap step来自settings authority snapshot；没有workspace/per-project/per-viewport restore。应定义版本化 `ViewportWorkspaceState`，区分user preference、project workspace和transient interaction，窗口重开/scene切换后恢复且旧schema可迁移。

### P1-19 · Camera/navigation是单viewport固定常量模型

一个controller只有一个camera snapshot和orbit target；移动/缩放速度及距离下限硬编码，没有fly/WASD、speed ladder、focus orbit、camera bookmarks、pilot/lock actor、camera preview、per-viewport camera或输入设备配置。应由ViewportSession拥有独立navigation mode、camera binding和可持久化bookmark，active scene camera与editor camera也必须明确分离。

### P1-20 · Point picking仍是owner中心+半径代理，不是可见像素/几何精确命中

renderer spatial query只缩小owner集合，最终candidate仍把renderable transform translation投影为circle，radius由transform估算。复杂mesh、thin geometry、skinned/deformed/alpha-tested object和重叠物体都可能选错；depth只是中心投影。应提供renderer-owned hit proxy/ID buffer或CPU acceleration structure精确ray test，复用当帧visibility、LOD、instance id、material alpha policy和depth；proxy circle只能是明确的fallback。

### P1-21 · Picking adapter硬编码pointer、viewport和camera identity

`EDITOR_VIEWPORT_POINTER_ID=1`、`RenderViewportHandle=0`、camera id=0，PointerAction::Move的delta固定 `Vec2::ZERO`。多窗口、多viewport、多pointer或远程输入会共享同一身份，hover/capture/drag无法隔离。身份必须来自host publication和真实event，delta由stateful collector在同一pointer generation计算，窗口销毁/capture loss必须显式cancel。

### P1-22 · Pointer route把dispatch错误静默转成“无命中”

路由中存在 `.ok()?`，runtime dispatcher或layout错误会表现为hover消失/selection miss，没有diagnostic、retry或degraded state。Picking pipeline应返回typed result，区分NoHit、StaleSnapshot、SurfaceUnavailable、DispatchRejected和PluginFault；用户输入失败至少进入rate-limited activity diagnostic，不能改变selection后才丢失错误。

### P1-23 · Box selection查询所有代理圆，不遵循真实可见性和方向语义

rectangle query遍历全部renderable candidates和scene gizmo pick shapes，以circle/segment与矩形相交判定；没有occlusion、renderer-visible snapshot、left-to-right containment/right-to-left intersection、locked/hidden policy或near-plane clipping contract。大scene成本为 `O(N)`，遮挡物也可被框选。应建立屏幕空间visibility/selection index，按产品策略查询并记录selection source。

### P1-24 · 一个owner多mesh的去重依赖输入连续排列

`renderable_candidates`只比较 `previous_owner`；同一owner的mesh packet若不连续，会产生重复candidate，而且代表owner的position/radius只取第一份mesh transform，不能覆盖整体bounds。应由renderer extract直接发布per-selectable-owner bounds/instance records，并带generation和stable owner mapping，而不是在Editor猜测packet排序。

### P1-25 · 内建scene gizmo只覆盖Camera和DirectionalLight

Render packet按NodeKind只为Camera与DirectionalLight构建gizmo；Point/Spot/Rect/Ambient light、audio、probe、volume、collider、navigation、trigger、decal等没有统一可视化/选择语义。应由component/editor visualization registry贡献bounds、icon、wire shape、pick proxy和details，并受show flags、distance、selection及capability控制。

### P1-26 · SceneModeCtx不足以实现真实专业工具

Context只提供mutable selection、immutable viewport settings、一个input effect和overlay invalidation。插件没有document/world query、transaction command emission、camera/viewport/time、async task、tool state persistence、cursor/capture或diagnostics接口。Terrain、mesh paint、navmesh、spline等只能绕过owner直接改状态。应提供最小能力化service facade，每次callback持有document/viewport generation，写操作只能提交transaction command或preview session。

### P1-27 · Scene Mode只有enter故障会被消费，update/overlay/exit故障会静默丢失

`enter_mode`调用 `take_boundary_failure()` 并返回EnterFailure；handle_input/update/build_overlay发生panic后只是把mode标为faulted。`pop`、`replace_base`和`shutdown`在exit后不读取failure，对象随后drop，last_failure消失。应由ModeHost持续收集fault event、显示owner/mode/operation、禁用并允许安全retry/reset；exit失败还必须阻止资源lease被假定已释放。

### P1-28 · Overlay provider同步、无预算、primary-only且缺恢复产品面

Provider context只有 `&Scene` 和 `Option<u64>`，在render extraction同步返回无界 `Vec<SceneGizmoOverlayExtract>`；没有time/item/byte budget、camera/viewport/multi-selection/theme/depth policy。panic会quarantine，但last_failure只在Debug/toggle错误中可见，没有activity incident或reset。应改为generation-bound bounded extract，插件可离线准备immutable product，主线程只adopt；超时/超限/故障有typed terminal state。

### P1-29 · Prefab Tools五个可见operation都没有command factory

插件注册Create/Open/Apply/Revert/Break command、menu item、asset toolkit和creation template，但batch没有operation factory。产品dispatch明确在无event且无factory时返回 `MissingFactory`。`authoring.rs` 的函数只被测试直接调用；`apply_prefab_overrides`只去重并返回overrides后清空instance，未修改source Prefab，`break`也只返回小型report。必须在能力可用前安装真正transaction factory；否则隐藏/禁用命令并显示Partial原因，不能把descriptor数量当成功能完成度。

### P1-30 · Prefab不是runtime/editor实例系统，只是弱DTO与placeholder

Prefab pane明确显示“host slot is ready...tooling is still placeholder”；runtime plugin importer是 `DiagnosticOnlyAssetImporter("prefab importer backend is not installed")`。`PrefabAsset`只有name、scene和raw exposed property strings；override用entity/property字符串路径+untyped JSON，没有稳定source entity/component/property id、base prefab version/digest、orphan/migration状态、added/removed node/component或nested/variant/cycle信息。生产代码没有instantiate、source update propagation、override diff、apply/revert/break transaction和reference repair。需要单独的Prefab Graph/Instance authority，而不是继续扩充当前helper。

## 6. P2：一致性、诊断与长期可维护性

### P2-01 · 注册Scene Mode会提前执行插件factory

扩展安装为了校验mode id会创建candidate mode，实际激活时再创建一次。带资源分配、线程或外部副作用的factory即使模式从未被选择也会运行。Descriptor应携带声明id；factory只在activate时执行，创建后的id mismatch作为插件故障处理。

### P2-02 · Mode/selection revision使用wrapping计数却没有epoch

Mode stack和selection generation使用wrapping增量，snapshot/command evaluation只比较整数。长期session或fuzz下回绕会把新状态误认成旧代。使用process/session epoch + checked monotonic generation；接近耗尽时显式重建publication，禁止静默wrap。

### P2-03 · 一个Scene Mode发出第二个input effect会assert

`push_input_effect` 用assert保证一次dispatch只有一个effect。插件调用位于panic boundary时会被quarantine，但builtin或未来非隔离模式可直接panic；组合工具也无法表达capture + command等多效果。改成有界effect list或typed conflict error，由host验证能力和顺序。

### P2-04 · Scale handle硬编码最小0.05并禁止负/零scale

每轴scale直接 `(initial + delta).max(0.05)`，没有per-project policy、uniform/plane scale、sign crossing、镜像或接近零的数值策略。阈值应由transform policy和component约束决定；UI要显示clamp原因，journal保存实际结果。

### P2-05 · Overlay provider顺序由BTreeMap字符串排序隐式决定

多个provider的extract顺序是provider id字典序，没有显式layer、priority、depth group或stable tie-break contract。新增/重命名插件可能改变绘制与命中优先级。Registration应声明layer/priority/occlusion/pick policy，host按typed key稳定排序。

### P2-06 · Gizmo尺寸、颜色与pick阈值散落硬编码

Handle extent clamp、frame distance、circle/segment threshold、selection颜色等由局部常量决定，未进入DPI、accessibility、theme或user preference。收敛到versioned editor visualization settings，并对极端FOV、orthographic size、高DPI和色觉模式建立golden/picking gate。

### P2-07 · Viewport settings虽可序列化但没有schema/version

即使后续直接把 `SceneViewportSettings` 写进workspace，当前裸Serde结构也无法区分新增字段、旧枚举或平台默认变化。持久化carrier需要schema version、unknown-field policy、migration和safe defaults；runtime render DTO与editor workspace DTO不要复用同一长期格式。

### P2-08 · Viewport/Prefab多处把typed failure压成String

Mode/overlay toggle、gateway caller、gizmo transaction和插件helper大量以String传播，丢失owner、document generation、stage、retryability和state-changed标记。建立 `SceneAuthoringFailure` 分层错误与incident envelope，UI负责本地化；日志文本不能成为控制流协议。

## 7. 参考引擎结论与适用边界

Unreal的 `FEditorFileUtils::SaveMap(UWorld*, Filename)` 明确同时接收world和目标文件，`SaveDirtyPackages`/checkout prompt再负责批量dirty policy；它不会从“当前world”反推default map路径。`FEditorModeTools` 又把StartTracking/EndTracking、InputDelta、selection、pivot和orbit pivot放在同一编辑模式owner，`HHitProxy`为world、wireframe和UI提供显式priority。Zircon应吸收的是document/source identity、interactive transaction和renderer-owned hit identity，不复制UObject全局状态或Unreal历史兼容API。

Godot `EditorData` 保存 `edited_scene[]`，每个scene有root、path、selection和独立undo history id；切tab、close和save按index/path操作，`EditorNode`在关闭前检查unsaved history。`PackedScene::pack/instantiate`记录ownership、instance和editable instance关系，source scene并不是被扁平化后丢掉关联。Zircon可借鉴per-scene session与scene-instance ownership，但应使用typed stable IDs/digest，而不是照搬NodePath字符串作为长期override identity。

Fyrox `SceneEntry` 将selection、command stack和interaction modes放在每个scene container中，clipboard只复制selection中的root nodes并通过deep clone建立old-to-new mapping；move/rotate/scale/navmesh/terrain模式都接入controller和command stack。这直接证明Rust架构不需要用一个全局world/selection换取简单性。Zircon现有gateway和transaction engine更适合做稳定session边界，应保留而不是复制Fyrox具体Handle/API。

Bevy `bevy_scene` 提供DynamicScene、reflection和SceneSpawner等runtime primitive，不提供完整Editor document、dirty close、gizmo、picking或Prefab authoring authority；只能参考数据驱动spawn/reflect，不能把“Bevy也没有Editor功能”当作Zircon完成依据。仓内Unity Graphics主要是render packages、测试与工具，不含Unity Editor Scene/Prefab核心闭源实现，本报告不从缺失源码推断Unity行为。

## 8. 目标架构

### 8.1 Scene Document Session

每个打开scene由 `SceneDocumentSessionRegistry` 拥有：DocumentKey、project/session generation、source URI/path/digest/schema、authoring world、history、dirty/save token、selection、viewports、prefab instance graph、load diagnostics和lifecycle state。Open/Create先离线prepare，经过dirty transition decision后原子commit；Save只从session读取source identity。

状态至少为：

`Preparing -> ReadyClean/ReadyDirty -> InteractivePreview -> SavePlanning/Saving -> ReadyClean | ReadyDirty | Conflict | Incident -> ClosePlanning -> Closed`

world replacement、history teardown和document message发布必须是一次可审计transition，不能分散在route、installer和workbench三处。

### 8.2 Interactive Edit Authority

`InteractiveEditSession` 由transaction engine创建，冻结document/world generation、tool、selection roots、pivot、space、snap和before snapshot。Preview只发布临时代次，Commit生成一个typed batch command，Cancel恢复全部affected roots；autosave默认读取最后committed generation。Hierarchy drag、property scrub、gizmo和未来terrain/paint工具都复用同一协议。

### 8.3 Selection、Spatial 与 Picking Authority

Selection key使用typed document/world/object identity；per-document model拥有primary和ordered set，viewport/tool可叠加hover与temporary selection。Renderer每帧发布generation-bound selectable spatial product：owner、instance/subobject、bounds、visibility、depth/hit proxy id和fallback CPU acceleration handle。Pointer只查询同代产品，Stale/Unavailable是typed结果；box、ray和GPU ID picking共享visibility与policy。

### 8.4 Prefab Graph / Instance Authority

Prefab source为带稳定node/component/property IDs的版本化graph；instance保存source asset id + base revision/digest + structural/property override log。实例化生成source-to-instance mapping，支持nested prefab/variant、added/removed nodes/components、orphan detection和schema migration。Apply/Revert/Break均为跨scene/source的transaction plan：preflight writable/conflict/reference，原子写source/scene，失败完整rollback，成功更新所有受影响instances与dirty generation。

### 8.5 Scene Tool Extension Host

Mode/overlay插件只得到capability facade：immutable document/selection/camera snapshot、bounded async preparation、interactive edit/command submitter、cursor/capture和typed diagnostics。所有callback有time/item/byte budget、owner/mode generation与terminal state；panic只是其中一种故障，hang、超限、stale output和shutdown lease同样必须处理。

## 9. 分阶段重构计划

### M0 · 封闭Scene/Prefab数据丢失

1. 把active scene DocumentKey/source URI贯穿route -> installer -> EditorState -> Save；删除保存到default scene的隐式路径。
2. Open/Create/Switch Project统一进入dirty transition coordinator，先Save/Discard/Cancel，再commit world/history替换。
3. Scene asset round-trip无损保留 `prefab_instance`；未支持的实例禁止保存并给出blocking diagnostic。
4. Prefab command在没有factory/backend时不再宣告可用；现有source文件不允许由placeholder helper清空override。

### M1 · 建立per-document authoring session

1. 引入SceneDocumentSessionRegistry，迁移world/history/selection/viewport/source identity与dirty/save token。
2. Route采用prepare/reserve/commit，install/activate失败可rollback；Gateway返回typed error。
3. 实现multi-scene tabs和per-document workspace restore，再扩展additive level/streaming composition。
4. Save改为immutable/streaming snapshot与generation/digest CAS，去除UI thread整Scene clone。

### M2 · 收敛Command、Selection与Gizmo

1. 增加component topology、duplicate/copy/paste、batch hierarchy命令与stable ID/reference remap。
2. Selection升级为typed per-document snapshot，补range anchor、stale prune、locked/hidden和多选root policy。
3. Gizmo改用world-space frozen basis + parent inverse，支持multi-selection、pivot/space、uniform/plane transform。
4. 用core InteractiveEditSession替代私有capture，preview/commit/cancel/autosave/observer语义统一。

### M3 · 收敛Viewport、Picking与Tool Host

1. 建立per-viewport input/camera/settings session，补键盘、modifier、capture loss、touch/pen和navigation模式。
2. Renderer发布selectable spatial/hit proxy product；point/box picking按同代visibility、depth和policy执行。
3. Scene Mode/overlay采用能力化context、bounded product、fault incident和reset/shutdown lease。
4. Component visualization registry覆盖light、camera、audio、physics、probe、volume、navigation等authoring gizmo。

### M4 · 完成Prefab产品与大型工程门禁

1. 版本化Prefab graph、stable source IDs、nested/variant/override diff与实例更新传播。
2. Create/Open/Apply/Revert/Break接入transaction、save coordinator、source-control和conflict UI。
3. 建立100k/1m node、多scene、多viewport、深层父子、数万Prefab instance、插件超限/故障profile矩阵。
4. 在同负载、同画质、同机器下与Unreal/Godot/Fyrox比较open/save、selection、gizmo、picking、memory和recovery；没有证据前不得宣称优于Unreal。

## 10. 验收门

1. 打开任意非default scene后Save只修改该scene；default scene内容和mtime保持不变。
2. Dirty scene经Open/New/Switch/Close/Exit任一路径都必须得到Save/Discard/Cancel，Cancel后world/history/selection完全不变。
3. Route在prepare、install、activation、message publication任一阶段注入失败时，旧session保持可编辑，或新session进入明确committed incident；不存在“返回失败但已换world”。
4. 含Prefab instance的scene执行load/save/reload后source identity、local transform和override逐字段一致；未知字段也不丢失。
5. 两个以上scene可同时打开，各自history/dirty/selection/viewport/source identity隔离；切tab不clone或重载无关world。
6. 100k node scene保存使用有界峰值内存与非阻塞UI snapshot；报告captured/committed generation和content digest。
7. Duplicate/copy/paste复杂subtree后internal references正确重映射，external references按policy保留；undo/redo完全恢复selection与ID关系。
8. Add/Remove/Replace component失败不会留下半个component或部分history；journal replay与直接执行结果一致。
9. 旋转+非均匀缩放父节点下，子节点Local/Global move/rotate/scale手柄位置与写回数学正确，undo逐bit/容差恢复before。
10. 多选含父子、locked、无bounds和极端scale节点时，pivot、group delta、highlight、frame selection与undo符合单一策略。
11. 拖动期间autosave只读取last committed generation；Cancel/capture loss/plugin fault恢复全部对象，Commit只生成一个transaction。
12. 多窗口、多viewport、多pointer同时输入不串hover/capture/selection；窗口销毁会终止对应interaction。
13. 重叠、thin、alpha-tested、instanced、skinned mesh的point picking与当帧可见像素/geometry一致；stale snapshot不会静默选中旧owner。
14. Box selection在100k/1m selectable下有明确CPU/内存预算，遵守occlusion、方向、hidden/locked和frustum policy。
15. Scene Mode/overlay panic、hang、超时、超量、stale output和exit失败均产生owner/mode/stage incident，可禁用/重试且不破坏document transaction。
16. Prefab nested/variant/source update能保留合法override、标记orphan、检测cycle；Apply/Revert/Break全程可undo并通过write-time conflict CAS。
17. Prefab importer、Editor toolkit和packaged runtime使用同一schema/semantic version corpus，N/N-1 migration和坏输入有有界failure。
18. 产品gate运行真实Editor window、renderer、filesystem和child process；source-string `contains()`测试不能单独作为上述验收依据。

## 11. 与既有计划的关系

- [Editor05](../../zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md) 中“Hierarchy仍只读”的旧判断已过时：当前rename/reparent、多选模型、Scene Mode registry/stack、transaction preview adapter和renderer-visible broad phase均已有真实实现。本文保留这些基础，同时把active scene identity、dirty scene switch、parent-space gizmo、primary-only多选和精确picking列为新的工程门。
- [Editor03](../../zircon_editor/editor/03-command-transaction-and-undo.md) 继续拥有transaction engine与scene command基础；本文只定义Scene InteractiveEdit、batch hierarchy/component command和per-document history如何消费该基础，不另建平行undo系统。
- [Plugin12](../../zircon_plugins/12-plugin-dx-and-structure-framework.md) 拥有第一方插件结构与Prefab Tools package方向；本文以当前源码证明descriptor/toolkit已注册但factory/importer/authoring语义未安装，并给出Prefab Graph/Instance的完成定义。
- [Optimize Editor02](02-document-transaction-save-autosave-recovery-review.md) 已拥有统一dirty/save/close/recovery authority；P0-01/P0-02要求Scene route成为该authority的participant，而不是另建scene专用prompt。
- 本文只关闭scene/prefab/selection/mode/gizmo/picking首轮静态审查。Content import/reimport、Inspector/property authoring、Editor plugin UX其余部分、Play viewport/runtime bridge与large-project workflow仍需后续独立报告。
