---
related_code:
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime/src/scene/tests/derived_state
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
reference_sources:
  - dev/bevy/crates/bevy_transform/src/systems.rs
  - dev/godot/scene/3d/node_3d.cpp
tests:
  - zircon_runtime/src/scene/world/generation/tests.rs
  - zircon_runtime/src/scene/tests/derived_state/hierarchy_rebuild.rs
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_archetype_index_structure.rs
  - current-source Windows zircon_runtime world tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene world core逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/world/**`核心状态/访问第一批当前源 **20/47** 个Rust文件、**2,838** 行、**10** 个就地tests已逐文件阅读：bootstrap、change detection、commands、component access/type registry、derived/dirty state、error、event mirror/events、generation及tests、hierarchy、identity、messages、root mod、observers、performance diagnostics、schedule与World storage。另读derived-state结构测试与archetype index/record支撑代码；dynamic components、project I/O、property access、query/records、render extract和typed API留给后续独立批次。

## 已直接修复

- `World::remove_entity`已经能从`EntityRegistry`取得目标archetype与row，却在每次despawn后调用`rebuild_archetype_index`全世界重建；递归删除K个节点会反复扫描剩余N个entity、重建signature并移动记录。现由`ArchetypeIndex::remove_entity_at`按可信row直接swap-remove，只更新被交换entity的table row，然后失效被删generational handle；无需全量重建。
- 新行为测试覆盖同archetype中间entity删除后swapped entity位置/组件可读，源码守卫禁止despawn回退全量refresh。源码守卫先RED后GREEN，scoped rustfmt/diff通过。本轮归PERF-MVP-458。
- hierarchy validity仍为每entity沿parent chain验证，但旧实现为每entity新分配`HashSet`。现单次rebuild只分配一个visited scratch并逐entity clear复用；这仅降低分配常数，不把O(N×depth)根因声明为解决。
- current-source受管Cargo test lane被`runtime10-runtime03-animation-frame-demand-producer-20260722`精确预约，本轮未启动Rust编译/测试，也未运行raw Cargo。

## P0派生状态根因

任意hierarchy dirty会先复制全entity parent map，再为每entity独立沿祖先链验环，最坏O(N×depth)。随后ActiveHierarchy和WorldTransform分别重新构建一份root/children HashMap并递归全树；NodeCache又全entity深clone宽`SceneNode`。深链递归还存在栈深风险。dirty bit只区分domain bool，不携带changed roots/descendants，导致单点rename/reparent/transform也容易触发多轮全场工作。

PERF-MVP-459交接Runtime07，Runtime08负责ECS identity/change tick支撑，Editor05消费共享投影：维护generation-owned hierarchy topology（parent、dense children range、roots、depth/topological order）与dirty-root frontier；cycle/reparent在事务入口验证，active/transform按受影响subtree迭代更新，NodeCache/render/inspection按component generation消费delta。不得再保留每stage临时children map或递归深链双路径。

## 参考引擎对照

Bevy transform系统用`Changed<Transform/ChildOf>`和`TransformTreeChanged`把dirty传播到祖先，遇到已dirty分支提前停止，并在多线程配置复用local/shared bitset与buffered channels；稳定静态子树可完全跳过。Godot `Node3D`持久维护parent children列表，transform变化只递归标记相关子树dirty，并把通知加入deferred change list，非主线程通过deferred call回主线程，而非每帧重建全树children索引。Zircon应保留确定性schedule，但把持久拓扑、dirty frontier和迭代传播作为World authority。

## 动态验收

1. current-source Cargo：identity/archetype/despawn、recursive remove、generation、hierarchy validity/active/world transform、deferred schedule与observer时序。
2. entities/depth/subtree为1/1k/100k与1/64/100k，single/batch/recursive delete记录entity scans、signature builds、archetype moves、alloc、p95：PERF-MVP-458要求每despawn full archetype rebuild=0、swapped row updates≤1。
3. PERF-MVP-459记录stable/rename/reparent/transform/active下parent visits、children-index builds、subtree visits、SceneNode clone bytes、stack depth和stage wall：stable全部为0；single transform近affected subtree；100k深链无递归栈溢出；F2/F4产品trace与current-source Cargo通过。

动态Cargo与产品规模验收未完成，因此本批继续保留在`pending.md`，不进入`review.md`。
