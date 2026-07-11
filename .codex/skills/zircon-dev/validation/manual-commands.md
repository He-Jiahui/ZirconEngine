# Manual Commands

## Managed Target Setup

Do not run bare `cargo` commands from the repository. Use the validator so acquire/start/finish/release and `--target-dir` cannot be skipped:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1
```

The validator acquires, starts, finishes and releases the Cargo job automatically. If `CARGO_TARGET_DIR` is inherited, it must already resolve below `D:`, `E:`, or `F:` `\targets\zircon-engine\lanes`.

## Low-Disk Rule

The validator checks the granted target drive and runs a scoped clean when free space is `<= 50 GB`. Preview scheduled stale-lane cleanup separately:

```powershell
.\tools\cleanup-stale-targets.ps1
```

## Workspace Build

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipTest -VerboseOutput
```

## Workspace Tests

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -VerboseOutput
```

## Single-Crate Loop

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -VerboseOutput
```

## Export Platform Contract

To mirror the CI export-platform policy matrix locally without running unrelated tests, use the validator. The current platform set is `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless`:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract
```

When the shared checkout already has active Cargo/Rust compiler lanes, run one low-interference export-platform check by platform:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
```

The platform selector is stage-scoped: `-ExportContractPlatform` without `-RunExportPlatformContract` is rejected so it cannot be silently ignored.

To inspect the selected command without requiring Cargo discovery or target-directory cleanup checks, add `-DryRun`. Dry-run still receives a managed, non-created audit lane:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -DryRun -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
```

## Profile Feature Contract

To mirror the profile feature CI contract locally, use the validator so all no-default-features cases share the managed lifecycle:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract
```

When the shared checkout already has active Cargo/Rust compiler lanes, run one low-interference profile check by label:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server"
```

The profile selector is stage-scoped: `-ProfileFeatureContractLabel` without `-RunProfileFeatureContract` is rejected so it cannot be silently ignored.

To inspect the selected command without requiring Cargo discovery or target-directory cleanup checks, add `-DryRun`. Dry-run still receives a managed, non-created audit lane:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -DryRun -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server"
```

## Managed Target Rules

- Use one coordinator job per validation process; lanes are unique, audited and stored outside the repository.
- Do not create `target/<name>` directories or use arbitrary `--target-dir` values.
- Explicit targets are allowed only below `D:\targets\zircon-engine\lanes`, `E:\targets\zircon-engine\lanes`, or `F:\targets\zircon-engine\lanes` and still pass through the coordinator.
- Use `.\tools\cleanup-stale-targets.ps1` to preview cleanup and add `-Apply` only for service-revalidated deletion.
- Read `../references/cargo-target-disk-policy.md` for cleanup and disk-usage guidance.

## CI Parity Notes

- The Linux CI job installs system dependencies for `winit`, `wgpu`, and `iced` before running workspace build and test.
- If a Linux-only failure appears locally or in CI, compare it against `.github/workflows/ci.yml` before changing the commands or acceptance criteria in this skill.
