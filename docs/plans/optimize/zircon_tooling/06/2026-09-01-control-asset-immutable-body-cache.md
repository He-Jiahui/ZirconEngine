# Control asset immutable body cache

## Change

`StaticAssetService` now caches response bytes for content-hashed assets carrying
the existing `immutable` cache policy. `index.html` and non-hashed assets remain
uncached, so no-store navigation continues to observe rebuilt content. Headers
are still constructed per response.

## Performance evidence

Windows local benchmark, 1 MiB hashed asset, 64 requests, 15 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 73,068,600 ns | 1,040,600 ns | 98.576% lower, 70.22x |
| p95 | 93,176,700 ns | 2,728,900 ns | 97.071% lower, 34.14x |
| Peak Python allocation | 67,112,313 bytes | 1,049,969 bytes | 98.436% lower, 63.92x |
| File reads | 64 | 1 | 98.438% lower |
| Bytes read | 67,108,864 | 1,048,576 | 98.438% lower |

## Validation

Immutable-cache contracts and existing control-assets behavior passed 5/5 in
0.436 seconds, including no-store index refresh and traversal rejection.
