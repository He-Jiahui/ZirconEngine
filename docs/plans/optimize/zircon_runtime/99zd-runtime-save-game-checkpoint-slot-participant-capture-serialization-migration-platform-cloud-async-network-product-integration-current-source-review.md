---
title: Runtime SaveGame / Checkpoint / Slot / Participant / Migration / Platform / Cloud 当前源码复审
category: zircon_runtime
report_id: Runtime129
review_date: 2026-08-23
baseline_head: 0e2bdaa9d3f6949e351ce4e77ccf1aca9e7032b1
baseline_epoch: 383
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/document
  - zircon_runtime/src/core/resource/io
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io
  - zircon_runtime/src/core/framework/platform/preferences
  - zircon_runtime/src/platform/preferences
  - zircon_runtime_interface/src/serialization
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_runtime/src/script/vm
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - examples/woc
  - examples/vampire/scripts/vampire_game/main.zr
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/runtime_state.rs
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene
  - zircon_runtime/src/platform/tests/preferences.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/38-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99v-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zb-runtime-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_app/05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SaveGameSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SaveGameSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameFramework/SaveGame.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/AsyncActionHandleSaveGame.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameFramework/AsyncActionHandleSaveGame.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameplayStatics.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Microsoft/GDKSaveGameSystem/Public/GDKSaveGameSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Microsoft/GDKSaveGameSystem/Private/GDKSaveGameSystem.cpp
  - dev/godot/core/io/file_access.h
  - dev/godot/core/io/file_access.cpp
  - dev/godot/core/io/resource_saver.h
  - dev/godot/core/io/resource_saver.cpp
  - dev/godot/tests/core/io/test_file_access.cpp
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Fyrox/fyrox-core/src/visitor/reader/binary.rs
  - dev/Fyrox/fyrox-core/src/visitor/writer/binary.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_asset_spawner.rs
  - dev/bevy/crates/bevy_reflect/src/serde/type_data.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/SerializableEnum.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/SerializedDictionary.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime129 · SaveGame / Checkpoint / Slot / Participant / Migration / Platform / Cloud 当前源码复审

## 1. 结论

Zircon 当前仍没有玩家可用、服务器可运维、平台用户可认证的 SaveGame 或 Checkpoint 产品。对 `zircon_runtime/src`、`zircon_app/src`、`zircon_editor/src`、`zircon_runtime_interface/src` 与 `zircon_plugins` 的精确符号检索中，`ISaveGameService`、`ICheckpointService`、`IPlatformSaveStorage`、`SaveGameParticipant`、`SaveGameService`、`PlatformSaveStorage`、`CloudSave` 和 `SaveGame` 均为 0。`RuntimeSessionArchive` 除 owner 与测试外只在 `scene/dynamic_scene/mod.rs` 和 `scene/mod.rs` 重导出，没有产品 consumer。

现有代码不是全无价值。Dynamic Scene 已具备 generation/schema/component registry 约束、隔离 preflight World、fallible adapter write 与紧凑 commit boundary；Session archive 已具备 slot、manifest/index、sealed artifact、retention、bounded keyed writer、deadline/ticket/outcome；Runtime Interface 有 `VersionedSchema`/`MigrationChain`；core resource transaction 有 journal、owner lock、file/parent sync 与 recovery；native plugin/VM 有热重载状态迁移；WOC 有 generation/tick-bound committed snapshot。这些都应保留为底座，但没有任何一项可以直接改名为 SaveGame。

关键 correctness 风险仍存在：Dynamic Scene capture 只检查 `serializable`，restore writable map 却要求 `serializable && editable`，因此可出现“保存成功、载入成功、字段静默丢失”；Session 外层仍固定 format version 1，私有 atomic writer 只 `flush()`，没有 file/parent sync、journal、startup recovery 或跨进程 CAS；`load_or_empty_from_path` 仍把 NotFound 映射为空 archive。WOC writer 已写 WOS118，而 `main.zr`、README/contract 仍声明 WOS113，普通 tick 仍 clone 并 encode 完整 committed state；Vampire `saveState()` 仍返回常量，`restoreState()` 丢弃输入；Editor Save/Load UI 仍只返回 `AutoSave_01` 的固定样例文本。

Runtime40 的 72 项 P1 本轮按当前源码重判为 **53 Open、19 Partial、0 Closed**；16 项 P2 全部 Open；40 项资格门为 **33 Fail、7 Partial、0 Pass**。Partial 只表示存在可复用的局部结构，不表示可发布 capability。本文新增 0 项 P0/P1/P2，不复制 Editor24、Runtime05/12、App03/05/06 等既有 canonical blocker。

## 2. 审查边界、方法与 currentness

