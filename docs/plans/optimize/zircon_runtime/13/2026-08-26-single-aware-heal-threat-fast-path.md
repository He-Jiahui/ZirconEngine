---
title: Runtime13 Single-Aware Heal Threat Fast Path
category: zircon_runtime
report_id: Runtime13-single-aware-heal-threat-fast-path-2026-08-26
date: 2026-08-26
session_id: root-runtime13-single-aware-heal-threat-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime13 single-aware heal threat fast path

## Scope

- Parent gap: `WOC-COMB-P1-051`, with the bounded scan work also contributing to `WOC-COMB-P1-050`.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: reachable `combat/heal_state.zr`, its source/performance contract, and this record.
- This slice optimizes only healing-threat publication when exactly one hostile mob is aware of the healed target. Multi-mob threat distribution, the first eligibility scan, WorldState projection allocation, aura string identity, general threat storage, timer scheduling, and complete Runtime13 qualification remain open.

## Change

- The existing eligibility scan now remembers the latest qualifying mob index while counting aware mobs.
- A zero-count result still returns without publishing threat.
- A one-count result writes the complete healing threat directly to the remembered index and returns without scanning every mob a second time.
- A count greater than one retains the existing division and second scan, preserving equal threat distribution and source-order behavior.
- The Zr contract places the only eligible mob in slot two behind a dead row and a non-hostile row, proving the fast path does not assume slot zero.

For a single-aware workload containing `N` mob rows, eligibility predicate evaluations fall from `2N` to `N`, a 50% structural reduction. The optimization adds two scalar locals and performs no allocation, collection copy, identity conversion, or storage-layout change.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime13_heal_threat_performance_contract -v` initially passed 2/5 and failed the three guards requiring unique-index capture, direct single-target publication, and Zr semantic coverage.
- GREEN: the same command passes 5/5 after the implementation and contract case were added.
- `node --check .codex/state/session-coordinator/cargo-runs/runtime13-single-aware-heal-threat-model.mjs` passes.
- Scoped `git diff --check` passes apart from Git's existing LF/CRLF checkout notice.

The deterministic Node model uses one eligible row at the end of 512 rows, 8,000 applications per sample, 31 alternating legacy/optimized sample pairs, nearest-rank percentiles, and observable predicate counters. Every pair must produce the same final threat value.

| Metric | Double scan | Single-aware fast path | Change |
|---|---:|---:|---:|
| P50 | 20.8120 ms | 10.1024 ms | -51.459% |
| P95 | 24.6268 ms | 12.6694 ms | -48.554% |
| predicate evaluations / 8k applications | 8,192,000 | 4,096,000 | -50.000% |

These timings isolate healing-threat row scans. They do not claim end-to-end WOC tick latency, WorldState projection cost, multi-aware raid performance, ZrVM startup, or general threat-table performance.

## Async validation

One coordinator batch must run the five Python contracts, the parity/performance model, scoped diff checks, and `woc_m4_heal_state_tests.zrp` against pinned external ZrVM commit `60f6bcf4dd22bb6f5247e353bd0d97964758f157` in one managed Cargo group.

Acceptance requires the Zr package to compile and return zero, the 5/5 source contracts to pass, model parity to pass, predicate evaluations to fall exactly 50%, and P50/P95 reductions to remain at least 35%. Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 and predicate-count row and label it as single-aware healing-threat scan evidence.
