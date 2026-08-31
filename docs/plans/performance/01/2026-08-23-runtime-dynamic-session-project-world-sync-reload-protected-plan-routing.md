---
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-project-world-sync-reload-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Runtime dynamic session项目、WorldSync与热重载受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：将`dynamic_api/session/{project,world_sync,scene_asset_reload_diagnostics}.rs`更新为current 3/3文件、1,478行、18 tests；注明diagnostics M0静态完成，project generation、WorldSync规模与reload产品动态验收仍open。本Session不直接编辑受保护ledger。
- `PERF-MVP-638` + Runtime04/11：补充session项目激活在install后仍两次clone完整`ProjectManager`，runtime UI扫描全部UI资产；继续由唯一`RuntimeAssetGenerationStore`、root dependency demand与shared task DAG收敛，不新增并行cache或snapshot owner。
- `PERF-MVP-468` + Runtime10/07 + Editor02：补充watch registration与`pending_dirty`缺少count/bytes/age硬界；page oversize二分会重复build/clone/full JSON encode。先加入encode attempts/visited/clone bytes计数，再硬切canonical generation cursor、bounded dirty/page与锁外单次encode。
- `PERF-MVP-471` + Runtime04/08/11：更新旧根因。current reload已有count/time/bytes预算、per-asset keyed single-flight/supersede和各stage byte caps；任务改为验证/收束ready-to-compiled transaction、cancel/stale/slow-consumer/frame-budget，不重复实现队列。
- Runtime09：runtime UI只从激活root surface dependency closure加载generation handles；不得从project snapshot全registry扫描并同步load全部UI asset。
- Runtime07：采集project snapshot/clone bytes、WorldSync page build/encode attempts、reload queue/job/apply、diagnostic lock以及WPR CPU/I/O/wake/power关联计数。
- `docs/plans/performance/review.md`：只有current Cargo、规模/行为/取消/故障恢复、WPR/allocator/power以及相关F2/F4 RenderDoc对拍通过后迁入；本轮不迁移、不提交milestone、不发送完成企微。