### 2.1 当前物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations / ignored | 本轮证据 |
|---|---:|---|
| Session owner 与 Dynamic Scene tests | **570 / 12,231 / 10,984 / 422,802 / 44 / 0** | slot、capture/restore、artifact、manifest/index、merge、retention、path IO、writer；fingerprint `c627391fbb7620e88405d5a297a5a77c516a4786c7ba3e20c295e1a9d3d783fb` |
| Runtime storage/schema foundations | **113 / 19,980 / 18,166 / 657,735 / 174 / 0** | Dynamic Scene scene/document、resource transaction、bounded keyed IO、preferences、versioned serialization；fingerprint `04cc65efe892a10a8f4e8d462c99415b163785b874b26a3e27ad02b8a3222cea` |
| Native plugin 与 VM state | **8 / 3,547 / 3,272 / 130,201 / 13 / 0** | ABI state callback、schema/migration、hot-reload coordinator 与真实 VM backend；fingerprint `621b71ed6db7bdad0d02683f2af2462b5fc23fe116cf029f96a8da0574c6505c` |
| WOC/Vampire/Editor 产品边界 | **10 / 72,698 / 69,788 / 3,528,384 / 0 / 0** | WOC state/transaction/app storage、Vampire fixture、Editor 假 Save/Load 状态；fingerprint `bc7f2a770bc4277269519d9e4bb4a63324c3968f9dbd4656672b87447c038c34` |
| Zircon 去重聚焦集 | **701 / 108,456 / 102,210 / 4,739,122 / 231 / 0** | 上述四组按 normalized path 去重；fingerprint `2d739b3dd8d5af490f5c2cb80e757410d1dbe0b641acc06b1697c1e9e2af5e47` |
| 五引擎参考集 | **23 / 12,755 / 11,031 / 487,757 / 19 / 0** | Unreal 8、Godot 5、Fyrox 3、Bevy 5、Unity Graphics 2；fingerprint `12a1b9599fb6fdbc63fe7d7ad89c8453e025ac3e042175ea7c987e4f8ddb3c91` |

指纹算法为：每个 normalized relative path 对应文件 lowercase SHA-256，按路径排序，以 `path|hash` 和 LF 连接且无末尾 LF，再取整体 SHA-256。测试数是静态 declaration 计数，不表示已执行或通过。Runtime99v 的 565 文件口径只冻结其当时 owner；本轮把当前 Session owner 与 Dynamic Scene direct tests 统一重算为 570 文件，不把文件数变化误写成产品能力进展。

### 2.2 检查方法

本轮沿 `request -> identity/capability -> participant discovery -> safe-point capture -> envelope/encode -> staged durable commit -> enumerate/load -> migrate -> preflight -> atomic publish -> receipt` 正向阅读，并从 Editor、App、WOC、Vampire、native plugin、VM 和网络/checkpoint 关键词反向搜索真实 consumer。对每项能力同时检查定义、调用者、失败语义、generation/currentness、预算、平台 owner 和产品可达性；只有 source、consumer、test/receipt 三者闭合才允许 Closed。

本轮不把以下概念互相替代：

1. Dynamic Scene Session 是开发/运行会话 World 投影。
2. native plugin/VM state 是进程内热重载与生命周期状态。
3. WOC `CommittedSnapshot` 是 fixed-tick candidate/rollback/network projection。
4. 玩家 SaveGame 由 game/platform-user/profile/slot 拥有。
5. server checkpoint 由 shard/session/authority epoch 拥有。
6. network snapshot 由 replication generation 拥有。

### 2.3 动态证据边界

本轮为 review-only，只修改审查报告与索引。没有执行 Cargo、真实 Editor/Vampire/WOC、平台 SDK、断电、跨进程竞争、历史存档、cloud conflict、服务器恢复、fuzz、24h soak 或多 GiB 规模测试。因此任何依赖这些证据的门禁不得 Pass，也不能根据“有 API/测试文件”推断能力已完成。

## 3. 必须保留的真实基础

1. 保留 Dynamic Scene 的 reflect registry、schema generation、entity remap、隔离 preflight 和 fallible adapter write，作为 World participant 的候选底层事务。
2. 保留 Session archive 的 slot/metadata、manifest/index、sealed artifact、preview/retention 及 bounded keyed writer，但把 547-function 组合 facade 收敛到内部 substrate。
3. 保留 Runtime Interface 的 `VersionedSchema`、text/binary envelope 与 `MigrationChain`，扩展为 Save envelope/participant catalog，而不是复制第三套 migration 框架。
4. 保留 core resource transaction 的 staged write、journal、owner lock、file/parent sync、recovery 与 kill-point tests，抽取为 durable blob primitive。
5. 保留 preferences 的 typed error、capacity/permission/corrupt/transient 分类与 platform-target injection；它继续服务小型设置，不直接承载大存档。
6. 保留 native plugin state schema callback、VM blob/migration 和 hot-reload rollback；只有显式注册的 Save participant 才可进入长期持久化。
7. 保留 WOC committed generation/tick/digest/candidate-publication 概念，作为 server checkpoint contributor；simulation digest 不冒充 storage integrity。
8. 保留 Runtime05、Runtime12、Runtime24、Runtime38、Interface02、Plugins01、App03/05/06 与 Editor24 的既有 owner，Runtime129 只做跨 owner closure 与 current-source 裁决。

## 4. 当前真实链路与断路

