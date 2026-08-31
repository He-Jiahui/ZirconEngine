---
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-ui-surface-routing-publication-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Runtime dynamic session UI surface路由与publication受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：将`dynamic_api/session/runtime_ui.rs`更新为current 1/1文件、577行、2 tests；注明last-consumer move M0静态完成，render publication、active pointer、startup/accessibility与动态trace仍open。本Session不直接编辑受保护ledger。
- Runtime09 + Render14 + PERF-MVP-433：补充dynamic session绕过`UiSurfaceFrame` Arc generation，每次capture/present深clone全部surface commands并重写node id；目标是唯一generation-owned globalized render artifact/ranges，stable aggregate build/clone=0。
- `PERF-MVP-003/334` + Runtime10/12：记录M0把generic/noncapture owned-event clone `S -> S-1`、single surface `1 -> 0`且capture保持0；pointer capture表无count/bytes/age硬界，终态仍由bounded typed admission、active-pointer contract和UI batch owner收敛。
- `PERF-MVP-425` + Runtime10：保留新增IME drain bridge；每surface Vec到session aggregate、semantic coalescing和非render continuation继续由统一HostIntentOutbox负责，不建UI私有队列。
- `PERF-MVP-638` + Runtime04/09/11：runtime UI startup从全asset registry同步load全部UI kinds改为root dependency closure、shared asset generation和异步candidate/短commit。
- `PERF-MVP-597` + Runtime10：accessibility rebuild/globalize/snapshot与JSON不得持session lifecycle mutex；按显式query ticket、generation和预算锁外物化。
- `docs/plans/performance/review.md`：只有M0静态/行为、current Cargo、UI/input/active-pointer/IME规模、WPR/allocator/power及F4 RenderDoc通过后迁入；本轮不迁移、不提交milestone、不发送完成企微。
