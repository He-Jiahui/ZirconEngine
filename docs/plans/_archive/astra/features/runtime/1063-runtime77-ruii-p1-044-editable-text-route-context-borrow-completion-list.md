---
doc_type: feature-completion
status: source_candidate_validation_pending
validation_status: managed_runtime_tests_pending
format_status: route_policy_and_tests_clean_editable_text_has_preexisting_format_diff
performance_status: managed_release_comparison_pending
independent_review_status: v3_delta_independent_static_review_complete
record_target: docs/plans/astra/features/runtime/1063-runtime77-ruii-p1-044-editable-text-route-context-borrow-completion-list.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/77/2026-09-29-ruii-p1-044-editable-text-route-context-borrow.md
implementation_files:
  - zircon_runtime/src/ui/surface/input/route_policy.rs
  - zircon_runtime/src/ui/surface/input/editable_text.rs
tests:
  - zircon_runtime/src/ui/surface/input/route_policy_tests.rs::runtime77_editable_text_route_context_borrow_preserves_annotations
  - zircon_runtime/src/ui/surface/input/route_policy_tests.rs::runtime77_borrowed_route_projection_preserves_pointer_notes_and_trace
candidate_id: runtime77-ruii-p1-044-editable-text-route-context-borrow-v3
apply_status: applied_source_candidate
---

# Runtime1063 / Runtime77 editable-text route-context borrow, v3

| Plan item | Candidate result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Remove the editable-text route-event deep clone | The result-based annotator borrows the event and applied effects while mutating diagnostics; the text wrapper uses it. | Candidate patch is based on three captured working-tree preimages. Exact hashes are in the Runtime77 record. Shared source was applied; managed validation remains pending. | applied; validation pending |
| Preserve policy, notes, trace, and route steps | The text regression compares the legacy and borrowed results for event, policy, notes, full route trace, route steps, and retained text. The touch-pointer regression checks Bubble policy, pointer-source notes, and bubble trace parity. | The route-step element type derives PartialEq, Eq. Tests are authored and unrun; managed Windows zircon_runtime library test filter route_policy_tests remains pending. | pending |
| Compare equivalent Release work | The ignored benchmark times legacy event-clone-plus-annotation against borrowed-result annotation, with a 16 KiB payload and 101 paired samples. | Managed Release run and raw samples are pending. Output labels identify legacy event-clone bytes and borrowed event-clone bytes; zero is only the borrowed event-clone payload-byte count, not all allocations. No local timing or speedup is claimed. | pending |
| RUII-P1-044 broad product objective | This candidate removes one text-result clone only. Cross-surface event carriers, bounded publication, and product performance objectives remain open. | Equivalent product input-to-frame latency and allocation evidence have not been collected. | open |

V3 independent static review covers only the route-step equality type and the
event-clone-specific benchmark labels. V2's broader static review remains the
basis for unchanged code. Shared source was applied. Managed
Cargo tests and the Release benchmark remain pending.
