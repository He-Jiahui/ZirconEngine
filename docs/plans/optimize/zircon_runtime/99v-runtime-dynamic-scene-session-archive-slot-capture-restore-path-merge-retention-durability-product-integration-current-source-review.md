---
title: Runtime Dynamic Scene Session Archive、Slot、Capture/Restore、Path、Merge、Retention、Durability 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime121
review_date: 2026-08-23
baseline_head: 1354e50da53db3dad1dc25a6c9e375942ba04d35
baseline_epoch: 368
supersedes:
  - docs/plans/optimize/zircon_runtime/52-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-review.md
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/document
  - zircon_runtime/src/scene/level_system
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io
  - zircon_runtime/src/core/resource/io/atomic_file
  - zircon_runtime/src/core/resource/io/transaction
  - zircon_runtime/src/dynamic_api
  - zircon_runtime_interface/src
  - zircon_editor/src
  - zircon_app/src
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene_session
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_core.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_manifest.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_mutation.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/level_apply.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene/session_capture_persistence.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene/session_load_query_path.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene/session_retention_mutation_merge.rs
  - zircon_runtime/tests/plan11_scene_serialization_contract.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/52-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-22-dynamic-scene-session-archive-artifact.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-dynamic-scene-session-indexed-transaction.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SaveGameSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SaveGameSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameplayStatics.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameFramework/AsyncActionHandleSaveGame.cpp
  - dev/godot/core/io/resource_saver.h
  - dev/godot/core/io/resource_saver.cpp
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Fyrox/fyrox-core/src/visitor/reader/binary.rs
  - dev/Fyrox/fyrox-core/src/visitor/writer/binary.rs
  - dev/Fyrox/fyrox-impl/src/scene/mod.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_asset_spawner.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeProfile.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeComponent.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeStack.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99v · Runtime Dynamic Scene Session Archive Current Source Review

## 1. 结论

当前 Session Archive 不是空壳。dense slot rows、`BTreeMap` 主索引、updated/tag 次级索引、`swap_remove` 修复、canonical serialization、lineage/revision-bound merge/prune plan、不可变 sealed artifact、512 MiB 输出上限和 bounded keyed writer lane 都是真实底座；25 个直接测试也覆盖正常 roundtrip、stale in-memory plan、同进程 path generation ordering、preview/commit、merge、retention 与 Level apply。这些底座应被吸收到产品服务，不应退回“把 World 直接写成 JSON”的临时实现。

但 Runtime52 之后，完整 565 文件 production owner、25 文件 direct tests 和 19 文件五引擎参考集合均未发生源码变化，当前 worktree 也没有这些 owner/direct-test 路径的未提交差异。当前产品反向搜索仍只找到 `scene/dynamic_scene/mod.rs` 与 `scene/mod.rs` 的 16 行重导出；普通 Runtime Session、Runtime ABI、Editor、App 和 plugin 没有 archive service/store/restore coordinator consumer。因此 547 个 public function 仍是一套孤立的 facade 组合矩阵，不是 SaveGame、Checkpoint、Session Recovery 或 Editor Recovery 产品链。

耐久性缺口同样没有变化。Session 私有 writer 仍只 `flush()`，依赖进程内 `OnceLock<Mutex<HashMap<...>>>` 判断 path revision，backup/restore/cleanup 多处吞错，没有 file/parent `sync_all`、journal、startup recovery、persistent digest CAS、跨进程 lease 或 transaction disposition。更关键的是，同仓 `core::resource::io::{atomic_file, transaction}` 已经拥有 file/parent sync、journal、owner lock、rollback/recovery 与故障测试；继续在 Session 的 565 文件树中修补第二套弱 I/O authority 会扩大漂移，正确方向是让 Session Archive Store 复用唯一 durable transaction primitive。

restore 仍未形成产品事务：apply-to-world 是 append spawn；level replacement 与 metadata publication 分两步；没有 World/Level epoch CAS、安全点、quiesce、participant preflight、rollback 或 resume。diff 仍是重新 capture 后只返回布尔值和计数；merge 只处理 slot ID 两方碰撞；retention 只处理 slot 数量、protected ID 与可选 tag；path query 仍完整解析嵌套 scene payload。

