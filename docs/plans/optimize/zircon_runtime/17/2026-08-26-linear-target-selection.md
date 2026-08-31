# Runtime17 linear target selection

## Scope

- Plan gaps: `WORLD-P1-053` and `WORLD-P1-054`.
- Owner: `examples/woc/scripts/woc_game/src/world/target_selection.zr`.
- This slice does not claim to close Runtime17 spatial indexing, LOS, collision, navigation, or full product qualification.

## Change

- Removed the enemy `EnemyOrder` allocation and four-column insertion sort.
- `tabEnemy` now finds the current candidate and then selects the strict successor plus wrap candidate in linear passes using the unchanged `(near desc, tier, distance, id)` key.
- `friendlyCycle` no longer allocates and insertion-sorts temporary ID/distance arrays. It selects by `(distance squared, original candidate index)` so equal-distance traversal order and wrap behavior remain stable.
- Added Zr contract fixtures for enemy ID ties, near-only wrap, fallback-to-primary wrap, friendly equal-distance order, and final wrap.

## Evidence

Local source contract:

```text
python -m unittest tools.tests.test_runtime17_target_selection_performance_contract -v
4 tests passed
```

The deterministic 4,096-candidate complexity gate compares 8,386,560 adversarial insertion comparisons with at most 12,288 enemy-selection and 8,192 friendly-selection comparisons.

The standalone Node model compared the legacy and linear algorithms across 1,000 seeded random fixtures before measuring 2,048 adversarial candidates, two invocations per sample, 21 alternating sample pairs, nearest-rank percentiles:

| Route | Legacy P50 | Linear P50 | P50 delta | Legacy P95 | Linear P95 | P95 delta |
|---|---:|---:|---:|---:|---:|---:|
| enemy tab cycle | 90.5828 ms | 0.3021 ms | -99.666% | 137.1215 ms | 0.7706 ms | -99.438% |
| friendly cycle | 8.0821 ms | 0.0569 ms | -99.296% | 9.9621 ms | 0.2726 ms | -97.264% |

These timings are an algorithm model tied to source-shape contracts, not a claim about full WOC frame latency. Windows-native coordinator validation will separately compile and execute `woc_target_selection_tests.zrp` against pinned `zr_vm@60f6bcf4dd22bb6f5247e353bd0d97964758f157`; that result is pending at candidate creation time.

## Acceptance

- Required local P50 and P95 improvement: at least 35% for both routes.
- Required functional evidence: source contracts green, legacy/linear randomized parity green, and the real ZrVM target-selection package returns zero.
- Integration and WeCom publication remain coordinator-owned after the full batch succeeds and independent review is available.
