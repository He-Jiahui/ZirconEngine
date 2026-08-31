# Editor280 Borrowed Reserved Atlas Stem

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime335-editor280-performance-batch-20260829bh-v1`

## Scope

Sprite-atlas build configuration previously trimmed the output stem twice and allocated an uppercase
copy for every Windows reserved-name check. Validation now caches one trimmed slice and compares the
borrowed base stem against a static reserved-name table without allocation. Error priority, safe
characters, dotted stem behavior, case-insensitive reserved names, and size validation are unchanged.

## Static Evidence

- `str::trim` calls per validation: `2 -> 1`.
- Uppercase stem allocations per otherwise-valid configuration: `1 -> 0`.
- Reserved-name lookup remains ASCII case-insensitive.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR280_BORROWED_RESERVED_ATLAS_STEM_BENCH_V1`. It
compares the baseline repeated-trim/uppercase-allocation path with borrowed validation over 8,192
valid 4-KiB stems across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
