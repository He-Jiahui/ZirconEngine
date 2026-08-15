---
related_code:
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/asset/runtime_asset_path/diagnostics_enabled.rs
  - zircon_runtime/src/asset/runtime_asset_path/diagnostics_disabled.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/diagnostic_log
implementation_files:
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/asset/runtime_asset_path/diagnostics_enabled.rs
  - zircon_runtime/src/asset/runtime_asset_path/diagnostics_disabled.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features ai-contracts --locked --offline --jobs 1
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features ai-contracts,diagnostic-log --locked --offline --jobs 1
doc_type: module-detail
---

# Runtime Asset Path Resolution

`zircon_runtime::asset::runtime_asset_path` treats a nonempty `ZIRCON_ASSET_ROOT` as the authoritative product root. A relative value is resolved from the product executable directory and an absolute value remains an explicit external override; a missing asset stays bound to that root and cannot fall through to source-tree resources. Without the override, resolution selects the first existing root from the executable-local `assets` directory, caller-supplied development roots, and finally the crate asset directory. The process working directory is deliberately excluded because project `res://` resources are resolved by the project asset system, not by the built-in engine-resource resolver. The selected engine root remains stable for every requested file, so a missing product asset cannot be satisfied from a later development root. Input paths are normalized to safe relative form by dropping root, prefix, current-directory, parent-directory, and leading `assets` components.

## Diagnostics Boundary

Asset path resolution is foundational and must compile without the optional `diagnostic-log` domain. The owner module therefore selects one of two private adapters at module declaration time:

- `diagnostics_enabled.rs` delegates verbose checks and writes to the normal diagnostic log with scope `runtime_asset_path`.
- `diagnostics_disabled.rs` reports verbose output disabled and accepts no-op writes without importing the optional domain.

The path-selection functions contain no feature cfg and use the same adapter API in both builds. This keeps diagnostic behavior compile-time optional without making every independent contract feature depend on logging, and without duplicating path-selection logic or adding a runtime compatibility branch.

## Validation

Frameworks 03's contract boundary guard rejects a direct `crate::diagnostic_log` import from the foundational owner and verifies both adapter declarations. WSL nightly checks pass with `ai-contracts` alone and with `ai-contracts,diagnostic-log`, covering both compile-time owners.
