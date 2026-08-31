---
title: Editor Scaled Image X Sample Cache 554
category: zircon_editor
report_id: Editor554-scaled-image-x-sample-cache-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Scaled Image X Sample Cache 554

The retained-host scaled-image rasterizer previously recomputed each column's source coordinate,
lower/upper texel, and interpolation mix on every destination row. It now materializes those X-axis
samples once per draw, reuses them across all rows, and advances the destination byte offset by one
RGBA pixel instead of multiplying the X coordinate for every pixel. Y-axis sampling and bilinear
pixel writes retain their existing formulas and order.

For a 512 by 512 destination, X-axis sample calculations fall from 262,144 to 512, a 99.80%
reduction, at the cost of one width-sized sample-vector allocation. A standalone Rust 1.94.1
`opt-level=3` benchmark included that allocation and full two-dimensional sample consumption at
1024 by 768. The 11-sample median changed from 4.9984 ms to 0.6220 ms, an 87.56% improvement on
this machine. Frame-buffer blending and memory writes are excluded.

## Static evidence

- TDD RED: the X-axis sample helper was called inside the destination-row loop.
- TDD GREEN: one `source_x_samples` vector is consumed by every row.
- A focused equivalence test checks cached lower/upper/mix values against the direct formula.
- Focused tests use prefix `optimization_batch_20260830et_editor554_`.
- Ignored evidence marker: `EDITOR554_X_AXIS_SAMPLE_CACHE_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `c6d46dd29e1174d8564a84a52a57dcfcca20d008b248f7b2e251c5a3be509c76`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor tests pass.
2. Cached sample coordinates remain bit-identical to the prior direct calculation.
3. Destination coverage, bilinear neighbors, and interpolation order remain unchanged.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
