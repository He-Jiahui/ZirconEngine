---
related_code:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_fragment.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/scene_node_identity.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-27-lossless-scene-node-control-identity.md
status: implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
---

# Editor1027 / Editor60 lossless scene node control identity completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| ED60-P1-02 / G07 full `u64` identity | Authored hierarchy controls store decimal `UiValue::String` for their exact `EntityId`; sparse patch admission parses the full `u64` and rejects mismatched properties. The saturating `i64` adapter is removed. | Real `SceneEntries` projection, lossless control map, sparse patch and wrong-ID rejection regression for `i64::MAX`, `i64::MAX + 1`, `u64::MAX`; managed Editor library execution pending. | implemented_pending_validation |
| ED60-G31/G34/G35/G38-G40 scale and product gates | The existing ten physical controls and changed-row-only fragment path remain. This slice adds decimal property storage and checked parsing; no performance gain is claimed. | 100K/1M hierarchy, p95/p99, RSS, allocations, actual Editor window and same-workload reference measurements remain pending. | product_gate_pending |

The five owned paths, exact preimages, source hashes and scoped static checks are tracked by the Editor1027 source manifest. This completion list records source implementation only; it does not claim a Cargo pass or Editor60 product acceptance.
