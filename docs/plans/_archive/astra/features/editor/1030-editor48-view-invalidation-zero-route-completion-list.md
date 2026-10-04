---
related_code:
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/tests/editor_message/refresh.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/48/2026-09-28-view-invalidation-zero-route.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
status: implemented_pending_validation
---

# Editor1030 / Editor48 view invalidation zero-route completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| E-MSG-P0-01 authoritative view invalidation | The editor host now writes the dirty view directly to the bus-owned projection state. Refresh correctness no longer depends on a `view.invalidated` subscriber. Presentation refresh uses full reflection; structure-only refresh uses scene-inspection delivery. | The real host regression publishes a dirty-bearing `view.invalidated` probe, observes zero deliveries and no dirty projection, then verifies `refresh_view` returns the requested mask and materializes the snapshot. The structure-only scene-delta regression remains included. Scoped rustfmt and direct source/doc byte checks passed; managed Cargo compile and execution remain pending. | implemented_pending_validation |
| Editor48 integration and performance | This slice closes the host-side zero-route invalidation implementation only. | Message lifecycle/admission work, broader integration, OS behavior where applicable, allocation/latency/RSS evidence, and all managed runtime gates remain pending. No performance or dynamic pass is claimed. | product_gate_pending |

Local hashes for the four candidate files and scoped static-check results have been collected for
batch preparation; coordinator manifest/receipt recording remains pending. The frozen 480-path v2
snapshot is unchanged, and managed validation remains pending.
