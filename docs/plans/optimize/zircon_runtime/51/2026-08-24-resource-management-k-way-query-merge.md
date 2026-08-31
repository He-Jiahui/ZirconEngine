# Runtime51 Resource-Management K-Way Query Merge

- Date: 2026-08-24
- Session: `optimize-runtime51-query-merge-heap-r1-20260824`
- Related plan: Runtime51 M6 query direction and `PERF-MVP-556`
- Status: candidate implemented; managed batch validation and product measurements pending

## Scope

`ResourceManagementGeneration` exposes a stable-order scan and page over 64 immutable,
sorted resource shards. Previously each emitted result rescanned every shard to find the
smallest locator/id candidate. The new merge cursor primes one matching candidate per shard
and, after each pop, advances only the source shard. The scan keeps its generation `Arc` for
its full lifetime; page output, query filtering, locator/id order, and offset/limit semantics
remain unchanged.

## Deterministic Work Reduction

The profiling fixture has two matching model rows and one filtered texture row. Its legacy
incremental scan made three full 64-shard selection passes, recording 192 candidate checks.
The legacy page materialized two rows and recorded 128 checks. The merge cursor records 64
initial shard advances plus one advance for each of the two emitted source shards: 66 checks
for both operations. Filtered-row and output counts remain unchanged.

This is an exact work-count change, not a latency, allocation, RSS, VRAM, or fragmentation
claim. The cursor deliberately retains an `O(64)` binary heap and shared row `Arc`s; the
managed validator must establish that this tradeoff improves representative workloads.

## Validation

The updated profiling assertions are the RED-to-GREEN contract for the 192/128 to 66 work
reduction. Scoped `rustfmt --check` passed and scoped `git diff --check` reported only the
repository's CRLF normalization warnings. Cargo compilation, behavior tests, and any product
p95/RSS evidence are intentionally deferred to a managed multi-task validation batch; this
record does not claim that Runtime51 M6/M8 or the product performance target is complete.
