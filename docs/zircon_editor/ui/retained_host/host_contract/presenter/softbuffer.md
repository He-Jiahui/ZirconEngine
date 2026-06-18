---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/backbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/backbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - softbuffer presenter subtree ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Softbuffer Presenter

`presenter/softbuffer.rs` owns the fallback native presenter state and the `HostChromePresenter` trait bridge. It keeps the `softbuffer::Context`/`Surface`, reusable `HostRgbaFrame` backbuffer, refresh diagnostics, and logging cache, while delegating lifecycle, present orchestration, and repaint mechanics to folder-backed child modules.

This backend is intentionally a fallback path. Normal native windows should use `GpuChromePresenter`; softbuffer remains the CPU-compatible presenter for fallback, tests, snapshots, and platform recovery. It must consume the same neutral chrome command stream as GPU presentation instead of owning a separate draw model.

## Child Modules

`softbuffer/lifecycle.rs` owns presenter creation and resize reset behavior. It creates the softbuffer context/surface, clamps and applies surface size, clears stale backbuffers after resize, and resets overlay text when the pixel surface changes.

`softbuffer/present.rs` owns present orchestration. It samples the current window size, plans diagnostics, builds the neutral `ChromeCommandStream`, records perf counters, asks the backbuffer module to repaint, emits verbose present diagnostics, copies the damaged frame region to the platform buffer, and submits softbuffer present damage.

`softbuffer/backbuffer.rs` owns reusable-frame repaint policy. It decides whether regional repaint is valid for the current surface, applies command-stream region replay when possible, falls back to full command-stream frame paint, and reports the resulting painted-pixel counts.

`softbuffer/diagnostics.rs` owns present planning for diagnostics. It decides whether requested damage can remain regional, expands damage for same-frame refresh overlay text changes, records full/region paint counters, updates the cloned presentation's debug overlay text, records chrome command stream patch/full counters, and builds verbose diagnostic summaries.

`softbuffer/surface_io.rs` owns platform-buffer mechanics: current window size clamping, softbuffer resize, damage-to-pixel bounds, damage pixel counting, softbuffer damage rect conversion, and RGBA-to-softbuffer pixel copy.

`softbuffer/tests.rs` owns the existing softbuffer copy, damage rect, overlay expansion, and diagnostics planning regressions that previously lived inline in the root presenter file.

The parent file therefore stays focused on presenter state and trait-facing delegation. New present sequencing belongs in `present.rs`; new resize/setup behavior belongs in `lifecycle.rs`; new repaint policy belongs in `backbuffer.rs`; new damage planning or overlay accounting belongs in `diagnostics.rs`; new platform copy/resize rules belong in `surface_io.rs`.

## Validation Notes

The 2026-06-18 presenter subtree split reduced `softbuffer.rs` to a 62-line state/trait entry. Production ownership is now split across `present.rs` 120 lines, `diagnostics.rs` 173 lines, `surface_io.rs` 71 lines, `backbuffer.rs` 49 lines, and `lifecycle.rs` 35 lines; `tests.rs` carries the moved 180-line regression body.

Evidence for this slice is formatting, softbuffer presenter subtree ownership scans, trailing-whitespace/diff checks, and scoped `zircon_editor` library type checks. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
