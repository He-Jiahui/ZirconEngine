---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-fixed-component-storage-and-stable-query-index
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime/src/scene/ecs/query/query_state/cache.rs
tests:
  - cargo test -p zircon_runtime --lib ecs --locked --jobs 1 -- --nocapture --test-threads=1
  - typed storage, serialization, query-order and archetype scale fixtures
---

# Runtime08：World固定组件单一存储与稳定query index交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene world query/records/typed API 4/4逐Rust文件性能审查，PERF-MVP-464/466
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有component storage、entity location、archetype和query data boundary。
- 生命周期键：`world-fixed-component-storage-and-stable-query-index`

## 失败现象与复现证据

27类固定组件在World专用HashMap与generic ComponentStorage各存一份；insert clone双写，typed CRUD走长TypeId/downcast链，deserialize presence rebuild逐组件clone并反复迁移archetype，最后还全场重建。另一方面QueryState虽命中matched archetypes，cache miss仍全扫World.entities并binary-search membership；稀疏query没有从archetype index获得候选规模收益。

不能把query简单改成逐archetype遍历：现有合同要求stable world entity order，而archetype rows使用swap-remove且不保持该顺序。

## 最低共享层根因

World固定组件map、ComponentStorage、archetype record和stable entity vector没有统一row/order authority。typed access/serde为了便利复制组件正文，query为了稳定顺序回退全表扫描；bulk restore也没有“计算最终signature→一次publish”的transaction入口。

## 架构修复验收

- 单一ComponentStorage拥有组件正文和ticks；typed fast API与serde/project projection通过generated/static adapter table访问同一row，不再维护专用map第二truth。
- bulk restore/insert bundle先计算最终signature/table+sparse locations，每entity至多一次archetype publish；swap-remove精确修正被移动entity location。
- 单一stable query-order identity/index随spawn/move/despawn增量维护；matched archetype rows可按稳定world顺序合并或以dense order bitset访问，cache rebuild不扫描unmatched entities。
- entities/components/matches 1/1k/100k、0.1%/1%/100% match记录clone bytes、TypeId branches、moves/rebuilds、world/location/archetype visits与RSS/p95；正文owner=1，restore final publish≤1/entity，sparse query work近matches+index words。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止在双存储外再加presence cache；禁止“写storage后异步同步专用map”的最终一致双truth。
- 禁止为每次query collect所有candidate再全量sort来冒充indexed stable order。
- 禁止删除或暗改现有stable query iteration order合同；如需变更必须单独公共契约裁决，不由性能修复隐式完成。

## 修复结果与回传

Open state: `implementation_complete / managed_validation_pending`; no Cargo pass is claimed.

### 2026-08-11 current-source architecture reconciliation

- Dense 正文 owner 已硬切为每个 `ArchetypeRecord` 直接拥有的 `ArchetypeTable`；`ComponentStorage` 明确只拥有 SparseSet。旧 `TableComponentStorage`、`table_components` backing map 和 table-location facade 已从生产 owner 删除，结构守卫同时禁止这些退役真值回流。该裁决比早期“统一 ComponentStorage”措辞更严格：table/sparse 仍是双存储形态，但任一 component value/ticks 只有一个正文 owner。
- typed get/get_mut/ticks、change detection、render extraction、persistence projection 和 dynamic/component count 均按 `StorageType` 分派：dense 经 `EntityLocation -> ArchetypeIndex -> compiled column slot`，sparse 经 `ComponentStorage`。clone/Deserialize 先聚合每实体完整 owned row，再调用一次 `commit_component_row`；NodeRecord restore 与 bundle 同样只发布最终 row，不再逐组件构造中间 archetype。
- `QueryState` 只持有 `cached_archetype_plans` 与标量诊断，不缓存 N 个 entity/location 或 N*K component location projection。每个 matching plan 编译 table column slot/sparse binding，并记录局部 membership generation；新 archetype 只编译新增 plan，未匹配 archetype 的 membership 变化不会重建已有 query projection。
- `StableQueryOrderIndex` 以单调 world-order key 维护 archetype-row 映射；move/remove 同步修复 swap 行，clone/Deserialize 经 registry rebuild 重建。`cached_name_query_keeps_stable_world_order_across_moves_clone_and_serde` 已从真实 World 覆盖跨 archetype 移动、swap-remove、clone 与 serde round-trip，补齐此前仅有 private-index 单测的端到端缺口。
- 现有性能 fixture 覆盖 1/100k columnar query、stable table hot path 的 zero hash probe/Any downcast、100k bundle final-publish，以及 query plan/column-slot/membership-generation 计数。结构测试明确断言 sparse-only storage、sorted column slot 与无 per-entity cache projection。

### 待完成证据

- 仍须执行 frontmatter 声明的受管 `ecs` lib gate，以及 typed storage、serialization、stable-order 和 1/1k/100k scale fixtures；记录 raw terminal test count 与 query/storage counters。取得 terminal GREEN 前不得改名为 `fixed-*`，也不得声称 Cargo 或 p95/RSS 验收通过。

### 2026-08-14 current-source compile recovery

- The split query state temporarily exposed two `cached_archetype_plans` accessors: one on the
  cache leaf and one on the root state module. The root duplicate is removed; the cache leaf is
  the sole `pub(crate)` projection accessor, so mutable query construction does not reach into a
  private field or retain a second cache-owner API.
- `World::remove_entity` already requires a stable camera fallback after an active camera is
  removed. The missing `first_stable_camera_entity` helper is restored in `world/query.rs`, where
  it walks the existing stable entity order and checks the canonical `CameraComponent` presence.
  The removal owner retains its indexed mutation path and no fallback collection/sort is added.
- Rust 1.94.1 `rustfmt --check` passed for the two changed source files. Static checks confirm
  exactly one cache accessor definition and the restored camera helper. No Cargo command ran;
  managed `ecs` validation and the performance matrix remain pending, so this failure stays open.

### 2026-08-14 UI12 current-source visibility reconciliation

- The hierarchy owner can call `World::first_stable_camera_entity` through its `pub(super)` World
  sibling boundary. Scene project loading likewise reaches the `pub(super)`
  `normalize_scene_asset_after_load` owner in `project_io/document.rs`; neither repair widens a
  public runtime API.
- Property entry factories accept `FnMut`, so targeted reads can invoke their value closure through
  a mutable visitor without consuming it. The physics property reader and typed projection rebuild
  helper both use `pub(in super::super)`, which is exactly the World-parent visibility required by
  their sibling callers.
- These current-source anchors were part of the 2026-08-14 17-check static audit, and scoped
  `rustfmt +1.94.1 --check` passed for their owner files. No Cargo or performance acceptance is
  claimed; this failure remains open for the declared managed evidence.
