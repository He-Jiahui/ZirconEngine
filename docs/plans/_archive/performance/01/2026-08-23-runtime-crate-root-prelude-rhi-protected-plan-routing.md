---
title: Runtime Crate Root Prelude and RHI Protected Plan Routing
date: 2026-08-23
status: routing_only
source_report:
  - docs/plans/performance/01/2026-08-23-runtime-crate-root-prelude-rhi-current-source-review.md
---

# Runtime Crate Root Prelude and RHI Protected Plan Routing

| Existing owner | Required adoption |
|---|---|
| Runtime42 profile/catalog owner | Keep Cargo feature presence, manifest selection, compiled module catalog and readiness as separate facts. Add target feature/output/startup measurements before changing unconditional modules. |
| Runtime46 engine-module owner | Ensure a compiled/linked module is not reported Ready without its selected dependencies, product owner, health and teardown. |
| Runtime90 RHI owner | Preserve `rhi.rs` as a curated facade only. Replace the editor-owned WGPU fallback with the single production device-generation/presenter lease path; measure any explicit recovery fallback separately. |
| App/editor build owners | Preserve `default-features = false` for server selection and add static feature-matrix guards. Client/editor feature union changes require artifact/startup measurements. |

Protected `docs/plans/performance/review.md` and `docs/plans/performance/pending.md` remain untouched. The protected-ledger owner may record one concise crate-root entry after adopting this report; dynamic status remains pending.
