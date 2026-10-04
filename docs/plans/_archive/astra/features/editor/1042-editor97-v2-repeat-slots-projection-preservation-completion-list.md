---
related_code:
  - zircon_editor/src/ui/asset_editor/session/lifecycle/v2_projection.rs
  - docs/plans/optimize/zircon_editor/97/2026-09-28-v2-repeat-slots-projection-preservation.md
plan_sources:
  - docs/plans/optimize/zircon_editor/97-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-product-integration-current-source-review.md
status: implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
---

# Editor1042 / Editor97 V2 repeat and node slot preservation completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Preserve reachable V2 node `repeat` and node-level `slots` through unrelated Designer projection edits | Existing node IDs reuse their prior V2 values; newly authored nodes retain empty defaults. | A real V2 load, production projection/serialization, and reload regression was added. Managed Cargo execution and product Save/reopen remain pending. | implemented_pending_validation |
| Do not restore removed nodes | Rebuilding only visits the current Designer tree. | A deleted-node regression checks that prior node data does not reappear in serialized V2. Managed Cargo execution is pending. | implemented_pending_validation |
| Editor97 P0-01 and G01/G02 product acceptance | Still open. | ThemeTokens, unknown/trivia, unreachable nodes, V2 source-span fidelity, full corpus, fault behavior, and product/performance gates require separate work and evidence. | product_gate_pending |

This record describes a field-specific candidate. It does not claim that legacy projection is
lossless or that Editor97 P0-01 is complete.
