---
title: Editor612 Reset Delta Direct Materialization
category: zircon_editor
report_id: Editor612-reset-delta-direct-materialization-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor612 Reset Delta Direct Materialization

`DirtyRegistry::changes_since` now skips incremental external-change materialization when the
cursor requires a reset. The previous reset path copied every current document into
`external_changed`, copied the same documents again into `external_present`, discarded both sets
after transaction reset capture, and then cloned the stable document registry a third time for the
actual reset snapshot.

Reset now starts with empty incremental change, present, and removed collections. It still captures
the transaction reset before reacquiring the registry lock and cloning the final stable document
set once, so generation retry behavior and atomic snapshot semantics are unchanged. A reset never
reported removed documents before this change because its intermediate change set contained only
currently registered documents. Incremental replay continues to use the existing ordered journal
deduplication and single-pass present/removed partition.

The ignored Windows Release harness emits
`EDITOR612_RESET_DELTA_DIRECT_MATERIALIZATION_BENCH_V2` over 31 alternating sample pairs, 4,096
documents, and 128 iterations. It reports nearest-rank p50/p95/p99 and compares two discarded
`BTreeSet` copies with the direct empty reset seed. This is a helper microbenchmark, not a full
`DirtyRegistry::changes_since` product-call measurement.

The known Editor caller, `EditorManagerLayout::dirty_document_toolkits`, is not invoked by this
harness. Managed Windows Release evidence through that real caller, including snapshot projection
and its allocation behavior, remains pending; the helper microbenchmark is not product acceptance.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regressions, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor612 is prepared with Runtime612 under request
`runtime612-editor612-ordered-level-reset-delta-performance-20260901ic-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
