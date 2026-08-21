---
related_code:
  - zircon_plugins/physics/editor
  - zircon_plugins/physics/runtime
  - zircon_plugins/physics/plugin.toml
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/scene
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/importer/ingest/import_physics_material.rs
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/simulation_physics.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/simulation_physics.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/simulation_physics.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/PhysicsAssetEditor
  - dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor
  - dev/UnrealEngine/Engine/Plugins/Experimental/ChaosEditor
  - dev/Fyrox/editor/src
  - dev/godot/editor/scene/3d/gizmos/physics
  - dev/godot/editor/scene/3d/physics
  - dev/godot/editor/scene/2d/physics
  - dev/bevy
  - dev/Graphics
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 18 · Physics Material、Rigid Body/Collider/Joint、Collision Cook、Ragdoll 与 Debug Authoring 工程化差距

## 1. 结论

Zircon Physics不是空实现。Runtime已有真实`RuntimeSceneSystem`、scene world sync、Jolt与builtin backend、body/collider/joint/material/query合同、fixed-step接点和Ragdoll profile/runtime helper；Editor也有能把`PhysicsWorldSyncState`转换为collider/trigger primitive的overlay builder，以及从骨架生成初始Ragdoll profile的纯函数。这些底座必须保留。

但Physics Editor产品仍不完整，五个P0断点是：

1. builtin registry只给Physics Material显示信息，不给toolkit；default linked first-party editor catalog不含Physics。默认产品不能编辑Physics Material，也不能保证装配Physics/Ragdoll authoring。
2. Physics插件四份ZUI全部存在，却以11个`Space`承载authoring、debug、diagnostics和ragdoll全部业务，0个event、0个controller/data provider；可见capability与实际功能不一致。
3. “Generate Ragdoll Profile From Skeleton”命令只发送`OpenView`，创建模板也复用该命令；真实generator只有测试调用。Runtime没有把Ragdoll profile纳入AssetKind/import/artifact/product spawn链，空Ragdoll view不能创建或保存资产。
4. “Toggle Physics Overlay”同样只打开一个空view，没有`ViewportOverlayProviderRegistration`；真实overlay builder无生产caller。2026-08-01的open failure handoff经当前源码复核仍成立。
5. Core Collision Proxy与Physics Collision两个Workbench各有19个route，但只切换静态控件和写固定反馈；18 proxies、124 bodies、32 contacts、82 kg、4 manifolds、CCD warning等均非job/runtime事实。仓内仍没有production physics mesh/heightfield/convex cook注册，Bake/Simulate/Validate属于能力伪装。

本报告记录5个P0、60个P1、12个P2，给出M0-M8与32个验收门。目标是建立`PhysicsAuthoringDocument + CollisionProfile + PhysicsMaterialAsset + PhysicsCookArtifact + Reversible Component/Gizmo Edit + RagdollAsset + PreviewWorld + Generation-bound Debug Snapshot`。本轮仅静态review，不修改production代码。[Runtime Physics 08A](../zircon_runtime/08a-physics-runtime-review.md)继续拥有solver/query/event/native constraint/scene sync运行时差距；本报告聚焦Editor authoring和产品证据。

上一轮同一工作树的`zircon_editor --lib`测试编译在617.2秒后被239个既有test-build错误和122个warning阻断，本轮不重复同一Cargo lane；124个test attributes只是静态inventory。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Physics editor/package | 13 / 722 / 25,310 | E3逐文件：四份ZUI、registration、overlay/generator及4个test attributes；fingerprint `9a080c1b...364ab6` |
| Physics runtime生产实现，不含tests | 66 / 8,516 / 299,179 | E2交叉复核08A，E3复核Editor所需scene/ragdoll/debug/cook接点；fingerprint `b2d6c8f3...827b3a` |
| Collision Proxy/Physics Collision Workbench链 | 7 / 2,180 / 123,251 | E3读取38 routes、binding/navigation/feedback与固定数据；fingerprint `a04eba99...f4f9f` |
| Core physics与scene physics合同 | 41 / 988 / 28,436 | E2完整inventory，E3复核material/filter/shape/joint字段；fingerprint `61c02c06...5f194` |
| Asset/scene physics纵向接点 | 49 / 14,397 / 563,334 | E2 inventory、E3 PhysicsMaterial/RigidBody/Collider/Joint关键链；fingerprint `366c59b6...20e2e` |
| Editor共享catalog/toolkit/operation/viewport extension接点 | 6 / 1,928 / 72,881 | E3默认装配、No-toolkit、operation与overlay provider合同；fingerprint `b885681a...1c737` |
| selected combined scope | 182 / 28,731 / 1,112,391 | 当前工作树去重fingerprint `49c08d6d...84cfb`；124个test attributes、0 ignored、1个import排序在途source |

