---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/runtime_diagnostics
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/config_manager
  - zircon_runtime/src/diagnostic_log
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime_interface/src/profiling.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/frameworks/01/2026-07-18-m1-runtime-diagnostics-facade-collector-hardcut.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-18-config-manager-synchronous-full-file-rewrite.md
reference_engines:
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CpuProfilerTrace.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CountersTrace.h
  - dev/godot/main/performance.h
  - dev/godot/core/config/project_settings.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs
---

# 03 · Core Runtime Diagnostics、Profiling 与 Config 工程化差距

## 1. 结论

Zircon 已经拥有可用的观测和配置散件：诊断 series 有有限 history、平滑值和 metadata；CPU profiler 有 feature gate、scope/frame/counter、ring、热点分析与 Perfetto/JSON 导出；Render17 已建立异步 GPU timestamp 与 `RenderFrameProfile`；配置 manager 也已经从同步整文件写入推进到 dirty generation、debounce worker、atomic replace、commit fence 和有界 flush。

但这些能力仍不是可支撑大型编辑器、并行 runtime、多 session 动态库和长期性能回归的统一工程系统。诊断路径和 metadata 可由任意调用方隐式创建，series 数量无上限，所有写入和深 snapshot 竞争同一 mutex；CPU recorder 是进程全局单 mutex，导出又把逻辑 stream 伪装成线程；capture token 没有 generation，旧 scope 可以写入新 capture。配置的 canonical manager 可持久化，但公开 `CoreHandle::store_config*` 仍能绕开 dirty generation，生产 animation manager 正在这样做。

本轮登记 7 项 P1 和 5 项 P2。没有新增独立 P0：配置 worker 在 shutdown timeout 后丢弃未退出 `JoinHandle` 的动态库安全风险，已经归并到 `02` 的 canonical execution/DLL-unload P0；本篇只拥有其配置持久化与 flush 语义。Render17 拥有 GPU query、pass timing 和各渲染 metric 的正确性，本篇只拥有共享 metric registry、CPU trace、snapshot、export 和 config contract。

## 2. 审查边界与物理覆盖

### 2.1 已读范围

- `core/runtime/diagnostics` 当前 52 个 Rust 文件、10,941 行；完整读取共享 store、snapshot、devtools 和 profiling 9 文件，并核对所有 runtime diagnostics 聚合入口及动态 ABI 导出。
- profiling 子树当前有 29 个 `#[test]`，diagnostics 整体有 61 个；核对 ring、macro、export、hotspot、poison 和跨线程 frame context 测试。未找到 `#[bench]`、Criterion、Bencher 或 Divan 的真实 benchmark harness。
- 读取 `ConfigStore`、`DefaultConfigManager`、state/worker/writer/commit fence、原子文件 owner、配置 manager tests，以及 runtime/editor/app 的 `set_value`、`store_config*`、`flush` 生产调用点。
- 读取 dynamic session `profile_control`，确认 session handle 只保护 session 查找，实际 start/stop/reset/snapshot/export 仍路由到同一个进程全局 recorder。
- 读取 Bevy diagnostic store、Unreal CPU trace/counter 前端、Godot performance/project settings 和 Unity Graphics profiling scope 的对应实现。

### 2.2 明确未覆盖

- diagnostics 目录多数行是 render stats 的大量字段投影。本篇确认其共享 store 写入与 snapshot 成本，但不逐字段判断光照、阴影、GI、虚拟几何或 UI GPU metric 是否正确；该工作属于后续 `09/10/11` 和 Render17。
- `diagnostic_log` 本轮只追踪到 runtime tick 的全量采集与逐 series 格式化入口，没有把 sink 的队列、文件轮转和 crash flush 宣称为完成；日志专篇仍需深入。
- 本篇不据静态源码判断 profiler 的实际开销或 Zircon 相对参考引擎的速度。缺少受管 benchmark 和产品 trace 时，只能给出必须测量的维度。

## 3. 当前实现闭环

### 3.1 DiagnosticStore

`DiagnosticStore` 用一个 `BTreeMap<DiagnosticPath, DiagnosticSeries>` 保存所有 series；每个 series 的 history 默认为 64，measurement 只有 `frame_index + f64`。`record` 会对不存在的字符串 path 自动建 series，非空 unit 覆盖旧 unit，tag 则持续去重合并（`store.rs:73-126,169-217`）。

