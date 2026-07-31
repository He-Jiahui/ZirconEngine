# Frameworks06 Batch39 UI Surface Resolve Production Budget Hard-Cut

Plan: `docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`

Milestone: M1 follow-up / production Rust module size convergence

Status: implemented_validation_pending

## Scope

- Move text-style alias parsing and font-size overflow normalization out of
  `ui/surface/render/resolve.rs` into a folder-backed owner module.
- Move the resolver's inline tests into the same folder-backed boundary.
- Keep metadata traversal and resolved-style orchestration in `resolve.rs`.
- Do not expose compatibility paths or broaden the resolver API.

## Static Evidence

- Before the hard cut, canonical Rust `str::lines()` counting reported
  `zircon_runtime/src/ui/surface/render/resolve.rs` at 859 lines, violating the
  Frameworks06 production-module budget (`>= 800`).
- After the hard cut, the same guard reports `resolve.rs` at 595 lines,
  `resolve/text_style_parsing.rs` at 165 lines, and `resolve/tests.rs` at 109
  lines. The Batch39 target is GREEN with a 205-line margin.
- The repository-wide guard remains RED on the separate
  `graphics/scene/scene_renderer/ui/render.rs` owner at 835 lines.
- Rust 1.94.1 `rustfmt --check` passes for all three Batch39 Rust paths.
- `git diff --check` passes for the exact four-path manifest.

## Validation State

- Static formatting, diff checks, exact-manifest review, and fresh managed
  Cargo evidence must be recorded before this milestone can be accepted.
- The record must remain pending while the Runtime graphics feature gate is
  blocked by uncommitted Text01 compile-input ownership.
