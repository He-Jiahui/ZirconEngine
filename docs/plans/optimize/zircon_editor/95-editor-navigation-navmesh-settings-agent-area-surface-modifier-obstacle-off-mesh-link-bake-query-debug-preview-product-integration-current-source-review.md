---
title: Editor Navigation / NavMesh / Settings / Agent / Area / Surface / Modifier / Obstacle / Off-Mesh Link / Bake / Query / Debug / Preview 与 Product Integration 当前源码复审
category: zircon_editor
report_id: Editor95
review_date: 2026-08-25
baseline_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
verification_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
related_failure:
  - docs/plans/zircon_plugins/05/failure-2026-07-15-navigation-bake-selection-operation-arguments.md
  - docs/plans/zircon_plugins/05/failure-2026-07-19-navigation-runtime-fallback-hotpath.md
  - docs/plans/zircon_plugins/05/failure-2026-07-27-navigation-world-scan-deserialize-value.md
  - docs/plans/zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md
  - docs/plans/zircon_plugins/05/failure-2026-08-02-navigation-editor-operation-status-v2-cutover.md
related_code:
  - zircon_plugins/navigation/editor
  - zircon_plugins/navigation/runtime
  - zircon_plugins/navigation/native
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/editor_extension
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/navigation
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem
  - dev/UnrealEngine/Engine/Source/Developer/NavigationTestSuite
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/WorldPartition/WorldPartitionNavigationDataBuilder.cpp
  - dev/Fyrox/editor/src/interaction/navmesh
  - dev/Fyrox/editor/src/scene/commands/navmesh.rs
  - dev/Fyrox/editor/src/settings/navmesh.rs
  - dev/godot/modules/navigation_3d/editor
  - dev/godot/scene/resources/navigation_mesh.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor95 · Navigation Authoring 与 Product Integration 当前源码复审

## 1. 结论

Zircon Navigation 不是从零开始。默认 App feature、first-party Runtime catalog和first-party Editor catalog已经能够按项目选择装配Navigation；framework有中立DTO、operation ID、NavMesh/Settings资源种类和六类scene component合同；插件Runtime/Native包含Recast、Detour、Crowd、TileCache、tiled bake、query和overlay frame；Editor也有typed runtime event consumer、PIE session/sequence/owner-generation拒旧、viewport provider、selected-surface typed payload、V2 progress消费以及共享document、transaction、job、extension和viewport底座。这些是真实基础，必须保留。

但默认可见的Editor产品仍没有闭环。Navigation Editor当前34个文件、3,901行，注册4个authoring surface、5个UI template、5个component customization、5个operation、2个asset toolkit augmentation、1个runtime consumer和1个viewport provider。注册数量不能代表可用能力：11份ZUI共55个node、12个`Space`、3个Button和5处route，其中11个`Space`是Surfaces、Agents/Areas、NavMesh/Settings、五类drawer及Debug viewport业务占位，另1个只是Bake toolbar布局填充；除Bake外的资产、component和authoring主体没有model、controller、binding或transaction。

Bake仍存在生产合同自相矛盾。ZUI和`NavigationBakePanel`已经能投影stable `surface_entity`与`force_full_rebuild`，这是应保留的局部前进；但是`NavigationBakePanelController::new`只在测试出现，surface row、diagnostics、progress和terminal report没有产品producer。更严重的是Runtime `BakeScene/BakeSurface` handler的`prepare`固定返回`navigation bake requires a pure prepare backend`，`apply`也固定拒绝，而插件Runtime focused test仍要求Bake成功并继续Clear/Restore。产品路径因此不能完成一次真实Bake。

Debug链也只完成了基础接线。Runtime会在debug capture开启时发布typed full overlay frame，Editor mirror拒绝错误session、stale sequence和回退owner generation，provider能把frame变成gizmo extract；近期修改还在全部分类开启时避免一次中间`NavigationGizmoSnapshot` clone。但provider始终使用`NavigationOverlayOptions::default()`，只消费PIE mirror，以当前selection或`0`充当overlay owner；Toggle operation没有event/factory/handler，四个filter checkbox没有event，编辑态静态NavMesh、query preview、per-viewport state、culling/LOD/budget和full-frame复制问题都未解决。

内置Navmesh AI Workbench仍是第二套静态authority。230行模板有27个control和19条route，固定展示`NavMesh_Main`、`Agent_Humanoid`、`Query_Patrol`、`Tile 12_08/12_09`、`96 polys`、`42 cm`、`180 cm`、`Door_A03`和`18 tiles / 4 agents / 1 blocked link`。通用dispatch只改变tab/row/control状态；Rebuild与Query Path只返回预写的`queued` feedback，不创建JobId、query ticket、artifact或Runtime receipt。

本轮重判Editor19的 **5项P0、60项P1、12项P2全部Open，32项Editor资格门全部Fail**。Runtime141继续唯一持有world/backend/native owner、geometry、artifact、query、Crowd、obstacle、off-mesh movement、streaming和Runtime性能差距；Editor95只持有asset/document/Inspector/gizmo/Bake控制面、query preview、debug与Workbench产品闭环。当前没有动态或竞争证据支持Zircon Navigation的功能、性能或表现优于Unreal。