| 链路 | 当前源码事实 | 裁决 |
|---|---|---|
| 产品入口 | 八个 SaveGame/Checkpoint 产品符号均为 0；Session 只有两层 public re-export | 没有 engine-owned service、capability 或 terminal receipt |
| Editor | Save/Open/Load action 返回 `AutoSave_01`、`schema v4` 等固定文本 | 仍是 fixture，不是 runtime operation projection |
| Slot identity | 只 trim 并拒绝空值 | 缺 game/user/profile/platform namespace、Unicode/case/reserved policy |
| Archive identity | format v1；lineage/revision 来自进程内原子状态 | 重启、外部 writer、跨设备与 server authority 不可比较 |
| Capture | `from_world` 捕获注册为 `serializable` 的 component/resource | `serializable` 被误当成 `savable`，瞬态和敏感状态无 deny-by-default policy |
| Restore | writable field 只含 `serializable && editable`，其他字段不写且不报错 | 已确认静默数据丢失路径，禁止进入 SaveGame participant |
| Restore publication | apply 是 append spawn；level restore 先建空 World，再替换并单独写 metadata | 无 safe point、participant closure、epoch CAS、rollback/resume |
| Migration | Dynamic Scene 有 v0->v1->v2；archive 外层只接受精确 v1 | 通用迁移底座存在，Save catalog/planner/support window 缺失 |
| Encoding | canonical JSON；artifact 同时持对象图、manifest/index/statistics 和完整 bytes | 512 MiB 上限不能证明 streaming，峰值 RSS 仍可能多份 resident |
| Integrity | slot index 使用 `DefaultHasher`；没有持久 strong digest/AEAD | bit rot、截断、内容错配和 tamper 不可可靠分类 |
| Durability | Session `BufWriter::flush()` 后 rename；无 file/parent sync | 成功不等于断电后 durable |
| Recovery | temp/backup 清理多处 `let _`；无 journal/startup state machine | 失败 disposition 和 last-known-good 不可信 |
| Concurrency | `COMMITTED_PATH_REVISIONS` 是进程全局 `HashMap` | 跨进程、重启、cloud/external writer CAS 不成立 |
| Better primitive | core resource transaction 已有 journal/lock/sync/recovery | Session 复制了较弱 authority，必须收敛复用 |
| Missing | `load_or_empty_from_path` 把 NotFound 视为空 archive | first-create、wrong user/path、数据丢失和 catalog 损坏被混淆 |
| Operation | writer 有容量、deadline、ticket/outcome/shutdown | 没有端到端 cancel/progress/stage 与 capture/migration/storage 一致状态机 |
| Cloud/platform | preferences 有 target injection，但无 save provider、platform user、quota 或 cloud | 小型偏好存储不能冒充 platform save system |
| Native/VM | state save/restore/migrate 只被 hot reload/lifecycle 消费；VM 默认可空/no-op | 非长期 Save participant，无 required/optional 和持久兼容承诺 |
| WOC | writer WOS118；main/docs WOS113；普通 tick clone+encode 完整 state | schema identity 分裂且 O(world bytes) 热路径未关闭 |
| Vampire | save 常量，restore 忽略输入 | 只能证明 lifecycle callback 可调用 |

## 5. 对 Runtime40 旧结论的纠正与保留

| 旧结论 | 当前裁决 |
|---|---|
| Session archive 很宽但不是 SaveGame | 保留；当前 570 文件 owner/tests 仍无产品 consumer |
| Dynamic Scene 有隔离 preflight | 保留并增强表述；当前 generation/schema/change-tick bound plan 更强，但 capture/restore 字段集合仍不对称 |
| Runtime Interface 有 migration 底座 | 保留；它只处理 schema value 迁移，没有 participant catalog、支持窗口、预算/cancel 与 content reconciliation |
| core resource transaction 比 Session IO 更强 | 保留；强 primitive 仍为 crate-private 且 Session 未复用，不能把“同仓存在”计作 durability 关闭 |
| WOC 是 snapshot 而非 SaveGame | 保留；current writer/docs identity 仍分裂，且 tick 仍完整 clone/encode |
| Editor/Vampire 只是假产品证据 | 保留；固定反馈文本与常量 state 未变化 |
| 旧报告新增 72 P1/16 P2 | 不新增编号；本轮只重判 current status，防止重复 authority |

## 6. P0 所有权路由：不重复登记

| canonical blocker | 唯一 owner | Runtime129 依赖 |
|---|---|---|
| 没有 SaveGame service、slot repository、participant/schema/migration/cloud authority | Editor24 P0-3 | Runtime129 M1-M6 提供 runtime truth，Editor 只投影 receipt |
| Dynamic Scene archive 不能直接改名为 SaveGame，World snapshot 会静默遗漏 | Editor24 P0-4 + Runtime05 | M2/M3 修复 projection 与无损 restore 后才允许接入 |
| WOS writer/reader/schema/docs identity 分裂 | App03 P0-5 | WOC participant 与历史 fixture 等其 hard cut |
| WOC transaction 不能证明 VM 内部 rollback | App03 P0-6 + Runtime12 | capture 必须消费 isolated candidate 或 restore token |
| WOC 普通 tick 全量 state 和 decoder 副作用 | Runtime12 P0-3/P0-5 | checkpoint 不得固化非确定 restore 或 O(world bytes) tick |
| Vampire 常量 state fixture | App06 P1-19 | App06 提供真实玩法 participant 和跨进程 oracle |

## 7. P1：Service、Identity、Participant 与 Ownership

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| SAV-P1-001 | Open | 产品符号和 owner 均为 0 | 建立唯一 `ISaveGameService`/`ICheckpointService`、capability、typed request 与 terminal receipt |
| SAV-P1-002 | Open | 无 request ID、base generation、idempotency | operation 绑定稳定 ID、expected generation、retry semantics 与 provenance |
| SAV-P1-003 | Open | 只有裸 slot/path，无复合身份 | 引入 game/title、platform user、profile、slot 强类型与 canonical encoding |
| SAV-P1-004 | Partial | slot 已 trim/拒绝空值，但无其他规则 | 冻结长度、字符、Unicode normalization、case、reserved name 与 display name 分离 |
| SAV-P1-005 | Open | 无 participant registry | owner 以稳定 ID 注册 capture/restore/schema/dependency/capability |
| SAV-P1-006 | Open | 无 required/optional participant | required 失败使事务失败；optional omission 写入稳定 receipt |
| SAV-P1-007 | Open | 无 participant phase/dependency graph | 定义 prepare/capture/finalize 与 preflight/apply/commit/rollback DAG，拒绝环 |
| SAV-P1-008 | Partial | plugin/type identity 与 generation 有底座，但没有持久 participant identity | 使用 namespace + stable UUID/schema family；rename 只经 alias/migration |
| SAV-P1-009 | Partial | plugin lease/drain/unload 存在，但 Save payload owner 生命周期不存在 | registry generation、lease、orphan payload 和 unload blocker 成为 Save 合同 |
| SAV-P1-010 | Partial | build/plugin/content catalog 分散存在，未绑定 Save admission | envelope 记录 build/content/plugin set 与兼容 fingerprint，load 先 admission |
| SAV-P1-011 | Open | local player、server authority、client principal 未定义 | request 强制 authority/principal，client 不可写 server-owned participant |
| SAV-P1-012 | Open | 无 Save capability truth | capability 由 storage、migration、cloud、encryption、participant closure 共同计算 |

