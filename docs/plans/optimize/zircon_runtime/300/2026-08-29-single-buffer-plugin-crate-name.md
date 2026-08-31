# Runtime300 Single-Buffer Plugin Crate Name

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime300-editor246-performance-batch-20260829aa-v1`

## Problem

Resolving a default runtime plugin crate name first replaced hyphens into a temporary `String` and
then formatted the prefix and suffix into another `String`. Manifest resolution therefore allocated
and copied the plugin ID twice whenever no explicit runtime crate override was configured.

## Optimization

- Reserve the exact final byte capacity from the fixed prefix, plugin ID, and suffix.
- Write the plugin ID directly into the result while mapping only hyphens to underscores.
- Preserve arbitrary Unicode and punctuation bytes, plus the existing explicit override branch.

## Regression Contract

The `optimization_batch_20260829aa_` Runtime tests cover hyphenated, plain, Unicode, and explicitly
overridden crate names and guard removal of both `replace` and `format!` from the default builder.
The ignored paired release benchmark emits `RUNTIME300_SINGLE_BUFFER_PLUGIN_CRATE_NAME_BENCH_V1`.
It performs 100,000 default crate-name resolutions per sample, reduces result allocations per name
from two to one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
