---
related_code:
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - docs/ui-and-layout/editor-host-final-cleanup.md
  - .codex/sessions/20260507-1502-ui-m8-final-validation.md
  - .codex/sessions/archive/20260507-1910-ui-m8-cleanup-continuation.md
  - zircon_editor/src/tests/ui/boundary/runtime_ui_golden.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_runtime/src/ui/tests/boundary.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - tests/acceptance/ui-debug-reflector-full-closure.md
  - tests/acceptance/ui-m7-invalidation-performance.md
implementation_files:
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/resize_surface.rs
  - zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_runtime/src/ui/tests/boundary.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
plan_sources:
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
tests:
  - cargo test -p zircon_editor --lib runtime_ui_golden --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never
  - cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --ignored --nocapture --test-threads=1
  - cargo build --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo test -p zircon_runtime_interface --lib manifest_dependencies_stay_contract_only --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never -- --nocapture
  - cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-lib-fresh-m8 --message-format short --color never
  - cargo test --workspace --locked --jobs 1 --target-dir D:\cargo-targets\zircon-root-fresh-m8 --message-format short --color never
  - cargo test --manifest-path zircon_plugins\Cargo.toml --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime --lib carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib material_icon_button_without_visual_icon_keeps_label_accessibility_only --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_fixture_assets_live_under_crate_assets --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never
  - cargo build --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never
  - cargo test --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never
  - cargo build --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-plugins --message-format short --color never
  - cargo test --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-plugins --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never
  - cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture
doc_type: acceptance-evidence
---

# UI M8 Final Cleanup Acceptance

## Scope

This file is the M8 final cleanup evidence index for the Slate audit milestone. It records the M8.1 stale geometry cleanup, M8.2 focused final gates, M8.3 broad workspace/plugin gates, and the final M8.T owner/risk closure.

Accepted evidence in this index covers:

- M8.1 stale host-coordinate cleanup evidence.
- Existing editor/runtime same-source `.ui.toml` semantic golden evidence.
- Existing Debug Reflector and runtime diagnostics snapshot evidence.
- Existing GUI screenshot artifacts that prove the M3 host-cutover visual gate.
- Fresh root workspace and plugin workspace broad validation.

Final M8.T status:

- No blocking owner remains after broad validation.
- Non-blocking residuals are existing warning noise, the shared dirty worktree from sibling sessions, and Cargo target-cache/disk hygiene for any future broad phase.

## M8.1 Cleanup Evidence

The current M8.1 cleanup evidence lives in the main milestone plan and `docs/ui-and-layout/editor-host-final-cleanup.md`.

Closed slices:

- `root_shell_projection.rs` no longer derives root frames, region frames, center/status frames, or splitter frames from `WorkbenchShellGeometry`.
- `shell_pointer/resize_surface.rs` no longer reads `WorkbenchShellGeometry.splitter_frames`; drawer resize hit targets consume shared root-shell splitter frames.
- `floating_window_projection.rs` no longer constructs shared sources or projection bundles from `WorkbenchShellGeometry` or legacy floating-window frame helpers.
- `shell_pointer/bridge.rs`, `shell_pointer/drag_surface.rs`, and `tab_drag/strip_hitbox.rs` no longer rebuild or consume root-frame fallback data from old host geometry.

Current focused M8.1 validation already recorded in the plan:

- `workbench_projection_cutover`: 8 passed / 0 failed.
- `slint_tab_drag`: 34 passed / 0 failed.
- `slint_drawer_resize`: 9 passed / 0 failed.
- Production `slint_host` source search found no matches for the old geometry/root-frame fallback strings listed in the plan.

Fresh production stale-path audit on 2026-05-07 11:26 +08:00:

- Production non-test `zircon_editor/src/ui/slint_host/**/*.rs`, excluding module-local `tests.rs`, had 0 matches for the old geometry/root-frame fallback patterns: `geometry.region_frame`, `geometry.splitter_frame`, `geometry.center_band_frame`, `geometry.status_bar_frame`, `geometry.viewport_content_frame`, `geometry.floating_window_frame`, `WorkbenchShellGeometry {`, `root_frames_from_geometry`, `shared_or_geometry_frame`, `shared_or_fallback_frame`, `derive_layout_frames_from_geometry`, `resolve_floating_window_projected_outer_frame_with_fallback`, `floating_window_projection_shared_source_from_geometry`, and `build_floating_window_projection_bundle(geometry`.
- Production `zircon_editor/src/ui/slint_host` plus `zircon_editor/assets/ui/editor/host` had 0 matches for stale drawer/menu/root callback patterns: `left_drawer_extent`, `right_drawer_extent`, `bottom_drawer_extent`, `set_*_menu_button_frame`, old UiAssetEditor `ui_asset_*selected/activated` callbacks, and pane-local `palette_selected(` / `binding_selected(` / `layout_semantic_selected(` callbacks.
- Module-local `app/tests.rs` and `ui/tests.rs` still contain 21 old-geometry fixture matches. Those are retained as stale inputs for regression coverage and are not production authority.

