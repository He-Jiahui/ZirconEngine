---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
---

# Platform and asset test wiring

## Repairs

Clock-domain, ambient-occlusion, resource-record and session-archive tests import
their current owner types explicitly. Surface-lease tests call the existing
display-topology fixture instead of resolving the sibling module with that name.
glTF texture-variant tests import the texture conversion function and image types
needed by their real import fixture.

Archive reader test blockers use `schedule`, which returns the completion handle
that their existing `wait` calls require. Release-before-wait ordering is retained.

Concurrent glTF decoder fixes were re-read and preserved. This batch does not
modify decoder behavior or restore retired public interfaces.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M14 | Seven platform, clock, scene and glTF test import repairs | implemented_pending_validation | Derived from the prior 129-error diagnostic; edited after M1-M12 submission |

Existing behavioral assertions remain active. Current-content test and performance
acceptance requires a subsequent combined batch; the running M1-M12 diagnostic
must not be assumed to contain these later edits.
