# UI Debug Reflector Full Closure Acceptance

Plan source: `docs/superpowers/plans/2026-05-06-ui-debug-reflector-full-closure.md`

## Milestone 1: Shared Snapshot Schema

Implemented shared runtime-interface DTOs for the first debug-reflector schema expansion:

- `UI_SURFACE_DEBUG_SCHEMA_VERSION` and `UiSurfaceDebugCaptureContext`.
- Debug options for command records, hit cells, overdraw cells, and overlay primitives.
- Render command records and optional backend render counters.
- Hit-grid cell records.
- Stable `UiHitTestRejectReason` plus human-readable reject messages.
- Overdraw cell records.
- Invalidation, damage, event, and overlay primitive reports on `UiSurfaceDebugSnapshot`.
- Public re-exports from `zircon_runtime_interface::ui::surface`.

Validation commands:

```powershell
cargo test -p zircon_runtime_interface --lib ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `1 passed; 0 failed; 0 ignored; 0 measured; 36 filtered out`.

```powershell
cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --message-format short --color never
```

Result: passed on retry. The first attempt timed out while waiting for Cargo's build-directory lock and produced no compiler diagnostics; the retry finished successfully for `zircon_runtime_interface`.

Remaining schema risk:

- Milestone 1 only establishes serializable shared DTOs and the reject-code contract. Runtime population of command, hit-cell, overdraw-cell, overlay, pick, and export payloads remains in Milestone 2.

## Milestone 2: Runtime Snapshot Generation And Export

Implemented runtime snapshot population from `UiSurfaceFrame`:

- Command records are generated from render commands with stable ids, visible frames, material/batch keys, break reasons, draw-call estimates, text summaries, and image summaries.
- Hit-grid cell records include cell bounds, entry indices, and node ids.
- Hit-test rejects now use stable `UiHitTestRejectReason` codes and messages in runtime debug dumps.
- Overdraw records include sampled cell bounds, layer counts, and contributing node ids.
- Selection snapshots fill capture context and selected/clip overlay primitives.
- Pick snapshots fill capture context, retain `pick_hit_test`, derive selected top-hit node, and emit hit-path/rejected-bounds overlay primitives.
- Runtime JSON export uses `UiSurface::debug_snapshot_json(...)`; file I/O remains editor-owned.

Validation commands:

```powershell
cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --message-format short --color never
```

First result: failed in `surface_debug_snapshot_reports_reflector_render_batch_and_hit_grid_stats`. Root cause was an outdated expectation that styled button commands had `kind=Quad;unclipped;opaque;style`; current render extraction preserves text on the styled command, so the correct break reason is `kind=Quad;unclipped;opaque;text`. New Milestone 2 diagnostics tests passed in that run.

Second result: passed. Output showed `16 passed; 0 failed; 0 ignored; 0 measured; 832 filtered out`. Existing runtime warnings remain unrelated to this milestone.

```powershell
cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `11 passed; 0 failed; 0 ignored; 0 measured; 837 filtered out`. Existing runtime warnings remain unrelated to this milestone.

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never
```

Result: passed. Cargo initially waited for a build-directory lock, then finished successfully. Existing runtime warnings remain unrelated to this milestone.

Remaining runtime risk:

- Backend render counters are optional placeholders until editor/host render backends provide measured values.
- Damage report population is still empty in runtime snapshots; editor host damage integration is expected in later milestones.

## Milestone 3: Editor Debug Reflector Model And Pane

Implemented the first editor Debug Reflector consumer surface:

- `zircon_editor::ui::workbench::debug_reflector` now contains snapshot-derived model, selection, JSON export/import, and overlay toggle helpers.
- Runtime Diagnostics payloads now include reflector summary, node rows, detail lines, and export status.
- Runtime Diagnostics initially rendered the stable no-active-surface state because the pane payload build context did not provide an active shared `UiSurfaceFrame`; the 2026-05-07 M7 follow-up now refreshes the pane from its own host-projected `body_surface_frame` during presentation conversion.
- Template projection injects reflector payload fields as root attributes for `pane.runtime.diagnostics.body`.
- The Runtime Diagnostics body asset exposes Debug Reflector summary/export/detail/list anchors.
- Native host conversion now keeps Runtime Diagnostics in its own pane data bucket instead of reusing Module Plugins nodes.

Validation commands:

```powershell
cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --message-format short --color never
```

Initial result: the first cold compile attempts timed out, then the repo-local `target` run failed while writing incremental artifacts with `os error 112` (`disk space insufficient`). Per the Cargo target disk policy, `cargo clean --target-dir "target"` removed 8.0 GiB of Cargo artifacts, but the rebuild dropped E: below the policy threshold again. Subsequent Milestone 3 validation used `$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"`.

