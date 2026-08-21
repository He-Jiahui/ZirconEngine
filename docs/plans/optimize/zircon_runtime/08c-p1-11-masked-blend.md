# Runtime08C P1-11 Per-Bone Masked Base Blend Plan

- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-11
- Owner: `optimize-runtime08c-masked-blend-r1-01a00797-20260820`
- Scope: compiled graph base-pose weight propagation and pose composition

## Change

1. Lock the disjoint-mask failure with a direct pose-composition regression.
2. Preserve authored positive finite base weights through graph sampling.
3. Normalize contributors independently for each bone after applying its dense
   mask, with an input-order-independent canonical quaternion hemisphere.
4. Cover invalid weights and input-order determinism.
5. Validate this slice in the shared Runtime08C/45/48/49 twelve-group managed
   batch instead of starting a standalone Cargo process.

## Acceptance

- A 0.5 full-body base plus a 0.5 arm-only base leaves the leg at full strength
  and blends the arm 50/50.
- Zero, negative, NaN, and infinite weights do not contribute.
- Reordering equivalent positive base inputs preserves the output pose.
- Equivalent `q` and `-q` rotation inputs converge to the same numeric output
  regardless of contributor order.
- The combined validator reports eight logical tasks, twelve Cargo groups, and seven
  independent 21-pair nearest-rank performance gates.
