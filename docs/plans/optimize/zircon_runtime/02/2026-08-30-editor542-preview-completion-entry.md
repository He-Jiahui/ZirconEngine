---
title: Editor Preview Completion Entry 542
category: zircon_editor
report_id: Editor542-preview-completion-entry-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Preview Completion Entry 542

`PreviewScheduler::complete_refresh` previously hashed the same asset UUID once to validate its
generation token and again to remove the admitted job. It now uses one occupied `HashMap::entry`
probe, compares the token through that entry, and removes in place. Missing assets and stale tokens
still return `false` without releasing the current admission.

The ignored Release evidence `EDITOR542_PREVIEW_COMPLETION_ENTRY_BENCH_V1` models 65,536 valid
completions. Hash probes fall from 131,072 to 65,536, a 50% reduction. A standalone Rust 1.94.1
`opt-level=3` check used a 64-entry map and 2,000,000 complete/reinsert operations per sample;
the 11-sample median changed from 127.21 ms to 85.60 ms, a 32.71% improvement on this machine.
That elapsed result is a local path sanity check, not whole-editor throughput evidence.

## Static evidence

- TDD RED: the structural gate found the legacy `get` followed by `remove` and no entry probe.
- TDD GREEN: completion contains one `self.in_flight.entry(asset_uuid)` and no keyed removal.
- Existing behavior tests cover valid completion, stale-token rejection, and retained current-token
  ownership.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_editor/src/ui/host/editor_asset_manager/preview.rs` SHA-256:
  `67fd1c2a1a4d071a9e9e616fb64d06ca71752cff2ac861463a64c31299dae406`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor tests pass.
2. Valid completion removes only its matching token; stale completion preserves the current token.
3. Ignored evidence emits the Editor542 marker and reports the 2-to-1 hash-probe gate.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
