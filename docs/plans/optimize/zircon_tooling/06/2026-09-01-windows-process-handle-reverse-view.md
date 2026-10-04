# Windows process handle reverse views

## Change

Windows process-tree termination now iterates dictionary item/value views in
reverse directly. The previous termination, retry, and final-close paths copied
the full handle registry into temporary tuples before reversing it. Modern
Python dictionary views are reversible and preserve the same reverse insertion
order, so descendant-first termination and close ordering remain unchanged.

## Performance evidence

Controlled Windows Python benchmark with 100,000 retained handles and the three
reverse traversals used by termination/cleanup, 31 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 46,528,700 ns | 24,509,000 ns | 47.325% lower, 1.90x |
| p95 | 80,019,900 ns | 33,365,100 ns | 58.304% lower, 2.40x |
| Peak Python allocation | 7,873,016 bytes | 112 bytes | 99.999% lower, 70,294.79x |
| Handle-view tuple copies | 3 | 0 | eliminated |

## Validation

The reverse-view contract and complete process-liveness module passed 13/13 in
8.986 seconds, including retained-handle PID identity, suspend races, real tree
termination, and kill-on-close behavior. Broader coordinator validation remains
delegated in the accumulated asynchronous batch.
