---
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session
  - zircon_runtime/src/scene/dynamic_scene/scene/capture.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn
  - zircon_runtime/src/scene/dynamic_scene/document
  - zircon_runtime/src/core/framework/platform/preferences
  - zircon_runtime/src/platform/preferences
  - zircon_runtime/src/core/resource/io/atomic_file
  - zircon_runtime/src/core/resource/io/transaction
  - zircon_runtime_interface/src/serialization
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/script/vm/plugin/vm_state_blob.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - examples/woc/README.md
  - examples/woc/contracts/world-state.md
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/native/plugins/woc_runtime/src/transaction.rs
  - examples/woc/native/apps/woc_client/src/shell/offline_session.rs
  - examples/woc/native/apps/woc_client/src/preferences
  - examples/woc/native/apps/woc_client/src/input/keybind/storage.rs
  - examples/vampire/scripts/vampire_game/main.zr
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene_session
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_core.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_manifest.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_mutation.rs
  - zircon_runtime/src/platform/tests/preferences.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/38-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_app/05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SaveGameSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SaveGameSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameFramework/SaveGame.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/AsyncActionHandleSaveGame.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Kismet/GameplayStatics.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameplayStatics.cpp
  - dev/godot/core/io/file_access.h
  - dev/godot/core/io/file_access.cpp
  - dev/godot/core/io/resource_saver.h
  - dev/godot/core/io/resource_saver.cpp
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Fyrox/fyrox-core/src/visitor/reader/mod.rs
  - dev/Fyrox/fyrox-core/src/visitor/writer/mod.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_reflect/src/serde/mod.rs
  - dev/bevy/crates/bevy_reflect/src/serde/type_data.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/SerializableEnum.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/SerializedDictionary.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 40 · SaveGame / Checkpoint / Slot / Migration / Platform / Cloud Runtime 工程化差距

## 1. 结论

Zircon当前没有玩家可用、服务器可运维或平台可认证的SaveGame产品。仓内最接近的`RuntimeSessionArchive`不是空壳：它有slot、metadata、sealed artifact、manifest/index、validation、preview、retention、bounded keyed writer和大量组合式path API；Dynamic Scene也有反射capture、隔离preflight、generation检查和无失败publication；platform preferences以及core resource transaction更提供了原子写、`sync_all`、parent-directory sync、journal、owner lock和crash recovery。这些基础应保留，但没有任何一项单独等价于SaveGame。

当前最危险的差距不是“少一个保存按钮”，而是状态种类被混为一谈。Dynamic Scene Session保存编辑/运行会话中的World投影；native plugin与Zr VM的state blob服务热重载；WOC的`CommittedSnapshot`服务fixed-tick candidate/rollback/网络投影；玩家SaveGame还需要game/platform-user/profile/slot identity、显式participant、schema catalog、版本迁移、事务capture/restore、平台存储、cloud conflict、保护策略和产品receipt。任何把前三者直接改名或接线为SaveGame的方案都会静默保存瞬态对象、遗漏合法字段、错误恢复插件状态，并把进程内并发控制误当成断电一致性。

本轮逐文件冻结了完整565文件Session archive，并验证了一个明确的数据丢失风险：capture只要求field `serializable`，restore却只写`serializable && editable`字段；被序列化但不可编辑的字段会被静默跳过。archive外层格式又固定为version 1且没有migration chain；writer只`BufWriter::flush()`，没有文件或父目录`sync_all`，通过进程内`HashMap`判断stale write，失败清理大量使用`let _`。同仓的core resource transaction已经有更强的journal/recovery，Session却复制了一套较弱实现。

产品证据同样不成立。仓内Runtime/App/Editor/Plugin没有`SaveGame`或`RuntimeSessionArchive`产品caller；WOC `main.zr::saveState()`只回传独立的`restoredState`字符串，不能表示真实world；其声明仍写WOS113而writer已写118；Vampire `saveState()`返回常量`"vampire_game"`，`restoreState()`丢弃输入。WOS分裂由App03 P0-5拥有，WOC世界codec由Runtime12拥有，Vampire fixture由App06拥有，本篇不重复登记根P0。

