# Runtime99u Entity remap successor probe batch

## Scope

- Owner report: `docs/plans/optimize/zircon_runtime/99u-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-current-source-review.md`
- Baseline: `0fd7df4ecdd157f9505cd51013780e3225cfb83c`, epoch `435`
- Session: `optimize-runtime99u-entity-remap-probe-r1-20260825`
- Production: `zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction.rs`
- Behavior/performance test: `zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/performance_tests.rs`
- Structural contract: `tools/tests/test_runtime99u_entity_remap_performance_contract.py`

## Problem

`build_entity_remap` restarted at each source entity ID and advanced one ID at a time while checking both the target `World` and a `BTreeSet` of targets reserved earlier in the same compile. Dense target IDs combined with many low source IDs therefore revisited the same occupied prefix for every scene entity, producing quadratic collision probes during dynamic-scene compilation.

## Change

- Replace the per-entity reserved-set scan with one `EntityIdReservationProbe` for the complete remap build.
- Keep batches below the private, module-local `ENTITY_REMAP_SUCCESSOR_PROBE_MIN_ENTITIES = 16` crossover threshold on the original linear path, avoiding one-shot cache construction cost.
- Cache the successor of each target-world or locally reserved entity ID only when that ID is encountered.
- Path-compress every walked collision chain to the successor of the newly reserved target.
- Preserve source-order allocation, duplicate-source behavior, sparse gaps, and the existing terminal `u64::MAX` exhaustion result.
- Keep the index lazy: an empty or collision-free scene does not scan or copy all target-world entity IDs.

This is a local compiled-spawn support optimization. It does not claim to close Runtime99u's registered-instance selection, replacement identity, revision-terminal, or product integration findings.

## TDD and static evidence

- RED: `python -m unittest tools.tests.test_runtime99u_entity_remap_performance_contract` failed `4/4` contracts against the repeated linear probe.
- Refinement RED: the first indexed model exposed a one-entity P50 regression of about 10x; the expanded contract then failed `1/4` until the measured crossover policy was added.
- GREEN: the same command passes `4/4` after the successor probe change.
- `python -m py_compile tools/tests/test_runtime99u_entity_remap_performance_contract.py` passes.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` passes for both owned Rust files without traversing the pre-existing dirty `transaction/tests.rs`.
- `git diff --check` passes for the owned candidate paths apart from Git's existing LF/CRLF checkout notice.
- The production file is `726` lines after the change.

## Local release-model evidence

The standalone Rust `-O` model uses 1,024 scene entities against 2,048 densely occupied target IDs, 21 alternating legacy/optimized sample pairs, 8 remap iterations per sample, and nearest-rank percentiles. It checks exact remap or exhaustion equivalence across dense, sparse, reordered, duplicate-source, terminal-ID, and 256 deterministic random fixtures.

| Metric | Legacy | Successor probe | Change |
|---|---:|---:|---:|
| 1,024-entity P50 | 431,744,700 ns | 4,677,600 ns | -98.917% |
| 1,024-entity P95 | 541,718,500 ns | 8,995,000 ns | -98.340% |
| collision probes | 2,098,176 | 5,117 | -99.756% |
| 1-entity P50, 256 iterations | 8,639,800 ns | 8,463,900 ns | -2.036% |

The crossover sweep found the uncached path faster through 8 entities and the successor probe faster from 16 entities. The formal crate benchmark uses both 1 and 1,024 source entities against 2,048 occupied targets with 21 alternating sample pairs. Acceptance requires at least 80% improvement for the large-batch P50 and P95, plus no more than 10% P50 regression for the one-entity path, after the validator independently recomputes nearest-rank percentiles from all raw sample arrays.

## Async validation

No Cargo command is run directly in the shared checkout. One coordinator batch contains:

1. the four Python source contracts;
2. the successor-probe dense/reordered equivalence and terminal-ID behavior tests;
3. the existing dynamic-scene spawn transaction regression module;
4. the ignored release benchmark with `--nocapture` and external P50/P95 recomputation.

The candidate remains pending until the coordinator reports the managed Rust batch and release benchmark green. Commit and automatic WeCom finalization must quote the managed benchmark row rather than promote the local model to crate-level acceptance.
