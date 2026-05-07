---
related_code:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/tests.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
implementation_files:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
plan_sources:
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - user: 2026-05-06 continue M6 text-system convergence from Unreal Slate audit
tests:
  - zircon_runtime/src/ui/text/layout_engine/tests.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - cargo test -p zircon_runtime --lib ui::text::layout_engine --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-ui-text-grapheme-layout" --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests::text_layout --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-ui-text-grapheme-layout" --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-ui-text-grapheme-layout" --message-format short --color never
  - cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib screen_space_ui_plan --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib text_attrs --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_runtime_text_painter --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
doc_type: module-detail
---

# Runtime UI Text Layout Engine

`layout_engine.rs` is the shared runtime owner for `UiResolvedTextLayout`. It turns template text plus `UiResolvedStyle` typography fields into neutral line records before graphics, editor, or debug consumers see the render command.

## Layout Flow

The helper intentionally stays data-oriented:

- resolve paragraph direction from `text_direction` or the first strong character mix
- parse plain/rich source runs through `rich_text.rs`
- split newlines while preserving original byte ranges
- apply word, glyph, or no-wrap policy with current fixed-advance, grapheme-counted measurement
- trim word-wrap separator spaces at line boundaries
- apply height overflow and ellipsis policy without splitting grapheme clusters
- convert mixed-direction lines into a low-fidelity visual-order string
- emit line frames, baselines, measured widths, source ranges, visual ranges, and resolved runs

The fixed `font_size * 0.5` advance is still a scaffold, but the scaffold now counts Unicode grapheme clusters through `grapheme.rs` instead of counting scalar values. Glyphon/cosmic-text, SDF font bake, and later HarfBuzz/ICU integration remain responsible for final glyph metrics, font fallback, atlas/cache state, script shaping, cluster positioning, and GPU submission.

## Grapheme Boundary Scaffold

`grapheme.rs` centralizes Unicode segmentation for the layout foundation. Glyph wrapping, fixed-advance measurement, ellipsis truncation, and the low-fidelity RTL reversal consume grapheme clusters so combining marks such as `a\u{0301}` stay with their base character while the current text scaffold is still pre-shaping. When a rich marker splits a visible cluster across adjacent runs, the helper treats a leading continuation mark as part of the preceding visible cluster so wrap/truncation/reversal do not isolate the mark.

This follows the responsibility split seen in Unreal Slate, where too-long words fall back to grapheme-cluster wrapping and editable movement uses a character-boundary iterator. It also matches Godot's shaped-text navigation surface where next/previous grapheme positions are the unit for caret movement once shaped glyph data exists. Zircon still performs this before real shaping, so the helper only protects byte-range and visible-string scaffolds; it is not a replacement for HarfBuzz/ICU shaping or font fallback.

## BiDi Visual-Order Scaffold

The current visual-order helper follows the same responsibility boundary as Unreal Slate's `FSlateTextShaper::ShapeBidirectionalText`: split a paragraph into direction runs before shaping, then keep source-to-visual relationships available for later glyph enumeration. It also mirrors Slint's text layout boundary where positioned glyphs keep the original text byte offset so selection and caret geometry can be derived from rendered glyph order.

This helper is deliberately low fidelity. It tokenizes resolved runs into strong LTR, strong RTL, and neutral spans. Neutral tokens inherit a surrounding same-direction span, so punctuation inside an RTL phrase such as `שלום-עולם` travels with that RTL visual span. Separators at an LTR/RTL boundary stay on the LTR side so existing mixed-line spacing remains stable. RTL spans are reversed at token order and character order, while each visual run keeps the original source byte range and a current visual byte range.

This does not implement full Unicode BiDi, glyph mirroring, script-specific shaping, glyph positioning, or font fallback. Grapheme boundaries now prevent the scaffold from splitting clusters during wrap/truncation/reversal, but true grapheme cluster shaping remains deferred to the real text shaping backends. The shared layout engine only provides a stable intermediate contract for tests, render extract, and future shaped-text DTO derivation.

## Range Rules

`source_range` always points into the authored text bytes, even when a visual run has moved. `visual_range` points into the emitted visual line string. Ellipsis runs use a zero-length source range at the truncated line end because the ellipsis character is generated by overflow policy rather than authored text.

These byte ranges are the foundation for later caret, selection, composition underline, and shaped-glyph diagnostics. Tests assert both text order and ranges so later shaper upgrades can replace the low-fidelity algorithm without weakening the cross-layer contract.

## M6 Shared Shape And Editing Paint

The M6 continuation makes the shared render DTO consume this layout output more directly. `UiShapedText::from_resolved_layout(...)` now derives per-grapheme synthetic glyph records from each resolved line. Those records are not final backend glyph ids, but they give Widget Reflector, editor painter, and runtime debug payloads stable glyph count, visual frame, advance, and source range data for combining marks and emoji clusters.

Editable selection, caret, and composition underline geometry now also snaps to grapheme cluster edges in `UiRenderCommand::text_paint(...)`. A selection whose byte range falls inside `a\u{0301}` expands to the whole visible cluster, so editor/runtime painters do not split an accent or emoji component. Runtime screen-space UI planning then consumes the same shared `UiTextPaintDecoration` frames: selection is emitted as a pre-text quad, while caret and composition underline are emitted as post-text quads after the glyphon/SDF text pass.

Rich style paint now follows the same route. `UiTextPaint.runs` is derived from shaped clusters and carries `UiTextPaintRun` records with Strong/Emphasis/Code style flags. Runtime planning prefers these shared runs over raw line text, so a resolved line containing plain, strong, and code fragments becomes separate text batches with stable run frames. The glyphon backend converts the flags to bold, italic, and monospace attrs, while the editor native painter uses the same DTO to apply software fallback bold/italic drawing. This closes the immediate renderer-local rich-run parsing gap without claiming final HarfBuzz-level metrics.

## Tests

`ui::text::layout_engine` module tests cover grapheme-safe fixed-advance measurement, glyph wrapping, ellipsis truncation, low-fidelity RTL reversal, and rich-run boundary clusters.

`ui::tests::text_layout` covers word wrapping, clip-frame line removal, rich ellipsis preservation, mixed LTR/RTL visual-order ranges, neutral separator assignment inside RTL spans, and editable text action interactions.

`render_contracts` covers the shared shaped artifact and decoration cluster snapping through `ui_shaped_text_contract_derives_grapheme_glyph_bounds` and `ui_text_decorations_snap_to_grapheme_cluster_edges`.

`screen_space_ui_plan` covers runtime renderer consumption of shared decoration geometry through `screen_space_ui_plan_uses_shared_text_decorations_as_pre_and_post_text_draws` and rich paint run splitting through `screen_space_ui_plan_splits_rich_text_runs_from_shared_paint`.

`text_attrs_maps_shared_rich_run_style_to_glyphon_attrs` covers the native glyphon mapping from shared rich run style flags to backend attrs. `native_runtime_text_painter` remains the editor native painter smoke gate for shared runtime text payload consumption.
