---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: zircon-build-hub-output-owner-budget
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_build_hub.py
  - tools/zircon_build_hub_outputs.py
tests:
  - tools/tests/test_zircon_build_hub_owner_boundaries.py
  - tools/tests/test_zircon_build_hub_managed_roots.py
---

# Plugins13 Zircon build Hub output owner budget

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：Hub/Tauri build tooling ownership sweep
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns export/build tooling and its structural boundaries.

## 失败现象与复现证据

The non-Cargo tooling owner sweep passes 31/32 checks. Its only failure is
`test_hub_tauri_build_lives_in_hub_owner`: `zircon_build_hub.py` is 198 lines
against the enforced 180-line owner budget.

## 最低共享层根因

Tauri command execution and Cargo environment setup share one module with an
independent output-staging responsibility: executable discovery, installer
tree projection, sidecar copying, and platform artifact naming.

## 架构修复验收

- Keep Hub build orchestration, Tauri CLI execution, and Cargo environment in
  `zircon_build_hub.py`.
- Move output discovery and staging to `zircon_build_hub_outputs.py`.
- Preserve existing facade imports for callers while naming the direct owner.
- Preserve managed-root rejection, dry-run, executable, installer, and sidecar
  semantics.
- Pass focused owner and managed-root behavior suites without product Cargo.

## 禁止临时方案

- Do not raise the production owner budget to hide mixed responsibilities.
- Do not weaken managed-root assertions or change artifact locations.
- Do not duplicate output staging between the orchestration and leaf modules.

## 修复结果与回传

Hub build execution remains in a 98-line orchestration owner. Executable,
installer, and sidecar staging now live in a 114-line output owner, with the
shared bundle target defined once and facade imports preserved. Focused owner
and managed-root behavior pass 7/7; changed Python files compile and the scoped
diff gate is clean. A broader Zircon build sweep passed 179 tests with one skip
and retained two unrelated shader-prewarm managed-root fixture errors. The
exact-five coordinator finalizer must reproduce the Hub gate without foreign
worktree inputs.

Open state: `source_validated / failure_return_pending`.
