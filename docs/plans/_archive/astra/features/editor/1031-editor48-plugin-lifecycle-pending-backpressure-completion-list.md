---
related_code:
  - zircon_editor/src/core/plugin/lifecycle_message_bridge.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/48/2026-09-28-plugin-lifecycle-pending-backpressure.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
status: implemented_pending_validation
---

# Editor1031 / Editor48 plugin lifecycle pending backpressure completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| E-MSG-P1-36 bounded shadow pending | When retained deliveries exist, the bridge retries them before draining newer deliveries. The bus retains its own bounded backlog and continues reporting backpressure at its configured limit. | A real bus/bridge/plugin-manager regression covers a concurrent mutation deferral, FIFO retry, one-pump delay for newer deliveries, one-message inbox backpressure, and admission after space is freed. Cargo compilation and execution remain pending. | implemented_pending_validation |
| E-MSG-P1-36 shared acknowledgement window | The private retry queue remains separate from bus count/byte/age reservation; one detached batch and the bounded bus inbox may coexist. | Shared reservation, acknowledgement, retained-byte/age accounting, and a terminal receipt remain open. This slice is partial and does not close E-MSG-P1-36. | product_gate_pending |
| Plugin callback failure lifecycle | Manager callback failures remain diagnostic report entries and are not retried by this bridge change. | Retry, quarantine, operator action, and fault-isolation semantics remain open under E-MSG-P1-37. | product_gate_pending |
| Performance and managed validation | No performance or runtime gate was executed here. | Run the focused test through the grouped managed Editor wave; add allocation/latency/RSS evidence before any performance claim. | validation_pending |

The source change and this record add no paths to the frozen 480-path v2 snapshot. No coordinator
receipt or managed validation pass is claimed.
