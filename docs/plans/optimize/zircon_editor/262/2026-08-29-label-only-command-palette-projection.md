# Editor262 Label-Only Command Palette Projection

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime317-editor262-performance-batch-20260829aq-v1`

## Scope

The command-palette label API previously delegated to the combined projection, which built labels
and full structured rows before discarding every structured row. The label entrypoint now validates
the component role, parses the same filtered command entries, and moves only their labels. The
structured-only and combined projection contracts are unchanged.

## Static Evidence

- Structured rows built for a 1,024-command label projection: `1024 -> 0`.
- Label clones after command parsing: `1024 -> 0`; labels move into the result vector.
- Filtered-entry order and non-command role behavior remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits
`EDITOR262_LABEL_ONLY_COMMAND_PALETTE_PROJECTION_BENCH_V1`. It performs 50 projections of 1,024
commands per sample across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
