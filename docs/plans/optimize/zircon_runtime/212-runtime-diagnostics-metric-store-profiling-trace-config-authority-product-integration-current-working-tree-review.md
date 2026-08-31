# Runtime212: Diagnostics / Metric Store / Profiling Trace / Config Authority / Product Integration 当前工作树复核

- 复核日期：2026-09-01
- 复核 HEAD：`f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`
- 复核类型：review-only；未修改 Rust、Cargo、ABI、tests、UI，也未运行 Cargo/GPU/动态库/soak/benchmark 验证。
- 参考前账：Runtime156/旧文 [104](104-runtime-diagnostics-metric-store-profiling-trace-config-authority-product-integration-current-source-review.md)、Runtime03、Runtime44、Runtime45、Runtime195，Interface04/13，Editor147。
- 责任域：`zircon_runtime::core::runtime` diagnostics/observation/config kernel；App、Editor、Interface、Host 只作为消费方和控制面。
- 参考路由：Bevy diagnostics/log plugin；Unreal `CpuProfilerTrace`/`CountersTrace`/`ConfigCacheIni`；Godot `Performance`/`ProjectSettings`；Fyrox renderer stats/settings；Unity Graphics `ProfilingScope`/`DebugManager`。

## 1. 冻结范围与证据

本轮逐文件读取以下输入集合：

- `zircon_runtime/src/core/runtime/diagnostics/**/*.rs`、`zircon_runtime/src/runtime_diagnostics/**/*.rs`；
- `zircon_runtime/src/foundation/runtime/config_manager/**/*.rs`；
- `core/framework/foundation/config_manager.rs`、`config_persistence_report.rs`、`core/runtime/config_store.rs`、`foundation/runtime/config_manager.rs`、`foundation/runtime/config_path.rs`；
- Dynamic Session 的 diagnostics/profile/ffi/frame、`zircon_runtime_interface/src/profiling.rs`、`zircon_runtime_host/src/foreign_output/item_count.rs`；
- App `engine_entry.rs`，Editor `layout_persistence.rs`/`runtime_services.rs`，Animation manager，Physics settings consumer。

精确参考文件为：`dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs`、`dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs`；Unreal `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CpuProfilerTrace.h`、`CountersTrace.h`、`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h`；Godot `dev/godot/main/performance.h/.cpp`、`dev/godot/core/config/project_settings.h/.cpp`；Fyrox `dev/Fyrox/fyrox-impl/src/renderer/stats.rs`、`settings.rs`；Unity Graphics `dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs`、`DebugManager.cs`。13 个文件在当前工作树均存在并已逐项核对。

| 项目 | 当前值 |
|---|---:|
| Zircon selected 文件 | **111** |
| 总行数 / non-empty | **19,770 / 18,589** |
| 字节数 | **662,014** |
| `#[test]`/`#[ignore]`/`unsafe` 词法命中 | **122 / 9 / 44** |
| tracked / dirty | **111 / 77** |
| current-tree fingerprint | `6912fdbe66334447402ae568ee1b095dfba962a31b35dfa1f41435479203368c` |

当前 diagnostics 树已从旧版的小型 store 扩展为 render stats graph/history、shader/mesh/virtual geometry/hybrid GI、profile/export/hotspot/UI hotspot 等多个模块；这是真实覆盖增长，但仍是一个跨域观察层，不能被误认为已经具备 Unreal/Unity 等级的 tracing authority。生产 render projection 中当前统计路径约 **833** 个文字出现、约 **798** 个唯一 `render.*` 路径，全部为静态字面量；这改善了动态格式化分配，却没有建立 descriptor/catalog/producer owner。

## 2. Runtime 运行时诊断与 Metric Store

### 已成立的底座

- `DiagnosticStore` 使用 `BTreeMap<DiagnosticPath, DiagnosticSeries>` 和 per-series `VecDeque`，默认单序列保留 64 个 measurement；快照有 current/full 两个消费面。
- `record_static` 可以复用完全匹配的 `unit`/tag 元数据，render stats 已拆成按域 dispatch 的投影模块，避免旧版集中式大函数继续膨胀。
- render stats 的 owned projection 已移动字符串所有权，Host 输出层有字节数、结构项数、JSON 深度、解码时间预算；这些是可保留的性能/边界进展。
- Dynamic Session 已把 scene/project/input/render-device/profile 放进同一个返回 envelope，旧 ABI payload 缺字段有 serde default 兼容测试。

### 当前差异与重构要求

