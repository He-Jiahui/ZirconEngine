# Runtime131 Texture Dimension Zero-Allocation Match

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Status: `implementation_complete / managed_validation_pending`
- Batch: `optimization_batch_20260826cn_`

## Problem

`RenderMaterialTextureDimension::from_shader_kind` allocated a lowercase `String` for every shader
texture kind before matching a small, fixed alias set. Material and shader preparation call this
path repeatedly even though ASCII case-insensitive comparison can operate on the borrowed input.

## Optimization

- Trim once, dispatch aliases by byte length, and use `eq_ignore_ascii_case` only for possible
  aliases in that length bucket.
- Preserve every prior 1D, 2D-array, cube, cube-array, and 3D alias plus the default D2 result.
- Remove the lowercase allocation without adding a compatibility path or changing public types.

## Test And Performance Contract

- The behavior regression covers whitespace, mixed case, every dimension family, the implicit 2D
  spelling, and unknown-kind fallback.
- The source regression requires ASCII case-insensitive matching and rejects
  `to_ascii_lowercase` in production.
- Ignored release evidence prints
  `RUNTIME131_TEXTURE_DIMENSION_ZERO_ALLOCATION_MATCH_BENCH_V1` for 21 alternating sample pairs
  over 65,536 mixed aliases.
- Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 70`.

## Validation State

Rust 1.94.1 formatting and scoped static checks are required before submission. Cargo results,
exact P50/P95 values, commit SHA, push result, and WeCom delivery remain coordinator-owned terminal
evidence and are not claimed by this pending record.

