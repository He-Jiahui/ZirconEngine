---
related_code:
  - zircon_plugins/navigation/editor/src/lib.rs
  - zircon_plugins/navigation/editor/src/plugin.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/mod.rs
implementation_files:
  - zircon_plugins/navigation/editor/src/lib.rs
  - zircon_plugins/navigation/editor/src/plugin.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/assets.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/components.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/operations.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/templates.rs
plan_sources:
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/test_plugin_structure_audit_registration.py
  - zircon_plugins/navigation/editor/src/tests.rs
doc_type: milestone-detail
---

# 2026-07-13 Navigation Editor registration hard cut 产出记录

## 状态与完成项目

| 项目 | 状态 | 完成日期 | 证据 |
|---|---|---|---|
| 注册所有权收敛 | `完成` | 2026-07-13 | `registration/{assets,components,operations,templates}` 已整体迁入 `plugin/registration/`，由 `plugin.rs` 直接声明和调用。 |
| 旧结构硬切换 | `完成` | 2026-07-13 | crate root 不再声明 `mod registration`；旧目录已删除，未保留 `pub use`、转发模块或路径兼容层。 |
| 插件结构门禁 | `完成` | 2026-07-13 | `audit_plugin_structure.py --json` exit 0；`m3_hard_cut_gate_status=registration-hard-cut-clean`，兼容 shim sites 从 2 降为 0。 |
| 审计回归测试 | `完成` | 2026-07-13 | registration audit 单测 10/10 通过；受影响 Rust 文件已由 `rustfmt --edition 2021` 格式化。 |
| 独立只读复审 | `完成` | 2026-07-13 | 0 Critical / 0 Important / 0 Minor，结论 `Ready`；确认无旧路径、shim、兼容 re-export、文档断链或其他会话内容损失。 |
| 当前源包级复验 | `排队` | 2026-07-13 | 当前存在 3 个其他会话的受管 Cargo 任务；遵循 Windows 验证通道与资源隔离规则，待通道空闲后执行 Navigation Editor 包验证。 |

## 架构结果

- `lib.rs` 只保留 crate 的模块边界和公开导出，不再拥有扩展注册实现。
- `plugin.rs` 是插件声明与注册生命周期的唯一所有者，子目录 `plugin/registration/` 按 assets、components、operations、templates 拆分实现。
- 此次迁移是硬切换：源码、文档机器头和 M6 产出记录均改为新路径，不维护旧路径兼容面。

## 验证记录

- `python tools/audit_plugin_structure.py --json`：exit 0；注册硬切换、descriptor single-source、runtime registration builder、capability single-source、SDK mirror、distribution boundary 与 skeleton debt 门禁均无违规。
- `python -m unittest tools.tests.test_plugin_structure_audit_registration`：10 tests，全部通过。
- 旧路径扫描：`zircon_plugins/navigation/editor/src/registration` 不存在；源码中不存在 `crate::registration` 或旧 `src/registration` 引用。
- 独立只读复审：`Ready`，0 Critical / 0 Important / 0 Minor；scoped `rustfmt --check` 与 `git diff --check` 通过。
- 包级 Cargo 复验尚未完成，因此本记录不将 Navigation M6 或 Plugins 05 总计划声明为完成。
