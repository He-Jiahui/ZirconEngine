---
title: Runtime Preference / Settings / Scope / Storage / Overlay / Bounded I/O / Generation / Fence / Durability / Migration / Multi-Process / Product Integration 当前源码复审
category: zircon_runtime
report_id: Runtime134
review_date: 2026-08-24
baseline_head: 8dc299a8b65813f692e222a709f951e6ace90be6
baseline_epoch: 393
verification_head: 8dc299a8b65813f692e222a709f951e6ace90be6
verification_epoch: 393
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/platform/preferences
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io
  - zircon_runtime/src/platform/preferences
  - zircon_runtime/src/platform/service_types
  - zircon_runtime/src/platform/capability
  - zircon_app/src/entry/platform_preferences.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_editor/src/core/settings
  - examples/woc/native/apps/woc_client/src/preferences
  - examples/woc/native/apps/woc_client/src/input/gamepad
  - examples/woc/native/apps/woc_client/src/input/keybind
  - examples/woc/native/apps/woc_client/src/windows/inventory.rs
  - examples/woc/native/apps/woc_client/src/windows/routes.rs
tests:
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/tests.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/tests
  - zircon_runtime/src/platform/preferences/persistence/tests.rs
  - zircon_runtime/src/platform/preferences/persistence/tests
  - zircon_runtime/src/platform/tests/preferences.rs
  - zircon_runtime/tests/runtime11_preference_backend_authority.rs
  - zircon_editor/src/core/settings/tests
  - examples/woc/native/apps/woc_client/tests
plan_sources:
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/zircon_runtime/frameworks/05/failure-2026-07-19-cross-platform-preference-storage-service.md
  - docs/plans/zircon_runtime/frameworks/05/failure-2026-07-22-preference-quota-error-kind-toolchain-drift.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-31-preference-storage-bounded-persistence-lane.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigContext.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigHierarchy.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameUserSettings.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameUserSettings.cpp
  - dev/godot/core/config/project_settings.h
  - dev/godot/core/config/project_settings.cpp
  - dev/godot/editor/settings/editor_settings.h
  - dev/godot/editor/settings/editor_settings.cpp
  - dev/Fyrox/editor/src/settings/mod.rs
  - dev/Fyrox/editor/src/plugins/settings.rs
  - dev/bevy/crates/bevy_render/src/settings.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipeline/RenderPipelineGlobalSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipeline/RenderPipelineGlobalSettingsUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipeline/RenderPipelineGraphicsSettingsContainer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/RenderPipeline/RenderPipelineGlobalSettingsUtilsTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/HDRenderPipelineGlobalSettings.Migration.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime134 · Preference / Settings / Storage / Durability 当前源码复审

## 1. 结论

当前实现已经不是最初级的内存字典。Runtime有中立`PreferenceStorage` SPI、受限key/read、read-your-write overlay、entry/byte admission、same-key generation coalescing、fence prerequisite、panic terminal、pre-start deadline、shutdown report、atomic-file backend和基础diagnostics。Editor又建立了类型化definition/schema、User/Project/Session三层覆盖、不可变generation snapshot、有界change log、版本化文档、原子写和真实退出时的fence+shutdown。这些都应保留。

但产品级Preference系统仍未成立。Runtime把HostProvided backend一次`write/remove`成功直接标成`Durable`，并未等待其独立`flush`；Platform cleanup也只shutdown lane，不建立最终durability fence。lane全局只有一个active work，active I/O没有合作取消或执行期deadline，而shutdown guard的`Drop`会无界等待，因此一个永久挂起backend既能冻结所有key，也能阻止进程或DLL退出。App默认root固定为`ZirconEngine/preferences`，地址没有product/project/channel/account/profile等owner identity；hashed裸payload没有record revision、schema、digest、writer或CAS，多进程整值写会静默覆盖。

产品接线仍是最大断路。WOC的`StoredClientSettings`、`StoredGamepadBindings`和`StoredKeybinds`没有生产构造点；`refresh_from_storage`与`take_persistence_submission`只有定义和测试caller。Inventory route只产生`PersistInventoryFilter(String)`，没有生产effect consumer。WOC可因此在cold read仍Pending时永久采用default，并且不harvest mutation terminal。其`main`当前只输出identity report。Editor的退出闭环比WOC真实，但仍拥有第二套filesystem authority；生产提交只接通viewport Project snap，User locale/keymap/appearance没有通用save owner。更危险的是Editor lane按setting key分流，却每次重写整个scope文档：当前仅靠全局单active偶然串行，一旦开放跨key并行便会发生同文件竞争。

Runtime45的5项P0本轮均为 **Open**；58项P1为 **47 Open、11 Partial、0 Closed**；14项P2为 **13 Open、1 Partial、0 Closed**；40项资格门为 **32 Fail、8 Partial、0 Pass**。三份Preference相关failure继续保持open。本轮只做静态review和报告，不修改Rust/Cargo，也不以既有unit test替代fresh-process、multi-process、crash、hung-I/O或产品重启资格。

## 2. 审查边界、方法与currentness

### 2.1 冻结物理范围

