---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-jobs-settings-recovery-currentness-revalidation.md
doc_type: protected-plan-routing
status: routing_pending
---

# Editor Jobs / Settings / Recovery currentness受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：更新Jobs 47/47、9,083行、108 tests；Settings 16/16、3,937行、34 tests；Recovery 20/20、4,890行、54 tests及三个current fingerprint。本Session不直接编辑受保护ledger。
- Editor14 + Runtime11：Jobs既有single TaskGraph hard cut、lifecycle bytes admission、named-affinity completion与stable generation UI消费全部保持open；新增全局bare-thread inventory drift由Runtime11 owner处理，不在Jobs切片错误修补Graphics/mesh代码。
- Editor17 + Runtime11：Settings继续按physical file generation合并；Recovery继续dirty delta demand、honest payload budget、O(1) manifest slot与project generation fence。禁止直接把同步autosave/heartbeat I/O接进retained tick。
- Editor17 test ownership：把`core/recovery/tests.rs`和`tests/autosave_adapter.rs`按session guard、catalog、store、admission、completion feature拆成folder-backed owner，使每个文件<=800行；Rust fixture root必须注入D/E/F，不能依赖Windows C盘temp。
- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`：保留Jobs scheduler/retention、Settings file generation、Recovery reachability/dirty/payload/storage/lifecycle优先项；记录本轮只完成currentness、没有动态关闭。
- `docs/plans/performance/review.md`：仅在current Cargo、源码合同、F0/F4、WPR/file-I/O/allocator/RSS/power矩阵通过后迁入；本轮不迁移、不commit、不发送企微。