| ID | 级别 | 当前证据 | 与工程级引擎的差异 | 必须重构为 |
|---|---|---|---|---|
| RT212-P1-001 | P1 Open | `store.rs` 的 `DiagnosticPath(String)` 可接受任意 key；`record` 会隐式创建 series；render 生产侧约 798 个唯一 literal path | 没有 descriptor、owner、unit 类型、权限、生命周期、schema version、cardinality 目录；插件/功能可无限注入 key | `MetricDescriptorRegistry` + producer owner lease + typed value kind + schema/version + per-owner/cardinality/bytes budget；未知 key 默认拒绝并产生 admission receipt |
| RT212-P1-002 | P1 Open | `DiagnosticSeries::record_measurement` 只检查容量；接受 NaN/Inf、重复或倒退 `frame_index`，EMA 固定 `0.9/0.1` | 时间域、采样语义和聚合策略是隐含实现；不同频率/不同 source 被混在同一序列，无法重放或比较 | 以 monotonic `SampleStamp { clock, sequence, frame, source_generation }` 作为入口；finite/order 校验；明确 last/gauge/counter/histogram/summary 聚合与可配置 window |
| RT212-P1-003 | P1 Open | `collect_runtime_diagnostics` 在查询时分别 query render/virtual debug、physics、animation，再写回 store；physics/animation 使用 `frame_index=0` | observer 的读取动作会改变 authoritative history；同一帧多个 query 会重复 append；render/physics/animation 不是一个 commit window | producer 在 frame/World commit 时发布 immutable batch；collector 只读取；snapshot 带 collection window、source generation、stale/partial/drop flags；重复读取必须幂等 |
| RT212-P1-004 | P1 Open | render query 失败时用 `stats_error.or(debug_error)`，仍设置 `available: true`；没有 per-provider completeness | “可用”与“成功查询”混为一谈，Editor/Host 不能区分 unavailable、stale、partial、error | `ProviderResult<T>`/`ObservationReceipt`，每域记录 status/error class/last-good generation/age；失败不得伪造 available |
| RT212-P1-005 | P1 Open | store snapshot 直接复制全部 series/history；Interface `RuntimeDiagnosticsSnapshot` 只有 `frame_index`、series、profile，无 collection window/revision/cursor | 每次 FFI diagnostics 都是 full deep snapshot；无 filter、if-newer、page/continuation、server-side projection；增长后 ABI 成为大对象复制热点 | 增加 snapshot revision/window/byte estimate；按 descriptor/tag/provider 分页、cursor、if-newer；ABI 使用 immutable page + continuation token，full dump 仅显式离线命令 |
| RT212-P1-006 | P1 Open | MetricStore 只有 per-series history limit；render projection 无总 series/cardinality/bytes ceiling | 798 path 当前可控只是静态来源，未来插件/动态来源仍可把 metadata、String、history 推到进程内存；没有 backpressure/drop accounting | 建立 store-wide hard budget（series、metadata bytes、history bytes、snapshot bytes、update rate）和 deterministic admission/eviction；暴露 dropped/rejected counters |
| RT212-P1-007 | P1 Open | `lock_*` 普遍 `unwrap_or_else(|poisoned| poisoned.into_inner())`；测试验证“能继续”但不验证被污染状态 | poison recovery 没有 authority invariant、generation quarantine 或 degraded receipt；错误状态可能继续向产品层传播 | 将 poison 变成带 generation 的 degraded state；只允许 read-only last-good snapshot，写入转入 quarantine；恢复必须显式 reset/health receipt |
| RT212-P2-001 | P2 Open | `BTreeMap` 路径查找、每次 `series()` 线性搜索 ABI Vec，render domain helper 大量重复 `record_static` | 热路径仍有字符串 key、锁、BTree/Vec 扫描和每 query 大量 f64 转换；没有 numeric metric id 或 lock-free publish | descriptor 编译成 numeric id；per-thread/per-domain staging buffer，frame boundary 批量合并；query 使用索引/bitset/filter，保留 debug-only string projection |
| RT212-P2-002 | P2 Open | counters 仍以 `f64` 传输；UI hotspot 的 `counter_value` 对非 finite 或 `<=0` 直接返回 0 | 负值、归零、signed delta、bytes/quantity 语义丢失；counter hotspot 不能表达释放、回退、债务或负向修正 | 按 descriptor 声明 `u64/i64/f64/bytes/duration`；counter delta/reset 语义显式；热点排序不能静默丢弃 zero/negative |

## 3. Profiling Trace / Capture / Export

### 已成立的底座

