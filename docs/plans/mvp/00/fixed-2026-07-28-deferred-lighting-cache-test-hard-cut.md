---
handoff_kind: fixed
status: fixed
created_at: 2026-07-28
summary_slug: deferred-lighting-cache-test-hard-cut
origin_plan: docs/plans/mvp/00-current-source-baseline-recovery.md
fixing_plan: docs/plans/zircon_runtime/render/05-lighting-shadows.md
origin_child_dir: docs/plans/mvp/00
fixing_child_dir: docs/plans/zircon_runtime/render/05
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
tests:
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter 'asset::tests::migration::project_commandlet::resolver_index'
resolved_at: 2026-07-28
---


# Render 05: deferred lighting cache test hard cut

## 来源执行者

- 来源计划：`docs/plans/mvp/00-current-source-baseline-recovery.md`
- 来源执行切片：M0.2 Runtime 04 Resolver Gate。
- 修复责任计划：`docs/plans/zircon_runtime/render/05-lighting-shadows.md`
- 交接原因：M0.2 的受管 lib-test 在执行 resolver filter 前必须编译整个 `zircon_runtime` 测试目标；最低共享错误位于 Render 05 所有的 deferred lighting pipeline 测试边界。

## 失败现象与复现证据

2026-07-28 运行：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter 'asset::tests::migration::project_commandlet::resolver_index'
```

测试目标编译在执行任何 migration 测试前失败：

```text
error[E0432]: unresolved import `super::super::create_lighting_pipeline`
 --> zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs:11:5
```

预期结果是受管测试目标完成编译并执行指定的 Runtime 04 resolver filter。

## 最低共享层根因

`create_lighting_pipeline` 已被 Render 05 的 `DeferredLightingPipelineCache` 硬切替代。生产构造路径 `DeferredSceneResources::new` 已迁移到该 cache；`runtime_pipeline.rs` 仍导入并调用已删除的自由函数，造成测试目标的 API 边界漂移。

## 架构修复验收

- runtime pipeline 测试使用 `DeferredLightingPipelineCache::new` 与 `pipeline(..., false)`，保留原来的标准 deferred/volumetric WGPU 验证语义。
- `custom_shading_model_deferred_lighting_pipeline_creates_with_project_include_source` 在 Windows coordinator lane 通过。
- 上述 Runtime 04 resolver reproduction 完成编译并执行；随后恢复 M0.2 migration resolver/index/source-boundary 与 parent batch。

## 禁止临时方案

- 不添加 alias、compatibility shim、silent fallback、duplicated truth、test-only bypass 或 call-site exception。
- 不降低测试或计划验收条件来隐藏失败。

## 修复结果与回传

- 根因：Render05 replaced the deferred lighting free-function API with DeferredLightingPipelineCache, while the owned WGPU runtime test retained the removed import and call.
- 架构修复：The test now constructs the canonical DeferredLightingPipelineCache and retrieves its standard deferred pipeline; no compatibility API, fallback, or test-only bypass was added.
- 验证：Windows managed custom_shading_model_deferred_lighting_pipeline_creates_with_project_include_source passed; the original Runtime04 resolver_index reproduction passed; rustfmt +1.94.1 check and scoped git diff check passed.
- 回传：The Render05 compile boundary is restored and M0.2 resolver validation can resume; the separate Runtime04 transaction-journal handoff remains open.
