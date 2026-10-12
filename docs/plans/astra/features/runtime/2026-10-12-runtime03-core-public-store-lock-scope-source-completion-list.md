# Runtime 03 Core public-store lock-scope Source completion

**Status:** The bounded Source repair is complete. Native/type-check, performance, product, Jenkins, and whole-milestone acceptance remain pending.

## Source scope

- [x] Convert the key and construct the replacement `Arc<Value>` before locking the values map; retire the replaced value after the inner map guard drops.
- [x] Preserve the public API, owner type, existing `if_changed` decision, and publication barrier.
- [x] Add one ordinary reentrant-key regression using `try_lock`. Its name exists in Source; it was not run. The exact postimages are documented in the [Source repair record](../../../optimize/zircon_runtime/03/2026-10-12-core-public-store-lock-scope-source-repair-record.md) and [actual apply receipt](../../../../../.codex/outbox/runtime03-core-store-retirement-and-key-conversion-repair-v1/actual-canonical-apply-v1.json).

## Acceptance still pending

- [ ] Rust type-check and Native execution. No case was executed by this adoption; Weak-reference behavior is not evidence of mutex acquisition order.
- [ ] The original 15-command validation contract: 14 original producers plus two Runtime 03 matrix producers, 16 total. Conditional selection floors are command #2: 64 unique ordinary cases; #15: 193; #9: 12 ignored-library matches. Floors are not execution counts.
- [ ] Full performance samples, product acceptance, Jenkins acceptance, and the overall milestone.

The prior Native07 recipe remains frozen and refused. A future run requires a new coordinator and Source identity. The user-authorized Jenkins handoff is being prepared and has not been submitted to a queue.