## 2. 审查边界、统计与currentness

### 2.1 冻结范围

统计对象为当前working tree的物理文件。行数和非空行按文本物理行统计，bytes取文件长度；`tests/ignored`只计Rust `#[test]`与`#[ignore]`声明，因此所选Unreal C++和Unity C#测试源不会增加该列。fingerprint对repository-relative lowercase path排序，为每个文件拼接`path + NUL + lowercase(file SHA-256) + LF`后再取SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Navigation Editor/plugin | **34 / 3,901 / 3,517 / 136,806 / 30 / 0** | `dab248c433019c4b2ecb03619de0100403f5d7903928981738756251b8510331` |
| Navigation Workbench selected | **7 / 2,644 / 2,468 / 141,692 / 0 / 0** | `392db23efe529518b448809f8f45428827482fee9a5cb241671f00701700185a` |
| Editor shared/product boundary | **417 / 65,401 / 60,190 / 2,325,158 / 355 / 9** | `04b49c69cc860879339a27d70f26d9f3129afac436a12c61082b55ea188104b7` |
| Runtime downstream selected | **183 / 49,284 / 43,730 / 1,575,995 / 156 / 7** | `e62647933d1f874ceed83e9881307a1ff5a66cf6350adcd8bd1a3a9f0826245a` |
| Zircon selected union | **599 / 114,627 / 103,868 / 3,899,047 / 511 / 16** | `2d20ee49a98581ba14fd7fd133d233fcb1e6658f13464ab9c13931828cc792d4` |
| Unreal selected | **15 / 22,090 / 18,637 / 811,729 / 0 / 0** | `142abd29afa8732338f11eb8702114155998d8b41bdc38ae54bb1731317041a0` |
| Fyrox selected | **3 / 1,086 / 973 / 38,928 / 0 / 0** | `199158b25d000c438097e1c075d78eda022ddd9f907106c4d55ea75abcadde86` |
| Godot selected | **12 / 3,045 / 2,501 / 125,699 / 0 / 0** | `40286ec40ee90f757395463e3c00001f8bcad606c139ba72670ad1c3eb9ae8d0` |
| Bevy selected | **2 / 686 / 628 / 27,172 / 0 / 0** | `71cabe0f237cee03277d9119be9a31eec49310cb5555e18a272592e90576277f` |
| Unity Graphics selected | **4 / 1,434 / 1,261 / 53,219 / 0 / 0** | `beb69e8e7ec8a976c37e9b23ff7a99081b31dcbcd0718956ec06ac3c98ef8ffb` |
| 五引擎reference selected union | **36 / 28,341 / 24,000 / 1,056,747 / 0 / 0** | `8c703e7f39cd4a480473a59d8a9dbe2e9e3bc1ac57e7157c0646f1383082b58f` |

选择规则：Editor/plugin为`zircon_plugins/navigation/editor/**`加`plugin.toml`；Workbench为assets入口、extension index、Navmesh AI workspace、template binding、navigation spec、feedback和preview action共7文件；Editor shared再加入first-party catalog、App接线、asset/editing/jobs/extension/runtime-event、scene viewport与通用Workbench dispatch；Runtime downstream为Navigation plugin runtime/native、framework navigation、builtin navigation与focused asset test，不重复vendored之外的无关Runtime。参考集包含Unreal NavigationSystem/World Partition及4份真实query test、Fyrox navmesh mode/commands/settings、Godot region/link/obstacle/navigation mesh、Bevy plugin/schedule边界和Unity Graphics debug UI consumer/test边界。

### 2.2 currentness与限制