本轮裁决为 **0 P0、60 P1 Open、16 P2 Open、40 Gate Fail**，没有任何 Closed 或 Partial。没有 P0 只因为当前仍无真实产品 consumer，不代表能力达到产品级。Runtime40 继续拥有 SaveGame/Checkpoint/Participant/Platform/Cloud 总合同，Runtime05 继续拥有 World/ECS 与 DynamicScene capture/spawn core，Runtime41 继续拥有统一 operation；Runtime121 只拥有 Session Archive 的 slot/path/artifact/merge/retention/restore/durability 接入纵切面。

本轮只做 current-source 静态审查与文档记录，没有修改 production、tests、Cargo、ABI 或参考源码，没有运行 Cargo、真实 Runtime/Editor、fault injection、crash recovery、跨进程竞争、慢盘、soak 或 benchmark。MVP `00` 仍为 `in_progress`，F0-F5 仍 blocked；三份相关 failure 仍为 `open`，本文不把 source-local candidate 或静态 guard 写成 accepted milestone，也不展开 tooling 优化。

## 2. 审查边界与物理冻结

| 范围 | 文件 / 行 / bytes / tests / ignored / dirty | fingerprint |
|---|---:|---|
| Session production owner | 565 / 10,510 / 360,657 / 7 / 0 / 0 | `771b808da25814d2d6746cd44dc4f7e8d691677cd32898bec4947ce5727d3a4f` |
| focused direct tests / guards | 25 / 6,502 / 244,347 / 126 / 0 / 0 | `49b88f71669c992575e59d0dfc3878516dea55a4534cd63f33758aab6a394a5d` |
| capture/spawn/document/Level support | 22 / 3,912 / 140,400 / 29 / 2 / 1 | `3e48180876bbd5031adfa59d0cfd8b07439a5f979fc910cbf5ff314a47e7d4a2` |
| shared bounded keyed I/O owner | 5 / 1,656 / 55,684 / 24 / 0 / 0 | `dcb3a8330326f3459d1d0eb4e85f3a711222a398e16785020cdf2db8b83cd497` |
| 五引擎参考实现 | 19 / 13,975 / 528,190 / 14 / 0 / 0 | `97e1ac5eeb38ede917aa4dc1add4fb1b60a52b345a69fdbefe8064436407ba78` |

fingerprint 算法：仓库相对路径转 `/`、排序去重，以 `path|lowercase per-file SHA-256` 逐行编码，LF 连接且末尾无 LF，再计算 UTF-8 SHA-256。它只冻结本轮实际读取集合，不是 archive digest、BuildSet、schema identity、ABI identity 或 release identity。

production owner、direct tests 与 reference corpus 的文件、bytes 和 fingerprint 与 Runtime52 完全相同；从旧基线 `bea1acf91b909525ab1759e2c800858b0eda6528` 到当前 HEAD，也没有命中这些 owner/direct-test 路径的 commit diff。support 行改为显式冻结 `dynamic_scene/{scene,document}` 与 `level_system` 的全部 22 个 Rust 文件，因此其规模不能与 Runtime52 当时的选择子集直接比较；其中一个 transaction test 正有外部 worktree 改动，实施前必须重冻。

Runtime52 将同一 production 指纹下的文件长度分布记为 `<=10/<=15/<=25 = 243/351/505`、`>100 = 6`。本轮用 `Get-Content` 逻辑行逐文件复算为 **223/327/481/8**；这是测量口径纠正，不是源码进展。public surface 仍为 547 个函数、367 个不同函数名；以函数名前缀计数，`preview/capture/import` 仍为 138/43/24，`apply/restore/diff` 各 18。

产品搜索在 `zircon_app/src`、`zircon_runtime/src`、`zircon_editor/src`、`zircon_runtime_interface/src` 与 plugin source 中对 `RuntimeSessionArchive*`、`SessionRestoreCoordinator`、`SessionArchiveStore`、`session_archive` 共得到 2,189 个命中；排除 owner 与测试后仅余 16 个命中，全部是 `scene/dynamic_scene/mod.rs` 和 `scene/mod.rs` 重导出。报告、failure、test-only caller 与 re-export 均不计产品 consumer。

## 3. 当前源码裁决