`CoreHandle::record_diagnostic` 每次取得 core diagnostics mutex；`diagnostic_store()` 和 `diagnostic_store_snapshot()` 在同一锁内 clone store 或全部 series/history（`handle/diagnostics.rs:8-30`）。ECS 一次 publish 会循环调用多次 `core.record_diagnostic`（`scene/ecs/frame_performance_diagnostics.rs:312-344`），因此一帧同一 producer 也重复 lock/unlock。

`collect_runtime_diagnostics` 先 clone core store，再把 render/physics/animation 值写入这个临时 clone，最后 snapshot 并丢弃（`runtime_diagnostics/collect.rs:72-121`）。这些临时投影没有写回权威 store，所以其 history、EMA 与 lifetime min/max 不会跨 collection 形成时间序列。

### 3.2 CPU profiling 与导出

`GLOBAL_RECORDER: OnceLock<Mutex<ProfileRecorder>>` 和 `CAPTURE_ACTIVE` 是进程静态状态（`profiling/mod.rs:40-77`）。active 时 begin/finish/frame/counter 都进入同一 recorder mutex；scope begin 构造/复制 path，finish 又分别锁一次取时间和一次提交 snapshot（`profiling/scope.rs:126-190`）。

Recorder 使用 frame/span/counter 三个独立 `VecDeque`，满时静默 `pop_front`；snapshot 在 recorder 锁内把所有 String 和 row 深 clone 到三个 Vec（`profiling/recorder.rs:29-37,116-153`）。DTO 没有 capture generation、runtime session、OS thread/process identity、clock domain、dropped count 或 completeness；Perfetto export 固定 `pid = 1`，直接用 `stream` 字符串作为 `tid`（`zircon_runtime_interface/src/profiling.rs:75-132`、`profiling/export.rs:158-205`）。

导出直接在 `output_root/sanitize(session_id)` 中依次 `fs::write` 六类文件。sanitize 允许 `.`，因此精确的 `..` 仍是父目录；导出没有 staging directory、manifest、checksum 或原子 publish，同一目录复用也不会删除本次未生成的旧 artifact（`profiling/export.rs:53-123`）。

### 3.3 Config persistence

底层 `ConfigStore` 是 `Arc<Mutex<HashMap<String, Arc<Value>>>>`，支持 raw JSON 与按调用点反序列化 typed value。`CoreHandle` 公开 `store_config_value/store_config/load_config/snapshot_config_values`，这些入口只改内存（`config_store.rs:14-62`、`handle/events.rs:34-63`）。

`DefaultConfigManager::set_value` 才会调用 `request_persistence(changed)`；worker 具有 generation、debounce、atomic writer、commit fence、失败报告与 `flush(timeout)`（`config_manager.rs:88-103`）。但是 production 没有显式 config flush 调用，当前只依赖最后 owner Drop。animation manager 加载时吞掉 typed decode 错误并回落默认值，保存时直接调用 `core.store_config`，所以设置进入内存却不会推进 persistence generation（`animation/manager/mod.rs:43-68`）。

配置文件本身仍是无版本的整张 JSON map。没有 key registry、类型/schema、default/source layer、revision/change event、deprecated key migration、restart requirement 或 secret policy；同一个 raw key 可以由不同模块以不同 Rust 类型解释。

## 4. 差距清单

### P1-1：CPU recorder 的进程全局单锁破坏多线程真实性和多 session 隔离

**证据**

- 所有线程的 active scope/counter 通过同一 `Mutex<ProfileRecorder>`；begin/end 观测动作本身串行化被测线程，热点越密集，observer effect 越强。
- `SPAN_STACK` 虽是 thread-local，但 snapshot 不保存真实 thread id；Perfetto 把多个 worker 的相同 `stream` 合并到一条 `tid`，并行重叠无法被正确重建。
- dynamic ABI 的每个 session 都可发 `StartCapture/StopCapture/Reset/ExportReport`，但实际调用同一个 static recorder；一个 session 可以清空或停止另一个 session 的 capture。

**后果**

当前 trace 可用于粗粒度单 session 调试，不能作为 1/8/64 worker 调度、并行 renderer、后台 asset pipeline 或多 runtime session 的可信性能证据。加更多埋点会同时增加锁争用和 String 分配，可能改变待测调度行为。

