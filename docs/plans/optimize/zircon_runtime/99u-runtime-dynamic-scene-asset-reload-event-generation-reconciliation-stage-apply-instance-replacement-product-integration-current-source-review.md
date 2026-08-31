---
title: Runtime Dynamic Scene Asset Reload、Event Generation、Reconciliation、Stage/Apply、Instance Replacement 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime120
review_date: 2026-08-23
baseline_head: 1354e50da53db3dad1dc25a6c9e375942ba04d35
baseline_epoch: 368
supersedes:
  - docs/plans/optimize/zircon_runtime/53-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-review.md
related_code:
  - zircon_runtime/src/scene/dynamic_scene/asset_reload
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
  - zircon_editor/src/core/gateway/session/world_sync.rs
  - zircon_editor/src/core/sync/pump.rs
  - zircon_editor/src/core/sync/watch_map.rs
  - zircon_editor/src/ui/host/editor_world_sync.rs
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene_asset_reload.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_asset_reload/byte_budgets.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/scene_patch_document.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests/resources.rs
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_editor/src/core/sync/pump/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene/asset_reload_selection_status.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/53-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-review.md
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99u · Runtime Dynamic Scene Asset Reload Current Source Review

## 1. 结论

当前 Dynamic Scene asset reload 已有值得保留的工程底座：typed `SceneAsset` 事件、event count/bytes/time 预算、`AssetId` keyed single-flight、一个 active worker 加一个 deferred successor、supersede/cancel、event-log generation gap 检测、增量 resource reconciliation、prepared/result/ready/apply resident byte 上限、异步 target capture/staging、World/schema/component/change-tick generation CAS、主线程 bounded commit、reactive wake 与内部队列诊断都是真实代码，不应退回同步读 JSON 后直接改 World 的临时实现。compiled spawn transaction 也已把 preflight artifact 与 infallible publication 分开，是 replacement transaction 可以复用的基础。

但 Runtime53 的三项 P0 在当前源码中全部仍成立。第一，任何 prepared project session 都订阅整个项目的 `SceneAsset` 事件，`scene_uri` 没有交给 reload owner；Added、Modified、Renamed 和 gap reconciliation 找到的全部 Ready/Reloading Scene 都可能进入当前唯一 Level。第二，queue 没有 `SceneInstanceId`、asset-to-instances 或 instance-to-owned-entities registry；成功 apply 仍是 fresh `EntityRemap` 的 append spawn，Removed/ReloadFailed 只 skip，frame report 中的 remap 没有持久 owner。第三，latest revision 在 locator、admission、schedule 和最终 publication 之前推进，ready resident 超限又只累加 drop 后返回；同 revision 随后会被 stale filter 永久压制，asset truth 与 World truth 可以无终态地分叉。

当前源码相对 Runtime53 只有一项 finding 获得实质进展：target-stage reservation 已由固定最大 snapshot 预留改成 `Arc<AtomicUsize>`，实际 capture 后会按 `prepared + actual target bytes` 收敛，并新增直接回归与 65,536 staged scene 的 ignored capacity benchmark。因此 `DSRL-P1-035` 判为 **Partial**；初始 admission 仍按最大 snapshot limit 保守预留，ignored benchmark 与受管 Windows 动态资格也未完成，不能判 Closed。诊断的 13 次 store 更新合并为一次临界区是局部热路径改善，但不改变实例、revision、终态或 WorldSync 合同。

当前总账为 **3 P0 Open、59 P1 Open、1 P1 Partial、16 P2 Open、40 Gate Fail**。目标必须硬切为 `SceneReloadExposurePolicy -> DynamicSceneInstanceRegistry -> SceneReloadCoordinator -> Source/Attempt/Artifact Resolver + LKG -> per-instance ReplacementPlan -> World Safe-Point Publish -> precise WorldFact/OperationReceipt`。本轮只做 current-source 静态审查与文档记录，没有修改 production、tests、Cargo、ABI 或参考源码，没有运行 Cargo、真实 Runtime/Editor、fault injection、multi-Level、RSS、profile 或 benchmark。MVP `00` 仍为 `in_progress`，F0-F5 仍 blocked；本文不把 source-local candidate 写成 milestone accepted，也不展开 tooling 优化。

## 2. 审查边界与物理冻结

| 范围 | 文件 / 行 / bytes / tests / ignored / dirty | fingerprint |
|---|---:|---|
| asset reload production owner | 14 / 2,989 / 104,826 / 4 / 0 / 3 | `e42dc3ac731508c0da330900c99973c36875f04946ad9239100581766697a08a` |
| spawn/preflight support | 10 / 1,877 / 63,185 / 6 / 0 / 0 | `e381afa2c55d697ca739ab575460049026b7f64c64e3bfc96ac4e516bcf825c2` |
| Session、ABI、Editor 产品链 | 10 / 3,473 / 126,748 / 20 / 0 / 3 | `000e202b87e07ed8b14400f1a88d70059949aa59ee5053ff1b6c89db9ee54855` |
| focused direct tests / guards | 8 / 3,656 / 144,782 / 67 / 1 / 3 | `51fb811a0f299c52545d972c880a4a9ce7989cd6c93c5f6694c033e2140f18d0` |
| 五引擎参考实现 | 15 / 25,705 / 998,003 / 31 / 0 / 0 | `1209933a1740800722af295f701545dd46bdceca0cc2f0fb401580178619b853` |

