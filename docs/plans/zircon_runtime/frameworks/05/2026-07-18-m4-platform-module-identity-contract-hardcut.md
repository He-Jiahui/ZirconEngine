---
related_code:
  - zircon_runtime/src/core/framework/platform/module_identity.rs
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/platform/mod.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
implementation_files:
  - zircon_runtime/src/core/framework/platform/module_identity.rs
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/platform/mod.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_platform_module_identity_has_one_neutral_contract_owner
  - tools.tests.test_frameworks_05_layer_direction
  - python tools/runtime_domain_dependency_audit.py
doc_type: milestone-detail
---

# Frameworks05 M4 Platform Module Identity Contract Hard Cut

Plan: `docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
Milestone: M4 module identity prerequisite
Status: implemented_static_and_review_passed_cargo_blocked
Date: 2026-07-18

## Delivered

| Slice | Status | Evidence |
|---|---|---|
| Neutral identity owner | implemented | `PLATFORM_MODULE_NAME` has one declaration in `core/framework/platform/module_identity.rs`. |
| Runtime consumers | implemented | Platform assembly, Input, Graphics, builtin registration tests and Runtime absorption tests consume the neutral contract owner. |
| App consumers | preserved facade | App profile/bootstrap tests keep the established `zircon_runtime::platform` public path; the facade now re-exports directly from the neutral contract owner. |
| Retired owner removal | implemented | `platform::module` no longer declares the identity; the concrete implementation root is not an owner or forwarding source. |
| Public projection | structural | `platform` facade and Runtime prelude preserve their established public paths through one curated re-export; no alias crate, shim, fallback, duplicate constant, or implementation-root forwarding export survives. |
| TDD guard | focused GREEN | The focused unique-owner guard was observed RED before the owner existed and then passed after the hard cut. |
| Full static suite | passed | The atomic Foundation successor preserves this hard cut: current-source Frameworks05 layer direction is 28/28 GREEN; production audit is 2,401 references / 74 edges with `graphics→platform=0` and `input→platform=0`. |
| Formatting | passed | Canonical Rust 1.94.1 rustfmt check passes for every changed Rust path. |
| Independent review | passed | First review found 0 Critical / 2 Important / 1 Minor; second review closed those items but found 0/1/0 in neutral grouped-import classification. After replacing the regex with brace-depth top-level use-tree splitting and adding explicit deny/allow self-tests, the third independent review is Critical 0 / Important 0 / Minor 0. |
| Managed Cargo | blocked | Acceptance requires the coordinator-owned immutable full-compile-input validation copy; shared-tree Cargo evidence is not accepted. |

## Architecture Decision

A runtime module name used in another domain's `ModuleDependencySpec` is contract vocabulary, not
implementation detail. Keeping `PLATFORM_MODULE_NAME` in `platform::module` forced Input and Graphics
to depend on the concrete Platform implementation root and would recreate cross-crate edges when
`zr_platform`, `zr_input`, and `zr_graphics` are extracted.

The canonical owner is therefore `core/framework/platform`, beside the already-neutral
`RuntimeTargetMode`. Concrete platform assembly consumes the same constant as every other caller.
The old implementation-root declaration is physically removed in the same change. The public
`zircon_runtime::platform` projection remains because Frameworks01 explicitly preserves established
facade paths during internal crate extraction; it re-exports the neutral owner directly and is not a
migration bridge.

The production dependency audit is interpreted at the targeted-edge level because the shared tree
contains concurrent foreign Runtime work. The Platform slice originally observed 2,395 references /
76 edges; that is a historical slice snapshot. The atomic Foundation successor's current-source snapshot is
2,401 references / 74 edges, while the acceptance claim preserved here is the absence of both
module-identity edges.

## Remaining M4 Work

This slice closes only the platform module-identity edges. Other module identities, the remaining
manager boundary work, whole-workspace validation, and Frameworks01 physical crate extraction remain
open. It does not promote Frameworks05 M4 or the parent plan to completed.