| 主题 | 当前直接证据 | 状态裁决 |
|---|---|---|
| archive identity/index | format 固定 v1；lineage/generation 来自 process-global `AtomicU64`；dense rows + BTree primary/secondary indexes 与 canonical view 可保留 | in-memory 底座保留；persistent identity/migration Open |
| artifact | 同一 artifact 同时强持 payload、manifest、statistics、hash index 与完整 serialized bytes；seal mutex 跨完整 validate/serialize/build | deterministic cache 可保留；RSS/锁粒度/streaming Open |
| writer admission | shared lane 有 count/retained-bytes、deadline、supersede、fence、shutdown 和 terminal 测试 | 通用 queue 底座已存在；archive 在 admission 前推进 path generation，operation/product contract Open |
| atomic path save | temp `BufWriter::flush()`；全局 revision mutex 跨 backup/rename；cleanup/restore 多处 `let _`；path identity 使用 lossy/lowercase string | crash durability、cross-process CAS、TOCTOU 与 disposition 全部 Open |
| bounded load | 文件长度和 reader 只限制 512 MiB，随后 `serde_json::from_reader` 一次性构造完整对象图 | slot/entity/string/depth/time/allocation budget Open |
| path mutation | `load_or_empty/load -> mutate -> seal/save` 整份 RMW | 跨进程丢更新、small mutation 整份重写 Open |
| restore | `apply_to_world` 调 `spawn_into`；replace 创建空 World 后 spawn，再分别 replace world 与 set metadata | Replace/Merge/Instantiate、safe point、epoch CAS、atomic publication Open |
| diff | 重新 capture target DynamicScene，只返回 equality 与 entity/resource counts | bounded structured delta/provenance Open |
| merge | generation/revision-bound in-memory plan；冲突仅 Reject/Keep/Replace slot ID | stale local plan 底座保留；base-aware persistent merge Open |
| retention | max slots、protected IDs、optional tag、caller wall-clock ordering；protected 超预算可保留超过上限 | bytes/age/quota/pressure/tombstone/violation Open |
| stronger shared primitive | `core::resource::io` 已有 file/parent sync、journal、owner lock、recovery 和 fault tests | Session 必须消费共享 owner，不再扩建私有 atomic writer |
| tests | 126 个 direct tests 证明正常局部行为；对 `sync_all/fsync/journal/crash/disk-full/permission/power-loss/flock/multi-process` 定向搜索为零 | fault/cross-process/crash/product evidence 缺失 |

## 4. Owner 与五引擎边界

| Owner / 参考 | 当前事实与可采用原则 | Runtime121 边界 |
|---|---|---|
| Runtime04 / Runtime05 | schema/artifact、World/ECS、DynamicScene compile/preflight/publish | Session 消费，不复制 serializer、ECS snapshot 或 spawn core |
| Runtime22 / Runtime24 / Runtime25 | clock domain、stable identity、path/VFS/sandbox/atomic I/O | Session 只定义 archive 所需依赖与门禁 |
| Runtime40 | SaveGame/Checkpoint、participant、platform/cloud/server 产品总合同 | 唯一产品 owner 和未来 consumer；Session 不另造 SaveGame |
| Runtime41 | admission、prepare/apply、progress/cancel、terminal retention、shutdown | 替代 Session 私有 poll/task 产品模型 |
| Unreal `ISaveGameSystem` / `GameplayStatics` | serialization 与 platform/user slot store 分层；exists/list/save/load/delete、platform user、async callback 与同 pipe ordering | 主导产品/平台边界；接口本身不是 fsync、journal 或 cloud conflict 证明 |
| Godot `ResourceSaver` / `PackedScene` | format saver/path flags 与 pack/instantiate 分层；SceneState 保存 node、instance、property、connection 与 editable state | 主导资源/场景分层；不是 SaveGame participant 或 durable slot transaction |
| Bevy resolved scene / world serialization | dependency resolve 后 apply；`WorldInstanceSpawner` 持 asset-to-instance、instance entity map，Modified 对已注册实例先 despawn owned entities 再 respawn | 主导 instance identity、entity remap 与 registered-only update；不提供跨进程 durability |
| Fyrox Visitor / Scene | explicit region/field visitor、binary reader/writer、scene async load/finish/resource resolution | 主导 schema traversal 与 staged load；不解决 slot policy、CAS 或 crash recovery |
| Unity Graphics Volume | authoring `VolumeProfile/Component` 与 transient evaluated `VolumeStack` 分离，stack 可 reload/dispose | 只作为“不要持久化派生 runtime state”的负向边界，不是通用 SaveGame 参考 |

