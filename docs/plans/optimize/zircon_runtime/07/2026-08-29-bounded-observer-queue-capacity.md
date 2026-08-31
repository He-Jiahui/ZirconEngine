---
title: Runtime07 Bounded Observer Queue Capacity
category: zircon_runtime
report_id: Runtime07-bounded-observer-queue-capacity-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Bounded Observer Queue Capacity

## Scope

This slice reduces allocation growth while staging terminal observers on a native plugin
discovery refresh ticket. The ticket already enforces a per-request observer budget, but the
staging vector previously grew geometrically even when a caller filled that known budget.

## Change

- Keep the observer vector lazy so tickets with no observer allocate no queue storage.
- Preserve the default small-vector growth path until a non-empty queue reaches capacity.
- At that boundary, reserve exactly the remaining configured observer budget before the next
  queued callback.
- Preserve bounded admission, terminal-state shallow clones, once-only delivery, lock-free
  callback execution, and callback panic accounting.
- Add a Rust regression for empty/small fan-out and the capacity jump to a 32-observer budget.
- Add a Python source performance contract for the bounded reserve placement and behavior.

## Deterministic Performance Evidence

The standalone optimized Rust model builds 131,072 saturated observer queues per sample across
31 alternating samples. Each queue contains 32 two-word slots, matching the size of one boxed
trait-object observer handle while isolating vector storage growth. A second 4-observer profile
proves that the small fan-out path retains the legacy allocation count and byte volume. Both
implementations produced checksum `8353127594834719136`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 65,536 saturated queues | 262,144 | 131,072 | 50.000% |
| Requested allocation bytes | 62,914,560 | 37,748,736 | 40.000% |
| Four-observer allocation calls | 65,536 | 65,536 | 0.000% |
| Four-observer requested bytes | 4,194,304 | 4,194,304 | 0.000% |
| Run 1 saturated P50 | 88.9211 ms | 51.4689 ms | 42.118% |
| Run 1 saturated P95 | 110.5318 ms | 69.0518 ms | 37.528% |
| Run 2 saturated P50 | 87.2016 ms | 47.8846 ms | 45.087% |
| Run 2 saturated P95 | 134.5306 ms | 79.5604 ms | 40.861% |

Evidence marker: `RUNTIME07_BOUNDED_OBSERVER_QUEUE_CAPACITY_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_bounded_observer_queue_capacity_performance_contract.py`:
  4 passed after the pre-change contract rejected the missing bounded-reservation helper.
- `python -m py_compile` passed for the source contract.
- The standalone Rust model retained identical saturated and small-queue output; two runs kept
  the same allocation profiles and positive P50/P95 reductions.
- The Rust regression locks lazy empty storage and the full-queue jump to the configured budget.
- Exact-file Rust formatting and scoped diff checks are required before snapshot publication.
- Managed Rust compilation and tests remain pending in the next asynchronous Runtime07 batch.

Managed batch request: `runtime07-plugin-five-task-batch-20260830-v1`.

Validation attempt: ticket `27e27a159794475b9bd8636cf2859288` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