fingerprint 算法：仓库相对路径转 `/`、排序去重，以 `path|lowercase per-file SHA-256` 逐行编码，LF 连接且末尾无 LF，再计算 UTF-8 SHA-256。它冻结本轮实际读取集合，不是 asset digest、BuildSet、project generation、World epoch、instance generation 或 release identity。

当前 worktree 中 owner 的 `queue.rs`、`queue/target_staging.rs`、`stage_task.rs`，产品链的 `construction.rs`、`state.rs`、`scene_asset_reload_diagnostics.rs`，以及 focused tests 的主 reload test、byte budget test、spawn transaction test 已有其他会话或用户改动。本文读取并审查当前内容，但不拥有、不覆盖这些差异；实施前必须按本表重新冻结，因此 `source_recheck_required` 为 true。coordinator 基线为 `1354e50da53db3dad1dc25a6c9e375942ba04d35` / epoch 368。

参考 corpus 的 31 个 test declaration 主要来自 Bevy/Fyrox 文件内测试和邻近通用测试；对选定 Unreal package reload、Godot resource/scene replacement、Unity Graphics `ResourceReloader` 的定向测试符号搜索没有找到等价 end-to-end replacement fixture。参考源码提供架构下限，不构成 Zircon 动态验收替代品。

## 3. 当前真实产品链

```text
RuntimeDynamicSession construction
  -> load scene_uri into one root/play Level
  -> ProjectAssetManager::subscribe_asset_events::<SceneAsset>()
     -> DynamicSceneAssetReloadQueue (project-wide; keyed only by AssetId)
        -> tick before Level tick
           -> event drain / resource reconciliation
           -> DynamicSceneSpawnTask prepare
           -> Level target capture + stage
           -> CompiledSceneSpawnTransaction append publish
        -> frame report { applied scenes/remap, failed, skipped, pending }
           -> WorldFact::AssetReloadApplied { four aggregate u64 fields }
              -> SubscriptionTable::invalidate_all_assets()
                 -> generic Editor world-fact pump
```

`construction.rs` 已算出并保存 `scene_uri`，但 queue 构造只接收 current `ProjectAssetManager`；产品链没有把 root scene、loaded subscene、World/Level/PIE identity 或 explicit reload exposure 传给 queue。`tick_frame` 在 Level tick 前运行 reload，shutdown 先停止 project watcher 再卸载 modules，没有 coordinator quiesce、attempt drain、journal seal 或 replacement rollback phase。

`DynamicSceneAssetReloadAppliedScene` 确实保存 event、`EntityRemap` 与 entity/component/resource 计数；但 Session 只把 applied/failed/stale/pending 四个聚合值投影到 `WorldFact`。subscription 对任何 reload fact 执行 `invalidate_all_assets()`，Editor gateway/pump 只是通用 fact transport，没有 per-instance apply acknowledgement、old-to-new map、selection/reference repair 或 terminal receipt。

## 4. 当前源码裁决

| 主题 | 当前直接证据 | 状态裁决 |
|---|---|---|
| selection | queue project-wide subscribe；pending/deferred/ready/staging/latest map 全以 `AssetId` 为 key；全 Runtime/Editor/Interface 无 `SceneInstanceId`、`DynamicSceneInstanceRegistry`、`asset_to_instances` 或 `instance_to_entities` | `DSRL-P0-001 Open` |
| replacement | spawn transaction 构造 fresh `EntityRemap`，插入新 records/components/resources；没有 old instance generation、owned set、despawn/replace/reference-fixup | `DSRL-P0-002 Open` |
| revision/terminal | latest 在 locator/admission/schedule 前记录；Removed 删除 tombstone；ReloadFailed 只 skip；ready resident overflow 只计 drop 后 return | `DSRL-P0-003 Open` |
| gap/reconcile | lag gap 会 cancel 并启动 `ResourceManagementScan`，但扫描所有 `ResourceKind::Scene`，并把 Ready/Reloading 映射为 Updated | bounded 恢复底座保留，instance qualification 仍 Open |
| prepare/stage | one active worker、deferred successor、budget、metadata、ready/stage/apply resident caps 与 CAS 均存在；cancel 只在 loader/parse/capture 边界轮询，Drop 不 join | 局部工程底座保留，lifecycle/interruptibility 仍 Open |
| target reservation | stage 初始预留 `prepared + snapshot_limit`；capture 后 atomic 更新为 `prepared + actual target bytes`，error 更新为 0 | `DSRL-P1-035 Partial` |
| diagnostics | 13 条 frame series 已在一次 `update_diagnostic_store` 临界区内写入 | 性能局部改善；语义/receipt finding 不关闭 |
| tests | reload apply 仍从空 World/Level 开始；双 asset 测试把两个无关 Scene 都应用进同一 World；无 loaded-instance replace/remove/LKG/external-reference fixture | 产品 replacement 资格仍缺失 |