参考 corpus 与 Runtime52 指纹一致。其 14 个 test declaration 主要来自 Bevy/Fyrox 文件内局部测试；选定 Unreal、Godot 与 Unity Graphics 文件没有等价的 Zircon archive crash/restore 产品 fixture。参考源码只定义架构下限，不能替代 Zircon 自己的动态资格证据。

## 5. P1：产品、API、Operation、并发与耐久

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| DSA-P1-001 | Open | 无产品 consumer；建立由 Runtime40 持有的唯一 `RuntimeSessionArchiveService`。 |
| DSA-P1-002 | Open | 547-function facade 组合爆炸；硬切为少量 service 方法、typed request、selector 与 outcome。 |
| DSA-P1-003 | Open | 任意裸 `Path` 直接进入 API；产品路径改为 platform/project/principal scoped store，裸路径仅留受控 import/export。 |
| DSA-P1-004 | Open | slot 无 principal/project/profile/server authority；每次 operation 强制 typed scope/capability。 |
| DSA-P1-005 | Open | 未接 Runtime41 operation；统一 ticket、deadline、cancel、progress、harvest、retention 与 shutdown。 |
| DSA-P1-006 | Open | load/save/seal/query/capture/diff 可同步执行重工作业；产品 API 默认异步且声明预算/线程。 |
| DSA-P1-007 | Open | completion 依赖 `take_outcome` 轮询；改用统一 terminal observer/stream 与唯一 harvest owner。 |
| DSA-P1-008 | Open | 无产品 shutdown/reopen/recovery 策略；service 定义 drain/cancel/timeout/restart disposition。 |
| DSA-P1-009 | Open | preview 可触发完整 validate/seal/serialize；声明 exact/approximate、成本等级与 side-effect-free 预算。 |
| DSA-P1-010 | Open | receipt 缺 operation/scope/expected-current revision/durability/correlation identity；统一审计 receipt。 |
| DSA-P1-011 | Open | path RMW 可跨进程丢更新；加入持久 revision/content digest CAS 与 typed conflict。 |
| DSA-P1-012 | Open | path revision 只在 process `HashMap`；authority 进入持久 manifest/store transaction。 |
| DSA-P1-013 | Open | 全局 mutex 跨 backup/rename I/O；改为 per-store/per-slot transaction 与短临界区。 |
| DSA-P1-014 | Open | 文件内容只 flush；terminal Durable 前完成共享 primitive 的 file sync。 |
| DSA-P1-015 | Open | rename 后无 parent/store metadata sync；失败不得返回 Durable。 |
| DSA-P1-016 | Open | backup/restore/cleanup 吞错；补偿失败转为 `Unknown/RecoveryRequired` 并保留证据。 |
| DSA-P1-017 | Open | 无 journal/startup recovery；复用 durable transaction journal、commit point、幂等恢复与 quarantine。 |
| DSA-P1-018 | Open | lossy/lowercase path key 不是 file identity；复用 Runtime25 canonical path/file-id owner。 |
| DSA-P1-019 | Open | 无 symlink/junction TOCTOU 防护；目录 handle/no-follow/file-id recheck 与 sandbox scope。 |
| DSA-P1-020 | Open | 无跨进程锁；store 提供 owner lock/lease 和 owner death 语义。 |
| DSA-P1-021 | Open | in-memory lineage 不能阻止重启/旧进程覆盖；持久比较 archive id/revision/digest。 |
| DSA-P1-022 | Open | writer 在 admission 前 prepare 并推进 generation；先成功保留 capacity，再创建不可回滚 intent。 |
| DSA-P1-023 | Open | direct save 绕过 lane；所有产品 write 进入同一 transaction scheduler。 |
| DSA-P1-024 | Open | retry 无幂等 request key；commit 带 operation/request id 和可查询 terminal receipt。 |
| DSA-P1-025 | Open | 无外部修改检测；最终 authority 比较持久 revision/digest，mtime/file-id 只作优化。 |
| DSA-P1-026 | Open | disk-full/permission/partial failure 无统一 disposition；区分 unchanged/rolled-back/committed/unknown/recovery-required。 |
| DSA-P1-027 | Open | temp/backup 无 store transaction identity/retention/orphan owner；journal 统一拥有 artifact lifecycle。 |
| DSA-P1-028 | Open | cancel 无 phase 语义；区分 requested/before-start/too-late/committed/compensated/unknown。 |
| DSA-P1-029 | Open | atomic generation 与 saturating revision 无耗尽策略；不可回绕并在边界 fail closed。 |
| DSA-P1-030 | Open | 无 last-known-good/quarantine；corrupt/unknown artifact 进入标准恢复与 operator receipt。 |

