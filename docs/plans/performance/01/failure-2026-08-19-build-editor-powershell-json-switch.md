---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-08-19
summary_slug: build-editor-powershell-json-switch
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/performance/01
plan_link_mode: child_record_only
related_code:
  - tools/build-editor.ps1
  - tools/zircon-session.ps1
  - tools/tests/build-editor.Tests.ps1
tests:
  - Invoke-Pester -Script tools/tests/build-editor.Tests.ps1 -PassThru
  - powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-editor.ps1
---

# Performance01: build-editor does not bind coordinator JSON mode

## Failure evidence

The managed Editor product build reaches product-staging acquisition and then fails before Cargo
with `Coordinator product staging command returned invalid JSON`. The command output begins with
`Coordinator ready.` or `Coordinator launch accepted` and is followed by the valid JSON response.

`tools/build-editor.ps1` invokes the PowerShell wrapper as
`& $CoordinatorScript --json artifact @Arguments`. The wrapper declares `[switch]$Json`; PowerShell
does not bind the GNU-style `--json` token to that switch. The wrapper therefore emits its human
status banner while its Python child still receives the remaining JSON flag, and `ConvertFrom-Json`
rejects the combined output. The existing test coordinator used raw `$args`, so it could not expose
the production parameter-binding behavior.

## Ownership and minimum repair

Performance01 owns the product-build workflow. Bind the wrapper's declared PowerShell switch with
`-Json`, and make its test double expose the same command, switch, and remaining-argument contract as
`tools/zircon-session.ps1`. Keep the coordinator as the sole product-staging authority.

Do not parse around arbitrary banner text, call the Python coordinator module directly, publish
outside the approved artifact roots, or bypass product-staging leases.

## Acceptance

- The focused Pester suite first reproduces the invalid mixed-output failure against the current
  invocation, then passes after the PowerShell switch is bound correctly.
- The real managed Editor build reaches Cargo through `tools/build-editor.ps1`, publishes a complete
  Editor bundle, and passes its normal smoke gate.
- Artifact audit reports no unmanaged product inputs after the build lifecycle completes.
