# Editor219 Owned Text Focus Control ID

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime273-editor219-performance-batch-20260828ia-v1`

## Problem

Editor retained-host text dispatch received an owned focus snapshot but cloned its control ID before
routing. Commit-only routing also called an owned target getter that cloned the edit/action/control
fallback string solely for comparison, after which the original focus snapshot was discarded.

## Optimization

- Resolve the edit target through borrowed string fields using the existing precedence.
- Move the control-ID allocation out of the owned focus snapshot with `mem::take`.
- Leave the remaining focus valid for dispatch-kind checks and redraw calculation.
- Preserve welcome, showcase, inspector, asset, commit-only, and fallback routing semantics.

## Regression Contract

The `optimization_batch_20260828ia_` Editor tests prove borrowed target and moved control-ID buffer
identity and prevent both prior clones from returning to text dispatch. The ignored paired release
benchmark emits `EDITOR219_OWNED_TEXT_FOCUS_CONTROL_ID_BENCH_V1`. It routes 256 focus snapshots per
sample with 64-KiB string fields and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
