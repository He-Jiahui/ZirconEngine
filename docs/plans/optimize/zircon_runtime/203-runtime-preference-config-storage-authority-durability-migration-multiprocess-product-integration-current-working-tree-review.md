# Runtime203 - Preference / Config Storage Authority / Durability / Migration / Multi-process / Product Integration 当前工作树复审

> 审查日期：2026-08-31
> 工作树基线：`5ffc4945095a6fc734bcbb2e632958026350b760`；`git status --porcelain=v1 -uno` 有 8,394 项，本文不回退、不覆盖共享工作树中的其他改动
> 审查性质：current-working-tree source review + refactor plan；只写文档，不修改生产 Rust/Cargo/ABI/test/UI 代码
> 排除范围：Tooling；未查询、轮询、等待或实时跟踪协调器
> 前置文档：[Runtime45](45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md)、[Runtime99zi](99zi-runtime-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-current-source-review.md)、[Editor265](../zircon_editor/265-editor-settings-preferences-project-settings-scope-schema-overlay-persistence-migration-restart-plugin-window-current-working-tree-review.md)

## 1. 结论

当前实现已经不是“只有 HashMap + 临时文件”的原型：Runtime 有中立 `PreferenceStorage`、worker-only backend authority、read-your-write overlay、entry/byte 双预算、same-key coalescing、fence prerequisite、pre-start deadline、panic terminal、atomic staging/commit、one-shot App bootstrap 注入和可查询的 bounded module cleanup；Foundation `DefaultConfigManager` 也有 debounce worker、atomic write、单进程 path commit fence、单备份恢复、显式 flush 与持久化统计。这些底座应保留。

但系统仍不是工程级统一 Settings/Preference/Config 平台，核心问题不是“少几个 API”，而是**持久化权威分裂且语义互相矛盾**：

1. `PreferenceStorage` 把 HostProvided backend 的一次 `write/remove` 成功直接报告为 `Durable`，独立 `flush()` 没有参与该 mutation 的 durability receipt；Platform cleanup 也只 drain，不先建立 final flush fence。
2. bounded keyed I/O 仍只有一个全局 active 工作槽，active work 没有 cooperative cancel/deadline。单个永久悬挂 provider 可以冻结所有 key，并在最终 shutdown guard `Drop` 中无界等待。
3. 默认 Preference root 和 Foundation config root 都只含 `ZirconEngine` 品牌名，不含 product/project/channel/profile/account/principal identity。多个 Zircon 产品、Editor 和 Client 会共享物理域。
4. Atomic Preference 文件只有 hash 路径和裸 payload，没有 address/schema/version/revision/digest/writer/manifest/CAS/watch/recovery；跨进程写入只能 last-writer-wins。
5. `ConfigStore + DefaultConfigManager` 是另一套生产持久化权威：它把 transient runtime launch config、Editor layout 和任意 JSON key 混在一个 `config.json` 中，每次重写整份 map；commit fence 只在当前进程生效。
6. App bootstrap 先发布 resolved entry config，Foundation 激活时又从旧 `config.json` 回灌并覆盖同一 map，所有模块激活完成后才再次写回 resolved config。最终 snapshot 看似正确，但激活窗口内的 authority precedence 没有证明。
7. Editor SettingsStore 是第三套文件系统权威，Editor layout 又走 Foundation ConfigManager；WOC 的 Stored Settings/Keybind/Gamepad 包装器仍只有测试构造，生产 route effect 没有 persistence terminal owner。

因此本轮保持 `PREF-P0-001..005` 全部 Open，新增 `CONFIG-P0-001` Open 与 `CONFIG-P0-002` Partial。Preference P1 为 **45 Open / 13 Partial / 0 Closed**；新增 Config P1 为 **12 Open / 4 Partial / 0 Closed**。Preference P2 为 **13 Open / 1 Partial**，新增 Config P2 3 项 Open。Preference 40 门为 **31 Fail / 9 Partial / 0 Pass**，新增 Config 12 门为 **10 Fail / 2 Partial / 0 Pass**。

## 2. 审查边界与证据冻结

统计口径：UTF-8 物理行、非空行、bytes；测试属性匹配 `#[test]`，ignored marker 匹配 `#[ignore`。fingerprint 为 normalized lowercase relative path 排序后，对 `path + NUL + lowercase(file SHA-256) + LF` 集合再做 SHA-256。

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Runtime Preference contract/lane/backend/platform/App | **46** | **10,310** | **9,425** | **360,576** | **90** | **10** | `bceb0d20fd514cd578e7c496f56892db687fea00a6697b1272427444b7b31194` |
| ConfigStore/ConfigManager/Foundation/Editor layout consumer | **22** | **3,458** | **3,090** | **113,806** | **35** | **3** | `adc92d52ceb7f821c86e5f553369cac178690e976947319734ba369b5d55d49d` |
| Editor Settings 与 Runtime/Config 边界 | **40** | **8,095** | **7,350** | **278,252** | **68** | **1** | `2631a8af7ca97ae75ec09e1a935fb5c92d8698036d2660e5796744a9aa7195f7` |
| WOC Preference consumer 与专用测试 | **33** | **6,863** | **6,314** | **212,106** | **84** | **0** | `df56981a241f1bc7627fa4703670adc6cdd14a7975a64592850748ada637944d` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics 参考集 | **20** | **21,544** | **18,385** | **811,232** | **0** | **0** | `00e972855ce8891ef79af4e4c94e6410cf4bfbcca4dddc53edd71845cc116f03` |

