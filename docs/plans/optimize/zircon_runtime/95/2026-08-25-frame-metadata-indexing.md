Plan: docs/plans/optimize/zircon_runtime/95-runtime-direct-lighting-photometry-light-grid-clustered-forward-plus-shadow-atlas-cascade-point-spot-rect-cookie-ies-submission-product-integration-current-source-review.md
Milestone: M8
Status: completed
Files: ["zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs", "zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer/performance_tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/frame_plan.rs", "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/frame_plan/performance_tests.rs", "tools/tests/test_runtime95_frame_metadata_performance_contract.py"]

# Runtime95 Frame Metadata Indexing

## Scope delivered

This batch removes two avoidable frame-local metadata costs without changing the packed light ABI,
cookie atlas ordering, duplicate resolution, or feature-disabled behavior.

- Volumetric participation keeps a zero-allocation linear path for at most eight IDs. Larger lists
  build one frame-local `HashSet<u64>` and reuse it for every packed light, replacing
  O(light count x volumetric ID count) slice scans.
- Cookie planning replaces one `BTreeMap` node allocation per unique cookie with one contiguous
  `(input order, borrowed cookie)` index. Sorting by `(light_id, input order)` preserves ascending
  atlas slot order and the legacy last-input-wins rule for duplicate light IDs; grouping stops once
  the 64 atlas entries are emitted.
- Private equivalence tests cover empty, duplicate, small, and large volumetric lists plus more than
  one atlas worth of reverse-ordered cookies with a duplicate replacement.

The broader Runtime95 plan remains open: prepared-lighting authority, real GPU cluster assignment,
photometry, shadow allocation/view convergence, persistent cookie residency, authoring reachability,
fault recovery, and complete product qualification are not claimed by this slice.

## Fresh testing evidence

TDD first produced three failures against the old linear membership and tree-based cookie planner.
After implementation, the Python source contract passes 3/3, Python bytecode compilation passes,
Rust 1.94.1 formatting/parsing passes for all four Rust files, and scoped whitespace validation
passes.

Five process-level repetitions of a standalone Rust 1.94.1 `-C opt-level=3` benchmark produced the
following median-of-run nearest-rank values. Each process alternated five legacy/optimized pairs;
the managed ignored tests use 21 alternating pairs against the real module types.

| workload | legacy | optimized | reduction |
| --- | ---: | ---: | ---: |
| 32,768 lights x 8,192 volumetric IDs, P50 | 45.3544 ms | 0.4334 ms | 99.044% |
| 32,768 lights x 8,192 volumetric IDs, P95 | 46.8802 ms | 0.6425 ms | 98.629% |
| membership operations | 268,435,456 comparisons | 40,960 build/lookups | 99.985% |
| 65,536 reverse-ordered cookies, P50 | 1.7066 ms | 0.2814 ms | 83.511% |
| 65,536 reverse-ordered cookies, P95 | 2.0942 ms | 0.3257 ms | 84.448% |
| cookie tree nodes | 65,536 | 0 | 100% |

The cookie path still holds a 65,536-entry contiguous borrowed index for this stress input; the
structural claim is removal of per-entry tree nodes and allocations, not constant total scratch.
The managed Windows validation batch will compile the actual module, run focused behavior and
equivalence tests, and enforce 75% volumetric and 25% cookie P95 reductions. No local Cargo command
or Cargo dry-run was launched.

## Review

The membership threshold prevents a hash allocation for the common empty or small-list case. The
large-list index is request-local and cannot leak membership across frames. Cookie sorting includes
the original input position, so an unstable sort remains deterministic and selecting the final item
in each equal-ID group exactly preserves `BTreeMap::collect` replacement semantics. Independent
review remains an integration gate after managed validation returns.
