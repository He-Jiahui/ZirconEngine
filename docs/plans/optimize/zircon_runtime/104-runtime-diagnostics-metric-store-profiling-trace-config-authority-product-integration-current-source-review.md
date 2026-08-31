---
title: Runtime Diagnostics、Metric Store、Profiling Trace、Config Authority 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime156
review_date: 2026-08-29
baseline_head: 8aabbee3e99dc919f6da4611e3a44e8463a7fe7f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_incomplete
source_recheck_required: true
related_code:
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/runtime_diagnostics
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/foundation/runtime/config_manager
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime_interface/src/profiling.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md
  - docs/plans/optimize/zircon_editor/147-editor-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-current-source-review.md
reference_engines:
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CpuProfilerTrace.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CountersTrace.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
  - dev/godot/main/performance.h
  - dev/godot/core/config/project_settings.h
  - dev/Fyrox/fyrox-impl/src/renderer/stats.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs
---

# Runtime Diagnostics、Metric Store、Profiling Trace、Config Authority 与 Product Integration 当前源码工程化差距

## 1. 结论

Runtime03 识别的基础组件仍然真实存在，而且部分实现已经明显推进：diagnostic store 同时提供 full/current snapshot；render stats projection 已拆分为多文件；profiler 增加了 retention 计数、有限 capture epoch、动态名称 inactive gate、UI/counter hotspot 和更严格的 session basename；dynamic ABI 对 profile response 施加 16 MiB、65,536 items、250 ms 的输出限制；ConfigManager 已有 dirty generation、debounce worker、backup recovery、atomic replace、commit fence、flush 和 persistence report。

这些进展没有把系统提升为工程级观测和配置 authority。当前最严重的结构问题不是“少几个字段”，而是 owner、时间和事务边界不成立：

- CPU recorder 仍是进程全局单 mutex，所有 Runtime session 共用 start/stop/reset/export 状态，sample 没有真实 process/thread/capture generation。
- 普通 scope/frame token 没有 capture epoch；旧 token 可以在 reset/restart 后写入新的 recorder generation。
- diagnostic collector 不再写临时 clone，但现在每次查询都会向权威 store 再写约 690 个 render metric。历史、EMA 和 current 值由观察者频率驱动，缺少观察者时又不更新。
- metric path 仍可由任意字符串隐式创建，只有单 series history 上限，没有 descriptor、owner、全局 cardinality、metadata bytes 或 snapshot bytes budget。
- runtime diagnostics 是分时读取多个 manager、再更新 store、再 clone profile 的弱一致组合，却只暴露单一 `frame_index`，没有 collection window、provider generation、stale age、partial/completeness 或 page cursor。
- ConfigManager 的 durable 写入与 `CoreHandle::store_config*` 的内存写入继续并存，App、Editor、builtin Animation、Animation plugin、Physics plugin 都在生产路径绕开 dirty generation。
- 配置文件仍是全局 raw JSON map；多个 in-process Runtime 默认指向同一 OS-user 文件，而 process-static commit fence 会让后创建 manager supersede 旧 manager。没有 project/user/session layer、typed key、migration transaction、跨进程 revision/CAS 或 watch。
- 产品关闭路径没有显式 `ConfigManager::flush` 和 `persistence_report` acceptance；当前依赖 Drop 的两秒尝试。worker/DLL unload 的 P0 继续由 Runtime02 唯一拥有，本篇不重复计数。

本报告新增 **0 项 P0、12 项 P1、6 项 P2**。30 项资格门当前为 **22 Fail / 4 Partial / 4 Pass**。Runtime03 的旧 P1-4“写入临时 clone”按原命题关闭，但被 Runtime156-P1-005 的 observer-driven authoritative mutation 取代；session basename 的 `..` 问题已关闭，artifact publish 仍未关闭。没有动态 benchmark、产品 trace 和同机参考基线时，不作“性能优于 Unreal/Unity”等结论。

## 2. 冻结范围、currentness 与方法

### 2.1 物理选择集

fingerprint 口径为 lower-case repo-relative path、文件 SHA-256，按路径排序后以 `path<TAB>hash` 和 LF 拼接，再计算 SHA-256。

| 范围 | files / lines / nonempty / bytes / tests / ignored | tracked / modified / untracked / dirty | fingerprint |
|---|---:|---:|---|
| Zircon diagnostics/profiling/config + facade/ABI/product consumers | **121 / 22,053 / 20,573 / 738,657 / 140 / 6** | **84 / 41 / 37 / 78** | `cbb5497a644398afce4a5060d32a257c888348810711f10805d598b559f857fa` |
| Unreal/Bevy/Godot/Fyrox/Unity Graphics reference selection | **13 / 8,685 / 7,499 / 344,216 / 1 / 0** | n/a | `490d5d48aa43a445c83a21c91f1492a69f66d93be92cd1f5d7d95e1085ea5ac4` |

Zircon 精确选择规则：

