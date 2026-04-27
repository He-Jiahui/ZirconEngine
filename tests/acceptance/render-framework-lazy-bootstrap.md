# Render Framework Lazy Bootstrap

## Scope
- Changed `GraphicsModule.Manager.RenderFramework` from immediate startup to lazy startup.
- Kept `GraphicsModule.Manager.RenderingManager` immediate so profile bootstrap can still prove graphics wiring without allocating a GPU adapter.
- Updated app profile bootstrap coverage so runtime/editor profile tests do not resolve the real render framework during wiring checks.

Affected layers: runtime module registration, core service activation, app entry profile bootstrap, and adapterless graphics bootstrap validation.

## Baseline
- User-reported `cargo test -p zircon_app --lib` failed on 2026-04-26 in `entry::tests::profile_bootstrap::{editor_bootstrap_registers_editor_and_primary_managers,runtime_bootstrap_excludes_editor_module}`.
- Both failures originated from eager `GraphicsModule.Manager.RenderFramework` initialization returning `GraphicsError::NoAdapter`.
- RED check added before production change: `cargo test -p zircon_runtime --locked graphics_module_defers_render_framework_until_resolved -- --nocapture` failed with `left: Immediate` and `right: Lazy`.

## Test Inventory
- Focused subsystem regression: `zircon_runtime/src/tests/graphics_surface/host_wiring.rs::graphics_module_defers_render_framework_until_resolved`.
- Upper-layer regression: `zircon_app/src/entry/tests/profile_bootstrap.rs`.
- Boundary case: activating `GraphicsModule` with the asset module available must not initialize `RenderFramework`, while `RenderingManager` still resolves.
- Failure path: adapter/device allocation remains deferred to consumers that explicitly call `resolve_render_framework(...)`.

## Tooling Evidence
- Windows targeted RED command: `cargo test -p zircon_runtime --locked graphics_module_defers_render_framework_until_resolved -- --nocapture`.
- Windows targeted GREEN command: `cargo test -p zircon_runtime --locked graphics_module_defers_render_framework_until_resolved -- --nocapture`.
- Windows upper-layer command: `cargo test -p zircon_app --locked --lib entry::tests::profile_bootstrap -- --nocapture`.
- WSL tool check: `wsl -d Ubuntu-22.04 -- bash -lc 'cd /mnt/e/Git/ZirconEngine && cargo --version'` reported `cargo 1.94.1`.
- WSL isolated target command: `wsl -d Ubuntu-22.04 -- bash -lc 'cd /mnt/e/Git/ZirconEngine && cargo test -p zircon_runtime --locked graphics_module_defers_render_framework_until_resolved --target-dir target/codex-wsl-lazy-bootstrap -- --nocapture'`.
- Windows isolated target command: `cargo test -p zircon_app --locked --lib --target-dir target/codex-win-lazy-bootstrap`.
- Windows editor-host feature retry: `cargo test -p zircon_app --locked --lib --features target-editor-host entry::tests::profile_bootstrap --target-dir target/codex-win-lazy-bootstrap-editor -- --nocapture`.

## Results
- Focused subsystem regression passed after `RenderFramework` became lazy.
- Original app profile bootstrap tests passed after removing the eager render framework resolution from the profile wiring assertion.
- WSL isolated lower-layer smoke passed: 1 passed, 0 failed.
- Windows isolated default app lib passed: 15 passed, 0 failed.
- The later Windows editor-host feature retry was blocked by concurrent Cargo package-cache work in the shared checkout before it produced a useful test result.

## Acceptance Decision
- Accepted for the adapterless profile bootstrap regression based on the focused RED/GREEN cycle, the original app profile regression pass, the WSL lower-layer smoke, and the isolated default app-lib pass.
- Remaining risk: consumers that explicitly resolve `RenderFramework` still require a compatible `wgpu` adapter; this change intentionally defers that failure to rendering consumers instead of profile registration.
