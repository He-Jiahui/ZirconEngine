# Runtime51 Lazy-Registration Streaming Batch Construction

- Date: 2026-08-24
- Session: `optimize-runtime51-query-merge-heap-r1-20260824`
- Related plan: Runtime51 M6 incremental update direction
- Status: candidate implemented; grouped managed validation pending

## Scope

`ResourceManager::register_lazy_records` previously collected every input
`ResourceRecord` into an intermediate `Vec`, traversed that vector to collect IDs, and then
traversed it again to build `ResourceMutationBatch`. It now consumes the input iterator once,
collecting only the IDs required to return handles while building the batch in the same loop.

The lower bound from `Iterator::size_hint` preallocates the returned-ID vector. Input order,
batch ordering, commit behavior, and returned handle ordering remain unchanged.

## Deterministic Work Reduction

For a batch of `N` lazy records, the intermediate `Vec<ResourceRecord>` and its second traversal
are removed. The mutation batch and output ID vector are still required, and the batch may retain
its own growth behavior; this record does not claim that all allocation has been eliminated.

This is not a p95, allocation-rate, RSS, VRAM, fragmentation, or product-performance claim. Those
metrics require current-source managed workload measurements.

## Validation

Focused contracts cover input-handle order and reject reintroducing complete input-record buffering
before batch construction. Scoped `rustfmt --check` passed. Scoped `git diff --check` reported only
the repository's CRLF normalization warning.

R9 is intentionally held for a grouped `zircon_runtime` library validation submission with another
independent resource-management candidate; Cargo is not run locally.
