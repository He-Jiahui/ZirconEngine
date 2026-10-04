---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-topology-index-rebuild-completeness.md
implementation_files:
  - zircon_runtime/src/scene/world/hierarchy_topology.rs
tests:
  - zircon_runtime/src/scene/world/hierarchy_topology.rs::tests::missing_parent_projection_row_forces_source_rebuild
---

# Runtime1011 hierarchy index rebuild completeness

| Work | Implementation evidence | Remaining acceptance |
|---|---|---|
| Repair the contradiction between index currentness and rebuild admission. | `needs_source_rebuild` reuses the complete currentness predicate, including parent projection membership count. | Independent source review complete; combined managed Runtime/Editor library execution pending. |
| Preserve the existing regression and surrounding topology behavior. | A single production expression changes; the existing missing-parent-row regression and all other source bytes are preserved. Exact preimage and scoped static evidence are saved with the Q slice. | No dynamic result is claimed from source inspection or formatting. |
| Preserve bounded index admission cost. | Constant-time length checks; no parent walk, full-world snapshot or new allocation is introduced. | Runtime62 product/scale gates and measured performance remain open. |
