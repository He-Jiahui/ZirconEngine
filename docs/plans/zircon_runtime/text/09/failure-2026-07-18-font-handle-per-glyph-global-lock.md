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
  - zircon_runtime/src/text/font/handle_registry/resolver_snapshot.rs
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

### 2026-09-08 Registry Regression Import Repair

Fixing Session `failure-roll-01a07160-text09` retains this open lifecycle.
Managed graphics library job `3db36b30be88485bb06b0a5ff2c6d3e4` reported 90
compiler errors and zero tests, including the missing
`shared_font_database_generation` import in the registry poison-recovery test.
The test file was HEAD-identical at
`77ac5a9f391c087233a2743bcdafe67f44ec19a82bc4181ba6a980290eb5ceb4`
before coordinator transfer `2728c3df655e4717b48eac5e30e24cec`.

Source `3058`, request `dc9dffe834e44b4994caf5253436982e`, changes only
`zircon_runtime/src/text/font/handle_registry/tests.rs`, hash
`2754a70af87f2b59f1b860a271b3f0372ca0b26dc2b9b14138385b71c92eea2d`.
It imports the existing shared generation function without changing the poison
test or production locking/snapshot behavior. Formatting and whitespace checks
pass. Actual registry tests, the 1/100/10k glyph by 1/2/16-thread measurement
matrix, independent review and formal closeout remain required.

Managed job `94560adfdb1a45daa7e2d5785ae6677c` subsequently checked input
`runtime-graphics-text-test-support-3067-20260908`, digest
`c93b37d1c23413b5f1ff16f4705abfbf56ec2511dfb45f89a1b1c85500a839da`.
The registry diagnostic disappeared; the overall library still has 67 errors
and zero executed tests. All input hashes were verified after the command.
Do not interpret the removed compiler diagnostic as a passed poison regression.

The existing task `优化协调器验证效率` reviewed source `3058` with Critical 0,
Important 0, Moderate 0; report
`.codex/tmp/text-framework-3080-review-20260908-result.txt` verifies snapshot,
ObjectStore, current source and attribution at both review boundaries.
The later Text-only job `1f242fb1789e44ce87d7a5cacc569d5f`, input digest
`4e5f800cd19e5748464ac759b4e1efbfae073ecf1c05d419327a6d6739cbe9e2`,
contains the same registry source and reports three library errors, zero
tests, none in this file. The scale/thread and poison-recovery gates remain
pending; the review result does not close this lifecycle.

### 2026-09-19 successor static source-contract receipt

Fixing Session `failure-roll-01a084c8-text09-font-handle-r1` sealed ticket
`229daa1711374de2b076330bca39979c`, with coordinator copy job
`2cbd1fe6f43c446d8b0ed0e79d998a2a` and run
`229daa1711374de2b076330bca39979c` exiting 0:
`TEXT09_FONT_HANDLE_BATCH_SOURCE_CONTRACT_PASS` (`CHECKED_PATHS=5`). The
snapshot covered the failure record plus the registry, service, layout-session,
and SDF consumer owners. Its pre-receipt source manifest was
`24bd682e3ff8006c41adf7941990b5a30df2699d9d2b4a0e6d5fc97da1eebd34`.

This is static source evidence only. The declared 1/100/10k glyph by 1/2/16
thread matrix, managed current-source Cargo/product gates, external
`E:/Git/zr_vm` admission, independent Critical/Important/Moderate zero-finding
review, canonical `failure return`, and closeout remain pending. The appended
receipt changes the document hash after the ticket snapshot; any dynamic
successor must reseal the current bytes.

### 2026-09-20 independent current-source review r1

Reviewer session: `review-text09-font-handle-r1`, child of
`failure-roll-01a084c8-text09-font-handle-r1`. The review covered the four
manifest production owners at the current baseline; no source was edited or
re-owned.

Result: **Critical=0 / Important=0 / Moderate=0**.

- `register_font_handle_batch` deduplicates `(face, instance)` pairs before the
  single registry mutex acquisition, publishes an immutable snapshot only when
  generation or contents change, and records batch/unique/rejected/lock timing
  metrics. Registration rejects collection mismatch and preserves generation
  ownership instead of truncating backend identities.
- `resolve_font_handle_batch` normalizes stale or mixed-generation handles,
  acquires one immutable resolver snapshot for the complete batch, checks
  collection/generation before projection, and remaps results to original glyph
  order while counting rejected pairs. The snapshot resolver retains the exact
  in-flight generation for artifact consumers.
- `TextLayoutSession` keeps the canonical `Arc<ShapedGlyphRun>` and retries or
  defers when the font generation changes; it does not round-trip through a
  neutral framework DTO. SDF face/instance preparation reuses the same batch
  resolver and clears generation-derived caches before accepting a new era.
