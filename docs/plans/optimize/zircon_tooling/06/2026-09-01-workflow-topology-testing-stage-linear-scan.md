# Workflow topology testing-stage linear scan

## Change

Fallback topology parsing now identifies nested testing-stage headings with one
forward heading-stack pass. Previously every testing-stage candidate scanned all
headings in reverse to find its parent. Reparenting by ordinary headings and
genuine duplicate milestone diagnostics remain unchanged.

## Performance evidence

Windows Python benchmark, 200 milestones and 601 headings, 25 rounds of 20 parses:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 209,758,900 ns | 23,636,200 ns | 88.732% lower, 8.87x |
| p95 | 273,431,800 ns | 40,185,000 ns | 85.303% lower, 6.80x |
| Complexity | O(candidates * headings) | O(headings) | linearized |

The first oversized 500-iteration benchmark exceeded 120 seconds and was
discarded; it is not used as evidence.

## Validation

Testing-stage behavior and all focused topology performance contracts passed 9/9
in 9.203 seconds.
