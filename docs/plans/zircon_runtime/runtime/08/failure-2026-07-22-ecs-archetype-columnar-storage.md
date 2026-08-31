---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: ecs-archetype-columnar-storage
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime/src/scene/ecs/storage/component_storage
  - zircon_runtime/src/scene/ecs/query
  - zircon_runtime/src/scene/world/identity.rs
tests:
  - cargo test -p zircon_runtime --lib ecs_storage --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib ecs_query --locked --jobs 1 -- --nocapture --test-threads=1
  - 100k entity archetype/query/despawn counters
---

# Runtime08：ECS archetype columnar storage交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：scene ECS archetype/component/entity/storage 28/28逐Rust文件审查，PERF-MVP-481
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有component storage、archetype、query location和World mutation权威。
- 生命周期键：`ecs-archetype-columnar-storage`

## 失败现象与复现证据

当前Table不是archetype-owned columns：每个ComponentId拥有独立`HashMap<InternalEntity, usize>`与`Vec<TableEntry>`，每值单独`Box<dyn Any>`。ArchetypeIndex另存signature+entity Vec。每次component add/remove为重算signature遍历全部component storages并对entity做contains hash；despawn同样遍历全部storages；query命中archetype后每个entity/component仍HashMap lookup+Any downcast。SparseSet也只是HashMap。

Query 25/25逐文件复核进一步确认：每个`QueryState`复制matched entity ids、sorted entity→cache index、StableEntityLocation、每实体ComponentStorageLocation与offset；一个World-global `query_cache_revision`在任意结构变化后失效所有query state。cache miss重新匹配archetypes、计数、遍历实体、收集component locations并排序entity index，而不是只更新受影响archetype generation/row。

## 最低共享层根因

archetype membership、component rows和query columns没有统一row authority；ArchetypeIndex是二级索引而非实际table owner，ComponentStorage无法按现有signature O(K)移动/删除，也不能给query一次解析的typed column range。

## 架构修复验收

- 每Archetype拥有同row对齐的type-erased contiguous columns、ticks和entity vector；EntityLocation的archetype+row是唯一table定位，swap-remove一次修复所有columns和swapped entity。
- add/remove/bundle先从旧signature增量生成目标signature，按affected columns一次move/commit；禁止扫描所有registered component storages重算membership。
- despawn只遍历entity当前signature的K个columns；lifecycle/removal按stable ComponentId次序从同一signature发布。
- query compile按signature/archetype generation一次解析required/optional component column slots，cache只保留matched archetype plan与局部generation；iteration按table row直接访问，禁止每QueryState复制全量entity/location projection。
- 单entity结构变化只使old/new archetype及引用它们的query plan局部失效；无关archetype/query cache不得因World-global revision重建、重排或重复制locations。
- SparseSet使用dense entities/values+entity-index sparse mapping或等价O(1)结构，并与table query location共用generation invalidation。
- entities/components/archetypes 1/1k/100k、change 0/1/100%记录hash probes、Any downcasts、membership scans、moves、cache misses、bytes与p95：table query连续row、despawn O(K)、stable query hash/downcast=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止在现有per-component HashMap外再加第三套archetype columns而长期双写。
- 禁止只缓存signature Vec却保留query per-entity HashMap+Box downcast。
- 禁止用unsafe pointer cache绕过generation/aliasing证明或继续让固定组件维护平行map真相。

## 修复结果与回传

Open state: `前向修复中`; no pass is claimed.

### 2026-08-10 current-source progress

- 已将 dense row 真值下沉到 `ArchetypeRecord -> ArchetypeTable`：实体向量、连续 erased columns 与 ticks 同 row swap-remove，`EntityLocation { archetype_id, table_row }` 负责唯一定位。
- 结构变化改为显式完整 row delta。目标签名、列集合与新增值具体类型在 source row take 前验证；take 后只执行 source swapped-row 修复、delta 应用、target append 与新 location 发布。
- typed add/remove、bundle、change detection、fixed snapshot、dynamic-scene affected-row transfer、light/post-process extraction 已开始改走 archetype table；bundle 与 dynamic-scene target commit 均按实体聚合为一次 dense row transition。
- `ArchetypeTable` 列目录已改为按 `ComponentId` 排序的连续向量，并提供一次编译的 `column_slot`；`QueryState` 已引入 per-archetype plan/binding 结构作为移除 per-entity cache projection 的前置边界。
- Windows managed compile 已形成 source-bound durable receipt：ticket `fc66d7eb3e7f4a58a160e073ac46e99c`，manifest `a7766283f3c6438307b09f18fb0a644ef6244daa254020fe557ffb4b4dcc17f0`，状态仅为 `queued`，不作为 green evidence。

