---
status: in_progress
finding: RT213-P1-009/010
owner: astra-cpu-bounds-20260905
---

# CPU Visibility Bounds

RT213-P1-009/010 closes the CPU/GPU bounds divergence by consuming the prepared
mesh local bounds and transforming them with the complete affine matrix before
visibility, spatial-index, and history decisions. Missing bounds remain
conservatively visible; invalid prepared bounds carry their generation and use
the existing fail-open policy.

## Scope

- `zircon_runtime/src/graphics/visibility` CPU bounds and focused regressions.
- Same-generation handoff from resource preparation / RenderScene admission.
- No scene-light, shadow-extraction, UI, or Hub changes.

## Acceptance

- Off-center, rotated, negatively and non-uniformly scaled meshes have CPU
  visibility matching GPU local-bounds projection.
- Bounds-only resource/deformation revisions invalidate spatial-index/history
  entries without stale reuse.
- Direct extracts with unavailable bounds remain visible and report the reason.
- Parent-managed Windows validation covers focused tests and product parity;
  this plan contains no local Cargo evidence.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | RT213-P1-009/010 prepared local bounds 到 CPU visibility 的完整 affine 投影、fail-open 与 bounds revision/history 接线 | `implemented_pending_validation` | 2026-09-09 | `cpu_bounds_use_off_center_local_bounds_after_affine_transform`、`cpu_bounds_fail_open_when_prepared_bounds_are_missing_or_invalid`、`visibility_context_projects_prepared_local_bounds_into_cpu_visibility`、`visibility_context_keeps_mesh_visible_while_prepared_bounds_are_unavailable` 与 bounds-only revision 回归已在源码；生产 extract 已传递 `prepared_local_bounds`。Rustfmt/静态复核通过；受管 Windows Cargo、GPU/CPU 一致性和 p50/p95/p99 仍待验证，外部 `E:/Git/zr_vm` dirty 阻止本轮封存。 |
| M2 | VisibilityStaticIndex bounds/ray 查询候选归一化 | `implemented_pending_validation` | 2026-09-09 | `collect_query_keys` 以单个 `Vec<u64>` 收集 overflow/cell memberships 后排序去重，替代每次查询的临时 `BTreeSet` 节点与二次 Vec 收集；跨 cell 重复 key 回归与源码合同已补，rustfmt/diff-check 通过。受管 Runtime Cargo 与 release p50/p95/p99 仍待验证。 |
