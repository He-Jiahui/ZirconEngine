---
title: Runtime Dynamic Scene Asset Reload、Event Generation、Reconciliation、Stage/Apply、Instance Replacement 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime53
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/scene/dynamic_scene/asset_reload
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/core/resource/event_stream.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene_asset_reload.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_asset_reload/byte_budgets.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/scene_patch_document.rs
  - zircon_runtime/src/scene/dynamic_scene/spawn_task/prepared.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/39-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-22-dynamic-scene-asset-reload-bounded-singleflight.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-dynamic-scene-compiled-spawn-transaction.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PackageReload.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/PackageReload.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BlueprintGraph/Public/Kismet2/KismetReinstanceUtilities.h
  - dev/UnrealEngine/Engine/Source/Editor/BlueprintGraph/Private/Kismet2/ReloadReinstancingBridgeImpl.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h
  - dev/bevy/crates/bevy_world_serialization/src/world_asset_spawner.rs
  - dev/godot/core/io/resource.cpp
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/editor/editor_data.cpp
  - dev/godot/editor/editor_node.cpp
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/resource/model/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/ResourceReloader.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 53 · Runtime Dynamic Scene Asset Reload、Event Generation、Reconciliation、Stage/Apply、Instance Replacement 与 Product Integration 工程化差距

## 1. 结论

Dynamic Scene asset reload并非空壳。当前14个owner文件已经实现typed `SceneAsset`事件、count/bytes/time预算、AssetId keyed single-flight、一个active worker加一个deferred successor、cooperative cancel、stale result拒绝、event-log generation gap检测、增量resource reconciliation、prepared/result resident byte上限、异步target capture/staging、target generation/change-tick CAS、主线程bounded commit、reactive wake与较完整的内部诊断。这些能力应保留，不能退回“文件变化后同步读JSON并直接改World”的临时路径。

但产品语义存在三项阻断性错误。第一，Runtime Session为任何有project的Level创建一个全项目`SceneAsset`订阅；Added、Modified、Renamed以及gap reconciliation发现的每个Ready/Reloading Scene都会被准备并spawn进当前唯一Level，没有验证该asset是否是play scene、已加载subscene或现存实例的source。一次无关场景导入或event gap即可向当前World注入项目内其他场景。第二，queue没有`AssetId -> SceneInstanceId -> EntityRemap/owned entities`登记表，Modified/Renamed继续走append-only `CompiledSceneSpawn`，Removed/ReloadFailed只产生skip；因此重复修改会复制实体，删除不会清理实例，rename不更新provenance，返回的`EntityRemap`在frame report之后被丢弃。第三，revision truth先于可靠终态发布：latest revision在locator、metadata admission和schedule前记录，ready residency超限又会静默丢结果；同revision随后会被判stale，资源代际可前进而World永久停在旧内容，产品只看到计数或甚至看不到failure。

这不是“再加几个if”可以修复的问题。目标必须是runtime-owned `DynamicSceneInstanceRegistry + SceneReloadCoordinator`：只有已实例化/显式请求的scene进入reload；每个实例保留source asset、source revision、instance epoch、owned entity map、override/provenance与last-known-good；新artifact完成prepare后生成replacement transaction，在安全点原子提交或保持旧实例；Removed、failure、retry、reconciliation和shutdown都有typed terminal disposition。通用resource hot reload仍归Runtime04，ECS/compiled spawn归Runtime05/Runtime08 failure，prefab override传播归Runtime39，operation框架归Runtime41，Session接线归Runtime43，asset registry代际归Runtime51。

