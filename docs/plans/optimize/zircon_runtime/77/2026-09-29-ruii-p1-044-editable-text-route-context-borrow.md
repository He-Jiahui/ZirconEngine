---
title: Runtime77 RUII-P1-044 Editable Text Route Context Borrow V3
category: zircon_runtime
report_id: Runtime77-RUII-P1-044-2026-09-29-v3
date: 2026-09-29
implementation_status: applied_source_candidate
validation_status: managed_runtime_tests_pending
format_status: route_policy_and_tests_clean_editable_text_has_preexisting_format_diff
performance_status: managed_release_comparison_pending
coordination_status: managed_validation_pending
review_status: v3_delta_independent_static_review_complete
---

# Runtime77 / RUII-P1-044 editable text route context borrow, v3

## Scope

The captured working-tree preimage still clones UiInputDispatchResult.event in
with_editable_text_route_policy. UiInputEvent::Text owns its payload as a
String, so this annotation-only clone duplicates payload bytes. This candidate
removes that clone at the editable-text result wrapper. It does not claim the
broader cross-surface carrier, arena, or bounded-publication work in
RUII-P1-044 is complete.

## Candidate change

A result-based annotator borrows result.event and result.applied_effects while
mutating only result.diagnostics. The existing event-argument annotator remains
for the other input owners. Both use the same route-policy, note, and generic
trace helpers. The editable-text wrapper calls the borrowed entry and keeps
route-step annotation afterward.

The text-result regression compares the legacy clone-and-annotate path with the
borrowed path for a large payload. It verifies event, route policy, retained
notes, route target, bubble path, full route trace, route steps, and payload
retention. A separate touch-pointer parity regression checks pointer-source
notes and route trace.

The ignored Release benchmark compares the old event clone plus annotation with
the borrowed-result annotation for a 16 KiB payload. It alternates 101 paired
samples after three warmup batches and reports nearest-rank p95 (rank 96 of
101), all raw sample values, and a per-batch event-clone payload-byte lower
bound. Its output labels explicitly identify legacy event-clone bytes and
borrowed event-clone bytes; zero refers only to event-clone payload bytes in
the measured batch, not to total allocations. It has no latency assertion and
is not a performance pass. It reuses the preexisting route-policy test module
timing helpers.

## V3 independent static review and limits

The v2 independent static review passed. The v3 delta was independently
reviewed against the existing contracts: UiDispatchReplyStepTrace derives
PartialEq and Eq in
zircon_runtime_interface/src/ui/dispatch/input/reply.rs, and
UiInputDispatchDiagnostics.route_steps is a Vec of that type in
zircon_runtime_interface/src/ui/dispatch/input/result.rs. The regression
compares route_steps after both annotation paths have completed. The benchmark
labels now say legacy_event_clone_bytes_lower_bound_per_batch and
borrowed_event_clone_bytes_per_batch; the zero is explicitly limited to
event-clone payload bytes and does not mean all allocations are zero.

The v3 source delta adds only route_steps equality to the parity regression and
clarifies the two event-clone byte labels. The existing v2 helper, imports,
route-preview behavior, and Runtime200 regression/benchmark are retained.
Static review is not Rust type checking. An actual rustfmt check with edition
2024, skip_children=true, and reorder_imports=false passes route_policy.rs and
route_policy_tests.rs. editable_text.rs reports only an existing import-wrap
difference; the same single difference occurs in the exact captured preimage.
No Cargo test or Release benchmark ran. Functional tests are authored but
unexecuted. Managed runtime tests, the 101-pair Release comparison, and product
input-to-frame latency/allocation evidence remain pending. This source candidate was applied; managed validation remains pending.

## Source fingerprints

| File | Captured working-tree preimage SHA-256 | Scratch candidate SHA-256 |
| --- | --- | --- |
| zircon_runtime/src/ui/surface/input/route_policy.rs | 1533D38FE2E7E4C7FE4E18255D20BD8EBCAAC315AD04010EF72417CB7FDAA22F | 7E9DE2C17C24AFCB1AD6E4F67965909E8B52DF743D7EAB0EB3CDB799BB51F881 |
| zircon_runtime/src/ui/surface/input/editable_text.rs | D5EBA3988A8E7D88AE0571DF4F4C64AF2783ED0D777B4D7DD52ECEEC193301B1 | BC4349E7297C9D3AB7B008C7DCEDC898A433533CA6161E590DB2534606A1EE98 |
| zircon_runtime/src/ui/surface/input/route_policy_tests.rs | 4650255E9CED7F61372B346D375AB107F179AC7431F2AADA7861653C496EB172 | 118293ABE9240A3027D7F280FCCEA2B0767D4C7647A8D8F4E27C6587B08FBC0C |

## Pending acceptance

- Managed Windows unit tests for package zircon_runtime, library tests, filter
  route_policy_tests.
- Managed Windows Release ignored test
  runtime77_editable_text_route_context_borrow_release_benchmark with
  --nocapture. Retain all 101 legacy/borrowed samples and compare p95. The
  event-clone-byte lower bound is not allocation or product evidence.
- Product input-to-frame p50/p95/p99 and allocation comparison on an equivalent
  product workload. This remains open.
