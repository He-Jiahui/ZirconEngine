status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime_interface/01/2026-08-17-host-output-policy-convergence.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md

# Runtime Interface UI exports

## Current source and repair

Editor consumers were importing UI contract types from the public `surface`
and `dispatch` roots, while those roots had stopped re-exporting types still
owned by their child modules. The roots now expose
`UiRenderFrameCommandRef`, `UiTextShapeArtifact`, `UiInputDiagnosticsMode`,
`UiInputDiagnosticsTruncationReceipt`, and `UiPointerRoutingReceipt` from their
existing owners. No ABI layout or runtime behavior changes.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M35 | Restore missing Runtime Interface UI contract re-exports | implemented_pending_validation | Scoped source diff check passed; combined Runtime/Editor batch pending |
| M35a | Add a public-root regression contract for input diagnostics and pointer-routing receipts | implemented_pending_validation | `test_runtime_ui_input_routing_receipt_contract`: 9/9 passed; it is included in the combined Runtime02/08/19 + Editor09 source batch (`39/39`); scoped `rustfmt --check` and `git diff --check` passed; the next managed interface batch must include the new Rust contract |

This is a public contract wiring repair and carries no independent performance
claim.
