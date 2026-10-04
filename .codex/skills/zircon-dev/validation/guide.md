# Zircon Dev Validation

The local coordinator, its tickets and `validate-matrix.ps1` managed workflow are retired. They are not validation prerequisites. Jenkins migration and its formal acceptance remain pending; do not treat a local command exit or an old receipt as that acceptance.

## Environment and output

- Prefer Windows PowerShell. Use WSL only for a concrete Linux-specific failure, tool, CI reproduction or explicit platform requirement; select [Linux tooling guidance](../../zircon-project-skills/evidence-driven-wsl-validation/guide.md) only for that exception.
- Every target, build directory, compiler cache and temporary compiler output must physically be below the drive-root `D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets` (their matching `/mnt/d`, `/mnt/e`, `/mnt/f` mounts in WSL). Other drives, repository targets, home/profile paths, legacy roots and aliases are forbidden.
- Use `tools/dev/local-cargo.ps1` or `python -B -m tools.dev.local_cargo` for independent local command evidence. The wrapper checks physical paths and a 35 GiB free-space reserve, sets target/build/cache/temporary paths, retains Cargo's native locks and returns the actual Cargo exit code. No service or session registration is used.
- Its new `zircon-local` namespace is separate from archived coordinator pools. Preserve historical pools, receipts and locks; do not clean them to make a new validation pass.

## Scope and cadence

Follow `docs/plans/milestone-validation-policy.md`. Ordinary implementation slices use formatting, diff checks and structural guards. A milestone runs one package-scoped check plus the focused changed-behavior regression batch; broaden only for changed contracts or the declared wave/release gate.

```powershell
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --locked
.\tools\dev\local-cargo.ps1 test -p zircon_runtime --lib <focused-filter> --locked
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --locked
.\tools\dev\local-cargo.ps1 -TargetDir D:\cargo-targets\zircon-local\windows\<owned-compatible-leaf> test -p zircon_runtime --lib <focused-filter> --locked
```

- Use `--release` for release evidence and `--profile profiling` for the workspace profiling profile. Separate platforms, toolchains, architectures, profiles and incompatible feature configurations.
- Keep `--locked`; the wrapper adds it if omitted. Lockfile updates require their own authorized workflow.
- Preserve exact filters, feature settings, environment/platform and source revision in evidence. Run ignored tests only with an explicitly scoped test selection.
- Workspace builds/tests belong to wave/release or root manifest/lockfile/toolchain changes. No routine workspace or duplicate Windows/WSL run.
- A dry run creates no directories, discovers no compiler and provides no passing validation evidence. Record terminal command output separately from milestone acceptance.
- Legacy export/profile/convention matrix scripts remain historical migration source. Compose their required scoped commands from current CI and the owning tests; do not restart the coordinator to execute them.

Read [manual commands](manual-commands.md) and [output storage policy](../references/cargo-target-disk-policy.md) when those details matter.
