# Zircon Dev Validation

The local coordinator, its tickets and `validate-matrix.ps1` managed workflow are retired. They are not validation prerequisites. Jenkins migration and its formal acceptance remain pending; do not treat a local command exit or an old receipt as that acceptance.

## Environment and output

- Jenkins owns compilation, generation, artifact reuse and cleanup under the
  [management standard](../../../../docs/tools/agent-artifact-management.md).
  Submit the affected package/feature/test coverage through the verified
  incremental interface; reconcile its original request. The local wrapper and
  commands below require an explicit user-authorized local exception and never
  replace a pending Jenkins operation or supply Jenkins acceptance.
- Prefer Windows PowerShell. Use WSL only for a concrete Linux-specific failure, tool, CI reproduction or explicit platform requirement; select [Linux tooling guidance](../../zircon-project-skills/evidence-driven-wsl-validation/guide.md) only for that exception.
- Every target, build directory, compiler cache and temporary compiler output must physically be below the drive-root `D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets` (their matching `/mnt/d`, `/mnt/e`, `/mnt/f` mounts in WSL). Other drives, repository targets, home/profile paths, legacy roots and aliases are forbidden.
- Use `tools/dev/local-cargo.ps1` or `python -B -m tools.dev.local_cargo` for independent local command evidence. The wrapper checks physical paths and a 35 GiB free-space reserve, sets target/build/cache/temporary paths, retains Cargo's native locks and returns the actual Cargo exit code. No service or session registration is used.
- Its new `zircon-local` namespace is separate from archived coordinator pools. Preserve historical pools, receipts and locks; do not clean them to make a new validation pass.

## Scope and cadence

Follow `docs/testing-and-validation/milestone-validation-policy.md`. Ordinary implementation slices use formatting, diff checks and structural guards. A milestone runs a minimal package/feature check plus the focused changed-behavior regression batch. Additional affected modules require their own declared scope.

### Declare every build

Before each compilation-capable command, including `check`, `build`, `test` and
`test --no-run`, send a user-visible declaration containing:

- The engine modules being verified and their explicit Cargo package names.
- `--no-default-features` and the exact required features, or an explicit empty feature set.
- The selected library/binary/test targets, test filters when applicable, action and profile.
- The exact scoped command and why those modules cover the changed behavior.

For example: "Modules: math; package: zr_math; features: none (defaults disabled);
target: library; action/profile: check/dev; reason: changed math contract.
Command: `.\tools\dev\dev-module-build.ps1 -Package zr_math`."
Record the same scope in the Jenkins handoff and resulting command evidence.
If the command or module selection changes, declare it again before execution.
Use the module entry with explicit `-Package` for its supported selections; use
the verified scoped Jenkins Cargo interface for a target it cannot express.

Full builds are prohibited. Do not use `--workspace`, `--all`, `--all-features`,
`--all-targets`, implicit root-wide defaults or all-package loops/matrices to
recreate one. Wave/release gates and root manifest/lockfile/toolchain changes do
not permit a full build. Inspect affected dependencies first, then select only
the required modules and features. Cargo may compile their required transitive
dependencies. Keep broader acceptance pending when scoped evidence is incomplete.

The following local examples require the explicit local exception described above;
normal execution uses Jenkins:

```powershell
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --no-default-features --features core-min --lib --locked
.\tools\dev\local-cargo.ps1 test -p zircon_runtime --no-default-features --features core-min --lib <focused-filter> --locked
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --no-default-features --features core-min --lib --locked
.\tools\dev\local-cargo.ps1 -TargetDir E:\cargo-targets\zircon-local\windows\<owned-compatible-leaf> test -p zircon_runtime --no-default-features --features core-min --lib <focused-filter> --locked
```

- Use `--release` for release evidence and `--profile profiling` for the workspace profiling profile. Separate platforms, toolchains, architectures, profiles and incompatible feature configurations.
- Keep `--locked`; the wrapper adds it if omitted. Lockfile updates require their own authorized workflow.
- Preserve exact filters, feature settings, environment/platform and source revision in evidence. Run ignored tests only with an explicitly scoped test selection.
- Apply the same minimal module selection to wave/release and root manifest/lockfile/toolchain changes. No full builds or duplicate Windows/WSL runs.
- A dry run creates no directories, discovers no compiler and provides no passing validation evidence. Record terminal command output separately from milestone acceptance.
- Legacy export/profile/convention matrix scripts remain historical migration source. Compose their required scoped commands from current CI and the owning tests; do not restart the coordinator to execute them.

Read [manual commands](manual-commands.md) and [output storage policy](../references/cargo-target-disk-policy.md) when those details matter.
