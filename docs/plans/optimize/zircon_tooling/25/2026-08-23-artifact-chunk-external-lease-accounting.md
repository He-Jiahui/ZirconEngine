---
title: Artifact chunk cache external lease accounting
date: 2026-08-23
plan: docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md
status: candidate_validation_pending
scope: artifact chunk cache-owned and externally leased compressed payload diagnostics
---

# Artifact Chunk External Lease Accounting

## Change

`ArtifactChunkResidencyDiagnostics.resident_*` continues to describe only payloads owned by the
LRU cache and remains bounded by `max_resident_bytes`. The residency state records a `Weak`
reference only when an evicted or non-admitted chunk has a caller-owned `Arc`; cache-only eviction
does not allocate retired-lease metadata. Diagnostics expose:

- `externally_leased_*`: payloads with at least one consumer `Arc`; these bytes can overlap a
  cache-owned payload while both references exist.
- `tracked_payload_*`: the de-duplicated union of cache-owned payloads and evicted payloads that
  remain live through an external lease.

Released weak references are removed while collecting diagnostics. The tracker indexes them by
payload address, so recording an eviction updates one `HashMap` entry rather than scanning all
live external leases; an expired entry at a reused address is replaced in that same entry. Its
metadata has a fixed 1,024-entry bound, derived from the default 64 MiB cache budget and 64 KiB
chunk size. At that bound it replaces one tracked record in constant time and increments
`external_lease_tracking_overflows` only when the displaced record was still live. A nonzero
overflow counter makes the external-lease and tracked-payload totals explicit lower bounds. The
tracker never retains a strong reference, so it cannot itself keep an evicted payload alive.

Budget eviction now uses a lazy min-index instead of scanning every resident `HashMap` entry to
find the least-recently-used chunk. A cache hit or insertion pushes one `(access, key)` candidate;
stale candidates are ignored when popping the oldest live key. The index rebuilds after it exceeds
two candidates per resident entry, so its retained metadata is bounded by `2 * resident_entries`.
The resident map and eviction index share an `Arc` cache-key allocation, so a cache hit records an
atomic key reference instead of deep-copying the chunk path and content-hash text into the index.

`ArtifactStore::trim_chunk_residency()` is an explicit idle or memory-pressure maintenance action.
It removes cache-owned chunks and replaces the cache map and lazy-index containers, returning the
logical released chunk and byte counts. Caller-held chunks remain live through their `Arc` and are
visible as external leases; the report does not claim an immediate process-RSS reduction.

The follow-on hot-hit path keeps the resident entry's canonical `Arc<ArtifactChunkCacheKey>` beside
the payload. A valid cache hit now performs one `HashMap` lookup, updates that entry, and clones its
canonical key for the lazy eviction index; it no longer probes the map once to fetch the key and a
second time to mutate the entry. The extra retained metadata is one `Arc` pointer per resident entry
(at most 1,024 entries for the default 64 MiB / 64 KiB budget, or approximately 8 KiB on 64-bit
targets), with no additional key allocation or path/content-hash deep copy.

## Regression Contract

The lazy-residency test reads one chunk twice, retains both consumer `Arc` references, then reads
a second chunk under a one-chunk cache budget. The first chunk must be evicted from cache-owned
residency but remain visible as one external lease. After both consumer references are dropped,
the external lease counters must return to zero and tracked payload bytes must equal cache-owned
bytes again. A second regression uses a one-byte cache budget so the requested chunk is returned
without cache admission; it must remain tracked while its consumer `Arc` is alive and disappear
after that `Arc` is dropped. The trim regression keeps a returned chunk alive, trims cache
ownership, and verifies that cache-owned bytes reach zero while the consumer lease remains visible
until it is dropped.

## Validation

The initial managed Windows `zircon_runtime` library regression batch is queued for its sealed
snapshot. The canonical-key follow-on is part of the next batch with the command-arena merge fast
path below; no local Cargo command was run and neither snapshot is treated as integrated before its
coordinator result is available.

## Performance Data

No RSS, allocator, frame-time, throughput, or tail-latency measurement is produced by this
change. The quantitative result under validation is diagnostic correctness: a payload retained by
external `Arc` leases is no longer reported as released solely because its cache entry was evicted;
retired-lease lookup and bounded metadata replacement are expected O(1) rather than a linear
live-lease scan. Cache-only eviction performs zero retired-lease map/slot insertions, and explicit
trim reports the logical cache-owned chunks and bytes released. The fixed tracker metadata bound
is 1,024 records; no process RSS, allocator, frame-time, throughput, or tail-latency measurement
is claimed.

For budget-driven artifact eviction, the previous full-cache oldest-entry scan is replaced by an
amortized `O(log resident_entries)` lazy-heap operation with stale-candidate skipping; lazy-index
rebuild is `O(resident_entries)` only after its bounded `2 * resident_entries` candidate
threshold. Each candidate reuses the resident key's `Arc`, avoiding a path/string deep-copy on a
cache hit. These are algorithmic and allocation-shape results, not CPU-time or throughput
measurements.

The follow-on cache-hit change removes one expected `HashMap` probe per valid hot hit. It has no
product RSS, allocator, frame-time, throughput, p95, or p99 measurement yet, so it is not evidence
that a product performance target has been met.

The subsequent inventory-identity change removes the remaining temporary key allocations from a
valid chunk-cache hit. Chunk roots are shared as `Arc<PathBuf>` per inventory and content hashes as
`Arc<str>` per descriptor; `read()` builds its stack-local lookup key with two `Arc` clones rather
than cloning the nonempty path and 64-character hash into two new heap allocations. The artifact
writer creates that `Arc<str>` directly from BLAKE3 hex and transfers it to the descriptor, so the
write path does not create an intermediate `String` solely for descriptor ownership. A valid hot
hit therefore has zero key-related heap allocations instead of two, while paying two atomic
reference-count operations. The manifest still serializes its hash as a string. This is allocation
shape evidence only: no product RSS, CPU-time, throughput, frame-time, p95, or p99 measurement is
claimed. The managed regression batch remains required for functional and serialization coverage.

`ChunkReader` already owns the `ArtifactChunkInventory` whose artifact content hash it validates at
EOF. It no longer clones that hash into a second `String` during reader construction; completion
compares against `self.inventory.content_hash()` directly. This removes one manifest-hash string
allocation and copy per reader initialization without changing the expected digest, reader
lifetime, or streamed content verification. It is an allocation-count result only, not a measured
CPU-time, throughput, frame-time, p95, p99, allocator, or RSS claim.
