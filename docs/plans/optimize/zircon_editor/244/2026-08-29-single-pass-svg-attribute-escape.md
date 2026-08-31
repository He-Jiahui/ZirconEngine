# Editor244 Single-Buffer SVG Attribute Escape

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime298-editor244-performance-batch-20260829y-v1`

## Problem

Each generated Material UI SVG attribute passed through four chained `String::replace` calls. The
path data was allocated and copied four times even when only a subset of XML-sensitive characters
was present, increasing retained-host document construction cost.

## Optimization

- Compute the exact escaped byte capacity while scanning the borrowed UTF-8 input.
- Write ordinary Unicode characters and the four existing XML entity substitutions into one
  preallocated `String`.
- Preserve the legacy replacement bytes, including leaving apostrophes unchanged.

## Regression Contract

The `optimization_batch_20260829y_` Editor tests compare empty, plain, Unicode, and repeated special
character inputs with the legacy escape chain and guard against reintroducing `.replace`. The
ignored paired release benchmark emits `EDITOR244_SINGLE_PASS_SVG_ATTRIBUTE_ESCAPE_BENCH_V1`. It
performs 40,000 representative attribute escapes per sample, reduces result allocations per escape
from four to one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