参考 revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine` 无独立 Git 元数据，只以所列物理文件和集合 fingerprint 冻结。

本轮逐层检查了：

1. Framework Preference key/error/backend kind/storage/snapshot/ticket/terminal/deadline/eviction。
2. `BoundedKeyedIoLane` admission、typed key、queue、coalescing、fence prerequisite、observer、deadline、shutdown 与关键测试。
3. Platform backend SPI、Unavailable、AtomicFile、overlay、adapter、work、driver/manager/module cleanup、capability report 与 App bootstrap root/provider 选择。
4. Core `ConfigStore`、Foundation `ConfigManager` contract/error/report、config path、worker/state/writer/path commit fence/recovery/tests。
5. Editor SettingsStore、Preferences UI、project lifecycle 与 Foundation layout persistence 的双重消费边界。
6. WOC Stored Settings/Gamepad/Keybind/Inventory effect 的生产构造、refresh、submission harvest 和 terminal consumer。
7. 五个参考引擎的 config/settings owner、typed metadata、layer/context、migration、apply/save 与 build/runtime projection。

## 3. 当前真实拓扑

```text
ResolvedProductHostConfig
  -> Core ConfigStore (raw in-memory JSON map)
  -> Foundation DefaultConfigManager
       -> global ZirconEngine/config.json
       -> whole-map debounce rewrite
       -> process-local path commit fence
       -> Editor workbench layout consumer

App platform bootstrap
  -> PlatformDriver factory captures one backend
  -> PreferencePersistenceAdapter
       -> overlay + one global BoundedKeyedIoLane
       -> AtomicFile or HostProvided backend
       -> global ZirconEngine/preferences root
  -> PlatformManager as dyn PreferenceStorage

Editor Settings
  -> independent SettingsAuthority + SettingsPersistence
  -> independent user/project settings documents
  -> Foundation ConfigManager for layouts
  -X-> Runtime PreferenceStorage

WOC
  -> StoredClientSettings / StoredGamepadBindings / StoredKeybinds
  -> tests construct them
  -X-> production App composition / refresh / wake / terminal harvest