本轮登记 **3项P0、60项P1、16项P2和40项验收门禁**。只做静态review与文档总账；没有修改production、tests、Cargo或reference source，没有运行Cargo、真实Editor/Runtime、fault injection、multi-Level、slow-worker soak、RSS或benchmark。Runtime04与Runtime08两份failure仍为`open`，本报告不把静态实现、queued/materializing receipt或source-shape test写成accepted milestone。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| asset reload production owner | 14 / 2,966 / 103,682 / 4 | SHA-256 `059c599adebec78ac798bed8943826cd2d0d8995cc09599ccd630b7aea768429` |
| focused direct tests | 3 / 1,498 / 54,629 / 26 | SHA-256 `06a1df00e7b123ccd7415aa0aefe5058387369012756f13f0a639eb39ce8bc19`；0 ignored |
| spawn/preflight support | 10 / 1,877 / 63,185 / 6 | SHA-256 `e381afa2c55d697ca739ab575460049026b7f64c64e3bfc96ac4e516bcf825c2` |
| product integration chain | 7 / 2,875 / 105,245 / 22 | SHA-256 `665f76a77d57e3f46b01368cc56c521a6a5e2cd273588b66f48680948370e898` |
| reference corpus | 15 / 25,705 / 998,003 | SHA-256 `1209933a1740800722af295f701545dd46bdceca0cc2f0fb401580178619b853` |

fingerprint算法与Runtime52一致：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`编码，LF连接且末尾不追加LF，再计算UTF-8 SHA-256。它只冻结本轮读取集合，不是asset content digest、BuildSet、instance epoch或release identity。

当前14个asset-reload production owner文件没有working-tree差异；focused direct tests中的`dynamic_scene_asset_reload.rs`与`byte_budgets.rs`已有并发差异，product集合中的`zircon_runtime/src/dynamic_api/session/state.rs`与spawn transaction test也正被MVP会话修改。上述current-source集合仍对应本节指纹与规模，但实施前必须重读direct tests、产品接线和transaction支持链，因此`source_recheck_required`置true。基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator epoch为335。

### 2.2 当前真实产品调用链

```text
ProjectAssetManager::subscribe_asset_events::<SceneAsset>()
  -> DynamicSceneAssetReloadQueue (全项目、无instance selector)
     -> RuntimeDynamicSession::tick_scene_asset_reload()
        -> queue.tick_into_level(scheduler, current Level)
           -> drain/reconcile -> prepare -> target stage -> append spawn
        -> WorldFact::AssetReloadApplied { applied, failed, stale, pending }
           -> SubscriptionTable::invalidate_all_assets()