fingerprint计算规则同Editor17。范围内`simulation_physics.rs`仅有一行rustfmt import排序变化，本轮不触碰；因此`source_recheck_required=true`，实施前必须重取该binding、两份Workbench、plugin/catalog、overlay/provider和动态结果。

### 2.2 证据等级

- E3：Physics editor/package逐文件，确认四份资源存在、11个业务`Space`、0 event及两条伪执行命令。
- E3：overlay builder/generator到全仓production caller搜索，确认只有re-export，没有产品owner。
- E3：default linked catalog、Physics Material builtin type、Ragdoll plugin type及`OpenAsset` No-toolkit路径闭环。
- E3：两个Workbench 38条route到binding、navigation、feedback逐项闭环，确认没有document/job/runtime acknowledgement。
- E3：open overlay failure handoff按当前`ViewportOverlayProviderRegistration`合同重新验证。
- E2：Runtime Physics沿08A复核真实scene system/Jolt基础与cook、filter、query、native constraint限制；不重复完整算法审查。
- E2：Unreal PhysicsAssetEditor 70个文件、StaticMeshEditor 28个文件、ChaosEditor 83个文件按资产、碰撞生成、body/constraint、仿真、事务职责抽样。
- E2：Godot Editor Physics 32个文件按shape/joint/physical-bone gizmo与UndoRedo对照。
- E1：Fyrox无专用Physics Editor模块，Bevy无内建physics editor，本地Unity Graphics无Physics authoring；不能作为降低基线的依据。
- 未覆盖：真实Jolt可视化、mesh corpus、convex decomposition、100k bodies、native contact/constraint、跨平台cook与物理正确性动态测试，全部进入验收门。

### 2.3 当前生产链事实

1. Physics runtime插件和scene system真实存在，不能把Editor缺陷误写成Runtime完全无物理。
2. Physics editor注册authoring、diagnostics、debug overlay、ragdoll profile四个view/surface及一个drawer。
3. 四份ZUI分别有2、2、3、4个`Space`，没有任何event。
4. overlay builder能按sensor区分Collider/Trigger并复制canonical shape/transform，这是正确的纯projection基础。
5. debug toggle command却只`OpenView(physics.debug_overlay)`，没有toggle viewport provider。
6. Physics extension registry已有正式provider注册/生命周期合同，Physics插件没有使用它。
7. Ragdoll generator能从骨骼局部translation估算capsule并validate，测试覆盖三骨链。
8. create operation只`OpenView(physics.ragdoll_profile)`；creation template没有skeleton locator、output locator或写入factory。
9. `RagdollProfile::from_toml`、`spawn_configured`和Editor generator没有production caller；asset pipeline也没有Ragdoll kind/import/artifact。
10. Physics Material有TOML asset/import/artifact/load链，字段为static/dynamic friction、restitution和两类combine rule。
11. Metadata derive Default使三项scalar为0；Runtime08A已确认native material映射和默认语义不完整。
12. builtin asset registry识别Physics Material但不给toolkit；Physics editor也没有贡献该类型。
13. scene edit projection只对RigidBody做一个名称映射；Collider/Joint没有专用typed inspector/gizmo路径。
14. Runtime component/schema/editor验证并未收敛，复杂shape union、mass/inertia、CCD/sleep、joint frames/limits/drives无法完整编辑。
15. Collision Proxy Workbench硬编码`SM_RockCliff 18K tris`、42%、6 hulls、18 proxies、9 channels和2 invalid。
16. Physics Collision Workbench硬编码Player Capsule、Ice Material、4 manifolds、124 bodies、32 contacts及CCD warning。
17. feedback对Bake/Test/Simulate/Validate只写“queued”和固定计数，没有JobId、source revision、backend generation或terminal result。
18. Jolt `register_mesh_asset`的生产caller为零，TriangleMesh/HeightField/Convex没有共享cook artifact链。
19. default first-party editor catalog仍只有Navigation/Neural；外部动态发现可能装载Physics，但不构成默认发行保证。
20. 既有Physics overlay failure记录仍为open，Runtime08A也明确要求重开过去的false-green里程碑。