## 6. P1：Schema、Metadata、Identity、Query 与存储

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| DSA-P1-031 | Open | load 只限总 bytes；增加 slot/entity/component/resource/tag/string/depth/count/CPU/time 预算。 |
| DSA-P1-032 | Open | 无逐段 admission；采用 header/table/chunk，并在分配前验证每段预算。 |
| DSA-P1-033 | Open | slot ID 只 trim/非空；定义长度、Unicode normalization、control、reserved namespace、scope 与 canonical key。 |
| DSA-P1-034 | Open | metadata 为自由字符串；引入强类型 `SessionArchiveManifestV2`。 |
| DSA-P1-035 | Open | machine-local project path/asset URI 进入 portable archive；持久化 content identity，resolution 留在 store context。 |
| DSA-P1-036 | Open | caller wall clock 决定 latest/retention；排序使用 store commit sequence，wall clock 只展示。 |
| DSA-P1-037 | Open | 无 clock domain/tie-break 合同；记录 clock source、capture sequence 与 deterministic policy。 |
| DSA-P1-038 | Open | archive 外层只接受精确 v1；建立 v1-vN migration、downgrade/read-only 与 fixture corpus。 |
| DSA-P1-039 | Open | nested scene header 不闭合 provider/component schema；manifest 绑定 BuildSet/provider/type schema/migration capability。 |
| DSA-P1-040 | Open | unknown provider/field 无 round-trip preservation；显式 fail-closed/read-only/opaque chunk policy。 |
| DSA-P1-041 | Open | 无 end-to-end checksum/authentication；每 chunk 和 root manifest 使用 stable digest/Merkle 关系。 |
| DSA-P1-042 | Open | 无 compression/encryption/chunking；由 store policy 注入 codec/key/chunk 策略。 |
| DSA-P1-043 | Open | touch/metadata/remove 重写完整 archive；immutable chunks + manifest transaction + compaction。 |
| DSA-P1-044 | Open | merge/copy/import 不做 content-addressed sharing；chunk digest、引用与安全 GC。 |
| DSA-P1-045 | Open | artifact 同时保留对象图与完整 bytes；流式 seal、分段 resident budget 与可释放 cache。 |
| DSA-P1-046 | Open | seal mutex 跨完整重工作业；只锁状态转换，重工作业在 immutable snapshot 上运行。 |
| DSA-P1-047 | Open | pretty JSON 再复制整份 String；debug/export 使用 bounded streaming writer 并标注非产品格式。 |
| DSA-P1-048 | Open | query 每次完整解析 scene payload；提供 header/index-only query。 |
| DSA-P1-049 | Open | display/tag query 重复线性扫描/分配；immutable secondary index + cursor/max-results。 |
| DSA-P1-050 | Open | `DefaultHasher` index 不能持久复用；stable index schema/digest，并可从 manifest 确定性重建。 |

## 7. P1：Capture、Restore、Diff、Merge 与 Retention

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| DSA-P1-051 | Open | capture 同步扫描 World；消费 Runtime05 增量/分区 snapshot 与预算，不复制 ECS 扫描器。 |
| DSA-P1-052 | Open | reflected serializable 集合不是产品 participant；消费 Runtime40 participant registry/provider fingerprint。 |
| DSA-P1-053 | Open | “restore to world”实际 append spawn；request 显式区分 Replace/Merge/Instantiate。 |
| DSA-P1-054 | Open | world replacement 与 metadata 分两步发布；候选 world/metadata/runtime reset 同事务 publish。 |
| DSA-P1-055 | Open | restore 无 world/level epoch CAS；绑定 world/level/build/schema epoch 与 safe-point token。 |
| DSA-P1-056 | Open | 无 quiesce/rollback/resume；`SessionRestoreCoordinator` 负责 freeze、preflight、publish、rollback、resume。 |
| DSA-P1-057 | Open | diff 只返回全量 equality/counts；提供 bounded typed delta、provenance、cursor 与 unsupported owners。 |
| DSA-P1-058 | Open | capture/diff/restore 无统一 deadline/cancel/progress；全部进入 Runtime41 operation phase model。 |
| DSA-P1-059 | Open | merge 仅处理 slot-id 两方碰撞；base-aware three-way merge、schema/provider conflict 与 durable publication。 |
| DSA-P1-060 | Open | retention 仅 slot count/protected/tag；补 bytes/age/quota/pressure/class/lease/tombstone/GC/compaction/reason。 |