- `ProfileCaptureConfig` 已有 session、output root、frame/span/counter capacity、budget、Perfetto 开关；`normalized` 会处理空 ID/root、零 capacity、非法 budget。
- recorder 有 frames/spans/counters retention counters，snapshot/export 受 Host profile-response 的输出预算约束；Perfetto/native/hotspot/counter/UI 多种输出格式已有测试。
- `CAPTURE_STATE` 已为部分异步 producer 提供 capture epoch；Realtime IBL CPU timing 会拒绝旧 epoch completion。这是局部 async 保护，不能扩展为完整 trace authority。

### 当前差异与重构要求

| ID | 级别 | 当前证据 | 与工程级引擎的差异 | 必须重构为 |
|---|---|---|---|---|
| RT212-P1-008 | P1 Open | `static GLOBAL_RECORDER: OnceLock<Mutex<ProfileRecorder>>`；start/stop/reset/snapshot/export 都操作同一个 process-global recorder | 多 Runtime、多 Editor session、动态库租约共享一份 capture；任何控制方都能停止/重置其他实例；全局 mutex 串行化所有 producer | `RuntimeId -> TraceSession` registry；每 session 独立 recorder/control lease；process sink 只作显式 aggregator，不能作为默认 authority |
| RT212-P1-009 | P1 Partial | epoch 只被少量异步 producer 使用；`ProfileScopeToken`/`ProfileFrameToken` 不携带 epoch、thread id、session id；finish 将旧 token 写入当前 recorder | stop/reset/start 后，普通 scope/frame token 仍可污染新 capture；thread-local parent stack 不能证明跨线程/跨 session 正确性 | token 必须携带 session/capture generation/thread identity；completion 先做 generation/owner 校验，拒绝 late/stale token；scope stack 与 session 绑定 |
| RT212-P1-010 | P1 Open | `ProfileRecorder` 是三个独立 `VecDeque`；frame/span/counter 各自 eviction；config 只有 item capacities，无 byte/duration hard cap | span parent、frame relation、counter frame relation 可能悬空；大字符串可在 item cap 内耗尽内存；旧事件没有统一 loss model | 单一 sequence/event arena 或关系感知 eviction；字节、持续时间、字符串池、事件率 hard cap；drop reason/oldest sequence/continuation 明确暴露 |
| RT212-P1-011 | P1 Open | `export_report` 先 clone snapshot，再同步 `create_dir_all`、多次 `to_vec_pretty`、`fs::write` 到最终目录；失败会留下部分文件 | capture stop 线程承担不可预测磁盘 I/O；崩溃/中断后目录可能半成品；无 manifest/hash/fsync/rename/generation/cancel | background export job + immutable snapshot lease；staging directory、manifest、hash/size/schema、fsync、atomic rename；失败清理或标记 incomplete，返回 job receipt |
| RT212-P1-012 | P1 Open | Perfetto projection 固定 `pid: 1`，`tid` 是 logical stream；output root 可由 env 任意指定；basename 只做局部 sanitize | 不能表达真实线程/process/session；跨进程合并 trace 会冲突；路径权限/产品 sandbox 不受 authority 管理 | trace identity 包括 process/runtime/session/thread/stream；由 host policy 分配 output root；导出 manifest 绑定 owner/generation，禁止 caller 越权路径 |
| RT212-P1-013 | P1 Open | normal macros 在 idle 时只用 global active bit，开始/结束仍要 global lock；dynamic names 产生 String；snapshot/export 是深复制 | capture 关闭时低成本提示存在，但打开后每次 scope 竞争单锁，无法达到高频 CPU/GPU tracing 的低扰动要求 | per-thread ring/chunk + lock-free publish，central merge 在 snapshot/export；interned category/name/path；sampling/level/filter 在 producer 入口生效 |
| RT212-P2-003 | P2 Open | `next_span_id`、retention counters 使用 saturating；epoch `wrapping_add`；timestamp 是进程 origin 相对 us | ID/sequence/generation 耗尽后静默复用或停滞；跨 clock/跨 process 无可比性 | checked exhaustion -> capture fault/rollover；128-bit or `(session,generation,sequence)` identity；声明 monotonic clock domain 与 wall-clock correlation |
| RT212-P2-004 | P2 Open | hotspots 从完整 snapshot 二次聚合；UI counter 非 finite/negative 归零；full profile ABI 计入 history 后可能接近 payload limit | 采集、聚合、传输、导出没有明确 stage；大 capture 只能“全量 clone 后失败” | online bounded aggregation + offline exact mode；query/export 采用 page/cursor；limit failure 返回可恢复 continuation，而不是只返回 generic limit-exceeded |

## 4. Config Authority / Persistence / Shutdown

### 已成立的底座