## 3. 必须保留的基础

- 保留RuntimeSceneSystem、world sync、typed shape/body/joint/material/query合同和Jolt backend边界，Editor不得另造平行physics world authority。
- 保留PhysicsMaterial TOML/artifact/load链，演进schema/default/validation/toolkit，而非新建第二种材料资产。
- 保留overlay builder与ragdoll generator纯函数，但把它们接入正式provider、document、job和asset lifecycle。
- 复用[Editor02](02-document-transaction-save-autosave-recovery-review.md) transaction/save/recovery、[Editor03](03-scene-prefab-selection-mode-gizmo-picking-review.md) mode/gizmo/overlay、[Editor04](04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md) import/cook和[Editor09](09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md) job authority。
- Unreal/Godot证明body/shape/constraint选择、可逆gizmo、仿真preview和碰撞生成是最低工程职责；Rust或插件边界不是停在空ZUI的理由。

## 4. 目标架构

| Authority | 持有 | 禁止持有 |
|---|---|---|
| `PhysicsAuthoringDocument` | scene/asset revision、stable element IDs、selection、dirty/history、validation | native handle、ZUI ID |
| `CollisionProfileAsset` | object/trace channels、response matrix、solver/query groups、version | live contact rows |
| `PhysicsMaterialAsset` | validated friction/restitution/combine/surface type、provenance | backend pointer |
| `PhysicsCookSource` | mesh/terrain source、decomposition/weld/LOD/material settings | runtime shape handle |
| `PhysicsCookArtifact` | backend/platform/version key、validated convex/mesh/heightfield blobs、diagnostics | mutable editor draft |
| `RagdollAsset` | skeleton binding、stable bone IDs、bodies/shapes/joints/filter/material/profile | live spawned entities |
| `PhysicsPreviewWorld` | document/artifact/backend generations、transport、isolated bodies、reset fence | project authoritative world |
| `PhysicsDebugSnapshot` | counters、bounded geometry/events、filter/generation/timestamp | fixed sample strings |

```text
typed scene/asset edit
  -> reversible transaction + source revision
  -> shared schema validation
  -> collision profile/material/cook/ragdoll artifact
  -> generation-bound preview or runtime scene bridge
  -> backend acknowledgement + bounded debug snapshot
  -> viewport/diagnostics projection
```

## 5. P0 阻断项

### P0-1：Physics Material和Physics插件都没有默认产品装配闭环

Physics Material无toolkit，default linked catalog无Physics；必须建立默认assembly/readiness、材料toolkit和安全fallback。

### P0-2：四份Physics surface以11个空节点发布完整能力

Authoring/Diagnostics/Debug/Ragdoll均无event/controller/provider；未实现能力必须隐藏或disabled，不能以空面板交付。

### P0-3：Ragdoll创建命令只开空view，真实generator与runtime资产链断开

必须建立typed RagdollAsset、skeleton input、transactional generator、import/save/artifact/spawn/despawn，而非OpenView事件。

### P0-4：Physics Debug toggle没有可执行viewport provider

open failure仍成立；必须注册Physics-owned provider、路由`ToggleOverlayProvider`并证明disable/unload清除stale geometry。

### P0-5：Collision Proxy/Physics Collision Workbench用固定数据伪造Bake、Simulate与Validate

38条route没有cook job、preview world或runtime acknowledgement；完成真实链前必须移除queued/计数成功反馈。

