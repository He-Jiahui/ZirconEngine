---
title: Editor406 Borrowed Duplicate Import Dependency
category: zircon_editor
report_id: Editor406-borrowed-duplicate-import-dependency-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor406 Borrowed Duplicate Import Dependency

Editor UI import traversal now probes its dependency set with the normalized borrowed string and
allocates an owned key only when the dependency is absent. Fragment normalization, first-insert
ownership, sorted `BTreeSet` output, generation reset, and unresolved-reference tracking are
unchanged.

This removes an owned string allocation for every repeated dependency, including multiple fragment
aliases backed by one UI asset. Regression coverage inserts the same dependency twice and verifies
one retained key. The ignored Windows Release benchmark emits
`EDITOR406_BORROWED_DUPLICATE_IMPORT_DEPENDENCY_BENCH_V1` over 17 alternating paired samples and
65,536 insert attempts per sample with a 2,048-byte dependency. The legacy path creates 65,536
owned keys while the optimized path creates one. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.65` (at least 35% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor406 is prepared with Runtime476 under request
`runtime476-editor406-performance-batch-20260831ft-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
