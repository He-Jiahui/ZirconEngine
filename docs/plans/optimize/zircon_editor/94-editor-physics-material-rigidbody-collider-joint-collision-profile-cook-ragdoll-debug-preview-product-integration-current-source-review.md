---
title: Editor Physics Material / RigidBody / Collider / Joint / Collision Profile / Cook / Ragdoll / Debug / Preview 与 Product Integration 当前源码复审
category: zircon_editor
report_id: Editor94
review_date: 2026-08-25
baseline_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
verification_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
related_failure:
  - docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md
related_code:
  - zircon_plugins/physics/editor
  - zircon_plugins/physics/runtime
  - zircon_plugins/first_party_editor_catalog
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/editor_extension
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/scene/physics.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/PhysicsAssetEditor
  - dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/StaticMeshAutomationTests.cpp
  - dev/Fyrox/editor/src/plugins/collider
  - dev/Fyrox/editor/src/plugins/inspector/editors/mod.rs
  - dev/godot/editor/scene/3d/gizmos/physics
  - dev/godot/editor/scene/2d/physics/collision_shape_2d_editor_plugin.cpp
  - dev/godot/editor/scene/3d/physics/physical_bone_3d_editor_plugin.cpp
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor94 · Physics Authoring 与 Product Integration 当前源码复审

## 1. 结论

当前 Zircon 已有一套值得保留的 Physics 类型与 Runtime 底座：Scene 能持久化 RigidBody、Collider、Joint 和 Physics Material，Collider 覆盖 Box、Sphere、Capsule、Cylinder、ConvexHull、TriangleMesh、HeightField 与 Compound；Runtime 有 backend contract、Jolt/builtin 实现、query/contact/trigger、manager/world sync、skeletal ragdoll profile，以及可序列化的 Physics Material。Editor 也有共享 document、transaction、job、operation、asset registry、viewport/provider 基础。这些说明工程并非从零开始，但不能据此宣称 Physics authoring 产品已经存在。

当前默认产品链仍断裂。first-party Editor catalog 只链接 Navigation 与 Neural，Physics 插件没有进入默认 Editor 装配；builtin Physics Material 只有 presentation，没有 toolkit。Physics Editor 的 13 个文件注册 Authoring、Diagnostics、Debug Overlay 与 Ragdoll Profile surface，但五个有效 command descriptor 全部只是 event route：三个 open view、一个把 debug toggle 路由成 `WorkbenchMenu(OpenView(...))`、一个把 ragdoll create 路由成 `WorkbenchMenu(OpenView(...))`。没有 operation factory、document/controller、cook job、preview owner 或 terminal artifact receipt。

四份插件 ZUI 共 **17 个 node、11 个 `Space`、0 个 event**，因此 Authoring、Diagnostics、Debug 与 Ragdoll 都只是空壳。`build_physics_overlay`和`generate_initial_ragdoll_profile`只有定义、重导出和测试，没有 production caller；Ragdoll runtime profile 也只被测试调用。已打开的 debug failure handoff 仍准确：Physics 没有注册 `ViewportOverlayProviderRegistration`，toggle 也没有进入 `ViewportCommand::ToggleOverlayProvider`。

Workbench 的 Collision Proxy 与 Physics Collision 两份 workspace 更严重：38 条业务 route 最终只做 tab/row selection、dropdown/control mutation或固定 feedback。Bake、Test Contacts、Simulate、Validate 不创建 JobId，不接 Runtime world，不生成 artifact，也没有 cancellation/terminal result，却固定显示“queued”、`18 proxies`、`42 percent`、`6 hulls`、`124 bodies`、`32 contacts`、`82 kg`和`1 warning`。这是能力真实性问题，不是 UI 完成度问题。

本轮重判 **Editor18 的 5 项 P0、60 项 P1、12 项 P2 全部 Open；32 项资格门全部 Fail**。Runtime138 继续唯一持有 solver/world/backend/fixed-step/query/contact/constraint 等 Runtime 差距；Editor94 只负责 authoring/product chain，避免重复登记。没有动态、质量或竞争证据支持 Zircon Physics 的功能、性能或表现优于 Unreal；同场景、同 solver 参数、同 shape/cook 质量、同 tick/thread/hardware 的 correctness-first benchmark 建立前，禁止作此声明。

## 2. 审查边界、统计与 currentness

### 2.1 冻结范围

