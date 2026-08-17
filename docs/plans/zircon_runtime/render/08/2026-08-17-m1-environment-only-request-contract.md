---
record_kind: milestone
status: completed
created_at: 2026-08-17
plan: docs/plans/zircon_runtime/render/08-material-shader-permutation.md
milestone: M1
---

Plan: docs/plans/zircon_runtime/render/08-material-shader-permutation.md
Milestone: M1
Status: completed
Files: ["zircon_runtime/src/graphics/shader/template/tests/environment_only_pbr.rs", "docs/plans/zircon_runtime/render/08/2026-08-17-environment-only-forward-retains-generic-provider-bindings-return.md"]

# M1 environment-only Standard-PBR request contract

## Scope Delivered

- 原失败用例不再把自定义 `user_surface` 请求误当作 Standard-PBR 特化；测试通过 `standard_material_surface_source_for_features` 构造真实 surface，并检查最终 assembly。
- 最终 WGSL 继续执行解析验证；相邻自定义 surface 合同保留通用 provider closure，两个语义不再混淆。
- Render08 failure 已回传 Tooling10，真实根因记录为测试请求构造错误，不宣称生产 assembler 缺陷或生产热路径修改。

## Fresh Testing Evidence

- Windows 托管验收：`Dry run: off`，原始特化 1/1、自定义 surface 相邻合同 1/1、provider guards 13/13；完整 Tooling10 矩阵共 126 个测试实例通过。
- 专用最终 WGSL 长度不超过通用版本的 50%，排除 5 个无关绑定和 14 个不可达 provider API，同时保留全局 IBL 所需入口。

## Review

- 独立复审：Critical 0，Important 0；`rustfmt --check` 与 scoped `git diff --check` 通过。
