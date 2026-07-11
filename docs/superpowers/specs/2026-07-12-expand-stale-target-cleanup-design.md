# Expand stale target cleanup design

## Goal

Extend `tools/cleanup-stale-targets.ps1` so one cleanup command covers build-output
directories below these drive-root locations when the drive is available:

- `D:\cargo-targets`, `E:\cargo-targets`, `F:\cargo-targets`
- `D:\targets`, `E:\targets`, `F:\targets`
- `D:\ZirconBuilds`, `E:\ZirconBuilds`, `F:\ZirconBuilds`

## Selected approach

Keep the Session coordinator as the owner of managed Cargo lane cleanup. The wrapper
script continues to request and optionally apply the coordinator cleanup plan first.
It then scans the configured roots for direct child directories that the coordinator
does not manage and handles those directories locally.

An unmanaged directory is a direct child of one of the configured roots whose
canonical path is absent from both the coordinator plan's candidates and denied
entries. This lets active, retained, or otherwise denied managed lanes remain under
the coordinator's protection while permitting old untracked build output to be
removed.

## Safety and behavior

- Without `-Apply`, print both managed and unmanaged cleanup candidates and delete
  nothing.
- With `-Apply`, apply the reviewed managed plan and directly delete the reviewed
  unmanaged candidates.
- Apply `OlderThanHours` to unmanaged direct children using their last-write time, so
  the command remains a stale-output cleanup rather than an unconditional purge.
- Revalidate existence, root containment, direct-child depth, and age immediately
  before each unmanaged deletion.
- Never delete a configured root itself, nested paths selected independently, files,
  junctions, symbolic links, or other reparse points.
- Skip missing drives and missing roots without creating them.
- Preserve PowerShell `ShouldProcess`, including `-WhatIf`, for every local deletion.
- A failed local deletion is reported and does not cause another path to be selected.

## Validation

Add isolated PowerShell tests using temporary directory fixtures for root discovery,
managed/unmanaged classification, age filtering, preview mode, apply mode, and the
root/depth/reparse-point guards. Run the existing Session coordinator cleanup tests to
confirm that managed-lane behavior remains unchanged.
