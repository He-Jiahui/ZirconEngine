# Pane button appearance publication design

Date: 2026-08-28

Status: current-source architecture review and implementation plan. The production owner paths are
already under external modification, so this slice deliberately does not edit them and does not
claim a measured product improvement.

## Decision

Button identity, visual kind and glyph selection must be resolved once at the pane
projection/publication boundary. A paint pass must consume a typed `ButtonAppearance`; it must not
infer visual semantics by scanning control IDs, labels, values, variants or validation strings.

This is part of the pane generation authority described by
`2026-08-25-pane-projection-generation-cache-design.md`. It is not a painter-private memoization
table. A private cache would duplicate ownership, require another invalidation protocol and retain
keys for nodes that have already left the pane.

## Current-source proof

The current button paint path performs semantic discovery before it can reject, clip or paint a
candidate:

- `host_contract/paint_template_nodes/template_buttons/commands.rs:19` calls
  `is_workbench_button(node)` before the extent and clip gates.
- `commands.rs:30` calls `button_kind(node)` for every accepted repaint.
- `template_buttons/content/entry.rs:35` calls `button_glyph(node)` for the same repaint.
- `template_buttons/identity.rs:61-71` assembles six borrowed string fields: `control_id`, `text`,
  `value_text`, `button_variant`, `surface_variant` and `validation_level`.
- `button_kind` tests as many as eight needles against those six fields. In the worst non-matching
  case that is 48 field/needle substring scans.
- `template_buttons/content/glyph.rs:14-31` repeats the same projection and tests as many as nine
  needles, or another 54 worst-case scans.
- `button_identity_contains` uses a byte-window walk for every field/needle pair. Its work therefore
  grows with both candidate count and string length, even when no semantic button property changed.
- `is_workbench_button` additionally checks component family/visual language, variants, six control
  ID prefixes and an `IconButton` substring before dispatching the painter.

`TemplatePaneNodeData` already carries a typed `ResolvedButtonStyle`, but button eligibility, kind,
glyph and special semantic role remain implicit strings. The painter consequently recomputes a
second, incomplete semantic model on each command extraction.

The shared owner paths are not clean in the current worktree: `TemplatePaneNodeData`, button
identity and button glyph selection contain external edits. This document records the structural
repair without absorbing or rewriting that work.

## Unreal reference contract

The checked-in Slate button implementation keeps visual identity outside paint-time string data:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SButton.h:44` initializes a
  typed `FButtonStyle`; line 71 accepts that style as a construction argument.
- `SButton.cpp:23-41` assigns explicit `Layout` or `Paint` invalidation reasons to content padding,
  pressed padding and pressed appearance attributes.
- `SButton.cpp:124-126` resolves hovered, pressed and clicked sounds from the button style during
  construction.
- `SButton.cpp:162` binds the style once.
- `SButton.cpp:220-238` updates the retained border image by selecting `Disabled`, `Pressed`,
  `Hovered` or `Normal` brushes from the style. It does not recover the button kind from labels or
  widget names during `OnPaint`.

Zircon should transfer that ownership boundary: semantic projection selects typed appearance;
interaction state selects a pre-resolved state within that appearance; paint emits commands.

## Target contract

The exact module placement should follow the pane-data owner when its current edits settle, but the
published data should be equivalent to:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ButtonVisualKind {
    Primary,
    Secondary,
    Tertiary,
    Danger,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ButtonGlyph {
    None,
    Plus,
    Trash,
    ChevronDown,
    Asset(IconAssetId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ButtonSemanticRole {
    Generic,
    AddComponent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ButtonAppearance {
    visual_kind: ButtonVisualKind,
    glyph: ButtonGlyph,
    glyph_placement: ButtonGlyphPlacement,
    content_mode: ButtonContentMode,
    semantic_role: ButtonSemanticRole,
}

struct TemplatePaneNodeData {
    button_appearance: Option<ButtonAppearance>,
    button_style: ResolvedButtonStyle,
    // Other retained node data.
}
```

`None` is the authoritative rejection result for this painter. `Asset(IconAssetId)` is a stable,
interned asset identity; it must not contain decoded pixels or create another image cache. The
existing `ResolvedButtonStyle` remains the theme/style product and can advance independently from
semantic appearance.

The hot path becomes one branch and enum matches:

```rust
let Some(appearance) = node.button_appearance else {
    return false;
};
if !has_paintable_button_extent(rect) {
    return true;
}
paint_button(node, appearance, rect, clip, state);
```

## Ownership and invalidation

`ButtonAppearance` belongs to the node's semantic/control generation. It is recomputed only when an
explicit appearance dependency changes:

| Input change | Required action |
| --- | --- |
| component family or authored button role | publish/revoke `button_appearance` |
| authored visual kind/variant | update `visual_kind` |
| authored glyph/icon ID/placement | update glyph fields |
| authored semantic action role | update `semantic_role` |
| icon-only/content-mode property | update `content_mode`; request layout if intrinsic size changes |
| theme or resolved style generation | update `button_style`; request paint or layout according to changed metrics |
| hover/pressed/enabled/focus | select retained state; paint-only unless resolved padding changes geometry |
| label/value text only | update text/layout; do not reclassify appearance |
| frame/clip only | geometry/paint update; do not reclassify appearance |

