---
related_code:
  - zircon_runtime/src/core/framework/platform/preferences
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io
  - zircon_runtime/src/platform/preferences
  - zircon_runtime/src/platform/service_types/driver.rs
  - zircon_runtime/src/platform/service_types/manager.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/platform/config.rs
  - zircon_runtime/src/platform/capability
  - zircon_app/src/entry/platform_preferences.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_editor/src/core/settings
  - examples/woc/native/apps/woc_client/src/preferences
  - examples/woc/native/apps/woc_client/src/input/gamepad/storage.rs
  - examples/woc/native/apps/woc_client/src/input/keybind/storage.rs
  - examples/woc/native/apps/woc_client/src/shell/offline_session.rs
  - examples/woc/native/apps/woc_client/src/windows/inventory.rs
tests:
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/tests.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/tests
  - zircon_runtime/src/platform/preferences/persistence/tests.rs
  - zircon_runtime/src/platform/preferences/persistence/tests
  - zircon_runtime/src/platform/tests/preferences.rs
  - zircon_editor/src/core/settings/tests
  - examples/woc/native/apps/woc_client/tests/preferences
  - examples/woc/native/apps/woc_client/tests/input/gamepad/storage.rs
  - examples/woc/native/apps/woc_client/tests/input/keybind/storage.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/zircon_runtime/frameworks/05/failure-2026-07-19-cross-platform-preference-storage-service.md
  - docs/plans/zircon_runtime/frameworks/05/failure-2026-07-22-preference-quota-error-kind-toolchain-drift.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-31-preference-storage-bounded-persistence-lane.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigHierarchy.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigContext.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameUserSettings.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameUserSettings.cpp
  - dev/godot/core/config/project_settings.h
  - dev/godot/core/config/project_settings.cpp
  - dev/godot/editor/settings/editor_settings.h
  - dev/godot/editor/settings/editor_settings.cpp
  - dev/godot/tests/core/config/test_project_settings.cpp
  - dev/Fyrox/editor/src/settings/mod.rs
  - dev/Fyrox/editor/src/plugins/settings.rs
  - dev/Fyrox/editor/src/settings
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 45 · Preference / Settings / Scope / Storage / Overlay / Bounded I/O / Generation / Fence / Durability / Migration / Multi-Process 工程化差距

## 1. 结论

Zircon当前的Preference链并非简单占位。中立层已有`PreferenceStorage`、受限key、结构化基础错误；平台层有read-your-write overlay、entry/byte双重admission、首次读取`max + 1`硬上限、per-key generation coalescing、fence prerequisite pin、任务开始deadline、panic捕获、Condvar shutdown guard和指标；atomic file后端复用了共享`stage_atomic_write`，具备exclusive staging、文件同步、替换/备份和父目录同步。App也会在Platform module激活前把backend注入descriptor factory。这些基础值得保留。

但现有合同存在五条不能用“后续增强”掩盖的正确性断路。第一，backend `write/remove`成功会立即把overlay terminal标为`Durable`，而trait另有独立`flush()`，HostProvided后端完全可能只完成buffered write；Platform cleanup又只shutdown lane，不先建立flush fence，所以API会对未耐久数据发出错误终态。第二，WOC的`refresh_from_storage()`与`take_persistence_submission()`在生产代码中没有caller：冷读Pending时模型直接固定默认值，mutation admission或terminal ticket也没有产品owner消费。第三，一个永久挂起的backend操作会阻塞全局唯一active lane、所有key、fence和shutdown；cleanup超时后guard最终`Drop`仍无界`wait()`，进程或DLL teardown可永久挂死。第四，App默认把所有产品放进同一`ZirconEngine/preferences`根，key又没有product/project/channel/profile/account owner，跨产品数据隔离与授权不存在。第五，跨进程没有lock、revision/CAS、external watch或merge，WOC整份JSON的读改写在并发进程中必然是last-writer-wins并可静默丢更新。

此外，`PreferenceKey`只有两个字符串，schema、scope、layer、default、validator、migration、apply policy、secret policy和namespace ownership全由consumer私造；`snapshot()`暗中触发I/O却不给ticket/wake/subscription；默认64 MiB单值上限配合两层各128 MiB预算，使默认配置下同一时刻只能容纳一个最大cold read。成功值和成功tombstone没有淘汰路径，4096个不同key后服务可永久拒绝新key。单lane没有key级并行、公平、优先级、backend timeout或cooperative cancellation，多个计数器又以saturating arithmetic掩盖身份耗尽。

Editor同时维护第二套`settings.toml` JSON envelope、同步读取、独立atomic writer和另一条Runtime11 persistence lane；这不是本报告重复展开Editor12，而是明确 durable storage owner 必须收敛。Runtime03继续拥有通用Config Registry/layer，Runtime25拥有filesystem/atomic recovery，Runtime40拥有SaveGame边界，Editor12拥有Preferences UI/schema/locale/editor scope。Runtime45只拥有跨产品Preference服务、scope/schema合同、backend capability、overlay/cache、bounded I/O、durability fence、multi-process conflict和产品消费闭环。

本报告新增 **5项P0、58项P1、14项P2和40个资格门**。三个既有failure继续保持`open`；Runtime11 failure所述“WOC hard cut implemented”与当前生产调用图不一致，只能保留为未验收基础，不能关闭。本文只做review和重构计划，不修改生产代码、测试、Cargo、ABI或资产。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 证据等级 | 本轮检查重点 |
|---|---:|---:|---|
| Framework preference contract | 5 / 493 / 12,593 | E3 | key、backend kind、error、snapshot/mutation/fence中立合同 |
| Bounded keyed I/O | 12 / 2,431 / 78,763 | E3 | admission、generation、coalescing、fence prerequisites、deadline、cancel、shutdown、panic与测试 |
| Platform backend and lifecycle | 17 / 3,715 / 129,918 | E3 | overlay、atomic file、backend replacement、module cleanup、capability、diagnostics |
| App integration | 3 / 1,187 / 43,955 | E3 | 默认根、descriptor factory注入、builtin entry重激活语义 |
| Editor settings owner | 16 / 3,937 / 135,386 | E2/E3 | 第二持久化authority、scope document、migration、flush/shutdown边界 |
| WOC preference consumers | 30 / 5,723 / 175,436 | E3 | settings/keybind/gamepad/offline/inventory真实caller、codec与测试闭环 |
| 开放failure与父报告 | 7 / 1,811 / 202,778 | E2 | 状态、唯一owner、不得重复计数的父问题 |
| Unreal、Godot、Fyrox参考 | 23 / 21,127 / 787,210 | E2/E3 | hierarchy/context、dirty/validate/apply/confirm/migrate、metadata/change tracking与Rust editor适配 |
| selected combined scope | 113 / 40,424 / 1,566,039 | E2/E3 | 工作树fingerprint `0ab3e6f7ee694d007190303cd1f9ca795fefcc7b3cc7a806363f1c2156ed31bf` |

