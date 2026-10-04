---
title: Runtime603 State Hook Single Buffer
category: zircon_runtime
report_id: Runtime603-state-hook-single-buffer-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime603 State Hook Single Buffer

State-transition dispatch now borrows the matching exit, transition, and enter hook buckets, sums
their exact lengths, and freezes them into one ordered `Vec<StateHook<T>>`. The previous path cloned
each bucket into a separate vector before the three vectors were immediately consumed. Callback
order remains exit, transition, enter, registration order within each bucket is unchanged, and the
frozen callbacks still run outside the runtime state lock.

A focused behavior test registers two hooks in each category and verifies the complete six-callback
order. A source guard requires one ordered dispatch buffer and rejects the previous three-vector
layout. The ignored Windows Release benchmark emits
`RUNTIME603_STATE_HOOK_SINGLE_BUFFER_BENCH_V1` over 17 alternating sample pairs and 16,384
transitions per sample with one matching hook in each category. The gate requires single-buffer P95
to be at most 75% of the legacy three-buffer P95; modeled heap allocations per transition fall from
three to one, while an empty dispatch retains zero-capacity allocation behavior.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime603 is prepared with Editor603 under request
`runtime603-editor603-hook-capability-performance-20260901hu-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