First shared-target result: failed after test execution with `7 passed; 1 failed`. Root cause was in the shared Runtime Diagnostics host conversion: it preserved the original template nodes and appended a rewritten copy, so the first `UiDebugReflectorSummary` node still contained the authored placeholder `UI Debug Reflector`. The fix changed `runtime_diagnostics.rs` to return one rewritten template-node set plus generated reflector rows.

Second shared-target result: passed. Output showed `8 passed; 0 failed; 0 ignored; 0 measured; 1023 filtered out`.

```powershell
cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --message-format short --color never
```

Result: passed with `$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"`. Output showed `13 passed; 0 failed; 0 ignored; 0 measured; 1018 filtered out`.

```powershell
cargo test -p zircon_editor --lib pane_body_documents --locked --jobs 1 --message-format short --color never
```

Result: passed with `$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"`. Output showed `8 passed; 0 failed; 0 ignored; 0 measured; 1023 filtered out`.

```powershell
cargo test -p zircon_editor --lib slint_runtime_diagnostics_template_body --locked --jobs 1 --message-format short --color never
```

Result: passed with `$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"`. Output showed `1 passed; 0 failed; 0 ignored; 0 measured; 1030 filtered out`.

```powershell
cargo test -p zircon_editor --lib pane_payload_projection --locked --jobs 1 --message-format short --color never
```

Result: passed with `$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"`. Output showed `2 passed; 0 failed; 0 ignored; 0 measured; 1029 filtered out`.

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
```

Result: passed with `$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"`. Output showed `Finished dev profile`. Existing runtime warnings remain, and new editor dead-code warnings remain for Debug Reflector helpers that are intentionally test-covered before the live active-surface and overlay wiring lands.

Remaining editor panel risk:

- Runtime Diagnostics now reflects its own host-projected pane body surface, but an arbitrary external active pane/surface selector is still a later workflow. The pane no longer needs to fabricate geometry from host rectangles for its own Debug Reflector section.

## Milestone 4: Snapshot-Derived Debug Overlay

Implemented the first native overlay path for the editor Debug Reflector:

- `EditorUiDebugReflectorOverlayState` filters shared `UiDebugOverlayPrimitive` records by overlay kind.
- Damage overlays can be derived from `UiDamageDebugReport.damage_region` when the snapshot has damage data but no explicit damage primitive.
- `RuntimeDiagnosticsPanePayload` carries a single `ui_debug_reflector_overlay_primitives` vector instead of adding per-kind host fields.
- Host conversion maps shared primitives into `RuntimeDiagnosticsPaneData.overlay_primitives`.
- `debug_reflector_overlay.rs` draws clipped native rectangles/labels for selected frames, clip frames, hit cells, hit paths, rejected bounds, overdraw cells, material batch bounds, and damage regions.
- `workbench.rs` remains a call site: overlay drawing happens after normal pane content and before the top-right refresh marker, without changing layout, hit testing, render extraction, or dispatch state.

