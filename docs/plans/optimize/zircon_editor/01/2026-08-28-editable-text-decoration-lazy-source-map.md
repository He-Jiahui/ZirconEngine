# Editable text decoration touched-line source-map review

Status: `static_candidate`; managed Rust and product profile remain pending.

## Finding

`zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs` generates selection and IME decoration geometry from an already-resolved text layout. The previous implementation eagerly constructed one `UiTextLineSourceMap` for every resolved line before it knew which lines intersected the selection or preedit ranges. Constructing a map projects each line's visual/source clusters and may initialize exact-advance state, so a one-line caret/selection update in a long editor document paid work proportional to all text clusters.

The output loop was decoration-major and line-major. That order is observable because composition highlight, selection, and preedit underline commands retain declaration order. Reordering to line-major would be unsafe. Assuming source ranges are globally sorted and applying binary search would also add a contract that `UiResolvedTextLayout` does not currently declare at the interface boundary.

## Reference-engine boundary

Unreal is the primary reference. `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/TextLayout.cpp` owns retained `LineModels` and `LineViews`, supports `bLazyViewGeneration`, creates a missing view through `EnsureLineViewIsCreatedForLineModel`, iterates from an explicit line model/view, and locates a model's first view with `Algo::UpperBoundBy`. This is evidence for retained line authority and localized materialization, not permission to copy Unreal types into Zircon.

Zircon's current DTO does not yet expose an equivalent durable line-view index. This slice therefore makes the narrow safe improvement: preserve the existing resolved layout and source-map geometry authority, but build transient maps only for lines whose authoritative `source_range` intersects at least one decoration.

## Implemented algorithm

- Preserve decoration declaration order and original line order.
- Probe the existing `UiResolvedTextLine::source_range` before any cluster projection.
- Store initialized maps in a transient `HashMap<line_index, UiTextLineSourceMap>` because the cache does not participate in output ordering.
- Reuse a touched line's map across selection and all IME clauses in the same command build.
- Do not retain the cache across layout generations; invalidation remains owned by the resolved layout publication.
- When range decorations exist, pass their transient cache into caret geometry so a caret on a
  touched line reuses that exact map rather than rebuilding its visual/source-cluster projection.
  A caret-only update still initializes exactly one map.
- Because `UiResolvedTextLayout` is an interface DTO rather than a type-level
  sorted slice, the transient cache checks line-range monotonicity once. The
  normal shaped-layout path keeps the binary interval probe; an unordered or
  foreign DTO uses a linear intersection fallback so the optimization cannot
  silently omit a decoration.
- Once either caller has established that a line intersects the decoration
  range, the append helper obtains the cached map through `for_line` directly.
  It no longer repeats the same range predicate before every span projection;
  the unordered fallback keeps its explicit predicate at the DTO boundary.

For `L` lines, `D` decoration ranges, and `T` unique touched lines, source-map construction changes from `O(all text clusters)` and `L` retained entries to `O(clusters in T)` and `T` entries. Range intersection probes remain `O(D * L)` in this slice, while the redundant touched-line predicate after a successful probe is removed (`O(D * T)` branch evaluations). Reducing the probes themselves requires a declared, tested monotonic line-range index at the resolved-layout authority and is not claimed here.

## Evidence and gates

The lower Rust regression constructs 128 lines and gives one line a composition highlight, selection, preedit underline, and caret. It requires exactly one map initialization while preserving declaration order, range, caret geometry, and the selection/underline frames. A companion regression swaps two line records and verifies that an unordered interface DTO still emits its matching decoration exactly once. `tools/tests/test_runtime_ui_text_decoration_source_map_pressure.py` locks the source invariant, direct touched-line lookup, and deterministic pressure model.

The default model covers 128, 4,096, and 65,536 lines with one touched line, three decorations, and 32 clusters per line. At 65,536 lines it models map constructions changing from 65,536 to 1 and cluster projection visits from 2,097,152 to 32. The follow-up model also records three redundant touched-line predicates before direct `for_line` lookup and zero afterward. These are algorithm work units, not CPU, allocator, RSS, or latency measurements. The unordered-DTO fallback is deliberately outside this model and is selected only after the one-time monotonicity check.

Acceptance still requires official managed validation of the focused Rust regression and existing bidi/multiline/IME source-map tests, followed by current-source Editor product profiling. Product evidence must record text-edit CPU and allocation counts plus UI input-to-present p50/p95/p99 and RSS; no timing claim is made from this model.