应保留而不重复实现的底座：raw/filtered event 都受 count/bytes/time 预算；每 `AssetId` 至多一个 active prepare worker和一个 logical successor；旧结果在 publish 前重新检查 authority；gap 会显式进入 reconciliation；prepared/result/ready/stage/apply 都有 resident cap；preflight 与 infallible publish 分离；target generation/schema/component/change tick 改变会拒绝 stale mutation；pending work 会请求 reactive frame demand。

## 5. 五套参考实现裁决

| 参考 | 已读取的直接事实 | 对 Zircon 的硬约束 |
|---|---|---|
| Unreal package reload / Blueprint reinstancing | `FPackageReloadedEvent` 持有 old/new package、repointed object map 与 referencers；reload 分 PreLoad/Fixup/PostFixup/batch/GC 阶段；Blueprint reinstancer维护 old/new class、CDO/archetype/instance map并替换引用 | scene reload 必须拥有 replacement identity、referencer fixup、阶段边界、batch lifetime 和失败时旧对象寿命；registry event 不能直接等价为 spawn command |
| Bevy `WorldInstanceSpawner` | `spawned_worlds: asset -> set<InstanceId>`，`spawned_instances: InstanceId -> { entity_map, parent }`；Modified 只更新已实例化 asset，update 先 despawn owned entities，再复用 instance record respawn | 即使采用较重的全实例 respawn，也必须先有 instance registry、owned entity map 和 registered-only selection；Zircon 可进一步做增量 patch，但不能省略身份 |
| Godot resource / editor scene replacement | resource reload 用 ignore-cache load 后 `copy_from` 保持同一 resource identity；Editor scene reload 保存 modification、additional nodes、external references、selection、history、connections和继承顺序，再 `replace_by` 并恢复状态 | authoring/runtime override、external reference、inheritance/provenance、selection/history 必须有显式保留策略；全局 append 不满足最低语义 |
| Fyrox resource/model reload | reload 成功在同一 resource 上提交 `ResourceState::Ok` 并广播 Reloaded；失败且旧 resource 有效时保留旧版本；engine 先 resolve resource dependency，再遍历 active scenes resolve model instance | failure 必须保留 last-known-good；instance 必须可追溯 source resource，并按 active scene/instance 定向传播 |
| Unity Graphics `ResourceReloader` | Editor-only utility 按显式 reload group/attribute 递归修补 missing/broken field/array element，区分 AssetDatabase not ready并标记 container dirty | 这是显式容器修复，不是 scene instance hot reload 模型；不能据此为扫描全项目 Scene 并注入 World 背书 |

共同架构下限是 explicit owner/source identity、instantiated instance identity、old/new mapping、last-known-good、replacement/fixup phase、lifecycle 和 terminal disposition。若 Zircon 要在性能上优于参考实现，应保留当前 bounded async prepare 和 atomic publish，再以 changed-instance closure、共享 artifact、增量 replacement、短 safe-point commit 和可复现 profile 证明优势；不能用缺失 replacement 语义换取表面吞吐。

## 6. Owner 边界

| Owner | 继续拥有 | Runtime120 的纵向边界 |
|---|---|---|
| Runtime04 / Runtime51 / Runtime64 | 通用 asset event、resource state、registry generation、artifact/version lease、last-good | Scene source event 到 registered instance selection、attempt/artifact revision 与 World apply 一致性 |
| Runtime05 / Runtime08 failure / Runtime61 | World/ECS、DynamicScene compile/preflight/publish、entity identity、Level/persistence | replacement plan 如何消费这些底座，不复制 ECS/World 总问题 |
| Runtime24 / Runtime39 | stable identity、prefab/archetype instance override 与传播 | `SceneInstanceId`、source provenance、owned entity set 与 scene reload vertical contract |
| Runtime41 / Runtime59 | operation admission、cancel、deadline、progress、terminal retention、shared execution 与 shutdown | reload coordinator 接入，不建立第二套通用 task/operation framework |
| Runtime43 / Interface02 | Dynamic Session、WorldFact、跨 DLL DTO、session shutdown | Session 持有 service、precise invalidation、operation receipt 与 reconnect contract |
| Editor05 / Editor07 | Scene authoring、selection/history、Play lifecycle 与 Runtime apply | 消费 replacement map/receipt，不在 Editor 重建 Runtime instance truth |
| Runtime120 | selection、reconciliation、prepare/stage/apply、instance replace/remove/failure 的完整纵切面 | 本文 3/60/16 finding 与 40 gate；原 Runtime53 不再作为 current 状态来源 |

## 7. P0：当前 World 事实破坏