## 8. P1：Capture、Snapshot、Restore 与 World Consistency

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| SAV-P1-013 | Open | preflight 不是 capture safe point；无 simulation quiescence | 在 schedule barrier 获取 generation lease，冻结结构变更或消费 COW snapshot |
| SAV-P1-014 | Open | Dynamic Scene 默认捕获全部 serializable 对象 | 显式 Save policy/participant projection，瞬态状态 deny-by-default |
| SAV-P1-015 | Open | reflection 仍把 serializable 当 Save 候选 | 分离 asset/scene/network/save/editor/diagnostic 与保护分类 |
| SAV-P1-016 | Open | `serializable && !editable` 可 capture 后静默不恢复 | 专用无损 restore adapter，或 capture/preflight 前 typed reject |
| SAV-P1-017 | Partial | type/field descriptor 与 schema fingerprint 有底座，缺长期 Save catalog | 编译 stable type/field ID、wire type、default、rename、removed-field 与 fingerprint |
| SAV-P1-018 | Open | global resource 的 world/profile/session lifetime 未声明 | resource participant 明确 owner scope、singleton key 与跨 world 规则 |
| SAV-P1-019 | Partial | runtime entity generation/remap 存在，缺跨 Save stable identity | 复用 Runtime24，建立 local object table 与 external reference table |
| SAV-P1-020 | Partial | Dynamic Scene 先建实体/remap 再 apply，但无完整外部引用策略 | decode 先建 object/ID 表，再 resolve；missing/stale/cycle 按 policy 处理 |
| SAV-P1-021 | Open | 无 transient catalog/rebuild closure | 排除 cache/GPU/physics broadphase/network handle，并测试 rebuild hook |
| SAV-P1-022 | Partial | artifact/writer 有 bytes/deadline admission，capture 无独立 CPU/allocation/cancel | admission 预估并在 producer 内执行 deadline/cancel/budget，禁止半成品发布 |
| SAV-P1-023 | Open | 每次仍是完整 World snapshot | participant 提供 dirty frontier/COW page；autosave 增量且完整快照限频 |
| SAV-P1-024 | Open | multi-world/travel capture 顺序未定义 | Gameplay owner 提供 world context set、travel barrier 与原子切换 receipt |

## 9. P1：Envelope、Schema、Migration、Integrity 与 Encoding

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| SAV-P1-025 | Partial | nested Dynamic Scene 有迁移，archive 外层仍精确 v1 | 用 `VersionedSchema` 建 Save envelope family 与连续 migration graph |
| SAV-P1-026 | Open | 无顶层 SaveGame/checkpoint header | header 包含 magic/kind/schema/identity/generation/build/time/flags |
| SAV-P1-027 | Partial | Session 有 slot manifest/index，但没有 participant table | 记录稳定 ID、schema、required、codec、sizes、digest、dependency 与 payload range |
| SAV-P1-028 | Open | unknown participant/field 无 opaque preservation policy | 定义 preserve/quarantine/optional skip/required failure |
| SAV-P1-029 | Partial | 有通用 `MigrationChain`，无跨 participant planner | 验证源/目标 catalog、依赖、支持窗口并产生可审计 plan |
| SAV-P1-030 | Open | migration 无 deadline/cancel/memory/determinism 合同 | bounded pure transform 与 content reconciliation 分离 |
| SAV-P1-031 | Open | downgrade 与长期支持窗口未定义 | 声明 min reader/writer、forward preservation 与不可逆迁移确认 |
| SAV-P1-032 | Open | 没有持久 strong checksum/content address | root/manifest/participant/chunk 使用稳定强 hash 并分类损坏 |
| SAV-P1-033 | Open | 无 compression policy | per participant/chunk 记录 codec/version/dictionary 与原始/压缩尺寸 |
| SAV-P1-034 | Open | 无 encryption/authentication/key rotation | provider 管理 AEAD、key ID、nonce、rotation 与不可恢复错误 |
| SAV-P1-035 | Partial | 有 512 MiB cap/bounded output，但 seal 保留完整对象图和 bytes | bounded streaming/chunk writer、spool/paged buffer，资格门限制峰值 RSS |
| SAV-P1-036 | Partial | Session/Dynamic Scene 有 typed error，不能定位 Save participant/chunk/migration | error 指向 header/participant/chunk/path，保留原件并导出 repair report |

