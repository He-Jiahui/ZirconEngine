---
related_code:
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - .codex/skills/zircon-dev/validation/SKILL.md
  - .codex/skills/zircon-dev/validation/manual-commands.md
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/default_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/platform/capability/matrix
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
implementation_files:
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - .codex/skills/zircon-dev/validation/SKILL.md
  - .codex/skills/zircon-dev/validation/manual-commands.md
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/default_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/platform/capability/matrix
plan_sources:
  - user: 2026-05-27 continue ZirconEngine Bevy-style Platform Window Input Gilrs completion plan
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
tests:
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
doc_type: workflow-detail
---

# Export Platform Contract

This document owns the M5 export-platform validation contract for generated package policy. It is separate from the profile-feature contract because export-platform validation checks target package shape, host resource policy, native dynamic compatibility, and generated shell files for each exported target.

## Purpose

M5 requires the export policy matrix to stay explicit for every supported generated target:

- `windows`, `linux`, and `macos` use desktop host policy.
- `android` and `ios` use mobile host policy.
- `web_gpu` and `wasm` use web host policy.
- `headless` uses a native headless host policy and server runtime profile.

The `headless` case is important because `target-server` must not silently reuse a windowed desktop package shape. Server export emits a headless binary entry point and no mobile/browser shell files.

## CI Contract

`.github/workflows/ci.yml` owns the CI matrix through the `export-platform` axis. The platform set is:

```text
windows, linux, macos, android, ios, web_gpu, wasm, headless
```

Each lane runs the focused runtime test:

```powershell
cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose
```

The workflow passes the selected matrix value through `ZR_EXPORT_CONTRACT_PLATFORM`, which lets the focused runtime test assert one export-platform policy per CI lane.

## Local Validator

The local validator mirrors the CI matrix through:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract
```

When active shared compiler lanes make the full eight-platform matrix too intrusive, run one platform:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
```

`validate-matrix.Tests.ps1` guards the contract in twenty-seven ways:

- the local export-platform list stays explicit and includes `headless`;
- the local list matches `.github/workflows/ci.yml`;
- the CI and local export-platform selectors remain unique for low-interference single-platform validation;
- every workflow export-platform value can be selected and rendered by the local validator with the same focused runtime test command;
- the generated Cargo arguments keep `--locked` and the selected target directory;
- selected dry-run commands include `--verbose` when `-VerboseOutput` is set, matching the verbose CI command shape;
- selected dry-run commands omit `--locked` only when `-NoLocked` is explicitly requested;
- the CLI dry-run entry point emits all eight export-platform commands and environment values when `-RunExportPlatformContract` is used without a selector;
- the single-platform selector rejects unknown platform names;
- the single-platform selector is rejected unless `-RunExportPlatformContract` is also set, so a focused export-platform request cannot be silently ignored;
- the CLI dry-run entry point also rejects the single-platform selector when the export contract stage switch is omitted;
- the CLI dry-run entry point emits only the selected single-platform command and environment value when `-ExportContractPlatform` is used;
- the CLI dry-run entry point can render selected export-platform commands without requiring Cargo discovery or target-directory cleanup checks;
- the CLI dry-run entry point defaults to `target/manual-check` without claiming a shared target slot when `-TargetDir` is omitted;
- the CLI dry-run entry point does not inherit `CARGO_TARGET_DIR` when `-TargetDir` is omitted;
- the CLI dry-run entry point still honors an explicit `-TargetDir` as a manual display override;
- the CLI dry-run entry point rejects unknown export-platform names with the complete expected-platform list;
- the validator documentation index states that selector-stage switch requirements are rejected rather than silently ignored;
- the validator documentation index states that dry-run command rendering does not require Cargo discovery, target-directory cleanup checks, or shared target slot claims;
- the export contract workflow keeps the same trigger, fail-fast, checkout, Rust toolchain, and cache scaffolding as the main CI shape;
- the export contract workflow stays centered on one matrix-driven focused test job and does not duplicate broad workspace or plugin workspace build/test commands;
- the export contract CI job installs the same Linux runtime dependency package set as the main CI workflow;
- `PlatformTarget::as_str()` and `ExportTargetPlatform::as_str()` stay aligned with the same eight platform tokens;
- the runtime export policy test's `ZR_EXPORT_CONTRACT_PLATFORM` parser stays aligned with the same eight platform tokens;
- generated headless server packages keep the `target-server` Cargo feature, a binary `src/main.rs` headless entry point, `EntryProfile::Headless`, and no mobile/browser `platform/*` shell or generated runtime library export;
- platform capability matrix source files remain free of `panic!`, `todo!`, and `unimplemented!` placeholder control flow;
- server/headless capability matrix paths keep explicit window, synthetic input, physical gamepad unavailable, and headless event-loop declarations.

## Current Evidence

On 2026-05-27, script-level validation passed:

- `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'`: 24 passed, 0 failed.
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless -DryRun -TargetDir target\manual-check`: emitted only the headless export-policy test with `ZR_EXPORT_CONTRACT_PLATFORM=headless`.
- After adding the runtime export policy test CI parser guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 41 passed, 0 failed.
- After adding the generated headless server package shape guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 43 passed, 0 failed.
- After adding the capability matrix explicit-status guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 46 passed, 0 failed.
- After adding the selector misuse guards for export-platform and profile-feature selectors, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 49 passed, 0 failed.
- After adding the selector-stage requirement documentation guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 50 passed, 0 failed.
- After adding the export contract focused-job guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 51 passed, 0 failed.
- After adding the CLI dry-run single-selector guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 53 passed, 0 failed.
- After adding the full-matrix CLI dry-run guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 55 passed, 0 failed.
- After adding the unknown-selector CLI rejection guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 57 passed, 0 failed.
- After adding the selector-without-stage CLI rejection guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 59 passed, 0 failed.
- After adding the side-effect-free dry-run guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 61 passed, 0 failed.
- After adding the dry-run documentation guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 62 passed, 0 failed.
- After adding the dry-run target isolation guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 64 passed, 0 failed.
- After adding selected dry-run verbose command guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 66 passed, 0 failed.
- After adding selected dry-run `-NoLocked` command guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 68 passed, 0 failed.
- After adding selected dry-run `CARGO_TARGET_DIR` isolation guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 70 passed, 0 failed.
- After adding the no-stage dry-run parsing guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 71 passed, 0 failed.
- After adding selected dry-run explicit `-TargetDir` override guards, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 73 passed, 0 failed.
- After adding the no-stage dry-run explicit `-TargetDir` override guard, `Invoke-Pester '.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1'` passed at 74 passed, 0 failed.
- A real selected headless export-platform validator run reached `zircon_runtime` compilation but failed before the focused export-policy test because `ui::accessibility::action::text` re-exported two helper functions whose child-module visibility was too narrow for the parent dispatcher. The helper visibility was widened only to `crate::ui::accessibility::action`.
- After that fix, `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless -TargetDir F:\cargo-targets\zircon-platform-m5-export-headless -VerboseOutput` passed: export platform contract `(headless)` OK.
- Full real export-platform validator coverage also passed through `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -TargetDir F:\cargo-targets\zircon-platform-m5-export-headless -VerboseOutput`: `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless` all reported OK.

Earlier direct runtime test-binary execution covered all eight `ZR_EXPORT_CONTRACT_PLATFORM` values with `platform_target_policy_matches_host_resource_and_plugin_strategy`: each platform variant ran 1 test, passed 1, failed 0, with 2086 tests filtered out.

The full eight-platform Cargo validator path is accepted for the focused export-platform contract. Broader workspace build/test gates remain part of the full M5 testing stage.
