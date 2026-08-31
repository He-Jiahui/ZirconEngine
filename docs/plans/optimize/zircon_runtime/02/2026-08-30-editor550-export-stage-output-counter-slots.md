---
title: Editor Export Stage Output Counter Slots 550
category: zircon_editor
report_id: Editor550-export-stage-output-counter-slots-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Export Stage Output Counter Slots 550

The export job backpressure adapter previously stored per-stage accepted-event counts in a growing
`Vec<(ExportStage, usize)>`. Every stdout or stderr event linearly searched that Vec, and the first
event allocated its backing storage. `ExportStage` has exactly eight exhaustive variants, so the
adapter now maps each variant to one slot in `[usize; 8]`. The per-stage cap, channel behavior, and
terminal-event delivery contract are unchanged.

The ignored Release evidence `EDITOR550_FIXED_STAGE_OUTPUT_COUNTERS_BENCH_V1` models 65,536 events
for the last of eight initialized stages. Lookup work falls from 524,288 stage comparisons to
65,536 direct slot dispatches, an 87.5% reduction, while counter-container heap allocations fall
from one to zero. A standalone Rust 1.94.1 `opt-level=3` check used 8,000,000 events per sample; the
11-sample median changed from 24.1945 ms to 8.4813 ms, a 64.95% improvement on this machine. Channel
send, event construction, and subprocess output costs are outside that elapsed result.

## Static evidence

- TDD RED: the structural gate found no fixed stage counter type and the production Vec scan.
- TDD GREEN: the fixed counter alias and exhaustive stage index are present; the Vec scan is gone.
- Focused behavior assigns distinct counts to all eight `ExportStage` variants.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/job.rs`
  SHA-256: `6ec181fc1b57de3de021ca3b4c9ee067389fdaa33cd1645e2d64845a02ede46f`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor tests pass.
2. All eight stages retain independent counters and the 16-event per-stage cap.
3. Ignored evidence emits the Editor550 marker and reports the 8-to-1 lookup gate.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