`protected slots > max slots` 等不可满足预算必须返回 policy violation，不能把“不删除 protected slot”冒充 retention 成功。merge/retention commit 必须验证持久 expected revision，而不是只验证同一进程内 lineage。

## 8. P2：可维护性、诊断与产品可用性

| ID | 状态 | 差距与建议 |
|---|---|---|
| DSA-P2-001 | Open | 481/565 production 文件不超过 25 行；按 store/artifact/transaction/query/restore/retention 收敛，删除组合 wrapper。 |
| DSA-P2-002 | Open | 函数名拼接全部选项；使用 request builder、selector enum 与 options struct。 |
| DSA-P2-003 | Open | basic/metadata 平行入口；metadata 策略成为显式 payload/enum。 |
| DSA-P2-004 | Open | named/selected 平行入口；统一 `SlotSelector`，一次 resolve 产生 stable selected revision。 |
| DSA-P2-005 | Open | loaded/source_path 平行入口；store handle 承载 identity，path 只作边界 adapter。 |
| DSA-P2-006 | Open | global/tag retention 平行入口；统一 predicate/policy AST 与 explainable plan。 |
| DSA-P2-007 | Open | display search 为 case-sensitive substring；normalized key、locale policy、cursor 与预算。 |
| DSA-P2-008 | Open | query 频繁 Vec 分配/排序；iterator/visitor/cursor 与 stable ordering/allocation budget。 |
| DSA-P2-009 | Open | error 缺 phase/store/safe path context；redacted typed diagnostic。 |
| DSA-P2-010 | Open | statistics 缺 RSS/I/O/latency 维度；记录 resident/copied/queue/fsync/recovery/compaction。 |
| DSA-P2-011 | Open | source-shape guards 容易冒充工程证据；与 behavior/fault/cross-process/performance 分栏。 |
| DSA-P2-012 | Open | open failure/历史 output 容易被误读为 accepted；索引统一显示 pending validation。 |
| DSA-P2-013 | Open | preview 不声明成本；暴露 estimated work/bytes、exactness 与 budget exceeded。 |
| DSA-P2-014 | Open | import/export 无 trust classification；区分 trusted local/untrusted exchange/debug/platform store。 |
| DSA-P2-015 | Open | manifest 无 redaction/display 分层；operator/UI/portable projections 分离。 |
| DSA-P2-016 | Open | 无统一 capability 状态；缺 store/migration/provider/recovery 时 fail closed 为 Unavailable/Degraded。 |

## 9. 目标架构

```text
Runtime40 SaveGame / Checkpoint product owner
  -> RuntimeSessionArchiveService
     -> Scoped SessionArchiveStore(platform + project + principal)
        -> core::resource::io::transaction durable primitive
           owner lock + expected revision/digest CAS
           journal + file/parent sync + startup recovery
     -> SessionSnapshotCompiler(Runtime40 participants + Runtime05 snapshot)
     -> SessionArchiveManifestV2 + immutable chunks
     -> SessionArchiveQueryIndex(header/index-only + cursor + budgets)
     -> SessionRestoreCoordinator
        resolve -> quiesce -> preflight -> stage -> safe-point CAS publish
        rollback/resume -> terminal receipt
     -> SessionArchiveRetentionManager(policy + tombstone + GC + compaction)
     -> Runtime41 OperationService
```

`SessionArchiveManifestV2` 至少持有 archive/store/project/principal/slot identity，revision/parent/commit sequence/content root digest，BuildSet/engine/schema/provider fingerprints，world/level/epoch/capture reason，clock domain/capture sequence，chunk table/codec/key id/sizes/schema，participant outcomes、compatibility、restore policy、durability/platform/cloud disposition 与 operation/correlation identity。caller 不能自行声明 `durable=true`。

## 10. 分层重构里程碑

### M121-0：产品语义、Owner 与删除目标