- baseline与verification HEAD均为`8ee9411db24b7b4bdaf3fe028194642a7557c0b6`，commit time为`2026-08-25T17:37:22+08:00`。
- Runtime141 verification基线后，Navigation相关可见变化集中在2个Workbench文件、2个Navigation Editor文件、7个plugin runtime文件和6个builtin navigation文件。它们改进selected payload、overlay clone、task-plan sharing、dispatch完成状态、world projection和局部查询/索引性能，但没有增加产品controller、toolkit、toggle/filter/query executor或pure Bake backend。
- 共享选择集包含用户或其他Session的在途修改与未跟踪文件；本轮读取现状，不回退、不覆盖，也不把在途代码写成已集成能力。
- 按用户要求未查询、轮询或等待协调器；Tooling不在本轮范围。
- 本轮只做源码review与文档记录，未运行Cargo、App/Editor、Recast native build、真实asset create/import/cook、save/reopen、PIE、export、fault、scale、soak、profiling或竞争benchmark。
- 参考revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`；Unreal tree没有独立Git元数据，以15文件fingerprint冻结。

### 2.3 Owner边界

- Editor95唯一负责Navigation Settings/NavMesh toolkit、scene projection、六类component Inspector/gizmo、Bake UI与job承接、artifact commit/undo、query testing、per-viewport debug state和Workbench收敛。
- [Runtime141](../zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md)唯一负责provider选择、per-World owner、Recast/Detour/Crowd/TileCache、source geometry、tile artifact、query scheduler、movement intent、streaming、native failure与Runtime性能资格。
- [Editor02](02-document-transaction-save-autosave-recovery-review.md)、[Editor04](04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md)、[Editor09](09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md)、[Editor47](47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md)分别持有共享document、asset、job和runtime gateway authority；Navigation必须消费它们，不复制第二套。
- 5份Navigation failure record保持其原owner与Open状态；本报告记录current-source进展，不越权写`fixed-*`或关闭managed gate。

## 3. 当前产品链事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| App/Catalog | 默认App feature链接builtin Navigation，并可按project selection投影first-party runtime/editor package与typed consumer | 真实装配底座 |
| Editor registration | 4 surface、5 template、5 customization、5 operation、2 toolkit augmentation、1 provider、1 consumer | Descriptor foundation |
| Plugin ZUI | 11文件、455行、55 nodes、12 `Space`、2个显式event table、5处route、3个Button | 11个业务占位，Open |
| Asset toolkit | NavMesh与Settings只有空`Space`；Open命令实际发送`OpenAssetBrowser` | Open |
| Scene customization | Runtime有六类component，Editor只注册Surface/Modifier/Agent/Obstacle/OffMeshLink五类空drawer；Bridge缺失 | Open |
| Bake retained model | stable row identity、selection失效清理、typed selected payload和monotonic progress模型存在 | 局部foundation |
| Bake product host | controller无production构造者，surface/progress/diagnostic无provider，Bake Scene不消费checkbox | Open |
| Runtime operation | 3个edit operation有factory；command固定16次`yield_now`，progress V2/result V1，post-submit错误标Applied | Open |
| Runtime Bake handler | Scene/Surface prepare/apply固定失败，focused runtime test仍期待成功 | P0 Open |
| Overlay frame | Runtime typed frame、demand capture、PIE mirror/provider和session/generation拒旧存在 | 局部foundation |
| Overlay control | toggle无executor，4 checkbox无event，provider固定Default、只读PIE mirror并使用selection/0作owner | Open |
| Workbench | 27 controls、19 routes全部投影sample常量；Rebuild/Query只写固定queued feedback | P0 Open |
| Evidence | plugin 30个unit test覆盖registration/model/mock/provider；不启动默认产品，不执行真实Bake/asset/query workflow | Open |

## 4. 必须保留的真实底座

1. 保留framework中立Navigation DTO、operation ID、typed report与manager/service边界，但identity、owner、generation、deadline、cancel和budget必须补全。
2. 保留Runtime141认定的Recast/Detour/Crowd/TileCache与Rust/C++ RAII桥，将其收敛为唯一persistent per-World owner，不另写Editor算法。
3. 保留selected-surface的stable entity投影、selection失效清理和typed payload，接入真实SceneProjection而不是退回row index或默认实体。
4. 保留V2 progress消费和snapshot restore语义，但改成job continuation与content-addressed artifact ref，删除UI线程固定poll和大snapshot历史。
5. 保留typed overlay frame、PIE session/sequence/owner-generation拒旧和provider factory；进一步拆为static generation page与bounded dynamic delta。
6. 保留共享Editor document、transaction、job、extension、asset toolkit、runtime consumer与viewport registry，Navigation只能成为这些authority的领域consumer。
7. 保留现有ZUI control ID作为迁移输入；controller未就绪时隐藏或显示typed Unavailable，不能继续以空`Space`或固定成功反馈表示能力。
8. 保持Navigation为Beta/Partial；G01-G32全部通过前不得提升成熟度或宣称优于Unreal。

## 5. P0：产品真实性与纵向闭环阻断

| ID | 状态 | 当前问题 | 必须重构为 |
|---|---|---|---|
| NAVED-P0-01 | Open | Runtime Bake Scene/Surface的`prepare/apply`固定失败，而focused runtime test要求成功 | immutable source snapshot进入pure bounded prepare，owner-thread验证generation并原子apply；生产与测试共享唯一backend |
| NAVED-P0-02 | Open | 11个业务`Space`发布资产、component和authoring能力，另有1个layout filler | 可见区域必须有真实document/controller/provider及loading/empty/error/read-only状态；未实现能力整面fail-close |
| NAVED-P0-03 | Open | Bake panel/controller、surface projection、job、artifact、scene reference、undo/recovery彼此断开 | 单一`NavigationAuthoringGateway`编排document、job、operation与artifact transaction，删除测试专用第二提交链 |
| NAVED-P0-04 | Open | Toggle/filter/query preview没有可执行控制面，provider固定Default且只消费PIE mirror | per-viewport state驱动toggle/filter/query ticket，绑定document/world/backend generation、预算和terminal receipt |
| NAVED-P0-05 | Open | Navmesh AI Workbench用固定对象、tile、agent、query和queued反馈冒充产品 | 改为真实Navigation document/runtime/job projection，或从production入口删除并标明不可用 |

## 6. P1：工程级完整性差距

### 6.1 装配、能力与生命周期

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| NAVED-P1-01 | Open | Editor capability不表达Runtime backend、Bake/query/debug精度与degraded原因 | 消费machine-readable runtime readiness/capability report，按backend和target决定可见性 |
| NAVED-P1-02 | Open | registration成功只证明descriptor存在 | materialization前验证resource、controller、factory、provider、event producer与toolkit closure |
| NAVED-P1-03 | Open | project selection测试停在manifest/provider projection | 增加默认Editor启动、项目启停、NavMesh/Settings双击、disable与wrong-target产品测试 |
| NAVED-P1-04 | Open | plugin disable/unload没有领域drain证据 | revoke admission后cancel Bake/query、清mirror/provider、拒绝late commit再卸载资源 |
| NAVED-P1-05 | Open | Editor直接依赖具体runtime crate的live类型 | DTO可共享，live service/job/capability经稳定runtime interface与lease访问 |
| NAVED-P1-06 | Open | Runtime OffMeshBridge没有Editor customization | 增加lane、capacity、direction、group和diagnostic的typed Inspector/gizmo |
| NAVED-P1-07 | Open | surface/drawer/toolkit没有readiness metadata | materialization返回`NoController/NoDataProvider/RuntimeUnavailable/ReadOnly`等typed状态 |
| NAVED-P1-08 | Open | operation payload schema仍是字符串ID | 建立versioned typed decoder、size/depth/items预算、capability与executor admission合同 |
| NAVED-P1-09 | Open | Editor session不绑定document/world/plugin generation | scene切换、PIE、runtime replacement和reload必须使旧selection/result失效 |
| NAVED-P1-10 | Open | 30个plugin test主要验证ID、model和mock | 增加真实host materialization、route、gateway、provider lifecycle与project close测试 |

### 6.2 NavMesh与Navigation Settings资产

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| NAVED-P1-11 | Open | NavMeshAsset仍是弱自描述DTO | 增加source provenance、backend/version/platform、content hash、compression、endianness和tile完整性 |
| NAVED-P1-12 | Open | Settings只覆盖agents/areas的局部合同 | 统一voxel、tile、partition、query、crowd、streaming、debug defaults与backend override |
| NAVED-P1-13 | Open | agent profile以可变字符串identity引用 | 使用stable ID、display name、rename mapping和引用迁移 |
| NAVED-P1-14 | Open | area使用裸`u8`且无删除/复用策略 | reserved/tombstone、引用分析、mask迁移、颜色、cost/filter版本化 |
| NAVED-P1-15 | Open | 两类asset toolkit主体都是单个`Space` | 实现typed fields、diff、validation、references、preview、save/reload和migration diagnostics |
| NAVED-P1-16 | Open | NavMesh toolkit没有tile/topology/area/link浏览器 | 显示bounds、poly/tile、build time、memory、island/bad edge/clearance与source回跳 |
| NAVED-P1-17 | Open | Settings没有agent/area表事务 | 新增、复制、排序、rename、删除、mask/cost编辑可撤销并显示受影响引用 |
| NAVED-P1-18 | Open | `output_asset`与相关引用仍可退化为字符串 | 使用typed locator/reservation并验证kind、project、collision和write permission |
| NAVED-P1-19 | Open | Bake结果没有staging artifact与原子commit | fail/cancel/crash/stale不覆盖last-known-good，commit后才改变dirty/history |
| NAVED-P1-20 | Open | 没有DDC/reimport/invalidation产品视图 | 显示source/settings/backend key、cache hit/miss、dependency invalidation与artifact reuse |

### 6.3 Scene component、Inspector与Gizmo

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| NAVED-P1-21 | Open | NavMeshSurface drawer为空 | agent、collect mode、geometry/layer、volume、voxel/tile、region和output使用shared schema |
| NAVED-P1-22 | Open | Surface Volume没有viewport handle | bounds gizmo、local/world transform、snap、multi-select和drag transaction |
| NAVED-P1-23 | Open | Modifier没有shape与影响tile预览 | area replace、agent mask、children、link generation和dirty tile范围可视化 |
| NAVED-P1-24 | Open | Agent movement/query字段无统一Inspector | radius/height/speed/priority/mask/repath/link/writeback/destination复用runtime validator |
| NAVED-P1-25 | Open | Obstacle没有Box/Capsule handles与carve反馈 | 编辑移动阈值、stationary timer、avoidance/carve policy并显示受影响tiles |
| NAVED-P1-26 | Open | OffMeshLink没有endpoint pick/snap/arc gizmo | start/end owner、local point、width、area、cost、direction和traversal mode可撤销 |
| NAVED-P1-27 | Open | OffMeshBridge lane/capacity authoring完全缺失 | lane generation、shared capacity group、方向和冲突诊断进入Inspector与viewport |
| NAVED-P1-28 | Open | DesiredVelocity/debug资源没有只读边界 | 区分runtime feedback与authoring source，瞬态数据不得保存进scene |
| NAVED-P1-29 | Open | Bake surface table没有SceneProjection producer | 按document增量投影stable entity、label、agent、output、dirty/status并响应undo |
| NAVED-P1-30 | Open | component edit没有共享semantic validation | Inspector、serialization、Bake和Runtime共享finite/range/reference/capability规则 |

### 6.4 Bake、Job、Undo与Artifact

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| NAVED-P1-31 | Open | Bake Scene/Surface缺pure prepare实现 | immutable snapshot在bounded worker产出staging artifact，apply只做短generation-check commit |
| NAVED-P1-32 | Open | command同步执行最多16次`yield_now` | 改为JobId驱动的异步transaction continuation和wake/terminal通知 |
| NAVED-P1-33 | Open | 没有cancel、pause、priority、deadline或shutdown fence | 接入Editor09 scheduler，定义取消延迟、资源预算与project close协议 |
| NAVED-P1-34 | Open | `NavigationBakePanelController`是孤立第二套backend模型 | 收敛到单一authoring session/gateway，或明确限制为测试fixture并移出产品合同 |
| NAVED-P1-35 | Open | selected payload有了，但surface rows无生产owner | 从当前scene/document generation持续投影，禁止测试手工rows作为完成证据 |
| NAVED-P1-36 | Open | diagnostics/progress/status控件没有terminal source | 展示phase、tile、geometry、warning/error、elapsed、memory、cache及可定位owner |
| NAVED-P1-37 | Open | Bake Scene route不携带Force Full Rebuild checkbox | Scene/Selected使用同一typed settings snapshot并记录effective settings |
| NAVED-P1-38 | Open | Undo保留完整before/after generated snapshot | 保存content-addressed artifact refs、generation与commit metadata，避免双份大对象 |
| NAVED-P1-39 | Open | Clear只改变runtime generated snapshot | 分离clear generated data、unlink scene asset和delete asset三类transaction |
| NAVED-P1-40 | Open | Bake缺source/document revision冲突处理 | geometry/settings变化、surface删除、scene切换和undo时cancel、rebase或拒绝commit |

### 6.5 Query、测试与预览工作流

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| NAVED-P1-41 | Open | Editor没有真实path query operation | start/end/filter/agent/generation生成typed query ticket和terminal result |
| NAVED-P1-42 | Open | 没有query start/end viewport handles | 参考Unreal NavigationTestingActor实现pick、drag、auto rerun和result lifecycle |
| NAVED-P1-43 | Open | sample/raycast/distance/filter/cost query不可视 | 覆盖Runtime公开query surface并显示输入、预算、结果与失败原因 |
| NAVED-P1-44 | Open | agent/area下拉使用固定Humanoid等样例 | 从Settings asset与backend capability生成选项，保留stable identity |
| NAVED-P1-45 | Open | 没有tile/area/cost/build-time热图 | 绑定真实generation、dirty reason、quality metric和viewport selection |
| NAVED-P1-46 | Open | 没有topology检查或修复模式 | 至少提供只读诊断/source定位；若可编辑，使用stable selection和可逆command |
| NAVED-P1-47 | Open | 没有multi-agent profile差异预览 | 并排或叠加radius/height/slope/climb变化导致的可达性差异 |
| NAVED-P1-48 | Open | partial/no-path/out-of-nodes不可解释 | 显示visited nodes、cost、filter rejection、最近失败边界和truncation |
| NAVED-P1-49 | Open | preview不绑定world/document/backend generation | 新Bake、reload或runtime replacement立即使旧query不可见 |
| NAVED-P1-50 | Open | 缺离线导航质量分析 | island、unreachable、clearance、stairs/slope、link、area coverage与golden path批量报告 |

### 6.6 Overlay、PIE、Workbench与测试真相

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| NAVED-P1-51 | Open | Toggle Navigation Gizmos descriptor没有executor | 路由到shared per-viewport overlay view state，支持capability revoke和持久化 |
| NAVED-P1-52 | Open | 四个debug checkbox没有任何event | 驱动provider options并从effective state反向投影checked状态 |
| NAVED-P1-53 | Open | provider只消费PIE mirror | 编辑态从当前document/runtime generation取得静态NavMesh snapshot |
| NAVED-P1-54 | Open | provider固定`NavigationOverlayOptions::default()` | per-view agent/area/tile/filter、opacity/depth/picking与profile |
| NAVED-P1-55 | Open | enabled capture仍可每tick构造完整triangle/link frame | static generation pages加bounded dynamic agent delta，stable数据不逐帧复制 |
| NAVED-P1-56 | Open | provider用selected entity或`0`作overlay owner | 使用明确NavWorld/NavMesh generation identity，selection只影响highlight/filter |
| NAVED-P1-57 | Open | overlay没有frustum/tile/LOD和command/vertex预算 | viewport culling、tile residency、drop policy与CPU/GPU telemetry |
| NAVED-P1-58 | Open | mirror缺age/drop/backpressure telemetry | 增加queue age、dropped frames、payload bytes、consumer lag与shutdown counters |
| NAVED-P1-59 | Open | Navmesh AI Workbench与plugin toolkit形成双authority | 改成同一Navigation document projection，或删除该生产模块 |
| NAVED-P1-60 | Open | unit/mock/source-shape test不能证明产品成功，且Bake test与handler冲突 | 默认host点击到runtime Bake/artifact/query/overlay的动态闭环必须成为验收门 |

## 7. P2：高级产品与竞争能力差距

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| NAVED-P2-01 | Open | 缺World Partition、Navigation Invoker与tile streaming authoring | 区域、优先级、residency、生成半径和stream state可视化 |
| NAVED-P2-02 | Open | 缺hierarchical path和多层图调试 | cluster/portal、coarse/fine path、fallback与cost error展示 |
| NAVED-P2-03 | Open | 缺Smart Link gameplay authoring | 条件、reservation、animation/root motion、network authority和terminal outcome |
| NAVED-P2-04 | Open | 缺Query Filter asset与semantic cost editor | mask、enter/travel cost、tag条件和profile继承独立版本化 |
| NAVED-P2-05 | Open | 缺Crowd density、velocity、deadlock和LOD分析 | 真实规模snapshot、heatmap、budget/drop和可重放case |
| NAVED-P2-06 | Open | 缺协作编辑、merge与Bake provenance | stable ID、semantic conflict、derived artifact lineage和审计记录 |
| NAVED-P2-07 | Open | 缺multi-PIE/world/session并行观察 | 每viewport/session绑定独立world/generation，禁止单mirror覆盖 |
| NAVED-P2-08 | Open | 缺server determinism与record/replay | 重放query/agent/link决策并比较client/server generation与结果 |
| NAVED-P2-09 | Open | 缺2D Navigation authoring | 独立region/polygon/link/obstacle/bake/query workflow，不复用3D空壳 |
| NAVED-P2-10 | Open | 缺vehicle/flying/swimming/climbing等locomotion domain | 按backend/domain路由profile、source、query与segment preview |
| NAVED-P2-11 | Open | 缺Navigation质量回归数据库 | 保存scene、settings、golden result、tolerance、performance与截图用于bisect |
| NAVED-P2-12 | Open | 缺同质量参考引擎性能对照 | 冻结geometry/Recast参数/agent/query规模和质量容差后再比较 |

## 8. 历史结论与failure重判

| 来源 | 当前重判 | 依据 |
|---|---|---|
| Editor19 | 5 P0 / 60 P1 / 12 P2全部Open | selected payload、V2 progress、provider/frame是局部基础，未闭合任何完整finding的验收条件 |
| Runtime141 | Editor currentness继续有效 | Runtime报告同样确认Bake固定失败、业务`Space`、无产品controller、固定Workbench与toggle/filter断路 |
| selected-surface arguments | Open，有局部前进 | ZUI/model已投影stable entity，但surface row producer、真实host route和managed产品门仍缺 |
| runtime fallback hotpath | Open，Runtime-owned | Arc/task/index等局部优化未完成单一provider、per-World snapshot、bounded query/crowd合同 |
| world-scan deserialize | Open record，源码已前进 | 当前projection已采用可编译typed读取方向，但没有本轮managed gate与`fixed-*`回传 |
| overlay publication | Open，有显著局部前进 | typed runtime frame、mirror和provider存在；toggle/filter/edit source/budget/managed gate仍缺 |
| operation status V2 | Open record，源码已前进 | production command和mock已消费V2；未运行其声明managed package gate，也没有fixed return |

不能因为failure原始现象部分过时就把整个记录改成Closed。正确做法是由原owner在声明验收命令通过后写同生命周期键的`fixed-*`，同时保留本报告发现的剩余产品差距。

## 9. 参考实现差异

| 参考 | 本轮读取的工程基线 | Zircon当前差距 |
|---|---|---|
| Unreal NavigationSystem | World-owned NavigationSystem、Octree/dirty-area聚合、active tile/invoker、Recast tile build与大量stats、NavMeshRendering细分detail、NavigationTestingActor、World Partition builder | 无唯一Editor/Runtime owner闭环、无真实dirty/build job与artifact UI、无TestingActor级query workflow、无完整debug/detail/budget |
| Unreal NavigationTestSuite | 真正构造NavMesh world，覆盖same-point/unreachable/partial/filter、move-along-surface、length/cost、raycast和area cost | Zircon Editor测试不启动默认产品，不通过真实Bake产物执行query corpus，当前还与handler行为冲突 |
| Fyrox Editor | navmesh独立interaction mode、vertex/edge selection、move gizmo、edge duplication/connect/add/delete command group与revert | Zircon五个component drawer为空，没有topology selection、可逆修复和viewport interaction |
| Godot Navigation 3D | Region多选Bake/clear与状态，Link/Obstacle handles和UndoRedo，NavigationMesh source geometry/bake lifecycle | Zircon只有selected payload/model基础，没有真实scene producer、gizmo、job、asset commit和2D/3D完整产品链 |
| Bevy | plugin有build/ready/finish/cleanup生命周期，main schedule明确阶段边界；本地checkout不提供游戏NavMesh Editor | 只借鉴activation/schedule边界，不把Bevy作为Navigation authoring完成基线 |
| Unity Graphics | DebugManager显式register/unregister panel/data、dirty/refresh/reset与runtime UI状态，并有Editor/Runtime tests；该checkout不拥有Navigation | 仅作为debug consumer lifecycle参考，不能从Graphics源码推断Unity Navigation能力 |

## 10. 目标架构

```text
ProjectPluginSelection + Runtime Capability Snapshot
  -> NavigationActivationPlan
     -> Runtime NavigationWorld lease (Runtime141 owner)
     -> NavigationAuthoringDocument
        -> NavigationSettingsDocument / NavMeshArtifactDocument
        -> NavigationSceneProjection
           -> Surface / Modifier / Agent / Obstacle / Link / Bridge view models
        -> NavigationAuthoringGateway
           -> BakeSourceSnapshot
           -> Bounded NavigationBakeJob
           -> PreparedNavMeshArtifact
           -> generation-checked atomic commit
           -> content-addressed undo/redo refs
        -> NavigationQueryPreviewSession
           -> typed query ticket/result/diagnostic
        -> NavigationObservationSession
           -> static generation pages
           -> bounded dynamic PIE deltas
           -> per-viewport filter/culling/budget state
  -> truthful Asset / Inspector / Viewport / Workbench projections
