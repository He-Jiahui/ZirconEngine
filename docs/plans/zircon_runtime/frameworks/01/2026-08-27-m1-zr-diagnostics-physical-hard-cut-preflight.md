---
plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
milestone: M1 Phase 1 - zr_diagnostics physical hard cut
status: preflight_reopened_current_source_drift_ordered_after_kernel_foreign_attribution_blocked
date: 2026-08-27
updated: 2026-08-29
related_code:
  - zircon_runtime/src/diagnostic_log
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/runtime_diagnostics
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/Cargo.toml
  - Cargo.toml
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Logging/LogMacros.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/OutputDevice.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CpuProfilerTrace.h
  - dev/UnrealEngine/Engine/Source/Runtime/TraceLog
  - dev/bevy/crates/bevy_log
  - dev/bevy/crates/bevy_diagnostic
  - dev/godot/core/io/logger.h
  - dev/godot/main/performance.h
---

# Frameworks01 M1 `zr_diagnostics` physical hard-cut preflight

## 1. 状态与非声明

本记录完成 `zr_diagnostics` 的 current-source 结构复核、依赖方向复核、参考引擎路由、
性能门设计和原子迁移清单。当前状态是：

- `architecture_review_complete`；
- `historical_source_fingerprint_complete`；
- `current_source_drift_reopened`；
- `current_source_drift_review_required`；
- `runtime_diagnostics_boundary_guard_green_2_of_2`；
- `physical_hard_cut_not_started`；
- `ordered_after_zr_kernel`；
- `foreign_attribution_blocked`；
- `managed_cargo_and_profile_not_run`；
- `performance_and_power_claims_not_admitted`；
- `milestone_not_accepted`。

本记录不是 accepted milestone，不证明源码迁移、产品编译、性能改善、瓶颈消失、功耗接近其它引擎或
算法达到最优规模。用户要求的“先调研与 profile、再优化”在这里被落实为禁止条件：在同一 current-source
fingerprint 上取得受管基线前，不批准修改 filter、queue、batch、store、trace recorder 或 snapshot 算法。

## 2. Current-source 指纹

采集基准为 HEAD `1c8076ac65faee28290c575356e9fee6cc1fac48`。manifest 算法为按仓库相对路径
排序，为每个文件记录 `path<TAB>bytes<TAB>lines<TAB>file-sha256`，以 LF 连接后再计算
SHA-256。当前结果如下：

| 集合 | Rust 文件 | 行数 | bytes | manifest SHA-256 |
|---|---:|---:|---:|---|
| `diagnostic_log` 全树 | 31 | 4,277 | 140,969 | `7439363a16193ba719c6bee76bafc5ccbb8589d9a63fe8ed3ff354f34b2f92c8` |
| 可迁移 production owner | 8 | 2,111 | 68,763 | `35002437fa72d8608d6b4c303017caf43db2ecc8fd7264559235bf822615209a` |
| sink owned tests | 17 | 1,660 | 54,676 | `ad06d3b6f640f1043e4feb9f8d92c0cd4a96a303ee38dac8ebb207fea4cfa7bb` |
| Runtime store-log adapter + tests | 5 | 468 | 15,617 | `f644ef6b5da98a6d2f3a6c3bcba3ea53fd445ee3fc6cb2b269970eb3c1114ecb` |
| `core/runtime/diagnostics` 全树 | 62 | 12,483 | 400,181 | `af3cedf05532f39229329c1e501e557c01597526cae5d93ede0992011d62bf7a` |

上表现在只代表 2026-08-27 的历史 preflight 输入，不再批准实施。2026-08-29 使用相同算法对共享
current source 重取 manifest，结果如下：

