---
related_code:
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_advances.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py
  - docs/zircon_runtime/structure/module-convention.md
  - tests/acceptance/runtime-architecture-current-progress.md
implementation_files:
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_advances.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py
  - tests/acceptance/runtime-architecture-current-progress.md
doc_type: milestone-detail
status: runtime_15_naming_boundary_current_debt_clear_static_audit_passed_cargo_deferred
---

# Runtime 15 命名边界当前债务清零记录

日期：2026-07-11

## 审计起点

新鲜完整 Runtime 结构审计覆盖 37 个顶层分组，所有非空 `risks` 分组均为零；当时
`module_convention_gate` 唯一剩余的是 5 个命名位置：四处 UI 测试夹具文本包含
`editor`，一处 graphics 内联测试名包含 `legacy`。这些位置都不属于生产接口、运行时
分支或兼容层。

## 实现

- `text_prewarm.rs` 中两处 `**editor base.zui**` 测试夹具改为等长的
  `**sample base.zui**`。
- `geometry.rs` 中 `editor base.zui` 与相应前缀长度改为等长的 `sample` 文本，保持
  caret/offset 测试语义不变。
- `sdf_advances.rs` 的内联测试名从 `legacy` 改为 `prior`；被测实现和断言不变。
- 没有新增兼容路径、白名单或 shim，也没有修改这三处测试所覆盖的生产行为。共享工作树
  中同文件的其他并发改动不属于本切片声明范围。

## 验证

- `runtime_naming_boundary_audit`：`gate_status=classified`，`editor_debt=[]`、
  `editor_unclassified=[]`、`legacy_debt=[]`、`legacy_unclassified=[]`。
- 重新组合的 `module_convention_gate`：`m1_gate_status=classified-and-clear`，
  `migration_debt_count=0`、render/non-render 债务均为 0、`risks=[]`。
- 三个源文件 `rustfmt --check` 通过。
- 独立质量复审先发现全局验收文档仍把五处位置描述为当前阻塞；该段已改为明确的历史
  起点，并紧邻记录当前清零结果与七个外部失败边界。修正后复审返回
  `QUALITY APPROVED`。
- 当前全部可见磁盘均低于仓库 50 GiB Cargo 启动阈值，且存在其他会话的 Cargo/rustc
  通道，因此没有启动本切片的 Cargo 行为回归，也没有终止外部进程。

## 状态裁决

本切片状态为
`runtime_15_naming_boundary_current_debt_clear_static_audit_passed_cargo_deferred`。
命名边界与模块约定的当前迁移债务已经静态清零，但 Runtime 15 整体不标记完成：最近
完整 standalone 结构族仍为 1297/1304，7 个 Render/UI production/test budget、
workload owner 与 deferred-lighting dispatch 失败继续由对应 owner 处理。