## 6. P1 核心重构差距

### 6.1 装配、资产与能力真相

### P1-1：default catalog与runtime Physics能力不对称

manifest启用Physics时必须成套验证editor registration、resources、toolkits、provider和runtime manager。

### P1-2：resource materialization只验证ID，不验证controller/readiness

bootstrap应拒绝发布依赖缺失的surface，并给typed diagnostic。

### P1-3：Physics Material没有专用toolkit

需要编辑、diff、validate、save/reload、reference和preview接点。

### P1-4：Ragdoll plugin asset type只在插件加载后存在

默认assembly、unknown-plugin placeholder与资源迁移策略必须明确。

### P1-5：Physics authoring drawer与view职责重叠

材料、scene component、cook、ragdoll、debug/diagnostics应有独立owner/lifecycle。

### P1-6：capability不描述Jolt/builtin精度和unsupported组合

Editor必须消费Runtime08A要求的machine-readable backend capability report。

### P1-7：operation readiness不包含provider/job/backend

显示命令前检查executor、target、write authority和backend feature。

### P1-8：schema字符串没有typed payload/version/budget

所有create/bake/simulate/validate/edit操作要有decoder、迁移和上限。

### P1-9：plugin disable没有preview/provider/job drain

撤销capability前停止preview、取消cook、清overlay并拒绝late commit。

### P1-10：注册测试只断言ID和view存在

增加默认bootstrap、物理资源、controller/provider、asset open和shutdown产品测试。

### 6.2 Scene component、Inspector与Gizmo

### P1-11：RigidBody只有通用字段投影

body type、mass/inertia、gravity/damping、CCD、sleep、locks必须typed编辑并共享validator。

### P1-12：Collider shape union没有完整Inspector

Box/Sphere/Capsule/Cylinder/Convex/Mesh/HeightField/Compound需按variant投影及验证。

### P1-13：Joint没有frames/limits/drives/break编辑器

六类joint应按backend support显示合法字段并提供对象引用picker。

### P1-14：多选编辑会复制不相容physics字段

必须按component presence/shape variant处理mixed values与per-target validation。

### P1-15：component validator在World/reflection/editor/runtime分裂

统一schema owner，非法值原子拒绝且保留旧值。

### P1-16：shape没有可逆viewport handles

尺寸、offset、rotation、joint frame/limit需drag transaction、cancel和undo。

### P1-17：local/world transform与scale policy不清

gizmo、cook、Jolt shape和debug geometry必须共享坐标/缩放规则。

### P1-18：Collider/Joint selection与hit proxy缺失

支持多shape/subshape选择、frame、duplicate/delete和稳定element ID。

### P1-19：mass/inertia preview缺失

显示COM、principal axes、density来源和backend-calculated结果及诊断。

### P1-20：Play/simulate期间可编辑性未定义

区分authoring、simulation-only和可安全live-edit字段，并绑定generation。

### 6.3 Collision profile、query与响应

### P1-21：没有versioned CollisionProfileAsset

channel、object/trace类别、response matrix、defaults与migration需单一authority。

### P1-22：Jolt native filtering未消费authoring matrix

Editor必须预览并验证编译后的backend filter generation，而非仅显示字符串。

### P1-23：Workbench channel mask只是固定WorldStatic/Player

改为真实profile matrix、搜索、批量编辑、diff与undo。

### P1-24：query/solver响应语义未区分

Block/Overlap/Ignore、QueryOnly/PhysicsOnly/Sensor及trace channels需明确。

### P1-25：layer容量与backend映射没有cook-time诊断

超限、冲突或降级必须阻止export或给明确fallback。

### P1-26：profile reload没有world rebuild/remap策略

运行中变更需generation、barrier和LKG，不可只换Editor表格。

### P1-27：contact row缺stable pair/subshape/material identity

diagnostics选择应定位场景实体、shape、face/material及runtime event generation。

### P1-28：query debug没有真实ray/sweep/overlap结果

需展示filter、backend、approximate/unsupported状态、hit排序与成本。

### P1-29：sensor/trigger authoring没有Enter/Stay/Exit预览

