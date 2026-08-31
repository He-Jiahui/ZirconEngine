---
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-api-root-viewport-json-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Runtime dynamic API root viewport/JSON受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：把dynamic API currentness更新为根层9/9文件、2,292行、7 tests；注明完整目录仍在逐子目录复审且current Cargo/F2/F4动态门阻塞。本Session不直接编辑受保护ledger。
- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`：在PERF-MVP-430附近新增或合并“稳定viewport identity + explicit resize”P0，量化create/destroy、surface/pipeline create、history invalidation与GPU wait；不要把它降格为窗口事件debounce。
- Runtime10/Runtime07：冻结manager service激活期生命周期与generation-aware typed snapshot，消除稳定帧/输入事件的registry lookup/downcast/双Arc clone，同时保留stale generation语义。
- Render03/Render17：增加`resize_viewport` owner contract，WGPU原位configure size-dependent资源；只失效尺寸相关history/product，保留handle、surface target、debug subscription和尺寸无关pipeline。
- Runtime10：为bounded inbound/outbound JSON记录passes/bytes/values/alloc/deadline，先用合法与攻击payload证明安全/性能权衡，再决定是否把depth/value budget合入单次typed deserialize。
- `docs/plans/performance/review.md`：仅在current Cargo、resize storm动态counter、WPR/Tracy/allocator、F2/F4产品帧及RenderDoc像素/draw parity通过后迁入；本轮不迁移。
