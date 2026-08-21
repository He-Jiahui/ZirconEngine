---
related_code:
  - zircon_plugins/navigation/editor
  - zircon_plugins/navigation/runtime
  - zircon_plugins/navigation/native
  - zircon_plugins/navigation/dist
  - zircon_plugins/navigation/plugin.toml
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/navigation
  - zircon_runtime/src/scene/navigation.rs
  - zircon_runtime/src/asset
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/05/2026-07-13-navigation-m6-output-records.md
  - docs/plans/zircon_plugins/05/2026-07-27-navigation-m6-selected-surface-manifest.md
  - docs/plans/zircon_plugins/05/failure-2026-07-15-navigation-bake-selection-operation-arguments.md
  - docs/plans/zircon_plugins/05/failure-2026-07-19-navigation-runtime-fallback-hotpath.md
  - docs/plans/zircon_plugins/05/failure-2026-07-27-navigation-world-scan-deserialize-value.md
  - docs/plans/zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md
  - docs/plans/zircon_plugins/05/failure-2026-08-02-navigation-editor-operation-status-v2-cutover.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/AI/Navigation
  - dev/Fyrox/editor/src/interaction/navmesh
  - dev/Fyrox/editor/src/scene/commands/navmesh.rs
  - dev/Fyrox/editor/src/settings/navmesh.rs
  - dev/godot/modules/navigation_2d/editor
  - dev/godot/modules/navigation_3d/editor
  - dev/godot/scene/resources/navigation_mesh.cpp
  - dev/bevy
  - dev/Graphics
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 19 · Navigation Settings、NavMesh、Agent/Area、Surface/Modifier/Obstacle/Off-mesh Link、Bake、Query 与 Debug Authoring 工程化差距

## 1. 结论

Zircon Navigation不是空实现。默认Editor target已链接Navigation runtime/editor插件；插件有真实Recast/Detour bridge和vendored基础、`DefaultNavigationManager`、scene system、settings校验、surface/modifier/agent/obstacle/link/bridge描述符、异步tile bake、Crowd/查询、typed runtime event、可拒绝stale frame的PIE mirror、正式`ViewportOverlayProviderRegistration`，以及通过Runtime Gateway保存before/after snapshot的可撤销Editor command。这些能力必须保留，不能因为产品层未闭环而另造第二套Navigation runtime。

但当前Navigation Editor仍有五个P0断点：

1. `NavigationOperationHandler::prepare`对Bake Scene和Bake Surface固定返回“navigation bake requires a pure prepare backend”，而focused runtime test仍断言相同operation成功Bake/Clear/Restore；当前生产实现、测试期望和Editor command三者互相矛盾，正式Gateway上的两类Bake必然失败。
2. 插件发布Surfaces、Agents/Areas、Bake、Debug、NavMesh Asset、Navigation Settings Asset和五个component drawer，但其中11个业务区域仍为无事件、无controller/data provider的`Space`；Bake虽有route，却没有surface数据、诊断、进度或资产状态producer。
3. `NavigationBakePanelController`、retained selection/progress模型和backend abstraction只有测试构造，未进入产品host；资产toolkit只能打开空view，缺少transaction、source revision、异步job、staging artifact、原子save/reload与Undo成本控制。
4. PIE overlay数据链已经真实存在，但“Toggle Navigation Gizmos”只有payload schema，没有event、factory或持久view state；四个filter checkbox没有事件，provider只读PIE mirror且每帧复制完整mesh/agent数据，编辑态NavMesh、query preview、tile/area/cost诊断均未闭环。
5. 内置Navmesh AI Workbench是与插件平行的第二套静态产品面：`NavMesh_Main`、`Agent_Humanoid`、18 tiles、96 polys、4 agents、42 cm/180 cm、Door_A03等全部为固定ZUI/feedback文本；Rebuild、Query Path和字段Commit不调用Navigation operation、job或runtime。

本报告记录5个P0、60个P1、12个P2，给出M0-M8与32个验收门。目标是建立`NavigationAuthoringDocument + NavigationSettingsAsset + SceneProjection + BakeSourceSnapshot + Cancelable Bake Job + Immutable NavMesh Artifact + QueryPreviewSession + Generation-bound Debug Snapshot`。本轮仅做静态review，不修改production代码。[Runtime Navigation 08D](../zircon_runtime/08d-navigation-runtime-review.md)继续拥有world/backend/query/crowd/movement/off-mesh/large-world运行时差距；本报告聚焦Editor authoring、资产事务和产品证据。

