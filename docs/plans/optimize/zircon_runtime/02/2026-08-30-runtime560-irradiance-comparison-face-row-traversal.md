---
title: Runtime Irradiance Comparison Face Row Traversal 560
category: zircon_runtime
report_id: Runtime560-irradiance-comparison-face-row-traversal-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Irradiance Comparison Face Row Traversal 560

The primary irradiance-cubemap error scan previously reconstructed face-local X/Y coordinates from
every linear texel index with remainder and division operations. It now traverses the already
canonical face and row slices directly. Texel order, channel order, floating-point accumulation,
edge membership, seam comparison, and returned statistics are unchanged.

For a 256 by 256 six-face cube, 393,216 texels previously evaluated three division/remainder
expressions each, or 1,179,648 expressions; the face/row traversal evaluates none. A standalone
Rust 1.94.1 `opt-level=3` benchmark compared 16 complete cube scans per sample, verified exact
statistics equality, and used 21 interleaved samples. Conservative P95 changed from 95.298 ms to
34.781 ms, a 63.50% improvement on this machine.

## Static evidence

- TDD RED: production recovered face-local coordinates through `index %`, `%`, and `/` per texel.
- TDD GREEN: production traverses exact face and row chunks without texel-index division.
- A focused regression compares complete statistics against the legacy loop for four face sizes.
- Focused tests use prefix `optimization_batch_20260830ew_runtime560_`.
- Ignored evidence marker: `RUNTIME560_IRRADIANCE_FACE_ROW_TRAVERSAL_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `7424ce7e346f06202a6d4ada4aa91be5b1d3c942fc7603330a2202ad8f986cf7`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Legacy and optimized complete statistics remain exactly equal for representative face sizes.
3. The primary production scan contains no per-texel division or remainder coordinate recovery.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
