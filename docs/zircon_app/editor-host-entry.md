---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/build.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/src/lib.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
implementation_files:
  - zircon_app/Cargo.toml
  - zircon_app/build.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/src/lib.rs
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
doc_type: module-detail
---

# Zircon App Editor Host Entry

`zircon_app` owns the process entry for the native editor host. `src/bin/editor.rs` delegates to `EntryRunner::run_editor_with_args`, which parses diagnostic and startup arguments, bootstraps the editor profile, loads the default runtime dynamic library, creates a runtime client, and hands control to `zircon_editor::run_editor_with_startup_request`.

## Runtime Profile Build Boundary

The live Editor visual validation path builds the same `zircon_editor` binary used for the retained host instead of relying only on library tests. That requires app bootstrap to reach the runtime-profile module selection helpers through the stable runtime crate root. `zircon_runtime/src/lib.rs` therefore re-exports the manifest-specific runtime-profile helper APIs, while the implementations remain owned by `zircon_runtime::builtin`.

`zircon_app/src/entry/builtin_modules.rs` keeps the project manifest optional until the resolver path is known. When `EntryConfig.runtime_profile()` is set, the plugin-registration and feature-registration paths use the caller manifest when present and otherwise ask `RuntimeProfileDescriptor` for the profile default manifest. The feature-registration path clones the optional manifest before creating the fallback so later feature dependency reporting can still inspect the original optional manifest. This is entry/profile wiring only; it does not move runtime module ownership into `zircon_app`.

The current live validation command is `cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation`. The 2026-05-25 closeout rerun of that command passed. The captured native window artifact is `target/visual-layout/editor-live-window-900x620.png`; the requested capture name is 900 x 620, while the actual OS window PNG reported `1296 x 759` and `86492` bytes.

## Windows Stack Reserve

The native editor host performs retained UI recompute, template projection, layout, host DTO conversion, and native painting on the Windows event-loop thread. The default UI Component Showcase is currently the deepest real editor page in that path. Real-window validation on 2026-05-22 showed that the unmodified Windows/MSVC binary could open the default page and Material Component Lab, but resizing the default page to 960 x 640 exited with a stack-overflow class failure before the second post-resize presentation commit.

`build.rs` reserves an 8 MB stack for the `zircon_editor` binary on Windows/MSVC only:

- The setting is emitted with `cargo:rustc-link-arg-bin=zircon_editor=/STACK:8388608`.
- It does not apply to `zircon_runtime`, non-MSVC targets, or library crates.
- It preserves the existing retained host architecture; it is a host process budget fix, not a layout algorithm fallback.

The temporary PE-header probe that used the same built editor exe plus `/STACK:8388608` stayed alive after the same 960 x 640 resize and produced `target/editor-visual-check/editor-default-960x640-stack8m-20260522-043217.png` with empty stdout/stderr. That probe is diagnostic evidence for the linker setting; the committed implementation is the Cargo build-script rule.

## Validation Notes

The 2026-05-22 focused build before the build-script change passed with the locked/offline host-only target dir listed in the header. After the local workspace gained unrelated plugin lockfile drift, `--locked` validation was no longer usable without changing the root lockfile. The follow-up offline build restored `Cargo.lock` after Cargo's temporary resolution update, rebuilt `zircon_editor` from source, and `dumpbin /headers` reported `800000 size of stack reserve`. The rebuilt binary then survived the 960 x 640 default resize probe and produced `target/editor-visual-check/editor-default-960x640-rebuilt-stack8m-20260522-043929.png` with no stdout/stderr diagnostics.
