---
related_code:
  - zircon_runtime/src/diagnostic_log
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceRedirector.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceFile.cpp
tests:
  - current 31 of 31 Rust files and 44 tests reviewed
  - M0 static performance contract 4 of 4 passed
  - focused rustfmt plus 1.94.1 and scoped diff check passed
  - added Rust backpressure behavior test not executed
  - current-source Windows Cargo and PERF-MVP-434 ignored matrix blocked
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime diagnostic_log 有界sink当前架构审查（2026-08-23）

## 范围与当前性

已按HEAD `9fee3ea0435961a81c85aa2502e64f1f357345d7`完整阅读`zircon_runtime/src/diagnostic_log/**` **31/31** 个Rust文件、**4,096** 行、**133,770 B**和 **44** 条测试；全范围manifest SHA256为`a365b5437cd6be6bb6fbcba8c66301bf6c0c039f7f00a6375edc9864454b5f56`。其中生产10文件/2,305行/74,354 B，SHA256 `384f8d04c9b4f84c25dded7f850c7576a60b8f4cc03ba39bb00abc3d24cdfeb5`；测试21文件/1,791行/59,416 B，加上生产文件内联测试合计44条。当前4个dirty文件的已有diff均是`rustfmt`导入/换行整形，本轮保留。

旧报告的主要根因已有真实结构修复：scope rules在启动时编译，调用点提供lazy gate，单一bounded worker按count/bytes/time批处理file/console I/O，并且flush/shutdown/crash边界有显式ack、`sync_data`与counter。caller不再持文件锁、写盘或逐条flush。因此不应继续照抄旧问题，本报告只记录当前剩余工作。

## 当前瓶颈与M0

1. `SinkRuntime::enqueue_lazy`在channel已知满时仍先执行message closure、clone scope/message，然后`try_send`丢弃verbose/debug/log。100k/s慢sink场景会把注定丢弃的`format!`留在runtime/editor caller。M0在active-sender/closed gate后对best-effort级别做known-full preflight；非空race语义不变，已知满时lazy closure调用由1降为0。
2. `flush_pending`对每条record调用`chrono::Local::now().format().to_string()`，但格式只有秒精度，默认50 ms批内的N次时区/格式化是worker冗余。M0以批次接收时间为语义，降为1次/批；这不会降低当前秒精度，也不把wall-clock成本搬回caller。
3. 0条scope rule是默认常见路径，当前compiled trie仍对scope首字节执行一次空`HashMap<u8, usize>` lookup。M0在只有root node时直接返回minimum，默认日志的hash probe由≥1降为0。
4. `send_control_until`在queue满时一直`try_send + thread::yield_now`到deadline；flush/panic/shutdown边界可以在慢sink下消耗一个CPU逻辑核。M0改为crossbeam的deadline-bounded `send_timeout`，保留FIFO和超时终态，忙转循环由1降为0。

## P0/P1结构剩余工作

- warn/error与best-effort共用一个queue，满时每条critical在caller线程最多等待2 ms，K条连续critical的上界为`K*timeout`；对16.67 ms帧预算，单条已占12%。目标是severity-reserved admission/coalesced overflow receipt，frame/main lane的critical queue wait为0，crash durability交给预留通道与显式flush，不用无界队列。
- 每条允许日志仍在caller拥有两个`String`并对scope逐字节hash；最终需稳定scope/category ID、结构化record与worker-side formatting，关闭路径alloc=0、接收路径不重复scope owner。
- `flush_pending`每批新建output `Vec<u8>`，应由worker保留scratch buffer并在批后`clear`；需用allocator counter证明warm steady batch allocation=0后验收。
- 最后dynamic session `Drop`可在lifecycle mutex内等待2 s worker join，`active_senders`和并发shutdown等待仍用`yield_now`。最终要显式log generation owner、event/completion通知、锁外join；drop/main-thread wait为0，DLL unload仍必须证明worker已退出。
- 54-case storm gate仍`#[ignore]`，caller P95预算是50 ms，未单独限制main/frame lane的p95/p99/max、critical wait sum、timestamp calls、message/scope alloc与batch scratch alloc。动态执行前不宣称性能达标。

## Unreal源码依据

`dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceRedirector.cpp:465-508,827-938`使用dedicated primary logging thread、MPSC buffered items和event wake，只在显式同步flush时等待idle；`OutputDeviceFile.cpp:73-135,209-310`让writer thread消费ring buffer并按间隔flush，客户线程只写内存buffer与通知；`554-594`只在显式`FORCELOGFLUSH`时逐行flush。Zircon应继承“单sink owner、事件唤醒、内存缓冲、周期/显式flush”，不照搬UE的无界队列或C++全局owner。

## 验收

先以focused Rust行为门和静态contract锁定M0；然后对1/1k/100k logs/s、1/64 callers、0/10/1k scopes、0/10/100 ms sink执行54-case matrix，补main/frame lane p50/p95/p99/max、critical wait sum、format/timestamp/hash/alloc、queue age/depth/drop、worker CPU/wake、RSS与功耗。F0/F2 current-source executable用WPR/xperf证明main thread file I/O/flush=0、idle sink wake接近0；current Cargo与产品trace未通过前继续留在`pending.md`。

## 2026-08-23 M0实施证据

- 新增`full_queue_does_not_evaluate_best_effort_lazy_message`真实Rust行为门，锁定known-full debug日志不执行lazy closure且drop counter加1；受管Cargo不可执行，该Rust测试本轮只是已写、**未运行**。
- `tools/tests/test_runtime_diagnostic_log_m0_performance_contract.py`在实现前4/4 RED，实现后4/4 GREEN；53行、2,128 B，SHA256 `1d40ca8b7c579d26867dd3dc1ed9188a564837baacb34a82b92f48c39a3a0bc4`。focused `rustfmt +1.94.1 --edition 2021 --check`与scoped `git diff --check`通过。
- 实现后范围为31文件、4,121行、134,682 B、45条测试，manifest SHA256 `fbd81f9d57578b0f86385f94ba55e9782a8d304059021a2bd6818977540e71dc`。静态操作数：known-full best-effort closure `1 -> 0`，0-rule hash probe `>=1 -> 0`，N-record batch timestamp format `N -> 1`，control queue-full busy-yield loop `1 -> 0`。这些不是wall-clock、CPU或功耗实测，不用于声称已接近Unreal经验值。
