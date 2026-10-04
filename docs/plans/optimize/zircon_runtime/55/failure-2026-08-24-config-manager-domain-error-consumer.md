---
handoff_kind: failure
status: open
created_at: 2026-08-24
summary_slug: config-manager-domain-error-consumer
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/55
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/foundation/config_manager_error.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/foundation/tests.rs
tests:
  - python -B -m unittest tools.tests.test_frameworks_01_contracts_kernel_test_boundary -v
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter foundation_registry_services_do_not_retain_the_runtime_root -VerboseOutput
---

# Runtime55: migrate the ConfigManager domain-error consumer

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 contracts/kernel boundary guard convergence
- 修复责任计划：`docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md`
- 交接原因：Runtime55 owns the Foundation integration-test consumer while Frameworks01 owns the contract DAG correction.

## 失败现象与复现证据

Frameworks01 strengthened the contracts/kernel boundary guard and obtained a focused TDD RED with
exactly two violations: `foundation/config_manager.rs` and `scene/mod.rs` imported
`crate::core::CoreError`. The production contract now owns `ConfigManagerError`; the static guard is
GREEN `3/3`, and a current-source call-site audit found one stale typed assertion outside the
Frameworks01 ownership boundary.

`zircon_runtime/src/foundation/tests.rs` is a dirty blob owned by active Runtime55 session
`optimize-runtime55-foundation-empty-driver-hard-cut-r1-20260823`. Its
`foundation_registry_services_do_not_retain_the_runtime_root` test still compares
`ConfigManager::set_value` with `CoreError::RuntimeUnavailable`. After the hard cut, the left side is
`Result<(), ConfigManagerError>`, so the assertion must use the contract-owned error.

## 最低共享层根因

Runtime55 owns the Foundation module integration test and its current dirty blob. Frameworks01 owns
the contract DAG correction but must not overwrite Runtime55's active empty-driver hard-cut changes.
The consumer migration is therefore routed to Runtime55 as an exact one-assertion update.

## 架构修复验收

- Change the assertion to `ConfigManagerError::RuntimeUnavailable`, imported from
  `crate::core::framework::foundation` or referenced through that canonical path.
- Preserve Runtime55's existing Foundation descriptor assertions and current blob content.
- Do not add `From<CoreError>`, cross-type `PartialEq`, aliases, re-exports, or any compatibility
  bridge. The contract must remain independent of the runtime kernel.
- Run the focused Foundation test through the managed Windows validator once the existing foreign
  `zr_rhi_wgpu` current-source compiler blockers have converged.

## 禁止临时方案

- Do not add `From<CoreError>`, cross-type `PartialEq`, aliases, re-exports, or any compatibility bridge.
- Do not overwrite unrelated Runtime55 changes in the shared dirty Foundation test blob.
- Do not claim the managed Foundation gate passed from the static boundary guard alone.

## 修复结果与回传

Return this handoff as fixed when the current blob uses `ConfigManagerError`, the Frameworks01
boundary guard remains GREEN, and the focused managed Foundation test executes successfully. Until
then Frameworks01 records source implementation complete but does not claim the product Rust gate.

## Current state

Open: `consumer_migration_verified / focused_runtime_tests_passed / independent_review_and_formal_binding_pending`.

### 2026-09-08 Exact Consumer Verification

The current Foundation test already uses the canonical
`ConfigManagerError::RuntimeUnavailable` assertion and retains the empty-driver,
weak-runtime and manager-generation checks. No source edit was necessary.
Its exact archived `claude-runtime55-foundation-m1-20260902` bytes were transferred
to stable fixing Session `failure-roll-01a07160-runtime55` by
`17c657b819c94117b3c8e49f45fc0493`. Pre-edit snapshot 3112 preserves the source and
original failure; source snapshot 3114, request `2614c3612c49421ea5827ea93261e390`,
freezes `zircon_runtime/src/foundation/tests.rs` at
`f3d56b7cea2c12fd6e08e50c0192b3632e71f91001ea99a3a395a9119e9a5e05`.

Managed Windows job `330c856fda1444918fecc8a6a004a76a` actually executed:

```text
cargo test -p zircon_runtime --no-default-features --features target-server --locked --lib foundation::tests:: --target-dir E:/cargo-targets/zircon-engine/pool/a7c726fcdd457d7fdb7ca72b3e112234afdbb64adbf582a15546e2d1366ad3b3
```

Result: 7 passed, 0 failed, 0 ignored, 6919 filtered; test body 0.03 seconds.
This includes `foundation_registry_services_do_not_retain_the_runtime_root`,
the original failure reproduction, plus descriptor, resolver, generation and
persistence regressions. Queue/sync/check/link/test-stage seconds were
6.343/21.919/0.010/2.602/2.617. The runner reused the exact compiled test executable
after verifying all 10,956 input files and 354 dependency packages, then executed
the seven tests. Input `runtime25-project-paths-support-3095-20260908` under
`E:/cargo-targets/zircon-engine/cache/build-benchmarks` has digest
`32efa2999d8e8e6a86707ebecf8e10e53d6e68bf89287f60c2c02483b9f66870`;
its `results/runtime55-foundation-config-consumer.json` and `.log` retain the
receipt and test names. Foundation consumer and both ConfigManager contract
files match the current checkout byte-for-byte. The input's distinct support
owners, including the four Runtime25 paths, remain recorded in its provenance.

