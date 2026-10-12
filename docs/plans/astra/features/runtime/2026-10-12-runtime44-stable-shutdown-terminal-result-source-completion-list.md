# Runtime 44 stable shutdown terminal-result Source completion

**Status:** The bounded Source repair is complete. Native/type-check, performance, product, Jenkins, and whole-milestone acceptance remain pending.

## Source scope

- [x] Preserve completed shutdown success or failure across repeated no-active calls; retain initial-idle success, timeout retry, fresh-generation reset, and independent worker join/unload behavior.
- [x] Add four ordinary lifecycle regression names for repeated failure, repeated success, fresh-generation reset, and timeout retry. The names exist in Source; they were not run.
- [x] Apply the exact two postimages recorded in the [Source repair record](../../../optimize/zircon_runtime/44/2026-10-12-stable-shutdown-terminal-result-source-repair-record.md) and [actual apply receipt](../../../../../.codex/outbox/runtime44-stable-shutdown-terminal-result-repair-v1/actual-canonical-apply-v1.json).

## Acceptance still pending

- [ ] Rust type-check and Native execution. No case was executed by this adoption; the independent tests use an isolated controller and do not invoke the global public wrapper.
- [ ] The original 15-command validation contract: 14 original producers plus two Runtime 03 matrix producers, 16 total. Conditional selection floors are command #2: 64 unique ordinary cases; #15: 193; #9: 12 ignored-library matches. Floors are not execution counts.
- [ ] Full performance samples, product acceptance, Jenkins acceptance, and the overall milestone.

The prior Native07 recipe remains frozen and refused. A future run requires a new coordinator and Source identity. The user-authorized Jenkins handoff is being prepared and has not been submitted to a queue. The narrow Source repair does not close the separate durability, ownership, byte-admission, hard-cap, worker-panic, or original hardware/performance limits.
