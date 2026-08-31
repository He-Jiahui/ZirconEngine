---
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-input-admission-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Runtime dynamic session输入准入受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：将`dynamic_api/session/{events,input_events}.rs`更新为current 2/2文件、1,074行、4 tests；注明空UI M0静态完成，但session/event串行、late coalescing、per-event service/state locks和dynamic trace仍open。本Session不直接编辑受保护ledger。
- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`：PERF-MVP-003补充“现有frame buffer coalescing发生在UI/manager/World工作之后”，PERF-MVP-334补充per-event manager resolve/state lock；不得把M0空UI guard记成结构任务完成。
- Runtime12：认领typed input admission、barrier-aware latest/accumulated分类、bounded entries/bytes/age、batch manager submit、active-session manager generation和125/500/1,000/10,000 Hz矩阵。
- Runtime07：认领session action锁范围、frame drain event/time budget、camera/World delta批提交与WPR lock/wake/CPU/energy归因。
- Runtime09 + EditorUI01：认领runtime UI batch consume、surface visit/event clone/layout counters、capture/focus/geometry barrier和F4产品行为对拍。
- Platform/host任务：输入producer只做typed bounded admission；若启用独立input thread，必须采用单consumer ownership和可审计shutdown，不得并发drain同一MPSC。
- `docs/plans/performance/review.md`：只有current Cargo、完整edge/order/focus/capture/IME行为、队列边界、WPR/allocator/F2/F4产品trace通过后迁入；本轮不迁移。
