# Runtime08C Per-Bone Masked Base Blend Record

- Date: 2026-08-20
- Owner: `optimize-runtime08c-masked-blend-r1-01a00797-20260820`
- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-11
- Execution plan: `docs/plans/optimize/zircon_runtime/08c-p1-11-masked-blend.md`
- Status: implementation and regression definition complete; combined managed validation pending

## Problem

Compiled graph evaluation divided every base clip by one global weight sum
before applying dense target masks. With two 0.5 clips where the second targets
only an arm, an unrelated leg received only the first clip's 0.5 contribution
and was incorrectly attenuated.

## Change

- Graph sampling now retains each authored positive finite base weight.
- Pose composition sums only contributors whose dense or legacy mask targets
  the current bone, then normalizes that bone's translation, scale, and
  canonically aligned rotation by its own contributor total. Quaternion signs
  use their largest absolute component, so equivalent `q`/`-q` inputs converge
  to one representation independently of input order.
- A bone with no valid contributor retains the first pose's transform as the
  existing topology fallback.
- Negative, zero, NaN, and infinite weights are excluded from base composition.

## Correctness and Performance Boundary

The repair changes an incorrect result, so it does not compare its added mask
checks against the faster wrong algorithm as a speedup. It remains allocation
neutral inside the per-bone loop: the existing pose vector is reused and no
per-bone contributor vector is built. The serialized batch carries seven
separate 21-pair release gates, including the Runtime08C event candidate heap,
so the eventual milestone and WeCom receipt will include measured P50/P95 data
without inventing a timing claim for this correctness slice.

## Acceptance

- `masked_base_blend_normalizes_weights_per_bone` locks the disjoint-mask
  oracle that failed under global normalization.
- `base_blend_ignores_non_positive_and_non_finite_weights` locks invalid input
  handling.
- `overlapping_base_blend_is_deterministic_across_input_order` locks the
  equivalent-input order oracle.
- `base_blend_equivalent_quaternion_signs_use_one_canonical_result` locks numerical
  quaternion sign determinism across reversed input order.
- The current Runtime08C/45/48/49 validator runs eight tasks in twelve Cargo
  groups. Validator SHA-256:
  `A2C1864BDCA19026FD02493EC066031AF95CE6A050E59A608859C64FBC9E0943`.
- Exact-file Rust 1.94.1 rustfmt, PowerShell AST parse, and scoped
  `git diff --check`: passed.
- Cargo regressions and release P50/P95: pending the managed multi-task batch;
  no direct or competing Cargo process was started.

## Remaining Plan Work

This closes the concrete global-normalization defect. Compile-time per-bone
contributor plans, reference-pose fill policy, dense pose pages, scratch arena
reuse, and full graph program compilation remain part of Runtime08C M3/M4.
