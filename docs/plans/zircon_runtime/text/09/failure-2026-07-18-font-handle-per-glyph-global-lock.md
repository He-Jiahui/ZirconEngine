---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: font-handle-per-glyph-global-lock
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/font/handle_registry.rs
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
---

# Font handle每字形全局锁

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/font`当前源32/32 Rust文件及service/layout/SDF调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md`
- 联动责任：Text01提供font generation与稳定face/instance identity；PERF-MVP-232负责删除不必要的DTO往返。
- 交接原因：handle projection的并发、缓存与批处理预算属于Text09，font reload identity属于Text01。

## 失败现象与复现证据

PERF-MVP-246：原主路径shape投影对face/instance分别注册，layout反投影再分别解析，最多4次同一全局Mutex/glyph。本轮已用paired API把service/layout各自降为一次锁，并加roundtrip测试；rustfmt与diff检查通过。结构性问题仍是稳定文本每glyph每阶段至少一次全局锁，SDF miss还有分离解析。

## 最低共享层根因

neutral glyph DTO只保存32-bit slot+generation，而64-bit face/instance identity存在进程级可变Vec/HashMap中。投影没有按shape/run收集unique identity，也没有generation-owned immutable lookup snapshot，因此每个glyph重复进入写锁式registry API。

## 架构修复验收

- 每个shape result或连续run先收集unique `(face, instance)`，一次批量投影handle；global registry lock/acquire按unique identity或batch计，不按glyph计。
- 稳定generation读路径使用immutable snapshot、read-mostly slab或等价无全局写锁结构；注册新identity与generation切换保留慢路。
- layout、SDF、advance report等消费者复用同一projection artifact，不各自重复resolve。
- 1/100/10k glyph、1/2/16 threads记录register/resolve calls、global lock acquire/wait/hold、unique identities、alloc与p50/p95；同face run锁次数O(1)。
- generation reload、stale/mixed handle、face-instance mismatch、poison recovery和ABI serialization回归通过；current-source Cargo通过。

## 禁止临时方案

- 不得只把Mutex换成RwLock而仍每glyph获取锁并宣称完成。
- 不得把64-bit backend identity截断进32-bit handle或绕过generation校验。
- 不得建立无界thread-local handle map而没有generation失效和memory budget。

## 修复结果与回传

2026-08-01 implementation state: `open / resolving_failure / non_validation_implementation_complete / managed_validation_pending`.

- `register_font_handle_batch(...)` and `resolve_font_handle_batch(...)` normalize a glyph stream into unique `(face, instance)` pairs. Registration publishes only on a changed snapshot; resolution takes one generation-checked immutable snapshot and remaps the results to the original glyph order.
- The resolver records batch count, unique-pair count, rejected stale pairs, and snapshot acquire/wait/hold time. The canonical internal layout session keeps `ShapedGlyphRun` directly and does not project font handles into the framework DTO only to resolve them again.
- The production guards cover one snapshot acquisition for a repeated batch, unique-pair accounting, an unchanged second registration batch without a republish, and the internal-session no-framework-roundtrip invariant.
- This records source implementation and deterministic regression coverage only. Managed current-source Cargo plus the 1/100/10k glyph and 1/2/16-thread evidence matrix remain coordinator-owned; keep the handoff open until their receipt is recorded.

2026-08-10 forward-review state:
`open / resolving_failure / implementation_complete / second_review_complete / managed_validation_pending`.

- Glyph-artifact projection borrows the canonical `Arc<ShapedGlyphRun>` directly, constructs the complete visual line before the final font-generation check, and registers all face/instance pairs in one batch. The caller rechecks the current generation immediately before installing the rebuilt line.
- Batch resolution atomically rejects stale generation and mixed face/instance generations. Test-only registration and neutral-projection diagnostics are thread-local, so the one-batch/no-neutral regression is isolated from parallel shaping tests without adding production TLS work.
- The old-placeholder pixel regression and its private generator now live with bake/cache-generation tests; the parent SDF bake test owner is 753 lines and the child is 612 lines, keeping both below the project structure threshold without changing production behavior.
- Independent read-only second review followed the canonical run, artifact projection, batch registry snapshot, generation retry, and SDF consumers and found no P0/P1/P2. Managed Cargo, the required scale/thread matrix, and product-frame evidence remain pending, so this handoff is not marked fixed.

2026-08-13 Runtime bootstrap follow-up:
`shared.rs` now registers the complete checked-in `default.font.toml` manifest under a permanent
Runtime owner before retained headless measurement can run. Its private retained-fallback family is
registered in both the logical matcher and glyphon's backend database, so CPU measure/paint and
native shaping select the same packaged face rather than divergent host fallback faces. The GPU UI
owner attaches to the already registered face-0/face-1 source keys without another TTC registration;
removing that owner retains the bootstrap faces. Focused source regressions cover logical matching,
glyphon query/shaping, final-owner alias cleanup, and the two-face attach/remove lifecycle. This is
implemented and second-source-reviewed evidence only; the handoff remains open pending its declared
managed validation matrix.
