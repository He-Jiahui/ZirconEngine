# Editor279 Single-Scan Asset Extension

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime334-editor279-performance-batch-20260829bg-v1`

## Scope

Asset-reference drag payload construction previously scanned backward for the final path separator
and then scanned the file name backward again for its extension. Extension extraction now performs
one reverse byte scan and stops at the first extension dot or directory boundary. Unix and Windows
separators, hidden files, trailing dots, Unicode paths, and the owned extension result are unchanged.

## Static Evidence

- Reverse locator scans: `2 -> 1`.
- Extension allocations: `1 -> 1` for non-empty extensions.
- Missing-extension allocation remains `0`.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR279_SINGLE_SCAN_ASSET_EXTENSION_BENCH_V1`. It
compares the baseline separator/extension scans with the single reverse scan over 8,192 valid 4-KiB
locators across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
