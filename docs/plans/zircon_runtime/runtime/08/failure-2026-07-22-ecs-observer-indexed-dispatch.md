---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: ecs-observer-indexed-dispatch
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/ecs/observer
  - zircon_runtime/src/scene/ecs/lifecycle.rs
  - zircon_runtime/src/scene/world/observers.rs
tests:
  - cargo test -p zircon_runtime --lib ecs_observer --locked --jobs 1 -- --nocapture --test-threads=1
  - observer storm scan/allocation counters
---

# Runtime08：ECS observer indexed dispatch交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：scene ECS observer 6/6逐Rust文件审查，PERF-MVP-484
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有observer注册、移除、触发时序与reentrant dispatch语义。
- 生命周期键：`ecs-observer-indexed-dispatch`

## 失败现象与复现证据

`ObserverStore`把lifecycle、global event和entity event分别保存在三条Vec。每个callback查询先调用`*_callback_count`全扫一次，再全扫一次筛选；随后分配Vec并为每个命中callback clone Arc。entity event按`(TypeId, EntityId)`仍扫描全表。`World::trigger_component_lifecycle`还clone descriptor type-name String，并为每个callback clone完整event。

## 最低共享层根因

ObserverId只有单调编号，没有key bucket或id→slot定位；为允许callback reentrant修改World，dispatch通过逐callback Arc复制脱离store borrow，却没有generation-owned bucket artifact，因此把全局扫描和命中数分配放到每次trigger热路径。

## 架构修复验收

- lifecycle按`(kind, component_id)`、global event按`TypeId`、entity event按`(TypeId, entity)`建立canonical bucket；ObserverId可O(1)或O(logN)定位和移除。
- 注册/移除发布immutable generation bucket；dispatch只取得一个共享bucket handle后释放store borrow，允许reentrant add/remove并定义当前dispatch可见性。
- 同bucket保持注册顺序；remove during dispatch、recursive trigger和entity despawn cleanup不跳过、不重复、不悬空。
- lifecycle payload借用或共享descriptor稳定type-name，callback fanout不按命中数复制String/event owned payload。
- observers 0/1/1k/100k记录full scans、alloc、Arc/String clones和dispatch p95：stable trigger全表scan=0、临时分配常数。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止保留三条Vec为truth并另建长期可能漂移的非generation索引。
- 禁止callback执行期间持ObserverStore或World全局锁。
- 禁止用swap_remove破坏已定稿的callback注册顺序。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
