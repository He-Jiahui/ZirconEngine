---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-virtual-list-reconciliation-scratch.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/virtual_list_materialization.rs
  - zircon_runtime/src/ui/layout/virtualization/materialization.rs
tests:
  - zircon_runtime/src/ui/surface/virtual_list_materialization.rs
  - zircon_runtime/src/ui/layout/virtualization/materialization.rs
  - tools/tests/test_runtime_ui_virtual_list_surface_materialization_performance_contract.py
  - tools/tests/test_runtime_ui_virtual_list_slot_materialization_performance_contract.py
---

# Runtime virtual-list reconciliation scratch reuse

`UiVirtualListMaterializationIndex::reconcile` now keeps transactional candidate slot/key/
generation buffers per owner. Warm scroll requests copy into the retained buffers with
`clone_from` and swap them into publication only after protected-slot validation succeeds. This
removes repeated bounded-vector allocations without changing logical item identity, slot
generation, protected capture rejection, or layout projection behavior. The slot map now
overrides `Clone::clone_from` to retain its internal slot vector, and Surface clones retain only
published assignment state while resetting candidate scratch.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A P1-17 | Reuse bounded virtual-list reconciliation scratch while preserving atomic candidate publication. | RED/GREEN source contract rejects the old per-request slot-map clone; lower Rust warm-capacity/pointer, `UiVirtualListSlotMap::clone_from`, and protected-rebind regressions are present; the latest batched Runtime/Editor UI invocation loaded 172 modules and passed `773/773` in 1.514s; managed Release allocation/latency evidence remains pending. | implemented_pending_validation |

## Acceptance boundary

Keep this record pending until the owner-attributed Windows Release batch verifies compilation,
allocation behavior, and virtual-list/navigation p50/p95/p99. Tooling and the broader real
data-source virtualization milestone remain out of scope; no coordinator status was queried.

Current source hashes:

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/layout/virtualization/materialization.rs` | `2924EF6A5CB646E7CBF54A90284C5ECACB5AAABEBE4FAFC8DB5567BF04D3162C` |
| `zircon_runtime/src/ui/surface/virtual_list_materialization.rs` | `DB856DFE940312EAE18ACFDACAD9F0373250437DFB275FC1129DB1AE5F46734D` |
| `tools/tests/test_runtime_ui_virtual_list_surface_materialization_performance_contract.py` | `5EE50E3F07F8162D0FF348260C4B2166CA23D576A79EF5CEDC2B748232251B67` |
| `tools/tests/test_runtime_ui_virtual_list_slot_materialization_performance_contract.py` | `B5AE0A7071EEE61A2E9D6F56DFCA90DA5DD1E8AFACFAA9A99CA6994DC3EC65FC` |
