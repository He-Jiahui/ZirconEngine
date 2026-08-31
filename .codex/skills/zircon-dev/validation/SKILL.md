---
name: zircon-dev-validation
description: Use when `zircon-dev` work needs command selection, validator usage, target-directory placement, or CI-parity guidance for validating the ZirconEngine Rust workspace with Cargo. Prefer Windows-native validation and allow WSL only for explicit Linux-specific requirements with targets under the mounted approved D, E, or F roots.
---

# Zircon Dev Validation

## Environment Priority

- Read `../../zircon-project-skills/prefer-windows-validation/SKILL.md` before choosing the validation environment.
- Run ordinary Cargo checks, builds, tests, and milestone validation from Windows PowerShell with `validate-matrix.ps1`.
- Use WSL only for a concrete Linux-specific failure, Linux CI reproduction, Linux-only tool, or explicit Linux platform requirement.
- A Linux CI baseline does not require duplicating every successful Windows validation run in WSL.
- Every Cargo target must be below an approved `cargo-targets`, `targets`, or `ZirconBuilds` root on `D:`, `E:`, or `F:` (or its `/mnt/d`, `/mnt/e`, `/mnt/f` equivalent in WSL). Never use any other location, including `~` or `/home/<user>`.

## Preferred Validator

From Windows PowerShell, run:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1
```

Useful switches:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -CargoProfile release
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -CargoProfile profiling
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -StorageMode compact
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -StorageMode diagnostic
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter export_visual_evidence -IgnoredTests
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipTest
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -ProfileFeatureContractLabel "zircon_runtime target-server"
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunConventionStructure
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunConventionClippy
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -VerboseOutput
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir E:\cargo-targets\manual-check
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -DryRun -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless
```

Read `../references/cargo-target-disk-policy.md` when shared-target setup, cleanup, or disk-usage tradeoffs are part of the task.

## Validation Cadence

- Read `../../zircon-project-skills/milestone-first-workflow-policy.md` before selecting validation commands.
- During implementation slices, do not run build or unit-test loops after every small task. Use scoped `cargo check` as the lightweight Rust syntax/type gate when needed.
- A plan must include a milestone testing stage. Run compile/build commands and unit tests in that stage, then debug and correct failures before promoting the milestone.
- Avoid generating debug build artifacts before the testing stage unless a concrete blocker requires earlier tool evidence.

## Validation Rules

