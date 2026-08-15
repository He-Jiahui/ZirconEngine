---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dynamic-scene-session-indexed-transaction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session/slot_store
  - zircon_runtime/src/scene/dynamic_scene/session/query
  - zircon_runtime/src/scene/dynamic_scene/session/merge
  - zircon_runtime/src/scene/dynamic_scene/session/retention
  - zircon_runtime/src/scene/dynamic_scene/session/selected_capture
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation
tests:
  - cargo test -p zircon_runtime --lib dynamic_scene_session --locked --jobs 1 -- --nocapture --test-threads=1
  - 100k slot merge, selection, preview and prune counters
---

# Runtime08：dynamic scene session索引化事务交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：dynamic scene session核心195/563逐Rust文件审查，PERF-MVP-476
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有scene/world identity、dense storage和原子mutation边界；Runtime04提供generation artifact。
- 生命周期键：`dynamic-scene-session-indexed-transaction`

## 失败现象与复现证据

slot/manifest查找为线性扫描，每次push/upsert/rename全量sort；selection先clone/normalize/sort完整manifest，再按id二次查找。preview重复验证所有embedded scenes并clone报告payload，commit再次查找/验证。merge逐incoming重复contains并逐项push/sort；capture+retention preview深clone整个archive，tag protection还反复构造slot-id集合并线性membership。

## 最低共享层根因

archive只有公开Vec排序约定，没有canonical slot-id index、generation validation ticket、borrowed selection handle或batch mutation plan；preview/commit只能各自重演算法。

## 架构修复验收

- 单一canonical slot storage同时提供slot-id index、stable order和updated/tag secondary index；mutation增量维护，禁止平行真相漂移。
- selection返回generation-bound borrowed handle/index及borrowed summary；不构造完整owned manifest，commit验证generation后直接命中。
- merge/import/prune/capture-retention先生成轻量mutation plan，批量查重/验证一次、sort/publish一次；失败丢弃plan，authority零变化。
- preview report只复制对外必要小字段，不clone scene/archive payload；`ensure_supported`结果以generation ticket复用。
- slots/incoming/tags 1/1k/100k记录lookup probes、validation passes、sort comparisons、payload/metadata clone bytes：lookup O(1)或O(logN)、batch sort≤1、preview payload clone=0、failure partial writes=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅把某个linear find替换为binary search，却允许公开未排序Vec绕过canonical invariant。
- 禁止为preview深clonearchive来换取rollback，复用PERF-MVP-467 affected-row transaction原则。
- 禁止每项mutation调用全量sort/validate或维护未绑定generation的旁路index。

## 修复结果与回传

Open state: `实现收敛中`; no Cargo pass is claimed.

2026-08-10 current-source progress:

- `RuntimeSessionArchivePayload` 已成为 slot-id、更新时间与 tag secondary index 的单一 owner；单 slot push/upsert/rename/metadata/touch/remove 增量维护这些索引，merge/prune 通过一次 batch publication 更新稳定 slot 顺序。
- selection 已提供借用 archive row 的 generation/revision-bound handle；merge 与 prune 使用 generation/revision-bound plan，commit 前拒绝 stale target，preview 仅复制报告所需的小字段。
- Runtime08 本轮进一步硬切 `RuntimeSessionArchive: DerefMut`。Archive payload 的可变访问现在只经私有 `payload_mut` 推进 revision 并执行 copy-on-write，crate 内调用方不能再直接取得 `slots: Vec<_>` 的可变入口绕过 canonical indexes；source guard 同步禁止该旁路。
- 已执行精确路径 `rustfmt +1.94.1 --check`、`git diff --check` 与仓内 slot mutation source scan，结果通过；本记录仍保持 open，直到 `dynamic_scene_session` 受管行为测试和 1/1k/100k lookup/validation/sort/payload-clone 指标取得终态证据。

### 2026-08-11：dense slot relocation 未关闭

当前 `RuntimeSessionArchivePayload` 的 key lookup 已由 `slot_indices` 接管，但 `insert_slot` 仍调用 `Vec::insert`，`remove_slot` 仍调用 `Vec::remove`，两者随后以 `shift_slot_indices` 遍历 primary index、updated index 与所有 tag index 修正每个受影响 slot offset。它避免了每次完整 sort，却仍对一次 slot mutation 执行 `O(N + tag-membership)` relocation，不能满足 1k/100k affected-row 成本目标，也不能把既有“incremental index”描述为尺度闭包。

下一原子 hard cut 必须把 dense payload slot storage 与 canonical slot-id iteration 分离：slot id index 直接定位 dense row，remove 只 repair swapped row；updated/tag secondary order 使用 stable keys 而非 mutable vector offsets；wire/artifact/manifest/query consumers 在同一提交中通过 canonical slot-id view 序列化和枚举。不得在 archive owner 旁再保留一个可变 sorted `Vec` 或在 preview/commit 各建 cache。`artifact.rs`、`manifest/*`、`query/*`、merge plan 与 capture/retention consumers 需要在取得同一 source scope 后一并迁移。本 handoff 继续为 `open`，没有 Cargo 或 1/1k/100k managed metrics acceptance。

### 2026-08-12：dense row hard cut implementation pending managed evidence

- `RuntimeSessionArchivePayload` now keeps slot rows in append-only dense storage for insert/upsert and uses `swap_remove` for removal. Only the swapped row's primary, updated-at, and tag secondary entries are repaired; the old `Vec::insert`/`Vec::remove` relocation and `shift_slot_indices` sweep are absent.
- Canonical ID order comes solely from the `BTreeMap<String, usize>` primary index. The public slot view, JSON serialization, artifact manifest, merge planning, capture-retention preview, prune planning, and validation consumers enumerate that view rather than relying on physical row order. Updated-at and tag ordering use `(updated_at, slot_id)` stable keys.
- Archive input validation deliberately iterates dense rows, not the canonical index, so duplicate incoming slot IDs cannot be hidden by index replacement before validation rejects them. The public mutable-Vec and physical-sort entry points were removed rather than retained as no-op compatibility APIs.
- Local evidence so far is source-guard coverage for swap removal/canonical views plus `rustfmt` and `git diff --check`. No managed Cargo test, 1/1k/100k metric run, fixed handoff, or acceptance is claimed; this failure remains `open` until those terminal receipts exist.
- Post-implementation second review found no P0/P1. Its two P2 coverage gaps were added as source tests: swap-removal now proves canonical JSON serialization and reload order, and an externally mutated archive rejects a prepared stale prune plan without deleting either row. The reviewer then rechecked those exact tests with P0/P1/P2 all zero. This is review evidence only, not managed test acceptance.
- Coordinator evidence: historical source snapshots `1643` and `1644` were accepted before the two post-review regression additions, so neither may be used as current validation evidence. The managed CPU reservation request for `cargo +1.94.1 test -p zircon_runtime --lib dynamic_scene_session --locked --jobs 1 -- --nocapture --test-threads=1`, and the later current-source snapshot request, were both rejected before admission with `database is locked`; neither created a reservation, job, run, or Cargo process. This is coordination infrastructure evidence, not a test result or code RED, and must not be converted into a fixed/accepted claim.