| ID | 状态 | 差距 | 当前证据与硬切目标 |
|---|---|---|---|
| DSRL-P0-001 | Open | 全项目 Scene 事件可向当前 Level 注入无关场景 | Session 对有 project 的 Level 创建 project-wide receiver；Added/Modified/Renamed 均 schedule，gap reconciliation 遍历所有 Ready/Reloading Scene。硬切 `SceneReloadExposurePolicy + registered-only SceneReloadSelection`；Added 不自动 spawn。 |
| DSRL-P0-002 | Open | Modified/Renamed append 副本，Removed/ReloadFailed 不回收实例 | compiled spawn 从 fresh remap 插入实体/组件/资源；queue 无 instance registry；Removed/ReloadFailed 只 skip。建立 `SceneInstanceId -> source/revision/owned entities/remap/override/LKG`，以 replacement/patch transaction 原子提交。 |
| DSRL-P0-003 | Open | revision 已推进但 World 结果可静默丢弃且同 revision 不可恢复 | latest 在 locator/admission/schedule 前记录；ready cap drop 无 retry/terminal entry；同 revision 后续被 stale。source observation、attempt 和 committed instance revision 必须分域，accepted request 必须有 terminal disposition。 |

## 8. P1：Owner、Scope 与 Selection

| ID | 状态 | 差距 | 需要重构的内容 |
|---|---|---|---|
| DSRL-P1-001 | Open | queue 对任何 project session 默认启用 | 由 compiled runtime profile/capability 决定 Editor hot reload、development runtime reload 或 shipping disabled。 |
| DSRL-P1-002 | Open | queue 只绑定一个 current Level | 显式记录 Project/World/Level/PIE/session identity，多 Level、多 viewport、多 session 互不串扰。 |
| DSRL-P1-003 | Open | queue 不知道 current play scene | construction 的 `scene_uri` 必须在成功实例化后成为 registry source edge，而不是仅留在 Session 字段。 |
| DSRL-P1-004 | Open | 无 instance register/unregister API | scene load/spawn 返回 `SceneInstanceId`，unload/Level destroy 原子撤销 source 与 owned set。 |
| DSRL-P1-005 | Open | 无 root/subscene/prefab/resource-only source 区分 | typed source edge 路由到 scene、prefab 与 resource owner，禁止同一 event 统一 append。 |
| DSRL-P1-006 | Open | 无 per-instance reload policy | 定义 auto/manual/disabled、preserve overrides、restart-required 与 authority-only。 |
| DSRL-P1-007 | Open | limits 是代码默认而非产品 policy | validated profile 提供预算、来源与 effective values，call-site 不能形成第二 authority。 |
| DSRL-P1-008 | Open | 未接 Runtime41 operation contract | batch reload 需要 OperationId、deadline、cancel、progress、terminal receipt、retention 与 shutdown disposition。 |
| DSRL-P1-009 | Open | public queue/task/report surface 暴露机制细节 | 对外只保留 service、instance handle、request、policy、snapshot 与 typed outcome。 |
| DSRL-P1-010 | Open | capability readiness 未表达 | provider/watch/receiver/registry/apply owner 缺失时返回 Unavailable/Degraded，不能发布虚假 ready。 |

## 9. P1：Event、Revision、Gap 与 Reconciliation

| ID | 状态 | 差距 | 需要重构的内容 |
|---|---|---|---|
| DSRL-P1-011 | Open | same-revision authority rank 是 queue 私有规则 | 把 source sequence、content revision、attempt generation 与 terminal state 纳入统一 version contract。 |
| DSRL-P1-012 | Open | ReloadFailed 压制同 revision 恢复 | failure attempt 与 content revision 分域，成功 retry 可替换同 content revision 的失败 attempt。 |
| DSRL-P1-013 | Open | Removed 删除 tombstone | 保留带 owner epoch 的 tombstone，直到 asset registry 证明新 identity；旧延迟事件不得复活。 |
| DSRL-P1-014 | Open | latest 以 60 秒 wall clock TTL 退休 | 只在 stream/generation checkpoint 证明安全时回收 ordering state。 |
| DSRL-P1-015 | Open | missing locator 发生在 latest 推进后 | locator 缺失产生 retry/reconcile token，不永久消费 committed instance revision。 |
| DSRL-P1-016 | Open | latest/metadata capacity 拒绝只生成局部 skip | 保持 source dirty，并安排 bounded reconciliation 或 terminal CapacityRejected。 |
| DSRL-P1-017 | Open | deferred admission 失败可失去旧 successor | 新 successor admission 成功后再原子替换，失败时保留旧可执行 successor。 |
| DSRL-P1-018 | Open | receiver disconnect 无重建 owner | sticky health、resubscribe generation、bounded backoff 和 registered-instance reconcile。 |
| DSRL-P1-019 | Open | reconciliation 把 Reloading 当 Updated | 只消费 published Ready artifact；Reloading 保留 attempt pending，不构造 scene apply。 |
| DSRL-P1-020 | Open | event/result 不绑定 content/artifact digest | 贯穿 source digest、artifact digest、project/import generation 与 registry lease。 |

## 10. P1：Instance、Override、Reference 与 Replacement

