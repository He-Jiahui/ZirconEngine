# Runtime317 Early UI Graph Order Resolution

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime317-editor262-performance-batch-20260829aq-v1`

## Scope

`runtime_ui_graph_pass_order` only needs the first positions of `uber`, `runtime-ui`, and
`overlay-gizmo`, but previously scanned the complete executed-pass list after all three were known.
The resolver now stops immediately when the three first matches are complete. The classification
and duplicate-pass semantics are unchanged.

## Static Evidence

- A 1,024-pass list with the three targets first changes visits per resolution from `1024 -> 3`.
- Additional allocations and retained state: `0`.
- Existing missing-target and zero-execution behavior remains unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME317_EARLY_UI_GRAPH_ORDER_RESOLUTION_BENCH_V1`.
It performs 5,000 resolutions per sample across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
