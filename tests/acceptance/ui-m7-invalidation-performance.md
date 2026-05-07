---
related_code:
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/tests.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
implementation_files:
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
plan_sources:
  - docs/superpowers/plans/2026-05-06-ui-m7-invalidation-performance.md
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md
  - .codex/plans/Editor 绘制与鼠标事件优化计划.md
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetReflector.cpp
tests:
  - rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/surface/diagnostics.rs" "zircon_runtime_interface/src/ui/surface/frame.rs" "zircon_runtime/src/ui/tree/hit_test.rs" "zircon_runtime/src/ui/surface/frame_hit_test.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/diagnostics.rs" "zircon_runtime/src/ui/tests/hit_grid.rs" "zircon_runtime/src/ui/tests/diagnostics.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs" "zircon_editor/src/ui/slint_host/host_contract/presenter.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs"
  - rustfmt --edition 2021 --check "zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs"
  - rustfmt --edition 2021 --check --config skip_children=true "zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs"
  - cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-m7 --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-m7 --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-m7 --message-format short --color never
  - cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-m7 --message-format short --color never
  - cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-m7 --message-format short --color never
  - cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-m7 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m7-final --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m7-final --message-format short --color never
  - rustfmt --edition 2021 --check zircon_editor/src/ui/workbench/debug_reflector/model.rs zircon_editor/src/ui/workbench/debug_reflector/tests.rs zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs zircon_editor/src/ui/slint_host/ui/apply_presentation.rs zircon_editor/src/ui/slint_host/ui.rs zircon_editor/src/ui/slint_host/mod.rs zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs zircon_runtime/src/ui/tests/event_routing.rs
  - cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never
  - cargo test -p zircon_runtime --lib repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never
  - cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never
doc_type: testing-guide
---

# UI M7 Invalidation Performance Acceptance

## Scope

M7 closes the runtime/editor diagnostics slice that was identified in the Unreal Slate audit. The accepted behavior is intentionally scoped to retained UI surfaces and the Rust-owned editor host:

- Runtime surface frames expose the last rebuild report with dirty reasons, dirty node counts, cache sizes, and phase timings.
- `UiSurfaceFrame` hit tests borrow the cached `UiHitTestGrid` instead of cloning it per query.
- Shared debug snapshots include rebuild stats and deterministic material batch break reasons.
- Runtime Diagnostics can build a live Debug Reflector model from its current host-projected pane body surface instead of showing only the authored no-active placeholder.
- Repeated same-target pointer moves are ignored as pointer-only no-ops and do not mutate dirty flags, requested damage, component events, or surface rebuild reports.
- The editor diagnostics overlay uses one shared top-right marker geometry helper for both painting and region damage expansion.
- Changed overlay text expands an existing region repaint once, while unchanged text leaves damage untouched and full paints stay full paints.

## Evidence To Record

Record exact command outputs here after the M7 validation stage runs. A passing line means the command exited successfully in this session; warning noise is preserved in the command output and summarized below.

## Validation Log

- 2026-05-06 08:07 +08:00: `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/surface/diagnostics.rs" "zircon_runtime_interface/src/ui/surface/frame.rs" "zircon_runtime/src/ui/tree/hit_test.rs" "zircon_runtime/src/ui/surface/frame_hit_test.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/diagnostics.rs" "zircon_runtime/src/ui/tests/hit_grid.rs" "zircon_runtime/src/ui/tests/diagnostics.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs" "zircon_editor/src/ui/slint_host/host_contract/presenter.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs"`: passed with no output after formatting the M7-owned snippets.
- 2026-05-06 08:12 +08:00: `rustfmt --edition 2021 --check "zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs"`: passed with no output.
- 2026-05-06 08:12 +08:00: `rustfmt --edition 2021 --check --config skip_children=true "zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs"`: passed with no output; this avoided formatting unrelated active sibling painter child files.
- 2026-05-06 08:00 +08:00: `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never`: passed.
- 2026-05-06 08:01 +08:00: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never`: passed with existing warning noise in math/graphics support modules.
- 2026-05-06 08:08 +08:00: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never`: passed with existing warning noise plus unused Widget Reflector exports from the active reflector session.
- 2026-05-06 08:03 +08:00: `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never`: passed, `11 passed; 0 failed; 842 filtered out`.
- 2026-05-06 08:03 +08:00: `cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never`: passed, `16 passed; 0 failed; 837 filtered out`.
- 2026-05-06 08:11 +08:00: `cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never -- --nocapture`: passed, `8 passed; 0 failed; 1015 filtered out`.
- 2026-05-06 08:16 +08:00: `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never`: passed, `1 passed; 0 failed; 1022 filtered out`.
- 2026-05-06 09:40 +08:00: `cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never`: passed after sibling render DTO convergence, `43 passed; 0 failed; 3 filtered out`.
- 2026-05-06 12:12 +08:00: `cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-ui-m7-final" --message-format short --color never -- --nocapture`: passed, `8 passed; 0 failed; 1027 filtered out`.
- 2026-05-06 12:28 +08:00: `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-ui-m7-final" --message-format short --color never`: passed, `1 passed; 0 failed; 1036 filtered out`. An earlier retry on the same target failed before test execution while the active UI Debug Reflector session was mid-editing `zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs`; the source settled and this rerun passed without M7 patching reflector-owned files.
- 2026-05-07 08:23 +08:00: `rustfmt --edition 2021 --check zircon_editor/src/ui/workbench/debug_reflector/model.rs zircon_editor/src/ui/workbench/debug_reflector/tests.rs zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs zircon_editor/src/ui/slint_host/ui/apply_presentation.rs zircon_editor/src/ui/slint_host/ui.rs zircon_editor/src/ui/slint_host/mod.rs zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs zircon_runtime/src/ui/tests/event_routing.rs`: passed with no output.
- 2026-05-07 08:23 +08:00: `cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never`: passed, `14 passed; 0 failed; 1091 filtered out`. This includes the live Runtime Diagnostics body-surface projection check and expanded model detail assertions for visibility, input policy, focus/capture, hit path, reject reason, material batch breaks, and dirty flags.
- 2026-05-07 08:23 +08:00: `cargo test -p zircon_runtime --lib repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never`: passed, `1 passed; 0 failed; 943 filtered out`.
- 2026-05-07 08:23 +08:00: `cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never`: passed, `17 passed; 0 failed; 927 filtered out`.
- 2026-05-07 08:23 +08:00: `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never`: passed, `12 passed; 0 failed; 932 filtered out`.
- 2026-05-07 08:23 +08:00: `cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never`: passed, `57 passed; 0 failed; 6 filtered out`.
- 2026-05-07 08:23 +08:00: `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never`: passed.
- 2026-05-07 08:23 +08:00: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never`: passed with existing runtime warning noise.
- 2026-05-07 08:23 +08:00: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never`: passed with existing runtime/editor warning noise.

## Known Boundaries

- M7 does not replace the editor host layout system or introduce a new renderer pass.
- Runtime draw-call counts remain CPU-side estimates until backend-submitted counters are added.
- Runtime Diagnostics now reflects its own host-projected pane body surface during presentation conversion; arbitrary external active-surface selection remains a later UI workflow.
- Existing sibling UI sessions own broader Material layout, native text/input, and neutral render DTO work; this acceptance file records only the invalidation/performance/reflector slice.
- Full workspace validation was not run for M7 because multiple active sibling sessions were editing and validating overlapping UI/runtime targets. The evidence above is intentionally scoped to the M7 runtime/interface/editor checks and focused tests.
