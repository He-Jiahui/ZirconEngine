---
name: zircon-dev-validation
description: Use when `zircon-dev` work needs command selection, validator usage, or CI-parity guidance for validating the zirconEngine Rust workspace with Cargo.
---

# Zircon Dev Validation

## Preferred Validator

From Windows PowerShell, run:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1
```

Useful switches:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipTest
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server"
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -VerboseOutput
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir target/manual-check
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -DryRun -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
```

Read `../references/cargo-target-disk-policy.md` when shared-target setup, cleanup, or disk-usage tradeoffs are part of the task.

## Validation Cadence

- Read `../../zircon-project-skills/milestone-first-workflow-policy.md` before selecting validation commands.
- During implementation slices, do not run build or unit-test loops after every small task. Use scoped `cargo check` as the lightweight Rust syntax/type gate when needed.
- A plan must include a milestone testing stage. Run compile/build commands and unit tests in that stage, then debug and correct failures before promoting the milestone.
- Avoid generating debug build artifacts before the testing stage unless a concrete blocker requires earlier tool evidence.

## Validation Rules

- In the milestone testing stage, match CI first: `cargo build --workspace --locked --verbose` and `cargo test --workspace --locked --verbose`.
- If the milestone is clearly crate-local, start the testing stage with `cargo test -p <crate>` and then expand back to workspace validation when shared contracts or manifests move.
- Keep `--locked` on by default. Disable it only when lockfile work is explicitly in scope.
- Prefer setting a shared external `CARGO_TARGET_DIR` across clones or worktrees when disk usage matters; the validator respects that environment variable automatically.
- If no shared `CARGO_TARGET_DIR` is configured, the validator auto-selects one of `target/codex-shared-a` or `target/codex-shared-b` and reuses it per active session instead of minting unlimited new target directories.
- Before build or test, the validator checks remaining free space on the drive that hosts the active target directory. If that free space is `<= 50 GB`, it runs `cargo clean --target-dir <active-target-dir>` before continuing.
- `-DryRun` renders selected commands without running `cargo`, requiring cargo discovery, running target-directory cleanup checks, or claiming a shared target slot. Without explicit `-TargetDir`, dry-run renders commands against `target/manual-check`; pass `-TargetDir` only when you need a different displayed path.
- If both shared slots are occupied by other active sessions, pass `-TargetDir` explicitly instead of inventing a third default slot.
- Use `-RunExportPlatformContract` to mirror the CI export-platform policy matrix locally. The matrix covers `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless` by setting `ZR_EXPORT_CONTRACT_PLATFORM` for the focused `zircon_runtime` export policy test.
- Add `-ExportContractPlatform <platform>` only with `-RunExportPlatformContract` when active shared compile lanes make a single low-interference export-platform check safer than the full eight-platform matrix. Passing `-ExportContractPlatform` without `-RunExportPlatformContract` is rejected so the selector cannot be silently ignored.
- Use `-RunProfileFeatureContract` to mirror `.github/workflows/profile-feature-contract.yml` locally. The matrix checks the M5 no-default-features profile contracts for `zircon_app` server/client-platform and `zircon_runtime` client/editor-host/server.
- Add `-ProfileFeatureContractLabel "<label>"` only with `-RunProfileFeatureContract` when active shared compile lanes make a single low-interference profile check safer than the full five-case matrix. Passing `-ProfileFeatureContractLabel` without `-RunProfileFeatureContract` is rejected so the selector cannot be silently ignored.
- Treat Linux CI as the canonical cross-platform baseline. Compare local failures against `.github/workflows/ci.yml` before changing the validation story.
- Read `manual-commands.md` when you need to run the commands directly instead of the validator script.
