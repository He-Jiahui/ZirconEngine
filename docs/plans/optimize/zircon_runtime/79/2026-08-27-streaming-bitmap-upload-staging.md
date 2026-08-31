---
title: Runtime79 Streaming Bitmap Upload Staging
category: zircon_runtime
report_id: Runtime79-streaming-bitmap-upload-staging-2026-08-27
date: 2026-08-27
session_id: root-runtime79-streaming-bitmap-upload-staging-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime79 Streaming Bitmap Upload Staging

## Scope

`glyph_atlas_bitmap_upload_staging_plan` previously filtered every upload copy for each upload
command and collected the matches into a temporary `Vec<&GlyphAtlasBitmapUploadCopy>`. The vector
was used only to detect an empty match set and then iterated once by either the missing-page or
normal staging branch.

The implementation now takes the first matching copy directly from the filter iterator and chains
it back in front of the remaining matches. Both branches consume the same ordered iterator. Page
and target-rectangle membership, empty-command behavior, missing-page failures, source validation,
page-shadow seeding, copy order, and final page/failure ordering are unchanged. No second scan was
introduced.

This slice does not close the open `ui-srgb-coverage-and-native-drop-order` handoff. Its WGPU
surface, shader, and native-submission test paths contain concurrent work owned outside this slice.

## Behavior Evidence

- Existing Rust tests cover multi-source page-row copies, RGBA stride, staged command binding,
  missing/mismatched sources, merged page uploads, packed R8/RGBA regions, and off-origin regions.
- `test_runtime79_streaming_bitmap_upload_staging_performance_contract.py` rejects a temporary
  match vector, requires one `next` selection followed by `once(first).chain(rest)`, preserves both
  copy-consumption branches, and freezes page/target membership and missing-page reporting.

## Deterministic Performance Model

The optimized release model uses 2,048 upload commands and 8,192 upload copies, with four matching
copies per command. Both implementations perform the same page/rectangle comparisons and produce
the exact result checksum `33550336`. The timed workload uses five warmups, 31 alternating samples,
and four complete scans per sample.

| Metric | Materialized matches | Streaming matches | Reduction |
|---|---:|---:|---:|
| allocations per scan | 2,048 | 0 | 100.000% |
| allocated bytes per scan | 65,536 | 0 | 100.000% |

| Run | Legacy P50 ns | Streaming P50 ns | Reduction | Legacy P95 ns | Streaming P95 ns | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 84,239,600 | 52,147,700 | 38.100% | 155,145,700 | 86,146,800 | 44.470% |
| 2 | 79,362,400 | 51,648,200 | 34.920% | 112,938,300 | 111,220,700 | 1.520% |
| 3 | 84,737,200 | 49,808,700 | 41.220% | 184,446,700 | 80,929,300 | 56.120% |
| 4 | 81,085,700 | 49,326,300 | 39.170% | 131,622,400 | 177,289,900 | -34.700% |

The four-run worst-case P50 reduction is 34.920%. P95 is retained as diagnostic evidence rather
than a gate because one Windows scheduling outlier regressed while the other runs improved. The
managed performance gate requires zero streaming match allocations/bytes, at least 30% lower P50,
exact result checksum `33550336`, and nonzero timing checksum `8320483328`.

This model isolates command-to-copy selection. It is not an end-to-end claim about glyph
rasterization, page-shadow copying, WGPU upload bandwidth, frame time, power, or another engine.

## Validation

Passed locally without Cargo:

- 3/3 Python source/performance contracts;
- Python bytecode compilation, Rust formatting, and scoped diff checks;
- four independent optimized release-model runs with exact result parity and all managed gates met.

Managed validation must run the focused bitmap staging Rust tests, the three Python contracts,
formatting, scoped diff, and a fresh release model in one coordinator ticket. Cargo validation is
not claimed until that asynchronous ticket reaches a passing terminal state.

## Remaining Parent-Plan Work

Runtime79 still owns ordered presentation, renderer convergence, color/surface correctness, atlas
residency and churn, stable-frame upload elimination, GPU submission, and product-scale evidence.
This slice only removes per-command transient match storage from the existing bitmap upload stage.