统计口径为当前 working tree 的物理文件、物理行、非空行、bytes、Rust `#[test]` 与 `#[ignore]` 声明。fingerprint 对 repository-relative lowercase path 排序，为每个文件拼接 `path + NUL + lowercase(file SHA-256) + LF` 后再取 SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Physics Editor/plugin | **13 / 709 / 637 / 25,310 / 4 / 0** | `c63d61e449b2f197db636ea2c6728da9370e3ecffcd81debe42f1f6485a36523` |
| Physics Workbench route selected | **7 / 1,731 / 1,526 / 98,424 / 0 / 0** | `ed07388538c5bf124df7f932843eadaab8eb12984445b6c7a4a386be83e3b578` |
| Editor shared/product boundary | **287 / 37,261 / 33,739 / 1,320,506 / 254 / 8** | `e44b11930367d35c6aa1d6d4cc7ae4bdcabfc14b2e20a09ab1a53221a29ecef9` |
| Runtime、asset 与 backend downstream selected | **130 / 16,143 / 14,938 / 561,551 / 104 / 2** | `eb501d2c05ff7e76a383b40bfdd6c36f636e3adaa2dcb56ef6a8e10bc8455448` |
| Zircon selected union | **417 / 53,404 / 48,677 / 1,882,057 / 358 / 10** | `ab41d1240825b399f88cb8f5d7d4bb95ae88f7c3aaf6fcb9feeb667c2677bd53` |
| Unreal selected | **9 / 7,106 / 5,890 / 264,288 / 0 / 0** | `89acda9e1a562abc9451f1b48c3a4901758c4a53a7bd30a4327dd535684d5800` |
| Fyrox selected | **16 / 3,467 / 3,138 / 126,270 / 0 / 0** | `9ebdc32941abc3e57bc5a7914099e1765ffc2cb2d90b156e5f9b91ed0db5077f` |
| Godot selected | **4 / 2,213 / 1,788 / 73,982 / 0 / 0** | `384c7cce071624f6ec4f80b1c74e779b7c42e609cce3c1d2a6eae2ece3332058` |
| Bevy selected | **1 / 367 / 319 / 13,890 / 3 / 0** | `62bc7bd81b937e28210addc10fda08f26065d47564aa89d26dc7e8180ece9bdd` |
| Unity Graphics selected | **3 / 596 / 506 / 23,837 / 0 / 0** | `6f26056b962ef3ca97731ad7d88542ac4b2c145ae5840948b82e8bc39dde265d` |
| 五引擎 reference selected union | **33 / 13,749 / 11,641 / 502,267 / 3 / 0** | `b4ff07111c25c39c312bda054e509fa7651f46a250c8ff94859a1cc41c8ece44` |

选择规则：Physics Editor/plugin 为`zircon_plugins/physics/editor/**`加`plugin.toml`；Workbench 为两个 index/assets 入口、两份 simulation workspace、navigation specs 与 physics feedback 共7文件；Editor shared 为前两组加 asset type、document、editing、jobs、editor extension、scene viewport及三份通用dispatch文件；Runtime为Physics plugin runtime、framework physics/scene physics及Physics Material/Scene asset/import/test选集。参考集为9个Unreal PhysicsAsset/StaticMesh文件、Fyrox collider目录加inspector registry、4个Godot physics editor文件、Bevy fixed time和3个Unity Graphics VFX collision consumer文件。

### 2.2 currentness 与限制

- baseline 与 verification HEAD 均为 `8ee9411db24b7b4bdaf3fe028194642a7557c0b6`，commit time 为 `2026-08-25T17:37:22+08:00`。
- 417 个 Zircon selected 文件中有 **147 个**包含用户或其他 Session 的 working-tree 修改/新增；本报告读取物理现状，不回退、不覆盖，也不把在途变化写成已集成能力。
- Runtime138 baseline 后可见的 Physics 在途变化包括 importer 借用源文本、manager snapshot 访问和 runtime tick delta 等局部调整；它们没有新增 Physics Editor catalog、toolkit、factory、provider、document、job或产品 caller，因此不关闭 Editor18 条目。
- 按用户要求未查询、轮询或等待协调器；Tooling 也不在本轮范围。
- 本轮只做源码 review 与文档记录，未运行 Cargo、Editor、Jolt native preview、asset import/cook、save/reopen、PIE、exported client、fault、scale、soak、profiling或竞争 benchmark。

### 2.3 Owner 边界

- Editor94 唯一负责 Physics Material/Collision Profile/Ragdoll asset toolkit、RigidBody/Collider/Joint inspector、gizmo、cook UI、PreviewWorld、debug/diagnostics provider、Workbench controller与transaction/job/product receipt。
- [Runtime138](../zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md) 负责 Physics World、body/shape/material/joint/query/contact/trigger、fixed-step、backend parity、Jolt、character/vehicle/ragdoll runtime和shutdown；Editor94不重复其42项Runtime Gate。
- [Editor02](02-document-transaction-save-autosave-recovery-review.md)、[Editor04](04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md)、[Editor05](05-inspector-reflection-property-authoring-customization-review.md)、[Editor09](09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md)分别持有共享document、asset/import、Inspector与job authority。
- [Physics debug overlay failure](../../zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md)保持Open；只有真实provider注册、正式toggle路由与stale清理证据才能关闭。

