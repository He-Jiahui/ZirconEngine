---
related_code:
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - tools/tests/test_frameworks_04_gltf_hot_reload_contract.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
  - docs/plans/zircon_runtime/frameworks/development-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -B -m unittest tools.tests.test_frameworks_04_gltf_hot_reload_contract -v
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_dist --locked gltf_importer_dist_state_round_trips_and_rejects_another_schema
---

# Frameworks04 M4 glTF state callback hardening

Status: `implementation_complete_static_and_second_review_green_managed_validation_pending`

## Completed Work

- Confirmed the glTF dist descriptor already owns non-stateless `save_state`, `restore_state`, and
  `unload` callbacks with a versioned state schema; this record does not claim those existing
  callbacks as newly implemented.
- Removed the remaining production `expect` from state restoration. The callback now copies the
  already length-validated epoch bytes into a fixed array before decoding, without changing ABI,
  schema, diagnostics, or invalid-state behavior.
- Routed all three `extern "C"` state callbacks through the SDK-owned panic catcher, so future
  implementation panics project a stable callback status instead of unwinding across the ABI.
- Added a focused source contract that keeps all three ABI callbacks wired, preserves the stateful
  descriptor fields, requires each callback panic boundary and the exact epoch-length precondition,
  and rejects `expect`, `unwrap`, and `panic!` in the production dist owner. The Rust behavior test
  also rejects truncated and overlong state without mutating the published epoch.

## Evidence And Remaining Acceptance

- TDD RED was observed before the implementation change because the fixed-array copy and panic-free
  production contract were absent.
- Focused Python 1/1, Python compile, Rust 1.94.1 scoped rustfmt, and exact-scope
  `git diff --check` are GREEN.
- First independent review returned Critical 0 / Important 2 / Minor 1. The missing panic
  boundaries, incomplete length regression, and edition-aware rustfmt evidence are repaired.
- Independent exact-scope second review returned Critical 0 / Important 0 / Minor 0 and repeated
  the focused Python, Python compile, Rust 1.94.1 edition-2024 rustfmt, and exact-scope diff checks
  successfully.
- Managed package-level Rust validation remains pending. This bounded hardening does not mark
  Frameworks04 M4 or its parent plan accepted.
