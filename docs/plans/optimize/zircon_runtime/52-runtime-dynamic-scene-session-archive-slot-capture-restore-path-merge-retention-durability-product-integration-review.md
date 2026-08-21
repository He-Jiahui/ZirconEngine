---
title: Runtime Dynamic Scene Session Archive、Slot、Capture/Restore、Path、Merge、Retention、Durability 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime52
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/document
  - zircon_runtime/src/scene/level_system
  - zircon_runtime/src/core/runtime/tasks
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene_session
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_core.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_manifest.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_mutation.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/level_apply.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene
  - zircon_runtime/tests/plan11_scene_serialization_contract.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 52 · Runtime Dynamic Scene Session Archive、Slot、Capture/Restore、Path、Merge、Retention、Durability 与 Product Integration 工程化差距

## 1. 结论

`zircon_runtime/src/scene/dynamic_scene/session`不是空壳。当前实现已经有带lineage/revision的内存archive、dense slot rows与主/次索引、确定性manifest顺序、slot capture/restore/diff、preview/commit、stale plan拒绝、merge/retention、typed scene envelope、不可变sealed artifact、512 MiB输出上限，以及按任务数和retained bytes admission的bounded writer lane。这些是可保留的底座，不能在后续重构中退回“直接把World写JSON”的临时实现。

但565个production文件、547个public function和367个不同public function name没有形成产品服务，反而把`world/level × named/selected × loaded/source_path × basic/metadata × preview/commit × retention scope`展开成数百个薄facade。除`scene/mod.rs`、`dynamic_scene/mod.rs`重导出和一份integration test外，本轮没有找到普通Runtime、Editor或App产品消费者。现状更接近一个巨大、孤立的API组合矩阵，不是SaveGame/Checkpoint/Session Recovery产品链。

更严重的是，当前“atomic”写入只保证单次rename的局部路径替换，不保证工程级事务和崩溃耐久。path mutation采用load-all → mutate → save；没有跨进程lock、持久expected revision或file digest CAS，因此并发进程可合法last-writer-wins并丢更新。进程内revision map在重启后消失，全局mutex还跨backup/rename I/O持锁；文件只`flush()`而不`sync_all()`，父目录也不fsync；backup清理与恢复错误被忽略，并且没有journal与启动恢复。这样的合同不能被称为durable save。

restore同样没有产品语义闭合：apply-to-world是增量spawn，不是replace；level替换与metadata publication分离；没有world epoch CAS、安全点、系统quiescence、participant capture/restore、rollback或resume。merge只处理slot-id碰撞，retention只处理slot数量，path query每次都完整解析整个archive及嵌套scene。外层schema v1、原始字符串metadata与512 MiB字节上限，也不足以承担版本迁移、恶意输入预算、平台用户、云冲突、加密压缩和大规模存档。

因此本轮登记 **0项新增P0、60项P1、16项P2和40项验收门禁**。没有新增P0不是因为实现已经达到产品级，而是因为本轮确认它尚无真实产品consumer；Runtime40继续拥有SaveGame/Checkpoint/Participant/Platform/Cloud产品总合同，Runtime05继续拥有World/ECS与DynamicScene capture/spawn core。Runtime52只拥有Session Archive的slot、path、artifact、merge、retention、restore、durability与产品接入合同。进入任何可见Save/Load产品链前，本报告的durability、concurrency与restore P1必须先清零。

本轮仅做静态review；没有修改production、tests、Cargo或reference source，没有运行Cargo、Editor、fault injection、crash recovery、跨进程竞争、慢盘、soak或benchmark。`runtime/04`、`runtime/08`、`runtime/11`三份failure记录仍为open，本报告不把其中的实现提交或source-shape tests改写为accepted milestone。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| Session production owner | 565 / 10,510 / 360,657 / 7 | SHA-256 `771b808da25814d2d6746cd44dc4f7e8d691677cd32898bec4947ce5727d3a4f` |
| focused direct tests | 25 / 6,502 / 244,347 / 126 | SHA-256 `49b88f71669c992575e59d0dfc3878516dea55a4534cd63f33758aab6a394a5d`；0 ignored |
| DynamicScene support | 22 / 3,327 / 120,526 / 24 | SHA-256 `e9618b9b0b48b64d2925497bdd9d0d5348663e1fe60838b8b04e5eee54981551`；1 ignored |
| reference corpus | 19 / 13,975 / 528,190 | SHA-256 `97e1ac5eeb38ede917aa4dc1add4fb1b60a52b345a69fdbefe8064436407ba78` |