上一轮同一工作树的`zircon_editor --lib`测试编译在617.2秒后被239个既有test-build错误和122个warning阻断，本轮不重复同一Cargo lane；97个test attributes只是静态inventory，不能作为当前动态通过证据。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Navigation editor/package | 34 / 3,838 / 134,903 | E3逐文件：registration、operation command、panel、mirror/provider、11份ZUI及focused tests；fingerprint `d881eedb...739502` |
| Runtime operation/scene/plugin生产桥，不含tests | 59 / 8,273 / 287,822 | E3复核Editor所需operation、bake、scene、event、manager接点；fingerprint `7a8527ff...529081` |
| Native Recast/Detour bridge与vendor | 91 / 35,032 / 1,087,347 | E2按ABI、build/query/crowd/tile-cache职责复核；fingerprint `2536c84e...3d6f1` |
| Core Navigation与Asset合同 | 30 / 4,737 / 170,064 | E3公共DTO、settings、asset/import/artifact/load纵向接点；fingerprint `5876a582...e4f1` |
| Editor assembly/toolkit/provider共享接点 | 7 / 1,453 / 52,989 | E3默认target装配、asset open与extension/provider合同；fingerprint `9a8f9fd6...87c03` |
| 静态Navmesh AI Workbench链 | 7 / 2,644 / 141,758 | E3读取ZUI、binding、navigation、preview action和feedback；fingerprint `3ba21b54...429d` |
| selected combined scope | 228 / 55,977 / 1,874,883 | 当前工作树去重fingerprint `d37f25c5...1f98c`；97个test attributes、0 ignored、1个import排序在途source |

fingerprint按排序后的仓库相对路径、换行和当前文件bytes连续计算SHA-256。范围内`gameplay_state.rs`只有一行rustfmt import排序变化，本轮不触碰；因此`source_recheck_required=true`，实施前必须重取该binding、Workbench、operation handler/tests、ZUI、plugin/catalog和动态结果。

### 2.2 证据等级

- E3：Navigation editor/package逐文件，确认11份ZUI的组件、事件、route、payload、controller和生产caller。
- E3：Bake ZUI、panel、factory、command、Runtime operation service/handler纵向追踪，确认selected payload已补齐，但surface/progress producer缺失且两类Bake prepare固定失败。
- E3：runtime focused operation test与当前handler逐行对照，确认“Bake成功”断言和生产实现不一致。
- E3：default `target-editor-host`到first-party catalog、plugin registration、asset toolkit和provider factory闭环，确认插件确实默认装配，不能错误登记为“未链接”。
- E3：overlay event producer、reader-count capture、PIE mirror、provider、shared viewport host及toggle/filter route逐项复核。
- E3：Navmesh AI Workbench的route到binding/navigation/feedback全链，确认只有本地控件状态和固定输出。
- E2：Runtime08D已完成manager/backend/query/crowd/bake hot path深审，本篇只重取Editor依赖的current-source断点。
- E2：Unreal NavigationSystem、RenderingComponent、Bounds/Modifier/TestingActor按Editor/runtime职责抽样；Fyrox NavMesh interaction/selection/commands全链抽样；Godot 2D/3D Region/Link/Obstacle editor按bake/gizmo/UndoRedo抽样。
- E1：Bevy当前checkout没有游戏世界NavMesh Editor；Unity Graphics checkout不拥有Navigation authoring。二者只能说明边界，不可作为降低完成基线的依据。
- 未覆盖：真实Editor启动、ZUI交互、native bake corpus、1M polygon overlay、长时间PIE、跨平台cook和受管Cargo测试，全部进入验收门。

### 2.3 当前生产链事实

1. `target-editor-host`显式启用first-party Navigation runtime/editor，catalog按ProjectPluginManifest投影registration。
2. Navigation editor declaration为Beta，发布`navigation_authoring`和`navigation_gizmos`两项Editor capability；runtime capability仍标记Partial。
3. 插件注册四个authoring surface、五个asset/template view、五个component drawer、五条command和一个viewport provider。
4. Runtime实际注册六类Navigation component：Surface、Modifier、Agent、Obstacle、OffMeshLink、OffMeshBridge；Editor漏掉Bridge drawer。
5. NavMesh与NavigationSettings builtin只有presentation；Navigation插件augment toolkit后可打开对应document view。
6. 两个asset view主体均为单个`Space`，没有locator读取、字段模型、preview、save或validation owner。
7. Surface和五个已注册drawer主体也是单个`Space`；Agents/Areas surface含两个`Space`。
8. Debug surface含四个固定checkbox、一个`Space` viewport、状态label和agent table，但没有任何event/binding/controller。
9. Bake surface有三条真实operation route，selected/clear payload使用stable `surface_entity`，无选择时禁用按钮。
10. Bake Scene按钮没有投影同面板的`force_full_rebuild`；selected Bake才投影该值。
11. `NavigationBakePanel`具有选择、单请求、单调progress和typed invocation模型，controller能包backend错误。
12. Panel/controller在production全仓没有构造者；surface rows、diagnostics、progress和status没有数据源。
13. 三条Editor edit operation均有factory，并经Runtime Gateway submit/poll/harvest，捕获before/after用于undo/redo。
14. command在调用线程最多循环16次`poll_operation`并仅`yield_now`；没有真正异步UI ticket、取消或持续progress订阅。
15. Runtime clear/restore operation有snapshot/apply实现；Bake Scene/Surface的prepare/apply明确固定失败。
16. focused runtime test仍期待Bake operation成功并加载asset，当前source事实与测试不相容。
17. Runtime plugin每tick生成`NavigationOverlayFrame`，reader存在时才开启agent debug capture，这是值得保留的gate。
18. mirror拒绝wrong session、stale sequence和stale owner generation；provider在PIE结束后返回空extract。
19. toggle command只有schema和菜单，没有可执行route；debug checkbox也不能改变`NavigationOverlayOptions`。
20. Navmesh AI Workbench的20余条action只切tab/row/control状态或写固定feedback，没有Navigation领域调用。