统计口径：物理UTF-8行、非空行、文件bytes；test declaration匹配`#[test]`，ignored匹配`#[ignore`；fingerprint为按normalized lowercase relative path排序后的`path + NUL + lowercase(file SHA-256) + LF`集合再做SHA-256。Core组包含源文件内嵌测试，专用测试组只统计独立test文件；各组互不用于夸大产品覆盖。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Preference core与Platform/App集成 | **34 / 5,927 / 5,370 / 199,629 / 12 / 2** | `01f8121e12dcfc03c1267f1bd885cbb64ef731218c82a6da5515bfc46d840ed3` |
| Editor与WOC产品consumer | **51 / 10,881 / 9,984 / 355,122 / 25 / 0** | `e5f05826028e0cd4e7f281e8985a554810a9e10641b3f2111e1a277f15acc138` |
| Preference/Settings专用测试 | **15 / 4,914 / 4,511 / 167,453 / 135 / 0** | `a502b41cf87d594c1cad9777b45eae5c6bb9c534aa5b06025d6f4f0f1528dcd8` |
| 五引擎参考选择集 | **20 / 21,544 / 18,385 / 811,232 / 0 / 0** | `00e972855ce8891ef79af4e4c94e6410cf4bfbcca4dddc53edd71845cc116f03` |

参考仓库revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine`没有独立Git元数据，`git -C`会向上解析到Zircon workspace；因此Unreal只以所列物理文件和参考集合fingerprint冻结，不伪造独立revision。

### 2.2 检查方法

1. 逐文件读取framework contract、bounded keyed I/O、Platform backend/overlay/adapter、module/manager/driver/capability与App root/install链。
2. 逐文件读取Editor settings owner、startup/load/save/persistence、context和retained-host shutdown，并反查所有生产`submit`、User/Project mutation与project切换caller。
3. 逐文件读取WOC settings/gamepad/keybind/inventory持久化和相邻模型，反查`Stored*`构造、refresh、submission harvest和route effect consumer。
4. 读取135个专用test declaration并检查关键fixture，区分unit语义、ignored benchmark、同进程reopen与真实产品/跨进程/故障资格。
5. 对Runtime45的5项P0、58项P1、14项P2及40项gate保留原编号逐项重判，不越权关闭Runtime03/25/40、Editor12或App owner。
6. 对照Unreal、Godot、Fyrox、Bevy和Unity Graphics本地源码；参考只用于证明成熟边界和反例，不把不同产品模型机械复制进Zircon。

### 2.3 currentness与共享工作树

- baseline为`8dc299a8b65813f692e222a709f951e6ace90be6` / epoch 393；注册时共享`main`已有3,335个porcelain path，本文不回退或覆盖任何外部修改。
- 本轮开始时Preference相关source diff只见`bounded_keyed_io/lane/coalescing.rs`和`platform/preferences/atomic_file.rs`的格式/import次序变化，不改变本报告语义；最终currentness以后文verification重扫为准。
- Runtime42 manifest与Runtime11 generic task会话正在处理相邻owner；本文写范围严格限于本报告和三级索引，不抢占其源码或failure。
- 本轮不运行Cargo、Editor、WOC、fault injection或benchmark；静态源码能证明未接线和合同缺失，却不能替代动态通过证据。

### 2.4 开放failure

| failure | 当前裁决 |
|---|---|
| `frameworks/05/failure-2026-07-19-cross-platform-preference-storage-service.md` | 保持open；跨平台service骨架存在，但product identity、provider资格、durability和产品闭环仍缺 |
| `frameworks/05/failure-2026-07-22-preference-quota-error-kind-toolchain-drift.md` | 保持open；quota和部分错误映射已存在，但error ABI、总内存预算及正式验证receipt仍缺 |
| `runtime/11/failure-2026-07-31-preference-storage-bounded-persistence-lane.md` | 保持open；lane已bounded并有fence/shutdown基础，但single-active、active hang、observer、复杂度和产品caller未闭合 |

## 3. 当前拓扑与断路

```text
WOC model constructor
  -> PreferenceStorage::snapshot()       [隐式触发cold read]
  -> Pending -> unwrap_or_default()
  -X-> read ticket / wake / production refresh owner

WOC mutation
  -> submit_preference_text() -> Option<Submission>
  -> Stored*.last_persistence_submission
  -X-> production harvest / retry / user-visible failure / shutdown fence

Editor startup/project switch
  -> synchronous SettingsStore filesystem read
  -> independent typed SettingsAuthority + immutable snapshot
  -> Project viewport mutation -> SettingsPersistenceService -> whole-document atomic rewrite
  -X-> Runtime Preference schema/address/revision authority
  -X-> generic User-setting persistence submission

Runtime Preference mutation
  -> overlay visible generation
  -> global single-active BoundedKeyedIoLane
  -> backend write/remove
  -> immediately marks Durable
  -X-> declared backend durability level + flush receipt