指纹按113个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path<TAB>hash`和LF连接、无末尾LF后取总SHA-256。统计冻结的是2026-08-19当前工作树；其中Editor settings测试含其他会话在途修改，本报告只读取并保留它，不据此改写测试结论。基线提交为`25e09a23178000f2e783ce2143cf70a8b118d404`。

### 2.2 检查方法

按`App product identity -> Platform module/backend install -> neutral PreferenceStorage -> key/scope/schema -> snapshot cold read -> overlay admission -> bounded lane -> backend primitive -> terminal observer -> fence/flush -> shutdown/drop -> Editor/WOC consumer -> second process/crash/restart`顺序逐段阅读，并反向搜索生产调用。每段核对authority、identity、ordering、memory、latency、failure、durability、security、migration、multi-process和qualification。

参考引擎只选择与本域直接可比的语料：Unreal的ConfigContext/Hierarchy与GameUserSettings，Godot的ProjectSettings/EditorSettings，Fyrox的Rust editor settings。Bevy没有可直接对标的产品Preference系统，Unity Graphics也不是持久化设置owner，因此未为了“覆盖名称”强行类比；相关思想只在它们真正拥有的模块报告中引用。

### 2.3 开放failure状态

| Failure | 当前源码复核 | 本报告处理 |
|---|---|---|
| Framework05 cross-platform preference storage | 中立service和平台adapter已存在，但scope/schema/provider qualification/product root仍未闭环 | 保持`open`；挂接P0-001/004与G01-G40 |
| Framework05 quota error kind/toolchain drift | 双层entry/byte quota已存在，但错误分类、per-owner政策、默认cold-read容量和缓存回收不足 | 保持`open`；不得把“有quota”写成fixed |
| Runtime11 bounded persistence lane | lane、overlay、fence测试基础存在；独立验收、Cargo证据和WOC生产refresh/harvest caller仍缺失 | 保持`open`；修正“WOC hard cut implemented”的过早表述 |

本轮未重新执行Cargo。既有Editor编译、Hub persist、WOC协议、npm计数和plugin locked metadata阻断仍属仓库其他owner；静态报告不能把这些阻断或本域动态资格标记为通过。

## 3. 必须保留的工程基础

1. 保留Framework层中立`PreferenceStorage`方向，但升级为versioned、scoped、revisioned合同。
2. 保留`PreferenceKey`长度上限，扩展为canonical structured identity和namespace lease。
3. 保留read-your-write overlay与generation防陈旧完成覆盖，补全cache revision、backend generation和外部变更失效。
4. 保留entry/retained-byte双重admission，把预算扩展到tenant、active I/O、backend buffer和metadata真实成本。
5. 保留首次读取`Read::take(max + 1)`，禁止先无界读再检查。
6. 保留per-key coalescing和fence prerequisite accounting，明确epoch/identity耗尽和多key transaction语义。
7. 保留任务开始deadline、panic捕获和结构化metrics，增加active I/O deadline/cancel、fairness和latency分布。
8. 保留共享atomic writer，补上Preference record envelope、recovery、sweep、quarantine、writer lock和CAS。
9. 保留App在module activation前冻结backend的顺序，改为冻结完整product/storage policy与backend capability receipt。
10. 保留Editor setting definition/registry/scope和WOC typed model，但删除它们各自发明的durability与codec隐含合同。
11. 保留真实错误注入和fence accounting测试，扩展到进程、崩溃、重启、磁盘和产品端到端。
12. 保留Unavailable backend的诚实错误方向，不得用enum或平台名伪造可用性。

## 4. 当前实现链与断路

```text
BuiltinEngineEntry
  -> fixed user-data/ZirconEngine/preferences root
  -> PlatformModule descriptor factory captures backend
  -> PlatformDriver installs PreferencePersistenceAdapter once
       -> overlay: PreferenceKey(namespace,key) -> bytes/tombstone/generation
       -> one global BoundedKeyedIoLane -> backend read/write/remove/flush
       -> terminal observer marks mutation Durable on write/remove success

WOC model constructor -> snapshot() -> Pending => unwrap_or_default()
    refresh_from_storage() -------------------------- test caller only
    take_persistence_submission() ------------------ test caller only