| 集合 | Rust 文件 | 行数 | bytes | current manifest SHA-256 |
|---|---:|---:|---:|---|
| `diagnostic_log` 全树 | 32 | 4,425 | 146,428 | `7b439927b3f47527cee2fc32f46d98727cd86eff75cf3922ea5e2b27d6aaedbe` |
| 可迁移 production owner | 8 | 2,123 | 69,230 | `ee592b1d66f49b45000d2ef33594334c0ade43cbca838a1abf43e6a60dd86dac` |
| sink owned tests | 17 | 1,660 | 54,676 | `6123d3f1f739868102689619113782023981610dc64ac7f3508eabe9adab4217` |
| Runtime store-log adapter + tests | 5 | 468 | 15,617 | `a4119c2170867a3ece0bdf5c5c36eb7f465e737e9ec9e32112e6fe3fb6e5bed8` |
| `core/runtime/diagnostics` 全树 | 68 | 13,039 | 418,815 | `735b369d1639c446039f1b0a54fc6fda21860636f2380af235849daa800d490d` |

漂移不是机械格式变化。`diagnostic_log` 新增 `level/borrowed_parse_tests.rs`，并修改了 5 个 production
owner blob：`level.rs`、`level/compiled.rs`、`settings.rs`、`sink.rs` 与 `sink/worker.rs`。其中包含
borrowed case-insensitive parse、single-node filter fast path、borrowed active-state read、full-queue lazy-message
short circuit、bounded control send 与 batch timestamp reuse。Runtime adapter 另有 Runtime44 的 schedule
coalescing 改造；`core/runtime/diagnostics` 又新增或拆分 dynamic-name handoff、render graph 与 Hybrid GI
统计 owner。现有 ignored benchmark 或源码内优化标签不构成 Frameworks01 的同指纹 managed profile，
不能直接吸收到 hard-cut candidate，也不能据此声明性能或功耗改善。

8-file production owner 是 `level.rs`、`level/compiled.rs`、`platform.rs`、`settings.rs`、
`sink.rs`、`sink/metrics.rs`、`sink/worker.rs` 与 `timestamp.rs`。Runtime adapter 是
`diagnostic_log/diagnostics.rs` 及其四个 tests。剩余的 `diagnostic_log/mod.rs` 是当前 Runtime
facade root，hard cut 后必须改为显式 curated projection，不能原样移入私有 crate。

当前不是干净输入。相关路径相对 HEAD 至少有 24 个 tracked 文件发生 1,078 insertions / 1,366 deletions，
另有新增的 level、profiling、render graph 与 Hybrid GI 测试/子 owner，覆盖
log schedule/filter、sink lifecycle/backpressure、DiagnosticStore、profiler/export、devtools、render stats
与 Runtime collector。2026-08-29 指纹也只可用于漂移复核；正式实施前必须在所有 upstream owner 稳定后
再次重取 manifest，并对每个 dirty blob
按 current hash 完成 attribution/transfer，不能把历史内容重新生成后覆盖共享工作区。

## 3. 致命结构问题：禁止整目录迁移

父计划锁定的 layer 顺序是
`zr_math/zr_resource -> zr_contracts -> zr_kernel -> zr_diagnostics`，并明确把
`zr_diagnostics` 的 M1 owner 写为 `diagnostic_log`。当前 `core/runtime/diagnostics` 不是同一职责：

1. `CoreRuntimeInner` 直接拥有 `Mutex<DiagnosticStore>`，`CoreHandle` 暴露 record/snapshot/update；把
   `DiagnosticStore` 移入 layer-1 会让 layer-0b `zr_kernel` 反向依赖 `zr_diagnostics`。
2. `core/runtime/diagnostics` 同时混合 series store、CPU profiling recorder/scope/export、devtools 产品
   DTO 和大体量 render statistics projection。它们的生命周期、数据率和 consumer 都不同。
3. `diagnostic_log/diagnostics.rs` 依赖 Runtime 的 `DiagnosticStore*Snapshot`，负责格式化与周期调度；若
   随 sink 移入私有 crate，就会形成 `zr_diagnostics -> zircon_runtime` 的禁止反向边。
