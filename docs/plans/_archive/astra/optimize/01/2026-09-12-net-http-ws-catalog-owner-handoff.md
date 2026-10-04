---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: W5 NET-P1-001/002/004/005 ordinary HTTP and WebSocket provider admission
session: astra-w5-net-provider-catalog-handoff-20260912
---

# Ordinary HTTP/WebSocket provider catalog owner handoff

## Finding

The ordinary first-party runtime path admits the root `net` package, but it
does not publish concrete HTTP or WebSocket feature-provider registrations.
Consequently a client/editor manifest may declare `net.http` or
`net.websocket` while the normal `zircon_app` composition has no feature
module to activate. This is the provider-closure portion of the existing
`NET-P1-001`, `NET-P1-002`, `NET-P1-004`, and `NET-P1-005` findings; it is
independent of the WSS security-policy slice recorded in
`2026-09-11-net-wss-policy.md`.

## Current-source evidence

* `zircon_plugins/first_party_runtime_catalog/src/lib.rs:13-20` returns a
  `PluginSelectionResolutionReport<RuntimePluginRegistrationReport>` and
  resolves each selected runtime id through
  `first_party_registration_for_runtime_plugin`. Its `RuntimePluginId::Net`
  branch at `:38-41` returns only
  `zircon_plugin_net_runtime::plugin_registration()`; there is no HTTP or
  WebSocket feature registration branch.
* `zircon_plugins/first_party_runtime_catalog/Cargo.toml:10-20,36-53` enables
  `zircon_plugin_net_runtime` for `base-runtime-plugins`, but has no
  `zircon_plugin_net_http_runtime` or
  `zircon_plugin_net_websocket_runtime` dependency/feature. The ordinary
  catalog therefore cannot call either feature crate without a separate
  composition contract.
* `zircon_plugins/net/runtime/src/package.rs:24-45` declares `net.http` and
  `net.websocket` as optional feature metadata and names their provider
  crates. Metadata is not a concrete registration. The same file's
  `:83-86` makes content download depend on HTTP, so a missing HTTP provider
  blocks that dependent feature as well.
* `zircon_plugins/net/runtime/src/plugin.rs:50-67` registers root options,
  event catalogs, and systems only. It does not register optional feature
  modules.
* `zircon_plugins/net/runtime/src/module.rs:21-38` constructs the canonical
  `DefaultNetManager` with no HTTP/WS backend. The `NetManager` wrapper at
  `:39-54` resolves that same backend-less instance.
* `zircon_plugins/net/features/http/runtime/src/feature.rs:34-44,46-64`
  exposes `plugin_feature_registration()` and a module factory, but the
  factory ignores its dependency core (`factory(|_|`) and creates a fresh
  `DefaultNetManager` with an HTTP backend. The WebSocket implementation does
  the same at
  `zircon_plugins/net/features/websocket/runtime/src/feature.rs:35-45,47-65`.
* The feature package manifests confirm the dependency direction:
  `zircon_plugins/net/features/http/runtime/Cargo.toml:8-16` and
  `zircon_plugins/net/features/websocket/runtime/Cargo.toml:8-13` both depend
  on `zircon_plugin_net_runtime`. Adding these crates as dependencies of the
  root net runtime would create a cycle; adding catalog branches alone would
  still leave the canonical-manager/backend injection unresolved.
* The ordinary app route requests only root reports. In
  `zircon_app/src/entry/first_party_runtime_plugins.rs:19-26,51-72`, the
  app delegates to the root-report catalog API. In
  `zircon_app/src/entry/engine_entry.rs:284-304`, that report is passed to
  `builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_registrations`.
  `zircon_app/src/entry/builtin_modules.rs:37-46` builds the catalog with
  `std::iter::empty()` for feature reports. The feature-aware route at
  `:55-71` exists only for explicit reports.
* `zircon_app/src/entry/product_composition/request.rs:175-186` confirms the
  split: `FirstPartyCatalog` calls the root-only constructor, while only
  `ExplicitReports` forwards `runtime_plugin_feature_registrations`.
* The runtime catalog deliberately requires concrete feature reports before
  proposing feature modules. `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project/selection.rs:88-90`
  indexes provider reports, and `:225-239,464-468` selects feature reports
  and their modules. Existing diagnostics in
  `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection/tests.rs:278-285`
  identify the missing case as “concrete runtime feature provider
  registration is missing”. A root feature row alone cannot activate HTTP or
  WebSocket.

