---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-22
summary_slug: realtime-sh9-graph-dispatch-parity
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/shader/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder/tests.rs
legacy_related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs; zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs; zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs; zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs; zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs; zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder/tests.rs
---

# realtime-sh9-graph-dispatch-parity: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：Render11 realtime IBL graph dispatch contract
- 修复责任计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Render11 realtime IBL graph dispatch contract` — Compile the terminal ProjectDiffuseSh9 realtime IBL graph. The graph ticket reports fixed workgroups [4,4,6], while the canonical IBL shader command plan and WGPU recorder encode [1,1,1].

## 最低共享层根因

Realtime graph metadata independently reconstructs a 4x4x6 workload instead of consuming the canonical SH9 dispatch contract owned by Shader06.

## 架构修复验收

- The shader plan, offline graph, realtime graph, and encoded WGPU SH9 command use one shared [1,1,1] dispatch constant; regression tests prove parity and the complete realtime ticket graph budget is 4124 workgroups.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

2026-09-11 索引复核：原始 `related_code` 中六个源码路径被分号压成一个字符串；保留原值于 `legacy_related_code`，并写入六条 canonical path。协调器 request `ac0646c6fd2a476fb2d512f3d4908413` 已完成导入，数据库现可逐路径解析。源码修复仍归历史 Shader06 Session，当前只完成 handoff 元数据整理；受管 library/product Cargo、PNG/timing 与 RenderDoc 门仍待原 owner 完成，故保持 `status: open`。

Open state: the shared dispatch repair and its graph/WGPU parity regressions are present in the current source. The recorder now carries the graph extent into the cacheable PMREM/SH9 command path and rejects graph/command drift on both cache hits and misses before compute encoding, pipeline creation, or parameter/bind-group creation. The success path adds one allocation-free `[u32; 3]` equality check per cacheable dispatch and does not change the 4,124-workgroup ticket budget. Pure regressions cover canonical `[1,1,1]`, reject legacy `[4,4,6]`, and pin validation ordering. Scoped `rustfmt --check` and `git diff --check` pass. Runtime87 has repaired the external Asset Registry `thiserror` E0599, but the first current P1-2 managed product attempt acquired job `c32547af1eef4453b5aa24f5c68228c8` and was orphaned before `cargo.start`; it did not execute Rust or produce a new artifact. This record remains `open` until a durable managed host completes the current library and product gates, including fresh PNG/timing and RenderDoc evidence. Do not return this handoff as `fixed` or reuse historical screenshot/RenderDoc evidence before that validation succeeds.

### 2026-09-19 successor validation dependency receipt

Successor Session `failure-roll-01a084c8-shader06-realtime-sh9-r2` sealed the
seven current source paths and submitted static ticket
`089af3ecb3ee497cb57b21617fcb9a17` (request
`failure-roll-01a084c8-shader06-realtime-sh9-20260919-r2`). Admission returned a
durable `validation_dependency_failed` blocker for the lower Render11 lifecycle
`environment-cubemap-rebind-last-good-transaction`; no checker process or source
assertion ran. The ticket remains queued, and the dependency must be resolved by
its owning Render11 Session before this request can execute. This is not a
static pass, Cargo pass, product acceptance, or closeout.
