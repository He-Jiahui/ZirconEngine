# Editor81 Picker Borrowed UTF-8 Selection

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime135-editor81-performance-batch-20260826cr-v1`

## Problem

Native output-folder picker parsing always copied the decoded selection into an intermediate
`String`, then copied it again into `PathBuf`. Valid UTF-8 is the normal process-output path and
does not need the first owner.

## Optimization

- Keep `String::from_utf8_lossy` as a `Cow<str>` while trimming the selection.
- Construct `PathBuf` directly from the borrowed trimmed slice on valid UTF-8.
- Preserve empty selection handling, surrounding whitespace trimming, and replacement-character
  repair for malformed process output.

## Regression Contract

The shared `optimization_batch_20260826cr_` filter owns three Editor tests: selection behavior,
source shape, and an ignored paired release P95 benchmark. The benchmark emits
`EDITOR81_PICKER_BORROWED_UTF8_SELECTION_BENCH_V1`, performs 90,000 representative picker parses
per sample, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
