# Runtime ECS data-kernel audit owner sync acceptance

## Scope

Runtime 08 audit-owner and mirror-document maintenance only. This slice does not change ECS entity, storage, observer, command, event/message, resource, or change-detection production behavior.

## Baseline problem

- `ecs_kernel_data_source_inventory.py` still counted eight route-parent test files after Runtime 08 guards moved into folder-backed children.
- The audit could not discover the real Runtime 08 Cargo pending-gate owner or the mirror-doc test owner.
- The Python audit therefore reported two missing test anchors and treated the mirror-doc aggregate guard as absent even though both tests existed.

## Invariants

- Runtime 08 owns 69 production source files and 10 current guard/test files.
- All 21 guard anchors and all 16 behavior-test anchors are discoverable from their real owners.
- The mirror-doc aggregate guard is discoverable and executable.
- Plan and module documentation reports the current 10-owner inventory while retaining historical eight-route-owner evidence as history.
- Runtime 08 remains `in_progress` until the declared entity/observer/command/messages/change_tick/ecs Cargo gates pass.

## Evidence

- `python -m unittest tools.tests.test_runtime_ecs_kernel_data_audit`: passed 1/1.
- Direct ECS data-kernel audit: source files 69/69, test files 10/10, guard anchors 21/21, behavior-test anchors 16/16, empty missing test/behavior/doc/Cargo anchors, mirror-doc guard present, and `risks = []`.
- Wrapped standalone Rust Runtime 08 mirror-doc guard: passed 1/1.
- Scoped Rust formatting and Python byte-code compilation passed; final scoped diff-health is recorded with the output row.
- Package-level Runtime 08 behavior tests were not rerun because the shared lib-test baseline remains blocked by active out-of-scope Runtime text-layout work at `ui/text/layout_engine/visual_order.rs:79` (`E0282`). This static evidence does not replace the broader behavior gate.

## Decision

The Runtime 08 audit-owner synchronization slice is accepted as static maintenance. Runtime 08 itself remains `in_progress` pending the declared entity/observer/command/messages/change_tick/ecs Cargo gates.