**目标契约**

由 `TraceSession` 显式拥有 capture generation 和 runtime/session identity；静态 callsite 注册为 dense marker id，每个 OS worker 写自己的有界 chunk/ring，frame/seal 时批量归并。事件至少携带 process/thread、clock domain、task/span context 和 capture generation；inactive fast path 必须由 benchmark 证明接近零成本。

### P1-2：capture start/stop/reset 与存活 scope 没有 generation barrier，旧 token 会污染新 capture

scope token 只保存 span id 和相对 origin 时间，没有 capture epoch。`start_capture/reset` 会重置 origin、next id 和 rings；`stop_capture` 只把 active 设 false。已经 begin 的 scope 在 drop 时，`finish_scope` 只检查 feature enabled，随后无条件按当前 recorder 的新 origin 取 end timestamp并写入 ring（`profiling/mod.rs:47-70,233-237`、`profiling/scope.rs:172-190`）。

因此 begin → stop/reset/start → finish 可以在新 capture 中产生重复 id、错误 duration、悬空 parent/frame，甚至在 stopped capture 中继续写样本。目标是 token 带 `TraceSessionId + generation + producer/thread sequence`；stop 进入 sealing，等待/截断已发 token，迟到 token 记为 dropped-stale，绝不写入其他 generation。测试必须覆盖跨线程和控制命令竞态。

### P1-3：diagnostic path/metadata 没有 schema registry，series cardinality 只有 history 上限而没有总量上限

任意字符串 path 都能自动建 series；全局只限制每条 history，不限制 path 数、metadata bytes 或整个 snapshot bytes。动态实体/资源/viewport 名若进入 path，会造成永久 cardinality 增长。unit 可以被后续写入覆盖，tag 可以持续并集增长，没有 descriptor collision 或 producer ownership error。

目标是启动/模块激活时注册 `MetricDescriptor { MetricId, stable path, kind, unit, aggregation, retention, cardinality, owner }`；热路径只用 dense id。动态维度必须作为 bounded label 或 sample payload，不得扩展 path；重复 descriptor 不一致应拒绝 activation。registry 提供全局/owner byte budget、dropped/unknown/collision diagnostics。

### P1-4：runtime diagnostics 聚合把 domain 指标写入临时 clone，时间统计语义是假的

render、physics、animation 的 facade collector 在 `core.diagnostic_store()` clone 上 record 后立即 snapshot。除非相同 path 已由别处写入 core store，这些 series 每次 collection 都从空状态开始，其 `history` 只有当前样本，EMA 等于当前值，min/max 也只反映当前值。API 却把它们投影成与长期 series 相同的 DTO，consumer 无法判断统计来源和有效窗口。

目标只允许两种明确模式：producer 在 frame/owner commit 时写入权威 metric store；或 collector 返回标注为 point-in-time 的 domain snapshot，不伪造 history/smoothing。跨 domain snapshot 必须带 capture generation/time window，每个 provider 报 observed generation 和 stale/unavailable reason。

### P1-5：配置存在可公开绕过 persistence 的双写入口，生产设置已经发生静默不落盘

`ConfigManager::set_value` 与 `CoreHandle::store_config*` 都是公开写入口，但只有前者推进 dirty generation。animation playback settings 走后者；进程内读立即成功，重启后却可能回到旧文件。其加载又把所有反序列化错误吞成 default，坏值、schema 漂移和真实缺省无法区分。

目标是单一 canonical write transaction：持久配置只能通过 typed `ConfigKey<T>` 和 `ConfigService::transaction/set`，返回 revision 与 durability policy；session-only override 使用不同 API/类型，不能共享同名 raw store。旧 `CoreHandle::store_config*` 在所有调用点迁移后删除，不保留兼容旁路。

### P1-6：配置没有 schema、layer、migration 和 change transaction，无法支撑项目/用户/会话边界

当前 whole-map JSON 只有 key/value，没有 format version、key version、source、override precedence、validation、restart flag 或 migration report。app bootstrap、project config、editor layout、runtime subsystem 和命令行/env override 因而只能靠调用顺序隐式决定最终值；未知/坏 key 也没有统一 policy。