## 3. 当前产品链事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| App/Catalog | Physics manifest声明editor module和experimental capability；first-party Editor catalog只链接Navigation、Neural | Open |
| Physics Material | TOML asset与builtin presentation存在；没有toolkit、preview、reference/migration UI | Open |
| Plugin surface | Authoring、Diagnostics、Debug Overlay、Ragdoll Profile view/surface均成功注册 | Descriptor foundation |
| Commands | 5个有效descriptor：3个open、debug toggle、ragdoll create；全部event-routed，0 factory/provider/job | Open |
| Plugin ZUI | 4文件、17 nodes、11 `Space`、0 event；所有业务区无controller/data projection | Open |
| Overlay | snapshot到primitive的纯转换存在，sensor会标Trigger；production caller/provider为0 | Open |
| Ragdoll generator | string bone path与translation-length capsule初始生成、rollback/spawn测试存在；生产asset链caller为0 | Open |
| Scene schema | RigidBody/Collider/Joint/Material持久化和部分property read/write存在 | Partial foundation |
| Workbench | 两份workspace 38业务route；command只固定反馈，field只切popup/control，数据全部sample常量 | Open |
| Runtime downstream | backend/Jolt/builtin/query/contact/trigger/manager/skeletal具真实代码；产品可达性及精确语义由Runtime138保持Open | Runtime-owned |
| Evidence | 4个Physics Editor tests只验证registration、overlay DTO、generator；不启动默认产品或执行资产工作流 | Open |

## 4. 必须保留的真实底座

1. 保留 Runtime-owned Physics contracts、typed body/shape/joint/material/query/event DTO和Jolt唯一backend owner；Editor不得另造第二套solver或world。
2. 保留 Scene physics schema及已有property访问，把它升级为共享schema/validator consumer，而不是再建Editor私有字段模型。
3. 保留 Physics Material TOML round-trip和importer fail-close路径，但source、derived cook artifact与runtime residency必须分层。
4. 保留 `build_physics_overlay`的纯snapshot-to-primitive边界，将它接入generation-bound provider，不允许直接读取可变World或native backend。
5. 保留 ragdoll生成与partial-spawn rollback雏形，但用stable skeleton identity、transactional source asset和isolated preview补全。
6. 保留 Editor共享transaction/document/job/operation/provider/viewport基础，Physics必须成为这些authority的consumer。
7. 保留现有ZUI control/asset ID作为迁移输入；provider未就绪时隐藏或明确Unavailable，不能继续显示空白或固定成功。
8. 保留Physics `Experimental`成熟度；G01-G32全Pass前不得提升为Ready/Shipping。

## 5. P0：产品虚假可达与纵向闭环断裂

| ID | 状态 | 当前问题 | 必须重构为 |
|---|---|---|---|
| PHY2-P0-01 | Open | Physics Material和Physics插件没有默认产品装配闭环 | profile-qualified activation plan原子绑定runtime/editor/resources/toolkit/controller/provider；缺项整能力fail-close |
| PHY2-P0-02 | Open | 四份Physics surface以11个`Space`发布完整能力 | 真实document/controller/provider投影；未实现区域隐藏或disabled并给typed reason |
| PHY2-P0-03 | Open | Ragdoll create只打开空view，generator/runtime profile与资产链断开 | stable skeleton输入、transactional `RagdollAsset`、build artifact、PreviewWorld、save/spawn/despawn receipt |
| PHY2-P0-04 | Open | debug toggle只OpenView且Physics没有Viewport overlay provider | Physics-owned provider registration、`ToggleOverlayProvider`、generation/filter与disable/unload stale清理 |
| PHY2-P0-05 | Open | Collision Proxy/Physics Collision用固定数据伪造Bake、Simulate、Validate | typed command进入document/job/preview/runtime，返回JobId/progress/terminal artifact或diagnostic；删除固定成功文本 |

## 6. P1：工程级完整性差距