## 3. 必须保留的基础

- 保留Navigation插件默认装配和manifest/project selection，不得因为其他首方插件缺装配而把Navigation也误判为未链接。
- 保留Recast/Detour bridge、Runtime manager、typed component/settings/query/bake DTO、scene system和reader-count debug capture；Editor不得持有第二个nav world。
- 保留operation factory、before/after snapshot语义、stale frame拒绝和共享viewport provider注册合同，但修复其executor、异步和数据规模。
- 保留selected-surface稳定实体payload；旧handoff的参数缺失子问题已在source中前进，不能倒退为row index或默认entity 0。
- 复用[Editor02](02-document-transaction-save-autosave-recovery-review.md) transaction/save/recovery、[Editor03](03-scene-prefab-selection-mode-gizmo-picking-review.md) selection/mode/gizmo、[Editor04](04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md) asset/reimport/DDC和[Editor09](09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md) job authority。
- Unreal/Godot/Fyrox表明NavMesh可视化、component/volume/link gizmo、bake状态、query testing和可逆编辑属于最低工程职责；插件化不是停在空view的理由。

## 4. 目标架构

| Authority | 持有 | 禁止持有 |
|---|---|---|
| `NavigationAuthoringDocument` | scene/settings/navmesh locator、source revision、selection、dirty/history、validation | native pointer、ZUI control ID |
| `NavigationSettingsAsset` | stable agent/area IDs、bake/query defaults、schema/version/migration | live crowd state |
| `NavigationSceneProjection` | Surface/Modifier/Obstacle/Link/Bridge typed rows、world/document generation | runtime manager mutable state |
| `NavigationBakeSourceSnapshot` | immutable geometry/collision/modifier/profile inputs、content hash、world revision | live scene references |
| `NavigationBakeJob` | JobId、phase/progress/cancel、diagnostics、source/artifact generation | UI-thread busy poll |
| `NavMeshArtifact` | backend/platform/version/settings/source key、tiles/links/bounds/quality metrics | editor draft state |
| `NavigationQueryPreviewSession` | start/end/filter/agent/generation、terminal result、debug steps | fixed sample strings |
| `NavigationDebugSnapshot` | bounded static generation pages、dynamic deltas、age/drop/cost/tile counters | unbounded full-frame clones |

```text
typed scene/settings edit
  -> reversible document transaction + source revision
  -> immutable bake-source snapshot
  -> cancelable background bake + diagnostics
  -> validated staging artifact
  -> atomic asset/scene commit + undo reference
  -> persistent runtime generation
  -> bounded query/debug snapshots
  -> viewport, toolkit and Workbench projections
```

## 5. P0 阻断项

### P0-1：Bake operation生产实现和focused test互相矛盾

两类Bake的`prepare`固定返回错误，而test仍宣称成功；必须恢复唯一pure prepare/apply backend并以current-source动态门验证，禁止靠mock结果掩盖。

### P0-2：11个业务`Space`发布了资产、组件和authoring能力

Surfaces、Agents/Areas、两类asset、五类drawer和Debug viewport没有controller/data provider；未实现能力必须隐藏或disabled，不能继续以空面板表示Beta authoring完成。

### P0-3：Bake面板、异步job、资产事务与产品host断开

Panel/controller只有测试caller；列表、诊断和progress无人填充，命令在UI线程16次busy poll，结果也没有staging/save/reload闭环。

### P0-4：Overlay、filter和query preview没有可执行产品控制面

Provider和PIE frame虽真实，但toggle/filter不可执行、编辑态无snapshot、无query testing，且full-frame复制不具备大地图性能资格。

### P0-5：内置Navmesh AI Workbench用固定数据伪造Rebuild与Query

它与Navigation插件形成双authority；在接入真实document/job/runtime前必须移除固定成功反馈或明确标成不可执行preview fixture。

## 6. P1 核心重构差距

### 6.1 装配、能力与生命周期

### P1-1：Editor capability没有继承runtime Partial和backend精度

