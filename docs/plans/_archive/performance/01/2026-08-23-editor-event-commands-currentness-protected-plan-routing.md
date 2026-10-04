---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-event-commands-currentness-revalidation.md
doc_type: protected-plan-routing
status: routing_pending
---

# Editor Event / Commands currentness受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：更新Event 36/36、2,700行、8 tests与current fingerprint；Commands 17/17、4,311行、31 tests与current fingerprint。本Session不直接编辑受保护ledger。
- EditorUI01：realtime input硬切到interaction state，绕开command/audit/replay；move/resize可合并但edge事件必须有序。
- Editor08：落实typed `CommandRoute`和唯一`CompiledCommandGeneration`；direct event registry visits=0，menu只按generation编译一次。
- Editor03：revision只在成功changed commit后推进；replay只接受versioned committed operation，raw/transient/failure/external side effect fail closed。
- Editor02 + Runtime11：audit/plugin派生流使用count+owned-bytes+deadline admission和声明式affinity；禁止私有event线程或无界handoff。
- Editor12：插件command batch锁外build全部indexes、一次generation commit；旧reader在reload/unload前quiesce。
- Save All owner：保留当前未提交实现，但补充明确replay disposition与current-source Rust集成测试，不能依赖`RetainedHost` source分支偶然抑制effect。
- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`：保持Event/Commands结构cutover为MVP P0，并记录本轮没有动态关闭或源代码优化。
- `docs/plans/performance/review.md`：只有managed Cargo、F4、WPR/allocator/RSS/power与必要RenderDoc parity通过后迁入；本轮不迁移、不commit、不发送企微。
