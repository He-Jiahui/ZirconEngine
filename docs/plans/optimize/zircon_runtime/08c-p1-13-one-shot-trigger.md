# Runtime08C P1-13 One-Shot State-Machine Trigger Plan

- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-13
- Owner: `optimize-runtime08c-one-shot-trigger-r1-01a00797-20260820`
- Scope: deterministic consumption of selected state-machine Trigger parameters

## Change

1. Lock compiled transition selection so only Trigger conditions belonging to
   the selected transition are reported for consumption.
2. Compile Trigger parameter names beside each dense transition program rather
   than rescanning authoring conditions during the frame.
3. Carry selected Trigger names through ordinary and interruption arbitration
   only after the transition actually begins.
4. Commit active state and Trigger removal in the same player-component
   mutation after clip-event admission; deferred entities retain both changes.
5. Run the focused behavior group in the next serialized Runtime08C managed
   batch instead of starting a standalone Cargo process.

## Acceptance

- A Trigger remains present while its selected transition is rejected by the
  exit-time gate.
- The Trigger is removed on the frame that the ordinary transition begins and
  remains absent on subsequent frames.
- An interruption consumes its Trigger only after policy and exit-time
  arbitration select the candidate and its source pose is available.
- Unselected Trigger parameters and all non-Trigger parameter values remain
  unchanged, including a consumed name overwritten with a later non-Trigger
  value before commit.
- Deferred clip-event admission filters the combined active-state/Trigger
  update, so retry observes the original Trigger.
- A failed ordinary or interruption transition pose sample commits neither
  active state nor Trigger removal; an existing crossfade remains retryable.
- The public compatibility-facing manager evaluator remains pure; consumption
  belongs only to the production scene transaction.
- This slice does not claim the remaining single `AnimationMachineInstance`
  owner, parameter generations, or rollback journal work from P1-13.