4. Runtime03 的目标是 telemetry registry、typed/sharded writer、session-owned trace 和 immutable
   generation。把现有 global store/profiler 顺手搬进 M1 crate 既不会修复算法，又会抢占 Runtime03 owner。

因此拒绝以下方案：移动整个 `core/runtime/diagnostics`、移动整个 `diagnostic_log`、让
`zr_diagnostics` 依赖 `zircon_runtime`、复制 `DiagnosticStore` DTO、保留旧 implementation forwarding
module，或用 public wildcard re-export 暂时闭合编译。

## 4. 锁定目标边界

| Owner | 本波次职责 | 明确不拥有 |
|---|---|---|
| `zr_diagnostics` | log level/filter/config、compile-time level gate、platform log location、settings、timestamp、bounded async sink、worker、sink metrics、process/dynamic-session lifecycle | Runtime DiagnosticStore、metric registry、trace profiler、render stats、devtools DTO、manager resolver |
| Runtime `diagnostic_log` facade | 只投影批准的产品 logging API；保留 Runtime-only dynamic lease assembly 可见性 | implementation 副本、wildcard projection、兼容 alias、对 App/Editor 暴露私有 assembly |
| Runtime store-log adapter | `DiagnosticStore*Snapshot` 格式化、write schedule、delta/budget 演进接点 | sink worker、platform path、重复 filter implementation |
| Runtime core diagnostics | 现有 store/profiling/product projection，直到各自 owner 的结构 hard cut | log sink implementation |
| Runtime03 | telemetry schema、typed/sharded writer、session-owned trace、sealed generation 与 artifact pipeline | Frameworks01 的 crate movement 与 facade 兼容层 |

`zr_diagnostics` 初始依赖限于 `std`、`arc-swap` 和 `crossbeam-channel`；若重取源码后出现新增依赖，必须
逐项解释，禁止因为迁移方便而依赖 Runtime facade、manager、graphics、scene、asset 或 editor。当前
`ProcessLogController` 的 process singleton 只有在 dynamic-session lease、unpublish-before-join、flush timeout、
bounded queue 和 worker census tests 同批保留时才允许迁移；它不是 CPU trace recorder 的全局控制面先例。

`LogModule`/`LogDiagnosticsModule` 一类依赖 EngineModule/Core descriptor 的组装代码不能反向塞进
`zr_diagnostics`。它们应在 `zr_kernel` hard cut 后使用下层 logging capability，或暂留 Runtime assembly，
直到同一原子波次具备正确依赖方向。M1 实施顺序因此必须严格等待 `zr_kernel`，不能为了先创建 crate
而制造临时反向依赖。

App、Editor 和 plugins 继续只依赖 `zircon_runtime::diagnostic_log`；不得直接依赖 private
`zr_diagnostics`。这项 facade 是批准的产品架构，不是保留旧实现。

## 5. 参考引擎复核

### Unreal（主要参考）

- `Logging/LogMacros.h` 把 category、verbosity、compile-time gate 和 source location 固定在低成本前端；
  `Misc/OutputDevice.h` 把输出目标作为独立抽象。对应 Zircon 的 level/filter 与 bounded sink owner。
- `TraceLog` 是独立 Runtime module；CPU profiler 又明确区分低成本 static event name 和更昂贵的 dynamic
  event，并在 channel enabled 后才写 per-thread begin/end。它不支持把 log、trace、metric store 和产品
  projection 合成一个“diagnostics”大 crate。
- 可吸收原则是：静态 identity、disabled fast path、输出 owner 与 trace capture 生命周期分离。不能照搬
  Unreal 常驻进程假设；Zircon 仍必须证明 dynamic runtime unload 后 worker 全部 join。

### Bevy 与 Godot（交叉验证）

- Bevy 分开 `bevy_log` 与 `bevy_diagnostic`；`DiagnosticsStore` 是 app/module resource，而不是低层 kernel
  内嵌后再由 logging crate 反向读取。descriptor 先注册，disabled path 不计算 value，写入可延迟批提交。