fingerprint将相对路径转换为`/`、排序去重，以`path|lowercase per-file SHA-256`编码，LF连接且末尾不追加LF，再计算UTF-8 SHA-256。它只冻结本轮读取集合，不是archive content id、release identity、schema digest或ABI证明。基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

冻结时`zircon_runtime/src/scene/dynamic_scene/session`及本轮direct test路径没有working-tree差异；但support集合中的`zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs`正由MVP会话修改。该文件不改变Session owner的当前代码事实，却使支持链必须在实施前重读，因此`source_recheck_required`保守置true。共享工作树的其他大量修改不在本轮写集，本报告没有暂存、回退或替换它们。

### 2.2 API形态不是工程化证据

| 统计 | 数量 | 判断 |
|---|---:|---|
| public functions | 547 | public surface已大于清晰产品合同所需规模 |
| unique public function names | 367 | 差异不只是re-export，也包含大量手工组合命名 |
| `preview` / `capture` / `import` public verbs | 138 / 43 / 24 | 组合维度被摊到函数名和目录树 |
| `apply` / `slot` / `restore` / `diff` | 18 / 18 / 18 / 18 | 对称数量来自facade矩阵，不等于产品闭环 |
| files `<= 10` / `<= 15` / `<= 25` lines | 243 / 351 / 505 | 505/565文件不超过25行，模块边界主要表达排列组合 |
| files `> 100` lines | 6 | 核心行为被薄wrapper淹没，review与演进成本失真 |

应保留typed request/report，但删除以函数和目录复制表达选项的设计。目标public surface应收敛为service方法、request options、selector、transaction plan和typed outcome；便利调用可以由client builder或生成代码提供，不能继续手写数百个owner不明的入口。

### 2.3 当前真实调用链

```text
scene/mod.rs
  -> dynamic_scene/mod.rs
     -> session/mod.rs
        -> 565-file facade/path_api/slot_* graph
        -> focused tests

普通 Runtime / Editor / App product consumer: 0
```

`zircon_runtime/tests/plan11_scene_serialization_contract.rs`能证明一部分序列化合同可调用，不能证明用户slot、checkpoint、world recovery、platform store或cloud conflict已经接入。报告、re-export与test-only caller都不能作为产品owner。

## 3. 当前应保留的真实底座

1. `RuntimeSessionArchivePayload`使用dense rows、`BTreeMap`主ID索引、updated/time+slot次级索引和tag索引；`swap_remove`后会修复被移动row，canonical serialization按BTree ID顺序枚举。
2. merge、prune和capture-retention plan绑定lineage/revision，旧plan对同一内存target提交时会被拒绝，避免明显的stale plan replay。
3. `RuntimeSessionArchiveArtifact`是不可变共享对象，持有payload、manifest、statistics、index与serialized bytes，并缓存确定性validation/serialization成功或失败。
4. 序列化输出有512 MiB硬上限；writer lane同时限制任务数和retained bytes，正常提交顺序与bounded admission已有测试。
5. typed scene envelope能识别current/legacy header，并对重复slot等基本不变量做验证。
6. DynamicScene spawn已有compile/preflight/commit和隔离staging，失败不会直接把半成品实体逐步泄露到target world。
7. roundtrip、replacement、stale plan、bounded artifact、writer ordering、manifest/index和level apply等正常路径已有回归。

这些能力应被吸收到新的Session Archive Service，不应继续由孤立facade各自复制。

## 4. Owner边界与“不得重复造轮子”

| Owner | 继续拥有 | Runtime52不得重复登记 |
|---|---|---|
| Runtime04 | resource/asset serialization、schema、artifact基础合同 | 通用serializer、资源生命周期与所有asset migration总问题 |
| Runtime05 | World/ECS lifecycle、DynamicScene capture/spawn/reload core | ECS archetype/query与scene spawn底层性能总问题 |
| Runtime22 / 24 | clock domain、stable identity、owner epoch、exhaustion | 全引擎时间与handle总合同 |
| Runtime25 | filesystem/path/URI/VFS/mount/watch/sandbox/atomic I/O | 通用VFS与所有路径安全总问题 |
| Runtime40 | SaveGame/Checkpoint/Participant/Platform/Cloud/Network产品总合同 | 新建第二套SaveGame产品、platform/cloud owner |
| Runtime41 | operation admission/prepare/apply/progress/cancel/harvest/shutdown | 新建session私有异步任务框架 |
| Runtime52 | session archive slot/path/artifact/merge/retention/restore/durability接入 | 只在archive边界给出上游依赖和验收门禁 |