The current-checkout command
`python -B -m unittest tools.tests.test_frameworks_01_contracts_kernel_test_boundary -v`
also passed all 5 current boundary/scanner tests in 4.735 seconds. The preceding
attempt inside the Rust-only input failed to import `tools.tests` and executed no
target tests; it is not a passing boundary result. The successful static guard is
separate from the managed Rust receipt above.

Independent review and a closeout validation contract bound to the fixing Session
remain required. Operational `validate-matrix` job identity is preserved and must
not be relabeled. No `failure return`, canonical `fixed-*` or closeout is claimed.

## 2026-09-19 rolling successor formal source binding

- Successor Session `failure-roll-01a084c8-runtime55-config-consumer-r3` reclaimed the
  archived exact-path ownership through coordinator transfer fingerprint
  `2ec24bb63569af1a90d18d8fbc9be003b72b0675399d1b4acdc6d22483d03253` at baseline epoch
  `611`; no source bytes were changed during attribution.
- Formal non-Cargo source-contract ticket `d700ce708187414290417875c07a5d33` was admitted
  from request `failure-roll-01a084c8-runtime55-config-consumer-20260919-r3` and is
  currently `queued`. Its sealed source-manifest hash is
  `d9c0209acc9adc1961e5a65c9ca9ee45956738889b818b4407c1367083649388`:

  | path | SHA-256 |
  | --- | --- |
  | `docs/plans/optimize/zircon_runtime/55/failure-2026-08-24-config-manager-domain-error-consumer.md` | `e8bfb5a8ebd49874c8c5233710b9e16c20809c2a6448b46622ccdc99b3df535b` |
  | `zircon_runtime/src/core/framework/foundation/config_manager_error.rs` | `40785dc3544c803d27fcfe61d08689dd659cc606195074f0d915c41c484e05bc` |
  | `zircon_runtime/src/core/framework/foundation/config_manager.rs` | `3264017a805a72093e267cd1daf0a4ca58126f87f17ebc5afe558bfd19fda019` |
  | `zircon_runtime/src/foundation/tests.rs` | `73e8095171813fca3099eaa6bbc8a6e00c6f678e4a6465a0d127dacb4b09f6b3` |

- The ticket executes a Windows PowerShell/rustfmt source-contract parse plus the current
  Frameworks01 boundary unittest. It binds the existing managed Foundation receipt
  (`330c856fda1444918fecc8a6a004a76a`, 7/7) as supporting evidence while deferring a fresh
  current-source Cargo run, independent C/I/M review, canonical fixed return and closeout.
- Failure remains `open`; no fixed return, commit, or notification is claimed. The external
  `E:\Git\zr_vm` dirty-worktree blocker remains recorded for any fresh Cargo gate.

### Corrected source-contract ticket terminal result

- The prior ticket `d700ce708187414290417875c07a5d33` reached its command but failed because
  the immutable copy omitted tracked `tools/audits/runtime_domain_dependency_audit.py`, producing a
  Python import error before the boundary tests ran. It remains retained as coordinator input
  evidence and is not reused.
- Corrected request `failure-roll-01a084c8-runtime55-config-consumer-20260919-r4` admitted
  ticket `8e19099878384a46aa3b8035aeb20acb`, sealed manifest
  `b12f36202b87d18c6e9a8166908bf04540e05285d9e4165562341cad9da8e2b3`. The immutable Windows
  command completed `passed` at `2026-09-19T05:02:20.500669Z` (exit code 0), emitting
`RUNTIME55_CONFIG_MANAGER_DOMAIN_ERROR_SOURCE_CONTRACT_AND_BOUNDARY_PASS`; the boundary
unittest ran 3 tests and all passed. This is a source-contract result only: fresh managed
Foundation Cargo validation, independent C/I/M review, fixed return and closeout remain open.

### 2026-09-20 independent source review r3

Reviewer session: `review-runtime55-config-consumer-r3`, parent
`failure-roll-01a084c8-runtime55-config-consumer-r3`. The review covered the three
manifest paths at the current baseline and did not edit or absorb the surrounding
Runtime55 dirty test blob.

Result: **Critical=0 / Important=0 / Moderate=0**.

- `ConfigManager::set_value` and `flush` return the canonical
  `core::framework::foundation::ConfigManagerError`; the domain error enum owns the
  `RuntimeUnavailable`, persistence, and timeout variants without a `CoreError`
  conversion or compatibility alias.
- `foundation_registry_services_do_not_retain_the_runtime_root` compares the
  post-drop `set_value` result to that same canonical `RuntimeUnavailable` variant.
  The resolver/weak-runtime assertions remain intact, so the consumer verifies both
  lifetime behavior and the migrated error contract.
- Scoped `git diff --check` passed and the current source hashes remain aligned with
  the static ticket. `rustfmt --check` reports import/order formatting differences in
  the preexisting broader `foundation/tests.rs` dirty blob; that owner-owned drift was
  not rewritten as part of this one-consumer review.

Current reviewed hashes:

```text
zircon_runtime/src/core/framework/foundation/config_manager_error.rs
  40785dc3544c803d27fcfe61d08689dd659cc606195074f0d915c41c484e05bc
zircon_runtime/src/core/framework/foundation/config_manager.rs
  3264017a805a72093e267cd1daf0a4ca58126f87f17ebc5afe558bfd19fda019
zircon_runtime/src/foundation/tests.rs
  73e8095171813fca3099eaa6bbc8a6e00c6f678e4a6465a0d127dacb4b09f6b3
```

Fresh current-source Foundation Cargo, upward acceptance, canonical `fixed-*`
return, and closeout remain pending. The existing 7/7 managed receipt is retained
as supporting evidence only and is not promoted to the current immutable snapshot.
