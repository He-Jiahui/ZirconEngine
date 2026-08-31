---
title: Editor371 Close Prompt Frame Only Clone
category: zircon_editor
report_id: Editor371-close-prompt-frame-only-clone-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor371 Close Prompt Frame Only Clone

Close-prompt replacement now borrows the current prompt and copies only its overlay frame for redraw
damage. The old path cloned the complete prompt, including window id, title, message, details, and
all button frames, solely to move out one frame.

Visible-prompt damage, first-prompt damage, prompt replacement, state mutation, and external redraw
behavior remain unchanged. Regression coverage requires the frame-only clone and rejects restoration
of a complete current-prompt clone.

The ignored Windows Release benchmark emits `EDITOR371_CLOSE_PROMPT_FRAME_ONLY_CLONE_BENCH_V1`
over 17 alternating paired samples. Each sample models 16,384 visible-prompt replacements with long
window and message strings. The legacy model clones 16,384 complete prompts per sample; the optimized
model clones only the allocation-free frame. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.20`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor371 is prepared with Runtime443 under request
`runtime443-editor371-performance-batch-20260831ei-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