### 6.1 装配、资产与能力真相

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY2-P1-01 | Open | default catalog与Runtime Physics能力不对称 | 同一package selection解析editor/runtime/resource/provider closure和generation |
| PHY2-P1-02 | Open | resource materialization只验证ID，不验证controller/readiness | publish前preflight资源、schema、factory、provider与backend，失败整批回滚 |
| PHY2-P1-03 | Open | Physics Material没有专用toolkit | 支持typed edit/diff/validate/preview/save/reload/reference和只读fallback |
| PHY2-P1-04 | Open | Ragdoll asset type只随插件加载存在 | 定义默认assembly、unknown-plugin placeholder、schema migration和recovery |
| PHY2-P1-05 | Open | generic authoring drawer与view职责重叠 | 分离asset toolkit、scene inspector、cook、ragdoll、debug与diagnostics owner |
| PHY2-P1-06 | Open | capability不描述Jolt/builtin精度和unsupported组合 | 消费Runtime machine-readable backend capability/degraded report |
| PHY2-P1-07 | Open | operation readiness不包含provider/job/backend | 可见命令必须证明executor、target、permission、write authority和backend readiness |
| PHY2-P1-08 | Open | schema字符串没有typed payload/version/budget | create/bake/simulate/validate/edit统一versioned decoder、migration和size/depth/items预算 |
| PHY2-P1-09 | Open | plugin disable没有preview/provider/job drain | revoke admission后cancel cook、stop preview、clear overlay、拒绝late commit再卸载资源 |
| PHY2-P1-10 | Open | tests只断言ID、view和纯函数DTO | 增加默认bootstrap、asset-open、controller/provider、dispatch、save/reopen与shutdown产品测试 |

### 6.2 Scene component、Inspector 与 Gizmo

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY2-P1-11 | Open | RigidBody虽有typed持久化/部分property path，仍无完整产品Inspector | body type、mass/inertia、gravity/damping、CCD、sleep、locks共享schema/validator |
| PHY2-P1-12 | Open | Collider shape union没有完整variant Inspector | Box/Sphere/Capsule/Cylinder/Convex/Mesh/HeightField/Compound按variant投影验证 |
| PHY2-P1-13 | Open | Joint没有frames/limits/drives/break编辑器 | 各joint按backend support显示合法字段，并用qualified对象picker |
| PHY2-P1-14 | Open | 多选缺shape/component兼容规则 | mixed values按component presence和variant分组，逐target验证后原子提交 |
| PHY2-P1-15 | Open | Scene property、Editor和Runtime validator仍未形成单一schema authority | 非法值prepare阶段拒绝并保留旧值，compiler/runtime复用同一规则 |
| PHY2-P1-16 | Open | shape/joint没有可逆viewport handles | begin/update/commit/cancel drag transaction，尺寸、offset、rotation、frame/limit可撤销 |
| PHY2-P1-17 | Open | local/world transform与non-uniform scale policy未闭合 | gizmo、cook、Jolt shape和debug geometry共享坐标、单位、缩放与mirror规则 |
| PHY2-P1-18 | Open | Collider/Joint/subshape选择与hit proxy缺失 | stable element ID、resolved hit、frame/duplicate/delete与reload restore |
| PHY2-P1-19 | Open | mass/inertia/COM preview缺失 | 显示density来源、COM、principal axes、tensor和backend计算误差 |
| PHY2-P1-20 | Open | Play/preview期间字段可编辑性未定义 | 区分authoring-only、simulation-only、safe live-edit并绑定world generation |

### 6.3 Collision Profile、Query 与 Response

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY2-P1-21 | Open | 没有versioned `CollisionProfileAsset` | 单一authority持有channel/object/trace类别、response matrix、default与migration |
| PHY2-P1-22 | Open | Jolt native filtering未消费authoring matrix | 编译backend filter artifact并在Editor预览其generation/capability |
| PHY2-P1-23 | Open | Workbench channel mask固定WorldStatic/Player | 真实profile matrix、搜索、批量编辑、semantic diff与undo |
| PHY2-P1-24 | Open | query/solver/sensor响应语义未分开 | 明确Block/Overlap/Ignore、QueryOnly/PhysicsOnly、Sensor与trace channel合同 |
| PHY2-P1-25 | Open | layer容量/backend映射无cook-time诊断 | 超限、冲突、unsupported或降级在export前结构化失败 |
| PHY2-P1-26 | Open | profile reload没有world rebuild/remap策略 | source revision到compiled generation，以barrier/LKG切换并拒绝stale结果 |
| PHY2-P1-27 | Open | contact row缺stable pair/subshape/material identity | runtime event带qualified entity/shape/face/material/world generation并可定位 |
| PHY2-P1-28 | Open | query debug没有真实ray/sweep/overlap结果 | 显示filter/backend/exactness/hit ordering/cost/overflow和unsupported |
| PHY2-P1-29 | Open | trigger authoring没有Enter/Stay/Exit预览 | PreviewWorld消费backend bounded event stream，不在Editor重算overlap |
| PHY2-P1-30 | Open | collision channel rename/delete无reference impact | 扫描scene/asset/script引用，transactionally migrate或拒绝并给fix plan |

