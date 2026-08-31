# Runtime51 Single-Resource Readiness Refresh Fast Path

- Date: 2026-08-24
- Session: `optimize-runtime51-query-merge-heap-r1-20260824`
- Related plan: Runtime51 M6 incremental update direction
- Status: candidate implemented; grouped managed validation pending

## Scope

`ResourceAuthority::refresh_readiness_many` is called by single-resource acquire/release and
single-record commit paths. It previously collected every input, including a one-element input,
into an ID `Vec`, sorted and deduplicated it, then materialized a second updates `Vec` before
calling the generic readiness projection API.

The new path returns for no IDs, directly passes one `ResourceReadinessSourceUpdate` to the
existing generic projection API for one ID, and keeps the previous sort/dedup/update-vector
behavior for two or more IDs. Therefore duplicate handling and multi-record ordering remain
unchanged.

## Deterministic Work Reduction

For a one-ID refresh, the fast path removes the ID `Vec`, sort/dedup work, and updates `Vec`.
The readiness projection still owns its normal source comparison, dependency-closure, and
publication work; unchanged source data continues to retain the exact generation.

This is not a p95, allocation-rate, RSS, VRAM, fragmentation, or product-performance claim.
Those values require current-source managed workload evidence.

## Validation

Focused contracts verify initial single-source publication, exact generation reuse when unchanged,
and the absence of the former buffered-and-sorted single-ID setup. Scoped `rustfmt --check`
passed. Scoped `git diff --check` reported only the repository's CRLF normalization warning.

R10 will be submitted with R9 as one `zircon_runtime` library validation batch; Cargo is not run
locally.