## 5. P1：产品Owner、API与Operation合同

| ID | 差距 | 代码事实与需要重构的内容 |
|---|---|---|
| DSA-P1-001 | 无产品consumer | 565个文件仅被scene root重导出和tests消费；建立唯一`RuntimeSessionArchiveService`，由Runtime40的SaveGame/Checkpoint owner显式持有并发布capability。 |
| DSA-P1-002 | facade组合爆炸 | 547个public function把选项编码进函数/目录；硬切为少量`capture/query/commit/restore/merge/prune`方法与typed request/selector。 |
| DSA-P1-003 | 无platform store边界 | 任意`Path`直接进入session API；接入Runtime40定义的平台/user store，裸路径只保留受控developer/import工具入口。 |
| DSA-P1-004 | 无principal/project/profile作用域 | slot ID没有用户、项目、存档域或server authority；所有读写必须由typed scope与capability admission限定。 |
| DSA-P1-005 | 未接Runtime41 operation | session维护私有writer/poll模型；改用统一OperationTicket、deadline、cancel、progress、completion、retention和shutdown语义。 |
| DSA-P1-006 | 同步重工作业暴露主线程 | load、save、seal、query、capture、diff均可同步执行大对象工作；公开产品API必须默认异步并声明线程/预算。 |
| DSA-P1-007 | completion只能轮询提取 | `take_outcome`没有wake、typed completion stream或统一harvest owner；接入operation completion并保留terminal disposition。 |
| DSA-P1-008 | 无产品shutdown策略 | generic lane行为没有定义flush、cancel、timeout、abandon、reopen与startup recovery；由archive service统一关停。 |
| DSA-P1-009 | preview也执行重序列化 | 多个preview为了报告target/statistics可验证或seal完整archive；preview必须有成本等级、预算与side-effect-free保证。 |
| DSA-P1-010 | receipt缺少因果身份 | report未贯穿operation id、principal、expected/current revision、durability disposition与correlation id；统一为可审计receipt。 |

## 6. P1：并发、原子性与崩溃耐久

| ID | 差距 | 代码事实与需要重构的内容 |
|---|---|---|
| DSA-P1-011 | read-modify-write会丢更新 | `mutate_archive_at_path_atomically`先完整load再save；两个进程可从同一基线提交并last-writer-wins。加入持久revision/digest CAS与冲突outcome。 |
| DSA-P1-012 | revision ordering仅进程内 | `COMMITTED_PATH_REVISIONS`是`OnceLock<Mutex<HashMap<...>>>`，重启、外部writer和多进程均绕过；revision必须进入持久manifest/store事务。 |
| DSA-P1-013 | 全局mutex跨I/O持锁 | backup与rename期间锁住所有path revision，互不相关存档相互阻塞；使用per-store/per-slot transaction与短临界区。 |
| DSA-P1-014 | 文件内容未durable sync | save只调用`flush()`，没有`File::sync_all()`；terminal `Durable`前必须完成平台对应的数据落盘合同。 |
| DSA-P1-015 | 目录entry未durable sync | rename后没有父目录fsync；断电后namespace publication不受保证，需按平台实现directory durability或明确降级。 |
| DSA-P1-016 | backup/restore错误被忽略 | cleanup和restore存在`let _ = ...`；任何补偿失败必须成为typed unknown/recovery-required outcome并保留证据。 |
| DSA-P1-017 | 无journal与启动恢复 | temp/backup阶段没有可重放事务记录；建立prepare journal、commit marker、startup scan、幂等恢复和隔离quarantine。 |
| DSA-P1-018 | path key使用lossy/lowercase | Windows归一化依赖字符串lowercase，不能代表真实file identity/case规则；复用Runtime25 canonical path/file-id owner。 |
| DSA-P1-019 | 无symlink/junction TOCTOU防护 | path验证、创建、backup和rename之间可换向；使用目录handle-relative操作、no-follow/file-id recheck与sandbox scope。 |
| DSA-P1-020 | 无跨进程锁 | 进程内mutex不能协调Editor、Game、Hub或外部工具；store提供可恢复lease/file lock，并定义owner死亡语义。 |