- 递归目录：`zircon_runtime/src/core/runtime/diagnostics/**/*.rs`、`zircon_runtime/src/runtime_diagnostics/**/*.rs`、`zircon_runtime/src/foundation/runtime/config_manager/**/*.rs`。
- Core/config facade：`handle/diagnostics.rs`、`handle/events.rs`、`config_store.rs`、`runtime.rs`、framework `config_manager*.rs`、foundation `config_manager.rs`、`config_manager_tests.rs`、`config_path.rs`。
- ABI/session/log：dynamic session `profile.rs/diagnostics.rs/ffi.rs/state.rs`、`dynamic_api/frame.rs`、`diagnostic_log/diagnostics.rs`、Runtime Interface `profiling.rs/buffer.rs`、Runtime Host `foreign_output/item_count.rs/state.rs`。
- 产品调用点：App `runtime_product_diagnostics.rs/engine_entry.rs`、Editor `runtime_services.rs` 与 retained-host profiling/workbench projection、builtin/plugin Animation、plugin Physics settings。

13 个 reference 文件精确为 Bevy `diagnostic.rs/log_diagnostics_plugin.rs`；Unreal `CpuProfilerTrace.h/CountersTrace.h/ConfigCacheIni.h`；Godot `performance.h/.cpp/project_settings.h/.cpp`；Fyrox renderer `stats.rs/settings.rs`；Unity Graphics `ProfilingScope.cs/DebugManager.cs`。

当前 HEAD 为 `8aabbee3e99dc919f6da4611e3a44e8463a7fe7f`，选择集 121 个文件中 78 个 dirty，37 个尚未跟踪；profiling、render stats、config manager、dynamic ABI 和产品消费者都在变更。本文审查 current worktree，而不是声称 clean checkout 已具同等能力；实施 M0 必须重取 manifest、fingerprint 和 caller inventory。Tooling 按用户要求排除；本轮未查询、轮询、等待或实时跟踪协调器。

### 2.2 动态证据边界

本轮是 review-only，没有运行 Cargo、产品进程、Miri、loom、fuzz、fault injection、磁盘故障、跨进程 config、Tracy/WPR、GPU capture、soak 或动态 benchmark。静态源码可以证明公开入口、锁和 generation 缺失、查询写入、同步 I/O、bounded response 和生产调用链；不能单独证明实际 ns/event、p95 hitch、磁盘 durability、跨平台原子性或相对参考引擎性能。

6 个 ignored test 属于当前源码选择集，但 ignored/固定阈值 characterization 不能替代受管性能资格。后续性能结论必须冻结 BuildSet、硬件、OS、驱动、capture profile、raw sample、噪声区间和 source fingerprint。

## 3. 当前可保留的工程基础

| 能力 | 当前证据 | 保留条件 |
|---|---|---|
| bounded per-series history | `DiagnosticSeries` 使用 `VecDeque`，默认 history 64；提供 full/current snapshot | 升级为 descriptor/owner/global-byte budget，不能把单 series 上限冒充全局有界 |
| static metric fast path | `record_static` 在 metadata 匹配时避免重复 metadata 合并 | static path 必须解析为注册后的 dense `MetricId`，collision fail-closed |
| render metric projection | render stats store 已拆分 domain 并使用静态路径；当前为 890 次 literal 出现、690 个 unique `render.*` path | producer 在 frame commit 时发布一次 sealed generation，query 不得再次生产数据 |
| current snapshot | logger 可以只取 current，而非始终复制全部 history | 增加 filter/if-newer/page/byte budget，且 current 带 age/generation/availability |
| profile feature/inactive gate | feature/capture active check、static macro 和 dynamic-name handoff test 已存在 | 用真实 benchmark证明 inactive/active 开销；dynamic tracing backend也必须纳入 |
| limited capture epoch | `CAPTURE_STATE` 包含 active bit 与 wrapping epoch，IBL async producer使用 completion epoch | 所有 scope/frame/counter/async token统一携带 generation，不允许局部接线 |
| retention counters | frames/spans/counters各报告 capacity/written/overwritten/retained/sequence | 建立跨 ring 一致 chunk、gap/completeness/bytes和relation validity |
| export basename | session basename已限制长度、处理分隔符/保留名/碰撞，旧 `..` 路径问题已修复 | output root authority、staging、manifest、hash和atomic publish仍必须补齐 |
| bounded foreign output | profile response限制16 MiB、65,536 items、250 ms；Host验证bytes/items/depth/time并可fuse session | 增加page/filter/cursor/partial，避免超限时整份结果失败 |
| config persistence worker | dirty generation、debounce、backup recovery、atomic writer、commit fence、flush/report | 所有persistent caller走唯一typed transaction，产品shutdown消费durability结果 |
| Editor/product consumers | Editor profiling projection和App product diagnostics已消费真实Runtime snapshot | consumer必须显示generation/stale/partial/drop，不得从弱一致数据推导“健康” |

## 4. 当前 owner、写入和观察链

```text
Render/Physics/Animation managers
  `- runtime diagnostics query
       |- resolve/read providers at different instants
       |- lock DiagnosticStore
       |- record ~690 render paths again (+ domain metrics)
       |- clone current/full store snapshot
       `- clone process-global ProfileSnapshot
            `- ABI / Editor / 1 Hz diagnostic logger

profile_scope!/profile_frame!/profile_counter!
  `- process-global Mutex<ProfileRecorder>
       |- all Runtime sessions share control state
       |- frames/spans/counters evict independently
       `- synchronous export -> caller-selected output_root/files

