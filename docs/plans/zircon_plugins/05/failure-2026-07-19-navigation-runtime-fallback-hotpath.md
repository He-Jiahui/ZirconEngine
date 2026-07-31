---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: navigation-runtime-fallback-hotpath
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/world_scan.rs
  - zircon_runtime/src/navigation/runtime/avoidance.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
  - zircon_runtime/src/navigation/runtime/state.rs
  - zircon_plugins/navigation/runtime/src/agent/repath.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
tests:
  - powershell -NoProfile -Command "Select-String -Path 'zircon_runtime/src/navigation/runtime/world_scan.rs' -Pattern 'node_records\(\)|serde_json::from_value'"
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_runtime -VerboseOutput
---

# Plugins05：Navigation runtime后备热路径交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：P5 Navigation逐Rust文件性能审查与PERF-MVP-437/438
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：最低共享原因是Plugins05 production/fallback driver、repath/crowd调度与navmesh query owner尚未收敛为同一套预算和索引约束。
- 生命周期键：`navigation-runtime-fallback-hotpath`

## 失败现象与复现证据

内置all-agent tick分别为agent和obstacle调用owned `World::node_records()`，clone动态JSON并反序列化；single-agent tick仍收集全部agent与obstacle。每个agent随后每tick执行path、全量agent/obstacle avoidance和sample。内置mesh加载枚举所有polygon pair建邻接，path/sample/raycast线性寻找polygon，A*每query分配P长scratch并在全局state mutex内运行。

当前12/12文件静态读完；源码计数可稳定命中上述`node_records()/serde_json::from_value`，但现有4条测试没有world visits、repath count、neighbor checks、polygon comparisons、query allocations或lock hold断言。area-cost查表和重复完整asset常驻已局部止损，不能视为架构问题已修复。

## 最低共享层根因

Runtime后备实现按通用动态World快照临时发现导航组件，Plugins05 production实现则已有`NavRepathBudget`、per-navmesh crowd和Detour查询；两条路径没有共享typed/change-tracked projection、预算和generation snapshot契约。后备mesh又自行维护无空间索引的矩形图，并把可并发读查询与写owner放在同一mutex。

## 架构修复验收

- production与fallback driver共享typed component/change generation：每frame至多一次增量agent/obstacle projection，stable tick不clone/deserialize JSON；single-agent入口不全世界扫描。
- 复用`NavRepathBudget`和per-navmesh crowd批量update/writeback；静止目标不重复寻路，avoidance只访问空间邻域并有确定性公平预算。
- 后备mesh以edge/tile key和空间索引近线性构建邻接/nearest候选，查询借用immutable generation snapshot并复用有界scratch/query slots，不持有全局写锁。
- bake prepare通过Runtime11 bounded worker，main thread只做generation校验和短apply；undo artifact不深复制多份完整navmesh。
- 1/100/10k agents/obstacles及1/1k/100k polygons测试加入确定性work counters，并通过Plugins05受管Windows package gate与F4 bake/PIE产品trace。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅提高预算、换并行iterator或把现有全量JSON扫描搬到线程池；输入投影和工作量必须先有界。
- 禁止为绕开mutex复制整份navmesh或为每query创建无界线程/缓存；snapshot、scratch和队列必须有generation与容量owner。
- 禁止删除后备路径或弱化路径/avoidance/bake undo语义来隐藏性能失败。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
