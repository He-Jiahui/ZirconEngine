# Runtime15 intrusive Card Duel queue

## Scope

- Plan gap: `SOC-P1-061`.
- Baseline: `bb5e357d03368f07c5f20b39cc43dcb25fab8638`, epoch `441`.
- Session: `root-runtime15-party-ready-sweep-20260826`.
- Owners: `card_duel_queue.zr`, `card_duel_queue_coordinator.zr`, their two existing Zr contract entries, the source/performance contract, and this record.
- This slice replaces queue shifts and full-candidate purge scans. It does not claim the later resident service, lifecycle index, snapshot budget, telemetry, or complete Runtime15 qualification.

## Change

- Replaced the compacting player-id array with reusable intrusive slots linked in FIFO order.
- Added generation-qualified slot handles so the coordinator removes a known candidate in O(1) and rejects stale handles after slot reuse.
- Kept duplicate membership in the coordinator and returned stable owner indexes from pairing, avoiding a player-id rescan when two heads leave the queue.
- Changed queueability purge from a full candidate sweep to a deduplicated list populated only by eligibility transitions.
- Preserved delayed purge semantics, FIFO survivor order, player-id-only snapshot wire data, and restore validation.
- Changed snapshot projection to one linked-list walk and rebuilt queue handles during restore.
- Extended existing Zr fixtures for stale handles, generation reuse, delayed purge, FIFO survivor pairing, and requeue after removal.

## TDD and static evidence

- RED: `python -m unittest tools.tests.test_runtime15_card_duel_queue_performance_contract -v` initially reported the expected `6/6` failures because intrusive slots, tracked handles, dirty purge, and the new fixture cases were absent.
- GREEN: the same command now passes `6/6`.
- A 1,024-entry drain replaces 523,776 legacy array-shift element moves with 1,024 intrusive head unlinks, over 500x fewer structural operations before purge costs are counted.
- `git diff --check` passes for the candidate paths apart from Git's existing LF/CRLF checkout notice.

## Local performance evidence

The standalone Node oracle first compared the legacy and intrusive state after 20,000 deterministic mixed join, leave, queueability, purge, pair, and snapshot operations. It then seeded 1,024 candidates, marked every third candidate unqueueable, drained the queue through the real pair/purge shape, and measured 21 alternating sample pairs using nearest-rank percentiles.

| Metric | Legacy shifting queue | Intrusive queue | Change |
|---|---:|---:|---:|
| P50 | 81.5214 ms | 0.0545 ms | -99.933% |
| P95 | 125.8094 ms | 1.0309 ms | -99.181% |
| head removal | array shift | O(1) unlink | converged |

Three earlier runs reported P50 improvements from 99.911% to 99.938% and P95 improvements from 99.486% to 99.677%. These timings are an algorithm model tied to source-shape contracts, not a full WOC server tick claim.

## Async validation

The old prebuilt `zr_vm_cli.exe` rejects the repository's package-wide legacy import/function syntax before reaching the candidate code, so it provides no candidate-specific result. The coordinator batch must therefore use pinned external ZrVM commit `60f6bcf4dd22bb6f5247e353bd0d97964758f157` and run the six Python contracts, the 20,000-operation parity/performance model, candidate diff checks, and these four packages in one managed Cargo group:

- `woc_card_duel_primitives_tests.zrp`
- `woc_card_duel_queue_coordinator_tests.zrp`
- `woc_card_duel_service_tests.zrp`
- `woc_card_duel_snapshot_tests.zrp`

Acceptance requires all four packages to return zero and P50/P95 improvements to remain at least 35%. Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 row.
