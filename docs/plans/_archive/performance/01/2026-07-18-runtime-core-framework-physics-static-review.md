---
related_code:
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/physics/runtime/src/manager/world_sync.rs
  - zircon_plugins/physics/runtime/src/manager/service.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/runtime.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/03-physics.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
reference_sources:
  - dev/Fyrox/fyrox-impl/src/scene/graph/physics.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
tests:
  - thirty-two of thirty-two framework physics Rust files reviewed against current source
  - focused production trace through plugin manager world sync and builtin backend
  - current-source Cargo, scale counters and F2 product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime physics framework逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已按当前工作树完整阅读`zircon_runtime/src/core/framework/physics/**` 32/32个Rust文件，并在外部领域错误契约迁移加入`PhysicsSettingsStoreError`后补读最新`manager/mod/tests`与新文件。framework主体是DTO与trait；为验证其运行成本，另聚焦追到plugin的`manager/world_sync.rs`、`manager/service.rs`和builtin backend runtime。该调用链抽样不等于`zircon_plugins/physics/**` 85文件逐文件验收，插件目录继续留在`pending.md`。

## PERF-MVP-335：固定步全量owned world snapshot与重复扫描

`PhysicsManager`当前以owned `PhysicsWorldSyncState`接收完整bodies/colliders/joints/materials，并以owned clone返回`synchronized_world`；query也返回新Vec。产品`tick_scene_world`对builtin即使`step_plan.steps == 0`仍调用`build_world_sync_state`，Jolt每次真实tick同样先构造全量snapshot。构造从`world.node_records()`开始复制、排序全场景node；随后每node调用`world_transform`，并递归复制convex points、heightfield payload、compound children、material/joint/skeleton Strings与metadata。

builtin manager随后再次sanitize全量snapshot、重算contact/trigger、复制previous trigger pairs，并以`sync.clone()`保存第二份world；`synchronized_world`再深clone。backend自己的`world_sync`也复制全部body/collider；每step先复制全部constraints，再按world重建snapshot与events。ray/shape query复制query/filter，重新构造world snapshot、线性扫描所有colliders并排序。于是稳定场景的主线程成本随scene nodes、physics objects、query count和contact density共同放大，而不是随changed objects或broad-phase candidates增长。

## 参考引擎裁决与实施边界

Fyrox的Graph长期持有`PhysicsWorld`、broad/narrow phase、body/collider/joint sets与native handles；graph虽仍逐node调用`sync_native`，但具体属性通过`needs_sync_model`/`try_sync_model`只在dirty时写native对象，且源码明确避免无变化时调用昂贵的collider `get_mut`。这支持Physics03既有change-detection方向，但当前Zircon的M1-T4只减少命令重复下发，尚未消除`tick_scene_world`的全场景owned projection、manager clone与builtin query重建。

Physics03/Runtime07应硬切persistent per-world backend state与generation delta：scene提供dirty body/collider/joint/material query和一次计算的world transform；稳定Entity→Body/Shape/Constraint handle table只增量create/update/remove；零step且generation稳定时snapshot build为0。查询直接走长期broad phase并写caller-owned/reused output，事件使用clear/swap有界buffer；debug snapshot按需从generation view物化，不能继续作为每帧同步协议。迁移需一次完成，不保留full-snapshot与delta双热路径。

## 验收要求

按scene nodes/physics bodies/colliders 1/1k/100k、changed 0/1/10/100%、fixed 30/60/120 Hz、queries 1/1k/100k、contacts sparse/dense记录node projections、world-transform visits、shape builds、clone bytes、alloc/realloc、manager/backend lock wait、broad/narrow candidates、event queue depth/drop与CPU p50/p95/p99：stable generation全量node projection/shape clone/snapshot clone=0，零step稳定帧sync build=0，query成本由broad-phase candidates主导，stable output/event buffers realloc=0。builtin/Jolt parity、active body回写、create/update/remove、trigger/contact顺序、Cargo及F2产品trace全部通过前，framework physics不得移入`review.md`。
