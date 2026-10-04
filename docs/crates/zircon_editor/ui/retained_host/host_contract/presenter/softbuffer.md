---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/backbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/counters.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/planned_present.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/planned_present/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/planned_present/outcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/summary.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present/log.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present/submit.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io/copy.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io/damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io/size.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/backbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/counters.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/planned_present.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/planned_present/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/planned_present/outcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/summary.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present/log.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present/submit.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io/copy.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io/damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io/size.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - softbuffer presenter subtree ownership scan
  - softbuffer diagnostics counters/overlay/planned-present/summary ownership scan
  - softbuffer planned-present model/outcome ownership scan
  - softbuffer present log/submit ownership scan
  - softbuffer surface-io copy/damage/size ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Softbuffer Presenter

`presenter/softbuffer.rs` owns the fallback native presenter state and the `HostChromePresenter` trait bridge. It keeps the `softbuffer::Context`/`Surface`, reusable `HostRgbaFrame` backbuffer, frozen native-resize raster snapshot, refresh diagnostics, and logging cache, while delegating lifecycle, present orchestration, and repaint mechanics to folder-backed child modules.

This backend is intentionally a fallback path. Normal native windows should use `GpuChromePresenter`; softbuffer remains the CPU-compatible presenter for fallback, tests, snapshots, and platform recovery. It must consume the same neutral chrome command stream as GPU presentation instead of owning a separate draw model.

## Child Modules

`softbuffer/lifecycle.rs` owns presenter creation and size-changing resize reset behavior. It creates the softbuffer context/surface, clamps and applies surface size, captures an existing backbuffer when reconfiguration starts an interactive resize transaction, discards later scaled backbuffers without replacing that source, and resets overlay text when the pixel surface changes.

`softbuffer/present.rs` owns both present paths. Ordinary presentation clears any native-resize snapshot, plans diagnostics, builds the neutral `ChromeCommandStream`, and repaints the reusable backbuffer. Interactive native resize always follows one acquisition order: reuse an existing transaction snapshot, otherwise capture the current backbuffer even when surface size is unchanged, otherwise build exactly one fallback snapshot. It then scales that source directly into the current softbuffer surface without rebuilding commands or CPU scene raster. The next ordinary present therefore performs the final fresh full paint.

`present/log.rs` owns verbose present diagnostics, duplicate-log suppression, and present-summary cache updates. `present/submit.rs` owns the softbuffer handoff: selecting the repainted backbuffer, copying RGBA bytes into the platform buffer, sending `pre_present_notify`, and choosing full present versus damage present.

`softbuffer/backbuffer.rs` owns reusable-frame repaint policy and the first-snapshot capture invariant. It decides whether regional repaint is valid for the current surface, applies command-stream region replay when possible, falls back to full command-stream frame paint, and reports the resulting painted-pixel counts.

`softbuffer/diagnostics.rs` is the diagnostics module entry. `diagnostics/planned_present.rs` decides whether requested damage can remain regional and records full/region paint counters. `planned_present/model.rs` owns the `PlannedPresent` result and cloned-presentation debug overlay update, while `planned_present/outcome.rs` owns repaint-outcome accounting for full versus region paint. `diagnostics/overlay.rs` expands damage for same-frame refresh overlay text changes. `diagnostics/counters.rs` records chrome command stream patch/full counters. `diagnostics/summary.rs` builds verbose diagnostic frame and presentation summaries.

`softbuffer/surface_io.rs` is the structural platform-buffer I/O entry. `surface_io/copy.rs` owns ordinary RGBA copy plus direct native-resize raster scaling; the latter computes each axis quotient/remainder once and uses only addition/comparison inside the pixel loop. `surface_io/damage.rs` owns damage-to-pixel bounds, damage pixel counting, and softbuffer damage rect conversion, and `surface_io/size.rs` owns current window size clamping plus softbuffer resize.

`softbuffer/tests.rs` owns softbuffer copy/scaling, first-snapshot retention, native-resize product wiring, damage rect, overlay expansion, and diagnostics planning regressions.

The parent file therefore stays focused on presenter state and trait-facing delegation. New present sequencing belongs in `present.rs`; new resize/setup behavior belongs in `lifecycle.rs`; new repaint policy belongs in `backbuffer.rs`; new damage planning or overlay accounting belongs in `diagnostics.rs`; new platform copy/resize rules belong in `surface_io.rs`.

## Validation Notes

The 2026-06-18 presenter subtree split reduced `softbuffer.rs` to a 62-line state/trait entry. Production ownership is now split across `present.rs` 120 lines, `diagnostics.rs` 173 lines, `surface_io.rs` 71 lines, `backbuffer.rs` 49 lines, and `lifecycle.rs` 35 lines; `tests.rs` carries the moved 180-line regression body.

Evidence for this slice is formatting, softbuffer presenter subtree ownership scans, trailing-whitespace/diff checks, and scoped `zircon_editor` library type checks. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-21 softbuffer diagnostics counters/overlay/planned-present/summary split reduced `softbuffer/diagnostics.rs` from 188 lines to an 11-line structural re-export entry. `diagnostics/planned_present.rs` is 112 lines and owns present diagnostics planning plus repaint-outcome accounting, `diagnostics/overlay.rs` is 22 lines and owns overlay damage expansion, `diagnostics/counters.rs` is 22 lines and owns chrome command-stream perf counters, and `diagnostics/summary.rs` is 38 lines and owns frame/presentation summary formatting. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a softbuffer diagnostics counters/overlay/planned-present/summary ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 planned-present model/outcome split reduced `diagnostics/planned_present.rs` from 112 lines to a 76-line planning entry. `planned_present/model.rs` owns the `PlannedPresent` DTO plus overlay-text presentation clone update, and `planned_present/outcome.rs` owns repaint-outcome accounting and painted-pixel calculation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a softbuffer planned-present model/outcome ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 present log/submit split reduced `softbuffer/present.rs` from 126 lines to a 62-line present orchestration entry. `present/log.rs` is 47 lines and owns verbose diagnostic output plus log cache mutation; `present/submit.rs` is 28 lines and owns backbuffer-to-softbuffer copy and present-with-damage submission. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a softbuffer present log/submit ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 surface-io copy/damage/size split reduced `softbuffer/surface_io.rs` from 92 lines to an 11-line structural re-export entry. `surface_io/copy.rs` owns pixel copy, `surface_io/damage.rs` owns pixel bounds, pixel counts, and softbuffer damage rects, and `surface_io/size.rs` owns window-size clamping and softbuffer resize. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a softbuffer surface-io copy/damage/size ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
