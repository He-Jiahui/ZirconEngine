---
source_report:
  - docs/plans/performance/01/2026-08-23-first-party-editor-plugin-catalog-instance-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# First-party editor plugin catalog schema/instance受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：新增`zircon_plugins/first_party_editor_catalog/src/**` current 3/3文件、222行、6 tests静态覆盖；Cargo/benchmark/F4 startup WPR/allocator/power仍open。本Session不直接编辑受保护ledger。
- Plan02 M1/M5 + PERF-MVP-629 + Plugins01/10 + Editor12：拆分process级immutable `EditorProviderSchemaGeneration`与session级mutable `EditorPluginInstanceGeneration`；schema<=1/provider/process，instance<=1/provider/session。
- Navigation/Neural接线：Navigation `Arc<Mutex<NavigationPieMirror>>`、runtime consumer、extensions和lifecycle不得放进process全局cached report；同session稳定复用，跨session隔离，unload后完整retire。
- Plugins12：generated dense slot替换O(S*P) if分派，保留target/duplicate/order/unknown行为；删除HashSet/if/no-match源码拼写门，以build/alloc/behavior counter替代。
- App entry tests：把21×1,024次普通单测wall-clock循环迁为受管cold schema/session instance/stable lookup benchmark；普通测试只保留确定性行为，不以250ms机器阈值冒充产品startup预算。
- `docs/plans/performance/review.md`：只有current Cargo、schema/instance counter、100 session/project矩阵、F4 startup WPR/allocator/power和unload隔离通过后迁入；该非渲染切片不要求RenderDoc。本轮不迁移、不提交milestone、不发送完成企微。
