---
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-construction-diagnostics-ffi-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Runtime dynamic session构建、诊断与FFI调度边界受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：将`dynamic_api/session/{construction,diagnostics,error,ffi}.rs`更新为current 4/4文件、1,598行、5 tests；注明linked module clone M0静态完成，startup/action/diagnostics/output动态trace仍open。本Session不直接编辑受保护ledger。
- `PERF-MVP-629/638` + Plan02 M1/M4 + Runtime04/06/11：补充session create同步重建并激活plugin/project/module/asset/script/UI全管线；目标是一份immutable candidate generation、显式startup phases、有界shared task DAG和短activation commit，不建construction私有pool/cache。
- `PERF-MVP-597` + Runtime10/11 + Editor01/02/04：补充session mutex覆盖tick/capture/present、World projection、diagnostics和JSON encode；建立每session唯一ordered bounded ticket lane，decode/encode移出lifecycle锁，World/render以affinity保持单owner，完成按generation提交。
- `PERF-MVP-324/418` + Runtime07 + Render17：补充显式diagnostics query仍在session锁内取得owned stats、写约541 series、clone history/DTO并编码JSON；收敛为domain generation、summary/detail、if-newer与锁外projection，不建立dynamic-session stats cache。
- `PERF-MVP-574` + Runtime10 + App：冻结output-on-error与teardown exactly-once合同；用RAII output guard、detach+retry终态或count/bytes/age有硬界的quarantine消除错误路径泄漏风险。
- `construction.rs` M0：记录linked module descriptor深clone从`2P -> P`、中间Vec分配`1 -> 0`；不得据此关闭compiled plugin generation、startup I/O或main-thread任务。
- Runtime07/Render17：采集startup phase、generation build/clone、session action queue/lock/affinity、diagnostics series/history/JSON、output free/teardown及WPR/allocator/power；RenderDoc仅做F2/F4 capture/present像素与GPU事件回归。
- `docs/plans/performance/review.md`：只有current Cargo、规模/行为/取消/失败恢复、WPR/allocator/power及相关F2/F4 RenderDoc对拍通过后迁入；本轮不迁移、不提交milestone、不发送完成企微。
