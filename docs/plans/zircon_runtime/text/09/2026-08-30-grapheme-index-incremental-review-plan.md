# Runtime Text grapheme-index incremental review plan

## Finding

`TextDocument` already reparses only the affected hard-line envelope, but
`TextDocumentSourceIndex` is invalidated on every changed revision and the next query rebuilds
grapheme boundaries from a complete flattened snapshot. This leaves the document edit path and its
navigation/index path with different work scales.

## Optimization hypothesis

For a local edit whose old range is already aligned to cached grapheme boundaries and whose local
source/replacement bytes contain only ASCII other than CR/LF, grapheme segmentation is byte-wise.
The index can therefore replace only the affected boundary interval and shift the suffix by the byte
delta. The splice itself is `O(old_range_bytes + replacement_bytes + suffix_boundary_count)` and
avoids rebuilding a full document snapshot. Piece-backed source extraction may still scan retained
pieces to recover local context, so this slice does not claim end-to-end `O(local_edit)` document
mutation until that storage cost is measured separately. It does not add a second line authority or
change the Unicode segmentation owner.

## Safety boundary

The incremental path is deliberately rejected when the index is stale, the edit endpoints are not
cached grapheme boundaries, the local context contains non-ASCII/CR/LF bytes, or arithmetic cannot be
checked. Those edits retain the existing invalidation and complete rebuild path. This conservative
first slice avoids guessing about combining marks, emoji ZWJ sequences, regional indicators, and
Unicode boundary state. CR/LF is excluded because UAX #29 treats the pair as one grapheme cluster.

## Measurement before expansion

The existing fixed profile names will gain low-cardinality incremental-update count/byte counters.
The managed matrix must compare beginning/middle/end edits over 1/100/1k/10k-line documents and
report index rebuild count, incremental bytes, query p50/p95/p99, allocation/RSS, and power. The
Unicode/combining/emoji/CRLF corpus must prove exact boundaries against the complete rebuild before
the safety boundary is widened. No performance gain or cross-engine claim is made by this source
review.

## Implementation Follow-up (2026-08-30)

The source index now carries a prepared, revision-qualified edit receipt. When the cached index,
edit endpoints, and local context satisfy the ASCII/no-CRLF safety boundary, commit splices the
affected byte boundaries and shifts the retained suffix. ASCII/no-CRLF admission walks existing
piece bytes directly without materializing a temporary context `String`; incomplete piece coverage
or checked source bounds reject the receipt. Any failed precondition leaves the index invalid so the
existing complete snapshot rebuild remains the only recovery path. Fixed profile
counters record successful incremental update count, input bytes, boundary count, and duration.

Rust regressions cover an ASCII replacement with suffix shifting, empty-document insertion,
deletion, and Unicode/CRLF-context edits that deliberately fall back to rebuild. The implementation
is intentionally narrower than a general Unicode incremental segmenter; no claim is made for
combining marks, emoji, regional indicators, ZWJ sequences, or CRLF edits.

The repository static contract `test_runtime_text_document_incremental_index_contract.py` passes
4/4 and locks the no-flatten/no-temporary-context preflight, checked piece coverage, fixed profile
names, and explicit Unicode/CRLF fallback regressions.

Status: `architecture_review_complete / conservative_ascii_incremental_path_static_implemented /
ascii_preflight_allocation_free_static_implemented / piece_coverage_fail_closed /
unicode_context_fallback_preserved / managed_profile_pending`.
