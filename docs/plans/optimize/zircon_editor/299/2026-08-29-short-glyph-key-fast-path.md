# Editor299 Short Glyph Key Fast Path

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime354-editor299-performance-batch-20260829ca-v1`

## Scope

Button glyph classification previously ran all nine substring searches for empty and one- or
two-byte keys even though the shortest recognized keyword is three bytes. Those keys now return the
existing `None` result before substring matching.

## Static Evidence

- Substring searches for keys shorter than three bytes: `9 -> 0`.
- Recognized keyword set and Trash, ChevronDown, Plus priority remain unchanged.
- Three-byte and longer key classification remains on the existing substring path.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR299_SHORT_GLYPH_KEY_BENCH_V1`. It compares the
prior nine searches with the length guard over 1,000,000 empty-key checks per sample and 31
interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
