---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: native-callback-per-call-lease-and-abi-copy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
tests:
  - 1/2/16/64-thread same-plugin 1M callback lease benchmark
  - transition-vs-callback epoch and dynamic-library lifetime stress
  - precompiled command identity and large output ownership matrix
---

# Plugins01：native callback每调用lease锁、诊断RMW与ABI payload复制

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260722-performance-mvp-audit`
- 来源执行切片：native plugin loader/live-host非验证生产路径逐文件审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`，ABI output合同联动`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`，任务预算联动Runtime11。

## 失败现象与复现证据

原`native-plugin-callback-global-lock` failure已把foreign callback移出全局loaded-table mutex并使用稳定library owner，解决了慢插件阻塞全表和重入自锁。但`NativePluginStableLibrary::acquire_callback`仍为每次callback获取per-plugin `callback_state: Mutex`、增加active count，lease drop再次获取同一Mutex；无论diagnostics是否被消费，每次还更新lock-wait、completed、total duration与max duration四类共享Atomic。native registration replay system每frame调用bridge/command时，同plugin多worker会争用这一把锁和同一组cache line。

`NativePluginBehaviorCallbacks::invoke_command`还为每次`&str` command构造`CString`；output由plugin分配`NativePluginOwnedByteBufferV2`，host完整`to_vec()`后才调用plugin free。大state/asset command形成plugin buffer+host Vec双owner峰值，caller若是主线程会直接承担分配、复制和free callback wall。

## 最低共享层根因

动态库quiescence、callback activity accounting、profiling diagnostics与ABI payload ownership尚未拆层：稀有reload/unload transition使用的排他状态被放入每call mutex；诊断没有off/sampled/sharded模式；command identity与output allocator没有load-generation编译/host-owned contract。

## 架构修复验收

- PERF-MVP-541：stable library generation使用atomic epoch/transition bit与in-flight lease，stable callback acquire/release不获取Mutex；reload/unload先关闭新lease，再拒绝或等待旧epoch自然归零，最后一个in-flight owner释放前不得卸载DLL。
- diagnostics off时除必要in-flight安全计数外近零；on时用thread-local/sharded或有界采样聚合duration/wait，不让每call写四个全局共享原子。
- PERF-MVP-542：manifest command在load generation编译为dense slot与稳定NUL-safe identity；stable dispatch不构造CString。ABI升级为caller-provided bounded output sink/buffer，或定义可安全transfer的统一allocator ownership；Windows跨CRT不得用未经协商的裸Vec接管。
- 参考Godot `gdextension_interface.cpp`的`StringName` identity、`classdb_get_method_bind`预解析和`object_method_bind_call`执行分层；Zircon保留typed status、panic containment、free failure diagnostics与version negotiation。

## 验收矩阵

- callbacks：1/2/16/64 threads × 1M same-plugin calls，记录callback-state mutex acquire/wait、shared RMW、throughput/p50/p95/p99、active count与generation age；stable mutex=0。
- lifecycle：callback进入/退出与reload/unload/bulk replacement交错，验证新lease拒绝、旧lease完成、rollback、reentrant descriptor、DLL lifetime和deterministic broadcast order。
- commands/output：1/1k/1M commands，payload/output 0/1KiB/1MiB/256MiB，记录CString alloc、allocator crossings、copied bytes、peak RSS与caller-thread wall；stable name alloc=0，成功output最多一个authoritative owner和一次必要copy。
- current-source native fixture、runtime/plugin broad Cargo、F2/F4产品trace与独立复审通过后，写fixed return；此前保持open。

## 禁止临时方案

- 不得换成RwLock或递归Mutex；transition是稀有写慢路，stable callback不得保留排他锁。
- 不得关闭全部diagnostics来掩盖成本，也不得让每worker创建无界私有统计表。
- 不得跨DLL直接`Vec::from_raw_parts`接管未知allocator内存，或同时保留CString热路径和dense slot第二套权威。

## 修复结果与回传

当前状态：`PERF-MVP-541 managed_focused_green / product_trace_pending`；
`PERF-MVP-542 current_source_loader_and_real_fixture_wired / managed_dynamic_acceptance_pending`，
因此本 failure 保持 `open`。

### 2026-07-22 atomic generation lease

- `Arc<NativePluginStableLibrary>` 现在就是一代 DLL owner；单个 `AtomicUsize` 的最高位为
  lifecycle transition，其余位为 in-flight lease。acquire 通过 CAS 线性化，drop 只执行
  `fetch_sub`；transition 只允许 `0 -> transition-bit`，因此旧 lease 与关闭新 admission 不存在
  检查/递增竞态，stable path 不再取得 per-plugin mutex。
- callback duration/count 默认写入 64 个 cache-line-aligned shard，线程首次使用时取得固定 shard；
  snapshot 冷路径才聚合。`set_callback_diagnostics_enabled(false)` 后仍保留 quiescence 必需的
  activity 原子，但不调用 `Instant::now()`、不写 completed/total/max 诊断原子。
- `NativePluginCallbackDiagnostics` 明确暴露 diagnostics mode、shard count 与
  `callback_state_mutex_acquisitions = 0`；历史 `lifecycle_lock_wait_ns` 保留为兼容字段并恒为 0。