- 冻结 Runtime04/05/22/24/25/40/41/121 owner 边界；
- 生成 565-file facade 等价/consumer 矩阵，标记保留、迁移、删除；
- 定义 manifest v2、scope、slot identity、revision/digest、operation receipt；
- 在真实 consumer/store/provider 存在前 capability fail closed。

### M121-1：Shared Store、CAS 与 durable transaction

- Session Store 适配 `core::resource::io::transaction`，不复制私有 journal；
- 加入 cross-process owner lock、expected revision/digest CAS 与 startup recovery；
- 故障注入覆盖 write/flush/sync/rename/journal/cleanup/recovery 每阶段；
- 删除 process-global path revision map 作为正确性 authority。

### M121-2：Manifest、Chunks、Migration 与 bounded load

- 整份 JSON 硬切为 manifest + immutable chunks + stable digest；
- 结构预算、streaming parse、provider/schema migration、unknown preservation；
- current/legacy/future/corrupt/hostile fixture corpus；
- compression/encryption/key policy 只由 store/provider 注入。

### M121-3：Participants 与 Restore Coordinator

- 接入 Runtime40 participants 和 Runtime05 snapshot；
- 显式 Replace/Merge/Instantiate 与 world epoch/safe-point CAS；
- quiesce/preflight/stage/atomic publish/rollback/resume；
- structured diff、partial compatibility 与 unsupported participant 可见。

### M121-4：Query、Merge、Retention 与 GC

- header/index-only query、cursor、stable ordering 与结果预算；
- base-aware three-way merge 与 typed conflicts；
- bytes/age/pressure/class/tombstone/lease/compaction/content GC；
- small mutation 不再重写无关 chunks。

### M121-5：产品接入与 API hard cut

- Runtime40 成为唯一产品 consumer，Editor/App 只经产品 command 调用；
- 接入 Runtime41 operation/diagnostics/receipt；
- 删除平行同步 authority 与组合 facade；
- 不保留旧 path 写入口的 compatibility re-export/shim。

### M121-6：恢复、规模、性能与发布资格

- 1/1K/100K slots，small/medium/max-policy snapshot；
- 0/10/1000 ms storage latency、write storm、two-process conflict、restart recovery 与 24h soak；
- p50/p95/p99、RSS peak、bytes copied/read/written、queue wait、fsync、recovery、compaction、restore pause；
- 与参考引擎使用同语义、同硬件、同 durability level，证据不足不得宣称优于 Unreal。

## 状态与产出记录

每个里程碑只在完整测试阶段通过后记录一次 accepted outcome；当前没有 accepted implementation evidence。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|
| M121-0 | 产品语义、Owner、manifest 与删除目标 | pending | - | 未进入实施 |
| M121-1 | shared durable store、CAS、journal/recovery | pending | - | 三份相关 failure 仍 open |
| M121-2 | manifest/chunks/migration/bounded load | pending | - | 无 fixture/动态证据 |
| M121-3 | participants 与 restore transaction | pending | - | 无产品 consumer |
| M121-4 | query/merge/retention/GC | pending | - | 无规模资格 |
| M121-5 | Runtime40/41 产品接入与 API hard cut | pending | - | 无产品闭环 |
| M121-6 | crash/cross-process/soak/performance | pending | - | 未执行 qualification |

## 11. 验收门禁

