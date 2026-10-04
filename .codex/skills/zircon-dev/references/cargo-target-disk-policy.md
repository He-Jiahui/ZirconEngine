# Cargo Target Disk Policy

All local Cargo outputs and compiler caches must physically stay below exactly one of:

```text
D:\cargo-targets\<owned-output>
E:\cargo-targets\<owned-output>
F:\cargo-targets\<owned-output>
```

Repository/profile/home targets, other drives, `D/E/F:\targets`, `D/E/F:\ZirconBuilds`, nested lookalike roots, path traversal and aliases are forbidden. Use the matching mounted `/mnt/d`, `/mnt/e`, `/mnt/f` root for a justified WSL run. Windows and Linux never share one artifact leaf.

## Local validation after retirement

- `tools/dev/local-cargo.ps1` / `python -B -m tools.dev.local_cargo` need no coordinator, registration or lease. Explicit target paths receive the same physical-path checks as projected paths.
- The default namespace is `zircon-local/<platform>/<configuration-key>/target`; its key includes repository identity, platform, toolchain declaration and Cargo arguments. Preserve Cargo's native target locks and use separate leaves for incompatible platforms/toolchains/architectures/profiles/features.
- Target, Cargo build directory, Cargo home, temporary compiler outputs and cache paths all remain inside the selected root. The wrapper uses its own cache/scratch namespace and disables inherited compiler wrappers; it does not reuse the retired service's scratch or managed pools.
- Preserve at least 35 GiB free on the target drive before a build/test. Reject admission when unavailable; never erase a reusable pool and immediately rebuild it.
- Do not schedule competing full builds for the same graph or delete another task's active output. Record the exact command, revision, profile, feature selection, platform and terminal result under the validation cadence.
- `--dry-run` projects a command without Cargo discovery, directory creation, storage admission or coordinator state. It is not validation acceptance.

## Preserved historical storage

The former pool, reservation, sccache, retention, cleanup and worker contracts are retained in coordinator source/data for migration. They do not require local registration or leases after retirement. Old pools and coordinator artifacts must remain intact until a separately authorized cleanup/migration identifies their owner, retained references and terminal process status.

The old `tools/maintenance/cleanup-stale-targets.ps1` workflow and its scheduled task are disabled for retirement. Do not run automatic `cargo clean` or restore legacy scheduled cleanup. Cleanup requires a reviewed exact path set and verification that no live process or retained data reference owns it.

Formal Jenkins migration acceptance is separate from local command evidence; it remains open until its declared gates are actually met.