### Remaining before fixed

- 2026-08-11 current-source reconciliation: `ComponentStorage.table_components` / `TableComponentStorage` / `ArchetypeMove` are already removed. Runtime08 owner-tree and runtime-absorption guards now inventory `ArchetypeTable` under the archetype owner and assert that component storage has no table owner; `ComponentStorage` is sparse-only. This static hard cut is not a managed validation result.
- 2026-08-13 current-source reconciliation: `render.rs` 的 mesh、sprite 与 camera dense extraction 已全部经 `ArchetypeIndex::for_each_table_component` 读取 archetype-owned columns；生产路径不再通过 `ComponentStorage` 的退役 table facade。此前 Runtime09 owner 依赖已由其前向集成解除，不再是本 failure 的剩余源码项。
- 2026-08-11 current-source reconciliation: `QueryState` now retains only `CachedArchetypePlan` bindings with table column slots and local membership generations; it no longer stores `cached_entities`, `cached_locations`, or N*K component-location projections. `cached_name_query_keeps_stable_world_order_across_moves_clone_and_serde` performs a real cross-archetype move whose source table swap-removes another entity, then verifies the same cached order after clone and serde. This is static/behavioral source coverage only, not a Cargo result.
- sparse-only transition、clone/serde final-row 聚合重建和 whole-operation despawn 的源码边界已完成：物理 entity owner 使用 `entity_dense_rows + Vec::swap_remove`，确定性枚举由 `StableQueryOrderIndex` 独立维护；层级与 active-camera 边界从索引取得，不再扫描或移动全量稳定顺序 Vec。`DetachedEntityBatch` 的 1/1k/100k fixture 与 columnar scale counters 已存在。剩余仅为 focused/parity managed validation、真实 counter/p95 终态证据和完成后二次审查；在 terminal receipt 前不得生成 fixed return。

### 2026-08-13 final-row projection rebuild repair

- `EntityRegistry`/stable-order rebuild no longer publishes temporary empty-archetype rows. It resets archetype membership only; `projection_rebuild` owns the subsequent publication of exactly one complete dense/sparse row per entity into its final archetype.
- Added a dedicated prevalidated projection commit path. It validates the final table schema before publication, restores sparse values into sparse storage, then appends the complete dense row once without a source-row take or intermediate membership generation.
- Removed the destructive second registry rebuild from project normalization. The former order cleared archetype-owned component rows before `rebuild_typed_component_presence` could snapshot them after the table hard cut.
- Source regressions now require the direct final-row owner, and clone/serde coverage asserts two entities produce exactly two row appends while persistent names and dynamic presence survive. Exact `rustfmt +1.94.1` and scoped `git diff --check` pass; no Cargo result is claimed in this update.
- `SparseComponentStorage` no longer hashes `InternalEntity` on every access. Its dense entity/value arrays are indexed by a generation-aware sparse slot vector keyed by `InternalEntity::index`; stale generations cannot alias a reused entity slot, and swap-remove repairs exactly one sparse locator. Structural guards reject the retired `HashMap<InternalEntity, usize>` owner.

### 2026-08-28 sparse locator high-water repair

- The generation-aware locator no longer retains a continuous vector through the highest entity
  index. It now uses 256-slot pages, a zero-based packed prefix, and one page-aligned offset window;
  either flat span is promoted only within a 1,024-slots-per-live-location density bound. Further
  disjoint pages use a private identity-hashed directory plus an ordered `BTreeSet` ownership index.
- Empty pages retire immediately. Low-density owners trim edges or rebase before the 1/2,048
  demotion threshold, and an empty locator releases all owners. The packed
  `(generation, dense_row + 1)` slot is 8 B on x86_64 while preserving stale-generation rejection
  and one-locator swap-remove repair.
- The standalone release model records `96,000,024 -> 2,048 B` at entity index 4,000,000 and
  `6,291,456 -> 2,097,152 B` for 262,144 contiguous rows. Three final dense runs range from 10.3439%
  faster to 4.2631% slower; partial P50 regresses at most 28.0677%, high offset mixed/hits improve
  6.4199%-15.0733% / 7.2077%-13.7094%, and dual-span hits regress 4.3081%-16.1994%. Truly disjoint
  overflow remains a memory-first profile boundary.
- Focused source/status contract 3/3 and direct real-owner Rust behavior harness 16/16 pass,
  including cross-representation deletion compaction. Status is
  `runtime_08_60_sparse_component_locator_algorithm_source_passed_diagnostics_cargo_product_profile_deferred`.
  Production locator-byte aggregation, managed Cargo, million-entity counters/RSS, real-scene P95,
  WPR/CPU/power, and wider query/table acceptance remain open, so `RECS-P1-11` is partial.
