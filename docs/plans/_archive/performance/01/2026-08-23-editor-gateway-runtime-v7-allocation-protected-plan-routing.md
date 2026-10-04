---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-gateway-runtime-v7-allocation-currentness-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Editor Gateway / Runtime V7 allocation受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：Gateway更新为21/21文件、2,934行、11 tests；V7变更9文件已current复审。记录Vec storage M0已静态实现，但Cargo、并发锁、F4/WPR/RSS/power仍open。本Session不直接编辑受保护ledger。
- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`：在`PERF-MVP-597`保留UI同步foreign/decode P0；补充V7 allocation的1/8/64 sessions、1/2/8/32 threads、capacity和outstanding矩阵。
- Runtime10：把allocation/census下沉`SessionSlot`或按slot generation稳定分片，保留opaque ABI、exactly-once、foreign-session拒绝和outstanding unload barrier；不要以裸pointer跨Rust DLL释放。
- Runtime11：提供单session ordered lane和generation completion，使tick/event/world/profile/operation的foreign+decode不在retained UI frame执行；allocation owner随ticket直到release完成。
- Editor01/Editor04：只消费current generation completion；stop/replacement保留旧runtime lease直到所有output释放，不在UI线程同步等待未知provider或release wall。
- Tooling/validation：managed Windows Cargo恢复后在D/E/F target运行Rust gates；WPR/ETW测lock/wake/CPU，allocator测realloc/copy/RSS，RenderDoc仅测current-source viewport present/readback/GPU。
- `docs/plans/performance/review.md`：仅在上述动态门和同机产品矩阵通过后迁入；本轮不迁移、不commit、不发送企微。