Validation commands:

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `11 passed; 0 failed; 0 ignored; 0 measured; 1026 filtered out`. Existing runtime warnings remain unrelated to this milestone.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `13 passed; 0 failed; 0 ignored; 0 measured; 1025 filtered out`. Existing runtime warnings remain unrelated to this milestone.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib shell_window --locked --jobs 1 --message-format short --color never
```

First result: failed before editor test execution with `zircon_runtime_interface::ui::layout` import errors in `slot.rs` / `mod.rs`. Coordination showed an active sibling session (`20260506-1215-ui-linear-slot-sizing-contract`) editing the same lower shared layout contract. No reflector code was changed for this failure.

Second result: the command progressed past the shared layout import failure and timed out during a cold dependency rebuild after 15 minutes.

Third result with a longer timeout: passed. Output showed `13 passed; 0 failed; 0 ignored; 0 measured; 1025 filtered out`.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `Finished dev profile`. Existing runtime warnings remain, and editor dead-code warnings remain for reflector helpers that are test-covered before live active-surface wiring lands.

Remaining overlay risk:

- Runtime Diagnostics overlay primitives can now be generated from the pane's own host-projected body surface; selecting and replaying another pane's snapshot remains a later UI workflow.
- The first overlay closure is CPU/native painter based. It accepts sampled overdraw primitives from the shared snapshot and does not implement a GPU overdraw pass.

## Milestone 5: Documentation, Acceptance, And Final Validation

Documentation and acceptance records were updated for the final debug-reflector closure:

- `docs/zircon_runtime/ui/surface/diagnostics.md` documents the shared/runtime snapshot source, export payload, hit reject codes, overdraw records, and current runtime damage/backend-counter limits.
- `docs/zircon_editor/ui/workbench/debug_reflector.md` documents the editor model, selection/export helpers, Runtime Diagnostics projection, and snapshot-derived overlay path.
- `docs/ui-and-layout/slate-style-ui-surface-frame.md` records that `UiSurfaceDebugSnapshot.overlay_primitives` is the only editor reflector overlay source.
- `docs/ui-and-layout/index.md` routes UI/Layout readers to the Debug Reflector overlay coverage through the Slate-style surface-frame document.

Final scoped validation commands:

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo test -p zircon_runtime_interface --lib ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out`.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `17 passed; 0 failed; 0 ignored; 0 measured; 871 filtered out`. Existing runtime warnings remain unrelated to this milestone.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `11 passed; 0 failed; 0 ignored; 0 measured; 877 filtered out`. Existing runtime warnings remain unrelated to this milestone.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `11 passed; 0 failed; 0 ignored; 0 measured; 1027 filtered out`. Existing runtime/editor warnings remain unrelated to this milestone.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `13 passed; 0 failed; 0 ignored; 0 measured; 1025 filtered out`. Existing runtime/editor warnings remain unrelated to this milestone.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `Finished dev profile`.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never
```

First result: failed in sibling-owned runtime input code with `E0027` because `UiDispatchEffect::Popup` was compiled with a transient `target` field while `host_request_for_effect(...)` destructured the earlier field set. Fresh coordination showed the active `20260506-0446-ui-complete-input-events` session editing `zircon_runtime_interface/src/ui/dispatch` and `zircon_runtime/src/ui/surface/input` at the same time. No reflector code was changed for this lower-layer mismatch.

Second result after the active input DTO state settled: passed. Output showed `Finished dev profile`. Existing runtime warnings remain unrelated to this milestone.

```powershell
$env:CARGO_TARGET_DIR = "D:\cargo-targets\zircon-shared"
cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
```

Result: passed. Output showed `Finished dev profile`. Existing runtime warnings remain, and editor dead-code warnings remain for reflector helpers that are test-covered before live active-surface wiring lands.

Workspace-wide validation follow-up:

- The root validator was attempted after the scoped milestone checks with `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir "D:\cargo-targets\zircon-shared" -VerboseOutput`. The workspace build leg completed, but the workspace test leg failed in `zircon_editor --lib`.
- A direct editor-lib repro showed multiple failures on the integrated dirty tree. Active coordination identified sibling-owned failures in the native idle redraw/window lane, Runtime Material import/preview lane, and Material/global surface lane.
- The unowned lower-layer failure was the host-scene conversion path for host-owned native pane payloads. `to_host_contract_pane(...)` previously used pane kind strings as the only payload authority for several native-body buckets, so synthetic host-scene pane kinds could drop valid `UiAssetEditor`, animation, hierarchy, inspector, console, assets, asset-browser, and project-overview payloads before rebuilding `PaneData.body_surface_frame`.
- The fix keeps the fast path narrow: pane kind still selects the normal authored pane, but a non-empty host-owned native-body bucket is now also enough to preserve that payload through the host contract conversion. No test-only branch or compatibility shim was added.

Follow-up validation commands:

```powershell
rustfmt --edition 2021 --check "zircon_editor/src/ui/slint_host/ui/apply_presentation.rs"
```

Result: passed.

```powershell
cargo test -p zircon_editor --lib host_scene_projection_converts_host_owned_panes_to_host_contract_panes --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Result: passed.

