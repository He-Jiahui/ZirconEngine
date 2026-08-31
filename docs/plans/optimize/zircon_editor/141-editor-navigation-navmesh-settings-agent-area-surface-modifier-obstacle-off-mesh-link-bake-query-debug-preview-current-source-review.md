---
title: Editor Navigation / NavMesh / Settings / Agent / Area / Surface / Modifier / Obstacle / Off-Mesh Link / Bake / Query / Debug / Preview 当前源码复审
category: zircon_editor
report_id: Editor141
review_date: 2026-08-26
baseline_head: f6f2fa1141da112c1a43abb5031dbbe6dec5b69d
verification_head: b41b0c0b9da31eb4d19e3f086d6027f745f11a38
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/95-editor-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-preview-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/zircon_plugins/05-navigation.md
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
  - zircon_plugins/first_party_runtime_catalog
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
  - dev/godot/modules/navigation_3d/3d/navigation_mesh_generator.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor141 · Navigation Authoring 与 Preview 当前源码复审

## 1. 结论

Zircon Navigation 已有值得保留的工程底座，而不是空目录。默认 App feature、first-party Runtime catalog和Editor catalog可以按项目选择装配Navigation；framework有中立DTO、typed operation/report、NavMesh与NavigationSettings资产种类以及六类scene component合同；Runtime/Native包含vendored Recast、Detour、Crowd、TileCache、tiled bake、query和overlay frame；资产层能typed load两类资产，并支持NavMesh V1到V2迁移；Editor有typed runtime event consumer、PIE session/sequence/owner-generation拒旧、viewport provider、selected-surface typed payload和V2 progress消费。这些基础必须保留。

但默认可见的Editor产品仍没有闭环。Navigation Editor当前34个文件、3,901行，注册4个authoring surface、5个UI template、5个component customization、5个operation、2个asset toolkit augmentation、1个runtime consumer和1个viewport provider。11份ZUI共有455行、55个node、12个`Space`、3个Button、5条route；其中11个`Space`是Surfaces、Agents/Areas、NavMesh/Settings、五类drawer与Debug viewport的业务占位，另1个只是Bake toolbar填充。Runtime已有六类component，Editor仍漏掉OffMeshBridge customization。

Bake产品合同仍自相矛盾。Scene/Surface/Clear已有真实`NavigationOperationCommandFactory`，Bake ZUI也能发送stable `surface_entity`与`force_full_rebuild`；但`NavigationBakePanelController::new`只在测试构造，surface row、diagnostics、progress、artifact与terminal report没有产品owner。命令在调用线程内最多固定轮询16次并`yield_now`；Runtime `BakeScene/BakeSurface` handler的`prepare`固定返回`navigation bake requires a pure prepare backend`，`apply`固定返回`navigation bake cannot reach owner apply without a prepared command`，而focused test仍要求Bake成功。这不是工程级异步Bake链。

Debug链只有基础接线。Runtime typed full frame、PIE mirror、session/generation拒旧和provider均存在，当前working tree还在全部分类开启时避免了一次中间snapshot clone，并补了category filter test；但provider只消费PIE mirror，固定使用`NavigationOverlayOptions::default()`，以selection或`0`充当owner。Toggle operation没有factory/executor，四个checked checkbox没有event；编辑态静态NavMesh、query preview、per-viewport state、culling/LOD/budget与static/dynamic分层仍缺失。

内置Navmesh AI Workbench仍是第二套静态authority。230行workspace包含27个control和19条route，固定显示`NavMesh_Main`、`Agent_Humanoid`、`Query_Patrol`、`Tile 12_08/12_09`、`96 polys`、`42 cm`、`180 cm`、`Door_A03`以及`18 tiles / 4 agents / 1 blocked link`。Rebuild与Query Path只返回预写的queued feedback，没有JobId、query ticket、artifact或Runtime receipt。当前移除Rebuild按钮错误selected/checked状态只是UI修正，不是domain闭环。

Runtime同样仍有影响Editor产品真实性的边界问题：Bake geometry把render mesh/Box近似为顶面quad，把sphere/capsule/cylinder近似为12段顶面disc，把convex hull近似为AABB顶面，TriangleMesh/HeightField在该路径被遗漏，empty source还会生成synthetic surface；每次query可重新从asset构造Detour owner；Crowd固定256 agents与8的最大radius；一旦存在runtime obstacle、obstacle world或asset off-mesh link，就清空crowds并回退legacy tick。当前`Arc<RecastTiledBakePlan>`、dispatch完成竞争修复、builtin agent索引和typed tick delta是真实局部改进，但没有关闭Editor产品条目。

