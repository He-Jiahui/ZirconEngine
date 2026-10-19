# Manual Commands

## Independent local Cargo

The coordinator and its managed matrix/ticket/commit workflows are retired. No registration or lease is required. Jenkins owns normal execution; these local commands require an explicit user-authorized local exception. Before every compilation-capable command, announce its modules, packages, features, targets/test filters, profile, exact command and coverage reason under [the build declaration](guide.md#declare-every-build). The local wrapper validates physical output/cache/temporary paths and the 35 GiB free-space reserve:

```powershell
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --no-default-features --features core-min --lib --locked
.\tools\dev\local-cargo.ps1 test -p zircon_runtime --no-default-features --features core-min --lib <focused-filter> --locked
.\tools\dev\local-cargo.ps1 build -p zr_math --no-default-features --lib --locked
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --no-default-features --features core-min --lib --locked
```

Choose scope with `docs/testing-and-validation/milestone-validation-policy.md`: no routine slice compile; minimal package/feature checks and focused tests at a milestone. Full builds are prohibited, including `--workspace`, `--all`, `--all-features`, `--all-targets`, implicit root-wide selections and all-package loops/matrices. Wave/release and root manifest/lockfile/toolchain changes still require explicitly scoped affected modules. Inspect dependencies when the necessary scope is unclear.

These are command examples, not evidence that the commands were run. A dry run or zero exit code does not grant milestone/Jenkins acceptance; retain actual logs, source identity, required review and failure resolution.

## Profiles, features and test filters

Use `--release` or `--profile profiling` explicitly when required. Keep platforms, toolchains, architectures, profiles and incompatible feature sets in separate output leaves. The wrapper defaults to a configuration-keyed `zircon-local` target and adds `--locked`; authorized lockfile changes need their own workflow.

Use only the required filter and current package/feature contract from its owning tests or CI. Run ignored tests only with an explicitly named scoped selection. Cargo test arguments after `--` remain test-binary arguments.

## WSL exception

Use WSL only for a recorded Linux-specific requirement. From the checkout run `python3 -B -m tools.dev.local_cargo -- <scoped Cargo arguments>`; outputs, caches and temporary compiler products stay below `/mnt/d/cargo-targets`, `/mnt/e/cargo-targets` or `/mnt/f/cargo-targets`. Preserve native locks and do not share a leaf with Windows.

## Historical tools

`validate-matrix.ps1`, coordinator tickets, failure-return APIs and automatic WeCom commit actions remain migration source only. Do not restore the service or old scheduled cleanup to make them run. Retain all historical pools and data until a separately reviewed migration/cleanup. See [storage policy](../references/cargo-target-disk-policy.md).