## Editor Runtime Golden Evidence

Existing same-source `.ui.toml` semantic golden coverage is implemented by `zircon_editor/src/tests/ui/boundary/runtime_ui_golden.rs`.

Covered fixture pairs:

- runtime HUD.
- pause dialog.
- settings dialog.
- inventory dialog.
- quest log dialog.

The M4.3/M4.T validation already recorded in the milestone plan shows:

- `cargo test -p zircon_editor --lib runtime_ui_golden --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture`: 2 passed / 0 failed.
- `cargo test -p zircon_runtime --lib runtime_ui_manager --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture`: 3 passed / 0 failed.
- `cargo test -p zircon_runtime --lib ui_boundary::assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture`: 2 passed / 0 failed.
- `cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture`: 9 passed / 0 failed.
- `cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture`: 11 passed / 0 failed.
- `cargo test -p zircon_runtime --lib render_framework_ --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture`: 33 passed / 0 failed.

M8.2 status: existing golden evidence is present and indexed here. The fresh M8 focused rerun on 2026-05-07 12:09 +08:00 passed with `2 passed; 0 failed`.

## Debug Snapshot Evidence

Existing debug snapshot evidence is split across:

- `tests/acceptance/ui-debug-reflector-full-closure.md`.
- `tests/acceptance/ui-m7-invalidation-performance.md`.
- `docs/zircon_editor/ui/workbench/debug_reflector.md`.
- `docs/zircon_runtime/ui/surface/diagnostics.md`.

The M7 validation already recorded in the milestone plan shows:

- `cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never`: 14 passed / 0 failed.
- `cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never`: 17 passed / 0 failed.
- `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never`: 12 passed / 0 failed.
- `cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never`: 57 passed / 0 failed.
- `cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never -- --nocapture`: 8 passed / 0 failed.
- `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never`: 1 passed / 0 failed.

M8.2 status: debug snapshot evidence is present and indexed here. The fresh M8 focused rerun on 2026-05-07 12:09 +08:00 passed with editor `ui_debug_reflector` 14 / 0, runtime `diagnostics` 17 / 0, and runtime `hit_grid` 12 / 0.

## GUI Screenshot Evidence

Existing GUI artifacts are the M3 screenshot gate outputs under `target/visual-layout/`:

- `editor-window-m3-welcome-input-900x620.png`.
- `editor-window-m3-workbench-900x620.png`.
- `editor-window-m3-asset-browser-900x620.png`.
- `editor-window-m3-assets-drawer-900x620.png`.
- `editor-window-m3-menu-popup-svg-icons-900x620.png`.
- `editor-window-m3-drag-after-release-900x620.png`.
- `editor-window-m3-svg-icon-scale-small-640x420.png`.
- `editor-window-m3-svg-icon-scale-large-1260x780.png`.

The recorded gate was:

```powershell
cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --ignored --nocapture --test-threads=1
```

Result: 1 passed / 0 failed, with artifacts refreshed on 2026-05-07 around 09:05 +08:00.

Fresh M8 screenshot rerun:

```powershell
cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --ignored --nocapture --test-threads=1
```

Result: 1 passed / 0 failed on 2026-05-07 12:09 +08:00. The eight `editor-window-m3-*.png` artifacts were refreshed between 2026-05-07 12:08 and 12:09 +08:00.

M8.2 status: GUI evidence is present as the current host-cutover screenshot baseline, and the fresh M8 focused screenshot gate passed.

## Fresh Focused Final Gates

Before the fresh focused gates, the active target directory was cleaned per Cargo target disk policy because E: free space was below 50GB:

```powershell
cargo clean --target-dir E:\zircon-build\targets-ui-m8-current
```

Result: removed 23.6GiB and E: free space returned to 57.9GB.

Fresh focused gates on `E:\zircon-build\targets-ui-m8-current`:

- `cargo test -p zircon_editor --lib runtime_ui_golden --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture`: 2 passed / 0 failed.
- `cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture`: 14 passed / 0 failed.
- `cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture`: 17 passed / 0 failed.
- `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture`: 12 passed / 0 failed.
- `cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --ignored --nocapture --test-threads=1`: 1 passed / 0 failed.

Existing runtime/editor warning noise remains unchanged and is not treated as an M8.2 failure.

## Index Maintenance Validation

2026-05-07 11:20 +08:00 maintenance checks for this evidence index:

- `git diff --check -- .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md docs/ui-and-layout/editor-host-final-cleanup.md .codex/sessions/20260507-0226-ui-milestone-continuation.md tests/acceptance/ui-m8-final-cleanup-acceptance.md`: passed. Git emitted only the existing LF-to-CRLF warning for `docs/ui-and-layout/editor-host-final-cleanup.md`.
- Trailing-whitespace search across the same four updated files returned no matches.
- Concurrent Cargo validation processes were observed after this docs-only update, including editor lib, root workspace build/test, and runtime hit-grid gates under `E:\zircon-build\targets-ui-m8-current` / `E:\zircon-build\targets-ui-m8-runtime`. They were not started by this evidence-index slice and are not counted as M8 acceptance evidence here because no terminal output or exit status is attached to this session.

## Broad Final Gates

The final broad gates ran on `D:\cargo-targets\zircon-m8-current` to avoid active sibling sessions writing E: target directories.

Root workspace gates:

- `cargo build --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`: exit 0, 18m33s.
- Initial `cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`: exit 101. The failure was `zircon_runtime_interface::tests::boundary::manifest_dependencies_stay_contract_only`, because the shared text DTO contract uses `unicode-segmentation` for grapheme boundaries but the dependency allowlist had not been updated.
- `rustfmt --edition 2021 --check zircon_runtime_interface/src/tests/boundary.rs`: passed.
- `cargo test -p zircon_runtime_interface --lib manifest_dependencies_stay_contract_only --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never -- --nocapture`: 1 passed / 0 failed.
- Rerun `cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`: exit 0. The `zircon_runtime_interface` harness finished with 63 passed / 0 failed.

Plugin workspace gates:

- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`: exit 0, 9m42s.
- `cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`: exit 0, 14m37s.
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`: exit 0, 11m38s.

Fresh M8 closeout reruns on isolated targets:

- `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-lib-fresh-m8 --message-format short --color never`: exit 0. Result: 1115 passed / 0 failed / 3 ignored.
- `cargo test --workspace --locked --jobs 1 --target-dir D:\cargo-targets\zircon-root-fresh-m8 --message-format short --color never`: exit 0.
- `cargo test --manifest-path zircon_plugins\Cargo.toml --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never`: exit 0.
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime --lib carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never -- --nocapture`: exit 0 after the fixture comment cleanup. Result: 1 passed / 0 failed.

One stale target directory produced a false editor lib-test compile split where `zircon_runtime_interface` appeared as two instances. A clean target reproduced the same editor test suite successfully, so the failure is recorded as target-cache contamination, not as an unresolved source-contract split. The contaminated `D:\cargo-targets\zircon-m8-current` and `D:\cargo-targets\zircon-editor-check-repro` directories were removed after resolving their paths under `D:\cargo-targets`.

Fresh current-target confirmation on 2026-05-07 15:12-15:45 +08:00:

- E: free space was below the project threshold. `cargo clean --target-dir E:\zircon-build\targets` removed 37.0GiB, and `cargo clean --target-dir E:\zircon-build\targets-ui-m6` removed 18.0GiB after both paths resolved under `E:\zircon-build`.
- `cargo test -p zircon_runtime --lib material_icon_button_without_visual_icon_keeps_label_accessibility_only --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture --test-threads=1`: 1 passed / 0 failed.
- `cargo test -p zircon_runtime --lib runtime_fixture_assets_live_under_crate_assets --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture --test-threads=1`: 1 passed / 0 failed.
- `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never`: 944 passed / 0 failed.
- `cargo build --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never`: exit 0. Cargo still emitted the existing `zircon_runtime.pdb` output filename collision warning.
- `cargo test --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never`: exit 0.
- From `zircon_plugins`, `cargo build --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-plugins --message-format short --color never`: exit 0.
- From `zircon_plugins`, `cargo test --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-plugins --message-format short --color never`: exit 0.

Scoped M8 continuation confirmation on 2026-05-07 19:10-19:38 +08:00:

- E: was below the 50GB Cargo target policy threshold, so the continuation used `D:\cargo-targets\zircon-ui-m8-current`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never`: exit 0.
- `cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture`: 9 passed / 0 failed.
- `cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture`: 8 passed / 0 failed.
- `cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture`: 34 passed / 0 failed.
- Final scoped `rustfmt --edition 2021 --check` over the touched M8 source/test file set passed with no output.
- Targeted production source guards for root-shell projection, shell-pointer, tab-drag, floating-window projection, drawer resize, and workspace docking returned 0 hits for the stale geometry/root-frame fallback patterns.

## M8.T Risk Record

M8 is accepted by the current evidence set. No broad gate has an unresolved owner.

Residual non-blocking signals:

- Existing runtime/editor/plugin warning noise remains.
- The shared worktree still contains sibling-session dirty and untracked files unrelated to this final cleanup evidence.
- Cargo target directories can produce stale cross-graph artifacts after repeated broad runs; future Cargo work should use isolated targets for final gates and clean only verified paths under the intended target root.
- E: is currently below the repository Cargo target free-space threshold; use D: or clean verified E: target directories before more broad validation.