本轮重判Editor19/95的 **5项P0、60项P1、12项P2全部Open，32项Editor资格门全部Fail**。Runtime141继续唯一持有world/backend/native、geometry、artifact、query、Crowd、obstacle、off-mesh movement、streaming和Runtime性能差距；Editor141只持有asset/document/Inspector/gizmo/Bake控制面、query preview、debug与Workbench产品闭环。当前没有动态或竞争证据支持Zircon Navigation的功能、性能或表现优于Unreal。

## 2. 审查边界、统计与currentness

### 2.1 当前物理范围

统计对象为本轮读取的working tree物理文件。行数、非空行与bytes按当前文件内容计算；`tests/ignored`只计Rust `#[test]`和`#[ignore]`声明。该表表达审查证据范围，不把文件数量当作完成度。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Navigation Editor/plugin | **34 / 3,901 / 3,517 / 136,806 / 30 / 0** | editor crate、ZUI、manifest与focused tests |
| Navigation Workbench selected | **7 / 2,644 / 2,468 / 141,692 / 0 / 0** | workspace、入口、feedback、navigation spec、preview action与template binding |
| Editor shared/product boundary | **251 / 37,453 / 33,921 / 1,313,317 / 293 / 10** | asset/editing/job/extension/runtime event/viewport/catalog/App与Workbench dispatch |
| Runtime/navigation downstream | **194 / 51,567 / 45,879 / 1,663,426 / 171 / 7** | framework、builtin navigation、plugin runtime/native、asset与focused tests |
| Zircon selected union | **479 / 92,921 / 83,317 / 3,113,549 / 494 / 17** | 上述Zircon范围去重物理集合 |
| Unreal selected | **15 / 30,340 / 25,403 / 1,157,890 / 0 / 0** | NavigationSystem、dirty controller、Recast、render、World Partition builder与query tests |
| Fyrox selected | **4 / 1,278 / 1,146 / 45,208 / 0 / 0** | interaction、selection、commands与settings |
| Godot selected | **12 / 2,912 / 2,402 / 123,087 / 0 / 0** | Region/Link/Obstacle editor与source geometry/bake lifecycle |
| Bevy selected | **2 / 686 / 628 / 27,172 / 0 / 0** | plugin lifecycle与main schedule边界 |
| Unity Graphics selected | **4 / 979 / 850 / 33,729 / 0 / 0** | DebugManager data/panel registration与refresh lifecycle |
| 五引擎reference union | **37 / 36,195 / 30,429 / 1,387,086 / 0 / 0** | 上述参考文件去重集合 |
| Plan/docs evidence | **19 / 6,332 / 4,748 / 587,026 / 0 / 0** | Editor/Runtime/Plugin owner报告、Navigation计划与5份failure |
| 全部证据 union | **535 / 135,448 / 118,494 / 5,087,661 / 494 / 17** | Zircon、参考和文档证据去重集合 |

### 2.2 currentness与限制

- baseline HEAD为`f6f2fa1141da112c1a43abb5031dbbe6dec5b69d`（`2026-08-26T17:01:15+08:00`）；最终静态复核HEAD为`b41b0c0b9da31eb4d19e3f086d6027f745f11a38`（`2026-08-26T17:23:20+08:00`）。两者之间的可见HEAD推进是其他Session的协调文档提交，不改变本轮Navigation源码判定。
- working tree包含其他Session和用户的在途修改与未跟踪报告；本轮按可见物理内容审查，不回退、不覆盖，也不把在途源码写成已集成能力。
- 5份Navigation failure handoff当前仍为Open；局部源码前进不能代替原owner的managed validation和`fixed-*`回传。
- 按用户要求未查询、轮询、等待或实时跟踪协调器；Tooling不在本轮范围。
- 本轮只写review与索引，没有运行Cargo、默认Editor、Recast native build、真实asset create/import/cook、save/reopen、PIE、export、fault、scale、soak、profiling或竞争benchmark。