## 10. P1：Platform Storage、Durability、Async 与 Recovery

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| SAV-P1-037 | Open | caller 仍可传任意物理 path | `IPlatformSaveStorage` 将 logical identity 映射到 sandbox path |
| SAV-P1-038 | Partial | Session 有丰富 path/slot 操作，但不是 provider，也不具备完整分页/receipt 合同 | 提供 async enumerate/exists/read/write/delete/rename/copy 与 metadata/generation |
| SAV-P1-039 | Open | 多用户/profile 路径和权限未隔离 | provider 绑定 platform principal/session，用户切换使旧 lease 失效 |
| SAV-P1-040 | Open | Session 成功路径仍只 flush | 复用 shared durable primitive，file 与 parent sync 是 commit 前置 |
| SAV-P1-041 | Open | Session 私有弱 atomic 实现仍未复用 resource transaction | 抽取通用 durable blob transaction，禁止第三套 Save IO authority |
| SAV-P1-042 | Open | `.tmp/.bak` 无 journal/startup recovery | intent/staged/retired/committed/cleanup 状态机确定 roll-forward/back |
| SAV-P1-043 | Open | stale revision 仍是进程内 map | 持久 generation/etag、owner lock 与 CAS 覆盖重启/跨进程/external writer |
| SAV-P1-044 | Open | lineage 不同仍可覆盖旧内容，外部 writer 不可见 | 冲突保留双方；force overwrite 必须显式权限和 receipt |
| SAV-P1-045 | Open | NotFound 仍可映射为空 archive | 区分 missing、first-create、wrong user/path、corrupt catalog 与 IO failure |
| SAV-P1-046 | Partial | writer 有 bounded lane、ticket、deadline、outcome；无端到端 cancel/progress | operation 覆盖 queued/capturing/encoding/staging/committing/syncing/terminal |
| SAV-P1-047 | Open | 无 quota/free-space/reservation/platform lifecycle | 写前 reserve；处理 suspend/device removal/account loss 并释放 reservation |
| SAV-P1-048 | Partial | retention 有 count/tag/time/protected ID，缺容量和平台政策 | 覆盖 bytes/age/class/quota/backup/protected generation/tombstone |

## 11. P1：Cloud、Network、Security 与 Server Checkpoint

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| SAV-P1-049 | Open | 无 cloud provider | 定义 capability、object identity、upload/download/list/delete 与 typed availability |
| SAV-P1-050 | Open | 无 etag/base generation/conflict object | conditional write；冲突保留 local/remote/base 并交给策略/用户决策 |
| SAV-P1-051 | Open | offline save 无 journal/idempotent sync | 本地 operation log 可重放/合并/取消，重启不重复上传/删除 |
| SAV-P1-052 | Open | 跨设备 causal ordering 未定义 | 依赖 server generation/causal metadata，禁止 wall-clock newest 覆盖 |
| SAV-P1-053 | Open | 平台 UI/login/logout/controller-user mapping 未接入 | provider init、user selection、session invalidation 与 callback thread 合同 |
| SAV-P1-054 | Open | client/server capture/load 权限未定义 | server state 只由 authoritative checkpoint service 写，client 仅写 owned profile |
| SAV-P1-055 | Open | App05 无可运维 server checkpoint repository | shard/realm/session/authority epoch identity、lease、retention 与 restore orchestration |
| SAV-P1-056 | Open | 无 anti-rollback/tamper competitive policy | server monotonic generation、signature/authentication 与 replay rejection |
| SAV-P1-057 | Open | PII/account/chat/social/payment 无数据分类 | participant 字段声明 privacy/region/retention/export/delete，默认最小化 |
| SAV-P1-058 | Partial | reader 只有总 raw bytes 上限，无 depth/count/ratio/CPU 等结构预算 | header-first admission、chunk/depth/count/ratio bound 与 fuzz 成为前门 |
| SAV-P1-059 | Open | cloud/local key owner 未定义 | platform secure storage/KMS 持有 key，脚本/plugin 不取得长期 key |
| SAV-P1-060 | Open | 无 Save telemetry privacy 合同 | 只记录稳定 code/size/stage/latency/provider ID，slot 与 payload 脱敏 |

## 12. P1：Product Integration、Performance、Testing 与 Authority

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| SAV-P1-061 | Partial | WOC 有 committed snapshot，但没有 file/slot/cloud repository | Runtime12 稳定后实现 WOC checkpoint participant 与 App05 repository |
| SAV-P1-062 | Open | `saveState` 只返回独立 `restoredState` | real backend 从一致 candidate 导出 versioned participant |
| SAV-P1-063 | Open | writer WOS118，main/docs/contract 仍 WOS113 | 服从 App03 hard cut，生成单一 writer/reader/docs identity 与 golden |
| SAV-P1-064 | Open | 巨型手写 codec 仍耦合 tick/checkpoint/migration | generated bounded codec、COW snapshot、独立 migration/reconciliation |
| SAV-P1-065 | Open | WOC FNV digest 仍不是持久完整性或认证 | simulation digest 与 storage strong hash/AEAD 分开 |
| SAV-P1-066 | Open | Vampire save 常量、restore 丢输入 | App06 提供真实 progression/world participant、旧 fixture 与 resume oracle |
| SAV-P1-067 | Partial | scene save ticket、WOC/plugin snapshot 是底座，但无统一 Gameplay Save caller | GameInstance/WorldContext/LocalPlayer 只经 service 保存，travel/restart 消费 receipt |
| SAV-P1-068 | Open | Editor 仍展示固定 AutoSave_01/sample 文本 | capability 未 qualified 时禁用，之后只投影真实 catalog/conflict/operation |
| SAV-P1-069 | Open | 无玩家 save/load/overwrite/delete/conflict UX | 实现 cancel、confirm、failure recovery、quota、corrupt、cloud choice 全流程 |
| SAV-P1-070 | Open | 无真实玩法变化后跨进程恢复测试 | Vampire/WOC E2E 比较 stable state 并验证 transient rebuild |
| SAV-P1-071 | Open | 无 1 MiB 到多 GiB latency/RSS/IO/compression sweep | 固定数据集记录 stall、peak RSS、read/write p50/p95/p99 与 dirty ratio |
| SAV-P1-072 | Open | 无 local/cloud/offline/server/platform fault matrix | required CI 注入 kill point、disk full、permission、corrupt、conflict、cancel/provider loss |