本报告新增0项P0、72项P1、16项P2和40个资格门。Editor24 P0-3/P0-4仍是SaveGame服务缺失和Dynamic Scene误复用的canonical blocker；Runtime40负责把其后续运行时工作拆成可执行合同。实施顺序必须先冻结identity、participant和错误模型，再实现一致capture、versioned envelope与platform storage，随后接cloud/server checkpoint和真实产品，最后以断电、跨版本、跨设备、长时运行和大存档规模资格硬收敛。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | test属性或宏 / ignored | 证据强度与结论 |
|---|---:|---:|---|
| Dynamic Scene Session archive | 565 / 10,510 / 360,657 | 7 / 0 | E3完整文件级扫描；API宽但没有SaveGame身份、迁移或durable commit |
| Runtime/platform/schema底座 | 96 / 16,839 / 552,054 | 145 / 0 | E3核对capture/restore、preferences、resource transaction和versioned serialization |
| Native plugin与Zr VM热重载状态桥 | 8 / 3,547 / 130,201 | 13 / 0 | E3核对schema、skip/error行为和真实VM callback |
| focused external tests | 23 / 7,853 / 288,375 | 151 / 0 | E3核对archive/preferences/hot-reload局部行为；无SaveGame产品E2E |
| WOC/Vampire产品证据 | 18 / 74,667 / 3,585,056 | 0 / 0 | E3核对真实writer、fixture、transaction和client preferences；无slot/storage闭环 |
| 父计划控制面 | 13 / 5,268 / 551,637 | 6 / 1 | E2核对唯一owner与重复P0路由 |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | 20 / 12,055 / 513,894 | 10 / 0 | E2/E3区分SaveGame产品、资源IO、visitor、scene projection和序列化容器 |
| selected combined scope | 743 / 130,739 / 5,981,874 | 332 / 1 | 工作树fingerprint `483035db8ac9fa636bacdecef1ba749949e8874c7a9aec8ea8b660574c244137` |

指纹按743个selected path排序，对每个文件取lowercase SHA-256，再以`forward/slash/path|hash`和LF连接、无末尾LF后取总SHA-256。测试统计只表示静态`#[test]`/C++/C#测试标记，不表示已执行或通过。旧报告曾把同一565文件、360,657 bytes的Session目录记成9,399行；本轮用`Get-Content`与`File.ReadLines`独立复算均为10,510行，后续以本轮物理行口径为准。

### 2.2 检查方法

本轮沿`capture -> envelope -> encode -> stage -> durable commit -> enumerate/load -> migrate -> preflight -> publish`阅读真实链路；对archive的565文件逐目录检查construction、slot、metadata、manifest、retention、path capture、mutation、preview、load/save和writer；再反向搜索Runtime/App/Editor/Hub/Plugins/Examples消费者。平台侧对比preferences的atomic backend与core resource durable transaction，状态侧区分Dynamic Scene、native plugin、VM与WOC snapshot，产品侧核对WOC/Vampire真实入口和本地参考源码。

### 2.3 动态证据边界

本轮是review-only，没有修改Runtime、Interface、App、Plugin、Editor生产代码或测试，也没有把既有失败lane当成新证据。此前`zircon_editor --lib`在617秒后被239个既有error和122个warning阻断；WOC native cargo、typed contract与Hub lane也有已登记阻断。本报告没有重跑这些无关或重复失败。没有执行断电、跨进程竞争、512 MiB archive、历史存档、cloud、多用户、服务器checkpoint或真实游戏resume，因此所有相关能力保持未通过。

## 3. 必须保留的真实基础

1. 保留`RuntimeSessionArchive`的slot、metadata/tag、sealed artifact、manifest/index、preview和retention数据结构，作为可选participant payload或开发期session产品，不把它删除后重造。
2. 保留`RuntimeSessionArchiveWriter`的bounded keyed lane、admission、deadline、ticket和terminal outcome方向，但把path组合API收敛到typed request。
3. 保留Dynamic Scene的reflect registry、component/resource adapter、schema generation、isolated preflight World与no-fail publication，作为World participant底层事务。
4. 保留`zircon_runtime_interface::serialization`的`VersionedSchema`、schema identity、text/binary bound、migration chain与finite-number validation，作为Save envelope/participant codec公共合同。
5. 保留platform preferences的typed key、capacity/permission/corrupt/transient错误、mutation generation、flush fence和durability state；它继续服务小型设置，不直接承载大存档。
6. 保留core resource IO的staged atomic write、file/parent sync、journal、owner lock、recovery和failure observation，抽取为platform save storage的durability primitive。
7. 保留native plugin state schema callback和诊断聚合，但只把明确声明`SaveGameParticipant`的插件接入持久化，不能默认保存全部热重载状态。
8. 保留`VmStateBlob`的type identity、reflected object、field rename/default migration和validation；opaque VM bytes必须通过participant policy与payload budget后才能持久化。
9. 保留WOC `CommittedSnapshot`的generation/tick/digest/candidate-publication概念；它应成为server checkpoint contributor，而不是直接充当平台slot格式。
10. 保留Editor24、Runtime05、Runtime12、App03/App05/App06的既有owner，Runtime40通过依赖门收敛，不另建平行authority。

## 4. 当前代码事实与断路