| ID | 差距 | 代码事实与需要重构的内容 |
|---|---|---|
| DSA-P1-021 | lineage只能拒绝同实例旧plan | 重新load会产生新lineage，不能阻止旧进程覆盖新文件；expected archive id/revision/content digest必须持久比较。 |
| DSA-P1-022 | admission失败仍推进write generation | `try_submit`在lane admission前prepare path write；拒绝后generation已前进，可把在途write判旧。prepare必须在成功保留capacity后执行或可回滚。 |
| DSA-P1-023 | direct save绕过lane | 同步save与writer lane并存，两套入口没有统一ordering/admission；所有产品write进入同一transaction scheduler。 |
| DSA-P1-024 | 无幂等request key | retry无法区分“未执行”和“已提交但响应丢失”；commit需operation/request id与可查询terminal receipt。 |
| DSA-P1-025 | 无外部修改检测 | 合法JSON被其他工具替换后仍可被新load接受并覆盖；比较file id、size、mtime仅可优化，最终必须比较持久digest/revision。 |
| DSA-P1-026 | disk-full/permission/partial failure语义不完整 | 错误未统一区分unchanged、rolled-back、committed、unknown、recovery-required；定义transaction disposition并测试每个阶段。 |
| DSA-P1-027 | temp/backup命名与清理无store协议 | 没有多进程唯一transaction identity、保留期和孤儿清理owner；journal必须拥有所有临时artifact。 |
| DSA-P1-028 | cancel没有phase语义 | prepare、serialize、fsync、publish后取消的结果不同；明确cancel requested、too late、committed、compensated和unknown。 |
| DSA-P1-029 | generation/revision耗尽未处理 | global atomic递增与`saturating_add`最终不再区分mutation；使用不可回绕epoch/revision策略并fail closed。 |
| DSA-P1-030 | 无last-known-good与quarantine | corrupt target、失败backup和未知格式没有标准恢复目录/选择策略；store必须保留LKG、隔离坏artifact并给operator receipt。 |

## 7. P1：Archive、Schema、Metadata、Identity与Query

| ID | 差距 | 代码事实与需要重构的内容 |
|---|---|---|
| DSA-P1-031 | load只限制总字节 | 512 MiB上限后仍一次性serde分配；增加slot/entity/component/resource/tag/string/depth/count与CPU/time预算。 |
| DSA-P1-032 | 无逐段admission | manifest、index与scene payload不能先验拒绝高放大输入；采用header/table/chunk结构并在分配前校验每段预算。 |
| DSA-P1-033 | slot ID验证过弱 | 只要求trim后非空；增加长度、Unicode normalization、control char、reserved namespace、scope与canonical key规则。 |
| DSA-P1-034 | metadata是原始可选字符串 | project root、asset URI、display、timestamp和tags缺少typed schema；引入`SessionArchiveManifestV2`与强类型字段。 |
| DSA-P1-035 | machine-local路径进入可移植存档 | project root/asset URI可能泄漏或在另一机器失效；持久化project/content identity，local resolution放store context。 |
| DSA-P1-036 | 调用者时间决定latest/retention | `updated_at`可被回拨、伪造或并列；排序需server/store commit sequence，wall clock仅作display metadata。 |
| DSA-P1-037 | 无clock domain与tie-break合同 | untimed按零值变最旧，同timestamp依赖slot排序；记录clock source、capture sequence与确定性policy。 |
| DSA-P1-038 | 外层archive只有精确v1 | 未定义v1→vN迁移链、downgrade、read-only兼容或unsupported version证据；建立版本注册和fixture corpus。 |
| DSA-P1-039 | component/provider schema不闭合 | nested DynamicScene header不能解决每个component/plugin版本；manifest绑定provider set、type schema和migration capability。 |
| DSA-P1-040 | unknown数据不能round-trip保留 | 缺provider/新字段时没有opaque preservation策略；定义fail-closed、read-only或保留未知chunk，禁止静默丢字段。 |