```

`DynamicSceneAssetReloadAppliedScene`确实保存event、`EntityRemap`与组件/实体/资源计数，但普通产品调用者没有读取`remap()`；Session只把四个计数写入ABI DTO，再保存一帧report。故返回映射不是实例身份或引用修复机制。

### 2.3 应保留的真实底座

1. raw typed event也受count/bytes/time预算，filtered event不再成为无成本旁路；跨帧carried event保留顺序。
2. 每AssetId最多一个active prepare worker和一个logical successor；新revision请求取消旧工作，旧结果在publication前重新检查authority。
3. event log明确报告lag gap，queue取消不可信工作后以`ResourceManagementScan`增量reconcile，而不是假设事件永不丢失。
4. prepared payload、ready bytes、target-stage reservation、metadata、active task、schedule、ready与apply均有硬限制。
5. compile/preflight/stage与最终commit分离；target Level、world generation、schema/component generation和change tick发生变化时拒绝stale mutation。
6. Level capture/stage在worker执行，产品main-thread入口只做有界harvest/commit；reactive frame demand可在pending时继续驱动。
7. Runtime04 failure中记录的single-flight/预算实现候选已实质进入source，但其受管Cargo与100k性能证据仍未终态。

## 3. 参考实现裁决

| 参考 | 直接源码事实 | 对Zircon的约束 |
|---|---|---|
| Unreal package reload / Blueprint reinstancing | `FPackageReloadedEvent`携带old/new package与repoint map；reload有PreLoad、Fixup、PostFixup、batch/GC阶段；Blueprint reinstancer保存old-to-new instance/class/field mapping并修复referencer | scene reload必须拥有replacement identity、referencer fixup、阶段边界与失败后旧对象寿命，不能把registry event等价为spawn命令 |
| Bevy `WorldInstanceSpawner` | 明确维护asset->instance set、instance->entity map；Modified只在`spawned_worlds.contains_key(id)`时更新，并先despawn该实例拥有的实体再respawn | 即使选择较重的replace策略，也必须先有instance registry、owned entity map和只更新已实例化asset的选择门 |
| Godot editor scene reload | 先定位打开scene/继承实例，保存local modification、selection、history、connections、references和additional nodes，再替换对应instance并恢复状态 | authoring/runtime override、外部引用、继承/provenance和selection/history必须有明确保留策略；全局append无法满足 |
| Fyrox resource/model reload | reload成功在同一resource上发布`Reloaded`；失败且旧resource有效时保留现有版本；model instance节点带resource/inheritance data，engine对active scenes做resolve | 失败不得把旧可用内容降成无主状态；实例必须可追溯source resource并可定向传播 |
| Unity Graphics `ResourceReloader` | 只在Editor中按attribute修补显式container的null/broken字段，并区分AssetDatabase not ready | 这是显式容器资源修复，不是scene instance reload。Zircon不能用“遍历所有SceneAsset再spawn”类比该utility |

共同约束是显式owner、source identity、instance identity、old/new mapping、last-known-good、failure disposition和有界publication。Zircon可以采用增量patch而非Bevy的heavy respawn，也可以比Unreal更低成本；但不能删除这些语义来制造表面性能优势。

## 4. Owner边界与不得重复登记

| Owner | 继续拥有 | Runtime53只登记 |
|---|---|---|
| Runtime04 / Runtime51 | 通用asset event、resource state、registry generation、artifact与last-good | Scene事件到实例选择、scene source revision与world apply一致性 |
| Runtime05 / Runtime08 failure | World/ECS、DynamicScene compile/preflight/commit、entity identity底座 | replacement transaction如何消费这些底座，不复制ECS总问题 |
| Runtime24 / Runtime39 | stable identity、prefab/archetype instance override与传播 | SceneInstanceId/source provenance和reload vertical contract |
| Runtime41 | operation admission、cancel、deadline、progress、terminal retention、shutdown | reload coordinator接入，不新建私有通用任务框架 |
| Runtime43 / Interface02 | Dynamic Session、WorldFact与跨DLL DTO | Session持有reload service、精确world invalidation和产品receipt |
| Runtime53 | scene event selection、reconciliation、prepare/stage/apply、instance replacement/remove/failure | 本报告的3项P0与纵向P1/P2 |

## 5. P0：会直接破坏当前World产品事实

| ID | 差距 | 当前证据与硬切目标 |
|---|---|---|
| DSRL-P0-001 | 全项目Scene事件可向当前Level注入无关场景 | Session无条件创建全局`SceneAsset`receiver；Added/Modified/Renamed均schedule，gap reconciliation遍历所有Ready/Reloading Scene。测试还明确让两个独立scene asset都spawn进同一空World。硬切为`SceneReloadSelection`：只有registry中已实例化或显式load request绑定的asset/instance可进入prepare；reconciliation只重建这些instance的source truth。 |
| DSRL-P0-002 | Modified/Renamed append副本，Removed/ReloadFailed不回收既有实例 | `CompiledSceneSpawn`从可用ID开始构造新remap并插入records；queue没有instance registry，apply后remap未持久；Removed/ReloadFailed只skip并supersede工作。建立`SceneInstanceId -> source asset/revision/owned entities/remap/overrides`，以原子replace/patch transaction提交；remove按policy unload/quarantine，失败保留last-good。 |
| DSRL-P0-003 | revision已推进但world结果可被静默丢弃且同revision不可恢复 | `record_latest_revision`发生在locator、metadata admission与schedule之前；ready resident超限仅增加`dropped_events`后return，不进入apply failure/reconciliation。后续同revision被stale过滤，资源管理代际与World永久分叉。revision只能在terminal instance disposition发布时推进；所有drop必须产生retry/reconcile或typed terminal failure，绝不静默。 |

## 6. P1：产品Owner、Scope与Selection

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| DSRL-P1-001 | queue对任何project session默认启用 | 由compiled runtime profile/capability决定Editor hot reload、development runtime reload或shipping disabled，不以“有project”作为唯一条件。 |
| DSRL-P1-002 | queue只绑定一个当前Level | 显式记录World/Level/PIE instance identity；同一project多Level、多viewport、多session互不串扰。 |
| DSRL-P1-003 | queue没有当前play scene identity | construction已有`scene_uri`却未交给reload owner；将play scene与loaded subscene source登记到instance registry。 |
| DSRL-P1-004 | 没有显式instance registration/unregistration API | scene load/spawn成功必须返回并登记`SceneInstanceId`，unload/level destroy原子撤销。 |
| DSRL-P1-005 | 无subscene/prefab引用来源区分 | source edge区分root scene、subscene、prefab/archetype和resource-only dependency，路由到各自owner。 |
| DSRL-P1-006 | 无per-instance reload policy | 定义auto/manual/disabled、preserve overrides、restart-required和authority-only策略。 |
| DSRL-P1-007 | limits是代码内默认而非产品配置 | 通过validated profile/policy提供预算，记录effective values与来源，禁止call-site私自改出第二authority。 |
| DSRL-P1-008 | 未接Runtime41 operation合同 | 一批reload必须有OperationId、deadline、cancel、progress、terminal receipt、retention与shutdown disposition。 |
| DSRL-P1-009 | 77个public declaration/accessor暴露机制细节 | 收敛为service、instance handle、request、policy和typed outcome；内部queue/task/report保持crate-private。 |
| DSRL-P1-010 | capability readiness未表达 | provider、watch、event receiver、instance registry或apply owner不可用时返回Unavailable/Degraded，不发布“queue ready”。 |

## 7. P1：Event、Revision、Gap 与 Reconciliation

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| DSRL-P1-011 | same-revision authority rank是队列私有规则 | Added/Modified < Renamed < ReloadFailed < Removed没有上游schema保证；把source event sequence、asset generation与terminal state纳入统一version contract。 |
| DSRL-P1-012 | ReloadFailed会压制同revision恢复 | rank 2使后续同revision Modified变stale；定义attempt generation与content revision分域，成功retry可合法替换失败attempt。 |
| DSRL-P1-013 | Removed删除tombstone | `remove_latest_revision_state`允许同AssetId revision 1重建，也会失去防旧事件依据；保留带owner epoch的tombstone直到registry证明新identity。 |
| DSRL-P1-014 | latest TTL后可接受延迟旧事件 | 60秒wall-time prune不等于stream ordering fence；只在subscriber/generation checkpoint证明安全时退休revision state。 |
| DSRL-P1-015 | missing locator发生在latest推进后 | locator缺失必须是可恢复input failure，不应永久消费revision；保留retry/reconcile token与原因。 |
| DSRL-P1-016 | latest/metadata容量拒绝只生成局部skip | 容量拒绝必须保持source dirty并安排有界reconciliation，不能把世界留旧后结束。 |
| DSRL-P1-017 | deferred successor容量失败会删除旧successor | 当前先remove旧deferred再drop新event；改为admission成功后原子替换，失败保留可执行的旧successor并报告新请求未接纳。 |
| DSRL-P1-018 | receiver disconnect没有重建owner | 每帧只报告disconnected，之后可能无pending而停止wake；service需sticky health、resubscribe generation和fail-close capability。 |
| DSRL-P1-019 | reconciliation把Reloading当Modified | Resource尚处Reloading即合成Updated并读取URI；应只消费published Ready artifact，Reloading保留pending attempt而非生成scene apply。 |
| DSRL-P1-020 | event/result不绑定content/artifact digest | AssetId+revision不足以证明prepare读取的bytes与current registry一致；贯穿artifact/source digest、project generation和import generation。 |

## 8. P1：Instance、Override、Reference 与 Replacement Transaction

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| DSRL-P1-021 | 无asset-to-instances索引 | 建立`AssetId -> ordered SceneInstanceId set`，Modified只触达现有instances；Added不自动spawn。 |
| DSRL-P1-022 | 无instance-to-entities所有权 | 持久保存source entity到live entity映射及owned entity set，支持精确replace/unload与泄漏检查。 |
| DSRL-P1-023 | apply remap只活一帧 | remap进入instance generation并供reference fixup、debug、selection、network/save与diagnostic查询。 |
| DSRL-P1-024 | rename只换URI读取，不更新provenance | instance source URI、asset identity、watch key与serialized provenance必须同事务更新。 |
| DSRL-P1-025 | remove无unload/quarantine policy | 定义root scene、subscene和prefab removal行为；拒绝时保留LKG并明确stale/degraded状态。 |
| DSRL-P1-026 | reload failure无last-known-good instance state | 记录old revision仍Active、failed attempt、error、retryability和operator action，不把skip当终态。 |
| DSRL-P1-027 | 无runtime/authoring override保存策略 | 区分source-owned、instance override、runtime transient、network authoritative和save-owned字段，按policy rebase或拒绝。 |
| DSRL-P1-028 | 无外部entity/reference fixup | replacement需old-to-new map并修复scene内外引用、parent、joint、script handle、selection/watch与render/physics handles。 |
| DSRL-P1-029 | scene resources可覆盖World全局resource | DynamicScene resource write作用于target World；按resource ownership声明merge/replace/forbid，不能由任意subscene reload改全局resource。 |
| DSRL-P1-030 | 多instance/多asset提交可半完成 | 建立dependency-ordered batch plan、preflight all、atomic publish或明确per-instance independent receipt与compensation。 |

## 9. P1：Prepare、Stage、Apply、Cancel 与 Backpressure

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| DSRL-P1-031 | prepared artifact只描述spawn | 增加replacement plan：old instance generation、expected owned set、new artifact、override rebase与reference fixup。 |
| DSRL-P1-032 | target-changed失败没有重试策略 | CAS拒绝后按原因重新capture、退避、取消或terminal conflict；不能只计failed后等待另一文件事件。 |
| DSRL-P1-033 | apply time预算在首项之后检查 | 单个commit可任意超过2ms；支持可预估/可分段safe-point publication，或超预算任务转专用loading barrier。 |
| DSRL-P1-034 | 多个commit不共享原子publication | 同frame前几个asset成功、后一个失败会形成混代World；batch必须绑定world epoch和依赖一致性。 |
| DSRL-P1-035 | target stage按最大snapshot limit预留 | `prepared bytes + target_snapshot_limit`不是实际resident size，导致容量严重保守；capture后以actual bytes更新reservation并保持上限。 |
| DSRL-P1-036 | prepare取消只在loader前后轮询 | 大I/O、parse、compile内部不能响应deadline/cancel；向reader/parser/compiler传入cooperative budget token。 |
| DSRL-P1-037 | stage取消只在capture/stage边界轮询 | 大snapshot/validation同样不可中断；细化phase checkpoint并保留no-partial-publication。 |
| DSRL-P1-038 | Drop只request cancel不join/drain | task仍持Arc state、Project/Level clone直到自然结束；由operation owner监督、deadline drain并在DLL/world teardown前quiesce。 |
| DSRL-P1-039 | poisoned mutex被静默恢复 | status/result poison应产生typed task fault和diagnostic；不能继续把可能破坏不变量的数据当成功候选。 |
| DSRL-P1-040 | resident drop没有terminal queue entry | ready超限必须回到deferred/reconcile或产生CapacityRejected outcome；所有accepted event最终可查询。 |

## 10. P1：Report、Diagnostics、World Sync 与 Lifecycle

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| DSRL-P1-041 | `pending_report`只列prepare tasks | 覆盖deferred、ready、target-stage、retry与conflict phase，并按instance呈现age/bytes/deadline。 |
| DSRL-P1-042 | `pending_count`按AssetId去重丢失实例数 | 分离asset count、instance count、physical worker count与logical operation count。 |
| DSRL-P1-043 | frame report携带多个Vec并每帧替换 | 采用bounded journal/cursor和small frame summary；详细result由operation/diagnostic store按retention查询。 |
| DSRL-P1-044 | report无project/world/level/instance generation | 每个outcome贯穿BuildSet、project generation、world epoch、asset/artifact revision与instance generation。 |
| DSRL-P1-045 | report无terminal disposition/retryability | 区分Applied、PreservedLkg、RetryScheduled、Cancelled、Conflict、Removed、Quarantined、CapacityRejected和Unknown。 |
| DSRL-P1-046 | ABI `WorldFact`只保留四个计数 | 发布受影响scene/instance/entity/resource范围与generation，或提供cursor引用；不能丢弃old-to-new map后只发计数。 |
| DSRL-P1-047 | 任意reload fact都会`invalidate_all_assets` | 按affected asset/instance/entity/component精确dirty；全量invalidate只用于明确overflow/gap fallback并带原因。 |
| DSRL-P1-048 | Runtime diagnostics漏掉关键内部指标 | 暴露drop、gap、bytes、budget exhaustion、cancel/waste、oldest age、retry、LKG和phase depth，限制cardinality。 |
| DSRL-P1-049 | log没有asset/instance/error/duration身份 | 结构化记录operation、asset、instance、old/new revision、phase、elapsed、bytes和disposition；错误不能只剩count。 |
| DSRL-P1-050 | shutdown没有显式reload coordinator phase | 先停watch intake，再冻结admission、取消/排空prepare/stage、完成或拒绝commit、撤销wake/subscription，最后卸载services。 |

## 11. P1：测试、性能与验收证据

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| DSRL-P1-051 | apply测试几乎都从空World开始 | 用真实已加载root/subscene实例验证replace、entity identity、external references和runtime state。 |
| DSRL-P1-052 | 双asset byte test固化错误选择语义 | 改为一个已实例化asset被更新、一个无关asset被忽略，并证明预算仍累计。 |
| DSRL-P1-053 | 无连续Modified防复制回归 | 同instance 100次修改后owned entity数量、引用和generation受控，不出现append副本。 |
| DSRL-P1-054 | 无Removed真实清理回归 | 覆盖unload、LKG/quarantine policy、外部引用、watch/selection和resource ownership。 |
| DSRL-P1-055 | 无ReloadFailed->成功retry矩阵 | 覆盖同content revision不同attempt、保留LKG、错误可见与恢复后原子切换。 |
| DSRL-P1-056 | 无capacity/drop终态测试 | latest、metadata、ready resident、apply bytes每种拒绝都必须有retry或terminal receipt，且不得永久stale。 |
| DSRL-P1-057 | 无disconnect/resubscribe测试 | publisher关闭、manager replacement、catalog reopen后恢复subscription并reconcile已登记instances。 |
| DSRL-P1-058 | 1/1k/100k矩阵只测单步count | 同机同profile记录event/schedule/ready/apply p50/p95、RSS、allocation、queue age、lock hold与worker waste。 |
| DSRL-P1-059 | 无hung/slow/fault/multi-world soak | 覆盖0/10/1000ms和不返回worker、parse error、OOM/capacity、world churn、Level unload、两个PIE session与shutdown。 |
| DSRL-P1-060 | 两份failure没有current-source terminal evidence | Runtime04 focused dynamic-scene test与Runtime08 compiled transaction仍open；必须取得source-bound受管Windows terminal receipt后才能关闭。 |

## 12. P2：可维护性、API与证据质量

| ID | 差距 | 建议 |
|---|---|---|
| DSRL-P2-001 | `AssetReloadApplied`在只有drain/pending/failure时也发布 | 改为中性`AssetReloadActivity`或按terminal disposition分fact。 |
| DSRL-P2-002 | task ID是进程全局AtomicU64 | 使用operation/owner epoch并处理耗尽，不能作为持久或跨session identity。 |
| DSRL-P2-003 | prepared bytes用JSON计数×2估算 | 建立artifact resident/capacity accounting，区分serialized、decoded、staging与GPU/side-system成本。 |
| DSRL-P2-004 | label/locator/error多次分配String | 采用interned/Arc identity与lazy structured diagnostics，热路径不为日志重复to_string。 |
| DSRL-P2-005 | public report暴露内部task descriptor/status | 对外只发布稳定operation snapshot，scheduler内部类型不成为scene API。 |
| DSRL-P2-006 | HashMap iteration使pending report无稳定顺序 | 以operation sequence/asset/instance canonical order输出。 |
| DSRL-P2-007 | diagnostics path无schema version | 建立metric descriptor、unit、kind、cardinality与versioned dashboard contract。 |
| DSRL-P2-008 | `usize as u64`转换无显式策略 | 使用checked/saturating typed conversion并记录overflow。 |
| DSRL-P2-009 | time report只保留单帧elapsed/max | 由profiling store输出phase histogram/quantile，CI不以未校准wall-clock硬断言。 |
| DSRL-P2-010 | `debug_assert`承担byte/accounting一致性 | production可观测地fail-close或修复counter drift，debug assert只作额外保护。 |
| DSRL-P2-011 | static/test-only queue路径与production语义并行 | 测试通过真实ProjectAssetManager/instance registry fixture，减少测试专用direct World apply。 |
| DSRL-P2-012 | skip reason把capacity与semantic state混在一层 | 分开AdmissionOutcome、SourceState、PrepareOutcome和ApplyDisposition。 |
| DSRL-P2-013 | reconciliation里的`expect`依赖局部构造不变量 | 让typed constructor返回不可错event或传播typed invariant fault。 |
| DSRL-P2-014 | 默认预算缺少来源与调优说明 | 文档化预算模型、平台class、scene规模、峰值resident公式和override validation。 |
| DSRL-P2-015 | API缺少instance lifecycle示例 | 提供load/register/reload/remove/retry/unload的可编译示例与failure receipts。 |
| DSRL-P2-016 | 当前报告只有静态fingerprint | 实施后绑定BuildSet、Cargo binary、workload corpus、trace与runtime evidence archive。 |

## 13. 目标架构

```text
Project Asset Event Stream
  -> SceneReloadCoordinator
     -> SceneReloadSelection (registered instances only)
     -> Source/Artifact Generation Resolver + LKG
     -> per-asset prepare single-flight
     -> per-instance ReplacementPlan
        { old generation, owned entities, overrides, references, new artifact }
     -> preflight all affected instances
     -> World Safe-Point Publish
        -> DynamicSceneInstanceRegistry generation commit
        -> precise WorldFact / old-to-new cursor
     -> OperationReceipt / retry / quarantine / removal