The current network review states the same product break at
`docs/plans/optimize/zircon_runtime/217-runtime-network-transport-session-rpc-replication-prediction-rollback-security-content-product-current-working-tree-review.md:109-142`:
the first-party catalog reaches “root net runtime only”, HTTP/WebSocket use
private managers, and metadata dependency is not instance injection. Its
canonical ledger keeps `NET-P1-001`, `NET-P1-002`, `NET-P1-004`, and
`NET-P1-005` open at `:225-234`.

## Ownership and overlap

No network source path is leased by this handoff. The generic runtime catalog
surface is actively owned by these registered primary sessions:

* `astra-plugin-catalog-convergence-20260912` — project selection and tests;
* `astra-plugin-catalog-feature-alias-20260912` — feature context/support and
  compiled-selection tests;
* `astra-plugin-catalog-feature-contract-20260912` — feature-definition lookup
  and registration matching.

The related Astra records
`2026-09-12-plugin-catalog-alias-convergence.md:27-35` and
`2026-09-11-explicit-product-plugin-admission.md:56-63` already mark feature
provider role/provenance and ordinary feature admission as unresolved. The
WSS source/test paths remain owned by the separate waiting-validation session
`astra-w5-network-wss-fail-close-20260911` and were not edited here.

## Why there is no safe lowest-layer fix in this slice

This defect cannot be closed by adding a single catalog branch or by changing
the root manager factory:

1. The catalog API and app ordinary path currently return/consume only root
   registration reports, so a feature branch would need a new typed result and
   ordinary composition plumbing for feature reports, target/profile feature
   selection, and required fail-closed diagnostics.
2. The generic catalog feature-report contract is under active ownership and
   still lacks typed provider-role provenance. Changing it here would overlap
   those leases and could reject valid external feature-only providers.
3. Once admitted, HTTP and WebSocket factories must install their backends into
   the canonical net-manager instance. Their current factories ignore `core`
   and allocate private authorities. Folding feature crates into
   `net/runtime` is not a safe shortcut because both feature crates already
   depend on that runtime.

Therefore no TDD source implementation was started. A helper or test-only
change would leave the ordinary product path broken and falsely suggest
closure.

## Dependency-ready owner order and first implementation slice

After the active catalog owners publish a stable feature-report/provenance
contract, the next bounded production slice should be coordinated as follows:

1. **Catalog owner:** expose a first-party result containing root reports plus
   concrete HTTP/WS feature reports, preserving canonical IDs, package roles,
   target modes, and required-feature diagnostics.
2. **App composition owner:** feed those reports through the ordinary
   `FirstPartyCatalog` path into `RuntimePluginCatalog`; select only the
   profile-requested feature rows and fail closed when a required provider is
   absent. Add focused client/editor admission tests.
3. **Net provider owner:** change the HTTP/WS module factories to resolve a
   typed same-generation `DefaultNetManager` lease and install the backend on
   that authority (or introduce an explicitly owned backend-install service),
   with no second manager/runtime. Add focused identity/backend-observation
   tests before product integration.
4. **Validation owner:** run the managed Windows catalog/app/net focused gates,
   then the two-process HTTP/WS product scenario. Cargo/native validation is
   intentionally outside this handoff.

Prospective exact source scope after ownership transfer:

* `zircon_plugins/first_party_runtime_catalog/src/lib.rs`
* `zircon_plugins/first_party_runtime_catalog/Cargo.toml`
* `zircon_app/src/entry/first_party_runtime_plugins.rs`
* `zircon_app/src/entry/engine_entry.rs`
* `zircon_app/src/entry/builtin_modules.rs`
* `zircon_app/src/entry/product_composition/request.rs`
* `zircon_plugins/net/features/http/runtime/src/feature.rs`
* `zircon_plugins/net/features/websocket/runtime/src/feature.rs`
* corresponding focused tests and any newly authorized catalog contract path

This record is a blocked owner handoff, not an implementation or acceptance
claim. It made no source edits, ran no Cargo/native commands, and created no
commit. Static source inspection is the only evidence in this lane; managed
validation remains subject to the coordinator's active external dirty
`E:\\Git\\zr_vm` dependency gate.