```powershell
cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Result: passed. Output showed `11 passed; 0 failed; 0 ignored; 0 measured` for the filtered Debug Reflector checks.

```powershell
cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Result: passed. Output showed `17 passed; 0 failed; 0 ignored; 0 measured` for the filtered native host contract checks.

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Result: passed with existing warning noise.

2026-05-07 M7 live Runtime Diagnostics follow-up:

```powershell
cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never
```

Result: passed. Output showed `14 passed; 0 failed; 1091 filtered out`. This run includes `runtime_diagnostics_live_body_surface_populates_debug_reflector_rows_and_overlays`, proving the Runtime Diagnostics pane can derive Debug Reflector rows and overlay primitives from its own host-projected `body_surface_frame`.

```powershell
cargo test -p zircon_runtime --lib repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never
```

Result: passed. Output showed `1 passed; 0 failed; 943 filtered out`, proving 100 repeated same-target mouse moves do not dirty, damage, emit component events, or change the surface rebuild report.

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets-ui-m7-current" --message-format short --color never
```

Result: passed with existing runtime/editor warning noise.

Remaining workspace risk:

- A fresh full editor-lib gate was attempted with `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`. It compiled and began executing `1068` tests, but the captured output contained `229` `... FAILED` lines across broad integrated dirty-tree clusters before the tool output ended without a final `test result:` summary.
- The failure clusters were outside the Debug Reflector projection fix: `tests::host::manager` (`68`), `tests::host::slint_callback_dispatch` (`52`), `tests::editor_event::runtime` (`43`), `tests::ui::component_adapter` (`13`), `tests::host::ui_asset_editor_theme_tooling` (`8`), runtime preview/UI asset (`5`), drawer/workbench layout/reflection, viewport-toolbar pointer, native-window/source-guard, and Material/runtime preview areas. These map to active sibling Slate/input/material/drawer/editor-manager lanes rather than this reflector closeout.
- The same editor-lib output kept the reflector-owned checks green: `runtime_diagnostics_template_body_projects_ui_debug_reflector_nodes`, both `tests::host::slint_window::ui_debug_reflector` tests, and `runtime_diagnostics_body_exposes_ui_debug_reflector_section` all reported `ok`; the native host contract tests that were previously scoped for this lane also reported `ok` in that run.
- A post-doc focused rerun of `host_scene_projection_converts_host_owned_panes_to_host_contract_panes` is currently blocked before test execution by active native input/window work: `zircon_editor/src/ui/slint_host/host_contract/window.rs:254` calls `UiBindingValue::string(...).native_binding()`, but `UiBindingValue` no longer exposes `native_binding()`. That file is owned by the active complete-input-events/native-window session, so this lane records the blocker instead of patching it.
- Full editor-lib, root workspace, and plugin workspace CI parity remain open until the sibling-owned integrated-tree blockers are resolved or intentionally accepted by their owning lanes.

Final accepted limitations:

- Live Runtime Diagnostics reflects the pane's own host-projected `body_surface_frame`; cross-pane active-surface selection and snapshot switching remain out of scope for this closure.
- Backend render counters remain optional placeholders.
- Runtime damage population is still limited; the editor overlay can draw snapshot damage regions when present.
- GPU overdraw visualization is out of scope for this closure; CPU sampled overdraw cells are the accepted first overlay source.
