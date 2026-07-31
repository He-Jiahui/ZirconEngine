# Frameworks06 Batch40 Screen-Space UI Record Production Budget Hard-Cut

Plan: `docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`

Milestone: M1 follow-up / production Rust module size convergence

Status: implemented_validation_pending

## Scope

- Move `ScreenSpaceUiRenderer` GPU pass submission and empty-frame attachment
  handling from `scene_renderer/ui/render.rs` into the folder-backed
  `render/record.rs` owner.
- Keep screen-space UI batch planning and geometry preparation in `render.rs`.
- Preserve the active empty-frame attachment operation correction while moving
  it; do not restore a compatibility entry point or re-export.
- Hard-cut the Runtime15 structure guard from the retired parent-method anchor
  to the folder-backed `render/record.rs` owner.
- Retarget the empty-frame source regression from the retired parent source to
  `render/record.rs`; no parent compatibility copy remains.

## Static Evidence

- Before the hard cut, canonical Rust `str::lines()` counting reported
  `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs` at 835 lines,
  violating the Frameworks06 production-module budget (`>= 800`).
- After the hard cut, the same guard reports `render.rs` at 706 lines and
  `render/record.rs` at 135 lines. Both production owners are GREEN.
- The canonical Runtime15 global production scan excludes `tests.rs`,
  `*_tests.rs`, and paths under `tests/`; applying those same rules reports no
  remaining production Rust file at or above 800 lines.
- Rust 1.94.1 `rustfmt --check` passes for all four Batch40 Rust paths.
- `git diff --check` passes for the exact five-path manifest.

## Validation State

- Static exact-manifest review and fresh managed Cargo evidence are required
  before this milestone can be accepted.
- The record remains pending while the Runtime graphics feature gate depends on
  the active Text01 compile-input closeout.

## Review State

- Snapshot 880 review returned C0/I1/M0 because `render/tests.rs` still read the
  retired parent source for the empty-frame helper assertion.
- Batch40 r3 hard-cuts that regression to `render/record.rs`; fresh exact-five
  review is pending and snapshot 880 is not acceptance evidence.