```

这不是三层架构，而是三套 durable writer 加一套未接产品 consumer。`ConfigStore`、Preference overlay 和 Editor SettingsAuthority 各自维护 generation、dirty/terminal、I/O worker、文件模型与 shutdown 语义，无法给用户或上层模块提供一个可证明的 effective value、revision、durability 和 failure receipt。

## 4. 必须保留的基础

1. 保留 `PreferenceStorage` 的 Runtime-neutral service boundary 和 backend work authority，但扩展成 address/schema/capability/revision/receipt 合同。
2. 保留 overlay 的 read-your-write、checked generation 和“stale cold read 不覆盖较新 visible mutation”规则。
3. 保留 entry/byte admission、typed domain key、same-key ordering/coalescing、fence prerequisite、panic containment 和 queryable shutdown report。
4. 保留 App 在 descriptor factory 中注入 backend 的生产路径；它已优于旧版激活后临时 replacement。
5. 保留 AtomicFile 的 hashed physical location、共享 `stage_atomic_write`/commit、file/parent sync 和 max+1 bounded read。
6. 保留 ConfigManager 的 change coalescing、off-caller serialization/write、bounded flush wait、single-backup recovery 和持久化统计，但它不能继续持有任意 runtime config。
7. 保留 Editor typed schema、scope/layer precedence、immutable snapshot、document-key lane、health/retry 和真实 close fence；durable writer 必须 hard cut 到统一 Runtime owner。
8. 保留 WOC 的 typed range/apply route、keybind repair 和 compatibility decode，将它们迁移成 versioned schema consumer。

## 5. P0 当前裁决

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PREF-P0-001 | Open | `write/remove` 的 backend primitive 成功经 lane 直接映射 `PreferenceMutationTerminal::Durable`；HostProvided 无 capability 可声明 buffered/remote durability，`flush_fence` 也不会升级该 operation | provider capability + mutation durability level + operation-set flush receipt；只有 receipt 覆盖的 generation 才能成为 Durable |
| PREF-P0-002 | Open | WOC 三个 `Stored*::new` 生产调用为 0；Inventory 只生成字符串 effect；refresh/harvest 仅存在于 helper/test | Product settings owner 在启动、运行和退出阶段消费 typed state、wake、mutation/fence terminal，并向 UI 暴露 retry/revert/failure |
| PREF-P0-003 | Open | `LaneState.active: Option<_>`，pump 同步执行一个 work；deadline 只调用 `expire_before_start`；shutdown guard `Drop::wait()` 无期限 | per-provider/partition bounded concurrency、same-key serial order、active cancel/deadline、不可取消 provider 的隔离进程和严格 bounded Drop |
| PREF-P0-004 | Open | root 固定 `ZirconEngine/preferences`；`PreferenceKey` 只有 namespace/key 字符串，任意 consumer 可自分配；WOC scope 可含 display name | `ProductStorageIdentity + ProjectIdentity + Principal + TypedScope + NamespaceLease`，逻辑/物理隔离 product/channel/profile/account/world/plugin |
| PREF-P0-005 | Open | `.zrpref` 是 hash 路径下裸 payload；无 record revision、expected revision、lock/CAS/watch/manifest；Editor/WOC 又整文档写 | versioned record + digest + revision/CAS + conflict receipt + broker/lock/watch/recovery，覆盖双进程、kill/restart 与 external update |
| CONFIG-P0-001 | Open | 所有 Runtime 启动 config 与 Editor layout 共用品牌级 `config.json`；每个进程持有全量 snapshot 并 whole-file replace，path fence 只在进程内 | transient launch config 与 persisted preference 分域；持久层按 product/principal/schema 管理，跨进程必须 CAS/lock/merge，不允许全量 last-writer-wins |
| CONFIG-P0-002 | Partial | bootstrap 先 `store_entry_config`，Foundation 激活时 `recover_and_load_from_disk` 可覆盖同 key，全部模块激活结束后才再次 store；最终值恢复但 activation window precedence 未冻结 | immutable boot plan 在 module graph freeze 前确定；persisted user override 必须经显式 layer merge，任何 module activation 只读同一 pinned effective snapshot |

## 6. Preference P1 当前重判

### 6.1 Address、schema、layer 与权限

| ID | 状态 | 当前证据与重构方向 |
|---|---|---|
| PREF-P1-01 | Open | 仍只有 namespace/key 字符串；建立 product/project/principal/scope/namespace/logical-key 完整地址 |
| PREF-P1-02 | Partial | Editor 有 User/Project/Session；Runtime/WOC 无 Account/Profile/World/Plugin taxonomy 与继承规则 |
| PREF-P1-03 | Open | namespace 无 catalog owner/lease；composition 分配 owner identity 并在每次访问鉴权 |
| PREF-P1-04 | Open | 只拒绝空/NUL并按 bytes 限长；冻结 Unicode、case、separator、confusable canonicalization |
| PREF-P1-05 | Partial | Editor 有 `SettingValue/SettingSchema`，Runtime payload 仍是 bytes；统一 codec/content type/version envelope |
| PREF-P1-06 | Partial | Editor 有四层 precedence，Runtime Preference 无 provenance/effective source |
| PREF-P1-07 | Partial | Editor 有 type/range/restart，WOC 会 clamp；Runtime 缺 registry/dependency/apply/revert/dirty comparator |
| PREF-P1-08 | Partial | Editor 有 v1 envelope 但拒绝 v0/unknown；WOC JSON 无版本且不保留 future field；定义 N-2/N/future policy |
| PREF-P1-09 | Open | 无 secret/PII 分类、redaction、encryption 或 OS secret-store 路由 |
| PREF-P1-10 | Open | 无 multi-key prepare/commit/abort，也没有明确原子性拒绝 receipt |
| PREF-P1-11 | Open | read 无 revision/etag，write/remove 无 expected revision/Conflict |
| PREF-P1-12 | Open | error 仅五类且 backend 为 static label；补 Conflict/Schema/Migration/Auth/Cancelled/Deadline/Unsupported/Integrity 与 source chain |

### 6.2 Backend、record、path 与 capability

| ID | 状态 | 当前证据与重构方向 |
|---|---|---|
| PREF-P1-13 | Open | `backend_kind().is_persistent()` 代替 capability；定义 durability/flush/CAS/watch/transaction/limit/thread/cancel contract |
| PREF-P1-14 | Open | 裸 payload 无 envelope；加入完整 address、schema、revision、digest、flags、writer、format version |
| PREF-P1-15 | Open | 无授权 manifest、enumeration、paged reset/export/import，hash 目录不可治理 |
| PREF-P1-16 | Open | 无 header/payload integrity 与 quarantine；bit flip 必须返回 Integrity 并保留证据 |
| PREF-P1-17 | Open | Preference 无 startup staging sweep/last-known-good/recovery receipt；共享 atomic primitive 不等于恢复协议 |
| PREF-P1-18 | Open | 无 root ACL/symlink-safe open/sandbox/security policy |
| PREF-P1-19 | Partial | Editor Project path 有 `ProjectPaths` physical identity；App/User root 仍直接接受环境路径且无 canonical root receipt |
| PREF-P1-20 | Open | mobile/Web/headless 只有“host 必须注入”，没有标准 provider、probe、conformance 与 unavailable reason |
| PREF-P1-21 | Partial | App 生产路径已在 descriptor factory one-shot 注入，避免临时 backend 可见；adapter replacement 本身仍无 freeze/flush/drain/generation/cache invalidation |
| PREF-P1-22 | Open | late install/reconnect 不会自动重试已失败 cold read；只能显式 evict 后重新 snapshot |
| PREF-P1-23 | Open | capability report 只投影 backend kind，不能区分 planned/installed/probed/healthy/degraded/closed |
| PREF-P1-24 | Partial | 有 queue/oldest age/overlay/path/backend wall 计数；仍缺 backend instance/generation、tenant、percentile、terminal/error correlation |
| PREF-P1-25 | Open | path cache 仅 4,096 entry FIFO，hit 不刷新且无 byte budget/generation invalidation |
| PREF-P1-26 | Partial | Permission/ReadOnly/Full/TooLarge/Quota 有映射；Busy/NotFound 等仍折为 TransientIo，source/correlation 不完整 |

### 6.3 Overlay、read、admission 与 cache

| ID | 状态 | 当前证据与重构方向 |
|---|---|---|
| PREF-P1-27 | Open | `snapshot()` 隐式提交 cold read，Pending 没有 load ticket；拆 pure peek 与显式 load/subscribe |
| PREF-P1-28 | Open | WOC 将 missing/loading/failure 折成 None/default；引入 Missing/Loading/Ready/Failed/Stale + revision/source |
| PREF-P1-29 | Open | 64 MiB max value 的 cold-read reservation 使默认 128 MiB overlay/lane 只能容纳约一个最大读；改 header probe/stream 与 schema/tenant limit |
| PREF-P1-30 | Open | 新增 `evict` 只允许 terminal VisibleNotDurable；durable value 与成功 tombstone 仍可用 4,097 个 key 永久封满 overlay |
| PREF-P1-31 | Open | 无 invalidate/reload/watch/poll generation 或 external changed/conflict/stale event |
| PREF-P1-32 | Open | 无 transient/permanent backoff、jitter、retry deadline 与 operator policy |
| PREF-P1-33 | Open | `known_non_durable_failure()` 从 HashMap 任取首项；fence 应按 operation order 分页聚合 |
| PREF-P1-34 | Open | retained bytes 使用常量近似，遗漏 container/control/backend buffer；需 allocator/RSS 校准 |
| PREF-P1-35 | Open | adapter submission 由 global mutex 串行，无 contention/lock wait/tenant fairness 指标 |
| PREF-P1-36 | Open | max value 只有全局值；由 schema/scope/tenant/backend 联合裁决并路由大 blob 到 SaveGame/Resource |
| PREF-P1-37 | Open | cache entry 无 backend generation、record revision、schema generation 与 freshness |
| PREF-P1-38 | Open | timer 不可用映射 CapacityExceeded；增加 SchedulerUnavailable/DeadlineUnsupported 与 degrade policy |
| PREF-P1-39 | Open | `caller_filesystem_wall` 固定为零且测试锁定；应测 queue/backend/fsync/callback wall |

### 6.4 Lane、fence、cancel 与 lifecycle

| ID | 状态 | 当前证据与重构方向 |
|---|---|---|
| PREF-P1-40 | Open | global single active；按 backend capability 提供 bounded cross-key concurrency，same-key 保序 |
| PREF-P1-41 | Open | 全局 FIFO 无 owner quota、priority、aging 与 starvation telemetry |
| PREF-P1-42 | Partial | typed key、线性 queue partition、fast-tail insertion、single-pass successor scan、reused fence set、ordered suspended index 均有源码/ignored benchmark；fence/admission 在 storm 下仍可能累计 O(N²) |
| PREF-P1-43 | Partial | overlay generation 使用 checked increment；public lane 仍信任 caller generation，缺 backend generation 单调验证 |
| PREF-P1-44 | Open | ticket ID/epoch 使用 `saturating_add`，最终可重复/冻结；改 checked composite identity |
| PREF-P1-45 | Open | fence pin、reservation 与多类统计使用 saturating arithmetic，掩盖 invariant 破坏 |
| PREF-P1-46 | Open | terminal observer 仍单槽，后注册可覆盖前者；改 multi-subscriber stream + backpressure |
| PREF-P1-47 | Open | lane failure 只有 static code；定义 versioned typed disposition ABI 并保留 unknown |
| PREF-P1-48 | Open | lane 绑定 TaskPool/TaskTimer，但无 runtime/session/module generation 与 shutdown domain |
| PREF-P1-49 | Open | active backend work 不可取消，执行期 deadline 被测试明确规定为不生效 |
| PREF-P1-50 | Open | shutdown 后不可 reactivate；定义 activate/quiesce/flush/drain/stop/reactivate state machine 与 receipt |

### 6.5 App、Editor、WOC 与产品闭环

| ID | 状态 | 当前证据与重构方向 |
|---|---|---|
| PREF-P1-51 | Partial | App 已把 backend 捕获到 Platform driver factory，正常 reactivation 可重建同 policy；public manual install 仍允许 composition 外 late authority，且无 provider generation receipt |
| PREF-P1-52 | Open | Unavailable 仍创建完整 adapter/lane 并占用 Runtime I/O scheduler 资源 |
| PREF-P1-53 | Open | WOC settings/keybind/gamepad 是无 schema version 的整份 JSON，encode 不保留 unknown |
| PREF-P1-54 | Open | keybind/offline scope 由字符串/display name 拼接；改稳定 account/profile/character principal ID |
| PREF-P1-55 | Open | `submit_preference_text` 返回 Option，key/admission/backend error 被静默吞掉 |
| PREF-P1-56 | Open | Inventory route 只返回字符串 effect，无生产 consumer 或 terminal disposition feedback |
| PREF-P1-57 | Partial | Editor coordinator、document key、source preflight、health/retry 和真实 Preferences 写入口有进展；仍有 project transition 绕过、执行时晚取 layer、无写入成功、第二 filesystem authority |
| PREF-P1-58 | Open | 专用测试多为 unit/manual poll/同进程；无 fresh process、crash、multi-process、permanent hang 与 App/WOC/Editor E2E receipt |

## 7. ConfigStore / ConfigManager 新增 P1

| ID | 状态 | 当前证据与重构方向 |
|---|---|---|
| CONFIG-P1-001 | Open | key 是任意 String，无 owner/catalog/namespace lease；不能区分 engine/module/plugin/product/user/project |
| CONFIG-P1-002 | Open | transient launch config、effective runtime state和durable user layout共用一个 map；建立 BootPlan、RuntimeState、PreferenceDocument 三个明确 domain |
| CONFIG-P1-003 | Open | value 为任意 `serde_json::Value`，无 type/schema/version/validator/migration/default/restart metadata |
| CONFIG-P1-004 | Open | 任一 layout 变化会序列化并原子重写整个 config snapshot，写放大和冲突域随全局 key 数增长 |
| CONFIG-P1-005 | Open | `CoreHandle::store_config*` 直接改 map，不通知 persistence dirty generation；是否持久化依赖之后是否恰有 `ConfigManager::set_value` |
| CONFIG-P1-006 | Open | 无 remove、expected revision、CAS、transaction、subscribe/watch 或 per-key terminal receipt |
| CONFIG-P1-007 | Partial | 启动会从单备份恢复 missing target；malformed JSON、ambiguous backups 会让 Foundation 激活失败，缺 quarantine/repair/default-safe policy |
| CONFIG-P1-008 | Open | commit registry 是进程内 static `HashMap<PathBuf, Weak<_>>`；跨进程没有 lock/CAS/watch/merge |
| CONFIG-P1-009 | Open | path gate epoch 使用 `wrapping_add`，generation exhaustion 后可重新接受旧 epoch |
| CONFIG-P1-010 | Partial | explicit flush 与 Drop wait 有 timeout；超时后 JoinHandle 被丢弃，worker 线程继续运行，只有进程内 commit fence 阻止部分 late commit |
| CONFIG-P1-011 | Open | error 只有 RuntimeUnavailable/Persistence/FlushTimedOut，reason 为 String；无 Corrupt/Conflict/Auth/Cancelled/Recovery/Schema/Migration typed chain |
| CONFIG-P1-012 | Open | path 只由 env/OS dir/品牌名决定，fallback 到 cwd 隐藏文件；无 canonical product root、ACL/symlink/sandbox receipt |
| CONFIG-P1-013 | Partial | worker off-caller、debounce、same-value retry、concurrent updates和 bounded latency ring 有测试；没有 queue age、write amplification、backend wall、process identity |
| CONFIG-P1-014 | Partial | atomic writer与single-backup recovery可保留；没有 record envelope、manifest、digest、schema generation、staging sweep receipt |
| CONFIG-P1-015 | Open | startup whole-map hydrate 没有 layer provenance；persisted value与resolved command line/profile/default的 precedence 只是调用顺序 |
| CONFIG-P1-016 | Open | 生产 typed consumer主要是 Editor layout，而 Editor SettingsStore/Runtime Preference又各自管理相邻数据；必须收敛唯一Settings/Preference authority |

## 8. P2 当前裁决

### 8.1 Preference P2

| ID | 状态 | 当前证据与方向 |
|---|---|---|
| PREF-P2-01 | Open | contract/manager 无显式协议版本与 capability negotiation |
| PREF-P2-02 | Open | 64 MiB/128 MiB/4,096/250 ms 等 limit 未进入 profile/platform policy asset 与 hash |
| PREF-P2-03 | Open | backend name 是 static label，不是 stable type ID + instance ID + display label |
| PREF-P2-04 | Open | hashed store 无受权 inspect/repair/quarantine 工具 |
| PREF-P2-05 | Open | Editor `settings.toml` 实为 JSON envelope，扩展名/content type 不一致 |
| PREF-P2-06 | Open | consumer/test 依赖 poll/yield；缺 wake latency 与 idle CPU 证据 |
| PREF-P2-07 | Open | diagnostics saturate 后继续服务且不报告 exhaustion |
| PREF-P2-08 | Open | 多处 poison `into_inner` 后继续，缺 invariant audit |
| PREF-P2-09 | Partial | path/cache/coalescing 有局部优化与 ignored release benchmark；无百万 key、allocator、cache locality 与产品 profile |
| PREF-P2-10 | Open | 有副作用的 `snapshot` 未改名，pure cache peek 不存在 |
| PREF-P2-11 | Open | flush/fence 只返回单个代表 failure，无 bounded paged receipt |
| PREF-P2-12 | Open | 大 blob 拒绝后不返回正确 SaveGame/Resource owner |
| PREF-P2-13 | Open | failure artifact/recovery manifest 的保留、redaction、清理策略缺失 |
| PREF-P2-14 | Open | 无 steady/cold/flush/shutdown p50/p95/p99/max、RSS 与写放大基线 |

### 8.2 Config P2

| ID | 状态 | 当前证据与方向 |
|---|---|---|
| CONFIG-P2-001 | Open | debounce 25 ms、shutdown 2 s、latency samples 64、品牌目录与 fallback 文件名均是局部常量，不来自产品 policy |
| CONFIG-P2-002 | Open | pretty JSON whole-map 重写没有格式/写放大/大 map 的产品级规模上限与 benchmark gate |
| CONFIG-P2-003 | Open | persistence report 无 backend/path identity redaction policy、schema/version、recovery state 或 operator action |

## 9. 五引擎参考差距

### 9.1 Unreal Engine

`FConfigBranch` 保留 source hierarchy、final combined layers、command-line overrides、in-memory file、runtime change tracker 和 async-load state；`FConfigContext` 明确 platform、generated config directory、remote policy 与 override context。`UGameUserSettings` 将 Validate、Load、Apply、Save、resolution confirm/revert、preload 与 cloud hook 组织成产品 workflow。Zircon 应借鉴 context、branch/layer provenance、dirty owner 和 apply/save transaction，不复制 INI 或其同步写路径。

### 9.2 Godot

`ProjectSettings` 的 value container 同时保存 initial/basic/internal/restart metadata，以 version 与 changed set 支持 cached read/deferred notification，并有 feature/editor override 与保存顺序；`EditorSettings` 是独立 editor-domain owner，管理 property hint、default/revert、shortcut、project metadata 与 compatibility rename。Zircon 当前只有 Editor 局部 schema，Runtime Preference/ConfigManager 均缺同等级 registry/change/migration contract。

### 9.3 Fyrox

Fyrox `SettingsData` 以强类型 group + reflection 驱动 Inspector，`DerefMut` 设置 `need_save` 并通知订阅者；但固定 `settings.ron`、同步 whole-file 写、dirty bool、无版本/CAS/多 scope。它证明 typed group/UI reflection 的价值，也证明 Zircon 不能把自己的 ConfigManager whole-map rewrite 当作目标上限。

### 9.4 Bevy

`WgpuSettings`、`WinitSettings` 是 composition-time typed resources，清楚区分 platform defaults、environment override、requested feature 与实际 adapter capability；Bevy 这里没有通用 durable Preference service。Zircon 应采用其强类型注入和 requested/effective 分离，不能用它为无 revision 的存储语义背书。

### 9.5 Unity Graphics

Unity Graphics 将 pipeline-wide settings 建模为注册到 pipeline type 的唯一 typed asset；`TryEnsure` 执行 find/default path/create/initialize/save，runtime container 又区分 Editor full list 与 build-time stripped list，HDRP 通过 `Version + MigrationStep` 逐版本迁移。Zircon 应借鉴 typed owner、ensure/registration、build projection 和迁移 ledger，但用户/account preference 不能直接套用 project asset 模型。

## 10. 目标 owner 架构

```text
App composition
  -> immutable ProductStorageIdentity
  -> BootPlanSnapshot (never persisted by generic settings writer)
  -> PlatformPreferenceProviderDescriptor + policy profile

