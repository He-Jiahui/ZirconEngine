---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: runtime-absorption-missing-session-fixture
origin_plan: docs/plans/zircon_runtime/render/05-lighting-shadows.md
fixing_plan: docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
origin_child_dir: docs/plans/zircon_runtime/render/05
fixing_child_dir: docs/plans/zircon_runtime/runtime/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption
  - .codex/sessions/20260612-0847-runtime-architecture-implementation.md
tests:
  - managed cargo test -p zircon_runtime render_shadow_atlas_compare_function_matches_forward_depth_contract --locked
resolved_at: 2026-07-16
---


# Runtime01：Runtime-absorption missing session fixture blocks shadow contract test

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/05-lighting-shadows.md`
- 来源执行切片：Render18 AF-M3 → Render05 forward-depth ShadowAtlas comparison handoff, focused red contract test.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md`
- 交接原因：the failure occurs before the Plan 05 unit test is executable, in the shared runtime-absorption structure fixture layer owned by Runtime01.

## 失败现象与复现证据

Managed CPU reservation `e87fbae24fac45f8ae6dac96c52d59be` launched job
`18e6ae937cc14bb2a02cf8b6b416c669` for:

```text
cargo test -p zircon_runtime render_shadow_atlas_compare_function_matches_forward_depth_contract --locked
```

The coordinator later marked the job `orphaned` at `2026-07-16T19:41:42+08:00` with no exit code and no process IDs. Its persisted stdout is empty. Persisted stderr proves the intended test did **not** run: lib-test compilation stopped with 27 `couldn't read` errors for the missing current-source fixture `.codex/sessions/20260612-0847-runtime-architecture-implementation.md`, followed by `could not compile zircon_runtime (lib test)`.

同一根因的首轮修复复现 job `0d015c0107664e6888663d16a37a6c3a` 已于 2026-07-16 自然终止并以 exit 101 / no live PIDs released。它确认 27 个缺失路径迁移后的第二层编译阻断：7 个 split-layout consumer 把非字面量 `current_source_fixture::RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT` 传给 `concat!`，rustc 报 `expected a literal`。该 job 仍未执行 Render05 focused test，只是 Runtime01 fixture owner 的二层 RED 证据，不构成 ShadowAtlas sampler 的 red/green 结果。

修正 7 个 `concat!` consumer 为 tracked archive 的字面 `include_str!` 后，managed retry `3b954179608e4144b0aa183c8a4497f6` 完成 fresh lib-test 编译，并真实执行 1 个 focused test（8177 filtered out）。测试按 Render05 预期进入 RED：`left: GreaterEqual`、`right: LessEqual`，0 passed / 1 failed，exit 101；这证明 Runtime01 fixture 编译前置阻塞已清除，但不宣称 ShadowAtlas sampler 已修复或 green。

## 最低共享层根因

Shared `runtime_absorption` compile-time guard fixtures retain `include_str!`-style references to a removed `.codex/sessions/20260612-0847-runtime-architecture-implementation.md` record. This breaks every consumer that builds `zircon_runtime` lib tests before its own focused test can execute.

## 架构修复验收

- Runtime01 restores or migrates the canonical current-source fixture ownership so all runtime-absorption guards compile without a historical-session-path dependency.
- A reconciled managed run reports an actual test count for `render_shadow_atlas_compare_function_matches_forward_depth_contract`; neither an orphaned job nor a compile-precondition failure is red/green evidence.
- After valid red evidence, Render05 resumes the unchanged focused test-first cycle and only then changes the ShadowAtlas comparison sampler to `LessEqual`.

## 禁止临时方案

- Do not weaken, skip, cfg-gate, or delete runtime-absorption guards merely to run the Plan 05 test.
- Do not add a Plan 05 test-only bypass or alter Render18 froxel/plugin or Shader06 source paths.
- Do not infer a red or green result from the orphaned coordinator job.

## 修复结果与回传

- 根因：27 runtime_absorption guards depended on an untracked deleted .codex session note; the previous durable-evidence guard scanned only structure_convention and missed sibling guard trees.
- 架构修复：Hard-cut all 27 consumers to tracked Runtime15 archive ownership, preserve literal include_str inputs for seven concat consumers, reuse numbered archive aggregation for five status guards, and widen durable-evidence scanning to the whole runtime_absorption tree.
- 验证：Static regression 1/1; zero stale session paths; independent review 0 critical/0 important; managed retry 3b954179608e4144b0aa183c8a4497f6 compiled zircon_runtime lib tests and executed exactly one Render05 test, producing the expected sampler RED GreaterEqual versus LessEqual.
- 回传：Runtime01 removed the shared compile precondition blocker and returned valid Render05 focused RED evidence; sampler production remains unchanged for Render05 TDD.