- The current-source contract marker and scoped `git diff --check` passed.
  Local rustfmt reports existing import-order differences in the current
  snapshot; no formatter-only rewrite was made during this review.

Current reviewed hashes:

```text
zircon_runtime/src/text/font/handle_registry.rs
  0655c13f583068541ddc882d1e54dd4fc31475fbe70b6557c554147114ff2ea4
zircon_runtime/src/text/service.rs
  86bc34822bc6132c4eabfea13c8f31ffdb39322b92045d55980670ae0eb2d011
zircon_runtime/src/text/layout_session.rs
  89890abf4bab12f773611e000241e3a6e9d5652d1988910d684c77ce46c3a636
zircon_runtime/src/text/sdf/font_bake.rs
  b67c9b57a868be6bba6c61b727e4727acdb5be1b33b6aa2245845e4f13636199
```

Managed current-source Cargo, the required 1/100/10k glyph and 1/2/16-thread
matrix, product evidence, canonical `fixed-*` return, and closeout remain
pending. The static marker and this review do not substitute for the declared
performance gates.

### 2026-09-25 successor current-source reconciliation r2

Fixing Session `failure-roll-01a084c8-text09-font-handle-r2` first sealed
coordinator snapshot `3818`, then corrected adjacent dirty-path provenance in
snapshot `3819`. Snapshot `3819` is the authoritative current manifest and
contains the failure record, the four owned production paths, and the adjacent
foreign resolver snapshot. The exact owned hashes are:

```text
zircon_runtime/src/text/font/handle_registry.rs
  0655c13f583068541ddc882d1e54dd4fc31475fbe70b6557c554147114ff2ea4
zircon_runtime/src/text/service.rs
  86bc34822bc6132c4eabfea13c8f31ffdb39322b92045d55980670ae0eb2d011
zircon_runtime/src/text/layout_session.rs
  89890abf4bab12f773611e000241e3a6e9d5652d1988910d684c77ce46c3a636
zircon_runtime/src/text/sdf/font_bake.rs
  b67c9b57a868be6bba6c61b727e4727acdb5be1b33b6aa2245845e4f13636199
```

The current-source probe completed successfully:
`TEXT09_FONT_HANDLE_BATCH_CURRENT_SOURCE_PASS 5 paths`. It checked the batch
register/resolve entry points, the canonical `TextLayoutSession` ownership,
the SDF collection resolver, immutable `Arc` snapshots, and generation-aware
projection. The separate
`zircon_runtime/src/text/font/handle_registry/tests.rs` remains dirty foreign
ownership from archived Session `failure-roll-01a07160-text09`; it was not
claimed, edited, or included in this snapshot.

The scoped `git diff --check` completed without errors. Rustfmt was rerun for
all four owned production files and remains non-passing only because of the
pre-existing import-order differences in the current checkout; no unrelated
formatting rewrite was made. This is source reconciliation evidence, not
managed validation. Cargo, the 1/100/10k glyph by 1/2/16-thread matrix,
external `E:/Git/zr_vm` admission, product-scale evidence, canonical
`failure return`, and closeout remain pending.

### 2026-09-25 resolver-snapshot provenance correction

The independent review found one additional dirty production module adjacent to
the four owned paths:
`zircon_runtime/src/text/font/handle_registry/resolver_snapshot.rs`, current
SHA256
`bfae5936df3f495043b064518cfc11e8ea1fdc13ecf1b7f421109f2a9b00c5ba`.
Coordinator read-only lease and attribution queries returned no owner for this
path. Its working-tree diff is limited to import ordering; it was not claimed,
edited, or attributed to this Session. It is recorded here as foreign/unowned
workspace provenance and is included in the corrected source manifest below.
The separate dirty `handle_registry/tests.rs` remains archived-owner foreign
provenance as described above.

### 2026-09-25 independent provenance-corrected review r2

Reviewer Session `review-text09-font-handle-r2` rechecked coordinator snapshot
`3819` without editing or re-owning any path. All four owned production hashes
match the current bytes; the adjacent
`handle_registry/resolver_snapshot.rs` hash
`bfae5936df3f495043b064518cfc11e8ea1fdc13ecf1b7f421109f2a9b00c5ba` is present
in the snapshot and explicitly remains foreign/unowned import-only drift. The
archived-owner `handle_registry/tests.rs` remains excluded. The five-path
current-source marker and provenance now agree.

Independent result: **Critical=0 / Important=0 / Moderate=0**. This is a
static source/provenance review only; no Cargo or scale benchmark was run.
