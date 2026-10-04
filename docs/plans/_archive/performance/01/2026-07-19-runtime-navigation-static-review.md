---
related_code:
  - zircon_runtime/src/navigation
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/godot/modules/navigation_3d/nav_map_3d.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavigationOctree.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Source/DetourNavMeshQuery.cpp
  - zircon_plugins/navigation/runtime/src/agent/repath.rs
tests:
  - zircon_runtime/src/navigation/runtime/tests.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs::performance_contract_tests
  - current-source Windows Cargo and navigation-scale product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime navigation逐文件性能静态审查（2026-07-19）

## 范围与覆盖

`zircon_runtime/src/navigation/**`当前源 **12/12** 个Rust文件、**1,553** 行、**4** 条测试已逐文件阅读，覆盖module/operation registration、builtin manager/state、baked mesh构建与path/sample/raycast、agent tick、avoidance、world scan及全部测试。现有测试证明单网格基础查询和agent写回，不覆盖规模、预算、锁、分配或operation异步性。

## 关键瓶颈

- **PERF-MVP-437 / Plugins05**：all-agent tick对`World::node_records()`至少做agent与obstacle两次全量owned扫描，clone动态component JSON并逐项`serde_json::from_value`；之后每agent重复path+sample并全扫agent/obstacle避障。single-agent入口也先收集全部对象，未复用Plugins05已有`NavRepathBudget`和DetourCrowd owner。
- **PERF-MVP-438 / Plugins05/Runtime11**：`build_adjacency`枚举所有polygon pair，shared-index判断又在线性Vec中`contains`；path/raycast/sample每次线性找polygon，A*每query新建两个P长Vec和heap，并在唯一state mutex内完成。generated snapshot按值clone完整asset，bake/clear/restore又走PERF-MVP-435的caller同步operation。
- `selected_mesh(None)`每次扫描loaded handle求min、tick内复制agent type String及stats/snapshot owned返回属于次级成本，随437/438的generation snapshot与typed projection一起收敛，不另建重复任务。

## 本轮直接止损

`BakedNavMesh::new`现在一次把资产area costs投影为固定64项表，A*每次edge expansion改为O(1)索引；表保持原有“首个重复area获胜”、最小cost 0.01及未知area回退语义。预计算后删除结构体中的完整`NavMeshAsset`字段，加载网格不再在generated snapshot之外常驻第二份vertices/indices/tiles/links。源码契约完成RED→GREEN，`rustfmt --edition 2021`与`git diff --check`通过。

## 参考约束与动态验收

Godot `NavMap3D`以immutable iteration slots、read ownership与有限path query slots允许查询并发，并可把dirty iteration build派发WorkerThreadPool；Unreal NavigationSystem用navigation octree和dirty-area增量范围组织世界输入；仓内Detour `findNearestPoly/queryPolygons`按tile/BV候选查询，Plugins05已实现repath budget与per-navmesh crowd。Zircon后备实现不必复制API，但必须遵守增量world projection、空间候选、immutable generation、bounded query scratch和每帧repath预算。

动态门需要1/100/10k nodes/agents/obstacles、1/1k/100k polygons/queries及1/64MiB asset，记录node visits、JSON clone/deserialize、path query、neighbor check、build comparison、nearest visits、scratch alloc、mutex wait/hold、snapshot clone bytes、RSS和p95；F4补bake/clear/undo与PIE crowd trace。受管Cargo仍因validator非JSON入口错误未到Cargo，规模counter和产品trace完成前留在`pending.md`，不进入`review.md`。
