# Cargo Target Disk Policy

- All Cargo output belongs to a coordinator-managed lane under an available drive-root target tree:

```powershell
D:\targets\zircon-engine\lanes\<kind>-<job-id>
```

- The service may choose `D:`, `E:`, or `F:` by available space. Machine-specific granted paths remain runtime data, not committed configuration.
- Each active target has one job owner. Explicit and inherited targets pass through the same allowlist and active-owner check.
- Repo-local `target` directories and arbitrary external targets are forbidden for development validation.
- Prefer crate-scoped loops such as `cargo build -p <crate>` and `cargo test -p <crate>` before escalating to full-workspace commands.
- Before build or test, check free space on the drive that hosts the active target directory. If remaining space is `<= 50 GB`, clean that target directory first.
- Clean stale outputs intentionally: use `cargo clean` after major profile or feature churn, or `cargo clean --release` when only release artifacts need pruning.
- Treat release-size tweaks such as `[profile.release] strip = true` as secondary. They do not replace shared target directories, targeted builds, or cleanup.

## Validator Target Resolution

- `validate-matrix.ps1` uses this priority order:
  1. Explicit `-TargetDir`, validated by the service.
  2. Inherited `CARGO_TARGET_DIR`, validated by the service.
  3. A fresh service-selected lane.
- Before entering `cargo build` or `cargo test`, the validator checks remaining free space on the target drive and runs `cargo clean --target-dir <active-target-dir>` first when that free space is `<= 50 GB`.
- The validator records PID, rendered command and exit status, then releases the job in `finally`. The daemon marks a running job `orphaned` when its process disappears.

## Diagnostics

- Preview managed cleanup with `.\tools\cleanup-stale-targets.ps1`. Apply with `-Apply`; the service rechecks allowlist realpath, PID, active lease and retention immediately before deletion.
- Use `.\tools\install-session-coordinator-task.ps1 -Action Install -DryRun` to inspect the hidden at-logon daemon and 15-minute maintenance task before installation.
