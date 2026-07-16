---
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/decision.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/ui/retained_host/viewport/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
implementation_files:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/decision.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
plan_sources:
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - user: 2026-07-13 continue HybridGI product validation
tests:
  - recompute_preserves_pending_render_work_until_render_submission_consumes_it
  - failed_render_submission_records_the_typed_error_in_process_diagnostics
  - editor_product_entry_bootstraps_linked_first_party_runtime_plugins
  - tools.tests.test_hybrid_gi_editor_profile
  - tools.tests.test_hybrid_gi_m4_contract
doc_type: module-detail
---

# Retained Host Render Submission

## Purpose

This boundary owns the Editor product path from retained-host invalidation to a submitted viewport frame. It must keep render work pending while the WGPU framework initializes asynchronously, preserve the actual submission error for diagnostics, and run under an App entry that has registered the linked first-party render providers.

## Startup And Provider Ownership

Both interactive Editor startup and headless Editor operations use `bootstrap_with_first_party_runtime_plugin_registrations(...)`. `EntryProfile::Editor` requests Hybrid GI, but the request alone is insufficient: the App target must also inject the linked first-party registrations so the runtime can resolve `plugin.hybrid_gi.runtime` instead of reporting the feature as unavailable.

Provider selection remains capability and profile gated. Registering the catalog does not force every provider to execute, and Editor code does not construct HybridGI runtime state directly.

## Invalidation Contract

`begin_recompute_invalidation_phase(...)` unions pending invalidation reasons with reasons reconstructed from the legacy dirty flags. It does not choose one source and discard the other. This matters during first-frame startup:

1. viewport submission requests a render;
2. `poll_or_start_render_framework(...)` begins asynchronous WGPU framework creation and returns no framework for the current pass;
3. `render_dirty` must remain set so a later recompute retries submission;
4. presentation-only invalidation may repaint the shell while initialization is pending, but it must not clear the render request;
5. only the render-submission path consumes completed render work.

Recompute therefore must not assign `render_dirty = false` merely because the current reason set is paint-only. Clearing it there can leave a valid viewport at zero submitted frames indefinitely.

## Submission Diagnostics

Every typed viewport submission error is written through process diagnostics with scope `editor_viewport_submission` before the short status-line message is updated. The persistent record is required because the status line can be replaced by later presentation updates and because errors such as render-graph resource contract mismatches are too detailed for the compact shell surface.

Runtime Diagnostics reads the latest runtime-owned `RenderStats`. HybridGI mode, profile, quality, budgets and fallback reason are projected from provider-resolved settings; the Editor does not recompute those values from the request. Missing or removed HGI state becomes `unavailable` in the same frame instead of retaining stale settings.

## Validation Expectations

A real product acceptance run requires all of the following:

- a visible project window rather than an auxiliary zero-sized host;
- at least one runtime viewport;
- a submitted frame count greater than zero;
- provider-resolved HybridGI settings in Runtime Diagnostics;
- an explicit fallback reason when a baked Profile runs without the Plan 11 baked contract;
- no typed submission error in the run log.

Unit and source-contract tests protect ownership, but they do not replace the actual Editor PNG acceptance under `docs/tests/runtime/render`.