| ID | 状态 | 差距 | 需要重构的内容 |
|---|---|---|---|
| DSRL-P1-021 | Open | 无 asset-to-instances index | 建立 `AssetId -> ordered SceneInstanceId set`；Modified 只触达现有 instances。 |
| DSRL-P1-022 | Open | 无 instance-to-entities ownership | 保存 source entity 到 live entity map 与 owned set，支持精确 replace/unload/leak check。 |
| DSRL-P1-023 | Open | apply remap 只活一帧 | remap 进入 instance generation，供 reference fixup、selection、save/network 与 diagnostics 查询。 |
| DSRL-P1-024 | Open | rename 只更换读取 URI | source URI、asset identity、watch key、serialized provenance 与 instance generation 同事务更新。 |
| DSRL-P1-025 | Open | remove 无 unload/quarantine policy | root、subscene、prefab 分别定义 unload、reject、quarantine 或 LKG 行为。 |
| DSRL-P1-026 | Open | failure 无 LKG instance state | 记录 active old revision、failed attempt、error、retryability 与 operator action。 |
| DSRL-P1-027 | Open | 无 runtime/authoring override policy | 区分 source-owned、instance override、runtime transient、network authoritative、save-owned 字段并 rebase/reject。 |
| DSRL-P1-028 | Open | 无 external entity/reference fixup | replacement old-to-new map 修复 parent/joint/script/selection/watch/render/physics 与 scene 外引用。 |
| DSRL-P1-029 | Open | subscene reload 可写 World-global resource | 声明 resource ownership 与 merge/replace/forbid policy，任意 scene 不得覆盖全局 authority。 |
| DSRL-P1-030 | Open | multi-instance/multi-asset 可半提交 | dependency-ordered batch preflight；原子 publish，或明确 per-instance receipt 与 compensation。 |

## 11. P1：Prepare、Stage、Apply、Cancel 与 Backpressure

| ID | 状态 | 差距 | 需要重构的内容 |
|---|---|---|---|
| DSRL-P1-031 | Open | prepared artifact 只描述 spawn | 增加 old instance generation、expected owned set、override rebase、reference fixup 与 new artifact 的 replacement plan。 |
| DSRL-P1-032 | Open | target-changed 失败无 retry policy | 按冲突原因重新 capture、退避、cancel 或 terminal conflict。 |
| DSRL-P1-033 | Open | apply time budget 在首个 commit 后检查 | 单个 commit 可超 2 ms；提供可预估/分段 safe-point，或进入显式 loading barrier。 |
| DSRL-P1-034 | Open | 多个 commit 不共享 atomic publication | batch 绑定 World epoch 与 dependency consistency，禁止不可解释的半新半旧。 |
| DSRL-P1-035 | Partial | target stage 初始按最大 snapshot limit 预留 | 已在 capture 后 atomic 收敛为 actual target bytes，并有直接回归与 ignored benchmark；仍需改善保守初始 admission，并完成 release/managed RSS、capacity 与 contention 资格。 |
| DSRL-P1-036 | Open | prepare cancel 只在 loader/parse 边界轮询 | reader/parser/compiler 接受 cooperative budget token，长 I/O/parse/compile 响应 deadline。 |
| DSRL-P1-037 | Open | stage cancel 只在 capture/stage 边界轮询 | snapshot/validation 增加 phase checkpoint，同时保持 no-partial-publication。 |
| DSRL-P1-038 | Open | Drop 只 request cancel，不 join/drain | operation owner 监督 worker，在 DLL/World teardown 前 quiesce、deadline drain 并撤销 lease。 |
| DSRL-P1-039 | Open | poisoned mutex 静默恢复并复用内容 | 转为 typed task fault 与 fail-closed diagnostic，不能继续信任可能破坏不变量的 candidate。 |
| DSRL-P1-040 | Open | resident drop 无 terminal queue entry | 回到 deferred/reconcile 或发布 CapacityRejected；每个 accepted event 最终可查询。 |

## 12. P1：Report、World Sync 与 Lifecycle

| ID | 状态 | 差距 | 需要重构的内容 |
|---|---|---|---|
| DSRL-P1-041 | Open | pending report 只覆盖 prepare | 覆盖 deferred/ready/stage/retry/conflict，并按 instance 报 age/bytes/deadline。 |
| DSRL-P1-042 | Open | pending count 按 AssetId 去重 | 分离 asset、instance、physical worker 与 logical operation count。 |
| DSRL-P1-043 | Open | frame report 每帧替换多个 Vec | 使用 bounded journal/cursor 与小型 frame summary，详细结果按 retention 查询。 |
| DSRL-P1-044 | Open | report 缺 project/world/level/instance generation | outcome 贯穿 BuildSet、project、World、asset/artifact 与 instance generation。 |
| DSRL-P1-045 | Open | report 无 terminal disposition/retryability | 区分 Applied、PreservedLkg、RetryScheduled、Cancelled、Conflict、Removed、Quarantined、CapacityRejected、Unknown。 |
| DSRL-P1-046 | Open | ABI fact 只有四个聚合计数 | 发布 affected scene/instance/entity/resource range 与 generation，或稳定 cursor。 |
| DSRL-P1-047 | Open | 任意 reload fact 都 invalidate all assets | 精确 dirty affected asset/instance/entity/component；full invalidation 仅用于带原因的 overflow/gap。 |
| DSRL-P1-048 | Open | Session diagnostics 漏关键 queue 事实 | 暴露 drop/gap/bytes/budget exhaustion/cancel/waste/oldest age/retry/LKG/phase depth，限制 cardinality。 |
| DSRL-P1-049 | Open | log 无 asset/instance/error/duration identity | 结构化记录 operation、asset、instance、old/new revision、phase、elapsed、bytes 与 disposition。 |
| DSRL-P1-050 | Open | shutdown 无 reload coordinator phase | stop intake、freeze admission、cancel/drain prepare/stage、finish/reject commit、seal receipt、revoke wake/subscription，再卸载。 |

