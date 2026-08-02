---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-readiness-generation-snapshot
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/facade/readiness.rs
  - zircon_runtime/src/core/resource
tests:
  - cargo test -p zircon_runtime --lib readiness_report --locked --jobs 1 -- --nocapture --test-threads=1
  - deep, wide, shared-dependency, missing-node, cycle and concurrent-generation fixtures
---

# Runtime04：asset readiness generation snapshot缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset root/load/facade逐Rust文件性能审查，PERF-MVP-493
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：聚合load state、dependency revision和一致generation snapshot依赖Runtime04的import/reload发布边界，不能由facade查询端建立第二份缓存truth。
- 生命周期键：`asset-readiness-generation-snapshot`

## 失败现象与复现证据

`readiness_report`先clone root并调用`load_states`遍历direct/recursive依赖，再为root查询runtime/payload并BFS同一依赖图生成rows。每个依赖分别clone registry record、读取runtime和payload；shared dependency在expanded判定前仍可能按每条incoming edge重复fetch。编辑器轮询会在稳定generation重复全部工作。

## 最低共享层根因

root/direct/recursive状态只在查询时从三套锁与依赖图临时投影，没有import/reload generation拥有的聚合状态、dependency revision或bulk一致snapshot。

## 架构修复验收

- import/reload提交时维护root/direct/recursive聚合state与dependency revision；稳定`load_states/is_loaded*`查询O(1)。
- 完整report从一次一致generation的bulk registry/runtime/payload snapshot或immutable readiness table构建，图遍历O(V+E)，每node record/runtime/payload fetch最多一次。
- changed dependency只重算受影响反向closure；失败候选不污染已发布generation。
- 保留missing/wrong-kind诊断、最浅depth、direct标记、cycle终止、确定顺序和serde输出语义。
- roots/depth/fanout/shared nodes 1/10/1k/100k记录edge visits、三类锁、clone/allocation、changed closure与p95；stable轮询不重复图遍历。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止在Editor或每个consumer缓存第二份readiness图。
- 禁止只合并两个facade函数但仍逐node重复获取registry/runtime/payload。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

### 2026-08-01 current-source implementation

- `core::resource` now publishes one immutable readiness generation containing
  registry/runtime/payload observations and dependency projections. Stable
  facade queries reuse the published root/direct/recursive state; full reports
  traverse one consistent snapshot and fetch each node at most once.
- Resource mutation paths invalidate and rebuild the affected readiness
  projection at the generation boundary. Missing nodes, wrong kinds, shallowest
  depth, direct markers, cycle termination and deterministic report order remain
  represented by the canonical facade result.
- The exact source was sealed as snapshot `1412`; managed ticket
  `21c34d2bf640450fbe21258dd4ce2f95` was accepted. This is receipt evidence only,
  not a terminal test claim.

Open state: `实现完成，受管验证待回执`; accepted closeout remains deferred.
