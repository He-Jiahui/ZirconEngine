Plan: docs/plans/optimize/zircon_runtime/98-runtime-hybrid-global-illumination-scene-representation-surface-cache-global-sdf-screen-probe-radiance-cache-product-integration-current-source-review.md
Milestone: M1
Status: validation_pending
Files: ["zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_trace_support.rs", "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_trace_support/performance_tests.rs", "tools/tests/test_runtime98_lineage_trace_support_performance_contract.py"]

# Runtime98 Lineage Trace Support Batch

## Scope delivered

The Hybrid GI lineage-support refresh now resolves the bounded scheduled trace-region set once per
refresh and shares the copied slice across direct, ancestor, and descendant probe scoring. The old
path rebuilt the same `Vec` and repeated the same region-ID tree lookups for every scored probe.

This batch preserves the existing support equation, ancestor and descendant falloff, cycle bounds,
requested-probe support, recent-support decay, and quantization. The production change is limited to
frame-local CPU work and does not change the render contract or GPU representation.

The broader Runtime98 plan remains open. This slice does not claim closure of scene authoring, the
dual GI owner, CPU readback/reupload, material-correct Surface Cache capture, GPU residency, full
resolution reconstruction, hardware ray tracing, Editor operations, or product qualification.

## Fresh local evidence

TDD first produced two failures and one error against the repeated-resolution implementation. After
the change, the Python performance source contract passes 3/3. Python bytecode compilation, Rust
1.94.1 formatting/parsing for both Rust files, and scoped whitespace validation also pass.

A standalone Rust 1.94.1 `-C opt-level=3` model used 4,096 probes in a one-root hierarchy, 16
scheduled trace regions, and 21 alternating legacy/optimized sample pairs. The percentile method is
nearest-rank.

| metric | legacy | optimized | reduction |
| --- | ---: | ---: | ---: |
| refresh P50 | 10.5595 ms | 2.7415 ms | 74.038% |
| refresh P95 | 19.0249 ms | 5.7225 ms | 69.921% |
| scheduled-region resolutions per refresh | 12,286 | 1 | 99.992% |

The managed ignored Rust benchmark uses the actual module state, the same 4,096-probe/16-region
fixture, 21 alternating pairs, and a 40% P95 reduction gate. Managed compilation, behavior
equivalence, and release timing are pending in the asynchronous coordinator batch. No local Cargo
command or Cargo dry-run was launched.

## Review

The resolved region records are copied values capped at 16 entries, so later mutation of recent
support maps cannot invalidate references or extend a borrow across the refresh. External single-
probe queries still resolve one bounded region slice per query; only the batch refresh shares it
across probes. Independent review remains an integration gate after managed validation returns.
