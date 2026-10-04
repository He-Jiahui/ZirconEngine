---
related_code:
  - zircon_editor/src/ui/layouts/views/asset_browser/name_lines.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/name_compaction.rs
implementation_files:
  - zircon_editor/src/ui/layouts/views/asset_browser/name_lines.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes.rs
plan_sources:
  - user: 2026-07-05 editor layout Unreal/Slate visual convergence and runtime text priority
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
tests:
  - zircon_editor/src/ui/layouts/views/asset_browser/name_lines.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes.rs
  - docs/tests/editor
doc_type: module-detail
---

# Asset Browser Runtime Name Lines

## Purpose

`name_lines.rs` owns the non-file display-name line splitting policy for Asset Browser titles. It exists so thumbnail tiles, selected summaries, and future compact title surfaces can share one runtime-measured line break algorithm without copying character-budget helpers into each projection file.

This module is intentionally separate from `name_compaction.rs`. File-like names such as `workbench_page_chrome.zui` remain on the extension-preserving compaction route because preserving the suffix is the main correctness rule. Non-file logical names such as `NavigationSettingsRuntimeProfile` use the two-line split route because the visual goal is a readable title rhythm inside Unreal Content Browser-style name areas.

## Behavior Model

Callers provide a `RuntimeNameLineSplit` with the available text slot width plus primary and continuation font sizes. `thumbnail_nodes.rs` passes the narrow 96 px thumbnail name slot. `summary_nodes.rs` passes the wider 220 px selected-summary slot. This keeps the policy shared while avoiding a hard-coded thumbnail width in summary UI.

The splitter trims the input and keeps it as one line only when both conditions are true:

- the name is at or below the short-name character threshold;
- `measure_runtime_text_width(...)` says the primary line fits the caller's width.

If the measured width does not fit, the module creates valid split candidates, including separators, camel-case boundaries, and every legal width-driven character boundary. Candidates are scored by measured overflow first, preferred separator/camel boundary second, visual line-width balance third, and distance from the target split last.

## Design and Rationale

The old thumbnail path treated character count as the first gate. That allowed short but wide glyph sequences to stay on one line and rely on later elision or clipping, which conflicts with the runtime text plan's requirement to make layout decisions from measured advances. The new owner calls the same retained runtime measurement function used by related Asset Browser compaction and badge layout code.

The module does not select a concrete font family, local RGB color, or painter branch. Font family and raster behavior still come from the global retained text preference route. The splitter only consumes the font size and slot width already known by the caller.

## Control Flow

`split_display_name_lines(...)` returns `(primary, continuation)`.

- Empty or short fitting names return `(name, "")`.
- Overflowing names call `name_split_byte(...)`.
- `name_split_byte(...)` builds and scores candidates, then returns the byte index for the selected character boundary.
- Each output line is trimmed of whitespace and title separators so a split at `_`, `-`, `.`, `/`, or `\` does not leave separator noise at the line edge.
- If trimming would make either line empty, the function falls back to the midpoint target split.

## Edge Cases and Constraints

The minimum split length is six characters on both sides, so tiny names do not produce unstable two-line labels. The score uses a small measurement epsilon only for fit checks; it does not hide real overflow in candidate scoring.

`summary_nodes.rs` deliberately passes its own width and font metrics. This prevents the Asset Browser summary from inheriting the thumbnail tile's narrow 96 px budget and breaking otherwise readable summary titles into unnecessary two-line labels.

## Test Coverage

Current source regressions cover:

- short but wide thumbnail-style names that must split before they hit the character limit;
- camel-case logical names that should keep a semantic boundary when width permits;
- selected summary names that fit the wider summary slot and should remain one line.

As of S15.4fa, full focused Cargo and screenshot refresh are deferred because other runtime Cargo/rustc lanes are active. Screenshot evidence for future runs must be written under `docs/tests/editor`, not `target`.

## Plan Sources

This module implements the S15.4fa/S15.6eb row in `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md` and supports the runtime text measurement priority in `docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`.

## Open Issues

This module only chooses display-name line boundaries. It does not generate real raster thumbnails, validate the whole Asset Browser window visually, or close the broader editor layout goal. Those remain higher-level S15 acceptance work.
