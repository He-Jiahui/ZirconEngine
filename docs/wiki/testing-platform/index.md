---
related_code:
  - zircon_app/Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_editor/Cargo.toml
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
implementation_files:
  - .codex/skills/zircon-dev/scripts
  - .github/workflows
  - tools/check_conventions.py
plan_sources:
  - docs/plans/milestone-validation-policy.md
  - docs/plans/mvp/index.md
tests:
  - .github/workflows
  - zircon_app/src/entry/tests
  - zircon_runtime/src
doc_type: category-index
---

# 测试与平台

ZirconEngine 的测试不只是一组 `cargo test`：feature/profile 选择决定代码是否存在，Windows coordinator 决定验证是否可复现，产品验收还需要真实宿主、持久化和连续运行证据。本分区给出从快速检查到 MVP 波次的完整路径。

## 文档地图

| 页面 | 内容 |
| --- | --- |
| [Feature 与 profile 矩阵](feature-profiles.md) | client/editor/server、feature gate 和 ABI 组合 |
| [验证工作流](validation-workflow.md) | Windows validator、target policy、测试分层和 CI 对齐 |
| [诊断与性能](diagnostics-performance.md) | 日志、指标、基准和失败证据 |
| [MVP 验收](mvp-acceptance.md) | RenderableEmpty 产品闭环和退出条件 |

## 证据等级

1. **静态证据**：源码路径、manifest、frontmatter 和结构检查。
2. **编译证据**：受管 `cargo check` / `cargo build`。
3. **行为证据**：聚焦测试、宿主 session、动态库/插件加载。
4. **产品证据**：真实二进制两次连续运行、可见帧、保存并重开。

低等级证据不能替代高等级退出条件。MVP 当前状态见[功能状态](../feature-status.md)，计划仍是 `in_progress`。
