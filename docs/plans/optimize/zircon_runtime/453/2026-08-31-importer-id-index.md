---
title: Runtime453 Importer ID Index
category: zircon_runtime
report_id: Runtime453-importer-id-index-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime453 Importer ID Index

Plugin package contribution registration now snapshots registered asset importer IDs once and uses a
`HashSet` for package-row admission. The former loop rebuilt `AssetImporterRegistry::descriptors`
for every package importer, cloning every registered descriptor before performing a linear ID scan.
That made the admission phase quadratic and allocation-heavy as plugin catalogs grew.
Packages without importer rows still return before taking a registry snapshot.

Duplicate descriptors are still validated before being ignored. A new ID enters the index only
after registry admission succeeds, so an invalid descriptor cannot shadow a later valid row with
the same ID. Regression coverage exercises an existing duplicate, an invalid-then-valid retry, and
a duplicate introduced within the same package.

The ignored Windows Release benchmark emits `RUNTIME453_IMPORTER_ID_INDEX_BENCH_V1` over 17
alternating paired samples with 512 registered importers and 512 probes. The legacy path clones and
scans the descriptor catalog for every probe; the optimized path builds one ID index and performs
constant-time membership probes. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.20` (at least 80% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime453 is prepared with Editor383 under request
`runtime453-editor383-performance-batch-20260831eu-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