| ID | 差距 | 代码事实与需要重构的内容 |
|---|---|---|
| DSA-P1-041 | 无端到端checksum/authentication | canonical JSON不校验独立manifest/chunk digest，也不能定位腐坏；每个chunk与root manifest使用稳定digest/Merkle关系。 |
| DSA-P1-042 | 无compression/encryption/chunking | 大存档只能整份明文JSON；由platform/store policy选择压缩、加密、key id、chunk和streaming，不能硬编码算法。 |
| DSA-P1-043 | 小mutation整份重写 | touch、metadata、remove或retention都会重写整个archive；改为immutable chunks + manifest transaction + 后台compaction。 |
| DSA-P1-044 | 无content-addressed sharing | merge/copy/import克隆完整payload，重复scene没有去重；chunk digest允许共享、引用计数与安全GC。 |
| DSA-P1-045 | artifact同时保留对象图与完整bytes | sealed artifact的payload、manifest/index和serialized bytes叠加RSS；引入流式seal、分段resident budget与可释放cache。 |
| DSA-P1-046 | seal持mutex做完整validate/serialize | 最大512 MiB工作会阻塞同artifact并发caller；一次状态转换只锁短临界区，重工作业在不可变snapshot上执行。 |
| DSA-P1-047 | pretty JSON再次整份复制 | `to_versioned_json_pretty`从bytes克隆String，峰值内存继续放大；debug/export使用bounded writer/stream并标注非产品格式。 |
| DSA-P1-048 | query每次完整load嵌套scene | manifest、contains、slot IDs、tag、selector、statistics和status都反序列化全archive；提供header/index-only query。 |
| DSA-P1-049 | manifest display/tag查询线性扫描 | archive Vec上重复扫描并分配结果；编译不可变二级索引，支持visitor/cursor和max-results budget。 |
| DSA-P1-050 | index身份不能持久复用 | artifact内部`DefaultHasher`bucket只适合进程内；持久查询索引必须有stable schema/digest并可从manifest确定性重建。 |

## 8. P1：Capture、Restore、Diff、Merge与Retention

| ID | 差距 | 代码事实与需要重构的内容 |
|---|---|---|
| DSA-P1-051 | capture同步扫描且近似N×T | `node_records`后按实体检查已注册serializable reflected types；接入Runtime05的增量/分区snapshot与预算，不在session复制ECS扫描器。 |
| DSA-P1-052 | reflected serializable集合不是产品participant | 物理、网络、脚本、音频、AI、在线和插件状态没有统一capture资格；消费Runtime40 participant registry和provider fingerprint。 |
| DSA-P1-053 | “restore to world”实际为增量spawn | 现有world内容保留，调用名易导致重复实体/状态；request必须显式`Replace/Merge/Instantiate`，默认不能含糊。 |
| DSA-P1-054 | level world与metadata分两步发布 | `replace_world_and_reset_runtime_state`后再`set_metadata`，中间状态可见；候选world、metadata、runtime reset同事务publish。 |
| DSA-P1-055 | 无world/level epoch CAS | restore没有验证preview后的target generation；绑定world id、level id、epoch、BuildSet与safe-point token。 |
| DSA-P1-056 | 无quiesce/rollback/resume | systems、scheduler、physics、network和script可在restore中继续运行；`SessionRestoreCoordinator`负责freeze、preflight、apply、rollback、resume。 |
| DSA-P1-057 | diff是全量capture后的布尔/计数 | 无结构化change set、entity/component provenance、分页或provider diff；返回bounded typed delta与unsupported owner列表。 |
| DSA-P1-058 | capture/diff/restore无统一deadline/cancel/progress | 大world操作可能占用线程和内存且不可观测；全部进入Runtime41 operation，并按phase报告预算和取消结果。 |
| DSA-P1-059 | merge只有slot-id两方碰撞 | `Reject/Keep/Replace`没有base revision、three-way conflict、schema/provider兼容、partial result或durable publication；设计基于digest的merge plan与冲突报告。 |
| DSA-P1-060 | retention只有slot count/protected/tag | 缺bytes、age、disk pressure、manual/autosave/critical、dependency、lease、tombstone、GC/compaction和删除原因；建立policy engine与dry-run receipt。 |

`protected_slots > max_slots`等不可满足预算必须显式报告policy violation，不能把“没有删除受保护slot”当作retention成功。merge与retention都必须对持久expected revision提交，而不是只验证同一内存archive lineage。

## 9. P2：可维护性、可诊断性与易用性

