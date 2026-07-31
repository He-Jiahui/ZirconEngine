---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: bridge-import-stable-call-double-mutex
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/bridge/import.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs
tests:
  - single and 16-thread stable bridge-call benchmark
  - concurrent reload/disable generation race test
  - provider lifetime and poison-recovery regression
---

# Plugins01：稳态 BridgeImport call 每次双 mutex

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：runtime plugin bridge 稳态调用静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 共同验收：Runtime06 插件生命周期与并发 bridge 基准
- 交接原因：binding publication、provider generation、reload/unbind 与调用期 provider lifetime 必须统一设计，不能删除单把锁制造悬垂 provider。

## 失败现象与复现证据

`BridgeImport::call` 先通过 `lock_binding()` 获取 `Mutex<Option<WeakBridge<T>>>`，clone `WeakBridge` 后释放；
`WeakBridge::provider_with_slot` 再获取 `cached: Arc<Mutex<Option<(generation, Arc<T>)>>>`。即使 generation
长期不变，调用仍需两次 mutex acquire，并 clone cached provider Arc。多 worker 高频调用同一 import 时，cache mutex
成为共享串行点；`is_enabled`/`pin` 也进入相同 provider cache 路径。

Provider reload/disable 本身已有 atomic generation，但 stable call 没有直接消费一个可无锁读取的 generation/provider
snapshot。当前锁保证了 provider lifetime 与 poison recovery，不能简单删除。

## 最低共享层根因

Bridge import binding 与 weak provider cache 都以“低频可变容器”建模，没有把 catalog freeze/reload 转换成一次发布、
多次读取的 immutable generation snapshot；读写比例极端偏读却仍使用独占 mutex。

## 架构修复验收

- catalog freeze/reload/unbind 发布带 generation/status/provider Arc 的不可变 snapshot；stable call 快路不获取独占 mutex。
- generation miss 进入有界慢路刷新 snapshot；并发调用持有的旧 provider Arc 在完成前保持有效。
- disabled/not-enabled diagnostics 与 generation 语义不变；reload 不得把旧 callback/provider 提前释放。
- 1 与 16 calling threads 各 1M stable calls，记录 mutex acquisition/wait、throughput、p95/p99；快路无全局独占串行。
- 并发 reload/disable/unbind、poison recovery、`pin` guard lifetime 和 absent/disabled 错误测试通过。

## 禁止临时方案

- 不得缓存裸指针或绕过 Arc lifetime。
- 不得只把 `Mutex` 改成 `RwLock` 就宣称完成；必须用指标证明 stable 多读不再被单写锁路径串行。
- 不得把 generation 检查移除或让 disabled provider 继续可调用。

## 修复结果与回传

当前状态：`implementation_complete / current_source_validation_queued`。

### 已完成项目

