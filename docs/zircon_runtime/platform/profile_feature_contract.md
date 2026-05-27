---
related_code:
  - .github/workflows/profile-feature-contract.yml
  - docs/zircon_runtime/platform/export_platform_contract.md
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - .codex/skills/zircon-dev/validation/SKILL.md
  - .codex/skills/zircon-dev/validation/manual-commands.md
  - zircon_app/Cargo.toml
  - zircon_app/build.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/plugin/export_build_plan/default_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/platform/capability/matrix
implementation_files:
  - .github/workflows/profile-feature-contract.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - .codex/skills/zircon-dev/validation/SKILL.md
  - .codex/skills/zircon-dev/validation/manual-commands.md
  - zircon_app/build.rs
  - zircon_runtime/src/plugin/export_build_plan/default_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/platform/capability/matrix
plan_sources:
  - user: 2026-05-27 continue ZirconEngine Bevy-style Platform Window Input Gilrs completion plan
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
tests:
  - .github/workflows/profile-feature-contract.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract
doc_type: workflow-detail
---

# Profile Feature Contract

This document owns the M5 profile-feature validation contract while `docs/zircon_runtime/platform/feature_matrix.md` is under active edit by other sessions. Once that file is available for a clean update, this workflow summary can be merged back into the main platform feature matrix document.

## Purpose

M5 requires the Bevy-style profile promises to compile independently from the broad default workspace build:

- `zircon_app` `target-server` must remain headless and avoid window/gilrs defaults.
- `zircon_app` `target-client,platform-winit,input-gamepad,gamepad-gilrs` must compile the windowed client host profile with desktop gamepad support.
- `zircon_runtime` `target-client` and `target-editor-host` must carry the default platform feature topology.
- `zircon_runtime` `target-server` must remain headless.

The contract is intentionally separate from the export-platform policy matrix. Export-platform validation checks generated package policy for `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless`; profile-feature validation checks the Cargo feature combinations that make those profile promises usable.

See `docs/zircon_runtime/platform/export_platform_contract.md` for the generated package policy matrix and the `-ExportContractPlatform` local validator selector.

## CI Contract

`.github/workflows/profile-feature-contract.yml` runs a matrix of `cargo check` commands with `--no-default-features`:

```powershell
cargo check -p zircon_app --no-default-features --features target-server --locked --verbose
cargo check -p zircon_app --no-default-features --features target-client,platform-winit,input-gamepad,gamepad-gilrs --locked --verbose
cargo check -p zircon_runtime --no-default-features --features target-client --locked --verbose
cargo check -p zircon_runtime --no-default-features --features target-editor-host --locked --verbose
cargo check -p zircon_runtime --no-default-features --features target-server --locked --verbose
```

The workflow installs the same Linux system dependency family as the main CI build because the client/editor profile checks can transitively touch winit and platform host crates.

## Local Validator

The local validator mirrors the workflow through:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract
```

The dry-run form is safe during active shared compile queues:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -DryRun -TargetDir target\manual-check
```

`validate-matrix.Tests.ps1` guards the contract in thirty ways:

- the local profile case list stays explicit;
- the local list matches `.github/workflows/profile-feature-contract.yml`;
- the CI and local profile labels remain unique for low-interference single-case validation;
- every workflow matrix case is renderable through the local validator by label with matching package, feature string, `--locked`, and selected target directory;
- every workflow matrix feature name is backed by the corresponding package manifest;
- every workflow matrix package name is backed by the corresponding package manifest name;
- the generated Cargo arguments keep `--no-default-features`, `--features`, `--locked`, and the selected target directory;
- selected dry-run commands include `--verbose` when `-VerboseOutput` is set, matching the verbose CI command shape;
- selected dry-run commands omit `--locked` only when `-NoLocked` is explicitly requested;
- the CLI dry-run entry point emits all five profile-feature commands when `-RunProfileFeatureContract` is used without a selector;
- the single-profile selector is rejected unless `-RunProfileFeatureContract` is also set, so a focused profile request cannot be silently ignored;
- the CLI dry-run entry point also rejects the single-profile selector when the profile feature stage switch is omitted;
- the CLI dry-run entry point emits only the selected single-profile command when `-ProfileFeatureContractLabel` is used;
- the CLI dry-run entry point can render selected profile-feature commands without requiring Cargo discovery or target-directory cleanup checks;
- the CLI dry-run entry point defaults to `target/manual-check` without claiming a shared target slot when `-TargetDir` is omitted;
- the CLI dry-run entry point does not inherit `CARGO_TARGET_DIR` when `-TargetDir` is omitted;
- the CLI dry-run entry point still honors an explicit `-TargetDir` as a manual display override;
- the CLI dry-run entry point rejects unknown profile labels with the complete expected-label list;
- the validator documentation index states that selector-stage switch requirements are rejected rather than silently ignored;
- the validator documentation index states that dry-run command rendering does not require Cargo discovery, target-directory cleanup checks, or shared target slot claims;
- the profile contract workflow keeps the same trigger, fail-fast, checkout, Rust toolchain, and cache scaffolding as the main CI shape;
- the profile contract workflow remains centered on one matrix-driven job, with every run using `${{ matrix.label }}`, `${{ matrix.package }}`, and `${{ matrix.features }}` rather than a broad workspace build or test command;
- the profile contract workflow installs the same Linux runtime dependency package set as the main CI workflow;
- the app/runtime default profile topology keeps client/editor-host on `default-platform` while server stays on `platform-headless` and excludes window, desktop backend, input, and gilrs feature families;
- the `platform-headless` feature definitions stay narrow: app forwards only to runtime headless support, and runtime headless itself enables no additional feature family;
- the built-in server export profile keeps `RuntimeTargetMode::ServerRuntime`, `ExportTargetPlatform::Headless`, and `RuntimeProfileId::Server`;
- the generated headless server package keeps the `target-server` Cargo feature, a binary `src/main.rs` headless entry point, and `EntryProfile::Headless` rather than mobile/browser shell files or a generated runtime library export;
- the platform capability matrix source files remain free of `panic!`, `todo!`, and `unimplemented!` placeholder control flow;
- the server/headless capability matrix paths keep explicit window, synthetic input, physical gamepad unavailable, and headless event-loop declarations;
- the `zircon_app` build script declaration remains backed by a local `build.rs`, so clean CI profile checks cannot reference a missing script.

## Current Evidence

On 2026-05-27, script-level validation passed:

- `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'`: 62 passed, 0 failed.
- After adding the dry-run target isolation guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 64 passed, 0 failed.
- After adding selected dry-run verbose command guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 66 passed, 0 failed.
- After adding selected dry-run `-NoLocked` command guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 68 passed, 0 failed.
- After adding selected dry-run `CARGO_TARGET_DIR` isolation guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 70 passed, 0 failed.
- After adding the no-stage dry-run parsing guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 71 passed, 0 failed.
- After adding selected dry-run explicit `-TargetDir` override guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 73 passed, 0 failed.
- After adding the no-stage dry-run explicit `-TargetDir` override guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 74 passed, 0 failed.
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server" -DryRun -TargetDir target\manual-check`: emitted only the `zircon_runtime target-server` no-default-features profile check.
- Passing `-ProfileFeatureContractLabel` without `-RunProfileFeatureContract` is covered by the Pester suite at helper and CLI entry-point level, and rejects with an explicit diagnostic.
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server" -TargetDir F:\cargo-targets\zircon-platform-m5-profile-runtime-server -VerboseOutput` passed: profile feature contract `(zircon_runtime target-server)` OK.
- Full real profile-feature validator coverage also passed through `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -TargetDir F:\cargo-targets\zircon-platform-m5-profile-runtime-server -VerboseOutput`: `zircon_app target-server`, `zircon_app target-client-platform`, `zircon_runtime target-client`, `zircon_runtime target-editor-host`, and `zircon_runtime target-server` all reported OK.

The full five-case Cargo validator path is accepted for the focused profile-feature contract. Broader workspace build/test gates remain part of the full M5 testing stage.

## Low-Interference Follow-Up

The profile-feature contract supports single-case validation with:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server"
```

Use this command before expanding to the full profile-feature matrix when other active sessions are still using Cargo/Rust compiler lanes. For export-platform low-interference validation, use the companion command documented in `docs/zircon_runtime/platform/export_platform_contract.md`.