| ID | 差距 | 建议 |
|---|---|---|
| DSA-P2-001 | 505个production文件不超过25行 | 按store、artifact、transaction、query、restore、retention职责收敛目录，删除排列组合wrapper。 |
| DSA-P2-002 | 命名把所有选项拼进函数 | 使用request builder、selector enum与options struct，保持API discoverability。 |
| DSA-P2-003 | basic/metadata等平行入口重复 | metadata策略成为显式enum或payload，而不是复制整条调用链。 |
| DSA-P2-004 | named/selected平行入口重复 | 统一`SlotSelector`并由一次resolve产生stable selected revision。 |
| DSA-P2-005 | loaded/source_path平行入口重复 | store handle承载loaded identity，import/export path作为边界adapter。 |
| DSA-P2-006 | global/tag retention平行入口重复 | 统一predicate/policy AST并提供可解释计划。 |
| DSA-P2-007 | display search为case-sensitive substring | 采用normalized search key、locale policy、cursor和结果预算；它仍不是Editor全文检索替代品。 |
| DSA-P2-008 | query结果频繁Vec分配排序 | 提供iterator/visitor/cursor，明确stable ordering与allocation budget。 |
| DSA-P2-009 | error上下文缺少phase/path-safe摘要 | diagnostic包含operation、phase、store、slot、revision与redacted cause，禁止泄漏本机敏感路径。 |
| DSA-P2-010 | statistics缺少peak/RSS/I/O维度 | 记录read/write/serialized/resident/chunk counts、queue wait、fsync与recovery成本。 |
| DSA-P2-011 | source-shape tests占据“工程化”叙事 | 结构守卫只防回退，必须与behavior、fault、cross-process和performance证据分栏。 |
| DSA-P2-012 | 历史plan output容易被误读为验收 | failure记录关闭前，索引和报告统一显示open/pending validation。 |
| DSA-P2-013 | preview report未声明成本 | 每种preview暴露estimated bytes/work、exact/approximate与budget exceeded。 |
| DSA-P2-014 | import/export没有清晰trust classification | 区分trusted local、untrusted exchange、developer debug和platform store来源。 |
| DSA-P2-015 | manifest字段缺少redaction/display分层 | operator diagnostic、UI display和portable persisted manifest使用不同projection。 |
| DSA-P2-016 | 无统一feature/capability状态 | 未接store、migration、provider或recovery时fail closed为Unavailable/Degraded，不得只因re-export而Available。 |

## 10. 参考引擎对照：采用边界，不复制历史包袱

| 参考实现 | 可采用的工程原则 | 不应错误外推 |
|---|---|---|
| Unreal `ISaveGameSystem` / `GameplayStatics` | 把对象到bytes序列化与platform/user slot store分开；提供exists/list/save/load/delete、多用户与异步callback；同一pipe串行化异步operation | Unreal接口本身不是fsync、journal、云冲突或无丢更新证明，不能以“像Unreal”替代Zircon的durability测试 |
| Godot `ResourceSaver` / `PackedScene` | format saver、path/save flags与scene pack/instantiate分层；`SceneState`持有实例/编辑状态 | ResourceSaver/PackedScene是资源与场景合同，不等于SaveGame participant、platform user或checkpoint事务 |
| Bevy `ResolvedSceneRoot` / world serialization | dependency先resolve再spawn/apply；失败时清理中间root；world asset分离template、related entities与entity references | Bevy scene spawn不是完整save system，也不提供本报告所需的跨进程durability |
| Fyrox Visitor / Scene visit | structured region read/write、binary reader/writer和scene显式visit，使schema owner与字段遍历可审查 | Visitor模式不能自动解决slot policy、platform store、CAS或crash recovery |
| Unity Graphics VolumeProfile/Component/Stack | authoring profile、typed component参数与瞬态runtime evaluated stack分离，是“不要把运行时派生状态混入持久authoring asset”的负向边界证据 | Unity Graphics该目录不是通用SaveGame实现，不能被当成slot/cloud/restore参考 |

参考选择结论：Unreal主导platform/user slot与async product boundary；Godot/Bevy主导pack/resolve/apply分阶段；Fyrox主导显式visitor/schema ownership；Unity Graphics只提供authoring asset与runtime state分层证据。Zircon必须在这些边界上补充更强的CAS、journal、crash durability、bounded parsing和typed operation evidence，而不是照抄任一引擎的历史API。

## 11. 目标架构与核心合同

