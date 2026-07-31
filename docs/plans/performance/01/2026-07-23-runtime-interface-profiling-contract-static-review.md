---
related_code:
  - zircon_runtime_interface/src/profiling.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
reference_sources:
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - current-source Windows profiling/diagnostics tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface profiling 合同性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/profiling.rs`当前源 **1/1** 个 Rust 文件、**427** 行已逐文件阅读，并追踪 config到 recorder、dynamic profile control、Editor visible-pane diagnostics与 profile merge。该文件定义 capture预算、wide timeline/counter/UI reports和 dynamic ABI response，因此其容量合同直接影响性能工具的观测者效应和最坏 RSS。

## 性能结论

- `ProfileCaptureConfig::normalized`只把 0替换为默认 `512 frames / 16,384 spans / 4,096 counters`，不限制非零上界；`usize::MAX`等配置可经 profile-control ABI进入 recorder并长期保留任意数量样本。`frame_budget_ms`也只检查 `<= 0`，NaN/无穷可穿透。新增 **PERF-MVP-566**。
- entry count不是内存预算：frame/span/counter拥有多个 `String`，span还有 stream/category/name/path；单条 metadata长度无硬限制。即使 count合理，恶意或动态长 path仍可把 retained bytes与 snapshot clone/JSON输出放大。必须同时限制 entries、owned bytes和单字段长度，并暴露drop/truncation计数。
- `ProfileRecorder::snapshot`把三个 VecDeque全量 `cloned().collect()`，会深复制所有 sample String；visible Performance/Diagnostics pane还会采集约541条series并在 profiling feature下通过 dynamic ABI取得完整 runtime profile后merge。该 observer成本继续归 **PERF-MVP-324/326**：同generation只能封存一次 immutable snapshot，UI按 visible range或增量页消费。
- `ProfileControlResponse`可同时拥有 snapshot、runtime diagnostics、三类hotspot report和files；当前命令通常只填一个分支，但 DTO没有 encoded-byte ceiling。Runtime10必须在 ABI输出前执行总字节预算，不能只依赖 producer的默认 count。
- `RuntimeDiagnosticsSnapshot::series(path)`为线性扫描；少量selected detail可接受，大量pane字段查询应消费PERF-MVP-324的 dense series token/index，不在interface DTO层另建缓存。
- capture inactive快路和active recorder单全局锁继续由 PERF-MVP-326拥有；本轮没有改 recorder或source。

## PERF-MVP-566 设计

1. Runtime07定义不可由外部配置突破的 hard maxima：frames/spans/counters entries、retained owned bytes、单metadata bytes、snapshot/export page bytes；用户值规范化为 effective config或明确拒绝，NaN/无穷失败。
2. recorder以 generation-owned interned metadata+dense sample rows和全局 byte accounting执行eviction；count或bytes任一到限都按确定顺序淘汰/拒绝，并记录 requested/effective、dropped entries/bytes、oldest age。
3. snapshot/export使用 sealed generation + page/cursor或共享 Arc；同generation多 consumer不重复深clone全部 String，Editor只拉可见/选中窗口。capture off仍为零锁/零metadata materialization。

这项预算与 PERF-MVP-324/326共用唯一 recorder/diagnostic generation，不创建“ABI ring”和“in-process ring”两套真相。

## 参考引擎对照

Bevy Diagnostic使用 `VecDeque`与 per-diagnostic `max_history_length`逐条淘汰历史，说明 bounded history应在 producer owner而非 UI补救；但其可配置 reserve不是Zircon外部 ABI的安全上界。Zircon sample含更多 owned String并可跨动态边界，所以还必须增加总 bytes、字段长度和输出页预算。

## 动态验收

1. config 0/default/hard-max/hard-max+1/`usize::MAX`，budget negative/NaN/infinite：effective或error确定，不能分配到外部请求规模；ABI serde与状态诊断通过。
2. threads 1/8/64、events/frame 0/100/10k、metadata 8B/4KiB/1MiB、capture 1/512/10k frames：记录 retained entries/bytes、eviction/drop、lock wait、snapshot clone/JSON bytes、RSS与p95；所有维度受 hard budget。
3. hidden/visible pane 0/30/60/120 Hz与同generation重复请求：diagnostic series builds、full snapshot clone、ABI encode和UI row build在stable generation为0或仅visible page；capture本身不被observer阻塞。
4. export/cursor完整性：分页重组与单次 sealed report内容一致，drop/truncation明确，stop/reset/restart和multi-session不泄漏旧generation。

动态门禁、current-source Cargo和产品 profiling trace未完成，因此该文件继续保留在 `pending.md`，不进入 `review.md`。
