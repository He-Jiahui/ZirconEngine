---
title: Editor workbench floating window test contract feature boundary review
date: 2026-08-22
module: zircon_editor/src/ui/workbench/floating_window.rs
priority: MVP-P2 build and public surface
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate development automation test guards
---

# Goal

Remove the test-only floating-window design-parity contract from default editor product compilation
and public API while preserving unit and integration-contract coverage. Do not confuse this static
test schema with the real workbench floating-window runtime model, layout, routing or native host.

## Reviewed source

- file: `zircon_editor/src/ui/workbench/floating_window.rs`
- Rust files: 1/1
- lines: 147
- bytes: 4,814
- SHA256: `82b9a5d860b706043904464b7c3291eb78ae019a6a9b0756da9ccd670f28e9d3`
- owning commit: `3c8cc61ee72723d8bafe8ed99ae57a8d6e2fbd2a`

The file and all direct symbol call sites were read. `FloatingWindow`, its five enums, three static
design contracts and two fixed `.zui` resource strings are referenced only by unit and the
feature-gated external integration-contract test. Product floating-window state uses separate
workbench layout/model and retained/native-host owners.

## Result

### No product runtime hotspot

The constructors allocate one fixed resource string, and contract lookup is a three-arm constant
match. Neither executes in a product path. Caching, indexing or algorithm changes here would not
improve editor frame time.

### P2: a parallel test schema remains in the default product API

`ui/workbench/mod.rs` unconditionally compiles the file and re-exports every test contract type.
This keeps 147 lines of serde/API code and duplicate floating-window semantics in the default
library even though the product does not consume them. It creates drift risk with the real layout,
retained presentation and native window owners.

The existing `integration-contracts` feature is the correct boundary. Gate both the private module
declaration and its public re-export with `cfg(any(test, feature = "integration-contracts"))`.
Default product code must not retain a fallback re-export.

M1 now gates both sites. Static source accounting excludes this 1-file, 147-line/4,814-byte module
and its two fixed resource strings from the default configuration. Final object-code savings remain
unclaimed until managed artifact measurement.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Test/TextLayoutTestCommon.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Test/TextLayoutTest_LazyGeneration.cpp`

Unreal places shared Slate test data under `Private/Test` and encloses both common fixtures and test
implementations in `WITH_DEV_AUTOMATION_TESTS`. The transferable rule is explicit build ownership:
a parity/test declaration does not belong to the normal Slate runtime surface merely because it is
small.

## Target architecture

1. Compile/re-export the parity types only for unit tests or `integration-contracts`.
2. Keep product floating-window semantics in the real workbench layout/model, retained presentation,
   pointer routing and native host owners.
3. If a product typed contract becomes necessary, define it from those owners and migrate tests to
   it; do not ungate this parallel schema.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| default-feature floating test-contract module | absent |
| default-feature test-contract exports | absent |
| unit test availability | preserved |
| integration-contract design parity | preserved with required feature |
| product callers | 0 |
| default release artifact size | no regression; record delta |

Run managed Windows default lib, unit and integration-contract matrices with targets on D/E/F.
Record compile wall time and artifact size on one fingerprint. WPR, power and RenderDoc are not
relevant to this cold build/public-surface cut.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Record source/caller/default-feature baseline. | completed source inventory |
| M1 | Gate module and re-export; add static boundary contract. | default off, both test modes on |
| M2 | Run managed default/unit/integration Cargo matrix. | all targeted gates pass |
| M3 | Record artifact/compile delta and remove stale public docs if any. | quantified current-source evidence |

## Validation state

- Source and caller review: passed, 1/1 file.
- Unreal automation-test ownership evidence: recorded.
- M1 source implementation: complete. The shared RED-to-GREEN test-support boundary contract is
  3/3; targeted `rustfmt` and source accounting pass.
- Managed Cargo and artifact-size measurement: pending while shared Cargo lanes are active.

Keep this file in `pending.md` until M0-M3 pass. Static exclusion is not package acceptance.