| 链路 | 当前事实 | 工程后果 |
|---|---|---|
| 产品入口 | Runtime/App/Editor/Plugin/Hub与examples没有SaveGame service或Session archive产品caller | API规模不能证明玩家能保存、列举、恢复或删除存档 |
| Archive identity | 外层只有`format_version=1`、slot和进程索引；lineage/revision来自进程内原子计数 | 重启、外部writer、跨设备与server authority没有稳定generation |
| Slot | slot ID只trim并拒绝空；metadata只有project/asset/display/time/tag | 没有game/user/profile/platform namespace、长度/case/Unicode与物理路径隔离 |
| Capture | `DynamicScene::from_world`捕获所有注册为serializable的component/resource | 瞬态、网络、编辑器、缓存、句柄和安全敏感状态可能被无意持久化 |
| Restore | field写入要求`serializable && editable`，不可写字段被跳过 | capture成功、load成功但数据静默丢失，不能作为SaveGame round trip |
| Migration | nested Dynamic Scene有v0->v1->v2；archive外层只接受精确v1 | 无Save envelope、participant catalog或跨版本迁移计划 |
| Encoding | canonical text JSON，单artifact上限512 MiB，构建时保留完整bytes | 大世界产生多份内存、CPU与IO峰值，没有streaming/compression |
| Integrity | slot index使用`DefaultHasher`，没有持久checksum/authentication | bit rot、截断、恶意修改与内容错配不能可靠分类 |
| Atomic write | temp写入只`flush()`，target先rename到`.bak`，再rename temp | 缺文件/目录fsync；掉电后target/temp/backup状态没有恢复协议 |
| Concurrency | stale控制是进程全局`COMMITTED_PATH_REVISIONS` | 重启、第二进程、cloud/external writer不会被观察，CAS不成立 |
| Failure | temp/backup清理和restore多处`let _` | 失败可能遗留工件或丢失恢复证据，却只返回原始IO错误 |
| Missing slot | `load_or_empty_from_path`在36个文件出现41次，NotFound映射为空archive | 首次创建、路径错误、用户切换和数据丢失无法区分 |
| Retention | 仅按slot数量、tag、更新时间与protected ID裁剪 | 没有bytes、quota、age、cloud tombstone与platform政策 |
| Better primitive | preferences复用core atomic write；resource transaction有journal/recovery | Session复制弱IO authority，修复应收敛而非继续补丁 |
| Native plugin state | save失败插件被跳过；missing/schema mismatch restore也skip并继续 | 热重载的best-effort语义不能成为required Save participant事务 |
| VM state | blob v2与migration只被hot-reload coordinator消费 | 有用codec不等于长期兼容、受预算、可认证的玩家存档 |
| WOC | `saveState`回传独立字符串；world writer 118而声明/文档113 | fixed-tick snapshot、host lifecycle和SaveGame身份互相分裂 |
| Vampire | save返回常量，restore忽略输入 | lifecycle smoke fixture不能证明任何玩法状态持久化 |

## 5. 参考实现给出的边界

Unreal本地源码提供了最低产品边界：`ISaveGameSystem`是platform feature，覆盖platform user、slot exists/list/save/load/delete和平台初始化；`GameplayStatics`提供memory/slot同步与异步入口，`AsyncActionHandleSaveGame`把完成回调交回游戏线程，`FSaveGameHeader`携带class/version信息。Zircon不应照搬其默认文件布局，但至少要有同等级typed service、user/slot identity、async completion和version header，并在cloud conflict、cancel、deadline、receipt与断电资格上超过该基线。

Godot `FileAccess`与`ResourceSaver`说明user-data路径、文件访问和resource serialization是可组合底座，不自动成为玩家SaveGame。Fyrox Visitor说明region/version/typed read-write适合长期对象图迁移；Bevy scene/reflect说明注册类型的world projection与spawn可以成为participant，却没有替产品决定哪些状态应保存。Unity Graphics镜像中的`SerializableEnum`和`SerializedDictionary`只证明序列化容器/回调模式，镜像不含同级SaveGame authority；本报告不根据闭源Unity产品作不可验证推断。

由此目标架构必须分层：

```text
Gameplay / Server / Editor diagnostics
  -> ISaveGameService / ICheckpointService
     -> Identity + Capability + Request admission
     -> ParticipantRegistry + SchemaCatalog
     -> CaptureTransaction / RestoreTransaction
     -> VersionedEnvelope + ParticipantCodec + MigrationPlanner
     -> PlatformSaveStorage / ServerCheckpointStorage
     -> CloudSyncProvider + ConflictResolver
     -> Receipt / Diagnostic / Telemetry

Participants
  -> WorldDynamicSceneParticipant
  -> GameplayFrameworkParticipant
  -> ScriptStateParticipant
  -> NativePluginParticipant
  -> Product-owned WOC/Vampire participants
```

SaveGame与Checkpoint共用envelope、participant、migration、integrity和durable storage原语，但身份、权限、保留和一致性不同。玩家SaveGame由platform user/profile/slot拥有；server checkpoint由shard/session/authority epoch拥有；network snapshot由replication generation拥有；hot-reload snapshot由process/plugin generation拥有。四者不得共享一个裸`String slot_id`或一个best-effort restore API。

## 6. P0 所有权路由：本轮不重复登记

| canonical blocker | 唯一owner | Runtime40依赖 |
|---|---|---|
| 运行时没有SaveGame service、slot repository、participant/schema/migration/cloud authority | Editor24 P0-3 | M1-M4实现runtime service，Editor只投影receipt |
| Dynamic Scene archive不能直接改名为SaveGame，World snapshot可能静默遗漏 | Editor24 P0-4 + Runtime05 | M2修participant与无损restore后才能接入 |
| WOS83/113/118/117 writer-reader-schema identity分裂 | App03 P0-5 | WOC participant与历史fixture必须等其关闭 |
| WOC transaction不能证明VM内部rollback | App03 P0-6 + Runtime12 | checkpoint capture必须消费isolated candidate或restore token |
| WOC普通tick全量state和decoder副作用 | Runtime12 P0-3/P0-5 | checkpoint不能固化非确定restore或O(world bytes)每tick路径 |
| Vampire产品证据与常量state fixture | App06 P1-19 | App06提供真实玩法participant和roundtrip oracle |