| Gate | 状态 | 验收条件 |
|---|---|---|
| DSA-G01 | Fail | 唯一 service 有真实 Runtime40 consumer；re-export/test-only caller 不计。 |
| DSA-G02 | Fail | platform/project/principal scope 在每个 slot operation 强制校验。 |
| DSA-G03 | Fail | 547-function facade 收敛为 request/selector/service，旧写入口 hard cut。 |
| DSA-G04 | Fail | 缺 store/provider/migration 时 capability fail closed。 |
| DSA-G05 | Fail | 所有重工作业通过 Runtime41 operation，具 deadline/cancel/progress/harvest/shutdown。 |
| DSA-G06 | Fail | terminal receipt 带 operation/scope/expected-current revision/durability disposition。 |
| DSA-G07 | Fail | 同进程并发 RMW 不丢更新，冲突返回 typed outcome。 |
| DSA-G08 | Fail | 两独立进程竞争同 slot 不静默 last-writer-wins。 |
| DSA-G09 | Fail | 外部替换后旧 expected digest 提交被拒绝。 |
| DSA-G10 | Fail | admission 失败不推进可使在途 write 失效的 generation。 |
| DSA-G11 | Fail | data sync 失败不返回 Durable。 |
| DSA-G12 | Fail | directory/store metadata sync 失败进入明确 terminal/recovery-required。 |
| DSA-G13 | Fail | backup/rename/restore/cleanup 任一步失败均不吞错。 |
| DSA-G14 | Fail | 每 journal 阶段断进程后都可幂等恢复或 quarantine。 |
| DSA-G15 | Fail | symlink/junction/path swap 不能逃逸 store scope。 |
| DSA-G16 | Fail | disk-full/permission/read-only/partial write 有确定 disposition。 |
| DSA-G17 | Fail | shutdown 覆盖 drain/cancel/timeout/too-late/restart recovery。 |
| DSA-G18 | Fail | revision/epoch 接近耗尽时 fail closed。 |
| DSA-G19 | Fail | manifest v2 绑定 scope/build/schema/provider/world epoch/content digest。 |
| DSA-G20 | Fail | slot ID 覆盖长度、Unicode、control、reserved namespace、canonical collision。 |
| DSA-G21 | Fail | load 分配前限制 slots/entities/components/resources/tags/strings/depth/bytes/time。 |
| DSA-G22 | Fail | current/legacy/future/missing-provider/corrupt fixture 有明确结果。 |
| DSA-G23 | Fail | unknown chunk 按策略保留或 fail closed，绝不静默丢失后重写。 |
| DSA-G24 | Fail | chunk/root digest 可检测并定位单点损坏。 |
| DSA-G25 | Fail | compression/encryption/key rotation 由 store policy 管理并有跨版本 fixture。 |
| DSA-G26 | Fail | touch/metadata/remove 不重写无关 scene chunk，crash 后引用一致。 |
| DSA-G27 | Fail | participant registry 证明 required subsystem capture，optional/unsupported 可见。 |
| DSA-G28 | Fail | Replace/Merge/Instantiate 语义互斥且默认 fail closed。 |
| DSA-G29 | Fail | stale world epoch/safe point/BuildSet 时 restore 拒绝。 |
| DSA-G30 | Fail | restore 故障不暴露半发布 world/metadata/runtime state，可 rollback/resume。 |
| DSA-G31 | Fail | structured diff 有 provenance/cursor/budget/unsupported owner。 |
| DSA-G32 | Fail | three-way merge 报告 base/ours/theirs、schema/provider conflicts 与 digest。 |
| DSA-G33 | Fail | retention 覆盖 bytes/age/pressure/class/protection/lease/tombstone，预算不可满足返回 violation。 |
| DSA-G34 | Fail | manifest/status/selector query 不解析无关 scene payload。 |
| DSA-G35 | Fail | query 有 stable ordering、cursor、max-results、allocation/time budget。 |
| DSA-G36 | Fail | 1/1K/100K slots 记录 latency/RSS/bytes/index rebuild。 |
| DSA-G37 | Fail | 0/10/1000 ms storage latency 下 main/runtime thread 无未声明阻塞。 |
| DSA-G38 | Fail | cross-process storm/restart/corrupt corpus/24h soak 无丢更新或无界增长。 |
| DSA-G39 | Fail | source-shape/unit/fault/cross-process/crash/performance 证据分栏。 |
| DSA-G40 | Fail | “优于 Unreal”使用同语义、同硬件、同 durability 与可复跑 raw evidence。 |

## 12. 当前状态与首个实施切片

- Review：`review_complete`。
- Implementation：`pending`。
- Finding：`0 P0`、`60 P1 Open`、`16 P2 Open`。
- Gate：`40 Fail`。
- Source delta：Session owner/direct tests 相对 Runtime52 为零；薄文件分布仅为测量纠正。
- Product delta：仍只有两层 re-export，普通 Runtime/Editor/App/Interface/plugin consumer 为零。
- Failure：Runtime04 archive artifact、Runtime08 indexed transaction、Runtime11 bounded async I/O 三份 handoff 均保持 `open`。
- 首个实现切片：先执行 M121-0 owner/manifest/facade deletion matrix，并在 M121-1 让 Session Store 复用 shared durable transaction；不得继续增加组合 facade 或在 Session 内复制第三套 atomic I/O。

