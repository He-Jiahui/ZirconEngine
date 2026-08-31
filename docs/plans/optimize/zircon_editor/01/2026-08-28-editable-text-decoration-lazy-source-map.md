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
- Keep caret geometry unchanged; it already creates exactly one line map.

For `L` lines, `D` decoration ranges, and `T` unique touched lines, source-map construction changes from `O(all text clusters)` and `L` retained entries to `O(clusters in T)` and `T` entries. Range intersection probes remain `O(D * L)` in this slice. Reducing those probes requires a declared, tested monotonic line-range index at the resolved-layout authority and is not claimed here.

## Evidence and gates

The lower Rust regression constructs 128 lines and gives one line a composition highlight, selection, and preedit underline. It requires exactly one map initialization while preserving declaration order, range, and the selection/underline frames. `tools/tests/test_runtime_ui_text_decoration_source_map_pressure.py` locks the source invariant and deterministic pressure model.

The default model covers 128, 4,096, and 65,536 lines with one touched line, three decorations, and 32 clusters per line. At 65,536 lines it models map constructions changing from 65,536 to 1 and cluster projection visits from 2,097,152 to 32. These are algorithm work units, not CPU, allocator, RSS, or latency measurements.

Acceptance still requires official managed validation of the focused Rust regression and existing bidi/multiline/IME source-map tests, followed by current-source Editor product profiling. Product evidence must record text-edit CPU and allocation counts plus UI input-to-present p50/p95/p99 and RSS; no timing claim is made from this model.