Authoring/Gizmos显示前必须消费machine-readable runtime/backend readiness，不能只看字符串capability存在。

### P1-2：registration成功只证明descriptor存在

bootstrap需验证resource、controller、operation executor、provider、runtime event producer和asset toolkit共同ready。

### P1-3：ProjectPluginManifest选择没有产品级Nav asset打开测试

增加默认Editor启动、项目启停、NavMesh/Settings双击和缺plugin降级测试。

### P1-4：plugin disable/unload没有领域级drain证明

撤销capability前取消Bake/query、清mirror/provider、释放reader并拒绝late result。

### P1-5：Editor直接依赖具体runtime crate

共享DTO可保留，但live service、job和capability应走稳定host/runtime interface，避免插件装载形态决定Editor实现。

### P1-6：OffMeshBridge runtime component没有Editor customization

六类runtime component只有五类drawer，bridge lane/capacity authoring在Inspector中缺席。

### P1-7：surface、drawer、asset view没有readiness metadata

view materialization应能报告`NoController`、`NoDataProvider`、`RuntimeUnavailable`和`ReadOnly`，而非打开空白。

### P1-8：operation schema只是字符串

Bake/clear/toggle需要typed payload版本、decoder limits、capability和executor admission合同。

### P1-9：Editor状态不绑定document/world generation

切scene、PIE、runtime replacement或plugin reload时，旧surface selection和结果可能误投新owner。

### P1-10：注册测试停在ID/manifest层

补完整host materialization、真实command route、provider lifecycle和关闭项目时的资源释放测试。

### 6.2 NavMesh与Navigation Settings资产

### P1-11：NavMeshAsset仍是弱自描述DTO

缺source provenance、cook backend/version/platform、content hash、compression、endianness和完整tile payload校验。

### P1-12：NavigationSettings只含agents和areas

全局voxel/tile/partition/query/crowd/streaming/debug defaults没有资产owner。

### P1-13：agent profile以可变字符串作为identity

重命名会破坏Surface/Agent/Link引用；需要stable ID、display name和迁移表。

### P1-14：area以裸`u8`身份且无删除策略

需要reserved/tombstone、引用分析、mask迁移、颜色和cost/filter版本。

### P1-15：两类asset toolkit完全空白

实现typed field model、diff、validation、references、preview、save/reload和migration diagnostics。

### P1-16：NavMesh toolkit没有tile/topology/area/link浏览器

必须支持bounds、poly/tile count、build time、memory、bad edge/island/clearance和source回跳。

### P1-17：Settings toolkit没有agent/area表事务

新增、复制、重排、删除、rename、mask/cost编辑都必须可撤销并实时显示破坏性引用。

### P1-18：`output_asset`仍是任意字符串

改为typed locator/reservation，验证scheme、kind、project ownership、冲突和写权限。

### P1-19：Bake结果没有staging artifact和原子commit

失败、取消、崩溃或scene切换不能覆盖最后一个有效NavMesh；commit后才改变document dirty/history。

### P1-20：Editor没有DDC/reimport/invalidation视图

source/settings/backend key、cache hit/miss、依赖变更和cook artifact复用应进入统一Asset/Job表面。

### 6.3 Scene component、Inspector与Gizmo

### P1-21：NavMeshSurface所有字段不可编辑

agent、collect mode、geometry source、layers、volume、voxel/tile override、region和output均缺typed drawer。

### P1-22：Surface Volume没有viewport handle

需要bounds gizmo、local/world transform、snap、multi-selection和transaction，而不是手输数组。

### P1-23：Modifier没有shape/影响范围可视化

area替换、agent mask、children和link generation必须在scene中预览实际受影响tiles。

### P1-24：Agent的movement/query字段没有统一Inspector

radius/height/speed/avoidance/priority/mask/repath/link/writeback/destination需共享runtime validator和unsupported提示。

### P1-25：Obstacle没有Box/Capsule handles与carve反馈

移动阈值、stationary timer、avoidance/carve互斥和dirty tile影响应有可视、可撤销编辑。

### P1-26：OffMeshLink没有endpoint pick/snap/方向/arc gizmo

start/end entity、local point、width、area、cost、traversal mode和motion都无法工程化编辑。

### P1-27：OffMeshBridge lane/capacity authoring完全缺失

Editor必须显示lane生成、shared capacity group、方向与冲突诊断，不能依赖JSON手工输入。

### P1-28：DesiredVelocity/debug resource没有只读Inspector边界

运行期输出和作者输入必须分层，避免用户把瞬态feedback当可保存字段。

### P1-29：surface list没有scene projection owner

需要按当前document增量投影stable entity、label、agent、output、dirty/status，并随创建/删除/undo更新。

### P1-30：component edit没有共享semantic validation

Inspector、serialization、bake和runtime必须共用finite/range/reference/capability规则并返回字段级诊断。