```

`DynamicSceneInstanceRegistry`是唯一instance truth，不能再由queue的latest map、World扫描、Editor selection或asset registry各自推断。`SceneReloadCoordinator`不拥有通用asset import、ECS transaction、prefab merge或operation framework；它组合这些owner并保证scene instance纵向一致性。

## 14. 重构里程碑

### M0 · P0 Characterization 与 Product Gate

- 暂停默认自动apply：没有instance selector时只记录事件，不修改World；
- 加入无关SceneAsset、重复Modified、Removed、ready-capacity drop四组RED测试；
- 冻结`SceneInstanceId`、source/attempt/artifact/world generation与terminal disposition schema。

### M1 · Instance Registry 与精准选择

- scene load/spawn返回instance handle并登记source、owned entities、remap、parent和policy；
- queue只为registered instances处理Modified/Removed；Added不自动spawn；
- gap/catalog reconciliation只扫描registered asset set并校验artifact generation。

### M2 · Replacement/LKG Transaction

- prepare构造replacement plan，保留old instance直到new preflight全部成功；
- 原子发布new owned set、old-to-new mapping、reference fixup与registry generation；
- Removed、ReloadFailed、rename、override rebase和resource ownership进入typed policy。

### M3 · Operation、Lifecycle 与 World Sync

- 接入Runtime41 operation和Runtime43 session shutdown/wake；
- count-only fact硬切为precise affected cursor，inspection不再全量invalidate；
- disconnected/resubscribe、retry/backoff、deadline、quarantine和terminal journal闭合。

### M4 · Scale、Fault 与产品资格

- 1/1k/100k asset/instance、0/10/1000ms/hung worker、multi-Level/PIE、world churn与shutdown soak；
- 采集p50/p95、RSS、allocation、resident bytes、lock hold、waste与frame budget；
- current-source managed Windows Cargo、真实Runtime/Editor workflow与source-bound evidence同时通过。

## 15. 验收门禁

| Gate | 验收内容 |
|---|---|
| DSRL-G01 | 无registered instance的SceneAsset Added/Modified/Renamed不改变任何World |
| DSRL-G02 | gap reconciliation只产生registered asset/instance work，不遍历后注入全部Ready Scene |
| DSRL-G03 | 同一instance连续100次Modified后实体数量无append增长 |
| DSRL-G04 | 两个instance引用同一asset时分别保留instance identity、parent与override |
| DSRL-G05 | Removed按policy精确unload或保留LKG，绝不只skip |
| DSRL-G06 | ReloadFailed保留旧可用instance并发布可查询错误/attempt generation |
| DSRL-G07 | 同content revision成功retry不被旧failure authority压制 |
| DSRL-G08 | remove/recreate使用新owner epoch，旧延迟事件不能复活旧identity |
| DSRL-G09 | missing locator/capacity拒绝不永久消费revision |
| DSRL-G10 | ready resident超限产生retry或terminal CapacityRejected，不静默丢结果 |
| DSRL-G11 | asset/source/artifact/project/world/instance generation贯穿prepare到receipt |
| DSRL-G12 | artifact digest变化或registry drift使旧prepare fail closed |
| DSRL-G13 | replacement commit前旧instance始终可用，失败后World与registry都不变 |
| DSRL-G14 | 成功commit同代发布World、instance registry和precise invalidation |
| DSRL-G15 | old-to-new mapping修复scene内外entity/reference并可供下游查询 |
| DSRL-G16 | runtime transient、authoring override、network/save-owned字段按声明策略处理 |
| DSRL-G17 | subscene不能无声明覆盖World-global resource |
| DSRL-G18 | rename原子更新source URI、watch/provenance与instance generation |
| DSRL-G19 | multi-asset dependency batch不产生不可解释的半新半旧World |
| DSRL-G20 | Level/world generation变化返回typed conflict并按policy retry或终止 |
| DSRL-G21 | 单个apply不能无界突破frame budget；超大reload走明确loading barrier或分段方案 |
| DSRL-G22 | prepare/stage parser/compiler响应deadline/cancel且不发布部分结果 |
| DSRL-G23 | shutdown在DLL/world卸载前终止watch、worker、wake、callback与Level lease |
| DSRL-G24 | poison/panic/hung worker成为typed terminal fault，不冻结session或静默继续 |
| DSRL-G25 | receiver断开后health sticky可见，并可在manager generation切换后resubscribe/reconcile |
| DSRL-G26 | pending snapshot覆盖deferred/prepare/ready/stage/retry/conflict并有稳定顺序 |
| DSRL-G27 | operation journal可按OperationId查询每个accepted event的terminal disposition |
| DSRL-G28 | ABI fact不再只靠四个计数表达instance mutation |
| DSRL-G29 | inspection只dirty受影响asset/entity/watch；overflow fallback明确标记full invalidation |
| DSRL-G30 | diagnostics覆盖depth/age/bytes/drop/gap/cancel/waste/retry/LKG和phase duration |
| DSRL-G31 | 真实play scene加载后修改同一asset执行replace而非append |
| DSRL-G32 | 无关asset事件、双project、双PIE、双Level互不串扰 |
| DSRL-G33 | 1/1k/100k矩阵记录事件、实例、任务、RSS、allocation、lock与frame指标 |
| DSRL-G34 | 0/10/1000ms/hung worker矩阵证明single-flight、bounded residency与bounded teardown |
| DSRL-G35 | event gap、capacity、parse、schema conflict、world churn、remove/recreate fault matrix通过 |
| DSRL-G36 | Runtime04 focused reload test取得current-source受管Windows terminal receipt |
| DSRL-G37 | Runtime08 compiled transaction test取得current-source受管Windows terminal receipt |
| DSRL-G38 | Runtime与Editor真实workflow验证selection、override/reference、remove/failure和recovery |
| DSRL-G39 | reference/fingerprint/BuildSet/workload/result绑定同一evidence archive，不能以source-shape代替运行证据 |
| DSRL-G40 | `git diff --check`、Markdown LF/BOM/trailing-space、finding ID、路径、链接、索引和portfolio计数全部通过 |

## 16. 状态与开放记录

| 项目 | 状态 | 说明 |
|---|---|---|
| Runtime53静态review | review_complete | 3 P0 / 60 P1 / 16 P2 / 40 gates |
| Production与tests修改 | pending | 本轮未修改 |
| Runtime04 bounded single-flight failure | open | 实现候选存在，缺current-source terminal managed validation |
| Runtime08 compiled spawn transaction failure | open | hard-cut源码存在，缺current-source terminal managed validation与完整规模证据 |
| Source currentness | recheck_required | `session/state.rs`与spawn transaction test有并发会话修改/lease |
| Cargo/Runtime/Editor/benchmark | not_run | MVP gate下本轮只读审查，不宣称green |
