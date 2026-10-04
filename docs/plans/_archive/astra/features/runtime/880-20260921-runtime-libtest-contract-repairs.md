---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/07/2026-08-26-vm-qualified-family-index.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/script/vm/backend/backend_registry/qualified_lookup_tests.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs
tests:
  - zircon_runtime/src/script/vm/backend/backend_registry/qualified_lookup_tests.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
---

# Runtime880 Library-Test Contract Repairs

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime07 VM qualified-family regression | Import `MockVmBackend` from its actual `script::vm::backend` owner after the public-surface migration; retain the existing lookup and percentile assertions. | v27 Runtime managed library check reported `E0432` for the obsolete root import; current module re-exports the mock backend. Rustfmt and scoped diff checks pass; no post-repair Cargo test ran. | implemented_pending_validation |
| Runtime08c Vampire animation regression fixture | Return the owned `AnimationParameterSet` actually stored by the state-machine player instead of claiming the old `BTreeMap` type. The two gameplay assertions still use its read-only map lookup via `Deref`; cloning the set retains the shared `Arc` until the caller finishes. | v27 Runtime managed library check reported `E0308`; `AnimationParameterSet` exposes `get` through `Deref` and `Default`. Rustfmt and scoped diff checks pass; no post-repair Cargo test ran. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/script/vm/backend/backend_registry/qualified_lookup_tests.rs` | `8C35E5039D6075264EBB6B2022A938F5C2A65A143E3A1DA52BB884CF7302BEF3` |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs` | `ED2BDCCE6AAC005131363B8D74D1101EFFCAA40D9A9ACE040708181AF32F59DB` |

## Managed gate

The v27 Runtime test-profile check reached Rust but ended before test execution
with 57 compiler errors across 24 owner files. These two errors were repaired
after that source snapshot. Most remaining errors reference other already
modified or untracked shared paths; do not treat this local repair as a green
package check, a VM/animation performance gain, or a reason to bypass managed
validation. Recheck these two lower contracts with the next owner-attributed
Runtime/Editor batch after the active Editor package returns.
