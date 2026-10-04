---
related_code:
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/identity.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/app/reference_drop_payload.rs
  - zircon_editor/src/core/document/lifecycle.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/hierarchy_identity.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-27-hierarchy-drag-document-identity.md
  - docs/plans/optimize/zircon_editor/60/2026-09-27-hierarchy-reparent-drag-threshold.md
status: implemented_pending_validation
---

# Editor1022 / Editor60 hierarchy drag document identity completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| ED60-G04 same-count World replacement | Press token records the complete Edit gateway identity. Up and SceneInstance drop reject an older generation before consuming equal numeric node IDs. Typed stale-source rejection also prevents Asset, Instance, and Object `FieldDropped` actions from synthesizing static demo fallback references; a genuinely absent payload still uses the existing fallback. Retained refresh clears stale state. | New real-host tests stage a same-count/same-ID gateway reload, including Up before refresh and all three field actions. A separate test covers the genuine no-payload fallback. Managed execution remains pending. Same-gateway Play replacement before invalidation pump remains open. | implemented_pending_validation |
| ED60-G05 document and window authority | Token records active history context, document activation revision, World domain, and callback source window; source replacement clears token and payload together. | Host tests cover document binding switch, callback-window mismatch, unchanged World, and competing Inspector source. Full native multi-window lifecycle evidence remains pending. | implemented_pending_validation |
| ED60-G06 generation-qualified terminal check | Reparent checks at admission and immediately before dispatch; SceneInstance reference drop checks before taking payload. Lifecycle authority provides a crate-local scalar revision read under existing locks. | Source review, scoped formatting, exact inverse, and final manifest are under the R Editor1022 artifact prefix. Managed tests are pending. | implemented_pending_validation |
| ED60-G01/threshold and Undo successor | Editor1021's 4px activation, terminal-repeat and Undo semantics are retained. | New unchanged-World native test adds a success/Undo guard; frozen Editor1021 behavior tests remain. | implemented_pending_validation |
| ED60-G02/G03/G30/G34/G35/G36/G38/G39/G40 | No new performance threshold or platform acceptance is inferred. | Native capture/cancellation; bounded Reparent; 100K/1M scale, 1% churn, multi-window isolation, latency/native feedback, cross-engine benchmark remain pending. | product_gate_pending |

This record succeeds Editor1021's frozen `events/drag.rs` SHA `64f9806bf96408209799df19c7915283c5aeae734c8b327c6c2f19d0cdfb9c92` and `drag_sources/mod.rs` SHA `8b4dab32214f2cbb1741b3d5232428c06ce263b72e9327d1cf44ae5735a1586b`. R Editor1021 evidence is preserved unchanged. No Cargo result or performance pass is claimed here.