### 6.4 Bake、Job、Undo与Artifact

### P1-31：Bake Scene/Surface缺少pure prepare实现

prepare必须只消费immutable snapshot并产出staging artifact；apply在owner thread验证generation后原子发布。

### P1-32：Editor command用16次`yield_now`同步轮询

改为JobId驱动的异步transaction continuation，UI线程不得赌operation在固定poll预算内完成。

### P1-33：没有cancel、pause、priority或shutdown fence

大场景Bake必须进入Editor09统一scheduler并定义取消延迟、资源预算和project close语义。

### P1-34：`NavigationBakePanelController`是孤立的第二套提交模型

选择保留单一UI session并适配统一operation/job，删除parallel backend abstraction或赋予明确测试fixture身份。

### P1-35：Bake surface table没有producer

表格需来自`NavigationSceneProjection`，不能由测试手工`ReplaceSurfaceRows`构造完成证明。

### P1-36：Diagnostics/progress/status没有terminal source

展示phase、tile、geometry、warning/error、elapsed、memory、cache和可定位entity，并处理late/out-of-order更新。

### P1-37：Bake Scene忽略面板的Force Full Rebuild

scene与selected操作必须使用同一typed settings snapshot；不能让同一checkbox只影响一条route。

### P1-38：Undo保存完整before/after NavMesh snapshot

大资产会双倍占用history；应保存content-addressed artifact refs、generation和可回滚commit metadata。

### P1-39：Clear没有资产文件/scene引用事务

清runtime snapshot不等于删除或解绑NavMesh asset；明确clear generated data、unlink asset和delete asset三种命令。

### P1-40：Bake缺document/source revision冲突处理

Bake期间修改geometry/settings、删除surface、切scene或undo时必须cancel、rebase或拒绝commit，不能静默覆盖。

### 6.5 Query、测试与预览工作流

### P1-41：Editor没有真实path query operation

Workbench的Query Path只写文本；需要start/end/filter/agent/generation和typed terminal result。

### P1-42：没有viewport query start/end handles

参考Unreal NavigationTestingActor，支持scene pick、拖动、自动重算和结果生命周期。

### P1-43：sample/raycast/distance-to-wall/filter/cost查询不可视

Editor应覆盖Runtime公开query surface，而不是只提供一条假path按钮。

### P1-44：agent profile和area下拉是固定列表

Humanoid/Crawler/Vehicle/Debug和Walkable/Jump/Door/Water必须来自Settings asset及backend capability。

### P1-45：没有tile/area/cost/build-time热图

Debug view应按generation和viewport选择显示真实tile状态、dirty原因与质量指标。

### P1-46：没有NavMesh topology修复模式

至少提供只读检查与source定位；若支持手工修补，应像Fyrox一样使用稳定selection和可逆vertex/edge/triangle command。

### P1-47：没有multi-agent profile差异预览

同一source对不同radius/height/slope/climb的可达性差异必须可并排或叠加检查。

### P1-48：失败路径没有搜索诊断

partial/no-path/out-of-nodes/filter rejection/off-mesh block需显示visited nodes、cost和最近失败边界。

### P1-49：preview不绑定world/document/backend generation

旧query不能在新Bake或runtime replacement后继续显示为当前结果。

### P1-50：缺少离线导航质量分析

提供island、unreachable target、clearance、stair/slope、link、area coverage和golden path批量验证报告。

### 6.6 Overlay、PIE、Workbench与测试真相

### P1-51：Toggle Navigation Gizmos没有executor

把command接到shared viewport overlay view state，支持per-viewport enable、capability disable和持久化。

### P1-52：四个debug filter checkbox没有事件

NavMesh areas、links、agent paths、avoidance/desired vector必须改变provider options，并能反映实际状态。

### P1-53：provider只消费PIE mirror

编辑态需从当前document/runtime generation获得静态NavMesh snapshot；未播放时不应永远空白。

### P1-54：overlay options固定为Default

增加per-view state、agent/area/tile筛选、opacity/depth/picking和profile，避免全局隐式设置。

### P1-55：runtime每tick构造并镜像完整overlay frame

改为static generation pages加bounded dynamic delta，稳定NavMesh不得每帧复制所有triangle/link。

### P1-56：provider用selected entity或0作为overlay owner

owner必须是明确NavWorld/NavMesh generation，不得把当前scene selection混作数据ownership。

### P1-57：overlay没有frustum/tile/LOD预算

大地图需viewport culling、tile residency、command/vertex上限、drop policy和CPU/GPU指标。

### P1-58：mirror没有age/drop/backpressure telemetry

sequence检查是基础，还需queue age、dropped frames、payload bytes、consumer lag和shutdown统计。

### P1-59：Navmesh AI Workbench形成第二套静态authority

将其改成真实Navigation document的projection，或删除该模块并复用插件toolkit；禁止长期双轨。