### 2.3 Owner边界

- Editor141唯一负责Navigation Settings/NavMesh toolkit、scene projection、六类component Inspector/gizmo、Bake UI/job承接、artifact commit/undo、query testing、per-viewport debug state和Workbench收敛。
- Runtime141唯一负责provider选择、per-World owner、Recast/Detour/Crowd/TileCache、source geometry、tile artifact、query scheduler、movement intent、streaming、native failure与Runtime性能资格。
- `zircon_app`负责project-selected runtime/editor package的host装配与生命周期；`zircon_plugins`负责Navigation package、native distribution和capability声明，不承接第二套Editor document或Runtime world。
- Editor02/04/09/47分别持有共享document、asset、job和runtime gateway authority；Navigation必须消费这些边界，不复制领域私有替代品。

## 3. 当前产品链事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| App/Catalog | 默认App与first-party runtime/editor catalog可以按project selection装配Navigation | 真实装配底座 |
| Editor registration | 4 surface、5 template、5 customization、5 operation、2 toolkit augmentation、1 provider、1 consumer | Descriptor foundation |
| Plugin ZUI | 11文件、455行、55 nodes、12 `Space`、3 Button、5 routes | 11个业务占位，Open |
| Asset toolkit | NavMesh/Settings augmentation存在，但Open command路由`OpenAssetBrowser`，两份toolkit主体仍为`Space` | Open |
| Scene customization | Runtime六类component；Editor只注册Surface/Modifier/Agent/Obstacle/OffMeshLink五类空drawer | Open |
| Bake control | Scene/Surface/Clear有factory，selected payload与V2 progress model存在 | 局部foundation |
| Bake product | controller仅测试构造，固定16次同步poll；Runtime prepare/apply固定失败 | P0 Open |
| Asset pipeline | NavMesh/Settings typed registry/load存在，NavMesh V1到V2 migration存在 | 真实底座 |
| Geometry | 多类source被顶面quad/disc/AABB或synthetic fallback近似，部分类型遗漏 | Runtime owner阻断 |
| Query/Crowd | query可逐次重建native owner；obstacle/link条件下Crowd清空并回退legacy | Runtime owner阻断 |
| Overlay | typed frame、PIE mirror/provider和拒旧存在；toggle/filter/edit source/budget缺失 | 局部foundation |
| Workbench | 27 controls、19 routes投影固定fixture，Rebuild/Query只写queued feedback | P0 Open |

## 4. 必须保留的真实底座

1. 保留framework中立Navigation DTO、typed operation/report、asset kinds和manager/service边界，继续补齐identity、owner、generation、deadline、cancel与budget。
2. 保留Recast/Detour/Crowd/TileCache和Rust/C++ RAII桥，收敛为唯一persistent per-World owner，不在Editor复制算法或缓存。
3. 保留NavMesh/NavigationSettings typed asset load与V1到V2迁移，向provenance、hash、platform/backend与atomic artifact commit扩展。
4. 保留selected-surface stable entity、selection失效清理与typed payload，接入真实SceneProjection，不退回row index、首项或默认entity。
5. 保留V2 progress和snapshot restore语义，改成job continuation与content-addressed artifact ref，删除UI线程固定poll与大snapshot历史。
6. 保留typed overlay frame、PIE session/sequence/owner-generation拒旧和provider factory，拆成static generation page与bounded dynamic delta。
7. 保留共享Editor document、transaction、job、extension、asset toolkit、runtime consumer与viewport registry；Navigation只做领域consumer。
8. 保留现有ZUI control ID作为迁移输入；controller未就绪时隐藏或显示typed Unavailable，不能以空`Space`或固定成功反馈表示能力。

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
| Editor19/95 | 5 P0 / 60 P1 / 12 P2全部Open | selected payload、V2 progress、typed asset、provider/frame与局部性能修正没有闭合完整finding |
| Runtime141 | Editor currentness继续有效 | Bake固定失败、geometry近似、query owner重建、Crowd fallback与debug full frame仍影响产品真实性 |
| Plugins14/Navigation计划 | package基础保留，产品门未通过 | manifest、catalog和option声明存在，但`navigation.default_settings_asset`等缺少生产消费闭环 |
| selected-surface arguments | Open，有局部前进 | stable entity payload存在；surface row producer、真实host route和managed产品门仍缺 |
| runtime fallback hotpath | Open，Runtime-owned | Arc/task/index改进未完成单一provider、persistent per-World query owner和bounded Crowd合同 |
| world-scan deserialize | Open record，源码已前进 | typed projection方向存在，但没有本轮managed gate与`fixed-*`回传 |
| overlay publication | Open，有局部前进 | typed frame、mirror、provider和clone削减存在；toggle/filter/edit source/budget/managed gate仍缺 |
| operation status V2 | Open record，源码已前进 | production command消费V2，但没有完成真实Bake或原owner声明的managed gate |