CoreHandle::store_config* ----------------> in-memory ConfigStore only
ConfigManager::set_value -> dirty generation -> worker -> whole JSON file
  `- default OS-user path shared by all Runtime instances
       `- newest in-process path gate epoch supersedes older manager
```

目标不是把所有观测数据塞入另一把更大的锁。目标是四个显式 authority：

1. `MetricCatalog + MetricPublicationService` 拥有 descriptor、producer lease、budget和sealed generation。
2. `TraceSessionService` 拥有 runtime/session/capture identity，worker只写per-thread bounded chunks。
3. `RuntimeObservationService` 按声明的一致性级别组合 provider generation，并提供filter/page/delta。
4. `ConfigService` 拥有typed registry、layers、transaction、revision、durability和migration；raw session override与persistent config物理分离。

## 5. Runtime03 既有 finding 当前状态

| Existing ID | 当前状态 | current-source 复核 |
|---|---|---|
| Runtime03-P1-1 | Open | recorder仍是process-global mutex；thread/process/session identity仍缺失 |
| Runtime03-P1-2 | Partial | `CAPTURE_STATE` epoch已加入且局部async producer消费；普通scope/frame token仍不携带epoch |
| Runtime03-P1-3 | Open | string path隐式创建、metadata覆盖/合并、全局cardinality/bytes/owner budget仍缺失 |
| Runtime03-P1-4 | Closed as written / superseded | collector已不再写临时clone；现在query写权威store，转由Runtime156-P1-005拥有 |
| Runtime03-P1-5 | Open | `CoreHandle::store_config*`旁路仍公开且生产调用更多，不只Animation |
| Runtime03-P1-6 | Open | raw JSON map仍无schema/layer/migration/change transaction |
| Runtime03-P1-7 | Partial | session basename traversal/collision已修；output root authority与atomic artifact publish仍缺失 |
| Runtime03-P2-1 | Partial | retention counters已加；三个ring仍独立evict，snapshot无capture completeness |
| Runtime03-P2-2 | Open | measurement仍只有frame index/f64，固定样本EMA与lifetime min/max未变 |
| Runtime03-P2-3 | Partial | current snapshot减少logger的history clone；690-series query写入/格式化和full ABI clone仍存在 |
| Runtime03-P2-4 | Open | devtools backend availability/count仍硬编码，provider快照仍无共同revision |
| Runtime03-P2-5 | Open | poison统一`into_inner`，没有mutation-stage invariant fault proof |

## 6. P0：本报告不新增独立 P0

Config worker 在shutdown timeout后无法证明线程退出、进而影响动态库安全的问题继续由 Runtime02 的execution/DLL-unload P0唯一拥有。Runtime156-P1-012只拥有“产品是否显式要求durability并消费结果”的配置契约，不重复登记thread join P0。Runtime153的source/Cargo currentness P0、Interface11当前Host compile failure、Render17 GPU timing和Editor147 UI产品问题也继续由各自报告计数。

## 7. P1：工程级 authority 前必须关闭

### Runtime156-P1-001：process-global recorder单锁同时破坏多线程真实性与多Runtime session隔离

`GLOBAL_RECORDER: OnceLock<Mutex<ProfileRecorder>>` 是进程静态对象。dynamic session 的 Start/Stop/Reset/Snapshot/Export最后都路由到它；session handle只证明请求来自一个存活session，不会把capture state分区。任意session可停止、清空或导出其他session正在产生的数据。

所有active begin/end/counter也竞争同一mutex。`SPAN_STACK`虽然thread-local，DTO却没有真实process/thread/task identity；Perfetto固定`pid: 1`并把逻辑`stream`直接作为`tid`。多个worker使用相同stream时会被合并到一条track，锁本身又串行化被测事件。

目标是`TraceSessionId { runtime_session, capture_generation }`、静态callsite catalog和per-OS-thread writer。线程写有界chunk/ring，控制面只seal/swap generation，后台assembler合并。session控制必须验证owner capability；snapshot/export只读取自己已seal generation。验收需覆盖1/8/64 producer、多Runtime、同stream跨线程、session并发start/stop/reset/export与teardown。

### Runtime156-P1-002：capture epoch只接到局部async producer，普通scope/frame可跨reset污染新capture

`CAPTURE_STATE`已经把active bit和epoch放入atomic，real-time IBL async producer也会在completion时比较epoch，这是正确方向。但`ProfileScopeToken`和`ProfileFrameToken`只保存id/path/start/frame等数据，没有epoch/session/thread。`finish_scope`先按当前recorder origin取`end_us`，再向当前recorder写sample；`finish_frame`同样如此。

因此 begin -> stop/reset/start -> drop 可把旧token写入新generation，产生重复id、错误duration、悬空parent/frame。epoch以`wrapping_add`推进，也没有exhaustion policy。目标是每个token携带session/generation/writer sequence；finish不匹配时只增加`dropped_stale_token`，不能触碰新ring。stop进入sealing状态并定义outstanding-token deadline/truncation；reset不得复用identity。加入控制命令与drop全排列、跨线程handoff和近epoch耗尽model test。

### Runtime156-P1-003：capture config没有不可突破的entry/byte/time上限，bounded request并未限制recorder保留内存

`ProfileCaptureConfig::normalized`只把0容量替换成默认值，并校验frame budget finite/positive；`max_frames/max_spans/max_counters`可取`usize::MAX`。ABI request body即使有字节上限，也可以用很短JSON声明巨大容量。recorder不会立即preallocate，但长期capture可以持续增长到该上限；每个sample又持有多个String，entry count不能代表retained bytes。

目标建立版本化`ProfileCapturePolicy`，由产品profile决定不可提升的entry/metadata/retained-byte/duration/chunk/snapshot/export budgets。请求只可在policy内降低；返回requested/effective limits。writer在分配前reserve budget，超限产生typed drop/gap并保持进程可用。fuzz/scale覆盖`usize::MAX`、极长name/path/tag、高cardinality、长capture和snapshot/export同时进行。

### Runtime156-P1-004：MetricStore没有descriptor、producer owner和全局cardinality/byte budget

`DiagnosticPath(String)`不验证grammar；`record`对任意path自动创建series。unit可被后来写入覆盖，tag持续合并；没有同path不同kind/unit/owner的collision error。每条history虽有限，但path数量、tag/string bytes、store bytes和snapshot bytes没有总上限。当前render projection已含690个unique静态path，未来把entity/resource/viewport名字拼进path会永久放大store。

目标是模块activation前注册`MetricDescriptor { MetricId, path, kind, value_type, unit, aggregation, retention, cardinality, owner, privacy }`，热路径只持dense id/producer lease。动态维度必须是有界label或event payload，不得扩展path；registry按全局、owner、kind限制series/labels/metadata/history bytes。未知id、descriptor collision、owner卸载后写入、budget exhaustion都返回typed receipt并发布accepted/dropped计数。

### Runtime156-P1-005：diagnostics query正在生产权威历史，统计值由观察者频率而不是producer cadence决定

`collect_diagnostic_store_snapshot`现在在`core.update_diagnostic_store`闭包内调用`record_diagnostic_domains`，随后`store.snapshot()`；current collector也先写再读。一次查询会重投影约690个render path，并写physics/animation。相同render frame可按UI/logger/ABI的查询次数重复写相同frame index；没有观察者时这些metric完全不写。physics/animation当前投影使用固定frame index 0。

optional值缺失时collector只是“不record”，旧current值会无限保留，没有unavailable/stale/age。查询还在store mutex内完成全部写入和snapshot clone。现有测试断言query之后authoritative history增长，说明这不是偶然caller。

目标只允许producer在domain frame/transaction commit时发布一次immutable `MetricGeneration`；query必须纯读。重复generation拒绝或幂等，缺值发布availability/stale reason。兼容迁移期也要把point-in-time provider snapshot标成无history，不能伪造EMA/min/max。RED覆盖0/1/2 observer、同frame重复query、provider消失、frame倒退和690-path contention。

### Runtime156-P1-006：RuntimeDiagnosticsSnapshot没有collection window、provider generation或一致性等级

collector先分别resolve/read render、physics、animation，再更新metric store，再读取global profile；dynamic session随后又读取real frame、session/input/reload状态。期间各owner可独立推进或卸载。公开snapshot却只有一个`frame_index`和三个domain availability status；status不含observed generation、age、collection error taxonomy或last-good。

目标由`RuntimeObservationRequest`声明`BestEffort/FrameCoherent/SealedCapture`等级，顶层返回collection start/end monotonic time、runtime/session/build identity和每个provider的`generation/observed_at/age/status/error`。BestEffort允许不同generation但必须显式；FrameCoherent通过frame publication barrier选择同一sealed frame；provider失败返回partial receipt，不用旧值伪装current。

### Runtime156-P1-007：diagnostics/profile ABI是单块深snapshot，没有filter、if-newer、page或continuation

profile response的16 MiB、65,536 items、250 ms限制和Host fail-closed验证是真实保护，但数据仍先被构造成完整Rust snapshot/JSON，再交给foreign output检查。`DiagnosticStoreSnapshot`包含全部series/tags/history，`ProfileSnapshot`包含全部frames/spans/counters。没有path/domain filter、summary/detail level、`if_newer_generation`、cursor、page completeness或取消。

超过输出限制只能使整次请求失败；consumer不能从已知generation继续，也不能区分“没有变化”和“无法装入”。目标先在owner侧规划bounded query，再materialize一页：`QueryPlan -> Page { generation, cursor, items, bytes, complete, dropped }`。generation在分页期间不可变；过期cursor返回typed resync。health summary保持固定小体积，history/trace按订阅或capture读取。

### Runtime156-P1-008：persistent config有双写入口，生产路径存在“内存成功、重启丢失”

`ConfigManager::set_value`才会比较值并调用`request_persistence(changed)`；`CoreHandle::store_config_value/store_config`只写`ConfigStore`。当前App startup、Editor runtime services、builtin Animation、Animation plugin和Physics plugin均直接走CoreHandle旁路。调用后同进程load立即成功，容易让测试误判持久化完成，但dirty generation没有推进。

目标建立唯一`ConfigService::transaction/set(ConfigKey<T>, value, durability)`入口。session-only override使用不同类型/API/store；persistent key不能经CoreHandle raw写入。逐caller硬切后删除公开旁路，不保留compatibility shim。每个现有生产setting必须通过save -> explicit flush -> destroy -> recreate -> reload roundtrip验收，并检查durability revision而非只读内存值。

### Runtime156-P1-009：raw JSON map没有typed schema、layer、migration、revision transaction或secret policy

`ConfigStore`是`HashMap<String, Arc<Value>>`；同一key可在不同调用点反序列化为不同Rust类型。文件没有format/key version、default/source layer、validator、restart/live-apply、deprecation、unknown-key、secret或PII policy。malformed whole file可使manager activation失败，坏单key与文件损坏没有隔离/migration report。whole-map snapshot在每次flush时clone、pretty serialize并重写。

目标`ConfigRegistry`声明`ConfigKey<T> { stable id, schema version, scope, default, validator, persistence target, restart policy, sensitivity, migration }`；layers固定为built-in -> engine/project -> user/editor -> command/env -> session。多key transaction先校验、再原子发布revision，observer在锁外收到typed delta。加载返回unknown/deprecated/invalid/migrated/quarantined report；secret不可进入普通diagnostics/export。

### Runtime156-P1-010：默认全局配置路径和process-static commit fence不能支撑多Runtime或多进程authority

默认路径是OS-user级`ZirconEngine/config.json`，也可由`ZIRCON_CONFIG_PATH`替换；每个Runtime构造自己的manager却指向同一文件。`PATH_COMMIT_GATES`是process-static path map，新manager注册会推进path epoch，使旧manager的后续commit失效。这防止旧worker覆盖新owner，但意味着多个并存Runtime不是独立owner，也没有显式shared authority。

跨进程没有OS lock、base revision/CAS、watch、merge或conflict artifact；另一个Editor/Runtime可覆盖整张map。目标先按scope解析明确文件：engine/project、user/editor、product/session，并返回path/provenance/revision。进程内同path共享一个service或显式拒绝第二owner；跨进程使用lock + optimistic revision + atomic transaction + watch/resync。Preference通用后端细节由Runtime45拥有，本篇只负责Runtime config接入和多session contract。

### Runtime156-P1-011：profile export同步直写调用方路径，没有staging、manifest或generation完整性

export在调用线程`create_dir_all`后顺序写native timeline、Perfetto、hotspot、counter/UI hotspot和summary。中途失败会留下新旧混合目录；本次关闭Perfetto时旧文件可继续存在。snapshot/export artifact没有schema version、capture id/generation、BuildSet/source fingerprint、clock/thread metadata、completeness和per-file hash。`output_root`仍是请求提供的任意可写路径；修复session basename不能限制root authority。

目标由Host授予opaque `ProfileArtifactTarget`，Runtime不能接受任意filesystem root。sealed generation交给有界后台I/O lane，在同父目录staging写全部artifact和versioned manifest，close/sync后atomic publish；失败只保留旧完整版本或隔离staging。manifest固定capture/build/schema/clock/provider generations、counts、drop/gap/truncation、hash。取消、磁盘满、权限、stale file和crash点都要故障注入。

### Runtime156-P1-012：产品shutdown没有显式config durability gate，persistence report没有生产consumer

ConfigManager公开`flush(timeout)`和`persistence_report()`，但聚焦生产调用点没有配置flush/report消费；产品依赖manager Drop的默认2秒尝试。Drop是否返回不等于requested generation已durable，产品也不能把失败映射为retry、block close、safe-to-unload或user-visible degraded state。

目标让session/editor/project close携带`ConfigDurabilityPolicy`，显式请求target revision、等待bounded flush并消费`ConfigPersistenceReport`。成功teardown receipt必须记录durable generation和worker termination evidence；超时/失败按产品策略阻止卸载、允许用户重试或保留recovery artifact。worker join和DLL unload安全继续由Runtime02验收，本篇只提供config-owned blocker/receipt。

## 8. P2：语义、可诊断性与长期维护差距

### Runtime156-P2-001：三个独立profile ring会产生悬空关系，retention计数不足以证明capture完整

frames、spans、counters分别按entry count `pop_front`。frame可能先被淘汰而span仍引用它，parent也可能先于child消失；retention只说明各ring written/overwritten/retained/sequence，不说明relation validity、retained bytes、capture completeness或gap原因。目标以sealed chunk/generation为一致单位，保留显式gap/drop event和跨表referential integrity；page/manifest声明完整或缺失范围。

### Runtime156-P2-002：DiagnosticSeries没有monotonic time、finite/ordering和明确统计窗口

measurement只有`frame_index + f64`。EMA固定按sample执行`previous * 0.9 + value * 0.1`，1 Hz和1 kHz producer有不同时间响应；min/max是lifetime值，history淘汰不重算，也无reset/window epoch。NaN/Inf、duplicate/regressing frame index没有policy。目标按metric kind定义timestamp、finite/ordering、time-based smoothing、window/lifetime统计、reset epoch和histogram/sketch。

### Runtime156-P2-003：所有counter被压成f64，hotspot会丢弃0和负值

metric/profile counter没有Gauge/MonotonicCounter/Delta/Memory/Duration/Histogram/Event kind。`counter_hotspot`直接跳过non-finite和`<= 0.0`，因此合法的signed delta、温度/位置/偏差gauge、归零和负值都不进入热点报告。目标使用typed descriptor和aggregation；hotspot按kind分别处理last/min/max/rate/sum/distribution，不能把“正值累加”应用于所有counter。

### Runtime156-P2-004：devtools backend状态硬编码且registry快照没有共同revision

`project_runtime_devtools_snapshot`固定native_dynamic available=true/count=0、VM available=false/count=0，而不是读取backend/plugin owner。module、service、plugin catalog又在不同锁和时刻读取。目标由owner发布versioned capability/provider snapshot，未知显示Unknown/Unavailable reason；顶层记录各provider generation与collection window，禁止硬编码成功。

### Runtime156-P2-005：统一poison recovery只证明进程继续，不证明状态仍满足不变量

diagnostic store、global recorder、config store/state、commit fence和devtools多处使用`poisoned.into_inner()`。现有测试通常在关键mutation前panic，再断言还能读写；没有在ring eviction、partial metric metadata、dirty/attempt generation、atomic writer阶段注入panic并验证守恒。目标按状态分类：派生cache可丢弃重建；trace generation标damaged并seal；persistence/control state必须校验或fail-closed。故障注入覆盖mutation每个阶段。

### Runtime156-P2-006：ID、sequence和generation的saturating/wrapping策略会静默重复身份

profile span/frame/retention计数使用`saturating_add`，capture epoch和path commit epoch存在wrapping推进。到边界后可能重复id/generation或永久冻结sequence，却没有terminal error、rollover generation或restart requirement。虽然现实运行很难抵达，identity contract仍必须定义。目标用小位宽model test验证near-exhaustion；分配失败返回typed exhausted，或以更高层new owner epoch安全rollover，禁止静默复用。

## 9. 参考引擎证据与适用边界

| 参考 | 已核对机制 | Zircon应吸收 | 不应照搬/证据边界 |
|---|---|---|---|
| Bevy diagnostics | `Diagnostic`先注册；measurement带`Instant`；time-based EMA；per-metric enabled/history；SystemParam只在enabled时求值并deferred写入 | descriptor registration、timestamp、disabled fast path、producer-side deferred batch、time-aware smoothing | Bevy store不是多session高频trace或durable config方案 |
| Unreal CPU trace | static spec比dynamic name低成本；matched begin/end、channel gate和线程事件前端 | static callsite id、per-thread buffer、channel gate、flush/seal、真实thread track | Unreal常驻全局trace infrastructure不能替代Zircon dynamic session/unload ownership |
| Unreal counters | int/float类型、memory display hint、static/dynamic counter和set/add语义 | typed counter、display/aggregation metadata、static registration、enabled gate | 不复制宏/API表面；需要Rust typed descriptor和budget receipt |
| Unreal config | `FConfigCacheIni/FConfigFile`区分文件、section、set/flush及hierarchical load机制 | layered source、明确file owner、flush boundary和变更追踪 | 大型legacy INI/全局cache并非Zircon目标；schema/migration仍需更强约束 |
| Godot performance | built-in monitor enum声明quantity/memory/time/percentage；custom monitor有duplicate rejection、remove和modification time | stable built-in identity、kind/unit、受控扩展、catalog revision | singleton/Variant不适合作为内部热路径 |
| Godot project settings | PropertyInfo、initial value、restart-if-changed、feature override、changed settings、save/load | key metadata、override来源、restart policy、changed set、持久化与运行时projection分离 | 不复制单例和动态Variant类型系统 |
| Fyrox renderer | producer-owned begin/end/finalize frame统计和typed renderer settings | producer在frame owner处commit统计、typed settings/reflect底座 | 只是renderer局部stats/settings，不是通用trace/config authority |
| Unity Graphics | reusable `ProfilingSampler`、CPU inline与CommandBuffer/GPU scope、Dispose配对、DebugManager panel注册/注销 | static sampler、CPU/GPU marker关联、scope lifetime、受控debug consumer注册 | 本地Graphics包不是完整Unity player/editor/config源码，只作profiling/debug窄参考 |

这些参考共同证明“工程级”依赖注册、owner、时间、generation、budget和failure receipt，而不是增加更多字符串metric或导出格式。它们也不能替代Zircon自己的多Runtime、动态库、Rust类型系统和产品SLO设计。

## 10. 目标架构与硬切边界

```text
Module activation
  -> MetricCatalog.register(descriptors, owner, budgets)
  -> TraceCallsiteCatalog.register(static callsites)
  -> ConfigRegistry.register(typed keys, layers, migrations)