Runtime PreferenceAuthority
  -> SchemaCatalog + NamespaceLeaseRegistry
  -> PreferenceAddress(product/project/principal/scope/namespace/key)
  -> immutable EffectiveSettingsSnapshot + provenance
  -> LoadTicket / MutationReceipt / FenceReceipt / ChangeStream
  -> fair bounded scheduler + provider isolation
  -> revisioned record/CAS/manifest/watch/recovery

Editor
  -> registers setting definitions, categories, UI metadata and apply hooks
  -> consumes Runtime snapshots/transactions
  -X-> independent SettingsStore filesystem writer

Product/WOC
  -> stable principal/profile/world identities
  -> subscribes and applies typed settings
  -> harvests receipts and exposes retry/revert/failure
  -X-> default-on-Pending and Option error swallowing

SaveGame / Resource / SecretStore
  -> own large blobs, authoritative saves, project assets and secrets
```

| Owner | 唯一职责 |
|---|---|
| App | product identity、boot plan、provider policy、process lifecycle、final shutdown receipt |
| Runtime PreferenceAuthority | address/schema/layer/cache/revision/admission/durability/watch/diagnostics |
| Platform provider | 按 capability 执行 I/O/CAS/flush/watch，并报告真实 durability 与平台错误 |
| Editor | definition/UI/search/revert/apply/restart projection，不拥有第二 durable writer |
| Product/WOC | 注册 domain schema，选择 principal/scope，消费 typed state/receipt |
| SaveGame/Resource/SecretStore | 接管不属于 Preference 的数据，禁止万能 JSON store |

## 11. Hard cutover 约束

1. 不保留 `PreferenceKey(namespace,key)` 到新地址的永久隐式转换；旧 key 只能由 manifest 驱动的一次性 migration 读取。
2. 删除 `submit_preference_text -> Option`、副作用 `snapshot()` 和“provider primitive success 即 Durable”的旧语义。
3. 将 `ConfigStore` 明确降级为 transient/pinned Runtime configuration；删除其对任意 map 的通用持久化，Editor layout 迁入 typed Preference schema。
4. Editor 切换后删除 SettingsStore 的独立 user/project filesystem writer；UI/schema/apply 层保留。
5. WOC 旧整文档 JSON 在一次性 migration 后停止写入；unknown field、repair 和失败必须有显式 receipt。
6. 不创建双写兼容层，不允许 Runtime Preference、ConfigManager、Editor SettingsStore 同时写同一逻辑 setting。
7. 所有 deprecated API/key/path 删除前必须有 inventory、zero production caller 和 fresh-process migration receipt。

## 12. 分层实施里程碑

### M0：冻结合同与 owner

定义 `ProductStorageIdentity`、`PreferenceAddress`、schema/codec/version、capability、typed failure、load/mutation/fence/shutdown receipt；将 BootPlan 与 durable Preference 分域。先写 compile-fail/contract tests，再改实现。

### M1：统一 schema/layer/effective snapshot

建立 namespace lease、scope taxonomy、default/user/project/session/account/profile/world layer、validator/migration、immutable effective snapshot 和 change stream。Editor definition 与 WOC registry 接入，但仍不切换文件写入。

### M2：调度与 lifecycle

把 single-active lane 改为 same-key serial + cross-key/provider bounded concurrency；加入 owner quota/priority/aging、active cancel/deadline、provider isolation、multi-subscriber terminal 与严格 bounded shutdown。

### M3：record/CAS/recovery/watch

实现 versioned envelope、digest、revision、expected revision、manifest、single-writer broker或process lock、watch/invalidate、staging sweep、quarantine、last-known-good 和 paged recovery receipt。

### M4：durability transaction

mutation terminal 只报告 provider 已证明级别；flush receipt 覆盖明确 operation/generation 集合；App shutdown 执行 quiesce -> final fence -> drain -> stop，并保存完整 failure set。

### M5：ConfigManager 收敛

将 runtime launch/configuration 变为 immutable BootPlanSnapshot；迁移 Editor layout 等真实 durable keys；删除 global whole-map persistence、process-local-only fence 和 `CoreHandle::store_config*` 的隐式持久化歧义。

### M6：Editor hard cutover

保留 Editor schema/UI/health/apply/restart；所有 user/project document 通过 Runtime transaction。修复 project transition、ticket document freeze、suppressed write success、plugin settings page、orphan/migration/recovery。

### M7：WOC/App 产品闭环

App composition 注入 product/principal/profile identity；WOC 在启动订阅、运行 apply/harvest、退出 final fence，Inventory effect 有真实 consumer，UI 能显示 Pending/Failed/Conflict/Retry。

### M8：平台、故障、规模与删除旧路径

运行 native/mobile/Web/headless provider conformance、fresh-process、two-process、kill-point、permanent hang subprocess、security、4097+ churn、100k storm、RSS/write amplification/latency benchmark；最后删除旧 path/key/API/document。

## 13. 资格门

### 13.1 Preference 40 门

| Gates | 当前状态 | 说明 |
|---|---|---|
| G01 product/project/channel/profile/account identity | Fail | 地址与 root 均无这些维度 |
| G02 namespace lease/auth | Fail | 任意 consumer 可自分配 namespace |
| G03 Unicode canonical address corpus | Fail | 只有空/NUL/byte length |
| G04-G05 version/future-field migration | Fail | Editor 仅 v1，WOC 无版本 |
| G06 schema validation | Partial | Editor schema/WOC clamp 局部存在 |
| G07 layer/provenance/apply | Partial | Editor 局部有，Runtime 未统一 |
| G08 secret/security | Fail | 无分类、redaction、secret store |
| G09-G10 reactive read/typed state | Fail | hidden cold read + default-on-Pending |
| G11 max-size concurrent read | Fail | 默认预算约容纳一个最大 cold read |
| G12 4097+ churn | Fail | durable/tombstone 不可 eviction |
| G13 backend generation transaction | Fail | adapter replacement 无 generation |
| G14 retry/backoff | Fail | 无分类策略 |
| G15-G16 cross-key concurrency/fairness | Fail | global single active/FIFO |
| G17 complexity | Partial | 有局部优化，缺 storm 产品门 |
| G18 identity/accounting exhaustion | Partial | overlay checked，lane 多处 saturating |
| G19 active deadline/cancel | Fail | 只在 start 前生效 |
| G20 permanent hang exit | Fail | final guard Drop 可永久等待 |
| G21 provider capability conformance | Fail | 只有 backend kind |
| G22 true durability | Fail | HostProvided primitive success 报 Durable |
| G23 full fence receipt | Partial | ordering有unit test，只返回代表 failure |
| G24 lifecycle transaction | Partial | Editor close有，Platform只drain |
| G25 deactivate/reactivate | Partial | descriptor factory可重建默认 policy，无 provider generation/state receipt |
| G26-G30 record/crash/CAS/watch | Fail | envelope、kill matrix、recovery、CAS、watch 均缺 |
| G31 platform error map | Partial | 部分 OS error 有测试 |
| G32 root security | Fail | 无 canonical/symlink/ACL/sandbox 矩阵 |
| G33 capability health | Fail | 不能区分 probed/healthy/degraded/closed |
| G34 observability | Partial | 有基础计数，缺 generation/percentile/correlation |
| G35 Editor single authority | Fail | SettingsStore 仍独立写 |
| G36 WOC production owner | Fail | `Stored*::new` 生产调用为 0 |
| G37 stable principal | Fail | display name 进入 scope |
| G38 fresh-process E2E | Fail | 无 App/WOC/Editor restart receipt |
| G39 performance qualification | Fail | 无 p50/p95/p99/RSS/write amplification |
| G40 hard cutover | Fail | 旧 API/key/document 均仍是 contract |

### 13.2 Config 12 门

| Gate | 状态 | 必须证明 |
|---|---|---|
| CONFIG-G01 domain separation | Fail | BootPlan/RuntimeState/Preference 不共享任意 JSON map |
| CONFIG-G02 product/principal isolation | Fail | Editor/Client/产品实例物理与逻辑隔离 |
| CONFIG-G03 boot precedence | Fail | 所有 module activation 读取 pinned boot/effective snapshot |
| CONFIG-G04 schema/migration | Fail | typed registry、版本、N-2迁移、unknown policy |
| CONFIG-G05 bounded change persistence | Fail | 不因单 key 变化重写全局 map |
| CONFIG-G06 multi-process conflict | Fail | two-process same revision 返回 Conflict，不丢数据 |
| CONFIG-G07 crash/recovery | Partial | 有 atomic+single backup，缺 kill/staging/quarantine receipt |
| CONFIG-G08 path/security | Fail | canonical root、ACL、symlink、sandbox 通过 |
| CONFIG-G09 bounded shutdown | Partial | 有 timeout，仍可能 detached worker/late operation |
| CONFIG-G10 typed failure | Fail | error/source/recovery action 完整跨层 |
| CONFIG-G11 fresh-process/multi-instance | Fail | Editor layout 与产品 config restart/race 通过 |
| CONFIG-G12 performance | Fail | 大 map、频繁更新的 latency/RSS/write amplification 门 |

## 14. 测试证据与缺口

| 现有测试 | 能证明 | 不能证明 |
|---|---|---|
| Runtime Preference 选择集 90 个 test / 10 ignored | key bound、overlay generation、coalescing、fence、pre-start deadline、panic、bounded cleanup、atomic同进程reload、局部性能优化 | true buffered durability、cross-key并行、active cancel、permanent hang退出、CAS/recovery/watch、跨进程 |
| Config authority 35 个 test / 3 ignored | debounce/coalescing、并发内存更新、same-value retry、atomic failure、bounded flush、进程内late-writer fence、single-backup recovery | BootPlan precedence、产品隔离、跨进程 conflict、schema migration、security、全局写放大 |
| Editor boundary 68 个 test / 1 ignored | schema/layer/document mutation/health/UI projection 的局部语义 | Runtime 单一 durable authority、project transition原子性、multi-process/crash |
| WOC 84 个 test | JSON兼容、手工 refresh、range clamp、keybind repair、MemoryPreferenceStorage terminal | 生产构造、wake、App effect消费、真实backend、restart/failure UX |

必须新增的最小动态矩阵：buffered provider、permanent-hang subprocess、4097+ durable/tombstone churn、diverse-key 100k storm、timer unavailable、reverse completion、two-process CAS race、two Editor layout race、kill-point record/config recovery、external watch、fresh-process App/WOC/Editor restart、root security、mobile/Web/headless provider、release profile p50/p95/p99/max/RSS/write amplification。

## 15. 逐文件/目录检查台账

| 文件/目录 | 当前事实 | 裁决 |
|---|---|---|
| `core/framework/platform/preferences/{key,error,backend_kind}.rs` | bounded string key、五类 error、三类 backend kind | 无 canonical address/schema/owner/capability |
| `preferences/storage/*` | snapshot/mutation/flush/evict/ticket/terminal/deadline | hidden load、无 revision/wake、Durable 语义过强 |
| `bounded_keyed_io/{admission,key,ticket,diagnostics}.rs` | typed equality、entry/byte reservation、multi-wait ticket、基础统计 | active cancel、typed failure、owner identity不足 |
| `bounded_keyed_io/lane.rs` | suspended/queue/pump/fence/shutdown linearization | single active、pre-start-only deadline、saturating identity |
| `lane/coalescing.rs` | linear partition、fast-tail insertion、single-pass successor | storm/fence 总体复杂度仍无门 |
| `lane/fence_prerequisites.rs` | prerequisite计费、superseded/failure chain | full receipt、稳定顺序、规模资格不足 |
| `lane/shutdown.rs` | wait_until/report 可查询 | guard Drop 无界 wait |
| `platform/preferences/backend.rs` | worker-only authority + read/write/remove/flush | 无 capability/context/CAS/watch/cancel |
| `atomic_file.rs` | hash path、atomic commit、sync、bounded path cache | 裸 payload、无 manifest/revision/integrity/recovery/security |
| `persistence/overlay.rs` | checked generation、read-your-write、防 stale read、O(1) durability counters | durable/tombstone 不可淘汰，failure顺序不确定 |
| `persistence/adapter.rs` | submission lock、lane、overlay、fence | hidden cold read、global mutex、flush不升级operation、backend无generation |
| `persistence/work.rs` | max+1 read、panic-safe work wrapper | backend primitive success直接成为 Durable |
| `platform/service_types/{driver,manager}.rs` | one-shot install、stable manager service | public manual late install、无 policy/provider generation |
| `platform/module.rs` | 250 ms cleanup budget与失败保留service | 不先flush，最终 owner Drop 仍可挂 |
| `zircon_app/src/entry/platform_preferences.rs` | desktop OS root、mobile/Web/headless injection、factory bootstrap | 品牌级root，无product/principal，非desktop无标准provider |
| `core/runtime/config_store.rs` | Arc<Value> map、poison recovery、短锁snapshot优化 | transient/durable混域、无revision/schema/notification |
| `foundation/runtime/config_manager.rs` | load whole map、manager service、same-value dirty判断 | arbitrary global persistence、boot precedence不显式 |
| `config_manager/worker.rs` | debounce worker、flush wait、bounded Drop wait | timeout后线程可detached，no active cancellation/cross-process owner |
| `config_manager/state.rs` | generation、bounded latency ring、report | saturating generation、pending只是一位、无queue/owner/backend identity |
| `config_manager/commit_fence/*` | 单进程path gate、late writer fencing | `wrapping_add`、无跨进程 lock/CAS |
| `config_path.rs` | OS config dir + env override + cwd fallback | 只有品牌名，无canonical product root/security receipt |
| `zircon_editor/ui/host/layout_persistence.rs` | production layout 通过 ConfigManager set/get | 与 Editor SettingsStore形成第二套Editor preference path |
| `zircon_editor/core/settings/*` | typed schema/layer/snapshot/document lane/health | 独立 filesystem writer，详见 Editor265 |
| `examples/woc/.../preferences/*` | typed definition与JSON storage helper | 无生产 owner、schema version、wake/receipt |
| `input/{gamepad,keybind}/storage.rs` | repair/compat与完整文档写 | 测试构造，scope identity不稳定，无CAS |
| `windows/{inventory,routes}.rs` | route生成 persistence effect | effect 无生产 consumer |

## 16. 父 owner 与依赖

| 主题 | canonical owner | Runtime203 边界 |
|---|---|---|
| Core config/schema/diagnostics | Runtime03 / Runtime99s | 将 ConfigStore 明确为 transient boot/runtime config，不重复建万能 catalog |
| Filesystem atomic/recovery/security | Runtime25 | 复用 durable transaction、安全 path、recovery primitive |
| Task scheduler/cancel/shutdown | Runtime59 / Runtime02 / Runtime11 | 修复 Preference single-active、active cancel/deadline、Drop 资格 |
| SaveGame/large payload | Runtime40 / Runtime99zd | Preference 拒绝并路由，不吞并 SaveGame |
| Editor Settings UI/schema | Editor265 / Editor12 | 保留 authoring/UI/apply，durable writer hard cut 到 Runtime |
| App product/platform lifecycle | App01 / Runtime57 | App 提供 product identity/provider/final shutdown receipt |
| Module/provider composition | Runtime42 / Runtime193 | provider policy来自 immutable composition snapshot |

## 状态与产出记录

| 日期 | 里程碑 | 状态 | 产出/验证 |
|---|---|---|---|