目标建立 `ConfigRegistry` 与显式 layers：built-in defaults → engine/project → user/editor → command-line/environment → session override。每个 key 声明类型、validator、version、scope、persistence target、restart/live-apply policy 和 migration。多 key 变更先校验再原子 publish revision，observer 在锁外收到 typed delta；加载产生可查询的 unknown/deprecated/invalid/migrated report。

### P1-7：profile export 的路径和 artifact publish 不具备工程数据完整性

精确 `session_id = ".."` 经 sanitize 后不变，`PathBuf::join` 会选择 output root 的父目录；固定文件名可能覆盖该处已有 profile artifact。导出依次直接写文件，中途失败留下新旧混合目录；关闭 Perfetto 时旧 `timeline.perfetto.json` 也可能残留。snapshot 本身没有 schema/generation/source fingerprint/completeness，报告无法证明各文件来自同一 capture。

目标拒绝 `.`/`..`、绝对路径和保留名，或使用生成的 opaque capture id；全部 artifact 写入同父目录 staging，生成 versioned manifest（capture id、source fingerprint、schema、clock、counts、dropped/truncated、hash），fsync/close 后原子 publish。导出在后台受控 I/O lane 执行，不能在 session control lock/帧线程内做全量 clone、分析和写盘。

### P2-1：三个独立 ring 静默截断，snapshot 没有完整性语义

frame/span/counter 分别 eviction，未记录 dropped count、first/last sequence、retained bytes 或 truncation reason。frame 可被淘汰而 span 仍引用它，parent span 也可先于 child 淘汰。`normalized` 只替换 0 容量，没有不可突破的 entry/byte/page 上限；`NaN`/infinity frame budget 也能通过 `<= 0.0` 检查。

目标以 generation chunk 为一致性单位，并同时限制 entries、retained bytes、metadata bytes 与 snapshot page。截断必须保留 explicit gap/drop event 和 completeness flag；外部请求返回 requested/effective limits。所有浮点配置和 counter 值有 finite/invalid policy。

### P2-2：DiagnosticSeries 的统计窗口与时间语义不足

measurement 没有 monotonic timestamp；EMA 固定按样本 `0.9/0.1`，因此 1 Hz 和 1 kHz producer 得到完全不同的时间响应。min/max 是 lifetime extrema，history 淘汰时不重算，也没有 reset/window identity。NaN/inf 不校验，frame index 倒退或重复也没有状态。

目标 descriptor 声明 gauge/counter/histogram/event 与 aggregation；measurement 带 monotonic time/generation。平滑按时间常数或由 consumer 明确选择，window min/max/percentile 与 lifetime extrema 分开，reset 有 epoch。高频数据使用 histogram/sketch 或采样，不能把所有数值压成一种 f64 series。

### P2-3：snapshot 和 diagnostic logging 在 owner 线程执行全量深拷贝与逐 series 格式化

runtime diagnostics collection 会解析 render/physics/animation service、clone 整个 store/history，再 clone 全局 profile。动态 diagnostic logging 每个周期在 frame tick 路径收集 snapshot，并对每条当前 series 单独格式化/写日志。series 达到 541 或 10K 时，observer 本身可以制造 frame hitch；当前没有 subscription filter、if-newer generation、delta、page 或每帧观测预算。

目标 producer 以 immutable sealed generation 发布；UI/logger/capture 通过 `query_if_newer` clone Arc 或分页读取。默认 health snapshot 只含小型固定指标，详细 domain 数据按订阅/捕获懒 materialize；日志有 filter、delta、rate/byte budget 和 dropped report。必须量化 diagnostics off/on、1/541/10K series、0/30/60/120 Hz consumer 的 CPU、allocation、lock wait 与 frame p95。

### P2-4：devtools snapshot 包含硬编码后端状态，并非一致时刻的事实

`project_runtime_devtools_snapshot` 固定返回 native backend available=true、loaded_plugin_count=0，VM available=false，而不是查询实际 backend/plugin owner（`diagnostics/devtools.rs:71-89`）。modules、services、plugin catalog 与 diagnostics 又分别在不同锁/时刻采集，没有共同 revision；一次 snapshot 可能组合 activation 前后的状态。

目标 backend/plugin owner 提供 versioned provider snapshot，未知能力显示 unknown/unavailable reason，禁止硬编码成功。devtools 顶层记录 collection time 和各 provider generation；需要强一致时走 coordinator barrier，不需要时明确标注 weakly-consistent。

