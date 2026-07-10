# Runtime asset-pipeline audit owner sync acceptance

## Scope

Runtime 04 audit-owner and mirror-document maintenance only. This slice does not change asset facade, artifact, watcher, worker-pool, importer, or resource production behavior.

## Baseline problem

- `asset_pipeline_source_inventory.py` still counted 11 route-parent guard files after the tests had been split into folder-backed child owners.
- Eight required anchors were consequently reported missing even though their current owners existed: facade query, four artifact scene roundtrips, worker policy, mirror docs, and the Runtime 04 Cargo gate.
- The Rust mirror guard and current module/architecture documentation repeated the stale owner count.

## Invariants

- Runtime 04 owns 22 source files and 17 current guard/test files.
- All 24 named guard anchors and all 20 behavior-test anchors are discoverable from their real child owners.
- Artifact scene roundtrips remain 4/4 and worker diagnostics remain 7/7.
- The mirror-doc aggregate guard is discoverable and executable.
- Current plan and module documentation reports 17 guard/test owners consistently.
- Runtime 04 remains `in_progress` until its broader `asset::` and `worker_pool` Cargo gates pass in a clean compilation window.

## Evidence

- `python -m unittest tools.tests.test_runtime_asset_pipeline_audit`: passed 1/1.
- Python byte-code compilation for the asset-pipeline audit inventories, boundary, and regression test: passed.
- Direct asset-pipeline audit: source files 22/22, guard/test files 17/17, guard anchors 24/24, behavior-test anchors 20/20, artifact scene roundtrips 4/4, worker diagnostics 7/7, empty missing guard/test/behavior/doc/Cargo anchors, mirror-doc guard present, and `risks = []`.
- Wrapped standalone Rust mirror-doc guard: passed 1/1 after aligning its aggregate input with Python by reading the `load_state.rs` test owner and completing the runtime-index mirror.
- Scoped Rust formatting and diff-health checks passed. The workspace-wide formatting check timed out after 124 seconds and is not counted as a pass.
- Package-level Runtime 04 behavior tests were not rerun because the shared lib-test baseline is currently blocked by active out-of-scope Runtime text-layout work at `ui/text/layout_engine/visual_order.rs:79` (`E0282`). This static evidence does not replace the broader behavior gate.

## Decision

The Runtime 04 audit-owner synchronization slice is accepted as static maintenance. Runtime 04 itself remains `in_progress` pending the broader `asset::` and `worker_pool` Cargo gates.
