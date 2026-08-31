# Runtime Text document no-op edit outcome

Date: 2026-08-27

Status: `typed_noop_edit_outcome_implemented /
allocation_free_piece_range_equality_implemented /
revision_index_history_churn_suppressed / product_gateway_unwired /
static_checks_complete / managed_validation_pending`

## Finding

The crate-private retained document treated an exact byte-for-byte replacement as a changed edit.
It advanced the document revision, appended another immutable addition chunk, reanalyzed hard lines,
invalidated the grapheme index and returned a changed receipt even though the source was identical.
A later history owner could not distinguish that request from a real mutation without comparing the
document again.

The local Unreal reference performs a case-sensitive equality check in
`FSlateEditableTextLayout::SetEditableText` before replacing line models. Its edit transaction also
publishes undo state and change notification only when the edited text differs from the saved state.
Zircon therefore needs an explicit no-op outcome at the document gateway rather than downstream
heuristics over a dirty receipt.

## Implementation

- `TextDocument::replace` now returns `TextDocumentEditOutcome::{Unchanged, Changed}`. A stale key and
  invalid range are still rejected first; exact equality is resolved before revision advance or any
  piece, hard-line, source-index or cache-identity mutation.
- `TextDocument::range_equals` compares the requested source range across original and addition
  pieces as byte slices. It does not flatten the range, allocate a `String`, or scan unrelated
  document bytes. Checked offsets and a typed `StorageInvariant` reject incomplete/corrupt coverage.
- An unchanged edit remains valid when the current revision is `u64::MAX`, because it does not need
  another publishable revision. A source-changing edit at that revision keeps the existing typed
  `RevisionExhausted` failure.

The comparison is `O(pieces touched + compared bytes)` and allocation-free. A real equal-length edit
that differs late in the range may pay an additional prefix comparison before the existing local
edit preparation. Profile that cost before adding caller hints or changing the piece sequence; this
slice makes no latency, power or optimality claim.

## Evidence and open work

- The cross-piece regression first creates `original/addition/original` storage, builds the grapheme
  index, then requests an identical whole-source replacement. It locks unchanged key, pieces,
  addition count, hard-line IDs and revision-bound grapheme boundaries.
- A second regression locks a typed unchanged outcome at revision `u64::MAX`.
- Existing changed-edit tests explicitly unwrap `Changed(receipt)`, so dirty ranges, line spans and
  length deltas cannot silently accept the no-op branch.
- Rust 2024 formatting passes for the 126-line edit owner, 297-line storage owner and 385-line test
  owner. Source scans find no consumer outside this crate-private document foundation.

Managed Cargo, fault injection, edit-scale allocation/timing/RSS/power, product document service,
history grouping, external-model rebase, WGPU and PNG remain pending. No product Runtime82 gate is
closed by this internal foundation.