```text
Runtime40 SaveGame / Checkpoint product owner
  -> RuntimeSessionArchiveService
     -> SessionArchiveStore (platform + project + principal scope)
     -> SessionSnapshotCompiler (Runtime40 participants + Runtime05 snapshot)
     -> SessionArchiveTransaction
        prepare(expected revision/digest)
        write immutable chunks
        fsync data + journal
        publish manifest
        fsync directory/store metadata
        terminal receipt
     -> SessionArchiveQueryIndex (header/index-only, cursor, budgets)
     -> SessionRestoreCoordinator
        resolve -> quiesce -> preflight -> stage -> atomic publish
        -> rollback/resume -> receipt
     -> SessionArchiveRetentionManager (policy, tombstone, GC, compaction)
     -> Runtime41 OperationService
```

建议`SessionArchiveManifestV2`至少持有：

```text
archive_id, store_id, project_id, principal_id, slot_id
revision, parent_revision, commit_sequence, content_root_digest
build_set, engine/schema/provider fingerprints
world_id, level_id, world_epoch, capture_reason
capture_clock_domain, capture_time, operation/correlation id
chunk table(digest, codec, encryption key id, sizes, type/schema)
participant outcomes, restore policy, compatibility status
durability/platform/cloud disposition, created/updated provenance
```

manifest不能直接持久化本机project root、未分类自由字符串或可伪造的“durable=true”。所有状态转换必须由store transaction生成terminal receipt；caller不能自报成功。

## 12. 分层重构里程碑

### M52-0：冻结产品语义与删除目标

- 明确Runtime40、05、25、41与52的owner边界；
- 生成565-file facade调用/等价矩阵，标记保留、迁移、删除；
- 定义`SessionArchiveManifestV2`、scope、slot ID、revision、digest和operation receipt；
- 在真实consumer存在前保持capability fail closed。

### M52-1：Store、CAS与durable transaction

- 建立platform/project/principal scoped store；
- 加入跨进程lease、expected revision/digest CAS、journal、fsync与startup recovery；
- 故障注入覆盖每个write/flush/sync/rename/cleanup阶段；
- 删除进程全局path revision map作为正确性authority。

### M52-2：Manifest、chunk、migration与bounded load

- 从整份JSON切换为manifest + immutable chunks + stable digest；
- 增加结构预算、streaming parse、schema/provider migration和unknown preservation；
- 建立current/legacy/corrupt/hostile fixture corpus；
- 压缩、加密和key policy只由store/provider注入。

### M52-3：Snapshot participant与Restore Coordinator

- 接入Runtime40 participant registry和Runtime05 snapshot；
- 定义Replace/Merge/Instantiate与world epoch CAS；
- 实现quiesce/preflight/stage/atomic publish/rollback/resume；
- 结构化diff、partial compatibility和unsupported participant必须可见。

### M52-4：Query、merge、retention与GC

- header/index-only query、cursor、stable ordering和结果预算；
- base-aware three-way merge与typed conflict；
- bytes/age/pressure/class policy、tombstone、lease、compaction和content GC；
- small mutation不再整份重写。

### M52-5：产品接入与API硬切

- Runtime40成为唯一产品consumer，Editor/App通过产品command调用；
- 接入Runtime41 operation和统一diagnostics；
- 删除平行同步authority与数百个组合facade；
- 不保留旧path的compat re-export或“deprecated但仍可写”shim。

### M52-6：性能、恢复与发布资格

- 1、1K、100K slots与small/medium/max-policy snapshot基准；
- 0/10/1000 ms storage latency、write storm、跨进程竞争、重启恢复和长时soak；
- 收集p50/p95/p99、RSS peak、bytes copied、queue wait、fsync、recovery、compaction和restore pause；
- 与同语义参考场景比较，证据不足时不得宣称优于Unreal。

## 13. 验收门禁

### Owner与产品链

- [ ] DSA-G01：唯一`RuntimeSessionArchiveService`有真实Runtime40 consumer，re-export/test-only caller不计产品接入。
- [ ] DSA-G02：platform/project/principal scope在每个slot operation中强制校验。
- [ ] DSA-G03：547-function facade矩阵收敛为request/selector/service合同，旧写入口完成hard cut。
- [ ] DSA-G04：未安装store/provider/migration时capability fail closed，UI与Runtime不得假Available。
- [ ] DSA-G05：所有重工作业通过Runtime41 operation，拥有deadline/cancel/progress/harvest/shutdown。
- [ ] DSA-G06：每次terminal outcome带operation、scope、expected/current revision和durability disposition。