### 6.4 Collision Cook、Proxy 与 Physics Material

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY2-P1-31 | Open | `register_mesh_asset`无production caller | 建立共享PhysicsCookArtifact加载、shape cache、refcount和runtime registration |
| PHY2-P1-32 | Open | Collision Proxy Bake没有job executor | source hash/settings/backend/platform、进度、cancel和terminal artifact齐全 |
| PHY2-P1-33 | Open | Convex decomposition只显示静态42%/6 hulls | versioned算法、hull/verts/error/seed预算、preview和source/artifact diff |
| PHY2-P1-34 | Open | triangle mesh缺weld/degenerate/material-slot验证 | bad geometry在bounded cook阶段失败，不进入fixed tick/native world |
| PHY2-P1-35 | Open | heightfield缺terrain revision与tile cook | region/tile增量、holes/materials/scale、dependency和runtime residency |
| PHY2-P1-36 | Open | compound collider缺child stable ID/transform编辑 | add/remove/reorder/duplicate、subselection和可逆transaction |
| PHY2-P1-37 | Open | Physics Material scalar默认全零且缺完整范围治理 | 合理default、finite/range/unit、surface type、schema version与migration |
| PHY2-P1-38 | Open | combine rule/native结果不可预览 | 两材质pair preview消费真实backend capability/result并显示降级原因 |
| PHY2-P1-39 | Open | cook artifact未接共享DDC/export/residency | content-addressed key、dependency closure、packaging、poison recovery和shape reuse |
| PHY2-P1-40 | Open | Bake没有source revision acknowledgement | old job不得覆盖新mesh/settings；UI显示stale/applied/rejected generation |

### 6.5 Ragdoll、Debug 与 Diagnostics

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY2-P1-41 | Open | Ragdoll profile用bone path字符串 | stable skeleton/bone identity、reimport remap和missing/ambiguous冲突报告 |
| PHY2-P1-42 | Open | generator只按translation长度估capsule | mesh/skin/orientation fit、shape policy、左右对称、预算与visual diagnostic |
| PHY2-P1-43 | Open | Ragdoll view没有骨架树、preview或property controller | 真实document绑定selection、body/shape/joint/material/filter编辑 |
| PHY2-P1-44 | Open | Ragdoll mutation没有transaction/save/recovery | generate/regenerate/manual edit/delete均可逆、原子持久化并可恢复 |
| PHY2-P1-45 | Open | spawn/despawn/reload没有Editor Preview owner | PreviewWorld管理native handle、reset、profile generation与close fence |
| PHY2-P1-46 | Open | physical animation/blend/recovery无authoring合同 | versioned profile、strength/mode/transition、preview和runtime acknowledgement |
| PHY2-P1-47 | Open | overlay builder没有provider generation/filter | provider绑定viewport/world/backend generation，支持body/shape/contact/joint/COM/sleep过滤 |
| PHY2-P1-48 | Open | debug toggle仍只是OpenView | 改为正式ViewportCommand，view只控制provider filters/readiness |
| PHY2-P1-49 | Open | Physics Diagnostics三块业务区域全空 | step history、matrix、world stats来自bounded runtime snapshot |
| PHY2-P1-50 | Open | debug/diagnostic无容量与stale治理 | geometry/event/counter有budget、timestamp、overflow、drop和disable/unload清理 |

### 6.6 Workbench、测试、性能与产品证据

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY2-P1-51 | Open | 38条Workbench route只做UI导航/控件变化 | 业务action进入typed controller/document/job；纯tab/row route明确UI-only |
| PHY2-P1-52 | Open | Bake/Test/Simulate/Validate固定显示queued | 显示真实JobId、generation、progress、terminal result与定位diagnostic |
| PHY2-P1-53 | Open | 固定124 bodies/32 contacts/4 manifolds伪装遥测 | 只投影runtime snapshot；无数据时显示Unavailable/stale age |
| PHY2-P1-54 | Open | 固定Ice/82kg/Player Capsule掩盖无selection | 从qualified asset/scene selection投影并处理missing/stale/read-only |
| PHY2-P1-55 | Open | 缺隔离Physics PreviewWorld | play/reset/step/pause不污染authoritative scene，apply-back policy显式 |
| PHY2-P1-56 | Open | Validate无统一规则集和对象定位 | 验证material/profile/shape/mass/joint/cook/backend，stable code加transactional quick fix |
| PHY2-P1-57 | Open | DTO/registration/approximate backend测试被误当产品证据 | 分Contract/Backend/Authoring/Product/Acceptance五层，逐层保留failure evidence |
| PHY2-P1-58 | Open | 缺真实Jolt Editor preview与native dist/export lane | 同一artifact在Editor/runtime/exported client报告一致digest/capability/result |
| PHY2-P1-59 | Open | 缺大规模authoring性能预算 | 100k body/collider、批量gizmo、cook corpus、debug extract、diagnostics refresh和shutdown |
| PHY2-P1-60 | Open | 没有同质量竞品基线 | 固定solver/shape/cook/tick/thread/hardware与correctness oracle后比较延迟、吞吐和内存 |

