# Runtime Text stable hard-line model review and implementation

Date: 2026-08-27

Status: `separator_aware_stable_line_owner_implemented / edit_local_reanalysis_implemented /
full_document_hard_line_rebuild_removed_from_edit / grapheme_index_full_rebuild_open /
product_session_unwired / static_checks_complete / managed_validation_pending`

## Problem

`TextDocument` already retained immutable source chunks and revision-qualified edit deltas, but its
only hard-line index was rebuilt from `snapshot()` after every edit. It also exposed only absolute
`HardLine` ranges, so no retained line identity could survive a local edit. That prevented a later
layout session from distinguishing a changed paragraph from unchanged source lines.

The rejected shortcut was to snapshot the complete old document, edit it, snapshot the complete new
document, and diff both line vectors. That is `O(document bytes)` work for a one-line edit and does
not establish an incremental owner.

## Reference boundary

Unreal Slate retains `FLineModel` independently from lazily generated `FLineView`. A line model owns
text, shaped cache, break candidates, estimated geometry and independent wrapping/direction/shaping/
view dirty flags. This implementation adopts only the prerequisite ownership rule: source hard-line
identity is retained before wrapped visual-line or layout-cache lifetime is introduced.

## Implementation

- `TextDocumentHardLineModel` retains a stable ID plus content and separator byte lengths. It does
  not store absolute suffix ranges that must all be rewritten when a preceding edit changes length.
- The owner recognizes the same canonical separators as `text/hard_line.rs`, including CRLF as one
  separator and VT/FF/NEL/LS/PS.
- Before piece mutation, `replace` snapshots only an edit-local line envelope with one context line
  on each side, applies the replacement to that local string, parses the new envelope and prepares a
  complete line-model splice. Any invariant failure is returned before source, revision or pieces
  change.
- Unchanged prefix/suffix line models retain IDs. A changed or merged line retains the left affected
  ID; additional split lines receive revision-qualified creation identities. The edit receipt
  publishes old/new reanalyzed line ordinal ranges.
- `TextDocumentSourceIndex` is no longer a second hard-line authority. It remains revision-bound and
  currently owns only grapheme boundaries.

## Complexity and open work

Hard-line parsing and source materialization are now proportional to the affected line envelope, not
the entire document. A very large edited unbroken line remains proportional to that line, which is
required before reshaping it. The model currently uses `Vec` order, so inserting/removing line models
may move suffix metadata; this is not claimed as final `O(log lines)` sequence storage. More
importantly, grapheme indexing still rebuilds from a complete snapshot and no UI/service/
`DocumentLayoutSession` consumes the line IDs yet.

Dynamic Cargo, edit-scale timing/allocation/RSS/power, product WGPU and PNG validation remain pending.
The required performance matrix is 1/100/1k/10k hard lines with beginning/middle/end edits and
separator split/merge cases before choosing a rope/tree sequence or changing cache/reflow policy.

## Static evidence

- Rust 2024 formatting passes for all six `text/document` files.
- New source owner is 281 lines; the document test owner is 290 lines.
- Regressions cover CRLF and Unicode separator lengths, line-local ID retention, split ID creation,
  merge-left identity, CRLF completion, empty document insertion and trailing terminal-line creation.
- No full old/new document snapshot was added to the edit path.
