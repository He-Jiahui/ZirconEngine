---
Plan: docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md
Milestone: M2
Status: completed
Files: ["zircon_runtime/tests/material_shader_redirect_dependency_contract.rs", "docs/plans/zircon_runtime/render/18/fixed-2026-07-16-material-redirect-asset-contract-drift.md", "docs/plans/zircon_runtime/shader/03/2026-07-16-m2-material-redirect-contract-acceptance.md"]
---

# M2 Material Redirect Asset Contract Acceptance

## Scope delivered

The Shader03 redirect-material contract test now builds materials through
`ZMaterialDocument`, persists the compound shader reference through
`ProjectManager::persist_runtime_reference`, and compares asset references by
`AssetUuid`. The canonical cross-plan repair is recorded in
[the Render18 fixed handoff](../../render/18/fixed-2026-07-16-material-redirect-asset-contract-drift.md).

## Fresh testing evidence

| Milestone | Stage | Status | Evidence |
| --- | --- | --- | --- |
| M2 | M2-T managed exact testing | passed | Coordinator runner job `4580ace9efab44ab99f14867d9d3a958` exited 0 with 2 passed, 0 failed, 0 ignored. Persisted stdout/stderr are retained under the managed cargo-run log. |

## Review

The distinct Shader03 review Session submits its zero-critical and zero-important result through the coordinator before the M2 commit. The coordinator binds that review to this exact manifest and rejects the commit if it is stale or missing.