preview必须使用backend事实和bounded event stream，而非重算overlap。

### P1-30：collision profile没有reference impact分析

重命名/删除channel必须列出scene、asset、script引用并transactionally迁移。

### 6.4 Collision cook、proxy与Physics Material

### P1-31：`register_mesh_asset`无production caller

建立共享PhysicsCookArtifact加载/注册/shape cache链。

### P1-32：Collision Proxy Bake没有job executor

Bake必须携source hash/settings/backend/platform、进度、cancel和terminal artifact。

### P1-33：Convex decomposition参数只有静态42%/6 hulls

提供算法/version、max hull/verts/error、deterministic seed和preview diff。

### P1-34：triangle mesh缺weld/degenerate/material slot验证

坏数据在cook阶段失败，不能到fixed tick才报native错误。

### P1-35：heightfield缺terrain revision与tile cook

按区域增量、hole/material、scale和runtime residency生成artifact。

### P1-36：compound collider缺child stable ID和局部变换编辑

支持add/remove/reorder/duplicate、selection与可逆transaction。

### P1-37：Physics Material scalar默认全零且缺范围验证

定义物理合理defaults、finite/range检查、surface type和版本迁移。

### P1-38：material combine rule与native结果不可预览

提供两材质pair preview及实际backend capability/降级诊断。

### P1-39：cook artifact未接DDC/export/residency

复用共享cache、packaging、dependency key和runtime shape refcount。

### P1-40：Bake结果没有source revision acknowledgement

旧job不得覆盖新mesh/settings；UI显示stale/applied/rejected generation。

### 6.5 Ragdoll、Debug与Diagnostics

### P1-41：Ragdoll用bone path字符串而非stable skeleton ID

重导骨架需映射/冲突报告，重复leaf不能静默丢失。

### P1-42：generator只按translation长度估capsule

需要mesh/skin/bone orientation fit、shape policy、左右对称和可视化诊断。

### P1-43：Ragdoll view没有骨架树、preview或属性controller

四个空节点应绑定真实document、selection、shape/joint/material/filter编辑。

### P1-44：Ragdoll mutation没有transaction/save/recovery

生成、regenerate、手调、删除与约束修改都必须可逆并保存原子化。

### P1-45：spawn/despawn/reload生命周期没有Editor preview owner

PreviewWorld管理native handles、reset、profile reload、scene/project close。

### P1-46：physical animation/blend/recovery没有authoring合同

先明确profiles、strength、mode、transition和runtime acknowledgement。

### P1-47：overlay builder没有provider generation/filter

provider应绑定world/backend generation并支持shape/body/contact/joint/COM/sleep过滤。

### P1-48：debug toggle只开view

改为正式ViewportCommand，并让view只控制provider filters/readiness。

### P1-49：Physics Diagnostics三块全空

step history、collision matrix、world stats必须来自bounded runtime snapshot。

### P1-50：debug/diagnostic无容量和stale清理

geometry/event/counter有预算、时间戳、overflow及disable/unload清除证明。

### 6.6 Workbench、测试、性能与产品证据

### P1-51：38条Workbench route只做UI导航

业务action应进入typed controller/document/job，纯选择route保持UI-only。

### P1-52：Bake/Test/Simulate/Validate固定显示queued

必须显示JobId、generation、progress、terminal result和可定位diagnostic。

### P1-53：静态124 bodies/32 contacts构成虚假遥测

无runtime snapshot时显示Unavailable，禁止默认样例数字。

### P1-54：静态Ice material/82kg掩盖无资产选择

从真实asset/scene selection投影，并处理missing/stale/read-only。

### P1-55：缺隔离Physics PreviewWorld

Simulate不能直接污染authoritative scene；支持play/reset/step/pause与apply-back policy。

### P1-56：validate没有统一规则集和对象定位

验证material/profile/shape/mass/joint/cook/backend，输出stable code与quick-fix事务。

### P1-57：测试把DTO、registration和approximate backend当产品证据

按Contract/Backend/Authoring/Product/Acceptance五层分级，沿用08A裁决。

### P1-58：缺真实Jolt editor preview、native dist/export lane

