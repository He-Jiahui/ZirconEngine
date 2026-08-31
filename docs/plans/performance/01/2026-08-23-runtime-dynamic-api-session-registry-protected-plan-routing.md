---
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-api-session-registry-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Runtime dynamic API session registry受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：更新session registry为9/9文件、1,636行、18 tests；注明producer Vec storage M0已静态实现，global registry短锁与锁外finalize已正确，session action长锁、global allocation单锁和无界teardown wait仍open。本Session不直接编辑受保护ledger。
- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`：更新PERF-MVP-430 current evidence；增加1/8/64 sessions allocation contention和destroy phase/wake callback观测，保留commit/rollback/action lease语义。
- Runtime10/Runtime11/Runtime07：实现短锁admission、bounded session/world/render lanes、generation publish和cancel/drain teardown；allocation table下沉slot或按slot generation分片，不建立第二个无界线程池。验收增加producer capacity与realloc/copy bytes，确认本轮Vec M0动态成立。
- App/Editor gateway计划：消费typed ticket/frame generation并显式选择async或sync等待；普通输入/control不得隐式等待GPU/JSON lane，library unload不得在仍有回调时强退。
- `docs/plans/performance/review.md`：仅在current Cargo、slow-action/多session/大allocation/wake/destroy矩阵、WPR contention、allocator/RSS和F2/F4通过后迁入；本轮不迁移。
