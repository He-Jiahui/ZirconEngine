---
related_code:
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/tests/editor_message/bus/dirty_set.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/48/2026-09-28-invalidation-mask-unknown-bits.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
status: partial_implemented_pending_validation
---

# Editor1033 / Editor48 invalidation mask unknown-bit completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| E-MSG-P1-34 unknown-bit admission | Deserialization rejects any bit outside the nine defined invalidation flags while valid masks keep their numeric `u16` wire shape. | A focused regression covers valid round-trip and unknown-bit rejection. Pinned Rustfmt read-only and scoped diff checks passed; managed Cargo test execution is pending. | implemented_pending_validation |
| E-MSG-P1-34 versioned forward compatibility | No versioned wire codec or cross-process compatibility policy was introduced. | Decide version negotiation and unknown-bit handling for future external message schemas before closing the full item. | product_gate_pending |
| Editor48 acceptance | This slice changes only mask input validation. | Grouped managed regression and broader Editor48 delivery/performance evidence remain pending. | validation_pending |

No coordinator ticket or dynamic test result is claimed by this completion list.
