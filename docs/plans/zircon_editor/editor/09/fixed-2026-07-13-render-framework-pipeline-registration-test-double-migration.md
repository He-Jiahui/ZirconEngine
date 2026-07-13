---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: render-framework-pipeline-registration-test-double-migration
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/render/01
related_code:
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_editor/src/ui/retained_host/viewport/test_render_framework.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/fake_render_framework.rs
tests:
  - cargo test -p zircon_editor --lib --no-run --locked --jobs 1
resolved_at: 2026-07-13
---


# Render 01：RenderFramework 管线注册硬切未同步编辑器测试替身

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1.4 source authority 与只读 command when/dispatch guard 的 Windows 编译门禁
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：失败由 Render01 将 graphics-owned `register_pipeline_asset` 从中立 `RenderFramework` trait 硬切到具体 `WgpuRenderFramework` API 引起；迁移所有 trait 测试替身属于该接口切换的调用方收口，Editor09 不应恢复旧 trait 方法或在资产管理切片中修改视口渲染测试。

## 失败现象与复现证据

Text01 与 Editor12 的下层编译问题越过后，Windows 受管门禁
`cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 在 `zircon_editor` 报两个 E0046：

- `zircon_editor/src/ui/retained_host/viewport/test_render_framework.rs` 的
  `impl RenderFramework for TestRenderFramework` 缺少 `register_pipeline_asset`；
- `zircon_editor/src/ui/retained_host/viewport/tests/fake_render_framework.rs` 的
  `impl RenderFramework for FakeRenderFramework` 同样缺少该方法。

该诊断发生在共享工作树完成最终 hard cut 前的中间快照；完整日志：
`.codex/tmp/editor09-m1-4-source-authority-compile-r3-20260713.log`。

## 最低共享层根因

最低边界是 Render01 正在迁移的 `RenderFramework` 公共契约与跨 crate 测试替身未在同一原子切片收口。正确结果是中立 trait 不再接收 graphics-owned `RenderPipelineAsset`，所有测试替身同步删除旧方法；不是重新给 trait 加默认实现或兼容入口。

## 架构修复验收

- `RenderFramework` 不含 `register_pipeline_asset`，具体 WGPU runtime 以 inherent API 拥有管线资产注册。
- Editor retained-host 两个 test double 与最终 trait 一致，不保留空实现、默认 shim 或 graphics 类型导入。
- 原始 `cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 越过这两个 E0046。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止把 `register_pipeline_asset` 放回中立 trait，禁止仅给两个测试替身补 no-op 兼容实现。

## 修复结果与回传

- 根因：Render01 removed graphics-owned pipeline registration from the neutral RenderFramework trait before editor retained-host test doubles reached the same hard-cut snapshot.
- 架构修复：The final trait and both editor test doubles no longer expose register_pipeline_asset; concrete WGPU registration remains an inherent graphics API.
- 验证：cargo test -p zircon_editor --lib --no-run --locked --jobs 1 reached successful test-binary generation; artifact .codex/tmp/zircon_editor-editor09-m1-4-source-authority-r4-20260713.exe.
- 回传：Editor09 r4 no-run gate compiled past both former E0046 diagnostics and produced the current lib-test binary.
