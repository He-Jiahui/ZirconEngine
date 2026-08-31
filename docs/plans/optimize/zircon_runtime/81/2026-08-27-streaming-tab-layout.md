---
title: Runtime81 Streaming Tab Layout
category: zircon_runtime
report_id: Runtime81-streaming-tab-layout-2026-08-27
date: 2026-08-27
session_id: root-runtime81-streaming-tab-layout-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime81 Streaming Tab Layout

## Scope

`tab_aligned_advances` previously materialized every Unicode grapheme as a temporary `Vec<&str>`
before allocating its final `Vec<f32>`. `tab_aligned_width` called that function and therefore
allocated both vectors even though the caller only needed one scalar width.

The implementation now performs an allocation-free grapheme-count/tab-presence pass. The advance
path allocates only its final result and streams the second grapheme pass into it. The width path
streams the second pass directly into its cursor and allocates no output vector. Existing semantics
remain unchanged when grapheme and advance cardinality differ or when the text contains no tab:
advances are returned verbatim and width sums the raw input advances. Tab-stop arithmetic, negative
advance sanitization on the active tab path, Unicode grapheme segmentation, minimum tab interval,
and `TextStyle::tab_size` handling are unchanged.

This slice does not close `RTS-P1-033`; explicit/right/center/decimal tab stops, leaders, BiDi, and
vertical-writing tab policy remain parent-plan work.

## Behavior Evidence

- `streaming_tab_layout_preserves_tab_stops_and_unicode_graphemes` covers a combining-mark
  grapheme, a tab, a negative non-tab advance, the exact adjusted vector, and scalar width parity.
- `streaming_tab_layout_preserves_unaligned_advance_fallbacks` covers both no-tab and cardinality
  mismatch fallbacks, including preservation of a negative raw advance.
- `test_runtime81_streaming_tab_layout_performance_contract.py` rejects a temporary grapheme vector,
  requires the advance path to allocate only its final vector, and requires scalar width
  accumulation without calling the vector-returning path.

## Deterministic Performance Model

The release model uses 131,072 single-byte graphemes with a tab every 16 graphemes and the same
tab-stop arithmetic in both paths. Its legacy grapheme token has the production `&str` size of 16
bytes on x64. Exact advance-vector equality, width equality, and nonzero checksums are required.

| Metric | Materialized graphemes | Streaming | Reduction |
|---|---:|---:|---:|
| advance allocations | 2 | 1 | 50.000% |
| advance allocated bytes | 2,621,440 | 524,288 | 80.000% |
| width allocations | 2 | 0 | 100.000% |
| width allocated bytes | 2,621,440 | 0 | 100.000% |

Each run uses five warmups and 31 width samples:

| Run | Legacy P50 ns | Streaming P50 ns | Reduction | Legacy P95 ns | Streaming P95 ns | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1,226,800 | 202,100 | 83.530% | 9,917,700 | 255,600 | 97.420% |
| 2 | 1,280,700 | 214,300 | 83.270% | 17,433,600 | 1,755,800 | 89.930% |
| 3 | 1,276,600 | 197,000 | 84.570% | 4,308,600 | 276,800 | 93.580% |
| 4 | 1,238,300 | 200,400 | 83.820% | 3,537,700 | 389,100 | 89.000% |

The four-run worst case reduces P50 by 83.270% and P95 by 89.000%. The timing checksum is
`37706792960`. The managed gate requires one final allocation for the advance path, zero allocations
for width, at least 75% fewer advance bytes, at least 70% lower P50, at least 50% lower P95, and
exact result/checksum parity.

This is an isolated ASCII/tab algorithm model. It proves the removed allocation and projection
cost but is not a claim about full Unicode shaping, end-to-end editor layout, GPU rendering, power,
or comparison with another engine.

## Validation

Passed locally without Cargo:

- 3/3 Python performance/source contracts;
- Rust formatting and scoped diff checks;
- four independent optimized release-model runs with exact output parity and all gates met.

Managed validation must run both focused Rust behavior tests, the three Python contracts,
formatting, scoped diff, and a fresh release model in one coordinator ticket. Cargo validation is
not claimed until that asynchronous ticket reaches a passing terminal state.

## Remaining Parent-Plan Work

Runtime81 still owns canonical paragraph tab-stop artifacts, writing-mode/direction-aware placement,
leaders, retained incremental layout, Unicode/locale tailoring, and product-scale multilingual
evidence. This slice only removes transient storage from the existing uniform-tab implementation.