```

| 层 | 当前事实 | 工程缺口 |
|---|---|---|
| address | `PreferenceKey(namespace,key)`只校验空值、NUL与bytes上限 | 无product/principal/scope/owner/schema/canonical identity |
| read | `snapshot()`会触发I/O，返回Pending/value/durability | 无ticket、wake、typed failure/stale/revision/source |
| overlay | read-your-write、generation防陈旧覆盖、entry/byte reservation | durable/tombstone不可evict，无backend/schema revision，任意failure选择 |
| lane | entry/byte bound、same-key coalescing、fence pin、panic/pre-start deadline | 一个active、无tenant fairness、执行期cancel/deadline、single observer、checked ID exhaustion |
| backend | AtomicFile与HostProvided SPI、worker authority、max+1 bounded read | 无capability contract、record envelope、CAS/watch/transaction/recovery/security |
| lifecycle | module cleanup最多等250ms并保留超时service | 不先flush；最终guard Drop无界等待；replacement/reactivation无generation transaction |
| Editor | 类型schema/layer/snapshot/versioned document/真实close fence | 第二storage owner、同步read、whole-document key错配、User save断路 |
| WOC | 84项setting registry、clamp、typed apply route及keybind repair | product构造/refresh/harvest为0，JSON无版本，scope拼字符串，错误吞成default/Option |

## 4. 必须保留的工程基础

1. 保留`PreferenceStorage`的中立trait与worker authority，扩展地址、capability、revision和receipt，不让consumer直接打开文件。
2. 保留overlay的read-your-write与“陈旧read不能覆盖新visible generation”规则，改造成backend/schema generation-qualified state machine。
3. 保留entry+byte双重admission、same-key coalescing、fence prerequisite、panic terminal和可查询shutdown report。
4. 保留AtomicFile的hashed物理定位、共享atomic staging/commit与max+1 bounded read，但在其上增加versioned record、manifest、recovery和CAS。
5. 保留Editor的definition/schema/three-layer precedence、immutable typed snapshot、bounded delta与locale hot-apply；它们应成为Runtime schema/UI projection的上层consumer。
6. 保留Editor真实退出时`flush_then_shutdown().finish()`的错误传播模式，将同样的durability transaction提升到Platform/App共同owner。
7. 保留WOC的setting range、apply route、keybind repair及inventory模型，把它们注册到versioned schema，而不是继续整份裸JSON。
8. 保留现有unit fixtures作为确定性底座，但明确禁止把manual polling、同进程第二adapter或ignored microbenchmark称作产品资格。

## 5. P0当前源码重判

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PREF-P0-001 | Open | HostProvided `write/remove`成功经lane直接映射`Durable`；独立`flush`未参与mutation terminal，Platform cleanup也不建final fence | capability声明durability level；mutation只报告实际级别，flush返回覆盖operation集合的receipt；shutdown先final fence再drain |
| PREF-P0-002 | Open | WOC三个`Stored*`无生产构造；refresh/harvest只有定义和测试，Inventory persistence effect无consumer，`main`只打印identity | App/WOC host拥有reactive load与mutation/fence terminal，启动等待或订阅、持续harvest、重试/回滚并向UI暴露失败 |
| PREF-P0-003 | Open | lane只有一个global active，active工作无deadline/cancel；250ms cleanup后仍可能在guard `Drop`无界wait | per-key/partition隔离、cooperative deadline/cancel或可遗弃provider进程；所有Drop路径严格有界 |
| PREF-P0-004 | Open | 默认root固定`ZirconEngine/preferences`；任意consumer可构造namespace/key，WOC scope含display name | `ProductStorageIdentity + Principal + TypedScope + NamespaceLease`，物理与逻辑隔离product/project/channel/profile/account |
| PREF-P0-005 | Open | hashed raw file仅atomic replace；无record revision、expected revision、process lock/CAS/watch/merge，WOC/Editor又整文档写 | revisioned record与CAS/conflict receipt；按provider选择single-writer broker/lock或typed merge；双进程与kill/restart资格 |

## 6. P1当前源码重判

### 6.1 Contract、Scope、Schema与权限

| ID | 状态 | 当前源码证据与后续方向 |
|---|---|---|
| PREF-P1-01 | Open | Runtime仍只有namespace/key字符串；建立含product/project/principal/scope/namespace/logical key的`PreferenceAddress` |
| PREF-P1-02 | Partial | Editor有User/Project/Session三种局部scope；Runtime/WOC没有Account/Profile/World/Plugin taxonomy或共享继承规则 |
| PREF-P1-03 | Open | namespace无catalog owner/lease；composition分配owner identity并在读写时授权 |
| PREF-P1-04 | Open | 只拒绝NUL并按bytes限长；冻结Unicode、大小写、separator、confusable canonicalization |
| PREF-P1-05 | Partial | Editor有`SettingValue + SettingSchema`，Runtime payload仍是`Arc<[u8]>`；统一schema/content-type/codec/version envelope |
| PREF-P1-06 | Partial | Editor有Default/User/Project/Session precedence，Runtime Preference没有source layer/provenance，WOC只有整值 |
| PREF-P1-07 | Partial | Editor有range/type/restart字段且WOC会clamp；Runtime registry、dependency/apply/revert/dirty comparator仍缺 |
| PREF-P1-08 | Partial | Editor使用versioned envelope但只接受v1并拒绝v0/unknown；WOC JSON仍丢future field；补N-2迁移和compat policy |
| PREF-P1-09 | Open | 无secret/PII分类、redaction、encryption或平台secret-store分域 |
| PREF-P1-10 | Open | 无multi-key prepare/commit/abort或明确的原子性拒绝receipt |
| PREF-P1-11 | Open | read无revision/etag，write/remove无expected revision与Conflict |
| PREF-P1-12 | Open | error仍仅5类且backend name为`&'static str`；补Conflict/Schema/Migration/Auth/Cancelled/Deadline/Unsupported/Integrity及correlation/source chain |

### 6.2 Backend、Record、路径与能力

