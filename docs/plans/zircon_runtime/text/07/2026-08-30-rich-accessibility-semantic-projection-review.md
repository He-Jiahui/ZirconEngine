# Rich accessibility semantic projection review

Date: 2026-08-30

Status: `RRT-P1-039_visibility_independent_surface_semantic_owner_static_complete /
RRT-P1-040_typed_children_and_managed_validation_pending`

## Scope

This slice fixes the MVP correctness failure where accessibility names and relation text could read
the authored `text`/`label`/`value` scalar as if rich markup were user-visible text. It does not claim
the RRT-P1-040 semantic children, actions, table/list structure, or inline-object alternatives.

## Current-source reproduction

`accessibility/name.rs` extracts scalar template metadata. Both the node-name fallback and
`labelled_by`/description target resolution previously cloned that value directly. A node authored as
`<b>Visible</b> label` with `HtmlSubsetV1` therefore exposed the tags instead of `Visible label`.

The visual path already had the required authority. A current `UiRenderCommand` retains the source,
resolved rich format, `UiResolvedTextLayout`, and opaque handle to the same `CompiledRichText` used by
layout and paint. Re-parsing in accessibility or reconstructing text from clipped/ellipsized layout
lines would create a second semantic owner and could disagree with the visual generation.

## Unreal boundary

Local Unreal `SRichTextBlock` creates one `FRichTextLayoutMarshaller` and one retained
`FSlateTextBlockLayout`; desired-size and paint update and consume that same layout cache. Plain
`STextBlock` publishes its bound text through `GetDefaultAccessibleText`. Zircon cannot copy the plain
path for rich source markup, but it follows the same ownership direction: accessibility consumes a
projection of the retained compiled/layout owner and never invokes a second markup parser.

## Implemented owner chain

1. `text/semantic_projection.rs` defines `RichSemanticProjection`. It retains
   `Arc<CompiledRichText>`, so visible text cannot outlive or detach from parser generation identity.
2. Resolution accepts an opaque artifact only when compiled source markup and versioned format match
   the current command. The projection exposes compiled visible text, not layout-line presentation.
3. `UiSurface::current_render_commands_for_node` reuses the render cache's node range index. The
   accessibility adapter never scans the complete command list.
4. `accessibility/semantic_text.rs` preserves plain scalar behavior. Rich text searches only the
   current node command range and rejects missing, stale, format-mismatched, or generation-ambiguous
   artifacts. It never imports `RichTextParser` or calls a parse helper.
5. Node-name fallback and referenced text both consume this adapter. Explicit `a11y.name`, alt text,
   and tooltip precedence are unchanged. A rejected rich projection can use those explicit fallbacks,
   but raw markup is never restored as the accessible name.

## Algorithm review

Let `N` be retained render-node ranges, `C` the commands owned by one node, `B` the source bytes, and
`V` the compiled visible bytes. Lookup is `O(log N)` in the existing `BTreeMap`, candidate inspection
is `O(C)`, exact source validation is bounded by `O(B)`, generation comparison is `O(1)` over the
existing three-field parser generation, and the required accessibility DTO materialization is
`O(V)`. There is no `O(total commands)` scan, markup parse, layout-line concatenation, or second
semantic cache. Common nodes have one owner text command.

No performance gain is claimed. This is a correctness and ownership cut. A future semantic snapshot
cache would require measured repeated accessibility extraction and explicit invalidation evidence.

## Tests and current evidence

- HTML own-name and BBCode `labelled_by` behavior tests require compiled visible text.
- A stale-source test mutates metadata without rebuilding and requires `None`, proving no fallback to
  raw markup or acceptance of the previous artifact.
- Text-owner tests cover source/format rejection and retained generation identity.
- The failing-first static contract checks that accessibility contains no parser call, uses the node
  command index, and routes both raw scalar call sites through semantic projection.
- The complete reproducible Runtime Text static suite passes 53/53. Targeted Rustfmt and scoped
  diff-check pass.

Managed Rust tests, AccessKit/screen-reader execution, product accessibility inspection, WGPU, PNG,
allocation/RSS/power, and matched-load evidence have not run. This non-visual slice does not create a
strategy screenshot. Future real framebuffer evidence remains under `docs/tests/runtime/text`, never
under `target`.

## Remaining boundary

The follow-up visibility-independent owner slice now resolves hidden relation targets through the same
Surface `SharedTextLayoutSession` only when no render command range exists; a published visual range
remains authoritative and stale artifacts still fail closed. RRT-P1-040 remains open for link actions,
inline image/icon/widget alternatives, and list/table semantic structure. Those require qualified
semantic identity and action routing, not raw scalar fallback, byte-offset ids, or an accessibility-
local parser. See
[`2026-08-30-rich-visibility-independent-semantic-owner-review.md`](2026-08-30-rich-visibility-independent-semantic-owner-review.md).
