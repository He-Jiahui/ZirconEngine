# MUI All Components Milestone 0

## Scope
- Reconciled the Material UI component matrix planning artifacts for MUI v9.0.1 all-components scope.
- Repaired the MUI X Date and Time Pickers prototype so it participates in the existing Material Component Lab contract instead of bypassing it.
- Affected layers: docs inventory, editor `.zui` prototype inventory, Material Lab shell/imports, boundary tests, builtin bindings, and runtime component catalog mapping.

## Baseline
- The matrix already tracked the 63 Core/Utils/Lab rows and 10 MUI X rows selected for this project scope.
- Date and Time Pickers was added as the eleventh MUI X row and must now satisfy the same Material Lab card, visibility, feedback-route, binding, and catalog mapping contracts as nearby MUI X prototypes.
- Windows native Cargo remains blocked outside this task by the known `wgpu-hal` DX12 / `windows` crate mismatch.
- The repository had unrelated dirty files before this slice; validation below is scoped to the MUI Milestone 0 files.

## Test Inventory
- Focused editor boundary tests: `material_ui_component_design_matrix` and `material_component_lab`.
- Boundary case: every explicit backticked `material_*.zui` filename in the matrix must exist under `zircon_editor/assets/ui/editor/material_components`.
- Boundary case: the `Date and Time Pickers` MUI X row must document the MUI X public URL, local Date/Time picker fallback globs, `DateTimePickers`, and `material_mui_x_date_time_pickers.zui`.
- Boundary case: `material_mui_x_date_time_pickers.zui` must keep the Material Lab prototype card contract: fixed `104/120/140` root height, `title/meta/sample/state_strip` child order, eight state pills, and exactly one `MaterialLab/MuiXDateTimePickers/Submit` route on the visible sample.
- Boundary case: the lab view must import and mount `prototype_mui_x_date_time_pickers` after Data Grid and before aggregate Charts, with MUI X and total counts updated to 11 and 74.
- Binding/catalog case: builtin Material Lab bindings must register the Date and Time Pickers submit route, and the runtime Material foundation catalog must expose `DateTimePickers`.
- Negative coverage: missing or renamed concrete `.zui` files now fail the matrix boundary test.

## Planned Validation Gates
- Controller-owned Cargo gate: run the focused editor boundary tests for `material_ui_component_design_matrix` and `material_component_lab` with `--locked`.
- Controller-owned catalog gate: run the runtime/editor catalog or Material foundation focused tests that cover `DateTimePickers` mapping.
- Controller-owned static gate: confirm the lab TOML imports/mounts every `material_*.zui` file exactly once and that all authored `MaterialLab/*` event ids have builtin bindings.
- Controller-owned formatting/static gate: run focused formatting or diff checks for the touched Rust/docs/assets if required by the validation harness.

## Results
- 2026-05-17 WSL target cleanup: `cargo clean --target-dir /mnt/e/cargo-targets/zircon-shared/wsl-mui-plan` ran before Cargo validation because `/mnt/e` had 25 GiB free; it removed 4570 files / 8.5 GiB and left 33 GiB free.
- 2026-05-17 WSL matrix gate passed: `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-plan cargo test -p zircon_editor --lib material_ui_component_design_matrix --locked --jobs 1` finished with 4 passed, 0 failed, 1360 filtered out.
- 2026-05-17 WSL Material Lab gate initially reached assertions and failed because `material_buttons.zui` had Button state-strip pills while the documented lab contract requires non-dispatchable Label pills. The asset was repaired by changing only the eight Button state-strip pill nodes back to `Label`; the routed `buttons_Sample` remains a `Button`.
- 2026-05-17 WSL Material Lab gate passed after the asset repair: `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-plan cargo test -p zircon_editor --lib material_component_lab --locked --jobs 1` finished with 27 passed, 0 failed, 1337 filtered out.
- 2026-05-17 WSL runtime catalog rerun first exposed a lower shared asset aggregate export drift: `zircon_runtime/src/asset/mod.rs` re-exported `MaterialTextureSlotValue`, `ShaderTextureSlotAsset`, `ZMaterialDocument`, and `ZShaderTextureSlotDocument`, but `zircon_runtime/src/asset/assets/mod.rs` did not expose those child-module DTOs. The fix was limited to the aggregate re-export list; no shader/material schema or importer behavior was changed.
- 2026-05-17 WSL runtime catalog gate passed after that shared-layer fix: `CARGO_TARGET_DIR=/mnt/e/cargo-targets/zircon-shared/wsl-mui-plan cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1` finished with 45 passed, 0 failed, 1502 filtered out. Existing warnings were limited to unused `required_nonzero_isize` / `optional_nonzero_isize` helpers in render backend/UI surface files.
- 2026-05-17 static hygiene passed: native `rustfmt --edition 2021 --check` and WSL `rustfmt --edition 2021 --check` both passed for the touched Rust test/binding/catalog files. WSL `rustfmt` was installed with `rustup component add rustfmt` because it was missing from the stable toolchain.
- 2026-05-17 focused whitespace checks passed: `git diff --check -- ...` reported only LF/CRLF warnings, and WSL `grep -n -E "[[:blank:]]+$"` found no trailing whitespace in the touched `.zui`, acceptance, and session-note files.

## Acceptance Decision
- Accepted for the Milestone 0 editor Material Lab / matrix contract surface and the runtime Material foundation catalog gate.
- Reason: the in-scope matrix, Material Lab, and runtime catalog validation gates passed after repairing the Button state-strip drift and the lower shared asset aggregate export drift. The new Date and Time Pickers prototype is covered by visible placement, feedback-route, binding, and catalog-mapping assertions in the passing suites.
- Remaining risk: workspace-wide formatting cannot be claimed clean until unrelated active-session rustfmt diffs are resolved by their owning sessions; focused `git diff --check` for the touched asset aggregate re-export passed with LF/CRLF warnings only.
- Remaining risk: Windows native Cargo remains blocked outside this task by the known `wgpu-hal` DX12 / `windows` crate mismatch.