Editor SettingsRegistry -> settings.toml(JSON) -> separate atomic writer/lane
second process ----------> same hashed key path -> last writer wins
```

| 阶段 | 当前事实 | 工程断路 |
|---|---|---|
| product/root | 默认固定`ZirconEngine/preferences` | 无product/project/channel/profile/account隔离 |
| key/schema | 两个String定位裸bytes | 无owner、scope、schema、revision、migration和secret policy |
| cold read | `snapshot()`隐式入队并返回Pending | 无ticket/wake/subscription，consumer只能轮询或固定默认值 |
| overlay | generation避免陈旧read覆盖new write | durable/tombstone不回收，外部进程和backend generation不可见 |
| lane | 全局单active、FIFO、count/byte bound | 一个hung backend阻塞全部key和shutdown；无tenant公平 |
| backend | hashed path + atomic replace | 无envelope、CAS、lock、watch、recovery sweep或manifest |
| terminal | write/remove成功即`Durable` | HostProvided可尚未flush，语义造假 |
| cleanup | 固定250 ms lane shutdown | 无final flush；超时后guard drop仍可无限等待 |
| product | WOC只有测试手动refresh/harvest | 生产闭环不存在，失败和durability无人消费 |
| editor | 第二套document/atomic/lane owner | scope/schema/持久化事实分裂且可能互相覆盖 |

## 5. P0：正确性与产品闭环阻断

| ID | 阻断 | 直接证据 | 必须修复 / 验收 |
|---|---|---|---|
| PREF-P0-001 | 未flush的HostProvided mutation被误报`Durable`，cleanup也不建立flush fence | backend trait把`write/remove`与`flush`分开；overlay在primitive成功后直接记录Durable；module cleanup只shutdown lane | 引入durability level与backend capability；只有完成声明级别的fence receipt才能发Durable；shutdown先final fence再drain；buffered backend故障/断电测试 |
| PREF-P0-002 | WOC生产代码没有cold-read完成和mutation terminal owner | `refresh_from_storage`、`take_persistence_submission`只在定义/测试出现；Pending被`unwrap_or_default`固定；admission error可变`None` | App/WOC host建立reactive owner，启动等待或订阅read terminal，持续harvest mutation/fence receipt并暴露失败；真实产品重启测试 |
| PREF-P0-003 | 永久挂起backend可冻结所有preferences并在最终drop永久挂进程/DLL | lane全局仅一个active；cleanup 250 ms后保留service；shutdown guard `Drop`无界`wait()` | active I/O必须有deadline/cancel/隔离或可遗弃worker边界；final drop不得无界join；hung backend进程退出资格 |
| PREF-P0-004 | 固定全局root与无owner namespace破坏产品、项目、渠道、账号隔离 | App默认root只含`ZirconEngine/preferences`；任何consumer可构造任意namespace/key | App冻结`ProductStorageIdentity`和principal scope；namespace经manifest/lease授权；不同product/project/channel/profile/account物理与逻辑隔离测试 |
| PREF-P0-005 | 多进程并发整值写入静默丢更新 | atomic replace只保证单文件完整；无writer lock、revision/CAS、watch、merge；WOC settings是整份JSON | record revision + expected revision/CAS + conflict receipt；必要时单writer broker/lock与typed merge；双进程同key压力和kill/restart测试 |

## 6. P1：必须完成的工程重构

### 6.1 合同、scope、schema与权限

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| PREF-P1-01 | `PreferenceKey`只含namespace/key字符串 | 定义稳定`PreferenceAddress`，包含product、project、principal、scope、namespace和logical key | Runtime45；跨平台roundtrip与canonical hash |
| PREF-P1-02 | 无User/Account/Profile/Project/Session/World/Plugin scope | 建立typed scope taxonomy和继承/禁止持久化规则 | Runtime45 + Editor12；scope矩阵无字符串猜测 |
| PREF-P1-03 | namespace无所有权或授权 | composition/catalog分配namespace lease，读写携带owner identity与权限 | Runtime42/Plugins01 + Runtime45；越权读写Denied |
| PREF-P1-04 | 无大小写、Unicode、分隔符和confusable规范 | 冻结canonical normalization，存储hash和诊断显示共用同一结果 | Runtime45；Unicode/pathological corpus |
| PREF-P1-05 | value只是`Arc<[u8]>` | 引入content type、schema id/version、codec id和validated payload envelope | Runtime03 + Runtime45；codec negotiation/roundtrip |
| PREF-P1-06 | 无default/source layer/override origin | 定义Default/Engine/Project/User/Profile/Session/CommandLine层及provenance | Runtime03 + Runtime45；effective value可解释 |
| PREF-P1-07 | 无validator、restart/live-apply与dirty语义 | schema registration声明range、dependency、apply policy、revert和dirty comparator | Editor12 + Runtime45；非法值不进入durable store |
| PREF-P1-08 | 无migration/unknown-field preservation | 每schema提供forward migration、compat window与未知字段保留/拒绝策略 | Runtime45；N-2到N与future-version测试 |
| PREF-P1-09 | 无secret/PII分类、redaction、encryption | schema声明sensitivity；日志/diagnostics默认redact；平台secret store与普通preference分域 | Runtime45 + Runtime03；secret corpus零泄漏 |
| PREF-P1-10 | 无multi-key batch/transaction | 提供prepare/commit/abort与原子可见receipt，或明确拒绝跨key原子性 | Runtime45；断电与partial failure矩阵 |
| PREF-P1-11 | API无expected revision和conflict | read返回revision/etag，write/remove可携带expected revision并返回Conflict | Runtime45；并发更新不静默覆盖 |
| PREF-P1-12 | error taxonomy过窄且缺correlation | 增加Conflict/Schema/Migration/Auth/Cancelled/Deadline/Unsupported/Integrity，附backend instance、generation、operation ID和source chain | Runtime45；所有路径typed且可追踪 |

### 6.2 Backend、record、路径与能力

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| PREF-P1-13 | HostProvided只靠enum被视为persistent | backend报告capabilities：durability、flush、CAS、watch、transactions、max value、thread/cancel模型 | Runtime45；capability conformance suite |
| PREF-P1-14 | hashed文件没有逻辑key/schema/revision envelope | versioned record写入address digest、schema、revision、payload digest、flags和writer identity | Runtime25 + Runtime45；corruption定位与升级 |
| PREF-P1-15 | 无manifest/enumeration/export/import/reset | 建立受权manifest/index与分页管理API，支持按scope诊断和重置 | Runtime45；不能靠扫描hash目录猜测 |
| PREF-P1-16 | 无integrity校验或corruption quarantine | 校验header/payload digest，将坏记录隔离并返回Integrity而非默认值 | Runtime25 + Runtime45；bit-flip测试 |
| PREF-P1-17 | 未调用共享backup recovery和staging sweep | 启动时恢复last-known-good，清理/隔离staging与backup，记录receipt | Runtime25；kill-point矩阵 |
| PREF-P1-18 | 无文件权限、ACL、symlink与sandbox策略 | root解析时冻结安全策略，创建/验证权限并拒绝越界解析 | Runtime25 + App01；平台安全矩阵 |
| PREF-P1-19 | Windows/macOS/root环境值缺绝对性与canonical receipt | 统一path policy compiler，支持portable/sandbox模式并返回实际root identity | App01/Runtime25；环境变量恶意输入测试 |
| PREF-P1-20 | mobile/Web/headless只要求外部注入，没有标准provider资格 | 定义平台provider包、capability probe和Unavailable原因，不以平台名替代证据 | Runtime45；各target真实provider矩阵 |
| PREF-P1-21 | backend replacement没有quiesce、generation或cache invalidation | transactional replace：freeze admission、flush/drain、swap generation、invalidate/reload、resume | Runtime01 + Runtime45；无跨backend陈旧完成 |
| PREF-P1-22 | one-shot install后失败cold read不会自动恢复 | 明确late install/reconnect协议，失败entry按backend generation重试并通知subscriber | Runtime45；Unavailable到Ready转换测试 |
| PREF-P1-23 | capability report只看backend kind | 报告planned/installed/probed/healthy/degraded/closed及last error和有效能力 | Runtime03/42 + Runtime45；不能假报persistent |
| PREF-P1-24 | diagnostics缺root policy/backend instance/generation/latency | 发布有界、redacted health snapshot和p50/p95/p99、queue age、bytes、last terminal | Runtime03 + Runtime45；Editor/host可查询 |
| PREF-P1-25 | path cache只有4096 entry，没有byte budget/GC | 使用canonical address digest、byte-bound cache、LRU/epoch invalidation和指标 | Runtime45；长key workload不超预算 |
| PREF-P1-26 | I/O错误被粗粒度折叠 | 保留平台错误链并稳定映射Denied/NoSpace/ReadOnly/NotFound/Corrupt/Busy等 | Runtime25 + Runtime45；fault injection无TransientIo滥用 |

### 6.3 Overlay、read API、admission与cache

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| PREF-P1-27 | `snapshot()`有隐藏I/O副作用且Pending无handle | 拆为cache snapshot和显式async load/subscribe；返回ticket、wake和terminal | Runtime45；caller不靠轮询猜测 |
| PREF-P1-28 | missing、not-ready、read failure在consumer常折叠为None/default | typed `ValueState::{Missing,Loading,Ready,Failed,Stale}`并保留revision/source | Runtime45；UI/产品能显示失败 |
| PREF-P1-29 | 默认64 MiB read quote使128 MiB overlay/lane各只能容纳一个最大cold read | 分离header probe/streamed body，按schema/tenant设限并为并行cold read留预算 | Runtime45；默认并发资格不自相矛盾 |
| PREF-P1-30 | durable value和成功tombstone永不evict | 建立pin/lease、LRU/TTL、dirty/durable状态和tombstone compaction | Runtime45；4097+ key持续可用 |
| PREF-P1-31 | 无invalidate/reload/external-change传播 | backend watch或poll generation进入overlay，订阅者收到changed/conflict/stale事件 | Runtime45；外部写后不永久读旧值 |
| PREF-P1-32 | failure重试只靠caller再次触发且无backoff | retry policy区分transient/permanent，带jitter、deadline和operator override | Runtime45；不热循环也不永久卡死 |
| PREF-P1-33 | `known_non_durable_failure()`取HashMap任意首项 | fence按operation order确定性聚合所有相关失败，返回bounded details/page token | Runtime45；hash seed不影响receipt |
| PREF-P1-34 | retained-byte quote用常量近似，遗漏Arc/HashMap/control block | 定义可审计accounting model并测量分配；元数据、queue capacity和backend buffer纳入预算 | Runtime02 + Runtime45；RSS/allocator证据 |
| PREF-P1-35 | admission是全局mutex且无contention/fairness指标 | 分片或actor化owner state，发布lock wait和tenant公平指标 | Runtime45；多线程压力门 |
| PREF-P1-36 | max value只有全局策略 | schema、scope、tenant和backend共同裁决有效上限，过大payload路由到SaveGame/resource store | Runtime40 + Runtime45；滥用Preference被拒绝 |
| PREF-P1-37 | cache没有source/backend revision | 每个entry携带backend generation、record revision、schema generation与freshness | Runtime45；replacement/migration不读旧缓存 |
| PREF-P1-38 | `DeadlineTimerUnavailable`映射CapacityExceeded | 独立SchedulerUnavailable/DeadlineUnsupported错误和degrade policy | Runtime02 + Runtime45；capacity诊断不被污染 |
| PREF-P1-39 | `caller_filesystem_wall`固定为0 | 接入真实task/I/O wall观测，区分queue wait、backend wait、fsync和callback成本 | Runtime03 + Runtime45；预算证据可复现 |

### 6.4 Bounded lane、fence、取消与生命周期

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| PREF-P1-40 | 全局只允许一个active I/O | 按backend capability提供bounded per-key/partition concurrency，仍保证same-key顺序 | Runtime45；慢key不冻结无关key |
| PREF-P1-41 | 全局FIFO无tenant公平、priority或aging | owner quota + weighted fair scheduling + bounded priority reserve + starvation telemetry | Runtime02 + Runtime45；noisy tenant隔离 |
| PREF-P1-42 | queue插入/扫描和coalescing/fence capture可退化O(N²) | indexed per-key queues、monotonic cursor和近O(1) prerequisite tracking | Runtime45；100k storm复杂度门 |
| PREF-P1-43 | caller自行提供generation且不验证单调性 | lane/overlay唯一分配generation，拒绝回退、重复和跨backend generation | Runtime45；property test覆盖wrap/乱序 |
| PREF-P1-44 | ticket ID与epoch使用`saturating_add`最终重复/冻结 | 明确checked exhaustion、process generation与不可复用复合ID | Runtime05 + Runtime45；耗尽测试不静默复用 |
| PREF-P1-45 | fence pin/counter饱和会隐藏accounting错误 | checked arithmetic、invariant failure和诊断快照；禁止饱和后继续服务 | Runtime45；model/property test |
| PREF-P1-46 | terminal observer只有一个slot，后注册覆盖前者 | 多subscriber registry或单canonical event bus，带lease/generation/backpressure | Runtime02 + Runtime45；observer不静默丢失 |
| PREF-P1-47 | failure code是`&'static str`且unknown降级TransientIo | versioned typed failure/disposition ABI，保留unknown code而不错误归类 | Interface01 + Runtime45；forward compatibility |
| PREF-P1-48 | lane依赖进程全局TaskTimer/TaskPools | 绑定runtime/session/module generation和shutdown domain，禁止跨实例污染 | Runtime01/02 + Runtime45；多runtime隔离 |
| PREF-P1-49 | active backend工作不可合作取消或超时 | backend context提供cancel token/deadline；不可取消provider必须进可遗弃隔离执行域 | Runtime02 + Runtime45；hung I/O门 |
| PREF-P1-50 | shutdown后lane不可重开，module重激活语义不完整 | lifecycle state machine定义activate/quiesce/flush/drain/stop/reactivate和receipt | Runtime01 + Runtime45；反复激活测试 |