## 13. P2：完整性、可用性与维护性

| ID | 状态 | 改进项 | 目标 |
|---|---|---|---|
| SAV-P2-001 | Open | slot thumbnail/screenshot metadata | 独立 optional blob 与尺寸/格式预算，render failure 不阻断核心存档 |
| SAV-P2-002 | Open | 细粒度 async progress | 阶段和 bytes/participants 进度单调、可节流 |
| SAV-P2-003 | Open | Save diff inspector | schema-aware 只读 diff，默认不暴露敏感数据 |
| SAV-P2-004 | Open | checkpoint 类别 | manual/autosave/quicksave/suspend/server/admin/imported 各有 policy |
| SAV-P2-005 | Open | delta compression | 明确 base generation/chain length，定期 compact，base 缺失可恢复 |
| SAV-P2-006 | Open | autosave 调度 | coalescing、minimum interval、dirty participant、frame/load budget 与 backoff |
| SAV-P2-007 | Open | suspend/emergency save | 短 deadline 和最小 required set，不假装等价完整 save |
| SAV-P2-008 | Open | cross-title/profile 隔离 | title/environment/branch namespace 防止测试与正式档互相覆盖 |
| SAV-P2-009 | Open | 本地化错误与操作文本 | typed code 映射文案，不直接显示 path/OS string |
| SAV-P2-010 | Open | accessibility | 保存列表、进度、冲突、确认支持键盘、读屏、缩放和色觉 |
| SAV-P2-011 | Open | DLC/mod orphan 工具 | 列 missing owner、保留 opaque payload、导出/修复且不静默删除 |
| SAV-P2-012 | Open | replay/save/checkpoint 术语 | API 明确 identity、determinism、lifetime，不共享模糊 Snapshot 命名 |
| SAV-P2-013 | Open | deterministic diagnostics | 同一错误产生稳定 code/path/participant/generation |
| SAV-P2-014 | Open | repair/export utility | 只读校验、manifest 导出、LKG 恢复与授权后的脱敏包 |
| SAV-P2-015 | Open | 参考/回归 benchmark | 固定硬件/数据集比较趋势，不以未同条件测量声称超过 Unreal |
| SAV-P2-016 | Open | SDK 与文档 | game/plugin 作者获得 participant、migration、budget、安全和测试规范 |

## 14. 五引擎参考裁决

| 参考 | 可借鉴工程边界 | 不能误读为 |
|---|---|---|
| Unreal `ISaveGameSystem` / `SaveGameSystem.cpp` | platform feature、platform user、exists/list/save/load/delete、异步串行 pipe、game-thread completion | 默认 generic file save 已证明 fsync、cloud conflict、cancel、streaming 或多 GiB 资格 |
| Unreal `GameplayStatics` / `ULocalPlayerSaveGame` / AsyncAction | `SAVG` header、file/package/engine/custom version、SaveGame class identity、LocalPlayer/platform user/slot 绑定、sync/async 产品入口 | 反射序列化所有对象即可成为一致 participant transaction |
| Unreal GDK SaveGame | per-user provider、container/blob、quota、sync/conflict UI、sign-out 生命周期 | 平台 SDK 自动解决游戏 participant、schema migration 和 server checkpoint |
| Godot `FileAccess` / `ResourceSaver` | `user://` 路由、typed file errors、compressed/encrypted access、format saver registry、resource UID/path | 资源 IO/saver registry 等价于玩家 SaveGame authority |
| Fyrox Visitor | magic/version、region tree、typed field、read/write mode、skip/rename/optional 方向 | reader 按输入长度分配的 object serializer 已具备 untrusted-save budget/security |
| Bevy DynamicWorld / Scene | type registry、reflection projection、entity reference map、先建实体再 apply、资源最后插入、dependency-ready spawn/despawn | scene/world serialization 自动决定 user/profile/slot、durability、cloud 和长期支持窗口 |
| Unity Graphics serialization utilities | `SerializableEnum`/`SerializedDictionary` 的 Unity serializer adapter 模式 | 该 Graphics 镜像含可验证的 Unity SaveGame 产品 authority；本报告不对闭源实现作推断 |

Unreal 是本轮产品接口的领先参考，但 Zircon 的目标不能停在 Unreal generic file save。要主张性能和可靠性优于 Unreal，必须增加持久 CAS、file/parent durability、bounded streaming、可取消 migration、cloud conflict object、server checkpoint 和 raw benchmark receipt，而不是用功能缺失或较小样例取得虚假优势。

## 15. 目标架构与所有权

```text
Gameplay / Server / Editor diagnostics
  -> ISaveGameService / ICheckpointService
     -> Identity + Principal + Capability + Request admission
     -> ParticipantRegistry + SchemaCatalog + DependencyPlan
     -> CaptureTransaction / RestoreTransaction
     -> VersionedEnvelope + ChunkCodec + MigrationPlanner
     -> PlatformSaveStorage / ServerCheckpointStorage
     -> CloudSyncProvider + ConflictResolver
     -> TerminalReceipt + Diagnostic + Privacy-safe Telemetry

Participants
  -> WorldDynamicSceneParticipant
  -> GameplayFrameworkParticipant
  -> ScriptStateParticipant
  -> Explicit NativePluginParticipant
  -> Product-owned WOC/Vampire participants
```

