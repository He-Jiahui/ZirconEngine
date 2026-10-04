---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/15/2026-08-26-segmented-controls-owner-split.md
  - docs/plans/optimize/zircon_runtime/15/2026-08-26-selection-controls-owner-split.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_runtime/src/ui/surface/render/dropdowns.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/tabs.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/checkbox.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/radio.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/toggle.rs
tests:
  - zircon_runtime/src/ui/tests/render_dropdowns.rs
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs
  - zircon_runtime/src/ui/tests/render_selection_controls.rs
---

# Runtime Fixed Control Command Capacity

## Scope

Five Runtime UI leaf painters emitted small, fixed-size command lists from empty
vectors. Their branch bounds are independent of authored collection size, so the
builders now reserve exactly the maximum number of output commands after the
existing classification and frame validation work:

| Painter | Maximum commands | Bound |
| --- | ---: | --- |
| Dropdown | 5 | surface, optional label, optional value, caret, optional open mark |
| Tab | 3 | optional background, optional selected underline, optional label |
| Checkbox | 5 | mark, three active tick strokes, optional label |
| Radio | 3 | mark, optional active dot, optional label |
| Toggle | 3 | optional label, track, thumb |

The Checkbox tick helper now appends its three fixed strokes directly to the
caller-owned command vector. This removes the checked-path temporary vector in
addition to preventing growth of the final command list. Painter order, z
offsets, clipping, style selection, and invalid-input early exits are unchanged.

This deliberately excludes SegmentedControl and Slider command vectors: their
option/tick counts are authored and dynamic, so eager upper-bound reservation
would require a separate bounded-input/admission decision rather than reusing a
small fixed-capacity optimization.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11C / RUI-670 | Reserve fixed leaf painter command buffers and stream Checkbox tick strokes into the retained output vector | implemented_pending_validation | Source regressions bind each capacity constant and the caller-owned Checkbox tick path. Scoped Rustfmt, scoped `git diff --check`, the five-painter capacity/branch probe, and the 82-test Runtime/Editor static-contract batch pass. Managed Runtime Cargo plus Windows Release allocation and p50/p95/p99 evidence remain pending; this record makes no product performance claim. |

## Static Evidence

- `render_dropdowns.rs`, `render_segmented_controls.rs`, and
  `render_selection_controls.rs` now prevent removal of the five fixed capacity
  reservations and the allocation-free Checkbox tick append path.
- Scoped `rustfmt --edition 2021 --check --config skip_children=true` passed for
  all five production leaves and the three focused test modules.
- Scoped `git diff --check` passed. Git reported only repository line-ending
  notices.
- The source probe found all five reservations and confirmed the expected
  Dropdown/Tab push-site counts plus Checkbox/Radio/Toggle branch structure.
- The combined Runtime/Editor static-contract batch passed `82/82` in `2.769s`.
  It covers navigation, incremental surface rebuild, asset-browser projection,
  console bounded materialization, and watcher invalidation contracts.

## Source Snapshot

- `dropdowns.rs`: `E0B5CEDEBA72AD12E0D3F3DAA66DC250A7AC56CB84EE50D8A6255A9A1FB867D1`
- `segmented_controls/tabs.rs`: `C0A52FDB7580BA1A901FBF7079D78EACD8C37699DE0A1F9967DFB75A60B63EBB`
- `selection_controls/checkbox.rs`: `DE7987C1241215E722142EAA36DC9CBC1B05ABC8B867FC50C8941E9ECD990867`
- `selection_controls/radio.rs`: `B7DEF89560C4B8C40BEB74CC4A9EB479BBFB1E405070AFFCD06B0C2DC368191B`
- `selection_controls/toggle.rs`: `6B120E95D1D8F31300956CC8F53EDD1EE20F4012D8DE00EE1C2EADEFC0DC8C0D`
- `render_dropdowns.rs`: `C4DF8DA17BD1AECF2B69835717E830BB3A2B0CAECF966356AD27CBE31C49A151`
- `render_segmented_controls.rs`: `30F618CE6A1C0ECB745F9B9C277BC962A3C3CA59D650CCD89DFEBF8DB681B15E`
- `render_selection_controls.rs`: `A8DD6D39911791A0C60B05F7BC61651197293CB27E61D8BA7392CF50536D3B07`

## Managed Gate

No direct Cargo command or coordinator status query was run for this slice.
The next immutable multi-task Runtime/Editor validation input must compile the
three focused test modules and collect a release allocation/time comparison.
Until that batched result exists, the status remains
`implemented_pending_validation`.