### P2-5：poison recovery 测试证明“继续运行”，没有证明数据仍满足不变量

diagnostic store、global recorder、config store/worker state 和 devtools registry 多处统一使用 `poisoned.into_inner()`。现有测试通常在任何关键 mutation 前先 panic，再断言之后能读写；它们没有覆盖 partial record、ring eviction、dirty generation/attempt 或 module/service snapshot 中途破坏。

目标按数据分类：纯派生缓存可丢弃重建；持久化 transaction 依赖 journal/generation 校验；trace generation 可标 damaged 并 seal；无法证明一致的 control state 必须 fail-closed。故障注入应在 mutation 的各阶段 panic，验证 conservation、旧 durable 文件保留和错误可观察性。

## 5. 参考引擎证据与适用边界

| 参考 | 已核对机制 | Zircon 应吸收 | 不应照搬 |
|---|---|---|---|
| Bevy diagnostics | `Diagnostic` 先注册 descriptor，单项拥有 enabled、max history、suffix 和带 `Instant` 的 measurement；写入 SystemParam 先检查 enabled 并在 local buffer 延迟提交；EMA 根据时间差计算（`bevy_diagnostic/src/diagnostic.rs`） | 注册式 metric、per-metric enable/retention、timestamp、低成本 disabled path、延迟批提交 | 它不是高频多线程 trace recorder，也不解决 Zircon dynamic DLL/session 隔离 |
| Unreal CPU trace/counters | CPU scope 是严格 per-thread begin/end，static marker 明确比 dynamic name 低开销，线程 buffer 可 flush；counter 区分 int/float、display hint、static/dynamic name，并在 channel enabled 后才 emit（`CpuProfilerTrace.h`、`CountersTrace.h`） | per-thread buffer、static callsite id、channel gate、typed counters、显式 flush/seal | Unreal 的全局 trace infrastructure 服务常驻进程；Zircon 仍需 session/generation 与卸载 owner |
| Godot performance | 固定 monitor enum 声明 quantity/memory/time/percentage 等类型，同时支持 custom monitor 的 add/remove 和 modification time（`main/performance.h:62-149`） | stable built-in identity、metric kind/unit、受控扩展 registry、change revision | singleton/Variant API 不适合作为 Zircon 的强类型热路径 |
| Godot project settings | setting 拥有 version/change tracking、persist/basic/internal/initial/restart metadata、property info、ordering、feature override、changed-settings 和 save/load（`core/config/project_settings.h`） | schema、来源/override、restart policy、change set、持久化与 runtime projection 分离 | 不复制单例与动态 Variant 作为内部 Rust 类型系统 |
| Unity Graphics | `ProfilingSampler` 复用静态 enum marker、recorder 懒创建；`ProfilingScope` 同时支持 CPU inline 与 CommandBuffer/GPU marker，feature/verbosity 控制并显式 Dispose；测试覆盖 lazy allocation 与 CPU/GPU 行为（`ProfilingScope.cs`） | marker registry、CPU/GPU 名称关联、lazy recorder、compile/runtime gate 与资源释放测试 | C# object/lifetime 模型不是 Zircon Rust API 目标，GPU query 具体实现仍由 Render17/RHI 拥有 |

这些参考共同说明：metric、trace 和 config 都需要预先声明 identity 与 owner，热路径写入与控制/导出路径分离，capture/save 必须有 generation 和完成边界。它们没有任何一个能替代 Zircon 对多 session 动态库、Rust task context 和 WGPU clock correlation 的专门设计。

## 6. 目标架构

### 6.1 Telemetry registry 与 metric store

- `TelemetryRegistry` 在 module activation 事务中注册 `MetricDescriptor`，返回 dense `MetricId`；descriptor 冲突使 activation 失败。
- `MetricWriter` 绑定 owner/session/generation，以 batch/worker shard 写入；gauge、monotonic counter、histogram、event 各有独立类型与 aggregation。
- `TelemetrySnapshot` 是 sealed immutable generation，带 monotonic interval、provider generation、stale/unavailable、dropped/truncated 和 source fingerprint。
- health、editor inspection、logging、remote/ABI export 是不同 projection；都从同一 sealed data 读取，不在 frame owner 临时重建长期统计。

### 6.2 Trace session

