# Prefer Windows Validation

Use Windows PowerShell and `tools/dev/local-cargo.ps1` for ordinary ZirconEngine checks. The coordinator and its managed validator are retired.

Use WSL when the task needs a Linux-only failure reproduction, a specific Linux CI failure, a required Linux-only tool, or explicitly requested Linux-platform evidence. Record that reason and return to Windows when it is satisfied. A successful Windows run does not automatically require a duplicate WSL run.

Command and batch selection belong to [Zircon Dev Validation](../../zircon-dev/validation/guide.md). Storage roots, compatibility identity, retention, free-space admission, and cleanup belong to [Cargo target disk policy](../../zircon-dev/references/cargo-target-disk-policy.md).

For WSL, use `python3 -B -m tools.dev.local_cargo` from the Linux checkout with a platform-specific target below the approved mounted D/E/F root. Keep the invoking process alive until the command is terminal. Preserve Cargo's locks and separate Windows/WSL leaves; no coordinator lease or heartbeat is required.

Report the actual platform, command scope, and result. For WSL include the Linux-specific reason and physical mounted target. Reuse matching evidence; do not claim both platforms were tested unless both actually completed.
