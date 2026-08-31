---
status: source_complete_validation_pending
related_code:
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs
tests:
  - sdk_resource_registration_supports_concurrent_world_initialization
  - resource_registration_rejects_unknown_owner_before_mutating_registry
  - resource_registration_rejects_non_runtime_owner_before_mutating_registry
---

# Runtime15 Plugin SDK resource registration concurrency

## Decision

The plugin SDK registration surface now exposes the same immutable `Fn + Send + Sync`
resource factory contract as the runtime registry. Resource registration therefore
does not serialize world initialization behind a per-registration mutex, while each
world still receives a fresh value from the factory.

## Source evidence

`registration.rs` adds a public-SDK regression that clones the shared registration
source into two independent world-creation lanes. Each lane finalizes its local
registry before applying the extension plan; the factory records a bounded overlap
receipt. The test also
asserts that each world contains the registered resource and that the active-call
counter returns to zero. The bounded wait is intentional: it detects serialized
factories without turning an older implementation into a test deadlock.

The runtime owner remains free of production `Mutex` state for resource factories;
resource registration now also validates the interned plugin owner before mutating
the typed extension point. Duplicate registration still returns
`DuplicatePluginResource` through the typed registry error path, while an unknown
owner or an interned non-`<plugin>.runtime` owner returns `InvalidPluginModule`
without leaving an orphaned resource entry.

## Validation

- `rustfmt --edition 2021 --check zircon_plugins/plugin_sdk/src/registration.rs`: passed.
- `git diff --check`: passed.
- `python tools/audit_plugin_structure.py --json`: passed with registration-builder
  violations `0`, manifest-schema violations `0`, skeleton migration debt `0`, and
  editor/runtime mirror violations `0`.
- `python -m unittest tools.tests.test_plugin_structure_audit_capability -v`: passed
  `5/5`.
- Source guard for concurrent factory and unknown-owner rejection: passed.
- Current source fingerprint (2026-08-30): `zircon_plugins/plugin_sdk/src/registration.rs`
  is 516 lines with SHA-256
  `CF0B279E49109FFE335E0B86090DA4A4CB9F7B25ECA17F8EB4778C01030AAE94`.
- Global `python tools/check_conventions.py --only docs --json`: remains RED with
  `1,522` shared-worktree stale-path findings. The two paths declared by this record
  exist, so this source slice does not claim or absorb that foreign documentation debt.
- Managed Cargo focused SDK test: pending the coordinator Cargo lane; no Cargo pass is
  claimed by this source record.

The earlier Runtime15 immutable source snapshot `2369` predates this SDK-only test
addition and is therefore not used as evidence for the current file set. A fresh
snapshot and managed validation remain coordinator-owned follow-up work.

Status token: `runtime_15_plugin_sdk_resource_concurrency_source_complete_static_passed_cargo_deferred`
