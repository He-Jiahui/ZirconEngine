---
title: Editor Export Ordered Command Index 545
category: zircon_editor
report_id: Editor545-export-ordered-command-index-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Export Ordered Command Index 545

`ExportWizardPipelinePlan::ordered_commands` previously walked the authoritative core stage order
and linearly searched the command Vec for every stage. Plans are constructed with commands in that
same order, so the iterator now checks the command at the matching position first. If the public
`stages` Vec was externally reordered or malformed, it falls back to the original `command(stage)`
lookup, preserving the prior authoritative ordering semantics.

The ignored Release evidence `EDITOR545_EXPORT_ORDERED_COMMAND_INDEX_BENCH_V1` models 65,536 full
eight-stage plan traversals. Normal constructed-plan stage comparisons fall from 2,359,296 to
524,288, a 77.78% reduction. A standalone Rust 1.94.1 `opt-level=3` check used two million full
traversals per sample; the 11-sample median for isolated mapping changed from 93.58 ms to 19.18 ms,
a 79.50% improvement on this machine. Export subprocess time is not included.

## Static evidence

- TDD RED: the structural gate found no `self.stages.get(index)` fast path.
- TDD GREEN: each core node checks the same command position before the legacy lookup fallback.
- Focused behavior proves canonical order for a constructed plan and after swapping two public
  command entries.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/plan.rs` SHA-256:
  `13bd742d6fb054e5a2ce9b5eebd24dcdfa8241c8abdadc5d8ee7fed89f1cd079`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor tests pass.
2. Constructed and externally reordered command vectors both preserve core-plan stage order.
3. Ignored evidence emits the Editor545 marker and reports the 36-to-8 comparison gate.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
