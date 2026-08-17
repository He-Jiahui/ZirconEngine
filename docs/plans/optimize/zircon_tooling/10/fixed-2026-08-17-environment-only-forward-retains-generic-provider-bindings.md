---
handoff_kind: fixed
status: fixed
created_at: 2026-08-17
summary_slug: environment-only-forward-retains-generic-provider-bindings
origin_plan: docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
fixing_plan: docs/plans/zircon_runtime/render/08-material-shader-permutation.md
origin_child_dir: docs/plans/optimize/zircon_tooling/10
fixing_child_dir: docs/plans/zircon_runtime/render/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/tests/environment_only_pbr.rs
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -LibTests -SkipBuild -TestFilter environment_only_forward_specialization_excludes_unreachable_environment_api
resolved_at: 2026-08-17
---


# Render 08: environment-only Forward retains generic provider bindings

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md`
- 来源执行切片：runtime lib-test API drift convergence shader contract gate
- 修复责任计划：`docs/plans/zircon_runtime/render/08-material-shader-permutation.md`
- 原始交接判断：当时将最低共享原因定位到 material shader template include selection 与 Forward pass specialization，因此交给 Render 08；后续真实 Standard-PBR request 复现证明该判断不成立。

## 原始失败现象与复现证据

协调器产物的直接行为复现显示，`environment_only_forward_specialization_excludes_unreachable_environment_api` 失败：带 `ENVIRONMENT_ONLY_PBR` 的 Forward assembly 仍包含通用环境 provider 的 `@group(1) @binding(16)`。该观察保留为 failure 生命周期的原始证据；它来自 custom `user_surface` 请求，不能证明 Standard-PBR specialization 失败。

## 原始根因假设（已证伪）

failure 提交时曾假设 module registry/include selection 与 pass specialization 未形成唯一 environment-only closure。真实 Standard-PBR request 的最终 assembly 证明生产 closure 正确；通用 provider 来自 custom surface 的既定合同，因此该生产侧根因假设已证伪。

## 原始验收假设与纠正

- `ENVIRONMENT_ONLY_PBR` Forward assembly 必须只保留专用环境 closure，排除 `@group(1) @binding(16)` 及所有不可达通用 provider API。
- 专用 normalized normal/view contract、零向量 early-out 与 generic PBR assembly 均不得退化。
- 原始 specialization 复现与 environment provider guard 组必须通过。原先要求的 WGPU pipeline validation 针对已证伪的生产缺陷，不再作为本次测试请求纠正的验收门；本记录不声明执行了 WGPU pipeline validation。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 不得仅删除断言、对 assembly 做字符串后处理或保留重复函数/绑定后依赖 dead-code elimination。

## 修复结果与回传

- 根因：The failing test combined ENVIRONMENT_ONLY_PBR with material_template_request custom user_surface input. Custom environment-only surfaces contractually retain the generic provider closure, so binding 16 was expected; the test did not construct the real Standard-PBR specialization request.
- 架构修复：Construct the environment-only Standard-PBR surface through standard_material_surface_source_for_features, assemble a MaterialShaderTemplateRequest from that source and entry point, assert the final assembly closure, and validate the resulting WGSL. The provider guard now exercises the same final assembly path.
- 验证：Managed Windows validation with Dry run: off and [OK] Cargo test: original specialization 1, custom-surface neighbor 1, provider guards 13；完整 Tooling 矩阵跨 10 个过滤器发现 126 项、执行并通过 125 项、忽略 1 项。最终 WGSL 通过 Naga parse/validate；专用版本长度不超过通用版本的 50%，排除 5 个绑定和 15 个不可达 provider API。
- 回传：Closed as a test-request contract correction, not a production assembler defect. Real Standard-PBR environment-only assembly is specialized and valid; custom surfaces continue to retain the generic closure by design.