### P1-60：测试用mock成功掩盖current-source产品失败

97个静态test attribute中缺真实host点击到runtime Bake/asset commit/overlay packet闭环，且operation test与handler已冲突；必须以动态产品门替代source-shape绿色。

## 7. P2 能力差距

### P2-1：缺少World Partition、Navigation Invoker与tile streaming authoring

需要区域、优先级、residency和生成范围可视化，与Runtime08D large-world owner一致。

### P2-2：缺少hierarchical path和多层图调试

显示cluster/portal、粗细路径切换、fallback原因和成本误差。

### P2-3：缺少Smart Link gameplay authoring

支持交互条件、reservation、animation/root motion/network authority和terminal outcome绑定。

### P2-4：缺少Query Filter资产与语义cost field编辑器

area mask、fixed/enter/travel cost、动态标签和profile继承需要独立versioned资产。

### P2-5：缺少Crowd密度、速度、deadlock和LOD分析

用真实规模snapshot/heatmap替代固定agent count。

### P2-6：缺少协作编辑、merge与Bake artifact provenance

多人修改Settings/Surface/Link时需stable IDs、冲突语义和可审计derived artifact。

### P2-7：缺少多PIE/world/session并行观察

每个viewport/session必须绑定独立world generation，禁止单mirror覆盖。

### P2-8：缺少server determinism、record/replay和网络路径差异工具

能够重放query/agent/link决策并比较client/server generation和结果。

### P2-9：缺少2D Navigation authoring

Godot证明2D region/polygon/link/obstacle需要独立交互与Bake工作流，不能复用3D空壳。

### P2-10：缺少车辆、飞行、游泳、攀爬等多locomotion domain

Editor需按backend/domain路由profile、source、query和segment preview。

### P2-11：缺少Navigation质量回归数据库

保存场景、参数、golden result、容差、性能与截图，支持跨版本bisect。

### P2-12：缺少同质量参考引擎性能对照

冻结geometry、Recast参数、agent/query规模和质量容差后，才允许声明优于Unreal/Fyrox/Godot。

## 8. 参考实现差异矩阵

| 参考 | 可验证基线 | Zircon当前差距 |
|---|---|---|
| Unreal NavigationSystem | Editor provider、viewport show flag、Bounds/Modifier volume、NavMeshRendering多debug detail、NavigationTestingActor的agent/filter/path/cost/step状态、dirty/build lifecycle | toggle/filter/query testing、volume/link gizmo、tile/build/cost诊断、编辑态render和产品Bake未闭环 |
| Fyrox | 独立NavMesh interaction mode、vertex/edge selection、connect/add/move/delete command、command group与debug drawing | Zircon没有topology interaction、selection和可逆修复模式；drawer为空 |
| Godot Navigation 2D/3D Editor | Region Bake/clear、多选Bake状态、Link/Obstacle handles、UndoRedo和source geometry lifecycle | Zircon只有route/DTO，没有真实面板状态、gizmo、job/transaction和2D workflow |
| Bevy | 当前checkout无游戏世界NavMesh Editor；可参考ECS/plugin分层 | 不作为Navigation authoring完成基线 |
| Unity Graphics | 当前checkout只覆盖Graphics，不拥有Navigation authoring | 不比较，不从Graphics源码推断Unity Navigation能力 |

## 9. 必须硬切的旧实现

1. 删除或改造固定Navmesh AI Workbench；不得与Navigation插件长期并存为两个事实源。
2. 收敛`NavigationBakePanelController`和operation/job链，只保留一个产品submission/session authority。
3. 修复Bake handler/test矛盾；禁止test gateway伪造当前production不可能返回的成功结果。
4. 所有业务`Space`在controller/data provider落地前隐藏或显示明确Unavailable，不得保留可误认成功的空view。
5. `output_asset`、agent profile和area/link引用迁移到typed/stable identity；禁止新增字符串猜测和entity 0 fallback。
6. full-frame overlay迁移到generation page/delta；旧unbounded payload不得成为兼容路径。
7. 清理已前进handoff的陈旧事实，同时保留未通过managed gate的open状态；不得把历史记录改写成整体完成。

## 10. 分层里程碑

### M0：能力真相、失败记录与唯一authority

- 建立current-source product capability matrix，标记Bake/asset/drawer/query/toggle真实状态。
- 修复Bake operation实现/test冲突，选择唯一Bake session/job authority。
- 对selected-surface、overlay publication和V2 status handoff逐项重验，分别关闭或保留，不批量false-green。

退出门：默认Editor不能显示无executor命令；真实Gateway Bake有terminal结果；所有文档状态与current source一致。

### M1：Navigation Document、Settings与NavMesh toolkit

- 建立transactional document、stable agent/area IDs、typed locator和shared validation。
- 实现Settings/NavMesh toolkit、references、diff、migration、save/reload和artifact inspector。

