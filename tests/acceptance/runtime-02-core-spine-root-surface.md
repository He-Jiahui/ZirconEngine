---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/framework
  - zircon_runtime/src/core/manager
  - zircon_runtime/src/core/math
  - zircon_runtime/src/core/resource
  - zircon_runtime/src/core/runtime
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_app/src/entry
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
output_records:
  - docs/plans/zircon_runtime/runtime/02/2026-07-11-runtime02-current-cargo-baseline.md
status: in_progress
---

# Runtime 02 Core Spine and Root Surface Acceptance

Date: 2026-07-11

## Scope

This acceptance record covers Runtime 02's five-part internal core spine, the
crate-root public surface, the generated-code boundary, and the downstream
application regression gate. It does not take ownership of Render pipeline or
Runtime UI text behavior merely because those tests are reached by the broad
`core::` filter.

## Required invariants

1. `zircon_runtime/src/core/` contains only `framework/`, `manager/`, `math/`,
   `resource/`, `runtime/`, and `mod.rs` at its root.
2. Former root fragments have one current owner under that spine; no retired
   compatibility modules, aliases, or bridge paths remain.
3. `zircon_runtime/src/lib.rs` exposes deliberate owner facades and does not
   recreate the retired graphics alias block.
4. Generated Rust files require the first-line marker
   `// @generated <generator> - do not edit by hand` and remain leaf
   bindings, DTOs, or tables rather than behavior owners.
5. Runtime absorption guards and downstream `zircon_app` tests remain green.
6. Runtime 02 is not accepted until all mandatory Cargo gates are green on
   current source.

## Current inventory

- Core root entries: five owner directories plus `mod.rs`; no loose legacy
  source fragments remain.
- Generated-code filter: 27 tests.
- Broad core filter: 657 tests in the fresh managed-lane baseline.
- Downstream application suite: 136 tests, including one documented ignored
  dynamic-runtime capture that requires the ZR VM/runtime-library environment.

## Tooling

- Windows Rust: `rustc 1.94.1 (e408947bf 2026-03-25)`.
- Windows Cargo: `cargo 1.94.1 (29ea6fb6a 2026-03-24)`.
- New authoritative Cargo reruns use coordinator-managed target lanes under
  `D:/targets/zircon-engine/lanes/`; historical target paths are retained only
  as dated evidence and are not reused as current acceptance lanes.

## Results

| Gate | Result | Evidence |
|---|---|---|
| Runtime 02 static structure audit | passed | Core root has the exact five-owner shape plus `mod.rs`; the existing audit reports no missing items or risks. |
| `generated` Runtime lib-test filter | passed, 27/27 | Current default-feature Runtime test binary; 7,456 tests filtered out, 102.23s. |
| `core::` Runtime lib-test filter | not green, 643/655 | Twelve failures remain: seven Render/post-process/IBL/pipeline cases and five Runtime UI text/layout/extract cases. Runtime 02 spine/root/generated owners did not fail. |
| `zircon_app` package tests | passed | Fresh package build exited 0; exact app lib-test rerun passed 135, failed 0, ignored 1 in 40.24s. |
| Managed-lane `core::` rerun | not green, 647/657 | Fresh default-feature build completed in 14m22s; five Render and five Runtime UI failures remain, with no Runtime 02 spine/root/generated owner failure. |
| Managed-lane `runtime_absorption` | not green, 1555/1631 | Seventy-six current source/archive-routing assertions remain, concentrated in priority review/structure guard ownership while the Runtime 15 route-convergence worktree is active. |
| Priority `structure_convention` managed snapshot | passed, 1303/1303 | Fresh managed-lane binary; 6,189 tests filtered out, 239.49s. |
| Priority `structure_convention` post-fix rerun | not green, 1299/1303 | Four active UI/Text line-budget and owner-anchor failures; no modified review guard exceeded budget. |
| Priority `code_review_findings` current source | passed, 80/80 | Standalone full direct-review harness after numbered-record ownership repair; package-level rebuild remains pending. |
| Remaining Runtime absorption/editor/plugin/full-runtime gates | pending | Required before Runtime 02 completion. |

## Failure ownership and boundary cases

- The five Render failures are inside active Render/post-process/IBL/pipeline
  ownership. Runtime 02 does not patch those paths.
- The five UI failures are inside active Runtime UI text/layout/extract
  ownership. Runtime 02 does not add a test-only compatibility path around
  their current contracts.
- The aggregate `runtime_absorption` module cannot be compiled as an isolated
  standalone crate because it deliberately imports production crate modules.
  Its acceptance evidence must therefore come from the real
  `zircon_runtime` lib-test target.
- The one ignored application test is not counted as a failure; it is an
  environment-dependent capture explicitly marked ignored by the suite.

## Acceptance decision

Not accepted yet. The owned core-spine, root-surface, generated-boundary,
priority review-guard, and application evidence is green, but the required
broad Runtime gates are still red in externally owned active paths. The
current structure filter also has four active UI/Text failures. The remaining
package gates have not all completed. Runtime 02 and the complete runtime
architecture goal remain `in_progress`.