## 13. P1：测试、性能与资格证据

| ID | 状态 | 差距 | 需要重构的内容 |
|---|---|---|---|
| DSRL-P1-051 | Open | apply tests 几乎都从 empty World/Level 开始 | 以真实 loaded root/subscene 验证 replace、identity、external references、override 与 runtime state。 |
| DSRL-P1-052 | Open | dual-asset byte test 固化错误 selection | 一个 registered asset 更新、一个 unrelated asset 忽略，同时证明预算累计。 |
| DSRL-P1-053 | Open | 无连续 Modified 防复制回归 | 同 instance 100 次修改后 owned count、references、generation 受控，无 append 副本。 |
| DSRL-P1-054 | Open | 无 Removed cleanup 回归 | 覆盖 unload/LKG/quarantine、external refs、watch/selection 与 resource ownership。 |
| DSRL-P1-055 | Open | 无 ReloadFailed -> success retry matrix | 同 content revision 不同 attempt、LKG、错误可见、恢复后 atomic switch。 |
| DSRL-P1-056 | Open | 无所有 capacity/drop terminal tests | latest/metadata/ready/apply bytes 每种拒绝都有 retry 或 terminal receipt，且不永久 stale。 |
| DSRL-P1-057 | Open | 无 disconnect/resubscribe 测试 | publisher close、manager replacement、catalog reopen 后恢复并 reconcile registered instances。 |
| DSRL-P1-058 | Open | 1/1K/100K 只覆盖单步 count | 记录 event/schedule/ready/apply p50/p95、RSS、allocation、queue age、lock hold、worker waste。 |
| DSRL-P1-059 | Open | 无 hung/slow/fault/multi-world soak | 0/10/1000 ms 与 hung worker、parse error、OOM/capacity、World churn、Level unload、双 PIE/session、shutdown。 |
| DSRL-P1-060 | Open | 两份 failure 无 current-source terminal receipt | Runtime04 single-flight 与 Runtime08 compiled transaction 均保持 `open`，直到 source-bound managed Windows receipt 和规模证据完成。 |

## 14. P2：API、可维护性与证据质量

| ID | 状态 | 差距 | 建议 |
|---|---|---|---|
| DSRL-P2-001 | Open | 只有 drain/pending/failure 也发布 `AssetReloadApplied` | 改为中性 activity 或按 terminal disposition 分 fact。 |
| DSRL-P2-002 | Open | task ID 是 process-global `AtomicU64` | 使用 operation/owner epoch 并定义 exhaustion，不作持久/跨 session identity。 |
| DSRL-P2-003 | Open | prepared bytes 用 JSON size ×2 估算 | 区分 serialized/decoded/staging/side-system resident，并建立可校准 accounting。 |
| DSRL-P2-004 | Open | label/locator/error 重复分配 String | interned/Arc identity 与 lazy structured diagnostics。 |
| DSRL-P2-005 | Open | public report 暴露 internal task descriptor/status | 对外只发布稳定 operation snapshot。 |
| DSRL-P2-006 | Open | HashMap iteration 使 pending report 无稳定顺序 | 按 operation sequence/asset/instance canonical order。 |
| DSRL-P2-007 | Open | diagnostics path 无 schema version | metric descriptor 定义 unit/kind/cardinality/version。 |
| DSRL-P2-008 | Open | `usize as u64` 无显式转换策略 | checked/saturating typed conversion并记录 overflow。 |
| DSRL-P2-009 | Open | time report 只有 frame elapsed/max | profiling store 输出 phase histogram/quantile，CI 不用未校准 wall clock 硬断言。 |
| DSRL-P2-010 | Open | `debug_assert` 承担 accounting consistency | production 可观测地 fail-close/reconcile counter drift。 |
| DSRL-P2-011 | Open | test-only direct World apply 与产品语义并行 | 测试通过真实 ProjectAssetManager、Session 和 instance registry fixture。 |
| DSRL-P2-012 | Open | skip reason 混合 capacity 与 semantic state | 分离 AdmissionOutcome、SourceState、PrepareOutcome、ApplyDisposition。 |
| DSRL-P2-013 | Open | reconciliation `expect` 依赖局部构造不变量 | typed constructor 形成不可错事件或传播 invariant fault。 |
| DSRL-P2-014 | Open | 默认预算无来源与调优模型 | 文档化 platform class、scene规模、resident公式与 override validation。 |
| DSRL-P2-015 | Open | API 无完整 instance lifecycle example | 提供 load/register/reload/remove/retry/unload 与 failure receipt 示例。 |
| DSRL-P2-016 | Open | 当前只有静态 fingerprint | 实施后绑定 BuildSet、binary、workload、trace、profile 与 evidence archive。 |

