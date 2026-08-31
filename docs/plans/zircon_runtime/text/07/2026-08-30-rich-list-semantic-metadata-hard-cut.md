# Rich list semantic metadata hard cut

Date: 2026-08-30

Status: `RRT-P1-037_typed_list_item_metadata_static_complete /
RRT-P1-040_qualified_publication_and_managed_validation_pending`

## Scope

This infrastructure slice prevents BBCode list semantics from being discarded during compilation.
It does not claim the complete RRT-P1-037 typed block tree, HTML list authoring, virtual accessibility
children, link actions, table header/caption semantics, or managed rendering acceptance.

## Current-source finding

The BBCode block parser knew whether a list was ordered, which marker algorithm it used, the current
ordinal, and the nesting stack depth. It then flattened that information into a visible marker string
and stored only `ParagraphOverride.list_prefix: Option<(u32, u32)>`. Downstream code could measure the
marker but could not distinguish ordered from unordered items, recover the canonical ordinal or marker
style, or expose the semantic list level without parsing rendered text. This made a future layout,
copy, and accessibility convergence impossible without a second heuristic parser.

## Reference boundary

Local Unreal rich-text code keeps parsing/marshalling, text layout, and retained run/widget ownership as
separate layers under `SRichTextBlock`, its marshaller, and `FSlateTextBlockLayout`. Decorators create
typed runs or real widgets; downstream consumers do not recover authoring identity from painted glyphs.
Zircon follows that ownership rule here: the compiler emits typed list identity once, while UI layout
consumes only a derived marker range needed for hanging-indent geometry.

## Implemented contract

1. `RichListItemKind` distinguishes unordered items from ordered items. Ordered identity contains both
   the canonical `ordinal` and `RichOrderedListMarker::{Decimal, AlphaLower, AlphaUpper, RomanLower,
   RomanUpper}`, so invalid kind/style combinations are not representable.
2. `RichListItem` carries a one-based semantic `level` and the exact compiled-visible `marker_range`.
   Nesting level is independent of visual indentation.
3. The BBCode list stack creates this metadata in the same single pass that emits the visible marker.
   Ordered ordinal advance is checked; it does not saturate into duplicate semantic identities.
4. `ParagraphOverride.list_prefix` is removed. `ParagraphOverride.list_item` is the canonical semantic
   field, and public model exports expose the typed list contract.
5. `UiParsedText` derives an `Option<UiTextRange>` from `list_item.marker_range` for layout only.
   Physical-paragraph overlap resolution uses a private `ResolvedParagraphLayoutOverride`; it cannot
   reconstruct or overwrite the semantic item.
6. Marker ranges are validated against the compiled visible text at measurement. No consumer infers
   kind, ordinal, style, or level from the marker string.

## Algorithm and performance boundary

The parser adds constant-size metadata per admitted list paragraph and reads the existing list-stack
depth in O(1). The document remains a single O(n) parse under the existing paragraph/output budgets.
Physical paragraph resolution keeps its existing sweep complexity and does not clone list marker text.
The layout projection stores one range rather than a second semantic object.

No timing, allocation, RSS, power, or cross-engine performance improvement is claimed. This is a data
ownership correction required before typed copy/accessibility publication. Further optimization remains
profile-gated by the E-drive corpus and managed validator.

## Evidence and remaining gates

- Failing-first static contracts require typed item/kind/ordered-marker declarations and forbid the old
  public `list_prefix` semantic field.
- Rust behavior tests cover unordered markers and nested ordered/unordered items, including one-based
  levels, AlphaUpper ordinals, and exact marker ranges.
- The external rich-block integration test reads `list_item.marker_range`; the layout sweep has a focused
  contract proving the range remains a private geometry projection.
- Rustfmt and scoped diff-check pass. The complete Runtime Text static suite passes 55/55 in 0.239 s.
- Production file sizes are 317 lines for `model/rich.rs`, 310 for `bbcode_blocks.rs`, 728 for
  `parser.rs`, 616 for `ui/text/rich_text.rs`, and 628 for `paragraph_layout.rs`.

Managed Cargo/Rust behavior, WGPU/PNG, screen-reader/AccessKit, allocation/RSS/power, milestone commit,
and WeCom remain open. RRT-P1-040 still requires a qualified semantic publication/action identity before
list children can be exposed. This non-visual slice creates no strategy screenshot.
