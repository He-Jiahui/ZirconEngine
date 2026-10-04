---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-22
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: W5 NET-P1-001/002/004/005 ordinary HTTP and WebSocket provider admission
session: astra-net-first-party-provider-admission-20260922
validation_session: astra-net-feature-backend-readiness-20260924
---

# Ordinary HTTP/WebSocket provider catalog admission

## Finding and scope

The ordinary first-party path selected the root `net` package but did not pass
concrete `net.http` or `net.websocket` feature-provider registrations into
`RuntimePluginCatalog`. The feature factories also constructed private
`DefaultNetManager` values, so a selected feature could not install its backend
on the canonical net authority.

This slice keeps the existing root catalog and generic feature-selection
contract intact. It adds the first-party catalog result for root and feature
reports, forwards that result through the ordinary app composition path, and
changes both feature factories to resolve and augment the root manager. WSS
security policy, native plugin loading, and product two-process acceptance are
outside this record.

## Implementation and tests

- `zircon_first_party_runtime_catalog` now links the HTTP and WebSocket runtime
  feature crates under `base-runtime-plugins` and exposes
  `FirstPartyRuntimeCatalogReport` with root and feature registration reports.
- `zircon_app` forwards the combined report through the ordinary
  `BuiltinEngineEntry` path while retaining the root-only helper for callers
  that intentionally do not provide feature reports.
- `DefaultNetManager` exposes the canonical manager name and backend-install
  operations. HTTP and WebSocket feature factories resolve that manager and
  install their backend instead of allocating a second authority. Their
  feature-manager descriptors use immediate startup so selected transports are
  installed during module activation, before any consumer resolves the root
  manager; neither feature opens a listener simply by being selected.
- Catalog tests cover selected-net feature publication and omission when net is
  not selected. HTTP and WebSocket tests assert backend observation immediately
  after activation, before feature-manager resolution, then assert manager
  identity. The ordinary app bootstrap test requests both feature rows and
  asserts both feature modules enter the composition and activate together
  during an ordinary bootstrap. The app does not depend directly on the net
  implementation crate, so concrete backend inspection remains in the feature
  crate tests.
- The Python static contract test checks dependency wiring, ordinary-path
  forwarding, and canonical-manager installation. The stale selection-contract
  fixture now reads the report-returning catalog API.

## Evidence boundary

Passing evidence:

- `python -m unittest tools.tests.test_astra_net_first_party_feature_provider_contract tools.tests.test_astra_plugin_selection_and_trust_contract` (10/10)
- After adding activation-time assertions, the static contract failed for
  both lazy feature descriptors, then the HTTP/WebSocket immediate-startup
  change passed the expanded focused static run. An additional red/green
  contract guards the ordinary app bootstrap assertion (14/14 including the
  related net feature identifier contract). The new Rust activation assertions
  and ordinary app bootstrap test have not yet run.
- focused runtime catalog/plugin static contracts (6/6),
  `python tools/audits/audit_plugin_structure.py` (39 manifests, 41 dist entries,
  all reported gates clean)
- `rustfmt --edition 2021 --check` on the touched Rust files and scoped
  `git diff --check`

The managed Windows focused Cargo attempt for
`zircon_plugin_net_http_runtime` was submitted with the canonical test filter,
but the coordinator rejected closure planning because a concurrently edited
unrelated input (`zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests/mod.rs`)
changed during the immutable-input check. A prior attempt also encountered a
stale locked-copy view before the current `Cargo.lock` entries were visible.
The later managed `zircon_app` check with `first-party-runtime-plugins`
likewise stopped during input synchronization when
`zircon_runtime/src/ui/text/geometry/tests.rs` changed concurrently; it did
not return a compiler result. The next native gate is a managed app check and
focused HTTP/WebSocket and ordinary bootstrap tests against stable inputs.
No Rust compile, native runtime, or product acceptance is claimed by this
record; the validation receipt remains pending and the external
`E:\\Git\\zr_vm` worktree remains dirty.

No commit was created.