### 并发与耐久

- [ ] DSA-G07：同进程两个并发RMW不会丢更新，冲突返回typed outcome。
- [ ] DSA-G08：两个独立进程竞争同slot不会静默last-writer-wins。
- [ ] DSA-G09：外部替换文件后旧expected digest提交被拒绝。
- [ ] DSA-G10：writer admission失败不推进可使在途write失效的generation。
- [ ] DSA-G11：data sync失败不会返回Durable。
- [ ] DSA-G12：directory/store metadata sync失败进入明确terminal或recovery-required状态。
- [ ] DSA-G13：backup、rename、restore、cleanup任一步失败均无吞错。
- [ ] DSA-G14：每个journal阶段断进程后都能幂等恢复或隔离为quarantine。
- [ ] DSA-G15：symlink/junction/path swap无法逃逸store scope或覆盖非目标文件。
- [ ] DSA-G16：disk full、permission denied、read-only和partial write有确定transaction disposition。
- [ ] DSA-G17：shutdown分别验证drain、cancel、timeout、too-late commit与restart recovery。
- [ ] DSA-G18：revision/epoch接近耗尽时fail closed，不使用饱和后相同代际继续写。

### Schema、输入与存储

- [ ] DSA-G19：manifest v2绑定project/principal/build/schema/provider/world epoch与content digest。
- [ ] DSA-G20：slot ID覆盖长度、Unicode、control、reserved namespace和canonical collision测试。
- [ ] DSA-G21：load在分配前限制slots/entities/components/resources/tags/strings/depth/bytes/time。
- [ ] DSA-G22：current、每个legacy、future unsupported、missing provider与corrupt fixture有明确结果。
- [ ] DSA-G23：unknown chunk按声明策略保留或fail closed，绝不静默丢失后重写。
- [ ] DSA-G24：每个chunk/root digest检测单点损坏并定位，不依赖JSON parse成功。
- [ ] DSA-G25：compression/encryption/key rotation由store policy管理并有跨版本fixture。
- [ ] DSA-G26：touch/metadata/remove不重写无关scene chunk，crash后manifest/chunk引用仍一致。

### Capture、Restore、Merge与Retention

- [ ] DSA-G27：participant registry证明所有required subsystem已capture，optional/unsupported结果可见。
- [ ] DSA-G28：Replace/Merge/Instantiate语义互斥且默认fail closed。
- [ ] DSA-G29：restore在旧world epoch、安全点失效或BuildSet不兼容时拒绝提交。
- [ ] DSA-G30：restore故障不会暴露半发布world/metadata/runtime state，并能rollback/resume。
- [ ] DSA-G31：structured diff有entity/component provenance、cursor、budget和unsupported owner列表。
- [ ] DSA-G32：three-way merge报告base/ours/theirs、schema/provider冲突和最终digest。
- [ ] DSA-G33：retention覆盖bytes/age/pressure/class/protection/lease/tombstone；不可满足预算返回violation。

### 查询、性能与证据

- [ ] DSA-G34：manifest/status/selector query不解析无关scene payload。
- [ ] DSA-G35：query有stable ordering、cursor、max-results和allocation/time budget。
- [ ] DSA-G36：1/1K/100K slots记录p50/p95/p99、RSS、bytes read/written/copied与index rebuild。
- [ ] DSA-G37：0/10/1000 ms storage latency下main/runtime thread无未声明阻塞。
- [ ] DSA-G38：跨进程write storm、restart loop、corrupt corpus与24h soak没有丢更新或无界增长。
- [ ] DSA-G39：source-shape、unit、fault、cross-process、crash、performance证据分栏，静态guard不冒充产品验收。
- [ ] DSA-G40：任何“优于Unreal”结论使用同语义场景、同硬件、同durability等级与可复跑raw evidence；否则保持未证明。

## 14. 当前状态

- Review：`review_complete`。
- Implementation：`pending`。
- 新增P0：0；P1：60；P2：16；验收门禁：40。
- Runtime52未接管Runtime40的SaveGame/Checkpoint产品总owner，也未重复登记Runtime05的World/ECS/DynamicScene底层总问题。
- 三份相关failure handoff保持open；当前只有静态证据，不存在accepted implementation commit或milestone closeout。
- 下一步应从M52-0 owner/schema冻结与M52-1 durable store/CAS开始，不能继续扩写facade组合。