## 7. P2：工程级能力扩展

| ID | 状态 | 扩展差距 | 目标 |
|---|---|---|---|
| PHY2-P2-01 | Open | Character Controller authoring与step/slope测试工具缺失 | capsule/controller asset、slope/step/contact test场景和runtime trace |
| PHY2-P2-02 | Open | Vehicle chassis/wheel/suspension/tire editor缺失 | versioned vehicle source、wheel gizmo、setup validator与isolated preview |
| PHY2-P2-03 | Open | Destruction/fracture/Geometry Collection authoring缺失 | fracture graph、chunk hierarchy、cook artifact和damage preview |
| PHY2-P2-04 | Open | Soft body、cloth与rope authoring缺失 | topology/constraint/material source、solver capability与debugger |
| PHY2-P2-05 | Open | Fluid、particle collision和buoyancy authoring缺失 | cross-system interaction schema、budget与preview/runtime parity |
| PHY2-P2-06 | Open | Rewind/resimulation/network determinism debugger缺失 | authoritative tick/input/state hash、divergence定位与bounded capture |
| PHY2-P2-07 | Open | Physics recording、scrub与state diff缺失 | generation-qualified capture、timeline、entity/constraint diff和export |
| PHY2-P2-08 | Open | Constraint profile library与批量retarget缺失 | reusable profile、stable binding、validation与semantic batch transaction |
| PHY2-P2-09 | Open | 自动collision质量评估与golden contact缺失 | coverage/error/penetration/stability oracle和reviewable artifact |
| PHY2-P2-10 | Open | 第三方backend/plugin cook兼容治理缺失 | ABI/schema/capability/artifact version、sandbox与fallback policy |
| PHY2-P2-11 | Open | 分布式convex/mesh cook与远程缓存未接入 | content-addressed remote execution、provenance、cancel和poison recovery |
| PHY2-P2-12 | Open | 多人协作physics asset semantic diff/merge缺失 | stable IDs、semantic diff、conflict model与review artifact |

## 8. 历史台账重判

| 历史报告 | 当前重判 | 说明 |
|---|---:|---|
| Editor18 P0 | **5 Open / 0 Partial / 0 Closed** | 默认装配、11个Space、Ragdoll链、overlay provider和两份虚假Workbench均未闭合 |
| Editor18 P1 | **60 Open / 0 Partial / 0 Closed** | Scene/Runtime typed基础不满足任一Editor端到端完成条件 |
| Editor18 P2 | **12 Open / 0 Partial / 0 Closed** | controller/vehicle/destruction/soft-body/fluid/rewind/recording/profile/golden/backend/DDC/collaboration均无产品owner |
| Editor18 Gates | **32 Fail / 0 Partial / 0 Pass** | 每个Gate要求可执行产品闭环；schema、descriptor、pure helper或单元测试不单独构成Partial |

## 9. 参考引擎差异裁决

### 9.1 Unreal Engine

- `PhysicsAssetEditorSharedData.cpp`把body/constraint创建、删除、复制、collision pair修改和refresh纳入`FScopedTransaction`与`Modify`，而Zircon Ragdoll generator没有Editor document/history owner。
- `PhysicsAssetEditorEditMode.cpp`与Selection代码实现body/constraint/COM hit proxy、tracking begin/end、simulation interaction和选择状态；Zircon没有Collider/Joint/COM pick或drag transaction。
- Physics handle component把simulation manipulation作为专用交互状态管理，不让UI按钮直接碰native world。
- `StaticMeshAutomationTests.cpp`提供collision生产链的自动化入口；Zircon Workbench的Bake/Test只有固定feedback，没有artifact或产品测试。

### 9.2 Fyrox

旧Editor18关于“Fyrox无独立physics authoring模块”的结论已失效，必须修正。当前本地Fyrox有专用`ColliderPlugin`，区分2D/3D collider shape gizmo，支持selection、picking、drag context，并在释放时提交`SetPropertyCommand`；cuboid等shape有handles和`Try Fit` grouped command，mesh路径能创建Trimesh/Convex collider，Inspector也注册ColliderShape、RigidBodyType与joint参数。Zircon不能再以“Rust参考引擎也只靠通用Inspector”为降低标准的依据。