```

架构原则：Editor持有authoring/document/interaction，Runtime持有world/backend/execution；Bake prepare只读immutable snapshot，apply只在owner线程短提交；资产、scene reference、history和runtime install通过同一transaction receipt；所有preview和overlay都携带project/document/world/backend/artifact generation；未ready能力整面fail-close。

## 11. 必须硬切的旧实现

1. 修复或移除当前固定失败的Bake handler，并同步改写冲突测试；禁止mock gateway制造production不可能成功的结果。
2. 收敛`NavigationBakePanelController`和operation/job链，只保留一个产品submission/session authority。
3. 11个业务`Space`在controller/data provider就绪前隐藏或显示typed Unavailable；禁止以空白页面声明Beta authoring完成。
4. Navmesh AI Workbench必须接同一document/runtime authority或从production删除；禁止保留固定成功计数和第二套可编辑状态。
5. Toggle/filter/query必须成为typed、generation-bound、per-viewport命令；禁止route字符串特判和全局隐式状态。
6. full-frame overlay迁移为static generation page与bounded delta；禁止让稳定三角形在每tick无界复制。
7. `output_asset`、agent/area/link/bridge引用迁移到stable typed identity；禁止entity `0`、首行、字符串猜测和silent fallback。
8. failure只按原owner的managed evidence关闭；禁止把source局部前进批量改写成产品通过。

## 12. 分层里程碑

### M0：能力真相、唯一owner与失败合同

- 建立runtime/editor/resource/controller/provider/executor闭包矩阵；不可执行入口整面Unavailable。
- 解决Bake handler/test矛盾，确定唯一Bake session/job/artifact authority。
- 逐项重验5份failure，分别记录source progress、managed blocker和真实terminal状态。

退出门：默认Editor不显示无executor命令；真实Gateway Bake返回与生产handler一致的typed terminal结果。

### M1：Navigation Document、Settings与NavMesh toolkit

- 建立transactional source document、stable agent/area ID、typed locator和shared validator。
- 完成Settings/NavMesh toolkit、diff/reference/migration/save/reload/recovery与artifact inspector。

退出门：create/open/edit/save/reload/undo/recovery无数据丢失，rename/delete有引用诊断。

### M2：SceneProjection、Inspector与Gizmo

- 为六类component实现typed projection、drawer、multi-edit和field diagnostic。
- 实现Surface/Modifier/Obstacle/Link/Bridge handles、picking、snap、commit/cancel与undo。

退出门：scene增删改/undo实时更新surface table；Inspector与gizmo提交同一command。

### M3：真实Geometry、Cancelable Bake与Prepare/Apply

- 消费Runtime141的canonical geometry/backend，不在Editor重建NavMesh算法。
- snapshot、bounded worker、cancel/deadline/progress、generation-check与atomic apply接Editor09。

退出门：cancel/fail/stale/shutdown不污染last-known-good，UI线程无busy poll。

### M4：Artifact、Commit、Undo与DDC

- content-addressed staging/final artifact绑定source/settings/backend key和依赖generation。
- history保存artifact ref与commit metadata，分离clear/unlink/delete语义。

退出门：save/reopen/reimport/cache/undo/redo维持相同provenance且无大snapshot复制。

### M5：Query Preview与Navigation Testing

- 实现start/end gizmo、agent/filter选择、path/sample/raycast/distance/cost与失败步骤。
- 实现multi-profile差异、tile/area/cost heatmap和offline quality validator。

退出门：success/partial/no-path/out-of-nodes均与Runtime一致并绑定当前generation。

### M6：Bounded Overlay与PIE Debug

- 接通per-viewport toggle/filter，提供编辑态静态source与PIE dynamic source。
- static pages、dynamic deltas、culling/LOD/budget/drop/age telemetry和shutdown cleanup。

退出门：hidden/stable不重建；1M triangle有明确预算；PIE end/reload/scene switch下一帧清stale。

### M7：Workbench收敛与产品工作流

- 删除固定fixture或改为真实document/job/runtime projection。
- 统一Asset Browser、Inspector、World menu、Job Center、Output/Notification和viewport导航。

退出门：19条可见route全部有真实domain effect或明确Unavailable，无固定queued/计数反馈。

### M8：Large World、质量与性能资格

- 加入partition/invoker/hierarchical/smart-link/multi-domain/record-replay等P2能力。
- 建立Windows优先的correctness、fault、soak、profile和同质量参考对照。

退出门：完整产品矩阵与规模证据通过后，才评估Beta/Partial晋级或“优于Unreal”。

## 13. G01-G32 Editor资格门

| Gate | 状态 | 通过条件 |
|---|---|---|
| G01 | Fail | 默认Editor按project manifest原子装配runtime/editor/resources/provider/controller，disable后所有入口降级 |
| G02 | Fail | NavMesh/Settings create/open/edit/save/reload/reimport/recover全链通过 |
| G03 | Fail | 六类component drawer读取真实selection并提交可撤销typed edit |
| G04 | Fail | surface table在spawn/delete/rename/undo/scene switch后无stale identity |
| G05 | Fail | Bake Scene/Selected/Clear/cancel/undo/redo经真实Gateway和Runtime handler完成 |
| G06 | Fail | Bake期间source/settings/surface/scene/project变化得到cancel或stale reject |
| G07 | Fail | staging artifact验证后原子commit，fail/crash保留last-known-good |
| G08 | Fail | 所有可见command/view都有executor/readiness，业务`Space`与固定成功feedback为0 |
| G09 | Fail | agent/area stable ID rename/delete/reorder和mask/reference migration有golden test |
| G10 | Fail | stairs/slope/clearance/ceiling/multi-floor/terrain/concave/modifier corpus通过 |
| G11 | Fail | render/collider source、layer/volume/hierarchy collect和empty source行为可解释 |
| G12 | Fail | Link/Bridge direction/lane/capacity/manual/automatic/disabled/invalid endpoint一致 |
| G13 | Fail | path/sample/raycast/distance/filter/cost各终态与Runtime一致 |
| G14 | Fail | corrupt version/hash/tile/index/area/link/NaN/wrong-kind/cross-platform artifact被拒绝 |
| G15 | Fail | panic/native failure/OOM budget/queue reject/cancel race/late apply返回typed diagnostic |
| G16 | Fail | 1k Bake/clear/restore后history/recovery/artifact GC无泄漏或错误引用 |
| G17 | Fail | per-viewport toggle和四类filter由真实UI event改变provider output并可恢复 |
| G18 | Fail | 编辑态NavMesh、PIE agent/path/vector与query preview可独立/组合显示 |
| G19 | Fail | world/document/owner generation变化后旧overlay/query下一帧不可见 |
| G20 | Fail | disable、PIE end、runtime crash/restart和viewport close释放reader/mirror/provider |
| G21 | Fail | Surface/Modifier/Obstacle/Link/Bridge handles支持pick/drag/snap/cancel/multi-select/undo |
| G22 | Fail | tile/area/cost/build-time/dirty heatmap显示真实数据并可回跳source |
| G23 | Fail | Navmesh AI Workbench所有action连接真实domain，或从production删除 |
| G24 | Fail | keyboard/focus/accessibility/high-DPI/multi-window/layout restore不破坏workflow |
| G25 | Fail | 1/1k/100k source mesh与tile报告snapshot/queue/build/commit/RSS/cancel p99 |
| G26 | Fail | 1/100/10k query报告wait/visits/alloc/p50/p95/p99/partial/out-of-nodes且不重建owner |
| G27 | Fail | 1/256/1k/10k agent报告mirror bytes/debug capture/drop/age/Editor frame impact |
| G28 | Fail | hidden/stable/dirty与1k/1M triangle报告rebuild/clone/culling/commands/extract p95 |
| G29 | Fail | Windows MSVC managed build/test先通过，再验证Linux/macOS和artifact ABI/endianness |
| G30 | Fail | C++ bridge通过sanitizer/fault injection，8小时Bake/query/PIE/overlay soak无增长 |
| G31 | Fail | 每个里程碑保存source fingerprint、命令、raw terminal、trace、failure和独立review |
| G32 | Fail | 与Unreal/Fyrox/Godot冻结同geometry/Recast参数/agent/query规模和质量容差后比较 |

## 14. 完成定义

Navigation Editor只有在以下条件同时成立后，才能从“默认链接的Beta骨架”提升为工程级authoring：

1. capability、view、command、provider、runtime backend、controller和toolkit readiness一致，不再发布空能力。
2. Settings/NavMesh与六类scene component都有transactional document、Inspector、gizmo、save与recovery工作流。
3. Bake使用真实immutable source、cancelable background prepare、generation-safe apply、staging artifact和atomic commit，且production/test一致。
4. Undo/Redo保存artifact reference与commit metadata，不复制无界NavMesh；clear/unlink/delete语义分离。
5. Query/testing覆盖Runtime公开能力并绑定world/document/artifact generation，所有失败可解释。
6. Overlay toggle/filter可执行，编辑态与PIE共用唯一snapshot authority，稳定静态数据不逐帧full copy。
7. 固定Navmesh AI Workbench被真实链替换或删除，所有Rebuild/Query/Edit/Commit有terminal acknowledgement。
8. correctness、fault、platform、soak和规模性能门有新鲜证据；任何“优于Unreal”结论都有同质量可复现对照。

在这些门完成前，本报告保持`implementation_status: pending`。

## 15. 本轮验证状态

- Production code：未修改。
- Dynamic validation：未执行；本轮是静态current-source review，不把现有unit test声明当运行结果。
- Static evidence：完成Navigation Editor/plugin、Workbench产品链、Runtime downstream、5份failure与36份参考选择文件的逐层读取和fingerprint冻结。
- Findings：5个P0、60个P1、12个P2编号连续；G01-G32全部Fail。
- Implementation：pending，后续从M0的能力真相、Bake合同与唯一authority开始。