| ID | 状态 | 当前源码证据与后续方向 |
|---|---|---|
| PREF-P1-13 | Open | `backend_kind().is_persistent()`即被视为Supported；定义durability/flush/CAS/watch/transaction/max/thread/cancel capability |
| PREF-P1-14 | Open | `.zrpref`只存裸payload；增加address/schema/revision/digest/flags/writer/version record envelope |
| PREF-P1-15 | Open | 无授权manifest、enumeration、paged reset/export/import，不能靠扫描hash目录管理 |
| PREF-P1-16 | Open | 无header/payload integrity和quarantine；bit flip必须返回Integrity并隔离 |
| PREF-P1-17 | Open | 没有Preference启动recovery/staging sweep/last-known-good receipt；共享atomic primitive不等于恢复协议 |
| PREF-P1-18 | Open | 无root权限、ACL、symlink、安全open或sandbox policy |
| PREF-P1-19 | Partial | Editor Project路径会经`ProjectPaths`解析physical identity；App/User root仍接受相对/未canonical环境路径且无root receipt |
| PREF-P1-20 | Open | mobile/Web/headless只有“必须注入”，无标准provider包、probe与Unavailable reason资格 |
| PREF-P1-21 | Open | backend replacement仅交换Arc，无freeze/flush/drain/generation/cache invalidation/resume transaction |
| PREF-P1-22 | Open | one-shot install和失败cold read没有late install/reconnect及backend-generation retry |
| PREF-P1-23 | Open | capability report只投影backend kind，不能区分planned/installed/probed/healthy/degraded/closed |
| PREF-P1-24 | Partial | 已有queue/overlay/path/backend wall等基础计数；仍无root policy、backend instance/generation、queue age、percentile和last terminal |
| PREF-P1-25 | Open | path cache只按4096 entry FIFO，hit不刷新且无byte budget/generation invalidation |
| PREF-P1-26 | Partial | Permission/ReadOnly和StorageFull/FileTooLarge/Quota已有映射；Busy/NotFound等仍折叠TransientIo且完整source/correlation不稳定 |

### 6.3 Overlay、Read API、Admission与Cache

| ID | 状态 | 当前源码证据与后续方向 |
|---|---|---|
| PREF-P1-27 | Open | `snapshot()`隐藏提交cold read且Pending无handle；拆为pure peek与显式load/subscribe ticket |
| PREF-P1-28 | Open | WOC把missing/failure/not-ready折为None/default；引入Missing/Loading/Ready/Failed/Stale并保留revision/source |
| PREF-P1-29 | Open | 64MiB max-value加metadata quote使128MiB overlay/lane默认预算只能容纳一个最大cold read；改header probe/stream body和schema/tenant limit |
| PREF-P1-30 | Open | Durable value和成功tombstone不能evict，4097个distinct key可永久封满默认overlay |
| PREF-P1-31 | Open | 无invalidate/reload/watch/poll generation及external changed/conflict/stale event |
| PREF-P1-32 | Open | 无按transient/permanent分类的backoff/jitter/deadline/operator retry policy |
| PREF-P1-33 | Open | `known_non_durable_failure()`从HashMap取任意首项；fence应按operation order确定性分页聚合 |
| PREF-P1-34 | Open | retained bytes仍是常量近似并遗漏container/control/backend buffer；建立allocator/RSS校准模型 |
| PREF-P1-35 | Open | adapter submission由global mutex串行且无lock wait、contention或tenant公平指标 |
| PREF-P1-36 | Open | max value只有全局值；由schema/scope/tenant/backend共同裁决并把大blob导向SaveGame/resource |
| PREF-P1-37 | Open | cache entry无backend generation、record revision、schema generation与freshness |
| PREF-P1-38 | Open | timer不可用仍映射CapacityExceeded；增加SchedulerUnavailable/DeadlineUnsupported及degrade policy |
| PREF-P1-39 | Open | `caller_filesystem_wall`被固定为0并由测试锁定；应测queue/backend/fsync/callback wall而非写死 |

### 6.4 Bounded Lane、Fence、取消与生命周期

| ID | 状态 | 当前源码证据与后续方向 |
|---|---|---|
| PREF-P1-40 | Open | global仅一个active；按backend capability提供bounded cross-key concurrency并保持same-key order |
| PREF-P1-41 | Open | 全局FIFO无owner quota、priority、aging和starvation telemetry |
| PREF-P1-42 | Partial | queue partition与fence prerequisite已有线性化/有界改进及ignored benchmark；admission/insertion仍扫描，diverse-key/fence storm累计可O(N²) |
| PREF-P1-43 | Partial | overlay使用checked generation exhaustion；public lane仍信任caller generation，缺跨backend单调验证 |
| PREF-P1-44 | Open | ticket ID/epoch继续`saturating_add`，最终重复或冻结；改checked composite identity |
| PREF-P1-45 | Open | fence pin与多类counter使用saturating arithmetic，可能掩盖accounting invariant破坏 |
| PREF-P1-46 | Open | terminal observer只有一个slot，后注册可覆盖前者；改multi-subscriber/event bus及backpressure |
| PREF-P1-47 | Open | failure code是static string，unknown code降级TransientIo；定义versioned typed disposition ABI并保留unknown |
| PREF-P1-48 | Open | lane绑定process-global TaskPools/TaskTimer，无runtime/session/module generation与shutdown domain |
| PREF-P1-49 | Open | active backend work不可合作取消或执行期超时；不可取消provider必须运行在可遗弃隔离域 |
| PREF-P1-50 | Open | shutdown后不可reactivate；定义activate/quiesce/flush/drain/stop/reactivate状态机和receipt |

### 6.5 App、Editor、WOC与资格闭环

| ID | 状态 | 当前源码证据与后续方向 |
|---|---|---|
| PREF-P1-51 | Open | manual backend install不是descriptor/composition truth，deactivate/reactivate无法按policy重建 |
| PREF-P1-52 | Open | Unavailable仍构建adapter/lane并取得process I/O scheduler；capability admission应避免无用worker资源 |
| PREF-P1-53 | Open | WOC settings/keybind/gamepad仍是无schema version裸JSON，decode忽略unknown且encode只写已知字段 |
| PREF-P1-54 | Open | keybind/offline scope仍由字符串和display name拼接；改稳定account/profile/character principal ID |
| PREF-P1-55 | Open | `submit_preference_text`返回`Option`，key/admission/backend错误被静默吞掉 |
| PREF-P1-56 | Open | Inventory只发字符串effect且无生产consumer/terminal disposition反馈 |
| PREF-P1-57 | Partial | Editor已有typed schema/layer/snapshot、versioned atomic write与真实shutdown fence；仍是第二authority、同步read、整文档重写，User save未接通，lane key还与真实文档冲突域不一致 |
| PREF-P1-58 | Open | 135个专用测试以unit/manual poll/同进程为主；无fresh process、crash、multi-process、permanent hang及App/WOC/Editor端到端receipt |