### 6.5 App、Editor、WOC与资格闭环

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| PREF-P1-51 | 手工backend install在deactivate/reactivate后会丢失 | backend policy进入descriptor/composition source of truth，重激活按generation重建 | App01/Runtime01 + Runtime45；manual install重激活测试 |
| PREF-P1-52 | Unavailable配置仍会构建adapter和TaskPools | capability admission先裁决provider；Unavailable路径不启动无用worker/资源 | Runtime42 + Runtime45；资源计数为0 |
| PREF-P1-53 | WOC settings/keybind/gamepad JSON无schema version并会丢future fields | 共享versioned codec、migration、unknown preservation和validation registry | App03/04 + Runtime45；旧版/未来版roundtrip |
| PREF-P1-54 | WOC scope用`offline:{class}:{player_name}`等字符串拼接 | 使用稳定account/profile/character principal ID与typed components，display name不参与identity | App03/04 + Runtime45；rename不迁移key |
| PREF-P1-55 | `submit_preference_text`用`Option`吞掉key/admission错误 | 返回typed submission/error；UI与host展示、重试或回滚，不静默保留假成功模型 | App04 + Runtime45；故障路径产品测试 |
| PREF-P1-56 | Inventory只发`PersistInventoryFilter(String)` effect，无host receipt | route effect进入统一operation/preference owner并把terminal disposition反馈模型 | Runtime41/App04 + Runtime45；状态与durable receipt关联 |
| PREF-P1-57 | Editor独立scope document会为不同key反复整文档重写 | 按Editor12收敛schema/dirty/apply UI，durability统一委托Runtime45；需要整文档时用revision transaction | Editor12 + Runtime45；无双authority |
| PREF-P1-58 | 测试多为同进程in-memory/manual pump，不能证明产品闭环 | 建立真实atomic backend、fresh process、crash、multi-process、App/WOC/Editor端到端矩阵 | Runtime45；40门有归档receipt |