- `TraceController` 按 runtime owner 创建 `TraceSession`；控制命令必须携带 session handle 和 capture generation，不能落到匿名 process global。
- 每线程/worker bounded chunks + static callsite ids；task spawn 自动传播 span/task context，不能只传播一个可能冲突的 frame index。
- stop 执行 close admission → drain/seal producer → merge/index → immutable capture；迟到 token 只能计 dropped-stale。
- CPU monotonic clock、GPU timestamp period/calibration、process/thread metadata 和 delayed GPU frame generation 在 manifest 中关联；Render17 继续拥有 GPU query/readback。

### 6.3 Config service

- `ConfigRegistry` 声明 `ConfigKey<T>`、schema/key version、scope/layer、default、validator、migration、restart/live apply 和 persistence target。
- `ConfigTransaction` 读取一个 revision，对多 key 先验证、再原子 publish；typed delta 在锁外投递，失败不部分生效。
- durable writer 按 target 拥有 dirty generation、atomic replace、recovery journal/backup、显式 `flush(deadline)`；shutdown coordinator 把 flush + worker join 纳入可卸载事务。
- session override 和 durable project/user config 使用不同 capability；raw JSON 只保留在文件/ABI adapter，不作为 production 内部写入口。

### 6.4 Artifact pipeline

```text
Running telemetry/config generation
  -> Close admission for capture/save target
  -> Seal immutable generation and completeness report
  -> Background serialize into sibling staging directory/file
  -> Write manifest + hashes + schema/source fingerprint
  -> Flush and atomic publish
  -> Return typed artifact result
```

## 7. Hard cut

- 删除 production `GLOBAL_RECORDER` 控制面；若保留 process trace aggregator，也只能由 host 显式拥有并以 session id 分区，DLL 不持有匿名静态 owner。
- 所有 `record_diagnostic("string", ...)` 调用迁移到注册 `MetricId`/typed writer 后删除动态内部入口；动态 names 只能进入受预算 labels/event payload。
- 删除 `CoreHandle::store_config*` 公共写入口；durable 与 session override 调用点分别迁移到 canonical typed API。
- 不保留“collector 临时 clone 后 record”与“producer 写权威 store”两条长期统计路径。
- profile/config artifact 新格式完成 reader/工具迁移后直接切换；旧格式只允许一次性离线迁移器，不保留双写。
- 不在本篇另建 GPU profiler、render metric DTO 或 log sink；分别复用 Render17 和后续 logging owner。

## 8. 测试先行重构里程碑

| 里程碑 | 先写的失败证据 | 实现范围 | 晋级条件 |
|---|---|---|---|
| M0 | 两 session 互相 stop/reset、begin-reset-start-finish、真实 thread track、config direct-write restart、`session_id=..`、export 中途失败 | 只建竞态/持久化/artifact 故障 harness | 当前问题可稳定复现；与 `02` 的 DLL worker test 共用 host |
| M1 | descriptor collision、unknown metric、dynamic cardinality、owner budget、batch conservation | `TelemetryRegistry` + typed/sharded writers | 所有共享 production metrics 有注册 owner，无无界 path 创建 |
| M2 | 1/8/64 producer、nested task spans、stale token、stop/seal deadline、ring truncation | session-owned TraceSession + per-thread chunks | thread/session/generation 正确；drop/completeness 可查询 |
| M3 | producer/domain generation mismatch、if-newer/delta/page、541/10K series | immutable telemetry generation + projections | frame owner 无全量深 snapshot；consumer 可辨 stale/partial |
| M4 | config layer precedence、schema error、migration、multi-key rollback、typed change | ConfigRegistry/Transaction/layers | 单一 typed write owner；未知/坏/迁移结果完整报告 |
| M5 | debounce/failure/retry/crash/backup/flush timeout、DLL unload thread census | durable config writer 接 shutdown coordinator | 成功 teardown 保证 durable policy且 worker joined；超时阻止 unload |
| M6 | staging failure、stale files、manifest/hash、path traversal、async cancellation | artifact pipeline | export 目录只出现完整 generation；路径不可逃逸 |
| M7 | ABI/editor/logger/render consumers 与旧 API 扫描 | 硬切旧 recorder/store/config routes | 无双轨/shim；全 workspace 与产品闭环门通过 |

