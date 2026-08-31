---
title: Runtime431 Borrowed Project Locator Index
category: zircon_runtime
report_id: Runtime431-borrowed-project-locator-index-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime431 Borrowed Project Locator Index

Project resource synchronization now indexes the active project's primary locators by reference
while comparing them with the previous owned locator set. It no longer clones every current
`ResourceLocator`, including its path and optional label strings, into a temporary hash set.

Removal behavior is unchanged: only locators absent from the current project are cloned into the
owned resource mutation batch. Regression tests verify pointer identity for the borrowed index and
the removed-locator membership contract.

The ignored Windows Release benchmark emits `RUNTIME431_BORROWED_PROJECT_LOCATOR_INDEX_BENCH_V1`
over 17 alternating paired samples, each building 128 indexes for 1,024 long project locators,
requiring `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime431 is prepared with Editor359 under request
`runtime431-editor359-performance-batch-20260831dw-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