- Foundation 已拆出 `ConfigPersistenceWorker`、generation state、debounce、flush timeout、atomic writer、per-path commit fence、backup recovery；worker panic 和 write failure 会进入 report。
- `ConfigStore` 用 `Arc<Value>` 做 shared snapshot，避免 typed load 先 deep clone；ConfigManager 有 `flush` 与 `persistence_report`，测试覆盖失败、超时、旧 manager 被新 manager supersede 等路径。
- platform path 已按 `ZIRCON_CONFIG_PATH`、Windows LocalAppData/AppData、XDG/HOME 分层，较旧版 process-global fixed basename 有明显改进。

### 当前差异与重构要求

| ID | 级别 | 当前证据 | 与工程级引擎的差异 | 必须重构为 |
|---|---|---|---|---|
| RT212-P1-014 | P1 Open | `DefaultConfigManager` 从磁盘读 `HashMap<String, Value>`，逐 key 写入同一个 `CoreHandle::ConfigStore`；App `store_entry_config` 在 bootstrap 前后各写一次 platform/render/window/editor 值 | 启动注入、session transient、Editor user/layout、plugin settings 共用一张 JSON map；Foundation load 会覆盖同表，worker snapshot 又把 transient 值整体写盘 | 分层 authority：`EngineStartup`、`Project`、`User`、`Session`、`RuntimeEphemeral`、`EditorWorkspace`；每层 typed schema/owner/revision，持久 worker 只能写被授权 durable layers |
| RT212-P1-015 | P1 Open | `ConfigManager::set_value` 是字符串 key + serde_json::Value；没有 schema id、migration version、secret/redaction、unknown-key policy；Editor layout 通过同一 manager 保存 | raw JSON facade 不能提供 Unreal ConfigCache/Godot ProjectSettings 类别、默认值、migration、scope/transaction/validation，也不支持安全审计 | `ConfigDescriptorRegistry` + typed setting document；scope/layer precedence、schema migration、CAS/revision、redaction/secret policy、invalid value quarantine；保留 JSON 仅为版本化 transport |
| RT212-P1-016 | P1 Open | `ConfigPersistenceReport` 只被 manager/test 定义和读取；production App/Editor 没有把 flush failure/dirty generation 接入 shutdown gate 或 user-visible recovery receipt | “set 返回 Ok”只代表内存更新；进程退出由 worker Drop 最多等待 2 秒，超时只写 tracing error，产品仍可能正常退出并丢 durable state | 产品 shutdown coordinator 必须登记 config participant；关闭前取得 durable receipt，超时/失败阻止 clean exit 或生成 recovery artifact，Editor 提供 retry/last-good/newer-generation 决策 |
| RT212-P1-017 | P1 Open | commit fence registry 是 process-global `OnceLock<HashMap<PathBuf, Weak<...>>>`；epoch `wrapping_add`；path normalization 是词法绝对化而非 canonical identity | 同一进程不同 runtime 仍共享 path fence；symlink/case/UNC/网络盘等路径可产生不同 authority；epoch wrap 无故障 | owner/principal/runtime identity + canonical filesystem identity；跨进程 lock/lease 或明确“不支持并拒绝”；checked epoch rollover 和 conflict receipt |
| RT212-P2-005 | P2 Open | Animation manager 在 `store_playback_settings` 先更新内存再调用 `core.store_config`；Physics 同样先改变 settings/world caches，再写 config；Editor layout set 后没有 manager flush | consumer 本地状态先于 durable commit，失败时 memory/disk 分叉；没有统一 transaction/rollback/last-good generation | 所有 consumer 经 `SettingsMutationCoordinator` staged transaction；prepare/validate/apply/commit/rollback；返回 `ConfigCommitReceipt`，consumer 只在 commit 后发布新 revision |
| RT212-P2-006 | P2 Open | worker serializes entire `snapshot()` map on every generation；没有 per-document journal/partial write；shutdown `Drop` 在 timeout 后可能不 join worker | 高频 Editor layout/physics/animation 改动会重复序列化全表；超时后后台线程仍可能活着，生命周期与 product shutdown 不闭合 | journal/WAL + coalesced per-layer snapshot；bounded queue/backpressure；shutdown participant 先 stop producers、drain worker、join or explicit detached-failure artifact |

## 5. Product Integration 分层结论

### Runtime / App

- `zircon_app/src/entry/engine_entry.rs` 将 platform/render/window 和 Editor sandbox/subsystems 写入 Core config，且在 module activation 前后重复 `store_entry_config`。这些值应属于 immutable startup receipt，不应自动进入 user durable config。
- Dynamic Session 的 `runtime_diagnostics_response` 每次请求调用 `collect_runtime_diagnostics`，随后把全量 series/history/profile 放进 `ProfileControlResponse`；它没有 `if_newer`、page token、collection generation 或 per-provider completeness。
- Host foreign-output budget 是“单响应预算”，不是 runtime observability memory budget；达到上限时只能拒绝整次 response，不能在 provider 级别稳定分页。

