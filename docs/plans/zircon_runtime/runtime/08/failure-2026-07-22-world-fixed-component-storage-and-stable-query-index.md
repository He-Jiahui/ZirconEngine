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

Open state: `待修复`; no pass is claimed.
