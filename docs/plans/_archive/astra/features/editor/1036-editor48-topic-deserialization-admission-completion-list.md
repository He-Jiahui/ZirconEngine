---
related_code:
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/core/editor_message/topic/deserialize_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/48/2026-09-28-topic-deserialization-admission.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
status: partial_implemented_pending_validation
---

# Editor1036 / Editor48 topic deserialization admission completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| E-MSG-P1-26 topic serde admission | Public `EditorTopic` deserialization now uses the existing parser, preserving valid string JSON and rejecting invalid topic strings inside complete delivery JSON. The editor-message module guide states the wire admission contract. | Direct DTO and complete delivery regressions are written. Scoped static checks are recorded; managed Cargo execution remains pending. | implemented_pending_validation |
| E-MSG-P1-26 full identifier and schema admission | This slice only closes the topic serde bypass. | Length and budget limits, schema registry, namespace owner, capability, and other field validation remain open. | product_gate_pending |
| Editor48 product and performance acceptance | No live external JSON decode path or performance improvement was established. | Grouped managed validation and broader Editor48 product and workload evidence remain pending. | validation_pending |

The preexisting topic TODO was preserved byte for byte; its specific serde bypass is addressed
by the source change. This list does not claim a coordinator ticket or dynamic test pass.