### Editor

- `layout_persistence.rs` 通过 ConfigManager 保存 default/presets/page-user layouts；项目存在时又写 asset。该双 authority 没有统一 revision/CAS，配置失败只映射为 `EditorError::Project`，没有 durable receipt/retry UI。
- `runtime_services.rs` 直接 `core.store_config_value` 写 enabled subsystems；绕过 ConfigManager worker，因此即使 layout 使用 worker，Editor capability setting 仍可能只留在内存。
- Editor settings 新的 typed `SettingsAuthority`/`SettingsPersistenceService` 是可保留方向，但它与 Runtime `ConfigManager` 仍是第二套 authority；必须明确 Editor user/workspace 层与 Runtime startup/session 层的 ownership，不能靠 key naming 维持一致性。

## 6. 参考引擎对照

| 参考 | 可借鉴的工程边界 | Zircon 当前缺口 |
|---|---|---|
| Bevy diagnostics/log plugin | `Diagnostic` descriptor、measurement history、registry、plugin lifecycle 与记录/读取分离 | Zircon 允许隐式 key，collector query 会写历史；没有注册/owner/cardinality admission |
| Unreal CPU/Counters trace | channel/category、thread/process trace identity、event/counter stream、trace session/export pipeline、typed config cache | Zircon 是 global mutex + logical stream，固定 pid，独立 ring，sync export，raw JSON config |
| Godot Performance/ProjectSettings | named monitor registry、typed project setting、default/override/validation、explicit persistence | Zircon 的 `Value` key 没有 typed schema/layer/migration；runtime transient 和 user durable 混表 |
| Fyrox renderer stats/settings | renderer-side stats snapshot、settings object、bounded renderer-facing view | Zircon query-time render clone 太重，provider failure 仍可 `available=true`，没有 producer commit receipt |
| Unity Graphics ProfilingScope/DebugManager | scope/category marker、debug manager、capture controls、GPU/CPU profiling separation | Zircon scope token 没 generation/thread/session，所有 producer 争一把 process-global mutex，GPU timeline/OS thread identity缺失 |

## 7. Finding / Gate 重新判定

本轮未新增 canonical P0；Runtime195 已有的 F0 attributable startup/failure evidence 仍由 `runtime_diagnostics`/log/ABI 负责。本轮将 Runtime156 的旧 12 项 P1 按当前所有权拆成 17 个可执行 P1 子项（不新增独立领域），并重判旧 6 项 P2：

- P1：**17 Open**。配置 worker、atomic write、commit fence、Host output budget、少量 async capture epoch 只能标记为底座，不满足 authority、scope、durability、bounded query 或 product shutdown 闭环。
- P2：**6 Open**。没有一项达到可删除或 Closed 的工程级证据。
- 30 gates：**22 Fail / 4 Partial / 4 Pass**。Pass 仅限静态 projection 无动态 path、per-series bounded deque、atomic staged file/backup recovery、Host 单响应 structural budget；不代表整个子系统可交付。

### 首个实施切片（仅计划，不在本轮实现）

1. 先建立 `RuntimeId + TraceSessionId + CaptureGeneration`，将 global recorder 改为 session-owned，并让所有 scope/frame/counter token 带 owner/generation/thread。
2. 建立 descriptor registry 与 producer commit batch，禁止 collector query 改写 authoritative history；增加 store-wide memory/cardinality budget 和 provider completeness receipt。
3. 把 diagnostics/profile ABI 改成 revisioned page/cursor/if-newer，full snapshot 限定离线 export；export 改为后台 staged manifest/hash/fsync/rename。
4. 将 ConfigStore 拆为 typed layered authority，启动注入与 durable user/editor workspace 分离；所有 consumer 走 mutation transaction 与 durable receipt。
5. 将 config persistence participant 接入 App/Editor shutdown coordinator，覆盖 timeout、worker crash、backup recovery、multi-runtime/multi-process conflict RED tests。

## 8. 验证边界

本报告只做源码和参考文件审查。没有运行 Cargo、GPU/device loss、动态库 unload/reload、fault injection、multi-process lock、long-running soak、large-cardinality profile、cross-platform path durability 或 p95 tracing benchmark；这些是进入实现里程碑前的必需验证，不得把本报告的静态 Pass 当作功能完成。
