# Manual Commands

## Independent local Cargo

The coordinator and its managed matrix/ticket/commit workflows are retired. No registration or lease is required. Use the local wrapper to validate physical output/cache/temporary paths and the 35 GiB free-space reserve:

```powershell
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --locked
.\tools\dev\local-cargo.ps1 test -p zircon_runtime --lib <focused-filter> --locked
.\tools\dev\local-cargo.ps1 build -p zircon_hub --locked
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --locked
```

Choose scope with `docs/plans/milestone-validation-policy.md`: no routine slice compile; one package check and focused test batch at a milestone. Workspace builds/tests are reserved for a wave/release or root manifest/lockfile/toolchain change:

```powershell
.\tools\dev\local-cargo.ps1 build --workspace --locked
.\tools\dev\local-cargo.ps1 test --workspace --locked
```

These are command examples, not evidence that the commands were run. A dry run or zero exit code does not grant milestone/Jenkins acceptance; retain actual logs, source identity, required review and failure resolution.

## Profiles, features and test filters

Use `--release` or `--profile profiling` explicitly when required. Keep platforms, toolchains, architectures, profiles and incompatible feature sets in separate output leaves. The wrapper defaults to a configuration-keyed `zircon-local` target and adds `--locked`; authorized lockfile changes need their own workflow.

Use only the required filter and current package/feature contract from its owning tests or CI. Run ignored tests only with an explicitly named scoped selection. Cargo test arguments after `--` remain test-binary arguments.

## WSL exception

Use WSL only for a recorded Linux-specific requirement. From the checkout run `python3 -B -m tools.dev.local_cargo -- <scoped Cargo arguments>`; outputs, caches and temporary compiler products stay below `/mnt/d/cargo-targets`, `/mnt/e/cargo-targets` or `/mnt/f/cargo-targets`. Preserve native locks and do not share a leaf with Windows.

## Historical tools

`validate-matrix.ps1`, coordinator tickets, failure-return APIs and automatic WeCom commit actions remain migration source only. Do not restore the service or old scheduled cleanup to make them run. Retain all historical pools and data until a separately reviewed migration/cleanup. See [storage policy](../references/cargo-target-disk-policy.md).
