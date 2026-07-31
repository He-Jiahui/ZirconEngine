---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/tracy.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - nine production Rust files reviewed
  - inactive payload regression test added
  - source-level RED to GREEN performance guard passed
  - rustfmt and scoped diff checks passed
  - current-source Cargo, allocation counters and product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core profiling逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime/diagnostics/profiling/**`生产Rust文件9/9，当前2397行。范围覆盖feature/macro入口、CPU scope/frame/counter recorder、Tracy bridge、bounded sample ring、CPU/counter/UI hotspot聚合、Perfetto/native/Markdown导出及其inline测试。

## PERF-MVP-326：关闭采集仍制造全局锁与分配

此前只要构建启用`profiling` feature，即使recorder未开始采集，每个静态scope也会先分配name String，动态scope会执行`format!`/clone并构造path，scope/frame/counter最终都争用单个`GLOBAL_RECORDER: Mutex<ProfileRecorder>`后才发现inactive。渲染graph stage/pass和ECS schedule等高频调用因此可在“没有采集数据”时仍改变被测程序的CPU、allocator与锁争用。`next_frame_index`还以`entry(stream.to_string())`查询，使稳定stream每帧分配临时键。

本轮先增加由start/stop/reset同步的Acquire/Release active hint：静态scope在name分配前返回，frame/counter在TLS/recorder前返回；非Tracy动态scope与counter macro把payload求值放进active分支，Tracy组合仍按其独立sink语义求值，但内部recorder只在active时clone；frame stream先借用查表，仅首次出现时拥有key。新增inactive macro回归测试代码并完成源码RED→GREEN、rustfmt与scoped diff检查。

## 剩余根因

活动采集期间所有线程的scope begin/finish、frame与counter仍串行争用一个Mutex；每个span保留owned name/path并在finish复制stream/category，TLS parent path逐层clone/format；finish scope/frame各两次锁recorder。Snapshot在该锁下深clone全部ring，随后hotspot/counter聚合又按每个sample clone key并排序全部duration，export同步生成多份JSON/Markdown/Perfetto DTO，若由主线程触发会形成与样本量成正比的停顿。UI counter报告还为每个sample分配scenario lookup String。

Runtime07/Render17应把静态stream/category/name注册为dense ID，按线程写有界chunk/ring并在frame/export边界批量merge；动态name有byte/intern预算，stop以generation封存，snapshot只交换/借用封存buffer，聚合使用borrowed key和selection而非per-sample owned key/full sort。导出移到后台任务并记录snapshot lock hold、dropped samples、export queue age/bytes，禁止在editor frame主线程同步深复制和写盘。

## 验收要求

对1/8/64 threads、scope depth 1/16/256、每帧scope/counter各0/100/10k、capture off/on、ring 1k/100k/1M记录global mutex acquisitions/wait、atomic loads、payload evaluations、String/path alloc bytes、TLS bytes、dropped samples、snapshot/export p50/p95/RSS：capture off recorder lock/静态name alloc/动态payload eval=0；稳定stream lookup key alloc=0；capture on写入成本按thread近线性且frame线程无导出I/O；bounded ring、nested parent/frame association、stop/reset race、Tracy/Chrome/native parity、current-source Cargo及F2/F4产品trace通过前，本目录留在`pending.md`。