| owner | 拥有 | 不拥有 |
|---|---|---|
| Runtime129/Runtime40 Save service | request、identity、participant、capture/restore、envelope、storage orchestration、receipt | Editor UI、具体游戏字段、平台 SDK 内部实现 |
| Runtime Interface02 | 稳定 DTO、schema/migration、typed error 公共合同 | 物理 slot 目录、cloud provider、World capture |
| Runtime04/core resource IO | durable blob transaction、journal、sync、recovery | Save identity、participant 与产品 policy |
| Runtime05/24 | World projection、stable entity/reference、generation/currentness | platform user/profile/slot/cloud |
| Runtime38 | GameInstance/WorldContext/LocalPlayer/travel/restart 接入 | 物理存储与 cloud conflict |
| Plugins01 | plugin identity/ABI/state owner/unload lifecycle | 默认持久化所有 hot-reload bytes |
| Runtime12 + App03/05 | WOC snapshot/schema/transaction/server repository | 第二套通用 Save service |
| App06 | Vampire 真实玩法 participant/E2E | 通用 envelope/provider |
| Editor24 | Save diagnostics 与 authoring projection | 自建 slot catalog、假 migration、直接文件 IO |

## 16. 分层里程碑

### M129-0 · Truth Freeze 与 RED 证据

1. SaveGame/Checkpoint capability 保持 Unavailable，删除/禁用无 receipt 的成功文本。
2. 固化 `serializable && !editable` 静默丢失、NotFound-to-empty、flush-only、WOS113/118 和 Vampire 常量 state RED fixtures。
3. 输出 owner/caller/deletion matrix，明确 Session、hot reload、network snapshot、SaveGame、server checkpoint 五类状态。
4. 记录 1/10/100/512 MiB capture/encode/write/read peak RSS 和 kill-point 基线。

### M129-1 · Identity、Service 与 Capability

1. 冻结 typed identity、principal、request、operation ID、base generation、receipt 和 error DTO。
2. 建立唯一 SaveGame/Checkpoint service，产品 caller 不得传物理 path。
3. capability 由 provider/participant/migration/security qualification 计算并 fail closed。

### M129-2 · Participant 与 Schema Catalog

1. 建 registry、stable participant/type/field identity、required/optional、dependency DAG 和 owner lease。
2. 建 transient/privacy/lifetime catalog，plugin/DLC unload 和 orphan payload 有明确 disposition。
3. World/gameplay/script/plugin/product projection 显式 opt-in。

### M129-3 · Consistent Capture 与 Transactional Restore

1. 在 schedule safe point 捕获 generation-bound bounded snapshot。
2. 修复 Dynamic Scene 无损字段、stable entity、external reference 与 unknown policy。
3. restore 执行 decode/migrate/preflight/apply/commit/rollback，required failure 保持原 World 可运行。
4. 增量 checkpoint 消费 dirty/COW frontier，普通 tick 不再全量 encode/copy。

### M129-4 · Envelope、Migration 与 Codec

1. 建 versioned header、participant manifest、chunk、strong hash、compression 和可选 AEAD。
2. migration planner 覆盖 catalog/dependency/support window/budget/cancel/downgrade/opaque preservation。
3. 采用 bounded streaming/spool，限制 decode depth/count/ratio/CPU/allocation。

### M129-5 · Platform Storage、Durability 与 Recovery

1. 抽取 shared durable blob transaction，Session 删除私有 flush-only writer authority。
2. 实现 logical slot provider、enumerate/read/write/delete/rename/copy、quota/reservation 与 lifecycle。
3. journal、file/parent sync、startup recovery、persistent CAS/owner lock 和 terminal durability receipt 全闭合。

### M129-6 · Cloud、Server 与 Security

1. cloud provider 实现 conditional write、offline journal、conflict object、tombstone 与 user lifecycle。
2. server checkpoint 实现 shard/authority epoch、lease、retention、replication barrier 与 disaster restore。
3. privacy/key/anti-tamper/anti-rollback/telemetry policy 进入 provider qualification。

### M129-7 · 产品闭环与竞争资格

1. Vampire 先完成小型真实玩法 roundtrip；WOC 在 schema/kernel blocker 关闭后接大规模 checkpoint。
2. 玩家 UI 覆盖 list/save/load/overwrite/delete/cancel/corrupt/quota/cloud conflict。
3. required CI 运行历史 fixture、跨进程、kill point、fuzz、跨设备模拟、24h soak 和规模 sweep。
4. 只有同硬件、同数据集、同可靠性语义 raw receipt 才能声明性能优于 Unreal。

## 17. 资格门当前状态

