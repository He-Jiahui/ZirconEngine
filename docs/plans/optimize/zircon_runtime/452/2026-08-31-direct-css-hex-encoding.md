---
title: Runtime452 Direct CSS Hex Encoding
category: zircon_runtime
report_id: Runtime452-direct-css-hex-encoding-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime452 Direct CSS Hex Encoding

Slider render-command projection now converts RGBA channels directly into one preallocated CSS hex
string. The former per-command path invoked the formatting machinery and then inserted `#` at the
front, shifting the formatted bytes and potentially growing the allocation for alpha-bearing
colors.

Opaque colors still emit `#rrggbb`, non-opaque colors still emit `#rrggbbaa`, channels remain
lowercase and zero-padded, and the existing `UiRgbaColor::to_u8` conversion remains authoritative.
Regression coverage compares black, mixed translucent, and boundary channel values with the former
formatter-and-insert implementation.

The ignored Windows Release benchmark emits `RUNTIME452_DIRECT_CSS_HEX_ENCODING_BENCH_V1` over 17
alternating paired samples. Each sample performs 65,536 conversions over four opaque/translucent
colors. The legacy path performs one formatting call and one front insertion per conversion; the
optimized path performs neither and uses one exact-capacity allocation. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.55`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime452 is prepared with Editor382 under request
`runtime452-editor382-performance-batch-20260831et-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
