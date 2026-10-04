---
title: Runtime88 Watch Fold Stack Scratch
category: zircon_runtime
report_id: Runtime88-watch-fold-stack-scratch-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_work_reduction_met_dynamic_pending
---

# Runtime88 watch fold stack scratch

The bounded watch loop snapshots one or two touched URIs before applying each event so it can
restore the previous fold when a pending-entry or byte limit is exceeded. The old path built two
short `Vec`s for every event: touched URIs and prior entries. A fixed two-slot array now holds
each snapshot on the stack. It also avoids the second URI clone previously stored in each prior
entry. The fold operation, byte calculation, capacity decision, and rollback order stay the same,
including the `from == to` rename case.

| Evidence | Before | After / acceptance target |
| --- | ---: | ---: |
| Short scratch `Vec` allocations per watch event | 2 | 0 |
| Extra URI clones into the prior-entry snapshot | 1 or 2 | 0 |
| Entry/byte limits and rename rollback | Existing behavior | Legacy parity regression and explicit rollback regression |
| Release P95, same-URI modification burst | pending | At most 95% of legacy |

The ignored `RUNTIME88_WATCH_FOLD_STACK_SCRATCH_BENCH_V1` benchmark compares the original
bounded fold to the stack-scratch fold over 16,384 repeated modifications per sample, with 17
alternating sample pairs and nearest-rank P95. It measures the bounded in-memory watch fold only;
it does not claim notify delivery or end-to-end asset visibility. The grouped managed Runtime
test and Release benchmark results remain pending. This slice does not close the wider Runtime88
event-source work.
