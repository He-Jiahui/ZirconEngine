---
title: Runtime77 Pointer Reply Projection Capacity
category: zircon_runtime
report_id: Runtime77-pointer-reply-capacity-2026-09-14
date: 2026-09-14
session_id: root-runtime77-pointer-reply-capacity-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Pointer Reply Projection Capacity

## Scope

Pointer reply construction and pointer/text-result merging previously grew their output vectors
from zero capacity even when the route already exposed bounded upper bounds. This slice reserves
the fixed capture/focus effects plus the larger of invocation and root-dirty output counts, and
reserves each moved projection before a text result is merged. The merge also retains the text
result's widget-event projection, which was previously dropped. Effect ordering, dirty fallback,
index rebasing, and all other reply semantics remain unchanged.

## Implementation

- `pointer_reply_effects` now computes a saturating capacity hint from fixed effects and the
  invocation/root-target upper bound before emitting effects.
- `merge_pointer_text_result` reserves reply effects, applied/rejected effects, host requests,
  component events, widget events, binding reports, and diagnostics notes using the incoming result
  lengths, then moves every projection into the combined result.
- Added a lower Rust capacity-hint regression and ignored marker
  `RUNTIME77_POINTER_REPLY_CAPACITY_BENCH_V1`.

## Deterministic work model

The pointer path's common fixed/dynamic output vectors and the eight merge projections start at
their known lower/upper bounds rather than relying on geometric growth. The model removes the
known growth events; route traversal, dirty fallback, and effect-index rebasing are untouched.
This is allocation-shape evidence, not a product CPU/RSS or p50/p95/p99 claim.

## Validation

- RED/GREEN Python source contract:
  `tools/tests/test_runtime_ui_pointer_reply_capacity_performance_contract.py` (`5/5`).
- Lower Rust capacity-hint regression and ignored release marker are present; Cargo execution is
  deferred to the shared managed Runtime/Editor batch.
- Scoped Rustfmt and diff checks pass for the production and regression files.
- The combined Runtime/Editor performance-contract loader passes `1832/1832` in one batch;
  managed Windows Cargo/Release and product allocation/latency evidence remain pending because
  the external `E:\Git\zr_vm` checkout is still dirty. Coordinator status was not polled.
- The subsequent full non-tooling Runtime/Editor discovery passes `3581/3581` tests in one process
  (`441.896s`); this is batched local regression evidence, not managed product-performance proof.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/input/pointer_reply.rs` | `2FBD49E18C86507AF61FE7A64758C52C6E0E4415E4AD8855ABE988EC43EACB85` |
| `zircon_runtime/src/ui/surface/input/pointer_reply/capacity_tests.rs` | `A93C4C53FF3C8580B787073624E22FF877BC2CB3260EB24E4AD232772D1D9CBE` |
| `tools/tests/test_runtime_ui_pointer_reply_capacity_performance_contract.py` | `0FFCDAADC52699DF7A8211C0E5F9C098E82F9C92F6CD87008100BB0B8A3530` |

## Remaining parent work

Runtime77 still owns atomic effect commit, qualified input identity, host acknowledgement, queue
budgets, and product input-to-present evidence. This capacity slice does not close those gates.