failure原始现象部分过时不等于记录Closed。原owner必须在声明验收命令通过后写同生命周期键的`fixed-*`，本报告不越权终结它们。

## 9. 参考实现差异

| 参考 | 本轮读取的工程基线 | Zircon当前差距 |
|---|---|---|
| Unreal NavigationSystem | World-owned NavigationSystem、dirty-area controller、editor/PIE/async-load build lock、invoker/active tile、time-sliced regeneration、per-tile wait/build stats | Zircon没有闭合唯一world lease、dirty/build job、artifact UI、build lock和large-world控制面 |
| Unreal Recast/Rendering/World Partition | Recast generator/tile task、细分NavMesh rendering detail、builder package save/delete/notification lifecycle | Zircon Editor没有真实tile artifact browser、source定位、partition/invoker authoring与atomic package workflow |
| Unreal NavigationTestSuite | 构造真实NavMesh test world，覆盖path length/cost、partial/unreachable、projection、filter与raycast | Zircon测试不启动默认产品，也不从生产Bake artifact执行query corpus，且Bake test与handler冲突 |
| Fyrox Editor | 独立navmesh interaction mode、vertex/edge selection、move gizmo、duplicate/connect/add/delete command group和execute/revert | Zircon component drawer为空，没有topology selection、可逆修复与viewport interaction |
| Godot Navigation 3D | Region多选Bake/clear/status，Link/Obstacle handle与UndoRedo；scene parse和source geometry bake分阶段 | Zircon缺SceneProjection、gizmo、main-thread source snapshot、bounded bake与asset commit产品链 |
| Bevy | plugin build/ready/finish/cleanup生命周期与main schedule阶段排序 | 仅作activation/schedule参考；本地checkout不构成Navigation Editor完成基线 |
| Unity Graphics | DebugManager显式Register/Unregister data、Get/Remove panel、dirty/reset/refresh lifecycle | 仅作debug consumer lifecycle参考；Graphics checkout不构成Navigation authoring基线 |

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
           -> immutable BakeSourceSnapshot
           -> bounded NavigationBakeJob
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

Editor持有authoring/document/interaction，Runtime持有world/backend/execution。Bake prepare只读immutable snapshot，apply只在owner线程短提交；asset、scene reference、history与runtime install通过同一transaction receipt；所有preview/overlay携带project/document/world/backend/artifact generation；未ready能力整面fail-close。

## 11. 必须硬切的旧实现

1. 修复或移除固定失败的Bake handler并同步冲突测试；禁止mock gateway制造production不可能成功的结果。
2. 收敛`NavigationBakePanelController`、operation、job与artifact commit，只保留一个产品submission/session authority。
3. 11个业务`Space`在controller/provider就绪前隐藏或显示typed Unavailable；禁止用静态Label/Table替换后宣告完成。
4. Navmesh AI Workbench接入同一document/runtime authority或从production删除；禁止固定成功计数和第二套可编辑状态。
5. Toggle/filter/query成为typed、generation-bound、per-viewport命令；禁止route字符串特判和全局隐式状态。
6. full-frame overlay迁移为static generation page与bounded delta；禁止稳定三角形每tick无界复制。
7. `output_asset`、agent/area/link/bridge迁移到stable typed identity；禁止entity `0`、首行、字符串猜测和silent fallback。
8. Runtime geometry/query/Crowd临时fallback必须在Runtime141按quality/budget合同重构，Editor不得掩盖或复制它们。
9. failure只按原owner managed evidence关闭；禁止把局部源码前进批量改写成产品通过。

## 12. 分层里程碑

### M0：能力真相、唯一owner与失败合同