## 7. P2当前源码重判

| ID | 状态 | 当前源码证据与后续方向 |
|---|---|---|
| PREF-P2-01 | Open | Preference contract/manager没有显式协议版本与capability negotiation |
| PREF-P2-02 | Open | 64MiB/128MiB/4096/250ms等magic limits尚未收敛到profile/platform policy asset及hash |
| PREF-P2-03 | Open | backend name仍是static label，不是stable type ID + instance ID + display label |
| PREF-P2-04 | Open | hashed store没有受权inspect/repair/quarantine工具 |
| PREF-P2-05 | Open | Editor文件名`settings.toml`，内容却是JSON versioned envelope；迁移时应统一扩展名/content type |
| PREF-P2-06 | Open | consumer和test仍靠poll/yield；改event/wake并测idle CPU/wake latency |
| PREF-P2-07 | Open | diagnostics计数饱和后继续服务且不报告exhaustion |
| PREF-P2-08 | Open | mutex poison多处`into_inner`继续，缺panic后invariant audit |
| PREF-P2-09 | Partial | path cache已有共享clone优化和ignored release benchmark；仍缺百万key、allocator和cache-locality产品证据 |
| PREF-P2-10 | Open | 有副作用的`snapshot`仍未改名，pure cache peek仍不存在 |
| PREF-P2-11 | Open | flush/fence只返回一个代表failure，没有bounded分页全量receipt |
| PREF-P2-12 | Open | 大blob被拒后没有指出resource/SaveGame正确owner |
| PREF-P2-13 | Open | 临时目录测试会cleanup，缺failure artifact和recovery manifest保留策略 |
| PREF-P2-14 | Open | 无steady/cold/flush/shutdown长期p50/p95/p99/max、RSS与写放大基线 |

## 8. 五引擎参考结论

### 8.1 Unreal Engine

`FConfigBranch`明确保存static/dynamic/saved/command-line层、runtime changes、hierarchy和async-load state；`FConfigContext`携platform、generated dir、remote policy与override layers；`FConfigCacheIni::Flush`会等待in-flight load并按branch保存。`UGameUserSettings`又把Validate、Apply、Confirm、Save、Cloud pre/post hook和version reset组织成产品workflow。Zircon应借鉴其context、branch/layer provenance、dirty/change ownership和产品apply transaction，而不是复制INI格式。Unreal的部分写路径仍同步、temp-move也有条件，因此它不是Zircon durability/性能门的自动答案。

### 8.2 Godot

`ProjectSettings`是thread-safe singleton，保存type/value/initial/persist/basic/internal/restart metadata，以version和changed set驱动cached read与deferred notification，并支持feature/editor override和config migration。`EditorSettings`独立管理editor-domain配置、compat rename、default/revert、shortcut和project metadata。它证明注册metadata、domain分离、change notification与兼容迁移必须是一等合同；同时其同步整文件save也只是参考下限，不满足Zircon的异步durability目标。

### 8.3 Fyrox

Fyrox `SettingsData`按typed group组织并用reflection驱动Inspector，`DerefMut`触发Changed订阅，`need_save`控制RON覆盖写。这适合借鉴typed group和editor reflection；但固定当前目录`settings.ron`、dirty bool、同步`File::create`、无原子/版本/CAS/多scope，使它成为Zircon必须超过的简单实现基线，而不是目标上限。

### 8.4 Bevy

Bevy的`WgpuSettings`和`WinitSettings`是启动前注入的typed configuration/resource：平台默认、environment override、能力约束和manual/automatic resource creation边界清晰，运行时读取的是实际adapter capability。Bevy这里不提供通用持久化settings service；可借鉴的是composition-time强类型注入与“requested settings”和“effective capabilities”分离，不能把它误写成durable preference参考。

### 8.5 Unity Graphics

Unity Graphics把pipeline-wide settings建模成typed `ScriptableObject` asset，注册到pipeline type；`TryEnsure`按default path、project search和create顺序恢复唯一资产，创建后Initialize/SetDirty/Save。Runtime container区分Editor完整列表与build-time stripped runtime列表；HDRP以显式Version和MigrationStep迁移旧字段到typed graphics settings。Zircon应借鉴typed owner identity、ensure/registration、build projection和显式迁移，不应把project asset模型直接代替用户/account preference。

## 9. 目标架构与Owner边界

```text
App composition
  -> ProductStorageIdentity + PlatformProviderDescriptor + policy profile
  -> Runtime PreferenceService generation

Schema/namespace catalog
  -> PreferenceAddress(product, project, principal, scope, namespace, key)
  -> schema/codec/version/validator/sensitivity/apply policy

PreferenceService
  -> immutable effective snapshot + typed ValueState + change stream
  -> bounded fair scheduler + per-key ordering + cross-key isolation
  -> backend capability adapter
  -> revisioned record/CAS/manifest/watch/recovery
  -> mutation/fence/shutdown receipts

Editor
  -> schema registration + settings UI + dirty/apply/revert projection
  -X-> independent filesystem writer

WOC/product
  -> stable principal scope + reactive snapshot subscription
  -> application transaction + terminal harvest/user feedback
  -X-> raw JSON/default-on-Pending/Option error swallowing
```

