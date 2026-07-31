---
name: prefer-windows-validation
description: Use when selecting a ZirconEngine validation environment or target directory for Cargo check, build, test, debugging, CI reproduction, Linux-specific tooling, or milestone acceptance.
---

# Prefer Windows Validation

## Environment Decision

Use Windows PowerShell and the repository validator for ordinary ZirconEngine validation. Do not launch WSL merely for general confidence, routine Cargo checks, workspace builds, tests, or because another workflow historically preferred Linux.

Use WSL only when at least one concrete requirement remains:

- reproduce a Linux-only or WSL-only failure;
- match a specific Linux CI failure that Windows cannot reproduce;
- run a required Linux-only tool such as `valgrind`, `helgrind`, or `heaptrack`;
- validate Linux platform behavior explicitly required by the user or active milestone.

Record the reason before running WSL. Return to Windows validation when the Linux-specific requirement is satisfied. Treat Linux CI as cross-platform evidence, not as a mandate to duplicate every local check in WSL.

## Windows Default

Run the Windows validator from the repository root:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1
```

Let the coordinator allocate or reuse the single compatible pool below one of these roots:

```text
D:\cargo-targets
E:\cargo-targets
F:\cargo-targets
D:\targets
E:\targets
F:\targets
D:\ZirconBuilds
E:\ZirconBuilds
F:\ZirconBuilds
```

This nine-root allowlist is a hard build rule for all Cargo commands, not only validation. `CARGO_TARGET_DIR` and `--target-dir` must resolve below one of these roots. Do not build under a repository `target` directory, another drive, a user profile, a temporary directory, or any other location.

The reusable-pool compatibility key must include repository identity, platform (`windows` or `wsl`), Rust toolchain, target architecture, workspace identity, and canonical build configuration. Compatible work across Sessions shares one primary pool. Only one task may own that pool at a time; a busy pool must be waited on or reported busy, never bypassed with a second compatible pool. If the complete key cannot be established, treat the output as ephemeral and delete it immediately after release.

Use package-scoped Windows checks before escalating to full-workspace validation when the affected boundary permits it.

## WSL Cargo Target Placement

When WSL is justified, place every Cargo target under the mounted equivalent of the Windows target roots:

| Windows root | WSL root |
|---|---|
| `D:\cargo-targets` | `/mnt/d/cargo-targets` |
| `E:\cargo-targets` | `/mnt/e/cargo-targets` |
| `F:\cargo-targets` | `/mnt/f/cargo-targets` |
| `D:\targets` | `/mnt/d/targets` |
| `E:\targets` | `/mnt/e/targets` |
| `F:\targets` | `/mnt/f/targets` |
| `D:\ZirconBuilds` | `/mnt/d/ZirconBuilds` |
| `E:\ZirconBuilds` | `/mnt/e/ZirconBuilds` |
| `F:\ZirconBuilds` | `/mnt/f/ZirconBuilds` |

Do not invent a WSL leaf. Acquire the pool through the coordinator with `platform=wsl` and the complete compatibility document. Translate the granted Windows path to its `/mnt/d`, `/mnt/e`, or `/mnt/f` form only for the WSL child process. The Windows host process that launches `wsl.exe` must remain alive as the coordinator owner for the entire Cargo command and must start, heartbeat, finish, and release the job. If no coordinator-aware host launcher is available, do not run the WSL Cargo build.

Follow these rules:

- Never place WSL Cargo targets under `~`, `$HOME`, or `/home/<user>`.
- Never use a repo-local `target` directory.
- Never point Windows and WSL at the same leaf directory; `platform` is part of the compatibility key, so the coordinator grants distinct primary pools.
- Use only the mounted equivalents of the nine approved Windows roots; choose the drive and root through the coordinator.
- Do not run direct, unleased WSL Cargo commands. Set `CARGO_TARGET_DIR` or `--target-dir` to the coordinator-granted mounted path.
- Encode sanitizer and specialized-tool settings in the canonical build configuration so incompatible runs receive distinct keys; do not create ad-hoc leaves.

## Evidence And Closeout

- State whether validation ran on Windows or WSL.
- For WSL, state the Linux-specific reason and exact mounted target directory.
- Do not claim Windows and WSL validation both ran unless both commands completed successfully.
- Do not leave WSL target paths or acceptance instructions pointing to home-directory storage.
- Before closing validation work, scan touched instructions and commands for `~/`, `/home/`, and unmounted Cargo target paths.
