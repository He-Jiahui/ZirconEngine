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
  - zircon_runtime/src/scene/tests/ecs_typed_api/bundle_width.rs
  - zircon_runtime/src/scene/ecs/storage
  - zircon_runtime/src/scene/ecs/archetype
tests:
  - one_thousand_bundle_spawns_publish_only_final_archetypes
  - one_hundred_thousand_bundle_spawns_publish_only_final_archetypes
  - managed Windows core-min zircon_runtime lib-test filter ecs_typed_api::bundle_width
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

Open state: `bundle-width test call repaired / managed current-source verification unavailable`; no pass is claimed.

### 2026-08-08 current-source compile recovery

- The Plugins01 managed native live-host gate was unable to compile its target tests because this
  Runtime08 test passed a bare `Health` component to `World::spawn` at both the 1k and ignored
  100k width gates. `Bundle` deliberately implements unary tuples, and `World::spawn` accepts
  only `B: Bundle`; the two calls now pass `(Health(...),)` through that shared normal path.
- The former `bundle_transaction.rs:530` `E0308` is already corrected in the current shared
  source and was not changed by this repair session. Scoped Rust 1.94.1 `rustfmt --check`,
  `git diff --check`, and source-shape assertions confirm zero bare `Health` spawns and two
  unary tuple spawns.
- The required Windows managed core-min lib-test gate was first deferred by compatible pool job
  `775162eecde34cacb2e2b7d31584d1d4`; that job later released. The post-repair request then
  failed before job materialization with `database is locked`, and its one controlled retry was
  not submitted after coordinator health preflight timed out (`cargo.acquire`, two attempts).
  No Cargo process, test result, broad result, or fixed return is claimed until the same focused
  gate reaches a terminal current-source outcome.

### 2026-08-09 current-source lifecycle audit

- `BundleTransaction::begin_commit` now calls
  `World::register_prevalidated_node_identity_without_components`, not the record-restoration
  path. Staged defaults and explicit bundle values are consequently the only component
  publishers. `bundle_default_overrides.rs` covers an empty spawn and explicit
  `Name`/`Hierarchy`/`LocalTransform` overrides with `Add=1`, `Replace=0`, and `Insert=1` per
  affected component; observers see the final values at first dispatch.
- Scoped `rustfmt +1.94.1 --check` and `git diff --check` are clean for the owned bundle/runtime
  sources. This is static source evidence only. The failure remains open: no managed Cargo gate,
  scale probe, independent post-validation review, failure return, or coordinator milestone
  commit is claimed.

### 2026-08-09 structure and typed-error audit

- The Runtime08 public path is now `World::{spawn, insert_bundle}` plus the narrow
  `Bundle::stage_into` staging contract. `Bundle` has no commit/insert authority, and the
  transaction has no legacy `insert_into` caller or compatibility surface. This keeps behavior
  in the named `bundle_transaction` leaf owner rather than a root facade.
- The transaction's externally reachable failure cases use typed `SceneError` variants
  (`BundleFinalStateNotValidated`, duplicate/width/reservation limits, and transaction
  invariants); a source scan found no `SceneError::Message` fallback in the bundle contract.
  `ArchetypeSignature::with_component_added` is idempotent, so staged default/explicit overrides
  produce the same final signature rather than duplicate membership.
- `bundle_transaction.rs` is 659 lines, below the structure convention's 800-line warning
  threshold. These are source-shape checks only; the failure remains open pending the managed
  Windows lib-test, scale probes, and independent post-validation review.