| Owner | 唯一职责 |
|---|---|
| App | product identity、platform provider policy、process lifecycle和最终shutdown receipt |
| Runtime PreferenceService | address/schema/runtime state、admission、cache、revision、durability、watch和diagnostics |
| Platform provider | 依据capability执行I/O/CAS/flush/watch，返回真实级别与平台错误 |
| Editor | definition/UI/search/revert/live-apply，不拥有第二durable writer |
| Product/WOC | 注册schema、选择principal/scope、消费typed state/receipt，不解释backend细节 |
| SaveGame/Resource/Secret owner | 接管大blob、权威存档与敏感数据，避免Preference成为万能存储 |

## 10. Hard Cutover约束

1. 不保留`PreferenceKey(namespace,key)`到新地址的永久隐式转换；迁移窗口只能由显式manifest和一次性migration管理。
2. 删除`submit_preference_text -> Option`、副作用`snapshot`和“primitive success即Durable”的旧语义，不做双写兼容层。
3. Editor持久化切换后删除独立`SettingsStore` writer；UI/schema层可以保留，filesystem authority不能并存。
4. WOC旧JSON/key一次性迁移后停止读写旧document，不以fallback永久掩盖migration failure。
5. backend replacement、reactivation和provider reconnect必须改变generation；旧completion不得进入新overlay。
6. cross-key并发启用前先按真实physical transaction key建队列；整文档writer不能用logical setting key隔离。
7. 所有Drop/teardown严格有界；不可取消I/O必须隔离到可放弃进程/worker domain，不能靠后台永远存活规避。
8. 任何`Closed/Pass`都要求production caller、typed receipt和相应动态gate，不能由API存在、mock或测试名关闭。

## 11. TDD实施里程碑

### M0：合同、错误与资格真值

先写compile-fail/model RED tests，冻结`PreferenceAddress`、scope/principal、schema envelope、revision/CAS、durability level、capability、typed failure和receipt。把5个P0转成旧API无法表达的编译期/模型阻断。

### M1：Product identity、namespace lease与schema registry

App在module activation前提供product/project/channel/profile/account identity；catalog分配namespace lease，注册schema/validator/sensitivity/layer/apply policy。完成越权、Unicode/canonical、scope与secret gates。

### M2：Reactive read与revisioned overlay

拆分pure peek和显式load/subscribe，发布typed ValueState、wake、source/revision/freshness；实现byte-bound LRU/pin/TTL/tombstone compaction及backend/schema generation invalidation。

### M3：Fair scheduler与hung-I/O隔离

按physical transaction key保证顺序，支持bounded cross-key concurrency、owner quota、weighted fairness和priority aging；加入active cancel/deadline、checked IDs、多subscriber及可遗弃provider边界。

### M4：Record、CAS、manifest、recovery与watch

在Runtime25 primitives上实现versioned record/digest/revision/writer、manifest、expected revision、lock/broker、backup/staging recovery、quarantine和external change。用双进程与kill-point矩阵关闭P0-005。

### M5：Durability fence与lifecycle transaction

provider声明Buffered/Written/DataSynced/MetadataSynced，mutation/fence返回per-operation receipt；module按activate/quiesce/final flush/drain/stop/reactivate运行，Drop有界，关闭P0-001/P0-003。

### M6：Editor hard cutover

把Editor definition/snapshot/UI/live-apply迁到Runtime catalog之上，迁移User/Project document，删除第二writer和同步load；补齐User设置生产save、project switch conflict与退出receipt。

### M7：WOC/App产品闭环

WOC host真实构造typed settings/keybind/gamepad/inventory owner，使用stable principal、reactive load和terminal harvest；把apply、rollback/retry与UI错误反馈接入真实窗口/输入/renderer/audio。

### M8：平台、故障、性能与删除旧路径

运行native/mobile/Web/headless provider conformance、fresh-process、multi-process、crash/hang、security、soak和同语义benchmark；最后删除旧key/API/document与compat旁路。

## 12. 资格门当前裁决