### 9.3 Godot

- 3D CollisionShape gizmo按shape类型创建handles，支持begin/set/commit/cancel、UndoRedo、readonly和debug geometry。
- Joint gizmo绘制frame/limits并按120 Hz节流增量刷新；PhysicalBone编辑器提供joint移动模式。
- 2D CollisionShape editor同样提供handle drag、cancel、UndoRedo和focus-out commit。Zircon现有Scene property path尚未形成这些交互合同。

### 9.4 Bevy 与 Unity Graphics

- Bevy本地参考没有内建Physics Editor；`bevy_time/src/fixed.rs`只能用于fixed-time accumulator/overstep等时间边界参考，不能证明authoring完成度。
- Unity Graphics checkout里的VFX CollisionShape/CollisionBase与RigidBody collision event binder证明Physics数据会被VFX等下游消费；它不是Unity Physics authoring源码，不能用于Editor feature parity，只能要求Zircon的collision identity/artifact保持跨系统稳定。

## 10. 目标架构

唯一允许的产品链为：

```text
PhysicsAuthoringSource
  -> PhysicsAuthoringDocument + Transaction/History
  -> shared schema/validator + reference impact
  -> bounded PhysicsCookJob / RagdollBuildJob
  -> immutable CollisionProfile/PhysicsCook/Ragdoll artifacts
  -> PhysicsPreviewWorld generation
  -> Runtime Physics world/backend generation acknowledgement
  -> bounded PhysicsDebugSnapshot
  -> Inspector/Gizmo/Workbench/Overlay truthful projection
```

核心资产至少包括`PhysicsMaterialAsset`、`CollisionProfileAsset`、`PhysicsCookSource/Artifact`和`RagdollAsset`。`PhysicsPreviewWorld`必须隔离于authoritative Scene/PIE；Editor只提交source mutation、build request和preview command，不得直接持有Jolt world。所有异步结果至少绑定project/document/source revision、backend/build/platform generation；stale completion只能丢弃或保留为诊断，不能覆盖当前资产。

## 11. 分层里程碑

| Milestone | 依赖 | 交付 | Exit |
|---|---|---|---|
| M0 Capability truth | MVP F0-F2 | Physics activation plan、catalog/toolkit/provider/factory/resource preflight | 缺一项整能力fail-close，无残余空view/假command |
| M1 Material/Profile | M0 + asset/document | versioned material/profile、reference impact、transaction、backend filter compile | edit/undo/save/reload/migrate/native preview通过 |
| M2 Scene authoring | M1 + Inspector | RigidBody/Collider/Joint schema、多选、validation、property transaction | 全variant round-trip且非法值原子拒绝 |
| M3 Viewport tools | M2 + Editor59 | pick、stable subshape、shape/joint handles、COM/inertia | drag cancel/undo、scale/coordinate/runtime parity通过 |
| M4 Cook/Proxy | M1-M3 + Editor09 | mesh/convex/heightfield/compound source、job、DDC/export artifact | cancel/stale/fault/corpus/runtime registration通过 |
| M5 Ragdoll | M2-M4 | stable skeleton mapping、generator、document、PreviewWorld | reimport/save/reload/step/reset/spawn/despawn通过 |
| M6 Debug/Diagnostics | M0 + Runtime138 snapshot | provider、filters、bounded geometry/event/counters | toggle/disable/unload/stale/overflow通过 |
| M7 Workbench | M1-M6 | 两份workspace接真实selection/document/job/runtime projection | 38条route各自证明UI-only或terminal product result |
| M8 Qualification | M1-M7 | native Jolt Editor/runtime/export、fault/scale/soak与竞争raw evidence | G01-G32全Pass才允许成熟度升级 |

M0不得绕过当前MVP主链。高级Physics实现只有在`.codex/plans/MVP重构方案/00-index.md`的F0-F5依赖允许后进入；本轮只记录差异，不提前实现旁路。

## 12. 资格门

