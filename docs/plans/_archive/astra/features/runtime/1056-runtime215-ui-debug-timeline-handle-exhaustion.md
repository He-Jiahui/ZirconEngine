---
title: Runtime215 UI debug timeline handle exhaustion
category: zircon_runtime
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/215/2026-09-29-ui-debug-timeline-handle-exhaustion.md
implementation_status: completion_candidate_validation_pending
validation_status: static_checks_passed_managed_runtime_validation_pending
regression_status: stronger_full_retention_applied_managed_validation_pending
---

# Runtime215 UI debug timeline handle exhaustion

UI debug timeline frame handles now use checked progression. `u64::MAX` can be
issued once; a later capture fails before modifying the retained history. The
stronger applied regression fills the capacity-two history with
`u64::MAX - 1` and `u64::MAX`, selects the earlier handle, and then compares
the complete timeline snapshot after an exhausted capture. That case checks that failure preserves both a full retention window
and a non-latest selection.

`rustfmt --edition 2021 --check` and scoped `git diff --check` passed. The
stronger regression is applied; managed Runtime execution remains pending,
so no test pass is claimed.