这些父条目在关闭前必须让SaveGame capability保持`Unavailable`或`Partial`。Runtime40新增的是P1/P2分解、owner合同和资格门，不通过改编号制造“发现更多P0”的假进展。

## 7. P1：Service、Identity、Participant 与 Ownership

| ID | 当前差距 | 必须重构 |
|---|---|---|
| SAV-P1-001 | 没有engine-owned `ISaveGameService`/`ICheckpointService` | 建立唯一service facade、capability query、typed request与terminal receipt；产品不得直接写文件 |
| SAV-P1-002 | 保存/载入没有request ID、base generation和idempotency key | 每次操作携带稳定operation ID、期望generation、重复提交语义和provenance |
| SAV-P1-003 | 缺game/title、platform user、profile与slot复合身份 | 引入强类型ID及canonical encoding，禁止裸字符串互换 |
| SAV-P1-004 | slot只有trim/非空验证 | 冻结长度、字符、Unicode normalization、case、reserved name与显示名分离策略 |
| SAV-P1-005 | 没有participant registry | owner按稳定participant ID注册capture/restore/schema/dependency/capability |
| SAV-P1-006 | required与optional participant没有区别 | required缺失/失败使事务失败；optional必须显式记录omission与降级receipt |
| SAV-P1-007 | participant没有capture/restore阶段与依赖图 | 定义prepare/capture/finalize和preflight/apply/commit/rollback拓扑，拒绝环 |
| SAV-P1-008 | participant只有type path或plugin ID，没有长期稳定identity | 使用namespace + UUID/stable ID + schema family；rename通过alias/migration处理 |
| SAV-P1-009 | plugin unload、DLC/mod缺失时无owner生命周期 | registry generation、lease、drain、orphan payload和卸载阻断成为合同 |
| SAV-P1-010 | build/content/plugin catalog未绑定save | envelope记录build ID、content catalog、plugin set及兼容指纹，load先做admission |
| SAV-P1-011 | local player、server authority与client权限未定义 | request声明authority/principal；client不能写server-owned world participant |
| SAV-P1-012 | capability来自“API存在”而非provider资格 | capability报告storage、migration、cloud、encryption、participant closure与阻断原因 |

## 8. P1：Capture、Snapshot、Restore 与 World Consistency

| ID | 当前差距 | 必须重构 |
|---|---|---|
| SAV-P1-013 | capture没有simulation safe point或quiescence合同 | 在固定barrier取得generation lease，冻结结构变更或消费COW snapshot |
| SAV-P1-014 | Dynamic Scene默认捕获全部serializable component/resource | 引入显式Save policy/participant projection，deny-by-default处理瞬态状态 |
| SAV-P1-015 | `serializable`被错误等同于`savable` | reflection分别表达asset/scene/network/save/editor/diagnostic用途与保护分类 |
| SAV-P1-016 | serializable但non-editable字段capture后restore静默跳过 | restore必须无损写入专用adapter，或在capture/preflight以typed error拒绝 |
| SAV-P1-017 | dynamic/plugin type缺稳定field schema catalog | 编译participant type/field ID、wire type、default、rename、removed-field与fingerprint |
| SAV-P1-018 | global resource的world/profile/session lifetime模糊 | resource participant声明owner scope、singleton key与跨world迁移规则 |
| SAV-P1-019 | entity只有运行时ID，跨load identity/rebinding不完整 | 复用Runtime24稳定ID/generation，建立local object table与external reference table |
| SAV-P1-020 | entity/component/resource引用没有两阶段resolve | decode先建对象/ID表，再resolve引用；missing/stale/cycle按策略失败或保留 |
| SAV-P1-021 | cache、GPU、physics broadphase、network handle等瞬态状态无排除目录 | 建立transient catalog与rebuild hook，测试确保它们不进入payload |
| SAV-P1-022 | capture没有独立CPU/time/bytes/allocation/cancel budget | admission预估，执行检查deadline/cancel，超预算不发布半成品 |
| SAV-P1-023 | 每次完整world snapshot，没有dirty/incremental checkpoint | participant提供dirty frontier/COW page；autosave可增量，完整快照受频率门控 |
| SAV-P1-024 | multi-world、level travel与seamless travel顺序未定义 | Gameplay Framework owner提供world context集合、travel barrier和原子切换receipt |

## 9. P1：Envelope、Schema、Migration、Integrity 与 Encoding