## 7. P2：重要但不阻断首个工程闭环

| ID | 改进项 | 验收方向 |
|---|---|---|
| PREF-P2-01 | 给Preference contract和manager暴露显式版本，不只依赖类型名 | capability negotiation与兼容测试 |
| PREF-P2-02 | 把当前magic limits收敛为profile/platform policy asset | policy hash进入diagnostic receipt |
| PREF-P2-03 | backend name从`&'static str`升级为稳定type ID + instance ID + display label | 多实例诊断无歧义 |
| PREF-P2-04 | 为hashed store提供受权的离线inspect/repair工具 | 不暴露secret且可定位损坏记录 |
| PREF-P2-05 | Editor的`settings.toml`实际为JSON，应在迁移时更名或真正使用声明格式 | 扩展名、content type和schema一致 |
| PREF-P2-06 | 用event/wake替代测试与consumer中的busy polling/yield | idle CPU和wake latency资格 |
| PREF-P2-07 | diagnostics counter饱和应报告exhaustion，不继续伪造总量 | checked snapshot与长期soak |
| PREF-P2-08 | poison recovery路径应有invariant audit，不只`into_inner`继续运行 | panic injection后状态证明 |
| PREF-P2-09 | 优化path digest/string allocation和cache locality | allocator与百万key microbenchmark |
| PREF-P2-10 | 将有副作用的`snapshot`重命名并提供纯cache peek | API语义与文档测试一致 |
| PREF-P2-11 | flush/fence error返回可分页的全部failure，而非单个代表项 | 大失败集有界且确定性 |
| PREF-P2-12 | 为超出Preference定位的大blob给出明确resource/SaveGame迁移提示 | 产品诊断能指出正确owner |
| PREF-P2-13 | 测试临时目录保留failure artifact和recovery manifest | CI失败可复盘，不被cleanup抹掉 |
| PREF-P2-14 | 增加steady-state/cold-start/flush/shutdown的长期性能基线 | p50/p95/p99/max、RSS、IO写放大受控 |

## 8. 参考引擎证据与适用性

### 8.1 Unreal Engine：显式context、hierarchy与产品设置事务

`FConfigContext`区分hierarchy、single file、local/GConfig、platform、plugin和async load所有权；`FConfigFileHierarchy`把Engine/Base/Project/Platform/Generated/User/Plugin等层与顺序显式建模，config cache也提供ready、flush、unload和platform cache边界。可借鉴的不是全局singleton或无类型字符串，而是“加载上下文、层、平台和最终authority必须显式且可追踪”。

`UGameUserSettings`进一步把dirty check、validate/version/reset、load、apply resolution、apply non-resolution、confirm/revert video mode、save、hardware benchmark和scalability分开。`SaveSettings`前重新读取窗口位置以避免覆盖另一进程更新，直接说明用户设置不是“把当前结构序列化就完成”。Zircon应吸收事务阶段、冲突意识、apply/confirm/revert和migration，不复制UE宏、singleton与Variant风格接口。

### 8.2 Godot：metadata、change tracking与兼容迁移

`ProjectSettings`为每项设置维护persist/basic/internal/initial/hide/restart/property info/order等metadata，并处理feature override、路径identity、changed settings、custom load/save和配置版本。`EditorSettings`继续维护current/initial/default、changed set、property hint/revert、project metadata dirty、shortcut、favorites/recent和兼容rename；测试明确验证same-value不会制造change。

Zircon应吸收typed metadata、dirty/change semantics、versioned migration和编辑器可解释性；不应复制Godot的global singleton和动态Variant作为核心Rust合同。

### 8.3 Fyrox：Rust editor分组与订阅可借鉴，durability不是目标上限

Fyrox用serde/reflect构建分组`SettingsData`，提供subscriber、`need_save` dirty状态和基于reflection/search的设置窗口，适合参考Rust类型组织和Editor UI生成。但其直接`File::create("settings.ron")`整文件覆盖没有Zircon所需的atomic recovery、multi-process conflict或durability receipt，只能作为UI/类型组织参考，不能作为存储验收基线。

## 9. 目标架构与owner边界

