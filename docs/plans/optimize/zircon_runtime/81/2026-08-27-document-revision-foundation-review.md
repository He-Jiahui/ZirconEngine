---
title: Runtime Text Document Revision Foundation Review
category: zircon_runtime
report_id: Runtime81-document-revision-foundation-review-2026-08-27
date: 2026-08-27
session_id: root-runtime81-document-revision-foundation-20260827
implementation_status: internal_foundation_complete
validation_status: static_validation_complete_managed_pending
product_integration_status: open
---

# Runtime Text Document Revision Foundation Review

## Current-Source Finding

The plan statement that Runtime Text had no document edit delta was stale. The crate-private
`text/document` owner already contains immutable original/addition chunks, a piece list, owner plus
revision identity, exact old/new byte dirty spans, length delta, explicit snapshots, and a
revision-bound hard-line/grapheme source index. It reuses unchanged chunks for replacement edits.

This is an internal foundation, not product document authority. Runtime UI, the component reducer,
IME, accessibility, render extraction, and `DocumentLayoutSession` do not consume it. Its source
index also rebuilds from a flattened whole-document snapshot after invalidation. Paragraph dirty
projection, stable hard-line identity, retained snapshot leases, incremental index repair, visual
line reuse, and reflow receipts remain open.

## Reference Review

Local Unreal Slate keeps source line models separate from visual line views. `FLineModel` owns text,
runs, break candidates, shaped cache, estimated geometry, and independent dirty flags for wrapping,
base direction, shaping, and views. `ModelChangeCounter` advances when model content changes, while
line views may be generated lazily per model. This supports a retained model/session boundary; it
does not support rebuilding a flat source twice inside every edit merely to calculate dirty lines.

## Correctness Hard Cut

The internal replace gateway now requires the expected `TextDocumentKey`. A stale key returns a
typed error before range validation or mutation, so an old keyboard, IME, paste, or model intent
cannot silently overwrite newer content when the owner is later wired. Revision advance uses
`checked_add`; exhaustion returns a typed error before any piece, source index, length, or source
content changes. The former saturating increment could mutate content while reusing the same cache
and layout identity at `u64::MAX`.

The document authority no longer implements `Clone` or value equality. Cloning the owner and
revision would allow two mutable branches to advance independently and publish different source
under the same key; comparing an authority by value would also include incidental source-index cache
state. Clone/equality remain available only on internal immutable pieces and index values that need
them.

The edit-delta regression also had an incorrect byte expectation: replacing six bytes with
`"second\nfourth"` inserts thirteen bytes, so the new dirty range ends at byte 19 and length grows by
seven. The test now reflects the actual UTF-8 byte contract.

## Algorithm Decision

No paragraph-range algorithm was added in this slice. Calling `source_index()` before and after an
edit would flatten and scan the old and new complete document, producing `O(N)` source copies and
analyses per edit while claiming incremental behavior. The next structural implementation must
first retain stable hard-line models and update only a separator-aware neighborhood, including CRLF
and Unicode hard separators. Dynamic profiling must then compare 1/100/1k/10k-line edit and scroll
work before selecting an index tree or reflow policy.

## Evidence And Open Work

Rust 2024 formatting, scoped diff checks, stale-entry source scans, and the document owner file
budget pass. The expected-revision, revision-exhaustion, byte-delta, UTF-8 rejection, chunk reuse,
revision-index, and range-snapshot regressions are authored. Managed Cargo did not complete in the
available validation lane, so no runtime, timing, allocation, RSS, power, WGPU, or PNG claim is
made.

Status: `revisioned_piece_storage_present / expected_revision_gateway_implemented /
revision_exhaustion_fail_closed / cloneable_authority_removed / edit_delta_regression_corrected / product_authority_unwired /
paragraph_dirty_stable_line_reflow_open / static_checks_complete / managed_validation_pending`.