| ID | 当前差距 | 必须重构 |
|---|---|---|
| SAV-P1-025 | Session archive外层精确version 1且无migration chain | 使用Runtime Interface `VersionedSchema`建立Save envelope family与连续迁移图 |
| SAV-P1-026 | 没有顶层SaveGame/checkpoint header | header含magic、kind、schema、game/user/profile/slot、generation、build、time和flags |
| SAV-P1-027 | 没有participant manifest与offset/index | 记录稳定ID、schema、required、codec、sizes、digest、dependencies和payload range |
| SAV-P1-028 | unknown participant/field只有拒绝或丢弃 | 定义opaque preservation、quarantine、optional skip和required failure策略 |
| SAV-P1-029 | participant各自版本没有统一migration planner | planner验证源/目标catalog、依赖顺序、支持窗口并生成可审计plan |
| SAV-P1-030 | migration没有deadline、cancel、内存预算与determinism约束 | migration运行在bounded context，纯转换与content reconciliation分离 |
| SAV-P1-031 | downgrade、回滚版本和长期支持窗口未定义 | 发布策略声明min reader/writer、forward preservation与不可逆迁移确认 |
| SAV-P1-032 | 没有持久化checksum或内容地址 | envelope、manifest、每participant和chunk使用稳定强hash，分类bit rot/截断 |
| SAV-P1-033 | 没有compression policy | per-participant/chunk选择codec、版本与dictionary，记录原始/压缩尺寸和预算 |
| SAV-P1-034 | 没有encryption、authentication或key rotation | 平台/服务端provider管理AEAD、key ID、nonce、rotation和不可恢复错误 |
| SAV-P1-035 | canonical JSON构建完整bytes，512 MiB可产生多份resident | 使用bounded streaming/chunk writer、spool或paged buffer，峰值内存受资格门限制 |
| SAV-P1-036 | corruption、partial migration和unsupported content诊断不充分 | typed error定位header/participant/chunk/path，保留原件并可导出repair report |

## 10. P1：Platform Storage、Durability、Async 与 Recovery

| ID | 当前差距 | 必须重构 |
|---|---|---|
| SAV-P1-037 | Session接受caller任意path | `IPlatformSaveStorage`解析logical identity到sandbox path，caller不可提供物理路径 |
| SAV-P1-038 | 没有完整enumerate/exists/read/write/delete/rename/copy合同 | 每项异步、typed、generation-aware并返回metadata/receipt；批量枚举有分页 |
| SAV-P1-039 | 同机多用户/profile路径与权限未隔离 | provider绑定platform principal/session，用户切换使旧lease失效 |
| SAV-P1-040 | Session atomic write只flush、不fsync | 复用core staged write，文件与parent directory durable sync成为commit前置 |
| SAV-P1-041 | Session复制弱atomic实现而未复用resource transaction | 抽取通用durable blob transaction；Save不依赖resource或preferences语义 |
| SAV-P1-042 | `.tmp/.bak`没有startup recovery状态机 | journal记录intent/staged/retired/committed/cleanup，启动时确定roll-forward/back |
| SAV-P1-043 | stale revision只存在进程内HashMap | 持久generation/etag、owner lock与compare-and-swap覆盖跨进程/重启/external writer |
| SAV-P1-044 | lineage不同即可覆盖旧内容，外部writer不可见 | storage返回current generation，冲突保留双方，force overwrite需显式权限 |
| SAV-P1-045 | NotFound被广泛映射为空archive | 区分missing slot、first create、wrong user/path、corrupt catalog与I/O故障 |
| SAV-P1-046 | writer只有admission deadline，缺端到端cancel/progress阶段 | operation状态覆盖queued/capturing/encoding/staging/committing/syncing/terminal |
| SAV-P1-047 | 没有quota/free-space/reservation与platform lifecycle | 写前reserve，处理中处理suspend/device removal/account loss，失败释放reservation |
| SAV-P1-048 | retention只按slot count/tag/time | policy同时覆盖bytes、age、slot class、protected generation、backup和tombstone |

## 11. P1：Cloud、Network、Security 与 Server Checkpoint

| ID | 当前差距 | 必须重构 |
|---|---|---|
| SAV-P1-049 | 没有cloud save provider | 定义provider capability、object identity、upload/download/list/delete与typed availability |
| SAV-P1-050 | 没有etag/base generation或冲突策略 | upload使用conditional write；冲突保留local/remote/base并返回决策对象 |
| SAV-P1-051 | offline写入没有journal和幂等sync | 本地operation log可重放/合并/取消，重启后不重复上传或删除 |
| SAV-P1-052 | 跨设备generation、clock skew与排序未定义 | 顺序依赖server generation/causal metadata，不用wall clock newest静默覆盖 |
| SAV-P1-053 | 平台native UI、登录/登出和controller-user映射未接入 | provider初始化、user selection、session invalidation和UI回调有明确线程合同 |
| SAV-P1-054 | network client/server谁能capture/load未定义 | server authoritative state只由server checkpoint service写入，client仅保存owned profile |
| SAV-P1-055 | App05没有可运维的server checkpoint repository | shard/realm/session/authority epoch identity、lease、retention和restore orchestration独立实现 |
| SAV-P1-056 | 没有anti-rollback/tamper与competitive policy | online profile记录monotonic server generation、签名/认证和replay rejection |
| SAV-P1-057 | PII、账号、聊天、社交和支付状态没有数据分类 | participant字段标注privacy/region/retention/export/delete策略，默认最小化 |
| SAV-P1-058 | untrusted save输入没有结构/CPU/allocation/decompression预算 | header先验、chunk bound、depth/count/ratio限制与fuzz覆盖成为reader前门 |
| SAV-P1-059 | cloud与local加密key owner未定义 | 平台secure storage/KMS拥有key，游戏脚本和插件拿不到原始长期key |
| SAV-P1-060 | telemetry可能泄露slot名或payload内容 | 只记录稳定错误码、尺寸、阶段、延迟和匿名provider ID，敏感字段脱敏 |