Runtime frame/domain commit
  -> MetricProducerLease.publish(MetricBatch, provider_generation)
  -> MetricPublicationService.seal(RuntimeObservationGeneration)
  -> per-thread TraceWriter.append(events, capture_generation)

RuntimeObservationService.query(request)
  -> select sealed generation / consistency level
  -> filter + if-newer + bounded page
  -> provider status/age/drop/completeness

TraceSessionService.stop()
  -> seal writer chunks -> assemble immutable capture
  -> background ArtifactPublisher(staging + manifest + atomic publish)

ConfigService.transaction()
  -> validate typed keys/layers/base revision
  -> publish immutable ConfigRevision + typed delta
  -> PersistenceCoordinator durable commit + report
  -> product shutdown consumes durability blocker/receipt
```

硬切规则：

- 删除query-time `record_diagnostic_domains`，不得保留“为了history兼容”的观察者写入。
- 所有persistent caller迁出`CoreHandle::store_config*`后删除旁路；session override使用不同API。
- `GLOBAL_RECORDER`不能作为多session兼容代理；迁移完成后每个trace control必须携带owner/capture id。
- 新ABI只从bounded query plan materialize page；旧full snapshot在App/Editor/Host caller全部迁移后删除，不长期双轨。
- exporter只接受Host授予的artifact target，不接受自由filesystem root。

## 11. 依赖顺序与实施里程碑

| Milestone | 先写失败证据 | 实现边界 | 完成定义 |
|---|---|---|---|
| M0 owner/currentness freeze | 重取121-file manifest、caller/metric/config key inventory、旧finding status tests | 只冻结schema/owner/预算/SLO，不改producer | current BuildSet、owner map、RED tests和source fingerprint可复现 |
| M1 MetricCatalog | duplicate descriptor、dynamic cardinality、owner unload、metadata/byte exhaustion | descriptor registry、dense id、producer lease、budgets | 未注册/冲突/超预算fail-closed且accepted+dropped守恒 |
| M2 metric publication/query | 0/1/2 observer、duplicate frame、provider missing/stale、page cursor过期 | producer commit、sealed generation、filter/if-newer/page | query纯读；观察频率不改变history；partial/stale显式 |
| M3 TraceSession | 多Runtime互相stop/reset、同stream跨线程、old token跨generation | per-session control、per-thread writer、token generation、seal | track/thread/session真实；旧token只计drop；inactive/active有量化预算 |
| M4 trace retention/artifact | ring relation eviction、huge config、disk full/crash/stale files/cancel | chunk budget、capture manifest、background atomic publisher | capture completeness可证明；目录只出现旧完整或新完整generation |
| M5 ConfigRegistry | type collision、layer precedence、bad key、migration rollback、secret export | typed keys/layers/transaction/revision/delta | 多key原子publish；unknown/deprecated/invalid/migrated可查询；secret隔离 |
| M6 persistence authority | 两Runtime同path、两进程CAS、watch/resync、shutdown deadline | scoped stores、single in-process owner、cross-process revision/lock、durability blocker | save/restart稳定；conflict不覆盖；成功teardown有durable revision证据 |
| M7 consumer hard cut | App/Editor/logger/Host/Animation/Physics旧API负扫描 | 新ABI/page、product status、caller迁移、旧API删除 | 无query写metric、无persistent旁路、无global recorder/full-snapshot兼容路由 |
| M8 qualification | 1/8/64 thread、690/10K metric、30/60/120 Hz observer、long capture/config fault | Windows首选产品矩阵+必要跨平台lane | 30门全Pass；raw evidence冻结；没有同条件数据不宣称超越参考引擎 |

M1/M2必须先让数据语义可信，M3/M4再让trace证据可信；在此之前不能使用当前observer-mutated metric或global recorder证明其他性能重构达标。Config M5/M6可以与M1-M4并行设计，但M7产品硬切必须等待两边稳定contract。

## 12. 资格门

| Gate | 当前 | 必须提交的证据 |
|---|---|---|
| G01 metric descriptor预注册且stable id/version | Fail | catalog manifest、duplicate/version tests |
| G02 descriptor kind/unit/owner collision fail-closed | Fail | activation RED/green与typed receipt |
| G03 global/owner cardinality与metadata/history byte budget | Fail | exhaustion/conservation/scale tests |
| G04 metric仅由producer commit，query纯读 | Fail | 0/1/2 observer等价history测试 |
| G05 timestamp、finite、duplicate/regressing generation policy | Fail | invalid/ordering/property tests |
| G06 stale/unavailable/drop/gap/completeness可观察 | Fail | provider loss/late sample/capacity tests |
| G07 filter/if-newer/page/cursor/resync | Fail | bounded query与cursor expiry tests |
| G08 snapshot声明并满足consistency level | Fail | concurrent provider update/barrier tests |
| G09 provider partial失败不伪装current | Fail | mixed success/failure snapshot tests |
| G10 profile ABI byte/item/time输出上限 | Pass | 当前16 MiB/65,536/250 ms常量与Host validator |
| G11 owner-side materialization也在同一预算内 | Partial | foreign output有界；完整Rust snapshot仍先构造 |
| G12 多Runtime/session capture隔离 | Fail | parallel session control tests |
| G13 真实process/thread/task track identity | Fail | 1/8/64 thread Perfetto/native trace验证 |
| G14 static marker与inactive gate | Partial | static宏/gate存在；缺完整backend开销与allocation证据 |
| G15 所有token带capture generation且late drop | Fail | reset/restart/drop全排列tests |
| G16 capture不可突破entry/byte/duration上限 | Fail | requested/effective policy与huge-input tests |
| G17 retention/drop sequence证据 | Partial | per-ring计数存在；跨ring完整性/gap/bytes缺失 |
| G18 session basename不可逃逸/碰撞且有长度界 | Pass | 当前session_path tests |
| G19 export staging/manifest/hash/atomic publish | Fail | crash matrix与artifact verification |
| G20 export后台有界、可取消且不阻塞frame/session锁 | Fail | thread/lock trace、deadline/cancel tests |
| G21 config atomic replace/backup/dirty generation | Pass | 当前writer/worker/backup基础 |
| G22 config explicit flush与persistence report API | Pass | trait/manager已有接口；产品消费另见G27 |
| G23 typed config schema/validator/restart/secret | Fail | registry manifest与negative tests |
| G24 layers/migration/unknown-key quarantine | Fail | version matrix与rollback tests |
| G25 persistent config唯一写authority | Fail | caller负扫描与save/restart roundtrip |
| G26 多Runtime/多进程revision/CAS/watch | Fail | controlled process conflict/resync tests |
| G27 product shutdown消费durability blocker/report | Fail | close/unload receipts与failure UI/policy |
| G28 poison/fault mutation-stage invariant证明 | Fail | panic/disk/crash injection matrix |
| G29 observer开销与trace开销受管benchmark | Fail | off/on、thread/series/rate矩阵及raw samples |
| G30 App/Editor/Host真实consumer已接入 | Partial | 已有snapshot/UI消费；缺generation/page/stale/drop contract |

当前结果：**22 Fail / 4 Partial / 4 Pass**。任何“增加字段”“新增test文件”或单次产品截图都不能下调门状态；必须同时证明owner调用链、失败路径、boundedness和产品consumer。

## 13. 性能与故障证据矩阵

### 13.1 Metric/observation

- producers 1/8/64；series 1/690/10K；history 1/64/1K；consumer 0/1/30/60/120 Hz。
- 记录write ns/sample、batch size、allocations、lock wait、retained/metadata/snapshot bytes、accepted/dropped和frame p50/p95/p99。
- 比较0个与多个observer时同一输入的history/generation digest，必须相同。
- provider unload/reload、generation倒退、duplicate commit、stale cursor和超budget必须有typed结果。

### 13.2 Trace

- inactive/active，static/dynamic name，1/8/64 threads，0/100/10K events/frame；记录ns/event、allocations、contention、drop rate和被测workload扰动。
- start/stop/reset/export与scope/frame/counter完成全排列；old token、session teardown、thread exit和capture epoch近耗尽。
- capture 1 s/60 s/1 h，tiny/default/max policy，metadata flood；snapshot/export并发和取消。
- native/Perfetto artifact验证thread/process/capture/parent/frame identity与manifest hash。

### 13.3 Config/artifact

- built-in/project/user/command/session precedence；unknown/deprecated/invalid/migrated/secret；multi-key rollback与observer reentrancy。
- 1/1K key burst、同值、失败后同值重试、1 B/1 MiB/100 MiB snapshot、whole-map rewrite成本和coalescing ratio。
- 两Runtime同path、两个进程CAS、watch/resync、磁盘满、权限、rename/sync失败、backup corrupt、crash在每个publish点。
- shutdown在snapshot/serialize/write/replace各阶段卡住；成功destroy必须证明target revision durable且worker已按Runtime02要求退出。

## 14. Canonical owner 与非目标

1. Runtime44唯一拥有process diagnostic log router/filter/queue/sink/rotation/crash flush；本篇只拥有metric query和logger订阅/预算接口。
2. Runtime45拥有Preference scope/storage/overlay/bounded I/O/multi-process通用后端；本篇拥有Runtime ConfigRegistry、caller hard cut和config durability product contract。
3. Runtime02拥有execution worker、shutdown coordinator和DLL unload P0；本篇只交付trace/config blocker与receipt。
4. Runtime Interface04拥有profiling/diagnostic公共DTO和crate归属；本篇定义其必须承载的generation/page/completeness语义，不重复其crate P0。
5. Editor147拥有Performance Timeline/Console/Telemetry工作台产品；本篇只提供可信Runtime数据源、分页和状态合同。
6. Render17及各render current-source报告拥有GPU timestamp、pass timing和具体690个render metric正确性；本篇只拥有共享metric publication/store/query和CPU trace。
7. Tooling按用户要求不在本轮优化；未来迁移到Rust时也必须消费同一bounded observation/config contract，不能获得旁路。

## 15. 工作区复核与产出边界

本轮只新增review、分类index、根index和coverage记录，没有修改production、tests、Cargo、ABI或配置文件，也没有运行动态验证。当前121-file选择集中78个dirty且37个untracked，报告只能冻结2026-08-29 current worktree的审查证据，不能替代合并后的source recheck。

实施M0必须先重读`store.rs`、`runtime_diagnostics/collect.rs`、profiling全子树、ConfigStore/ConfigManager/commit fence、dynamic profile ABI和App/Editor/Animation/Physics调用点。若在途代码删除query-time写入、引入session-owned recorder或迁移config caller，应按资格门重新判定；文件名、DTO或test数量变化本身不构成关闭证据。
