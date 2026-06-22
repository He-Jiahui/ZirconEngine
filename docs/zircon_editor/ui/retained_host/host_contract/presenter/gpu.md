---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/present.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/present.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - GPU presenter lifecycle/present/stats/geometry ownership scan
  - scoped trailing-whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# GPU Presenter

`presenter/gpu.rs` owns the GPU presenter state and the `HostChromePresenter` trait bridge. It keeps the runtime `UiSurfacePresenter`, current surface size, refresh diagnostics, last upload/draw-call counters, and direct-surface cache state.

## Child Modules

`gpu/lifecycle.rs` owns presenter construction, resize behavior, diagnostics snapshots, and last-upload/draw-call accessors. Resize clamps the surface size and invalidates the direct-surface cache so the next present rebuilds a full command stream.

`gpu/present.rs` owns present orchestration. It decides whether requested damage can be sent as a patch based on cache initialization, builds the neutral `ChromeCommandStream`, converts it to a runtime draw list, submits it to the RHI presenter, records painted-pixel diagnostics, and returns invalidation-merged refresh diagnostics.

`gpu/stats.rs` owns runtime present-stat fanout into host fields and UI perf counters: upload bytes, draw calls, visible commands, visible draw items, batch layers, batch dependencies, and command-stream patch/full rebuild counters.

`gpu/geometry.rs` owns size and pixel accounting helpers used by lifecycle and present diagnostics: clamp size, full-surface pixel count, and clipped damage pixel count.

`gpu/tests.rs` owns GPU presenter regressions for runtime surface failure propagation, upload/draw-call diagnostics, first-present cache bootstrap, patch damage after cache warmup, and resize cache invalidation.

## Validation Notes

The 2026-06-21 GPU presenter lifecycle/present/stats/geometry split reduced `presenter/gpu.rs` from 171 lines to a 44-line state/trait bridge. `gpu/present.rs` is 62 lines and owns GPU present orchestration, `gpu/lifecycle.rs` is 44 lines and owns construction/resize/accessors, `gpu/stats.rs` is 39 lines and owns RHI stat/perf-counter fanout, and `gpu/geometry.rs` is 23 lines and owns clamp/pixel accounting. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a GPU presenter lifecycle/present/stats/geometry ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