M1-M3 先建立可相信的测量工具，再允许 Runtime07 或 Render17 依据 profile 做算法优化。不能用当前 recorder 产生的数字证明重构收益，再用同一缺陷工具验收自己。

## 9. 验收矩阵

### 9.1 Correctness 与并发

- 1/8/64 producer 的 span parent、task propagation、thread/process track、frame/capture generation；start/stop/reset/export 与 scope drop 全排列竞态。
- metric accepted/dropped/invalid 总量守恒；descriptor collision、late provider、module unload/reload 和 owner budget exhaustion fail-closed。
- snapshot 在 provider 同时更新/卸载时只返回 sealed generation或明确 stale/partial，不组合无标记的不同时刻状态。

### 9.2 Config 与故障恢复

- 各 layer precedence、unknown/deprecated key、逐版本 migration、validator、restart/live apply、multi-key rollback 和 concurrent optimistic revision conflict。
- 1/1K burst、同值、失败后同值重试、磁盘满、权限失败、序列化 panic、atomic replace 中断、backup recovery 和 corrupt file quarantine。
- animation 等每个现有 caller 做 restart roundtrip；禁止仅断言进程内 store 的值。
- shutdown flush deadline 与 config worker 卡在 snapshot/serialize/write/replace 各阶段；destroy 成功时线程 census 回基线，失败时 host 不卸载。

### 9.3 Artifact 与 ABI

- session id/path traversal、Unicode/保留名/超长字段、巨大容量请求、NaN/inf、旧 schema 和 unknown field。
- 每个 export artifact 的 manifest generation/hash/source fingerprint 相同；中途失败只能看见旧完整或新完整版本。
- snapshot/export 分页和取消不会在 session mutex、frame thread 或 render owner 上执行全量 I/O。

### 9.4 性能

- inactive/active scope：1/8/64 threads、0/100/10K events per frame，记录 ns/event、allocations、lock wait、drop rate、retained bytes 和被测 workload 扰动。
- metric write/snapshot：1/541/10K series、history 1/64/1K、consumer 0/30/60/120 Hz，记录 frame p50/p95/p99 和 writer/reader CPU。
- config：1B/1MiB/100MiB snapshots、1/1K key burst、serialize/write/replace latency、coalescing ratio、shutdown deadline。
- Windows product run必须附 source fingerprint、CPU/GPU/driver、profile、raw sample 和噪声区间；无同条件数据不作“优于 Unreal/Unity”等结论。

## 10. 既有计划纠正与 owner

1. `runtime/07-runtime-performance-hotpath.md` 已明确记录 active profiling 单 mutex、diagnostic 541-series 深 snapshot 和 capture 容量缺口；其 `status: completed` 只代表既定局部切片，不代表观测底座工程化完成。PERF-MVP-324/326/566 对应项保持开放，并由本篇目标架构统一。
2. `frameworks/01/...runtime-diagnostics-facade-collector-hardcut.md` 正确完成了 manager-resolving collector 的 owner 切分，但它明确没有承诺 `zr_diagnostics` 提取或 package acceptance；本篇不重开 owner hard cut，只修 collector/store 的数据语义和性能。
3. `render/17-performance-and-profiling.md` 与其 2026-08-11 research 继续拥有 WGPU capability、query reservation、delayed readback、pass timing 和 RenderDoc generation。共享 CPU trace/thread context、artifact manifest 和 CPU/GPU clock correlation由本篇提供。
4. `runtime/02/...config-manager-synchronous-full-file-rewrite.md` 的 dirty generation、atomic replace 和 commit fence 是保留基础；其 failure 仍为 open。新增 direct-write bypass、typed schema/layers 和 teardown join 证明后，不能只凭现有 round-3 static review 关闭。
5. `02-core-runtime-events-tasks-review.md` 唯一拥有 worker/thread/DLL unload P0；本篇配置 M5 只提交 durable flush policy 和 config worker blocker，不另造 execution owner。

## 11. 工作区复核标记

本轮 diagnostics 大量文件正被其他会话修改。本文证据按 2026-08-15 current source 读取；实施 M0 前必须重新核对 `store.rs`、profiling 全子树、`runtime_diagnostics/collect.rs`、config manager worker 和 dynamic session control。任何新增 API 只有在 session/generation/thread/path/restart 的失败测试通过后，才允许下调 finding；文件名或 DTO 字段增加不构成关闭证据。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