```text
App ProcessHost
  -> ProductStorageIdentity(product/project/channel/install/user/profile)
  -> PreferencePolicy(schema registry, scope rules, quota, security)
  -> PreferenceService generation
       -> namespace leases / authorization
       -> revisioned cache + subscription
       -> fair bounded operation scheduler
       -> backend capability adapter
       -> transaction / fence / durability receipts
       -> health + audit stream
            |
            +-> AtomicFileBackend: envelope/CAS/lock/recovery/watch
            +-> Platform cloud/secret providers: explicit capabilities

Editor Settings UI ---- typed schema/dirty/apply/revert ----+
WOC settings/keybinds - typed model/migration/receipt -------+-> one PreferenceService
Runtime Config ------- layered read-only/effective view -----+
SaveGame/resources --- rejected/routed to their own owners
```

| Owner | 唯一职责 | 不得继续拥有 |
|---|---|---|
| App01 | product/storage identity、root policy、final shutdown decision | 裸固定`ZirconEngine/preferences` |
| Runtime45 | Preference contract、scope/schema、cache、scheduler、backend capability、revision/fence/receipt | Editor/WOC私有durability语义 |
| Runtime03 | 通用config layer/effective view、canonical diagnostics | 文件后端与产品设置UI |
| Runtime25 | path/VFS/atomic writer/recovery/lock/watch primitive | Preference schema和consumer policy |
| Runtime40 | SaveGame/checkpoint/cloud大状态 | 小型用户Preference |
| Editor12 | Preferences UI、definition、locale、appearance、plugin extension、apply/revert | 第二套durable store/lane |
| App03/04 | WOC产品模型、principal和错误呈现 | 手写string scope、无人harvest ticket |
| Runtime01/02/42 | lifecycle/task/composition capability | 由enum假报backend健康 |

## 10. Hard Cutover约束

1. 禁止保留“write成功即Durable”的兼容别名；旧terminal必须一次性替换为分级receipt。
2. 禁止让旧`PreferenceKey(namespace,key)`与新structured address长期双写；迁移器必须有单次、幂等、可回滚receipt。
3. 禁止Editor `settings.toml`和Runtime Preference同时成为同一设置的authoritative writer。
4. 禁止WOC继续用`Option`、默认值或测试手工poll掩盖loading/failure/terminal状态。
5. 禁止用atomic rename声称解决multi-process冲突；没有revision/CAS就只能声明last-writer-wins。
6. 禁止backend kind或路径存在即声明persistent capability；必须以probe和实际generation health为准。
7. 禁止永久hung provider留在进程内无界join路径；provider capability必须说明cancel/timeout/isolation。
8. 禁止新schema把secret、SaveGame或大blob塞入普通Preference以绕过owner。
9. 禁止以缓存4096项“有上限”宣称工程化；必须证明持续使用时可回收且不永久拒绝新key。
10. 所有旧路径删除前必须有migration、fresh-process reload、failure rollback和source guard证据。

## 11. TDD实施里程碑

### M0：合同与失败语义冻结

先写compile-fail/API tests和model tests，冻结`PreferenceAddress`、scope/schema、revision、durability level、typed error、submission/fence receipt和backend capability。PREF-P0-001/004/005在此成为不可被旧API表达的编译期阻断。

### M1：Product identity、namespace lease与schema registry

App冻结`ProductStorageIdentity`，composition发放namespace lease；实现schema registration、validator、default/layer/source、migration和security metadata。先接只读peek/load与typed state，不动旧文件。

### M2：Revisioned overlay与reactive read

把隐藏I/O的snapshot拆为cache peek、explicit load和subscription；entry携带backend/schema/record generation，加入LRU/TTL/tombstone compaction、deterministic failure aggregation和backend replacement invalidation。

### M3：Fair bounded scheduler与hung-I/O隔离

lane收敛为per-key ordered、cross-key bounded concurrent和tenant-fair scheduler；加入active deadline/cancel、checked identity exhaustion、multi-observer event bus及可遗弃provider隔离边界，关闭PREF-P0-003。

### M4：Record envelope、CAS、recovery与watch

在Runtime25 primitives之上实现versioned envelope、digest、revision/CAS、writer lock/broker、backup recovery、staging sweep、quarantine、manifest和external change。通过双进程与kill-point测试后关闭PREF-P0-005。

### M5：Durability fence与lifecycle transaction

backend conformance声明Buffered/Written/DataSynced/MetadataSynced；mutation terminal和flush fence返回per-operation/per-backend receipt。module按activate/quiesce/final flush/drain/stop/reactivate运行，drop不无界等待，关闭PREF-P0-001/003。

### M6：Editor/WOC hard cutover

Editor12只保留schema/UI/dirty/apply，删除第二storage authority；WOC用stable principal、versioned codec、reactive refresh和terminal harvest接入产品host，inventory effect取得receipt。完成旧key/document一次性迁移并关闭PREF-P0-002/004。

### M7：产品、平台、性能与删除旧路径

对Windows/Linux/macOS及声明支持的mobile/Web/headless provider执行fresh process、two-process、crash/restart、disk-full/read-only/corruption、4097-key、hung I/O和长期性能矩阵。归档40门receipt后删除旧API、旧文件authority、手工scope和兼容双写。

## 12. 资格门

