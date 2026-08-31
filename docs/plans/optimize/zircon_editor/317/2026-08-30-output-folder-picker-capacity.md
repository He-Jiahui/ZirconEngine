---
title: Editor317 Output Folder Picker Capacity
category: zircon_editor
report_id: Editor317-output-folder-picker-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor317 Output Folder Picker Capacity

`pick_output_folder` now retains the picker command vector and reserves its length for missing
program diagnostics before executing the commands. Picker fallback order, early success/exit
handling, and unavailable-program reporting are unchanged.

Regression coverage checks the capacity contract and missing-program order. The ignored Windows
Release benchmark emits `EDITOR317_OUTPUT_FOLDER_CAPACITY_BENCH_V1` over 17 paired samples with
two commands per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor317 is included in ticket `bb793f894807473ea8c78a90c6fc2d35` for request
`runtime-editor-369-371-315-317-20260830-v2`, with source manifest hash
`391c0060104af61c0806431d76bebbaf6f1d74c41c216b63aa899577269baf4c`. The batch also binds
`external_image_copy.rs` at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.
Cargo, performance, review, commit, push, and WeCom remain coordinator-owned and pending.

## Validation attempt (2026-08-30)

Corrected batch ticket `bb793f894807473ea8c78a90c6fc2d35` ended `failed`. The coordinator
provided no valid Cargo, performance, or commit evidence; the external validation resource was
left unchanged and no successful WeCom notification was sent.
