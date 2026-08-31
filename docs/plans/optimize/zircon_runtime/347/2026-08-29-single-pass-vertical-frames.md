# Runtime347 Single-Pass Vertical Frames

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime347-editor292-performance-batch-20260829bt-v1`

## Scope

Vertical right-to-left text layout previously materialized every column frame and then traversed the
strided frame array again to recover the maximum height. Frame construction now reserves the exact
capacity and folds the measured height into the same loop that writes each frame.

## Static Evidence

- Frame-array traversals per layout: `2 -> 1`.
- Frame vector growth reallocations remain `0` for the known input length.
- Coordinate sanitization, frame ordering, extents, and empty-column behavior remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME347_SINGLE_PASS_VERTICAL_FRAME_BENCH_V1`. It
compares the collect-then-rescan baseline with fused frame construction over 16,384 columns, 128
layouts per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