## 12. P1：Product Integration、Performance、Testing 与 Authority

| ID | 当前差距 | 必须重构 |
|---|---|---|
| SAV-P1-061 | WOC world snapshot没有file/slot/cloud storage路径 | 在Runtime12稳定snapshot后实现WOC checkpoint participant和App05 repository |
| SAV-P1-062 | WOC `saveState`只保存独立`restoredState` | real backend从一致world candidate导出versioned participant，不再用lifecycle fixture |
| SAV-P1-063 | WOC main/README/contract仍宣称WOS113而writer已118 | 服从App03 schema hard cut，生成单一writer/reader/docs identity并保留golden |
| SAV-P1-064 | WOC 68k行手写codec把checkpoint、tick和迁移耦合 | Runtime12收敛为generated/bounded codec、COW snapshot和独立migration/reconciliation |
| SAV-P1-065 | WOC FNV digest不是持久完整性或认证机制 | simulation digest与storage strong hash/AEAD分开，不能互相冒充 |
| SAV-P1-066 | Vampire保存常量且restore丢弃输入 | App06提供真实player/progression/world participant、旧版fixture和resume oracle |
| SAV-P1-067 | Gameplay Framework/App没有SaveGame调用链 | GameInstance/WorldContext/LocalPlayer通过service request保存，travel/restart消费receipt |
| SAV-P1-068 | Editor Workbench只能展示fixture时序 | service未qualified时禁用；qualified后只显示真实slot/catalog/conflict/operation状态 |
| SAV-P1-069 | 没有玩家save/load/overwrite/delete/conflict UX闭环 | 产品实现可取消操作、确认、失败恢复、空间不足、损坏和cloud选择流程 |
| SAV-P1-070 | 测试没有真实玩法变化、进程重启后恢复 | Vampire/WOC E2E跨进程比较stable state，并验证transient state正确重建 |
| SAV-P1-071 | 没有1 MiB到多GiB的延迟、内存、IO、压缩规模资格 | 建立size/participant/entity sweep，记录capture stall、peak RSS、write/read p50/p99 |
| SAV-P1-072 | 没有local/cloud/offline/server/platform故障矩阵 | required CI注入kill point、disk full、permission、corruption、conflict、cancel和provider loss |

## 13. P2：完整性、可用性与维护性

| ID | 改进项 | 目标 |
|---|---|---|
| SAV-P2-001 | slot thumbnail/screenshot metadata | 独立optional blob与尺寸/格式预算，不让render failure阻断核心存档 |
| SAV-P2-002 | 细粒度async progress | 阶段与bytes/participants进度单调、可节流，UI不猜百分比 |
| SAV-P2-003 | Save diff inspector | 对schema-aware字段和participant摘要做只读diff，不默认暴露敏感数据 |
| SAV-P2-004 | checkpoint类别 | 区分manual/autosave/quicksave/suspend/server/admin/imported及各自保留策略 |
| SAV-P2-005 | delta compression | 基于明确base generation和chain length，定期compact，base缺失可恢复 |
| SAV-P2-006 | autosave调度 | coalescing、minimum interval、dirty participant、frame/load预算与backoff |
| SAV-P2-007 | suspend/emergency save | 独立短deadline与最小required participant集合，不能假装等价完整save |
| SAV-P2-008 | cross-title/profile隔离 | game/title/environment/branch namespace阻止测试、开发、正式档互相覆盖 |
| SAV-P2-009 | 本地化错误与操作文本 | typed code映射可本地化文案，不把底层path/OS字符串直出给玩家 |
| SAV-P2-010 | accessibility | 保存列表、进度、冲突和确认对键盘、读屏、缩放与色觉可达 |
| SAV-P2-011 | DLC/mod orphan工具 | 列出缺失owner、保留opaque payload、导出/修复并避免静默删除 |
| SAV-P2-012 | replay/save/checkpoint术语 | API与文档明确三者identity、determinism和lifetime，不共享模糊Snapshot命名 |
| SAV-P2-013 | deterministic diagnostics | 同一错误产生稳定code/path/participant/generation，便于golden和支持 |
| SAV-P2-014 | repair/export utility | 只读校验、manifest导出、last-known-good恢复和用户授权后的脱敏包 |
| SAV-P2-015 | 参考/回归benchmark | 固定硬件与数据集比较版本趋势；不以未经同条件测量的“超过Unreal”作结论 |
| SAV-P2-016 | SDK与文档 | 为game/plugin作者提供participant、migration、预算、安全和测试规范 |