## 15. 目标架构

```text
Asset Watch / Import / Registry Generation
  -> SceneReloadExposurePolicy
     -> DynamicSceneInstanceRegistry
        AssetId -> SceneInstanceId set
        SceneInstanceId -> { project/world/level, source, committed revision,
                             owner epoch, owned entities, remap, overrides, LKG, policy }
     -> SceneReloadCoordinator
        -> Source / Attempt / Artifact Resolver + version leases
        -> registered-instance reconciliation
        -> shared per-asset prepare single-flight
        -> per-instance ReplacementPlan
           { expected old generation, new artifact, owned-set diff,
             override rebase, old-to-new reference fixup, resource policy }
        -> dependency-ordered preflight
        -> World Safe-Point Publish
           -> World + instance registry same-generation commit
           -> precise affected cursor / WorldFact
        -> OperationReceipt / retry / LKG / quarantine / removal
```

`DynamicSceneInstanceRegistry` 是唯一 instance truth。queue latest map、World scan、Editor selection 和 asset registry 都不得重新猜测 instance identity。`SceneReloadCoordinator` 只编排现有 asset、ECS transaction、prefab override、operation、Session 与 Editor owner；它不复制通用 importer、resource manager、task scheduler 或 World persistence。

性能目标以 changed registered-instance closure 为工作量边界：同一 artifact prepare 可供多个 instances 共享，per-instance plan 可并行 preflight，publication 在短 safe point 同代提交。任何“快于 Unreal”的结论都必须绑定相同硬件、fixture、画质/功能语义、warm-up、median/range、RSS/allocator/lock/frame trace；当前文档没有这种证据。

## 16. 重构里程碑

### M120-0 · Product Gate 与 RED Characterization

- 没有 registered instance selector 时禁止自动 apply，只保留 bounded observation；
- 增加 unrelated Scene、repeat Modified、Removed、ReloadFailed retry、ready capacity drop 的 RED fixtures；
- 冻结 Project/World/Level/SceneInstance/source/content/attempt/artifact generation 与 disposition schema。

### M120-1 · Exposure Policy 与 Instance Registry

- load/spawn 成功返回 `SceneInstanceId` 并登记 source、owned entities、remap、parent、policy 和 LKG；
- Added 不自动 spawn；Modified/Removed 只作用于 registered instances；
- gap reconcile 只遍历 registered asset set 并校验 artifact lease。

### M120-2 · Replacement、LKG 与 Reference Repair

- prepare 构造 replacement plan，new preflight 全部成功前 old instance 保持 active；
- 原子发布 owned-set diff、old-to-new map、reference fixup 与 registry generation；
- remove/failure/rename/override/resource ownership 进入 typed policy。

### M120-3 · Revision、Retry、Reconciliation 与 Terminal Journal

- 分离 observed source、content、attempt、artifact 和 committed instance revision；
- capacity/disconnect/gap/conflict 都进入 bounded retry/reconcile 或 terminal outcome；
- accepted request 可按 OperationId/SceneInstanceId 查询完整 phase 与 disposition。

### M120-4 · Operation、Shutdown、WorldSync 与 Editor

- 接入 Runtime41/59 operation/execution lifecycle 和 Runtime43 Session quiesce；
- count-only fact 硬切 precise affected cursor，subscription 不再无条件 invalidate all；
- Editor 消费 replacement mapping、selection/history/reference repair 与 apply acknowledgement。

### M120-5 · Memory、Latency 与 Backpressure Qualification

- 收敛 conservative initial target reservation，校准 serialized/decoded/target/commit resident accounting；
- 1/1K/100K asset/instance、1 KiB/64 MiB、0/10/1000 ms/hung worker 矩阵；
- 采集 p50/p95、RSS、allocation、queue age、lock hold、waste、safe-point wall 与 teardown bound。

### M120-6 · Fault、Multi-World 与 Product Acceptance

- parse/schema/capacity/gap/disconnect/remove-recreate/World churn/Level unload/双 PIE/双 project/shutdown soak；
- 真实 Runtime 和 Editor workflow 验证 replace/remove/LKG/override/reference/selection/recovery；
- Runtime04、Runtime08 failure 与本报告 gates 取得同一 current-source managed Windows evidence archive。

## 17. 验收门禁