| Gate | 状态 | 必须证明的结果 |
|---|---|---|
| G01 | Fail | Physics启用时默认加载editor/resources/toolkits/provider/runtime；缺项capability不可用 |
| G02 | Fail | Physics Material可打开、编辑、验证、undo/save/reload，缺插件有安全只读fallback |
| G03 | Fail | 四份ZUI不再用11个`Space`承载业务，每个visible control有data/state/action/error |
| G04 | Fail | create/bake/simulate/validate有typed executor、target、generation和terminal result |
| G05 | Fail | RigidBody全部authoring字段共享Runtime validator并支持mixed-value transaction |
| G06 | Fail | Collider全部shape variant可编辑、验证、序列化并runtime round-trip |
| G07 | Fail | Joint frames/limits/drives/break/collide-connected按backend capability编辑 |
| G08 | Fail | shape/joint gizmo支持cancel/undo，local/world/scale与Runtime一致 |
| G09 | Fail | subshape选择/duplicate/delete使用stable ID，reload后可恢复 |
| G10 | Fail | mass/COM/inertia可视化与backend计算在定义容差内一致 |
| G11 | Fail | CollisionProfile version/migration/reference impact/native filter generation闭环 |
| G12 | Fail | Block/Overlap/Ignore、query/solver/sensor语义在Editor、Jolt和event一致 |
| G13 | Fail | channel/profile超限在cook/export前结构化失败 |
| G14 | Fail | Physics Material defaults/finite/range/combine验证和native映射正确 |
| G15 | Fail | mesh/convex/heightfield/compound cook绑定source/settings/backend/platform key |
| G16 | Fail | 恶意/退化/超大mesh满足bytes/time/memory/triangle/hull预算 |
| G17 | Fail | cook可取消、失败清理、stale generation拒绝并可DDC复用 |
| G18 | Fail | production注册cooked mesh，Editor/runtime/export使用同一digest |
| G19 | Fail | Ragdoll create读取真实skeleton并写`RagdollAsset`，不再只OpenView |
| G20 | Fail | skeleton reimport按stable bone ID迁移并报告missing/ambiguous mapping |
| G21 | Fail | Ragdoll body/joint/filter/material可undo/save/reload和preview |
| G22 | Fail | PreviewWorld reset/step/pause/close不污染Scene且释放native handles |
| G23 | Fail | debug command切换正式provider并在disable/unload清空stale frame |
| G24 | Fail | overlay shape/transform/sensor与canonical Runtime generation一致 |
| G25 | Fail | diagnostics显示真实step/body/sleep/pair/contact/query/constraint/cook/capacity counters |
| G26 | Fail | debug geometry/event/counter有budget、overflow、timestamp和filter |
| G27 | Fail | Collision Proxy不再显示固定18 proxies/42 percent/6 hulls |
| G28 | Fail | Physics Collision不再显示固定124 bodies/32 contacts/82 kg/4 manifolds |
| G29 | Fail | close/unload取消cook、停止preview、清provider并拒绝late completion |
| G30 | Fail | Editor、runtime app、native dist/export同一Jolt build/profile/artifact通过产品测试 |
| G31 | Fail | 100k scene与cook/debug压力满足CPU、memory、UI latency和shutdown预算 |
| G32 | Fail | Unreal/Godot/Fyrox同任务报告版本、硬件、solver、质量、线程与原始结果后才允许性能声明 |

## 13. 禁止继续采用的临时实现

1. 禁止用manifest、capability、descriptor、view registration或测试DTO证明功能完成。
2. 禁止给Physics命令补no-op factory、OpenView、固定queued/Ready/计数或只改control property的executor。
3. 禁止把11个`Space`换成静态Label/Table后关闭条目；每个数据必须有真实owner与generation。
4. 禁止Editor直接创建Jolt/builtin world、临时solver、私有collision profile或第二份cook cache。
5. 禁止fixed tick临时分解/cook mesh；cook必须是bounded job和immutable artifact。
6. 禁止overlay直接遍历可变Scene/native world；只消费bounded、generation-qualified snapshot。
7. 禁止用string bone path继续充当Ragdoll长期identity，或把partial spawn测试当成资产产品闭环。
8. 禁止用unit test、approximate builtin backend或单一小场景benchmark替代native product/fault/scale/quality门。
9. 禁止以当前Runtime局部性能改动推导Editor功能已完成。
10. 禁止在同任务correctness、cook质量和原始数据缺失时宣称优于Unreal。

## 14. 完成定义

Editor94只有在以下条件同时满足时才可关闭：M0-M8按依赖顺序完成；Editor18的5项P0、60项P1、12项P2逐项有current-source产品证据；G01-G32全部Pass；默认Editor可打开Physics Material/Collision Profile/Ragdoll，能transactionally编辑RigidBody/Collider/Joint，能完成cook/preview/debug/diagnostics并save/reopen；Runtime138提供唯一可执行world/backend及generation receipt；真实Jolt Editor/runtime/export、fault、scale、soak和竞争证据可复现；旧Space、OpenView、固定feedback、无provider和无artifact路径已hard cutover且无兼容壳。

本轮没有修改production代码，也没有宣告整体Physics、Editor或Engine目标完成。