| Gate | 状态 | 当前证据 / 必须证明 |
|---|---|---|
| PREF-G01 | Fail | 无product/project/channel/profile/account五维identity隔离 |
| PREF-G02 | Fail | 无namespace lease和越权拒绝 |
| PREF-G03 | Fail | 无跨平台Unicode canonical address corpus |
| PREF-G04 | Fail | Editor只有v1且拒绝旧版，WOC无版本；无N-2/N/future negotiation |
| PREF-G05 | Fail | WOC丢unknown，Editor deny unknown；无preserve/reject policy roundtrip |
| PREF-G06 | Partial | Editor schema与WOC clamp能拒绝/规范部分非法值；Runtime cache/store无统一validator |
| PREF-G07 | Partial | Editor有layer/source snapshot和live locale apply；全局dirty/revert/provenance/persistence未闭合 |
| PREF-G08 | Fail | 无secret/PII redaction/security corpus |
| PREF-G09 | Fail | cold read无ticket/wake，consumer依赖poll |
| PREF-G10 | Fail | WOC仍把Loading/Failed/Missing折为default |
| PREF-G11 | Fail | 默认预算没有声明的并行max-size cold-read资格 |
| PREF-G12 | Fail | durable/tombstone 4097+ churn会封死overlay |
| PREF-G13 | Fail | backend replacement无generation transaction |
| PREF-G14 | Fail | 无bounded retry/backoff分类 |
| PREF-G15 | Fail | unrelated key不能并行，一个slow key冻结全lane |
| PREF-G16 | Fail | 无tenant fairness/noisy-neighbor测试 |
| PREF-G17 | Partial | 有100k admission count与ignored局部benchmark；无diverse-key/fence复杂度和latency门 |
| PREF-G18 | Partial | overlay generation exhaustion checked；ticket/epoch/fence计数仍会饱和 |
| PREF-G19 | Fail | active operation deadline/cancel无效 |
| PREF-G20 | Fail | permanent hung backend可阻止最终Drop/退出 |
| PREF-G21 | Fail | backend capability只有enum kind，无法conformance |
| PREF-G22 | Fail | HostProvided primitive success仍越级报Durable |
| PREF-G23 | Partial | fence ordering和单failure kind有unit test；不聚合全部有界failure receipt |
| PREF-G24 | Partial | Editor真实退出先fence再shutdown；Platform cleanup不flush且Drop仍无界 |
| PREF-G25 | Fail | deactivate/reactivate不保留policy并更新generation |
| PREF-G26 | Fail | atomic record无完整envelope/digest |
| PREF-G27 | Fail | 无kill-point old/new完整值矩阵 |
| PREF-G28 | Fail | 无staging/backup startup sweep receipt |
| PREF-G29 | Fail | 无双进程same-revision Conflict |
| PREF-G30 | Fail | 无external update invalidation/subscriber |
| PREF-G31 | Partial | Denied/ReadOnly/Full/TooLarge/Quota有映射测试；Busy等完整fault matrix缺失 |
| PREF-G32 | Fail | 无root canonical/symlink/ACL/sandbox平台安全矩阵 |
| PREF-G33 | Fail | report不能区分planned/probed/healthy/degraded/closed |
| PREF-G34 | Partial | 有基础queue/overlay/path/backend-wall metrics；缺generation/latency percentile/last error/redaction schema |
| PREF-G35 | Fail | Editor仍拥有第二durable writer |
| PREF-G36 | Fail | WOC production refresh/subscription/harvest caller为0 |
| PREF-G37 | Fail | offline display name仍参与scope key |
| PREF-G38 | Fail | 无App/WOC/Editor fresh-process restart矩阵 |
| PREF-G39 | Fail | 无p50/p95/p99/max、RSS与I/O写放大profile receipt |
| PREF-G40 | Fail | 旧API/key/document仍是生产contract，未hard cut |

## 13. 测试与证据矩阵

| 现有证据 | 能证明 | 不能证明 |
|---|---|---|
| Bounded lane 24个专用test | admission、coalescing、fence pin/order、pre-start deadline、panic、可查询shutdown、多waiter等局部语义 | cross-key并行、active cancel、permanent hang退出、tenant fairness、完整complexity/RSS |
| Platform persistence 17个专用test | max+1 read、oversize、read-your-write、generation、防陈旧read、单failure映射、stall离caller wall | 真正buffered durability、flush覆盖全集、record/CAS/recovery/watch、跨进程 |
| Platform集成10个test + 外部authority 2个test | key验证、Unavailable、one-shot install、atomic同进程reload、manager/SPI authority | fresh process、platform root/security、reactivation、provider health、process crash |
| Editor settings 29个test | schema/scope/layer、versioned roundtrip、atomic layer replacement、typed ticket/retry/fence、bounded change log | Runtime统一authority、User设置产品save、并行文档事务、多进程/conflict/crash |
| WOC storage 48个相关test | cold read手工refresh、JSON兼容、range clamp、keybind repair、unavailable default | 生产构造、wake、harvest、host effect消费、restart、failure UX |
| ignored path/coalescing benchmark | 局部优化方向 | CI性能门、长期tail latency、RSS、同语义跨引擎优势 |

必须新增的最小动态矩阵：buffered backend、permanent hang subprocess、4097+ churn、diverse-key 100k storm、timer unavailable、backend replace reverse completion、two-process CAS race、kill-point recovery、external watch、App/WOC/Editor fresh restart、root security、mobile/Web/headless provider以及release profile latency/RSS/write amplification。

## 14. 逐文件检查台账

### 14.1 Framework与lane

| 文件/目录 | 当前事实 | 裁决 |
|---|---|---|
| `preferences/backend_kind.rs` | 三值enum，`is_persistent`只排除Unavailable | 不足以代表capability/health |
| `preferences/error.rs` | 五种kind、operation、static backend label、bounded detail | taxonomy/correlation/source chain不足 |
| `preferences/key.rs` | namespace/key空值、NUL与128/512 bytes限制 | 无canonical address/scope/owner |
| `preferences/storage.rs` | snapshot/mutation/flush/evict contract和三态durability | hidden read、无revision/wake、Durable语义过强 |
| `bounded_keyed_io/admission.rs`、`ticket.rs` | admission/cancel authority、多waiter、pre-start deadline | ID/epoch饱和，active不可取消 |
| `bounded_keyed_io/lane.rs` | 全局queue+single active+pump+single observer | hung全局冻结、公平/并行/observer不足 |
| `lane/coalescing.rs` | same-key partition已线性化并有ignored release benchmark | admission/ordered insertion仍扫描 |
| `lane/fence_prerequisites.rs`、`fence.rs` | prerequisite先计费、连续fence记录线性受限 | failure只取代表项，顺序含HashMap来源 |
| `lane/shutdown.rs` | queued取消、pinned/active drain、bounded `wait_until` | guard `Drop`仍无界wait |
| 三个lane test文件 | 24个test，无ignored | 强unit基础，不是产品/跨进程资格 |

### 14.2 Platform backend、overlay与integration

