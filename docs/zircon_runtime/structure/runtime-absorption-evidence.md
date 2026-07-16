---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/current_source_fixture.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/evidence_ownership.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-11-stable-evidence-owner-hard-cutover.md
tests:
  - tools/tests/test_runtime_absorption_current_source_fixture.py
  - runtime_15_structure_guards_use_durable_evidence_not_session_notes
  - runtime_architecture_implementation_output_is_tracked_plan_evidence
implementation_files:
  - zircon_runtime/src/tests/runtime_absorption/current_source_fixture.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/evidence_ownership.rs
---

# Runtime absorption evidence ownership

`zircon_runtime` 的 compile-time absorption guards 只能依赖 Git 跟踪的生产源码、模块文档、编号计划产出或归档产出。`.codex/sessions` 是会话临时状态，不是干净 checkout 可重建的测试 fixture，禁止通过 `include_str!` 或其他文件读取重新引入。

Runtime architecture implementation evidence 的唯一共享编译期 owner 是 `current_source_fixture.rs`。它直接读取 tracked Runtime 15 code-structure output archive；已经通过 `runtime_numbered_archive_sources()` 聚合编号归档的 plan-status guards 继续使用该聚合 owner，不重复挂载同一 evidence source。

`runtime_15_structure_guards_use_durable_evidence_not_session_notes` 从 `tests/runtime_absorption` 根递归扫描全部 Rust guard，而不是只扫描 `structure_convention` 子树。静态 Python regression 同时锁定共享 owner、15 个普通直接 consumer、7 个必须向 `concat!` 提供字面量的直接 `include_str!` consumer、5 个编号归档聚合 consumer，以及全树零 `.codex/sessions` 路径。

该切换只改变测试证据所有权，不修改 Render05 ShadowAtlas sampler、WGPU binding、渲染行为或 guard 的断言锚点。