同一artifact必须在Editor、runtime app和exported client报告一致digest/capability。

### P1-59：缺大规模authoring性能预算

覆盖100k bodies/colliders、批量gizmo、cook corpus、debug extract和diagnostics刷新。

### P1-60：没有同质量竞品基线

与Unreal/Godot固定solver、shape、场景、tick、线程和输出语义后比较authoring延迟与runtime成本。

## 7. P2 扩展差距

### P2-1：Character Controller authoring与step/slope测试工具缺失

### P2-2：Vehicle chassis/wheel/suspension/tire editor缺失

### P2-3：Destruction/fracture/Geometry Collection authoring缺失

### P2-4：Soft body、cloth与rope physics authoring缺失

### P2-5：Fluid、particle collision和buoyancy authoring缺失

### P2-6：Rewind/resimulation/network determinism调试器缺失

### P2-7：Physics recording、scrub与state diff工具缺失

### P2-8：Constraint profile library与批量retarget缺失

### P2-9：自动collision质量评估与golden contact测试缺失

### P2-10：第三方physics backend/plugin cook兼容治理缺失

### P2-11：分布式convex/mesh cook与远程缓存尚未接入

### P2-12：多人协作physics asset semantic diff/merge缺失

## 8. 参考引擎裁决

| 参考 | 已证明职责 | Zircon差距 |
|---|---|---|
| Unreal PhysicsAssetEditor | body/primitive/constraint graph、骨架树、选择、生成/重建、仿真交互、collision/mass与transaction | Zircon Ragdoll空view、无document/gizmo/preview/transaction |
| Unreal StaticMeshEditor/ChaosEditor | simple/convex collision生成、复杂collision、fracture/cook/diagnostics工具 | Zircon Proxy仅静态42%/6 hulls和queued文本 |
| Godot Physics Editor | shape/joint/physical-bone gizmo、handle commit与UndoRedo | Zircon仅RigidBody通用字段投影，无Collider/Joint handles |
| Fyrox editor | 无独立physics authoring模块，主要依赖通用Inspector | 只能参考反射边界，不能作为完成基线 |
| Bevy | 无内建physics/editor | 不可用于Physics Editor完成度证明 |
| Unity Graphics | 本地checkout无Physics authoring | 不比较，不从Graphics路径推断Unity Physics |

## 9. 分层里程碑

### M0：能力真相与产品装配

默认装配Physics editor；未实现surface/command隐藏或disabled；建立factory/provider/controller bootstrap硬门。

### M1：Physics Material与Collision Profile

完成versioned schema、合理defaults、toolkits、transaction、reference migration与backend filter compile。

### M2：Scene component authoring

统一RigidBody/Collider/Joint typed schema、Inspector、多选、validation与save/undo。

### M3：Viewport gizmo与selection

完成shape/joint handles、stable subshape IDs、hit proxy、drag transaction及mass/COM可视化。

### M4：Physics cook与Collision Proxy

实现mesh/convex/heightfield/compound source、job、artifact、DDC/export、runtime shape registration。

### M5：RagdollAsset与PreviewWorld

完成skeleton mapping、generator、document、body/joint编辑、isolated simulation、spawn/despawn/reload。

### M6：Debug Overlay与Diagnostics

注册真实provider，消费generation-bound counters/geometry/events，支持filters、预算和stale清理。

### M7：Workbench产品化

两份静态workspace改为真实selection/document/job/runtime projection，所有action具备terminal result。

### M8：规模与竞品验收

执行真实Jolt/native dist/export、fault/soak、cook corpus、100k规模和同质量Unreal/Godot基线。

依赖顺序：`M0 -> M1 -> M2 -> M3 -> M4 -> M5 -> M6 -> M7 -> M8`。M4可与M2部分并行，但不得绕过artifact/transaction；M6不得用测试DTO代替provider。

## 10. 验收矩阵

