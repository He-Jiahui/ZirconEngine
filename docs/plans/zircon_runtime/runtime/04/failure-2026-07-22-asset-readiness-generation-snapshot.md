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

### 2026-09-08 current lower-layer evidence

Rolling fixing Session: `failure-roll-01a07160-runtime04`. Existing ticket
`21c34d2bf640450fbe21258dd4ce2f95` retains its original owner and receipt;
this continuation does not replace or resubmit it.

The current lower owner is
`zircon_runtime/crates/zr_resource/src/manager/readiness_projection.rs`,
SHA-256 `348909388f2583925418466c4c0448ffa51988def927bab0ef9ce47eb90dc564`.
It computes strongly connected components using an explicit traversal stack,
fails cyclic readiness closed, and publishes the affected reverse closure.
The Runtime facade reads one immutable generation and marks dependency IDs
discovered before queuing them. These source observations alone are not acceptance.

Frozen managed input `interface03-v8-layout-3199-20260908`, manifest
`58164b9667051219bec66342bcb79d9325648eb2cfcea1f3ad307ebc743c1abd`, contains
the same projection and test bytes as the current source. Resource library job
`64402e468e94498f96357bb26d716f81` passed 224 tests, failed 0 and ignored 10.
Two old RED cases were among the ignored tests and were subsequently executed
explicitly by managed job `fa7175e14cd44ddfa9b6d2d10c5ef1aa`:

- `self_and_multi_node_cycles_do_not_publish_recursive_loaded`: passed.
- `deep_chain_10000_publishes_without_native_stack_growth`: passed.

The second run used the `manager::readiness_projection::tests::behavior_red::`
filter with `--ignored`, Windows native, no default features and Cargo `--locked`:
2 passed, 0 failed, 0 ignored, 232 filtered out. Log:
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/interface03-v8-layout-3199-20260908/results/frameworks01-resource-readiness-ignored-3199.log`.
The behavior source hash is
`1ed6d7444c65bd6bb52cd7dd54c34b7f69a21673ffc14959d8763f73513e2de9`.

The existing release profile orchestrator completed under Runtime04 in managed
job `dcdbb970628e4d608d67de51a3c81000`, exit 0. Its orchestration CSV records
32/32 completed workers, each exit 0 with raw and summary reports: 16 scenarios
in both `manager_end_to_end` and `evaluator_only` harness scopes, 31 measured
samples and 3 warmups each. The harness's manager scope materializes prepared
authority-shaped updates; it is not the complete locked Runtime asset facade.
Projection, generation, profile, allocation-counter and resource-manager source
hashes were checked against the frozen input and remain identical.

Results are under the input's
`results/runtime04-readiness-profile-20260908` directory. Representative manager
harness measurements (p95 milliseconds, allocation/closure/edge counters p50):

| Scenario | p95 ms | Allocations | Affected closure | Edge visits |
| --- | ---: | ---: | ---: | ---: |
| Initial chain, 100000 nodes | 576.4521 | 600943 | 100000 | 99999 |
| Leaf reload chain, 10000 nodes | 23.8741 | 20293 | 10000 | 9999 |
| Leaf reload diamond, 100000 nodes | 435.5870 | 282602 | 100000 | 199996 |
| Leaf reload fanout, 100000 nodes | 54.5617 | 31 | 2 | 99999 |
| No change, 100000 nodes | 0.0153 | 2 | 0 | 0 |

These are observations, not an invented performance threshold. In particular,
fanout still scans the affected parent's direct edges; a small reverse closure
alone does not prove constant update cost. The no-change harness performs no
graph traversal, while retaining its two input-materialization allocations.

Its inventory includes 1/2/64/1k/4096/10k/100k cases, but does not alone cover
the original 10-node matrix, three lock classes, facade/report clone counts,
concurrent generations or the complete Runtime `readiness_report` gate.
Those gaps, formal fixing-Session binding, independent review and closeout remain
open. No source was changed and external zr_vm remains excluded.

### 2026-09-08 facade execution found a lower authority type defect

Managed job `195f6995f19e49cc9e6c9a9f7b7bd919` passed Cargo check and
executed all 26 `asset::tests::facade` tests on Windows, static/no-default,
Cargo `--locked`: 15 passed, 11 failed, 0 ignored, 6823 filtered out.
Frozen input `editorui12-button-focus-3228-20260908` has manifest
`e1bb2d3955a335f2f0ba3462155d9dd5c9d6db866d535c760ac1ebd77f5401e6`;
its `results/runtime04-readiness-facade-3228.json` and `.log` retain the result.
The current readiness facade, manager and nine facade test files matched those
input bytes. Current catalog-import drift remains outside this evidence.

The eleven failures share typed root `NotLoaded`, even after real payload
registration. The resource authority's `readiness_source_update` calls
`as_any()` on `Arc<dyn ResourceData>` and records the Arc container TypeId,
which the typed facade correctly rejects. The projection's earlier passing
tests manually provide TypeId and do not exercise this boundary.
The shared foundation repair and actual-payload regressions are tracked in
[Frameworks01 resource readiness payload type erasure](../../frameworks/01/failure-2026-09-08-resource-readiness-payload-type-erasure.md).

The Runtime04 functional gate is failed, its original ticket is retained, and
the declared performance matrix and formal acceptance remain open. The lower
repair must pass before rerunning this same full facade batch.

### 2026-09-08 lower repair and original functional gate passed

Frameworks01 source snapshot 3237 fixes the authority's concrete payload TypeId.
Its three new regressions first executed as 0 passed / 3 failed against the
unchanged production expression, then passed in the complete resource library:
227 passed, 0 failed, 10 ignored. The same frozen input
`frameworks01-readiness-payload-3237-20260908`, manifest
`5dc7c3d6891874a69436ba9e7641c9e0863eb7ed548c05db7306233c5da9543d`,
then passed all 26 Runtime `asset::tests::facade` tests, 0 failed, 0 ignored,
6823 filtered out, managed job `fede77172a564c74baa6ce25f9dcd425`.
All eleven original functional failures are resolved without changing the
facade assertions. The linked Frameworks01 record owns the source repair,
227-test lower gate, exact hashes and independent C0/I0/M0 review.

One result-boundary read found original ticket
`21c34d2bf640450fbe21258dd4ce2f95` terminal as `snapshot_stale`; its owner remains
`runtime04-artifact-chunked-generation-r2-20260731`. Earlier receipt-only history
above is retained, and no replacement request was submitted. The Runtime04
functional gate is now green on the stated immutable input. The original
performance matrix, current catalog-import drift, formal fixing-Session binding,
return and closeout remain open. External zr_vm remains excluded.
