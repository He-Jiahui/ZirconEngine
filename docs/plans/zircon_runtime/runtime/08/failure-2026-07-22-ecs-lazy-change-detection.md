---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: ecs-lazy-change-detection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/ecs/change_detection
  - zircon_runtime/src/scene/ecs/query/query_data.rs
  - zircon_runtime/src/scene/ecs/system/res.rs
  - zircon_runtime/src/scene/ecs/resource_store/store.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage
  - zircon_runtime/src/scene/world/change_detection.rs
tests:
  - cargo test -p zircon_runtime --lib ecs_change_detection --locked --jobs 1 -- --nocapture --test-threads=1
  - fetch-only versus true-mutation change counters
---

# Runtime08：ECS lazy change detection交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：scene ECS change_detection/resource/resource_store 13/13逐Rust文件审查，PERF-MVP-483
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有component/resource tick authority、query data与system param语义。
- 生命周期键：`ecs-lazy-change-detection`

## 失败现象与复现证据

`TableComponentStorage::get_mut_at_tick`、`SparseComponentStorage::get_mut_at_tick`和`ResourceStore::get_mut_at_tick_with_ticks`在返回mutable value前立即`set_changed(tick)`。因此Query `Mut<T>`和`ResMutParam<T>`即使只取得item/param、只调用`is_changed`或完全未DerefMut，也已把本帧记为changed；后续`Changed<T>`系统、render extract与editor projection会被无效唤醒。

Bevy对照`dev/bevy/crates/bevy_ecs/src/change_detection/params.rs`把value与mutable changed tick放入wrapper，并在`DerefMut`、`into_inner`、`as_mut`或显式`set_changed`时标记；本仓wrapper只保存ticks副本，无法延迟写回。

## 最低共享层根因

storage/resource API把“取得独占借用”和“发生语义修改”合并为一个操作，wrapper没有changed tick可变authority与当前tick，raw `&mut T`和change-aware `Mut<T>`也共用同一eager入口。

## 架构修复验收

- table、sparse与resource提供sound split borrow，向change-aware wrapper同时交付value、mutable changed tick与current tick；fetch不改tick。
- `Mut<T>`/`ResMut<T>`首次`DerefMut`、`into_inner`、`as_mut`或显式`set_changed`才mark；重复mutable deref每item至多写同一tick，不增加额外downstream trigger。
- 返回raw `&mut T`的World API继续在交付时mark，因为之后无法观察写入；另提供命名明确、受控的bypass/unchanged映射，而非静默漏报。
- fetch-only、read-through-mut、true mutation、optional resource、wraparound、table/sparse与cached query均有行为测试。
- entities/resources 1/100k、mutate 0/1/100%记录changed matches、downstream system runs和tick writes：fetch-only false positive=0，真实写不漏报。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止在wrapper Drop时无条件mark；这仍把只读mutable fetch误报为changed。
- 禁止只修ResMut或只修table，留下另一条mutable param语义分叉。
- 禁止以unsafe裸指针缓存tick/value绕过aliasing证明。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
