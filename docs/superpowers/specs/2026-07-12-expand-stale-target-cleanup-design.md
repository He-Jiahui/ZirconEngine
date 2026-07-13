# Cargo target single-pool reuse and cleanup design

## Goal

Prevent Cargo build output from exhausting local disks while preserving the highest
safe amount of compilation reuse. Session-managed jobs share one primary target pool
per compatibility key across Sessions. Jobs without a complete compatibility key are
non-reusable and their target directory is deleted immediately after release.

The standalone cleanup wrapper also covers stale unmanaged output below the same
allowlisted roots without bypassing coordinator protection for managed lanes.

## Hard build-root allowlist

Cargo output may exist only below these drive-root locations when the drive exists:

- `D:\cargo-targets`, `E:\cargo-targets`, `F:\cargo-targets`
- `D:\targets`, `E:\targets`, `F:\targets`
- `D:\ZirconBuilds`, `E:\ZirconBuilds`, `F:\ZirconBuilds`

WSL uses only the mounted equivalents below `/mnt/d`, `/mnt/e`, or `/mnt/f`. A WSL
Cargo command is allowed only through a coordinator-aware Windows host wrapper that
acquires with `platform=wsl`, remains alive as owner, heartbeats, and translates the
granted Windows path for its child. Every Cargo command must set `CARGO_TARGET_DIR` or
`--target-dir` to the coordinator-granted descendant of one of these roots.
Repository-local `target`, user-profile/home directories, temporary
directories, other drive roots, and every other location are rejected by both the
Session coordinator and repository-local skills/validators.

## Compatibility key

A reusable acquisition supplies a canonical compatibility document containing all of
these fields:

- repository: injected by the coordinator from the registered repository identity;
- platform: exactly `windows` or `wsl`;
- toolchain: the effective Rust toolchain/rustc identity;
- target architecture: explicit Cargo target triple or the rustc host triple;
- workspace: canonical repository-relative Cargo workspace/manifest identity;
- build configuration: canonical profile, feature set, Rust flags, incremental/debug
  policy, and other compilation-affecting settings.

The coordinator validates the document, canonicalizes it, adds the repository
identity, and hashes the result. An incomplete, malformed, or conflicting document is
not silently broadened into a reusable key. Callers either correct it or acquire an
ephemeral target.

Source and lockfile changes do not create a new pool: Cargo's own fingerprints
invalidate affected units inside the compatible pool. Windows and WSL, different
toolchains/architectures/workspaces, or different build configurations never share a
pool.

## Single primary pool and exclusivity

Each compatibility hash maps to one retained primary target directory. The first
acquisition places it on the allowlisted root with the most free space. Later Sessions
reuse that exact directory.

The coordinator serializes acquisition in SQLite and treats an active lease or
running job for the compatibility hash as exclusive ownership. A concurrent request
receives a stable `cargo_reuse_pool_busy` denial; it never creates a second fallback
pool. Cleanup reservations also block acquisition. These rules guarantee one writer
and one retained directory per compatibility key.

If the primary directory was evicted or disappeared, the next acquisition creates one
replacement primary directory and marks the previous retained record as deleted. It
still never creates two live retained directories for one key.

If legacy or imported history contains multiple existing retained directories for one
key, acquisition selects the newest as authoritative and demotes every other directory
to prompt `delete_on_release` cleanup before granting new ownership.

## Non-reusable lifecycle

An acquisition without a complete compatibility document, or with explicit
`ephemeral` intent, receives a unique target directory and `delete_on_release` policy.
Release commits terminal ownership state and wakes a single prompt worker. The worker
drains requests that arrive during an active pass, reserves each directory, revalidates
that no overlapping active lease or live process exists, deletes outside the database
writer transaction, and records the result.

Deletion failure records `failed` plus a sanitized error. The periodic maintenance
loop retries `pending` and `failed` deletions. Orphan reconciliation applies the same
policy after process liveness has been disproved.

## Disk-pressure eviction

Reusable pools remain available while their drive has more than 50 GiB free. After a
release and during periodic maintenance, each pressured root evicts idle reusable
pools in least-recently-used order until the reserve is restored or no safe idle pool
remains. Active leases, live processes, cleanup reservations, and the root itself are
never deletion candidates.

Normal stale cleanup remains available for idle managed pools older than
`OlderThanHours`. Immediate ephemeral deletion and pressure eviction do not wait for
that retention window.

## Unmanaged cleanup wrapper

`tools/cleanup-stale-targets.ps1` first requests the coordinator cleanup plan. It then
scans the nine roots for direct child directories absent from both the coordinator
candidate and denial sets.

- Without `-Apply`, it prints managed and unmanaged candidates and deletes nothing.
- With `-Apply`, it applies the reviewed managed plan and directly removes only the
  reviewed unmanaged candidates older than `OlderThanHours`.
- Before every local deletion it revalidates existence, canonical root containment,
  direct-child depth, age, type, and absence of reparse-point attributes.
- It never deletes an allowlisted root, file, junction, symbolic link, or independently
  selected nested path; missing drives/roots are skipped and never created.
- PowerShell `ShouldProcess`, including `-WhatIf`, governs every local deletion.

## Validation

Focused Python tests cover key validation, cross-Session reuse, incompatibility,
single-pool busy denial, replacement after eviction, ephemeral release deletion,
retry, orphan handling, pressure LRU eviction, and the nine-root path policy. Pester
tests cover validator enforcement and isolated unmanaged cleanup fixtures. Existing
coordinator cleanup, server, migration, and validation-matrix suites must remain green.
