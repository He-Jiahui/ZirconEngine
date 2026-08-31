# Runtime99zn Sound graph hierarchy index batch

## Scope

- Owner report: `docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md`
- Baseline: `8ee9411db24b7b4bdaf3fe028194642a7557c0b6`, epoch `434`
- Session: `optimize-runtime99zn-graph-hierarchy-index-r1-20260825`
- Production: `zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs`
- Behavior/performance test: `zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/performance_tests.rs`
- Structural contract: `tools/tests/test_runtime99zn_sound_graph_hierarchy_performance_contract.py`

## Problem

The incremental sound graph diff rebuilt a complete `track -> parent` hash map for every depth and ancestor query. Every rebuilt root also rebuilt that map and repeatedly scanned all tracks until its subtree stopped growing. A structural batch with many parent changes therefore multiplied full-graph allocations and scans by the candidate/root count.

## Change

- Add a `TrackHierarchyIndex` containing one parent lookup and one parent-to-children lookup.
- Lazily materialize at most one before index and one after index per graph diff through `OnceCell`.
- Reuse the index for rebuilt-root reduction, subtree projection, removal ordering, rebuild ordering, and addition ordering.
- Traverse subtrees from the children lookup with an explicit stack instead of fixed-point full-map scans.
- Preserve the zero-index-cost path for parameter-only graph updates because both cells remain uninitialized until a structural query is required.

## TDD and static evidence

- RED: `python -m unittest tools.tests.test_runtime99zn_sound_graph_hierarchy_performance_contract` failed `4/4` contracts against the repeated-map implementation.
- GREEN: the same command passes `4/4` after the hierarchy index change.
- `python -m py_compile tools/tests/test_runtime99zn_sound_graph_hierarchy_performance_contract.py` passes.
- `rustfmt +1.94.1 --check` passes for both owned Rust files.
- `git diff --check` passes apart from Git's existing LF/CRLF checkout notice.
- The production file is `578` lines after the change.

## Local release-model evidence

The standalone Rust `-O` model uses 2,048 tracks, 182 structural candidates, 21 alternating legacy/indexed sample pairs, nearest-rank percentiles, and an equality check over the depth/ancestor/subtree projection checksum.

| Metric | Legacy | Indexed | Change |
|---|---:|---:|---:|
| P50 | 61,603,600 ns | 930,800 ns | -98.489% |
| P95 | 119,019,400 ns | 1,640,100 ns | -98.622% |
| hierarchy construction | per query/root | once per graph | bounded to at most 2 per diff |

The formal crate fixture uses the same 2,048-track scale and 21 alternating sample pairs. Its acceptance gate requires at least 40% improvement for both P50 and P95 and preserves the legacy depth, ancestor, and subtree results.

## Async validation

No Cargo command was run directly in the shared checkout. Coordinator validation is submitted as one batch containing:

1. the four Python source contracts;
2. the focused hierarchy behavior-equivalence test;
3. the existing sound graph sync regression surface;
4. the ignored release benchmark with `--nocapture`;
5. plugin runtime `cargo check`.

The candidate remains pending until the coordinator reports the managed Rust batch and release benchmark green. Commit/WeCom finalization must quote the managed benchmark row rather than promote this local model to crate-level acceptance.