退出门：创建/打开/编辑/保存/重载/undo/crash recovery不丢数据，破坏性rename/delete有引用诊断。

### M2：Scene projection、Inspector与Gizmo

- 实现六类component drawer、surface rows和typed property edit。
- 实现Surface/Modifier/Obstacle/Link/Bridge handles、selection/picking和multi-edit transaction。

退出门：scene增删改/undo实时更新面板；非法字段不能到达Bake/runtime；gizmo与Inspector共享同一command。

### M3：真实Geometry、Cancelable Bake与Prepare/Apply

- 复用Runtime08D的真实source geometry和backend convergence工作。
- immutable snapshot在worker prepare，owner-thread generation check和atomic apply；接Editor09 scheduler。

退出门：cancel/fail/stale/shutdown均不污染最后有效artifact，UI线程无busy poll，progress/diagnostic来自真实job。

### M4：Artifact、Commit、Undo与DDC

- 建立content-addressed staging/final artifact、source/settings/backend key和原子asset commit。
- history保存artifact ref而非双份大snapshot，支持clear/unlink/delete三种语义。

退出门：save/reload/reimport/cache/undo/redo/scene reopen维持相同generation和可追溯provenance。

### M5：Query Preview与Navigation Testing

- 实现start/end gizmo、agent/filter/area选择、path/sample/raycast/distance/cost和失败步骤。
- 实现multi-profile comparison、tile/area/cost heatmap和offline quality validator。

退出门：每个结果绑定document/world/artifact generation；partial/no-path/out-of-nodes均可解释且不显示stale。

### M6：Bounded Overlay与PIE Debug

- 接通per-viewport toggle/filter/state，补编辑态与PIE双source。
- static generation pages、dynamic agent deltas、culling/LOD/budget/telemetry和shutdown cleanup。

退出门：hidden/stable不重建，1M triangle有明确预算；PIE结束、plugin disable和scene切换下一帧清空旧数据。

### M7：Workbench收敛与产品工作流

- 删除固定样例或改成真实document/job/runtime projection。
- 统一Asset Browser、Inspector、World menu、Job Center、Output/Notification和viewport导航。

退出门：Rebuild/Query/Edit/Commit每条可见action都有executor、acknowledgement和terminal failure；无固定成功计数。

### M8：Large World、质量与性能资格

- 加入partition/invoker/hierarchical/smart link/multi-domain/record-replay等P2能力。
- 建立Windows优先的correctness、fault、soak、profile和同质量参考引擎对照。

退出门：完整产品矩阵与规模证据通过后，才评估从Beta/Partial晋级或性能优于Unreal。

## 11. 验收门

### 11.1 产品与事务门

1. G1：默认`target-editor-host`按项目manifest装配runtime/editor/provider/resources，禁用插件后所有入口立即降级。
2. G2：NavMesh和Settings从Asset Browser创建/打开/编辑/save/reload/reimport/recover全链通过。
3. G3：六类component drawer均读取真实selection并提交可撤销typed edit。
4. G4：surface table在spawn/delete/rename/undo/scene切换后无stale entity或row-index identity。
5. G5：Bake Scene、Selected、Clear、cancel、undo、redo均经真实Gateway和Runtime handler完成。
6. G6：Bake期间修改source/settings、删除surface、切scene、关闭项目时结果被正确cancel或拒绝。
7. G7：staging artifact只在validation成功后原子commit，崩溃不覆盖last-known-good。
8. G8：所有可见command/view都有executor/readiness；空`Space`和固定成功feedback计数为0。

### 11.2 正确性与故障门

9. G9：agent/area stable ID rename/delete/reorder和mask/reference迁移有golden test。
10. G10：stairs、slope、clearance、low ceiling、multi-floor、terrain、concave和modifier corpus通过。
11. G11：render mesh/collider source、layer/volume/hierarchy collect和empty source行为可解释。
12. G12：Link/Bridge方向、lane、capacity、manual/automatic、disabled和endpoint失效均有Editor预览与runtime一致结果。
13. G13：path/sample/raycast/distance/filter/cost的success/partial/no-path/out-of-nodes与Runtime一致。
14. G14：corrupt version/hash/tile/index/area/link/NaN、错误kind和跨平台artifact被拒绝且保留旧资产。
15. G15：worker panic、native failure、OOM预算、queue reject、cancel race和late apply均返回typed diagnostic。
16. G16：Undo历史、autosave recovery和artifact GC在1k Bake/clear/restore循环后无泄漏或错误引用。

### 11.3 Overlay、PIE与交互门

