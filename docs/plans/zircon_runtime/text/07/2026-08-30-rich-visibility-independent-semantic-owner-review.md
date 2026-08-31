# Rich visibility-independent semantic owner review

Date: 2026-08-30

Status: `RRT-P1-039_visibility_independent_surface_semantic_owner_static_complete /
RRT-P1-040_typed_children_and_managed_validation_pending`

## Scope

This slice closes the hidden relation-target gap left by the first RRT-P1-039 correction. A rich
`labelled_by` or description target can be retained in the accessibility snapshot while hidden from
arrange/paint, so it has no current render command or resolved layout artifact. It still must expose
compiled visible text without reading raw markup. This slice does not claim typed link/inline/list/
table children or interactive rich-run actions.

## Reproduction and owner review

The accessibility extractor intentionally retains a hidden relation target, clears its children and
actions, and removes it from normal traversal. Render extraction intentionally omits the same hidden
node. The initial RRT-P1-039 adapter therefore had no artifact to query and removed the relation text.

`UiSurface` already owns `UiTextMeasureCache`, and that cache owns the Surface's
`SharedTextLayoutSession`. The session owns the versioned `RichTextParser` and its bounded compiled
cache used by measure, layout, render preparation, link hit testing, and paint artifacts. Creating an
accessibility parser or another semantic cache would split parser generation and invalidation. Eagerly
compiling every hidden rich node during rebuild would also add work for nodes that are neither rendered
nor included by accessibility.

## Unreal boundary

Local Unreal `SRichTextBlock` keeps its rich marshaller and `FSlateTextBlockLayout` under the widget
owner; desired size, arrange, children, and paint consume that retained owner. Hyperlink and widget
decorators produce real Slate runs/widgets with routable lifetime and action ownership. Zircon follows
the same direction: a non-painted semantic query enters the retained Surface text owner, while future
interactive children must be backed by real qualified identities rather than adapter-generated ids.

## Implemented contract

1. `RichSemanticProjection` can now be constructed from an exact source/format-matched
   `Arc<CompiledRichText>` as well as an opaque render artifact.
2. `UiTextMeasureCache::compile_rich_semantic_projection` calls the retained
   `SharedTextLayoutSession`. Cold requests use its existing parser admission and compiled-cache
   budgets; warm requests reuse the same generation-owned artifact.
3. `UiSurface` exposes only that narrow semantic service. The accessibility module still imports no
   parser and owns no cache.
4. A published per-node render-command range remains authoritative. Missing, stale, mismatched, or
   ambiguous visual artifacts fail closed and cannot be hidden by compiling newer template metadata.
5. Only when a node has no published command range does accessibility ask the Surface text owner for
   a visibility-independent projection. Parse/admission failure returns no rich name and may continue
   only through existing explicit alt/tooltip fallbacks; raw markup is never restored.

## Algorithm and engineering boundary

For a visually published node, the existing cost remains `O(log N + C + B + V)`: indexed node-range
lookup, inspection of its `C` commands, exact source validation over `B` bytes, and required `V`-byte
DTO materialization. A non-rendered cold projection hashes and parses the admitted source under the
existing rich parser budgets, then builds the canonical compiled artifact; a warm projection hashes/
validates the source and reuses that cache entry. It performs no shaping, line layout, glyph build, or
full command scan. No second parser, semantic cache, or eager hidden-tree pass was added.

No timing, allocation, RSS, or power improvement is claimed. A dedicated semantic cache or eager
publication remains prohibited until matched repeated-snapshot profiling demonstrates a bottleneck and
defines source/format/provider/tree invalidation.

## Typed-child blocker review

Current `UiAccessibilityNode`, child relations, focus, and `UiAccessibilityActionRequest` all use
`UiNodeId`, a single `u64` identity that dispatch resolves against the real UI tree. A rich link now
retains a parser-approved typed target plus its compiled run/range, while an inline object retains its
compiled run/range; neither has a qualified child identity and neither can safely impersonate a `UiNodeId`. Doing so would
misroute focus/activate actions and create unstable ids across source generations. The subsequent image
slice now retains alt/tooltip and compiled fallback text, but image actions/resource outcomes and
icon/widget alternatives remain incomplete; table and full block-tree semantics also remain open.

RRT-P1-040 must first select one qualified semantic identity and action route backed by compiled run or
real UI-child ownership. The subsequent RRT-P1-037 list slice now retains typed BBCode list kind,
ordinal, marker style, level, and exact range, but it does not change this publication blocker. Until
the qualified identity route exists, synthetic AccessKit children and byte-offset ids are rejected.

## Evidence and remaining gates

- A failing-first contract requires the Surface session owner and forbids an accessibility parser.
- A Rust behavior test requires a hidden HTML relation target to have no render command and still
  resolve to `Hidden relation`.
- Existing visible HTML, BBCode relation, and stale-render tests retain their contracts.
- The complete Runtime Text static suite passes 54/54 after implementation.

Rust behavior tests are written but managed Cargo remains unavailable. AccessKit/screen-reader,
product accessibility inspection, WGPU, PNG, allocation/RSS/power, milestone commit, and WeCom remain
open. This non-visual slice creates no strategy screenshot.
