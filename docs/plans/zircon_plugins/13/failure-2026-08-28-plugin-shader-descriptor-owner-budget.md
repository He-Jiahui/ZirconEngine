---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: plugin-shader-descriptor-owner-budget
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_build_plugin_shader_descriptors.py
  - tools/zircon_build_plugin_shader_descriptor_support.py
tests:
  - tools/tests/test_zircon_build_plugin_shader_descriptor_owner_boundaries.py
---

# Plugins13 plugin shader descriptor owner budget

## Failure evidence

The dedicated plugin shader descriptor owner reached 209 lines while its
structure guard requires at most 150. Public descriptor collection, private
TOML row validation, shader source path validation, hashing, and ordered
deduplication had accumulated in one file. Raising the budget would preserve
the mixed owner and weaken the existing module boundary.

## Repair contract

- Keep every public function imported by `zircon_build.py` in
  `zircon_build_plugin_shader_descriptors.py`.
- Move only private normalization, path validation, hashing, descriptor row,
  and ordered-deduplication helpers into one named support leaf.
- Preserve package and top-level import modes used by `zircon_build.py`.
- Retain shader permutation, geometry source, shading model, source-path, and
  content-hash behavior.
- Keep the public owner at or below 150 lines and the private leaf at or below
  120 lines.

## Forward repair

The exact-four candidate keeps the public owner at 146 lines and the support
leaf at 84 lines. Its focused owner/behavior suite passes 2/2, all three Python
files compile, and both package import and top-level script import modes load
the public API. The adjacent shader permutation registry suite passes its two
registry validation cases; its two prewarm cases stop earlier at their existing
repository-local target fixture because that target is outside the approved
Windows build roots. No Cargo process starts in those two cases.

The canonical failure remains open until isolated validation and a
coordinator-managed atomic commit establish that these results do not consume
concurrent `zircon_build.py` worktree changes.
