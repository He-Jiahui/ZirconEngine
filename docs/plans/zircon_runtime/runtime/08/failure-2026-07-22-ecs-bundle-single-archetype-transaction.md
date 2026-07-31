---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: ecs-bundle-single-archetype-transaction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/ecs/bundle.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/ecs/storage
  - zircon_runtime/src/scene/ecs/archetype
tests:
  - cargo test -p zircon_runtime --lib bundle --locked --jobs 1 -- --nocapture --test-threads=1
  - 1..8 component bundle transition and rollback counters
---

# Runtime08：ECS Bundle单archetype事务交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：scene ECS root 16/16逐Rust文件审查，PERF-MVP-479
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有component storage、archetype signature与World mutation原子性。
- 生命周期键：`ecs-bundle-single-archetype-transaction`

## 失败现象与复现证据

1..8元tuple Bundle实现展开为N次`world.insert(entity, component)?`；spawn bundle先创建empty entity再逐组件插入。每次insert可产生ComponentStorage写、archetype迁移、change tick和lifecycle event，N组件bundle形成N个中间signature；任一步失败时前序组件已发布。

## 最低共享层根因

Bundle只暴露立即执行的`insert_into`，没有component-id/signature预声明、owned value staging、最终archetype reservation或batch commit合同。

## 架构修复验收

- Bundle可一次枚举component descriptors/ids并stage owned values；duplicate/schema/storage检查在World authority变化前完成。
- 计算最终signature，reserve最终archetype row并一次写入各canonical component columns；entity location只从empty/absent发布到final一次。
- change ticks、Add/Insert/Replace和observer/deferred事件在commit后按tuple stable order发布；失败丢弃staging且World/entity authority零变化。
- 与PERF-MVP-464单storage authority、467 affected-row transaction共用底层，不建立Bundle专用平行storage。
- bundle width 0..8及1k/100k spawns记录archetype transitions/signatures/storage moves/events/alloc：每entity final transition≤1、intermediate signatures=0、failure partial writes=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅在`spawn_bundle`外层clone整个World回滚。
- 禁止先逐组件插入再把archetype索引“修正”为最终值；中间storage/event仍已发生。
- 禁止为固定tuple宽度复制八套storage事务实现。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