## 14. 重构所有权

| owner | 拥有 | 不拥有 |
|---|---|---|
| Runtime40 Save Service | request、identity、participant、capture/restore、envelope、storage orchestration、receipt | Editor UI、具体游戏字段、平台SDK内部实现 |
| Runtime Interface02 | 稳定DTO、versioned schema/migration、typed error公共合同 | 具体slot目录、cloud provider、World capture |
| Runtime04 / core resource IO | durable atomic/journal/recovery primitive | SaveGame产品identity和participant策略 |
| Runtime05 | World/Dynamic Scene无损capture/restore、stable entity/reference | player/profile/slot、cloud和产品UX |
| Runtime24 | stable ID/generation/owner epoch与stale reference | 保存哪些业务状态 |
| Runtime38 | GameInstance/WorldContext/LocalPlayer/travel/restart接入点 | 物理存储和cloud conflict |
| Plugins01 | plugin package/ABI/state owner与unload生命周期 | 默认把所有plugin hot-reload bytes持久化 |
| Runtime12 + App03/App05 | WOC world snapshot、schema/transaction、server repository | 通用Save service第二authority |
| App06 | Vampire真实玩法participant与产品E2E | 通用envelope或platform provider |
| Editor24 | Save diagnostics/authoring视图与产品truth | 自建slot列表、假migration或直接文件写入 |

## 15. 分层里程碑

### M0 · Truth Freeze 与损坏阻断

1. 保持SaveGame/Checkpoint capability不可用，删除或禁用所有无receipt成功状态。
2. 为`serializable && !editable`建立最小复现，修复前禁止Dynamic Scene进入Save participant。
3. 固化Session archive、preferences、resource transaction、native/VM/WOC state的边界与owner。
4. 收集真实1/10/100/512 MiB capture/encode/write/read峰值，冻结kill-point基线。

### M1 · Identity、Service 与 Participant Contract

1. 在Runtime Interface冻结强类型identity、request、receipt、typed error、capability和schema DTO。
2. 建立engine-owned service与participant registry，支持required/optional、dependency、lease和unload。
3. 分离SaveGame、server checkpoint、network snapshot与hot-reload snapshot identity。
4. Gameplay Framework只接service，不暴露caller path或archive内部组合API。

### M2 · Consistent Capture 与 Transactional Restore

1. World、gameplay、script和plugin participant在safe point生成bounded snapshot。
2. 修复Dynamic Scene无损字段、stable entity、reference table与unknown policy。
3. restore执行decode/migrate/preflight/apply/commit，required participant失败可完整rollback。
4. 增量checkpoint消费dirty/COW frontier，ordinary tick不做全量world encode。

### M3 · Envelope、Migration、Durable Storage

1. 建立versioned header、participant manifest、chunk、strong hash、compression与可选AEAD。
2. migration planner覆盖catalog、依赖、budget、downgrade/forward preservation和golden fixtures。
3. PlatformSaveStorage复用core durable transaction，实现logical slot到sandbox的唯一映射。
4. 完成enumerate/read/write/delete/rename/copy、quota、cancel、progress和startup recovery。

### M4 · Cloud、Server 与 Lifecycle

1. cloud provider实现etag/CAS、offline journal、conflict object、tombstone和用户生命周期。
2. server checkpoint实现shard/authority epoch、lease、retention、replication barrier与灾难恢复。
3. suspend、autosave、travel、restart、logout、shutdown按照deadline与required set协同。
4. privacy、key、anti-tamper、anti-rollback和telemetry政策进入provider资格。

### M5 · 产品闭环与竞争资格

1. Vampire先完成小型真实玩法roundtrip，WOC在schema/kernel blocker关闭后接大规模checkpoint。
2. 玩家UI覆盖列举、保存、覆盖、载入、删除、损坏、空间不足、cancel和cloud conflict。
3. required CI运行历史fixture、跨进程、kill point、fuzz、跨设备模拟、24h soak和规模sweep。
4. 删除Session/Workbench/产品侧第二authority，只保留兼容adapter并设置硬删除版本。

## 16. 资格门

