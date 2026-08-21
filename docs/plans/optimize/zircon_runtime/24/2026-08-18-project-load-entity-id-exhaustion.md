Plan: docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
Milestone: IDENTITY-P1-021 project-load allocator admission
Status: completed

# Runtime24 project-load entity ID exhaustion

## Delivered

- Project and scene-asset normalization reconstruct the next entity ID with checked arithmetic
  before mutating schedules, derived registries or default nodes. Project normalization also
  reserves any missing default camera/light IDs and requires the post-default cursor to remain
  allocatable before it creates either node.
- A persisted `u64::MAX` ID, or a state whose next allocatable ID would be the reserved maximum,
  returns typed `SceneError::EntityIdExhausted` instead of debug panic or release wraparound.
- Path-based project loading wraps only normalization failure in
  `SceneProjectError::ProjectNormalization`, preserving the exact document path and typed source.

## Correctness evidence

- Exhaustion behavior: `panic_or_wrap -> typed_path_rejection`.
- The regression covers persisted `u64::MAX`, `u64::MAX - 2` (old second-default overflow), and
  `u64::MAX - 3` (old reserved final cursor), preserving the exact document path in each rejection.
- Direct spawn, slot-width, generation rollover and the other allocator policies in
  IDENTITY-P1-022 and later findings remain open; this record does not claim broad identity-plan
  completion.

## Validation

- Coordinator copy `3a1de0a8d2fe47e9b809ea4b355f2c84`, input manifest
  `dee829a080d5519f05d6cbe5c8d8a96e3b931a523375251f9d1a57ccd937bd29`, ran the current source as
  receipt `4a381be1df1e43caaf4c07345c15e2a9`.
- The project-load exhaustion group passed `17/17` with zero failures in 0.27 seconds; its managed
  stage completed in 5.634 seconds.
- The same receipt passed the Runtime, Runtime Interface and Editor all-target feature checks in
  one 917.188-second stage. Its later Pester failure occurred after these gates and did not alter
  their source snapshot or result.
- This record completes only project-load allocator admission. The direct-spawn, slot-width and
  generation-rollover work named above remains open.
