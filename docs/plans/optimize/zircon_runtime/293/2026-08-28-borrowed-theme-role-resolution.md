# Runtime293 Borrowed Theme-Role Resolution

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime293-editor239-performance-batch-20260828iu-v1`

## Problem

Resolving every semantic theme role normalized the role to a borrowed token name and then copied
that name into a temporary `UiThemeTokenRef`. The registry consumed only the copied string during
the same call, so the hot style-resolution path paid one avoidable heap allocation per role.

## Optimization

- Share the existing token lookup through a private borrowed `&str` helper.
- Route both owned token references and normalized semantic roles through the same lookup table.
- Preserve every supported role prefix and unknown-token result.
- Remove the temporary token object from semantic role resolution.

## Regression Contract

The `optimization_batch_20260828iu_` Runtime tests compare all four supported role prefixes with
the legacy allocation path and guard the borrowed helper call. The ignored paired release benchmark
emits `RUNTIME293_BORROWED_THEME_ROLE_RESOLUTION_BENCH_V1`. It performs 100,000 resolutions of a
24-byte role per sample, reduces temporary token allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