- Godot 的 `Logger`/stdout/rotated/composite output 与 `Performance` monitors 分属不同 owner；profiler 也有
  独立 registration/toggle/tick 生命周期。

三者共同支持“日志基础设施先形成物理 owner，metric/trace/product projection 分层演进”，不支持按当前
目录名做机械搬家。

## 6. 性能分析计划与批准门

当前已有 ignored gate `PERF-MVP-434`，覆盖 54 个组合：rate `1/1,000/100,000`，caller `1/64`，
scoped rules `0/10/1,000`，sink delay `0/10/100 ms`。固定压力配置为 queue capacity `4,096`、
max batch `256`、flush interval `25 ms`；验收检查 caller P95 `<= 50 ms`、RSS growth
`<= 128 MiB`、accepted/dropped conservation、queue bound、durability 与 thread affinity。

物理 hard cut 前先在相同 source fingerprint 与 Windows D/E/F target 上采集：

1. disabled/filtered/accepted 三条 write path 的 ns/event、allocations、lock wait 与 branch/filter cost；
2. 1/8/64 producer 下 queue depth、drop rate、batch fill、worker CPU、flush latency 和 shutdown deadline；
3. console off/file off、console only、file only 和 slow sink 下的 caller P50/P95/P99；
4. dynamic session 1/N acquire/release 与 DLL unload 的线程 census；
5. 同条件 pre/post 原始样本、机器/OS/CPU/存储 manifest、source manifest 与噪声区间。

只有 profile 证明瓶颈位于 filter lookup、global lifecycle mutex、queue contention、format/allocation 或 I/O
batch 之一后，才允许修改对应算法。结构目标是 disabled path `O(1)` 且不格式化/分配，accepted write
amortized `O(1)` enqueue，batch drain `O(records + bytes)`，空间有界于 queue/batch/settings；不批准无 profile
依据的 map 替换或细节微优化。没有 ETW/WPR 或外接功耗数据时，不声明功耗与 Unreal/Unity 接近。

本轮遵守跨 Session 短窗口协调，没有启动 Cargo 或性能工具。因而上述阈值是既有 gate 设计，不是本轮
GREEN 结果。

## 7. 原子 hard-cut 实施清单

正式 owner 在 kernel 顺序与 attribution 收敛后，必须把以下内容作为一个 candidate：

1. 新建 `zircon_runtime/crates/zr_diagnostics/{Cargo.toml,src/...}`，`publish = false`，只迁移第 2 节
   8-file production owner 与 17-file sink tests；由新 crate `lib.rs` 显式导出批准 surface。
2. 同批更新根 workspace、Runtime manifest 与 `Cargo.lock`；feature 映射必须保持当前
   `diagnostic-log` product behavior，禁止临时 default-on 双实现。
3. 重写 Runtime `diagnostic_log/mod.rs` 为显式 curated projection；Runtime-only store adapter 与 tests
   改为消费私有 crate API，不反向下沉。
4. 更新 Runtime root/prelude、dynamic-session lifecycle assembly、App/Editor/plugin consumers、docs、examples、
   API guards 和 crate-boundary guards。consumer manifest 必须是 literal + structured Rust use-tree 联集。
5. 同批删除旧八个 implementation 文件及其 implementation child paths；old implementation count、direct
   product dependency on `zr_diagnostics`、wildcard/compatibility alias 和 duplicate sink owner 全部为 0。
6. 保留 public surface seal：外部只见当前批准的 Runtime log API；dynamic lease、worker、state、compiled
   filter 与 raw output internals 不因跨 crate 编译而升级为产品 API。
7. 运行 focused sink/lifecycle/backpressure/durability/filter tests、54-case performance gate、managed Windows
   Runtime/App/Editor builds、plugin workspace gate、rustdoc/public API seal 和 dynamic DLL unload census。
8. pre/post build timing、runtime profile 与 power evidence必须绑定同一源码与机器 manifest；否则只记录
   correctness，不晋级性能或功耗状态。

