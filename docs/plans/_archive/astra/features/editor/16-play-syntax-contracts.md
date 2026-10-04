status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md

# Editor Play and syntax contracts

## Current source and repair

The animation session used a Rust 2024 let-chain in a Rust 2021 crate. The
selection rebind now uses equivalent nested conditions. The Play input fixture
stored an ABI event containing a raw payload pointer inside a shared test
mutex, which violates the gateway's `Send + Sync` contract. It now records the
all event fields, including payload address and length, in a value snapshot
that preserves the original ABI event equality assertion without dereferencing
or retaining the payload pointer. The fixture also
implements the three required operation methods as explicit unavailable
capabilities.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M33 | Repair Rust 2021 syntax and Play gateway test Send/Sync/operation contracts | implemented_pending_validation | Scoped rustfmt and diff checks passed; combined Editor batch pending |

The animation control flow preserves selection rebind behavior, and the
test fixture leaves the runtime event ABI unchanged.
