# Dispatch reply route-trace exact capacity

Plan: `docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md`

Status: implementation and focused source evidence complete; managed release benchmark pending.

## Problem

`UiDispatchReply::merge_route` built its diagnostic route trace with `Vec::new()` even when
the input iterator published an exact length. A routed input event therefore allowed the trace
buffer to grow geometrically while every final trace entry was already known to be retained.

## Change

The merge now converts the input once, reads its size hint, and reserves only when lower and
upper bounds are exactly equal. Iterators without an upper bound retain zero eager capacity, so
a misleading lower bound cannot cause an oversized allocation. Merge order, stop semantics,
effect ranges, ignored-effect counts, and serialization are unchanged.

## Performance gate

The ignored release benchmark compares the same production merge algorithm over 64 steps. The
control iterator deliberately hides its size while the candidate exposes the exact size; both
clone the same steps and execute the same merge logic. It records 21 alternating sample pairs,
10,000 merges per sample, raw nanosecond series, and nearest-rank P50/P95. Acceptance requires
the exact-size path to reduce P95 by at least 5%.

Deterministic evidence is limited to capacity ownership: a three-step exact route finishes with
capacity 3, while an unbounded `size_hint` of `(4096, None)` is not trusted. This is not a claim
about allocator calls or wall-clock speed. Product acceptance waits for the release marker
`EDITOR01_DISPATCH_REPLY_ROUTE_TRACE_CAPACITY_BENCH_V1` and managed Rust regression results.

## Validation

- Rust 1.94.1 formatting passes for the implementation and focused test module.
- Targeted diff checking passes.
- The complete static performance-contract batch passes 1,630/1,630 after the adjacent
  pointer-feedback guard convergence.
- The two focused Rust regressions and ignored release benchmark are authored but have not yet
  received a managed Cargo receipt; no wall-clock performance claim is made before that ticket.