| Gate | 必须证明的证据 |
|---|---|
| SAV-G01 | public surface只有一个SaveGame service和一个Checkpoint service，caller不能提供物理path |
| SAV-G02 | game/platform-user/profile/slot与server shard/epoch使用不可互换强类型identity |
| SAV-G03 | operation ID、base generation、idempotency和terminal receipt在重试/重启后保持一致 |
| SAV-G04 | capability由实际storage/participant/migration/provider资格计算，缺失时fail closed |
| SAV-G05 | participant registry拒绝duplicate ID、依赖环、stale generation和unload race |
| SAV-G06 | required participant缺失/失败使事务失败，optional omission有稳定diagnostic |
| SAV-G07 | build/content/plugin catalog不兼容在任何world mutation前被拒绝 |
| SAV-G08 | SaveGame、checkpoint、network与hot-reload snapshot不能通过类型系统误传 |
| SAV-G09 | capture发生在qualified safe point，压力下无mixed-generation snapshot |
| SAV-G10 | transient component/resource/handle/GPU/network缓存不会进入Save payload |
| SAV-G11 | serializable non-editable字段要么无损恢复，要么capture前明确拒绝，绝不静默丢失 |
| SAV-G12 | stable entity/object identity跨load重建，stale reference与missing external有typed结果 |
| SAV-G13 | multi-world/travel capture具有明确barrier，失败不切换半恢复World |
| SAV-G14 | capture CPU/time/bytes/allocation/deadline/cancel预算由producer侧强制 |
| SAV-G15 | dirty/COW checkpoint与完整snapshot结果等价，base generation错误被拒绝 |
| SAV-G16 | restore required participant任一点失败均能回滚到原generation和可运行状态 |
| SAV-G17 | envelope magic/kind/schema/build/catalog/identity/generation均有验证和golden |
| SAV-G18 | 每个supported旧版本有decode+migration+re-encode golden，future version fail closed |
| SAV-G19 | unknown optional participant可opaque保留；unknown required participant不被静默跳过 |
| SAV-G20 | migration是deterministic、bounded、可取消的纯转换，content reconciliation另有receipt |
| SAV-G21 | envelope、manifest、chunk与participant strong hash能定位bit rot和截断 |
| SAV-G22 | compression/decompression有ratio和output bound，zip-bomb类输入被前置拒绝 |
| SAV-G23 | encryption/authentication使用qualified key provider，tamper不能进入restore |
| SAV-G24 | 1 MiB至目标上限streaming encode/decode峰值RSS满足版本化预算 |
| SAV-G25 | write commit包含file和parent directory durability，成功receipt只在sync后发布 |
| SAV-G26 | 每个journal kill point重启后确定roll-forward/back，target/temp/backup不歧义 |
| SAV-G27 | 同进程、跨进程、重启和external writer都服从持久CAS/owner lock |
| SAV-G28 | missing、first-create、wrong-user、corrupt、permission、disk-full错误不互相映射 |
| SAV-G29 | enumerate/read/write/delete/rename/copy分页、取消、deadline和receipt行为一致 |
| SAV-G30 | quota reservation、suspend、account loss、device removal失败不泄露资源或谎报成功 |
| SAV-G31 | cloud conditional write冲突保留local/remote/base，禁止wall-clock last-writer-wins |
| SAV-G32 | offline journal在重复上传、崩溃、重连和tombstone场景幂等收敛 |
| SAV-G33 | server checkpoint只接受authoritative principal，client不能覆盖server world |
| SAV-G34 | privacy删除/导出、retention、key rotation和anti-rollback有平台/服务端集成测试 |
| SAV-G35 | Vampire真实玩法跨进程save/load后stable state一致、transient state正确重建 |
| SAV-G36 | WOC current writer能被current reader读取，host/schema/contract/docs identity一致 |
| SAV-G37 | WOC checkpoint不在普通tick执行O(world bytes) encode/copy，10k实体预算通过 |
| SAV-G38 | 玩家产品完成save/list/load/overwrite/delete/corrupt/quota/cloud conflict全流程 |
| SAV-G39 | required CI覆盖历史fixture、fuzz、failure injection、kill point、跨设备和24h soak |
| SAV-G40 | 规模sweep记录p50/p95/p99、peak RSS、IO bytes、compression、dirty ratio且无未解释回退 |

## 17. 禁止的临时修补

- 禁止把`RuntimeSessionArchive`重命名为SaveGame，或直接把Workbench/Gameplay按钮接到其path API。
- 禁止用`serializable`、`editable`、`replicated`任一单独标志推断Save参与资格。
- 禁止让产品、脚本或插件提供任意物理路径；logical identity必须由provider解析。
- 禁止把`BufWriter::flush()`、rename或进程内HashMap描述为断电安全/跨进程CAS。
- 禁止把NotFound静默转换为空档后继续载入，也禁止损坏时自动覆盖原文件。
- 禁止best-effort跳过required plugin/script participant并仍返回load成功。
- 禁止使用wall-clock newest作为cloud冲突唯一规则，或静默last writer wins。
- 禁止把simulation digest/FNV当作存储完整性、认证或反篡改机制。
- 禁止在ordinary tick全量encode World来“顺便支持autosave/checkpoint”。
- 禁止用单进程roundtrip、test attribute数量或fixture常量声称SaveGame产品完成。
- 禁止为绕过migration而提升version并丢弃unknown participant/field。
- 禁止新增第二套atomic file、schema registry、participant identity或cloud authority。

## 18. 本轮产出边界

本轮只新增审查与重构计划，不修改Runtime、Interface、App、Plugin、Editor生产代码或测试，不创建临时SaveGame实现。结论基于743个文件、130,739行、5,981,874 bytes的冻结语料；工作树指纹为`483035db8ac9fa636bacdecef1ba749949e8874c7a9aec8ea8b660574c244137`。当前WOC source writer 118与WOS113文档分裂、相关计划和本仓工作树都可能继续演进，且`source_recheck_required: true`；实施前必须重新取证、重算fingerprint并复核父P0状态。
