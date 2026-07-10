# Cargo Target Disk Policy

- Prefer one shared `CARGO_TARGET_DIR` outside the repository across clones and worktrees when local disk usage matters.
- PowerShell example:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
```

- Keep machine-specific paths out of committed repository files.
- Do not run multiple conflicting Cargo writers against the same shared target directory at the same time.
- Prefer crate-scoped loops such as `cargo build -p <crate>` and `cargo test -p <crate>` before escalating to full-workspace commands.
- Before build or test, check free space on the drive that hosts the active target directory. If remaining space is `<= 50 GB`, clean that target directory first.
- Clean stale outputs intentionally: use `cargo clean` after major profile or feature churn, or `cargo clean --release` when only release artifacts need pruning.
- Treat release-size tweaks such as `[profile.release] strip = true` as secondary. They do not replace shared target directories, targeted builds, or cleanup.

## Validator Target Priority

- `validate-matrix.ps1` uses this priority order:
  1. Explicit `-TargetDir`
  2. Inherited `CARGO_TARGET_DIR`
  3. Repo-local fallback slots `target/codex-shared-a` or `target/codex-shared-b`
- Before entering `cargo build` or `cargo test`, the validator checks remaining free space on the target drive and runs `cargo clean --target-dir <active-target-dir>` first when that free space is `<= 50 GB`.
- The repo-local slots are a fallback for sessions that did not preconfigure a shared `CARGO_TARGET_DIR`; they are not the preferred steady-state workflow across multiple checkouts.

## Diagnostics

- When `target` grows unexpectedly, measure subdirectory sizes first so cleanup decisions distinguish `debug`, `release`, and one-off custom target paths.
- When recommending new clones or worktrees, mention the shared `CARGO_TARGET_DIR` strategy up front instead of assuming each checkout should own a full `target`.
