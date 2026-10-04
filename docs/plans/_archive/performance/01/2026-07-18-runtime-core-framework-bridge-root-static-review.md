---
related_code:
  - zircon_runtime/src/core/framework/bridge
  - zircon_runtime/src/core/framework/platform
  - zircon_runtime/src/core/framework/asset.rs
  - zircon_runtime/src/core/framework/channel.rs
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/framework/foundation
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/ui.rs
  - zircon_runtime/src/core/framework/ui
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - bridge four of four Rust files reviewed
  - platform three of three Rust files reviewed
  - seven framework root contract Rust files reviewed
  - current-source Cargo, bridge counters and plugin product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core framework bridge/root逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读framework `bridge` 4/4、`platform` 3/3，以及asset/channel/events/foundation/root/UI合同7/7。Platform/UI/root为module identity与结构导出；asset/foundation/events是显式manager DTO/trait。`recv_latest`会无预算drain channel，但当前生产检索无调用，暂不冒充产品瓶颈。`ConfigManager::contains_key`默认通过owned JSON get判断存在会深clone值，但当前生产无调用，后续与PERF-MVP-318的typed/generation config API一并硬切。

## PERF-MVP-330：debug bridge每调用共享原子计数

每个FrozenBridge entry持`BridgeDiagnostics`；debug_assertions下每次enabled/not-enabled bridge invocation都执行Relaxed `AtomicU64::fetch_add`。生产调用覆盖weak typed bridge、native host adapter与script bridge module。Release构建将其编译为空，但基本编辑器和插件开发主要使用debug构建，多worker/脚本对同一physics/render/asset接口高频调用时会争用同一cache line并污染被测路径。

Runtime06/07应让bridge diagnostics有Disabled/Sampled/Sharded模式：默认editor idle与普通debug产品路径不做per-call共享RMW，诊断页或capture显式开启；worker-local/sharded counters在snapshot时聚合，not-enabled边沿可保留精确事件而enabled常态采样。interface status/generation读取与调用本身保持frozen slot O(1)，不得为计数重新做string resolve或全表锁。

## 验收要求

对interfaces 1/100/10k、threads 1/8/64、calls 1/100/1M、diagnostics off/on/sampled记录slot resolves/status loads、atomic RMW/cache misses、snapshot merge、throughput/p95与editor frame：off hot-path RMW=0，on摊销有界且snapshot精确度声明，not-enabled/absent边沿不丢；native/script/typed bridge generation/status/error parity、Cargo/F0/F2/F4 plugin trace通过前，本批留在`pending.md`。