任何单独“先复制 crate”、保留旧 module forwarding、只改 imports、跳过 Runtime adapter，或把 foreign dirty
blob 重新格式化后认领的 candidate 都应 fail-closed。

## 8. Ownership、阻塞与下一动作

coordinator baseline epoch 502 的精确 matrix 如下；所有列出的前缀当前 live lease 都是 0：

| 前缀 | matrix entries | executable attribution | attribution missing | 其余历史 attribution |
|---|---:|---:|---:|---:|
| `diagnostic_log` | 31 | MVP00 active 5 | 19 | stale/cancelled/archived 7 |
| `core/runtime/diagnostics` | 63（含 current tombstone） | MVP00 active 3；Runtime03 registered 2 | 54 | archived 4 |
| `runtime_diagnostics` | 4 | MVP00 active 2 | 2 | 0 |
| `core/runtime/handle/diagnostics.rs` | 1 | MVP00 active 1 | 0 | 0 |

无 live lease 不等于可直接改写：`diagnostic_log/{mod.rs,sink.rs}`、lifecycle 与 performance case/report
仍有 executable source attribution；MVP00/Runtime03 的 Runtime core blobs 同理。缺归属和历史归属 blob 也必须
先按 current hash 归因，不能由 Frameworks01 静默吸收。当前既没有完整 current-hash ownership union，也没有
M1 所要求的 `zr_kernel` 物理前置，因此不开始 production edit。

2026-08-29 coordinator baseline epoch 548 的 `diagnostic_log` current-hash matrix 进一步确认：32 个 entry
均无 live lease；5 个 blob 仍指向 active MVP00 attribution，7 个指向 archived/cancelled Runtime44、
plugins/optimization-era attribution，20 个缺 attribution。特别是
`mod.rs`、`sink.rs`、`sink/tests/lifecycle.rs`、`sink/tests/performance/{case,report}.rs` 仍关联 active MVP00，
而 current hash 已被 coordinator 标记为 baseline/hash stale 或 missing lease。新增
`level/borrowed_parse_tests.rs` 与当前 `level.rs` 虽带 Runtime150 source identifier，却没有可执行
attribution。该矩阵只证明 hard cut 必须重新做 owner transfer，不构成 Frameworks01 对任何 production
blob 的认领。

同轮只读 guard：

- `python -B -m unittest tools.tests.test_frameworks_01_runtime_diagnostics_boundary -v`：2/2 通过，39.150 秒；
- 未启动 Cargo、profile 或 build；当时 coordinator 另有 Runtime22 managed Cargo lease，Frameworks01 没有
  抢占共享 build 窗口；
- guard 仅证明 manager-resolving collector 仍由 Runtime facade 拥有、consumer 未回流 retired core path，
  不证明 logging source move、性能 gate 或产品编译通过。

合法下一动作按顺序是：

1. 完成或接收 `zr_kernel` hard cut 的依赖方向与 facade seal；
2. 先核对 Runtime44/Runtime150 source identifier 与 coordinator 中 missing/stale attribution，并由 MVP00
   等当前 executable owner 固化或转移上述 current blob，再对 32-file log tree、
   Runtime adapter、manifests/lock、roots、lifecycle consumers、guards/docs 重新生成
   structured atomic manifest；
3. 在协调器上对每个 dirty blob 重取 current hash，preview/apply transfer；任一 executable foreign owner
   或 hash drift 都返回 preflight；
4. 先跑 pre-cut managed performance baseline，再执行一次性 source move；
5. correctness/product/performance gates 全部有 receipt 后，才创建 integration candidate、milestone commit 和
   企微量化通知。

本记录完成的是可执行边界和准入条件，不是把验收队列当作唯一工作项。当前可落地的非验收产出是：
阻止错误的 whole-directory move、冻结单一 owner 设计，并把后续优化限定到 profile 证明的结构瓶颈。
