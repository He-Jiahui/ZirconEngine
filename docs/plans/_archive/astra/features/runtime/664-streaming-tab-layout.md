---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/81/2026-08-27-streaming-tab-layout.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
related_code:
  - zircon_runtime/src/text/layout/tab.rs
  - zircon_runtime/src/text/layout/measure.rs
tests:
  - zircon_runtime/src/text/layout/tab.rs
  - tools/tests/test_runtime81_streaming_tab_layout_performance_contract.py
  - tools/tests/test_runtime_text_grapheme_projection_cursor_contract.py
---

# Runtime81 Streaming Tab Layout

Tab alignment now validates grapheme cardinality in a bounded pass, then
streams the final advance projection. The scalar width path reuses the same
tab-stop arithmetic without materializing an output vector, while the advance
path allocates only its returned vector. Unicode grapheme segmentation,
negative-advance sanitization on the active tab path, mismatch fallbacks, and
finite geometry handling are unchanged.

The final physical-line measurement owner also records exact tab presence while
it builds the grapheme ranges that projection already requires. Its private
width route consumes that marker, so ordinary no-tab lines return the shaped
width without a second `text.contains('\t')` scan. For a present tab, that owner
also already proves grapheme cardinality, so it streams the tabbed width once
without repeating the public validation pass. The geometry-only entry
instantiates the same projection with tab tracking disabled, so callers that do
not consume width avoid the marker work entirely. The public `MeasuredTextLine`
contract, rich/caller-owned projection API, tab-stop arithmetic, and mismatch
fallback remain unchanged.

The public scalar width helper now fuses cardinality/tab admission with raw and
tabbed finite accumulation in one grapheme stream. A valid no-tab sequence
returns the raw accumulator, while an early/late cardinality mismatch retains
the existing `finite_sum` fallback. This removes the second segmentation pass
for direct valid tab width callers without weakening their public contract.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime81 | Remove temporary grapheme/output vectors from tab width and advance projection; reuse measurement's grapheme pass for no-tab admission | implemented_pending_validation | Tab and text-layout source-contract batch `98/98`, merged Runtime/Editor source batch `125/125`, and the formatting-tolerant tab-presence guard are green; scoped Rustfmt and diff checks pass. The ignored Runtime81 release gate now records the old two-vector allocation model plus 31-sample p50/p95 parity. Managed Runtime Cargo and release p50/p95/p99 remain pending. |

## Coordinator dispatch

The concurrent Runtime/Editor batch `astra-runtime-editor-compile-batch-20260911-r2`
was recorded in `.codex/tmp/astra-runtime-editor-batch-20260911-r2.json`.
Admission rejected all three requests with `session_not_found` before ticket
creation, so no Cargo or performance result is inferred from that response.

A follow-up Runtime81 registration request
`c6d0d3f450ae4a1688c699f858e61a57` was accepted into coordinator admission for
the current Codex thread. At the recorded observation point it had not created
a validation ticket because the maintenance mutation gate was occupied. This
record does not infer Cargo, release timing, allocation, or power evidence, and
the request was not polled after submission.

The later five-request current-source batch
`astra-runtime-editor-current-source-20260911-r3` was also logged without
polling. All entries were rejected at admission with
`validation_copy_overlay_not_owned` because the changed paths lacked current
Session attribution; no validation copy, Cargo run, ticket, or performance
sample was created.

## Acceptance boundary

The deterministic allocation model in the source plan predicts one advance
allocation and zero width allocations, but no product CPU, RSS, WGPU, or power
claim is made here. The managed batch must run the focused Rust behavior tests
and `optimization_batch_20260827bl_streaming_tab_layout_p95` with exact result
parity.