| Gate | 状态 | 验收内容 |
|---|---|---|
| DSRL-G01 | Fail | 无 registered instance 的 Scene Added/Modified/Renamed 不改变任何 World |
| DSRL-G02 | Fail | gap reconciliation 只生成 registered asset/instance work |
| DSRL-G03 | Fail | 同一 instance 连续 100 次 Modified 后实体数无 append 增长 |
| DSRL-G04 | Fail | 同 asset 两个 instances 分别保留 identity、parent 与 override |
| DSRL-G05 | Fail | Removed 按 policy unload 或保留 LKG，绝不只 skip |
| DSRL-G06 | Fail | ReloadFailed 保留旧 instance，并发布 error/attempt/retryability |
| DSRL-G07 | Fail | 同 content revision 成功 retry 不被失败 attempt 压制 |
| DSRL-G08 | Fail | remove/recreate 使用新 owner epoch，旧事件不能复活旧 identity |
| DSRL-G09 | Fail | missing locator/capacity reject 不永久消费 committed revision |
| DSRL-G10 | Fail | ready overflow 有 retry 或 CapacityRejected，不静默 drop |
| DSRL-G11 | Fail | source/artifact/project/World/instance generation 贯穿 receipt |
| DSRL-G12 | Fail | digest 或 registry drift 使旧 prepare fail closed |
| DSRL-G13 | Fail | replacement publish 前 old instance 可用，失败后 World/registry 不变 |
| DSRL-G14 | Fail | 成功 publish 同代提交 World、registry 与 precise invalidation |
| DSRL-G15 | Fail | old-to-new map 修复内外 references 并可供下游查询 |
| DSRL-G16 | Fail | transient/authoring/network/save 字段按声明 policy 处理 |
| DSRL-G17 | Fail | subscene 无声明时不能覆盖 World-global resource |
| DSRL-G18 | Fail | rename 原子更新 URI、watch/provenance 与 instance generation |
| DSRL-G19 | Fail | dependency batch 不产生不可解释的半新半旧 World |
| DSRL-G20 | Fail | World/Level generation drift 返回 typed conflict 并重试或终止 |
| DSRL-G21 | Fail | 单次 apply 有硬 frame bound，超大任务走显式 barrier/分段 |
| DSRL-G22 | Fail | I/O/parser/compiler/stage 响应 deadline/cancel 且不部分 publish |
| DSRL-G23 | Fail | shutdown 在卸载前终止 intake、worker、wake、callback 与 Level lease |
| DSRL-G24 | Fail | poison/panic/hung worker 成为 terminal fault，不冻结或静默继续 |
| DSRL-G25 | Fail | disconnect health sticky，manager generation 切换后可 resubscribe/reconcile |
| DSRL-G26 | Fail | pending snapshot 覆盖全部 phase，并按 canonical order 输出 |
| DSRL-G27 | Fail | accepted request 可按 OperationId 查询 terminal disposition |
| DSRL-G28 | Fail | ABI 不再只靠四个 counts 表达 instance mutation |
| DSRL-G29 | Fail | inspection 精确 dirty；full invalidation 只用于标记的 overflow/gap |
| DSRL-G30 | Fail | diagnostics 覆盖 depth/age/bytes/drop/gap/cancel/waste/retry/LKG/duration |
| DSRL-G31 | Fail | 真实 loaded play scene 修改执行 replace 而非 append |
| DSRL-G32 | Fail | unrelated asset、双 project、双 PIE、双 Level 互不串扰 |
| DSRL-G33 | Fail | 1/1K/100K 矩阵绑定 event/instance/task/RSS/allocation/lock/frame 指标 |
| DSRL-G34 | Fail | 0/10/1000 ms/hung worker 证明 single-flight、bounded resident/teardown |
| DSRL-G35 | Fail | gap/capacity/parse/schema/World churn/remove-recreate fault matrix 通过 |
| DSRL-G36 | Fail | Runtime04 focused reload 获得 current-source managed Windows receipt |
| DSRL-G37 | Fail | Runtime08 compiled transaction 获得 current-source managed Windows receipt |
| DSRL-G38 | Fail | Runtime/Editor workflow 验证 selection/override/reference/remove/failure/recovery |
| DSRL-G39 | Fail | fingerprint/BuildSet/workload/result 绑定同一 evidence archive |
| DSRL-G40 | Fail | Markdown、finding ID、路径、链接、索引、状态计数与 scoped diff 全部通过 |

## 18. 状态与开放记录

| 项目 | 状态 | 说明 |
|---|---|---|
| Runtime120 current-source review | review_complete | 3 P0 Open / 59 P1 Open / 1 P1 Partial / 16 P2 Open / 40 Gate Fail |
| Runtime53 | superseded_for_current_status | finding ID 保留用于连续追踪，当前状态以本文为准 |
| Production/tests/Cargo/ABI 修改 | pending | 本轮未修改 |
| Runtime04 bounded single-flight failure | open | candidate source 存在，缺 current-source terminal managed validation |
| Runtime08 compiled spawn transaction failure | open | candidate source 存在，缺 terminal managed validation 与完整规模证据 |
| `DSRL-P1-035` actual target reservation | partial | source + direct test + ignored benchmark；未完成 conservative admission 与动态资格 |
| Cargo/Runtime/Editor/benchmark | not_run | review-only；不宣称 green、accepted 或性能领先 Unreal |
| Source currentness | recheck_required | 9 个冻结文件含其他会话/用户 worktree 差异；实施前按 fingerprint 重读 |
