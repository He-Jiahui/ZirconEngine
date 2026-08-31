---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: runtime-operation-phase-terminal-matcher
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_editor/editor/03
related_code:
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix.rs
tests:
  - cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 -- pbr_matrix::render_product_environment_pbr_matrix_quantitative --exact --nocapture --test-threads=1
resolved_at: 2026-07-15
---


# Editor03: runtime operation terminal matcher does not compile

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：M1 / 独立审查收紧后的非 ignored 8x8 HDRI PBR 产品门禁
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：最低原因位于 Editor03 新增的 runtime operation interface 枚举 helper，不属于 Shader06 渲染或测试范围。

## 失败现象与复现证据

Shader06 managed job `284f626ee0fe4d37a6b56e0670896337` 在测试体前编译 `zircon_runtime_interface` 失败。`ZrRuntimeOperationPhase::is_terminal` 写成 `matches!(Self::Completed | Self::Failed, self)`，导致 E0424 和 E0369；正确的 matcher 语义应以 `self` 为 expression、terminal variants 为 pattern。

## 最低共享层根因

Editor03 operation API 的 terminal-state helper 把 `matches!` 的 expression 与 pattern 参数顺序颠倒，并把两个 enum variant 解析成位或表达式。

## 架构修复验收

- `ZrRuntimeOperationPhase::is_terminal` 对 `Completed` 和 `Failed` 返回 true，对其余阶段返回 false。
- runtime-interface focused operation tests通过，并覆盖所有枚举阶段。
- Shader06 上述非 ignored产品门禁重新编译并实际执行 1 项通过。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not disable operation API compilation or weaken Shader06 tests.

## 修复结果与回传

- 根因：Editor03 reversed the matches! expression and pattern in ZrRuntimeOperationPhase::is_terminal; operation FFI visibility was also too narrow for the dynamic API exports table.
- 架构修复：The matcher now tests self against Completed or Failed, and submit/poll/harvest operation FFI functions are visible only within crate::dynamic_api so the session re-export can feed the API table.
- 验证：Shader06 managed current-source job 77c7b62ea56343339237e62348fc1abc rebuilt and passed the non-ignored HDRI/PBR product test 1/1; runtime operation contracts cover Completed, Failed, and Queued terminal semantics.
- 回传：Runtime operation terminal matching and internal FFI export visibility compile correctly; Shader06 current-source product acceptance passes again.
