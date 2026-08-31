---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-message-runtime-consumer-currentness-revalidation.md
doc_type: protected-plan-routing
status: routing_pending
---

# Editor Message / Runtime Consumer currentness受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：Message保持35/35、2,935行、10 tests、指纹`f67e...652e`；Runtime Consumer保持8/8、1,526行、2 tests、指纹`1023...c46`。本Session不直接编辑受保护ledger。
- Editor02 + EditorUI08：UI patch retention退出generic bus锁；inbox/UI delta采用count+owned-bytes+deadline page，失败或stale generation至多full rebuild一次且不重放旧patch。
- Runtime10 + Plugins01/11：consumer ABI增加delivery policy、stable key、affinity；同route共享subscription/page/decode，Latest在producer queue替换。
- Editor04 + Editor12：Play begin或capability/route generation变化时才reconcile；stable tick map/set/clone/subscribe为0。
- Runtime11 + AI06 + Navigation05：重decode/map preparation放有界worker ticket，Editor只提交current projection；AI双endpoint和Navigation snapshot不得重复序列化/解码。
- Editor02测试owner：把静态契约路径更新到`bus/backpressure/{behavior,performance}.rs`，不能恢复旧`backpressure.rs`聚合文件。
- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`：保留Message/Runtime Consumer为MVP P0，并记录当前只是指纹复验，没有动态关闭。
- `docs/plans/performance/review.md`：managed Cargo、scale benchmark、F4 WPR/allocator/RSS/power及必要UI parity未通过前不得迁入；本轮不commit、不发送企微。