| Gate | 必须证明的结果 |
|---|---|
| G1 | manifest启用Physics时默认加载editor/resources/toolkits/provider/runtime；缺项则capability不可用。 |
| G2 | Physics Material可打开、编辑、验证、undo/save/reload，缺插件有只读fallback。 |
| G3 | 四份ZUI无业务`Space`占位，每个visible control有provider/action/error。 |
| G4 | create/bake/simulate/validate均有typed executor、target、generation和terminal result。 |
| G5 | RigidBody全部authoring字段共享runtime validator并支持mixed-value transaction。 |
| G6 | Collider所有shape variant可编辑、验证、序列化和runtime round-trip。 |
| G7 | Joint frames/limits/drives/break/collide-connected按backend capability编辑。 |
| G8 | shape/joint gizmo drag支持cancel/undo，local/world/scale与runtime一致。 |
| G9 | subshape selection/duplicate/delete使用stable ID，reload后可恢复。 |
| G10 | mass/COM/inertia可视化与backend计算在容差内一致。 |
| G11 | CollisionProfile version/migration/reference impact与native filter generation闭环。 |
| G12 | Block/Overlap/Ignore、query/solver/sensor语义在Editor、Jolt和events一致。 |
| G13 | channel/profile超限在cook/export前结构化失败。 |
| G14 | Physics Material defaults/finite/range/combine验证和native映射正确。 |
| G15 | mesh/convex/heightfield/compound cook产物绑定source/settings/backend/platform key。 |
| G16 | 恶意/退化/超大mesh corpus受byte/time/memory/triangle/hull预算限制。 |
| G17 | cook job可取消、失败清理、旧generation拒绝、artifact可DDC复用。 |
| G18 | production注册cooked mesh asset，Editor/runtime/export使用同一digest。 |
| G19 | Ragdoll create读取真实skeleton并写入RagdollAsset，而非只OpenView。 |
| G20 | skeleton reimport以stable bone ID迁移并报告missing/ambiguous mapping。 |
| G21 | Ragdoll body/joint/filter/material修改可undo/save/reload和preview。 |
| G22 | PreviewWorld reset/step/pause/close不污染项目scene且回收native handles。 |
| G23 | Physics overlay command切换正式provider并在disable/unload清空stale frame。 |
| G24 | overlay shape/transform/sensor与canonical runtime generation一致。 |
| G25 | diagnostics显示真实step/body/sleep/pair/contact/query/constraint/cook/capacity counters。 |
| G26 | debug geometry/event/counter有预算、overflow、timestamp和filter。 |
| G27 | Collision Proxy Workbench不再显示固定18 proxies/42%/6 hulls。 |
| G28 | Physics Collision Workbench不再显示固定124 bodies/32 contacts/82kg/4 manifolds。 |
| G29 | close project/plugin unload取消cook、停止preview、清provider并拒绝late completion。 |
| G30 | Editor、runtime app、native dist/export同一Jolt build/profile/artifact通过产品测试。 |
| G31 | 100k scene与cook/debug压力满足CPU、memory、UI latency和shutdown预算。 |
| G32 | Unreal/Godot同任务报告固定版本/硬件/solver/质量/线程并保留原始结果，才允许性能声明。 |

## 11. 实施约束

1. 不另造Editor私有solver、scene physics world、collision profile或cook cache。
2. 不用no-op factory、OpenView事件、固定queued文本、默认数字或builtin approximate backend冒充产品成功。
3. 所有mutation进入Editor transaction/save/recovery，所有cook/preview/provider结果绑定source/backend generation。
4. Physics debug只通过正式viewport provider发布，不直接改viewport或全局cache。
5. cook复用共享Job/DDC/export/artifact系统，fixed tick不得临时分解或cook mesh。
6. 实施前重取在途`simulation_physics.rs`及本报告182文件fingerprint，不回滚用户变化。

## 12. 本轮状态

- Review：完成Editor Physics纵向静态审查；Runtime算法差距归08A。
- Production code：未修改。
- Dynamic validation：未执行，既有Editor test-build阻断未变化。
- Static validation：完成；5/60/12/32编号连续，38个frontmatter路径与71个报告/索引相对链接存在，182文件fingerprint复核一致，`git diff --check`无whitespace错误（仅既有CRLF转换提示）。
- Implementation：pending，按M0-M8推进。