| Gate | 状态 | 必须证明的证据 |
|---|---|---|
| SAV-G01 | Fail | public surface 只有一个 SaveGame service 和一个 Checkpoint service，caller 无物理 path |
| SAV-G02 | Fail | game/user/profile/slot 与 server shard/epoch 使用不可互换强类型 identity |
| SAV-G03 | Fail | operation ID/base generation/idempotency/receipt 在 retry/restart 后一致 |
| SAV-G04 | Fail | capability 由实际 provider/participant/migration qualification 计算 |
| SAV-G05 | Fail | registry 拒绝 duplicate ID、dependency cycle、stale generation、unload race |
| SAV-G06 | Fail | required failure 使事务失败，optional omission 有稳定 diagnostic |
| SAV-G07 | Fail | build/content/plugin catalog mismatch 在 World mutation 前拒绝 |
| SAV-G08 | Fail | SaveGame/checkpoint/network/hot-reload snapshot 不能类型误传 |
| SAV-G09 | Fail | qualified safe point 下无 mixed-generation capture |
| SAV-G10 | Fail | transient cache/handle/GPU/network state 不进入 Save payload |
| SAV-G11 | Fail | non-editable serializable 字段无损恢复或 capture 前拒绝 |
| SAV-G12 | Partial | runtime generation/remap 有底座；跨 load stable identity/external reference 未证明 |
| SAV-G13 | Fail | multi-world/travel capture 有 barrier，失败不发布半恢复 World |
| SAV-G14 | Partial | artifact/writer 有局部 bytes/deadline；producer CPU/allocation/cancel 未闭合 |
| SAV-G15 | Fail | dirty/COW 与 full snapshot 等价，错误 base generation 被拒绝 |
| SAV-G16 | Partial | Dynamic Scene 有隔离 preflight；无 required participant rollback/resume |
| SAV-G17 | Partial | nested scene 有 schema/header；无 Save envelope identity/golden |
| SAV-G18 | Partial | Dynamic Scene 有旧版 migration；无每个 supported Save 版本 corpus |
| SAV-G19 | Fail | unknown optional opaque preserve，unknown required fail closed |
| SAV-G20 | Partial | 通用 migration chain 存在；deterministic budget/cancel/reconciliation 未闭合 |
| SAV-G21 | Fail | root/manifest/chunk/participant strong hash 可定位 bit rot/截断 |
| SAV-G22 | Fail | compression ratio/output bound 前置拒绝 decompression bomb |
| SAV-G23 | Fail | qualified key provider/AEAD 阻止 tamper 进入 restore |
| SAV-G24 | Fail | 1 MiB 至目标上限 streaming peak RSS 满足版本化预算 |
| SAV-G25 | Fail | 成功 receipt 只在 file 与 parent directory sync 后发布 |
| SAV-G26 | Fail | 每个 journal kill point 重启后确定 roll-forward/back |
| SAV-G27 | Fail | 同进程/跨进程/重启/external writer 服从持久 CAS/lock |
| SAV-G28 | Fail | missing/first-create/wrong-user/corrupt/permission/disk-full 不互相映射 |
| SAV-G29 | Partial | Session 有广泛 path/slot 操作；provider pagination/cancel/receipt 不完整 |
| SAV-G30 | Fail | quota reservation/suspend/account loss/device removal 不泄漏或谎报成功 |
| SAV-G31 | Fail | cloud conditional conflict 保留 local/remote/base，不用 wall-clock LWW |
| SAV-G32 | Fail | offline journal 在 crash/reconnect/tombstone 下幂等收敛 |
| SAV-G33 | Fail | server checkpoint 只接受 authoritative principal |
| SAV-G34 | Fail | privacy export/delete、retention、key rotation、anti-rollback 有集成测试 |
| SAV-G35 | Fail | Vampire 跨进程 save/load 后 stable state 一致、transient 正确重建 |
| SAV-G36 | Fail | WOC current writer/readers/host/schema/docs identity 一致 |
| SAV-G37 | Fail | WOC 普通 tick 不做 O(world bytes) encode/copy，10k entity budget 通过 |
| SAV-G38 | Fail | 玩家产品完成 save/list/load/overwrite/delete/corrupt/quota/cloud conflict |
| SAV-G39 | Fail | required CI 覆盖 history/fuzz/fault/kill-point/cross-device/24h soak |
| SAV-G40 | Fail | 规模 sweep 有 p50/p95/p99、peak RSS、IO、compression、dirty ratio raw receipt |

## 18. 禁止的临时修补

1. 禁止把 `RuntimeSessionArchive`、native/VM hot-reload state 或 WOC network snapshot 直接改名为 SaveGame。
2. 禁止让 Gameplay、Editor 或 plugin 直接写 caller path，或再建一套 `.tmp/.bak` atomic helper。
3. 禁止把 `serializable` 自动等同于 `savable`，也禁止继续静默跳过不可写字段。
4. 禁止以总 bytes cap 代替 streaming/decode structure budget，或以 FNV/`DefaultHasher` 代替持久 strong integrity。
5. 禁止以进程内 `HashMap` revision 代替持久 CAS，以 `flush()` 代替 durable commit。
6. 禁止把 NotFound 自动解释为新存档，把 corrupt/schema mismatch/plugin missing 自动降级为空 World。
7. 禁止让 optional/best-effort hot-reload restore 语义进入 required Save participant。
8. 禁止继续使用 Editor 固定样例文本、WOC 独立字符串或 Vampire 常量作为产品完成证据。
9. 禁止在没有同场景、同硬件、同可靠性和 raw receipt 时宣称性能优于 Unreal。

## 19. 本轮产出边界

本轮完成 Runtime40 SaveGame/Checkpoint 主题的 current-source 重审、五引擎证据更新、72 项 P1/16 项 P2/40 项门禁重判、目标架构和分层重构路线。没有修改生产代码、测试或平台集成，没有关闭任何 finding，也没有创建新的根 P0。后续实现必须从 M129-0 truth freeze 与 RED evidence 开始，而不是先增加 Save 按钮、slot 文件或 cloud mock。