| Gate | 必须证明的事实 | 最低证据 |
|---|---|---|
| PREF-G01 | product/project/channel/profile/account identity稳定且隔离 | 五维交叉读写测试 |
| PREF-G02 | namespace lease拒绝越权读写 | first-party/plugin adversarial test |
| PREF-G03 | canonical address跨平台/Unicode一致 | property + corpus |
| PREF-G04 | schema/codec/version negotiation可前后兼容 | N-2/N/future fixtures |
| PREF-G05 | unknown field按policy保留或明确拒绝 | roundtrip fixture |
| PREF-G06 | validator阻止非法值进入cache与store | table/property test |
| PREF-G07 | default/layer/source/dirty/apply/revert可解释 | model + Editor integration |
| PREF-G08 | secret/PII不进入普通日志、诊断和明文store | redaction/security corpus |
| PREF-G09 | cold read返回ticket/wake/typed terminal | no-poll async test |
| PREF-G10 | Missing/Loading/Failed/Stale不折叠为default | product state test |
| PREF-G11 | 默认预算允许声明的并行cold-read workload | concurrency/RSS receipt |
| PREF-G12 | 4097+ durable/tombstone key不会永久封死服务 | churn/eviction test |
| PREF-G13 | backend replacement不允许陈旧完成污染新generation | reverse completion test |
| PREF-G14 | transient failure有bounded retry，permanent failure不热循环 | virtual-time fault test |
| PREF-G15 | same-key保持顺序，unrelated key可并行 | stalled-key concurrency test |
| PREF-G16 | noisy tenant不能饿死其他owner | weighted fairness test |
| PREF-G17 | 100k storm无O(N²)退化 | complexity/latency benchmark |
| PREF-G18 | generation/ticket/epoch/fence counter耗尽显式失败 | boundary/model test |
| PREF-G19 | active operation deadline/cancel有效 | blocking backend fixture |
| PREF-G20 | 永久hung backend不阻止process/DLL退出 | subprocess timeout test |
| PREF-G21 | backend capability与实际行为一致 | conformance suite |
| PREF-G22 | write成功不会越级声明Durable | buffered backend fixture |
| PREF-G23 | flush fence只覆盖其前序且报告全部失败 | order/failure aggregation test |
| PREF-G24 | final shutdown先flush再drain且返回receipt | lifecycle integration test |
| PREF-G25 | deactivate/reactivate保留policy并更新generation | repeated lifecycle test |
| PREF-G26 | atomic record含完整envelope与digest | binary fixture/bit flip |
| PREF-G27 | crash point可恢复old或new完整值，不返回混合值 | kill-point matrix |
| PREF-G28 | staging/backup可恢复、清理或隔离 | startup sweep test |
| PREF-G29 | two-process same revision产生Conflict而非静默覆盖 | process race test |
| PREF-G30 | external update使cache失效并通知subscriber | watch/poll integration |
| PREF-G31 | disk full/read-only/denied/busy分类正确 | filesystem fault matrix |
| PREF-G32 | root canonicalization/symlink/ACL/sandbox安全 | platform security test |
| PREF-G33 | capability report区分planned/probed/healthy/degraded/closed | state transition test |
| PREF-G34 | diagnostics含generation、queue、latency、last error且已redact | snapshot schema test |
| PREF-G35 | Editor不再拥有第二durable writer | source guard + migration test |
| PREF-G36 | WOC生产host真实调用refresh/subscription和harvest | production call-chain guard |
| PREF-G37 | WOC rename display name不改变principal key | product migration test |
| PREF-G38 | App/WOC/Editor fresh-process重启保持设置 | end-to-end matrix |
| PREF-G39 | p50/p95/p99/max、RSS、I/O写放大满足profile预算 | benchmark receipt |
| PREF-G40 | 旧API、旧key、旧document双写和兼容旁路已删除 | repository source guard |

## 13. 测试与证据矩阵

| 层级 | 现有可保留证据 | 必须新增的证据 |
|---|---|---|
| unit/model | key limits、quota、generation、coalescing、fence accounting、panic/error mapping | address/schema/revision/durability state machine、checked exhaustion、deterministic aggregation |
| concurrency | same-key storm、reverse activation、timeout、shutdown stall | cross-key parallel、tenant fairness、active cancel、permanent hung provider、observer fanout |
| filesystem | atomic backend second-adapter reload、oversize bounded read | fresh process、kill points、backup recovery、staging sweep、disk full、ACL、corruption quarantine |
| multi-process | 当前缺失 | CAS conflict、lock/broker、watch、merge、simultaneous shutdown/restart |
| lifecycle | one-shot install、cleanup timeout、manager handle | final flush receipt、backend replacement、manual install reactivation、drop boundedness |
| Editor | scope/registry/persistence测试 | 单authority、migration、apply/revert、external conflict与product restart |
| WOC | 测试手工refresh/harvest、in-memory backend | production owner、reactive wake、terminal UI、stable principal、future schema与fresh process |
| performance | lane 1k/100k与基础metrics | cold-start、churn、p99/max、RSS、contention、real disk、24h soak |

现有测试中的manual pump和同进程第二adapter只能保留为确定性unit基础，不能被重新命名为产品或多进程资格。Runtime11 failure只有在正确Cargo target实际执行、WOC生产调用链存在且PREF-G09/G20/G22/G24/G29/G36/G38通过后才可关闭。

## 14. 逐文件检查台账

### 14.1 Framework contract

| 文件 | 已检查事实 | 重构落点 |
|---|---|---|
| `backend_kind.rs` | Unavailable/AtomicFile/HostProvided分类，persistent由enum决定 | capability receipt，不以kind替代行为 |
| `error.rs` | 五类基础错误、detail字符串 | 完整taxonomy、operation/backend identity与source chain |
| `key.rs` | namespace/key字符串及长度限制 | structured canonical address与namespace lease |
| `storage.rs` | snapshot/mutation/fence公开合同 | typed state、revision、durability receipt、subscription |
| `mod.rs` | re-export中立面 | versioned contract，禁止平台细节泄漏 |

### 14.2 Bounded keyed I/O

| 文件 | 已检查事实 | 重构落点 |
|---|---|---|
| `admission.rs` | entry/byte quote和全局状态 | per-owner预算、真实allocation accounting |
| `diagnostics.rs` | totals/high-water基础 | queue age、tenant/backend latency与health |
| `fence.rs` | epoch prerequisite和terminal收集 | deterministic all-failure receipt与checked exhaustion |
| `lane.rs` | 单active pump、队列、observer、TaskPools | fair partitioned scheduler、runtime generation、multi-subscriber |
| `lane/coalescing.rs` | same-key generation coalescing | indexed queue、validated generation、复杂度门 |
| `lane/fence_prerequisites.rs` | pin/counter accounting | checked invariant和bounded cursor |
| `lane/shutdown.rs` | Condvar guard、deadline等待、Drop wait | final flush transaction、hung provider隔离、bounded drop |
| `ticket.rs` | ticket/cancel/terminal handle | composite non-reusable ID、wake/subscription |
| `mod.rs` | lane公开构造和类型 | service-owned policy而非caller magic limits |
| `tests.rs`及`tests/*` | 25+ quota/coalescing/fence/timeout/shutdown/panic/storm测试 | 保留model基础，新增active I/O/process/product资格 |

### 14.3 Platform preference backend

