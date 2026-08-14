# Manual Commands

## Managed Target Setup

Do not run bare `cargo` commands from the repository. Use the validator so acquire/start/finish/release and `--target-dir` cannot be skipped:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1
```

The validator derives the compatibility key, acquires the single compatible primary pool, starts, finishes and releases the Cargo job automatically. If `CARGO_TARGET_DIR` is inherited, it must resolve below one of the approved `cargo-targets`, `targets`, or `ZirconBuilds` roots on `D:`, `E:`, or `F:` and agree with that pool.

The repository `PreToolUse` Hook also rejects direct artifact-producing Cargo commands (`build`, `check`, `test`, `run`, `bench`, `clippy`, `doc`, and `clean`) and direct `git commit`. Do not bypass it with `cargo.exe`, nested PowerShell, aliases, a manually selected target directory, or a direct commit. The guard records only a sanitized local diagnostic for later debugging; it never stores command text, commit messages, or credentials.

For a numbered-plan milestone, use the coordinator action rather than a raw validator invocation:

```powershell
& .\tools\zircon-session.ps1 milestone prepare --session-id <session> --milestone M1
& .\tools\zircon-session.ps1 milestone validate --session-id <session> --run-id <run> --milestone M1 --template coordinator-actions
```

After the managed validation result and an independent review are accepted, `milestone commit` performs the scoped commit, records `M1`, and sends the one permitted WeCom notification. The coordinator imports terminal validation evidence before deleting its temporary copy. A failed deletion remains `cleanup_pending` and the daemon retries it every 30 seconds; bounded stdout/stderr evidence remains in SQLite for diagnosis.

## WSL Exception

Do not use WSL for routine validation and do not run a direct, unleased WSL Cargo command. When a Linux-specific requirement justifies WSL, a coordinator-aware Windows host launcher must acquire with `platform=wsl`, keep ownership while its `wsl.exe` child runs, translate the granted path to the matching `/mnt/d`, `/mnt/e`, or `/mnt/f` path, and finish/release the job. If that managed launcher is unavailable, report the validation gap instead of building in an ad-hoc location. Never use `~`, `$HOME`, `/home/<user>`, a per-Session leaf, or a Windows-compatible pool.

## Low-Disk Rule

The validator checks the granted target drive and runs a scoped clean when free space is `<= 50 GB`. Preview scheduled stale-lane cleanup separately:

```powershell
.\tools\cleanup-stale-targets.ps1
```

## Workspace Build (wave closeout only)

Reserve for execution-wave closeout, release candidates, or root manifest/lockfile/toolchain changes per `docs/plans/milestone-validation-policy.md` §4:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipTest -VerboseOutput
```

## Workspace Tests (wave closeout only)

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -VerboseOutput
```

## Single-Crate Loop

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -VerboseOutput
```

## Cargo Profiles

Omitting `-CargoProfile` preserves the historical development behavior and reads artifacts from the Cargo `debug` directory. Use the release profile for throughput, power, or shippable artifacts:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -CargoProfile release
```

Use the workspace `profiling` profile only when symbolized CPU or ETW stack attribution is required. It inherits release optimization, retains debug information, emits `--profile profiling`, and reads artifacts from the Cargo `profiling` directory:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -CargoProfile profiling
```

The selected profile is part of the coordinator compatibility identity. Do not compare development, release, and profiling binaries as though they came from the same immutable build.

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

- Use one coordinator job per validation process. Compatible jobs share one audited primary pool across Sessions, but only one task may own it at a time.
- Do not create `target/<name>` directories or use `--target-dir` values outside the nine approved drive-root trees.
- Explicit targets are allowed only below an approved `cargo-targets`, `targets`, or `ZirconBuilds` root on `D:`, `E:`, or `F:` and still pass through the coordinator.
- Use `.\tools\cleanup-stale-targets.ps1` to preview cleanup and add `-Apply` only for service-revalidated deletion.
- Read `../references/cargo-target-disk-policy.md` for cleanup and disk-usage guidance.

## CI Parity Notes

- The Linux CI job installs system dependencies for `winit` and `wgpu` before running workspace build and test.
- If a Linux-only failure appears locally or in CI, compare it against `.github/workflows/ci.yml` before changing the commands or acceptance criteria in this skill.
