---
title: Runtime03 Static Diagnostic Metadata Cardinality Fast Path
category: zircon_runtime
report_id: Runtime03-static-diagnostic-metadata-cardinality-2026-09-09
date: 2026-09-09
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime03 Static Diagnostic Metadata Cardinality Fast Path

## Scope

This slice reduces repeated metadata comparison work on the existing `DiagnosticStore::record_static`
path. It preserves normalized lexical tag storage, unordered tag equivalence, duplicate-tag
compatibility, unit matching, history, smoothing, and snapshot behavior. It does not introduce a
new diagnostics schema, cache identity, generation snapshot, cardinality policy, or product
profiling claim.

## Implementation

`DiagnosticSeries::metadata_matches` now recognizes the common case where the incoming tag slice
has the same cardinality as its already-normalized unique tag set. In that case it verifies set
equivalence with one membership pass over retained tags, avoiding the retired preliminary
prefix-scan used only to count unique input tags.

The original compatibility branch remains when lengths differ. It continues to accept an input
such as `["time", "frame", "time"]` for an existing `{frame,time}` set and rejects unknown tags,
so dynamic or repeated tag callers retain their former semantics.

## Deterministic work model

For the common four-unique-tag case, the retired route performs six prefix duplicate comparisons
plus up to sixteen retained-membership comparisons: 22 modeled comparisons per metadata check.
The equal-cardinality route performs at most sixteen membership comparisons, eliminating six
modeled comparisons (2,727 basis points, 27.27%). This is an operation-count bound, not a CPU,
allocation, RSS, frame-time, or product-latency measurement.

## Validation

- The Runtime03 source regression locks the cardinality branch and validates reordered unique tags,
  duplicate-compatible input, and unknown-tag rejection.
- One combined Editor10/Runtime03/Runtime438 source-contract batch passed `21/21`; Python
  compilation passed for all six invoked contract modules.
- Rustfmt passed for the Runtime03 store, both Editor10 owners, Runtime input owners, and Runtime
  table owners. Scoped `git diff --check` passed with only normal CRLF conversion warnings.
- Managed Cargo, Rust unit execution, and release CPU/allocation/RSS/frame p50/p95/p99 remain
  pending because the external `E:/Git/zr_vm` validation checkout is dirty. No coordinator status
  was queried or waited on.

## Remaining parent-plan work

Diagnostic metadata and series cardinality are still unbounded; store snapshots still clone
history under the diagnostics lock. Paged/generation snapshots, schema admission, real
CPU/allocation profiling, and product-scale validation remain owned by the Runtime03 parent plan.

## Current source fingerprint

- `zircon_runtime/src/core/runtime/diagnostics/store.rs`: `254E2E77927D00C2F734DCF626BAEE4604A2B35EEDBA328B52F08D545BA3F4FB`
