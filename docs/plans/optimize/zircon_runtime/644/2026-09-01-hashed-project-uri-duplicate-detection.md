---
title: Runtime644 Hashed Project URI Duplicate Detection
category: zircon_runtime
report_id: Runtime644-hashed-project-uri-duplicate-detection-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime644 Hashed Project URI Duplicate Detection

Project import-source validation now uses an input-sized hash index to detect duplicate project
asset URIs instead of an ordered tree. Package URIs remain excluded, the scan still follows source
input order, and the first repeated URI still reports the same previous and current paths. No
ordered output is produced from the temporary index, so tree ordering was unnecessary.

The ignored Windows Release benchmark emits
`RUNTIME644_HASHED_PROJECT_URI_DUPLICATE_BENCH_V1` over 17 alternating sample pairs with 32,768
unique project URIs. It verifies identical duplicate outcomes, then compares ordered-tree insertion
with preallocated hash insertion. The gate requires hashed P95 to be at most 80% of ordered-tree
P95.

No direct Cargo validation was run. The coordinator owns combined Runtime644/Editor644 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime644 is prepared with Editor644 under the shared `optimization_batch_je_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
