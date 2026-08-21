# Runtime08C P1-17 Borrowed Deferred IK Set Plan

- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-17
- Owner: `optimize-runtime08c-ik-deferred-set-r1-01a00797-20260820`
- Scope: deferred IK command admission membership and queue ordering

## Change

1. Change the neutral animation-manager contract to borrow the caller-owned
   `BTreeSet<EntityId>` used by clip-event admission.
2. Remove the frame-path `BTreeSet -> Vec` materialization before draining IK
   commands.
3. Use ordered-set membership in both the core fallback and first-party plugin
   manager while preserving queue order and replacement-epoch behavior.
4. Add a 1,024-command behavior regression plus a 21-pair alternating release
   gate against the former materialized-Vec linear membership path.
5. Run the gate inside the next post-Main Runtime aggregate validation rather
   than starting a competing Cargo process.

## Acceptance

- Deferred commands remain queued in their original relative order; admitted
  commands retain their original relative order.
- The production tick passes the existing admission set by reference and does
  not allocate an intermediate entity vector.
- Both manager implementations expose one identical borrowed-set contract.
- The release gate emits exactly 21 alternating pairs, raw nanosecond arrays,
  and nearest-rank P95 for 4,096 commands and 2,048 deferred entities.
- Borrowed-set P95 is at most 25% of the legacy materialize-and-linear-scan P95,
  and optimized materialized entity count is zero.
- This slice does not claim prepared skeleton residency, one-model-pose-per-rig,
  command priority, or the remaining solver work from P1-17.
