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

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：plugin shader contribution descriptor discovery
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：the Plugins13 build owner also owns its descriptor discovery module boundary.
- 当前阶段：`resolving_failure`

## 失败现象与复现证据

The dedicated plugin shader descriptor owner reached 209 lines while its
structure guard requires at most 150. Public descriptor collection, private
TOML row validation, shader source path validation, hashing, and ordered
deduplication had accumulated in one file. Raising the budget would preserve
the mixed owner and weaken the existing module boundary.

## 最低共享层根因

The public discovery facade also owned six private parsing and source-resource
helpers. That mixed API routing with validation implementation and forced every
new descriptor rule into the public owner.

## 架构修复验收

- Keep every public function imported by `zircon_build.py` in
  `zircon_build_plugin_shader_descriptors.py`.
- Move only private normalization, path validation, hashing, descriptor row,
  and ordered-deduplication helpers into one named support leaf.
- Preserve package and top-level import modes used by `zircon_build.py`.
- Retain shader permutation, geometry source, shading model, source-path, and
  content-hash behavior.
- Keep the public owner at or below 150 lines and the private leaf at or below
  120 lines.

## 禁止临时方案

- Do not raise or remove the owner budget.
- Do not move public functions consumed by `zircon_build.py` into the support leaf.
- Do not change descriptor ordering, source validation, content hashing, or
  package/top-level import behavior to make the structural test pass.

## 修复结果与回传

The exact-four candidate keeps the public owner at 146 lines and the support
leaf at 84 lines. Its focused owner/behavior suite passes 3/3, all three Python
files compile, and both package import and top-level script import modes load
the public API. The adjacent shader permutation registry suite passes its two
registry validation cases; its two prewarm cases stop earlier at their existing
repository-local target fixture because that target is outside the approved
Windows build roots. No Cargo process starts in those two cases. Coordinator
isolated validation passed the same 3/3 suite and atomically committed exact
four paths as `ab258198c2c282cbe45d7323b6c384074353fdce`, without consuming
concurrent `zircon_build.py` worktree changes.

Open state: `source_fixed_validated_committed / failure_return_pending`.
