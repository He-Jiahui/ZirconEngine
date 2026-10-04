# Codex history rollout scan projection

## Change

Codex historical evidence discovery now calls `rglob()` directly without a
separate month-root `exists()` probe and feeds matching rollout paths directly
into `sorted()`. Missing roots and enumeration failures still produce an empty
current-month candidate list, archived incomplete cursors are still appended,
and resolved-path deduplication and file limits are unchanged.

## Performance evidence

Controlled Windows Python benchmark sorting and limiting 100,000 path objects
over 21 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 145,040,300 ns | 133,538,800 ns | 7.930% lower, 1.09x |
| p95 | 317,345,900 ns | 160,904,400 ns | 49.297% lower, 1.97x |
| Missing month-root pre-probes | 1 | 0 | eliminated |

Peak Python allocation was effectively flat (6,301,110 vs 6,301,310 bytes), so
no memory improvement is claimed.

## Validation

The scan contracts plus normal rollout streaming and archived incomplete-cursor
resume behaviors passed 4/4 in 16.704 seconds. Broader coordinator validation
remains delegated in the accumulated asynchronous batch.
