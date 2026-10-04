---
handoff_kind: failure
status: open
created_at: 2026-07-22
updated_at: 2026-09-27
summary_slug: ecs-archetype-columnar-storage
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/tests/ecs_component_storage_structure.rs
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime/src/scene/ecs/storage/component_storage
  - zircon_runtime/src/scene/ecs/query
  - zircon_runtime/src/scene/world/identity.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib --locked -- scene::ecs::storage::component_storage::sparse::tests --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked -- ecs_component_storage_structure --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked -- ecs_identity_storage --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked -- ecs_query --nocapture --test-threads=1
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
- Earlier focused source/status contract 3/3 and direct real-owner Rust behavior harness 16/16
  passed, including cross-representation deletion compaction. Locator entry/page/modeled-byte
  snapshots now aggregate through the shared `ComponentStorage` owner; the current focused source
  contract is 5/5; its two-owner Rust regression includes a 32 KiB modeled structural bound and
  awaits managed Cargo. Status is
  `runtime_08_60_sparse_component_locator_source_complete_cargo_product_profile_pending`.
  Managed Cargo, million-entity counters/RSS, real-scene P95, WPR/CPU/power, and wider
  query/table acceptance remain open, so the managed Runtime08 qualification is not closed.

## 2026-09-27 sparse locator 测试 owner 修复

- 沿用 Runtime08 稳定 primary `failure-roll-01a0df1a-runtime08-message-lifecycle-r1`，base `bc02eefafead65dbf5050482110e8175250a5e77`，epoch `628`；新增领取仅为结构测试及本 failure，保留原 messages 范围、snapshot4599/4600 和未提交验证规格的归属。生产 diagnostics 等活动增量未接管。
- 根因：`sparse_component_storage_keeps_dense_rows_and_a_single_entity_index` 仍要求父 `sparse.rs` 有连续 `sparse_rows: Vec<Option<SparseRowLocation>>` 与独立 `generation: u32` 字段；这两条件在 HEAD 与现行源码均为 false。`dense_row: usize` 因方法参数存在而为 true，不能证明 row 存储合同。本次是精确静态复现，尚无 Cargo RED。
- 改为核对唯一 `SparseRowLocator` owner 和现行 child 路径、opaque location 的 generation/dense-row accessor、lookup 与 removal 的 stale-generation 检查，以及被交换实体的 locator 修复。保留 dense table 唯一 owner 和 column-slot 两条结构测试；总数保持 3。未把旧连续高水位 Vec 重新放回生产代码，未修改布局、算法或公共接口。
- 原始命令保留：`cargo test -p zircon_runtime --lib ecs_storage --locked --jobs 1 -- --nocapture --test-threads=1`；`cargo test -p zircon_runtime --lib ecs_query --locked --jobs 1 -- --nocapture --test-threads=1`。当前 Rust 源码无 `ecs_storage` 模块/目标名称匹配，文头已纠正为真实 owner 过滤器并移除 jobs 覆盖；必须核真实执行数，不能以零测试通过回传。
- 最小受管回归需实际运行基底已有 sparse 下层 16 项（generation、swap-remove、最高合法 index、空 locator 释放与混合操作）、本结构模块 3 项，以及 identity storage / query 直接消费者。同一受管源码闭包精确保留已归属的既有 Runtime08 EventReader 闭包类型修正（snapshot4599）；其编译必要性仍待受管结果确认，不吸收其他活动 owner 的 dirty 源码。原 ticket `fc66d7eb3e7f4a58a160e073ac46e99c` 当前为 `snapshot_stale`，不得复用为通过证据。
- 当前外部工作树捕获漂移和 metadata 代理故障仍未恢复；保留源码并挂起受管动态门，不重复已有请求、不手工 Cargo或 caller archive。原文 entities/components/archetypes 1/1k/100k、change 0/1/100%、hash/downcast/move/cache/bytes/p95，以及百万实体/RSS/WPR/产品与 query/table 验收均继续开放。没有完整通过、failure return、closeout 或提交完成声明。

### 源码冻结、独立审查与待提交规格

- 源码冻结 snapshot `4601`，请求 `e997ba3492e34215995708ebfe082956`；结构测试 SHA-256 `ee2d32d03f5fe305dfe33f27c5f65d6ce737a1a1e0611a8baa09955f3d8d9c18`。`rustfmt +1.94.1 --check` 与两路径 `git diff --check` 实际通过。独立 scoped 审查在收窄编译必要性措辞后为 Critical / Important / Moderate = `0 / 0 / 0`；29 条新旧结构断言在真实 HEAD 路径静态成立，反向还原精确回到领取前测试 SHA-256 `bd4c0f9eb8a2d1bd0ba613396a84e6d10b2c9aac05eb4ddc4fba5d34684c7e6f`。这些是静态结构与源码审查证据，非动态行为或正式命名 closeout 审查结果。
- 四个受管命令的待提交规格保存在 `.codex/tmp/failure-roll-01a0df1a-runtime08-columnar-validation-prepared.json`，状态 `prepared_not_submitted`，accepted request/ticket 列表为空。请求 ID 按 `failure-roll-01a0df1a-runtime08-columnar-4601-{sparse-lower|storage-owner|identity-storage|query}-20260927-r1` 分别保留；需要真实执行 sparse 16、结构 3、identity storage 35、query 56 项，并核实际输出允许的额外匹配。identity 35 是已挂载父模块 28 加 child 5/2；不会把无挂载文件计入通过。
- 规格源码闭包只包含本结构测试和已归属 snapshot4599 的消息测试支持修正；仍未证明该注解对编译是否必需。生产文件来自同一 pinned baseline，没有 overlay 活动 owner 的 diagnostics/queue/cursor 等脏源码。原 messages 待验规格及 lifecycle 独立保留，不因本次修复关闭。
- 已向外部 zr_vm 实际写入/构建根任务各排入一份就绪协调请求（`01a0e382-5de0-7141-be89-884a64de2cd7`、`01a0e382-6188-7773-ad08-de23257110d9`），尚未证明 quiet 窗口或捕获成功；本机联网方式选择也仍待回复。保持本项待验，不重复验证、提交或消息，不生成 fixed return。