- 建立runtime/editor/resource/controller/provider/executor闭包矩阵；不可执行入口整面Unavailable。
- 解决Bake handler/test矛盾，确定唯一Bake session/job/artifact authority。
- 逐项重验5份failure，记录source progress、managed blocker与真实terminal状态。

退出门：默认Editor不显示无executor命令；真实Gateway Bake与production handler返回一致的typed terminal结果。

### M1：Navigation Document、Settings与NavMesh toolkit

- 建立transactional source document、stable agent/area ID、typed locator与shared validator。
- 完成Settings/NavMesh toolkit、diff/reference/migration/save/reload/recovery与artifact inspector。

退出门：create/open/edit/save/reload/undo/recovery无数据丢失，rename/delete有引用诊断。

### M2：SceneProjection、Inspector与Gizmo

- 为六类component实现typed projection、drawer、multi-edit与field diagnostic。
- 实现Surface/Modifier/Obstacle/Link/Bridge handles、picking、snap、commit/cancel与undo。

退出门：scene增删改/undo实时更新surface table；Inspector与gizmo提交同一command。

### M3：真实Geometry、Cancelable Bake与Prepare/Apply

- 消费Runtime141 canonical geometry/backend，不在Editor重建NavMesh算法。
- snapshot、bounded worker、cancel/deadline/progress、generation-check与atomic apply接Editor09。

退出门：cancel/fail/stale/shutdown不污染last-known-good，UI线程无busy poll。

### M4：Artifact、Commit、Undo与DDC

- content-addressed staging/final artifact绑定source/settings/backend key和依赖generation。
- history保存artifact ref与commit metadata，分离clear/unlink/delete语义。

退出门：save/reopen/reimport/cache/undo/redo维持相同provenance且无大snapshot复制。

### M5：Query Preview与Navigation Testing

- 实现start/end gizmo、agent/filter选择、path/sample/raycast/distance/cost与失败步骤。
- 实现multi-profile差异、tile/area/cost heatmap和offline quality validator。

退出门：success/partial/no-path/out-of-nodes与Runtime一致并绑定当前generation。

### M6：Bounded Overlay与PIE Debug

- 接通per-viewport toggle/filter，提供编辑态静态source与PIE dynamic source。
- 实现static pages、dynamic deltas、culling/LOD/budget/drop/age telemetry与shutdown cleanup。

退出门：hidden/stable不重建；1M triangle有明确预算；PIE end/reload/scene switch下一帧清stale。

### M7：Workbench收敛与产品工作流

- 删除固定fixture或改为真实document/job/runtime projection。
- 统一Asset Browser、Inspector、World menu、Job Center、Output/Notification与viewport导航。

退出门：19条可见route全部有真实domain effect或明确Unavailable，无固定queued/计数反馈。

### M8：Large World、质量与性能资格

- 加入partition/invoker/hierarchical/smart-link/multi-domain/record-replay等P2能力。
- 建立Windows优先的correctness、fault、soak、profile与同质量参考对照。

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

Editor141只有在以下条件同时满足时才能关闭：M0-M8按依赖顺序完成；Editor19/95的5项P0、60项P1、12项P2逐项有current-source产品证据；G01-G32全部Pass；默认Editor能transactionally创建、打开、编辑、保存和恢复Settings/NavMesh以及六类scene component；真实Bake经bounded prepare、generation-safe apply、staging artifact和atomic commit完成；query/debug/overlay/Workbench都消费唯一domain authority；Runtime141提供唯一可执行world/backend及generation receipt；correctness、fault、platform、soak和同质量规模性能证据可复现；旧`Space`、fixed feedback、busy poll、implicit owner与fallback伪成功路径已hard cutover且无兼容壳。

本轮没有修改production代码，也没有宣告Navigation、Editor或Engine总体目标完成。

## 15. 本轮验证状态

- Production code：未修改。
- Dynamic validation：未执行；本轮是静态current-source review，不把现有unit test声明当运行结果。
- Static evidence：读取Navigation Editor/plugin、Workbench、Editor shared/product boundary、Runtime downstream、资产/Bake/query/Crowd/overlay路径、5份failure和37份参考选择文件。
- Findings：5个P0、60个P1、12个P2保持连续且全部Open；G01-G32全部Fail。
- Implementation：pending；后续从M0能力真相、Bake合同和唯一authority开始。
