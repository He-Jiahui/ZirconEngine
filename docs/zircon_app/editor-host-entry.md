---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/build.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/src/lib.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs
implementation_files:
  - zircon_app/Cargo.toml
  - zircon_app/build.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/src/lib.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs
plan_sources:
  - user: 2026-05-22 continue Editor/runtime UI layout visual validation
  - user: UI Layout 架构评审与 Taffy 收敛计划
  - user: 2026-05-25 complete live Editor visual rendering and 16px readability validation for wired editor_pages icons
tests:
  - cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-editor-visual-20260521-hostonly --message-format short --color never
  - real-window probe: target/editor-visual-check/editor-default-moveonly-20260522-042143.png
  - real-window probe: target/editor-visual-check/editor-material-lab-topmost2-20260522-035453.png
  - temporary stack probe: target/editor-visual-check/editor-default-960x640-stack8m-20260522-043217.png
  - rebuilt source validation: dumpbin reports 800000 size of stack reserve for zircon_editor.exe
  - rebuilt source validation: target/editor-visual-check/editor-default-960x640-rebuilt-stack8m-20260522-043929.png
  - live editor build: cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation
  - live editor screenshot: target/visual-layout/editor-live-window-900x620.png
  - rustfmt --edition 2021 --check --config skip_children=true zircon_editor/src/ui/retained_host/run_config.rs zircon_editor/src/ui/retained_host/mod.rs zircon_editor/src/lib.rs zircon_editor/src/ui/retained_host/app.rs zircon_editor/src/ui/retained_host/host_contract/globals/state.rs zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs zircon_editor/src/ui/retained_host/host_contract/window/tests.rs zircon_app/src/entry/entry_runner/editor.rs zircon_app/src/entry/tests/profile_bootstrap.rs
  - cargo test -p zircon_editor editor_host_run_config --lib --no-default-features --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-popup-preference-0704 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor first_presented_frame_exit_policy_defaults_off_and_can_be_enabled --lib --no-default-features --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-popup-preference-0704 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_app first_frame_exit_flag_projects_into_editor_host_config --lib --no-default-features --features target-editor-host --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-popup-preference-0704 --message-format short --color never -- --nocapture --test-threads=1
  - cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-popup-preference-0704 --message-format short --color never
  - bounded editor startup smoke: ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME=1 F:\cargo-targets\zircon-editor-popup-preference-0704\debug\zircon_editor.exe ExitCode 0
doc_type: module-detail
---

# Zircon App Editor Host Entry

`zircon_app` owns the process entry for the native editor host. `src/bin/editor.rs` delegates to `EntryRunner::run_editor_with_args`, which parses diagnostic and startup arguments, creates one `ProductComposition`, loads the default runtime dynamic library, creates a runtime client, and hands control to the retained editor host. The composition retains Core, module/plugin receipts, and plugin owners until the editor host returns, then releases it before the dynamic runtime session. Normal startup remains interactive; validation-only startup may use `zircon_editor::run_editor_with_config` to request a bounded first presented frame.

## Runtime Profile Build Boundary

The live Editor visual validation path builds the same `zircon_editor` binary used for the retained host instead of relying only on library tests. That requires app bootstrap to reach the runtime-profile module selection helpers through the stable runtime crate root. `zircon_runtime/src/lib.rs` therefore re-exports the manifest-specific runtime-profile helper APIs, while the implementations remain owned by `zircon_runtime::builtin`.

`EntryConfig::resolve()` owns the single request-to-runtime-profile projection. It merges the optional caller manifest with the `RuntimeProfileDescriptor` baseline and records provenance in `ResolvedProductHostConfig`; `builtin_modules.rs` and first-party provider projection accept only that result. This is entry/profile wiring only and does not move runtime module ownership into `zircon_app`.

The current live validation command is `cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation`. The 2026-05-25 closeout rerun of that command passed. The captured native window artifact is `target/visual-layout/editor-live-window-900x620.png`; the requested capture name is 900 x 620, while the actual OS window PNG reported `1296 x 759` and `86492` bytes.

## Bounded First-Frame Startup Smoke

The Frameworks02 editor startup smoke uses `ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME` as a validation-only policy. `zircon_app/src/entry/entry_runner/editor.rs` owns the environment projection, builds an `EditorHostRunConfig`, and passes that config to `zircon_editor::run_editor_with_config(...)`.

`EditorHostRunConfig` stays in `zircon_editor/src/ui/retained_host/run_config.rs` so startup requests and validation flags do not accumulate in the retained-host root. `UiHostWindow` stores the default-off policy in host globals through `window/lifecycle.rs`, and `window/event_loop/redraw/present.rs` exits only after `presenter.present(...)` succeeds and refresh diagnostics have been updated. This keeps the smoke tied to an actual presented frame and does not change normal editor UX.

The same target-editor-host validation also updated `profile_bootstrap.rs` to use `asset_manager.shared().pipeline_info()`. That preserves the current F18 asset-manager hard cutover and does not reintroduce a legacy `AssetManagerHandle.pipeline_info()` forwarding API.

## Windows Stack Reserve

The native editor host performs retained UI recompute, template projection, layout, host DTO conversion, and native painting on the Windows event-loop thread. The default UI Component Showcase is currently the deepest real editor page in that path. Real-window validation on 2026-05-22 showed that the unmodified Windows/MSVC binary could open the default page and Material Component Lab, but resizing the default page to 960 x 640 exited with a stack-overflow class failure before the second post-resize presentation commit.

`build.rs` reserves an 8 MB stack for the `zircon_editor` binary on Windows/MSVC only:

- The setting is emitted with `cargo:rustc-link-arg-bin=zircon_editor=/STACK:8388608`.
- It does not apply to `zircon_runtime`, non-MSVC targets, or library crates.
- It preserves the existing retained host architecture; it is a host process budget fix, not a layout algorithm fallback.

The temporary PE-header probe that used the same built editor exe plus `/STACK:8388608` stayed alive after the same 960 x 640 resize and produced `target/editor-visual-check/editor-default-960x640-stack8m-20260522-043217.png` with empty stdout/stderr. That probe is diagnostic evidence for the linker setting; the committed implementation is the Cargo build-script rule.

## Validation Notes

The 2026-05-22 focused build before the build-script change passed with the locked/offline host-only target dir listed in the header. After the local workspace gained unrelated plugin lockfile drift, `--locked` validation was no longer usable without changing the root lockfile. The follow-up offline build restored `Cargo.lock` after Cargo's temporary resolution update, rebuilt `zircon_editor` from source, and `dumpbin /headers` reported `800000 size of stack reserve`. The rebuilt binary then survived the 960 x 640 default resize probe and produced `target/editor-visual-check/editor-default-960x640-rebuilt-stack8m-20260522-043929.png` with no stdout/stderr diagnostics.
