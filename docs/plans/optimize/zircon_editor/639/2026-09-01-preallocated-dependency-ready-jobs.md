---
title: Editor639 Preallocated Dependency-Ready Jobs
category: zircon_editor
report_id: Editor639-preallocated-dependency-ready-jobs-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor639 Preallocated Dependency-Ready Jobs

Dependency promotion now reserves the ready-job vector from the number of dependents indexed for the
completed dependency. Each indexed dependent can become ready at most once; missing queue entries are
still skipped with unchanged semantics.

Dependency ordering, waiting-count transitions, ready-set insertion, and fairness selection remain
unchanged. The regression exercises a full promotion batch and verifies that all promoted jobs fit the
initial capacity.

The ignored Windows Release benchmark emits
`EDITOR639_PREALLOCATED_DEPENDENCY_READY_JOBS_BENCH_V1` over 17 alternating sample pairs and 65,536
dependent projections. The gate requires preallocated P95 to be at most 85% of unreserved P95.

No direct Cargo validation was run. The coordinator owns aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, and WeCom outcome are recorded only after
coordinator completion.