- 测试从接近 1K 行的 `runtime_behavior.rs` 拆到独立 `tests/callback_lease.rs`，覆盖 source guard、
  active-count busy、transition 拒绝新 lease、diagnostics off/on、64-thread race，以及 ignored
  `1/2/16/64 threads x 1M` same-plugin lease benchmark。静态 source guard 已先 RED（Mutex、off switch、
  zero-mutex proof 三项缺失）再 GREEN；scoped rustfmt 使用 `skip_children=true`，避免触碰其他会话
  正占用的 hot-reload test owner。

64-thread/1M 与 reload/unload/rollback focused/broad 数据已在下文补齐；尚需产品 trace 后才能把
PERF-MVP-541 写 fixed return。PERF-MVP-542 的 runtime loader 与动态 fixture 已有 current-source 接线；
规模矩阵与 managed 动态验收仍按本 failure 验收矩阵继续执行。

合并 current-source Windows Cargo reservation `443ed28a879c42228127596358928d6a` / job
`a2f858fdd9894cb88df122fb92780da9` / run `255a862363d74b699f0b23cf308dfe3b`
已自然 `exit 0`：完整 core-min lib-test 构建耗时 19m53s，过滤组
`5 passed / 0 failed / 1 ignored / 4373 filtered`。64-thread transition/lease race、diagnostics off、
零 callback-state mutex source guard、atomic owner 指标与 descriptor 重入均通过。ignored 的
`1/2/16/64 threads x 1M` benchmark 随后直接执行同一不可变二进制并 `1 passed / 0 failed`：
每档总计 1,000,000 次 lease，吞吐分别为 10,016,607.54、10,051,059.38、4,829,928.56、
5,117,741.32 leases/s，四档 `state_mutex_acquires=0`。reload/unload/rollback broad 及产品 trace 仍待执行；
PERF-MVP-542 已有 current-source runtime/fixture 接线，managed 动态验收仍未完成，因此本 failure 保持 open。

同一 `r9` 二进制的 lifecycle/broad focused 回归继续通过：host API/context `22/0/1 ignored`，
runtime behavior `11/0/1 ignored`，hot reload/rollback `12/12`。两个 ignored 门亦分别 `1/1`：
context lookup 在 1/16 线程下各 1,000,000 次，吞吐约 1.10M/4.98M lookups/s、p95 800ns、
p99 900ns、writer acquire 0；broadcast 在 1/8/32 plugins × 100 iterations 下完成
100/800/3200 callbacks。上述证据进一步关闭 PERF-MVP-541 的 reload/rollback 与稳定查找疑点，
但不替代产品 trace，也不覆盖 current-source runtime 接线之后仍待完成的 PERF-MVP-542 managed 动态验收。

### 2026-07-22 PERF-MVP-542 SDK 与真实 fixture 第一段

- descriptor/entry symbol 继续保持 native ABI v3；entry-report layout epoch 提升到 5，behavior table
  hard cut 为 ABI v4。`NativePluginBehaviorV4::invoke_command` 接收 load-generation 编译后的 `u32 slot`，
  不再接收 command C string；command identity 因此允许内部 NUL，stable dispatch 不需要构造 `CString`。
- SDK 新增 schema v4 command manifest（dense `slot == index`、唯一非空 name、非空 payload schema、
  每命令 `max_output_bytes <= 256 MiB`）以及 `NativePluginOutputSinkV4`。插件只能调用 host 提供的
  bounded write callback；命令结果的 authoritative `Vec<u8>` 由 host 直接拥有，消除 Windows 跨 CRT
  plugin allocation/free 与完整 plugin-buffer-to-host-Vec 复制。
- `native_dynamic_fixture` 的 runtime command table 已固定为四个 slot：echo、4-byte bounded overflow、
  panic、asset import；echo/asset 直接流写 host sink，overflow 特意写第 5 byte 验证累计上限，panic
  保留 containment 覆盖。editor behavior 使用空 v4 manifest；旧 `mismatched_buffer` 与 name-based C ABI
  dispatch 已从 SDK/fixture 源码移除。
- TDD 静态 guard 先因缺少 behavior v4、output sink、manifest v4、slot callback 四项 RED，实现后 GREEN；
  独立 Rust 1.94.1 source probe 直接编译实际 `plugin_sdk/src/native.rs` 并执行 `9 passed / 0 failed`。
  scoped rustfmt、SDK/fixture 旧符号搜索与 `git diff --check` 通过。该探针只证明 SDK native contract，
  不替代 `dist.rs`、runtime ABI mirror、loader manifest、host sink 或真实 DLL 的 current-source 验证；
  规模矩阵与 managed 动态验收尚未完成，所以不得把 PERF-MVP-542 或本 failure 写为 fixed。

导出消费者的合同字符串也已同步 hard cut 为 `NativePluginBehaviorV4` 与
`NativePluginBehaviorV4.save_state/restore_state`；Python compile 与 `git diff --check` 通过，
定向 native-dynamic 13 项及 package-report schema 20 项均 GREEN。全量发现式 native-dynamic
工具矩阵为 `486 passed / 1 failed`（487 项，440.105 秒）；唯一失败来自未改动的
`test_pipeline_report_native_dynamic_payload` 仍生成已被 current compile-host schema 拒绝的
`link_plan`、空 `staged_engine_root` 与旧 Cargo command，诊断不含本次 behavior V4 字段。
该外部 fixture drift 不计为 PERF-MVP-542 通过或失败，留给其 compile-host/schema owner 收敛。
