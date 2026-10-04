---
related_code:
  - zircon_runtime/src/scene/dynamic_scene
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
reference_sources:
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/godot/core/io/resource_loader.cpp
tests:
  - zircon_runtime/src/scene/dynamic_scene/document/migration/project_world.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/capture.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/validation.rs
  - zircon_runtime/src/scene/dynamic_scene/spawn_task/task.rs
  - zircon_runtime/src/scene/tests/dynamic_scene
  - current-source Windows zircon_runtime dynamic-scene tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime dynamic scene非session基础逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/dynamic_scene/**`排除`session/**`后的当前源 **35/35** 个Rust文件、**2,724** 行、**4** 个就地test已逐文件阅读；覆盖root/error/patch/remap、versioned document+migration、entity/value DTO、scene asset bridge、capture/validation/spawn、background spawn task及asset reload queue/reports。`session/**`当前563文件另行拆分，不计入本证据。

## 已直接修复

- legacy project-world migration拥有输入`Value`，原`as_object().cloned()`深复制document和world，再clone entities数组。现以`Value::Object/Array` match消费owned Map/Vec，并从world remove entities；字段输出合同不变。
- `World::node_records()`已经按entity id排序，dynamic capture随后再次`sort_by_key`。现删除第二次排序。
- component descriptor唯一性原clone每个type-id进`BTreeSet<String>`；现借用`BTreeSet<&str>`，仅duplicate错误真正clone id。
- 三组源码守卫先RED后GREEN，rustfmt/diff通过，归PERF-MVP-470。

## Asset reload主线程与队列放大

`drain_events`循环到channel为空，无frame count/time/bytes预算；每个event调用`take_superseded_pending`，完整drain/rebuild pending Vec，burst最坏O(E×P)。pending任务和latest-revision表无容量/TTL；任务使用`DetachOnDrop`，superseded仅丢handle，后台旧prepare仍可消耗worker。`take_ready_report`每tick全pending重建，`tick_into_level`再在一次Level world mutex范围内apply全部ready scene，没有apply预算。

PERF-MVP-471交接Runtime04/11：per-AssetId latest-only single-flight slot、cancel/generation ticket、bounded drain/schedule/apply、queue age/drop/peak诊断与lifecycle prune；ready apply消费PERF-MVP-472 transaction并可续作，不能只给Vec reserve或提高队列上限。

## Capture/preview/spawn重复与非事务写

后台`PreparedDynamicSceneSpawn::new`只执行payload schema/duplicate验证，不绑定target world。preview在主线程构造remap、遍历parents/components/resources并物化remapped JSON/value；actual spawn重新build remap，先注册type/insert records，再逐component/resource field写入。每component clone adapter和完整field-info Vec，`should_write_field`对每field线性扫描metadata形成O(F²)，field adapter写还可能逐字段clone整组件/迁移archetype；中途失败会留下partial World。

capture反向路径先深投影全部SceneNode，再为每entity遍历全TypeRegistry并逐type contains/read/sort，复杂度近N×T；serializable field过滤又逐field扫metadata。PERF-MVP-472交接Runtime08：以target world/schema generation编译唯一spawn transaction，preview借用同一plan，main thread只做有预算affected-row原子commit；capture按实际component storage/type generation遍历，不维护第二套全表投影。

## 参考引擎对照

Bevy scene API把load/resolve依赖与spawn/apply分成明确阶段，queued scene只在依赖ready后进入World；Zircon应进一步让resolve产物绑定target generation并复用到preview/apply。Godot PackedScene实例化消费预打包node/property表，ResourceLoader集中管理load/cache而非每asset event重复创建无界任务。这里采用的是“resolved artifact + bounded owner + atomic apply”原则，不复制其对象模型。

## 动态验收

1. current-source Cargo：legacy v0/v1/v2 roundtrip/error、capture deterministic order/components/resources、preview/remap/parent/failure atomicity、spawn task poison/result、asset reload latest/stale/removed/renamed/tick_into World+Level。
2. entities/types/fields/resources/events/pending 1/1k/100k、scene 1/64MiB、prepare 0/10/1000ms记录JSON/String/value/adapter clone bytes、sort/probes、pending scans/jobs/cancel、queue peak/age、world-lock/main/worker wall与p95。
3. 470当前要求三类full clone/resort/key clone=0；471最终per asset active≤1、每event不全扫pending、apply受预算；472 preview+apply compile≤1、failure authority零变化、field lookup O(1)。

受管Cargo、规模/burst counter与F2/F4产品trace未完成，因此该范围继续保留在`pending.md`，不进入`review.md`。
