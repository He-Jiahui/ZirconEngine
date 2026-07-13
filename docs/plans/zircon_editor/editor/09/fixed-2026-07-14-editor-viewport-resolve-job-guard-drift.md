---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: editor-viewport-resolve-job-guard-drift
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/render/01
related_code:
  - zircon_editor/src/ui/retained_host/viewport/
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::host::render_framework_boundary::editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings -- --exact --test-threads=1
resolved_at: 2026-07-14
---


# Render01：Editor viewport RenderFramework resolve-job 结构守卫漂移

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-13 | Editor09 M1 完整门的 viewport RenderFramework boundary exact 为 0/1。生产仍由 `ViewportState::lazy(core)` + `EditorJobSystem` typed ticket 懒解析，但 resolver 已提取到 `render_framework_resolve_job.rs`；旧守卫继续在 `viewport_state.rs` 查找 `resolve_render_framework(&core)` 字符串。已交接 Render01 更新当前唯一 owner 守卫，不恢复内联旧 helper。 |
| `FIXED / 已回传` | 2026-07-14 | 守卫已跟随提取后的 typed `RenderFrameworkResolveJob` owner，验证 Context jobs 提交、取消检查与唯一 Runtime resolve 路径。独立 harness 直接编译当前原始测试源，Editor03/Render01 合并结果 6 passed / 0 failed；未重新内联 resolver，也未恢复 thread/JoinHandle/raw-wgpu 路径。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1 完整 Windows lib-test acceptance 的 render-framework boundary 聚类
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：该门禁验证 Editor viewport 到 Runtime RenderFramework 的唯一边界，归 Render01；Editor09 只消费完整门。

## 失败现象与复现证据

fully-qualified exact
`editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings` 为 0/1，panic 为
`editor viewport controller should lazily resolve RenderFramework from core`。当前源码仍满足：

- `new.rs` 使用 `ViewportState::lazy(core)`；
- `viewport_state.rs` 持有 `JobTicket<Arc<dyn RenderFramework>>`、`JobCategory::Misc` 并提交
  `RenderFrameworkResolveJob::new(core)`；
- startup assembly 调用 `viewport.bind_jobs(editor_jobs.clone())`。

失败仅因静态守卫还要求 `viewport_state.rs` 包含已提取的 `resolve_render_framework(&core)` 旧字符串。
完整门日志：`.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。

## 最低共享层根因

RenderFramework lazy resolve 已按模块边界提取为 typed `RenderFrameworkResolveJob`，但 Render01 的 source
shape guard 没有与提取原子更新。当前证据指向守卫漂移，不支持把 resolver 重新内联到 state owner。

## 架构修复验收

- exact guard 验证 `viewport_state` 提交 typed resolve job、resolve job 从 `CoreHandle` 取得唯一
  `RenderFramework`、startup 显式绑定 EditorJobSystem。
- 保留无私有 thread/JoinHandle、无 editor raw wgpu、无第二 RenderFramework owner 的反向断言。
- viewport focused tests 与 Editor full gate 向上复验。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止恢复内联 `resolve_render_framework` 只为字符串断言，禁止新增第二 resolver 或重新引入 raw wgpu preview path。

## 修复结果与回传

- 根因：RenderFramework lazy resolution had been correctly extracted from ViewportState into the typed RenderFrameworkResolveJob, while the Render01 source-shape guard still searched viewport_state.rs for the retired inline resolve_render_framework(&core) call.
- 架构修复：The guard now verifies ViewportState submits RenderFrameworkResolveJob through the EditorJobSystem ticket/category contract and verifies the extracted job checks cancellation before calling resolve_render_framework on its CoreHandle. Existing no-bare-thread, no-JoinHandle and no-raw-wgpu checks remain; the resolver was not re-inlined.
- 验证：rustfmt --check and scoped diff check passed. A standalone rustc --test harness directly included the current original render_framework_boundary/mod.rs; all three RenderFramework boundary tests passed, including editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings. Combined Editor03/Render01 source-guard run: 6 passed, 0 failed, 9.72s; log .codex/tmp/editor03-render01-guard-standalone-20260714.log.
- 回传：Render01 viewport structure guard now follows the current typed resolve-job owner without restoring the retired inline helper.