| 文件 | 已检查事实 | 重构落点 |
|---|---|---|
| `atomic_file.rs` | BLAKE3目录/文件名、shared atomic write、bounded read | envelope/CAS/lock/recovery/watch/manifest/security |
| `backend.rs` | read/write/remove/flush primitives和replace | capability、operation context、cancel/deadline、generation |
| `unavailable.rs` | 诚实返回Unavailable | provider probe reason与零worker路径 |
| `persistence/adapter.rs` | storage API、lane、observer、shutdown bridge | revisioned reactive service和lifecycle transaction |
| `persistence/overlay.rs` | read-your-write/generation/dual quota/cache | durability修正、LRU/TTL、source revision、external invalidation |
| `persistence/work.rs` | backend work和terminal投影 | durability level、typed backend failure、active cancel |
| `persistence/tests.rs`及`tests/*` | bounded read、failure、stale read、deadline、diagnostics | buffered durability、4097 key、late install、process/crash |
| `mod.rs` / `persistence/mod.rs` | re-export和构造 | backend generation与schema/policy注入 |

### 14.4 Platform/App lifecycle integration

| 文件组 | 已检查事实 | 重构落点 |
|---|---|---|
| `platform/service_types/{driver,manager}.rs` | manager暴露storage/diagnostics，driver one-shot install | capability/probe/lifecycle receipt与backend generation |
| `platform/module.rs` | cleanup固定250 ms，只shutdown lane | final flush、policy deadline、operator disposition |
| `platform/config.rs` | limits/backend配置 | product/profile policy asset和validated compiler |
| `platform/capability/*` | 以kind报告persistent | planned/installed/probed/healthy状态机 |
| `platform/tests/preferences.rs` | key/backend/cleanup/install/reload基础 | reactivation、hung drop、capability truth |
| `zircon_app/entry/platform_preferences.rs` | OS用户目录下固定产品root | ProductStorageIdentity、安全root receipt、portable/sandbox |
| `zircon_app/entry/engine_entry.rs` | 激活前factory捕获backend | 完整policy source of truth、manual replacement持久化 |
| `entry/tests/builtin_engine_entry.rs` | builtin注入/生命周期基础 | product isolation与多次reactivate矩阵 |

### 14.5 Editor第二settings owner

| 文件组 | 已检查事实 | 重构落点 |
|---|---|---|
| `definition.rs`、`registry.rs`、`defaults.rs` | typed definitions/default/registry基础 | 接Runtime45 schema registry，保留Editor UI metadata |
| `scope.rs`、`snapshot.rs`、`change_log.rs` | Editor scope/snapshot/change model | 统一principal/revision/source/dirty语义 |
| `io.rs`、`startup.rs` | 同步`read_to_string`，versioned envelope startup | bounded async load、migration/recovery、typed state |
| `persistence.rs` | 独立Runtime11 lane，按key job整scope document写入 | 删除第二durable owner；document需CAS transaction |
| `authority.rs`、`page.rs`、`keymap_overrides.rs` | UI authority/page/keymap扩展 | Editor12保留UI/apply职责，不持有文件authority |
| `tests/*` | registry/persistence覆盖较多 | 单authority、external conflict、migration/product restart |

### 14.6 WOC真实consumer

| 文件组 | 已检查事实 | 重构落点 |
|---|---|---|
| `preferences/storage.rs` | text submit返回Option，snapshot辅助 | typed submission/error、reactive load/terminal owner |
| `preferences/settings/storage.rs` | cold readPending默认、JSON整值、last submission | versioned schema、unknown preserve、subscription/harvest |
| settings application/options/state/registry | 84项模型与apply分类基础 | validator、dirty/confirm/revert、migration和receipt |
| gamepad/keybind storage/options/combo | 独立JSON和raw scope key | stable principal、shared codec/schema、conflict handling |
| `shell/offline_session.rs` | display class/player name拼scope | typed account/profile/character identity |
| `windows/inventory.rs`、`routes.rs` | effect只携带String，无持久化terminal | operation owner、receipt反馈与错误UI |
| `lib.rs`、`main.rs` | main只打印identity，无真实host/settings组合 | App04产品host wiring与shutdown harvest |
| focused tests | 手工refresh/harvest和in-memory backend | 保留unit fixture，新增真实product/process矩阵 |

## 15. 父报告与failure回写要求

| Owner | 本报告新增的精确回写 | 不得改写为 |
|---|---|---|
| Runtime03 | config layers/schema source与Preference effective view分界；health进入canonical diagnostics | Runtime03拥有Preference backend |
| Runtime25 | record envelope需要atomic recovery、lock/CAS/watch/security primitives | atomic rename已经解决durability/冲突 |
| Runtime40 | 超大/事务性玩家状态不得塞进Preference | Preference取代SaveGame |
| Editor12 | 保留definition/UI/locale/appearance/apply，删除第二durable owner | Editor继续写独立settings document |
| Runtime01/02/42 | lifecycle/task/capability必须承载backend generation和真实健康 | enum/descriptor存在即Ready |
| App01/03/04 | product identity、WOC principal、refresh/harvest/错误呈现 | 测试手工poll等于产品闭环 |
| Framework05 failures | 保持open，挂接scope/schema/quota/provider gates | 类型存在或quota存在即fixed |
| Runtime11 failure | 保持open，修正WOC hard-cut表述并挂接G09/G20/G22/G24/G29/G36/G38 | lane unit tests等于独立验收完成 |

## 16. 审查状态与输出记录

- 审查状态：`review_complete`。
- 实施状态：`pending`。
- 新增finding：5 P0 / 58 P1 / 14 P2。
- 新增资格门：40，当前0项完整通过。
- 既有开放failure：Framework05两项、Runtime11一项，全部保持`open`。
- 本轮没有修改生产代码、测试、Cargo、ABI、feature、资产或发行配置。
- 本轮没有运行Cargo测试；静态源码审查不替代buffered durability、hung I/O、multi-process、crash/restart和产品端到端证据。
- 实施必须遵循MVP L0-L5依赖与M0-M7里程碑：先修合同/identity/durability真相，再切scheduler/backend，最后切Editor/WOC产品与删除旧路径。
- 实施前必须重取source fingerprint，并复核当前在途Editor settings测试与Platform/App生命周期源码漂移。
