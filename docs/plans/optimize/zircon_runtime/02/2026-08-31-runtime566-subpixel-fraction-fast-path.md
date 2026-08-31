---
title: Runtime Text Subpixel Fraction Fast Path 566
category: zircon_runtime
report_id: Runtime566-text-subpixel-fraction-fast-path-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Text Subpixel Fraction Fast Path 566

Subpixel glyph placement previously used `f32::rem_euclid(1.0)` for every raster key. The common
finite screen-coordinate path now takes `fract()` and adds one only for negative fractions. This
preserves the `[0, 1)` fraction and all three subpixel bins, including negative integer and
negative fractional coordinates.

A Rust 1.94.1 `opt-level=3` standalone benchmark used 13 interleaved sample pairs, 30,000,000
coordinates per sample, and a workload with one negative coordinate per 128 inputs. P95 changed
from `326,216,800 ns` to `217,911,000 ns`, a `33.20%` reduction. Exact bin equality was checked
across positive, negative, integer, and bin-boundary inputs.

## Static evidence

- TDD RED: signed `fract()` normalization was absent before implementation.
- TDD GREEN: focused tests compare the optimized bin against `rem_euclid` at negative and positive boundaries.
- Ignored benchmark marker: `RUNTIME566_SUBPIXEL_FRACTION_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831fa_runtime566_`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production source SHA-256: `264feceb47ce53001b0e00bb56bd26ef0498c3f7601585326a8be8bb564dcdb5`.
- Focused test source SHA-256: `441fc7c0b955dd17bc5cc49b148166a5a9dba80c4ff49f62d7851e7680625e86`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime text tests pass.
2. Optimized and legacy subpixel bins remain identical for signed finite coordinates.
3. Managed ignored benchmark retains at least a 15% P95 improvement.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted validation.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
