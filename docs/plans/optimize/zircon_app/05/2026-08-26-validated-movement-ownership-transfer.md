# Zircon App05 validated movement ownership transfer

## Scope

- Owner report: `docs/plans/optimize/zircon_app/05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md`
- Finding: `WOC-SVC-P2-009`
- Baseline: `a8eca85cc83008aeb200dce2d2b01e2ae3c157c9`, epoch `436`
- Current Session: `root-app05-movement-transfer-release-r2-20260831`
- Reclaimed from archived Session: `root-zircon-app05-movement-ownership-transfer-20260826`
- Production: `examples/woc/native/apps/woc_server/src/fixed_tick_driver.rs`
- Behavior regression: `examples/woc/native/apps/woc_server/tests/fixed_tick_driver.rs`
- Structural contract: `tools/tests/test_woc_server_movement_transfer_performance_contract.py`

## Problem

`FixedServerTickDriver::advance` took the pending movement vector, cloned it into `MovementFrameBatch::new`, copied the canonical frames back into the fault diagnostic, and cloned that diagnostic vector again for `tick_with_movement`. A successful server tick therefore copied the complete movement payload three times in the service layer before the runtime consumed it. The repeated validation also duplicated work already completed atomically by `enqueue_movement`.

## Change

- Keep `enqueue_movement` as the sole per-frame validation and duplicate-actor admission boundary.
- Replace the cloned `MovementFrameBatch` round trip with `canonicalize_pending_movement`, which checks the same public protocol frame limit and sorts by the same `(actor.id, actor.generation)` key.
- Copy the canonical vector once for fault diagnostics, then transfer the original vector directly into `tick_with_movement`.
- Preserve the existing oversized-batch error and rebuild every pending command/movement index before returning it; a new 65,537-frame regression proves the pending queue is not lost.
- Preserve the diagnostic payload on VM fault. This candidate intentionally does not remove the one copy required by the current failure-recovery contract or change runtime payload encoding.

## TDD and static evidence

- RED: `python -m unittest tools.tests.test_woc_server_movement_transfer_performance_contract -v` failed `4/4` contracts against the clone/batch/to-vec path.
- GREEN: the same command passes `5/5` after the ownership-transfer implementation
  and release-gate hardening.
- `python -m py_compile tools/tests/test_woc_server_movement_transfer_performance_contract.py` passes.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` passes for both owned Rust files.
- `git diff --check` passes for the candidate paths apart from Git's existing LF/CRLF checkout notice.
- The production file is 376 lines after the change.

## Local release-model evidence

The independent Rust `-O` model uses 32,768 reverse-ordered movement frames, four transfer iterations per sample, 21 alternating legacy/transferred sample pairs, and nearest-rank percentiles. Fixture construction occurs outside each timed interval; both paths perform the same canonical sort, and both retain the fault diagnostic payload.

| Metric | Clone/batch/to-vec | Ownership transfer | Change |
|---|---:|---:|---:|
| P50 | 9,649,100 ns | 4,934,700 ns | -48.858% |
| P95 | 18,983,000 ns | 8,111,200 ns | -57.271% |
| service-layer full-vector copies | 3 | 1 | -66.667% |

The formal in-crate release benchmark measures the actual protocol type and production
canonicalization helper. Before timing, it proves that `MovementFrameBatch` and the
transferred helper produce the same canonical frame order. It performs four warm-up
pairs, emits both raw 21-sample arrays and computed nearest-rank P50/P95 values, and
asserts at least 35% improvement for both distributions inside the Rust test. The
validator still recomputes the percentiles from the raw arrays as receipt hardening.

## Async validation

No Cargo command is run directly in the shared checkout. One coordinator batch contains:

1. the four Python source contracts;
2. formatting and candidate diff checks;
3. all seven `fixed_tick_driver` integration regressions, including oversized restoration and fault diagnostics;
4. the ignored release benchmark with `--nocapture`, including its in-test P50/P95 gates;
5. external parsing of the raw arrays and independent percentile recomputation.

The candidate remains pending until the coordinator reports both managed Cargo groups green. The historical parent-plan evidence recorded six `woc_protocol` compile errors; if they remain, this batch must report that lowest-layer failure. Commit and automatic WeCom finalization must quote the managed benchmark row rather than the standalone model.
