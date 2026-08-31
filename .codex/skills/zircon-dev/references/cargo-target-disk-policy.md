# Cargo Target Disk Policy

- All Cargo output belongs below one of the nine allowed drive-root directories:

```powershell
D:\cargo-targets\<coordinator-managed-path>
E:\cargo-targets\<coordinator-managed-path>
F:\cargo-targets\<coordinator-managed-path>
D:\targets\<managed-build-path>
E:\targets\<managed-build-path>
F:\targets\<managed-build-path>
D:\ZirconBuilds\<managed-build-path>
E:\ZirconBuilds\<managed-build-path>
F:\ZirconBuilds\<managed-build-path>
```

- The service may choose an approved root by available space. No Cargo build may write to any other drive or directory tree.
- The compatibility key is derived from repository identity, platform (`windows` or `wsl`), Rust toolchain, target architecture, workspace identity, and canonical build configuration.
- Exactly one retained primary pool exists per compatibility key and only one active job may own it. The reservation service may admit at most one additional disposable CPU burst target for an eligible target-free `cargo check` or package-scoped library test when CPU, memory, and disk headroom all pass; it never creates a second retained pool.
- Compatible pools survive release for cross-Session reuse. Missing or incomplete compatibility metadata makes the output ephemeral; delete it immediately after release, retry failed deletion during maintenance, and evict idle reusable pools oldest-first under disk pressure.
- The default validator storage mode is `reuse`: retain Cargo fingerprints and compiled dependencies in the compatible target, disable Cargo incremental and dev/test debug information, and use the storage-root shared `sccache` with a `12 GiB` bound. Managed sccache always uses `SCCACHE_CLIENT_SIDE=1`. The nine D/E/F `cargo-targets`, `targets`, and `ZirconBuilds` roots use distinct ports `42260` through `42268`, so a daemon cannot silently serve another root's cache. Only the daemon initializer uses the persistent, guardian-protected `zircon-engine\cache\sccache-temporary`; Cargo, rustc, build scripts, and sccache clients use `zircon-engine\scratch\<job-id>\temporary`. Use `compact` to move Cargo's build-script directory into the same per-job scratch lifecycle. Use `diagnostic` only when symbols or Cargo defaults are required.
- Cargo home, compiler cache, and daemon TEMP are shared by jobs under the same managed storage root. Compiler-client TEMP remains job-scoped, so parallel requests cannot overwrite one another and retiring one job cannot invalidate the daemon. Job-owned scratch lives below `zircon-engine\scratch\<job-id>` and is deleted only after terminal process-tree observation and durable release; the stable cache/daemon-TEMP directories are bounded infrastructure and are not job cleanup targets.
- Explicit and inherited targets pass through the same allowlist, compatibility, and active-owner checks.
- Repo-local `target` directories, user-profile targets, temporary targets, other drives, and arbitrary external targets are forbidden for every Cargo build or validation command.
- Prefer crate-scoped loops such as `cargo build -p <crate>` and `cargo test -p <crate>` before escalating to full-workspace commands.
- Before build or test, preserve at least `35 GiB` on the drive that hosts the active target directory. Reject the run when the reserve is unavailable; do not erase a reusable hot pool and immediately rebuild it.
- Clean stale outputs through `tools\cleanup-stale-targets.ps1`, which previews the coordinator-owned deletion set before `-Apply`. Do not put an automatic `cargo clean` in a validation path.
- Treat release-size tweaks such as `[profile.release] strip = true` as secondary. They do not replace shared target directories, targeted builds, or cleanup.

## WSL Exception Mapping

- Prefer Windows-native validation. Apply `../../zircon-project-skills/prefer-windows-validation/SKILL.md` before launching WSL.
- Map each approved Windows root to the same root name below `/mnt/d`, `/mnt/e`, or `/mnt/f`.
- Use only the coordinator-selected WSL compatibility pool. A live Windows host wrapper must own and heartbeat the job while its WSL child runs.
- Windows and WSL compatibility keys are distinct. Never share one leaf across operating systems because Cargo artifacts are platform-specific.
- Never store WSL Cargo targets under `~`, `$HOME`, `/home/<user>`, or the repository.
- Direct unleased WSL Cargo commands are forbidden. Set `CARGO_TARGET_DIR` or `--target-dir` only to the coordinator-granted mounted path.

## Validator Target Resolution

- `validate-matrix.ps1` uses this priority order:
  1. Explicit `-TargetDir`, validated by the service.
  2. Inherited `CARGO_TARGET_DIR`, validated by the service.
  3. The service-selected compatible primary pool.
- Before entering `cargo build` or `cargo test`, the validator checks remaining free space on the target drive and refuses to start when the `35 GiB` reserve cannot be preserved.
- `-DryRun` computes the same compatibility-keyed pool path locally without coordinator state, Cargo discovery, storage admission checks, or filesystem creation. An actual validation run remains coordinator-owned.
- The validator records PID, rendered command and exit status, then releases the job in `finally`. The daemon marks a running job `orphaned` when its process disappears.

## Diagnostics

- Preview managed cleanup with `.\tools\cleanup-stale-targets.ps1`. Apply with `-Apply`; the service rechecks allowlist realpath, PID, active lease and retention immediately before deletion.
- Use `.\tools\install-session-coordinator-task.ps1 -Action Install -DryRun` to inspect the hidden at-logon daemon and 15-minute maintenance task before installation.
