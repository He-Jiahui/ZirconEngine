---
status: local_candidate
implementation_files:
  - tools/analysis/visual/zircon_editor_ui_visual_oracle.py
  - tools/tests/test_editor_visual_oracle_profile_single_read_performance_contract.py
---

# Visual oracle capture inputs single read

## Problem

For every captured editor extent, the visual oracle first read both profile
geometry JSON and the screenshot to verify their SHA-256 values. It then
reopened the profile for JSON parsing and the screenshot for PNG decoding.
Both large capture inputs therefore paid two full content reads.

## Change

`_load_verified_json` now reads profile geometry once as bytes. The same payload
supplies SHA-256 verification, UTF-8-sig decoding, JSON parsing, and object
validation. The returned digest continues to populate the report binding.
`_load_verified_bytes` similarly verifies the screenshot payload once and passes
those bytes to `_RgbImage.from_png_bytes`. `load_png` remains available for
callers that do not already own a payload.

The static performance contract prevents `_analyze_capture` from returning to
the separate hash, parse, or decode read paths for either input.

## Validation

```text
python -m unittest tools.tests.test_editor_visual_oracle_profile_single_read_performance_contract tools.tests.test_zircon_editor_ui_visual_oracle
28/28 passed in 26.685s
```

## Performance

Windows local benchmark with one 8 MiB profile geometry JSON file, two warm-ups
and 41 alternating-order iterations; p95 is the 39th ordered sample.

| Metric | Baseline | Candidate | Improvement |
| --- | ---: | ---: | ---: |
| Content reads | 2 | 1 | -50.000% |
| Bytes per capture | 16,777,160 | 8,388,580 | -50.000% |
| p50 | 43,343,200 ns | 31,306,000 ns | -27.772% (1.39x) |
| p95 | 102,667,900 ns | 85,222,000 ns | -16.993% (1.21x) |

Screenshot I/O/hash was measured separately with an 8 MiB PNG-like payload,
excluding PNG decompression so only the eliminated reopen is attributed. Reads
and bytes fell `2 -> 1`; p50 improved `16,936,200 -> 10,848,800 ns`
(-35.943%, 1.56x), and p95 improved `26,616,100 -> 19,114,800 ns`
(-28.183%, 1.39x).
