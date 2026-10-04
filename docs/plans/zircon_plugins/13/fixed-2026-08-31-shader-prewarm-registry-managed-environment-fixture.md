---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: shader-prewarm-registry-managed-environment-fixture
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/build/zircon_build.py
tests:
  - tools/tests/test_zircon_build_shader_permutation_registry_contract.py
resolved_at: 2026-08-31
---

# Plugins13 shader prewarm registry managed environment fixture

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：Zircon build adjacent regression sweep
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns build tooling tests for plugin shader projection.

## 失败现象与复现证据

The adjacent Zircon build sweep passes 179 tests with one skip but errors in
two shader permutation registry contract cases. Both reach
`managed_cargo_environment` with the fixture root
`target/prewarm-registry-contract-test`, which is correctly rejected outside
approved Windows build roots before the mocked subprocess can run.

## 最低共享层根因

The two registry-focused tests predate mandatory managed Cargo environment
routing. Unlike adjacent prewarm tests, they neither isolate that dependency
nor accept and verify the `env` argument passed to `subprocess.run`.

## 架构修复验收

- Mock only `managed_cargo_environment` in the registry contract fixtures.
- Assert the production code requests the shader-prewarm target and cache root.
- Assert the exact returned environment reaches the mocked subprocess.
- Preserve registry validation ordering and acceptance-projection assertions.
- Pass the focused registry test and the complete Zircon build test sweep.

## 禁止临时方案

- Do not weaken managed-root policy or use a repository-local Cargo target.
- Do not run product Cargo from the test.
- Do not remove environment propagation assertions from the subprocess mock.

## 修复结果与回传

- 根因：The registry fixtures invoked the mandatory managed Cargo environment with a repository-local target and did not model or verify the environment passed to the subprocess boundary.
- 架构修复：The focused fixtures mock only managed_cargo_environment, assert the requested shader-prewarm target and cache roots, and require the exact managed environment object to reach subprocess.run without weakening production root policy.
- 验证：Fresh 2026-09-01 evidence: python -B -m unittest tools.tests.test_zircon_build_shader_permutation_registry_contract -v passed 4/4; python -B -m unittest discover -s tools/tests -p test_zircon_build*.py -v passed 181/181 with one unavailable-executable platform skip; no product Cargo process started.
- 回传：Returned to Plugins13 after source commit 3ecca7a8d and fresh focused plus full mocked build-tool sweep; managed-root policy remains unchanged.
