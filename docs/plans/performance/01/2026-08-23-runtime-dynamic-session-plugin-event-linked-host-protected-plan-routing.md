---
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-plugin-event-linked-host-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Runtime dynamic session插件事件、linked plan与host request受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：将`dynamic_api/session/{event_mirror,host_requests,linked_plugins,linked_session}.rs`更新为current 4/4文件、1,076行、11 tests；注明descriptor JSON M0静态完成，shared broadcast、compiled plugin plan、host outbox continuation和动态trace仍open。本Session不直接编辑受保护ledger。
- `PERF-MVP-615/432` + Runtime10/11 + Plugins01：保留current单subscription页/队列硬界和raw JSON改善；补充subscription数量/aggregate bytes无界、producer E*S同步serialize/mutex/private retention、session锁内最多约8次prefix encode。目标仍是type-level一次编码的shared segment + cursor/lag，不新增plugin私有pool。
- `PERF-MVP-432`：记录M0把descriptor转义从每attempt `2*N`降为page热路0、subscription固定2，并新增`plugin_event.page_encode_attempt`；不得据此关闭payload copy、JSON ABI或session-lock任务。
- `PERF-MVP-538/630` + Runtime06 + Plugins01：补充linked session从registration/project manifest重新物化module report、临时catalog、extension report和package ids；并入唯一`CompiledProjectPluginPlan` generation，不为linked session新建cache。
- `PERF-MVP-425` + Runtime10/12 + App：保留current empty/IME bound/borrowed page/counter改善；producer Vec aggregate、single typed drain、semantic coalescing与257+ row非render continuation继续由host-intent既有任务认领。
- Runtime07/Render17：采集event serialize/lock/owners/lag/page attempts、plugin generation builds/clone bytes、host continuation/tick/present和WPR/allocator/power；RenderDoc仅做F2/F4像素/draw/present回归。
- `docs/plans/performance/review.md`：只有current Cargo、event/plugin/host规模行为、overflow/cancel/reload/shutdown、WPR/allocator/power及相关F2/F4回归全部通过后迁入；本轮不迁移、不提交milestone、不发送完成企微。