- In the milestone testing stage, run the smallest complete batch per `docs/plans/milestone-validation-policy.md` §3: one package-scoped `cargo check` for the changed targets, then one focused `cargo test` batch for the milestone's changed behavior.
- Reserve `cargo build --workspace` / `cargo test --workspace` for execution-wave closeout, release candidates, or root manifest/lockfile/toolchain changes (policy §4). Do not run them per milestone.
- Expand a crate-local batch to multi-package validation only when shared contracts, DTOs, or manifests actually moved in the milestone.
- Keep `--locked` on by default. Disable it only when lockfile work is explicitly in scope.
- Every non-dry validator run submits the complete compatibility description to the local Session coordinator and acquires the corresponding single primary pool below an approved root.
- `-CargoProfile development` is the compatibility default and uses Cargo's `debug` directory. Use `release` for throughput, power, and release artifacts; it adds exactly one `--release` to compiling commands and publishes from `release`. Use `profiling` for symbolized CPU/ETW attribution; it adds exactly one `--profile profiling`, consumes the workspace profile that inherits `release` with debug symbols retained, and publishes from `profiling`. Each profile has a distinct compatibility identity; cleanup remains profile-independent.
- Compatible work reuses that pool across Sessions. The default `reuse` mode keeps the hot target while disabling Cargo incremental and dev/test debug information and routes compiler objects through a bounded shared `sccache`. Each D/E/F storage root has a dedicated endpoint and a guardian-protected `cache\sccache-temporary` directory used only to initialize the daemon. Cargo, rustc, build scripts, and sccache clients use `scratch\<job-id>\temporary`; client-side mode keeps rustc in the requesting process, and terminal release deletes only that job's scratch. Use `diagnostic` when symbol-oriented Cargo defaults are required.
- One task owns a retained compatibility pool at a time. Validation tickets execute from immutable workspace copies, and the worker advances at most two independent tickets concurrently. The Cargo reservation budget retains one warm lane and can add only one resource-admitted disposable burst lane for eligible scoped commands; it never multiplies retained pools.
- Route asynchronous multi-Session verification through `milestone validate`/validation tickets so the durable worker owns FIFO waiting and source-copy isolation. Use direct `validate-matrix.ps1` for an interactive owner run or a recorded exact-return command, not as a background queue. If direct admission reports `cargo_cpu_lane_reserved` or its client response times out after submission, reconcile that exact request/job and wait for the existing FIFO owner; never launch a parallel retry or bypass the managed lane.
- Explicit `-TargetDir` and inherited `CARGO_TARGET_DIR` values are accepted only when they resolve below one of the nine approved roots and agree with the coordinator-selected pool. Every other location fails before Cargo runs.
- Before build or test, the validator checks remaining free space on the drive that hosts the active target directory. It refuses admission when less than the required `35 GiB` reserve remains and never runs an automatic `cargo clean` followed by a full rebuild.
- `-DryRun` renders selected commands without running Cargo, discovering Cargo or rustc, performing storage admission checks, registering a validation Session, acquiring a Cargo lane, or writing coordinator state. It derives the compatibility-keyed target projection locally and does not create the directory.
- Never create `target/<name>` directories in the repository. Use the validator or request an explicit path below an allowed drive-root `cargo-targets` directory.
- The repository `PreToolUse` Hook rejects raw artifact-producing Cargo commands and direct `git commit` before execution. This is a guardrail, not a replacement for the coordinator: do not evade it through shell nesting, `cargo.exe`, aliases, or an ad-hoc `--target-dir`; accepted business commits are created only by `milestone commit`.
- On Windows, `tools/check_conventions.py` delegates its compiling `structure` and `clippy` gates to the two exact convention switches above unless it can prove that it already inherited a managed target, job scratch, and shared cache. This keeps convention checks on the same retained compatibility pool instead of creating a repository-local `target` tree.
- At a child-plan milestone boundary, use `tools/zircon-session.ps1 milestone validate` instead of manually composing a Cargo command. The resulting validation copy lives under a managed drive-root `cargo-targets` verification directory and is terminally cleaned by the coordinator.
- Use `-RunExportPlatformContract` to mirror the CI export-platform policy matrix locally. The matrix covers `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless` by setting `ZR_EXPORT_CONTRACT_PLATFORM` for the focused `zircon_runtime` export policy test.
- Add `-ExportContractPlatform <platform>` only with `-RunExportPlatformContract` when active shared compile lanes make a single low-interference export-platform check safer than the full eight-platform matrix. Passing `-ExportContractPlatform` without `-RunExportPlatformContract` is rejected so the selector cannot be silently ignored.
- Use `-IgnoredTests` only for one explicitly named ignored test through a non-empty `-TestFilter` plus `-LibTests` or `-TestTarget`. The validator rejects unfiltered ignored-test runs and rejects `-IgnoredTests` with `-SkipTest`.
- Use `-RunProfileFeatureContract` to mirror `.github/workflows/profile-feature-contract.yml` locally. The matrix checks the M5 no-default-features profile contracts for `zircon_app` server/client-platform and `zircon_runtime` client/editor-host/server.
- Add `-ProfileFeatureContractLabel "<label>"` only with `-RunProfileFeatureContract` when active shared compile lanes make a single low-interference profile check safer than the full five-case matrix. Passing `-ProfileFeatureContractLabel` without `-RunProfileFeatureContract` is rejected so the selector cannot be silently ignored.
- Treat Linux CI as the canonical Linux cross-platform baseline. Compare Linux-specific failures against `.github/workflows/ci.yml` without making WSL the default local validation environment.
- Read `manual-commands.md` when you need to run the commands directly instead of the validator script.