17. G17：per-viewport toggle和四类filter通过真实UI事件改变provider output并可恢复。
18. G18：编辑态静态NavMesh、PIE agent/path/vector和query preview可独立或组合显示。
19. G19：world/document/owner generation变化后旧overlay/query在下一帧不可见。
20. G20：plugin disable、PIE end、runtime crash/restart和viewport close释放reader、mirror和provider state。
21. G21：Surface/Modifier/Obstacle/Link/Bridge handles支持pick、drag、snap、cancel、multi-select和UndoRedo。
22. G22：tile/area/cost/build-time/dirty heatmap显示真实数据，点击能回跳source entity/asset/diagnostic。
23. G23：Navmesh AI Workbench所有action连接真实domain，或模块从生产构建删除。
24. G24：keyboard/focus/accessibility、high-DPI、多窗口和layout restore不破坏Navigation workflow。

### 11.4 性能、平台与证据门

25. G25：1/1k/100k source mesh与1/1k/100k tile报告snapshot、queue、build、commit、peak RSS和cancel p99。
26. G26：1/100/10k query报告wait、node visits、alloc、p50/p95/p99、partial/out-of-nodes，steady-state不重建native navmesh。
27. G27：1/256/1k/10k agent报告mirror bytes、debug capture cost、drop/age和Editor frame impact。
28. G28：hidden/stable/dirty、1k/1M triangle overlay报告rebuild count、clone bytes、culling、commands和extract p95。
29. G29：Windows MSVC受管build/test先通过，再验证Linux clang/GCC、macOS和目标artifact ABI/endianness。
30. G30：C++ bridge覆盖ASan/UBSan/fault injection，8小时Bake/query/PIE/overlay soak无增长队列或RSS drift。
31. G31：每个里程碑保存source fingerprint、命令、原始terminal结果、trace/capture、失败记录和独立review。
32. G32：与Unreal/Fyrox/Godot对照冻结build、geometry、Recast参数、agent/query规模和质量容差；能力或质量不等时禁止标注“更快”。

## 12. 既有计划与handoff处理

`docs/plans/zircon_plugins/05-navigation.md`仍是Navigation插件实现owner，[Runtime08D](../zircon_runtime/08d-navigation-runtime-review.md)拥有runtime authority收敛和性能门。本篇不复制其manager/backend算法任务，而是把Editor产品面和两者之间的prepare/apply、artifact、generation合同写成独立验收项。

2026-07-13 M6记录本身已写明三个共享产品wiring未返回、M6未验收，因此保留为历史产出证据，不把文件存在改写成产品完成。current source对旧handoff的状态如下：

- selected-surface参数：ZUI和retained model现已投影stable `surface_entity`，原始“arguments为空”事实已前进；但surface rows生产owner和managed产品门仍缺，不能直接关闭整体M6。
- overlay publication：typed frame、mirror和provider已落地，原始“没有producer/provider”事实已前进；toggle/filter、编辑态source、bounded payload和managed gate仍缺，failure记录仍不能宣告全部fixed。
- operation status V2：production command已消费V2 progress，但必须重跑全包source audit和focused managed test后再关闭旧handoff。
- fallback hotpath与world-scan JSON：继续由Runtime08D收敛；Editor不得通过另建preview navmesh绕过。
- 新增current-source blocker：Bake handler固定失败而test期待成功，必须在M0创建独立failure/fix记录或并入现有Navigation operation owner，不能藏在UI修复中。

## 13. 完成定义

Navigation Editor只有在以下条件同时成立后，才能从“默认链接的Beta骨架”提升为工程级authoring：

1. 默认产品、capability、view、command、provider、runtime backend和asset toolkit的readiness一致，不再发布空能力。
2. Settings/NavMesh和六类scene component均有transactional document/Inspector/gizmo/save/recovery工作流。
3. Bake使用真实immutable source、cancelable background prepare、generation-safe apply、staging artifact和atomic commit；生产实现与tests一致。
4. Undo/Redo保存artifact reference与commit metadata，不复制无界NavMesh；clear/unlink/delete语义分离。
5. Query/testing覆盖Runtime公开能力并绑定world/document/artifact generation；失败结果可解释。
6. Overlay toggle/filter可执行，编辑态与PIE共用唯一snapshot authority，稳定静态数据不逐帧full copy。
7. 固定Navmesh AI Workbench被真实数据链替换或删除，所有Rebuild/Query/Commit都有terminal acknowledgement。
8. correctness、fault、platform、soak和规模性能门有新鲜证据；任何“优于Unreal”结论均有同质量可复现对照。

在这些门完成前，本篇保持`implementation_status: pending`。

## 14. 本轮验证状态

- Production code：未修改。
- Dynamic validation：未执行；既有Editor test-build阻断未变化，同一Cargo lane未重复。
- Static validation：完成；5/60/12/32编号连续，49个frontmatter路径与74个报告/索引相对链接存在，228文件fingerprint复核一致，`git diff --check`无whitespace错误（仅既有CRLF转换提示）。
- Implementation：pending，按M0-M8推进。
