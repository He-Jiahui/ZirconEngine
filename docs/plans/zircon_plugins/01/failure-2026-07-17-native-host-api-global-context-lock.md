---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-host-api-global-context-lock
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
tests:
  - native host context registry 1/16-thread benchmark
  - concurrent scope-drop and in-flight call lifetime test
  - stale/wrong-kind handle status regression
---

# Plugins01：native host API context registry 全局锁

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：native host bridge ABI 热路径完整静态复读
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 共同验收：Runtime06 native lifecycle/ABI 并发与压力测试
- 交接原因：handle allocation、lookup、scope drop、reload generation 与 callback/dynamic-library lifetime 必须共同收敛，不能缓存裸 context 指针。

## 失败现象与复现证据

性能审计已把 `NativeHostApiV3Context::BridgeCall` 改为 `Arc<NativeHostBridgeCallContext>`，使
`bridge_context_for` 不再逐 call 深 clone method `BTreeMap`。但每次 `native_host_bridge_call_v1_inner` 仍调用
`lock_contexts()`：一个进程级 `OnceLock<Mutex<BTreeMap<u64, NativeHostApiV3Context>>>`。

所有 native plugins、registration scopes、bridge scopes 与 calling threads 共享这把 mutex；稳定 handle 的每次调用
都独占锁、做 BTreeMap lookup、clone Arc 后才锁外 dispatch。高频多线程 bridge workload 仍会在 context registry
串行，只是锁持有时间比旧实现短。

## 最低共享层根因

Host handle 被设计为单调 u64 + 全局可变 map，没有 slot generation 与并发读 owner。scope drop 通过 map remove
表达失效，因此每次 call 都必须进全局锁确认 lifetime。

## 架构修复验收

- 使用带 slot generation 的 handle；lookup 能并发读取并取得 Arc context，注册/remove/reuse 走有界写慢路。
- scope drop 原子阻止新的 lookup；已经取得 Arc 的在途 call 完成前，其 method table、bridge table 与动态库 owner 有效。
- stale/reused handle 必须返回 `NotFound`，registration handle 调 bridge 继续返回 `UnsupportedVersion`。
- 1 与 16 threads 各 1M stable calls，记录 context registry exclusive mutex acquire/wait=0、lookup p95/p99 和吞吐。
- 并发 drop/reload/panic、disabled interface、missing method 与 method output buffer 合约全部回归。

## 禁止临时方案

- 不得把 u64 handle reinterpret 为裸指针或让插件持有 Rust Arc 指针。
- 不得永不 remove context 以换取无锁读取，造成 generation 混淆和内存泄漏。
- 不得把 method callback 调用移回任何 registry 写锁内。

## 修复结果与回传

当前状态：`implementation_complete / current_source_validation_queued`。

### 2026-07-22 current-source 实现

- `host_api_adapter/context_registry.rs` 以 `ArcSwap<Vec<Arc<Slot>>>` 发布 immutable slot table；稳定 lookup
  只做 generation acquire、`ArcSwapOption::load_full` 与二次 generation acquire，不取得 writer mutex。
- handle 编码固定为高 32 位 generation、低 32 位 one-based slot。remove 先原子清空 context，再推进 generation；
  generation 到 `u32::MAX` 时永久 retire slot，禁止 wrap 后旧 handle 复活。
- allocation/reuse/remove 只走 writer mutex 慢路；scope drop 后新 lookup 返回 `NotFound`，已取得的 Arc context
  继续固定 bridge table、method table 与 callback library owner，直到在途 call 返回。
- bridge call context 使用共享 Arc，不 deep clone method `BTreeMap`；registration 与 bridge context kind 仍由同一
  typed enum 区分，wrong-kind bridge call 保持 `UnsupportedVersion`。
- 已新增 stale generation/reused slot、scope drop + in-flight call、16-thread stable lookup writer-acquire=0，
  以及 ignored 1/16-thread、各 1M lookup throughput/p95/p99 benchmark。bridge scope owner pin 测试同时证明
  active owner 阻止 unload，纯 pin 不污染 callback completion/duration。

Rust `1.94.1` scoped rustfmt、`git diff --check`、native public-surface 68/68 与 lifecycle risks `[]` 静态门均通过。
managed job `93f88e221e244b93b176afa90a07cdff` 保留的 current-source test binary（SHA-256
`0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）执行完整
`context_registry::tests` 过滤组为 `3 passed / 0 failed / 1 ignored / 4306 filtered`。忽略的
1/16-thread benchmark 随后单独通过 `1/1`：两组均为 `1,000,000` lookups、
`writer_acquires=0`；1 thread 为 `1,204,450/s, p95=700ns, p99=900ns`，16 threads 为
`8,010,227/s, p95=800ns, p99=900ns`。

broad native-host/bridge parity、failure return 与 owner milestone 记录完成前，本 failure 保持
`open`。

### 2026-07-22 append与dispatch剩余放大

原全局lookup Mutex已清零且现有1/16-thread证据有效；本轮复核新增PERF-MVP-545：`HostContextRegistry::insert`每次追加新slot仍clone完整`Vec<Arc<Slot>>`发布目录，批量registration/bridge scopes近O(H²) Arc refcount流量。stable bridge call取得outer context Arc后又clone variant内inner `Arc<NativeHostBridgeCallContext>`，method dispatch仍查`BTreeMap<(u32,u32),Fn>`。

最终验收补充chunked/page-table slot directory：append只替换末页或小目录，slot唯一Arc直接pin context，method table使用PERF-MVP-543同代dense slot。contexts/methods 1/100/10k与1M calls记录directory Arc clones、refcount ops、generation loads、tree probes和p95；append近O(H)，stable call单context pin+dense lookup。broad parity、该规模门与failure return前继续open。

### 2026-07-30 current-source 恢复状态

- `plugins01-native-host-context-validation-r1-20260730` 已取得 host API adapter、page-table
  registry、host tests、registration replay、bridge methods 与本 handoff 的精确六路径租约。
- 当前源码已经包含 `HostContextRegistry` 的 page directory、generation double-check、stale handle
  regression、in-flight Arc lifetime、1/100/10k append metrics、16-thread writer-acquire guard，以及
  `DenseBridgeMethodTable` 的 sparse-slot contract。scoped Rust `1.94.1` formatting、`git diff --check`
  和 page-table/dense-dispatch static contract 已通过。
- 上述检查不能替代编译或性能验收。先前 job `93f88e221e244b93b176afa90a07cdff` 的 binary 与数字只保留
  历史诊断；新的 current-source evidence 必须经 Windows coordinator validator 按 FIFO 执行 host-context
  focused group、ignored 1/16-thread benchmark 和 native-host/bridge broad gate，再复核与 fixed return。
- 当前 managed Cargo lane 由 Frameworks04 占用；本 failure 保持 `open`，并等待新的受管终态，不重建或复用
  已撤销的 validation-copy。
