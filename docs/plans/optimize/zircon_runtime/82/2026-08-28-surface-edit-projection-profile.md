# Runtime82 Surface edit projection profile

Date: 2026-08-28

Status: `current_source_and_runtime11b_review_complete / unreal_owner_model_reviewed /
instrumentation_implemented_unvalidated / direct_current_source_5_of_5_passed /
baseline_profile_pending / document_handle_cutover_not_started`

## Problem

The retained `TextDocumentStore` removes repeated full-source copying inside document edits, but the
product editable route still materializes a complete `UiEditableTextState.text` from template
metadata and projects the complete next string back into Surface properties. A bound Change event
may clone the complete state text again. Therefore a local one-byte edit remains at least `O(N)` in
the Surface projection even when the retained document transaction itself is range-local.

This is the still-open Runtime11B P1-17 boundary: component metadata is simultaneously serialized
property storage, edit buffer, render source, and binding payload. Optimizing `String::replace_range`
or changing document piece storage cannot remove copies owned by those other authorities.

## Reference and current-source review

Unreal's editable layout keeps editable text/run state in `FSlateEditableTextLayout` and separates
it from `BoundText`; focused bound refresh does not replace the active buffer unless explicitly
forced. Zircon now has a manager-owned retained document UUID/revision, but Surface metadata still
stores and republishes the body String on each edit. The existing focused-bound-value architecture
review already requires a model/edit identity split and typed mutation origin before changing
refresh behavior.

Current local ownership points to measure independently:

- `editable_text_state_for_node`: complete body String materialization from metadata;
- `prepare_editable_text_properties_with_edit`: complete body clone into the proposed property;
- shared metadata/style/component/binding projection inside the fixed-ten-property commit;
- `push_text_component_event_report`: optional complete body clone for bound Change/Commit payload;
- document prepare/commit and history delta, which are already separately instrumented/profiled and
  must not be blamed for Surface-owned copies.

## Selected instrumentation

Add a folder-backed `editable_text/profile.rs` with fixed, low-cardinality counters and spans:

- state materialization count and source bytes;
- proposed property-value clone count/bytes and successfully admitted property projection
  count/source bytes, so rejected preflight work is not hidden;
- committed versus state-only projection count;
- composition-active projection count and visible preedit bytes;
- component payload count and payload bytes;
- stage spans for state materialization, property prepare, and property commit.

Counters report lengths only. They contain no node/property/source label and no text, so secure input
cannot leak values through profiling. Calls compile to no-op bodies without the profiling features.
Logical byte counters are not allocator evidence; the managed matrix must capture allocator/RSS data
beside these counters.

## Baseline matrix before optimization

Run actual `UiInputManager` edit routes for 1 KiB, 64 KiB, 1 MiB, and 10 MiB sources, with tail,
middle, selection replacement, IME preedit update/commit, undo/redo, and Change-bound/unbound cases.
For each lane record 31 cold/warm samples of p50/p95/p99 latency, allocation count/bytes, peak RSS,
document range bytes, the new Surface logical-copy counters, layout/shaping work, and power. Compare a
matched Unreal editable control under the same source sizes and event sequence; record hardware,
build profile, locale, and capture method.

Only after this matrix may implementation choose among:

- a session document handle plus compact presentation/interaction state in Surface;
- immutable revision leases for render/binding consumers;
- range-scoped binding events with explicit snapshot requests;
- a separate typed model value and edit buffer for focused bound text and NumberField.

The cutover must remove the old metadata-body edit authority rather than layering a third editable
state. It must preserve serialization, secure projection, binding semantics, accessibility, IME,
render invalidation, and content-free public receipts.

## Acceptance for this instrumentation slice

- profiler names are fixed and content-free;
- ordinary, state-only, composition, and bound-event routes can be distinguished by counters;
- no profiling feature means no counter storage, dynamic label allocation, or source formatting;
- scoped formatting/static checks pass;
- no claim that P1-17, allocation, latency, or power is fixed before the managed baseline and post
  matrix.

## 2026-08-28 implementation status

Completed in source:

- `editable_text/profile.rs` is the folder-backed owner for fixed counter names. It accepts only
  lengths and booleans; no node, property, source label, or text is accepted by its API;
- state materialization records the complete body length after editable classification succeeds;
- proposed property-value clone bytes are recorded before the existing `state.text.clone()`, while
  admitted projection bytes and committed/state-only/composition classification are recorded only
  after property preflight succeeds;
- the property prepare and commit phases use fixed profile spans. The commit span includes metadata,
  runtime-style, component-state, binding, dirty, and clipboard invalidation projection work;
- a bound Change/Commit records payload bytes immediately before the existing payload clone. An
  unbound edit records no component payload;
- all profiling APIs compile to no-op bodies when neither profiling feature is enabled. A direct
  E-drive executable that includes the current profile owner passes `5/5` calls; scoped Rustfmt,
  whitespace, fixed-name, and forbidden-label scans pass.

Still open:

- managed product allocation/RSS/latency/power baseline and matched Unreal capture;
- selection of the document-handle/model-edit hard cutover;
- post-change matrix and WGPU product acceptance. No rendering screenshot was produced by this
  instrumentation slice.