- `BridgeImport` 的 binding 发布改为 `ArcSwapOption<WeakBridge<T>>`，稳态 `call`/`is_enabled` 不再获取 binding mutex。
- `BridgeEntry` 将 generation、enablement 和 provider 合并到单个不可变 `BridgeEntryState`，通过 `ArcSwap::rcu` 原子发布，避免 generation/provider 分离读导致错配。
- `WeakBridge` 缓存改为按 generation 发布的弱 provider snapshot；慢路刷新仍返回并持有 `Arc<T>`，旧 provider 可安全存活到在途调用结束，同时缓存本身不会延长 provider 生命周期。
- 新增 9 个行为/性能回归，覆盖 1/16 线程稳定调用、reload/disable 竞态、unbind 在途调用、provider 释放、generation 推进与 callback panic 恢复；并同步更新既有 tracked performance baseline，避免旧 Mutex 源码断言伪 RED。
- 性能门使用同机同调用量的 16-thread Mutex serialized control：actual/control 均以 ready/start 双 Barrier 对齐，lock-free 吞吐须至少为 serialized control 的 `1.10x`；另以 `4096/thread` 独立 pass 记录真实 acquisition/wait，不污染吞吐比值。import/weak/table/diagnostics 完整本地快路的 exclusive-wait token 计数必须为 `0`。
- 并发 reload 测试按调用前后 generation window 核对 provider payload，并在每次完整发布后精确断言 `generation == 2 * (value - 1)`，可检出稳定代 torn provider/generation。
- `zircon_runtime` 已接线根工作区既有的 `arc-swap` 依赖；`FrozenBridgeTable::resolve_weak<T>` 使用显式 `WeakBridge::<T>`，修复 managed RED `feef12b0258748eda07e3c630d732585` 中的 `E0432`/`E0283` 最低层根因。
- 2026-07-22 的 fresh locked upward gate job `8e229f6cd2c749f495b0f701e0c07bc0` / run `b410b0de35d14f2d9980be50241c640e` 在根 lockfile 恢复后暴露 `import.rs:66` E0631：`ArcSwapOption::load().as_ref()` 投影为 `&Arc<WeakBridge<T>>`，不能直接接受 `WeakBridge::is_enabled` 函数指针。`BridgeImport::is_enabled` 已改为 `as_deref()` 后再执行相同 predicate，保留无 clone、无 mutex 的 stable read 语义；scoped rustfmt、diff check 与 deref source contract 均通过。
- current-source owner gate job `378142f7f96e45baa800695a357b2002` / run `6b2c040c395a4adfb76ec31ea0dd6a8a` 执行 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1`，exit `0` 并由 coordinator 自动 release；同一 `import.rs` snapshot `29730aadb131bbb404ec1941c24f6f0895aed2b89b2e728c4b5ddbd489826f0a` 通过 owner feature 编译。Layout15 warm upward gate 也确认 E0631 消失，随后仅被外部 Text01 `font_assets.rs` E0502 截断。
- 共享 catalog 格式化更新 `bridge_lifecycle.rs` 后，当前精确 11-file source manifest fingerprint 为 `8cbe96ba0fb4482d0dd3cd8d9236804420f5d9ee62004b292705f710df800079`；既有 `rustfmt --check` 与 scoped `git diff --check` 通过，生产/测试文件均未超过结构预算，fresh gate 将覆盖合并后的当前字节。
- 四轮独立只读复审逐项关闭旧 performance baseline、generation/provider 配对和公平性能证据问题，最终为 Critical `0` / Important `0` / Minor `0`。

### 待完成项目

- 2026-07-26 coordinator 只读审计发现本 artifact 引用的 `93f88e221e244b93b176afa90a07cdff` 实际为 native discovery 测试，不能证明 bridge stable snapshot；因此本文件中所有历史 managed binary、吞吐与 broad 结果仅保留为诊断背景，**不再作为当前源码验收证据**。当前恢复 owner 已认领 bridge 源与回归测试，必须先物化新的不可变 validation-copy，再重新执行 9 项 focused/performance gate、bridge/plugin broad gate、独立复核和 failure return；在这些终态证据齐备前保持 `open`。
- 修复/复审期间的旧 reservations（含共享格式化后 stale 的 `5d7b18bf5ef44884ac4c263d8e116914`）已由 owner 释放；final 11-file Rust 1.94.1 reservation `7da5bb5a88ad4789b168cc7b6900057c` 已按当前字节重新登记并按 FIFO 等待，不直跑 Cargo。
- focused GREEN 后仍需 bridge/plugin broad gate、review 登记、failure fixed return 与 coordinator milestone atomic commit。
- current-source owner gate 已确认 E0631/E0308 均消失；仍需执行本 failure 要求的 stable-call focused/performance matrix、登记复审、fixed return 与 milestone commit，不能把 core-min compile GREEN 代替性能验收。
- 同一外部 RED 中的 `runtime_plugin_catalog/feature_blocking.rs` `E0505` 已由现有 catalog owner 以一次性 owned unresolved-key set 修复并另行 source-bound；bridge owner 未吸收其 manifest。

上述旧 reservation 未产生终局证据；以下更新的 immutable managed binary 执行结果取代它。

### 2026-07-22 历史 focused 与性能记录（已撤销为当前源码证据）

> 2026-07-26 coordinator audit 已证实下列 job-id 与其声称的 Bridge 测试范围不一致；数字和输出仅保留用于追溯先前诊断，不能支持 `GREEN`、性能阈值、broad gate 或 fixed return。新的恢复验证将以 validation-copy 的 source manifest、命令、终态 run 和独立复核为唯一依据。

managed job `93f88e221e244b93b176afa90a07cdff` 保留的 test binary（SHA-256
`0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）已执行
stable snapshot 的 9 项回归：8 项功能/竞态测试通过，`stable_bridge_calls_scale_across_one_and_sixteen_threads`
单独通过 `1/1`。性能数据：

- 1 thread / 1M calls：`664,758/s, call p95=1.4us, p99=2.5us`；
- 16 threads / 16M calls：`5,386,322/s, call p95=1.7us, p99=2.1us`；
- stable path `exclusive_wait_sites=0`；16-thread serialized Mutex control 为 `508,449/s`，
  65,536-call wait sample 为 `65,536 acquisitions / 1.7474196s total wait`。

因此 lock-free 实测吞吐约为 control 的 `10.59x`，超过 `1.10x` 门限。broad bridge/plugin
gate、failure fixed return 与 milestone review 仍待完成，因此保持 `open`。

`tests::plugin_extensions` core-min broad 执行到 `392 passed / 17 failed`；其中两条 bridge
performance baseline 失败属于源码守卫漂移：native call path 已从旧
`interface_snapshot(slot)` 收敛为同样 pre-resolved 的 `entry(slot)`，VM helper 已是 generic
`function_callback<Table>` / `ensure_bridge_enabled<Table>`。守卫现在锁定当前 typed entry 与
generic helper，仍拒绝 runtime `.resolve_slot(...)` / name lookup。scoped rustfmt、diff-check 与
source-token scan 已通过；fresh 编译后需重跑这两项及 broad gate。

### 2026-07-30 current-source 恢复状态

- 恢复会话 `plugins01-bridge-current-validation-r1-20260730` 已按精确 10-file scope 取得 source、
  regression 与本 handoff 的租约；没有重建、重试或清理 2026-07-26 已撤销的 historical validation-copy。
- 当前字节已经以 Rust `1.94.1` 重格式化并通过 scoped `rustfmt --check` 与 `git diff --check`；
  scoped diff 中只有 Git 的 CRLF 转换告警，没有 whitespace error。
- import/weak/table/diagnostics 的 stable-path token 审计为
  `exclusive_wait_token_count=0`，覆盖 `Mutex`、`RwLock`、`Condvar`、`parking_lot`、
  `spin_loop`、`yield_now` 与 `.park(`；这只证明源码快路形状，**不是**功能或性能 GREEN。
- Windows managed Cargo lane 仍由 `frameworks04-native-entry-fixture-r8-20260730` 占用。下一个
  当前源码证据将通过 coordinator validator 按 FIFO 运行 stable-snapshot 9 项与 performance baseline，
  之后才安排 bridge/plugin broad、独立复核和 fixed return；本 handoff 保持 `open`。
