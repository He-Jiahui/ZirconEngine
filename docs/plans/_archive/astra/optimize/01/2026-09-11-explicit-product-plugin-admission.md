---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-11
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: PLUGIN-A1/A2 explicit product plugin selection admission
session: astra-plugin-selection-explicit-product-20260911-r2
---

# Explicit product plugin selection admission

This bounded App slice closes the explicit-registration gap at the product
composition boundary. Every effective manifest selection now keeps its typed
`PluginSelectionResolution` outcome, while only registrations returned by the
resolver are forwarded to the product catalog. Required `InvalidId`,
`Duplicate`, or `Unsupported` outcomes fail preparation before catalog
construction/Ready; optional unsupported rows remain observable and do not
block composition.

## Finding status and ownership

| Finding | Status | Evidence and lowest owner |
|---|---|---|
| P0 `PLUGIN-A1` selection silent drops / required admission | `implemented_pending_validation` | `zircon_app/src/entry/engine_entry.rs:328-376,474-520` carries typed outcomes through both explicit constructors, resolves against the effective manifest, and applies required fail-closed conversion. Lowest owner for this App boundary is `zircon_app::entry`; shared resolver contract remains `zircon_runtime::core::framework::project`. |
| P0 `PLUGIN-A2` carrier roles entering product catalog | `implemented_pending_validation` | `zircon_app/src/entry/engine_entry.rs:484-494` filters `PluginPackageRole` to product-catalog-eligible roles before resolution/catalog; role predicate is `zircon_runtime/src/plugin/package_manifest/plugin_package_role.rs:5-20`. Lowest owner for this explicit path is `zircon_app::entry`; generated/editor catalog role policy remains in `zircon_runtime::plugin`. |
| P0 `PLUGIN-A3` native trust admission | `implemented_pending_validation` | Existing native admission remains separately owned by `zircon_runtime::plugin::native`; this slice does not execute DLLs or alter trust policy. Real DLL/ABI and managed Windows evidence remain pending. |

## Implementation and tests

- `zircon_app/src/entry/engine_entry.rs:474-520` filters Sample/TestFixture
  reports, resolves each effective selection, retains all typed outcomes, and
  forwards only `resolution`'s selected registrations (not every explicit
  input report) to the catalog.
- `zircon_app/src/entry/product_composition/composition.rs:30-47` makes the
  existing editor outcome append idempotent by retaining exact outcomes only
  once. Existing editor callers at
  `zircon_app/src/entry/entry_runner/editor.rs:241-259` and
  `zircon_app/src/entry/entry_runner/editor/composition.rs:72-83` therefore do
  not duplicate runtime outcomes now owned by explicit composition.
- Focused regressions in
  `zircon_app/src/entry/tests/product_composition.rs:136-267` cover optional
  unsupported outcomes, Sample/TestFixture exclusion, required fixture
  rejection, extra unselected report exclusion, resolved status, and
  idempotent editor append.

## Static evidence and validation boundary

`rustfmt --check --edition 2021` passed for all three Rust files, and scoped
`git diff --check` passed. A static source assertion confirmed role filtering
precedes resolution, selected-report collection is present, both explicit
constructors use the helper, and all focused test names are present. No Cargo,
native, DLL, or product command was run: managed validation is deferred by the
external dirty `E:\Git\zr_vm` dependency. This record therefore remains
`implemented_pending_validation`, not accepted.

## Residual risks and dependency-ready follow-up

`RuntimePluginFeatureRegistrationReport` currently has no typed package-role
field, so feature-only explicit reports can still require a separate
provider-role admission slice in the lowest runtime catalog/native projection
owner. The runtime catalog also compares some project-selection IDs by raw
string after the shared resolver canonicalizes aliases; alias-specific
explicit product coverage should be added in
`zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project/selection.rs`
with its focused tests before claiming the whole PLUGIN-A1 contract. The next
dependency-ready slice is that runtime-owner role/alias convergence, followed
by managed Runtime/App/Editor Cargo and real native trust validation.

## Coordination

Session `astra-plugin-selection-explicit-product-20260911-r2` is
`waiting_validation`. Exact source, test, and record leases were explicitly
released after the edits (final release request
`4618d6ac95fa45658a2c63eac60d9018`, `released=4`); a subsequent lease listing
showed no paths owned by this session. No commit was created.

## 2026-09-25 managed validation increment

- `zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs` now projects
  `SDK_API_VERSION` into importer package manifests instead of retaining the
  runtime package constructor's legacy `0.1.0` default. Managed Windows
  validation passed `cargo check`; `importer_runtime_manifest_builder` passed
  `3/3` with one release-only performance test ignored.
- The `declare_plugin!` native registration-manifest macro no longer emits
  dangling commas after optional system-set/access repetitions. This was a
  real SDK test-compilation error; the same managed lane then compiled and
  executed successfully.
- With `ui-document-importer`, catalog `runtime_catalog` projection passed
  `2/2`, and descriptor-to-generated-manifest parity passed `1/1`.
- A full catalog test invocation still has an independent source-sealing
  limitation: tests invoking `tools/audits/audit_plugin_structure.py` cannot find
  that ignored tooling file inside the managed source snapshot. Those audit
  tests remain failures and are not reclassified as product acceptance.
