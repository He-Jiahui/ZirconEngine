# Editor76 Hex Color Nibble Decode

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Status: `implementation_complete / managed_validation_pending`
- Batch: `optimization_batch_20260826cm_`

## Problem

Retained workbench projection can request many fallback color properties for each node. Every
successful property used a generic `u8::from_str_radix` call for each channel and sliced the UTF-8
string by byte ranges. The general parser adds overhead on this fixed six/eight-digit ASCII format,
and malformed non-ASCII text could place a slice boundary inside a code point.

## Optimization

- Decode fixed ASCII hexadecimal nibbles directly from the validated six/eight-byte input.
- Preserve whitespace trimming, the required `#` prefix, upper/lowercase input, RGB default alpha,
  explicit RGBA alpha, and `None` for malformed values.
- Remove byte-range string slicing so malformed UTF-8 input is rejected rather than panicking.

## Test And Performance Contract

- The behavior regression covers RGB, RGBA, mixed case, surrounding whitespace, invalid digits,
  invalid length, and a non-ASCII boundary input.
- The source regression requires byte/nibble decoding and rejects `from_str_radix` and the former
  string range slice.
- Ignored release evidence prints `EDITOR76_HEX_COLOR_NIBBLE_DECODE_BENCH_V1` for 21 alternating
  sample pairs over 65,536 RGB/RGBA colors.
- Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 70`.

## Validation State

Exact Rust 1.94.1 formatting and scoped static checks are required before submission. Cargo tests,
exact P50/P95 values, commit SHA, push result, and WeCom delivery remain coordinator-owned terminal
evidence and are not claimed by this pending record.