During compatibility migration, the projection layer may translate the existing six string fields
once per semantic generation. It must count `legacy_button_appearance_resolution_total` and typed
ambiguity/fallback reasons. The translation is removed after every producer publishes explicit
properties. In the final contract, control IDs, visible labels, value text and validation messages
are not visual type information.

No normal hover, press, pointer move, repaint or resize is allowed to advance the semantic/control
generation or execute legacy classification.

## Migration sequence

1. Add projection-level characterization tests for the complete current identity/kind/glyph matrix,
   including exclusions for drawer tabs, tools, toolbars, rail, status, mini and icon buttons.
2. Introduce shared typed appearance models outside painter-private modules. Resolve them at all
   `TemplatePaneNodeData` construction/publication boundaries.
3. Add an incremental contract: changing only label/value/frame/clip preserves appearance identity;
   changing an explicit visual/glyph property publishes exactly one affected-node appearance patch.
4. Switch button command/content painters to the typed field and add a source guard that forbids
   `button_identity_values`, substring windows and control-ID prefix classification in the paint
   subtree.
5. Run visual equivalence tests before removing the legacy resolver. Then hard-cut the resolver and
   require every document/pane producer to publish explicit appearance.
6. Integrate appearance generation with retained pane generations so closed/replaced panes release
   their products without a separate cache lifecycle.

## Complexity and acceptance budget

Let `B` be button painter candidates, `M` the aggregate inspected string length and `C` the number
of nodes whose authored appearance changed.

| Operation | Current upper shape | Required shape |
| --- | --- | --- |
| unchanged command extraction | `O(B * M)` byte-window classification | `O(B)` typed branch/match |
| hover/pressed repaint | repeats identity/kind/glyph scans | `O(1)` state selection per affected button |
| label/value update | may change inferred kind/glyph accidentally | text/layout only; appearance work `O(0)` |
| explicit appearance patch | inferred on every later repaint | `O(C)` publication plus affected paint |
| resize | repeats string classification for repainted candidates | geometry/damage work, no appearance work |

Required counters:

- appearance resolve/reuse/patch counts by pane and generation;
- legacy resolver and ambiguity/fallback counts by reason;
- command extraction button candidates, typed accepts and rejects;
- resolved-style generation changes separated into paint-only and layout changes;
- per-frame button command count and affected-node count;
- allocation count/bytes and CPU duration for idle, 10,000 hover transitions, 10,000 clicks and a
  continuous resize trace;
- process RSS at startup, peak interaction and post-quiescence.

Dynamic acceptance requires identical command/visual output for the characterization corpus, zero
legacy resolver calls after hard cut, zero appearance-generation advances during hover/click/resize,
and product p50/p95/p99 plus CPU/RSS evidence from the Editor executable. Static scan counts are not
accepted as product timing.

## Deterministic pressure evidence

`tools/editor_button_appearance_classification_pressure.py` models the exact eight kind needles and
nine glyph needles against the six fields used by current source. Its default representative field
lengths are 32, 24, 24, 9, 16 and 8 bytes. A secondary/non-matching identity is intentionally used
because it executes every branch; actual matching buttons may short-circuit earlier.

| Candidates per repaint | Repaints | Candidate visits | Field/needle scans | Substring windows | Byte comparisons (upper bound) | Typed field reads |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 10,000 | 10,000 | 1,020,000 | 14,410,000 | 79,610,000 | 20,000 |
| 32 | 10,000 | 320,000 | 32,640,000 | 461,120,000 | 2,547,520,000 | 640,000 |
| 512 | 1,000 | 512,000 | 52,224,000 | 737,792,000 | 4,076,032,000 | 1,024,000 |

The byte-comparison column assumes every `eq_ignore_ascii_case` comparison reaches the end of the
needle, so it is an upper bound rather than a CPU-instruction count. The current classifier borrows
its strings, so this model claims zero allocation reduction. It also excludes eligibility checks,
layout, text shaping, clipping, command-fragment filtering, GPU work and present.

Artifact:
`E:\zircon-profiles\editor-button-appearance-classification-pressure-20260828.json`, SHA-256
`4F38A91488543AC2E09B9E2FAAD5D00F8E95E55C582FF2A4439DE5666660A296`. The focused deterministic and
source-binding test suite is 4/4 green. This artifact is not product timing.

## Adjacent work deliberately excluded

- text label allocation and shaping belong to the text-layout/command-fragment caches;
- SVG decode, tessellation, atlas upload and GPU residency belong to the asset/render cache contract;
- retained per-node command fragments belong to `HostPaintFragmentCache`;
- pane reconstruction and large collection virtualization belong to the pane generation design.

Those systems consume the typed appearance, but this migration must not silently combine their
lifecycles or use button appearance as a substitute cache key.
