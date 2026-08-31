# Runtime16 keyed reset lock store

## Scope

- Plan gap: `INST-P1-035`.
- Owner: `examples/woc/scripts/woc_game/src/instances/dungeon_reset_state.zr`.
- Baseline: `bb5e357d03368f07c5f20b39cc43dcb25fab8638`, epoch `441`.
- Session: `root-runtime16-keyed-lock-store-20260826`.
- This slice closes parallel lock-column ownership and owner-key linear lookup. It does not claim durable transaction/CAS, a global expiry heap, or full Runtime16 product qualification.

## Change

- Replaced `lockOwnerKeys`, `lockDungeonIds`, `lockAvailableAt`, and `lockClaimIds` with one `DungeonResetLock` row type.
- Grouped rows in `DungeonResetLockBucket` values ordered by durable owner key.
- Added lower-bound binary search for owner lookup. New owner buckets retain sorted order even when locks arrive out of order.
- Lock updates mutate one row in place. Exact-key lookup retains lazy expiry and removes the now-empty owner bucket without cross-column shifts.
- Updated reset, inheritance, admission, and claim lookup consumers to borrow the resolved lock row.
- Added Zr contract fixtures for bucket order, keyed update, and expiry removal.

## TDD and static evidence

- RED: `python tools/tests/test_runtime16_keyed_reset_lock_performance_contract.py` failed four source/behavior contracts against the four-column linear implementation.
- GREEN: the same command passes `5/5` after the keyed bucket implementation.
- The deterministic 4,096-owner complexity gate compares 4,096 worst-case legacy owner comparisons with at most 13 binary-search comparisons, a reduction of more than 300x.
- `git diff --check` passes for the candidate paths apart from Git's existing LF/CRLF checkout notice.

## Local performance evidence

The standalone Node model first compared legacy and keyed set/get/expiry behavior across 20,000 seeded operations. It then measured 4,096 owners, 256 near-tail lookups per sample, 21 alternating sample pairs, and nearest-rank percentiles. Fixture construction occurs outside timed intervals.

| Metric | Four-column linear scan | Keyed owner buckets | Change |
|---|---:|---:|---:|
| P50 | 1.4174 ms | 0.0356 ms | -97.488% |
| P95 | 2.2193 ms | 0.1204 ms | -94.575% |
| worst-case owner comparisons | 4,096 | <=13 | -99.683% |

Two additional runs reported P50 improvements of 97.256% and 97.898%, and P95 improvements of 98.524% and 97.837%. These timings are an algorithm model tied to source-shape contracts, not a claim about full WOC server tick latency.

## Async validation

The coordinator batch must run the five Python contracts, the 20,000-operation parity/performance model, candidate diff checks, and a Windows-native pinned-ZrVM compile/run of `woc_m7_dungeon_reset_state_tests.zrp`. Acceptance requires the Zr package to return zero and both P50/P95 improvements to remain at least 35%.

Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom result must include the managed P50/P95 row rather than only this local model.
