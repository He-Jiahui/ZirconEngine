---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/77/2026-09-14-pointer-reply-capacity.md
related_code:
  - zircon_runtime/src/ui/surface/input/pointer_reply.rs
  - zircon_runtime/src/ui/surface/input/pointer_reply/capacity_tests.rs
tests:
  - zircon_runtime/src/ui/surface/input/pointer_reply/capacity_tests.rs
  - tools/tests/test_runtime_ui_pointer_reply_capacity_performance_contract.py
---

# Runtime77 Pointer Reply Projection Capacity

Pointer reply construction now reserves the bounded fixed/dynamic effect upper bound, and
pointer/text-result merging reserves and moves every projection, including `widget_events`, before
extending it. Existing route ordering, dirty fallback, effect-index rebasing, and failure behavior
are unchanged; the previously dropped text widget event is now retained.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime77 / pointer reply projection | Reserve fixed + invocation/root effect capacity and all merged result vectors, preserving text widget events | implemented_pending_validation | RED/GREEN source contract `5/5`; lower Rust capacity-hint and widget-event merge regression plus ignored `RUNTIME77_POINTER_REPLY_CAPACITY_BENCH_V1` marker added; scoped Rustfmt/diff checks pass. The current batched non-tooling Runtime/Editor discovery passes `3581/3581` in `441.896s`; managed Cargo and product allocation/latency evidence remain pending. |

## Complexity boundary

The existing single invocation scan, fallback condition, output ordering, and merge index rebasing
remain unchanged. Only initial vector capacity changes; no product CPU/RSS/p50/p95/p99 claim is
made before the managed Release batch.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/input/pointer_reply.rs` | `2FBD49E18C86507AF61FE7A64758C52C6E0E4415E4AD8855ABE988EC43EACB85` |
| `zircon_runtime/src/ui/surface/input/pointer_reply/capacity_tests.rs` | `A93C4C53FF3C8580B787073624E22FF877BC2CB3260EB24E4AD232772D1D9CBE` |
| `tools/tests/test_runtime_ui_pointer_reply_capacity_performance_contract.py` | `0FFCDAADC52699DF7A8211C0E5F9C098E82F9C92F6CD87008100BB0B8A3530` |

## Managed gate

Cargo was not started locally. The shared managed admission remains blocked by the external dirty
`E:\Git\zr_vm` checkout; this slice joins the next batched Runtime/Editor validation. Coordinator
status was not polled, and no product performance claim is made here.