| 文件/目录 | 当前事实 | 裁决 |
|---|---|---|
| `platform/preferences/backend.rs` | worker authority及read/write/remove/flush SPI | 无context/capability/CAS/watch/transaction |
| `atomic_file.rs` | BLAKE3路径、4096 FIFO cache、atomic staging、file/parent sync、部分OS error mapping | 裸payload、无manifest/recovery/security/revision；Windows parent sync为空 |
| `persistence/overlay.rs` | checked generation、reservation、防陈旧read | durable/tombstone不可evict，failure选择无序，saturating accounting |
| `persistence/adapter.rs` | read/write/remove/fence接lane并维护visible state | global submission mutex、snapshot隐式read、backend swap无代际、flush只投影一个已知failure |
| `persistence/work.rs` | max+1 read和panic-safe worker函数 | primitive success被映射Durable，执行期deadline/cancel不进入backend |
| `unavailable.rs` | 明确Unavailable error | 仍会构建完整adapter/lane资源 |
| `service_types/{driver,manager}.rs` | driver安装adapter，manager委托稳定service handle | one-shot replacement、process-global scheduler、无policy generation |
| `platform/module.rs` | cleanup最多等待250ms并保留超时service | 不先flush，最终Drop仍可能永久挂 |
| `platform/capability/*` | 根据backend kind投影Supported | 无probe/health/effective capability |
| `zircon_app/entry/platform_preferences.rs` | desktop默认root和manual backend factory | root固定品牌且无product/principal隔离，mobile/Web/headless无标准provider |
| Platform/App tests | 同进程atomic reload、typed SPI和bounded cleanup | 无fresh process/crash/multi-process/provider矩阵 |

### 14.3 Editor settings owner

| 文件/目录 | 当前事实 | 裁决 |
|---|---|---|
| `definition.rs`、`scope.rs`、`registry.rs` | typed key/value/schema、User/Project/Session precedence、revision/change | 高价值底座，但taxonomy和Runtime统一owner不足 |
| `snapshot.rs`、`change_log.rs`、`authority.rs` | ArcSwap immutable snapshot、bounded delta、单subscriber、project source cache | generation仍饱和；subscriber单slot；project load同步 |
| `io.rs`、`startup.rs` | versioned JSON envelope、atomic whole-document write、startup provenance | 文件扩展名错误、拒绝旧版、同步I/O、无CAS/recovery |
| `persistence.rs` | shared lane typed ticket/retry/fence/shutdown | logical key与whole-document physical conflict域不一致 |
| context/viewport/workbench/retained host | project snap真实submit，退出真实fence；locale hot-apply | generic User save未接线，仍是第二filesystem authority |
| 三个settings test文件 | 29个test覆盖schema/layer/persistence/change log | 不证明统一Runtime owner和产品User持久化 |

### 14.4 WOC consumer

| 文件/目录 | 当前事实 | 裁决 |
|---|---|---|
| `preferences/storage.rs` | Pending/Ready包装及text read/write helper | 无terminal/wake；所有错误折为default或Option |
| `preferences/settings/{registry,state,application}.rs` | 84项定义、range clamp和typed apply route | 未注册Runtime schema，部分route为空，未接真实host |
| `preferences/settings/storage.rs` | 整份JSON load/save与last submission slot | 无版本/unknown preserve；无生产构造/refresh/harvest |
| `input/gamepad/storage.rs` | JSON object兼容和完整重写 | 无生产owner/terminal |
| `input/keybind/{combo,profile,storage}.rs` | scope字符串、legacy fallback、repair与完整profile重写 | display/string identity、无显式migration/revision |
| `shell/offline_session.rs` | `offline:{class}:{player_name}`生成scope | rename改变identity，不是稳定principal |
| `windows/{inventory,routes}.rs` | model可编码，route生成Persist effect | effect无生产consumer，旧storage helper也无产品owner |
| `application.rs`、`main.rs` | host-neutral session和identity-report main | 没有Preference composition或UI/apply/persistence闭环 |
| 五个WOC相关test文件 | 48个test可手工等待cold read并验证JSON | 正好反证生产缺少同样的refresh/harvest owner |

## 15. 父Owner与实施回写

| 主题 | 主owner | Runtime134边界 |
|---|---|---|
| Runtime config/schema/diagnostics | Runtime03 | 提供Preference address/schema/health/metrics需求，不重复建立总诊断系统 |
| filesystem/atomic/recovery/security | Runtime25 | 复用durable transaction、安全path和recovery primitives |
| SaveGame/large payload | Runtime40 | Preference拒绝大blob并路由，不吞并SaveGame语义 |
| Editor settings UI/i18n/plugin extension | Editor12 | Editor保留UI/schema/apply，durability hard cut到Runtime134 |
| App product/platform lifecycle | App01 | App提供product identity/provider和最终shutdown owner |
| module/provider composition | Runtime42/Runtime01 | namespace/provider policy来自统一composition，不建立第五套catalog |
| bounded jobs/timers | Runtime02/Runtime11 | 复用task owner，同时修复single-active、cancel/deadline和Drop资格 |

三份open failure只有在各自主owner拿到对应动态receipt后才能关闭；Runtime134不直接改写、重命名或关闭这些failure。

## 16. 审查状态与输出记录

- 审查类型：current-source review + refactor plan，review-only。
- 实现状态：pending；本轮未改Rust/Cargo，未创建compat shim。
- 静态结论：5 P0 Open；58 P1 = 47 Open / 11 Partial；14 P2 = 13 Open / 1 Partial；40 Gates = 32 Fail / 8 Partial / 0 Pass。
- 动态证据：本轮未运行Cargo、真实Editor/WOC、fresh process、crash、multi-process、hang、security、soak或benchmark；后续不得把本报告的静态确认写成动态通过。
- currentness：收尾重扫确认`verification_head=8dc299a8b65813f692e222a709f951e6ace90be6`、`verification_epoch=393`，四组文件数量、物理统计与fingerprint均未漂移。
