# Manual Commands

## Shared Target Setup

Prefer a shared external `CARGO_TARGET_DIR` across clones and worktrees:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
```

## Low-Disk Rule

Before build or test, inspect the drive that hosts the active target directory. If the remaining free space is `<= 50 GB`, clean that target directory first:

```powershell
cargo clean --target-dir $env:CARGO_TARGET_DIR
```

If you are using the repo-local fallback slots instead of `CARGO_TARGET_DIR`, replace the argument with the active slot such as `target/codex-shared-a`.

## Workspace Build

```powershell
cargo build --workspace --locked --verbose
```

## Workspace Tests

```powershell
cargo test --workspace --locked --verbose
```

## Single-Crate Loop

```powershell
cargo test -p zircon_runtime --locked --verbose
```

## Export Platform Contract

To mirror the CI export-platform policy matrix locally without running unrelated tests, set `ZR_EXPORT_CONTRACT_PLATFORM` and run the focused runtime export policy test for each platform:

```powershell
$env:ZR_EXPORT_CONTRACT_PLATFORM = "headless"
cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose
```

The current platform set is `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless`. The validator shortcut is:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract
```

When the shared checkout already has active Cargo/Rust compiler lanes, run one low-interference export-platform check by platform:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
```

The platform selector is stage-scoped: `-ExportContractPlatform` without `-RunExportPlatformContract` is rejected so it cannot be silently ignored.

To inspect the selected command without requiring Cargo discovery, target-directory cleanup checks, or shared target slot claims, add `-DryRun`. Without explicit `-TargetDir`, dry-run renders `target/manual-check`:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -DryRun -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
```

## Profile Feature Contract

To mirror the profile feature CI contract locally, run no-default-features checks for the default profile combinations:

```powershell
cargo check -p zircon_app --no-default-features --features target-server --locked --verbose
cargo check -p zircon_app --no-default-features --features target-client,platform-winit,input-gamepad,gamepad-gilrs --locked --verbose
cargo check -p zircon_runtime --no-default-features --features target-client --locked --verbose
cargo check -p zircon_runtime --no-default-features --features target-editor-host --locked --verbose
cargo check -p zircon_runtime --no-default-features --features target-server --locked --verbose
```

The validator shortcut is:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract
```

When the shared checkout already has active Cargo/Rust compiler lanes, run one low-interference profile check by label:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server"
```

The profile selector is stage-scoped: `-ProfileFeatureContractLabel` without `-RunProfileFeatureContract` is rejected so it cannot be silently ignored.

To inspect the selected command without requiring Cargo discovery, target-directory cleanup checks, or shared target slot claims, add `-DryRun`. Without explicit `-TargetDir`, dry-run renders `target/manual-check`:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -DryRun -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server"
```

## Shared Target Rules

- Prefer one shared external `CARGO_TARGET_DIR` for routine local validation loops across multiple clones or worktrees.
- If no shared `CARGO_TARGET_DIR` is configured, `validate-matrix.ps1` falls back to `target/codex-shared-a` or `target/codex-shared-b`.
- Do not keep minting new `target/<name>` directories for normal Cargo build/test commands.
- If you need strict one-off isolation, use an explicit temporary `--target-dir` and treat it as an exception, not the default workflow.
- Read `../references/cargo-target-disk-policy.md` for cleanup and disk-usage guidance.

## CI Parity Notes

- The Linux CI job installs system dependencies for `winit`, `wgpu`, and `iced` before running workspace build and test.
- If a Linux-only failure appears locally or in CI, compare it against `.github/workflows/ci.yml` before changing the commands or acceptance criteria in this skill.
