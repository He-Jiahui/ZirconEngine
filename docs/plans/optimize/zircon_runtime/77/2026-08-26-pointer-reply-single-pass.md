---
title: Runtime77 Pointer Reply Single Pass
category: zircon_runtime
report_id: Runtime77-pointer-reply-single-pass-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Pointer Reply Single Pass

## Scope

This slice removes the second full invocation scan from pointer-reply construction. Handler and
redraw phase selection, dispatch disposition, effect ordering, dirty fallback, component-event
fallback, capture, focus, and release behavior remain unchanged.

## Change

- Track the last handler phase and last dirty/damage phase while the existing invocation loop emits
  dirty-redraw effects.
- Select the reply phase from the retained scan result instead of reverse-scanning the invocation
  list after effects have already been built.
- Keep capture/release/focus effects before per-invocation dirty effects and preserve the original
  target/root dirty fallback after them.

## Deterministic Performance Evidence

| 16,384-invocation route, 128 replies per sample | Before | After |
|---|---:|---:|
| Full invocation passes per reply | 2 | 1 |
| Invocation visits per sample | 4,194,304 | 2,097,152 |
| Temporary phase indexes/maps | 0 | 0 |
| Reply/effect semantic changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME77_POINTER_REPLY_SINGLE_PASS_BENCH_V1`. Acceptance requires single-pass reply construction
P95 to be at least 20% below the legacy double scan. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ar_pointer_reply_preserves_phase_and_dirty_effects` covers handler
  phase priority, dirty effect emission, and last dirty/damage phase selection.
- `optimization_batch_20260826ar_pointer_reply_scans_invocations_once` requires one invocation
  loop and rejects the legacy reverse phase scan.
- `optimization_batch_20260826ar_pointer_reply_single_pass_p95` reports paired P50/P95 samples and
  enforces the 20% P95 reduction gate.

## Remaining Parent-plan Work

Runtime77 still owns navigation indexing, reusable route scratch, qualified pointer identity,
transactional effects, queue backpressure, device coverage, and product-scale performance
receipts. This slice only converges pointer-reply invocation traversal.
