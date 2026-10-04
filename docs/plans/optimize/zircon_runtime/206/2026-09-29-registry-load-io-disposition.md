---
title: Runtime206 registry load I/O disposition
category: zircon_runtime
report_id: Runtime206-registry-load-io-disposition-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
implementation_status: implemented_pending_validation
validation_status: static_checks_complete_managed_tests_pending
performance_status: measurement_pending
related_code:
  - zircon_runtime/src/asset/registry/persistence.rs
  - zircon_runtime/src/asset/registry/persistence/load_disposition_tests.rs
tests:
  - unreadable_registry_does_not_rebuild_from_project_sources
  - missing_registry_still_rebuilds_from_project_sources
plan_sources:
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
---

# Runtime206 registry load I/O disposition

## Change

`ASSETREG-P1-015` records that registry read errors can enter the same rebuild
path as corrupt serialized data. `load_or_rebuild` now attempts one load. A
missing registry file still causes a rebuild; any other I/O error is returned
to the caller before a source scan or registry write. This includes permission
errors, directories in place of the registry file, and a read failure after
the file was observed to exist. Decode and unsupported-version handling retain
their existing fallback behavior in this bounded change.

Removing the separate `Path::exists` probe removes one metadata operation from
normal opens and removes the gap between that probe and the file read. No
Release latency, allocation, or filesystem-call-count result has been measured,
so this is not a quantified performance pass.

The regression places a directory at the formal registry path and supplies an
invalid asset root. It requires the returned I/O error to name the registry
path; the old implementation instead enters a source scan and reports the
invalid asset root. This exercises a non-`NotFound` read error; it is not a
portable permission-denied regular-file fixture. A second test keeps the
missing-file rebuild contract.
The existing corrupt-JSON recovery tests remain in the separate registry test
module and are included in the grouped Runtime library test batch.

## Evidence and limits

| Path | Preimage SHA-256 | Candidate SHA-256 |
|---|---|---|
| `zircon_runtime/src/asset/registry/persistence.rs` | `b7fe2072595b045eb393c85f35e5474ba8789846129fdaec4a5aa7cc20bb6967` | `d8e7fb858ef80dae713c2da17dd4930c06afc92c262b03c5b783909b9a4c08d5` |
| `zircon_runtime/src/asset/registry/persistence/load_disposition_tests.rs` | absent | `ede1a0b7d768c7c6ea19cbd444eae0e8c713bdb1aabb2602c18359f52451548b` |

The preimage already had an unrelated import-order formatting diff from an
archived Session. That diff was preserved. The four exact source, test, and
record paths were claimed under request
`a5bed09511d6454f8012746c268bf5ea`; no foreign lease conflicted.
Pinned Rustfmt 1.94.1 and the tracked source diff check passed. The focused
tests and package compile have not run and remain in the next grouped managed
Windows validation batch.

This closes only the unsafe I/O-to-rebuild branch of `ASSETREG-P1-015` as a
candidate. Corrupt/unsupported-version migration and quarantine policy,
durable recovery receipts, and valid-but-stale registry detection remain under
`ASSETREG-P1-011`, `ASSETREG-P1-016`, `ASSETREG-P1-017`, and
`ASSETREG-P1-018`. Performance acceptance requires a measured Release open
profile and a declared target; neither exists for this slice yet.
