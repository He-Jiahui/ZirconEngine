# Runtime15 linear party ready-check sweep

## Scope

- Plan gap: `SOC-P1-020`.
- Baseline: `bb5e357d03368f07c5f20b39cc43dcb25fab8638`, epoch `441`.
- Session: `root-runtime15-party-ready-sweep-20260826`.
- Owners: `world/state.zr` integration and `social/party_raid_state.zr` expiry logic.
- This slice removes the fixed-tick nested scan. It does not claim the later `PartyRecord`, incremental pending-count index, deadline heap, event/outbox, or complete Runtime15 product qualification.

## Change

- Placed ready-check expiry in the existing party/raid social-domain module instead of adding more logic to the 68k-line world owner.
- The fixed-tick sweep now visits every row once and clears only rows whose shared deadline elapsed.
- Ready responses already finalize when the last pending response arrives. Member removal now applies the same completion rule at its mutation boundary, so the tick path no longer rescans all rows to discover completed checks.
- Preserved the existing row representation, wire/save schema, response values, and timeout boundary.
- Extended the existing M6 party/raid Zr package with mixed parties, expired rows, future rows, and canonical empty rows.

## TDD and static evidence

- RED: `python tools/tests/test_runtime15_party_ready_check_performance_contract.py` failed because the linear sweep module/package and world delegation did not exist.
- GREEN: the same command passes `5/5` after extraction and integration.
- The deterministic 4,096-row complexity gate compares 8,386,560 legacy prior-row comparisons with 4,096 linear expiry checks, over 2,000x fewer checks before counting the legacy pending/clear scans.
- `git diff --check` passes for the candidate paths apart from Git's existing LF/CRLF checkout notice.

## Local performance evidence

The standalone Node oracle compared the legacy and linear end-of-tick state across 1,000 seeded valid snapshots plus the last-pending-member removal path. It then measured 4,096 ready-check rows across 2,048 expired parties, 21 alternating sample pairs, and nearest-rank percentiles. State cloning occurs outside timed intervals.

| Metric | Nested tick sweep | Linear row expiry | Change |
|---|---:|---:|---:|
| P50 | 27.0382 ms | 0.0472 ms | -99.825% |
| P95 | 69.7693 ms | 0.0905 ms | -99.870% |
| expiry row growth | quadratic | linear | slope converged |

Three additional runs reported P50 improvements from 99.805% to 99.875%, and P95 improvements from 99.657% to 99.903%. These timings are an algorithm model tied to source-shape contracts, not a full WOC server tick claim.

## Async validation

The coordinator batch must run the five Python contracts, the randomized parity/performance model, candidate diff checks, `woc_m6_party_raid_tests.zrp`, and `woc_world_state_tests.zrp` so the existing response/removal/timeout integration fixture remains green. Acceptance requires both packages to return zero and P50/P95 improvements to remain at least 35%.

Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 row.
