# Runtime354 Single-Pass Button State Evaluation

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime354-editor299-performance-batch-20260829ca-v1`

## Scope

Input action evaluation previously traversed every binding button separately for pressed,
just-pressed, and just-released state. Those three state traversals now share one loop while the
consumed-input rejection pass remains independently first.

## Static Evidence

- Button-state traversals per non-consumed binding: `3 -> 1`.
- Total button traversals including consumed-input rejection: `4 -> 2`.
- Empty bindings, per-state short circuiting, axis routing, and action transition semantics remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME354_BUTTON_STATE_SINGLE_PASS_BENCH_V1`. It
compares three 32-button state traversals with the fused traversal over 100,000 checks per sample and
31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
