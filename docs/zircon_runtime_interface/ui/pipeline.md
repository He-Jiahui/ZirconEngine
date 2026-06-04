---
related_code:
  - zircon_runtime_interface/src/ui/pipeline/mod.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/dirty_reason.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_counters.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_report.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - dev/bevy/crates/bevy_ui/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - dev/Fyrox/editor/src/lib.rs
implementation_files:
  - zircon_runtime_interface/src/ui/pipeline/mod.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/dirty_reason.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_counters.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_report.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report.rs
  - zircon_runtime_interface/src/ui/mod.rs
plan_sources:
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
  - docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md
  - user: 2026-05-08 continue M2 UI pipeline/schedule interface slice
tests:
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - 2026-05-08: cargo test -p zircon_runtime_interface --lib pipeline_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-pipeline-m2 --message-format short --color never (3 passed; 0 failed; 70 filtered out)
  - 2026-05-08: cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-pipeline-m2 --message-format short --color never (passed)
  - 2026-05-08: rustfmt --edition 2021 --check touched M2 pipeline files (passed)
  - 2026-05-08: git diff --check -- touched M2 pipeline files and docs (passed with CRLF conversion warnings only)
  - 2026-06-04: rustfmt --edition 2021 --check zircon_runtime_interface\src\ui\pipeline\stage.rs zircon_runtime_interface\src\tests\pipeline_contracts.rs (passed)
  - 2026-06-04: cargo test -p zircon_runtime_interface pipeline_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-stage-policy-0604 --message-format short --color never -- --test-threads=1 --nocapture (3 passed; 0 failed; 130 filtered out)
doc_type: module-detail
---

# UI Pipeline Contracts

`zircon_runtime_interface::ui::pipeline` owns the neutral M2 pipeline-report DTO vocabulary. It names the UI frame stages and records timing/counter evidence, but it does not schedule systems, rebuild surfaces, run layout, hit-test, render, or submit paint.

## Reference Anchors

Bevy is the dominant reference for this slice. `dev/bevy/crates/bevy_ui/src/lib.rs` names the UI schedule around focus, preparation, content, layout, post-layout, and stack work. `dev/bevy/crates/bevy_ui_render/src/lib.rs` separates UI render extraction from queue, prepare, and pass submission. Zircon keeps these as neutral DTO stages rather than importing Bevy schedules or ECS system sets.

Fyrox is the Rust-native editor/runtime cross-check. `dev/Fyrox/editor/src/lib.rs` keeps window event processing, redraw requests, and UI update loop state explicit at the editor boundary. The Zircon DTOs preserve that separation by reporting what happened in a frame without owning host event-loop behavior.

## Stage Order

`UiPipelineStage::ORDER` is the M2 schedule contract:

- `InputCollect`: gather window and shared input events into the frame.
- `Focus`: update focus, capture, interaction, and focus-visible state.
- `WidgetBehavior`: run widget-level behavior that can affect text, style, layout, or input domains.
- `TextMeasure`: measure text, image, viewport, or other content requirements before layout.
- `Layout`: compute layout geometry through the current or future layout engine.
- `PostLayout`: update paint order, z stack, clipping, scroll-dependent text state, and other post-layout data.
- `Picking`: rebuild or reuse hit-test grid records.
- `A11yExtract`: extract accessibility-facing state.
- `RenderExtract`: extract render or paint commands from arranged UI state.
- `BatchPrepare`: group extracted paint/render work into backend-ready batches.

## Archived Diagnostic Stage Names

`UiPipelineStage::ARCHIVED_DIAGNOSTIC_FORMAT_VERSION` is the explicit version marker for stored diagnostic payloads that predate the current M2 schedule. `FocusInteraction`, `ContentMeasure`, `PostLayoutStack`, `HitGrid`, `PaintSubmit`, and `Diagnostics` remain deserializable so archived reports can still be read, but `UiPipelineStage::is_runtime_schedule_stage()` returns `false` for them and `UiPipelineStage::ORDER` excludes them. New runtime and editor scheduling code must use the current 10-stage order above.

## Reports

`UiPipelineStageReport` records one stage, elapsed microseconds, skipped state, dirty reasons, counters, and optional notes. `UiPipelineFrameReport` aggregates those records for one frame, recomputes total elapsed time, sums counters, reports missing required stages, checks exact M2 stage order, and exposes the repeated pointer-move fast-path predicate.

`UiPipelineStageCounters` is deliberately a flat counter bag so runtime and editor producers can fill the same fields without depending on runtime surface internals. The M2 counters cover input events, pointer moves, focus changes, content measurements, template reloads, layout nodes, full and incremental layout passes, stack nodes, hit-grid rebuilds, render extraction, render command reuse/rebuild, batch count, paint submissions, and diagnostics records.

`UiPipelineDirtyReason` is a compact reason enum for frame and stage reports. It distinguishes input, focus, text, style, layout, layout metrics, hit grid, render, template, window, host request, and diagnostics causes without importing runtime dirty-flag behavior into the interface crate.

## Acceptance Boundary

The focused tests in `zircon_runtime_interface/src/tests/pipeline_contracts.rs` prove the DTOs can express the M2 acceptance contract: the exact named stage order, the archived diagnostic stage policy, per-stage timing and dirty-reason reporting, layout/hit/render/batch counters, and the 100 pointer-move fast path with zero template reloads and zero full-layout work. The 2026-05-08 scoped interface gate passed for the focused contract tests and crate check.

This module does not prove that runtime or editor code already populates these reports. Later M2 runtime/editor slices must connect `UiSurface`, editor hosts, and render submission to this contract after the active layout/render/input lanes settle.
