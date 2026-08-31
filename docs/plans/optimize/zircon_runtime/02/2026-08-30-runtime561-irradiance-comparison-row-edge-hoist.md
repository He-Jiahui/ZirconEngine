---
title: Runtime Irradiance Comparison Row Edge Hoist 561
category: zircon_runtime
report_id: Runtime561-irradiance-comparison-row-edge-hoist-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Irradiance Comparison Row Edge Hoist 561

After face/row traversal establishes a stable row coordinate, the primary irradiance comparison now
classifies top and bottom rows once per row. The inner texel loop combines that cached result with
the unchanged left/right checks. Edge membership and the order of every error accumulation remain
identical.

For a 256 by 256 six-face cube, Y-edge comparisons fall from 786,432 per-texel checks to 3,072
per-row checks, a 99.61% reduction. A standalone Rust 1.94.1 `opt-level=3` benchmark isolated this
increment from the face/row traversal, compared 16 complete scans per sample, verified exact
statistics equality, and used 21 interleaved samples. Conservative P95 changed from 109.943 ms to
75.360 ms, a 31.45% improvement on this machine.

## Static evidence

- TDD RED: top/bottom row conditions were evaluated in the innermost texel loop.
- TDD GREEN: `row_is_edge` is computed once per row and reused for every texel in that row.
- A focused regression compares complete statistics against the legacy loop for four face sizes.
- Focused tests use prefix `optimization_batch_20260830ew_runtime561_`.
- Ignored evidence marker: `RUNTIME561_IRRADIANCE_ROW_EDGE_HOIST_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `7424ce7e346f06202a6d4ada4aa91be5b1d3c942fc7603330a2202ad8f986cf7`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Edge sample counts and all edge error accumulators remain exactly equal to the legacy loop.
3. Y-edge classification occurs once per row, while left/right classification remains per texel.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
