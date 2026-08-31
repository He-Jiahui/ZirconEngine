---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-plugin-extension-currentness-revalidation.md
doc_type: protected-plan-routing
status: routing_pending
---

# Editor Plugin / Extension currentness受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：更新Plugin 35/35、6,323行、51 tests、current fingerprint；Extension 30/30、5,267行、38 tests、指纹未变。本Session不直接编辑受保护ledger。
- Editor12：拆分definition/runtime/diagnostic generations；普通lifecycle callback不得重建catalog/extension；project registration与manifest apply合并为一个validated transaction和一次publication。
- Editor06 + Editor02：实现唯一ExtensionReconciler、mounted generation、owner receipt、bounded lifecycle page和锁外callback supervisor；stable frame的extension/Inspector build必须为0。
- Plugins01 + Runtime11：native unload前关闭admission并等待callback/job/snapshot roots归零；callback affinity、deadline、output budget和stale generation rejection由明确owner负责。
- Editor12验证债务：复核并更新9个失真的Python静态源码锚点；不得通过恢复旧owner命名来迁就测试。新增scale counters和真实lifecycle/toggle/reload product tests。
- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`：保持plugin/extension structural cutover为MVP P0，并显式记录当前只有静态currentness、没有动态关闭。
- `docs/plans/performance/review.md`：只有managed Cargo、合同、F0/F4、WPR/allocator/RSS/power与必要RenderDoc parity全部通过后才能迁入；本轮不迁移、不commit、不发送企微。
